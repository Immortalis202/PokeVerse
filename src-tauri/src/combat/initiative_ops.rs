//! MOVE-A52 initiative, reaction and maneuver cards.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct InitiativeMark {
    /// "first" (After You) or "last" (Quash).
    pub rank: String,
    pub caster: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct AmbushState {
    pub target: String,
    /// The surprise action has been declared.
    pub used: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct LoggedMove {
    pub round: u32,
    pub name: String,
    pub targets: Vec<String>,
}

fn op(c: &Campaign, name: &str) -> Option<&'static str> {
    completion::definition(c, name)
        .filter(|(kind, _)| *kind == "initiative-op")
        .and_then(|(_, d)| d["op"].as_str())
}
fn round(c: &Campaign) -> u32 {
    battle(c).map_or(0, |b| b.round)
}
fn active_ids(c: &Campaign) -> BTreeSet<String> {
    battle(c).map_or_else(
        |_| BTreeSet::new(),
        |b| {
            b.combatants
                .iter()
                .filter(|p| p.deployed && p.hp > 0)
                .map(|p| p.id.clone())
                .collect()
        },
    )
}

// ----- Initiative order -----

/// 0 for After You targets, 2 for Quash targets, 1 otherwise.
pub fn rank(c: &Campaign, id: &str) -> u8 {
    match flow(c)
        .ok()
        .and_then(|f| f.completion.initiative.get(id))
        .map(|m| m.rank.as_str())
    {
        Some("first") => 0,
        Some("last") => 2,
        _ => 1,
    }
}
pub fn order(c: &Campaign, order: &mut [&Combatant]) {
    order.sort_by_key(|p| rank(c, &p.id));
}

// ----- Turn exceptions and guards before a move is declared -----

/// Ambush and the surprise action it grants happen before the first round's turns.
pub fn pre_start(c: &Campaign, id: &str, name: &str) -> bool {
    let Ok(f) = flow(c) else { return false };
    if f.started || f.turn != 0 || !f.queue.is_empty() {
        return false;
    }
    (op(c, name) == Some("ambush") && !f.completion.ambush.contains_key(id))
        || f.completion
            .ambush
            .get(id)
            .is_some_and(|a| !a.used && name != "Ambush")
}
/// Grappling users cannot act; reaction-only cards cannot be declared as actions.
pub fn action_guard(c: &Campaign, id: &str, name: &str) -> Result<()> {
    if grappling(c, id) {
        return Err(error("Release the Grapple before acting"));
    }
    if op(c, name) == Some("reaction-only") {
        return Err(error(
            "This move can only be used as a Reaction against a Physical or Special move",
        ));
    }
    if op(c, name) == Some("encore")
        && flow(c)?
            .completion
            .encore_casts
            .get(id)
            .is_some_and(|r| *r + 1 == round(c))
    {
        return Err(error("Encore fails if it was used in the previous round"));
    }
    Ok(())
}
/// Reaction checks: grappling users, Ambush victims, reaction-only cards and evasion limits.
pub fn reaction_guard(
    c: &Campaign,
    id: &str,
    kind: &str,
    name: &str,
    incoming: &QueuedAction,
    incoming_move: &serde_json::Value,
) -> Result<()> {
    if completion::definition(c, "Grapple").is_none() {
        return Ok(());
    }
    if grappling(c, id) {
        return Err(error("Release the Grapple before reacting"));
    }
    if !flow(c)?.started
        && flow(c)?
            .completion
            .ambush
            .get(&incoming.actor_id)
            .is_some_and(|a| a.target == id)
    {
        return Err(error(
            "The Ambushed target cannot act during the surprise action",
        ));
    }
    if kind == "move"
        && op(c, name) == Some("reaction-only")
        && !["Physical", "Special"].contains(&incoming_move["Category"].as_str().unwrap_or(""))
    {
        return Err(error(
            "This move fails unless it reacts against a Physical or Special move",
        ));
    }
    if kind == "evade" && evasion_actions(c, id) {
        let (r, n) = flow(c)?
            .completion
            .evasions
            .get(id)
            .copied()
            .unwrap_or((0, 0));
        if r == round(c) && n >= 5 {
            return Err(error("Up to five Evasions per round"));
        }
    }
    Ok(())
}
/// Double Team and Minimize lift the one-Evasion-per-turn slot.
pub fn extra_evasion(c: &Campaign, id: &str, kind: &str) -> bool {
    kind == "evade" && evasion_actions(c, id)
}
fn evasion_actions(c: &Campaign, id: &str) -> bool {
    flow(c).is_ok_and(|f| {
        f.special
            .effects
            .iter()
            .any(|e| e.target == id && e.slot == "evasion-actions")
    })
}
pub fn evaded(c: &mut Campaign, id: &str, kind: &str) {
    if kind != "evade" || completion::definition(c, "Double Team").is_none() {
        return;
    }
    let now = round(c);
    let entry = flow_mut(c)
        .completion
        .evasions
        .entry(id.into())
        .or_insert((now, 0));
    if entry.0 != now {
        *entry = (now, 0);
    }
    entry.1 += 1;
}
fn grappling(c: &Campaign, id: &str) -> bool {
    flow(c).is_ok_and(|f| {
        f.special
            .effects
            .iter()
            .any(|e| e.source == id && e.slot == "grapple")
    })
}
pub fn release(c: &mut Campaign, id: &str) -> Result<()> {
    if !grappling(c, id) {
        return Err(error("This combatant is not Grappling"));
    }
    flow_mut(c)
        .special
        .effects
        .retain(|e| e.source != id || e.slot != "grapple");
    Ok(())
}
pub fn minimized(c: &Campaign, id: &str) -> bool {
    flow(c).is_ok_and(|f| {
        f.special
            .effects
            .iter()
            .any(|e| e.target == id && e.name == "Minimize" && e.slot == "minimize")
    })
}

// ----- Encore -----

fn log(c: &Campaign, id: &str, round: u32) -> Vec<LoggedMove> {
    flow(c)
        .ok()
        .and_then(|f| f.completion.move_log.get(id))
        .map(|v| v.iter().filter(|m| m.round == round).cloned().collect())
        .unwrap_or_default()
}
/// An Encored combatant repeats its previous round's moves, in order and on the same targets.
pub fn encore_guard(c: &Campaign, id: &str, name: &str, targets: &[String]) -> Result<()> {
    let now = round(c);
    if flow(c)?.completion.encored.get(id) != Some(&now) {
        return Ok(());
    }
    let sequence = log(c, id, now.saturating_sub(1));
    let k = log(c, id, now).len();
    let Some(expected) = sequence.get(k) else {
        return Err(error("Encore: the repeated action sequence is depleted"));
    };
    if expected.name != name {
        return Err(error(&format!(
            "Encore: this action must repeat {}",
            expected.name
        )));
    }
    let active = active_ids(c);
    if expected.targets.iter().all(|t| active.contains(t)) && expected.targets != targets {
        return Err(error(
            "Encore: repeat the same targets while they remain in battle",
        ));
    }
    Ok(())
}
pub fn log_move(c: &mut Campaign, id: &str, name: &str, targets: &[String]) {
    if completion::definition(c, "Encore").is_none() {
        return;
    }
    let now = round(c);
    let started = flow(c).is_ok_and(|f| f.started);
    if !started && name != "Ambush" {
        if let Some(a) = flow_mut(c).completion.ambush.get_mut(id) {
            a.used = true;
        }
    }
    let entries = flow_mut(c)
        .completion
        .move_log
        .entry(id.into())
        .or_default();
    entries.retain(|m| m.round + 1 >= now);
    entries.push(LoggedMove {
        round: now,
        name: name.into(),
        targets: targets.to_vec(),
    });
}

// ----- Timing choice for Sucker Punch and Thunderclap -----

pub fn move_data(c: &Campaign, p: &Combatant, m: &mut serde_json::Value) {
    let name = m["Name"].as_str().unwrap_or("").to_string();
    if op(c, &name) != Some("reaction-only") {
        return;
    }
    let late = flow(c).is_ok_and(|f| {
        f.completion
            .reaction_timing
            .get(&format!("{}:{name}", p.id))
            .is_some_and(|t| t == "late")
    });
    let attrs = m["Attributes"].as_object_mut();
    if let Some(attrs) = attrs {
        if late {
            attrs.remove("Reaction");
            attrs.insert("LateReaction".into(), "1".into());
        } else {
            attrs.remove("LateReaction");
            attrs.insert("Reaction".into(), "6".into());
        }
    }
}
pub fn set_timing(c: &mut Campaign, id: &str, name: &str, late: bool) -> Result<()> {
    if op(c, name) != Some("reaction-only") {
        return Err(error(
            "Only Sucker Punch and Thunderclap choose their reaction timing",
        ));
    }
    flow_mut(c).completion.reaction_timing.insert(
        format!("{id}:{name}"),
        if late { "late" } else { "early" }.into(),
    );
    Ok(())
}

// ----- Effects -----

pub fn effect_dice(c: &Campaign, q: &QueuedAction) -> Result<Option<u32>> {
    let Some(op) = op(c, &q.move_name) else {
        return Ok(None);
    };
    if q.ice {
        return Ok(Some(0));
    }
    let target = fighter(c, &q.target_id)?;
    let s = rules::effective_sheet(c, target)?;
    Ok(Some(match op {
        "grapple" => rules::score(&s, "Strength")?.max(rules::score(&s, "Dexterity")?) as u32,
        "ambush" => (rules::score(&s, "Insight")? + rules::score(&s, "Alert")?).max(0) as u32,
        _ => 0,
    }))
}
fn mark(c: &mut Campaign, name: &str, source: &str, target: &str, slot: &str) -> Result<()> {
    let started = round(c);
    let f = flow_mut(c);
    f.special
        .effects
        .retain(|e| e.name != name || e.target != target || e.slot != slot);
    f.special.effects.push(special::TimedEffect {
        name: name.into(),
        source: source.into(),
        target: target.into(),
        rounds: None,
        amount: 0,
        started,
        slot: slot.into(),
    });
    Ok(())
}
pub fn effect(c: &mut Campaign, q: &QueuedAction, dice: &[u8], cat: &Catalog) -> Result<()> {
    if q.ice {
        return Ok(());
    }
    let (a, t) = (q.actor_id.clone(), q.target_id.clone());
    let resisted = rules::successes(dice) >= q.successes;
    match op(c, &q.move_name).unwrap_or("") {
        rank @ ("first" | "last") => {
            if !completion::immune_target(c, q, cat)? {
                let f = flow_mut(c);
                f.completion
                    .initiative
                    .retain(|_, m| !(m.caster == a && m.rank == rank));
                f.completion.initiative.insert(
                    t,
                    InitiativeMark {
                        rank: rank.into(),
                        caster: a,
                    },
                );
            }
        }
        "ambush" => {
            if !completion::immune_target(c, q, cat)? && !resisted {
                flow_mut(c).completion.ambush.insert(
                    a,
                    AmbushState {
                        target: t,
                        used: false,
                    },
                );
            }
        }
        "encore" => {
            let now = round(c);
            flow_mut(c).completion.encore_casts.insert(a, now);
            if !completion::immune_target(c, q, cat)?
                && !log(c, &t, now.saturating_sub(1)).is_empty()
            {
                flow_mut(c).completion.encored.insert(t, now);
            }
        }
        "grapple" => {
            if !completion::immune_target(c, q, cat)? && !resisted {
                mark(c, "move-block:Grapple", &a, &t, "grapple")?;
            }
        }
        "evasions" => {
            mark(c, &q.move_name, &a, &a, "evasion-actions")?;
            if q.move_name == "Minimize" {
                mark(c, "Minimize", &a, &a, "minimize")?;
                compound::mark_minimized(c, &a);
            }
        }
        "chip-away" => {
            if q.damage > 0 {
                flow_mut(c)
                    .special
                    .effects
                    .retain(|e| e.target != t || e.slot != "evasion-actions");
            }
        }
        "reaction-only" => {}
        _ => return Err(error("Unknown initiative or maneuver operation")),
    }
    Ok(())
}

/// Start of turns: the Ambush window closes; its pre-initiative actions are refunded.
pub fn start_turns(c: &mut Campaign) -> Result<()> {
    let ambushers: Vec<String> = flow(c)?.completion.ambush.keys().cloned().collect();
    for id in ambushers {
        fighter_mut(c, &id)?.actions = 0;
    }
    flow_mut(c).completion.ambush.clear();
    Ok(())
}
pub fn lifecycle(c: &mut Campaign) -> Result<()> {
    let active = active_ids(c);
    let f = flow_mut(c);
    // Quash persists across switching; After You ends when its target leaves.
    f.completion
        .initiative
        .retain(|id, m| m.rank == "last" || active.contains(id));
    f.special.effects.retain(|e| {
        !["grapple", "evasion-actions", "minimize"].contains(&e.slot.as_str())
            || (active.contains(&e.source) && active.contains(&e.target))
    });
    Ok(())
}
