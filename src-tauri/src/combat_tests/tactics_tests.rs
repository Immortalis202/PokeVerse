use super::*;
use crate::combat::{field, tactics};
fn setup(name: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = super::special_tests::setup(name);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .tactics
        .enabled = true;
    c.battle.as_mut().unwrap().slots_per_side = 3;
    for p in &mut c.battle.as_mut().unwrap().combatants {
        p.field_slot = Some(0);
    }
    let ids = c
        .battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .map(|p| p.id.clone())
        .collect::<Vec<_>>();
    for id in ids {
        let p = c
            .battle
            .as_ref()
            .unwrap()
            .combatants
            .iter()
            .find(|p| p.id == id)
            .unwrap()
            .clone();
        let mut tr = c.sheets[&p.sheet_id].clone();
        tr.id = format!("trainer-{}", p.owner_id);
        tr.kind = "trainer".into();
        c.sheets.get_mut(&p.sheet_id).unwrap().trainer_id = Some(tr.id.clone());
        c.sheets.insert(tr.id.clone(), tr);
    }
    (c, a, t)
}
fn fighter<'a>(c: &'a Campaign, id: &str) -> &'a Combatant {
    c.battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .find(|p| p.id == id)
        .unwrap()
}
fn f(c: &Campaign) -> &TurnFlow {
    c.battle.as_ref().unwrap().flow.as_ref().unwrap()
}
fn learn(c: &mut Campaign, id: &str, name: &str) {
    let sheet = fighter(c, id).sheet_id.clone();
    c.sheets.get_mut(&sheet).unwrap().moves.push(name.into());
}
fn ability(c: &mut Campaign, id: &str, name: &str) {
    c.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == id)
        .unwrap()
        .selected_ability = name.into();
}
fn add(c: &mut Campaign, id: &str, deployed: bool) -> String {
    let mut p = fighter(c, id).clone();
    let mut s = c.sheets[&p.sheet_id].clone();
    p.id = crate::model::id();
    s.id = crate::model::id();
    p.sheet_id = s.id.clone();
    p.name = format!("Added {}", p.id);
    p.deployed = deployed;
    p.field_slot = if deployed { Some(1) } else { None };
    p.initiative_rolled = deployed;
    p.actions = 0;
    let key = p.id.clone();
    c.sheets.insert(s.id.clone(), s);
    c.battle.as_mut().unwrap().combatants.push(p);
    if deployed {
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .order
            .push(key.clone());
    }
    key
}
fn run(c: &Campaign, actor: &str, a: &Action, face: u8) -> Campaign {
    let n = rules::dice_count(c, a, &catalog()).unwrap();
    let (mut c, _) = battle::transition(c, actor, a, vec![face; n as usize], &catalog()).unwrap();
    c.revision += 1;
    c
}
fn action(a: &str, targets: Vec<String>, name: &str) -> Action {
    Action::AccuracyTargets {
        combatant_id: a.into(),
        target_ids: targets,
        move_name: name.into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Reviewed tactical action".into(),
    }
}
fn hit(c: &Campaign, a: &str, t: &str, name: &str) -> Campaign {
    run(c, "gm", &action(a, vec![t.into()], name), 4)
}
fn pass(c: &Campaign) -> Campaign {
    run(
        c,
        "gm",
        &Action::PassReaction {
            combatant_id: f(c).queue[0].target_id.clone(),
            reason: "Take hit".into(),
        },
        4,
    )
}
fn effect(c: &Campaign) -> Campaign {
    run(
        c,
        "gm",
        &Action::ResolveEffect {
            sequence: f(c).queue[0].sequence,
            reason: "Apply reviewed effect".into(),
        },
        6,
    )
}
fn damage(c: &Campaign) -> Campaign {
    run(c, "gm", &super::targeting_tests::damage_command(c), 4)
}
fn tac(command: tactics::TacticsCommand) -> Action {
    Action::Tactics {
        command,
        reason: "Reviewed tactical choice".into(),
    }
}
fn turn(c: &mut Campaign, id: &str) {
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.queue.clear();
    f.window = false;
    f.cursor = f.order.iter().position(|v| v == id).unwrap();
    f.turn += 1;
    f.reacted.clear();
}
#[test]
fn tactics_multi_target_reaction_resolves_each_response_then_resumes_the_original_attack() {
    let (mut c, a, t) = setup("Tackle");
    let ally = add(&mut c, &a, true);
    learn(&mut c, &t, "Air Cutter");
    ability(&mut c, &t, "Gale Wings");
    let c = hit(&c, &a, &t, "Tackle");
    let start = f(&c).turn;
    let c = run(
        &c,
        "gm",
        &tac(tactics::TacticsCommand::Reaction {
            combatant_id: t.clone(),
            move_name: "Air Cutter".into(),
            target_ids: vec![a.clone(), ally],
        }),
        4,
    );
    assert_eq!(f(&c).queue[0].move_name, "Air Cutter");
    assert_eq!(f(&c).queue[0].targets.len(), 2);
    assert!(f(&c).window);
    assert_eq!(fighter(&c, &t).actions, 1);
    let c = pass(&c);
    let c = pass(&c);
    assert!(!f(&c).window);
    let c = damage(&c);
    let mut c = effect(&c);
    if f(&c).queue[0].move_name == "Air Cutter" {
        c = damage(&c);
        c = effect(&c);
    }
    assert_eq!(f(&c).queue[0].move_name, "Tackle");
    assert_eq!(f(&c).turn, start);
    assert!(!f(&c).window);
    let c = damage(&c);
    assert!(f(&c).queue.is_empty() || f(&c).queue[0].damage_done);
}
#[test]
fn tactics_redirection_uses_live_ability_sources_and_bypass_preserves_the_chosen_target() {
    for (name, ability_name) in [
        ("Thunder Shock", "Lightning Rod"),
        ("Water Gun", "Storm Drain"),
    ] {
        let (mut c, a, t) = setup(name);
        let rod = add(&mut c, &t, true);
        ability(&mut c, &rod, ability_name);
        let c = hit(&c, &a, &t, name);
        assert_eq!(f(&c).queue[0].target_id, rod);
        let c = pass(&c);
        let n =
            rules::dice_count(&c, &super::targeting_tests::damage_command(&c), &catalog()).unwrap();
        assert_eq!(n, 0);
        let c = damage(&c);
        assert_eq!(fighter(&c, &rod).hp, 20);
        assert_eq!(fighter(&c, &rod).runtime.ability.counters[ability_name], 1);
        let (mut c, a, t) = setup(name);
        let rod = add(&mut c, &t, true);
        ability(&mut c, &rod, ability_name);
        ability(&mut c, &a, "Stalwart");
        let c = hit(&c, &a, &t, name);
        assert_eq!(f(&c).queue[0].target_id, t);
    }
}
#[test]
fn tactics_competing_redirectors_require_an_exact_gm_choice() {
    let (mut c, a, t) = setup("Water Gun");
    let other = add(&mut c, &t, true);
    ability(&mut c, &t, "Storm Drain");
    ability(&mut c, &other, "Storm Drain");
    let a1 = action(&a, vec![t.clone()], "Water Gun");
    assert!(crate::combat::preview(&c, &a1, &catalog()).is_err());
    let choose = tac(tactics::TacticsCommand::Redirect {
        combatant_id: a.clone(),
        recipient_id: other.clone(),
    });
    assert!(battle::transition(&c, "player", &choose, vec![], &catalog()).is_err());
    let c = run(&c, "gm", &choose, 4);
    let c = hit(&c, &a, &t, "Water Gun");
    assert_eq!(f(&c).queue[0].target_id, other);
}
#[test]
fn tactics_reaction_blockers_leave_maneuvers_available_and_check_allied_range() {
    for name in ["Dazzling", "Queenly Majesty", "Armor Tail"] {
        let (mut c, a, t) = setup("Quick Attack");
        ability(&mut c, &t, name);
        let a1 = action(&a, vec![t.clone()], "Quick Attack");
        let n = rules::dice_count(&c, &a1, &catalog()).unwrap();
        assert!(
            battle::transition(&c, "gm", &a1, vec![4; n as usize], &catalog()).is_err(),
            "{name}"
        );
        let (mut c, a, t) = setup("Tackle");
        ability(&mut c, &a, name);
        let c = hit(&c, &a, &t, "Tackle");
        let a1 = Action::React {
            combatant_id: t,
            kind: "clash".into(),
            move_name: "Tackle".into(),
            reason: "Maneuvers remain legal".into(),
        };
        assert!(rules::dice_count(&c, &a1, &catalog()).is_ok());
    }
    let (mut c, a, t) = setup("Quick Attack");
    let guard = add(&mut c, &t, true);
    ability(&mut c, &guard, "Armor Tail");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ranges
        .insert(guard, vec![]);
    let c = hit(&c, &a, &t, "Quick Attack");
    assert_eq!(f(&c).queue[0].target_id, t);
}
#[test]
fn tactics_magic_bounce_reflects_once_without_charging_the_reflector_an_action() {
    let (mut c, a, t) = setup("Growl");
    ability(&mut c, &t, "Magic Bounce");
    ability(&mut c, &a, "Magic Bounce");
    let c = hit(&c, &a, &t, "Growl");
    assert!(!f(&c).window);
    let c = effect(&c);
    assert_eq!(fighter(&c, &a).runtime.modifiers["Strength"], -1);
    assert!(fighter(&c, &t).runtime.modifiers.is_empty());
    assert_eq!(fighter(&c, &t).actions, 0);
    assert!(f(&c).queue.is_empty());
}
#[test]
fn tactics_good_as_gold_blocks_foe_and_ally_support_but_not_self_or_field_effects() {
    let (mut c, a, t) = setup("Growl");
    ability(&mut c, &t, "Good As Gold");
    let c = hit(&c, &a, &t, "Growl");
    let c = effect(&c);
    assert!(fighter(&c, &t).runtime.modifiers.is_empty());
    let (mut c, a, _) = setup("Recover");
    ability(&mut c, &a, "Good As Gold");
    c.battle.as_mut().unwrap().combatants[0].hp = 10;
    let c = hit(&c, &a, &a, "Recover");
    let c = effect(&c);
    assert_eq!(fighter(&c, &a).hp, 13);
}
#[test]
fn tactics_foe_switches_use_only_the_owners_registered_reserves_and_suction_cups_blocks() {
    let (mut c, a, t) = setup("Roar");
    let reserve = add(&mut c, &t, false);
    let c = hit(&c, &a, &t, "Roar");
    let c = pass(&c);
    let c = effect(&c);
    assert_eq!(f(&c).tactics.switches.len(), 1);
    let req = f(&c).tactics.switches[0].id.clone();
    let command = tac(tactics::TacticsCommand::Switch {
        request_id: req,
        incoming_id: Some(reserve.clone()),
        ruling: "replace".into(),
    });
    assert!(battle::transition(&c, "player", &command, vec![], &catalog()).is_err());
    let owner = fighter(&c, &t).owner_id.clone();
    let c = run(&c, &owner, &command, 4);
    assert!(!fighter(&c, &t).deployed);
    assert!(fighter(&c, &reserve).deployed);
    assert_eq!(fighter(&c, &reserve).available_round, 1);
    assert_eq!(fighter(&c, &reserve).initiative, fighter(&c, &t).initiative);
    assert!(f(&c).tactics.switches.is_empty());
    let (mut c, a, t) = setup("Roar");
    ability(&mut c, &t, "Suction Cups");
    let c = hit(&c, &a, &t, "Roar");
    let c = effect(&pass(&c));
    assert!(f(&c).tactics.switches.is_empty());
    assert!(fighter(&c, &t).deployed);
}
#[test]
fn tactics_user_switches_preserve_the_slot_and_remaining_actions_without_changing_sheets() {
    let (mut c, a, t) = setup("U-Turn");
    let reserve = add(&mut c, &a, false);
    let sheets = serde_json::to_value(&c.sheets).unwrap();
    let c = hit(&c, &a, &t, "U-Turn");
    let c = damage(&pass(&c));
    let c = effect(&c);
    let req = f(&c).tactics.switches[0].id.clone();
    let c = run(
        &c,
        "gm",
        &tac(tactics::TacticsCommand::Switch {
            request_id: req,
            incoming_id: Some(reserve.clone()),
            ruling: "replace".into(),
        }),
        4,
    );
    assert_eq!(fighter(&c, &reserve).actions, 1);
    assert_eq!(fighter(&c, &reserve).field_slot, Some(0));
    assert_eq!(serde_json::to_value(&c.sheets).unwrap(), sheets);
}
#[test]
fn tactics_ability_switch_crossing_is_once_per_threshold_and_not_an_automatic_battle_end() {
    let (mut c, a, t) = setup("Tackle");
    ability(&mut c, &t, "Wimp Out");
    let reserve = add(&mut c, &t, false);
    let before = c.clone();
    c.battle.as_mut().unwrap().combatants[1].hp = 10;
    tactics::lifecycle(&before, &mut c, &catalog()).unwrap();
    assert_eq!(f(&c).tactics.switches.len(), 1);
    let again = c.clone();
    tactics::lifecycle(&again, &mut c, &catalog()).unwrap();
    assert_eq!(f(&c).tactics.switches.len(), 1);
    let command = tac(tactics::TacticsCommand::Switch {
        request_id: f(&c).tactics.switches[0].id.clone(),
        incoming_id: Some(reserve.clone()),
        ruling: "replace".into(),
    });
    let c = run(&c, "gm", &command, 4);
    assert!(fighter(&c, &reserve).deployed);
    assert!(c.battle.as_ref().unwrap().active);
    let _ = a;
}
#[test]
fn tactics_prankster_keeps_printed_priority_and_conflicting_late_timing_needs_a_ruling() {
    let (mut c, a, t) = setup("Roar");
    ability(&mut c, &a, "Prankster");
    let m = catalog().find("moves", "Name", "Roar").unwrap();
    assert!(tactics::priority_guard(&c, fighter(&c, &a), &m).is_err());
    let c = run(
        &c,
        "gm",
        &tac(tactics::TacticsCommand::Priority {
            combatant_id: a.clone(),
            move_name: "Roar".into(),
            mode: "ability".into(),
        }),
        4,
    );
    let m = field::move_data(&c, fighter(&c, &a), &m);
    assert_eq!(m["Attributes"]["Reaction"], "1");
    assert!(m["Attributes"]["LateReaction"].is_null());
    let _ = t;
}
#[test]
fn tactics_pending_choices_and_targeting_state_survive_serialization_and_rollback() {
    let (mut c, a, t) = setup("Roar");
    let _ = add(&mut c, &t, false);
    let c = effect(&pass(&hit(&c, &a, &t, "Roar")));
    let encoded = serde_json::to_value(&c).unwrap();
    let copy: Campaign = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(
        f(&copy).tactics.switches[0].id,
        f(&c).tactics.switches[0].id
    );
    assert!(battle::transition(&copy, "gm", &Action::NextRound, vec![], &catalog()).is_err());
    assert_eq!(serde_json::to_value(&copy).unwrap(), encoded);
    let command = tac(tactics::TacticsCommand::Switch {
        request_id: f(&c).tactics.switches[0].id.clone(),
        incoming_id: None,
        ruling: "waive".into(),
    });
    let next = run(&copy, "gm", &command, 4);
    let restored = run(
        &next,
        "gm",
        &Action::RollbackBattle {
            event_sequence: next.battle_undo.last().unwrap().event_sequence,
            reason: "Restore pending switch".into(),
        },
        4,
    );
    assert_eq!(
        f(&restored).tactics.switches[0].id,
        f(&c).tactics.switches[0].id
    );
}
#[test]
fn tactics_multiple_clashes_apply_the_highest_counter_damage_once() {
    let (mut c, a, t) = setup("Swift");
    let other = add(&mut c, &t, true);
    learn(&mut c, &t, "Low Kick");
    learn(&mut c, &other, "Low Kick");
    let mut attack = action(&a, vec![t.clone(), other.clone()], "Swift");
    if let Action::AccuracyTargets {
        success_modifier, ..
    } = &mut attack
    {
        *success_modifier = -7;
    }
    let c = run(&c, "gm", &attack, 4);
    let defender = f(&c).queue[0].target_id.clone();
    let c = run(
        &c,
        "gm",
        &Action::React {
            combatant_id: defender,
            kind: "clash".into(),
            move_name: "Tackle".into(),
            reason: "First counter".into(),
        },
        4,
    );
    assert_eq!(fighter(&c, &a).hp, 20);
    let defender = f(&c).queue[0].target_id.clone();
    let c = run(
        &c,
        "gm",
        &Action::React {
            combatant_id: defender,
            kind: "clash".into(),
            move_name: "Low Kick".into(),
            reason: "Highest counter".into(),
        },
        4,
    );
    assert_eq!(fighter(&c, &a).hp, 18);
    assert!(f(&c).queue.is_empty());
}
#[test]
fn tactics_ally_switch_refunds_an_action_and_lets_the_incoming_reserve_react() {
    let (mut c, a, t) = setup("Tackle");
    learn(&mut c, &t, "Ally Switch");
    let reserve = add(&mut c, &t, false);
    let c = hit(&c, &a, &t, "Tackle");
    let c = run(
        &c,
        "gm",
        &Action::React {
            combatant_id: t.clone(),
            kind: "move".into(),
            move_name: "Ally Switch".into(),
            reason: "Switch before hit".into(),
        },
        4,
    );
    let c = effect(&c);
    assert!(f(&c).tactics.switches[0].immediate);
    let c = run(
        &c,
        "gm",
        &tac(tactics::TacticsCommand::Switch {
            request_id: f(&c).tactics.switches[0].id.clone(),
            incoming_id: Some(reserve.clone()),
            ruling: "replace".into(),
        }),
        4,
    );
    assert_eq!(fighter(&c, &reserve).actions, 0);
    assert_eq!(fighter(&c, &t).actions, 0);
    assert_eq!(f(&c).queue[0].target_id, reserve);
    assert!(f(&c).window);
    let a1 = Action::React {
        combatant_id: reserve,
        kind: "evade".into(),
        move_name: "".into(),
        reason: "Incoming ally responds".into(),
    };
    assert!(rules::dice_count(&c, &a1, &catalog()).is_ok());
}
#[test]
fn tactics_baton_pass_transfers_move_boosts_and_blocks_self_boosting_until_recipient_faints() {
    let (mut c, a, _) = setup("Baton Pass");
    let reserve = add(&mut c, &a, false);
    crate::combat::extra::change_stat(&mut c, &a, &a, "Strength", 2).unwrap();
    let c = hit(&c, &a, &a, "Baton Pass");
    let c = effect(&c);
    let c = run(
        &c,
        "gm",
        &tac(tactics::TacticsCommand::Switch {
            request_id: f(&c).tactics.switches[0].id.clone(),
            incoming_id: Some(reserve.clone()),
            ruling: "replace".into(),
        }),
        4,
    );
    assert_eq!(fighter(&c, &reserve).runtime.modifiers["Strength"], 2);
    assert!(tactics::baton_blocked(&c, &a, &a, "Strength", 1));
    let mut c = c;
    c.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == reserve)
        .unwrap()
        .hp = 0;
    assert!(!tactics::baton_blocked(&c, &a, &a, "Strength", 1));
}
#[test]
fn tactics_rage_powder_only_attracts_damage_moves_and_lures_expire_on_source_departure() {
    let (mut c, a, t) = setup("Rage Powder");
    let other = add(&mut c, &t, true);
    turn(&mut c, &t);
    learn(&mut c, &t, "Rage Powder");
    let c = effect(&hit(&c, &t, &t, "Rage Powder"));
    let mut c = c;
    turn(&mut c, &a);
    learn(&mut c, &a, "Growl");
    let m = catalog().find("moves", "Name", "Growl").unwrap();
    let ids = tactics::redirect_move(&c, &a, vec![other.clone()], &m, &catalog()).unwrap();
    assert_eq!(ids, vec![other.clone()]);
    let m = catalog().find("moves", "Name", "Tackle").unwrap();
    let ids = tactics::redirect_move(&c, &a, vec![other], &m, &catalog()).unwrap();
    assert_eq!(ids, vec![t.clone()]);
    let before = c.clone();
    c.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == t)
        .unwrap()
        .deployed = false;
    tactics::lifecycle(&before, &mut c, &catalog()).unwrap();
    assert!(f(&c).tactics.lures.is_empty());
}
#[test]
fn tactics_stall_remains_last_when_trick_room_changes_only_uncompleted_turns() {
    let (mut c, a, t) = setup("Trick Room");
    let other = add(&mut c, &t, true);
    ability(&mut c, &other, "Stall");
    let c = run(&c, "gm", &action(&a, vec![], "Trick Room"), 4);
    let c = effect(&c);
    assert_eq!(f(&c).order.last(), Some(&other));
    assert_eq!(f(&c).order[0], a);
}
#[test]
fn tactics_evasion_is_not_refreshed_between_successive_hits_but_clash_can_answer_the_next_hit() {
    let (c, a, t) = setup("Double Kick");
    let c = run(
        &c,
        "gm",
        &Action::SpecialMove {
            command: crate::combat::special::SpecialCommand::Successive {
                combatant_id: a,
                target_id: t.clone(),
                move_name: "Double Kick".into(),
                attempts: 2,
                reaction: false,
            },
            reason: "One turn sequence".into(),
        },
        4,
    );
    let evade = Action::React {
        combatant_id: t.clone(),
        kind: "evade".into(),
        move_name: "".into(),
        reason: "Attempt Evasion".into(),
    };
    let c = run(&c, "gm", &evade, 1);
    let c = effect(&damage(&c));
    assert!(f(&c).window);
    assert!(rules::dice_count(&c, &evade, &catalog()).is_err());
    let clash = Action::React {
        combatant_id: t,
        kind: "clash".into(),
        move_name: "Tackle".into(),
        reason: "Clash remains eligible".into(),
    };
    assert!(rules::dice_count(&c, &clash, &catalog()).is_ok());
}
#[test]
fn tactics_no_reserve_requires_a_gm_outcome_and_ending_preserves_the_recorded_engine() {
    let (c, a, t) = setup("Roar");
    let c = effect(&pass(&hit(&c, &a, &t, "Roar")));
    let req = f(&c).tactics.switches[0].id.clone();
    let command = tac(tactics::TacticsCommand::Switch {
        request_id: req,
        incoming_id: None,
        ruling: "withdraw".into(),
    });
    let owner = fighter(&c, &t).owner_id.clone();
    assert!(battle::transition(&c, &owner, &command, vec![], &catalog()).is_err());
    let c = run(&c, "gm", &command, 4);
    assert!(c.battle.as_ref().unwrap().active);
    let c = run(&c, "gm", &Action::EndBattle, 4);
    assert!(!c.battle.as_ref().unwrap().active);
    assert_eq!(c.engine, ENGINE);
    assert!(!c.finished_battles.is_empty());
}
#[test]
fn tactics_reflected_leech_seed_spends_original_will_and_preserves_effect_source() {
    let (mut c, a, t) = setup("Leech Seed");
    ability(&mut c, &t, "Magic Bounce");
    let c = hit(&c, &a, &t, "Leech Seed");
    let c = effect(&c);
    let e = f(&c)
        .special
        .effects
        .iter()
        .find(|e| e.name == "Leech Seed")
        .unwrap();
    assert_eq!(e.target, a);
    assert_eq!(e.source, t);
    assert_eq!(fighter(&c, &a).runtime.will, 5);
    assert_eq!(fighter(&c, &t).runtime.will, 6);
    assert!(f(&c).tactics.payer.is_none());
}
#[test]
fn tactics_unsupported_foe_field_effect_keeps_its_reflected_scope_in_the_gm_record() {
    let (mut c, a, t) = setup("Stealth Rock");
    ability(&mut c, &t, "Magic Bounce");
    let c = run(&c, "gm", &action(&a, vec![], "Stealth Rock"), 4);
    let c = run(
        &c,
        "gm",
        &Action::AdjudicateField {
            sequence: f(&c).queue[0].sequence,
            rounds: None,
            reason: "Reflect the recorded hazard; damage remains adjudicated".into(),
        },
        4,
    );
    let e = f(&c).field_effects.last().unwrap();
    assert_eq!(e.scope, "side:Players");
    assert_eq!(e.actor_id, t);
}
#[test]
fn tactics_magic_coat_requires_a_recorded_duration_and_consumes_only_the_selected_scope() {
    let (mut c, a, t) = setup("Magic Coat");
    turn(&mut c, &t);
    learn(&mut c, &t, "Magic Coat");
    let c = hit(&c, &t, &t, "Magic Coat");
    let a1 = Action::ResolveEffect {
        sequence: f(&c).queue[0].sequence,
        reason: "No assumed duration".into(),
    };
    assert!(battle::transition(&c, "gm", &a1, vec![], &catalog()).is_err());
    let c = run(
        &c,
        "gm",
        &tac(tactics::TacticsCommand::CoatDuration {
            combatant_id: t.clone(),
            duration: "next_effect".into(),
        }),
        4,
    );
    let mut c = effect(&c);
    turn(&mut c, &a);
    learn(&mut c, &a, "Growl");
    let c = hit(&c, &a, &t, "Growl");
    let c = effect(&pass(&c));
    assert_eq!(fighter(&c, &a).runtime.modifiers["Strength"], -1);
    assert!(f(&c).tactics.lures.iter().all(|l| l.name != "Magic Coat"));
}
#[test]
fn tactics_older_engine_keeps_its_reaction_path_and_rejects_new_commands() {
    let (mut c, a, t) = setup("Tackle");
    c.engine = SPECIAL_ENGINE.into();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .tactics
        .enabled = false;
    let command = tac(tactics::TacticsCommand::Redirect {
        combatant_id: a.clone(),
        recipient_id: t.clone(),
    });
    assert!(rules::dice_count(&c, &command, &catalog()).is_err());
    let c = hit(&c, &a, &t, "Tackle");
    assert!(f(&c).queue[0].targets.is_empty());
    assert!(f(&c).special.enabled);
    assert!(!f(&c).tactics.enabled);
}
#[tokio::test]
async fn tactics_room_owner_replacement_is_host_validated_idempotent_stale_checked_and_reconnectable(
) {
    let hd = tempfile::tempdir().unwrap();
    let gd = tempfile::tempdir().unwrap();
    let pd = tempfile::tempdir().unwrap();
    let mut hs = Store::open(hd.path()).unwrap();
    let gs = Arc::new(Mutex::new(Store::open(gd.path()).unwrap()));
    let ps = Arc::new(Mutex::new(Store::open(pd.path()).unwrap()));
    let gm_id = gs.lock().unwrap().player_id.clone();
    let player_id = ps.lock().unwrap().player_id.clone();
    let (mut c, a, t) = setup("Roar");
    let reserve = add(&mut c, &t, false);
    let c1 = effect(&pass(&hit(&c, &a, &t, "Roar")));
    c = c1;
    let previous = fighter(&c, &t).owner_id.clone();
    for p in &mut c.battle.as_mut().unwrap().combatants {
        if p.owner_id == previous {
            p.owner_id = player_id.clone();
        }
    }
    for s in c.sheets.values_mut() {
        if s.owner_id == previous {
            s.owner_id = player_id.clone();
        }
    }
    c.host_id = hs.player_id.clone();
    c.gm_id = gm_id.clone();
    hs.put_campaign(&c).unwrap();
    let shared = Arc::new(Mutex::new(hs));
    let cat = Arc::new(catalog());
    let notify: room::Notify = Arc::new(|_, _| {});
    let mut host = room::start(shared.clone(), cat.clone(), c.id.clone(), 0, notify.clone())
        .await
        .unwrap();
    let join = JoinRequest {
        address: host.info.address.replace("<host-ip>", "127.0.0.1"),
        token: host.info.token.clone(),
    };
    let (gm, _) = room::join(gs, cat.clone(), join.clone(), notify.clone())
        .await
        .unwrap();
    let (owner, _) = room::join(ps.clone(), cat.clone(), join.clone(), notify.clone())
        .await
        .unwrap();
    let command = Command {
        id: id(),
        expected_revision: c.revision,
        action: tac(tactics::TacticsCommand::Switch {
            request_id: f(&c).tactics.switches[0].id.clone(),
            incoming_id: Some(reserve.clone()),
            ruling: "replace".into(),
        }),
    };
    assert!(host.host.apply(&c.host_id, &command).is_err());
    let (tx, rx) = tokio::sync::oneshot::channel();
    owner.sender.send((command.clone(), tx)).await.unwrap();
    let next = rx.await.unwrap().unwrap();
    assert!(fighter(&next, &reserve).deployed);
    assert!(!fighter(&next, &t).deployed);
    assert_eq!(
        host.host.apply(&player_id, &command).unwrap().revision,
        next.revision
    );
    let mut stale = command.clone();
    stale.id = id();
    assert!(host.host.apply(&player_id, &stale).is_err());
    owner.task.abort();
    let (owner, snapshot) = room::join(ps, cat, join, notify).await.unwrap();
    assert!(fighter(&snapshot, &reserve).deployed);
    assert!(f(&snapshot).tactics.switches.is_empty());
    owner.task.abort();
    gm.task.abort();
    let _ = host.stop.take().unwrap().send(());
}
#[test]
fn tactics_bystander_follow_me_intercepts_without_spending_the_original_targets_reaction() {
    let (mut c, a, t) = setup("Tackle");
    let guard = add(&mut c, &t, true);
    learn(&mut c, &guard, "Follow Me");
    let c = hit(&c, &a, &t, "Tackle");
    let c = run(
        &c,
        "gm",
        &tac(tactics::TacticsCommand::Reaction {
            combatant_id: guard.clone(),
            move_name: "Follow Me".into(),
            target_ids: vec![guard.clone()],
        }),
        4,
    );
    let c = effect(&c);
    assert_eq!(f(&c).queue[0].target_id, guard);
    assert_eq!(fighter(&c, &t).actions, 0);
    assert!(!f(&c).reacted.contains(&t));
    assert!(f(&c).window);
}
#[test]
fn tactics_rage_powder_respects_the_manuals_grass_and_overcoat_immunity() {
    for immunity in ["Grass", "Overcoat"] {
        let (mut c, a, t) = setup("Rage Powder");
        let other = add(&mut c, &t, true);
        turn(&mut c, &t);
        learn(&mut c, &t, "Rage Powder");
        let mut c = effect(&hit(&c, &t, &t, "Rage Powder"));
        turn(&mut c, &a);
        if immunity == "Grass" {
            field::change_type(&mut c, &a, "replace", &["Grass".into()]).unwrap();
        } else {
            ability(&mut c, &a, "Overcoat");
        }
        let m = catalog().find("moves", "Name", "Tackle").unwrap();
        assert_eq!(
            tactics::redirect_move(&c, &a, vec![other.clone()], &m, &catalog()).unwrap(),
            vec![other]
        );
    }
}
#[test]
fn tactics_legacy_missing_trainer_links_queue_a_gm_ruling_instead_of_cancelling_damage() {
    let (mut c, a, t) = setup("Tackle");
    ability(&mut c, &t, "Emergency Exit");
    let sheet = fighter(&c, &t).sheet_id.clone();
    c.sheets.get_mut(&sheet).unwrap().trainer_id = None;
    let before = c.clone();
    c.battle.as_mut().unwrap().combatants[1].hp = 10;
    tactics::lifecycle(&before, &mut c, &catalog()).unwrap();
    assert_eq!(fighter(&c, &t).hp, 10);
    assert_eq!(f(&c).tactics.switches.len(), 1);
    let request = f(&c).tactics.switches[0].id.clone();
    let c = run(
        &c,
        "gm",
        &tac(tactics::TacticsCommand::Switch {
            request_id: request,
            incoming_id: None,
            ruling: "waive".into(),
        }),
        4,
    );
    assert!(f(&c).tactics.switches.is_empty());
    let _ = a;
}
#[test]
fn tactics_field_reaction_keeps_its_side_scope_and_resumes_main_damage_after_the_barrier() {
    let (mut c, a, t) = setup("Tackle");
    ability(&mut c, &t, "Prankster");
    learn(&mut c, &t, "Reflect");
    let c = hit(&c, &a, &t, "Tackle");
    let before = f(&c).turn;
    let c = run(
        &c,
        "gm",
        &tac(tactics::TacticsCommand::Reaction {
            combatant_id: t.clone(),
            move_name: "Reflect".into(),
            target_ids: vec![],
        }),
        4,
    );
    assert_eq!(
        f(&c).queue[0].field_scope.as_deref(),
        Some("side:Opponents")
    );
    assert!(!f(&c).window);
    let c = effect(&c);
    assert!(f(&c)
        .battlefield
        .rules
        .iter()
        .any(|e| e.kind == "Reflect" && e.scope == "side:Opponents"));
    assert_eq!(f(&c).queue[0].move_name, "Tackle");
    assert_eq!(f(&c).turn, before);
    assert!(!f(&c).window);
}
