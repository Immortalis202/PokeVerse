use super::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct CompendiumReference {
    pub kind: String,
    pub name: String,
    pub role: String,
    pub id: Option<String>,
    #[ts(type = "Record<string, unknown> | null")]
    pub summary: Option<Value>,
    #[ts(type = "Record<string, unknown>")]
    pub requirements: Value,
    pub issue: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct CompendiumDetail {
    pub kind: String,
    #[ts(type = "Record<string, unknown>")]
    pub entry: Value,
    pub references: Vec<CompendiumReference>,
    pub warnings: Vec<String>,
}
impl Catalog {
    pub fn compendium_detail(&self, kind: &str, id: &str) -> Result<Option<CompendiumDetail>> {
        if !["species", "moves", "abilities", "items", "natures"].contains(&kind) {
            return Err(AppError::new("catalog", "Unknown compendium category"));
        }
        let Some(entry) = self.lookup(kind, "AppId", id)? else {
            return Ok(None);
        };
        let db = self.lock()?;
        let mut references = Vec::new();
        let mut resolve = |kind: &str, name: &str, role: &str, requirements: Value| -> Result<()> {
            if name.is_empty() {
                return Ok(());
            }
            let mut stmt = db
                .connection
                .prepare("SELECT id,summary FROM entries WHERE kind=?1 AND name=?2 ORDER BY id")?;
            let matches = stmt
                .query_map(params![kind, name], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let unique = matches.len() == 1;
            references.push(CompendiumReference {
                kind: kind.into(),
                name: name.into(),
                role: role.into(),
                id: unique.then(|| matches[0].0.clone()),
                summary: if unique {
                    Some(serde_json::from_str(&matches[0].1)?)
                } else {
                    None
                },
                requirements,
                issue: if matches.is_empty() {
                    Some("Missing catalog reference".into())
                } else if !unique {
                    Some("Ambiguous catalog reference".into())
                } else {
                    None
                },
            });
            Ok(())
        };
        if kind == "species" {
            for (slot, role) in [
                ("Ability1", "Ordinary"),
                ("Ability2", "Ordinary"),
                ("HiddenAbility", "Hidden"),
                ("EventAbilities", "Event"),
            ] {
                match &entry[slot] {
                    Value::String(name) => resolve("abilities", name, role, json!({}))?,
                    Value::Array(names) => {
                        for name in names {
                            if let Some(name) = name.as_str() {
                                resolve("abilities", name, role, json!({}))?;
                            }
                        }
                    }
                    _ => {}
                }
            }
            if let Some(moves) = entry["Moves"].as_array() {
                for m in moves {
                    resolve(
                        "moves",
                        m["Name"].as_str().unwrap_or(""),
                        m["Learned"].as_str().unwrap_or("Unspecified method"),
                        json!({}),
                    )?;
                }
            }
            if let Some(evolutions) = entry["Evolutions"].as_array() {
                for evolution in evolutions {
                    for (field, role) in [("From", "Pre-evolution"), ("To", "Evolution")] {
                        if let Some(name) = evolution[field].as_str() {
                            resolve("species", name, role, evolution.clone())?;
                        }
                    }
                }
            }
        }
        let mut warnings = Vec::new();
        if let Some(review) = entry.get("Review") {
            let unverified = review["fields"]
                .as_object()
                .map(|fields| {
                    fields
                        .iter()
                        .filter(|(_, v)| v["status"] != "reviewed")
                        .map(|(k, _)| k.as_str())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if !unverified.is_empty() {
                warnings.push(format!(
                    "Unverified fields (reference resolution is not rule verification): {}",
                    unverified.join(", ")
                ));
            }
            if let Some(coverage) = review["coverage"].as_str() {
                warnings.push(format!("Battle effect coverage: {coverage}"));
            }
            if let Some(note) = review["notes"].as_str() {
                warnings.push(note.into());
            }
        }
        if let Some(creation) = entry.get("Creation") {
            for field in ["unresolved_fields", "unresolved_moves"] {
                if let Some(values) = creation[field].as_array().filter(|v| !v.is_empty()) {
                    warnings.push(format!(
                        "Metadata review required: {}: {}",
                        field.replace('_', " "),
                        values
                            .iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
            }
            if let Some(note) = creation["review_note"].as_str().filter(|v| !v.is_empty()) {
                warnings.push(note.into());
            }
        }
        Ok(Some(CompendiumDetail {
            kind: kind.into(),
            entry,
            references,
            warnings,
        }))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn species(c: &Catalog, name: &str) -> CompendiumDetail {
        let entry = c.lookup("species", "Name", name).unwrap().unwrap();
        c.compendium_detail("species", entry["AppId"].as_str().unwrap())
            .unwrap()
            .unwrap()
    }
    #[test]
    fn branches_forms_and_exact_references() {
        let c = Catalog::bundled().unwrap();
        let eevee = species(&c, "Eevee");
        let evos: Vec<_> = eevee
            .references
            .iter()
            .filter(|r| r.role == "Evolution")
            .collect();
        assert_eq!(evos.len(), 8);
        assert!(evos.iter().all(|r| r.id.is_some()));
        assert!(evos.iter().any(|r| r.requirements["Stat"] == "Loyalty"));
        let pika = species(&c, "Pikachu");
        assert!(pika
            .references
            .iter()
            .any(|r| r.role == "Pre-evolution" && r.name == "Pichu"));
        assert!(pika
            .references
            .iter()
            .any(|r| r.name == "Raichu (Alolan Form)" && r.requirements["Region"] == "Alola"));
        assert!(species(&c, "Raichu (Alolan Form)")
            .references
            .iter()
            .any(|r| r.name == "Pikachu" && r.role == "Pre-evolution"));
        assert!(c.compendium_detail("species", "missing").unwrap().is_none());
        assert!(c.compendium_detail("invalid", "missing").is_err());
    }
    #[test]
    fn unresolved_references_preserve_names_and_special_methods() {
        let mut c = Catalog::bundled().unwrap();
        let mut fixture = c.lookup("species", "Name", "Mew").unwrap().unwrap();
        fixture["Ability1"] = json!("Missing ability");
        fixture["Ability2"] = json!("Synchronize");
        fixture["Moves"] = json!([{"Name":"Missing tutor move","Learned":"Tutor"}]);
        fixture["Evolutions"] = json!([]);
        c.replace_for_test(fixture);
        let result = species(&c, "Mew");
        assert!(!result.references.iter().any(|r| r.kind == "species"));
        let missing = result
            .references
            .iter()
            .find(|r| r.name == "Missing tutor move")
            .unwrap();
        assert_eq!(missing.role, "Tutor");
        assert!(missing.id.is_none());
        assert_eq!(missing.issue.as_deref(), Some("Missing catalog reference"));
        c.lock().unwrap().connection.execute("INSERT INTO entries SELECT kind,id || ':duplicate',name,dex,type1,type2,position,summary,data FROM entries WHERE kind='abilities' AND name='Synchronize'",[]).unwrap();
        let result = species(&c, "Mew");
        let duplicate = result
            .references
            .iter()
            .find(|r| r.name == "Synchronize")
            .unwrap();
        assert!(duplicate.id.is_none());
        assert_eq!(
            duplicate.issue.as_deref(),
            Some("Ambiguous catalog reference")
        );
    }
}
