//! MOVE-A51 history and condition cards: use guards, damage-dice bonuses and Fell Stinger.
use super::*;

fn op(c: &Campaign, name: &str) -> Option<&'static serde_json::Value> {
    completion::definition(c, name)
        .filter(|(kind, _)| *kind == "condition-op")
        .map(|(_, d)| d)
}
fn op_name(c: &Campaign, name: &str) -> Option<&'static str> {
    op(c, name).and_then(|d| d["op"].as_str())
}
fn adopted(c: &Campaign) -> bool {
    flow(c).is_ok_and(|f| f.completion.blocks.contains("MOVE-A51"))
}

/// Record each move Accuracy result; Temper Flare and Stomping Tantrum read the previous one.
pub fn accuracy(c: &mut Campaign, id: &str, move_name: &str, success: bool) -> Result<()> {
    if !adopted(c) {
        return Ok(());
    }
    let previous = flow(c)?.completion.last_accuracy.get(id).copied();
    let sequence = flow(c)?
        .queue
        .iter()
        .rev()
        .find(|q| q.actor_id == id && q.move_name == move_name)
        .map(|q| q.sequence);
    let tantrum = op_name(c, move_name) == Some("tantrum") && previous == Some(false);
    let f = flow_mut(c);
    if let Some(s) = sequence.filter(|_| tantrum) {
        f.completion.tantrum.insert(s);
    }
    f.completion.last_accuracy.insert(id.into(), success);
    if success {
        f.completion
            .performed
            .entry(id.into())
            .or_default()
            .insert(move_name.into());
    }
    Ok(())
}
/// Lash Out reads reductions applied this round.
pub fn lowered(c: &mut Campaign, id: &str) {
    if !adopted(c) {
        return;
    }
    if let Ok(round) = battle(c).map(|b| b.round) {
        flow_mut(c).completion.lowered.insert(id.into(), round);
    }
}
/// Revenge reads any HP loss earlier this round.
pub fn observe(before: &Campaign, next: &mut Campaign) -> Result<()> {
    let (Some(old), Ok(now)) = (before.battle.as_ref(), battle(next)) else {
        return Ok(());
    };
    if old.id != now.id || old.round != now.round {
        return Ok(());
    }
    let round = now.round;
    let hurt: Vec<String> = now
        .combatants
        .iter()
        .filter(|p| old.combatants.iter().any(|v| v.id == p.id && p.hp < v.hp))
        .map(|p| p.id.clone())
        .collect();
    let queued: BTreeSet<u32> = flow(next)?.queue.iter().map(|q| q.sequence).collect();
    let f = flow_mut(next);
    for id in hurt {
        f.completion.damaged.insert(id, round);
    }
    f.completion.tantrum.retain(|s| queued.contains(s));
    Ok(())
}

fn ate_this_round(c: &Campaign, id: &str) -> bool {
    let round = battle(c).map_or(0, |b| b.round);
    items::enabled(c)
        && items::state(c).history.iter().any(|e| {
            e.round == round
                && ["consume_berry", "berry_hit"].contains(&e.kind.as_str())
                && (e.from == id || e.to.as_deref() == Some(id))
        })
}
fn side_used_items(c: &Campaign, side: &str) -> bool {
    let ids: BTreeSet<&str> = battle(c).map_or_else(
        |_| BTreeSet::new(),
        |b| {
            b.combatants
                .iter()
                .filter(|p| p.side == side)
                .map(|p| p.id.as_str())
                .collect()
        },
    );
    items::enabled(c)
        && items::state(c).history.iter().any(|e| {
            (e.kind.starts_with("consume")
                || ["medicine", "recycle", "harvest"].contains(&e.kind.as_str()))
                && (ids.contains(e.from.as_str())
                    || e.to.as_deref().is_some_and(|t| ids.contains(t)))
        })
}

/// Belch and Last Resort check the user before Accuracy.
pub fn guard(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> Result<()> {
    match op_name(c, m["Name"].as_str().unwrap_or("")) {
        Some("belch") if !ate_this_round(c, &p.id) => Err(error(
            "Belch requires the user to have eaten a berry or food this round",
        )),
        Some("last-resort") => {
            let known = rules::effective_sheet(c, p)?.moves;
            let performed = flow(c)?.completion.performed.get(&p.id);
            let missing: Vec<&String> = known
                .iter()
                .filter(|name| *name != "Last Resort")
                .filter(|name| !performed.is_some_and(|set| set.contains(*name)))
                .collect();
            if missing.is_empty() {
                Ok(())
            } else {
                Err(error(&format!(
                    "Last Resort requires every known move to have succeeded this battle; missing: {}",
                    missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                )))
            }
        }
        _ => Ok(()),
    }
}
/// Poltergeist and Ruination check the target before Accuracy.
pub fn target_guard(c: &Campaign, id: &str, m: &serde_json::Value) -> Result<()> {
    match op_name(c, m["Name"].as_str().unwrap_or("")) {
        Some("poltergeist") => {
            let target = fighter(c, id)?;
            if items::held(c, id).is_empty() && !side_used_items(c, &target.side) {
                return Err(error(
                    "Poltergeist requires a held item or an item used on the target's side",
                ));
            }
            Ok(())
        }
        Some("ruination") if flow(c)?.completion.ruined.contains(id) => Err(error(
            "Ruination fails against a target it already hit this battle",
        )),
        _ => Ok(()),
    }
}
/// Extra damage dice from this round's history.
pub fn damage_bonus(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> i32 {
    let name = m["Name"].as_str().unwrap_or("");
    let Some(d) = op(c, name) else {
        return 0;
    };
    let Ok(f) = flow(c) else {
        return 0;
    };
    let round = battle(c).map_or(0, |b| b.round);
    let bonus = d["dice"].as_i64().unwrap_or(0) as i32;
    let applies = match d["op"].as_str() {
        Some("lash-out") => f.completion.lowered.get(&p.id) == Some(&round),
        Some("revenge") => f.completion.damaged.get(&p.id) == Some(&round),
        Some("tantrum") => f
            .queue
            .iter()
            .find(|q| q.actor_id == p.id && q.move_name == name)
            .is_some_and(|q| f.completion.tantrum.contains(&q.sequence)),
        _ => false,
    };
    if applies {
        bonus
    } else {
        0
    }
}
/// Ruination's damage pool is the target's total HP in dice.
pub fn ruination_dice(c: &Campaign, t: &Combatant) -> Result<i32> {
    if op_name(c, "Ruination") != Some("ruination") {
        return Err(AppError::new(
            "manual_rule",
            "Ruination's HP-based damage requires the MOVE-A51 rules",
        ));
    }
    Ok(t.max_hp)
}
pub fn effect(c: &mut Campaign, q: &QueuedAction) -> Result<()> {
    if q.ice {
        return Ok(());
    }
    match op_name(c, &q.move_name).unwrap_or("") {
        "fell-stinger" => {
            let target = fighter(c, &q.target_id)?;
            if q.damage > 0 && target.hp == 0 {
                extra::change_stat(c, &q.actor_id, &q.actor_id, "Strength", 2)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Chilly Reception activates Snowy Weather for 4 rounds before its user switches out.
pub fn switcher_weather(c: &mut Campaign, q: &QueuedAction) -> Result<()> {
    if op_name(c, &q.move_name) != Some("chilly") || q.ice {
        return Ok(());
    }
    field::set_weather(
        c,
        field::WeatherSource {
            weather: field::Weather::Snow,
            source: format!("move:{}", q.sequence),
            rounds: Some(4),
        },
    )
}

/// Ruination remembers every target its damage reached; formula damage has no effect step.
pub fn after_damage(c: &mut Campaign, q: &QueuedAction, damage: i32) {
    if damage > 0 && op_name(c, &q.move_name) == Some("ruination") {
        flow_mut(c).completion.ruined.insert(q.target_id.clone());
    }
}
