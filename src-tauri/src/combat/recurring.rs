//! M2-D: saved encounter clocks and host-selected sleep actions.
use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct RecurringState {
    pub enabled: bool,
    pub yawn: BTreeMap<String, Deadline>,
    pub nightmares: BTreeMap<String, String>,
    pub perish: BTreeMap<String, Deadline>,
    pub malignant: BTreeMap<String, Chain>,
    pub pending_talk: Option<TalkDraw>,
    pub executing: Option<TalkDraw>,
    pub ended_round: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Deadline {
    pub source: String,
    pub due_round: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Chain {
    pub source: String,
    pub original_side: String,
    pub allied_side: String,
    pub cast_round: u32,
    pub activates: Option<u32>,
    pub active: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct TalkDraw {
    pub actor: String,
    pub move_name: String,
    pub foe: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export, export_to = "../../src/bindings/")]
pub enum RecurringCommand {
    Loyalty { combatant_id: String },
    Draw { combatant_id: String },
    UseCopy { combatant_id: String },
    Discard { combatant_id: String },
}
pub fn enabled(c: &Campaign) -> bool {
    flow(c).is_ok_and(|f| f.recurring.enabled)
}
fn subject<'a>(c: &'a Campaign, id: &str) -> Result<&'a Combatant> {
    battle(c)?
        .combatants
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| error("Missing recurring subject"))
}
fn state(c: &Campaign) -> &RecurringState {
    &flow(c).unwrap().recurring
}
fn state_mut(c: &mut Campaign) -> &mut RecurringState {
    &mut flow_mut(c).recurring
}
pub fn chance(c: &Campaign, name: &str) -> Option<&'static chance_status::Chance> {
    if !enabled(c) {
        return None;
    }
    static R: std::sync::LazyLock<BTreeMap<String, chance_status::Chance>> =
        std::sync::LazyLock::new(|| {
            let r: serde_json::Value =
                serde_json::from_str(include_str!("../../../rules/recurring-status-moves.json"))
                    .unwrap();
            serde_json::from_value(r["chance_moves"].clone()).unwrap()
        });
    R.get(name)
}
pub fn copy_matches(c: &Campaign, id: &str, name: &str) -> bool {
    enabled(c)
        && state(c)
            .executing
            .as_ref()
            .is_some_and(|s| s.actor == id && s.move_name == name)
}
pub fn sleep_allowed(c: &Campaign, id: &str, name: &str) -> bool {
    enabled(c)
        && fighter(c, id).is_ok_and(|p| status(p, "Sleep"))
        && (["Snore", "Sleep Talk"].contains(&name) || copy_matches(c, id, name))
}
pub fn guard(c: &Campaign, id: &str, name: &str) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    if ["Snore", "Sleep Talk"].contains(&name) && !status(fighter(c, id)?, "Sleep") {
        return Err(error("This move fails unless its user is asleep"));
    }
    if name == "Sleep Talk" {
        return Err(AppError::new(
            "manual_rule",
            "Use the host Sleep Talk draw before resolving its selected move",
        ));
    }
    Ok(())
}
fn loyalty(c: &Campaign, id: &str) -> Result<u32> {
    let p = subject(c, id)?;
    c.sheets[&p.sheet_id]
        .workflow
        .as_ref()
        .map(|w| w.loyalty)
        .filter(|n| (0..=5).contains(n))
        .map(|n| n as u32)
        .ok_or_else(|| {
            AppError::new(
                "manual_rule",
                "Record the target's reviewed Loyalty before resolving Malignant Chain",
            )
        })
}
pub fn pending(c: &Campaign) -> bool {
    enabled(c)
        && state(c).malignant.iter().any(|(id, m)| {
            m.activates.is_none()
                && m.cast_round <= battle(c).unwrap().round
                && subject(c, id).is_ok_and(|p| {
                    p.hp > 0 && p.runtime.statuses.iter().any(|s| s.kind.contains("Poison"))
                })
        })
}
pub fn phase(c: &Campaign, a: &Action) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    if state(c).pending_talk.is_some()
        && !matches!(
            a,
            Action::RecurringMove { .. }
                | Action::RollbackBattle { .. }
                | Action::Override { .. }
                | Action::SetStatus { .. }
                | Action::ResolveAbility { .. }
                | Action::Battlefield { .. }
                | Action::SpecialMove { .. }
                | Action::CompoundMove { .. }
        )
    {
        return Err(error("Resolve the recorded Sleep Talk draw, or have the GM record its failure, before continuing"));
    }
    if pending(c)
        && flow(c)?.battlefield.end_round == battle(c)?.round
        && !matches!(
            a,
            Action::RecurringMove {
                command: RecurringCommand::Loyalty { .. },
                ..
            } | Action::NextRound
                | Action::RollbackBattle { .. }
                | Action::Override { .. }
                | Action::SetStatus { .. }
                | Action::ResolveAbility { .. }
                | Action::Battlefield { .. }
                | Action::SpecialMove { .. }
        )
    {
        return Err(error(
            "Resolve Malignant Chain's round-end Loyalty check first",
        ));
    }
    Ok(())
}
fn draw_guard(c: &Campaign, id: &str, cat: &Catalog) -> Result<()> {
    if !enabled(c) {
        return Err(error("Sleep-action rules require a new encounter"));
    }
    turn(c, id)?;
    if state(c).pending_talk.is_some() || state(c).executing.is_some() {
        return Err(error("A Sleep Talk draw is already committed"));
    }
    if !status(fighter(c, id)?, "Sleep") {
        return Err(error("Sleep Talk requires Sleep"));
    }
    special::ready_move(c, id, "Sleep Talk")?;
    let s = rules::base_sheet(c, fighter(c, id)?)?;
    if !s.moves.iter().any(|m| m == "Sleep Talk") {
        return Err(error("The user has not learned Sleep Talk"));
    }
    let (moves, foes) = draw_options(c, id)?;
    if moves.is_empty() || moves.len() > 216 || foes.is_empty() || foes.len() > 216 {
        return Err(error(
            "Sleep Talk needs a supported moveset size and an available foe in range",
        ));
    }
    // Resolve references only after selection; an unsupported drawn card must
    // stay committed instead of silently changing the probability distribution.
    let _ = cat;
    Ok(())
}
fn draw_options(c: &Campaign, id: &str) -> Result<(Vec<String>, Vec<String>)> {
    let p = fighter(c, id)?;
    let mut moves = rules::base_sheet(c, p)?.moves;
    moves.sort();
    moves.dedup();
    let mut foes: Vec<_> = battle(c)?
        .combatants
        .iter()
        .filter(|t| t.deployed && t.hp > 0 && t.side != p.side && reviewed::in_range(c, p, t))
        .map(|t| t.id.clone())
        .collect();
    foes.sort();
    Ok((moves, foes))
}
fn copy_input(c: &Campaign, id: &str, cat: &Catalog) -> Result<(Campaign, Action)> {
    let draw = state(c)
        .pending_talk
        .as_ref()
        .filter(|s| s.actor == id)
        .cloned()
        .ok_or_else(|| error("No committed Sleep Talk draw for this user"))?;
    if !status(fighter(c, id)?, "Sleep") {
        return Err(error("The Sleep Talk user is no longer asleep"));
    }
    if draw.move_name == "Sleep Talk" {
        return Err(AppError::new("manual_rule","Sleep Talk selected itself; recursive execution is undefined. GM must record the drawn attempt's outcome"));
    }
    let m = move_entry(cat, &draw.move_name)?;
    let action = if m["Target"] == "Self" {
        Action::Accuracy {
            combatant_id: id.into(),
            target_id: Some(id.into()),
            move_name: draw.move_name.clone(),
            pool_modifier: 0,
            success_modifier: 0,
            reason: "Sleep Talk selected self-target move".into(),
        }
    } else if [
        "Area",
        "Battlefield",
        "Ally's Battlefield",
        "Foe's Battlefield",
    ]
    .contains(&m["Target"].as_str().unwrap_or(""))
    {
        Action::AccuracyTargets {
            combatant_id: id.into(),
            target_ids: vec![],
            move_name: draw.move_name.clone(),
            pool_modifier: 0,
            success_modifier: 0,
            reason: "Sleep Talk selected area/field move".into(),
        }
    } else {
        Action::Accuracy {
            combatant_id: id.into(),
            target_id: Some(draw.foe.clone()),
            move_name: draw.move_name.clone(),
            pool_modifier: 0,
            success_modifier: 0,
            reason: "Sleep Talk selected random foe".into(),
        }
    };
    let mut input = c.clone();
    state_mut(&mut input).pending_talk = None;
    state_mut(&mut input).executing = Some(draw);
    Ok((input, action))
}
pub fn dice(c: &Campaign, command: &RecurringCommand, cat: &Catalog) -> Result<u32> {
    if !enabled(c) {
        return Err(error("Recurring move rules require a new encounter"));
    }
    match command {
        RecurringCommand::Loyalty { combatant_id } => {
            if !flow(c)?.queue.is_empty()
                || flow(c)?.battlefield.end_round != battle(c)?.round
                || state(c)
                    .malignant
                    .get(combatant_id)
                    .is_none_or(|m| m.activates.is_some())
            {
                return Err(error("No pending round-end Malignant Chain check"));
            }
            loyalty(c, combatant_id)
        }
        RecurringCommand::Draw { combatant_id } => {
            draw_guard(c, combatant_id, cat)?;
            Ok(6)
        }
        RecurringCommand::UseCopy { combatant_id } => {
            let (input, action) = copy_input(c, combatant_id, cat)?;
            super::dice_count(&input, &action, cat)
        }
        RecurringCommand::Discard { combatant_id } => {
            if state(c)
                .pending_talk
                .as_ref()
                .is_none_or(|d| d.actor != *combatant_id)
            {
                return Err(error("No drawn Sleep Talk attempt to adjudicate"));
            }
            Ok(0)
        }
    }
}
pub fn sample(dice: &[u8], size: usize) -> Option<usize> {
    let n = (dice[0] as usize - 1) * 36 + (dice[1] as usize - 1) * 6 + dice[2] as usize - 1;
    (size > 0 && n < 216 / size * size).then(|| n % size)
}
pub fn transition(
    c: &Campaign,
    actor: &str,
    command: &RecurringCommand,
    why: &str,
    faces: &[u8],
    cat: &Catalog,
) -> Result<(Campaign, String)> {
    reason(why)?;
    check_count(faces, dice(c, command, cat)?)?;
    let mut next = c.clone();
    let description = match command {
        RecurringCommand::Loyalty { combatant_id } => {
            if actor != c.gm_id && subject(c, combatant_id)?.owner_id != actor {
                return Err(AppError::new(
                    "permission",
                    "Only control your own combatant",
                ));
            }
            if rules::successes(faces) >= 3 {
                state_mut(&mut next).malignant.remove(combatant_id);
                "Malignant Chain Loyalty passed; no allegiance change".into()
            } else {
                state_mut(&mut next)
                    .malignant
                    .get_mut(combatant_id)
                    .unwrap()
                    .activates = Some(battle(c)?.round + 1);
                "Malignant Chain Loyalty failed; allegiance changes next round until Poison is cured".into()
            }
        }
        RecurringCommand::Draw { combatant_id } => {
            owner(c, actor, combatant_id)?;
            let (moves, foes) = draw_options(c, combatant_id)?;
            match (sample(&faces[..3], moves.len()), sample(&faces[3..], foes.len())) {
                (Some(m), Some(t)) => {
                    let draw = TalkDraw {
                        actor: combatant_id.clone(),
                        move_name: moves[m].clone(),
                        foe: foes[t].clone(),
                    };
                    let log = format!("Sleep Talk committed {} against {}; accuracy/action not spent yet", draw.move_name, fighter(c, &draw.foe)?.name);
                    state_mut(&mut next).pending_talk = Some(draw);
                    log
                }
                _ => "Sleep Talk rejected an out-of-range d6 sample; reroll without spending an action".into(),
            }
        }
        RecurringCommand::UseCopy { combatant_id } => {
            owner(c, actor, combatant_id)?;
            let (mut input, action) = copy_input(c, combatant_id, cat)?;
            fighter_mut(&mut input, combatant_id)?
                .used_moves
                .push("Sleep Talk".into());
            let (mut done, log) = super::transition(&input, actor, &action, faces.to_vec(), cat)?;
            done.battle_undo = c.battle_undo.clone();
            next = done;
            format!("Sleep Talk: {log}")
        }
        RecurringCommand::Discard { combatant_id } => {
            gm(c, actor)?;
            turn(c, combatant_id)?;
            state_mut(&mut next).pending_talk = None;
            let p = fighter_mut(&mut next, combatant_id)?;
            p.actions += 1;
            p.used_moves.push("Sleep Talk".into());
            advance(&mut next);
            "GM recorded the committed Sleep Talk attempt as failed; one action spent".into()
        }
    };
    ability_changes::lifecycle(c, &mut next, cat)?;
    tactics::lifecycle(c, &mut next, cat)?;
    field::lifecycle(c, &mut next, cat)?;
    abilities::lifecycle(c, &mut next)?;
    extra::lifecycle(c, &mut next, cat)?;
    special::lifecycle(c, &mut next, cat)?;
    lifecycle(&mut next)?;
    sync_labels(&mut next);
    let log = format!("{description}. {why}");
    checkpoint(c, &mut next, &log);
    Ok((next, log))
}
pub fn effect_dice(c: &Campaign, q: &QueuedAction, cat: &Catalog) -> Result<Option<u32>> {
    if !enabled(c) {
        return Ok(None);
    }
    match q.move_name.as_str() {
        "Malignant Chain" => {
            loyalty(c, &q.target_id)?;
            chance_status::effect_dice(c, q, cat)
        }
        "Snore" => {
            guard(c, &q.actor_id, "Snore")?;
            chance_status::effect_dice(c, q, cat)
        }
        "Yawn" | "Nightmare" | "Perish Song" | "Sleep Talk" => Ok(Some(0)),
        _ => Ok(None),
    }
}
pub fn effect(c: &mut Campaign, q: &QueuedAction, dice: &[u8], cat: &Catalog) -> Result<bool> {
    if !enabled(c) {
        return Ok(false);
    }
    let name = q.move_name.as_str();
    if name == "Sleep Talk" {
        return Err(AppError::new(
            "manual_rule",
            "Use the host Sleep Talk selection and its ordinary copied move",
        ));
    }
    if name == "Malignant Chain" || name == "Snore" {
        guard(c, &q.actor_id, name)?;
        return chance_status::effect(c, q, dice, cat);
    }
    if !["Yawn", "Nightmare", "Perish Song"].contains(&name) {
        return Ok(false);
    }
    let m = move_entry(cat, name)?;
    let source = reviewed::source(c, q)?;
    let t = fighter(c, &q.target_id)?;
    if q.ice
        || special::target_effects_blocked(c, q, &m)
        || (!reviewed::extreme(c, &source)
            && (extra::effects_immune(c, t, &m) || abilities::move_immune(c, t, &m)))
    {
        return Ok(true);
    }
    let due = battle(c)?.round + if name == "Perish Song" { 3 } else { 1 };
    if name == "Nightmare" {
        if !status(t, "Sleep") {
            return Err(error("Nightmare requires a sleeping target"));
        }
        state_mut(c)
            .nightmares
            .entry(q.target_id.clone())
            .or_insert(q.actor_id.clone());
    } else {
        let map = if name == "Yawn" {
            &mut state_mut(c).yawn
        } else {
            &mut state_mut(c).perish
        };
        map.entry(q.target_id.clone()).or_insert(Deadline {
            source: q.actor_id.clone(),
            due_round: due,
        });
    }
    Ok(true)
}
pub fn on_status(c: &mut Campaign, id: &str, kind: &str, source: Option<&str>) -> Result<()> {
    if !enabled(c) || !kind.contains("Poison") {
        return Ok(());
    }
    let Some(q) = flow(c)?
        .queue
        .first()
        .filter(|q| {
            q.move_name == "Malignant Chain"
                && q.target_id == id
                && source == Some(q.actor_id.as_str())
        })
        .cloned()
    else {
        return Ok(());
    };
    loyalty(c, id)?;
    let entry = Chain {
        source: q.actor_id.clone(),
        original_side: fighter(c, id)?.side.clone(),
        allied_side: fighter(c, &q.actor_id)?.side.clone(),
        cast_round: battle(c)?.round,
        activates: None,
        active: false,
    };
    state_mut(c).malignant.entry(id.into()).or_insert(entry);
    Ok(())
}
pub fn sleep_attempt(c: &mut Campaign, id: &str) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    if fighter(c, id)?.has_ability("Comatose") && !status(fighter(c, id)?, "Sleep") {
        fighter_mut(c, id)?.runtime.statuses.push(Status {
            kind: "Sleep".into(),
            detail: "Comatose".into(),
            rounds: None,
            progress: 0,
            elapsed_rounds: 0,
        });
    }
    if state(c).nightmares.contains_key(id) && status(fighter(c, id)?, "Sleep") {
        special::indirect(c, id, 1)?;
    }
    lifecycle(c)
}
pub fn end_round(c: &mut Campaign) -> Result<()> {
    if !enabled(c) || state(c).ended_round == battle(c)?.round {
        return Ok(());
    }
    lifecycle(c)?;
    let round = battle(c)?.round;
    state_mut(c).ended_round = round;
    let due: Vec<_> = state(c)
        .perish
        .iter()
        .filter(|(_, e)| e.due_round <= round)
        .map(|(id, _)| id.clone())
        .collect();
    for id in due {
        let hp = fighter(c, &id)?.hp;
        special::indirect(c, &id, hp)?;
        state_mut(c).perish.remove(&id);
    }
    lifecycle(c)
}
pub fn round_start(c: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    lifecycle(c)?;
    let round = battle(c)?.round;
    let due: Vec<_> = state(c)
        .yawn
        .iter()
        .filter(|(_, e)| e.due_round <= round)
        .map(|(id, e)| (id.clone(), e.source.clone()))
        .collect();
    for (id, source) in due {
        state_mut(c).yawn.remove(&id);
        match set_status_from(c, &id, "Sleep", "Yawn", None, cat, Some(&source)) {
            Err(e)
                if [
                    "Species type is immune to this condition",
                    "Selected ability is immune to this condition",
                    "Battlefield prevents this condition",
                ]
                .contains(&e.message.as_str()) => {}
            other => other?,
        };
    }
    let changes: Vec<_> = state(c)
        .malignant
        .iter()
        .filter(|(_, e)| !e.active && e.activates.is_some_and(|r| r <= round))
        .map(|(id, e)| (id.clone(), e.allied_side.clone()))
        .collect();
    for (id, side) in changes {
        fighter_mut(c, &id)?.side = side;
        state_mut(c).malignant.get_mut(&id).unwrap().active = true;
    }
    Ok(())
}
pub fn lifecycle(c: &mut Campaign) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let active: BTreeSet<_> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    let sleeping: BTreeSet<_> = battle(c)?
        .combatants
        .iter()
        .filter(|p| active.contains(&p.id) && status(p, "Sleep"))
        .map(|p| p.id.clone())
        .collect();
    state_mut(c).yawn.retain(|id, _| active.contains(id));
    state_mut(c).perish.retain(|id, _| active.contains(id));
    state_mut(c)
        .nightmares
        .retain(|id, _| sleeping.contains(id));
    let cleared: Vec<_> = state(c)
        .malignant
        .iter()
        .filter(|(id, _)| {
            subject(c, id)
                .is_ok_and(|p| !p.runtime.statuses.iter().any(|s| s.kind.contains("Poison")))
        })
        .map(|(id, e)| (id.clone(), e.clone()))
        .collect();
    for (id, e) in cleared {
        if e.active && subject(c, &id)?.side == e.allied_side {
            fighter_mut(c, &id)?.side = e.original_side;
        }
        state_mut(c).malignant.remove(&id);
    }
    if state(c)
        .pending_talk
        .as_ref()
        .is_some_and(|d| !sleeping.contains(&d.actor))
    {
        state_mut(c).pending_talk = None;
    }
    if state(c)
        .executing
        .as_ref()
        .is_some_and(|d| !flow(c).unwrap().queue.iter().any(|q| q.actor_id == d.actor))
    {
        state_mut(c).executing = None;
    }
    Ok(())
}
pub fn before_end(c: &mut Campaign) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let entries = state(c).malignant.clone();
    for (id, e) in entries {
        if e.active && subject(c, &id)?.side == e.allied_side {
            fighter_mut(c, &id)?.side = e.original_side;
        }
    }
    Ok(())
}
