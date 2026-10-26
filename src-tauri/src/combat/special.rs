//! Pokerole special-move state. All authoritative decisions remain on the host.
use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct SpecialState {
    pub enabled: bool,
    #[serde(default)]
    pub bides: BTreeMap<String, Bide>,
    pub charges: BTreeMap<String, Charge>,
    pub rampages: BTreeMap<String, Rampage>,
    pub shields: BTreeMap<String, Shield>,
    pub effects: Vec<TimedEffect>,
    pub history: Vec<Hit>,
    pub parameters: BTreeMap<String, Parameters>,
    pub selections: BTreeMap<u32, Selection>,
    pub random: BTreeMap<u32, i32>,
    pub series: Option<Series>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Charge {
    pub action: QueuedAction,
    pub remaining: u32,
    pub turn: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Bide {
    pub target_id: String,
    pub basis_id: String,
    pub received: u32,
    pub damage: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Rampage {
    pub name: String,
    pub uses: u32,
    pub round: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Shield {
    pub name: String,
    pub source: String,
    pub round: u32,
    pub armed_turn: u32,
    pub available: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct TimedEffect {
    #[serde(default)]
    pub amount: i32,
    pub name: String,
    pub source: String,
    pub target: String,
    pub rounds: Option<u32>,
    pub started: u32,
    pub slot: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Hit {
    pub source: String,
    pub target: String,
    pub name: String,
    pub category: String,
    pub dice: u32,
    pub damage: i32,
    pub round: u32,
    pub turn: u32,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Parameters {
    #[serde(default)]
    pub lift_kg: Option<f64>,
    pub weight_kg: Option<f64>,
    pub night: bool,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Selection {
    pub recipient_id: String,
    pub secondary_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Series {
    #[serde(default)]
    pub follow_up: Vec<QueuedAction>,
    pub actor_id: String,
    pub target_id: String,
    pub name: String,
    pub attempts: u32,
    pub start_actions: u32,
    pub preparing: bool,
    pub pending: Vec<QueuedAction>,
    pub landed: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export, export_to = "../../src/bindings/")]
pub enum SpecialCommand {
    Release {
        combatant_id: String,
    },
    CancelCharge {
        combatant_id: String,
    },
    EndRampage {
        combatant_id: String,
    },
    Successive {
        #[serde(default)]
        reaction: bool,
        combatant_id: String,
        target_id: String,
        move_name: String,
        attempts: u32,
    },
    Select {
        sequence: u32,
        recipient_id: String,
        secondary_id: String,
    },
    Randomize {
        sequence: u32,
    },
    Parameters {
        combatant_id: String,
        weight_kg: Option<f64>,
        #[serde(default)]
        lift_kg: Option<f64>,
        night: bool,
    },
    Tick {
        index: u32,
    },
}
pub fn enabled(c: &Campaign) -> bool {
    c.battle
        .as_ref()
        .and_then(|b| b.flow.as_ref())
        .is_some_and(|f| f.special.enabled)
}
fn state(c: &Campaign) -> &SpecialState {
    &flow(c).unwrap().special
}
fn state_mut(c: &mut Campaign) -> &mut SpecialState {
    &mut flow_mut(c).special
}
pub fn preparing(c: &Campaign) -> bool {
    enabled(c) && state(c).series.as_ref().is_some_and(|s| s.preparing)
}
pub fn repeat(c: &Campaign, id: &str, name: &str) -> bool {
    enabled(c)
        && (state(c)
            .series
            .as_ref()
            .is_some_and(|s| s.preparing && s.actor_id == id && s.name == name)
            || state(c).rampages.get(id).is_some_and(|r| {
                r.name == name && r.uses < 3 && r.round == battle(c).unwrap().round
            })
            || name == "Echoed Voice"
            || (name == "Bide" && state(c).bides.get(id).is_some_and(|b| b.received >= 2)))
}
pub fn limit(m: &serde_json::Value) -> u32 {
    if m["Attributes"]["TripleAction"] == true {
        3
    } else if m["Attributes"]["DoubleAction"] == true {
        2
    } else if m["Attributes"]["SuccessiveActions"] == true {
        5
    } else {
        1
    }
}
fn accuracy_action(id: &str, target: &str, name: &str) -> Action {
    Action::Accuracy {
        combatant_id: id.into(),
        target_id: Some(target.into()),
        move_name: name.into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Successive accuracy".into(),
    }
}
pub fn dice(c: &Campaign, cmd: &SpecialCommand, cat: &Catalog) -> Result<u32> {
    if !enabled(c) {
        return Err(error("Upgrade special-move rules between battles"));
    }
    Ok(match cmd {
        SpecialCommand::Successive {
            combatant_id,
            target_id,
            move_name,
            attempts,
            reaction,
        } => {
            let p = fighter(c, combatant_id)?;
            let m = move_entry(cat, move_name)?;
            if *attempts < 1 || *attempts > limit(&m) || limit(&m) == 1 || p.actions + attempts > 5
            {
                return Err(error(
                    "Choose a legal number of successive actions within the five-action budget",
                ));
            }
            if state(c)
                .effects
                .iter()
                .any(|e| e.name == "Roar Of Time" && e.target == *combatant_id)
                && p.runtime.will < *attempts as i32
            {
                return Err(error(
                    "Roar Of Time requires one Will per committed successive action",
                ));
            }
            let base = if *reaction {
                let incoming = incoming(c)?;
                if incoming.actor_id != *target_id {
                    return Err(error("A successive reaction targets the incoming attacker"));
                }
                reaction_pool(c, combatant_id, "move", move_name, cat)?
            } else {
                turn(c, combatant_id)?;
                ready(c, combatant_id)?;
                rules::accuracy(&old(c), combatant_id, move_name, 0, cat)?.0
            };
            base.checked_add(sequencing::successive_loyalty(c, combatant_id)?)
                .and_then(|n| {
                    n.checked_add(u32::from(
                        p.runtime.chance_round == battle(c).unwrap().round,
                    ))
                })
                .and_then(|n| n.checked_mul(*attempts))
                .ok_or_else(|| error("Successive pool overflow"))?
        }
        SpecialCommand::Randomize { sequence } => {
            let q = current(c)?;
            if q.sequence != *sequence
                || !["Magnitude", "Present"].contains(&q.move_name.as_str())
                || state(c).random.contains_key(sequence)
            {
                return Err(error("No unresolved random move power"));
            }
            1
        }
        SpecialCommand::Tick { index } => {
            if !flow(c)?.queue.is_empty()
                || !flow(c)?.order.is_empty()
                || flow(c)?.battlefield.end_round != battle(c)?.round
            {
                return Err(error(
                    "Ongoing move rolls are resolved during the recorded round-end phase",
                ));
            }
            let e = state(c)
                .effects
                .get(*index as usize)
                .ok_or_else(|| error("Effect changed"))?;
            if e.started >= battle(c)?.round || !["Leech Seed"].contains(&e.name.as_str()) {
                return Err(error("No pending ongoing roll"));
            }
            2
        }
        _ => 0,
    })
}
pub fn command(
    c: &mut Campaign,
    actor: &str,
    cmd: &SpecialCommand,
    faces: &[u8],
    r: &str,
    cat: &Catalog,
) -> Result<String> {
    reason(r)?;
    check_count(faces, dice(c, cmd, cat)?)?;
    if abilities::pending(c) || field::pending(c) {
        return Err(error(
            "Resolve pending ability and battlefield events first",
        ));
    }
    match cmd {
        SpecialCommand::Parameters {
            combatant_id,
            weight_kg,
            lift_kg,
            night,
        } => {
            gm(c, actor)?;
            fighter(c, combatant_id)?;
            if lift_kg.is_some_and(|n| !n.is_finite() || n <= 0. || n > 100000.) {
                return Err(error("Lifting capacity must be finite and positive in kg"));
            }
            if weight_kg.is_some_and(|n| !n.is_finite() || n <= 0. || n > 100000.) {
                return Err(error("Weight must be a finite positive value in kg"));
            }
            state_mut(c).parameters.insert(
                combatant_id.clone(),
                Parameters {
                    lift_kg: *lift_kg,
                    weight_kg: *weight_kg,
                    night: *night,
                },
            );
        }
        SpecialCommand::CancelCharge { combatant_id } => {
            owner(c, actor, combatant_id)?;
            if !flow(c)?.queue.is_empty() {
                return Err(error("Resolve the attack first"));
            }
            state_mut(c)
                .charges
                .remove(combatant_id)
                .ok_or_else(|| error("No charged move"))?;
        }
        SpecialCommand::EndRampage { combatant_id } => {
            owner(c, actor, combatant_id)?;
            if !flow(c)?.queue.is_empty() {
                return Err(error("Resolve the attack first"));
            }
            end_rampage(c, combatant_id, cat)?;
        }
        SpecialCommand::Release { combatant_id } => {
            owner(c, actor, combatant_id)?;
            turn(c, combatant_id)?;
            ready(c, combatant_id)?;
            let mut charge = state(c)
                .charges
                .get(combatant_id)
                .cloned()
                .ok_or_else(|| error("No charged move"))?;
            if flow(c)?.turn <= charge.turn {
                return Err(error("A charged move requires the user's next turn"));
            }
            fighter_mut(c, combatant_id)?.actions += 1;
            if charge.remaining > 0 {
                charge.remaining -= 1;
                charge.turn = flow(c)?.turn;
                state_mut(c).charges.insert(combatant_id.clone(), charge);
                advance(c);
            } else {
                let q = &charge.action;
                let m = move_entry(cat, &q.move_name)?;
                let ids = if q.targets.is_empty() {
                    if q.target_id == q.actor_id && q.field_scope.is_some() {
                        vec![]
                    } else {
                        vec![q.target_id.clone()]
                    }
                } else {
                    q.targets.iter().map(|t| t.target_id.clone()).collect()
                };
                state_mut(c).charges.remove(combatant_id);
                plan(c, combatant_id, &ids, &m)?;
                let target = charge.action.target_id.clone();
                flow_mut(c).queue.push(charge.action);
                flow_mut(c).window = target != *combatant_id;
                prepare_targets(c);
            }
        }
        SpecialCommand::Successive {
            combatant_id,
            target_id,
            move_name,
            attempts,
            reaction,
        } => {
            owner(c, actor, combatant_id)?;
            if *reaction {
                return reaction_series(
                    c,
                    actor,
                    combatant_id,
                    target_id,
                    move_name,
                    *attempts,
                    faces,
                    r,
                    cat,
                );
            }
            turn(c, combatant_id)?;
            ready(c, combatant_id)?;
            if state(c).series.is_some() {
                return Err(error("Finish the existing successive sequence"));
            }
            let navigation = flow(c)?.clone();
            let undo = c.battle_undo.clone();
            let start_actions = fighter(c, combatant_id)?.actions;
            state_mut(c).series = Some(Series {
                follow_up: vec![],
                actor_id: combatant_id.clone(),
                target_id: target_id.clone(),
                name: move_name.clone(),
                attempts: *attempts,
                start_actions,
                preparing: true,
                pending: vec![],
                landed: 0,
            });
            let mut offset = 0;
            let mut results = vec![];
            let mut pending = vec![];
            for i in 0..*attempts {
                let a = accuracy_action(combatant_id, target_id, move_name);
                let n = super::dice_count(c, &a, cat)? as usize;
                let (mut next, log) =
                    super::transition(c, actor, &a, faces[offset..offset + n].to_vec(), cat)?;
                offset += n;
                results.push(log);
                if let Some(mut q) = flow_mut(&mut next).queue.pop() {
                    q.special_index = i + 1;
                    q.sequence = c.revision.wrapping_add(1 + i);
                    pending.push(q);
                }
                flow_mut(&mut next).window = false;
                *c = next;
                if fighter(c, combatant_id)?.hp == 0 {
                    fighter_mut(c, combatant_id)?.actions = start_actions + attempts;
                    break;
                }
            }
            let s = state_mut(c).series.as_mut().unwrap();
            s.preparing = false;
            s.pending = pending;
            let f = flow_mut(c);
            f.order = navigation.order;
            f.cursor = navigation.cursor;
            f.turn = navigation.turn;
            f.reacted = navigation.reacted;
            f.held = navigation.held;
            f.deferred = navigation.deferred;
            c.battle_undo = undo;
            if !continue_series(c) {
                advance(c);
            }
            return Ok(format!(
                "Successive {move_name}, {attempts} committed actions: {}. {r}",
                results.join("; ")
            ));
        }
        SpecialCommand::Select {
            sequence,
            recipient_id,
            secondary_id,
        } => {
            let q = current(c)?.clone();
            if q.sequence != *sequence {
                return Err(error("Pending action changed"));
            }
            owner(c, actor, &q.actor_id)?;
            if ![
                "Healing Wish",
                "Lunar Dance",
                "Lunar Blessing",
                "Parabolic Charge",
                "Purify",
                "Bide",
            ]
            .contains(&q.move_name.as_str())
            {
                return Err(error("This move has no additional owner choice"));
            }
            if !recipient_id.is_empty() {
                let t = registered(c, recipient_id)?;
                if t.side != fighter(c, &q.actor_id)?.side {
                    return Err(error("Choose an allied recipient"));
                }
            }
            if !secondary_id.is_empty() {
                registered(c, secondary_id)?;
            }
            if q.move_name == "Bide" && !house::enabled(c) {
                gm(c, actor)?;
                if secondary_id != &q.actor_id && secondary_id != &q.target_id {
                    return Err(error("Bide's ambiguous wording requires a GM-recorded User or Target damage basis"));
                }
            }
            state_mut(c).selections.insert(
                *sequence,
                Selection {
                    recipient_id: recipient_id.clone(),
                    secondary_id: secondary_id.clone(),
                },
            );
        }
        SpecialCommand::Randomize { sequence } => {
            let q = current(c)?;
            owner(c, actor, &q.actor_id)?;
            state_mut(c).random.insert(*sequence, faces[0] as i32);
        }
        SpecialCommand::Tick { index } => {
            let e = state(c).effects[*index as usize].clone();
            own_registered(c, actor, &e.source)?;
            let n = rules::successes(faces);
            let before = fighter(c, &e.target)?.hp;
            indirect(c, &e.target, n)?;
            let dealt = before - fighter(c, &e.target)?.hp;
            let ooze = fighter(c, &e.target)?.has_ability("Liquid Ooze");
            if ooze {
                indirect(c, &e.source, dealt)?;
            } else {
                heal(c, &e.source, dealt, false)?;
            }
            let round = battle(c)?.round;
            state_mut(c).effects[*index as usize].started = round;
        }
    }
    Ok(format!("Special move {cmd:?}. {r}"))
}
pub fn continue_series(c: &mut Campaign) -> bool {
    if !enabled(c) {
        return false;
    }
    let Some(s) = state(c).series.clone() else {
        return false;
    };
    if s.preparing {
        return true;
    }
    if fighter(c, &s.actor_id).is_ok_and(|p| p.hp > 0)
        && fighter(c, &s.target_id).is_ok_and(|p| p.hp > 0 && p.deployed)
        && !s.pending.is_empty()
    {
        let q = state_mut(c).series.as_mut().unwrap().pending.remove(0);
        flow_mut(c).window = q.actor_id != q.target_id;
        flow_mut(c).queue.push(q);
        return true;
    }
    state_mut(c).series = None;
    if !s.follow_up.is_empty() {
        flow_mut(c).queue = s.follow_up;
        let live: BTreeSet<_> = battle(c)
            .unwrap()
            .combatants
            .iter()
            .filter(|p| p.deployed && p.hp > 0)
            .map(|p| p.id.clone())
            .collect();
        for q in &mut flow_mut(c).queue {
            if !live.contains(&q.actor_id) || (q.targets.is_empty() && !live.contains(&q.target_id))
            {
                q.cancelled = true;
            }
            for t in &mut q.targets {
                if t.target_id == s.actor_id {
                    t.responded = true;
                }
                if !live.contains(&t.target_id) {
                    t.avoided = true;
                    t.responded = true;
                }
            }
        }
        close(c);
        return true;
    }
    false
}
#[allow(clippy::too_many_arguments)]
fn reaction_series(
    c: &mut Campaign,
    actor: &str,
    id: &str,
    target: &str,
    name: &str,
    attempts: u32,
    faces: &[u8],
    r: &str,
    cat: &Catalog,
) -> Result<String> {
    if state(c).series.is_some() {
        return Err(error("Finish the active successive sequence"));
    }
    let navigation = flow(c)?.clone();
    let undo = c.battle_undo.clone();
    let start_actions = fighter(c, id)?.actions;
    state_mut(c).series = Some(Series {
        follow_up: navigation.queue.clone(),
        actor_id: id.into(),
        target_id: target.into(),
        name: name.into(),
        attempts,
        start_actions,
        preparing: true,
        pending: vec![],
        landed: 0,
    });
    let (mut offset, mut pending, mut results) = (0usize, vec![], vec![]);
    for i in 0..attempts {
        flow_mut(c).queue = navigation.queue.clone();
        flow_mut(c).window = true;
        flow_mut(c).reacted.remove(id);
        let a = Action::React {
            combatant_id: id.into(),
            kind: "move".into(),
            move_name: name.into(),
            reason: "Successive reaction accuracy".into(),
        };
        let n = super::dice_count(c, &a, cat)? as usize;
        let (mut next, log) =
            super::transition(c, actor, &a, faces[offset..offset + n].to_vec(), cat)?;
        offset += n;
        results.push(log);
        if let Some(index) = flow(&next)?
            .queue
            .iter()
            .position(|q| q.actor_id == id && q.move_name == name)
        {
            let mut q = flow_mut(&mut next).queue.remove(index);
            q.special_index = i + 1;
            q.sequence = c.revision.wrapping_add(1 + i);
            pending.push(q);
        }
        *c = next;
        if fighter(c, id)?.hp == 0 {
            fighter_mut(c, id)?.actions = start_actions + attempts;
            break;
        }
    }
    let series = state_mut(c).series.as_mut().unwrap();
    series.pending = pending;
    series.preparing = false;
    let f = flow_mut(c);
    f.queue.clear();
    f.window = false;
    f.order = navigation.order;
    f.cursor = navigation.cursor;
    f.turn = navigation.turn;
    f.reacted = navigation.reacted;
    f.reacted.insert(id.into());
    f.held = navigation.held;
    f.deferred = navigation.deferred;
    c.battle_undo = undo;
    continue_series(c);
    Ok(format!(
        "Successive reaction {name}, {attempts} committed actions: {}. {r}",
        results.join("; ")
    ))
}
fn end_rampage(c: &mut Campaign, id: &str, cat: &Catalog) -> Result<()> {
    if state_mut(c).rampages.remove(id).is_some() && fighter(c, id)?.hp > 0 {
        abilities::inflict(c, id, id, "Confusion", "", None, cat)?;
    }
    Ok(())
}
pub fn accuracy_guard(c: &Campaign, id: &str, name: &str) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    if state(c).charges.contains_key(id) {
        return Err(error("Release or cancel the stored charged move"));
    }
    if state(c).rampages.get(id).is_some_and(|r| r.name != name) {
        return Err(error(
            "End the rampage and resolve Confusion before choosing another move",
        ));
    }
    Ok(())
}
pub fn ready_move(c: &Campaign, id: &str, name: &str) -> Result<()> {
    if enabled(c) && name == "Matcha Gotcha" {
        let mut next = c.clone();
        fighter_mut(&mut next, id)?
            .runtime
            .statuses
            .retain(|s| s.kind != "Frozen Solid");
        ready(&next, id)
    } else {
        ready(c, id)
    }
}
pub fn roost(c: &Campaign, id: &str) -> bool {
    enabled(c)
        && state(c)
            .effects
            .iter()
            .any(|e| e.name == "Roost" && e.target == id)
}
pub fn after_accuracy(
    c: &mut Campaign,
    id: &str,
    m: &serde_json::Value,
    success: bool,
    cat: &Catalog,
) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let name = m["Name"].as_str().unwrap_or("");
    if name == "Matcha Gotcha" {
        fighter_mut(c, id)?
            .runtime
            .statuses
            .retain(|s| s.kind != "Frozen Solid");
    }
    let counter = &mut fighter_mut(c, id)?.runtime.ability.counters;
    if name == "Echoed Voice" {
        let n = counter.entry("special:echo".into()).or_default();
        *n = (*n + 1).min(8);
    } else {
        counter.remove("special:echo");
    }
    if m["Attributes"]["Rampage"] == true {
        let round = battle(c)?.round;
        if success || state(c).rampages.contains_key(id) {
            let r = state_mut(c).rampages.entry(id.into()).or_insert(Rampage {
                name: name.into(),
                uses: 0,
                round,
            });
            r.uses += 1;
        }
    }
    if success && m["Attributes"]["Charge"] == true {
        let weather = field::weather(c, Some(fighter(c, id)?));
        let mut skip = (["Solar Beam", "Solar Blade"].contains(&name)
            && matches!(
                weather,
                Some(field::Weather::Sunny | field::Weather::Desolate)
            ))
            || (name == "Electro Shot"
                && matches!(
                    weather,
                    Some(field::Weather::Rain | field::Weather::Typhoon)
                ));
        if !skip && items::completion::power_herb(c, id, cat)? {
            skip = true;
        }
        if name == "Meteor Beam" || name == "Electro Shot" {
            extra::change_stat(c, id, id, "Special", 1)?;
        }
        if !skip {
            let q = flow_mut(c)
                .queue
                .pop()
                .ok_or_else(|| error("Missing charge accuracy"))?;
            if name == "Sky Drop" {
                let source = fighter(c, id)?.clone();
                let target = fighter(c, &q.target_id)?.clone();
                if field::types(c, &target, cat)?.contains(&"Flying".into()) {
                    return Err(error("Sky Drop fails against a Flying target"));
                }
                let sheet = rules::effective_sheet(c, &source)?;
                let override_lift = state(c).parameters.get(id).and_then(|p| p.lift_kg);
                let pain =
                    (rules::pain(source.hp, source.max_hp) - source.runtime.pain_ignored).max(0);
                if pain > 0 && override_lift.is_none() {
                    return Err(AppError::new(
                        "manual_rule",
                        "Sky Drop while in pain requires a recorded GM lifting capacity",
                    ));
                }
                let strength = sheet.stats["Strength"].clamp(1, 10) as usize;
                let capacity = override_lift.unwrap_or(
                    [18., 45., 113., 181., 294., 362., 408., 453., 544., 680.][strength - 1]
                        + 4. * rules::score(&sheet, "Athletic")? as f64,
                );
                if weight(c, &target, cat)? > capacity {
                    return Err(error(
                        "Sky Drop target exceeds the reviewed or GM-recorded lifting capacity",
                    ));
                }
                if target.runtime.flinch_round != battle(c)?.round {
                    abilities::inflict(c, id, &target.id, "Flinch", "", None, cat)?;
                }
            }
            let extra = u32::from(
                ["Solar Beam", "Solar Blade"].contains(&name)
                    && matches!(
                        weather,
                        Some(
                            field::Weather::Rain
                                | field::Weather::Sandstorm
                                | field::Weather::Snow
                                | field::Weather::Hail
                                | field::Weather::Typhoon
                        )
                    ),
            );
            let turn = flow(c)?.turn;
            state_mut(c).charges.insert(
                id.into(),
                Charge {
                    action: q,
                    remaining: extra,
                    turn,
                },
            );
            flow_mut(c).window = false;

            advance(c);
        }
    }
    if !success && state(c).rampages.get(id).is_some_and(|r| r.uses >= 3) {
        end_rampage(c, id, cat)?;
    }
    Ok(())
}
pub fn lifecycle(_before: &Campaign, c: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !enabled(c) || c.battle.as_ref().is_none_or(|b| !b.active) {
        return Ok(());
    }
    if !preparing(c)
        && enabled(_before)
        && _before
            .battle
            .as_ref()
            .is_some_and(|b| b.active && c.battle.as_ref().is_some_and(|next| next.id == b.id))
    {
        let restricted: Vec<_> = state(_before)
            .effects
            .iter()
            .filter(|e| e.name == "Roar Of Time")
            .map(|e| e.target.clone())
            .collect();
        for id in restricted {
            let old = registered(_before, &id)?;
            let new = registered(c, &id)?;
            let key = format!("special:roar:{}", battle(c)?.round);
            let paid = new
                .runtime
                .ability
                .counters
                .get(&key)
                .copied()
                .unwrap_or(old.actions as i32)
                .max(old.actions as i32);
            let cost = (new.actions as i32 - paid).max(0);
            if cost > 0 {
                spend(c, &id, cost)?;
                let actions = registered(c, &id)?.actions as i32;
                fighter_mut(c, &id)?
                    .runtime
                    .ability
                    .counters
                    .insert(key, actions);
            }
        }
    }
    let active: BTreeSet<String> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    state_mut(c).charges.retain(|id, _| active.contains(id));
    state_mut(c).bides.retain(|id, _| active.contains(id));
    state_mut(c).shields.retain(|id, _| active.contains(id));
    let turn = flow(c)?.turn;
    let current = flow(c)?.order.get(flow(c)?.cursor).cloned();
    for (id, s) in &mut state_mut(c).shields {
        if s.name == "Endure" && current.as_ref() == Some(id) && s.armed_turn != turn {
            s.available = true;
            s.armed_turn = turn;
        }
    }
    state_mut(c).effects.retain(|e| {
        active.contains(&e.target) || ["Wish", "Shadow Force"].contains(&e.name.as_str())
    });
    let ids: Vec<_> = state(c)
        .rampages
        .iter()
        .filter(|(id, r)| {
            !active.contains(*id)
                || r.round != battle(c).unwrap().round
                || (r.uses >= 3 && flow(c).unwrap().queue.is_empty())
        })
        .map(|(id, _)| id.clone())
        .collect();
    for id in ids {
        end_rampage(c, &id, cat)?;
    }
    let fallen: Vec<_> = battle(c)?
        .combatants
        .iter()
        .filter(|p| {
            p.hp == 0
                && _before
                    .battle
                    .as_ref()
                    .is_some_and(|b| b.combatants.iter().any(|old| old.id == p.id && old.hp > 0))
                && !state(c).history.iter().any(|h| {
                    h.name == "[fainted]" && h.target == p.id && h.round == battle(c).unwrap().round
                })
        })
        .map(|p| p.id.clone())
        .collect();
    for id in fallen {
        let round = battle(c)?.round;
        let turn = flow(c)?.turn;
        state_mut(c).history.push(Hit {
            source: id.clone(),
            target: id,
            name: "[fainted]".into(),
            category: String::new(),
            dice: 0,
            damage: 0,
            round,
            turn,
        });
    }
    if _before
        .battle
        .as_ref()
        .is_some_and(|b| b.round < battle(c).unwrap().round)
    {
        let wishes: Vec<_> = state(c)
            .effects
            .iter()
            .filter(|e| e.name == "Wish")
            .cloned()
            .collect();
        for e in wishes {
            let target = registered(c, &e.target)?;
            let recipient = if target.deployed {
                target.id.clone()
            } else {
                battle(c)?
                    .combatants
                    .iter()
                    .find(|p| {
                        p.deployed
                            && format!("{}:{}", p.side, p.field_slot.unwrap_or(u32::MAX)) == e.slot
                    })
                    .map_or(e.target.clone(), |p| p.id.clone())
            };
            heal(c, &recipient, 3, false)?;
        }
        state_mut(c).effects.retain(|e| e.name != "Wish");
    }
    Ok(())
}
fn indirect(c: &mut Campaign, id: &str, n: i32) -> Result<()> {
    let ring = fighter(c, id).is_ok_and(|p| house::ring(c, p));
    let p = fighter_mut(c, id)?;
    if ring || !p.has_any_ability(&["Magic Guard", "Wonder Guard"]) {
        p.hp = (p.hp - n.max(0)).max(0);
    }
    Ok(())
}
pub fn healing_blocked(c: &Campaign, id: &str) -> bool {
    items::completion::healing_locked(c, id)
        || (enabled(c)
            && state(c).effects.iter().any(|e| {
                e.target == id && ["Heal Block", "Psychic Noise"].contains(&e.name.as_str())
            }))
}
pub fn heal(c: &mut Campaign, id: &str, n: i32, revive: bool) -> Result<()> {
    let recipient = if enabled(c) {
        state(c)
            .effects
            .iter()
            .find(|e| e.name == "Snatch" && e.target == id)
            .map(|e| e.source.clone())
    } else {
        None
    };
    let id = recipient.as_deref().unwrap_or(id);
    if healing_blocked(c, id) {
        return Ok(());
    }
    let debt = if enabled(c) {
        state(c)
            .effects
            .iter()
            .filter(|e| e.name == "Shadow Force" && e.target == id)
            .map(|e| e.amount)
            .sum::<i32>()
    } else {
        0
    };
    let p = fighter_mut(c, id)?;
    if p.hp == 0 && !revive {
        return Ok(());
    }
    let cap = if n >= 5 { n } else { 3 };
    let amount = n
        .max(0)
        .min((p.max_hp - debt - p.hp).max(0))
        .min((cap - p.runtime.healed).max(0));
    p.hp += amount;
    p.runtime.healed += amount;
    Ok(())
}
fn spend(c: &mut Campaign, id: &str, n: i32) -> Result<()> {
    let id = tactics::payer(c, id);
    let p = fighter_mut(c, &id)?;
    if p.runtime.will < n {
        return Err(error(
            "This healing effect requires accepted remaining Will",
        ));
    }
    p.runtime.will -= n;
    Ok(())
}
fn pay_once(c: &mut Campaign, q: &QueuedAction) -> Result<()> {
    let marker = format!("paid:{}", q.sequence);
    if !state(c).effects.iter().any(|e| e.name == marker) {
        spend(c, &q.actor_id, 1)?;
        add_effect(c, &marker, &q.actor_id, &q.actor_id, Some(1))?;
    }
    Ok(())
}
fn ailment(s: &Status) -> bool {
    s.kind.starts_with("Burn")
        || s.kind.contains("Poison")
        || ["Sleep", "Frozen Solid", "Paralysis"].contains(&s.kind.as_str())
}
fn cure(c: &mut Campaign, id: &str, sleep: bool) -> Result<bool> {
    let p = fighter_mut(c, id)?;
    let old = p.runtime.statuses.len();
    p.runtime
        .statuses
        .retain(|s| !ailment(s) || (!sleep && s.kind == "Sleep"));
    Ok(p.runtime.statuses.len() != old)
}
fn add_effect(
    c: &mut Campaign,
    name: &str,
    source: &str,
    target: &str,
    rounds: Option<u32>,
) -> Result<()> {
    if state(c)
        .effects
        .iter()
        .any(|e| e.name == name && e.target == target)
    {
        return Ok(());
    }
    let p = registered(c, target)?;
    let slot = format!("{}:{}", p.side, p.field_slot.unwrap_or(u32::MAX));
    state_mut(c).effects.push(TimedEffect {
        amount: 0,
        name: name.into(),
        source: source.into(),
        target: target.into(),
        rounds,
        started: 0,
        slot,
    });
    Ok(())
}
pub fn registered<'a>(c: &'a Campaign, id: &str) -> Result<&'a Combatant> {
    battle(c)?
        .combatants
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| error("Choose a registered battle participant"))
}
fn own_registered(c: &Campaign, actor: &str, id: &str) -> Result<()> {
    let p = registered(c, id)?;
    if actor != c.gm_id && actor != p.owner_id {
        return Err(error(
            "Only the owner or appointed GM may resolve this registered participant",
        ));
    }
    Ok(())
}
fn charge_for<'a>(c: &'a Campaign, id: &str) -> Option<&'a Charge> {
    state(c).charges.get(id).or_else(|| {
        state(c)
            .charges
            .values()
            .find(|q| q.action.move_name == "Sky Drop" && q.action.target_id == id)
    })
}
pub fn carried(c: &Campaign, id: &str) -> bool {
    enabled(c)
        && state(c)
            .charges
            .values()
            .any(|q| q.action.move_name == "Sky Drop" && q.action.target_id == id)
}
pub fn effects(c: &mut Campaign, q: &QueuedAction, dice: &[u8], cat: &Catalog) -> Result<bool> {
    if !enabled(c) {
        return Ok(false);
    }
    let name = q.move_name.as_str();
    if ["Substitute", "Tidy Up"].contains(&name) && !decoys::enabled(c) {
        return Err(AppError::new(
            "manual_rule",
            "This decoy effect requires adoption of the current engine between battles",
        ));
    }
    let a = &q.actor_id;
    let t = &q.target_id;
    let m = move_entry(cat, name)?;
    if m["Attributes"]["ShieldMove"] == true && name != "Snatch" {
        if state(c)
            .effects
            .iter()
            .any(|e| e.name == "Feint" && e.target == *t)
        {
            return Ok(true);
        }
        if name == "Max Guard" {
            return Err(error(
                "Max Guard requires an unimplemented Max transformation; record a GM ruling",
            ));
        }
        let round = battle(c)?.round;
        let turn = flow(c)?.turn;
        state_mut(c).shields.insert(
            t.clone(),
            Shield {
                name: name.into(),
                source: a.clone(),
                round,
                armed_turn: turn,
                available: true,
            },
        );
        return Ok(true);
    }
    if m["Attributes"]["MustRecharge"] == true {
        let round = battle(c)?.round;
        // A shared all-target attack creates one recharge obligation, not one per target.
        if !state(c)
            .effects
            .iter()
            .any(|e| e.name == format!("recharge:{}", q.sequence))
        {
            let rt = &mut fighter_mut(c, a)?.runtime;
            rt.recharge_actions += 1;
            rt.recharge_round = round + 1;
            add_effect(c, &format!("recharge:{}", q.sequence), a, a, Some(1))?;
        }
        if name == "Roar Of Time" && !decoys::intercepted(c, q) {
            add_effect(c, "Roar Of Time", a, t, Some(1))?;
        }
        return Ok(true);
    }
    if m["Attributes"]["Recoil"] == true {
        let recoil = if recoil_immune(c, fighter(c, a)?) {
            0
        } else {
            q.raw_damage_successes as usize
        };
        let p = fighter(c, a)?;
        if house::ring(c, p) || !p.has_any_ability(&["Rock Head", "Magic Guard"]) {
            indirect(c, a, rules::successes(&dice[..recoil]))?;
        }
        if name == "Wave Crash" {
            extra::change_stat(c, a, a, "Dexterity", -1)?;
        }
        apply_chance(c, q, &dice[recoil..], cat)?;
        return Ok(true);
    }
    if ["Burn Up", "Double Shock"].contains(&name) {
        return Ok(false);
    }
    let ordinary_heal = [
        "Recover",
        "Slack Off",
        "Soft Boiled",
        "Milk Drink",
        "Heal Order",
        "Heal Pulse",
        "Floral Healing",
        "Rest",
        "Roost",
        "Shore Up",
        "Synthesis",
        "Morning Sun",
        "Moonlight",
        "Swallow",
        "Lunar Blessing",
    ];
    if ordinary_heal.contains(&name) {
        if a != t && target_effects_blocked(c, q, &m) {
            return Ok(true);
        }
        if name == "Swallow"
            && !state(c)
                .effects
                .iter()
                .any(|e| e.name == "Stockpile" && e.target == *a)
        {
            return Err(error("Swallow requires Stockpile"));
        }
        spend(c, a, 1)?;
        let p = fighter(c, a)?;
        let weather = field::weather(c, Some(p));
        let amount = match name {
            "Rest" | "Lunar Blessing" | "Swallow" => 5,
            "Floral Healing" if field::has(c, "Grassy Terrain", fighter(c, t)?) => 5,
            "Heal Pulse" if p.has_ability("Mega Launcher") => 5,
            "Shore Up" if weather == Some(field::Weather::Sandstorm) => 5,
            "Synthesis" | "Morning Sun" | "Moonlight" => {
                if matches!(
                    weather,
                    Some(field::Weather::Sunny | field::Weather::Desolate)
                ) || (name == "Moonlight" && state(c).parameters.get(a).is_some_and(|p| p.night))
                {
                    5
                } else if matches!(
                    weather,
                    Some(
                        field::Weather::Rain
                            | field::Weather::Typhoon
                            | field::Weather::Sandstorm
                            | field::Weather::Snow
                            | field::Weather::Hail
                    )
                ) {
                    1
                } else {
                    3
                }
            }
            _ => 3,
        };
        if name == "Rest" {
            abilities::inflict(c, a, a, "Sleep", "Rest", None, cat)?;
        }
        heal(c, t, amount, false)?;
        if name == "Lunar Blessing" {
            cure(c, t, true)?;
            let selected = state(c)
                .selections
                .get(&q.sequence)
                .ok_or_else(|| error("Choose the foe affected by Lunar Blessing"))?;
            let foe = selected.secondary_id.clone();
            if fighter(c, &foe)?.side == fighter(c, a)?.side {
                return Err(error("Choose an opposing foe"));
            }
            add_effect(c, "Lunar accuracy", a, &foe, None)?;
        }
        if name == "Roost" {
            add_effect(c, "Roost", a, a, Some(1))?;
        }
        if name == "Swallow" {
            extra::change_stat(c, a, a, "Defense", -1)?;
            extra::change_stat(c, a, a, "SpecialDefense", -1)?;
            state_mut(c)
                .effects
                .retain(|e| !(e.name == "Stockpile" && e.target == *a));
        }
        return Ok(true);
    }
    if drain(name) {
        if decoys::intercepted(c, q) {
            return Ok(true);
        }
        let amount = q.damage / 2;
        let ooze = fighter(c, t)?.has_ability("Liquid Ooze");
        if name == "Parabolic Charge" {
            let choice = state(c).selections.get(&q.sequence).ok_or_else(|| {
                error("Select one Parabolic Charge damage recipient for draining")
            })?;
            if choice.secondary_id != *t {
                return Ok(true);
            }
        }
        if ooze {
            indirect(c, a, amount)?;
        } else {
            heal(c, a, amount.min(3), false)?;
        }
        apply_chance(c, q, dice, cat)?;
        return Ok(true);
    }
    // Decoys block target effects, not the attacker's own costs or changes.
    if decoys::intercepted(c, q) {
        match name {
            "Spit Up" => {
                extra::change_stat(c, a, a, "Defense", -1)?;
                extra::change_stat(c, a, a, "SpecialDefense", -1)?;
                state_mut(c)
                    .effects
                    .retain(|e| !(e.name == "Stockpile" && e.target == *a));
            }
            "Scale Shot"
                if q.special_index == 5
                    && state(c).series.as_ref().is_some_and(|s| s.landed == 5) =>
            {
                extra::change_stat(c, a, a, "Dexterity", 1)?;
                extra::change_stat(c, a, a, "Defense", -1)?;
            }
            "Triple Arrows"
                if q.special_index == 3
                    && state(c).series.as_ref().is_some_and(|s| s.landed == 3) =>
            {
                add_effect(c, "Triple Arrows", a, a, None)?;
            }
            "Sparkly Swirl" => {
                for id in allies(c, a)? {
                    cure(c, &id, true)?;
                }
            }
            _ => {
                if decoys::unreviewed_user_clause(&m["AddedEffects"]) {
                    return Err(AppError::new("manual_rule", "Decoy blocks target effects, but this move has an unsupported user effect; record its GM resolution explicitly"));
                }
            }
        }
    }
    if target_effects_blocked(c, q, &m) {
        return Ok(true);
    }
    match name {
        "Feint" => {
            state_mut(c).shields.remove(t);
            fighter_mut(c, t)?.runtime.shield = 0;
            add_effect(c, "Feint", a, t, Some(1))?;
        }
        "Triple Arrows" => {
            extra::change_stat(c, a, t, "Defense", -1)?;
            if q.special_index == 3 && state(c).series.as_ref().is_some_and(|s| s.landed == 3) {
                add_effect(c, "Triple Arrows", a, a, None)?;
            }
        }
        "Scale Shot" => {
            if q.special_index == 5 && state(c).series.as_ref().is_some_and(|s| s.landed == 5) {
                extra::change_stat(c, a, a, "Dexterity", 1)?;
                extra::change_stat(c, a, a, "Defense", -1)?;
            }
        }
        "Sparkly Swirl" => {
            for id in allies(c, a)? {
                cure(c, &id, true)?;
            }
        }
        "Bide" => {
            let basis = if house::enabled(c) {
                a.clone()
            } else {
                state(c).selections.get(&q.sequence).ok_or_else(|| error("Bide requires the GM to choose User or Target as the ambiguous damage basis"))?.secondary_id.clone()
            };
            state_mut(c).bides.insert(
                a.clone(),
                Bide {
                    target_id: t.clone(),
                    basis_id: basis,
                    received: 0,
                    damage: 0,
                },
            );
        }
        "Substitute" => {
            decoys::cast(c, a, name, cat)?;
        }
        "Tidy Up" => {
            decoys::tidy(c);
            field::clear_ability_hazards(c);
            flow_mut(c).field_effects.retain(|e| {
                !["Spikes", "Toxic Spikes", "Sticky Web", "Stealth Rock"]
                    .contains(&e.move_name.as_str())
            });
            for key in ["Strength", "Dexterity"] {
                extra::change_stat(c, a, a, key, 1)?;
            }
        }
        "Snatch" => {
            if state(c)
                .effects
                .iter()
                .any(|e| e.name == "SnatchUsed" && e.target == *t)
            {
                return Err(error(
                    "Cannot Snatch from a Pokemon that used Snatch this round",
                ));
            }
            if stat_channels::enabled(c) {
                stat_channels::transfer(c, t, a, "Snatch")?;
                clear_modifiers(c, t);
                fighter_mut(c, t)?.runtime.modifiers.clear();
            } else {
                for e in &mut state_mut(c).effects {
                    if e.target == *t
                        && (e.name.starts_with("buff:") || e.name.starts_with("debuff:"))
                    {
                        e.target = a.clone();
                    }
                }
                let mods = std::mem::take(&mut fighter_mut(c, t)?.runtime.modifiers);
                fighter_mut(c, a)?.runtime.modifiers.extend(mods);
                let keys: Vec<_> = fighter(c, a)?.runtime.modifiers.keys().cloned().collect();
                for key in keys {
                    let positive = format!("buff:{key}");
                    let negative = format!("debuff:{key}");
                    let values: Vec<_> = state(c)
                        .effects
                        .iter()
                        .filter(|e| e.target == *a && (e.name == positive || e.name == negative))
                        .map(|e| e.amount)
                        .collect();
                    if !values.is_empty() {
                        fighter_mut(c, a)?.runtime.modifiers.insert(
                            key,
                            values.iter().copied().max().unwrap_or(0).max(0)
                                + values.iter().copied().min().unwrap_or(0).min(0),
                        );
                    }
                }
            }
            if let Some(mut s) = state_mut(c).shields.remove(t) {
                s.source = a.clone();
                state_mut(c).shields.insert(a.clone(), s);
            }
            let from = fighter(c, t)?.side.clone();
            let to = fighter(c, a)?.side.clone();
            for r in &mut flow_mut(c).battlefield.rules {
                if r.scope == format!("side:{from}") {
                    r.scope = format!("side:{to}");
                }
            }
            for pending in &mut flow_mut(c).queue {
                if pending.sequence != q.sequence
                    && pending.actor_id == *t
                    && pending.target_id == *t
                {
                    let raw = move_entry(cat, &pending.move_name)?;
                    if raw["Attributes"]["ZMove"] == true || raw["Attributes"]["MaxMove"] == true {
                        return Err(error("Snatch cannot steal Z/Max effects"));
                    }
                    pending.target_id = a.clone();
                }
            }
            decoys::snatch(c, t, a, cat)?;
            add_effect(c, "Snatch", a, t, Some(1))?;
            add_effect(c, "SnatchUsed", a, a, Some(1))?;
        }
        "Aqua Ring" | "Ingrain" | "Life Dew" => {
            pay_once(c, q)?;
            add_effect(
                c,
                name,
                a,
                t,
                if name == "Life Dew" { Some(4) } else { None },
            )?;
        }
        "Wish" => {
            spend(c, a, 1)?;
            add_effect(c, "Wish", a, t, Some(1))?;
        }
        "Heal Block" => {
            add_effect(c, name, a, t, Some(4))?;
        }
        "Psychic Noise" => {
            add_effect(c, name, a, t, Some(2))?;
        }
        "Purify" => {
            if cure(c, t, true)?
                && state(c)
                    .selections
                    .get(&q.sequence)
                    .is_some_and(|s| s.recipient_id == *a)
            {
                spend(c, a, 1)?;
                heal(c, a, 2, false)?;
            }
        }
        "Psycho Shift" => {
            let conditions: Vec<_> = fighter(c, a)?
                .runtime
                .statuses
                .iter()
                .filter(|s| ailment(s))
                .cloned()
                .collect();
            for s in conditions {
                abilities::inflict(c, a, t, &s.kind, &s.detail, s.rounds, cat)?;
            }
            cure(c, a, true)?;
        }
        "Refresh" => {
            cure(c, a, false)?;
        }
        "Take Heart" => {
            cure(c, a, true)?;
            extra::change_stat(c, a, a, "Special", 1)?;
            extra::change_stat(c, a, a, "SpecialDefense", 1)?;
        }
        "Heal Bell" | "Aromatherapy" => {
            let ids: Vec<_> = battle(c)?
                .combatants
                .iter()
                .filter(|p| p.side == fighter(c, a).unwrap().side)
                .map(|p| p.id.clone())
                .collect();
            for id in ids {
                cure(c, &id, true)?;
            }
        }
        "Jungle Healing" => {
            if !state(c)
                .effects
                .iter()
                .any(|e| e.name == format!("healing:{}", q.sequence))
            {
                spend(c, a, 1)?;
                add_effect(c, &format!("healing:{}", q.sequence), a, a, Some(1))?;
            }
            cure(c, t, true)?;
            heal(c, t, 1, false)?;
        }
        "Strength Sap" => {
            spend(c, a, 1)?;
            extra::change_stat(c, a, t, "Strength", -1)?;
            heal(c, a, 1, false)?;
        }
        "Pain Split" => {
            spend(c, a, 1)?;
            let n = 3.min((fighter(c, t)?.hp - 1).max(0));
            fighter_mut(c, t)?.hp -= n;
            heal(c, a, n, false)?;
        }
        "Leech Seed" => {
            if field::types(c, fighter(c, t)?, cat)?.contains(&"Grass".into()) {
                return Ok(true);
            }
            spend(c, a, 1)?;
            if !state(c)
                .effects
                .iter()
                .any(|e| e.target == *t && e.name == "Leech Seed")
            {
                add_effect(c, name, a, t, None)?;
            }
        }
        "Stockpile" => {
            extra::change_stat(c, a, a, "Defense", 1)?;
            extra::change_stat(c, a, a, "SpecialDefense", 1)?;
            add_effect(c, name, a, a, None)?;
        }
        "Spit Up" => {
            extra::change_stat(c, a, a, "Defense", -1)?;
            extra::change_stat(c, a, a, "SpecialDefense", -1)?;
            state_mut(c)
                .effects
                .retain(|e| !(e.name == "Stockpile" && e.target == *a));
        }
        "Revival Blessing" => {
            let will = fighter(c, a)?.runtime.will;
            if will <= 0 {
                return Err(error("Revival Blessing requires remaining Will"));
            }
            let target = registered(c, t)?;
            if target.hp > 0 || target.side != fighter(c, a)?.side {
                return Err(error("Choose a fainted registered ally"));
            }
            spend(c, a, will)?;
            heal(c, t, 2, true)?;
            if registered(c, t)?.hp == 0 {
                return Err(error(
                    "The revival is prevented by healing restrictions; GM resolution is required",
                ));
            }
        }
        "Healing Wish" | "Lunar Dance" => {
            let sel = state(c)
                .selections
                .get(&q.sequence)
                .ok_or_else(|| error("Choose the healing recipient before resolving"))?
                .clone();
            if sel.recipient_id == *a {
                return Err(error("Choose another ally"));
            }
            let target = registered(c, &sel.recipient_id)?.clone();
            let source = fighter(c, a)?.clone();
            if target.side != source.side || (target.hp == 0 && name == "Healing Wish") {
                return Err(error("Healing Wish cannot revive; Lunar Dance can"));
            }
            let loss = if name == "Lunar Dance" {
                rules::successes(dice)
            } else {
                source.hp
            };
            fighter_mut(c, a)?.hp = (source.hp - loss).max(0);
            // These are explicit full recovery effects; the ordinary healing cap does not apply.
            if !healing_blocked(c, &target.id) {
                let n = target.max_hp + target.runtime.healed;
                heal(c, &target.id, n, name == "Lunar Dance")?;
                fighter_mut(c, &target.id)?.runtime.statuses.clear();
            }
            if registered(c, &target.id)?.hp == 0 {
                return Err(error("The chosen recipient cannot recover consciousness under its current healing restrictions"));
            }
            if !target.deployed {
                let round = battle(c)?.round;
                if source.hp - loss > 0 {
                    return Err(error(
                        "The user's active slot is occupied; resolve its fainting first",
                    ));
                }
                let p = fighter_mut(c, &target.id)?;
                p.deployed = true;
                p.field_slot = source.field_slot;
                p.actions = source.actions;
                p.initiative = source.initiative;
                p.initiative_rolled = source.initiative_rolled;
                p.available_round = round;
                fighter_mut(c, a)?.deployed = false;
                fighter_mut(c, a)?.field_slot = None;
                for id in &mut flow_mut(c).order {
                    if *id == *a {
                        *id = target.id.clone();
                    }
                }
            }
        }
        "Present" => {
            let die = *state(c)
                .random
                .get(&q.sequence)
                .ok_or_else(|| error("Host-roll Present before resolving"))?;
            if die <= 3 {
                fighter_mut(c, t)?.hp = (fighter(c, t)?.hp - 2).max(0);
            } else {
                heal(c, t, 2, false)?;
            }
        }
        "Pollen Puff" if fighter(c, t)?.side == fighter(c, a)?.side => {
            spend(c, a, 1)?;
            heal(c, t, 2, false)?;
        }
        "Sparkling Aria"
            if fighter(c, t)?
                .runtime
                .statuses
                .iter()
                .any(|s| s.kind.starts_with("Burn")) =>
        {
            fighter_mut(c, t)?
                .runtime
                .statuses
                .retain(|s| !s.kind.starts_with("Burn"));
        }
        "Geomancy" => {
            for stat in ["Dexterity", "Special", "SpecialDefense"] {
                extra::change_stat(c, a, a, stat, 2)?;
            }
        }
        "Defense Curl" => {
            extra::change_stat(c, a, a, "Defense", 1)?;
            add_effect(c, name, a, a, Some(1))?;
        }
        "Final Gambit" => {
            fighter_mut(c, a)?.hp = 0;
        }
        "Wake-Up Slap" => {
            fighter_mut(c, t)?
                .runtime
                .statuses
                .retain(|s| s.kind != "Sleep");
        }
        "Smelling Salts" => {
            fighter_mut(c, t)?
                .runtime
                .statuses
                .retain(|s| s.kind != "Paralysis");
        }
        _ => {
            if let Some((kind, _)) = chance(name) {
                if dice.contains(&6)
                    && !(kind == "Flinch"
                        && fighter(c, t)?.runtime.flinch_round == battle(c)?.round)
                    && !(fighter(c, t)?.has_ability("Shield Dust")
                        && fighter(c, t)?.side != fighter(c, a)?.side)
                {
                    abilities::inflict(c, a, t, kind, "", None, cat)?;
                }
                return Ok(true);
            }
            if m["Attributes"]["Charge"] == true
                || m["Attributes"]["Rampage"] == true
                || limit(&m) > 1
                || formula_name(name)
                || variable_name(name)
            {
                return Ok(true);
            }
            return Ok(false);
        }
    }
    Ok(true)
}
pub fn drain(name: &str) -> bool {
    [
        "Absorb",
        "Mega Drain",
        "Giga Drain",
        "Drain Punch",
        "Draining Kiss",
        "Leech Life",
        "Horn Leech",
        "Bitter Blade",
        "Dream Eater",
        "Bouncy Bubble",
        "Matcha Gotcha",
        "Parabolic Charge",
    ]
    .contains(&name)
}
pub fn chance(name: &str) -> Option<(&'static str, u32)> {
    match name {
        "Flare Blitz" => Some(("Burn 3", 1)),
        "Volt Tackle" => Some(("Paralysis", 3)),
        "Matcha Gotcha" => Some(("Burn", 2)),
        "Bounce" | "Freeze Shock" => Some(("Paralysis", 3)),
        "Ice Burn" => Some(("Burn", 3)),
        "Sky Attack" | "Double Iron Bash" => Some(("Flinch", 3)),
        "Twineedle" => Some(("Poison", 2)),
        "Barb Barrage" => Some(("Poison", 3)),
        "Bitter Malice" => Some(("Frozen Solid", 3)),
        _ => None,
    }
}
fn chance_dice(c: &Campaign, q: &QueuedAction) -> Result<u32> {
    Ok(chance(&q.move_name).map_or(0, |(_, n)| {
        let n = n
            + items::completion::loaded(c, &q.actor_id)
            + 2 * u32::from(fighter(c, &q.actor_id).is_ok_and(|p| p.has_ability("Serene Grace")));
        n.saturating_sub(
            2 * u32::from(fighter(c, &q.target_id).is_ok_and(|p| p.has_ability("Wonder Skin"))),
        )
    }))
}
fn apply_chance(c: &mut Campaign, q: &QueuedAction, faces: &[u8], cat: &Catalog) -> Result<()> {
    if provenance::enabled(c) && q.sheer_force {
        return Ok(());
    }
    if let Some((kind, _)) = chance(&q.move_name) {
        let m = move_entry(cat, &q.move_name)?;
        if !target_effects_blocked(c, q, &m)
            && faces.contains(&6)
            && !(kind == "Flinch"
                && fighter(c, &q.target_id)?.runtime.flinch_round == battle(c)?.round)
            && !(fighter(c, &q.target_id)?.has_ability("Shield Dust")
                && fighter(c, &q.target_id)?.side != fighter(c, &q.actor_id)?.side)
        {
            abilities::inflict(c, &q.actor_id, &q.target_id, kind, "", None, cat)?;
        }
    }
    Ok(())
}
pub fn effect_dice(c: &Campaign, q: &QueuedAction, cat: &Catalog) -> Result<Option<u32>> {
    if !enabled(c) {
        return Ok(None);
    }
    let m = move_entry(cat, &q.move_name)?;
    if m["Attributes"]["Recoil"] == true {
        return Ok(Some(
            (if provenance::enabled(c) && q.sheer_force {
                0
            } else {
                chance_dice(c, q)?
            }) + if recoil_immune(c, fighter(c, &q.actor_id)?) {
                0
            } else {
                q.raw_damage_successes
            },
        ));
    }
    if q.move_name == "Lunar Dance" {
        return Ok(Some(fighter(c, &q.actor_id)?.hp.max(0) as u32));
    }
    if let Some((_, n)) = chance(&q.move_name) {
        let n = n
            + items::completion::loaded(c, &q.actor_id)
            + 2 * u32::from(fighter(c, &q.actor_id)?.has_ability("Serene Grace"));
        return Ok(Some(n.saturating_sub(
            2 * u32::from(fighter(c, &q.target_id)?.has_ability("Wonder Skin")),
        )));
    }
    Ok(None)
}
pub fn shield_reduction(c: &Campaign, q: &QueuedAction, m: &serde_json::Value) -> i32 {
    if !enabled(c) {
        return fighter(c, &q.target_id).map_or(0, |p| p.runtime.shield);
    }
    if bypass(c, q, m) {
        return 0;
    }
    state(c)
        .shields
        .get(&q.target_id)
        .filter(|s| s.available && s.round == battle(c).unwrap().round)
        .map_or(0, |s| match s.name.as_str() {
            "Crafty Shield" | "Endure" => 0,
            "Quick Guard"
                if m["Attributes"]["Reaction"].is_null()
                    && m["Attributes"]["LateReaction"].is_null() =>
            {
                0
            }
            "Max Guard" => 5,
            _ => 3,
        })
}
fn recoil_immune(c: &Campaign, p: &Combatant) -> bool {
    if house::ring(c, p) {
        return false;
    }
    if interactions::enabled(c) {
        p.has_any_ability(&["Rock Head", "Magic Guard"])
    } else {
        ["Rock Head", "Magic Guard"].contains(&p.selected_ability.as_str())
    }
}
fn bypass(c: &Campaign, q: &QueuedAction, m: &serde_json::Value) -> bool {
    if fighter(c, &q.actor_id).is_ok_and(|p| interactions::unseen(c, p, m)) {
        return true;
    }
    fighter(c, &q.actor_id).is_ok_and(|p| reviewed::extreme(c, p))
        || m["Attributes"]["IgnoreShield"] == true
        || vulnerable(c, &q.target_id, &q.move_name)
        || fighter(c, &q.actor_id).is_ok_and(|p| p.has_ability("Infiltrator"))
}
pub fn target_effects_blocked(c: &Campaign, q: &QueuedAction, m: &serde_json::Value) -> bool {
    if decoys::blocks_effects(c, q, m) {
        return true;
    }
    if !enabled(c) || bypass(c, q, m) {
        return false;
    }
    if q.actor_id != q.target_id && m["Category"] != "Support" && holding_back(c, q) {
        return true;
    }
    state(c)
        .effects
        .iter()
        .any(|e| e.name == format!("shield:{}", q.sequence) && e.target == q.target_id)
        || state(c).shields.get(&q.target_id).is_some_and(|s| {
            s.available
                && s.round == battle(c).unwrap().round
                && [
                    "Protect",
                    "Detect",
                    "Wide Guard",
                    "Crafty Shield",
                    "Max Guard",
                ]
                .contains(&s.name.as_str())
        })
}
pub fn before_damage(
    c: &mut Campaign,
    q: &QueuedAction,
    m: &serde_json::Value,
    n: i32,
    cat: &Catalog,
) -> Result<i32> {
    if !enabled(c) || bypass(c, q, m) {
        return Ok(n);
    }
    let Some(shield) = state(c)
        .shields
        .get(&q.target_id)
        .cloned()
        .filter(|s| s.round == battle(c).unwrap().round)
    else {
        return Ok(n);
    };
    let endure = shield.name == "Endure" && shield.available;
    if !shield.available && !endure {
        return Ok(n);
    }
    if [
        "Protect",
        "Detect",
        "Wide Guard",
        "Crafty Shield",
        "Max Guard",
    ]
    .contains(&shield.name.as_str())
        || (shield.name == "Quick Guard"
            && (!m["Attributes"]["Reaction"].is_null()
                || !m["Attributes"]["LateReaction"].is_null()))
    {
        add_effect(
            c,
            &format!("shield:{}", q.sequence),
            &shield.source,
            &q.target_id,
            Some(1),
        )?;
    }
    let turn = flow(c)?.turn;
    let s = state_mut(c).shields.get_mut(&q.target_id).unwrap();
    s.available = false;
    s.armed_turn = turn;
    if endure && decoys::protects(c, q, m) {
        return Ok(n);
    }
    if endure {
        return Ok(n.min((fighter(c, &q.target_id)?.hp - 1).max(0)));
    }
    if abilities::contact(m) && !items::completion::pads(c, &q.actor_id) {
        match shield.name.as_str() {
            "Baneful Bunker" => {
                abilities::inflict(c, &q.target_id, &q.actor_id, "Poison", "", None, cat)?
            }
            "Burning Bulwark" => {
                abilities::inflict(c, &q.target_id, &q.actor_id, "Burn", "", None, cat)?
            }
            "King's Shield" => extra::change_stat(c, &q.target_id, &q.actor_id, "Strength", -2)?,
            "Obstruct" => extra::change_stat(c, &q.target_id, &q.actor_id, "Defense", -2)?,
            "Silk Trap" => extra::change_stat(c, &q.target_id, &q.actor_id, "Dexterity", -2)?,
            "Spiky Shield" => {
                let p = fighter(c, &q.target_id)?.clone();
                flow_mut(c).ability_events.push(abilities::AbilityEvent {
                    id: format!("shield:{}:{}", q.sequence, q.target_id),
                    combatant_id: p.id,
                    target_id: q.actor_id.clone(),
                    ability: "Spiky Shield".into(),
                    kind: "retaliate".into(),
                    move_name: q.move_name.clone(),
                    choices: vec!["roll".into()],
                });
            }
            _ => {}
        }
    }
    Ok(n)
}
pub fn formula_name(name: &str) -> bool {
    [
        "Bide",
        "Sonic Boom",
        "Dragon Rage",
        "Fissure",
        "Guillotine",
        "Horn Drill",
        "Sheer Cold",
        "Night Shade",
        "Seismic Toss",
        "Psywave",
        "Super Fang",
        "Nature's Madness",
        "Endeavor",
        "Counter",
        "Mirror Coat",
        "Metal Burst",
        "Comeuppance",
        "Final Gambit",
    ]
    .contains(&name)
}
pub fn fixed(name: &str) -> bool {
    [
        "Sonic Boom",
        "Dragon Rage",
        "Fissure",
        "Guillotine",
        "Horn Drill",
        "Sheer Cold",
    ]
    .contains(&name)
}
pub fn formula(
    c: &Campaign,
    a: &Combatant,
    t: &Combatant,
    m: &serde_json::Value,
    sequence: u32,
    modifier: i32,
    cat: &Catalog,
) -> Result<Option<rules::DamagePool>> {
    let name = m["Name"].as_str().unwrap_or("");
    if !enabled(c) || !formula_name(name) {
        return Ok(None);
    }
    let ty = m["Type"].as_str().unwrap_or("Normal");
    let mut effect = 0;
    let mut immune = false;
    if name != "Sonic Boom" {
        for dt in field::types(c, t, cat)? {
            let n = if extra::ignores_ghost(c, a, ty, &dt) {
                0
            } else {
                rules::type_effect(ty, &dt)?
            };
            if n == -100 {
                if !(ty == "Ground"
                    && dt == "Flying"
                    && (field::has(c, "Gravity", t)
                        || roost(c, &t.id)
                        || items::iron_ball(c, &t.id)))
                {
                    immune = true;
                }
            } else {
                effect += n;
            }
        }
    }
    if reviewed::enabled(c) && name != "Sonic Boom" {
        (effect, immune) = rules::type_matchup(c, a, t, ty, cat)?;
    }
    reviewed::unique_damage_guard(c, a, t, effect)?;
    immune |= abilities::damage_immune_from(c, a, t, m) || extra::source_immune(c, a, t, effect);
    let rank = crate::creation::rank(&c.sheets[&a.sheet_id].rank)
        .ok_or_else(|| error("Unknown damage rank"))?;
    let n = match name {
        "Bide" => {
            let b = state(c)
                .bides
                .get(&a.id)
                .filter(|b| b.received >= 2)
                .ok_or_else(|| error("Bide must receive its next two attacks before releasing"))?;
            if b.target_id != t.id {
                return Err(error("Bide retains its original target"));
            }
            b.damage * 2
        }
        "Sonic Boom" | "Dragon Rage" | "Fissure" | "Guillotine" | "Horn Drill" | "Sheer Cold" => 0,
        "Night Shade" | "Seismic Toss" | "Psywave" => [1, 2, 4, 6, 8, 10, 10, 10][rank],
        "Super Fang" | "Nature's Madness" => {
            if name == "Nature's Madness" && t.hp <= 1 {
                return Err(error("Nature's Madness fails against one remaining HP"));
            }
            (t.hp / 2).clamp(1, 10)
        }
        "Endeavor" => {
            let n = (t.max_hp - a.hp).clamp(0, 10);
            if n == 0 || t.hp <= a.hp {
                return Err(error(
                    "Endeavor cannot lower this target below the user's HP",
                ));
            }
            n
        }
        "Final Gambit" => t.hp + 4,
        "Counter" | "Mirror Coat" | "Metal Burst" | "Comeuppance" => {
            let h = state(c)
                .history
                .iter()
                .rev()
                .find(|h| {
                    h.source == t.id
                        && h.target == a.id
                        && ["Physical", "Special"].contains(&h.category.as_str())
                })
                .ok_or_else(|| error("No previous incoming damage pool from this target"))?;
            if (name == "Counter" && h.category != "Physical")
                || (name == "Mirror Coat" && h.category != "Special")
            {
                return Err(error(
                    "The last incoming move has the wrong damage category",
                ));
            }
            h.dice as i32 + 2
        }
        _ => unreachable!(),
    };
    let dice = if fixed(name) {
        if modifier != 0 {
            return Err(error("Set damage does not have a dice modifier"));
        }
        0
    } else {
        let n = n
            + modifier
            + if items::enabled(c) {
                provenance::sheer_force_bonus(c, &a.id, sequence)
            } else {
                0
            }
            + match abilities::context(c, &a.id, sequence) {
                Some(ctx) => ctx.item_damage_bonus,
                None => items::damage_dice_bonus(c, a, sequence, m, cat)?,
            };
        if !(0..=100).contains(&n) || (n == 0 && name != "Bide") {
            return Err(error("Varied damage requires 1–100 dice"));
        }
        n as u32
    };
    Ok(Some(rules::DamagePool {
        dice: if immune { 0 } else { dice },
        attribute: 0,
        power: n,
        defense: 0,
        stab: 0,
        critical: 0,
        pain: 0,
        effectiveness: effect,
        immune,
    }))
}
pub fn set_amount(c: &Campaign, q: &QueuedAction) -> i32 {
    match q.move_name.as_str() {
        "Sonic Boom" => 1,
        "Dragon Rage" => 2,
        _ => fighter(c, &q.target_id).map_or(0, |p| p.hp),
    }
}
pub fn resolve_formula(
    c: &mut Campaign,
    actor: &str,
    q: &QueuedAction,
    action: &Action,
    faces: &[u8],
    cat: &Catalog,
) -> Result<String> {
    let Action::Damage {
        pool_modifier,
        success_modifier,
        damage_modifier,
        ..
    } = action
    else {
        unreachable!()
    };
    owner(c, actor, &q.actor_id)?;
    if actor != c.gm_id && (*pool_modifier != 0 || *success_modifier != 0 || *damage_modifier != 0)
    {
        return Err(error("Damage overrides require GM"));
    }
    if !(-20..=20).contains(damage_modifier) || !(-20..=20).contains(success_modifier) {
        return Err(error("Damage modifier out of bounds"));
    }
    if fixed(&q.move_name) && *success_modifier != 0 {
        return Err(error("Set damage has no success modifier"));
    }
    let mut temp = old(c);
    install_pending(&mut temp, q);
    let pool = rules::damage_pool(&temp, &q.actor_id, q.sequence, *pool_modifier, cat)?;
    check_count(faces, pool.dice)?;
    let base = if fixed(&q.move_name) {
        set_amount(c, q)
    } else {
        (rules::successes(faces) + success_modifier - pool.pain)
            .max(0)
            .max(1)
    };
    let m = field::move_data(c, fighter(c, &q.actor_id)?, &move_entry(cat, &q.move_name)?);
    let mut amount = if pool.immune || (q.move_name == "Bide" && pool.dice == 0) {
        0
    } else {
        (base + pool.effectiveness + damage_modifier).max(0)
    };
    amount += provenance::stakeout_bonus(c, q, pool.immune);
    amount += items::extra_damage(
        c,
        &q.actor_id,
        &q.target_id,
        q.sequence,
        (pool.effectiveness, pool.immune),
        cat,
    )?;
    provenance::consume_stakeout(c, q, pool.immune);
    if q.move_name == "Endeavor" {
        amount = amount.min((fighter(c, &q.target_id)?.hp - fighter(c, &q.actor_id)?.hp).max(0));
    }
    if !fixed(&q.move_name) {
        amount = (amount
            - abilities::reduction(
                c,
                &decoys::damage_subject(c, &q.actor_id, fighter(c, &q.target_id)?, &m),
                &m,
                pool.effectiveness,
                &q.actor_id,
            )
            - field::reduction(c, fighter(c, &q.actor_id)?, fighter(c, &q.target_id)?, &m))
        .max(0);
    }
    if pool.immune {
        amount = 0;
    }
    let before = fighter(c, &q.target_id)?.hp;
    // Set damage and explicitly defense-ignoring formulas bypass Shields; Endure remains a survival effect.
    if state(c)
        .shields
        .get(&q.target_id)
        .is_some_and(|s| s.name == "Endure")
    {
        amount = before_damage(c, q, &m, amount, cat)?;
    }
    let decoy_hit = decoys::damage(c, q, &m, amount);
    let amount = if decoy_hit {
        amount
    } else {
        abilities::sturdy(c, &q.target_id, amount, before)?
    };
    let amount = if !decoy_hit {
        items::completion::survive(c, q, before, amount)
    } else {
        amount
    };
    fighter_mut(c, &q.target_id)?.hp = if decoy_hit {
        before
    } else {
        (before - amount).max(0)
    };
    let current = flow_mut(c).queue.first_mut().unwrap();
    current.damage_done = true;
    current.damage = if decoy_hit { 0 } else { amount };
    current.raw_damage_successes = rules::successes(faces) as u32;
    if !decoy_hit {
        abilities::hit(c, q, before, &m, cat)?;
    }
    hit(
        c,
        q,
        &m,
        pool.dice,
        before - fighter(c, &q.target_id)?.hp,
        cat,
    )?;
    if q.move_name == "Final Gambit" {
        fighter_mut(c, &q.actor_id)?.hp = 0;
    }
    if q.move_name == "Bide" {
        state_mut(c).bides.remove(&q.actor_id);
    }
    if items::recoil_dice(c, flow(c)?.queue.first().unwrap(), cat)? == 0
        && items::chance_dice(c, flow(c)?.queue.first().unwrap(), cat)? == 0
    {
        finish(c);
    }
    Ok(format!("{}: {} dice, {amount} final damage; typed immunity/effectiveness {}, explicit formula without STAB/critical bonuses",q.move_name,pool.dice,pool.effectiveness))
}
pub fn variable_name(n: &str) -> bool {
    [
        "Magnitude",
        "Flail",
        "Reversal",
        "Eruption",
        "Water Spout",
        "Dragon Energy",
        "Hard Press",
        "Crush Grip",
        "Wring Out",
        "Return",
        "Frustration",
        "Veevee Volley",
        "Electro Ball",
        "Gyro Ball",
        "Grass Knot",
        "Low Kick",
        "Heavy Slam",
        "Heat Crash",
        "Facade",
        "Brine",
        "Venoshock",
        "Hex",
        "Bitter Malice",
        "Barb Barrage",
        "Wake-Up Slap",
        "Smelling Salts",
        "Stored Power",
        "Power Trip",
        "Trump Card",
        "Rage Fist",
        "Last Respects",
        "Bolt Beak",
        "Fishious Rend",
        "Assurance",
        "Avalanche",
        "Payback",
        "Retaliate",
        "Fusion Bolt",
        "Fusion Flare",
        "Echoed Voice",
        "Punishment",
        "Rising Voltage",
        "Expanding Force",
        "Collision Course",
        "Electro Drift",
        "Spit Up",
    ]
    .contains(&n)
}
fn weight(c: &Campaign, p: &Combatant, cat: &Catalog) -> Result<f64> {
    if standalone::enabled(c) {
        return standalone::weight(
            c,
            p,
            cat,
            state(c).parameters.get(&p.id).and_then(|v| v.weight_kg),
        );
    }

    if let Some(n) = state(c).parameters.get(&p.id).and_then(|v| v.weight_kg) {
        return Ok(n);
    }
    let s = &c.sheets[&p.sheet_id];
    cat.find("species", "AppId", s.species_id.as_deref().unwrap_or(""))
        .and_then(|v| v["Weight"]["Kilograms"].as_f64())
        .filter(|n| n.is_finite() && *n > 0.)
        .ok_or_else(|| error("Missing weight: GM must supply the encounter weight in kg"))
}
pub fn bonus(
    c: &Campaign,
    a: &Combatant,
    t: &Combatant,
    m: &serde_json::Value,
    cat: &Catalog,
) -> Result<i32> {
    if !enabled(c) {
        return Ok(0);
    }
    let name = m["Name"].as_str().unwrap_or("");
    let seq = flow(c)?.queue.first().map_or(0, |q| q.sequence);
    let source = if provenance::ignores_attributes(c, t, a) {
        rules::base_sheet(c, a)?
    } else {
        rules::effective_sheet(c, a)?
    };
    let target = if provenance::ignores_attributes(c, a, t) {
        rules::base_sheet(c, t)?
    } else {
        rules::effective_sheet(c, t)?
    };
    let happy = c.sheets[&a.sheet_id]
        .workflow
        .as_ref()
        .map_or(2, |w| w.happiness);
    let ailments = |p: &Combatant| p.runtime.statuses.iter().any(ailment);
    let round = battle(c)?.round;
    let turn = flow(c)?.turn;
    Ok(match name {
        "Flail" | "Reversal" => (a.max_hp - a.hp).min(5),
        "Eruption" | "Water Spout" | "Dragon Energy" | "Hard Press" => -(a.max_hp - a.hp).min(5),
        "Crush Grip" => (t.hp - 4).clamp(0, 9),
        "Wring Out" => -(t.max_hp - t.hp).min(4),
        "Return" | "Veevee Volley" => happy.clamp(0, 5),
        "Frustration" => (5 - happy).clamp(0, 5),
        "Electro Ball" => (source.stats["Dexterity"] - target.stats["Dexterity"]).clamp(0, 4),
        "Gyro Ball" => (target.stats["Dexterity"] - source.stats["Dexterity"]).clamp(0, 5),
        "Grass Knot" | "Low Kick" => (weight(c, t, cat)? / 50.).floor().clamp(0., 4.) as i32,
        "Heavy Slam" | "Heat Crash" => ((weight(c, a, cat)? - weight(c, t, cat)?) / 50.)
            .floor()
            .clamp(0., 4.) as i32,
        "Facade" if ailments(a) => 3,
        "Brine" if t.hp <= t.max_hp / 2 => 3,
        "Venoshock" if t.runtime.statuses.iter().any(|s| s.kind.contains("Poison")) => 2,
        "Hex" | "Bitter Malice" | "Barb Barrage" if ailments(t) => 2,
        "Wake-Up Slap" if status(t, "Sleep") => 2,
        "Smelling Salts" if status(t, "Paralysis") => 2,
        "Trump Card" if a.runtime.will == 0 => 8,
        "Trump Card"
            if a.runtime.will
                <= c.sheets[&a.sheet_id]
                    .workflow
                    .as_ref()
                    .map_or(a.runtime.will, |w| w.max_will)
                    / 2 =>
        {
            2
        }
        "Stored Power" | "Power Trip" => a
            .runtime
            .modifiers
            .values()
            .filter(|n| **n > 0)
            .sum::<i32>()
            .min(7),
        "Punishment" => t
            .runtime
            .modifiers
            .values()
            .filter(|n| **n > 0)
            .sum::<i32>()
            .min(7),
        "Payback"
            if state(c).history.iter().any(|h| {
                h.source == t.id && h.target == a.id && h.round == round && h.damage > 0
            }) =>
        {
            2
        }
        "Retaliate"
            if state(c).history.iter().any(|h| {
                h.name == "[fainted]"
                    && h.round + 1 >= round
                    && registered(c, &h.target).is_ok_and(|p| p.side == a.side)
                    && !a
                        .runtime
                        .ability
                        .flags
                        .contains(&format!("Retaliate:{}:{}", h.target, h.round))
            }) =>
        {
            c.sheets[&a.sheet_id]
                .workflow
                .as_ref()
                .map_or(2, |w| w.loyalty)
                .clamp(0, 5)
        }
        "Echoed Voice" => a
            .runtime
            .ability
            .counters
            .get("special:echo")
            .copied()
            .unwrap_or(0)
            .min(8),
        "Rising Voltage" if field::has(c, "Electric Terrain", a) => 3,
        "Expanding Force" if field::has(c, "Psychic Terrain", a) => 2,
        "Collision Course" | "Electro Drift" => {
            let ty = if name == "Collision Course" {
                "Fight"
            } else {
                "Electric"
            };
            let mut total = 0;
            for dt in field::types(c, t, cat)? {
                total += rules::type_effect(ty, &dt)?;
            }
            if total > 0 {
                4
            } else {
                0
            }
        }
        "Rage Fist" => (2 * state(c)
            .history
            .iter()
            .filter(|h| h.target == a.id && h.round == round && h.damage > 0)
            .count() as i32)
            .min(8),
        "Last Respects" => (2 * flow(c)?
            .extra
            .fainted
            .iter()
            .filter(|id| registered(c, id).is_ok_and(|p| p.side == a.side))
            .count() as i32)
            .min(8),
        "Assurance"
            if state(c).history.iter().any(|h| {
                h.source == a.id && h.target == t.id && h.round == round && h.damage > 0
            }) =>
        {
            2
        }
        "Avalanche"
            if state(c).history.iter().any(|h| {
                h.source == t.id && h.target == a.id && h.turn == turn && h.damage > 0
            }) =>
        {
            2
        }
        "Avalanche" if field::weather(c, Some(a)) == Some(field::Weather::Snow) => 2,
        "Bolt Beak" | "Fishious Rend" if a.actions == 1 => 2,
        "Fusion Bolt"
            if state(c)
                .history
                .iter()
                .any(|h| h.name == "Fusion Flare" && h.round == round) =>
        {
            4
        }
        "Fusion Flare"
            if state(c)
                .history
                .iter()
                .any(|h| h.name == "Fusion Bolt" && h.round == round) =>
        {
            4
        }
        "Magnitude" => {
            *state(c)
                .random
                .get(&seq)
                .ok_or_else(|| error("Host-roll Magnitude's power before rolling damage"))?
                - 1
        }
        "Triple Axel" => flow(c)?
            .queue
            .first()
            .map_or(0, |q| q.special_index.saturating_sub(1).min(2) as i32),
        "Triple Kick" if flow(c)?.queue.first().is_some_and(|q| q.special_index == 3) => 1,
        "Ice Ball" | "Rollout"
            if flow(c)?.queue.first().is_some_and(|q| q.special_index == 5)
                && state(c).series.as_ref().is_some_and(|s| s.landed == 4) =>
        {
            1
        }
        _ => 0,
    })
}
pub fn move_data(c: &Campaign, p: &Combatant, m: &mut serde_json::Value) {
    if !enabled(c) {
        return;
    }
    if m["Type"] == "Fighting" {
        m["Type"] = serde_json::json!("Fight");
    }
    let name = m["Name"].as_str().unwrap_or("").to_owned();
    if name == "Bide" && state(c).bides.get(&p.id).is_none_or(|b| b.received < 2) {
        m["Category"] = serde_json::json!("Support");
    }
    if name == "Present"
        || (name == "Pollen Puff"
            && flow(c)
                .ok()
                .and_then(|f| f.queue.first())
                .and_then(|q| fighter(c, &q.target_id).ok())
                .is_some_and(|t| t.side == p.side))
    {
        m["Category"] = serde_json::json!("Support");
    }
    if name == "Triple Axel" {
        let index = state(c)
            .series
            .as_ref()
            .filter(|s| s.preparing && s.actor_id == p.id)
            .map_or(1, |s| p.actions - s.start_actions + 1);
        m["Attributes"]["AccuracyReduction"] = serde_json::json!(-(index.saturating_sub(1) as i32));
    }
    if ["Ice Ball", "Rollout"].contains(&name.as_str())
        && state(c)
            .effects
            .iter()
            .any(|e| e.name == "Defense Curl" && e.target == p.id)
    {
        m["Power"] = serde_json::json!(2);
    }
    if name == "Magnitude" {
        m["Power"] = serde_json::json!(1);
    }
    if m["Category"] != "Support"
        && state(c)
            .effects
            .iter()
            .any(|e| e.name == "Triple Arrows" && e.target == p.id)
    {
        m["Attributes"]["HighCritical"] = serde_json::json!(true);
    }
    if name == "Sparkling Aria"
        && flow(c)
            .ok()
            .and_then(|f| f.queue.first())
            .and_then(|q| fighter(c, &q.target_id).ok())
            .is_some_and(|t| {
                t.runtime
                    .statuses
                    .iter()
                    .any(|s| s.kind.starts_with("Burn"))
            })
    {
        m["Category"] = serde_json::json!("Support");
    }
}
pub fn hit(
    c: &mut Campaign,
    q: &QueuedAction,
    m: &serde_json::Value,
    dice: u32,
    damage: i32,
    cat: &Catalog,
) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    if decoys::intercepted(c, q) {
        if let Some(series) = state_mut(c).series.as_mut() {
            if series.actor_id == q.actor_id && series.name == q.move_name {
                series.landed += 1;
            }
        }
        return Ok(());
    }
    let round = battle(c)?.round;
    let turn = flow(c)?.turn;
    let group = house::enabled(c);
    for (id, bide) in &mut state_mut(c).bides {
        if group && *id == q.target_id && q.actor_id != q.target_id {
            bide.target_id = q.actor_id.clone();
        }
        if bide.received < 2 {
            if bide.basis_id == q.target_id {
                bide.damage += damage;
            }
            if *id == q.target_id {
                bide.received += 1;
            }
        }
    }
    state_mut(c).history.push(Hit {
        source: q.actor_id.clone(),
        target: q.target_id.clone(),
        name: q.move_name.clone(),
        category: m["Category"].as_str().unwrap_or("").into(),
        dice,
        damage,
        round,
        turn,
    });
    if let Some(s) = state_mut(c).series.as_mut() {
        if s.actor_id == q.actor_id && s.name == q.move_name {
            s.landed += 1;
        }
    }
    if q.move_name == "Retaliate" {
        if let Some(h) = state(c)
            .history
            .iter()
            .find(|h| {
                h.name == "[fainted]"
                    && h.round + 1 >= round
                    && registered(c, &h.target)
                        .is_ok_and(|p| p.side == fighter(c, &q.actor_id).unwrap().side)
                    && !fighter(c, &q.actor_id)
                        .unwrap()
                        .runtime
                        .ability
                        .flags
                        .contains(&format!("Retaliate:{}:{}", h.target, h.round))
            })
            .cloned()
        {
            fighter_mut(c, &q.actor_id)?
                .runtime
                .ability
                .flags
                .insert(format!("Retaliate:{}:{}", h.target, h.round));
        }
    }
    if q.move_name == "Shadow Force" {
        add_effect(c, "Shadow Force", &q.actor_id, &q.target_id, None)?;
        if let Some(e) = state_mut(c)
            .effects
            .iter_mut()
            .find(|e| e.name == "Shadow Force" && e.target == q.target_id)
        {
            e.amount += damage;
        }
    }
    if let Some(charge) = state(c).charges.get(&q.target_id).cloned() {
        if charge.action.move_name == "Beak Blast"
            && abilities::contact(m)
            && !items::completion::pads(c, &q.actor_id)
        {
            abilities::inflict(c, &q.target_id, &q.actor_id, "Burn 2", "", None, cat)?;
        }
        if charge.action.move_name == "Focus Punch" && fighter(c, &q.target_id)?.hp > 0 {
            state_mut(c).charges.remove(&q.target_id);
            abilities::inflict(c, &q.actor_id, &q.target_id, "Flinch", "", None, cat)?;
        }
    }
    Ok(())
}
pub fn end_round(c: &mut Campaign) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let effects = state(c).effects.clone();
    for e in effects {
        if ["Aqua Ring", "Ingrain", "Life Dew"].contains(&e.name.as_str()) {
            heal(c, &e.target, 1, false)?;
        }
    }
    Ok(())
}
pub fn vulnerable(c: &Campaign, id: &str, name: &str) -> bool {
    if !enabled(c) {
        return false;
    }
    charge_for(c, id).is_some_and(|q| match q.action.move_name.as_str() {
        "Fly" | "Bounce" | "Sky Drop" => [
            "Thunder",
            "Twister",
            "Sky Uppercut",
            "Gust",
            "Hurricane",
            "Smack Down",
        ]
        .contains(&name),
        "Dig" => [
            "Earth Power",
            "Earthquake",
            "Fissure",
            "Magnitude",
            "Stomping Tantrum",
        ]
        .contains(&name),
        "Dive" => ["Scald", "Surf", "Whirlpool", "Freeze-Dry", "Sheer Cold"].contains(&name),
        _ => false,
    })
}
pub fn target_guard(c: &Campaign, source: &str, id: &str, m: &serde_json::Value) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let name = m["Name"].as_str().unwrap_or("");
    if name == "Dream Eater" && !status(fighter(c, id)?, "Sleep") {
        return Err(error("Dream Eater requires a sleeping target"));
    }
    if !fighter(c, source).is_ok_and(|p| reviewed::extreme(c, p))
        && charge_for(c, id).is_some_and(|q| {
            [
                "Fly",
                "Bounce",
                "Sky Drop",
                "Dig",
                "Dive",
                "Phantom Force",
                "Shadow Force",
            ]
            .contains(&q.action.move_name.as_str())
        })
        && !vulnerable(c, id, name)
    {
        return Err(error("Target is out of range while charging this move"));
    }
    Ok(())
}
pub fn reaction_guard(c: &Campaign, id: &str, kind: &str, name: &str, cat: &Catalog) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    if ["evade", "clash"].contains(&kind) && state(c).bides.get(id).is_some_and(|b| b.received < 2)
    {
        return Err(error(
            "Bide forgoes Evasion and Clash against the next two attacks",
        ));
    }
    if let Ok(q) = incoming(c) {
        if vulnerable(c, id, &q.move_name) {
            return Err(error(
                "A vulnerable charging target cannot react to this move",
            ));
        }
    }
    if kind == "evade" && state(c).rampages.contains_key(id) {
        return Err(error(
            "End the rampage and resolve Confusion before Evasion",
        ));
    }
    if kind == "clash"
        && state(c)
            .charges
            .get(id)
            .is_some_and(|q| q.action.move_name != name)
    {
        return Err(error("While charging, Clash must use the charged move"));
    }
    if kind == "move" && move_entry(cat, name)?["Attributes"]["Charge"] == true {
        return Err(AppError::new(
            "manual_rule",
            "A charge reaction other than Clash requires an explicit GM ruling",
        ));
    }
    Ok(())
}
pub fn after_reaction(c: &mut Campaign, id: &str, kind: &str, cat: &Catalog) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    if ["evade", "clash"].contains(&kind) {
        fighter_mut(c, id)?
            .runtime
            .ability
            .counters
            .remove("special:echo");
    }
    if kind == "clash" {
        state_mut(c).charges.remove(id);
        if let Some(r) = state_mut(c).rampages.get_mut(id) {
            r.uses += 1;
        }
        if state(c).rampages.get(id).is_some_and(|r| r.uses >= 3) && flow(c)?.queue.is_empty() {
            end_rampage(c, id, cat)?;
        }
    }
    Ok(())
}
pub fn blocked(c: &Campaign, id: &str) -> bool {
    enabled(c)
        && state(c)
            .effects
            .iter()
            .any(|e| e.name == "Ingrain" && e.target == id)
        && registered(c, id).is_ok_and(|p| !p.has_ability("Run Away"))
}
pub fn low_accuracy(c: &Campaign, id: &str) -> i32 {
    if enabled(c)
        && state(c)
            .effects
            .iter()
            .any(|e| e.name == "Lunar accuracy" && e.target == id)
    {
        1
    } else {
        0
    }
}
pub fn clear_modifiers(c: &mut Campaign, id: &str) {
    if stat_channels::enabled(c) {
        flow_mut(c)
            .stat_channels
            .grants
            .retain(|g| g.target != id || g.channel != "move");
    }
    if enabled(c) {
        state_mut(c).effects.retain(|e| {
            e.target != id || (!e.name.starts_with("buff:") && !e.name.starts_with("debuff:"))
        });
    }
}
pub fn record_modifier(c: &mut Campaign, source: &str, id: &str, key: &str, n: i32) -> Result<i32> {
    let positive = format!("buff:{key}");
    let negative = format!("debuff:{key}");
    let previous = *fighter(c, id)?.runtime.modifiers.get(key).unwrap_or(&0);
    if !state(c)
        .effects
        .iter()
        .any(|e| e.target == id && (e.name == positive || e.name == negative))
        && previous != 0
    {
        let name = if previous > 0 { &positive } else { &negative };
        add_effect(c, name, "legacy", id, None)?;
        state_mut(c).effects.last_mut().unwrap().amount = previous;
    }
    let name = if n > 0 { &positive } else { &negative };
    if let Some(e) = state_mut(c)
        .effects
        .iter_mut()
        .find(|e| e.target == id && e.name == *name)
    {
        e.amount = if n > 0 {
            e.amount.max(n)
        } else {
            e.amount.min(n)
        };
    } else {
        add_effect(c, name, source, id, None)?;
        state_mut(c).effects.last_mut().unwrap().amount = n;
    }
    let values: Vec<_> = state(c)
        .effects
        .iter()
        .filter(|e| e.target == id && (e.name == positive || e.name == negative))
        .map(|e| e.amount)
        .collect();
    Ok(values.iter().copied().max().unwrap_or(0).max(0)
        + values.iter().copied().min().unwrap_or(0).min(0))
}
pub fn modifier(c: &Campaign, p: &Combatant, key: &str) -> Option<i32> {
    let item = items::stat_reduction(c, p, key);
    let item_up = items::stat_increase(c, p, key);
    if !enabled(c) {
        return (item != 0 || item_up != 0).then(|| {
            let amount = p.runtime.modifiers.get(key).copied().unwrap_or(0);
            let amount = if extra::enabled(c) {
                match p.ability_for(&["Contrary", "Simple"]) {
                    "Contrary" => -amount,
                    "Simple" => amount + amount.signum(),
                    _ => amount,
                }
            } else {
                amount
            };
            amount.max(item_up) + amount.min(item)
        });
    }
    let names = [format!("buff:{key}"), format!("debuff:{key}")];
    let mut values: Vec<_> = state(c)
        .effects
        .iter()
        .filter(|e| e.target == p.id && names.contains(&e.name))
        .map(|e| match p.ability_for(&["Contrary", "Simple"]) {
            "Contrary" => -e.amount,
            "Simple" => e.amount + e.amount.signum(),
            _ => e.amount,
        })
        .collect();
    if key == "Defense"
        && state(c)
            .charges
            .get(&p.id)
            .is_some_and(|q| q.action.move_name == "Skull Bash")
    {
        values.push(match p.ability_for(&["Contrary", "Simple"]) {
            "Contrary" => -1,
            "Simple" => 2,
            _ => 1,
        });
    }
    let copied = stat_channels::move_values(c, p, key);
    if values.is_empty() && (!copied.is_empty() || item != 0 || item_up != 0) {
        if let Some(amount) = p.runtime.modifiers.get(key) {
            values.push(match p.ability_for(&["Contrary", "Simple"]) {
                "Contrary" => -*amount,
                "Simple" => amount + amount.signum(),
                _ => *amount,
            });
        }
    }
    values.extend(copied);
    if item != 0 {
        values.push(item);
    }
    if item_up != 0 {
        values.push(item_up);
    }
    if values.is_empty() {
        None
    } else {
        Some(
            values.iter().copied().max().unwrap_or(0).max(0)
                + values.iter().copied().min().unwrap_or(0).min(0),
        )
    }
}
pub fn guard(c: &Campaign, id: &str, m: &serde_json::Value, cat: &Catalog) -> Result<()> {
    items::guard_move(c, id, m, cat)?;
    if !enabled(c) {
        return Ok(());
    }
    let p = fighter(c, id)?;
    if state(c)
        .effects
        .iter()
        .any(|e| e.name == "Roar Of Time" && e.target == id)
        && p.runtime.will < 1
    {
        return Err(error(
            "Roar Of Time requires one Will for each attempted action",
        ));
    }
    let name = m["Name"].as_str().unwrap_or("");
    if decoys::enabled(c)
        && (m["Attributes"]["MaxMove"] == true || m["Attributes"]["ZMove"] == true)
    {
        return Err(AppError::new(
            "out_of_scope",
            "Dynamax, Max Moves (including Max Guard) and Z-Moves are out of scope",
        ));
    }
    if name == "Pain Split" && p.hp > p.max_hp / 2 {
        return Err(error("Pain Split fails above half HP"));
    }
    if ["Spit Up", "Swallow"].contains(&name)
        && !state(c)
            .effects
            .iter()
            .any(|e| e.name == "Stockpile" && e.target == id)
    {
        return Err(error("This move requires Stockpile"));
    }
    if ["Bouncy Bubble", "Veevee Volley", "Sparkly Swirl"].contains(&name) {
        let species = cat
            .find(
                "species",
                "AppId",
                c.sheets[&p.sheet_id].species_id.as_deref().unwrap_or(""),
            )
            .ok_or_else(|| error("Evolution-stage metadata needs adjudication"))?;
        if species["Evolutions"]
            .as_array()
            .is_none_or(|v| v.is_empty())
        {
            return Err(error(
                "This partner move fails at its user's final evolution stage",
            ));
        }
    }
    Ok(())
}
pub fn preview_notes(c: &Campaign, a: &Action, cat: &Catalog) -> Result<Vec<String>> {
    if !enabled(c) {
        return Ok(vec![]);
    }
    let mut notes = vec![];
    match a {
        Action::SpecialMove{command:SpecialCommand::Successive{attempts,..},..}=>notes.push(format!("Commit {attempts} actions and independent accuracy rolls now. All hits retain one turn and one Evasion/reaction slot; each incoming attack may still be Clashed.")),
        Action::Accuracy{combatant_id,move_name,..}|Action::AccuracyTargets{combatant_id,move_name,..}=>{let m=move_entry(cat,move_name)?;if m["Attributes"]["Charge"]==true{notes.push("Charge accuracy is stored. Release spends another action on a later user turn; reactions open only on release. Reviewed weather may skip or extend preparation.".into());}
 if m["Attributes"]["Rampage"]==true{notes.push("Rampage permits up to three uses this round. Ending it inflicts Confusion; Evasion requires ending it first.".into());}
 if move_name=="Bide" && !house::enabled(c){notes.push(format!("Bide requires a recorded GM choice for its ambiguous damage basis; {} then forgoes Evasion/Clash against the next two attacks",fighter(c,combatant_id)?.name));}},
        Action::Damage{combatant_id,pool_modifier,..}=>{let q=current(c)?;let mut temp=old(c);install_pending(&mut temp,q);let pool=rules::damage_pool(&temp,combatant_id,q.sequence,*pool_modifier,cat)?;let m=field::move_data(c,fighter(c,combatant_id)?,&move_entry(cat,&q.move_name)?);if formula_name(&q.move_name){notes.push(format!("{}: literal card formula, {} dice, effectiveness {}, immunity {}; set/varied pools bypass ordinary attribute/defense calculations",q.move_name,pool.dice,pool.effectiveness,pool.immune));}else{let n=q.damage_context.as_ref().and_then(|ctx|ctx.special_bonus.get(&q.target_id)).copied().unwrap_or(bonus(c,fighter(c,combatant_id)?,fighter(c,&q.target_id)?,&m,cat)?);if n!=0{notes.push(format!("{} conditional/successive damage modifier: {n:+} dice",q.move_name));}}},
        _=>{},
    }
    Ok(notes)
}
pub fn ally_shield(m: &serde_json::Value) -> bool {
    ["Wide Guard", "Quick Guard", "Crafty Shield", "Mat Block"]
        .contains(&m["Name"].as_str().unwrap_or(""))
}
pub fn allies(c: &Campaign, id: &str) -> Result<Vec<String>> {
    let p = fighter(c, id)?;
    let mut ids: Vec<_> = battle(c)?
        .combatants
        .iter()
        .filter(|v| {
            v.deployed
                && v.hp > 0
                && v.side == p.side
                && (v.id == id
                    || flow(c)
                        .unwrap()
                        .ranges
                        .get(id)
                        .is_none_or(|range| range.contains(&v.id)))
        })
        .map(|p| p.id.clone())
        .collect();
    ids.sort_by_key(|v| (v != id, v.clone()));
    ids.truncate(
        [1, 2, 3, 4, 5, 6, 8, 10][crate::creation::rank(&c.sheets[&p.sheet_id].rank).unwrap_or(0)],
    );
    Ok(ids)
}
pub fn ticks_pending(c: &Campaign) -> bool {
    enabled(c)
        && state(c).effects.iter().any(|e| {
            e.name == "Leech Seed"
                && e.started < battle(c).unwrap().round
                && registered(c, &e.target).is_ok_and(|p| p.hp > 0)
        })
}
pub fn expire(c: &mut Campaign) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    for e in &mut state_mut(c).effects {
        if e.name != "Wish" {
            if let Some(n) = &mut e.rounds {
                *n = n.saturating_sub(1);
            }
        }
    }
    state_mut(c).effects.retain(|e| e.rounds != Some(0));
    Ok(())
}
