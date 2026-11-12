//! MOVE-A53 allied contributions: Helping Hand, Help Another, Beat Up and Round.
use super::*;

fn op(c: &Campaign, name: &str) -> Option<&'static str> {
    completion::definition(c, name)
        .filter(|(kind, _)| *kind == "allied-op")
        .and_then(|(_, d)| d["op"].as_str())
}
fn round(c: &Campaign) -> u32 {
    battle(c).map_or(0, |b| b.round)
}

/// Extra dice on the next Accuracy roll: Helping Hand (2) plus Help Another's sixes.
pub fn accuracy_dice(c: &Campaign, id: &str) -> i32 {
    let Ok(f) = flow(c) else { return 0 };
    2 * i32::from(f.completion.helping_accuracy.contains(id))
        + f.completion.help_dice.get(id).map_or(0, |(n, _)| *n as i32)
}
pub fn consume_accuracy(c: &mut Campaign, id: &str) {
    if let Some(f) = c.battle.as_mut().and_then(|b| b.flow.as_mut()) {
        f.completion.helping_accuracy.remove(id);
        f.completion.help_dice.remove(id);
    }
}
/// Extra damage dice: Helping Hand (2), Beat Up allies (2 each) and a Round chorus (3).
pub fn damage_bonus(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> i32 {
    let Ok(f) = flow(c) else { return 0 };
    let name = m["Name"].as_str().unwrap_or("");
    let helping = 2 * i32::from(
        f.completion.helping_damage.contains(&p.id)
            && ["Physical", "Special"].contains(&m["Category"].as_str().unwrap_or("")),
    );
    let beat_up = if op(c, name) == Some("beat-up") {
        f.queue
            .iter()
            .find(|q| q.actor_id == p.id && q.move_name == name)
            .and_then(|q| f.completion.beat_up.get(&q.sequence))
            .map_or(0, |allies| 2 * allies.len() as i32)
    } else {
        0
    };
    let chorus = if op(c, name) == Some("round") {
        3 * i32::from(f.completion.round_chorus.get(&p.id) == Some(&round(c)))
    } else {
        0
    };
    helping + beat_up + chorus
}
pub fn after_damage(c: &mut Campaign, q: &QueuedAction) {
    if completion::definition(c, "Helping Hand").is_none() {
        return;
    }
    let is_round = op(c, &q.move_name) == Some("round");
    let f = flow_mut(c);
    f.completion.helping_damage.remove(&q.actor_id);
    f.completion.beat_up.remove(&q.sequence);
    if is_round {
        f.completion.round_chorus.remove(&q.actor_id);
    }
}

pub fn effect(c: &mut Campaign, q: &QueuedAction, cat: &Catalog) -> Result<()> {
    if q.ice {
        return Ok(());
    }
    if op(c, &q.move_name) == Some("helping-hand") && !completion::immune_target(c, q, cat)? {
        // Helping Hand does not stack: a second cast refreshes the same bonus.
        let f = flow_mut(c);
        f.completion.helping_accuracy.insert(q.target_id.clone());
        f.completion.helping_damage.insert(q.target_id.clone());
    }
    Ok(())
}

fn ally_of(c: &Campaign, helper: &str, ally: &str) -> Result<()> {
    let (h, a) = (fighter(c, helper)?, fighter(c, ally)?);
    if helper == ally || h.side != a.side || !a.deployed || a.hp == 0 {
        return Err(error("Choose an active ally"));
    }
    if !h.deployed || h.hp == 0 || h.actions >= 5 {
        return Err(error("The helper has no action available"));
    }
    Ok(())
}
/// Help Another: the helper rolls the ally's intended move's Accuracy pool as Chance dice.
pub fn help_dice(
    c: &Campaign,
    helper: &str,
    ally: &str,
    move_name: &str,
    cat: &Catalog,
) -> Result<u32> {
    if op(c, "Help Another") != Some("help-another") {
        return Err(error("Help Another requires the MOVE-A53 rules"));
    }
    ally_of(c, helper, ally)?;
    let ally_sheet = rules::effective_sheet(c, fighter(c, ally)?)?;
    if !ally_sheet.moves.iter().any(|m| m == move_name) {
        return Err(error("Choose a move the ally knows"));
    }
    let (_, helpers) = flow(c)?
        .completion
        .help_dice
        .get(ally)
        .copied()
        .unwrap_or((0, 0));
    if helpers >= 3 {
        return Err(error("Up to 3 characters may help a single action"));
    }
    let m = move_entry(cat, move_name)?;
    let s = rules::effective_sheet(c, fighter(c, helper)?)?;
    let mut pool = 0;
    for key in ["Accuracy1", "Accuracy2"] {
        if let Some(source) = m[key].as_str().filter(|v| !v.is_empty()) {
            pool += rules::score(&s, source)?;
        }
    }
    u32::try_from(pool)
        .ok()
        .filter(|n| (1..=100).contains(n))
        .ok_or_else(|| error("The helper has no Accuracy pool for that move"))
}
pub fn help(c: &mut Campaign, helper: &str, ally: &str, dice: &[u8]) -> Result<u32> {
    let sixes = dice.iter().filter(|d| **d == 6).count() as u32;
    let entry = flow_mut(c)
        .completion
        .help_dice
        .entry(ally.into())
        .or_insert((0, 0));
    let added = sixes.min(6 - entry.0);
    entry.0 += added;
    entry.1 += 1;
    fighter_mut(c, helper)?.actions += 1;
    Ok(added)
}
/// Beat Up and Round: an ally joins the pending cast.
pub fn join(c: &mut Campaign, ally: &str, sequence: u32) -> Result<String> {
    let q = current(c)?.clone();
    if q.sequence != sequence || q.damage_done {
        return Err(error("Join the current cast before its damage"));
    }
    // The joining ally needs an action; the caster must still be active.
    ally_of(c, ally, &q.actor_id)?;
    match op(c, &q.move_name) {
        Some("beat-up") => {
            let allies = flow(c)?
                .completion
                .beat_up
                .get(&sequence)
                .cloned()
                .unwrap_or_default();
            if allies.len() >= 3 || allies.iter().any(|a| a == ally) {
                return Err(error("Up to 3 different allies may join Beat Up"));
            }
            flow_mut(c)
                .completion
                .beat_up
                .entry(sequence)
                .or_default()
                .push(ally.into());
            fighter_mut(c, ally)?.actions += 1;
            Ok("joins Beat Up (+2 dice)".into())
        }
        Some("round") => {
            if !rules::effective_sheet(c, fighter(c, ally)?)?
                .moves
                .iter()
                .any(|m| m == "Round")
            {
                return Err(error("The ally must be able to use Round"));
            }
            let now = round(c);
            let f = flow_mut(c);
            f.completion.round_chorus.insert(q.actor_id.clone(), now);
            f.completion.round_chorus.insert(ally.into(), now);
            Ok("sings Round together (+3 dice to both Rounds this round)".into())
        }
        _ => Err(error("Only Beat Up and Round can be joined")),
    }
}
pub fn lifecycle(c: &mut Campaign) -> Result<()> {
    let now = round(c);
    let queued: BTreeSet<u32> = flow(c)?.queue.iter().map(|q| q.sequence).collect();
    let f = flow_mut(c);
    f.completion.beat_up.retain(|s, _| queued.contains(s));
    f.completion.round_chorus.retain(|_, r| *r == now);
    Ok(())
}

/// Help Another is rolled through its own command, not as an ordinary move.
pub fn guard(c: &Campaign, m: &serde_json::Value) -> Result<()> {
    if op(c, m["Name"].as_str().unwrap_or("")) == Some("help-another") {
        return Err(error(
            "Use the Help Another control: roll your Accuracy for the ally's intended move",
        ));
    }
    Ok(())
}
