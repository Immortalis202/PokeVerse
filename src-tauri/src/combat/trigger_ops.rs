//! MOVE-A55 Shell Trap, Powder, Curse and Flame Burst.
use super::*;

fn op(c: &Campaign, name: &str) -> Option<&'static str> {
    completion::definition(c, name)
        .filter(|(kind, _)| *kind == "trigger-op")
        .and_then(|(_, d)| d["op"].as_str())
}

// ----- Shell Trap -----

/// Shell Trap only answers a non-ranged Physical attack aimed at its user.
pub fn reaction_guard(
    c: &Campaign,
    id: &str,
    name: &str,
    incoming: &QueuedAction,
    incoming_move: &serde_json::Value,
) -> Result<()> {
    if op(c, name) != Some("shell-trap") {
        return Ok(());
    }
    let aimed = incoming.target_id == id || incoming.targets.iter().any(|t| t.target_id == id);
    if incoming_move["Category"] != "Physical"
        || incoming_move["Attributes"]["Ranged"] == true
        || !aimed
    {
        return Err(error(
            "Shell Trap only reacts to a non-ranged Physical attack against its user",
        ));
    }
    Ok(())
}
pub fn action_guard(c: &Campaign, name: &str) -> Result<()> {
    if op(c, name) == Some("shell-trap") {
        return Err(error(
            "Shell Trap is a Late Reaction to a non-ranged Physical attack against its user",
        ));
    }
    Ok(())
}
/// Shell Trap's damage is rolled only once that triggering attack has hit its user.
pub fn damage_guard(c: &Campaign, key: &str, sequence: u32, m: &serde_json::Value) -> Result<()> {
    if op(c, m["Name"].as_str().unwrap_or("")) != Some("shell-trap") {
        return Ok(());
    }
    let f = flow(c)?;
    let response = f
        .queue
        .iter()
        .find(|q| q.sequence == sequence)
        .and_then(|q| q.response_to);
    let round = battle(c)?.round;
    let hit = special::history(c).iter().any(|h| {
        h.target == key
            && h.round == round
            && h.damage > 0
            && h.category == "Physical"
            && response.is_none_or(|r| h.sequence == r)
    });
    if hit {
        Ok(())
    } else {
        Err(error(
            "Shell Trap was not triggered: the attack did not hit its user; cancel this damage",
        ))
    }
}
pub fn responds(c: &Campaign, name: &str) -> bool {
    op(c, name) == Some("shell-trap")
}

// ----- Powder -----

/// A Powdered combatant that declares a Fire move owes typeless damage dice equal to its Power.
pub fn declared(c: &mut Campaign, id: &str, m: &serde_json::Value) {
    if op(c, "Powder").is_none() || m["Type"] != "Fire" {
        return;
    }
    let powdered = flow(c).is_ok_and(|f| f.completion.powdered.contains(id));
    if !powdered {
        return;
    }
    let power = m["Power"].as_i64().unwrap_or(0).max(0) as u32;
    let f = flow_mut(c);
    f.completion.powdered.remove(id);
    if power > 0 {
        f.completion.powder_due.insert(id.into(), power.min(100));
    }
}
pub fn powder_dice(c: &Campaign, id: &str) -> Result<u32> {
    flow(c)?
        .completion
        .powder_due
        .get(id)
        .copied()
        .ok_or_else(|| error("No Powder damage is due for this combatant"))
}
pub fn powder(c: &mut Campaign, id: &str, dice: &[u8]) -> Result<i32> {
    powder_dice(c, id)?;
    let damage = rules::successes(dice);
    let p = fighter_mut(c, id)?;
    p.hp = (p.hp - damage).max(0);
    flow_mut(c).completion.powder_due.remove(id);
    Ok(damage)
}

// ----- Curse -----

/// Curse removes one success from its target's damage rolls (Accuracy uses the shared loss channel).
pub fn curse_loss(c: &Campaign, id: &str) -> i32 {
    i32::from(flow(c).is_ok_and(|f| {
        f.special
            .effects
            .iter()
            .any(|e| e.target == id && e.name == "accuracy-loss:Curse")
    }))
}

// ----- Flame Burst -----

fn foes(c: &Campaign, q: &QueuedAction) -> Result<Vec<String>> {
    let side = fighter(c, &q.actor_id)?.side.clone();
    let mut ids: Vec<String> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0 && p.side != side && p.id != q.target_id)
        .map(|p| p.id.clone())
        .collect();
    ids.sort();
    Ok(ids)
}
/// Flame Burst's splash: "none", one other foe, or two other foes joined with '+'.
pub fn options(c: &Campaign, q: &QueuedAction) -> Result<Vec<String>> {
    if op(c, &q.move_name) != Some("flame-burst") || !q.damage_done || q.damage == 0 {
        return Ok(vec![]);
    }
    let foes = foes(c, q)?;
    if foes.is_empty() {
        return Ok(vec![]);
    }
    let mut options = vec!["none".to_string()];
    for (i, a) in foes.iter().enumerate() {
        options.push(a.clone());
        for b in &foes[i + 1..] {
            options.push(format!("{a}+{b}"));
        }
    }
    Ok(options)
}
fn splash(c: &Campaign, q: &QueuedAction) -> Vec<String> {
    flow(c)
        .ok()
        .and_then(|f| f.completion.move_options.get(&q.sequence))
        .filter(|o| *o != "none")
        .map(|o| o.split('+').map(str::to_owned).collect())
        .unwrap_or_default()
}

pub fn effect_dice(c: &Campaign, q: &QueuedAction) -> Option<u32> {
    let op = op(c, &q.move_name)?;
    if q.ice {
        return Some(0);
    }
    Some(match op {
        "curse" => fighter(c, &q.actor_id).map_or(0, |p| (p.hp / 2).max(0) as u32),
        "flame-burst" => splash(c, q).len() as u32,
        _ => 0,
    })
}
pub fn effect(c: &mut Campaign, q: &QueuedAction, dice: &[u8], cat: &Catalog) -> Result<()> {
    if q.ice {
        return Ok(());
    }
    match op(c, &q.move_name).unwrap_or("") {
        "powder" => {
            if !completion::immune_target(c, q, cat)? {
                flow_mut(c).completion.powdered.insert(q.target_id.clone());
            }
        }
        "curse" => {
            let loss = rules::successes(dice);
            let p = fighter_mut(c, &q.actor_id)?;
            p.hp = (p.hp - loss).max(0);
            if !completion::immune_target(c, q, cat)? {
                let started = battle(c)?.round;
                let f = flow_mut(c);
                f.special
                    .effects
                    .retain(|e| e.name != "accuracy-loss:Curse" || e.target != q.target_id);
                f.special.effects.push(special::TimedEffect {
                    name: "accuracy-loss:Curse".into(),
                    source: q.actor_id.clone(),
                    target: q.target_id.clone(),
                    rounds: None,
                    amount: 1,
                    started,
                    slot: "accuracy-success".into(),
                });
            }
        }
        "flame-burst" => {
            let targets = splash(c, q);
            let valid = foes(c, q)?;
            for (i, id) in targets.iter().enumerate() {
                if !valid.contains(id) {
                    return Err(error("The saved Flame Burst target is no longer available"));
                }
                if rules::successes(&dice[i..=i]) > 0 {
                    field::hurt(c, id, 1, "Fire", cat)?;
                }
            }
        }
        _ => {}
    }
    flow_mut(c).completion.move_options.remove(&q.sequence);
    Ok(())
}
pub fn lifecycle(c: &mut Campaign) -> Result<()> {
    let active: BTreeSet<String> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    flow_mut(c)
        .completion
        .powdered
        .retain(|id| active.contains(id));
    Ok(())
}
