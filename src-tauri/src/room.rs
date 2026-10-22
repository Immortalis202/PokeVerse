use crate::{battle, catalog::Catalog, model::*, store::Store};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use rusqlite::{params, OptionalExtension};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::Message as ClientMessage};

pub type Notify = Arc<dyn Fn(&str, serde_json::Value) + Send + Sync>;
pub type SharedStore = Arc<Mutex<Store>>;
pub fn lock<T>(m: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>> {
    m.lock().map_err(|_| {
        AppError::new(
            "internal",
            "State lock unavailable; restart the application",
        )
    })
}
fn hash(s: &str) -> String {
    format!("{:x}", Sha256::digest(s.as_bytes()))
}

pub struct Host {
    // No wire/API switch: deterministic rolls exist only in Rust test builds.
    #[cfg(test)]
    pub(crate) test_roll: Mutex<Option<(String, Vec<u8>)>>,
    pub store: SharedStore,
    pub catalog: Arc<Catalog>,
    pub campaign_id: String,
    pub token: String,
    pub changes: broadcast::Sender<Campaign>,
    pub notify: Notify,
    pub connected: Mutex<HashSet<String>>,
    pub shutdown: tokio::sync::watch::Sender<bool>,
}
impl Host {
    pub fn members(&self) -> Result<Vec<String>> {
        let store = lock(&self.store)?;
        let mut members = vec![store.player_id.clone()];
        let mut query = store
            .campaigns
            .prepare("SELECT player_id FROM members WHERE campaign_id=?1 ORDER BY player_id")?;
        for member in query.query_map([&self.campaign_id], |row| row.get::<_, String>(0))? {
            members.push(member?);
        }
        Ok(members)
    }
    pub fn snapshot(&self) -> Result<Campaign> {
        lock(&self.store)?
            .campaigns()?
            .into_iter()
            .find(|c| c.id == self.campaign_id)
            .ok_or_else(|| AppError::new("missing", "Campaign not found"))
    }
    pub fn apply(&self, actor: &str, cmd: &Command) -> Result<Campaign> {
        let mut store = lock(&self.store)?;
        let current = store
            .campaigns()?
            .into_iter()
            .find(|c| c.id == self.campaign_id)
            .ok_or_else(|| AppError::new("missing", "Campaign not found"))?;
        if cmd.id.is_empty() || cmd.id.len() > 100 {
            return Err(AppError::new("validation", "Invalid command ID"));
        }
        let receipt = store
            .campaigns
            .query_row(
                "SELECT actor_id FROM receipts WHERE campaign_id=?1 AND id=?2",
                params![self.campaign_id, cmd.id],
                |r| r.get::<_, String>(0),
            )
            .optional()?;
        if let Some(owner) = receipt {
            if owner != actor {
                return Err(AppError::new(
                    "ownership",
                    "Command ID belongs to another player",
                ));
            }
            return Ok(current);
        }
        if current.revision != cmd.expected_revision {
            return Err(AppError::new(
                "stale",
                "Campaign changed; review the latest state before retrying",
            ));
        }
        if let Action::AssignGm { player_id } = &cmd.action {
            if actor != current.host_id() {
                return Err(AppError::new(
                    "permission",
                    "Only the host can choose the GM",
                ));
            }
            let known = player_id == current.host_id()
                || store.campaigns.query_row(
                    "SELECT EXISTS(SELECT 1 FROM members WHERE campaign_id=?1 AND player_id=?2)",
                    params![current.id, player_id],
                    |r| r.get::<_, bool>(0),
                )?;
            if !known {
                return Err(AppError::new(
                    "validation",
                    "The GM must be the host or an authenticated room member",
                ));
            }
        }
        if current.catalog_hash != self.catalog.hash
            && !matches!(
                cmd.action,
                Action::AdoptCatalog
                    | Action::AssignGm { .. }
                    | Action::EndBattle
                    | Action::CancelDamage { .. }
                    | Action::CancelTraining { .. }
                    | Action::ApproveWorkflow { accept: false, .. }
            )
        {
            return Err(AppError::new("catalog_adoption","The appointed GM must adopt the installed catalog in Training encounters before shared changes"));
        }
        crate::combat::information::authorize(&current, actor, &cmd.action)?;
        let count = crate::rules::dice_count(&current, &cmd.action, &self.catalog)?;
        if count > crate::combat::sequencing::roll_limit(&current, &cmd.action) {
            return Err(AppError::new(
                "validation",
                "Dice exceed the validated action/ability roll limit",
            ));
        }
        let dice: Vec<u8> = (0..count)
            .map(|_| rand::rng().random_range(1..=6))
            .collect();
        #[cfg(test)]
        let dice = match lock(&self.test_roll)?.take() {
            Some((command_id, faces)) => {
                if command_id != cmd.id
                    || faces.len() != count as usize
                    || faces.iter().any(|face| !(1..=6).contains(face))
                {
                    return Err(AppError::new(
                        "test_dice",
                        "Scripted roll does not match this command",
                    ));
                }
                faces
            }
            None => dice,
        };
        let (mut next, description) =
            battle::transition(&current, actor, &cmd.action, dice.clone(), &self.catalog)?;
        next.revision = current
            .revision
            .checked_add(1)
            .ok_or_else(|| AppError::new("limit", "Campaign revision exhausted"))?;
        next.events.push(Event {
            sequence: next.revision,
            command_id: cmd.id.clone(),
            actor_id: actor.into(),
            description,
            dice,
        });
        let tx = store.campaigns.transaction()?;
        tx.execute(
            "UPDATE campaigns SET data=?2 WHERE id=?1",
            params![next.id, serde_json::to_string(&next)?],
        )?;
        tx.execute(
            "INSERT INTO receipts VALUES (?1,?2,?3)",
            params![next.id, cmd.id, actor],
        )?;
        tx.commit()?;
        let viewer = store.player_id.clone();
        drop(store);
        let _ = self.changes.send(next.clone());
        (self.notify)(
            "room-message",
            serde_json::to_value(ServerMessage::Snapshot {
                campaign: crate::privacy::for_player(&next, &viewer),
            })?,
        );
        Ok(next)
    }
    fn authenticate(&self, hello: &Hello) -> Result<String> {
        if hello.protocol != PROTOCOL
            || hello.build_version != env!("CARGO_PKG_VERSION")
            || hello.catalog_hash != self.catalog.hash
            || hello.engine != ENGINE
        {
            return Err(AppError::new(
                "version",
                "Protocol, catalog, or rules-engine mismatch; install matching builds",
            ));
        }
        if hello.token != self.token || uuid::Uuid::parse_str(&hello.player_id).is_err() {
            return Err(AppError::new(
                "authentication",
                "Invalid room token or player identity",
            ));
        }
        let store = lock(&self.store)?;
        if hello.player_id == store.player_id {
            return Err(AppError::new(
                "authentication",
                "The host identity cannot join as a remote player",
            ));
        }
        let existing = store
            .campaigns
            .query_row(
                "SELECT credential_hash FROM members WHERE campaign_id=?1 AND player_id=?2",
                params![self.campaign_id, hello.player_id],
                |r| r.get::<_, String>(0),
            )
            .optional()?;
        let credential = if let Some(expected) = existing {
            let supplied = hello.credential.as_deref().ok_or_else(|| {
                AppError::new(
                    "authentication",
                    "This identity needs its saved reconnect credential",
                )
            })?;
            if hash(supplied) != expected {
                return Err(AppError::new(
                    "authentication",
                    "Invalid reconnect credential",
                ));
            }
            supplied.into()
        } else {
            let credential = format!("{}{}", id(), id());
            store.campaigns.execute(
                "INSERT INTO members VALUES (?1,?2,?3)",
                params![self.campaign_id, hello.player_id, hash(&credential)],
            )?;
            credential
        };
        Ok(credential)
    }
}
pub struct Hosting {
    pub host: Arc<Host>,
    pub info: RoomInfo,
    pub stop: Option<oneshot::Sender<()>>,
}
pub async fn start(
    store: SharedStore,
    catalog: Arc<Catalog>,
    campaign_id: String,
    port: u16,
    notify: Notify,
) -> Result<Hosting> {
    start_bound(store, catalog, campaign_id, port, notify, "0.0.0.0").await
}
#[cfg(test)]
pub(crate) async fn start_loopback(
    store: SharedStore,
    catalog: Arc<Catalog>,
    campaign_id: String,
    notify: Notify,
) -> Result<Hosting> {
    start_bound(store, catalog, campaign_id, 0, notify, "127.0.0.1").await
}
async fn start_bound(
    store: SharedStore,
    catalog: Arc<Catalog>,
    campaign_id: String,
    port: u16,
    notify: Notify,
    bind: &str,
) -> Result<Hosting> {
    let listener = tokio::net::TcpListener::bind((bind, port)).await?;
    let port = listener.local_addr()?.port();
    let (changes, _) = broadcast::channel(64);
    let token = {
        let s = lock(&store)?;
        let existing = s
            .campaigns
            .query_row(
                "SELECT token FROM room_tokens WHERE campaign_id=?1",
                [&campaign_id],
                |r| r.get::<_, String>(0),
            )
            .optional()?;
        let token = existing.unwrap_or_else(id);
        s.campaigns.execute(
            "INSERT OR IGNORE INTO room_tokens VALUES (?1,?2)",
            params![campaign_id, token],
        )?;
        token
    };
    let (shutdown, _) = tokio::sync::watch::channel(false);
    let host = Arc::new(Host {
        #[cfg(test)]
        test_roll: Mutex::new(None),
        store,
        catalog,
        campaign_id: campaign_id.clone(),
        token: token.clone(),
        changes,
        notify,
        connected: Mutex::new(HashSet::new()),
        shutdown,
    });
    host.snapshot()?;
    let router = Router::new()
        .route("/room", get(upgrade))
        .with_state(host.clone());
    let (stop, stopped) = oneshot::channel();
    tokio::spawn(async move {
        let _ = axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = stopped.await;
            })
            .await;
    });
    Ok(Hosting {
        host,
        info: RoomInfo {
            address: format!("ws://<host-ip>:{port}/room"),
            token,
            campaign_id,
        },
        stop: Some(stop),
    })
}
async fn upgrade(State(host): State<Arc<Host>>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.max_message_size(512 * 1024)
        .on_upgrade(move |socket| serve_socket(host, socket))
}
async fn send(socket: &mut WebSocket, msg: ServerMessage) -> bool {
    socket
        .send(Message::Text(
            serde_json::to_string(&msg).unwrap_or_default().into(),
        ))
        .await
        .is_ok()
}
async fn serve_socket(host: Arc<Host>, mut socket: WebSocket) {
    let hello = match tokio::time::timeout(Duration::from_secs(10), socket.recv()).await {
        Ok(Some(Ok(Message::Text(t)))) => serde_json::from_str::<Hello>(&t),
        _ => return,
    };
    let hello = match hello {
        Ok(h) => h,
        Err(_) => {
            send(
                &mut socket,
                ServerMessage::Error {
                    command_id: None,
                    error: AppError::new("format", "Expected room handshake"),
                },
            )
            .await;
            return;
        }
    };
    let credential = match host.authenticate(&hello) {
        Ok(c) => c,
        Err(e) => {
            send(
                &mut socket,
                ServerMessage::Error {
                    command_id: None,
                    error: e,
                },
            )
            .await;
            return;
        }
    };
    let admitted = {
        let Ok(mut connected) = host.connected.lock() else {
            return;
        };
        connected.len() < 8 && connected.insert(hello.player_id.clone())
    };
    if !admitted {
        send(
            &mut socket,
            ServerMessage::Error {
                command_id: None,
                error: AppError::new("room_full", "Room full or player already connected"),
            },
        )
        .await;
        return;
    }
    // Subscribe before snapshot so no committed update can fall between the two.
    let mut changes = host.changes.subscribe();
    let mut shutdown = host.shutdown.subscribe();
    if let Ok(c) = host.snapshot() {
        if !send(
            &mut socket,
            ServerMessage::Welcome {
                campaign: crate::privacy::for_player(&c, &hello.player_id),
                credential,
            },
        )
        .await
        {
            if let Ok(mut set) = host.connected.lock() {
                set.remove(&hello.player_id);
            }
            return;
        }
    }
    let mut heartbeat = tokio::time::interval(Duration::from_secs(20));
    let mut last_seen = std::time::Instant::now();
    loop {
        tokio::select! {
            _=shutdown.changed()=>{let _=socket.send(Message::Close(None)).await;break;},
            message=socket.recv()=>match message {
                Some(Ok(Message::Text(text)))=>{
                    last_seen=std::time::Instant::now();
                    let msg=match serde_json::from_str::<Command>(&text) {
                        Ok(cmd)=>match host.apply(&hello.player_id,&cmd) {Ok(c)=>ServerMessage::Accepted{command_id:cmd.id,campaign:crate::privacy::for_player(&c,&hello.player_id)},Err(e)=>ServerMessage::Error{command_id:Some(cmd.id),error:e}},
                        Err(_)=>ServerMessage::Error{command_id:None,error:AppError::new("format","Malformed command")}
                    };
                    if !send(&mut socket,msg).await {break;}
                },
                Some(Ok(Message::Pong(_)))|Some(Ok(Message::Ping(_)))=>{last_seen=std::time::Instant::now();},
                Some(Ok(Message::Close(_)))|None|Some(Err(_))=>break,
                _=>{}
            },
            update=changes.recv()=>{
                let c=match update {Ok(c)=>Ok(c),Err(broadcast::error::RecvError::Lagged(_))=>host.snapshot(),Err(_)=>break};
                if let Ok(c)=c {if !send(&mut socket,ServerMessage::Snapshot{campaign:crate::privacy::for_player(&c,&hello.player_id)}).await {break;}}
            },
            _=heartbeat.tick()=>{
                if last_seen.elapsed()>Duration::from_secs(65) || socket.send(Message::Ping(vec![].into())).await.is_err() {break;}
            }
        }
    }
    if let Ok(mut connected) = host.connected.lock() {
        connected.remove(&hello.player_id);
    }
}

pub struct Remote {
    pub sender: mpsc::Sender<(Command, oneshot::Sender<Result<Campaign>>)>,
    pub task: tokio::task::JoinHandle<()>,
}
pub async fn join(
    store: SharedStore,
    catalog: Arc<Catalog>,
    request: JoinRequest,
    notify: Notify,
) -> Result<(Remote, Campaign)> {
    if !request.address.starts_with("ws://") || !request.address.ends_with("/room") {
        return Err(AppError::new(
            "address",
            "Use ws://host-ip:port/room on your LAN or private VPN",
        ));
    }
    let player_id = lock(&store)?.player_id.clone();
    let credentials: Vec<(String, String)> = {
        let s = lock(&store)?;
        let mut q = s
            .library
            .prepare("SELECT campaign_id,credential FROM credentials")?;
        let rows = q
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        rows
    };
    // The persistent room token identifies the campaign even if its LAN/VPN address changes.
    let previous = lock(&store)?
        .library
        .query_row(
            "SELECT value FROM metadata WHERE key=?1",
            [format!("room-token:{}", hash(&request.token))],
            |r| r.get::<_, String>(0),
        )
        .optional()?;
    let credential = previous.and_then(|cid| {
        credentials
            .iter()
            .find(|(id, _)| *id == cid)
            .map(|(_, c)| c.clone())
    });
    let mut hello = Hello {
        player_id,
        token: request.token.clone(),
        credential,
        protocol: PROTOCOL,
        build_version: env!("CARGO_PKG_VERSION").into(),
        catalog_hash: catalog.hash.clone(),
        engine: ENGINE.into(),
    };
    let (mut socket, campaign, new_credential) = connect(&request.address, &hello).await?;
    hello.credential = Some(new_credential.clone());
    {
        let s = lock(&store)?;
        s.cache_campaign(&campaign)?;
        s.library.execute("INSERT INTO credentials VALUES (?1,?2) ON CONFLICT(campaign_id) DO UPDATE SET credential=excluded.credential",params![campaign.id,new_credential])?;
        s.library.execute("INSERT INTO metadata VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",params![format!("room-token:{}",hash(&request.token)),campaign.id])?;
    }
    let (sender, mut receiver) = mpsc::channel::<(Command, oneshot::Sender<Result<Campaign>>)>(32);
    let task = tokio::spawn(async move {
        let mut pending = BTreeMap::<String, oneshot::Sender<Result<Campaign>>>::new();
        let mut heartbeat = tokio::time::interval(Duration::from_secs(20));
        let mut last_seen = std::time::Instant::now();
        loop {
            let disconnected = tokio::select! {
                request=receiver.recv()=>match request {
                    Some((cmd,response))=>{
                        let text=serde_json::to_string(&cmd).unwrap_or_default();
                        match pending.entry(cmd.id) {
                            std::collections::btree_map::Entry::Occupied(_)=>{let _=response.send(Err(AppError::new("pending","This command is already pending")));false},
                            std::collections::btree_map::Entry::Vacant(entry)=>{entry.insert(response);socket.send(ClientMessage::Text(text.into())).await.is_err()}
                        }
                    },None=>break
                },
                incoming=socket.next()=>match incoming {
                    Some(Ok(ClientMessage::Text(text)))=>{
                        last_seen=std::time::Instant::now();
                        if let Ok(message)=serde_json::from_str::<ServerMessage>(&text) {
                            if let ServerMessage::Snapshot{campaign}|ServerMessage::Accepted{campaign,..}=&message {
                                if let Err(e)=lock(&store).and_then(|s|s.cache_campaign(campaign)){crate::diagnostics::record(&e.code);}
                            }
                            match &message {
                                ServerMessage::Accepted{command_id,campaign}=>{if let Some(p)=pending.remove(command_id){let _=p.send(Ok(campaign.clone()));}},
                                ServerMessage::Error{command_id:Some(id),error}=>{if let Some(p)=pending.remove(id){let _=p.send(Err(error.clone()));}},
                                _=>{}
                            }
                            if let Ok(value)=serde_json::to_value(message){notify("room-message",value);}
                        }
                        false
                    },
                    Some(Ok(ClientMessage::Ping(data)))=>{last_seen=std::time::Instant::now();socket.send(ClientMessage::Pong(data)).await.is_err()},
                    Some(Ok(ClientMessage::Pong(_)))=>{last_seen=std::time::Instant::now();false},
                    Some(Ok(ClientMessage::Close(_)))|None|Some(Err(_))=>true,
                    _=>false
                },
                _=heartbeat.tick()=>last_seen.elapsed()>Duration::from_secs(65) || socket.send(ClientMessage::Ping(vec![].into())).await.is_err()
            };
            if disconnected {
                for (_, p) in std::mem::take(&mut pending) {
                    let _=p.send(Err(AppError::new("disconnected","Room disconnected; state retained. Retry the same command ID after reconnect.")));
                }
                notify(
                    "room-status",
                    serde_json::json!({"connected":false,"message":"Host unavailable; battle paused. Reconnecting…"}),
                );
                let mut delay = 2;
                loop {
                    tokio::time::sleep(Duration::from_secs(delay)).await;
                    match connect(&request.address, &hello).await {
                        Ok((new_socket, campaign, _)) => {
                            if let Err(e) = lock(&store).and_then(|s| s.cache_campaign(&campaign)) {
                                crate::diagnostics::record(&e.code);
                            }
                            socket = new_socket;
                            last_seen = std::time::Instant::now();
                            notify(
                                "room-message",
                                serde_json::to_value(ServerMessage::Snapshot { campaign })
                                    .unwrap_or_default(),
                            );
                            notify(
                                "room-status",
                                serde_json::json!({"connected":true,"message":"Reconnected"}),
                            );
                            break;
                        }
                        Err(e) => {
                            notify(
                                "room-status",
                                serde_json::json!({"connected":false,"message":e.message}),
                            );
                            delay = (delay * 2).min(30);
                        }
                    }
                }
            }
        }
    });
    Ok((Remote { sender, task }, campaign))
}
type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;
async fn connect(address: &str, hello: &Hello) -> Result<(Socket, Campaign, String)> {
    tokio::time::timeout(Duration::from_secs(10), async {
        let (mut socket, _) = connect_async(address)
            .await
            .map_err(|e| AppError::new("connection", e.to_string()))?;
        socket
            .send(ClientMessage::Text(serde_json::to_string(hello)?.into()))
            .await
            .map_err(|e| AppError::new("connection", e.to_string()))?;
        match socket.next().await {
            Some(Ok(ClientMessage::Text(text))) => {
                match serde_json::from_str::<ServerMessage>(&text)? {
                    ServerMessage::Welcome {
                        campaign,
                        credential,
                    } => Ok((socket, campaign, credential)),
                    ServerMessage::Error { error, .. } => Err(error),
                    _ => Err(AppError::new("format", "Unexpected handshake response")),
                }
            }
            _ => Err(AppError::new("connection", "Host closed the handshake")),
        }
    })
    .await
    .map_err(|_| AppError::new("timeout", "Host did not respond within 10 seconds"))?
}
