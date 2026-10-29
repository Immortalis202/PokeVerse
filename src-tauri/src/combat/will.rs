//! MASTER 472/511. Encounter-local Will drains and recurring Grudge checks.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Grudge {
    pub source: String,
    pub difficulty: u32,
    pub checked_round: u32,
}
pub fn enabled(c: &Campaign) -> bool {
    flow(c).is_ok_and(|f| f.will_drains)
}
pub fn reset(rt: &mut CombatState) {
    rt.pain_ignored = 0;
    rt.fate_pending = false;
    rt.chance_cancelled = true;
}
fn loyalty(c: &Campaign, target: &str) -> Result<u32> {
    let p = special::registered(c, target)?;
    let n = c.sheets[&p.sheet_id]
        .workflow
        .as_ref()
        .map(|w| w.loyalty)
        .ok_or_else(|| {
            AppError::new(
                "manual_rule",
                "Record the target's reviewed Loyalty before resolving Grudge",
            )
        })?;
    if !(0..=5).contains(&n) {
        return Err(error("Loyalty must be between zero and five"));
    }
    Ok(n as u32)
}
pub fn effect(c: &mut Campaign, q: &QueuedAction, cat: &Catalog) -> Result<bool> {
    if !enabled(c) || !["Grudge", "Eerie Spell"].contains(&q.move_name.as_str()) {
        return Ok(false);
    }
    let source = reviewed::source(c, q)?;
    let m = field::move_data(c, &source, &move_entry(cat, &q.move_name)?);
    let target = fighter(c, &q.target_id)?;
    if special::target_effects_blocked(c, q, &m)
        || (!reviewed::extreme(c, &source)
            && (extra::effects_immune(c, target, &m) || abilities::move_immune(c, target, &m)))
    {
        return Ok(true);
    }
    if q.move_name == "Eerie Spell" {
        let (effectiveness, immune) = rules::type_matchup(
            c,
            &source,
            target,
            m["Type"].as_str().unwrap_or("Psychic"),
            cat,
        )?;
        if immune
            || abilities::damage_immune_from(c, &source, target, &m)
            || extra::source_immune(c, &source, target, effectiveness)
        {
            return Ok(true);
        }
        let rt = &mut fighter_mut(c, &q.target_id)?.runtime;
        rt.will = rt.will.saturating_sub(3).max(0);
    } else {
        loyalty(c, &q.target_id)?;
        let rt = &mut fighter_mut(c, &q.target_id)?.runtime;
        rt.will = 0;
        reset(rt);
        let round = battle(c)?.round;
        flow_mut(c)
            .special
            .grudges
            .entry(q.target_id.clone())
            .or_insert(Grudge {
                source: q.actor_id.clone(),
                difficulty: 1,
                checked_round: round.saturating_sub(1),
            });
        fighter_mut(c, &q.actor_id)?.hp = 0;
    }
    Ok(true)
}
pub fn pending(c: &Campaign) -> bool {
    enabled(c)
        && flow(c).unwrap().special.grudges.iter().any(|(id, g)| {
            g.checked_round < battle(c).unwrap().round
                && special::registered(c, id).is_ok_and(|p| p.hp > 0)
        })
}
pub fn dice(c: &Campaign, target: &str) -> Result<u32> {
    if !enabled(c)
        || !flow(c)?.queue.is_empty()
        || flow(c)?.battlefield.end_round != battle(c)?.round
        || flow(c)?
            .special
            .grudges
            .get(target)
            .is_none_or(|g| g.checked_round >= battle(c).unwrap().round)
        || special::registered(c, target)?.hp == 0
    {
        return Err(error("No pending round-end Grudge check for this target"));
    }
    loyalty(c, target)
}
pub fn check(c: &mut Campaign, actor: &str, target: &str, faces: &[u8]) -> Result<String> {
    check_count(faces, dice(c, target)?)?;
    let p = special::registered(c, target)?;
    if actor != c.gm_id && actor != p.owner_id {
        return Err(error("Only the target owner or GM may roll Grudge Loyalty"));
    }
    let round = battle(c)?.round;
    let g = flow_mut(c).special.grudges.get_mut(target).unwrap();
    let difficulty = g.difficulty;
    let successes = rules::successes(faces).max(0) as u32;
    g.checked_round = round;
    g.difficulty = difficulty.saturating_add(1);
    let failed = successes < difficulty;
    if failed {
        fighter_mut(c, target)?.hp = 0;
        flow_mut(c).special.grudges.remove(target);
    }
    Ok(format!("Grudge Loyalty: {faces:?}, {successes} successes / {difficulty} required; fainted={failed}"))
}
pub fn lifecycle(c: &mut Campaign) {
    let Some(b) = c.battle.as_mut() else { return };
    let alive: BTreeSet<_> = b
        .combatants
        .iter()
        .filter(|p| p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    if let Some(f) = b.flow.as_mut() {
        f.special
            .grudges
            .retain(|id, _| b.active && alive.contains(id));
    }
}
pub fn phase(c: &Campaign, a: &Action) -> Result<()> {
    if pending(c)
        && flow(c)?.battlefield.end_round == battle(c)?.round
        && !matches!(
            a,
            Action::SpecialMove {
                command: special::SpecialCommand::GrudgeCheck { .. }
                    | special::SpecialCommand::Tick { .. },
                ..
            } | Action::RecurringMove { .. }
                | Action::NextRound
                | Action::Battlefield { .. }
                | Action::ResolveAbility { .. }
                | Action::RollbackBattle { .. }
                | Action::Override { .. }
                | Action::SetStatus { .. }
        )
    {
        return Err(error("Resolve the round-end Grudge Loyalty checks first"));
    }
    Ok(())
}
