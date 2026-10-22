use crate::{catalog::Catalog, model::*, sync};

// Dice results are injected, making transitions reproducible and testable.
pub fn transition(
    current: &Campaign,
    actor: &str,
    action: &Action,
    dice: Vec<u8>,
    catalog: &Catalog,
) -> Result<(Campaign, String)> {
    if [
        ENGINE,
        ITEM_ENGINE,
        EVENT_ENGINE,
        PROVENANCE_ENGINE,
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
        TURN_ENGINE,
    ]
    .contains(&current.engine.as_str())
    {
        return crate::combat::transition(current, actor, action, dice, catalog);
    }
    let mut c = current.clone();
    let gm = actor == c.gm_id;
    let gm_only = || {
        if gm {
            Ok(())
        } else {
            Err(AppError::new(
                "permission",
                "Only the GM can resolve or change the battle",
            ))
        }
    };
    let description = match action {
        Action::Tactics { .. }
        | Action::SpecialMove { .. }
        | Action::BattleStat { .. }
        | Action::Battlefield { .. }
        | Action::ResolveAbility { .. }
        | Action::HeldItem { .. }
        | Action::Form { .. }
        | Action::ChooseSheerForce { .. }
        | Action::InflictStatus { .. }
        | Action::AccuracyTargets { .. }
        | Action::SetBattleRange { .. }
        | Action::CoverAlly { .. }
        | Action::AdjudicateField { .. }
        | Action::RemoveFieldEffect { .. }
        | Action::StartTurns
        | Action::AdjustWill { .. }
        | Action::TakeHeldTurn { .. }
        | Action::BreakIce { .. }
        | Action::AdjudicateStatus { .. }
        | Action::AuthorizeWill { .. }
        | Action::PassTurn { .. }
        | Action::React { .. }
        | Action::PassReaction { .. }
        | Action::ResolveEffect { .. }
        | Action::AdjudicateEffect { .. }
        | Action::SetStatus { .. }
        | Action::StatusCheck { .. }
        | Action::SpendWill { .. } => {
            return Err(AppError::new(
                "version",
                "Adopt the new engine between battles",
            ))
        }
        Action::StartTeamBattle {
            sheet_ids,
            sides,
            active_sheet_ids,
            slots_per_side,
        } => {
            return crate::teams::start(
                current,
                actor,
                sheet_ids,
                sides,
                active_sheet_ids,
                *slots_per_side,
                catalog,
            );
        }
        Action::SwitchPokemon {
            outgoing_id,
            incoming_id,
            gm_override,
            reason,
        } => {
            let (next, description) = crate::teams::switch(
                current,
                actor,
                outgoing_id,
                incoming_id,
                *gm_override,
                reason,
            )?;
            c = next;
            description
        }
        Action::AdoptCatalog
        | Action::PublishTraining { .. }
        | Action::CancelTraining { .. }
        | Action::ResolveTraining { .. } => {
            return crate::encounter::transition(current, actor, action, catalog);
        }
        Action::SetRulesProfile { profile, reason } => {
            gm_only()?;
            validate_reason(reason)?;
            if c.battle.as_ref().is_some_and(|b| b.active)
                || !crate::creation::valid_profile(profile)
            {
                return Err(AppError::new(
                    "validation",
                    "Choose a supported rules profile between battles",
                ));
            }
            c.rules_profile = profile.clone();
            format!("GM set campaign rules profile: {reason}")
        }
        Action::ProposeWorkflow { request } => {
            if request.sheet.owner_id != actor {
                return Err(AppError::new(
                    "ownership",
                    "Only propose changes to your own character",
                ));
            }
            if c.proposals.len() >= 50 {
                return Err(AppError::new(
                    "limit",
                    "Resolve pending proposals before adding more",
                ));
            }
            if c.proposals.iter().any(|p| p.id == request.operation_id) {
                return Err(AppError::new("duplicate", "Proposal already submitted"));
            }
            let mut req = request.as_ref().clone();
            let old = c.sheets.get(&req.sheet.id);
            req.expected_revision = old.map_or(0, |s| s.revision);
            req.trainer_revision = req
                .sheet
                .trainer_id
                .as_ref()
                .and_then(|id| c.sheets.get(id))
                .map(|t| t.revision);
            c.proposals.push(WorkflowProposal {
                id: req.operation_id.clone(),
                actor_id: actor.into(),
                request: req,
            });
            format!(
                "Submitted {} proposal for {}: {}",
                request.kind, request.sheet.name, request.reason
            )
        }
        Action::ApproveWorkflow {
            proposal_id,
            accept,
            reason,
        } => {
            gm_only()?;
            validate_reason(reason)?;
            let index = c
                .proposals
                .iter()
                .position(|p| p.id == *proposal_id)
                .ok_or_else(|| AppError::new("missing", "Unknown proposal"))?;
            let proposal = c.proposals[index].clone();
            let req = &proposal.request;
            if *accept {
                if c.battle.as_ref().is_some_and(|b| {
                    b.active && b.combatants.iter().any(|p| p.sheet_id == req.sheet.id)
                }) {
                    return Err(AppError::new(
                        "battle_locked",
                        "Cannot approve changes to active combatants",
                    ));
                }
                let old = c.sheets.get(&req.sheet.id);
                if old.is_some_and(|s| s.owner_id != proposal.actor_id) {
                    return Err(AppError::new("ownership", "Proposal owner mismatch"));
                }
                let trainer = req
                    .sheet
                    .trainer_id
                    .as_ref()
                    .and_then(|id| c.sheets.get(id));
                let mut next =
                    crate::creation::apply(old, trainer, req, catalog, Some((&c.id, actor)))?;
                next.workflow
                    .as_mut()
                    .unwrap()
                    .ledger
                    .last_mut()
                    .unwrap()
                    .approval_reason = Some(reason.clone());
                if next.workflow.as_ref().unwrap().profile != c.rules_profile {
                    return Err(AppError::new(
                        "profile",
                        "Character rules profile must match the campaign",
                    ));
                }
                catalog.validate(&next)?;
                c.sheets.insert(next.id.clone(), next);
            }
            c.proposals.remove(index);
            format!(
                "GM {} progression proposal for {}: {reason}",
                if *accept { "accepted" } else { "rejected" },
                req.sheet.name
            )
        }
        Action::BattleConfig {
            automatic_validation,
            reason,
        } => {
            gm_only()?;
            validate_reason(reason)?;
            active(&mut c)?.automatic_validation = *automatic_validation;
            format!(
                "GM set automatic battle rule checks {}: {reason}",
                if *automatic_validation {
                    "ON"
                } else {
                    "OFF (manual adjudication)"
                }
            )
        }
        Action::BattleStats {
            combatant_id,
            stats,
            skills,
            max_hp,
            reason,
        } => {
            gm_only()?;
            validate_reason(reason)?;
            if !(1..=999).contains(max_hp)
                || stats.len() > 30
                || skills.len() > 30
                || stats
                    .iter()
                    .chain(skills.iter())
                    .any(|(k, v)| k.trim().is_empty() || k.len() > 50 || !(0..=999).contains(v))
            {
                return Err(AppError::new(
                    "validation",
                    "Battle overrides require bounded names, 0–999 scores and 1–999 maximum HP",
                ));
            }
            let p = combatant(active(&mut c)?, combatant_id)?;
            p.stat_overrides = stats.clone();
            p.skill_overrides = skills.clone();
            p.max_hp = *max_hp;
            p.hp = p.hp.min(*max_hp);
            format!("GM corrected battle-only stats for {}: {reason}", p.name)
        }
        Action::AbilityTransfer { .. } => {
            return Err(AppError::new(
                "version",
                "Ability transfer requires the current engine",
            ))
        }
        Action::RollbackBattle {
            event_sequence,
            reason,
        } => {
            gm_only()?;
            validate_reason(reason)?;
            if c.battle_undo
                .last()
                .is_some_and(|step| step.event_sequence != *event_sequence)
            {
                return Err(AppError::new(
                    "stale",
                    "The last battle step changed; review it before rolling back",
                ));
            }
            let checkpoint = c
                .battle_undo
                .pop()
                .ok_or_else(|| AppError::new("battle", "No battle step available to roll back"))?;
            if c.battle
                .as_ref()
                .is_none_or(|b| b.id != checkpoint.battle.id)
            {
                return Err(AppError::new(
                    "battle",
                    "Cannot roll back a different encounter",
                ));
            }
            if let Some(inheritance) = checkpoint.ability_inheritance {
                c.ability_inheritance = inheritance;
            }
            c.form_memory = checkpoint.form_memory;
            c.battle = Some(checkpoint.battle);
            c.finished_battles
                .truncate(checkpoint.finished_count as usize);
            for (key, resource) in checkpoint.resources {
                let s = c
                    .sheets
                    .get_mut(&key)
                    .ok_or_else(|| AppError::new("missing", "Missing rollback sheet"))?;
                if s.hp != resource.hp
                    || s.conditions != resource.conditions
                    || s.workflow.as_ref().map(|w| w.will) != resource.will
                {
                    if let (Some(will), Some(build)) = (resource.will, s.workflow.as_mut()) {
                        build.will = will;
                    }
                    s.hp = resource.hp;
                    s.conditions = resource.conditions;
                    s.revision = s
                        .revision
                        .checked_add(1)
                        .ok_or_else(|| AppError::new("limit", "Sheet revision exhausted"))?;
                }
                if let Some(equipment) = resource.equipment {
                    if s.held_items != equipment.held_items
                        || s.stored_items != equipment.stored_items
                        || s.inventory != equipment.inventory
                    {
                        s.held_items = equipment.held_items;
                        s.stored_items = equipment.stored_items;
                        s.inventory = equipment.inventory;
                        s.revision = s
                            .revision
                            .checked_add(1)
                            .ok_or_else(|| AppError::new("limit", "Sheet revision exhausted"))?;
                    }
                }
            }
            format!(
                "GM rolled back battle step #{} ({}): {reason}. Original events and dice retained",
                checkpoint.event_sequence, checkpoint.description
            )
        }
        Action::AssignGm { player_id } => {
            if actor != c.host_id() {
                return Err(AppError::new(
                    "permission",
                    "Only the host can choose the GM",
                ));
            }
            if player_id.is_empty() || player_id.len() > 100 {
                return Err(AppError::new("validation", "Select a room member"));
            }
            c.host_id = c.host_id().to_string();
            c.gm_id = player_id.clone();
            format!("Host appointed GM: {player_id}; validation remains on the host")
        }
        Action::UpgradeRules => {
            gm_only()?;
            if c.battle.as_ref().is_some_and(|b| b.active) {
                return Err(AppError::new(
                    "battle_locked",
                    "Upgrade rules between battles",
                ));
            }
            if ![
                ENGINE,
                ITEM_ENGINE,
                EVENT_ENGINE,
                PROVENANCE_ENGINE,
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
                TURN_ENGINE,
                PREVIOUS_ENGINE,
                LEGACY_ENGINE,
            ]
            .contains(&c.engine.as_str())
            {
                return Err(AppError::new("version", "Unsupported campaign engine"));
            }
            c.engine = ENGINE.into();
            "GM enabled verified Pokerole 3.0 core rolls; prior battle records preserved".into()
        }
        Action::Initiative { combatant_id } => {
            if !core_engine(&c.engine) {
                return Err(AppError::new(
                    "version",
                    "Upgrade campaign to enable core rolls",
                ));
            }
            validate_roll(1, "Initiative", &dice)?;
            let p = combatant(active(&mut c)?, combatant_id)?.clone();
            crate::teams::ready(&p, c.battle.as_ref().unwrap().round)?;
            if p.initiative_rolled && !gm {
                return Err(AppError::new(
                    "permission",
                    "Only the GM can reroll initiative to break a tie",
                ));
            }
            if !gm && p.owner_id != actor {
                return Err(AppError::new(
                    "permission",
                    "Only roll for your own combatant",
                ));
            }
            let s = crate::rules::effective_sheet(&c, &p)?;
            if s.rank == "Champion" && s.workflow.is_none() {
                return Err(AppError::new(
                    "manual_rule",
                    "Champion modifiers require GM adjudication",
                ));
            }
            let base = crate::rules::score(&s, "Dexterity")?
                + crate::rules::score(&s, "Alert")?
                + if crate::creation::rank(&s.rank).is_some_and(|r| r >= 6) {
                    3
                } else {
                    0
                };
            let item_bonus = crate::combat::items::initiative_bonus(&c, combatant_id);
            let total = base + i32::from(dice[0]) + item_bonus;
            let p_mut = combatant(active(&mut c)?, combatant_id)?;
            p_mut.initiative = total;
            p_mut.initiative_rolled = true;
            if item_bonus == 0 {
                format!("{} initiative: Dexterity + Alert ({base}) + {} = {total}; ties use base score, then GM reroll", p.name, dice[0])
            } else {
                format!("{} initiative: Dexterity + Alert ({base}) + {} + held-item bonus {item_bonus} = {total}; ties use base score, then GM reroll", p.name, dice[0])
            }
        }
        Action::Accuracy {
            combatant_id,
            target_id,
            move_name,
            pool_modifier,
            success_modifier,
            reason,
        } => {
            if let Some(target) = target_id {
                combatant(active(&mut c)?, target)?;
            }
            if combatant(active(&mut c)?, combatant_id)?
                .pending_damage
                .is_some()
            {
                return Err(AppError::new(
                    "battle",
                    "Resolve or cancel the pending damage first",
                ));
            }
            let sequence = c
                .revision
                .checked_add(1)
                .ok_or_else(|| AppError::new("limit", "Revision exhausted"))?;
            if !(-20..=20).contains(success_modifier) {
                return Err(AppError::new(
                    "validation",
                    "Success modifier must be between -20 and 20",
                ));
            }
            if !gm && (*pool_modifier != 0 || *success_modifier != 0) {
                return Err(AppError::new(
                    "permission",
                    "Only the GM can adjust core roll modifiers",
                ));
            }
            let (count, m) =
                crate::rules::accuracy(&c, combatant_id, move_name, *pool_modifier, catalog)?;
            validate_roll(count, reason, &dice)?;
            let p_snapshot = combatant(active(&mut c)?, combatant_id)?.clone();
            let sheet = crate::rules::effective_sheet(&c, &p_snapshot)?;
            let attribute = m["Accuracy1"].as_str().unwrap_or_default();
            let pain_exempt = attribute.split('/').any(|a| a.trim() == "Vitality")
                && crate::rules::score(&sheet, "Vitality")?
                    == crate::rules::score(&sheet, attribute)?;
            let p = combatant(active(&mut c)?, combatant_id)?;
            if !gm && p.owner_id != actor {
                return Err(AppError::new(
                    "permission",
                    "Only roll for your own combatant",
                ));
            }
            let required = p.actions + 1;
            let pain = if pain_exempt {
                0
            } else {
                crate::rules::pain(p.hp, p.max_hp)
            };
            let low = m["Attributes"]["AccuracyReduction"]
                .as_i64()
                .unwrap_or(0)
                .min(0) as i32;
            let raw = crate::rules::successes(&dice);
            let net = (raw - pain + low + success_modifier).max(0);
            let critical_margin = if m["Attributes"]["HighCritical"].as_bool() == Some(true) {
                2
            } else {
                3
            };
            let outcome = if net >= required as i32 + critical_margin {
                if m["Category"].as_str() == Some("Support") {
                    "exceptional accuracy (support move; no damage bonus)"
                } else {
                    "critical hit (+2 damage dice before other effects)"
                }
            } else if net >= required as i32 {
                "accuracy succeeded; resolve reactions before damage"
            } else if required as i32 - net >= 3 {
                "critical failure; GM determines consequences"
            } else {
                "accuracy failed"
            };
            p.actions += 1;
            if net >= required as i32 && m["Category"].as_str() != Some("Support") {
                p.pending_damage = target_id.as_ref().map(|target| PendingDamage {
                    sequence,
                    target_id: target.clone(),
                    move_name: move_name.clone(),
                    critical: net >= required as i32 + critical_margin,
                });
            }
            if !p.used_moves.contains(move_name) {
                p.used_moves.push(move_name.clone());
            }
            format!("{} — {}: {count} dice ({pool_modifier:+} GM pool); {raw} raw successes − {pain} pain {low:+} low accuracy {success_modifier:+} GM successes = {net}; requires {required}: {outcome}. {reason}", p.name, move_name)
        }
        Action::Damage {
            combatant_id,
            sequence,
            pool_modifier,
            success_modifier,
            damage_modifier,
            reason,
        } => {
            if !gm && (*pool_modifier != 0 || *success_modifier != 0 || *damage_modifier != 0) {
                return Err(AppError::new(
                    "permission",
                    "Only the GM can adjust damage modifiers",
                ));
            }
            if !(-20..=20).contains(success_modifier) || !(-20..=20).contains(damage_modifier) {
                return Err(AppError::new(
                    "validation",
                    "Success modifier must be between -20 and 20",
                ));
            }
            let p = combatant(active(&mut c)?, combatant_id)?.clone();
            if !gm && p.owner_id != actor {
                return Err(AppError::new("permission", "Only resolve your own attack"));
            }
            let pending = crate::rules::pending(&c, combatant_id, *sequence)?.clone();
            let result =
                crate::rules::damage_pool(&c, combatant_id, *sequence, *pool_modifier, catalog)?;
            validate_roll(result.dice, reason, &dice)?;
            let raw = crate::rules::successes(&dice);
            let net = (raw - result.pain + success_modifier).max(0);
            let damage = if result.immune {
                0
            } else {
                (net.max(1)
                    + if result.effectiveness > 0 && net == 0 {
                        0
                    } else {
                        result.effectiveness
                    }
                    + damage_modifier)
                    .max(0)
            };
            let target = combatant(active(&mut c)?, &pending.target_id)?;
            let before = target.hp;
            target.hp = (target.hp - damage).max(0);
            let target_name = target.name.clone();
            let after = target.hp;
            combatant(active(&mut c)?, combatant_id)?.pending_damage = None;
            format!("{} — {} damage to {}: {} dice (attribute {} + power {} − defense {}, STAB +{}, critical +{}, GM pool {pool_modifier:+}); {raw} successes − {} pain {success_modifier:+} GM successes = {net}; type {:+}, immune {}, GM damage {damage_modifier:+}; {} damage, HP {before} → {after}. Added effects, abilities, shields and lethal wounds require GM adjudication. {reason}", p.name, pending.move_name, target_name, result.dice, result.attribute, result.power, result.defense, result.stab, result.critical, result.pain, result.effectiveness, result.immune, damage)
        }
        Action::CancelDamage {
            combatant_id,
            sequence,
            reason,
        } => {
            gm_only()?;
            validate_roll(0, reason, &dice)?;
            crate::rules::pending(&c, combatant_id, *sequence)?;
            let p = combatant(active(&mut c)?, combatant_id)?;
            p.pending_damage = None;
            format!("{} pending damage cancelled: {reason}", p.name)
        }
        Action::Sync { sheets } => {
            if sheets
                .iter()
                .map(|s| &s.sheet.id)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != sheets.len()
            {
                return Err(AppError::new("duplicate", "Submit each sheet once"));
            }
            if sheets.len() > 100 {
                return Err(AppError::new("limit", "Submit at most 100 sheets"));
            }
            for submitted in sheets {
                let s = &submitted.sheet;
                if s.owner_id != actor
                    || submitted
                        .baseline
                        .as_ref()
                        .is_some_and(|b| b.id != s.id || b.owner_id != actor)
                {
                    return Err(AppError::new(
                        "ownership",
                        "Cannot submit another player's sheet",
                    ));
                }
                if c.battle
                    .as_ref()
                    .is_some_and(|b| b.active && b.combatants.iter().any(|p| p.sheet_id == s.id))
                {
                    return Err(AppError::new(
                        "battle_locked",
                        "Participating sheets are frozen until battle ends",
                    ));
                }
                let mut merged = if let Some(remote) = c.sheets.get(&s.id) {
                    if remote.owner_id != actor {
                        return Err(AppError::new(
                            "ownership",
                            "Sheet ID belongs to another player",
                        ));
                    }
                    let base = submitted.baseline.as_ref().ok_or_else(|| {
                        AppError::new(
                            "conflict",
                            "Existing sheet requires its synchronization baseline",
                        )
                    })?;
                    sync::merge(base, s, remote, &submitted.resolutions)?
                } else {
                    s.clone()
                };
                merged.revision = c.sheets.get(&s.id).map_or(1, |s| s.revision + 1);
                catalog.validate(&merged)?;
                c.sheets.insert(merged.id.clone(), merged);
            }
            for s in c.sheets.values().filter(|s| s.kind == "pokemon") {
                if !c.sheets.values().any(|t| {
                    Some(&t.id) == s.trainer_id.as_ref()
                        && t.kind == "trainer"
                        && t.owner_id == s.owner_id
                }) {
                    return Err(AppError::new(
                        "validation",
                        "Submit the Pokémon's trainer together with its squad",
                    ));
                }
            }
            for submitted in sheets {
                let merged = &c.sheets[&submitted.sheet.id];
                if core_engine(&c.engine) && merged.workflow.is_none() {
                    return Err(AppError::new(
                        "review_required",
                        "Unversioned characters need guided creation or GM review",
                    ));
                }
                let trainer = merged.trainer_id.as_ref().and_then(|id| c.sheets.get(id));
                if !crate::points::has_transfers(c.sheets.values()) {
                    crate::creation::verify_sync(
                        current.sheets.get(&merged.id),
                        merged,
                        trainer,
                        catalog,
                    )?;
                }
                if merged
                    .workflow
                    .as_ref()
                    .is_some_and(|b| b.profile != c.rules_profile)
                {
                    return Err(AppError::new(
                        "profile",
                        "Character and campaign rules profiles differ; request a reviewed rebuild",
                    ));
                }
            }
            if crate::points::has_transfers(c.sheets.values()) {
                crate::points::verify_batch(
                    &current.sheets,
                    &c.sheets,
                    &sheets
                        .iter()
                        .map(|s| s.sheet.id.clone())
                        .collect::<Vec<_>>(),
                    catalog,
                )?;
            }
            format!("Synchronized {} sheets", sheets.len())
        }
        Action::StartBattle { sheet_ids, sides } => {
            gm_only()?;
            if c.battle.as_ref().is_some_and(|b| b.active) {
                return Err(AppError::new("battle", "End the current battle first"));
            }
            if sheet_ids.is_empty() || sheet_ids.len() > 32 {
                return Err(AppError::new("validation", "Choose 1–32 combatants"));
            }
            let mut combatants = vec![];
            for key in sheet_ids {
                if combatants.iter().any(|p: &Combatant| p.sheet_id == *key) {
                    return Err(AppError::new("validation", "Duplicate combatant"));
                }
                let s = c
                    .sheets
                    .get(key)
                    .filter(|s| !s.archived)
                    .ok_or_else(|| AppError::new("missing", "Unknown or archived combatant"))?;
                if core_engine(&c.engine)
                    && s.workflow.as_ref().is_none_or(|b| {
                        !["strict", "exception"].contains(&b.status.as_str())
                            || b.profile != c.rules_profile
                    })
                {
                    return Err(AppError::new(
                        "review_required",
                        "Review this character and match the campaign rules profile before battle",
                    ));
                }
                combatants.push(Combatant {
                    runtime: Default::default(),
                    deployed: true,
                    field_slot: None,
                    available_round: 0,
                    id: id(),
                    sheet_id: s.id.clone(),
                    owner_id: s.owner_id.clone(),
                    name: s.name.clone(),
                    side: sides.get(key).cloned().unwrap_or_else(|| "Players".into()),
                    hp: s.hp,
                    max_hp: s.max_hp,
                    conditions: s.conditions.clone(),
                    actions: 0,
                    reactions: 0,
                    initiative: 0,
                    used_moves: vec![],
                    initiative_rolled: false,
                    stat_overrides: Default::default(),
                    skill_overrides: Default::default(),
                    pending_damage: None,
                    rules_profile: s
                        .workflow
                        .as_ref()
                        .map_or_else(RulesProfile::default, |b| b.profile.clone()),
                    selected_ability: s.ability.clone(),
                });
            }
            c.battle = Some(Battle {
                flow: None,
                slots_per_side: 0,
                trainer_switches: Default::default(),
                trainer_actions: Default::default(),
                id: id(),
                active: true,
                round: 1,
                phase: "GM declaration".into(),
                combatants,
                automatic_validation: core_engine(&c.engine),
            });
            format!(
                "Battle started with engine {}; roll initiative before taking turns",
                c.engine
            )
        }
        Action::NextRound => {
            gm_only()?;
            let b = active(&mut c)?;
            if b.combatants.iter().any(|p| p.pending_damage.is_some()) {
                return Err(AppError::new(
                    "battle",
                    "Resolve or cancel pending damage before the next round",
                ));
            }
            b.round = b
                .round
                .checked_add(1)
                .ok_or_else(|| AppError::new("limit", "Round limit reached"))?;
            for p in &mut b.combatants {
                p.actions = 0;
                p.reactions = 0;
                p.used_moves.clear();
                p.pending_damage = None;
            }
            b.trainer_switches.clear();
            b.trainer_actions.clear();
            format!("Round {} started; GM resolves duration effects", b.round)
        }
        Action::SetPhase { phase } => {
            gm_only()?;
            if phase.trim().is_empty() || phase.len() > 100 {
                return Err(AppError::new("validation", "Enter a short phase name"));
            }
            active(&mut c)?.phase = phase.clone();
            format!("Phase: {phase}")
        }
        Action::EndBattle => {
            gm_only()?;
            if active(&mut c)?
                .combatants
                .iter()
                .any(|p| p.pending_damage.is_some())
            {
                return Err(AppError::new(
                    "battle",
                    "Resolve or cancel pending damage before ending battle",
                ));
            }
            let b = active(&mut c)?;
            b.active = false;
            let finished = b.clone();
            let combatants = b.combatants.clone();
            c.finished_battles.push(finished);
            for p in combatants {
                if let Some(s) = c.sheets.get_mut(&p.sheet_id) {
                    s.hp = p.hp.clamp(0, s.max_hp);
                    s.conditions = p.conditions;
                    s.revision += 1;
                }
            }
            "Battle ended; HP and conditions persisted to campaign sheets".into()
        }
        Action::Roll {
            combatant_id,
            dice: count,
            reason,
            use_action,
        } => {
            validate_roll(*count, reason, &dice)?;
            let b = active(&mut c)?;
            let automatic = b.automatic_validation;
            let round = b.round;
            let p = combatant(b, combatant_id)?;
            crate::teams::ready(p, round)?;
            if !gm && p.owner_id != actor {
                return Err(AppError::new(
                    "permission",
                    "You can only roll for your own combatant",
                ));
            }
            if *use_action {
                check_action(p, automatic)?;
                p.actions = p
                    .actions
                    .checked_add(1)
                    .ok_or_else(|| AppError::new("limit", "Action counter exhausted"))?;
            }
            format!(
                "{} rolled {} dice: {} (GM determines successes and legality)",
                p.name, count, reason
            )
        }
        Action::Resolve {
            combatant_id,
            target_id,
            move_name,
            dice: count,
            hp_delta,
            condition,
            use_action,
            reason,
        } => {
            gm_only()?;
            if move_name.len() > 200 {
                return Err(AppError::new(
                    "validation",
                    "Move name exceeds 200 characters",
                ));
            }
            validate_roll(*count, reason, &dice)?;
            let b = active(&mut c)?;
            let source = b
                .combatants
                .iter()
                .find(|p| p.id == *combatant_id)
                .ok_or_else(|| AppError::new("missing", "Unknown acting combatant"))?
                .clone();
            crate::teams::ready(&source, b.round)?;
            let automatic = b.automatic_validation;
            if automatic && !move_name.is_empty() {
                let sheet = c
                    .sheets
                    .get(&source.sheet_id)
                    .ok_or_else(|| AppError::new("missing", "Missing source sheet"))?;
                if !sheet.moves.contains(move_name) {
                    return Err(AppError::new(
                        "validation",
                        "Acting Pokémon has not learned this move",
                    ));
                }
            }
            let b = active(&mut c)?;
            if *use_action {
                let round = b.round;
                let p = combatant(b, combatant_id)?;
                crate::teams::ready(p, round)?;
                check_action(p, automatic)?;
                if automatic && !move_name.is_empty() && p.used_moves.contains(move_name) {
                    return Err(AppError::new("battle", "Move already used this round"));
                }
                p.actions = p
                    .actions
                    .checked_add(1)
                    .ok_or_else(|| AppError::new("limit", "Action counter exhausted"))?;
                if !move_name.is_empty() && !p.used_moves.contains(move_name) {
                    p.used_moves.push(move_name.clone());
                }
            }
            let target = combatant(b, target_id)?;
            target.hp = target.hp.saturating_add(*hp_delta).clamp(0, target.max_hp);
            if let Some(condition) = condition {
                if condition.len() > 100 {
                    return Err(AppError::new("validation", "Condition name too long"));
                }
                if !condition.trim().is_empty() && !target.conditions.contains(condition) {
                    target.conditions.push(condition.clone());
                }
            }
            format!(
                "GM resolved {} → {}: {}; HP {:+}; {}",
                source.name, target.name, move_name, hp_delta, reason
            )
        }
        Action::Override {
            combatant_id,
            hp,
            conditions,
            actions,
            reactions,
            initiative,
            reason,
        } => {
            gm_only()?;
            if reason.trim().is_empty()
                || reason.len() > 1000
                || conditions.len() > 30
                || conditions.iter().any(|s| s.len() > 100)
                || *actions > 1000
                || *reactions > 1000
            {
                return Err(AppError::new(
                    "validation",
                    "Override requires a reason and bounded counters/conditions",
                ));
            }
            let p = combatant(active(&mut c)?, combatant_id)?;
            if *hp < 0 || *hp > p.max_hp {
                return Err(AppError::new("validation", "HP outside combatant bounds"));
            }
            p.hp = *hp;
            p.conditions = conditions.clone();
            p.actions = *actions;
            p.reactions = *reactions;
            p.initiative = *initiative;
            format!("GM override for {}: {}", p.name, reason)
        }
        Action::Reward {
            sheet_id,
            xp,
            reason,
        } => {
            gm_only()?;
            if *xp <= 0 || reason.trim().is_empty() || reason.len() > 1000 {
                return Err(AppError::new(
                    "validation",
                    "Reward needs positive Training Points and a reason",
                ));
            }
            if c.battle.as_ref().is_some_and(|b| b.active) {
                return Err(AppError::new("battle_locked", "Issue rewards after battle"));
            }
            if !c
                .sheets
                .get(sheet_id)
                .is_some_and(|s| s.kind == "trainer" && !s.archived && s.workflow.is_some())
            {
                return Err(AppError::new(
                    "validation",
                    "Award Training Points to a reviewed trainer pool",
                ));
            }
            if let Some(s) = c.sheets.get(sheet_id).filter(|s| s.workflow.is_some()) {
                let req = WorkflowRequest {
                    operation_id: id(),
                    expected_revision: s.revision,
                    trainer_revision: s
                        .trainer_id
                        .as_ref()
                        .and_then(|id| c.sheets.get(id))
                        .map(|t| t.revision),
                    rules_version: crate::creation::VERSION,
                    catalog_hash: catalog.hash.clone(),
                    kind: "award".into(),
                    sheet: s.clone(),
                    move_name: String::new(),
                    replacement: String::new(),
                    method: String::new(),
                    target_species: String::new(),
                    amount: *xp,
                    reason: reason.clone(),
                };
                let next = crate::creation::apply(
                    Some(s),
                    s.trainer_id.as_ref().and_then(|id| c.sheets.get(id)),
                    &req,
                    catalog,
                    Some((&c.id, actor)),
                )?;
                let name = next.name.clone();
                c.sheets.insert(next.id.clone(), next);
                format!("Awarded {xp} Training Points to {name}: {reason}")
            } else {
                unreachable!("Reviewed trainer pool checked above")
            }
        }
    };
    // Rollback restores battle/resources and campaign ability inheritance, never identities, receipts or event history.
    if matches!(
        action,
        Action::StartBattle { .. }
            | Action::StartTeamBattle { .. }
            | Action::Sync { .. }
            | Action::Reward { .. }
            | Action::UpgradeRules
            | Action::ApproveWorkflow { .. }
            | Action::SetRulesProfile { .. }
    ) {
        c.battle_undo.clear();
    } else if !matches!(
        action,
        Action::RollbackBattle { .. } | Action::AssignGm { .. }
    ) {
        if let Some(b) = current.battle.as_ref().filter(|b| b.active) {
            let resources = crate::combat::items::resource_sheets(current, &c)
                .into_iter()
                .filter_map(|sheet_id| {
                    current.sheets.get(&sheet_id).map(|s| {
                        (
                            s.id.clone(),
                            BattleResources {
                                equipment: crate::combat::items::enabled(current)
                                    .then(|| crate::combat::items::EquipmentResources::snapshot(s)),
                                will: s.workflow.as_ref().map(|w| w.will),
                                hp: s.hp,
                                conditions: s.conditions.clone(),
                            },
                        )
                    })
                })
                .collect();
            c.battle_undo.push(BattleCheckpoint {
                event_sequence: current
                    .revision
                    .checked_add(1)
                    .ok_or_else(|| AppError::new("limit", "Revision exhausted"))?,
                description: description.clone(),
                battle: b.clone(),
                form_memory: current.form_memory.clone(),
                ability_inheritance: Some(current.ability_inheritance.clone()),
                resources,
                finished_count: current.finished_battles.len() as u32,
            });
            if c.battle_undo.len() > 20 {
                c.battle_undo.remove(0);
            }
        }
    }
    Ok((c, description))
}
fn validate_reason(reason: &str) -> Result<()> {
    if reason.trim().is_empty() || reason.len() > 1000 {
        Err(AppError::new(
            "validation",
            "Provide a reason of at most 1000 characters",
        ))
    } else {
        Ok(())
    }
}
fn check_action(p: &Combatant, automatic: bool) -> Result<()> {
    if p.actions >= 1000 {
        return Err(AppError::new("limit", "Action counter limit reached"));
    }
    if automatic && (p.hp == 0 || p.actions >= 5) {
        return Err(AppError::new(
            "battle",
            "Combatant is fainted or has used all five actions",
        ));
    }
    Ok(())
}
fn active(c: &mut Campaign) -> Result<&mut Battle> {
    c.battle
        .as_mut()
        .filter(|b| b.active)
        .ok_or_else(|| AppError::new("battle", "No active battle"))
}
fn combatant<'a>(b: &'a mut Battle, key: &str) -> Result<&'a mut Combatant> {
    b.combatants
        .iter_mut()
        .find(|p| p.id == key && p.deployed)
        .ok_or_else(|| AppError::new("missing", "Unknown combatant"))
}
fn validate_roll(count: u32, reason: &str, dice: &[u8]) -> Result<()> {
    if count > 100
        || reason.trim().is_empty()
        || reason.len() > 1000
        || dice.len() != count as usize
        || dice.iter().any(|d| !(1..=6).contains(d))
    {
        return Err(AppError::new("validation", "Enter a reason and 0–100 dice"));
    }
    Ok(())
}
