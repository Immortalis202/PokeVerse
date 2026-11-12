//! MOVE-A49 typing, accuracy and targeting cards.
use super::*;

const TYPES: [&str; 18] = [
    "Normal", "Fire", "Water", "Electric", "Grass", "Ice", "Fight", "Poison", "Ground", "Flying",
    "Psychic", "Bug", "Rock", "Ghost", "Dragon", "Dark", "Steel", "Fairy",
];

fn op(c: &Campaign, name: &str) -> Option<&'static str> {
    completion::definition(c, name)
        .filter(|(kind, _)| *kind == "type-op")
        .and_then(|(_, d)| d["op"].as_str())
}
fn has(c: &Campaign, name: &str, source: Option<&str>, target: &str) -> bool {
    flow(c).is_ok_and(|f| {
        f.special
            .effects
            .iter()
            .any(|e| e.name == name && e.target == target && source.is_none_or(|s| e.source == s))
    })
}

/// Moongeist Beam and Sunsteel Strike ignore whatever would prevent their damage.
pub fn unstoppable(c: &Campaign, m: &serde_json::Value) -> bool {
    op(c, m["Name"].as_str().unwrap_or("")) == Some("unstoppable")
}
/// Snipe Shot ignores redirection and cover.
pub fn snipe(c: &Campaign, m: &serde_json::Value) -> bool {
    op(c, m["Name"].as_str().unwrap_or("")) == Some("snipe")
}
pub fn miracle_eye(c: &Campaign, id: &str) -> bool {
    op(c, "Miracle Eye").is_some() && has(c, "Miracle Eye", None, id)
}
/// Automatic successes Lock-On and Mind Reader add to `actor`'s roll against `target`.
pub fn accuracy_gain(c: &Campaign, actor: &str, target: Option<&str>, is_move: bool) -> i32 {
    let lock = is_move && op(c, "Lock-On").is_some() && has(c, "Lock-On", None, actor);
    let mind = op(c, "Mind Reader").is_some()
        && target.is_some_and(|t| has(c, "Mind Reader", Some(actor), t));
    2 * i32::from(lock) + 2 * i32::from(mind)
}
/// Spend Lock-On on the next move roll and Mind Reader on a roll against its target.
pub fn consume_gain(c: &mut Campaign, actor: &str, targets: &[String], is_move: bool) {
    if let Some(f) = c.battle.as_mut().and_then(|b| b.flow.as_mut()) {
        f.special.effects.retain(|e| {
            !((is_move && e.name == "Lock-On" && e.target == actor)
                || (e.name == "Mind Reader"
                    && e.source == actor
                    && targets.iter().any(|t| *t == e.target)))
        });
    }
}
/// With Miracle Eye, only printed Low Accuracy and field effects can lower the user's roll.
pub fn accuracy_floor(c: &Campaign, p: &Combatant, m: &serde_json::Value, low: i32) -> i32 {
    if !miracle_eye(c, &p.id) {
        return low;
    }
    let printed = m["Attributes"]["AccuracyReduction"]
        .as_i64()
        .unwrap_or(0)
        .min(0) as i32
        - field::low_accuracy(c, p, m);
    low.max(items::reduce_low_accuracy(c, &p.id, printed))
}

/// Combatant ids Reflect Type can copy: active foes of the caster.
fn foes(c: &Campaign, actor: &str) -> Result<Vec<String>> {
    let side = fighter(c, actor)?.side.clone();
    Ok(battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0 && p.side != side)
        .map(|p| p.id.clone())
        .collect())
}
pub fn options(c: &Campaign, q: &QueuedAction) -> Result<Vec<String>> {
    if op(c, &q.move_name) != Some("reflect-type") {
        return Ok(vec![]);
    }
    let foes = foes(c, &q.actor_id)?;
    Ok(if foes.len() > 1 { foes } else { vec![] })
}
pub fn effect_dice(c: &Campaign, q: &QueuedAction) -> Option<u32> {
    let op = op(c, &q.move_name)?;
    Some(if op == "conversion" && !q.ice { 2 } else { 0 })
}
fn mark(c: &mut Campaign, name: &str, source: &str, target: &str) -> Result<()> {
    let started = battle(c)?.round;
    let f = flow_mut(c);
    f.special
        .effects
        .retain(|e| e.name != name || e.target != target || e.source != source);
    f.special.effects.push(special::TimedEffect {
        name: name.into(),
        source: source.into(),
        target: target.into(),
        rounds: None,
        amount: 0,
        started,
        slot: "type-op".into(),
    });
    Ok(())
}
fn set_types(c: &mut Campaign, id: &str, types: &[String]) -> Result<()> {
    field::change_type(c, id, "replace", types)?;
    let scope = format!("combatant:{id}");
    for r in &mut flow_mut(c).battlefield.rules {
        if r.scope == scope && r.kind.starts_with("typing:") && r.source == "typing" {
            r.source = "completion-type".into();
        }
    }
    Ok(())
}
/// Marks end when their source or target leaves battle.
pub fn lifecycle(c: &mut Campaign) -> Result<()> {
    let active: BTreeSet<String> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    flow_mut(c).special.effects.retain(|e| {
        e.slot != "type-op" || (active.contains(&e.source) && active.contains(&e.target))
    });
    Ok(())
}

pub fn effect(c: &mut Campaign, q: &QueuedAction, dice: &[u8], cat: &Catalog) -> Result<()> {
    if q.ice {
        return Ok(());
    }
    let (a, t) = (q.actor_id.clone(), q.target_id.clone());
    match op(c, &q.move_name).unwrap_or("") {
        "conversion" => {
            let value = (dice[0] as usize - 1) * 6 + (dice[1] as usize - 1);
            let main = TYPES[value % TYPES.len()].to_string();
            let current = field::types(c, fighter(c, &a)?, cat)?;
            let mut types = vec![main.clone()];
            if let Some(second) = current.get(1).filter(|s| **s != main) {
                types.push(second.clone());
            }
            set_types(c, &a, &types)?;
        }
        "reflect-type" => {
            let foes = foes(c, &a)?;
            let foe = match foes.len() {
                0 => return Ok(()),
                1 => foes[0].clone(),
                _ => {
                    let choice = flow(c)?
                        .completion
                        .move_options
                        .get(&q.sequence)
                        .cloned()
                        .ok_or_else(|| error("Choose which foe's types Reflect Type copies"))?;
                    if !foes.contains(&choice) {
                        return Err(error("The saved foe is no longer active"));
                    }
                    choice
                }
            };
            let types = field::types(c, fighter(c, &foe)?, cat)?;
            set_types(c, &a, &types)?;
        }
        "lock-on" => mark(c, "Lock-On", &a, &a)?,
        "mind-reader" => {
            if !completion::immune_target(c, q, cat)? {
                mark(c, "Mind Reader", &a, &t)?;
            }
        }
        "miracle-eye" => mark(c, "Miracle Eye", &a, &a)?,
        // Damage-only cards: their rules apply during targeting and damage.
        "snipe" | "unstoppable" => {}
        _ => return Err(error("Unknown typing or accuracy operation")),
    }
    flow_mut(c).completion.move_options.remove(&q.sequence);
    Ok(())
}
