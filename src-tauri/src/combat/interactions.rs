//! Batch 5: exact reviewed move definitions and shared protection predicates.
use super::*;
pub fn enabled(c: &Campaign) -> bool {
    flow(c).is_ok_and(|f| f.interaction_rules && f.reviewed_rules)
}
pub fn unseen(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> bool {
    enabled(c)
        && reviewed::effective_subject(c, p).has_ability("Unseen Fist")
        && abilities::contact(m)
}
pub fn cover_bypass(c: &Campaign, id: &str, m: &serde_json::Value) -> bool {
    fighter(c, id).is_ok_and(|p| enabled(c) && (unseen(c, p, m) || reviewed::extreme(c, p)))
}
#[derive(Deserialize)]
struct Definition {
    changes: Vec<Change>,
}
#[derive(Deserialize)]
struct Change {
    keys: Vec<String>,
    amount: i32,
    #[serde(rename = "self")]
    user: bool,
}
pub fn move_effect(c: &mut Campaign, q: &QueuedAction, dice: &[u8], cat: &Catalog) -> Result<bool> {
    if !enabled(c) {
        return Ok(false);
    }
    let definitions: &BTreeMap<String, Definition> = {
        static DATA: std::sync::OnceLock<BTreeMap<String, Definition>> = std::sync::OnceLock::new();
        DATA.get_or_init(|| {
            let data: serde_json::Value =
                serde_json::from_str(include_str!("../../../rules/battle-interactions.json"))
                    .unwrap();
            serde_json::from_value(data["moves"].clone()).unwrap()
        })
    };
    if let Some(effect) = chances().get(&q.move_name) {
        let m = move_entry(cat, &q.move_name)?;
        let source = reviewed::source(c, q)?;
        let target = fighter(c, &q.target_id)?;
        let (effectiveness, immune) =
            rules::type_matchup(c, &source, target, m["Type"].as_str().unwrap_or(""), cat)?;
        if special::target_effects_blocked(c, q, &m)
            || immune
            || abilities::damage_immune_from(c, &source, target, &m)
            || extra::source_immune(c, &source, target, effectiveness)
            || (!reviewed::extreme(c, &source)
                && source.side != target.side
                && target.has_ability("Shield Dust"))
            || (effect.status == "Flinch" && target.runtime.flinch_round == battle(c)?.round)
        {
            return Ok(true);
        }
        if dice.contains(&6) {
            if let Err(e) =
                abilities::inflict(c, &q.actor_id, &q.target_id, &effect.status, "", None, cat)
            {
                if ![
                    "Species type is immune to this condition",
                    "Selected ability is immune to this condition",
                    "Battlefield prevents this condition",
                ]
                .contains(&e.message.as_str())
                {
                    return Err(e);
                }
            }
        }
        return Ok(true);
    }
    let Some(definition) = definitions.get(&q.move_name) else {
        return Ok(false);
    };
    let m = move_entry(cat, &q.move_name)?;
    if special::target_effects_blocked(c, q, &m)
        || (!reviewed::extreme(c, &reviewed::source(c, q)?)
            && (extra::effects_immune(c, fighter(c, &q.target_id)?, &m)
                || abilities::move_immune(c, fighter(c, &q.target_id)?, &m)))
    {
        return Ok(true);
    }
    for change in &definition.changes {
        let target = if change.user {
            &q.actor_id
        } else {
            &q.target_id
        };
        for key in &change.keys {
            if ["AccuracyPool", "HighCritical"].contains(&key.as_str()) {
                let amount = special::record_modifier(c, &q.actor_id, target, key, change.amount)?;
                fighter_mut(c, target)?
                    .runtime
                    .modifiers
                    .insert(key.clone(), amount);
            } else {
                extra::change_stat(c, &q.actor_id, target, key, change.amount)?;
            }
        }
    }
    Ok(true)
}

/// Reviewed numeric trait channels keep their own units and never change sheets.
pub fn trait_value(c: &Campaign, p: &Combatant, key: &str) -> i32 {
    if !enabled(c) {
        return 0;
    }
    let values = p
        .runtime
        .modifiers
        .get(key)
        .copied()
        .into_iter()
        .chain(stat_channels::move_values(c, p, key));
    values.max().unwrap_or(0).max(0)
}

#[derive(Deserialize)]
struct Chance {
    status: String,
    dice: u32,
}
fn chances() -> &'static BTreeMap<String, Chance> {
    static DATA: std::sync::OnceLock<BTreeMap<String, Chance>> = std::sync::OnceLock::new();
    DATA.get_or_init(|| {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../../../rules/battle-interactions.json")).unwrap();
        serde_json::from_value(data["chance_moves"].clone()).unwrap()
    })
}
pub fn effect_dice(c: &Campaign, q: &QueuedAction) -> Result<Option<u32>> {
    if !enabled(c) {
        return Ok(None);
    }
    let Some(effect) = chances().get(&q.move_name) else {
        return Ok(None);
    };
    let source = reviewed::source(c, q)?;
    let target = fighter(c, &q.target_id)?;
    let bypass = source.side != target.side && reviewed::extreme(c, &source);
    let n = effect.dice + 2 * u32::from(source.has_ability("Serene Grace"));
    Ok(Some(n.saturating_sub(
        2 * u32::from(!bypass && target.has_ability("Wonder Skin")),
    )))
}
pub fn native_flinch(c: &Campaign, name: &str) -> Option<(&'static str, u32)> {
    enabled(c)
        .then(|| chances().get(name))
        .flatten()
        .filter(|e| e.status == "Flinch")
        .map(|e| ("Flinch", e.dice))
}
