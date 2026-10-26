//! Literal Pokerole action sequencing; all transient state belongs to the encounter.
use super::*;
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct SequencingState {
    pub enabled: bool,
    pub identities: BTreeMap<String, String>,
    pub gorilla: BTreeMap<String, String>,
    pub visits: BTreeMap<String, u32>,
    pub visited_turn: Option<u32>,
    pub truant: BTreeMap<String, u32>,
    pub penalties: BTreeSet<String>,
    pub preparing: BTreeSet<String>,
    pub dancer_round: BTreeMap<String, u32>,
    pub pending: Vec<QueuedAction>,
    pub dances: BTreeMap<String, QueuedAction>,
    pub resume: bool,
    pub copies: BTreeSet<String>,
    pub parental: BTreeMap<String, [Vec<u8>; 2]>,
    pub history: Vec<String>,
}
pub fn enabled(c: &Campaign) -> bool {
    flow(c).is_ok_and(|f| f.sequencing.enabled)
}
fn subject(a: &Action) -> Option<&str> {
    match a {
        Action::HeldItem {
            command: items::ItemCommand::Retrieve { combatant_id, .. },
            ..
        } => Some(combatant_id),
        Action::Accuracy { combatant_id, .. }
        | Action::AccuracyTargets { combatant_id, .. }
        | Action::React { combatant_id, .. }
        | Action::CoverAlly { combatant_id, .. }
        | Action::BreakIce { combatant_id, .. } => Some(combatant_id),
        Action::StatusCheck {
            combatant_id, kind, ..
        } if ["sleep", "burn"].contains(&kind.as_str()) => Some(combatant_id),
        Action::Tactics {
            command: tactics::TacticsCommand::Reaction { combatant_id, .. },
            ..
        } => Some(combatant_id),
        Action::SpecialMove {
            command: special::SpecialCommand::Release { combatant_id },
            ..
        }
        | Action::SpecialMove {
            command: special::SpecialCommand::Successive { combatant_id, .. },
            ..
        } => Some(combatant_id),
        _ => None,
    }
}
fn loyalty(c: &Campaign, id: &str) -> Result<u32> {
    let p = fighter(c, id)?;
    let n = c.sheets[&p.sheet_id]
        .workflow
        .as_ref()
        .map(|w| w.loyalty)
        .ok_or_else(|| {
            AppError::new(
                "manual_rule",
                "Loyalty must be recorded on a reviewed sheet before this ability can roll",
            )
        })?;
    if !(0..=5).contains(&n) {
        return Err(error("Loyalty must be between zero and five"));
    }
    Ok(n as u32)
}
fn check(c: &Campaign, a: &Action) -> Result<Option<(&'static str, u32)>> {
    if !enabled(c) {
        return Ok(None);
    }
    let Some(id) = subject(a) else {
        return Ok(None);
    };
    if flow(c)?.sequencing.preparing.contains(id) {
        return Ok(None);
    }
    let p = fighter(c, id)?;
    if p.has_ability("Truant")
        && flow(c)?.order.get(flow(c)?.cursor).is_some_and(|v| v == id)
        && flow(c)?
            .sequencing
            .visits
            .get(id)
            .copied()
            .unwrap_or(1)
            .is_multiple_of(2)
        && flow(c)?.sequencing.truant.get(id) != Some(&flow(c)?.turn)
    {
        return Ok(Some(("Truant", loyalty(c, id)?)));
    }
    if p.has_ability("Defeatist")
        && p.hp > 0
        && p.hp * 2 <= p.max_hp
        && !matches!(
            a,
            Action::SpecialMove {
                command: special::SpecialCommand::Successive { .. },
                ..
            }
        )
    {
        return Ok(Some(("Defeatist", loyalty(c, id)?)));
    }
    Ok(None)
}
pub fn successive_loyalty(c: &Campaign, id: &str) -> Result<u32> {
    let p = fighter(c, id)?;
    if enabled(c) && p.has_ability("Defeatist") && p.hp * 2 <= p.max_hp {
        loyalty(c, id)
    } else {
        Ok(0)
    }
}
pub fn move_guard(c: &Campaign, id: &str, name: Option<&str>) -> Result<()> {
    if !enabled(c) || !fighter(c, id)?.has_ability("Gorilla Tactics") {
        return Ok(());
    }
    let selected = flow(c)?
        .sequencing
        .gorilla
        .get(id)
        .ok_or_else(|| error("Choose Gorilla Tactics' entry move first"))?;
    if name != Some(selected.as_str()) {
        return Err(error("Gorilla Tactics permits only its recorded entry move; holding or passing remains available"));
    }
    Ok(())
}
pub fn repeat(c: &Campaign, id: &str, name: &str) -> bool {
    enabled(c)
        && fighter(c, id).is_ok_and(|p| p.has_ability("Gorilla Tactics") && p.actions < 5)
        && flow(c).is_ok_and(|f| f.sequencing.gorilla.get(id).is_some_and(|n| n == name))
}
pub fn apply_stats(c: &Campaign, p: &Combatant, s: &mut Sheet) {
    if enabled(c)
        && (flow(c).unwrap().sequencing.penalties.contains(&p.id)
            || (!flow(c).unwrap().sequencing.preparing.contains(&p.id)
                && flow(c)
                    .unwrap()
                    .queue
                    .first()
                    .is_some_and(|q| q.actor_id == p.id && q.sequencing_penalty)))
    {
        for key in ["Strength", "Special"] {
            if let Some(v) = s.stats.get_mut(key) {
                *v = (*v - 2).max(1);
            }
        }
    }
}
fn parental(c: &Campaign, a: &Action) -> bool {
    matches!(a, Action::Damage { .. })
        && enabled(c)
        && current(c).is_ok_and(|q| {
            ability_changes::frozen_subject(
                c,
                &q.actor_id,
                q.sequence,
                fighter(c, &q.actor_id).unwrap(),
            )
            .selected_ability
                == "Parental Bond"
        })
}
pub fn dice(c: &Campaign, a: &Action, cat: &Catalog) -> Result<u32> {
    validate_action(c, a)?;
    let n = base_dice_count(c, a, cat)?;
    if parental(c, a) {
        return n
            .checked_mul(2)
            .ok_or_else(|| error("Parental Bond pool overflow"));
    }
    Ok(n + check(c, a)?.map_or(0, |(_, n)| n))
}
fn repair_checkpoint(c: &Campaign, next: &mut Campaign) {
    if let (Some(b), Some(cp)) = (c.battle.as_ref(), next.battle_undo.last_mut()) {
        if cp.event_sequence == c.revision + 1 {
            cp.battle = b.clone();
        }
    }
}
pub fn transition(
    c: &Campaign,
    actor: &str,
    a: &Action,
    faces: Vec<u8>,
    cat: &Catalog,
) -> Result<(Campaign, String)> {
    if !enabled(c) {
        return transition_inner(c, actor, a, faces, cat);
    }
    validate_action(c, a)?;
    // The host rolls two independent lanes. Shared targets reuse both lanes and
    // may choose different winners when their defense requires another prefix.
    if parental(c, a) {
        let base = base_dice_count(c, a, cat)? as usize;
        check_count(&faces, (base * 2) as u32)?;
        let q = current(c)?.clone();
        let key = format!("{}:{}:{}", q.actor_id, q.sequence, q.special_index);
        let mut temp = c.clone();
        let mut lanes = flow(c)?
            .sequencing
            .parental
            .get(&key)
            .cloned()
            .unwrap_or_default();
        lanes[0].extend_from_slice(&faces[..base]);
        lanes[1].extend_from_slice(&faces[base..]);
        let total = if q.targets.is_empty() {
            base
        } else {
            let Action::Damage { pool_modifier, .. } = a else {
                unreachable!()
            };
            damage_dice(c, &q, *pool_modifier, cat)? as usize
        };
        if lanes.iter().any(|v| v.len() < total) {
            return Err(error(
                "Parental Bond's shared damage evidence is incomplete",
            ));
        }
        let counts = [
            rules::successes(&lanes[0][..total]),
            rules::successes(&lanes[1][..total]),
        ];
        let winner = usize::from(counts[1] > counts[0]);
        let previous = q.shared_dice.len();
        if !q.targets.is_empty() {
            flow_mut(&mut temp).queue[0].shared_dice = lanes[winner][..previous].to_vec();
        }
        flow_mut(&mut temp)
            .sequencing
            .parental
            .insert(key, lanes.clone());
        let selected = if q.targets.is_empty() {
            lanes[winner][..total].to_vec()
        } else {
            lanes[winner][previous..].to_vec()
        };
        let (mut next, log) = transition_inner(&temp, actor, a, selected, cat)?;
        let note = format!(
            "Parental Bond: pools {:?} / {:?}; successes {:?}; keep pool {} once",
            &lanes[0][..total],
            &lanes[1][..total],
            counts,
            winner + 1
        );
        flow_mut(&mut next).sequencing.history.push(note.clone());
        repair_checkpoint(c, &mut next);
        return Ok((next, format!("{log} {note}")));
    }
    let mut temp = c.clone();
    let plan = check(c, a)?;
    let mut note = None;
    let mut offset = 0;
    if let Some((ability, n)) = plan {
        let base = base_dice_count(c, a, cat)?;
        check_count(&faces, base + n)?;
        offset = n as usize;
        let passed = rules::successes(&faces[..offset]) >= 2;
        let id = subject(a).unwrap();
        owner(c, actor, id)?;
        match a {
            Action::Accuracy { reason: r, .. }
            | Action::AccuracyTargets { reason: r, .. }
            | Action::React { reason: r, .. }
            | Action::CoverAlly { reason: r, .. }
            | Action::BreakIce { reason: r, .. }
            | Action::StatusCheck { reason: r, .. }
            | Action::SpecialMove { reason: r, .. }
            | Action::Tactics { reason: r, .. }
            | Action::HeldItem { reason: r, .. } => reason(r)?,
            _ => {}
        }
        note = Some(format!(
            "{ability}: {} Loyalty dice {:?}, requires two successes; {}",
            fighter(c, id)?.name,
            &faces[..offset],
            if passed { "passed" } else { "failed" }
        ));
        if ability == "Truant" {
            if !passed {
                let mut validation = temp.clone();
                let turn = flow(c)?.turn;
                flow_mut(&mut validation)
                    .sequencing
                    .truant
                    .insert(id.into(), turn);
                transition_inner(&validation, actor, a, faces[offset..].to_vec(), cat)?;
                advance(&mut temp);
                ability_changes::lifecycle(c, &mut temp, cat)?;
                tactics::lifecycle(c, &mut temp, cat)?;
                field::lifecycle(c, &mut temp, cat)?;
                abilities::lifecycle(c, &mut temp)?;
                extra::lifecycle(c, &mut temp, cat)?;
                special::lifecycle(c, &mut temp, cat)?;
                let log = note.unwrap();
                flow_mut(&mut temp).sequencing.history.push(log.clone());
                checkpoint(c, &mut temp, &log);
                return Ok((temp, log));
            }
            let turn = flow(c)?.turn;
            flow_mut(&mut temp)
                .sequencing
                .truant
                .insert(id.into(), turn);
        } else if !passed {
            flow_mut(&mut temp).sequencing.penalties.insert(id.into());
        }
    }
    if plan.is_some() {
        flow_mut(&mut temp)
            .sequencing
            .preparing
            .insert(subject(a).unwrap().into());
    }
    let penalty =
        subject(a).is_some_and(|id| flow(&temp).unwrap().sequencing.penalties.contains(id));
    let actual = base_dice_count(&temp, a, cat)? as usize;
    if faces.len() < offset + actual {
        return Err(error(
            "Reserved action dice are insufficient after the ability check",
        ));
    }
    if plan.is_none() {
        check_count(&faces, actual as u32)?;
    }
    let (mut next, log) = transition_inner(
        &temp,
        actor,
        a,
        faces[offset..offset + actual].to_vec(),
        cat,
    )?;
    if let Some(id) = subject(a).filter(|_| {
        !matches!(
            a,
            Action::SpecialMove {
                command: special::SpecialCommand::Successive { .. },
                ..
            }
        )
    }) {
        let old_keys: BTreeSet<_> = flow(c)?
            .queue
            .iter()
            .map(|q| (q.actor_id.clone(), q.sequence, q.special_index))
            .collect();
        for q in &mut flow_mut(&mut next).queue {
            if q.actor_id == id
                && !old_keys.contains(&(q.actor_id.clone(), q.sequence, q.special_index))
            {
                q.sequencing_penalty = penalty;
            }
        }
        if matches!(a, Action::Accuracy { .. } | Action::AccuracyTargets { .. }) {
            if let Some(charge) = flow_mut(&mut next).special.charges.get_mut(id) {
                charge.action.sequencing_penalty = penalty;
            }
        }
        flow_mut(&mut next).sequencing.penalties.remove(id);
        flow_mut(&mut next).sequencing.preparing.remove(id);
    }
    if let Some(id) = subject(a) {
        flow_mut(&mut next).sequencing.preparing.remove(id);
    }
    if let Some(note) = note {
        flow_mut(&mut next).sequencing.history.push(note.clone());
        repair_checkpoint(c, &mut next);
        return Ok((next, format!("{log} {note}")));
    }
    Ok((next, log))
}
pub fn completed(c: &mut Campaign, q: &QueuedAction) {
    if !enabled(c) || !is_dance(&q.move_name) {
        return;
    }
    let round = battle(c).unwrap().round;
    if battle(c).unwrap().combatants.iter().any(|p| {
        p.deployed
            && p.hp > 0
            && p.has_ability("Dancer")
            && flow(c).unwrap().sequencing.dancer_round.get(&p.id) != Some(&round)
    }) {
        flow_mut(c).sequencing.pending.push(q.clone());
        flow_mut(c).sequencing.resume = true;
    }
}
pub fn waiting(c: &Campaign) -> bool {
    enabled(c) && flow(c).unwrap().sequencing.resume
}
pub fn resume(c: &mut Campaign) {
    if !waiting(c)
        || !flow(c).unwrap().sequencing.pending.is_empty()
        || flow(c)
            .unwrap()
            .ability_events
            .iter()
            .any(|e| e.kind.starts_with("sequence-dancer"))
    {
        return;
    }
    flow_mut(c).sequencing.resume = false;
    if !flow(c).unwrap().queue.is_empty() {
        prepare_targets(c);
    } else if !special::continue_series(c) {
        advance(c);
        let _ = record_turn(c);
    }
}
pub fn lifecycle(c: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let ps = battle(c)?.combatants.clone();
    let live: BTreeSet<_> = ps
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    for p in &ps {
        if !live.contains(&p.id)
            || flow(c)?.sequencing.identities.get(&p.id) != Some(&p.selected_ability)
        {
            flow_mut(c).sequencing.gorilla.remove(&p.id);
            flow_mut(c).sequencing.visits.remove(&p.id);
            flow_mut(c).sequencing.truant.remove(&p.id);
            if p.has_ability("Truant")
                && flow(c)?.started
                && flow(c)?.sequencing.visited_turn == Some(flow(c)?.turn)
                && flow(c)?.order.get(flow(c)?.cursor) == Some(&p.id)
            {
                flow_mut(c).sequencing.visits.insert(p.id.clone(), 1);
            }
            flow_mut(c)
                .ability_events
                .retain(|e| !e.kind.starts_with("sequence-") || e.combatant_id != p.id);
        }
        if live.contains(&p.id) {
            flow_mut(c)
                .sequencing
                .identities
                .insert(p.id.clone(), p.selected_ability.clone());
        } else {
            flow_mut(c).sequencing.identities.remove(&p.id);
        }
    }
    record_turn(c)?;
    for p in ps
        .iter()
        .filter(|p| live.contains(&p.id) && p.has_ability("Gorilla Tactics"))
    {
        if !flow(c)?.sequencing.gorilla.contains_key(&p.id)
            && !flow(c)?
                .ability_events
                .iter()
                .any(|e| e.kind == "sequence-gorilla" && e.combatant_id == p.id)
        {
            let mut choices = rules::base_sheet(c, p)?.moves;
            choices.sort();
            choices.dedup();
            if choices.is_empty() {
                flow_mut(c)
                    .sequencing
                    .gorilla
                    .insert(p.id.clone(), String::new());
                flow_mut(c).sequencing.history.push(format!("Gorilla Tactics: {} has no learned move; holding or withdrawing remains available",p.name));
                continue;
            }
            let event_id = format!("gorilla:{}:{}", p.id, c.revision + 1);
            flow_mut(c).ability_events.push(abilities::AbilityEvent {
                id: event_id,
                combatant_id: p.id.clone(),
                target_id: p.id.clone(),
                ability: p.selected_ability.clone(),
                kind: "sequence-gorilla".into(),
                move_name: "Choose the only move for this field entry".into(),
                choices,
            });
        }
    }
    if !special::preparing(c) {
        let pending = std::mem::take(&mut flow_mut(c).sequencing.pending);
        for q in pending {
            for p in ps
                .iter()
                .filter(|p| live.contains(&p.id) && p.has_ability("Dancer"))
            {
                if flow(c)?.sequencing.dancer_round.get(&p.id) == Some(&battle(c)?.round)
                    || flow(c)?
                        .ability_events
                        .iter()
                        .any(|e| e.kind.starts_with("sequence-dancer") && e.combatant_id == p.id)
                {
                    continue;
                }
                let m = field::move_data(c, p, &move_entry(cat, &q.move_name)?);
                let pattern = m["Target"].as_str().unwrap_or("");
                let mut choices = vec![];
                if ["Self", "Battlefield", "Area"].contains(&pattern) {
                    if plan(
                        c,
                        &p.id,
                        if pattern == "Self" {
                            std::slice::from_ref(&p.id)
                        } else {
                            &[]
                        },
                        &m,
                    )
                    .is_ok()
                    {
                        choices.push("copy".into());
                    }
                } else {
                    for t in ps.iter().filter(|t| live.contains(&t.id)) {
                        if plan(c, &p.id, std::slice::from_ref(&t.id), &m).is_ok() {
                            choices.push(t.id.clone());
                        }
                    }
                }
                if choices.is_empty() {
                    continue;
                }
                choices.push("decline".into());
                let id = format!("dancer:{}:{}:{}", p.id, q.actor_id, q.sequence);
                flow_mut(c).ability_events.push(abilities::AbilityEvent {
                    id: id.clone(),
                    combatant_id: p.id.clone(),
                    target_id: p.id.clone(),
                    ability: "Dancer".into(),
                    kind: if pattern == "Random Foe" {
                        "sequence-dancer-random"
                    } else {
                        "sequence-dancer"
                    }
                    .into(),
                    move_name: q.move_name.clone(),
                    choices,
                });
                flow_mut(c).sequencing.dances.insert(id, q.clone());
            }
        }
    }
    let keys: BTreeSet<_> = flow(c)?
        .queue
        .iter()
        .map(|q| format!("{}:{}:{}", q.actor_id, q.sequence, q.special_index))
        .collect();
    flow_mut(c)
        .sequencing
        .parental
        .retain(|k, _| keys.contains(k));
    let events: BTreeSet<_> = flow(c)?
        .ability_events
        .iter()
        .map(|e| e.id.clone())
        .collect();
    flow_mut(c)
        .sequencing
        .dances
        .retain(|id, _| events.contains(id));
    resume(c);
    Ok(())
}
pub fn resolve(
    c: &mut Campaign,
    actor: &str,
    e: &abilities::AbilityEvent,
    choice: &str,
    cat: &Catalog,
) -> Result<()> {
    let p = fighter(c, &e.combatant_id)?.clone();
    if p.hp == 0 || !p.has_ability(&e.ability) {
        return Err(error("Sequencing ability is no longer active"));
    }
    if e.kind == "sequence-gorilla" {
        if !rules::base_sheet(c, &p)?.moves.iter().any(|n| n == choice) {
            return Err(error("Choose a currently learned Gorilla Tactics move"));
        }
        flow_mut(c)
            .sequencing
            .gorilla
            .insert(p.id.clone(), choice.into());
        flow_mut(c)
            .sequencing
            .history
            .push(format!("Gorilla Tactics: {} selected {choice}", p.name));
        return Ok(());
    }
    let source = flow_mut(c)
        .sequencing
        .dances
        .remove(&e.id)
        .ok_or_else(|| error("Dancer trigger expired"))?;
    if choice == "decline" {
        return Ok(());
    }
    if e.kind == "sequence-dancer-random" {
        gm(c, actor)?;
    }
    let round = battle(c)?.round;
    if flow(c)?.sequencing.dancer_round.get(&p.id) == Some(&round) {
        return Err(error("Dancer's free action was already used this round"));
    }
    let m = field::move_data(c, &p, &move_entry(cat, &source.move_name)?);
    // A free action bypasses action-count and learned-move restrictions, while
    // normal status, range, move-blocker and target validation remain binding.
    let mut validation = c.clone();
    flow_mut(&mut validation).ability_events.clear();
    let temporary = fighter_mut(&mut validation, &p.id)?;
    temporary.actions = 0;
    temporary.used_moves.retain(|n| n != &source.move_name);
    validation
        .sheets
        .get_mut(&p.sheet_id)
        .unwrap()
        .moves
        .push(source.move_name.clone());
    forms::grant_temporary_move(&mut validation, &p.id, &source.move_name);
    ready(&validation, &p.id)?;
    rules::accuracy(&validation, &p.id, &source.move_name, 0, cat)?;
    if p.runtime
        .statuses
        .iter()
        .any(|s| s.kind == "Disable" && s.detail == source.move_name)
    {
        return Err(error("This dance is disabled"));
    }
    let requested = if choice == "copy" {
        if m["Target"] == "Self" {
            vec![p.id.clone()]
        } else {
            vec![]
        }
    } else {
        vec![choice.into()]
    };
    let (ids, scope) = plan(c, &p.id, &requested, &m)?;
    let ids = tactics::redirect_move(c, &p.id, ids, &m, cat)?;
    let source_results: BTreeMap<_, _> = if events::enabled(c) {
        source
            .targets
            .iter()
            .filter_map(|t| t.successes.map(|n| (t.target_id.clone(), n)))
            .collect()
    } else {
        BTreeMap::new()
    };
    let copied_successes = source_results
        .values()
        .copied()
        .max()
        .unwrap_or(source.successes);
    let critical_margin = extra::critical_margin(c, &p, &m);
    flow_mut(c)
        .sequencing
        .dancer_round
        .insert(p.id.clone(), round);
    fighter_mut(c, &p.id)?
        .used_moves
        .push(source.move_name.clone());
    flow_mut(c).sequencing.history.push(format!(
        "Dancer: {} copies {} after {}, {} Accuracy successes; free action, one rampage use",
        p.name, source.move_name, source.actor_id, copied_successes
    ));
    if copied_successes < 1 {
        return Ok(());
    }
    let q = QueuedAction {
        response_to: None,
        actor_id: p.id.clone(),
        target_id: ids.first().cloned().unwrap_or_else(|| p.id.clone()),
        move_name: source.move_name,
        sequence: c.revision + 1,
        special_index: 0,
        successes: copied_successes,
        critical: m["Category"] != "Support" && copied_successes > critical_margin,
        timing: "main".into(),
        priority: 0,
        cancelled: false,
        damage_done: false,
        damage: 0,
        raw_damage_successes: 0,
        ice: false,
        targets: ids
            .into_iter()
            .map(|target_id| {
                let copied = (!source_results.is_empty()).then(|| {
                    source_results
                        .get(&target_id)
                        .copied()
                        .unwrap_or(copied_successes)
                });
                TargetImpact {
                    successes: copied,
                    critical: copied.map(|n| m["Category"] != "Support" && n > critical_margin),
                    target_id,
                    responded: copied.is_some_and(|n| n < 1),
                    avoided: copied.is_some_and(|n| n < 1),
                    resolved: false,
                }
            })
            .collect(),
        field_scope: scope,
        shared_dice: vec![],
        damage_modifiers: None,
        damage_context: None,
        defense_key: if m["Category"] == "Support" {
            "Support"
        } else if m["Category"] == "Physical" {
            "Defense"
        } else {
            "SpecialDefense"
        }
        .into(),
        sequencing_penalty: false,
        sheer_force: false,
    };
    let index = flow(c)?
        .queue
        .iter()
        .take_while(|q| {
            flow(c)
                .unwrap()
                .sequencing
                .copies
                .contains(&format!("{}:{}", q.actor_id, q.sequence))
        })
        .count();
    flow_mut(c)
        .sequencing
        .copies
        .insert(format!("{}:{}", q.actor_id, q.sequence));
    flow_mut(c).queue.insert(index, q);
    Ok(())
}
pub fn notes(c: &Campaign, a: &Action) -> Vec<String> {
    if !enabled(c) {
        return vec![];
    }
    let mut notes = vec![];
    if let Ok(Some((ability, n))) = check(c, a) {
        notes.push(format!("{ability}: roll {n} Loyalty dice first, needing two successes. Truant failure forfeits this turn; Defeatist failure reduces Strength and Special by two for this action. Reserved surplus action dice are discarded, never reused."));
    }
    if parental(c, a) {
        notes.push("Parental Bond: roll each damage pool twice, use the higher success count once. Shared targets retain both independent dice lanes.".into());
    }
    if subject(a).is_some_and(|id| fighter(c, id).is_ok_and(|p| p.has_ability("Gorilla Tactics"))) {
        notes.push("Gorilla Tactics: only the selected entry move, up to five actions per round; Strength +1 uses the ability channel.".into());
    }
    notes
}

pub fn missed(c: &mut Campaign, id: &str, name: &str, successes: i32) {
    missed_with_targets(c, id, name, successes, vec![]);
}
pub fn missed_with_targets(
    c: &mut Campaign,
    id: &str,
    name: &str,
    successes: i32,
    targets: Vec<TargetImpact>,
) {
    if !enabled(c) || !is_dance(name) {
        return;
    }
    let q = QueuedAction {
        response_to: None,
        actor_id: id.into(),
        target_id: id.into(),
        move_name: name.into(),
        sequence: c.revision + 1,
        special_index: 0,
        successes,
        critical: false,
        timing: "main".into(),
        priority: 0,
        cancelled: false,
        damage_done: true,
        damage: 0,
        raw_damage_successes: 0,
        ice: false,
        targets,
        field_scope: None,
        shared_dice: vec![],
        damage_modifiers: None,
        damage_context: None,
        defense_key: String::new(),
        sequencing_penalty: false,
        sheer_force: false,
    };
    completed(c, &q);
}

pub fn roll_limit(c: &Campaign, a: &Action) -> u32 {
    if !enabled(c) {
        100
    } else if parental(c, a) {
        200
    } else if matches!(
        a,
        Action::SpecialMove {
            command: special::SpecialCommand::Successive { .. },
            ..
        }
    ) {
        525
    } else if check(c, a).is_ok_and(|v| v.is_some()) {
        105
    } else {
        100
    }
}

pub fn finish(c: &mut Campaign) {
    if let Some(f) = c
        .battle
        .as_mut()
        .and_then(|b| b.flow.as_mut())
        .filter(|f| f.sequencing.enabled)
    {
        let history = std::mem::take(&mut f.sequencing.history);
        f.sequencing = SequencingState {
            enabled: true,
            history,
            ..Default::default()
        };
    }
}

fn validate_action(c: &Campaign, a: &Action) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    match a {
        Action::CoverAlly { combatant_id, .. } => move_guard(c, combatant_id, None),
        Action::StatusCheck {
            combatant_id, kind, ..
        } if ["sleep", "burn"].contains(&kind.as_str()) => move_guard(c, combatant_id, None),
        Action::BreakIce {
            combatant_id,
            move_name,
            ..
        } => move_guard(c, combatant_id, Some(move_name)),
        Action::SpecialMove {
            command: special::SpecialCommand::Release { combatant_id },
            ..
        } => {
            let name = flow(c)?
                .special
                .charges
                .get(combatant_id)
                .map(|q| q.action.move_name.as_str());
            move_guard(c, combatant_id, name)
        }
        _ => Ok(()),
    }
}

fn record_turn(c: &mut Campaign) -> Result<()> {
    if flow(c)?.started && flow(c)?.sequencing.visited_turn != Some(flow(c)?.turn) {
        if let Some(id) = flow(c)?.order.get(flow(c)?.cursor).cloned() {
            *flow_mut(c).sequencing.visits.entry(id).or_default() += 1;
        }
        let turn = flow(c)?.turn;
        flow_mut(c).sequencing.visited_turn = Some(turn);
    }
    Ok(())
}

fn is_dance(name: &str) -> bool {
    name.split_whitespace().any(|word| word == "Dance")
}
