use crate::{catalog::Catalog, model::*, sync};
use rusqlite::{params, Connection, OptionalExtension};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub struct Store {
    pub library: Connection,
    pub campaigns: Connection,
    pub directory: PathBuf,
    pub player_id: String,
}
fn migrate(path: &Path, schema: &str) -> Result<Connection> {
    let mut c = Connection::open(path)?;
    c.busy_timeout(std::time::Duration::from_secs(5))?;
    c.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    let version: u32 = c.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if version > 4 {
        return Err(AppError::new(
            "migration",
            "This database needs a newer PokeVerse version",
        ));
    }
    if version == 0 {
        if path.exists()
            && c.query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table'",
                [],
                |r| r.get::<_, i32>(0),
            )? > 0
        {
            let backup = path.with_extension(format!("pre-migration-{}.db", id()));
            c.execute("VACUUM INTO ?1", [backup.to_string_lossy().as_ref()])?;
        }
        let tx = c.transaction()?;
        tx.execute_batch(schema)?;
        tx.execute_batch("PRAGMA user_version=1;")?;
        tx.commit()?;
    }
    Ok(c)
}
fn migrate_encounters(
    directory: &Path,
    library: &Connection,
    campaigns: &Connection,
) -> Result<()> {
    let encounter_migrated: bool = library.query_row(
        "SELECT EXISTS(SELECT 1 FROM metadata WHERE key='encounters_v1')",
        [],
        |r| r.get(0),
    )?;
    if !encounter_migrated {
        for (name, db) in [("library", &library), ("campaigns", &campaigns)] {
            db.execute(
                "VACUUM INTO ?1",
                [directory
                    .join(format!("{name}-pre-encounters-{}.db", id()))
                    .to_string_lossy()
                    .as_ref()],
            )?;
        }
        library.execute_batch("BEGIN IMMEDIATE; CREATE TABLE encounter_drafts(id TEXT PRIMARY KEY,data TEXT NOT NULL); CREATE TABLE campaign_cache(id TEXT PRIMARY KEY,data TEXT NOT NULL); INSERT INTO metadata VALUES ('encounters_v1','1'); PRAGMA user_version=3; COMMIT;")?;
    }
    library.execute_batch(
        "CREATE TABLE IF NOT EXISTS campaign_cache(id TEXT PRIMARY KEY,data TEXT NOT NULL);",
    )?;
    let teams: bool = library.query_row(
        "SELECT EXISTS(SELECT 1 FROM metadata WHERE key='teams_v1')",
        [],
        |r| r.get(0),
    )?;
    if !teams {
        for (name, db) in [("library", library), ("campaigns", campaigns)] {
            db.execute(
                "VACUUM INTO ?1",
                [directory
                    .join(format!("{name}-pre-teams-{}.db", id()))
                    .to_string_lossy()
                    .as_ref()],
            )?;
        }
        let tx = library.unchecked_transaction()?;
        tx.execute("INSERT INTO metadata VALUES ('teams_v1','1')", [])?;
        tx.execute_batch("PRAGMA user_version=4;")?;
        tx.commit()?;
    }
    library.execute_batch("PRAGMA user_version=4;")?;
    campaigns.execute_batch("PRAGMA user_version=4;")?;
    Ok(())
}
impl Store {
    #[cfg(test)]
    pub fn open(directory: &Path) -> Result<Self> {
        Self::open_with_catalog(directory, &Catalog::bundled()?)
    }
    pub fn open_with_catalog(directory: &Path, cat: &Catalog) -> Result<Self> {
        std::fs::create_dir_all(directory)?;
        let library = migrate(&directory.join("library.db"), "
            CREATE TABLE metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL);
            CREATE TABLE sheets(id TEXT PRIMARY KEY,data TEXT NOT NULL);
            CREATE TABLE history(id TEXT PRIMARY KEY,data TEXT NOT NULL);
            CREATE TABLE membership(sheet_id TEXT PRIMARY KEY,campaign_id TEXT NOT NULL,baseline TEXT NOT NULL);
            CREATE TABLE receipts(id TEXT PRIMARY KEY,data TEXT NOT NULL);
            CREATE TABLE credentials(campaign_id TEXT PRIMARY KEY,credential TEXT NOT NULL);
        ")?;
        let campaigns = migrate(&directory.join("campaigns.db"), "
            CREATE TABLE campaigns(id TEXT PRIMARY KEY,data TEXT NOT NULL);
            CREATE TABLE members(campaign_id TEXT NOT NULL,player_id TEXT NOT NULL,credential_hash TEXT NOT NULL,PRIMARY KEY(campaign_id,player_id));
            CREATE TABLE receipts(campaign_id TEXT NOT NULL,id TEXT NOT NULL,actor_id TEXT NOT NULL,PRIMARY KEY(campaign_id,id));
            CREATE TABLE room_tokens(campaign_id TEXT PRIMARY KEY,token TEXT NOT NULL);
        ")?;
        let player_id = library
            .query_row(
                "SELECT value FROM metadata WHERE key='player_id'",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()?
            .unwrap_or_else(id);
        library.execute(
            "INSERT OR IGNORE INTO metadata VALUES ('player_id',?1)",
            [&player_id],
        )?;
        library.execute_batch(
            "CREATE TABLE IF NOT EXISTS workflow_drafts(id TEXT PRIMARY KEY,data TEXT NOT NULL);",
        )?;
        let migrated: bool = library.query_row(
            "SELECT EXISTS(SELECT 1 FROM metadata WHERE key='workflow_v1')",
            [],
            |r| r.get(0),
        )?;
        if !migrated {
            for (name, c) in [("library", &library), ("campaigns", &campaigns)] {
                let file = directory.join(format!("{name}-pre-workflow-{}.db", id()));
                c.execute("VACUUM INTO ?1", [file.to_string_lossy().as_ref()])?;
            }
            let rows = {
                let mut q = library.prepare("SELECT id,data FROM sheets")?;
                let rows = q
                    .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                rows
            };
            library.execute_batch("BEGIN IMMEDIATE")?;
            for (key, text) in rows {
                let mut s: Sheet = serde_json::from_str(&text)?;
                if s.workflow.is_none() {
                    s.workflow = Some(crate::creation::build(&s, cat));
                    let history=Progression {id:id(),sheet_id:s.id.clone(),revision:s.revision,description:format!("Workflow migration: legacy XP {} converted 1:1 to Training Points; sheet requires review",s.xp),xp_cost:0,before:serde_json::from_str(&text)?,after:s.clone()};
                    library.execute(
                        "INSERT INTO history VALUES (?1,?2)",
                        params![history.id, serde_json::to_string(&history)?],
                    )?;
                }
                library.execute(
                    "UPDATE sheets SET data=?2 WHERE id=?1",
                    params![key, serde_json::to_string(&s)?],
                )?;
            }
            let old_campaigns = {
                let mut q = campaigns.prepare("SELECT id,data FROM campaigns")?;
                let rows = q
                    .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                rows
            };
            for (key, text) in old_campaigns {
                let mut campaign: Campaign = serde_json::from_str(&text)?;
                // Currency migration does not adopt a new rules/habitat catalog.
                // The appointed GM explicitly accepts that change in the room.
                for s in campaign.sheets.values_mut() {
                    if s.workflow.is_none() {
                        s.workflow = Some(crate::creation::build(s, cat));
                    }
                }
                campaign.proposals.clear();
                campaign.battle_undo.clear();
                campaigns.execute(
                    "UPDATE campaigns SET data=?2 WHERE id=?1",
                    params![key, serde_json::to_string(&campaign)?],
                )?;
            }
            library.execute(
                "INSERT INTO metadata VALUES ('workflow_v1','xp-to-tp-1:1')",
                [],
            )?;
            library.execute_batch("COMMIT")?;
            library.execute_batch("PRAGMA user_version=2;")?;
            campaigns.execute_batch("PRAGMA user_version=2;")?;
        }
        migrate_encounters(directory, &library, &campaigns)?;
        Ok(Self {
            library,
            campaigns,
            directory: directory.into(),
            player_id,
        })
    }
    pub fn library(&self) -> Result<Library> {
        fn rows<T: serde::de::DeserializeOwned>(c: &Connection, table: &str) -> Result<Vec<T>> {
            let mut q = c.prepare(&format!("SELECT data FROM {table} ORDER BY rowid"))?;
            let values = q
                .query_map([], |r| r.get::<_, String>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            values
                .into_iter()
                .map(|v| Ok(serde_json::from_str(&v)?))
                .collect()
        }
        Ok(Library {
            player_id: self.player_id.clone(),
            sheets: rows(&self.library, "sheets")?,
            history: rows(&self.library, "history")?,
        })
    }
    pub fn sheet(&self, sheet_id: &str) -> Result<Option<Sheet>> {
        self.library
            .query_row("SELECT data FROM sheets WHERE id=?1", [sheet_id], |r| {
                r.get::<_, String>(0)
            })
            .optional()?
            .map(|s| serde_json::from_str(&s).map_err(Into::into))
            .transpose()
    }
    pub fn preview_workflow(
        &self,
        request: &WorkflowRequest,
        catalog: &Catalog,
    ) -> Result<WorkflowPreview> {
        if request.sheet.owner_id != self.player_id {
            return Err(AppError::new(
                "ownership",
                "Draft must belong to this player",
            ));
        }
        let old = self.sheet(&request.sheet.id)?;
        let trainer = request
            .sheet
            .trainer_id
            .as_ref()
            .map(|id| self.sheet(id))
            .transpose()?
            .flatten();
        Ok(crate::creation::preview(
            old.as_ref(),
            trainer.as_ref(),
            request,
            catalog,
        ))
    }
    pub fn workflow_drafts(&self) -> Result<Vec<WorkflowRequest>> {
        let mut q = self
            .library
            .prepare("SELECT data FROM workflow_drafts ORDER BY rowid DESC")?;
        let texts = q
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        texts
            .into_iter()
            .map(|t| serde_json::from_str(&t).map_err(Into::into))
            .collect()
    }
    pub fn save_workflow_draft(&mut self, request: &WorkflowRequest) -> Result<()> {
        if request.sheet.owner_id != self.player_id || request.operation_id.is_empty() {
            return Err(AppError::new("ownership", "Invalid draft owner or ID"));
        }
        let text = serde_json::to_string(request)?;
        if text.len() > 500_000 {
            return Err(AppError::new("limit", "Draft exceeds 500 KB"));
        }
        self.library.execute("INSERT INTO workflow_drafts VALUES (?1,?2) ON CONFLICT(id) DO UPDATE SET data=excluded.data",params![request.operation_id,text])?;
        Ok(())
    }
    pub fn commit_workflow(
        &mut self,
        request: &WorkflowRequest,
        catalog: &Catalog,
    ) -> Result<Sheet> {
        if let Some(text) = self
            .library
            .query_row(
                "SELECT data FROM receipts WHERE id=?1",
                [&request.operation_id],
                |r| r.get::<_, String>(0),
            )
            .optional()?
        {
            return Ok(serde_json::from_str(&text)?);
        }
        if request.sheet.owner_id != self.player_id {
            return Err(AppError::new(
                "ownership",
                "Personal character must belong to this player",
            ));
        }
        let old = self.sheet(&request.sheet.id)?;
        let trainer = request
            .sheet
            .trainer_id
            .as_ref()
            .map(|id| self.sheet(id))
            .transpose()?
            .flatten();
        let next = crate::creation::apply(old.as_ref(), trainer.as_ref(), request, catalog, None)?;
        catalog.validate(&next)?;
        let history = Progression {
            id: request.operation_id.clone(),
            sheet_id: next.id.clone(),
            revision: next.revision,
            description: format!("{}: {}", request.kind, request.reason),
            xp_cost: old
                .as_ref()
                .and_then(|s| s.workflow.as_ref())
                .map_or(0, |b| b.training_points)
                - next.workflow.as_ref().map_or(0, |b| b.training_points),
            before: old.unwrap_or_else(|| next.clone()),
            after: next.clone(),
        };
        let text = serde_json::to_string(&next)?;
        let tx = self.library.transaction()?;
        tx.execute(
            "INSERT INTO sheets VALUES (?1,?2) ON CONFLICT(id) DO UPDATE SET data=excluded.data",
            params![next.id, text],
        )?;
        tx.execute(
            "INSERT INTO history VALUES (?1,?2)",
            params![history.id, serde_json::to_string(&history)?],
        )?;
        tx.execute(
            "INSERT INTO receipts VALUES (?1,?2)",
            params![request.operation_id, text],
        )?;
        tx.execute(
            "DELETE FROM workflow_drafts WHERE id=?1",
            [&request.operation_id],
        )?;
        tx.commit()?;
        Ok(next)
    }
    pub fn save(&mut self, request: SaveRequest, catalog: &Catalog) -> Result<Sheet> {
        if let Some(text) = self
            .library
            .query_row(
                "SELECT data FROM receipts WHERE id=?1",
                [&request.operation_id],
                |r| r.get::<_, String>(0),
            )
            .optional()?
        {
            return Ok(serde_json::from_str(&text)?);
        }
        if request.operation_id.is_empty()
            || request.description.trim().is_empty()
            || request.xp_cost < 0
        {
            return Err(AppError::new(
                "validation",
                "Operation ID, description, and nonnegative cost required",
            ));
        }
        let mut next = request.sheet;
        if next.owner_id != self.player_id {
            return Err(AppError::new(
                "ownership",
                "Personal sheets must belong to this player",
            ));
        }
        let before = self.sheet(&next.id)?;
        if let Some(old) = before.as_ref().filter(|s| s.workflow.is_some()) {
            if crate::creation::fingerprint(old)? != crate::creation::fingerprint(&next)?
                || old.hp != next.hp
                || old.conditions != next.conditions
                || old.workflow.as_ref().map(|b| b.will) != next.workflow.as_ref().map(|b| b.will)
            {
                return Err(AppError::new(
                    "workflow_required",
                    "Use a creation/progression operation for mechanical changes",
                ));
            }
        } else if next.workflow.is_some() {
            if let Some(b) = next.workflow.as_mut() {
                b.status = "review".into();
                b.ledger.clear();
            }
        }
        if before.as_ref().map_or(0, |s| s.revision) != request.expected_revision {
            return Err(AppError::new(
                "stale",
                "Sheet changed; reload before saving",
            ));
        }
        if let Some(old) = &before {
            if old.kind != next.kind || old.owner_id != next.owner_id {
                return Err(AppError::new(
                    "ownership",
                    "Sheet kind and owner are immutable",
                ));
            }
            if request.xp_cost > old.xp {
                return Err(AppError::new("validation", "Not enough XP"));
            }
            if request.xp_cost > 0 {
                next.xp = old.xp - request.xp_cost;
            }
        } else if request.xp_cost > 0 {
            return Err(AppError::new("validation", "New sheets cannot spend XP"));
        }
        next.revision = request
            .expected_revision
            .checked_add(1)
            .ok_or_else(|| AppError::new("limit", "Revision exhausted"))?;
        catalog.validate(&next)?;
        if next.kind == "pokemon" {
            let trainer = self
                .sheet(next.trainer_id.as_deref().unwrap_or_default())?
                .ok_or_else(|| AppError::new("validation", "Select a trainer first"))?;
            if trainer.kind != "trainer" || trainer.archived {
                return Err(AppError::new("validation", "Trainer must be active"));
            }
        }
        let text = serde_json::to_string(&next)?;
        let tx = self.library.transaction()?;
        tx.execute(
            "INSERT INTO sheets VALUES (?1,?2) ON CONFLICT(id) DO UPDATE SET data=excluded.data",
            params![next.id, text],
        )?;
        let history = Progression {
            id: request.operation_id.clone(),
            sheet_id: next.id.clone(),
            revision: next.revision,
            description: request.description,
            xp_cost: request.xp_cost,
            before: before.unwrap_or_else(|| next.clone()),
            after: next.clone(),
        };
        tx.execute(
            "INSERT INTO history VALUES (?1,?2)",
            params![history.id, serde_json::to_string(&history)?],
        )?;
        tx.execute(
            "INSERT INTO receipts VALUES (?1,?2)",
            params![request.operation_id, text],
        )?;
        tx.commit()?;
        Ok(next)
    }
    pub fn sync_payload(
        &self,
        ids: &[String],
        campaign_id: &str,
        resolutions: BTreeMap<String, String>,
    ) -> Result<Vec<SyncSheet>> {
        let linked = self.linked_point_ids(ids, campaign_id, None)?;
        let mut out = vec![];
        for key in &linked {
            let sheet = self
                .sheet(key)?
                .ok_or_else(|| AppError::new("missing", "Unknown personal sheet"))?;
            let membership = self
                .library
                .query_row(
                    "SELECT campaign_id,baseline FROM membership WHERE sheet_id=?1",
                    [key],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
                )
                .optional()?;
            let baseline = if let Some((campaign, text)) = membership {
                if campaign != campaign_id {
                    return Err(AppError::new(
                        "membership",
                        "Duplicate this character to join a different campaign",
                    ));
                }
                Some(serde_json::from_str(&text)?)
            } else {
                None
            };
            out.push(SyncSheet {
                sheet,
                baseline,
                resolutions: resolutions.clone(),
            });
        }
        Ok(out)
    }
    pub fn accept_campaign(
        &mut self,
        campaign: &Campaign,
        ids: &[String],
        resolutions: &BTreeMap<String, String>,
        catalog: &Catalog,
    ) -> Result<()> {
        let linked = self.linked_point_ids(ids, &campaign.id, Some(campaign))?;
        let mut changes = vec![];
        for key in &linked {
            let Some(remote) = campaign
                .sheets
                .get(key)
                .filter(|s| s.owner_id == self.player_id)
            else {
                continue;
            };
            let existing = self.sheet(key)?;
            if existing.is_none() {
                catalog.validate(remote)?;
                changes.push((remote.clone(), remote.clone(), remote.clone()));
                continue;
            }
            let local = existing.unwrap();
            let membership = self
                .library
                .query_row(
                    "SELECT campaign_id,baseline FROM membership WHERE sheet_id=?1",
                    [key],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
                )
                .optional()?;
            let mut next = if let Some((cid, text)) = membership {
                if cid != campaign.id {
                    return Err(AppError::new(
                        "membership",
                        "Character belongs to another campaign",
                    ));
                }
                sync::merge(&serde_json::from_str(&text)?, &local, remote, resolutions)?
            } else {
                remote.clone()
            };
            catalog.validate(&next)?;
            next.revision = local.revision + 1;
            changes.push((next, remote.clone(), local));
        }
        let mut reconciled = campaign.sheets.clone();
        for (next, _, _) in &changes {
            reconciled.insert(next.id.clone(), next.clone());
        }
        if crate::points::has_transfers(reconciled.values()) {
            let changed_ids = changes
                .iter()
                .map(|(next, _, _)| next.id.clone())
                .collect::<Vec<_>>();
            crate::points::verify_batch(&campaign.sheets, &reconciled, &changed_ids, catalog)
                .map_err(|e| AppError {
                    code: "conflict".into(),
                    message: format!("Reconcile the complete linked progression chain: {}. Choose campaign values or request a reviewed rebuild", e.message),
                    conflicts: vec!["/progression".into()],
                })?;
        }
        let tx = self.library.transaction()?;
        for (next, remote, before) in changes {
            let history = Progression {
                id: id(),
                sheet_id: next.id.clone(),
                revision: next.revision,
                description: format!("Reconciled campaign {}", campaign.name),
                xp_cost: 0,
                before,
                after: next.clone(),
            };
            tx.execute(
                "INSERT INTO sheets VALUES (?1,?2) ON CONFLICT(id) DO UPDATE SET data=excluded.data",
                params![next.id, serde_json::to_string(&next)?],
            )?;
            tx.execute("INSERT INTO membership VALUES (?1,?2,?3) ON CONFLICT(sheet_id) DO UPDATE SET baseline=excluded.baseline",params![next.id,campaign.id,serde_json::to_string(&remote)?])?;
            tx.execute(
                "INSERT INTO history VALUES (?1,?2)",
                params![history.id, serde_json::to_string(&history)?],
            )?;
            if let Some(build) = &remote.workflow {
                for record in &build.ledger {
                    tx.execute("DELETE FROM workflow_drafts WHERE id=?1", [&record.id])?;
                }
            }
        }
        tx.execute("INSERT INTO campaign_cache VALUES (?1,?2) ON CONFLICT(id) DO UPDATE SET data=excluded.data",params![campaign.id,serde_json::to_string(campaign)?])?;
        tx.commit()?;
        Ok(())
    }
    pub fn campaigns(&self) -> Result<Vec<Campaign>> {
        let mut q = self
            .campaigns
            .prepare("SELECT data FROM campaigns ORDER BY rowid DESC")?;
        let rows = q
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        rows.into_iter()
            .map(|s| Ok(serde_json::from_str(&s)?))
            .collect()
    }
    pub fn cache_campaign(&self, c: &Campaign) -> Result<()> {
        let prior: Option<String> = self
            .library
            .query_row(
                "SELECT data FROM campaign_cache WHERE id=?1",
                [&c.id],
                |r| r.get(0),
            )
            .optional()?;
        if prior
            .as_ref()
            .map(|text| serde_json::from_str::<Campaign>(text))
            .transpose()?
            .is_some_and(|old| old.revision > c.revision)
        {
            return Ok(());
        }
        self.library.execute("INSERT INTO campaign_cache VALUES (?1,?2) ON CONFLICT(id) DO UPDATE SET data=excluded.data",params![c.id,serde_json::to_string(c)?])?;
        Ok(())
    }
    pub fn available_campaigns(&self) -> Result<Vec<Campaign>> {
        let mut all: BTreeMap<String, Campaign> = BTreeMap::new();
        let mut query = self.library.prepare("SELECT data FROM campaign_cache")?;
        for text in query.query_map([], |r| r.get::<_, String>(0))? {
            let c: Campaign = serde_json::from_str(&text?)?;
            all.insert(c.id.clone(), c);
        }
        for c in self.campaigns()? {
            all.insert(c.id.clone(), c);
        }
        Ok(all.into_values().collect())
    }
    pub fn put_campaign(&mut self, c: &Campaign) -> Result<()> {
        self.campaigns.execute(
            "INSERT INTO campaigns VALUES (?1,?2) ON CONFLICT(id) DO UPDATE SET data=excluded.data",
            params![c.id, serde_json::to_string(c)?],
        )?;
        Ok(())
    }
    pub fn backup(&self) -> Result<String> {
        let directory = self.directory.join("backups").join(id());
        std::fs::create_dir_all(&directory)?;
        for (name, c) in [
            ("library.db", &self.library),
            ("campaigns.db", &self.campaigns),
        ] {
            c.execute(
                "VACUUM INTO ?1",
                [directory.join(name).to_string_lossy().as_ref()],
            )?;
        }
        Ok(directory.to_string_lossy().into())
    }
    pub fn restore_backup(&mut self, backup_id: &str) -> Result<()> {
        if uuid::Uuid::parse_str(backup_id).is_err() {
            return Err(AppError::new("validation", "Select a generated backup ID"));
        }
        let directory = self.directory.join("backups").join(backup_id);
        let root = self.directory.join("backups").canonicalize()?;
        if !directory.canonicalize()?.starts_with(root) {
            return Err(AppError::new(
                "validation",
                "Backup is outside application data",
            ));
        }
        let open = |name: &str| -> Result<Connection> {
            let connection = Connection::open_with_flags(
                directory.join(name),
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )?;
            if connection.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))?
                != "ok"
                || !matches!(
                    connection.query_row("PRAGMA user_version", [], |r| r.get::<_, u32>(0))?,
                    2..=4
                )
            {
                return Err(AppError::new(
                    "validation",
                    "Invalid or incompatible backup",
                ));
            }
            Ok(connection)
        };
        let library = open("library.db")?;
        let campaigns = open("campaigns.db")?;
        let player = library.query_row(
            "SELECT value FROM metadata WHERE key='player_id'",
            [],
            |r| r.get::<_, String>(0),
        )?;
        if player != self.player_id {
            return Err(AppError::new(
                "ownership",
                "Restore requires a backup of this player identity",
            ));
        }
        // Verify the table shapes before modifying either live connection.
        library.prepare("SELECT data FROM sheets")?;
        campaigns.prepare("SELECT data FROM campaigns")?;
        let rollback = self.backup()?;
        fn copy(source: &Connection, destination: &mut Connection) -> Result<()> {
            rusqlite::backup::Backup::new(source, destination)?.run_to_completion(
                100,
                std::time::Duration::from_millis(10),
                None,
            )?;
            Ok(())
        }
        let result = copy(&library, &mut self.library)
            .and_then(|_| copy(&campaigns, &mut self.campaigns))
            .and_then(|_| migrate_encounters(&self.directory, &self.library, &self.campaigns));
        if let Err(error) = result {
            for (name, destination) in [
                ("library.db", &mut self.library),
                ("campaigns.db", &mut self.campaigns),
            ] {
                let previous = Connection::open(std::path::Path::new(&rollback).join(name))?;
                copy(&previous, destination)?;
            }
            return Err(error);
        }
        Ok(())
    }
    pub fn import_library(&mut self, text: &str, catalog: &Catalog) -> Result<()> {
        #[derive(serde::Deserialize)]
        struct Export {
            version: u32,
            library: Library,
        }
        let export: Export = serde_json::from_str(text)?;
        if export.version != 1 {
            return Err(AppError::new("version", "Unsupported export version"));
        }
        let mut imported = export.library.sheets;
        let imported_history = export.library.history;
        let remap: BTreeMap<_, _> = imported.iter().map(|s| (s.id.clone(), id())).collect();
        for sheet in &mut imported {
            sheet.id = remap[&sheet.id].clone();
            sheet.owner_id = self.player_id.clone();
            sheet.revision = 1;
            if sheet.workflow.is_none() {
                sheet.workflow = Some(crate::creation::build(sheet, catalog));
            }
            if let Some(b) = sheet.workflow.as_mut() {
                b.status = "review".into();
                b.origin = "legacy".into();
                b.ledger.clear();
            }
            if let Some(t) = &sheet.trainer_id {
                sheet.trainer_id = Some(
                    remap
                        .get(t)
                        .ok_or_else(|| {
                            AppError::new("validation", "Export is missing a Pokémon's trainer")
                        })?
                        .clone(),
                );
            }
            catalog.validate(sheet)?;
        }
        for sheet in &imported {
            if sheet.kind == "pokemon"
                && !imported
                    .iter()
                    .any(|t| Some(&t.id) == sheet.trainer_id.as_ref() && t.kind == "trainer")
            {
                return Err(AppError::new("validation", "Invalid trainer relationship"));
            }
        }
        let tx = self.library.transaction()?;
        for sheet in imported {
            tx.execute(
                "INSERT INTO sheets VALUES (?1,?2)",
                params![sheet.id, serde_json::to_string(&sheet)?],
            )?;
        }
        for mut history in imported_history {
            let Some(key) = remap.get(&history.sheet_id) else {
                continue;
            };
            history.id = id();
            history.sheet_id = key.clone();
            history.description = format!("Imported history: {}", history.description);
            for sheet in [&mut history.before, &mut history.after] {
                sheet.id = key.clone();
                sheet.owner_id = self.player_id.clone();
                sheet.trainer_id = sheet
                    .trainer_id
                    .as_ref()
                    .and_then(|t| remap.get(t).cloned());
            }
            tx.execute(
                "INSERT INTO history VALUES (?1,?2)",
                params![history.id, serde_json::to_string(&history)?],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}
