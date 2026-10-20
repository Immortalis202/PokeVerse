use super::targeting_tests::{damage_command, pass, roll};
use super::*;
use crate::combat::{abilities, extra, field, reviewed};
fn setup(name: &str, a: &str, t: &str) -> (Campaign, String, String) {
    let (mut c, u, v) = super::special_tests::setup(name);
    let b = c.battle.as_mut().unwrap();
    b.combatants[0].selected_ability = a.into();
    b.combatants[1].selected_ability = t.into();
    b.flow.as_mut().unwrap().tactics.enabled = true;
    b.flow.as_mut().unwrap().reviewed_rules = true;
    (c, u, v)
}
fn p<'a>(c: &'a Campaign, id: &str) -> &'a Combatant {
    c.battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .find(|p| p.id == id)
        .unwrap()
}
fn pm<'a>(c: &'a mut Campaign, id: &str) -> &'a mut Combatant {
    c.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == id)
        .unwrap()
}
fn m(c: &Campaign, id: &str, name: &str) -> serde_json::Value {
    field::move_data(c, p(c, id), &catalog().find("moves", "Name", name).unwrap())
}
fn queued(c: &Campaign) -> crate::combat::QueuedAction {
    c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].clone()
}
fn pending(c: &Campaign, a: &str, t: &str, name: &str) -> Campaign {
    pass(&hit(c, a, t, name))
}
fn dmg(c: &Campaign) -> Campaign {
    roll(c, "gm", &damage_command(c), 4)
}
#[test]
fn reviewed_mold_breaker_bypasses_type_and_damage_immunity_without_mutating_identity() {
    for ability in ["Mold Breaker", "Teravolt", "Turboblaze"] {
        let (mut c, a, t) = setup("Thunder Shock", ability, "Volt Absorb");
        let sid = p(&c, &t).sheet_id.clone();
        c.sheets.get_mut(&sid).unwrap().species_id = Some(
            catalog().find("species", "Name", "Sandshrew").unwrap()["AppId"]
                .as_str()
                .unwrap()
                .into(),
        );
        let sheets = serde_json::to_value(&c.sheets).unwrap();
        let c = pending(&c, &a, &t, "Thunder Shock");
        let dice = rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap();
        assert!(dice > 0, "{ability}");
        let c = dmg(&c);
        assert!(p(&c, &t).hp < 20);
        assert_eq!(p(&c, &t).selected_ability, "Volt Absorb");
        assert!(c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events
            .is_empty());
        assert_eq!(serde_json::to_value(&c.sheets).unwrap(), sheets);
    }
}
#[test]
fn reviewed_bypass_does_not_remove_damage_reductions_or_ally_immunity() {
    let (c, a, t) = setup("Ember", "Mold Breaker", "Thick Fat");
    let data = m(&c, &a, "Ember");
    assert_eq!(abilities::reduction(&c, p(&c, &t), &data, 0, &a), 1);
    let mut ally = p(&c, &t).clone();
    ally.side = p(&c, &a).side.clone();
    ally.selected_ability = "Volt Absorb".into();
    assert!(abilities::damage_immune_from(
        &c,
        p(&c, &a),
        &ally,
        &serde_json::json!({"Type":"Electric"})
    ));
}
#[test]
fn reviewed_historical_engine_keeps_bypass_guard_and_old_accuracy() {
    let (mut c, a, t) = setup("Tackle", "Mold Breaker", "Tangled Feet");
    c.engine = TRANSFER_ENGINE.into();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .reviewed_rules = false;
    assert!(abilities::guard(&c, p(&c, &a)).is_err());
    assert_eq!(reviewed::accuracy_penalty(&c, p(&c, &a), p(&c, &t), -1), -1);
    assert!(!reviewed::damage_bypass(&c, p(&c, &a), p(&c, &t)));
}
#[test]
fn reviewed_keen_eye_and_tangled_feet_adjust_successes_not_action_budget() {
    let (mut c, a, t) = setup("Tackle", "Keen Eye", "Tangled Feet");
    pm(&mut c, &t).runtime.statuses.push(crate::combat::Status {
        kind: "Confusion".into(),
        detail: String::new(),
        rounds: None,
        progress: 0,
    });
    assert_eq!(reviewed::accuracy_penalty(&c, p(&c, &a), p(&c, &t), -3), 0);
    pm(&mut c, &a).selected_ability.clear();
    assert_eq!(reviewed::accuracy_penalty(&c, p(&c, &a), p(&c, &t), -3), -5);
    assert_eq!(p(&c, &a).actions, 0);
}
#[test]
fn reviewed_minds_eye_night_and_ghost_identity() {
    let (mut c, a, t) = setup("Tackle", "Mind's Eye", "");
    assert_ne!(m(&c, &a, "Tackle")["Attributes"]["NeverMiss"], true);
    assert!(extra::ignores_ghost(&c, p(&c, &a), "Normal", "Ghost"));
    let action = Action::Battlefield {
        command: field::FieldCommand::Night { night: true },
        reason: "Night gym".into(),
    };
    assert!(battle::transition(&c, "player", &action, vec![], &catalog()).is_err());
    c = battle::transition(&c, "gm", &action, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(m(&c, &a, "Tackle")["Attributes"]["NeverMiss"], true);
    let c = hit(&c, &a, &t, "Tackle");
    assert!(battle::transition(
        &c,
        "gm",
        &Action::React {
            combatant_id: t,
            kind: "evade".into(),
            move_name: "".into(),
            reason: "Evade".into()
        },
        vec![6; 7],
        &catalog()
    )
    .is_err());
}
#[test]
fn reviewed_victory_star_only_damaging_moves_and_range() {
    let (mut c, a, t) = setup("Tackle", "Victory Star", "");
    assert_eq!(m(&c, &a, "Tackle")["Attributes"]["NeverMiss"], true);
    assert_ne!(m(&c, &a, "Growl")["Attributes"]["NeverMiss"], true);
    pm(&mut c, &t).side = p(&c, &a).side.clone();
    assert_eq!(m(&c, &t, "Tackle")["Attributes"]["NeverMiss"], true);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ranges
        .insert(a.clone(), vec![]);
    assert_ne!(m(&c, &t, "Tackle")["Attributes"]["NeverMiss"], true);
    pm(&mut c, &a).selected_ability.clear();
    assert_ne!(m(&c, &a, "Tackle")["Attributes"]["NeverMiss"], true);
}
#[test]
fn reviewed_mycelium_support_gets_late_reaction_without_bypass() {
    let (c, a, t) = setup("Growl", "Mycelium Might", "");
    let data = m(&c, &a, "Growl");
    assert_eq!(data["Attributes"]["NeverMiss"], true);
    assert_eq!(data["Attributes"]["LateReaction"], "1");
    assert!(!reviewed::damage_bypass(&c, p(&c, &a), p(&c, &t)));
    assert_ne!(m(&c, &a, "Tackle")["Attributes"]["NeverMiss"], true);
}
#[test]
fn reviewed_supersweet_syrup_blocks_only_foe_evasion_in_range() {
    let (mut c, a, t) = setup("Tackle", "Supersweet Syrup", "");
    assert!(reviewed::syrup(&c, p(&c, &a), p(&c, &t)));
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ranges
        .insert(a.clone(), vec![]);
    assert!(!reviewed::syrup(&c, p(&c, &a), p(&c, &t)));
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ranges
        .clear();
    pm(&mut c, &t).side = p(&c, &a).side.clone();
    assert!(!reviewed::syrup(&c, p(&c, &a), p(&c, &t)));
}
#[test]
fn reviewed_motor_drive_immunity_and_once_boost_are_bypassed_by_mold_breaker() {
    for bypass in [false, true] {
        let (c, a, t) = setup(
            "Thunder Shock",
            if bypass { "Mold Breaker" } else { "" },
            "Motor Drive",
        );
        let c = pending(&c, &a, &t, "Thunder Shock");
        let q = queued(&c);
        let mut out = dmg(&c);
        assert_eq!(p(&out, &t).hp == 20, !bypass);
        assert_eq!(
            p(&out, &t)
                .runtime
                .ability
                .counters
                .get("boost:Dexterity")
                .copied()
                .unwrap_or(0),
            i32::from(!bypass)
        );
        reviewed::hit(&mut out, &q, &m(&c, &a, "Thunder Shock"), &catalog()).unwrap();
        assert_eq!(
            p(&out, &t)
                .runtime
                .ability
                .counters
                .get("boost:Dexterity")
                .copied()
                .unwrap_or(0),
            i32::from(!bypass)
        );
    }
}
#[test]
fn reviewed_justified_caps_at_three_and_requires_dark_hit() {
    let (c, a, t) = setup("Bite", "", "Justified");
    let mut out = pending(&c, &a, &t, "Bite");
    let q = queued(&out);
    for _ in 0..5 {
        reviewed::hit(&mut out, &q, &m(&c, &a, "Bite"), &catalog()).unwrap();
    }
    assert_eq!(p(&out, &t).runtime.ability.counters["boost:Strength"], 3);
}
#[test]
fn reviewed_cotton_down_uses_ability_channel_and_skips_ranged_contact() {
    let (c, a, t) = setup("Tackle", "", "Cotton Down");
    let mut out = pending(&c, &a, &t, "Tackle");
    let q = queued(&out);
    let mut data = m(&c, &a, "Tackle");
    data["Attributes"]["Ranged"] = serde_json::json!(true);
    reviewed::hit(&mut out, &q, &data, &catalog()).unwrap();
    assert!(!p(&out, &a)
        .runtime
        .ability
        .counters
        .contains_key("debuff:Dexterity"));
    reviewed::hit(&mut out, &q, &m(&c, &a, "Tackle"), &catalog()).unwrap();
    assert_eq!(p(&out, &a).runtime.ability.counters["debuff:Dexterity"], -1);
    assert!(p(&out, &a).runtime.modifiers.is_empty());
    assert_eq!(
        rules::effective_sheet(&out, p(&out, &a)).unwrap().stats["Dexterity"],
        rules::effective_sheet(&c, p(&c, &a)).unwrap().stats["Dexterity"] - 1
    );
}
#[test]
fn reviewed_seed_sower_is_four_rounds_does_not_refresh_and_overlap_requires_ruling() {
    let (mut c, _a, t) = setup("Tackle", "", "Seed Sower");
    let target = p(&c, &t).clone();
    field::seed_sower(&mut c, &target).unwrap();
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .battlefield
            .rules[0]
            .rounds,
        Some(4)
    );
    field::expire(&mut c).unwrap();
    field::seed_sower(&mut c, &target).unwrap();
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .battlefield
            .rules[0]
            .rounds,
        Some(3)
    );
    let f = &mut c
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield;
    f.rules[0].kind = "Electric Terrain".into();
    field::seed_sower(&mut c, &target).unwrap();
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .battlefield
            .events[0]
            .kind,
        "terrain-ruling"
    );
}
#[test]
fn reviewed_screen_cleaner_removes_both_sides_force_fields_without_erasing_weather() {
    let (mut c, _, _) = setup("Tackle", "Screen Cleaner", "");
    let f = &mut c
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield;
    f.natural = Some(field::Weather::Rain);
    for (i, name) in ["Reflect", "Light Screen", "Aurora Veil", "Trick Room"]
        .iter()
        .enumerate()
    {
        f.rules.push(field::FieldRule {
            id: i.to_string(),
            kind: (*name).into(),
            source: "test".into(),
            scope: "all".into(),
            rounds: Some(4),
            parameter: 1,
            started_round: 1,
        });
    }
    let before = c.clone();
    extra::lifecycle(&before, &mut c, &catalog()).unwrap();
    let f = &c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .battlefield;
    assert_eq!(f.rules.len(), 1);
    assert_eq!(f.rules[0].kind, "Trick Room");
    assert_eq!(f.natural, Some(field::Weather::Rain));
}
#[test]
fn reviewed_toxic_debris_places_one_persistent_hazard_on_attackers_side() {
    let (c, a, t) = setup("Tackle", "", "Toxic Debris");
    let mut out = pending(&c, &a, &t, "Tackle");
    let q = queued(&out);
    for _ in 0..2 {
        reviewed::hit(&mut out, &q, &m(&c, &a, "Tackle"), &catalog()).unwrap();
    }
    let f = &out
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .battlefield;
    assert_eq!(f.rules.len(), 1);
    assert_eq!(f.rules[0].kind, "Toxic Spikes");
    assert_eq!(f.rules[0].scope, format!("side:{}", p(&c, &a).side));
    assert_eq!(f.rules[0].rounds, None);
}
#[test]
fn reviewed_extreme_bypasses_status_immunity_and_decoys_without_negating_target() {
    let (mut c, a, t) = setup("Toxic", "Teravolt", "Immunity");
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some(
        catalog().find("species", "Name", "Magnemite").unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
    abilities::inflict(&mut c, &a, &t, "Poison", "", None, &catalog()).unwrap();
    assert!(p(&c, &t)
        .runtime
        .statuses
        .iter()
        .any(|s| s.kind == "Poison"));
    assert_eq!(p(&c, &t).selected_ability, "Immunity");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .decoys
        .enabled = true;
    crate::combat::decoys::cast(&mut c, &t, "Substitute", &catalog()).unwrap();
    assert!(!crate::combat::decoys::protected_subject(
        &c,
        &a,
        &t,
        &m(&c, &a, "Toxic")
    ));
}
#[test]
fn reviewed_bypass_preview_explains_scope() {
    let (c, a, t) = setup("Tackle", "Mold Breaker", "");
    let c = pending(&c, &a, &t, "Tackle");
    let preview = crate::combat::preview(&c, &damage_command(&c), &catalog()).unwrap();
    assert!(preview.warnings.iter().any(|v| v.contains("Mold Breaker")));
}

#[test]
fn reviewed_early_bird_halves_sleep_duration_and_wakes_at_two_except_rest() {
    for rest in [false, true] {
        let (mut c, a, _) = setup("Tackle", "Early Bird", "");
        abilities::inflict(
            &mut c,
            &a,
            &a,
            "Sleep",
            if rest { "Rest" } else { "" },
            Some(5),
            &catalog(),
        )
        .unwrap();
        assert_eq!(
            p(&c, &a).runtime.statuses[0].rounds,
            Some(if rest { 5 } else { 3 })
        );
        let action = Action::StatusCheck {
            combatant_id: a.clone(),
            kind: "sleep".into(),
            reason: "Wake".into(),
        };
        let n = rules::dice_count(&c, &action, &catalog()).unwrap();
        let mut dice = vec![1; n as usize];
        dice[0] = 4;
        dice[1] = 4;
        let next = battle::transition(&c, "player", &action, dice, &catalog())
            .unwrap()
            .0;
        assert_eq!(
            p(&next, &a)
                .runtime
                .statuses
                .iter()
                .any(|s| s.kind == "Sleep"),
            rest
        );
        assert_eq!(p(&next, &a).actions, 1);
    }
}
#[test]
fn reviewed_toxic_spikes_entry_roll_is_required_and_does_not_repeat_on_ability_refresh() {
    for face in [1, 6] {
        let (mut c, a, t) = setup("Tackle", "", "Toxic Debris");
        let target = p(&c, &t).clone();
        let actor = p(&c, &a).clone();
        field::toxic_debris(&mut c, &target, &actor);
        let mut before = c.clone();
        pm(&mut before, &a).deployed = false;
        field::lifecycle(&before, &mut c, &catalog()).unwrap();
        let e = c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .battlefield
            .events[0]
            .clone();
        assert_eq!(e.kind, "toxic-spikes");
        let cmd = field::FieldCommand::Resolve {
            event_id: e.id,
            accept: true,
        };
        assert_eq!(field::dice(&c, &cmd).unwrap(), 1);
        assert!(field::command(&mut c, "player", &cmd, &[face], "Entry", &catalog()).is_err());
        field::command(&mut c, "gm", &cmd, &[face], "Entry", &catalog()).unwrap();
        assert_eq!(
            p(&c, &a).runtime.statuses[0].kind,
            if face == 6 {
                "Badly Poisoned"
            } else {
                "Poison"
            }
        );
        let before = c.clone();
        field::lifecycle(&before, &mut c, &catalog()).unwrap();
        assert!(c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .battlefield
            .events
            .is_empty());
    }
}
#[test]
fn reviewed_unique_damage_stalemate_is_explicit_and_common_bypass_cannot_erase_wonder_guard() {
    let (c, a, t) = setup("Tackle", "Mold Breaker", "Wonder Guard");
    assert!(!reviewed::damage_bypass(&c, p(&c, &a), p(&c, &t)));
    let (mut c, a, t) = setup("Tackle", "Teravolt", "Wonder Guard");
    assert!(reviewed::unique_damage_guard(&c, p(&c, &a), p(&c, &t), 0).is_err());
    assert!(reviewed::unique_damage_guard(&c, p(&c, &a), p(&c, &t), 1).is_ok());
    pm(&mut c, &a).selected_ability.clear();
    assert!(reviewed::unique_damage_guard(&c, p(&c, &a), p(&c, &t), 0).is_ok());
}

#[test]
fn reviewed_gas_negation_and_scene_copy_control_bypass_identity() {
    let (mut c, a, t) = setup("Tackle", "Run Away", "Neutralizing Gas");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_changes
        .enabled = true;
    let old = c.clone();
    crate::combat::ability_changes::lifecycle(&old, &mut c, &catalog()).unwrap();
    assert!(p(&c, &a).selected_ability.is_empty());
    let state = &mut c
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_changes;
    state.assignments.insert(a.clone(), "Mold Breaker".into());
    let old = c.clone();
    crate::combat::ability_changes::lifecycle(&old, &mut c, &catalog()).unwrap();
    assert!(p(&c, &a).selected_ability.is_empty());
    pm(&mut c, &t).deployed = false;
    let old = c.clone();
    crate::combat::ability_changes::lifecycle(&old, &mut c, &catalog()).unwrap();
    assert_eq!(p(&c, &a).selected_ability, "Mold Breaker");
    let mut foe = p(&c, &t).clone();
    foe.side = "Opponents".into();
    assert!(reviewed::damage_bypass(&c, p(&c, &a), &foe));
}
#[test]
fn reviewed_shared_damage_retains_frozen_bypass_source_after_live_identity_changes() {
    let (mut c, a, t, h) = super::targeting_tests::multi("Swift");
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.reviewed_rules = true;
    f.extra.enabled = true;
    f.special.enabled = true;
    f.ability_rules = true;
    f.battlefield.enabled = true;
    f.ability_changes.enabled = true;
    pm(&mut c, &a).selected_ability = "Mold Breaker".into();
    for id in [&t, &h] {
        let sid = p(&c, id).sheet_id.clone();
        c.sheets.get_mut(&sid).unwrap().species_id = Some("0092:gastly".into());
        pm(&mut c, id).selected_ability = "Levitate".into();
    }
    let action = super::targeting_tests::attack(&a, vec![t, h], "Swift");
    let mut c = roll(&c, "player", &action, 6);
    while c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
        c = pass(&c);
    }
    let mut c = dmg(&c);
    assert!(queued(&c).damage_context.is_some());
    pm(&mut c, &a).selected_ability.clear();
    let q = queued(&c);
    assert_eq!(
        reviewed::source(&c, &q).unwrap().selected_ability,
        "Mold Breaker"
    );
    assert!(rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap() > 0);
}

#[test]
fn reviewed_mold_breaker_preserves_reactive_stamina_boost() {
    let (c, a, t) = setup("Tackle", "Mold Breaker", "Stamina");
    let c = dmg(&pending(&c, &a, &t, "Tackle"));
    assert_eq!(p(&c, &t).runtime.ability.counters["boost:Defense"], 1);
    assert_eq!(
        p(&c, &t).runtime.ability.counters["boost:SpecialDefense"],
        1
    );
}
#[test]
fn reviewed_wonder_guard_does_not_prevent_independent_type_bypass() {
    let (mut c, a, t) = setup("Thunder Shock", "Mold Breaker", "Wonder Guard");
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some("0195:quagsire".into());
    let (effect, immune) =
        rules::type_matchup(&c, p(&c, &a), p(&c, &t), "Electric", &catalog()).unwrap();
    assert_eq!(effect, 1);
    assert!(!immune);
    assert!(!extra::source_immune(&c, p(&c, &a), p(&c, &t), effect));
}

#[test]
fn reviewed_tidy_up_and_defog_remove_toxic_debris_hazards() {
    for name in ["Tidy Up", "Defog"] {
        let (mut c, a, t) = setup(name, "", "Toxic Debris");
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .decoys
            .enabled = true;
        let target = p(&c, &t).clone();
        let actor = p(&c, &a).clone();
        field::toxic_debris(&mut c, &target, &actor);
        let action = super::targeting_tests::attack(&a, vec![], name);
        let c = roll(&c, "player", &action, 6);
        let q = queued(&c);
        let c = battle::transition(
            &c,
            "player",
            &Action::ResolveEffect {
                sequence: q.sequence,
                reason: "Clear entry hazards".into(),
            },
            vec![],
            &catalog(),
        )
        .unwrap()
        .0;
        assert!(
            !c.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .battlefield
                .rules
                .iter()
                .any(|r| r.kind == "Toxic Spikes"),
            "{name}"
        );
    }
}

#[test]
fn reviewed_supersweet_syrup_respects_rank_target_caps() {
    let (mut c, a, t) = setup("Tackle", "Supersweet Syrup", "");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().rank = "Starter".into();
    let mut extra = p(&c, &t).clone();
    extra.id = "000-first-foe".into();
    c.battle.as_mut().unwrap().combatants.push(extra.clone());
    assert!(reviewed::syrup(&c, p(&c, &a), &extra));
    assert!(!reviewed::syrup(&c, p(&c, &a), p(&c, &t)));
}
