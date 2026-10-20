use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct TargetImpact {
    pub target_id: String,
    pub responded: bool,
    pub avoided: bool,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct FieldEffect {
    pub sequence: u32,
    pub actor_id: String,
    pub move_name: String,
    pub scope: String,
    pub rounds: Option<u32>,
    pub ruling: String,
}

pub(super) fn require_targeting(c: &Campaign) -> Result<()> {
    if ![
        ENGINE,
        SEQUENCING_ENGINE,
        STAT_ENGINE,
        REVIEWED_ENGINE,
        TRANSFER_ENGINE,
        ASSIGNMENT_ENGINE,
        DECOY_ENGINE,
        TACTICS_ENGINE,
        SPECIAL_ENGINE,
        EXTRA_ENGINE,
        FIELD_ENGINE,
        ABILITY_ENGINE,
        TARGET_ENGINE,
    ]
    .contains(&c.engine.as_str())
    {
        return Err(AppError::new(
            "version",
            "Upgrade the engine between battles to use advanced targeting",
        ));
    }
    Ok(())
}

pub(super) fn selected(a: &Action) -> Vec<String> {
    match a {
        Action::Accuracy { target_id, .. } => target_id.iter().cloned().collect(),
        Action::AccuracyTargets { target_ids, .. } => target_ids.clone(),
        _ => vec![],
    }
}

pub(super) fn accuracy_command(a: &Action) -> Action {
    match a {
        Action::AccuracyTargets {
            combatant_id,
            target_ids,
            move_name,
            pool_modifier,
            success_modifier,
            reason,
        } => Action::Accuracy {
            combatant_id: combatant_id.clone(),
            target_id: target_ids.first().cloned(),
            move_name: move_name.clone(),
            pool_modifier: *pool_modifier,
            success_modifier: *success_modifier,
            reason: reason.clone(),
        },
        _ => a.clone(),
    }
}

pub(super) fn plan(
    c: &Campaign,
    id: &str,
    requested: &[String],
    m: &serde_json::Value,
) -> Result<(Vec<String>, Option<String>)> {
    let p = fighter(c, id)?;
    if special::enabled(c) && ["Wish", "Lunar Blessing"].contains(&m["Name"].as_str().unwrap_or(""))
    {
        if requested.len() != 1 {
            return Err(error("Choose the user or one active ally"));
        }
        let t = fighter(c, &requested[0])?;
        if t.hp == 0 || t.side != p.side {
            return Err(error("Choose a living allied target"));
        }
        return Ok((requested.to_vec(), None));
    }
    if special::enabled(c)
        && m["Name"] == "Expanding Force"
        && field::has(c, "Psychic Terrain", p)
        && requested.len() == 2
    {
        let cap = [1, 2, 3, 4, 5, 6, 8, 10]
            [crate::creation::rank(&c.sheets[&p.sheet_id].rank).unwrap_or(0)];
        if cap < 2 || requested[0] == requested[1] {
            return Err(error(
                "Expanding Force targets exceed the rank cap or repeat an identity",
            ));
        }
        for key in requested {
            let t = fighter(c, key)?;
            if t.hp == 0
                || t.side == p.side
                || flow(c)?
                    .ranges
                    .get(id)
                    .is_some_and(|range| !range.contains(key))
            {
                return Err(error("Choose two available foes in range"));
            }
            special::target_guard(c, id, key, m)?;
        }
        return Ok((requested.to_vec(), None));
    }
    if special::enabled(c) && m["Name"] == "Revival Blessing" {
        if requested.len() != 1 {
            return Err(error("Choose one fainted registered ally"));
        }
        let target = special::registered(c, &requested[0])?;
        if target.side != p.side || target.hp != 0 {
            return Err(error("Revival Blessing requires a fainted ally"));
        }
        return Ok((requested.to_vec(), None));
    }
    let pattern = m["Target"].as_str().unwrap_or("");
    let advanced = !["Self", "Foe", "Ally"].contains(&pattern);
    if advanced {
        require_targeting(c)?;
    }
    let cap = [1, 2, 3, 4, 5, 6, 8, 10][crate::creation::rank(&c.sheets[&p.sheet_id].rank)
        .ok_or_else(|| error("Unknown rank for target limit"))?
        as usize];
    let active =
        |v: &&Combatant| v.deployed && v.hp > 0 && v.available_round <= battle(c).unwrap().round;
    let in_range = |key: &str| {
        key == id
            || flow(c)
                .unwrap()
                .ranges
                .get(id)
                .is_none_or(|ids| ids.iter().any(|v| v == key))
    };
    let mut ids = requested.to_vec();
    if ids.iter().collect::<BTreeSet<_>>().len() != ids.len() {
        return Err(error("Duplicate targets are not allowed"));
    }
    let scope = match pattern {
        "Battlefield" => Some("all".into()),
        "Ally's Battlefield" => Some(format!("side:{}", p.side)),
        "Foe's Battlefield" => Some(format!("opponents-of:{}", p.side)),
        _ => None,
    };
    if scope.is_some() {
        if !ids.is_empty() {
            return Err(error(
                "Battlefield moves target a field scope, not combatants",
            ));
        }
        return Ok((vec![], scope));
    }
    if pattern == "Area" {
        let mut all: Vec<_> = battle(c)?
            .combatants
            .iter()
            .filter(active)
            .filter(|v| {
                v.id != id && in_range(&v.id) && special::target_guard(c, id, &v.id, m).is_ok()
            })
            .collect();
        all.sort_by_key(|v| (v.side == p.side, v.id.clone()));
        let expected: Vec<_> = all.into_iter().take(cap).map(|v| v.id.clone()).collect();
        if !ids.is_empty()
            && ids.iter().collect::<BTreeSet<_>>() != expected.iter().collect::<BTreeSet<_>>()
        {
            return Err(error("Area moves include the full rank-limited range, prioritizing foes then allies; GM can change range"));
        }
        ids = expected;
    }
    if pattern == "All Allies" && ids.is_empty() {
        ids.push(id.into());
    }
    if ids.is_empty() {
        return Err(error("Select at least one target"));
    }
    let limit = if ["Self", "Foe", "Ally", "Random Foe"].contains(&pattern) {
        1
    } else {
        cap
    };
    if ids.len() > limit {
        return Err(error("Selected targets exceed the rank's target limit"));
    }
    for key in &ids {
        let t = fighter(c, key)?;
        special::target_guard(c, id, key, m)?;
        if t.hp == 0 || t.available_round > battle(c)?.round || !in_range(key) {
            return Err(error(
                "Target is fainted, unavailable or outside the GM-confirmed range",
            ));
        }
        match pattern {
            "Self" if key == id => {}
            "Foe" if key != id => {} // The manual permits targeting an ally with a Foe move.
            "Ally" if key != id && t.side == p.side => {}
            "All Foes" | "Random Foe" if t.side != p.side && key != id => {}
            "All Allies" if t.side == p.side => {}
            "Area" if key != id => {}
            _ => return Err(error("Target does not match the move's target pattern")),
        }
    }
    if pattern == "All Allies" && !ids.iter().any(|v| v == id) {
        return Err(error("User and allies targeting must include the user"));
    }
    // Highest defense first; stable IDs resolve equal defenses consistently.
    let defense_key = if m["Category"] == "Physical" {
        "Defense"
    } else {
        "SpecialDefense"
    };
    ids.sort_by_key(|key| {
        let score = rules::effective_sheet(c, fighter(c, key).unwrap())
            .map(|s| crate::creation::profile_stats(&s)[defense_key])
            .unwrap_or(0);
        (-score, key.clone())
    });
    Ok((ids, None))
}

pub(super) fn break_cover(c: &mut Campaign, id: &str) {
    flow_mut(c)
        .covers
        .retain(|ally, coverer| ally != id && coverer != id);
}

pub(super) fn cleanup_covers(before: &Campaign, next: &mut Campaign, a: &Action) {
    if next.battle.as_ref().and_then(|b| b.flow.as_ref()).is_none() {
        return;
    }
    let invalid: Vec<_> = flow(next)
        .ok()
        .map(|f| {
            f.covers
                .iter()
                .filter_map(|(ally, coverer)| {
                    let valid = [ally, coverer].iter().all(|id| {
                        fighter(next, id).is_ok_and(|p| {
                            p.hp > 0
                                && (matches!(a, Action::CoverAlly { .. })
                                    || fighter(before, id)
                                        .is_ok_and(|old| p.actions <= old.actions))
                        })
                    });
                    if valid {
                        None
                    } else {
                        Some(ally.clone())
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    for id in invalid {
        flow_mut(next).covers.remove(&id);
    }
}

pub(super) fn redirect(c: &Campaign, source: &str, ids: Vec<String>, bypass: bool) -> Vec<String> {
    if special::enabled(c) && bypass {
        return ids;
    }
    if extra::enabled(c) && fighter(c, source).is_ok_and(|p| p.selected_ability == "Infiltrator") {
        return ids;
    }
    let mut result = vec![];
    for id in ids {
        let key = flow(c)
            .unwrap()
            .covers
            .get(&id)
            .filter(|v| {
                fighter(c, v)
                    .is_ok_and(|p| p.hp > 0 && p.available_round <= battle(c).unwrap().round)
            })
            .cloned()
            .unwrap_or(id);
        if !result.contains(&key) {
            result.push(key);
        }
    }
    result
}

// Resume an attack's remaining target responses before rolling its damage.
pub(super) fn prepare_targets(c: &mut Campaign) {
    let Some(q) = flow(c).ok().and_then(|f| f.queue.first()).cloned() else {
        return;
    };
    if q.targets.is_empty() {
        return;
    }
    let actor = q.actor_id.clone();
    let next_response = q
        .targets
        .iter()
        .find(|t| !t.responded && !t.avoided && fighter(c, &t.target_id).is_ok_and(|p| p.hp > 0));
    if let Some(t) = next_response {
        let key = t.target_id.clone();
        let automatic = key == actor || tactics::automatic_response(c, &q, &key);
        let q = flow_mut(c).queue.first_mut().unwrap();
        q.target_id = key.clone();
        if automatic {
            q.targets
                .iter_mut()
                .find(|t| t.target_id == key)
                .unwrap()
                .responded = true;
            prepare_targets(c);
        } else {
            flow_mut(c).window = true;
        }
        return;
    }
    if tactics::enabled(c) {
        flow_mut(c).window = false;
    }
    let mut ordered = q.targets.clone();
    if q.damage_modifiers.is_none() {
        ordered.sort_by_key(|t| {
            let score = fighter(c, &t.target_id)
                .and_then(|p| rules::effective_sheet(c, p))
                .map(|s| {
                    crate::creation::profile_stats(&s)
                        .get(&q.defense_key)
                        .copied()
                        .unwrap_or(0)
                        + fighter(c, &t.target_id)
                            .map(|p| abilities::defense(c, p, q.defense_key == "Defense"))
                            .unwrap_or(0)
                })
                .unwrap_or(0);
            let score = fighter(c, &t.target_id)
                .and_then(|p| field::order_defense(c, p, q.defense_key == "Defense", score))
                .unwrap_or(score);
            (-score, t.target_id.clone())
        });
    }
    let next_damage = ordered
        .iter()
        .find(|t| !t.resolved && !t.avoided && fighter(c, &t.target_id).is_ok_and(|p| p.hp > 0));
    let next_id = next_damage.map(|t| t.target_id.clone());
    let q = flow_mut(c).queue.first_mut().unwrap();
    q.targets = ordered;
    if let Some(id) = next_id {
        q.target_id = id;
    } else {
        q.cancelled = true;
    }
}

pub(super) fn responded(c: &mut Campaign) {
    let enabled = tactics::enabled(c);
    let q = if enabled {
        flow_mut(c).queue.first_mut()
    } else {
        flow_mut(c).queue.last_mut()
    };
    if let Some(q) = q {
        for t in &mut q.targets {
            if t.target_id == q.target_id {
                t.responded = true;
            }
        }
    }
}

pub(super) fn avoid(c: &mut Campaign, whole_area: bool) {
    let enabled = tactics::enabled(c);
    let q = if enabled {
        flow_mut(c).queue.first_mut().unwrap()
    } else {
        flow_mut(c).queue.last_mut().unwrap()
    };
    if q.targets.is_empty() {
        q.cancelled = true;
    } else {
        for t in &mut q.targets {
            if whole_area || t.target_id == q.target_id {
                t.avoided = true;
                t.responded = true;
            }
        }
    }
}

pub(super) fn finish_target(c: &mut Campaign) -> bool {
    let q = flow_mut(c).queue.first_mut().unwrap();
    if q.targets.is_empty() || q.cancelled {
        return false;
    }
    for t in &mut q.targets {
        if t.target_id == q.target_id {
            t.resolved = true;
        }
    }
    if !q.targets.iter().any(|t| !t.resolved && !t.avoided) {
        return false;
    }
    q.damage_done = false;
    q.damage = 0;
    q.raw_damage_successes = 0;
    prepare_targets(c);
    !flow(c).unwrap().queue[0].cancelled
}

pub(super) fn damage_dice(
    c: &Campaign,
    q: &QueuedAction,
    modifier: i32,
    cat: &Catalog,
) -> Result<u32> {
    let mut temp = old(c);
    install_pending(&mut temp, q);
    Ok(rules::damage_pool(&temp, &q.actor_id, q.sequence, modifier, cat)?.dice)
}

pub(super) fn cover_pool(c: &Campaign, id: &str, ally: &str, cat: &Catalog) -> Result<u32> {
    require_targeting(c)?;
    ready(c, id)?;
    let p = fighter(c, id)?;
    let t = fighter(c, ally)?;
    let f = flow(c)?;
    let q = incoming(c)?;
    let m = move_entry(cat, &q.move_name)?;
    if special::enabled(c)
        && (m["Attributes"]["SoundMove"] == true
            || ["Phantom Force", "Shadow Force"].contains(&q.move_name.as_str())
            || special::vulnerable(c, ally, &q.move_name))
    {
        return Err(error("This move bypasses Cover"));
    }
    if extra::enabled(c) && fighter(c, &q.actor_id)?.selected_ability == "Infiltrator" {
        return Err(error("Infiltrator bypasses Cover an Ally"));
    }
    if !f.window
        || q.target_id != ally
        || p.side != t.side
        || id == ally
        || q.actor_id == id
        || f.reacted.contains(id)
        || q.timing == "after"
        || q.priority >= 1
        || q.field_scope.is_some()
    {
        return Err(error(
            "Cover requires an ally under attack, an unused reaction slot, and Reaction 1 priority",
        ));
    }
    if flow(c)?
        .ranges
        .get(id)
        .is_some_and(|ids| !ids.iter().any(|v| v == ally))
    {
        return Err(error("Ally is outside the covering Pokemon's range"));
    }
    let s = rules::effective_sheet(c, p)?;
    if s.workflow.as_ref().is_none_or(|w| w.loyalty <= 2) {
        return Err(error("Cover an Ally requires reviewed Loyalty above 2"));
    }
    if p.used_moves.iter().any(|m| m == "Cover an Ally") {
        return Err(error("Cover an Ally has already been used this round"));
    }
    Ok(rules::score(&s, "Will")?.max(0) as u32)
}

pub(super) fn targeting_action(
    c: &Campaign,
    next: &mut Campaign,
    actor: &str,
    a: &Action,
    dice: &[u8],
    cat: &Catalog,
) -> Result<Option<String>> {
    let text = match a {
        Action::SetBattleRange {
            combatant_id,
            in_range_ids,
            reason: r,
        } => {
            require_targeting(c)?;
            gm(c, actor)?;
            reason(r)?;
            fighter(c, combatant_id)?;
            if !flow(c)?.queue.is_empty() {
                return Err(error("Change range only between action chains"));
            }
            if in_range_ids.iter().collect::<BTreeSet<_>>().len() != in_range_ids.len() {
                return Err(error("Duplicate range entries"));
            }
            for id in in_range_ids {
                fighter(c, id)?;
            }
            flow_mut(next)
                .ranges
                .insert(combatant_id.clone(), in_range_ids.clone());
            format!("GM confirmed range for {combatant_id}: {in_range_ids:?}. {r}")
        }
        Action::CoverAlly {
            combatant_id,
            ally_id,
            reason: r,
        } => {
            owner(c, actor, combatant_id)?;
            reason(r)?;
            let base = cover_pool(c, combatant_id, ally_id, cat)?;
            check_count(dice, dice_count(c, a, cat)?)?;
            let p = fighter(c, combatant_id)?;
            let saved_pain = p.runtime.pain_ignored;
            fighter_mut(next, combatant_id)?.runtime.pain_ignored = 3;
            let net =
                action_success(next, combatant_id, dice, base) + i32::from(p.runtime.fate_pending);
            fighter_mut(next, combatant_id)?.runtime.pain_ignored = saved_pain;
            break_cover(next, combatant_id);
            let p = fighter_mut(next, combatant_id)?;
            p.actions += 1;
            p.reactions += 1;
            p.used_moves.push("Cover an Ally".into());
            flow_mut(next).reacted.insert(combatant_id.clone());
            responded(next);
            if net > fighter(c, combatant_id)?.actions as i32 {
                flow_mut(next).covers.retain(|_, v| v != combatant_id);
                flow_mut(next)
                    .covers
                    .insert(ally_id.clone(), combatant_id.clone());
                let enabled = tactics::enabled(c);
                let q = if enabled {
                    flow_mut(next).queue.first_mut().unwrap()
                } else {
                    flow_mut(next).queue.last_mut().unwrap()
                };
                q.target_id = combatant_id.clone();
                let duplicate = q
                    .targets
                    .iter()
                    .any(|t| t.target_id == *combatant_id && t.target_id != *ally_id);
                if let Some(t) = q.targets.iter_mut().find(|t| t.target_id == *ally_id) {
                    if duplicate {
                        t.avoided = true;
                    } else {
                        t.target_id = combatant_id.clone();
                    }
                }
            }
            close(next);
            format!("Cover an Ally: {dice:?}, {net} successes. {r}")
        }
        Action::AdjudicateField {
            sequence,
            rounds,
            reason: r,
        } => {
            require_targeting(c)?;
            gm(c, actor)?;
            reason(r)?;
            if *rounds == Some(0) || rounds.is_some_and(|v| v > 1000) {
                return Err(error(
                    "Field duration must be 1-1000 rounds or encounter-long",
                ));
            }
            let q = current(c)?;
            if q.sequence != *sequence {
                return Err(error("Pending field action changed"));
            }
            let original = q.clone();
            let q = tactics::effect_target(next, &original, cat)?.unwrap_or(original);
            tactics::clear_payer(next);
            let scope = q
                .field_scope
                .clone()
                .ok_or_else(|| error("This action does not target the field"))?;
            flow_mut(next).field_effects.push(FieldEffect {
                sequence: *sequence,
                actor_id: q.actor_id.clone(),
                move_name: q.move_name.clone(),
                scope,
                rounds: *rounds,
                ruling: r.clone(),
            });
            finish(next);
            format!("GM recorded field effect; mechanical effects remain adjudicated: {r}")
        }
        Action::RemoveFieldEffect {
            sequence,
            reason: r,
        } => {
            require_targeting(c)?;
            gm(c, actor)?;
            reason(r)?;
            if !flow(c)?
                .field_effects
                .iter()
                .any(|v| v.sequence == *sequence)
            {
                return Err(error("No matching field effect"));
            }
            flow_mut(next)
                .field_effects
                .retain(|v| v.sequence != *sequence);
            format!("GM removed field effect: {r}")
        }
        _ => return Ok(None),
    };
    Ok(Some(text))
}
