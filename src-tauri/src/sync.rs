use crate::model::*;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

// Three-way merge preserves independent edits. Arrays are indivisible fields.
pub fn merge(
    base: &Sheet,
    local: &Sheet,
    remote: &Sheet,
    resolutions: &BTreeMap<String, String>,
) -> Result<Sheet> {
    if base.id != local.id
        || local.id != remote.id
        || base.owner_id != local.owner_id
        || local.owner_id != remote.owner_id
    {
        return Err(AppError::new(
            "ownership",
            "Sheet identity cannot change during reconciliation",
        ));
    }
    let mut conflicts = vec![];
    fn walk(
        b: &Value,
        l: &Value,
        r: &Value,
        path: &str,
        choices: &BTreeMap<String, String>,
        conflicts: &mut Vec<String>,
    ) -> Value {
        // Explicit progression reconciliation selects the complete branch,
        // including coupled sheets that did not independently conflict.
        if path == "/progression" {
            match choices.get(path).map(String::as_str) {
                Some("local") => return l.clone(),
                Some("remote") => return r.clone(),
                _ => {}
            }
        }
        if !(path.is_empty() && choices.contains_key("/progression")) {
            if l == r {
                return l.clone();
            }
            if l == b {
                return r.clone();
            }
            if r == b {
                return l.clone();
            }
        }
        if path != "/progression" {
            if let (Some(bo), Some(lo), Some(ro)) = (b.as_object(), l.as_object(), r.as_object()) {
                let keys: BTreeSet<_> = bo.keys().chain(lo.keys()).chain(ro.keys()).collect();
                let mut out = serde_json::Map::new();
                for k in keys {
                    if path.is_empty() && k == "revision" {
                        out.insert(k.clone(), r[k].clone());
                        continue;
                    }
                    let v = walk(
                        &b[k],
                        &l[k],
                        &r[k],
                        &format!("{path}/{k}"),
                        choices,
                        conflicts,
                    );
                    if !v.is_null() || (lo.contains_key(k) && ro.contains_key(k)) {
                        out.insert(k.clone(), v);
                    }
                }
                return Value::Object(out);
            }
        }
        match choices.get(path).map(String::as_str) {
            Some("local") => l.clone(),
            Some("remote") => r.clone(),
            _ => {
                conflicts.push(path.into());
                r.clone()
            }
        }
    }
    let grouped = base.workflow.is_some() || local.workflow.is_some() || remote.workflow.is_some();
    let group = |s: &Sheet| -> Result<Value> {
        let mut value = serde_json::to_value(s)?;
        if grouped {
            let map = value.as_object_mut().unwrap();
            let mut protected = serde_json::Map::new();
            for key in [
                "workflow",
                "stats",
                "skills",
                "rank",
                "moves",
                "ability",
                "species_id",
                "max_hp",
                "xp",
                "inventory",
                "held_items",
                "stored_items",
                "trainer_id",
            ] {
                if let Some(v) = map.remove(key) {
                    protected.insert(key.into(), v);
                }
            }
            map.insert("progression".into(), Value::Object(protected));
        }
        Ok(value)
    };
    let mut value = walk(
        &group(base)?,
        &group(local)?,
        &group(remote)?,
        "",
        resolutions,
        &mut conflicts,
    );
    if grouped {
        let map = value.as_object_mut().unwrap();
        if let Some(Value::Object(fields)) = map.remove("progression") {
            map.extend(fields);
        }
    }
    if !conflicts.is_empty() {
        return Err(AppError {
            code: "conflict".into(),
            message: "Choose personal or campaign values for conflicting fields".into(),
            conflicts,
        });
    }
    let mut sheet: Sheet = serde_json::from_value(value)?;
    sheet.revision = remote.revision;
    Ok(sheet)
}
