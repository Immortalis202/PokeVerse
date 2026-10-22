pub mod battle;
pub mod catalog;
pub mod combat;
pub mod creation;
pub mod diagnostics;
pub mod encounter;
pub mod model;
pub mod points;
pub mod privacy;
pub mod room;
pub mod rules;
pub mod store;
pub mod sync;
pub mod teams;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod workflow_tests;

use model::*;
use std::{collections::BTreeMap, sync::Arc};
use tauri::{Emitter, Manager, State};

enum Session {
    Idle,
    Host(room::Hosting),
    Remote(room::Remote),
}
#[tauri::command]
fn encounter_habitats(state: State<AppState>) -> Result<Vec<serde_json::Value>> {
    state.catalog.habitats()
}
#[tauri::command]
fn encounter_pool(
    state: State<AppState>,
    habitat: String,
    query: String,
    offset: u32,
) -> Result<serde_json::Value> {
    state.catalog.habitat_pool(&habitat, &query, offset)
}
#[tauri::command]
fn generate_encounter(
    state: State<AppState>,
    campaign_id: String,
    trainer_id: String,
    habitat: String,
    rank: String,
    size: u32,
    seed: String,
) -> Result<encounter::TrainingEncounter> {
    encounter::generate(
        &campaign_id,
        &trainer_id,
        &habitat,
        &rank,
        size,
        &seed,
        &state.catalog,
    )
}
#[tauri::command]
fn encounter_drafts(state: State<AppState>) -> Result<Vec<encounter::TrainingEncounter>> {
    let store = room::lock(&state.store)?;
    let mut stmt = store
        .library
        .prepare("SELECT data FROM encounter_drafts ORDER BY rowid DESC")?;
    let rows = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    rows.into_iter()
        .map(|r| Ok(serde_json::from_str(&r)?))
        .collect()
}
#[tauri::command]
fn save_encounter_draft(
    state: State<AppState>,
    campaign: Campaign,
    encounter: encounter::TrainingEncounter,
) -> Result<()> {
    let store = room::lock(&state.store)?;
    if campaign.gm_id != store.player_id || campaign.id != encounter.campaign_id {
        return Err(AppError::new(
            "permission",
            "Only the GM may prepare this encounter",
        ));
    }
    encounter::validate(&encounter, &state.catalog)?;
    store.library.execute("INSERT INTO encounter_drafts VALUES (?1,?2) ON CONFLICT(id) DO UPDATE SET data=excluded.data",rusqlite::params![encounter.id,serde_json::to_string(&encounter)?])?;
    Ok(())
}
#[tauri::command]
fn delete_encounter_draft(state: State<AppState>, encounter_id: String) -> Result<()> {
    room::lock(&state.store)?
        .library
        .execute("DELETE FROM encounter_drafts WHERE id=?1", [encounter_id])?;
    Ok(())
}
#[tauri::command]
fn preview_capture(
    state: State<AppState>,
    campaign: Campaign,
    encounter: encounter::TrainingEncounter,
    individual_id: String,
    trainer_id: String,
    edited: Option<WorkflowRequest>,
) -> Result<encounter::CapturePreview> {
    if campaign.gm_id != room::lock(&state.store)?.player_id {
        return Err(AppError::new(
            "permission",
            "Only the GM can preview captures",
        ));
    }
    let individual = encounter
        .individuals
        .iter()
        .find(|p| p.id == individual_id)
        .ok_or_else(|| AppError::new("missing", "Unknown individual"))?;
    let trainer = campaign
        .sheets
        .get(&trainer_id)
        .ok_or_else(|| AppError::new("missing", "Unknown trainer"))?;
    encounter::capture_preview(
        &encounter,
        individual,
        trainer,
        &campaign,
        &state.catalog,
        edited,
    )
}
pub struct AppState {
    store: room::SharedStore,
    catalog: Arc<catalog::Catalog>,
    session: tokio::sync::Mutex<Session>,
    _profile_lock: std::fs::File,
}
fn notify(app: tauri::AppHandle) -> room::Notify {
    Arc::new(move |event, value| {
        let _ = app.emit(event, value);
    })
}

#[tauri::command]
fn get_library(state: State<AppState>) -> Result<Library> {
    room::lock(&state.store)?.library()
}
#[tauri::command]
fn save_sheet(state: State<AppState>, request: SaveRequest) -> Result<Sheet> {
    let mut store = room::lock(&state.store)?;
    if store.sheet(&request.sheet.id)?.is_none() && request.sheet.workflow.is_none() {
        return Err(AppError::new(
            "workflow_required",
            "Create characters through the guided workflow",
        ));
    }
    store.save(request, &state.catalog)
}
#[tauri::command]
fn preview_workflow(state: State<AppState>, request: WorkflowRequest) -> Result<WorkflowPreview> {
    room::lock(&state.store)?.preview_workflow(&request, &state.catalog)
}
#[tauri::command]
fn preview_campaign_workflow(
    campaign: Campaign,
    request: WorkflowRequest,
    state: State<AppState>,
) -> WorkflowPreview {
    let old = campaign.sheets.get(&request.sheet.id);
    let trainer = request
        .sheet
        .trainer_id
        .as_ref()
        .and_then(|id| campaign.sheets.get(id));
    creation::preview(old, trainer, &request, &state.catalog)
}
#[tauri::command]
fn preview_battle_action(
    state: State<AppState>,
    campaign: Campaign,
    action: Action,
) -> Result<crate::combat::ActionPreview> {
    crate::combat::preview(&campaign, &action, &state.catalog)
}
#[tauri::command]
fn preview_point_transfer(
    state: State<AppState>,
    request: PointTransferRequest,
) -> Result<PointTransferResult> {
    let store = room::lock(&state.store)?;
    store.preview_point_transfer(&request, &state.catalog)
}
#[tauri::command]
fn commit_point_transfer(
    state: State<AppState>,
    request: PointTransferRequest,
) -> Result<PointTransferResult> {
    room::lock(&state.store)?.commit_point_transfer(&request, &state.catalog)
}
#[tauri::command]
fn commit_workflow(state: State<AppState>, request: WorkflowRequest) -> Result<Sheet> {
    room::lock(&state.store)?.commit_workflow(&request, &state.catalog)
}
#[tauri::command]
fn save_workflow_draft(state: State<AppState>, request: WorkflowRequest) -> Result<()> {
    room::lock(&state.store)?.save_workflow_draft(&request)
}
#[tauri::command]
fn list_workflow_drafts(state: State<AppState>) -> Result<Vec<WorkflowRequest>> {
    room::lock(&state.store)?.workflow_drafts()
}
#[tauri::command]
fn delete_workflow_draft(state: State<AppState>, operation_id: String) -> Result<()> {
    room::lock(&state.store)?
        .library
        .execute("DELETE FROM workflow_drafts WHERE id=?1", [operation_id])?;
    Ok(())
}
#[tauri::command]
fn get_catalog(state: State<AppState>) -> Result<serde_json::Value> {
    state.catalog.bootstrap()
}
#[tauri::command]
fn compendium_detail(
    state: State<AppState>,
    kind: String,
    id: String,
) -> Result<Option<catalog::CompendiumDetail>> {
    state.catalog.compendium_detail(&kind, &id)
}
#[tauri::command]
fn catalog_entry(
    state: State<AppState>,
    kind: String,
    id: String,
) -> Result<Option<serde_json::Value>> {
    state.catalog.lookup(&kind, "AppId", &id)
}
#[tauri::command]
fn search_catalog(
    state: State<AppState>,
    kind: String,
    query: String,
    type_filter: String,
    offset: u32,
) -> Result<serde_json::Value> {
    state.catalog.search(&kind, &query, &type_filter, offset)
}
#[tauri::command]
fn list_campaigns(state: State<AppState>) -> Result<Vec<Campaign>> {
    let store = room::lock(&state.store)?;
    Ok(store
        .available_campaigns()?
        .iter()
        .map(|c| privacy::for_player(c, &store.player_id))
        .collect())
}
#[tauri::command]
fn create_campaign(state: State<AppState>, name: String) -> Result<Campaign> {
    if name.trim().is_empty() || name.len() > 100 {
        return Err(AppError::new(
            "validation",
            "Enter a campaign name (up to 100 characters)",
        ));
    }
    let mut store = room::lock(&state.store)?;
    let c = Campaign::new(name, store.player_id.clone(), state.catalog.hash.clone());
    store.put_campaign(&c)?;
    Ok(c)
}
#[tauri::command]
async fn host_room(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    campaign_id: String,
    port: u16,
) -> Result<RoomInfo> {
    let mut session = state.session.lock().await;
    if !matches!(*session, Session::Idle) {
        return Err(AppError::new("session", "Leave the current room first"));
    }
    {
        let store = room::lock(&state.store)?;
        let c = store
            .campaigns()?
            .into_iter()
            .find(|c| c.id == campaign_id)
            .ok_or_else(|| AppError::new("missing", "Unknown campaign"))?;
        if c.host_id() != store.player_id
            || ![
                ENGINE,
                FORM_ENGINE,
                ITEM_ENGINE,
                EVENT_ENGINE,
                LEGACY_ENGINE,
            ]
            .contains(&c.engine.as_str())
        {
            return Err(AppError::new(
                "version",
                "Only the campaign host can host this campaign with a supported rules engine",
            ));
        }
    }
    let hosting = room::start(
        state.store.clone(),
        state.catalog.clone(),
        campaign_id,
        port,
        notify(app),
    )
    .await?;
    let info = hosting.info.clone();
    *session = Session::Host(hosting);
    Ok(info)
}
#[tauri::command]
async fn join_room(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: JoinRequest,
) -> Result<Campaign> {
    let mut session = state.session.lock().await;
    if !matches!(*session, Session::Idle) {
        return Err(AppError::new("session", "Leave the current room first"));
    }
    let (remote, c) = room::join(
        state.store.clone(),
        state.catalog.clone(),
        request,
        notify(app),
    )
    .await?;
    *session = Session::Remote(remote);
    Ok(c)
}
#[tauri::command]
async fn leave_room(state: State<'_, AppState>) -> Result<()> {
    let mut session = state.session.lock().await;
    match std::mem::replace(&mut *session, Session::Idle) {
        Session::Host(mut hosting) => {
            let _ = hosting.host.shutdown.send(true);
            if let Some(stop) = hosting.stop.take() {
                let _ = stop.send(());
            }
        }
        Session::Remote(remote) => remote.task.abort(),
        Session::Idle => {}
    }
    Ok(())
}
#[tauri::command]
async fn room_members(state: State<'_, AppState>) -> Result<Vec<String>> {
    let session = state.session.lock().await;
    match &*session {
        Session::Host(hosting) => hosting.host.members(),
        _ => Err(AppError::new(
            "permission",
            "Only the host can list GM candidates",
        )),
    }
}
#[tauri::command]
async fn room_command(state: State<'_, AppState>, command: Command) -> Result<Campaign> {
    let session = state.session.lock().await;
    match &*session {
        Session::Host(hosting) => {
            let player = room::lock(&state.store)?.player_id.clone();
            hosting
                .host
                .apply(&player, &command)
                .map(|c| privacy::for_player(&c, &player))
        }
        Session::Remote(remote) => {
            let sender = remote.sender.clone();
            drop(session);
            let (reply, response) = tokio::sync::oneshot::channel();
            sender
                .send((command, reply))
                .await
                .map_err(|_| AppError::new("disconnected", "Room connection stopped"))?;
            tokio::time::timeout(std::time::Duration::from_secs(15), response)
                .await
                .map_err(|_| {
                    AppError::new(
                        "timeout",
                        "No acknowledgement; retry the same command ID after reconnect",
                    )
                })?
                .map_err(|_| AppError::new("disconnected", "Connection stopped"))?
        }
        Session::Idle => Err(AppError::new("session", "Join or host a room first")),
    }
}
#[tauri::command]
fn prepare_sync(
    state: State<AppState>,
    sheet_ids: Vec<String>,
    campaign_id: String,
    resolutions: BTreeMap<String, String>,
) -> Result<Vec<SyncSheet>> {
    room::lock(&state.store)?.sync_payload(&sheet_ids, &campaign_id, resolutions)
}
#[tauri::command]
fn accept_campaign(
    state: State<AppState>,
    campaign: Campaign,
    sheet_ids: Vec<String>,
    resolutions: BTreeMap<String, String>,
) -> Result<()> {
    room::lock(&state.store)?.accept_campaign(&campaign, &sheet_ids, &resolutions, &state.catalog)
}
#[tauri::command]
fn export_library(state: State<AppState>) -> Result<String> {
    Ok(serde_json::to_string_pretty(
        &serde_json::json!({"version":1,"library":room::lock(&state.store)?.library()?}),
    )?)
}
#[tauri::command]
fn import_library(state: State<AppState>, text: String) -> Result<()> {
    if text.len() > 10_000_000 {
        return Err(AppError::new("limit", "Import exceeds 10 MB"));
    }
    room::lock(&state.store)?.import_library(&text, &state.catalog)
}
#[tauri::command]
fn backup(state: State<AppState>) -> Result<String> {
    room::lock(&state.store)?.backup()
}
#[tauri::command]
fn list_backups(state: State<AppState>) -> Result<Vec<String>> {
    let path = room::lock(&state.store)?.directory.join("backups");
    if !path.exists() {
        return Ok(vec![]);
    }
    Ok(std::fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into())
        .collect())
}
#[tauri::command]
async fn restore_backup(state: State<'_, AppState>, backup_id: String) -> Result<()> {
    let session = state.session.lock().await;
    if !matches!(*session, Session::Idle) {
        return Err(AppError::new("session", "Leave the room before restoring"));
    }
    room::lock(&state.store)?.restore_backup(&backup_id)
}
pub fn run() {
    let portable = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("portable-data")))
        .filter(|p| p.is_dir());
    let mut builder = tauri::Builder::default();
    if portable.is_none() {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }));
    }
    builder
        .setup(move |app| {
            let directory = portable.clone().unwrap_or(app.path().app_data_dir()?);
            std::fs::create_dir_all(&directory)?;
            let mut options = std::fs::OpenOptions::new();
            options.create(true).truncate(false).read(true).write(true);
            #[cfg(windows)]
            {
                use std::os::windows::fs::OpenOptionsExt;
                options.share_mode(0);
            }
            let profile_lock = options.open(directory.join("profile.lock"))?;
            diagnostics::initialize(&directory);
            let catalog =
                catalog::Catalog::open(&app.path().resource_dir()?.join("resources/catalog.db"))
                    .map_err(|e| std::io::Error::other(e.message))?;
            let store = store::Store::open_with_catalog(&directory, &catalog)
                .map_err(|e| std::io::Error::other(e.message))?;
            app.manage(AppState {
                store: Arc::new(std::sync::Mutex::new(store)),
                catalog: Arc::new(catalog),
                session: tokio::sync::Mutex::new(Session::Idle),
                _profile_lock: profile_lock,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            encounter_habitats,
            encounter_pool,
            generate_encounter,
            encounter_drafts,
            save_encounter_draft,
            delete_encounter_draft,
            preview_capture,
            get_library,
            save_sheet,
            preview_workflow,
            preview_campaign_workflow,
            commit_workflow,
            preview_battle_action,
            preview_point_transfer,
            commit_point_transfer,
            save_workflow_draft,
            list_workflow_drafts,
            delete_workflow_draft,
            get_catalog,
            catalog_entry,
            compendium_detail,
            search_catalog,
            list_campaigns,
            create_campaign,
            host_room,
            join_room,
            leave_room,
            room_command,
            room_members,
            prepare_sync,
            accept_campaign,
            export_library,
            import_library,
            backup,
            list_backups,
            restore_backup
        ])
        .run(tauri::generate_context!())
        .expect("Unable to start PokeVerse");
}
