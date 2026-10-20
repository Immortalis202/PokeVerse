//! MASTER p.531; Substitute p.502, Shed Tail p.500, Tidy Up p.505.
//! Decoys are encounter resources, never character sheets or new combatants.
use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct DecoyState {
    pub enabled: bool,
    pub active: BTreeMap<String, Decoy>,
    // Keep the interception after a decoy breaks, until that hit's effects resolve.
    pub intercepted: BTreeSet<String>,
    pub history: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Decoy {
    pub hp: i32,
    pub defenses: [i32; 2],
    pub source: String,
    pub move_name: String,
}
pub fn enabled(c: &Campaign) -> bool {
    c.battle
        .as_ref()
        .and_then(|b| b.flow.as_ref())
        .is_some_and(|f| f.decoys.enabled)
}
fn state(c: &Campaign) -> &DecoyState {
    &c.battle.as_ref().unwrap().flow.as_ref().unwrap().decoys
}
fn state_mut(c: &mut Campaign) -> &mut DecoyState {
    &mut c.battle.as_mut().unwrap().flow.as_mut().unwrap().decoys
}
fn key(q: &QueuedAction) -> String {
    format!("{}:{}:{}", q.sequence, q.actor_id, q.target_id)
}
fn record(c: &mut Campaign, text: String) {
    state_mut(c).history.push(text);
}
pub fn protected_subject(c: &Campaign, source: &str, target: &str, m: &serde_json::Value) -> bool {
    enabled(c) && state(c).active.contains_key(target) && !bypass_data(c, source, m)
}
fn bypass_data(c: &Campaign, source: &str, m: &serde_json::Value) -> bool {
    fighter(c, source).is_ok_and(|p| reviewed::extreme(c, p))
        || m["Attributes"]["SoundMove"] == true
        || fighter(c, source).is_ok_and(|p| {
            p.selected_ability == "Infiltrator"
                || (p.selected_ability == "Unseen Fist" && abilities::contact(m))
        })
        || [
            "Chip Away",
            "Hyperspace Hole",
            "Hyperspace Fury",
            "Phantom Force",
            "Shadow Force",
            "Future Sight",
            "Doom Desire",
            "Snatch",
            "Tidy Up",
        ]
        .contains(&m["Name"].as_str().unwrap_or(""))
}
pub fn bypass(c: &Campaign, q: &QueuedAction, m: &serde_json::Value) -> bool {
    bypass_data(c, &q.actor_id, m)
}
pub fn protects(c: &Campaign, q: &QueuedAction, m: &serde_json::Value) -> bool {
    enabled(c)
        && q.field_scope.is_none()
        && state(c).active.contains_key(&q.target_id)
        && !bypass(c, q, m)
}
pub fn intercepted(c: &Campaign, q: &QueuedAction) -> bool {
    enabled(c) && state(c).intercepted.contains(&key(q))
}
pub fn blocks_effects(c: &Campaign, q: &QueuedAction, m: &serde_json::Value) -> bool {
    if !enabled(c) || q.field_scope.is_some() {
        return false;
    }
    // Shield Moves explicitly protect decoys; casting, theft and removal are explicit exceptions.
    if m["Attributes"]["ShieldMove"] == true
        || ["Substitute", "Shed Tail", "Snatch", "Tidy Up"].contains(&q.move_name.as_str())
    {
        return false;
    }
    intercepted(c, q) || protects(c, q, m)
}
fn defenses(c: &Campaign, id: &str, cat: &Catalog) -> Result<[i32; 2]> {
    let p = fighter(c, id)?;
    let stats = crate::creation::profile_stats(&rules::effective_sheet(c, p)?);
    Ok([
        field::defense(
            c,
            p,
            true,
            cat,
            stats["Defense"] + abilities::defense(c, p, true),
        )?
        .max(1),
        field::defense(
            c,
            p,
            false,
            cat,
            stats["SpecialDefense"] + abilities::defense(c, p, false),
        )?
        .max(1),
    ])
}
pub fn pool_defense(
    c: &Campaign,
    source: &str,
    target: &str,
    m: &serde_json::Value,
    physical: bool,
) -> Option<i32> {
    if !protected_subject(c, source, target, m) {
        return None;
    }
    Some(state(c).active[target].defenses[usize::from(!physical)])
}
// Defensive abilities belong to the Pokemon. The decoy retains only its frozen defense scores.
pub fn damage_subject(
    c: &Campaign,
    source: &str,
    target: &Combatant,
    m: &serde_json::Value,
) -> Combatant {
    let mut p = target.clone();
    if protected_subject(c, source, &p.id, m) {
        p.selected_ability.clear();
    }
    p
}
pub fn cast(c: &mut Campaign, id: &str, name: &str, cat: &Catalog) -> Result<()> {
    if !enabled(c) {
        return Err(AppError::new(
            "manual_rule",
            "Substitute decoys require the current battle engine",
        ));
    }
    let defenses = defenses(c, id, cat)?;
    let p = fighter_mut(c, id)?;
    if p.hp == 0 {
        return Err(error("A fainted Pokemon cannot create a decoy"));
    }
    p.hp = (p.hp - 2).max(0);
    state_mut(c).active.insert(
        id.into(),
        Decoy {
            hp: 2,
            defenses,
            source: id.into(),
            move_name: name.into(),
        },
    );
    record(
        c,
        format!(
            "{name}: {id} pays 2 HP; decoy 2 HP, frozen defenses {}/{}",
            defenses[0], defenses[1]
        ),
    );
    Ok(())
}
// Returns true even for zero damage: the hit's added effects must remain blocked.
pub fn damage(c: &mut Campaign, q: &QueuedAction, m: &serde_json::Value, amount: i32) -> bool {
    if !protects(c, q, m) || m["Category"] == "Support" {
        return false;
    }
    state_mut(c).intercepted.insert(key(q));
    let decoy = state_mut(c).active.get_mut(&q.target_id).unwrap();
    let lost = amount.max(0).min(decoy.hp);
    decoy.hp -= lost;
    let remaining = decoy.hp;
    if remaining == 0 {
        state_mut(c).active.remove(&q.target_id);
    }
    record(c, format!("{} hits {}'s decoy: {lost} HP lost, {remaining} HP remaining; no overflow or added target effects", q.move_name, q.target_id));
    true
}
pub fn snatch(c: &mut Campaign, from: &str, to: &str, cat: &Catalog) -> Result<()> {
    if !enabled(c) || from == to {
        return Ok(());
    }
    if let Some(mut decoy) = state(c).active.get(from).cloned() {
        decoy.defenses = defenses(c, to, cat)?;
        state_mut(c).active.remove(from);
        record(
            c,
            format!(
                "Snatch: decoy transferred {from} -> {to}, {} HP; new user's defenses {}/{}",
                decoy.hp, decoy.defenses[0], decoy.defenses[1]
            ),
        );
        state_mut(c).active.insert(to.into(), decoy);
    }
    Ok(())
}
pub fn switch(c: &mut Campaign, from: &str, to: Option<&str>, name: &str) {
    if !enabled(c) {
        return;
    }
    if let Some(decoy) = state_mut(c).active.remove(from) {
        if name == "Shed Tail" {
            if let Some(to) = to {
                state_mut(c).active.insert(to.into(), decoy);
                record(c, format!("Shed Tail: decoy transferred {from} -> {to}; retains casting HP and defenses"));
            }
        } else {
            record(c, format!("{from}'s decoy removed on departure ({name})"));
        }
    }
}
pub fn tidy(c: &mut Campaign) {
    if enabled(c) {
        state_mut(c).active.clear();
        record(c, "Tidy Up removes every active decoy".into());
    }
}
pub fn lifecycle(c: &mut Campaign) {
    if !enabled(c) {
        return;
    }
    let ids: BTreeSet<_> = c
        .battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0 && c.battle.as_ref().unwrap().active)
        .map(|p| p.id.clone())
        .collect();
    state_mut(c).active.retain(|id, _| ids.contains(id));
    let pending: BTreeSet<_> = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .iter()
        .map(key)
        .collect();
    state_mut(c).intercepted.retain(|k| pending.contains(k));
}

// Identify typed user clauses without inferring their mechanics from prose.
pub fn unreviewed_user_clause(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(map) => map.iter().any(|(key, value)| {
            (["Affects", "Target"].contains(&key.as_str())
                && ["User", "Self"].contains(&value.as_str().unwrap_or("")))
                || unreviewed_user_clause(value)
        }),
        serde_json::Value::Array(values) => values.iter().any(unreviewed_user_clause),
        _ => false,
    }
}
