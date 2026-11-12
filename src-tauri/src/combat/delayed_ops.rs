//! MOVE-A54 delayed and damage-triggered cards: Future Sight, Doom Desire, Rage,
//! Destiny Bond, Pursuit and Glaive Rush.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct DelayedHit {
    pub source: String,
    pub target: String,
    pub side: String,
    pub slot: Option<u32>,
    pub move_name: String,
    pub damage: i32,
    pub due_round: u32,
}

fn op(c: &Campaign, name: &str) -> Option<&'static str> {
    completion::definition(c, name)
        .filter(|(kind, _)| *kind == "delayed-op")
        .and_then(|(_, d)| d["op"].as_str())
}
fn round(c: &Campaign) -> u32 {
    battle(c).map_or(0, |b| b.round)
}
fn damaging(m: &serde_json::Value) -> bool {
    ["Physical", "Special"].contains(&m["Category"].as_str().unwrap_or(""))
}

/// Future Sight and Doom Desire keep their rolled damage for the start of the next round.
pub fn defer(c: &mut Campaign, q: &QueuedAction, damage: i32) -> Result<bool> {
    if op(c, &q.move_name) != Some("delayed") {
        return Ok(false);
    }
    let target = fighter(c, &q.target_id)?.clone();
    let due_round = round(c) + 1;
    flow_mut(c).completion.delayed.push(DelayedHit {
        source: q.actor_id.clone(),
        target: target.id,
        side: target.side,
        slot: target.field_slot,
        move_name: q.move_name.clone(),
        damage,
        due_round,
    });
    Ok(true)
}
/// Apply due delayed hits when turns start; a departed target's replacement or ally takes it.
pub fn start_turns(c: &mut Campaign) -> Result<Vec<String>> {
    let now = round(c);
    let due: Vec<DelayedHit> = flow(c)?
        .completion
        .delayed
        .iter()
        .filter(|d| d.due_round <= now)
        .cloned()
        .collect();
    flow_mut(c).completion.delayed.retain(|d| d.due_round > now);
    let mut log = vec![];
    for hit in due {
        let combatants = &battle(c)?.combatants;
        let active = |p: &&Combatant| p.deployed && p.hp > 0 && p.side == hit.side;
        let recipient = combatants
            .iter()
            .filter(active)
            .find(|p| p.id == hit.target)
            .or_else(|| {
                combatants
                    .iter()
                    .filter(active)
                    .find(|p| hit.slot.is_some() && p.field_slot == hit.slot)
            })
            .or_else(|| {
                let mut v: Vec<&Combatant> = combatants.iter().filter(active).collect();
                v.sort_by(|a, b| a.id.cmp(&b.id));
                v.into_iter().next()
            })
            .map(|p| p.id.clone());
        if let Some(id) = recipient {
            let p = fighter_mut(c, &id)?;
            p.hp = (p.hp - hit.damage.max(0)).max(0);
            log.push(format!(
                "{} deals {} delayed damage to {}",
                hit.move_name, hit.damage, p.name
            ));
        }
    }
    Ok(log)
}

/// Rage, Destiny Bond and Glaive Rush react to damage the marked combatant receives.
pub fn after_damage(
    c: &mut Campaign,
    q: &QueuedAction,
    m: &serde_json::Value,
    damage: i32,
) -> Result<()> {
    if completion::definition(c, "Rage").is_none() || !damaging(m) {
        return Ok(());
    }
    let now = round(c);
    let target = fighter(c, &q.target_id)?.clone();
    let attacker_side = fighter(c, &q.actor_id)?.side.clone();
    let f = &flow(c)?.completion;
    let raging = f.rage.get(&target.id) == Some(&now) && attacker_side != target.side;
    let bonded = f.destiny_bond.get(&target.id) == Some(&now) && target.hp == 0;
    if damage > 0 && raging {
        extra::stack_stat(c, &target.id, &target.id, "Strength", 1, "Rage")?;
    }
    if damage > 0 && bonded {
        fighter_mut(c, &q.actor_id)?.hp = 0;
    }
    // Glaive Rush vulnerability is spent by the next damaging move that targets its user.
    flow_mut(c).completion.glaive.remove(&target.id);
    Ok(())
}
pub fn vulnerable(c: &Campaign, id: &str) -> bool {
    flow(c).is_ok_and(|f| f.completion.glaive.contains(id))
}
pub fn guard(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> Result<()> {
    if op(c, m["Name"].as_str().unwrap_or("")) == Some("destiny-bond")
        && flow(c)?
            .completion
            .destiny_casts
            .get(&p.id)
            .is_some_and(|r| *r + 1 == round(c))
    {
        return Err(error(
            "Destiny Bond fails if it was used in the previous round",
        ));
    }
    Ok(())
}
pub fn effect(c: &mut Campaign, q: &QueuedAction) -> Result<()> {
    if q.ice {
        return Ok(());
    }
    let now = round(c);
    let (a, t) = (q.actor_id.clone(), q.target_id.clone());
    match op(c, &q.move_name).unwrap_or("") {
        "rage" => {
            flow_mut(c).completion.rage.insert(a, now);
        }
        "destiny-bond" => {
            let f = &mut flow_mut(c).completion;
            f.destiny_bond.insert(a.clone(), now);
            f.destiny_casts.insert(a, now);
        }
        "pursuit" => {
            if q.damage > 0 {
                flow_mut(c).completion.pursued.insert(t, a);
            }
        }
        "glaive" => {
            flow_mut(c).completion.glaive.insert(a);
        }
        _ => {}
    }
    Ok(())
}
/// Pursuit deals 2 typeless damage to a marked target that is recalled, switched out or runs.
pub fn observe(before: &Campaign, next: &mut Campaign) -> Result<()> {
    let (Some(old), Ok(now)) = (before.battle.as_ref(), battle(next)) else {
        return Ok(());
    };
    if old.id != now.id {
        return Ok(());
    }
    let pursued = flow(next)?.completion.pursued.clone();
    let mut hit = vec![];
    for id in pursued.keys() {
        let (Some(was), Some(is)) = (
            old.combatants.iter().find(|p| &p.id == id),
            now.combatants.iter().find(|p| &p.id == id),
        ) else {
            continue;
        };
        let left = was.deployed && !is.deployed;
        let ran = is.used_moves.len() > was.used_moves.len()
            && is.used_moves[was.used_moves.len()..]
                .iter()
                .any(|m| m == "Run Away");
        if (left || ran) && was.hp > 0 {
            hit.push(id.clone());
        }
    }
    for id in hit {
        let p = fighter_mut(next, &id)?;
        p.hp = (p.hp - 2).max(0);
        flow_mut(next).completion.pursued.remove(&id);
    }
    Ok(())
}
pub fn lifecycle(c: &mut Campaign) -> Result<()> {
    let active: BTreeSet<String> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    let now = round(c);
    let f = flow_mut(c);
    f.completion.glaive.retain(|id| active.contains(id));
    f.completion.rage.retain(|_, r| *r == now);
    f.completion.destiny_bond.retain(|_, r| *r == now);
    Ok(())
}
