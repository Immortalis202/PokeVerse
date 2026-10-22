//! MASTER 541-577: attack-scoped immunity bypass, accuracy and event abilities.
//! Identity is never suppressed or rewritten by a bypassing attack.
use super::*;

pub fn enabled(c: &Campaign) -> bool {
    c.battle
        .as_ref()
        .and_then(|b| b.flow.as_ref())
        .is_some_and(|f| f.reviewed_rules)
}
fn effective_name<'a>(c: &'a Campaign, p: &'a Combatant) -> &'a str {
    c.battle
        .as_ref()
        .and_then(|b| b.flow.as_ref())
        .and_then(|f| f.queue.first())
        .filter(|q| q.actor_id == p.id)
        .and_then(|q| q.damage_context.as_ref())
        .and_then(|ctx| ctx.ability_sources.iter().find(|a| a.id == p.id))
        .map_or(p.selected_ability.as_str(), |a| a.selected_ability.as_str())
}
pub fn extreme(c: &Campaign, p: &Combatant) -> bool {
    enabled(c) && ["Teravolt", "Turboblaze"].contains(&effective_name(c, p))
}
pub fn type_bypass(c: &Campaign, a: &Combatant, t: &Combatant) -> bool {
    enabled(c) && a.side != t.side && (effective_name(c, a) == "Mold Breaker" || extreme(c, a))
}
pub fn damage_bypass(c: &Campaign, a: &Combatant, t: &Combatant) -> bool {
    type_bypass(c, a, t) && t.selected_ability != "Wonder Guard"
}
pub fn source(c: &Campaign, q: &QueuedAction) -> Result<Combatant> {
    Ok(ability_changes::frozen_subject(
        c,
        &q.actor_id,
        q.sequence,
        fighter(c, &q.actor_id)?,
    ))
}
pub fn in_range(c: &Campaign, a: &Combatant, t: &Combatant) -> bool {
    flow(c).is_ok_and(|f| {
        f.ranges
            .get(&a.id)
            .is_none_or(|ids| a.id == t.id || ids.contains(&t.id))
    })
}
pub fn syrup(c: &Campaign, a: &Combatant, t: &Combatant) -> bool {
    if !enabled(c)
        || a.selected_ability != "Supersweet Syrup"
        || a.side == t.side
        || !in_range(c, a, t)
    {
        return false;
    }
    let cap =
        [1, 2, 3, 4, 5, 6, 8, 10][crate::creation::rank(&c.sheets[&a.sheet_id].rank).unwrap_or(0)];
    let mut ids: Vec<_> = extra::participants(c)
        .iter()
        .filter(|p| p.deployed && p.hp > 0 && p.side != a.side && in_range(c, a, p))
        .map(|p| p.id.as_str())
        .collect();
    ids.sort();
    ids.truncate(cap);
    ids.contains(&t.id.as_str())
}
pub fn accuracy_penalty(c: &Campaign, a: &Combatant, t: &Combatant, penalty: i32) -> i32 {
    if enabled(c) && a.selected_ability == "Keen Eye" {
        return 0;
    }
    let penalty = penalty
        - if enabled(c)
            && t.side != a.side
            && t.selected_ability == "Tangled Feet"
            && status(t, "Confusion")
        {
            2
        } else {
            0
        };
    items::accuracy_penalty(c, a, t, penalty)
}
pub fn move_data(c: &Campaign, p: &Combatant, m: &mut serde_json::Value) {
    if !enabled(c) {
        return;
    }
    let support = m["Category"] == "Support";
    let night = flow(c).is_ok_and(|f| f.battlefield.night);
    let victory = !support && field::aura(c, p, "Victory Star", true);
    if victory
        || (p.selected_ability == "Mind's Eye" && night)
        || (p.selected_ability == "Mycelium Might" && support)
    {
        m["Attributes"]["NeverMiss"] = serde_json::json!(true);
    }
    if p.selected_ability == "Mycelium Might" && support {
        // Printed priority collisions remain explicit; do not silently erase a card's Reaction.
        m["Attributes"]["LateReaction"] = serde_json::json!("1");
    }
}
pub fn hit(c: &mut Campaign, q: &QueuedAction, m: &serde_json::Value, cat: &Catalog) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let a = source(c, q)?;
    let t = fighter(c, &q.target_id)?.clone();
    if a.id == t.id || decoys::intercepted(c, q) {
        return Ok(());
    }
    let (_, immune) = rules::type_matchup(c, &a, &t, m["Type"].as_str().unwrap_or("Normal"), cat)?;
    if immune {
        return Ok(());
    }
    let bypass = damage_bypass(c, &a, &t);
    if !bypass && t.hp > 0 && t.selected_ability == "Motor Drive" && m["Type"] == "Electric" {
        extra::once(c, &t.id, "Motor Drive", &[("Dexterity", 1)])?;
    }
    if t.hp > 0 && t.selected_ability == "Justified" && m["Type"] == "Dark" {
        let key = "Justified";
        let n = (*t.runtime.ability.counters.get(key).unwrap_or(&0) + 1).min(3);
        fighter_mut(c, &t.id)?
            .runtime
            .ability
            .counters
            .insert(key.into(), n);
        extra::boost(c, &t.id, "Strength", n)?;
    }
    if t.selected_ability == "Seed Sower"
        && ["Physical", "Special"].contains(&m["Category"].as_str().unwrap_or(""))
    {
        field::seed_sower(c, &t)?;
    }
    if abilities::contact(m) && a.selected_ability != "Long Reach" {
        if t.selected_ability == "Cotton Down" {
            let ids: Vec<_> = battle(c)?
                .combatants
                .iter()
                .filter(|p| p.deployed && p.hp > 0 && p.id != t.id && in_range(c, &t, p))
                .map(|p| p.id.clone())
                .collect();
            let cap = [1, 2, 3, 4, 5, 6, 8, 10]
                [crate::creation::rank(&c.sheets[&t.sheet_id].rank).unwrap_or(0)];
            let mut ids = ids;
            ids.sort_by_key(|id| (fighter(c, id).is_ok_and(|p| p.side == t.side), id.clone()));
            ids.truncate(cap);
            for id in ids {
                extra::ability_drop(c, &id, "Dexterity", 1)?;
            }
        }
        if t.selected_ability == "Toxic Debris" {
            field::toxic_debris(c, &t, &a);
        }
    }
    Ok(())
}
pub fn notes(c: &Campaign, a: &Action) -> Vec<String> {
    if !enabled(c) {
        return vec![];
    }
    let mut out=vec!["Reviewed ability rules: immunity bypass is scoped to this action; target abilities and permanent sheets remain intact. Keen Eye cancels success-removal penalties, not action thresholds or pain. Night is a GM battlefield setting.".into()];
    let source = match a {
        Action::Accuracy { combatant_id, .. }
        | Action::AccuracyTargets { combatant_id, .. }
        | Action::Damage { combatant_id, .. }
        | Action::React { combatant_id, .. } => fighter(c, combatant_id).ok(),
        _ => None,
    };
    if let Some(p) = source {
        if let Some(scope) = abilities::coverage(c, &p.selected_ability) {
            out.push(format!("{}: {scope}", p.selected_ability));
        }
        if ["Mold Breaker", "Teravolt", "Turboblaze"].contains(&p.selected_ability.as_str()) {
            out.push(format!("{}: ignore applicable foe damage immunities (including type immunity); damage reductions, unique non-bypassable effects and ordinary target/action eligibility remain enforced. Extreme targeting/effect bypass covers supported effects; held items and unsupported effects require GM adjudication.",p.selected_ability));
        }
    }
    out
}

pub fn unique_damage_guard(c: &Campaign, a: &Combatant, t: &Combatant, effect: i32) -> Result<()> {
    if extreme(c, a) && a.side != t.side && t.selected_ability == "Wonder Guard" && effect <= 0 {
        return Err(AppError::new("manual_rule","Unique ability stalemate: Teravolt/Turboblaze versus Wonder Guard requires a recorded GM ruling"));
    }
    Ok(())
}

pub fn history(before: &Campaign, next: &mut Campaign) {
    if !enabled(next) {
        return;
    }
    let changes: Vec<_> = next
        .battle
        .as_ref()
        .into_iter()
        .flat_map(|b| &b.combatants)
        .filter_map(|p| {
            let old = before
                .battle
                .as_ref()?
                .combatants
                .iter()
                .find(|q| q.id == p.id)?;
            if old.runtime.ability.counters == p.runtime.ability.counters {
                return None;
            }
            let keys = [
                "Justified",
                "Motor Drive",
                "boost:Strength",
                "boost:Dexterity",
                "debuff:Dexterity",
            ];
            let delta: Vec<_> = keys
                .into_iter()
                .filter_map(|key| {
                    let v = p.runtime.ability.counters.get(key).copied().unwrap_or(0);
                    (old.runtime.ability.counters.get(key).copied().unwrap_or(0) != v)
                        .then(|| format!("{key}={v}"))
                })
                .collect();
            (!delta.is_empty()).then(|| format!("{} ability channel: {}", p.name, delta.join(", ")))
        })
        .collect();
    if !changes.is_empty() {
        flow_mut(next).battlefield.corrections.extend(changes);
    }
}
