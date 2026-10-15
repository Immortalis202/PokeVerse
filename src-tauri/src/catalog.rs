use crate::model::*;
use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::Path,
    sync::{Arc, Mutex},
};

const CACHE_LIMIT: usize = 128;
struct CatalogDb {
    connection: Connection,
    cache: HashMap<(String, String, String), Value>,
    order: VecDeque<(String, String, String)>,
}
#[derive(Clone)]
pub struct Catalog {
    db: Arc<Mutex<CatalogDb>>,
    pub hash: String,
    pub source_hash: String,
}
impl Catalog {
    pub fn habitats(&self) -> Result<Vec<Value>> {
        let db = self.lock()?;
        let mut stmt = db
            .connection
            .prepare("SELECT data FROM habitats ORDER BY rowid")?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        rows.into_iter()
            .map(|s| Ok(serde_json::from_str(&s)?))
            .collect()
    }
    pub fn habitat_pool(&self, habitat: &str, query: &str, offset: u32) -> Result<Value> {
        if query.len() > 200 {
            return Err(AppError::new("validation", "Search is too long"));
        }
        let db = self.lock()?;
        let condition = "h.habitat_id=?1 AND json_extract(h.provenance,'$.included_in_pool')=1 AND instr(lower(e.name),lower(?2))>0";
        let total: u32 = db.connection.query_row(&format!("SELECT count(*) FROM species_habitats h JOIN entries e ON e.id=h.species_id AND e.kind='species' WHERE {condition}"), params![habitat,query], |r| r.get(0))?;
        let mut stmt = db.connection.prepare(&format!("SELECT e.summary,h.provenance FROM species_habitats h JOIN entries e ON e.id=h.species_id AND e.kind='species' WHERE {condition} ORDER BY e.position LIMIT 150 OFFSET ?3"))?;
        let rows = stmt
            .query_map(params![habitat, query, offset], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut entries = vec![];
        for (summary, provenance) in rows {
            let mut v: Value = serde_json::from_str(&summary)?;
            v["habitat_provenance"] = serde_json::from_str(&provenance)?;
            entries.push(v);
        }
        Ok(json!({"entries":entries,"total":total}))
    }
    pub fn habitat_members(&self, habitat: &str) -> Result<Vec<String>> {
        let db = self.lock()?;
        let mut stmt=db.connection.prepare("SELECT species_id FROM species_habitats WHERE habitat_id=?1 AND json_extract(provenance,'$.included_in_pool')=1 ORDER BY species_id")?;
        let rows = stmt
            .query_map([habitat], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }
    pub fn habitat_conditions(&self, habitat: &str, species: &str) -> Result<String> {
        let db = self.lock()?;
        Ok(db.connection.query_row("SELECT coalesce(json_extract(provenance,'$.conditions'),'') FROM species_habitats WHERE habitat_id=?1 AND species_id=?2 AND json_extract(provenance,'$.included_in_pool')=1",params![habitat,species],|r|r.get(0))?)
    }
    pub fn open(path: &Path) -> Result<Self> {
        let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        connection.execute_batch("PRAGMA query_only=ON; PRAGMA cache_size=-2048;")?;
        let version: u32 = connection.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version != 1 {
            return Err(AppError::new("catalog", "Unsupported catalog schema"));
        }
        let metadata = |key: &str| -> Result<String> {
            let text: String =
                connection.query_row("SELECT value FROM metadata WHERE key=?1", [key], |r| {
                    r.get(0)
                })?;
            Ok(serde_json::from_str(&text)?)
        };
        let hash = metadata("hash")?;
        let source_hash = metadata("source_hash")?;
        Ok(Self {
            db: Arc::new(Mutex::new(CatalogDb {
                connection,
                cache: HashMap::new(),
                order: VecDeque::new(),
            })),
            hash,
            source_hash,
        })
    }
    #[cfg(test)]
    pub fn bundled() -> Result<Self> {
        Self::open(&Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/catalog.db"))
    }
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, CatalogDb>> {
        self.db
            .lock()
            .map_err(|_| AppError::new("catalog", "Catalog lock unavailable"))
    }
    pub fn lookup(&self, kind: &str, key: &str, id: &str) -> Result<Option<Value>> {
        let column = match key {
            "AppId" => "id",
            "Name" => "name",
            _ => return Err(AppError::new("catalog", "Unsupported lookup key")),
        };
        let cache_key = (kind.to_owned(), key.to_owned(), id.to_owned());
        let mut db = self.lock()?;
        if let Some(value) = db.cache.get(&cache_key) {
            return Ok(Some(value.clone()));
        }
        let text: Option<String> = db
            .connection
            .prepare_cached(&format!(
                "SELECT data FROM entries WHERE kind=?1 AND {column}=?2 ORDER BY position LIMIT 1"
            ))?
            .query_row(params![kind, id], |r| r.get(0))
            .optional()?;
        let value = text
            .map(|s| serde_json::from_str::<Value>(&s))
            .transpose()?;
        if let Some(value) = &value {
            if db.cache.len() >= CACHE_LIMIT {
                if let Some(old) = db.order.pop_front() {
                    db.cache.remove(&old);
                }
            }
            db.order.push_back(cache_key.clone());
            db.cache.insert(cache_key, value.clone());
        }
        Ok(value)
    }
    pub fn find(&self, kind: &str, key: &str, id: &str) -> Option<Value> {
        // A damaged installed catalog must not silently authorize a rule operation.
        // Initialization and IPC report errors; missing records remain validation failures.
        self.lookup(kind, key, id).unwrap_or_else(|e| {
            crate::diagnostics::record(&e.code);
            None
        })
    }
    pub fn bootstrap(&self) -> Result<Value> {
        let db = self.lock()?;
        let mut out = serde_json::Map::new();
        for key in ["edition", "hash", "warnings"] {
            let text: String =
                db.connection
                    .query_row("SELECT value FROM metadata WHERE key=?1", [key], |r| {
                        r.get(0)
                    })?;
            out.insert(key.into(), serde_json::from_str(&text)?);
        }
        for kind in ["species", "moves", "abilities", "natures", "items"] {
            let mut stmt = db
                .connection
                .prepare_cached("SELECT summary FROM entries WHERE kind=?1 ORDER BY position")?;
            let rows = stmt
                .query_map([kind], |r| r.get::<_, String>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            out.insert(
                kind.into(),
                Value::Array(
                    rows.into_iter()
                        .map(|s| serde_json::from_str(&s))
                        .collect::<std::result::Result<Vec<_>, _>>()?,
                ),
            );
        }
        Ok(Value::Object(out))
    }
    pub fn search(&self, kind: &str, query: &str, type_filter: &str, offset: u32) -> Result<Value> {
        if !["species", "moves", "abilities", "items", "natures"].contains(&kind)
            || query.len() > 200
            || type_filter.len() > 40
        {
            return Err(AppError::new("catalog", "Invalid catalog search"));
        }
        let db = self.lock()?;
        let condition =
            "kind=?1 AND instr(lower(name),lower(?2))>0 AND (?3='' OR type1=?3 OR type2=?3)";
        let total: u32 = db.connection.query_row(
            &format!("SELECT count(*) FROM entries WHERE {condition}"),
            params![kind, query, type_filter],
            |r| r.get(0),
        )?;
        let mut stmt = db.connection.prepare_cached(&format!(
            "SELECT summary FROM entries WHERE {condition} ORDER BY position LIMIT 150 OFFSET ?4"
        ))?;
        let rows = stmt
            .query_map(params![kind, query, type_filter, offset], |r| {
                r.get::<_, String>(0)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let entries = rows
            .into_iter()
            .map(|s| serde_json::from_str::<Value>(&s))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(json!({"entries": entries, "total": total}))
    }
    #[cfg(test)]
    pub fn replace_for_test(&mut self, entry: Value) {
        // Isolated copy for adversarial fixtures; never mutate the bundled catalog.
        let mut target = Connection::open_in_memory().unwrap();
        let source = self.db.lock().unwrap();
        rusqlite::backup::Backup::new(&source.connection, &mut target)
            .unwrap()
            .run_to_completion(1000, std::time::Duration::ZERO, None)
            .unwrap();
        drop(source);
        target
            .execute(
                "UPDATE entries SET data=?1 WHERE kind='species' AND id=?2",
                params![entry.to_string(), entry["AppId"].as_str().unwrap()],
            )
            .unwrap();
        self.db = Arc::new(Mutex::new(CatalogDb {
            connection: target,
            cache: HashMap::new(),
            order: VecDeque::new(),
        }));
    }
    pub fn validate(&self, s: &Sheet) -> Result<()> {
        if s.id.is_empty()
            || s.owner_id.is_empty()
            || s.name.trim().is_empty()
            || s.name.len() > 200
            || s.notes.len() > 20_000
        {
            return Err(AppError::new(
                "validation",
                "A sheet needs an ID, owner, and name; text exceeds permitted size",
            ));
        }
        if !["trainer", "pokemon"].contains(&s.kind.as_str())
            || ![
                "Starter", "Rookie", "Standard", "Advanced", "Expert", "Ace", "Master", "Champion",
            ]
            .contains(&s.rank.as_str())
        {
            return Err(AppError::new("validation", "Invalid sheet kind or rank"));
        }
        if s.max_hp < 1
            || s.max_hp > 999
            || s.hp < 0
            || s.hp > s.max_hp
            || s.xp < 0
            || s.stats
                .values()
                .chain(s.skills.values())
                .chain(s.inventory.values())
                .any(|n| *n < 0 || *n > 100_000)
        {
            return Err(AppError::new(
                "validation",
                "HP, XP, attributes, skills, and inventory must be within valid numeric bounds",
            ));
        }
        if s.moves.len() != s.moves.iter().collect::<HashSet<_>>().len() {
            return Err(AppError::new("validation", "Duplicate learned move"));
        }
        if !s.nature.is_empty() && self.find("natures", "Name", &s.nature).is_none() {
            return Err(AppError::new("validation", "Unknown nature"));
        }
        for m in &s.moves {
            if self.find("moves", "Name", m).is_none() {
                return Err(AppError::new("validation", format!("Unknown move: {m}")));
            }
        }
        if s.kind == "pokemon" {
            let species = self
                .find(
                    "species",
                    "AppId",
                    s.species_id.as_deref().unwrap_or_default(),
                )
                .ok_or_else(|| AppError::new("validation", "Select a catalog species/form"))?;
            for (key, value) in &s.stats {
                let max_key = format!("Max{key}");
                if let Some(max) = species[&max_key].as_i64() {
                    if s.workflow.is_none() && i64::from(*value) > max {
                        return Err(AppError::new(
                            "validation",
                            format!("{key} exceeds species maximum {max}"),
                        ));
                    }
                }
            }
            if !s.ability.is_empty() {
                let listed = ["Ability1", "Ability2", "HiddenAbility", "EventAbilities"]
                    .iter()
                    .any(|k| species[*k].as_str() == Some(&s.ability));
                if (!listed && s.workflow.is_none())
                    || self.find("abilities", "Name", &s.ability).is_none()
                {
                    return Err(AppError::new(
                        "validation",
                        "Ability does not belong to this species",
                    ));
                }
            }
            for m in &s.moves {
                if s.workflow.is_none()
                    && !species["Moves"]
                        .as_array()
                        .is_some_and(|a| a.iter().any(|v| v["Name"].as_str() == Some(m)))
                {
                    return Err(AppError::new(
                        "validation",
                        format!("{m} is not in this species' learnset"),
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod sqlite_tests {
    use super::*;
    #[test]
    fn read_only_catalog_and_normalized_relations() {
        let catalog = Catalog::bundled().unwrap();
        let db = catalog.lock().unwrap();
        assert!(db.connection.execute("DELETE FROM entries", []).is_err());
        let stats: u32 = db
            .connection
            .query_row("SELECT count(*) FROM species_stats", [], |r| r.get(0))
            .unwrap();
        assert_eq!(stats, 1200 * 5);
        let missing: u32 = db.connection.query_row("SELECT count(*) FROM learnsets l LEFT JOIN entries e ON e.kind='species' AND e.id=l.species_id WHERE e.id IS NULL", [], |r| r.get(0)).unwrap();
        assert_eq!(missing, 0);
    }
    #[test]
    fn summaries_do_not_ship_full_records() {
        let catalog = Catalog::bundled().unwrap();
        let bootstrap = catalog.bootstrap().unwrap();
        assert_eq!(bootstrap["species"].as_array().unwrap().len(), 1200);
        assert!(bootstrap["species"][0].get("Moves").is_none());
        assert!(bootstrap["species"][0].get("Creation").is_none());
        assert!(bootstrap["moves"][0].get("Effect").is_none());
        assert!(bootstrap.to_string().len() < 500_000);
        let entry = catalog
            .lookup("species", "Name", "Pikachu")
            .unwrap()
            .unwrap();
        assert!(entry["Moves"].is_array());
        assert!(entry["Creation"].is_object());
        assert_eq!(bootstrap["hash"], catalog.hash);
    }
    #[test]
    fn search_pagination_filters_and_literal_input() {
        let catalog = Catalog::bundled().unwrap();
        let first = catalog.search("species", "", "", 0).unwrap();
        let second = catalog.search("species", "", "", 150).unwrap();
        assert_eq!(first["total"], 1200);
        assert_eq!(first["entries"].as_array().unwrap().len(), 150);
        assert_ne!(first["entries"][0]["AppId"], second["entries"][0]["AppId"]);
        let found = catalog.search("species", "pikachu", "Electric", 0).unwrap();
        assert!(found["total"].as_u64().unwrap() > 0);
        assert_eq!(
            catalog.search("species", "' OR 1=1 --", "", 0).unwrap()["total"],
            0
        );
        let percent = catalog.search("species", "%", "", 0).unwrap();
        assert!(percent["total"].as_u64().unwrap() < 1200);
        assert!(percent["entries"]
            .as_array()
            .unwrap()
            .iter()
            .all(|e| e["Name"].as_str().unwrap().contains('%')));
        assert!(catalog.search("invalid", "", "", 0).is_err());
        assert!(catalog.lookup("species", "invalid", "x").is_err());
    }
    #[test]
    fn cache_is_bounded_and_clones_share_read_only_connection() {
        let catalog = Catalog::bundled().unwrap();
        let clone = catalog.clone();
        let list = catalog.bootstrap().unwrap();
        for entry in list["species"].as_array().unwrap().iter().take(200) {
            catalog
                .lookup("species", "AppId", entry["AppId"].as_str().unwrap())
                .unwrap();
        }
        assert_eq!(clone.lock().unwrap().cache.len(), CACHE_LIMIT);
        assert_eq!(clone.lock().unwrap().order.len(), CACHE_LIMIT);
        assert!(clone
            .lookup("species", "AppId", "missing")
            .unwrap()
            .is_none());
    }
    #[test]
    fn missing_or_wrong_schema_fails_without_creating_database() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("absent.db");
        assert!(Catalog::open(&path).is_err());
        assert!(!path.exists());
        Connection::open(&path)
            .unwrap()
            .execute_batch("PRAGMA user_version=99")
            .unwrap();
        assert!(Catalog::open(&path).is_err());
    }
}

#[path = "compendium.rs"]
mod compendium;
pub use compendium::{CompendiumDetail, CompendiumReference};
