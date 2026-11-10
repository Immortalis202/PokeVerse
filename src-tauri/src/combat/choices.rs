//! Saved move-specific decisions; discretionary outcomes still use normal commands.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct HoldHandsDecision {
    pub actor_id: String,
    pub target_id: String,
    pub accuracy: i32,
    pub will_dice: u32,
    pub resistance: Option<i32>,
    pub resisted: Option<bool>,
    pub faces: Vec<u8>,
    pub end_battle: Option<bool>,
    pub reason: String,
}

fn pending(c: &Campaign, sequence: u32) -> Result<QueuedAction> {
    if !special::enabled(c) {
        return Err(error(
            "Adopt special-move rules before using saved move decisions",
        ));
    }
    let q = current(c)?;
    if q.sequence != sequence
        || q.move_name != "Hold Hands"
        || completion::definition(c, "Hold Hands").is_none()
    {
        return Err(error("No matching current Hold Hands decision"));
    }
    Ok(q.clone())
}

fn decision(c: &Campaign, sequence: u32) -> Result<&HoldHandsDecision> {
    pending(c, sequence)?;
    let d = flow(c)?
        .completion
        .hold_hands
        .get(&sequence)
        .ok_or_else(|| error("Prepare the Hold Hands decision first"))?;
    if d.end_battle.is_some() {
        return Err(error("Hold Hands ruling already resolved"));
    }
    Ok(d)
}

pub fn dice(c: &Campaign, cmd: &special::SpecialCommand) -> Result<u32> {
    use special::SpecialCommand::*;
    match cmd {
        PrepareHoldHands { sequence } => {
            pending(c, *sequence)?;
            if flow(c)?.completion.hold_hands.contains_key(sequence) {
                return Err(error("Hold Hands decision already prepared"));
            }
            Ok(0)
        }
        ResistHoldHands { sequence, resist } => {
            let d = decision(c, *sequence)?;
            if d.resisted.is_some() {
                return Err(error("Hold Hands resistance already recorded"));
            }
            Ok(if *resist { d.will_dice } else { 0 })
        }
        RuleHoldHands {
            sequence,
            end_battle,
        } => {
            let d = decision(c, *sequence)?;
            let resisted = d
                .resisted
                .ok_or_else(|| error("Target must choose whether to resist Hold Hands first"))?;
            if *end_battle && resisted {
                return Err(error(
                    "Successful Will resistance prevents Hold Hands from ending the battle",
                ));
            }
            Ok(0)
        }
        _ => Err(error("Not a move decision command")),
    }
}

pub fn command(
    c: &mut Campaign,
    actor: &str,
    cmd: &special::SpecialCommand,
    faces: &[u8],
    why: &str,
    cat: &Catalog,
) -> Result<String> {
    use special::SpecialCommand::*;
    reason(why)?;
    check_count(faces, dice(c, cmd)?)?;
    match cmd {
        PrepareHoldHands { sequence } => {
            let q = pending(c, *sequence)?;
            owner(c, actor, &q.actor_id)?;
            let resolved = tactics::effect_target(c, &q, cat)?.unwrap_or(q);
            tactics::clear_payer(c);
            if resolved.cancelled || completion::immune_target(c, &resolved, cat)? {
                return Err(error(
                    "Hold Hands is blocked; resolve its protected effect normally",
                ));
            }
            let sheet = rules::effective_sheet(c, fighter(c, &resolved.target_id)?)?;
            let will_dice = if sheet.workflow.is_some() {
                rules::score(&sheet, "Will")?
            } else {
                rules::score(&sheet, "Insight")? + 3
            }
            .max(0) as u32;
            flow_mut(c).completion.hold_hands.insert(
                *sequence,
                HoldHandsDecision {
                    actor_id: resolved.actor_id,
                    target_id: resolved.target_id,
                    accuracy: resolved.successes,
                    will_dice,
                    resistance: None,
                    resisted: None,
                    faces: vec![],
                    end_battle: None,
                    reason: why.into(),
                },
            );
        }
        ResistHoldHands { sequence, resist } => {
            let d = decision(c, *sequence)?.clone();
            owner(c, actor, &d.target_id)?;
            let successes = rules::successes(faces);
            let d = flow_mut(c).completion.hold_hands.get_mut(sequence).unwrap();
            d.resistance = resist.then_some(successes);
            d.resisted = Some(*resist && successes >= d.accuracy);
            d.faces = faces.to_vec();
            d.reason = why.into();
        }
        RuleHoldHands {
            sequence,
            end_battle,
        } => {
            gm(c, actor)?;
            if *end_battle {
                return Err(error(
                    "Battle-ending rulings require the authoritative end transition",
                ));
            }
            let d = flow_mut(c).completion.hold_hands.get_mut(sequence).unwrap();
            d.end_battle = Some(false);
            d.reason = why.into();
            finish(c);
        }
        _ => return Err(error("Not a move decision command")),
    }
    Ok(format!("Hold Hands decision {cmd:?}: {why}"))
}

pub fn end(
    c: &Campaign,
    actor: &str,
    sequence: u32,
    why: &str,
    faces: &[u8],
    cat: &Catalog,
) -> Result<(Campaign, String)> {
    reason(why)?;
    gm(c, actor)?;
    check_count(
        faces,
        dice(
            c,
            &special::SpecialCommand::RuleHoldHands {
                sequence,
                end_battle: true,
            },
        )?,
    )?;
    let mut prepared = c.clone();
    let d = flow_mut(&mut prepared)
        .completion
        .hold_hands
        .get_mut(&sequence)
        .unwrap();
    d.end_battle = Some(true);
    d.reason = why.into();
    let f = flow_mut(&mut prepared);
    f.queue.clear();
    f.order.clear();
    f.window = false;
    let (mut next, ended) = super::transition(&prepared, actor, &Action::EndBattle, vec![], cat)?;
    let log = format!("Hold Hands #{sequence}: GM ended the battle: {why}. {ended}");
    // One atomic undo restores the pending ruling and all battle resources.
    next.battle_undo = c.battle_undo.clone();
    checkpoint(c, &mut next, &log);
    Ok((next, log))
}
