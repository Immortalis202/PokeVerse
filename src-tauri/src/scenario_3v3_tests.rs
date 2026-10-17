//! Full current-engine gym encounter, with identical direct and WebSocket drivers.
//! No UI executable, private library, post-start state patching or GM stat overrides.
use super::*;
use crate::{
    combat::{tactics::TacticsCommand, TurnFlow},
    creation, rules,
};
use std::{collections::BTreeSet, time::Duration};

const LIMIT: usize = 600;
fn difference(a: &serde_json::Value, b: &serde_json::Value, path: &str) -> Option<String> {
    if a == b {
        return None;
    }
    match (a, b) {
        (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
            for key in a.keys().chain(b.keys()) {
                if let Some(diff) = difference(
                    a.get(key).unwrap_or(&serde_json::Value::Null),
                    b.get(key).unwrap_or(&serde_json::Value::Null),
                    &format!("{path}/{key}"),
                ) {
                    return Some(diff);
                }
            }
        }
        (serde_json::Value::Array(a), serde_json::Value::Array(b)) if a.len() == b.len() => {
            for (i, (a, b)) in a.iter().zip(b).enumerate() {
                if let Some(diff) = difference(a, b, &format!("{path}/{i}")) {
                    return Some(diff);
                }
            }
        }
        _ => {}
    }
    Some(format!(
        "{path}: expected {}, actual {}",
        a.to_string().chars().take(250).collect::<String>(),
        b.to_string().chars().take(250).collect::<String>()
    ))
}

fn request(sheet: &Sheet, trainer: Option<&Sheet>) -> WorkflowRequest {
    WorkflowRequest {
        operation_id: id(),
        expected_revision: sheet.revision,
        trainer_revision: trainer.map(|s| s.revision),
        rules_version: creation::VERSION,
        catalog_hash: catalog().hash,
        kind: "create".into(),
        sheet: sheet.clone(),
        move_name: String::new(),
        replacement: String::new(),
        method: "ordinary".into(),
        target_species: String::new(),
        amount: 0,
        reason: "Validated headless gym fixture".into(),
    }
}
fn fixture(owners: &[String; 4], variant: &str) -> (Campaign, Action) {
    let cat = catalog();
    let mut c = Campaign::new(
        "Headless 3v3 gym".into(),
        owners[0].clone(),
        cat.hash.clone(),
    );
    c.gm_id = owners[3].clone();
    let mut roster = vec![];
    let mut starters = vec![];
    let mut sides = BTreeMap::new();
    for (role, owner) in owners.iter().enumerate() {
        let mut t = trainer(owner);
        t.id = format!("scenario-trainer-{role}");
        t.name = format!("Trainer {role}");
        t.rank = "Standard".into();
        t.xp = 0;
        t.nature = "Hardy".into();
        let mut build = creation::build(&t, &cat);
        build.origin = "starter".into();
        build.age = "Teen".into();
        build.allocations.clear();
        t.workflow = Some(build);
        let t = creation::apply(
            None,
            None,
            &request(&t, None),
            &cat,
            Some((&c.id, &c.gm_id)),
        )
        .unwrap();
        for n in 0..if role == 3 { 6 } else { 2 } {
            let mut p = pokemon(owner, &t.id);
            p.id = format!("scenario-pokemon-{role}-{n}");
            p.name = format!("Eevee {role}/{n}");
            p.species_id = Some("0133:eevee".into());
            p.rank = "Standard".into();
            p.ability = "Run Away".into();
            p.xp = 0;
            p.nature = "Hardy".into();
            p.stats.clear();
            p.skills = ["Brawl", "Channel", "Evasion", "Clash"]
                .map(|s| (s.into(), 3))
                .into();
            p.moves = ["Tackle", "Quick Attack", "Swift", "Bite"]
                .map(String::from)
                .into();
            if variant == "trace" && role == 1 && n == 0 {
                p.name = "Trace Ralts".into();
                p.species_id = Some("0280:ralts".into());
                p.ability = "Trace".into();
                p.moves = vec!["Growl".into(), "Confusion".into()];
            }
            if variant == "transfer" && role == 1 && n == 0 {
                p.name = "Receiver Passimian".into();
                p.species_id = Some("0766:passimian".into());
                p.ability = "Receiver".into();
                p.moves = vec!["Tackle".into()];
            }
            if variant == "transfer" && role == 3 && n == 0 {
                p.name = "Lingering Aroma Oinkologne".into();
                p.species_id = Some("0916:oinkologne-male-form".into());
                p.ability = "Lingering Aroma".into();
                p.moves = vec!["Tackle".into()];
            }
            let mut build = creation::build(&p, &cat);
            build.origin = "starter".into();
            build.allocations.clear();
            p.workflow = Some(build);
            let req = request(&p, Some(&t));
            let preview = creation::preview(None, Some(&t), &req, &cat);
            assert!(
                preview.errors.is_empty(),
                "fixture {}: {:?}",
                p.name,
                preview.errors
            );
            let p = creation::apply(None, Some(&t), &req, &cat, Some((&c.id, &c.gm_id))).unwrap();
            assert_eq!(p.workflow.as_ref().unwrap().status, "strict");
            if n == 0 || role == 3 && n < 3 {
                starters.push(p.id.clone());
            }
            sides.insert(
                p.id.clone(),
                if role == 3 { "Opponents" } else { "Players" }.into(),
            );
            roster.push(p.id.clone());
            c.sheets.insert(p.id.clone(), p);
        }
        c.sheets.insert(t.id.clone(), t);
    }
    (
        c,
        Action::StartTeamBattle {
            sheet_ids: roster,
            sides,
            active_sheet_ids: starters,
            slots_per_side: 3,
        },
    )
}

struct Network {
    host: room::Hosting,
    remotes: BTreeMap<usize, room::Remote>,
    stores: Vec<room::SharedStore>,
    join: JoinRequest,
    cat: Arc<Catalog>,
}
impl Network {
    async fn close(mut self) {
        let _ = self.host.host.shutdown.send(true);
        for (_, remote) in std::mem::take(&mut self.remotes) {
            remote.task.abort();
            let _ = remote.task.await;
        }
        let stores = self.stores.clone();
        drop(self);
        // Wait for the server's graceful shutdown to release SQLite connections
        // before TempDir removal (particularly important for Windows file locks).
        tokio::time::timeout(Duration::from_secs(5), async {
            while stores.iter().any(|store| Arc::strong_count(store) != 1) {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("scenario room retained database handles after shutdown");
    }
    async fn send(&self, role: usize, actor: &str, command: Command) -> Result<Campaign> {
        if role == 0 {
            return self.host.host.apply(actor, &command);
        }
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.remotes[&role]
            .sender
            .send((command, tx))
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(5), rx)
            .await
            .expect("3v3 command timed out")
            .unwrap()
    }
    async fn reconnect(&mut self, role: usize) -> Campaign {
        let remote = self.remotes.remove(&role).unwrap();
        remote.task.abort();
        let _ = remote.task.await;
        let (remote, snapshot) = tokio::time::timeout(
            Duration::from_secs(5),
            room::join(
                self.stores[role].clone(),
                self.cat.clone(),
                self.join.clone(),
                Arc::new(|_, _| {}),
            ),
        )
        .await
        .expect("3v3 reconnect timed out")
        .unwrap();
        self.remotes.insert(role, remote);
        snapshot
    }
}
impl Drop for Network {
    fn drop(&mut self) {
        for remote in self.remotes.values() {
            remote.task.abort();
        }
        let _ = self.host.host.shutdown.send(true);
        if let Some(stop) = self.host.stop.take() {
            let _ = stop.send(());
        }
    }
}
struct Driver {
    c: Campaign,
    original: Campaign,
    start: Action,
    owners: [String; 4],
    cat: Catalog,
    net: Option<Network>,
    seen: BTreeSet<String>,
    identities: BTreeMap<String, String>,
    steps: usize,
    last: Option<(usize, Command)>,
    _dirs: Vec<tempfile::TempDir>,
}
impl Driver {
    async fn new(network: bool) -> Self {
        Self::with_trace(network, false).await
    }
    async fn with_trace(network: bool, trace: bool) -> Self {
        Self::variant(network, if trace { "trace" } else { "ordinary" }).await
    }
    async fn variant(network: bool, variant: &str) -> Self {
        let dirs: Vec<_> = (0..if network { 4 } else { 0 })
            .map(|_| tempfile::tempdir().unwrap())
            .collect();
        let stores: Vec<_> = dirs
            .iter()
            .map(|d| Arc::new(Mutex::new(Store::open(d.path()).unwrap())))
            .collect();
        let owners = if network {
            std::array::from_fn(|i| stores[i].lock().unwrap().player_id.clone())
        } else {
            ["host", "player-1", "player-2", "gm"].map(String::from)
        };
        let (c, start) = fixture(&owners, variant);
        let cat = catalog();
        let net = if network {
            stores[0].lock().unwrap().put_campaign(&c).unwrap();
            let cat = Arc::new(cat.clone());
            let host = room::start_loopback(
                stores[0].clone(),
                cat.clone(),
                c.id.clone(),
                Arc::new(|_, _| {}),
            )
            .await
            .unwrap();
            let join = JoinRequest {
                address: host.info.address.replace("<host-ip>", "127.0.0.1"),
                token: host.info.token.clone(),
            };
            let mut remotes = BTreeMap::new();
            for (role, store) in stores.iter().enumerate().skip(1) {
                let (remote, _) = tokio::time::timeout(
                    Duration::from_secs(5),
                    room::join(
                        store.clone(),
                        cat.clone(),
                        join.clone(),
                        Arc::new(|_, _| {}),
                    ),
                )
                .await
                .unwrap()
                .unwrap();
                remotes.insert(role, remote);
            }
            Some(Network {
                host,
                remotes,
                stores,
                join,
                cat,
            })
        } else {
            None
        };
        Self {
            original: c.clone(),
            c,
            start,
            owners,
            cat,
            net,
            seen: BTreeSet::new(),
            identities: BTreeMap::new(),
            steps: 0,
            last: None,
            _dirs: dirs,
        }
    }
    fn b(&self) -> &Battle {
        self.c.battle.as_ref().unwrap()
    }
    fn f(&self) -> &TurnFlow {
        self.b().flow.as_ref().unwrap()
    }
    fn p(&self, id: &str) -> &Combatant {
        self.b().combatants.iter().find(|p| p.id == id).unwrap()
    }
    fn pokemon(&self, role: usize, n: usize) -> String {
        self.b()
            .combatants
            .iter()
            .find(|p| p.sheet_id == format!("scenario-pokemon-{role}-{n}"))
            .unwrap()
            .id
            .clone()
    }
    fn role(&self, id: &str) -> usize {
        self.owners
            .iter()
            .position(|o| o == &self.p(id).owner_id)
            .unwrap()
    }
    fn context(&self, role: usize, action: &Action, faces: &[u8]) -> String {
        format!("step {} role {role} revision {} round/turn {:?} pending {:?}\n{action:?}\ndice={faces:?}", self.steps,self.c.revision,self.c.battle.as_ref().map(|b|(b.round,b.flow.as_ref().map(|f|f.turn))),self.c.battle.as_ref().and_then(|b|b.flow.as_ref()).map(|f|&f.queue))
    }
    // successes=None means every die gets face; Some(n) gives n successes and failures after them.
    async fn step(&mut self, role: usize, action: Action, face: u8, successes: Option<usize>) {
        self.steps += 1;
        assert!(
            self.steps < LIMIT,
            "3v3 scenario exceeded {LIMIT} accepted steps"
        );
        let n = rules::dice_count(&self.c, &action, &self.cat)
            .unwrap_or_else(|e| panic!("{}\n{e:?}", self.context(role, &action, &[])));
        let faces = (0..n as usize)
            .map(|i| {
                if successes.is_none_or(|s| i < s) {
                    face
                } else {
                    1
                }
            })
            .collect::<Vec<_>>();
        let context = self.context(role, &action, &faces);
        let command = Command {
            id: format!("scenario-command-{}", self.steps),
            expected_revision: self.c.revision,
            action,
        };
        let (mut expected, description) = battle::transition(
            &self.c,
            &self.owners[role],
            &command.action,
            faces.clone(),
            &self.cat,
        )
        .unwrap_or_else(|e| panic!("{context}\n{e:?}"));
        expected.revision += 1;
        expected.events.push(Event {
            sequence: expected.revision,
            command_id: command.id.clone(),
            actor_id: self.owners[role].clone(),
            description,
            dice: faces.clone(),
        });
        self.c = if let Some(net) = &self.net {
            *net.host.host.test_roll.lock().unwrap() = Some((command.id.clone(), faces));
            let actual = net
                .send(role, &self.owners[role], command.clone())
                .await
                .unwrap_or_else(|e| panic!("{context}\nWebSocket: {e:?}"));
            // StartBattle allocates fresh UUIDs on each transition; match identities by sheet, not random bytes.
            let mut encoded = serde_json::to_string(&expected).unwrap();
            if matches!(command.action, Action::StartTeamBattle { .. }) {
                if let (Some(a), Some(e)) = (&actual.battle, &expected.battle) {
                    encoded = encoded.replace(&e.id, &a.id);
                    for ep in &e.combatants {
                        let ap = a
                            .combatants
                            .iter()
                            .find(|p| p.sheet_id == ep.sheet_id)
                            .unwrap();
                        encoded = encoded.replace(&ep.id, &ap.id);
                    }
                }
            }
            // Reparse typed sets after UUID remapping so set order is canonical.
            let normalized: Campaign = serde_json::from_str(&encoded).unwrap();
            if let Some(diff) = difference(
                &serde_json::to_value(normalized).unwrap(),
                &serde_json::to_value(&actual).unwrap(),
                "campaign",
            ) {
                panic!("engine/WebSocket divergence: {context}\n{diff}");
            }
            assert!(
                net.host.host.test_roll.lock().unwrap().is_none(),
                "scripted dice were not consumed: {context}"
            );
            actual
        } else {
            expected
        };
        self.last = Some((role, command));
        if let Some(b) = &self.c.battle {
            for p in &b.combatants {
                let previous = self
                    .identities
                    .entry(p.sheet_id.clone())
                    .or_insert_with(|| p.id.clone());
                assert_eq!(previous, &p.id, "combatant identity changed: {context}");
                assert!(p.actions <= 5 && p.hp >= 0 && p.hp <= p.max_hp, "{context}");
                if p.deployed {
                    self.seen.insert(p.sheet_id.clone());
                }
            }
            for side in ["Players", "Opponents"] {
                assert!(
                    b.combatants
                        .iter()
                        .filter(|p| p.deployed && p.side == side)
                        .count()
                        <= 3,
                    "{context}"
                );
            }
            assert!(
                b.automatic_validation,
                "scenario must not disable validation"
            );
        }
    }
    async fn reject(&mut self, role: usize, action: Action, contains: &str) {
        let before = serde_json::to_value(&self.c).unwrap();
        let n = rules::dice_count(&self.c, &action, &self.cat).unwrap_or(0);
        let faces = vec![6; n as usize];
        let context = self.context(role, &action, &faces);
        let result = battle::transition(
            &self.c,
            &self.owners[role],
            &action,
            faces.clone(),
            &self.cat,
        );
        let err = match result {
            Err(e) => e,
            Ok(_) => panic!("unexpected acceptance: {context}"),
        };
        assert!(
            err.message.contains(contains),
            "{context}\nwrong rejection {err:?}, expected {contains}"
        );
        if let Some(net) = &self.net {
            let cmd = Command {
                id: id(),
                expected_revision: self.c.revision,
                action,
            };
            *net.host.host.test_roll.lock().unwrap() = Some((cmd.id.clone(), faces));
            assert!(
                net.send(role, &self.owners[role], cmd).await.is_err(),
                "{context}"
            );
            *net.host.host.test_roll.lock().unwrap() = None;
            assert_eq!(
                serde_json::to_value(net.host.host.snapshot().unwrap()).unwrap(),
                before,
                "rejected command mutated host state: {context}"
            );
        }
        assert_eq!(serde_json::to_value(&self.c).unwrap(), before);
    }
    async fn reconnect(&mut self, role: usize) {
        if let Some(net) = &mut self.net {
            let c = net.reconnect(role).await;
            assert_eq!(
                serde_json::to_value(c).unwrap(),
                serde_json::to_value(&self.c).unwrap()
            );
        }
    }
    async fn retry(&self, receipt: &(usize, Command)) {
        if let Some(net) = &self.net {
            let c = net
                .send(receipt.0, &self.owners[receipt.0], receipt.1.clone())
                .await
                .unwrap();
            assert_eq!(
                serde_json::to_value(c).unwrap(),
                serde_json::to_value(&self.c).unwrap(),
                "retry duplicated command"
            );
            let mut stale = receipt.1.clone();
            stale.id = id();
            let err = net
                .send(receipt.0, &self.owners[receipt.0], stale)
                .await
                .unwrap_err();
            assert_eq!(err.code, "stale");
            let other = (receipt.0 + 1) % 4;
            let err = net
                .send(other, &self.owners[other], receipt.1.clone())
                .await
                .unwrap_err();
            assert_eq!(err.code, "ownership");
        }
    }
    async fn damage(&mut self, face: u8) {
        let q = self.f().queue[0].clone();
        let action = Action::Damage {
            combatant_id: q.actor_id.clone(),
            sequence: q.sequence,
            pool_modifier: 0,
            success_modifier: 0,
            damage_modifier: 0,
            reason: "Scenario damage".into(),
        };
        self.step(self.role(&q.actor_id), action, face, None).await;
    }
    async fn settle(&mut self, face: u8) {
        for _ in 0..40 {
            if self.f().queue.is_empty() {
                return;
            }
            let q = self.f().queue[0].clone();
            if self.f().window {
                self.step(
                    self.role(&q.target_id),
                    Action::PassReaction {
                        combatant_id: q.target_id,
                        reason: "Accept incoming scenario move".into(),
                    },
                    1,
                    None,
                )
                .await;
            } else if !q.damage_done {
                self.damage(face).await;
            } else {
                self.step(
                    self.role(&q.actor_id),
                    Action::ResolveEffect {
                        sequence: q.sequence,
                        reason: "Resolve reviewed scenario effects".into(),
                    },
                    1,
                    None,
                )
                .await;
            }
        }
        panic!("scenario action chain did not finish: {:?}", self.f());
    }
    async fn hold(&mut self) {
        let id = self.f().order[self.f().cursor].clone();
        self.step(
            self.role(&id),
            Action::PassTurn {
                combatant_id: id,
                reason: "Hold remaining scenario turn".into(),
            },
            1,
            None,
        )
        .await;
    }
    async fn round(&mut self) {
        for _ in 0..20 {
            if self.f().order.is_empty() {
                break;
            }
            self.hold().await;
        }
        assert!(self.f().order.is_empty());
        let previous = self.b().round;
        self.step(3, Action::NextRound, 1, None).await;
        assert_eq!(self.b().round, previous + 1);
        assert!(self
            .b()
            .combatants
            .iter()
            .all(|p| p.actions == 0 && p.reactions == 0));
        assert!(self.f().reacted.is_empty());
        let newcomers = self
            .b()
            .combatants
            .iter()
            .filter(|p| p.deployed && p.hp > 0 && !p.initiative_rolled)
            .map(|p| p.id.clone())
            .collect::<Vec<_>>();
        for id in newcomers {
            let role = self.role(&id);
            let face = if role == 3 {
                3 - self.p(&id).field_slot.unwrap() as u8
            } else {
                [5, 6, 4][role]
            };
            self.step(role, Action::Initiative { combatant_id: id }, face, None)
                .await;
        }
        self.step(3, Action::StartTurns, 1, None).await;
    }
    async fn switch(&mut self, out: &str, into: &str) {
        self.step(self.role(out), switch(out, into), 1, None).await;
        assert!(!self.p(out).deployed);
        assert!(self.p(into).deployed);
        assert_eq!(self.p(into).available_round, self.b().round + 1);
    }
}
fn attack(actor: &str, targets: Vec<String>, name: &str) -> Action {
    Action::AccuracyTargets {
        combatant_id: actor.into(),
        target_ids: targets,
        move_name: name.into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Headless 3v3 move".into(),
    }
}
fn react(id: &str, kind: &str, name: &str) -> Action {
    Action::React {
        combatant_id: id.into(),
        kind: kind.into(),
        move_name: name.into(),
        reason: "Headless 3v3 reaction".into(),
    }
}
fn switch(out: &str, into: &str) -> Action {
    Action::SwitchPokemon {
        outgoing_id: out.into(),
        incoming_id: into.into(),
        gm_override: false,
        reason: "Headless trainer reserve choice".into(),
    }
}

async fn scenario(network: bool) {
    let mut d = Driver::new(network).await;
    d.reject(0, d.start.clone(), "GM").await;
    d.step(3, d.start.clone(), 1, None).await;
    assert_eq!(d.b().combatants.len(), 12);
    assert_eq!(d.b().combatants.iter().filter(|p| p.deployed).count(), 6);
    assert!(d.f().tactics.enabled && d.f().special.enabled && d.f().decoys.enabled);
    let players: Vec<_> = (0..3).map(|r| d.pokemon(r, 0)).collect();
    let gyms: Vec<_> = (0..3).map(|n| d.pokemon(3, n)).collect();
    let main = players[1].clone();
    let reserve = d.pokemon(1, 1);
    d.reject(
        1,
        attack(&reserve, vec![gyms[0].clone()], "Tackle"),
        "active combatant",
    )
    .await;
    d.reject(3, attack(&gyms[0], vec![reserve.clone()], "Tackle"), "turn")
        .await;
    for (id, face) in players.iter().chain(&gyms).zip([5, 6, 4, 3, 2, 1]) {
        d.step(
            d.role(id),
            Action::Initiative {
                combatant_id: id.clone(),
            },
            face,
            None,
        )
        .await;
    }
    d.reject(0, Action::StartTurns, "GM").await;
    d.step(3, Action::StartTurns, 1, None).await;
    assert_eq!(d.f().order[d.f().cursor], main);
    d.reject(0, attack(&main, vec![gyms[0].clone()], "Tackle"), "own")
        .await;
    d.reject(1, attack(&main, vec![reserve.clone()], "Tackle"), "active")
        .await;
    let root_turn = d.f().turn;
    d.step(1, attack(&main, gyms.clone(), "Swift"), 4, Some(2))
        .await;
    let sequence = d.f().queue[0].sequence;
    d.reject(
        1,
        Action::Damage {
            combatant_id: main.clone(),
            sequence,
            pool_modifier: 0,
            success_modifier: 0,
            damage_modifier: 0,
            reason: "Premature damage".into(),
        },
        "react",
    )
    .await;
    d.reject(3, Action::NextRound, "pending").await;
    d.reject(1, switch(&main, &reserve), "pending").await;
    for (i, clash) in ["Bite", "Tackle", ""].into_iter().enumerate() {
        assert!(d.f().window);
        let defender = d.f().queue[0].target_id.clone();
        d.step(
            3,
            Action::Tactics {
                command: TacticsCommand::Reaction {
                    combatant_id: defender,
                    move_name: "Quick Attack".into(),
                    target_ids: vec![main.clone()],
                },
                reason: "Gym member counters Swift".into(),
            },
            4,
            Some(2),
        )
        .await;
        assert_eq!(d.f().queue[0].move_name, "Quick Attack");
        assert_eq!(d.f().turn, root_turn);
        if i < 2 {
            d.step(1, react(&main, "clash", clash), 6, None).await;
            assert_eq!(d.p(&main).actions, 2 + i as u32);
            assert!(
                !d.f().reacted.contains(&main),
                "Clash must not spend Evasion slot"
            );
        } else {
            d.reconnect(1).await; // Actual response window survives disconnect.
            d.step(1, react(&main, "evade", ""), 1, None).await;
            d.reject(1, react(&main, "evade", ""), "once per turn")
                .await;
            let before_damage = serde_json::to_value(d.b()).unwrap();
            d.damage(1).await;
            let receipt = d.last.clone().unwrap();
            d.retry(&receipt).await;
            let event = d.c.battle_undo.last().unwrap().event_sequence;
            d.step(
                3,
                Action::RollbackBattle {
                    event_sequence: event,
                    reason: "Undo completed reaction damage".into(),
                },
                1,
                None,
            )
            .await;
            assert_eq!(serde_json::to_value(d.b()).unwrap(), before_damage);
            d.retry(&receipt).await; // Receipt must survive rollback without reapplying damage.
            d.damage(1).await; // A new command explicitly commits the restored pending hit.
        }
    }
    assert_eq!(d.p(&main).actions, 4);
    let before: BTreeMap<_, _> = gyms.iter().map(|id| (id.clone(), d.p(id).hp)).collect();
    let mut shared = None;
    for _ in 0..10 {
        if d.f().queue.is_empty() {
            break;
        }
        if d.f().window {
            let q = &d.f().queue[0];
            let target = q.target_id.clone();
            d.step(
                3,
                Action::PassReaction {
                    combatant_id: target,
                    reason: "Take shared Swift damage".into(),
                },
                1,
                None,
            )
            .await;
            continue;
        }
        assert_eq!(d.f().queue[0].move_name, "Swift");
        d.damage(1).await;
        if let Some(q) = d.f().queue.first() {
            if let Some(prefix) = &shared {
                assert_eq!(
                    &q.shared_dice, prefix,
                    "shared damage must reuse the original roll"
                );
            } else {
                shared = Some(q.shared_dice.clone());
            }
        }
    }
    assert!(d.f().queue.is_empty());
    assert!(shared.is_some());
    for id in &gyms {
        assert_eq!(d.p(id).hp, before[id] - 1);
    }
    // Two owners deploy their reserves. Roll one replacement back, replay its receipt,
    // and accept it again under a new ID to prove neither receipts nor costs rewind.
    let out = players[0].clone();
    let into = d.pokemon(0, 1);
    d.reject(2, switch(&out, &into), "own").await;
    d.reject(0, switch(&out, &d.pokemon(2, 1)), "same trainer")
        .await;
    let saved = serde_json::to_value(d.b()).unwrap();
    d.switch(&out, &into).await;
    let receipt = d.last.clone().unwrap();
    d.retry(&receipt).await;
    let event = d.c.battle_undo.last().unwrap().event_sequence;
    d.step(
        3,
        Action::RollbackBattle {
            event_sequence: event,
            reason: "Undo reserve choice".into(),
        },
        1,
        None,
    )
    .await;
    assert_eq!(serde_json::to_value(d.b()).unwrap(), saved);
    d.retry(&receipt).await;
    d.switch(&out, &into).await;
    d.reconnect(2).await;
    let out = players[2].clone();
    let into = d.pokemon(2, 1);
    d.switch(&out, &into).await;
    assert_eq!(d.b().trainer_actions["scenario-trainer-0"], 0);
    d.reject(
        0,
        Action::Initiative {
            combatant_id: d.pokemon(0, 1),
        },
        "next round",
    )
    .await;
    // Recorded GM authorization spends real Will; no resource overrides are used.
    let will = Action::SpendWill {
        combatant_id: main.clone(),
        mode: "pain".into(),
        reason: "GM confirms the narrative pain prerequisite".into(),
    };
    d.reject(1, will.clone(), "GM must authorize").await;
    let previous_will = d.p(&main).runtime.will;
    d.step(3, will, 1, None).await;
    assert_eq!(d.p(&main).runtime.will, previous_will - 1);
    // Evasion is available again in a later turn of the SAME round. This fifth
    // action exhausts the defender even though Clash did not spend its slot.
    for _ in 0..12 {
        if d.role(&d.f().order[d.f().cursor]) == 3 {
            break;
        }
        d.hold().await;
    }
    assert_eq!(d.b().round, 1);
    assert!(d.f().turn > root_turn);
    assert!(!d.f().reacted.contains(&main));
    let gym = d.f().order[d.f().cursor].clone();
    d.step(3, attack(&gym, vec![main.clone()], "Tackle"), 4, Some(2))
        .await;
    d.step(1, react(&main, "evade", ""), 6, None).await;
    assert!(d.f().queue.is_empty());
    assert_eq!(d.p(&main).actions, 5);
    d.reject(1, react(&main, "clash", "Quick Attack"), "five actions")
        .await;
    d.round().await;
    assert!(d
        .b()
        .combatants
        .iter()
        .filter(|p| p.deployed)
        .all(|p| p.available_round <= d.b().round));
    // The defender can evade again on a new turn; then a later ordinary attack
    // faints the starter and its real owner replaces it without a GM override.
    for _ in 0..10 {
        if d.role(&d.f().order[d.f().cursor]) == 3 {
            break;
        }
        d.hold().await;
    }
    let gym = d.f().order[d.f().cursor].clone();
    d.step(3, attack(&gym, vec![main.clone()], "Tackle"), 4, Some(2))
        .await;
    d.step(1, react(&main, "evade", ""), 6, None).await;
    assert!(d.f().queue.is_empty(), "successful Evasion cancels the hit");
    assert_eq!(d.p(&main).actions, 1);
    let gym = d.f().order[d.f().cursor].clone();
    assert_eq!(d.role(&gym), 3);
    d.step(3, attack(&gym, vec![main.clone()], "Tackle"), 4, Some(2))
        .await;
    d.settle(6).await;
    assert_eq!(d.p(&main).hp, 0);
    d.switch(&main, &reserve).await;
    d.reconnect(1).await;
    d.reject(
        1,
        attack(&main, vec![gyms[2].clone()], "Tackle"),
        "active combatant",
    )
    .await;
    assert_eq!(d.p(&reserve).hp, d.p(&reserve).max_hp);
    // Finish through ordinary moves/holds/rounds. Opponents hold after the scripted
    // exchanges; every gym reserve must be sent out and defeated normally.
    for _ in 0..300 {
        let fainted = d
            .b()
            .combatants
            .iter()
            .filter(|p| p.deployed && p.side == "Opponents" && p.hp == 0)
            .map(|p| p.id.clone())
            .collect::<Vec<_>>();
        for out in fainted {
            let incoming = d
                .b()
                .combatants
                .iter()
                .find(|p| !p.deployed && p.side == "Opponents" && p.hp > 0)
                .map(|p| p.id.clone());
            if let Some(into) = incoming {
                d.switch(&out, &into).await;
            }
        }
        if d.b()
            .combatants
            .iter()
            .filter(|p| p.side == "Opponents")
            .all(|p| p.hp == 0)
        {
            break;
        }
        if d.f().order.is_empty() {
            d.round().await;
            continue;
        }
        let actor = d.f().order[d.f().cursor].clone();
        if d.role(&actor) == 3 {
            d.hold().await;
            continue;
        }
        let target = d
            .b()
            .combatants
            .iter()
            .find(|p| {
                p.deployed && p.side == "Opponents" && p.hp > 0 && p.available_round <= d.b().round
            })
            .map(|p| p.id.clone());
        let name = ["Tackle", "Swift", "Quick Attack", "Bite"]
            .into_iter()
            .find(|name| !d.p(&actor).used_moves.iter().any(|m| m == name));
        if let (Some(target), Some(name)) = (target, name) {
            d.step(d.role(&actor), attack(&actor, vec![target], name), 6, None)
                .await;
            d.settle(6).await;
        } else {
            d.hold().await;
        }
        assert!(d.b().round < 15, "scenario failed to converge");
    }
    assert!(d
        .b()
        .combatants
        .iter()
        .filter(|p| p.side == "Opponents")
        .all(|p| p.hp == 0));
    assert!(d
        .b()
        .combatants
        .iter()
        .any(|p| p.side == "Players" && p.hp > 0));
    assert_eq!(
        d.seen.len(),
        12,
        "every registered Pokemon must have entered the battle"
    );
    assert!(d.b().active, "victory does not silently end a battle");
    let final_battle = d.b().clone();
    d.reject(0, Action::EndBattle, "GM").await;
    d.step(3, Action::EndBattle, 1, None).await;
    assert_eq!(d.c.finished_battles.len(), 1);
    assert!(!d.b().active);
    for p in &final_battle.combatants {
        let sheet = &d.c.sheets[&p.sheet_id];
        let original = &d.original.sheets[&p.sheet_id];
        assert_eq!(sheet.hp, p.hp);
        assert_eq!(sheet.workflow.as_ref().unwrap().will, p.runtime.will);
        assert_eq!(sheet.conditions, p.conditions);
        assert_eq!(sheet.id, original.id);
        assert_eq!(sheet.species_id, original.species_id);
        assert_eq!(sheet.rank, original.rank);
        assert_eq!(sheet.nature, original.nature);
        assert_eq!(sheet.ability, original.ability);
        assert_eq!(
            sheet.workflow.as_ref().unwrap().training_points,
            original.workflow.as_ref().unwrap().training_points
        );
        assert_eq!(sheet.owner_id, original.owner_id);
        assert_eq!(sheet.trainer_id, original.trainer_id);
        assert_eq!(sheet.stats, original.stats);
        assert_eq!(sheet.skills, original.skills);
        assert_eq!(sheet.moves, original.moves);
        assert_eq!(
            sheet.workflow.as_ref().unwrap().allocations,
            original.workflow.as_ref().unwrap().allocations
        );
    }
    if let Some(net) = &d.net {
        let stored = net.stores[0].lock().unwrap().campaigns().unwrap().remove(0);
        assert_eq!(
            serde_json::to_value(stored).unwrap(),
            serde_json::to_value(&d.c).unwrap()
        );
        assert_eq!(net.host.host.members().unwrap().len(), 4);
    }
    d.reconnect(1).await;
    if let Some(net) = d.net.take() {
        net.close().await;
        let reopened = Store::open(d._dirs[0].path()).unwrap();
        assert_eq!(reopened.player_id, d.owners[0]);
        let persisted = reopened.campaigns().unwrap().remove(0);
        assert_eq!(
            serde_json::to_value(persisted).unwrap(),
            serde_json::to_value(&d.c).unwrap()
        );
    }
    eprintln!(
        "3v3 {}: {} accepted commands, {} rounds, 12 deployed, 6 gym defeats",
        if network { "WebSocket" } else { "engine" },
        d.steps,
        final_battle.round
    );
}

#[tokio::test]
async fn scenario_3v3_engine_complete_gym_battle() {
    tokio::time::timeout(Duration::from_secs(60), scenario(false))
        .await
        .expect("headless scenario deadline exceeded");
}
#[tokio::test]
async fn scenario_3v3_websocket_matches_engine_receipts_and_reconnect() {
    tokio::time::timeout(Duration::from_secs(120), scenario(true))
        .await
        .expect("WebSocket scenario deadline exceeded");
}

#[tokio::test]
async fn scenario_3v3_trace_websocket_owner_receipts_rollback_reconnect() {
    tokio::time::timeout(Duration::from_secs(60), async {
        let mut d = Driver::with_trace(true, true).await;
        d.step(3, d.start.clone(), 1, None).await;
        let trace = d.pokemon(1, 0);
        assert!(d.f().ability_changes.enabled);
        let e = d.f().ability_events[0].clone();
        assert_eq!(e.kind, "ability-trace");
        let action = Action::ResolveAbility {
            event_id: e.id,
            choice: "roll".into(),
            reason: "Host selects random eligible gym foe".into(),
        };
        d.reject(0, action.clone(), "own").await;
        let before = serde_json::to_value(d.b()).unwrap();
        d.step(1, action.clone(), 4, None).await;
        assert_eq!(d.p(&trace).selected_ability, "Run Away");
        assert_eq!(d.c.sheets[&d.p(&trace).sheet_id].ability, "Trace");
        let receipt = d.last.clone().unwrap();
        d.retry(&receipt).await;
        d.reconnect(1).await;
        d.reconnect(3).await;
        let event = d.c.battle_undo.last().unwrap().event_sequence;
        d.step(
            3,
            Action::RollbackBattle {
                event_sequence: event,
                reason: "Restore Trace entry choice".into(),
            },
            1,
            None,
        )
        .await;
        assert_eq!(serde_json::to_value(d.b()).unwrap(), before);
        d.retry(&receipt).await;
        d.step(1, action, 4, None).await;
        d.reconnect(1).await;
        if let Some(net) = d.net.take() {
            net.close().await;
        }
        let reopened = Store::open(d._dirs[0].path()).unwrap();
        assert_eq!(
            serde_json::to_value(reopened.campaigns().unwrap().remove(0)).unwrap(),
            serde_json::to_value(&d.c).unwrap()
        );
    })
    .await
    .expect("Trace 3v3 deadline exceeded");
}

#[tokio::test]
async fn scenario_3v3_contact_inheritance_clock_and_reconnect() {
    use crate::combat::transfers::TransferCommand;
    tokio::time::timeout(Duration::from_secs(120), async {
        let mut d = Driver::variant(true, "transfer").await;
        d.step(3, d.start.clone(), 1, None).await;
        let players: Vec<_> = (0..3).map(|r| d.pokemon(r, 0)).collect();
        let gyms: Vec<_> = (0..3).map(|n| d.pokemon(3, n)).collect();
        for (id, face) in players.iter().chain(&gyms).zip([6, 5, 4, 3, 2, 1]) {
            d.step(
                d.role(id),
                Action::Initiative {
                    combatant_id: id.clone(),
                },
                face,
                None,
            )
            .await;
        }
        d.step(3, Action::StartTurns, 1, None).await;
        d.step(
            0,
            attack(&players[0], vec![gyms[0].clone()], "Tackle"),
            4,
            Some(2),
        )
        .await;
        d.step(
            3,
            Action::PassReaction {
                combatant_id: gyms[0].clone(),
                reason: "Allow qualifying contact".into(),
            },
            1,
            None,
        )
        .await;
        let before = serde_json::to_value(d.b()).unwrap();
        d.step(0, damage_command(&d.c), 4, Some(1)).await;
        assert_eq!(d.p(&players[0]).selected_ability, "Lingering Aroma");
        let receipt = d.last.clone().unwrap();
        d.retry(&receipt).await;
        d.reconnect(3).await;
        let event = d.c.battle_undo.last().unwrap().event_sequence;
        d.step(
            3,
            Action::RollbackBattle {
                event_sequence: event,
                reason: "Review contact propagation".into(),
            },
            1,
            None,
        )
        .await;
        assert_eq!(serde_json::to_value(d.b()).unwrap(), before);
        d.retry(&receipt).await;
        d.step(0, damage_command(&d.c), 4, Some(1)).await;
        for _ in 0..4 {
            if d.f().order[d.f().cursor] == gyms[0] {
                break;
            }
            d.hold().await;
        }
        assert_eq!(d.f().order[d.f().cursor], gyms[0]);
        for _ in 0..3 {
            if d.p(&players[0]).hp == 0 {
                break;
            }
            let gym = d.f().order[d.f().cursor].clone();
            assert_eq!(d.role(&gym), 3);
            d.step(3, attack(&gym, vec![players[0].clone()], "Tackle"), 6, None)
                .await;
            d.step(
                0,
                Action::PassReaction {
                    combatant_id: players[0].clone(),
                    reason: "Take gym hit".into(),
                },
                1,
                None,
            )
            .await;
            d.step(3, damage_command(&d.c), 6, None).await;
        }
        assert_eq!(d.p(&players[0]).hp, 0);
        let event = d.f().ability_events[0].clone();
        assert_eq!(event.kind, "transfer-inherit");
        let action = Action::ResolveAbility {
            event_id: event.id,
            choice: "activate:0".into(),
            reason: "Receiver copies fainted ally's current ability".into(),
        };
        d.reject(0, action.clone(), "own").await;
        let before = serde_json::to_value(d.b()).unwrap();
        d.step(1, action.clone(), 1, None).await;
        assert_eq!(d.p(&players[1]).selected_ability, "Lingering Aroma");
        assert_eq!(d.c.ability_inheritance.copies.len(), 1);
        assert_eq!(d.c.sheets[&d.p(&players[1]).sheet_id].ability, "Receiver");
        let receipt = d.last.clone().unwrap();
        d.retry(&receipt).await;
        d.reconnect(1).await;
        let event = d.c.battle_undo.last().unwrap().event_sequence;
        d.step(
            3,
            Action::RollbackBattle {
                event_sequence: event,
                reason: "Review inherited ability provenance".into(),
            },
            1,
            None,
        )
        .await;
        assert!(d.c.ability_inheritance.copies.is_empty());
        assert_eq!(serde_json::to_value(d.b()).unwrap(), before);
        d.retry(&receipt).await;
        d.step(1, action, 1, None).await;
        d.step(3, Action::EndBattle, 1, None).await;
        let advance = Action::AbilityTransfer {
            command: TransferCommand::Advance { minutes: 1439 },
            reason: "23 game hours and 59 minutes pass".into(),
        };
        d.reject(0, advance.clone(), "GM").await;
        d.step(3, advance, 1, None).await;
        let receipt = d.last.clone().unwrap();
        d.retry(&receipt).await;
        assert_eq!(d.c.ability_inheritance.minutes, 1439);
        let mut start = d.start.clone();
        if let Action::StartTeamBattle {
            active_sheet_ids, ..
        } = &mut start
        {
            active_sheet_ids.retain(|id| id != "scenario-pokemon-0-0");
            active_sheet_ids.push("scenario-pokemon-0-1".into());
        }
        d.identities.clear();
        d.seen.clear();
        d.step(3, start, 1, None).await;
        let receiver = d.pokemon(1, 0);
        assert_eq!(d.p(&receiver).selected_ability, "Lingering Aroma");
        assert_eq!(d.f().ability_changes.originals[&receiver], "Receiver");
        d.reconnect(1).await;
        d.step(3, Action::EndBattle, 1, None).await;
        d.step(
            3,
            Action::AbilityTransfer {
                command: TransferCommand::Advance { minutes: 1 },
                reason: "24-hour copy expires".into(),
            },
            1,
            None,
        )
        .await;
        assert_eq!(d.c.ability_inheritance.minutes, 1440);
        assert!(d.c.ability_inheritance.selected.is_empty());
        d.reconnect(3).await;
        if let Some(net) = d.net.take() {
            net.close().await;
        }
        let reopened = Store::open(d._dirs[0].path()).unwrap();
        assert_eq!(
            serde_json::to_value(reopened.campaigns().unwrap().remove(0)).unwrap(),
            serde_json::to_value(&d.c).unwrap()
        );
    })
    .await
    .expect("3v3 contact/inheritance deadline exceeded");
}

fn damage_command(c: &Campaign) -> Action {
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    Action::Damage {
        combatant_id: q.actor_id.clone(),
        sequence: q.sequence,
        pool_modifier: 0,
        success_modifier: 0,
        damage_modifier: 0,
        reason: "Scenario contact/inheritance damage".into(),
    }
}
