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
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .interaction_rules = true;
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
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .decoys
        .enabled = true;
    crate::combat::decoys::cast(&mut c, &a, "Substitute", &catalog()).unwrap();
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
    assert_eq!(f(&snapshot).decoys.active[&a].hp, 2);
    assert_eq!(f(&snapshot).decoys.history, f(&next).decoys.history);
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

// Decoys deliberately use the authoritative action path rather than a separate simulator.
fn decoy_setup(name: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = setup(name);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .decoys
        .enabled = true;
    (c, a, t)
}
#[test]
fn decoys_creation_freezes_defenses_and_no_damage_overflows() {
    let (c, a, t) = decoy_setup("Substitute");
    let hp = fighter(&c, &a).hp;
    let c = hit(&c, &a, &a, "Substitute");
    let mut c = effect(&c);
    assert_eq!(fighter(&c, &a).hp, hp - 2);
    assert_eq!(f(&c).decoys.active[&a].hp, 2);
    assert!(f(&c).tactics.switches.is_empty());
    let defenses = f(&c).decoys.active[&a].defenses;
    crate::combat::extra::change_stat(&mut c, &a, &a, "Defense", 3).unwrap();
    assert_eq!(f(&c).decoys.active[&a].defenses, defenses);
    turn(&mut c, &t);
    learn(&mut c, &t, "Tackle");
    let c = hit(&c, &t, &a, "Tackle");
    let c = pass(&c);
    let q = &f(&c).queue[0];
    assert_eq!(
        crate::combat::decoys::pool_defense(
            &c,
            &q.actor_id,
            &q.target_id,
            &catalog().find("moves", "Name", "Tackle").unwrap(),
            true
        ),
        Some(defenses[0])
    );
    let before = fighter(&c, &a).hp;
    let c = run(&c, "gm", &super::targeting_tests::damage_command(&c), 6);
    assert_eq!(fighter(&c, &a).hp, before);
    assert!(!f(&c).decoys.active.contains_key(&a));
    assert!(f(&c)
        .decoys
        .history
        .iter()
        .any(|s| s.contains("no overflow")));
}
#[test]
fn decoys_broken_target_still_blocks_added_effects() {
    let (mut c, a, t) = decoy_setup("Thunderbolt");
    crate::combat::decoys::cast(&mut c, &t, "Substitute", &catalog()).unwrap();
    let c = hit(&c, &a, &t, "Thunderbolt");
    let c = pass(&c);
    let before = fighter(&c, &t).hp;
    let c = run(&c, "gm", &super::targeting_tests::damage_command(&c), 6);
    assert_eq!(fighter(&c, &t).hp, before);
    assert!(!f(&c).decoys.active.contains_key(&t));
    assert!(!f(&c).decoys.intercepted.is_empty());
    let c = effect(&c);
    assert!(fighter(&c, &t).runtime.statuses.is_empty());
    assert!(f(&c).decoys.intercepted.is_empty());
}
#[test]
fn decoys_support_is_blocked_but_shields_protect_the_decoy() {
    let (mut c, a, t) = decoy_setup("Growl");
    crate::combat::decoys::cast(&mut c, &t, "Substitute", &catalog()).unwrap();
    // Growl is Sound and must bypass, so use a non-sound Support move instead.
    learn(&mut c, &a, "Leer");
    let c = hit(&c, &a, &t, "Leer");
    let c = pass(&c);
    let mut c = effect(&c);
    assert!(fighter(&c, &t).runtime.modifiers.is_empty());
    turn(&mut c, &t);
    learn(&mut c, &t, "Protect");
    let c = hit(&c, &t, &t, "Protect");
    let c = effect(&c);
    assert!(f(&c).special.shields.contains_key(&t));
    assert_eq!(f(&c).decoys.active[&t].hp, 2);
}
#[test]
fn decoys_sound_and_infiltrator_bypass_without_damaging_decoy_or_mutating_sheets() {
    for (move_name, selected) in [
        ("Hyper Voice", ""),
        ("Tackle", "Infiltrator"),
        ("Chip Away", ""),
    ] {
        let (mut c, a, t) = decoy_setup(move_name);
        ability(&mut c, &a, selected);
        crate::combat::decoys::cast(&mut c, &t, "Substitute", &catalog()).unwrap();
        let sheets = serde_json::to_value(&c.sheets).unwrap();
        let c = hit(&c, &a, &t, move_name);
        let c = pass(&c);
        let before = fighter(&c, &t).hp;
        let c = run(&c, "gm", &super::targeting_tests::damage_command(&c), 4);
        assert!(fighter(&c, &t).hp < before, "{move_name}");
        assert_eq!(f(&c).decoys.active[&t].hp, 2);
        assert_eq!(serde_json::to_value(&c.sheets).unwrap(), sheets);
    }
}
#[test]
fn decoys_fixed_damage_hits_decoy_without_overflow() {
    let (mut c, a, t) = decoy_setup("Dragon Rage");
    crate::combat::decoys::cast(&mut c, &t, "Substitute", &catalog()).unwrap();
    let c = hit(&c, &a, &t, "Dragon Rage");
    let c = pass(&c);
    let before = fighter(&c, &t).hp;
    let c = damage(&c);
    assert_eq!(fighter(&c, &t).hp, before);
    assert!(!f(&c).decoys.active.contains_key(&t));
}
#[test]
fn decoys_snatch_preserves_hp_and_uses_new_user_defenses() {
    let (mut c, a, t) = decoy_setup("Snatch");
    crate::combat::decoys::cast(&mut c, &t, "Substitute", &catalog()).unwrap();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .decoys
        .active
        .get_mut(&t)
        .unwrap()
        .hp = 1;
    crate::combat::extra::change_stat(&mut c, &a, &a, "Defense", 2).unwrap();
    crate::combat::extra::change_stat(&mut c, &t, &t, "Defense", 3).unwrap();
    let c = hit(&c, &a, &t, "Snatch");
    let c = pass(&c);
    let c = effect(&c);
    assert!(!f(&c).decoys.active.contains_key(&t));
    assert_eq!(f(&c).decoys.active[&a].hp, 1);
    let expected = field::snapshot(&c, &catalog()).unwrap().defenses[&a][0];
    assert_eq!(f(&c).decoys.active[&a].defenses[0], expected);
}
#[test]
fn decoys_shed_tail_transfers_to_registered_reserve() {
    let (mut c, a, _) = decoy_setup("Shed Tail");
    let reserve = add(&mut c, &a, false);
    let hp = fighter(&c, &a).hp;
    let c = hit(&c, &a, &a, "Shed Tail");
    let c = effect(&c);
    let snapshot = f(&c).decoys.active[&a].clone();
    assert_eq!(fighter(&c, &a).hp, hp - 2);
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
    assert!(!f(&c).decoys.active.contains_key(&a));
    assert_eq!(f(&c).decoys.active[&reserve].defenses, snapshot.defenses);
    assert_eq!(f(&c).decoys.active[&reserve].hp, 2);
    assert_eq!(fighter(&c, &reserve).hp, fighter(&c, &reserve).max_hp);
}
#[test]
fn decoys_serialize_rollback_and_departure_cleanup_preserve_state() {
    let (c, a, _) = decoy_setup("Substitute");
    let c = hit(&c, &a, &a, "Substitute");
    let mut c = effect(&c);
    let json = serde_json::to_string(&c).unwrap();
    let restored: Campaign = serde_json::from_str(&json).unwrap();
    assert_eq!(f(&restored).decoys.active[&a].hp, 2);
    let c0 = run(
        &c,
        "gm",
        &Action::RollbackBattle {
            event_sequence: c.battle_undo.last().unwrap().event_sequence,
            reason: "Restore pre-cast checkpoint".into(),
        },
        4,
    );
    assert!(f(&c0).decoys.active.is_empty());
    c.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == a)
        .unwrap()
        .deployed = false;
    crate::combat::decoys::lifecycle(&mut c);
    assert!(f(&c).decoys.active.is_empty());
}
#[test]
fn decoys_dynamax_and_z_moves_are_explicitly_excluded_and_old_engines_unchanged() {
    let (mut c, a, _) = decoy_setup("Max Guard");
    for name in ["Max Guard", "Breakneck Blitz"] {
        let data = catalog().find("moves", "Name", name).unwrap();
        let err = crate::combat::special::guard(&c, &a, &data, &catalog()).unwrap_err();
        assert_eq!(err.code, "out_of_scope");
    }
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .decoys
        .enabled = false;
    c.engine = TACTICS_ENGINE.into();
    assert!(crate::combat::decoys::cast(&mut c, &a, "Substitute", &catalog()).is_err());
    assert!(core_engine(&c.engine));
}

#[test]
fn decoys_weather_and_existing_poison_hit_owner_not_decoy() {
    let (mut c, a, _) = decoy_setup("Substitute");
    crate::combat::decoys::cast(&mut c, &a, "Substitute", &catalog()).unwrap();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .natural = Some(field::Weather::Hail);
    let before = fighter(&c, &a).hp;
    field::end(&mut c, &catalog()).unwrap();
    assert_eq!(fighter(&c, &a).hp, before - 1);
    assert_eq!(f(&c).decoys.active[&a].hp, 2);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .order
        .clear();
    c.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == a)
        .unwrap()
        .runtime
        .statuses
        .push(crate::combat::Status {
            kind: "Poison".into(),
            detail: "Existing ailment".into(),
            rounds: None,
            progress: 0,
        });
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .end_round = 0;
    let before = fighter(&c, &a).hp;
    let c = run(&c, "gm", &Action::NextRound, 4);
    assert!(fighter(&c, &a).hp < before);
    assert_eq!(f(&c).decoys.active[&a].hp, 2);
}
#[test]
fn decoys_tidy_up_removes_both_sides_and_cast_can_faint_without_reviving() {
    let (mut c, a, t) = decoy_setup("Tidy Up");
    crate::combat::decoys::cast(&mut c, &a, "Substitute", &catalog()).unwrap();
    crate::combat::decoys::cast(&mut c, &t, "Substitute", &catalog()).unwrap();
    let c = run(&c, "gm", &action(&a, vec![], "Tidy Up"), 4);
    let mut c = effect(&c);
    assert!(f(&c).decoys.active.is_empty());
    assert_eq!(fighter(&c, &a).runtime.modifiers["Strength"], 1);
    c.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == a)
        .unwrap()
        .hp = 1;
    crate::combat::decoys::cast(&mut c, &a, "Substitute", &catalog()).unwrap();
    crate::combat::decoys::lifecycle(&mut c);
    assert_eq!(fighter(&c, &a).hp, 0);
    assert!(f(&c).decoys.active.is_empty());
}
#[test]
fn decoys_shared_attack_isolates_damage_and_does_not_consume_owner_sturdy() {
    let (mut c, a, t) = decoy_setup("Swift");
    let other = add(&mut c, &t, true);
    ability(&mut c, &t, "Sturdy");
    crate::combat::decoys::cast(&mut c, &t, "Substitute", &catalog()).unwrap();
    let before = fighter(&c, &t).hp;
    let other_hp = fighter(&c, &other).hp;
    let c = run(
        &c,
        "gm",
        &action(&a, vec![t.clone(), other.clone()], "Swift"),
        4,
    );
    let c = pass(&c);
    let mut c = pass(&c);
    while !f(&c).queue.is_empty() {
        c = run(&c, "gm", &super::targeting_tests::damage_command(&c), 6);
    }
    assert_eq!(fighter(&c, &t).hp, before);
    assert!(fighter(&c, &other).hp < other_hp);
    assert!(!fighter(&c, &t).runtime.ability.flags.contains("Sturdy"));
}
#[test]
fn decoys_successive_hits_break_then_damage_owner_on_the_next_hit() {
    let (mut c, a, t) = decoy_setup("Double Kick");
    crate::combat::decoys::cast(&mut c, &t, "Substitute", &catalog()).unwrap();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .decoys
        .active
        .get_mut(&t)
        .unwrap()
        .hp = 1;
    let before = fighter(&c, &t).hp;
    let c = run(
        &c,
        "gm",
        &Action::SpecialMove {
            command: crate::combat::special::SpecialCommand::Successive {
                combatant_id: a.clone(),
                target_id: t.clone(),
                move_name: "Double Kick".into(),
                attempts: 2,
                reaction: false,
            },
            reason: "Two hits versus decoy".into(),
        },
        6,
    );
    let c = pass(&c);
    let mut c = run(&c, "gm", &super::targeting_tests::damage_command(&c), 6);
    if !f(&c).queue.is_empty() && f(&c).queue[0].damage_done {
        c = effect(&c);
    }
    assert_eq!(fighter(&c, &t).hp, before);
    let c = pass(&c);
    let c = run(&c, "gm", &super::targeting_tests::damage_command(&c), 6);
    assert!(fighter(&c, &t).hp < before);
}

#[test]
fn decoys_do_not_cancel_user_switcher_or_burn_up_type_cost() {
    for name in ["U-Turn", "Burn Up"] {
        let (mut c, a, t) = decoy_setup(name);
        let _reserve = add(&mut c, &a, false);
        if name == "Burn Up" {
            field::change_type(&mut c, &a, "replace", &["Fire".into()]).unwrap();
        }
        crate::combat::decoys::cast(&mut c, &t, "Substitute", &catalog()).unwrap();
        let c = pass(&hit(&c, &a, &t, name));
        let c = run(&c, "gm", &super::targeting_tests::damage_command(&c), 6);
        let c = effect(&c);
        if name == "U-Turn" {
            assert_eq!(f(&c).tactics.switches[0].outgoing, a);
        } else {
            assert!(!field::types(&c, fighter(&c, &a), &catalog())
                .unwrap()
                .contains(&"Fire".into()));
        }
    }
}
#[test]
fn decoys_sonic_boom_damages_decoy_and_unseen_fist_only_bypasses_non_ranged_physical() {
    let (mut c, a, t) = decoy_setup("Sonic Boom");
    crate::combat::decoys::cast(&mut c, &t, "Substitute", &catalog()).unwrap();
    let before = fighter(&c, &t).hp;
    let c = pass(&hit(&c, &a, &t, "Sonic Boom"));
    let c = damage(&c);
    assert_eq!(fighter(&c, &t).hp, before);
    assert_eq!(f(&c).decoys.active[&t].hp, 1);
    let (mut c, a, t) = decoy_setup("Tackle");
    ability(&mut c, &a, "Unseen Fist");
    crate::combat::decoys::cast(&mut c, &t, "Substitute", &catalog()).unwrap();
    for (name, expected) in [("Tackle", false), ("Swift", true)] {
        let m = catalog().find("moves", "Name", name).unwrap();
        assert_eq!(
            crate::combat::decoys::protected_subject(&c, &a, &t, &m),
            expected
        );
    }
}

#[test]
fn decoys_clash_hits_both_decoys_without_harming_or_triggering_the_owners() {
    let (mut c, a, t) = decoy_setup("Tackle");
    learn(&mut c, &t, "Low Kick");
    for id in [&a, &t] {
        crate::combat::decoys::cast(&mut c, id, "Substitute", &catalog()).unwrap();
        ability(&mut c, id, "Sturdy");
    }
    let ahp = fighter(&c, &a).hp;
    let thp = fighter(&c, &t).hp;
    let mut incoming = action(&a, vec![t.clone()], "Tackle");
    if let Action::AccuracyTargets {
        success_modifier, ..
    } = &mut incoming
    {
        *success_modifier = -7;
    }
    let c = run(&c, "gm", &incoming, 4);
    let c = run(
        &c,
        "gm",
        &Action::React {
            combatant_id: t.clone(),
            kind: "clash".into(),
            move_name: "Low Kick".into(),
            reason: "Clash with both participants behind decoys".into(),
        },
        6,
    );
    assert_eq!(fighter(&c, &a).hp, ahp);
    assert_eq!(fighter(&c, &t).hp, thp);
    assert!(
        f(&c)
            .decoys
            .history
            .iter()
            .filter(|e| e.contains("hits"))
            .count()
            >= 2
    );
    assert!(!fighter(&c, &a).runtime.ability.flags.contains("Sturdy"));
    assert!(!fighter(&c, &t).runtime.ability.flags.contains("Sturdy"));
}

#[test]
fn decoys_unreviewed_user_effect_is_not_silently_discarded() {
    let (mut c, a, t) = decoy_setup("Superpower");
    crate::combat::decoys::cast(&mut c, &t, "Substitute", &catalog()).unwrap();
    let c = pass(&hit(&c, &a, &t, "Superpower"));
    let c = run(&c, "gm", &super::targeting_tests::damage_command(&c), 6);
    let command = Action::ResolveEffect {
        sequence: f(&c).queue[0].sequence,
        reason: "Resolve user effect".into(),
    };
    let n = rules::dice_count(&c, &command, &catalog()).unwrap();
    let err = battle::transition(&c, "gm", &command, vec![6; n as usize], &catalog()).unwrap_err();
    assert_eq!(err.code, "manual_rule");
    assert!(err.message.contains("unsupported user effect"));
}

#[test]
fn stat_channels_baton_host_switch_copies_both_channels_to_reserve_and_cleans_outgoing() {
    let (mut c, a, _) = setup("Baton Pass");
    let reserve = add(&mut c, &a, false);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .stat_channels
        .enabled = true;
    crate::combat::extra::change_stat(&mut c, &a, &a, "Strength", 2).unwrap();
    crate::combat::extra::boost(&mut c, &a, "Strength", 1).unwrap();
    let c = effect(&hit(&c, &a, &a, "Baton Pass"));
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
    assert_eq!(
        crate::rules::effective_sheet(&c, fighter(&c, &reserve))
            .unwrap()
            .stats["Strength"],
        7
    );
    assert!(!fighter(&c, &a).deployed);
    assert!(f(&c)
        .stat_channels
        .grants
        .iter()
        .all(|g| g.target == reserve));
    assert!(tactics::baton_blocked(&c, &a, &a, "Strength", 1));
}
#[test]
fn stat_channels_snatch_host_resolution_steals_ability_and_move_channels() {
    let (mut c, a, t) = setup("Snatch");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .stat_channels
        .enabled = true;
    crate::combat::extra::change_stat(&mut c, &t, &t, "Strength", 2).unwrap();
    crate::combat::extra::boost(&mut c, &t, "Strength", 1).unwrap();
    let c = effect(&pass(&hit(&c, &a, &t, "Snatch")));
    assert_eq!(
        crate::rules::effective_sheet(&c, fighter(&c, &a))
            .unwrap()
            .stats["Strength"],
        7
    );
    assert_eq!(
        crate::rules::effective_sheet(&c, fighter(&c, &t))
            .unwrap()
            .stats["Strength"],
        4
    );
}

#[test]
fn stat_channels_snatch_keeps_resolved_values_and_stronger_recipient_move_modifiers() {
    for ability_name in ["Simple", "Contrary", ""] {
        let (mut c, a, t) = setup("Snatch");
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .stat_channels
            .enabled = true;
        ability(&mut c, &a, ability_name);
        crate::combat::extra::change_stat(&mut c, &t, &t, "Strength", 2).unwrap();
        let c = effect(&pass(&hit(&c, &a, &t, "Snatch")));
        assert_eq!(
            crate::rules::effective_sheet(&c, fighter(&c, &a))
                .unwrap()
                .stats["Strength"],
            6,
            "{ability_name} must not transform the transferred snapshot again"
        );
        assert_eq!(
            crate::rules::effective_sheet(&c, fighter(&c, &t))
                .unwrap()
                .stats["Strength"],
            4
        );
    }
    let (mut c, a, t) = setup("Snatch");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .stat_channels
        .enabled = true;
    c.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == a)
        .unwrap()
        .runtime
        .modifiers
        .insert("Strength".into(), 3);
    crate::combat::extra::change_stat(&mut c, &t, &t, "Strength", 1).unwrap();
    let c = effect(&pass(&hit(&c, &a, &t, "Snatch")));
    assert_eq!(
        crate::rules::effective_sheet(&c, fighter(&c, &a))
            .unwrap()
            .stats["Strength"],
        7
    );
}
#[test]
fn interactions_unseen_fist_bypasses_shields_screens_and_cover_only_for_contact() {
    use crate::combat::{interactions, special};
    for (name, expected) in [
        ("Tackle", true),
        ("Thunderbolt", false),
        ("Rock Slide", false),
    ] {
        let (mut c, a, t) = setup(name);
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .reviewed_rules = true;
        ability(&mut c, &a, "Unseen Fist");
        let cover = add(&mut c, &t, true);
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .covers
            .insert(t.clone(), cover.clone());
        let m = catalog().find("moves", "Name", name).unwrap();
        assert_eq!(interactions::cover_bypass(&c, &a, &m), expected);
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .special
            .shields
            .insert(
                t.clone(),
                special::Shield {
                    name: "Spiky Shield".into(),
                    source: t.clone(),
                    round: 1,
                    armed_turn: 0,
                    available: true,
                },
            );
        let screen_scope = format!("side:{}", fighter(&c, &t).side);
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .battlefield
            .rules
            .push(field::FieldRule {
                id: "screen".into(),
                kind: if m["Category"] == "Physical" {
                    "Reflect"
                } else {
                    "Light Screen"
                }
                .into(),
                source: t.clone(),
                scope: screen_scope,
                rounds: Some(4),
                parameter: 2,
                started_round: 1,
            });
        let pending = hit(&c, &a, &t, name);
        if expected {
            let cover_action = Action::CoverAlly {
                combatant_id: cover.clone(),
                ally_id: t.clone(),
                reason: "Attempt new cover".into(),
            };
            let error = rules::dice_count(&pending, &cover_action, &catalog()).unwrap_err();
            assert!(
                error.message.contains("bypasses Cover"),
                "{}",
                error.message
            );
        }
        let mut q = f(&pending).queue[0].clone();
        assert_eq!(q.target_id, if expected { t.clone() } else { cover });
        q.actor_id = a.clone();
        q.target_id = t.clone();
        q.move_name = name.into();
        assert_eq!(
            special::shield_reduction(&c, &q, &m),
            if expected { 0 } else { 3 }
        );
        assert_eq!(
            field::reduction(&c, fighter(&c, &a), fighter(&c, &t), &m),
            if expected { 0 } else { 2 }
        );
        c.engine = STANDALONE_ENGINE.into();
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .interaction_rules = false;
        assert!(!interactions::unseen(&c, fighter(&c, &a), &m));
        assert_eq!(special::shield_reduction(&c, &q, &m), 3);
    }
}
#[test]
fn interactions_guard_dog_blocks_forced_switch_and_extreme_bypass_is_scoped() {
    for ab in ["Guard Dog", "Suction Cups"] {
        for attacker in ["", "Teravolt", "Turboblaze"] {
            let (mut c, a, t) = setup("Roar");
            c.battle
                .as_mut()
                .unwrap()
                .flow
                .as_mut()
                .unwrap()
                .reviewed_rules = true;
            ability(&mut c, &t, ab);
            ability(&mut c, &a, attacker);
            add(&mut c, &t, false);
            let c = effect(&pass(&hit(&c, &a, &t, "Roar")));
            assert_eq!(
                f(&c).tactics.switches.is_empty(),
                attacker.is_empty(),
                "{ab} / {attacker}"
            );
        }
    }
}
#[test]
fn interactions_mycelium_timing_choice_is_authorized_persisted_and_reversible() {
    let (mut c, a, _) = setup("Protect");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .reviewed_rules = true;
    ability(&mut c, &a, "Mycelium Might");
    let raw = catalog().find("moves", "Name", "Protect").unwrap();
    assert!(tactics::priority_guard(&c, fighter(&c, &a), &raw).is_err());
    for mode in ["ability", "printed"] {
        let command = tac(tactics::TacticsCommand::Priority {
            combatant_id: a.clone(),
            move_name: "Protect".into(),
            mode: mode.into(),
        });
        assert!(battle::transition(&c, "player", &command, vec![], &catalog()).is_err());
        let next = run(&c, "gm", &command, 4);
        let next: Campaign = serde_json::from_value(serde_json::to_value(&next).unwrap()).unwrap();
        let data = field::move_data(&next, fighter(&next, &a), &raw);
        assert!(tactics::priority_guard(&next, fighter(&next, &a), &data).is_ok());
        assert_eq!(data["Attributes"]["NeverMiss"], true);
        assert_eq!(data["Attributes"]["Reaction"].is_null(), mode == "ability");
        assert_eq!(
            data["Attributes"]["LateReaction"].is_null(),
            mode == "printed"
        );
        let restored = run(
            &next,
            "gm",
            &Action::RollbackBattle {
                event_sequence: next.battle_undo.last().unwrap().event_sequence,
                reason: "Undo timing ruling".into(),
            },
            4,
        );
        assert!(f(&restored).tactics.priority_choices.is_empty());
    }
}
#[test]
fn interactions_reviewed_stat_moves_use_protection_reflection_reversal_and_retaliation() {
    for (ab, stat, expected, retaliate) in [
        ("Big Pecks", "Defense", 0, 0),
        ("Mirror Armor", "Defense", 0, 0),
        ("Contrary", "Defense", -2, 0),
        ("Simple", "Defense", -2, 0),
        ("Competitive", "Special", 2, 2),
        ("Defiant", "Strength", 2, 2),
    ] {
        let (mut c, a, t) = setup("Screech");
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .reviewed_rules = true;
        ability(&mut c, &t, ab);
        let c = effect(&pass(&hit(&c, &a, &t, "Screech")));
        if retaliate > 0 {
            assert_eq!(
                fighter(&c, &t).runtime.ability.counters[&format!("boost:{stat}")],
                expected
            );
        } else {
            assert_eq!(
                fighter(&c, &t)
                    .runtime
                    .modifiers
                    .get("Defense")
                    .copied()
                    .unwrap_or(0),
                expected
            );
        }
        if ab == "Mirror Armor" {
            assert_eq!(fighter(&c, &a).runtime.modifiers["Defense"], -2);
        }
    }
}

#[test]
fn interactions_chance_dice_status_immunity_dust_and_sheer_force_have_distinct_effects() {
    use crate::combat::interactions;
    for (source, target, dice_count, burned) in [
        ("Serene Grace", "", 3, true),
        ("Serene Grace", "Wonder Skin", 1, true),
        ("", "Wonder Skin", 0, false),
        ("Serene Grace", "Shield Dust", 3, false),
        ("Teravolt", "Shield Dust", 1, true),
        ("Teravolt", "Wonder Skin", 1, true),
        ("", "Water Veil", 1, false),
    ] {
        let (mut c, a, t) = setup("Ember");
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .reviewed_rules = true;
        ability(&mut c, &a, source);
        ability(&mut c, &t, target);
        let c = damage(&pass(&hit(&c, &a, &t, "Ember")));
        let q = f(&c).queue[0].clone();
        assert_eq!(interactions::effect_dice(&c, &q).unwrap(), Some(dice_count));
        let cmd = Action::ResolveEffect {
            sequence: q.sequence,
            reason: "Resolve chance effect".into(),
        };
        assert_eq!(rules::dice_count(&c, &cmd, &catalog()).unwrap(), dice_count);
        let n = run(&c, "gm", &cmd, 6);
        assert_eq!(
            fighter(&n, &t)
                .runtime
                .statuses
                .iter()
                .any(|s| s.kind == "Burn"),
            burned,
            "{source}/{target}"
        );
        let restored = run(
            &n,
            "gm",
            &Action::RollbackBattle {
                event_sequence: n.battle_undo.last().unwrap().event_sequence,
                reason: "Undo chance".into(),
            },
            4,
        );
        assert!(fighter(&restored, &t).runtime.statuses.is_empty());
        assert_eq!(f(&restored).queue[0].sequence, q.sequence);
    }
}
#[test]
fn interactions_secondary_rock_head_reserves_only_chance_dice_and_historical_pools_stay_saved() {
    use crate::combat::special;
    let (mut c, a, t) = setup("Flare Blitz");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .reviewed_rules = true;
    ability(&mut c, &a, "Stench");
    c.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == a)
        .unwrap()
        .runtime
        .ability
        .flags
        .insert("as-one:Rock Head".into());
    let mut q = f(&hit(&c, &a, &t, "Flare Blitz")).queue[0].clone();
    q.raw_damage_successes = 5;
    assert_eq!(special::effect_dice(&c, &q, &catalog()).unwrap(), Some(1));
    let hp = fighter(&c, &a).hp;
    assert!(special::effects(&mut c, &q, &[1], &catalog()).unwrap());
    assert_eq!(fighter(&c, &a).hp, hp);
    c.engine = STANDALONE_ENGINE.into();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .interaction_rules = false;
    assert_eq!(special::effect_dice(&c, &q, &catalog()).unwrap(), Some(6));
}
#[test]
fn interactions_every_owned_stat_definition_executes_and_unknown_cards_remain_manual() {
    use crate::combat::{extra, interactions};
    let data: serde_json::Value =
        serde_json::from_str(include_str!("../../../rules/battle-interactions.json")).unwrap();
    for (name, definition) in data["moves"].as_object().unwrap() {
        let (mut c, a, t) = setup("Tackle");
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .reviewed_rules = true;
        let mut q = f(&hit(&c, &a, &t, "Tackle")).queue[0].clone();
        q.move_name = name.clone();
        let sheets = serde_json::to_value(&c.sheets).unwrap();
        assert!(
            interactions::move_effect(&mut c, &q, &[], &catalog()).unwrap(),
            "{name}"
        );
        for change in definition["changes"].as_array().unwrap() {
            let id = if change["self"] == true { &a } else { &t };
            for key in change["keys"].as_array().unwrap() {
                let key = key.as_str().unwrap();
                assert_eq!(
                    fighter(&c, id).runtime.modifiers[key],
                    change["amount"].as_i64().unwrap() as i32,
                    "{name}/{key}"
                );
            }
        }
        assert_eq!(serde_json::to_value(&c.sheets).unwrap(), sheets);
        c.engine = STANDALONE_ENGINE.into();
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .interaction_rules = false;
        assert!(!interactions::move_effect(&mut c, &q, &[], &catalog()).unwrap());
    }
    let (mut c, a, t) = setup("Tackle");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .reviewed_rules = true;
    ability(&mut c, &a, "Teravolt");
    ability(&mut c, &t, "Clear Body");
    extra::change_stat(&mut c, &a, &t, "Strength", -1).unwrap();
    assert_eq!(fighter(&c, &t).runtime.modifiers["Strength"], -1);
    ability(&mut c, &t, "Full Metal Body");
    assert!(extra::change_stat(&mut c, &a, &t, "Strength", -1).is_err());
    let mut q = f(&hit(&c, &a, &t, "Tackle")).queue[0].clone();
    q.move_name = "Acupressure".into();
    assert!(!interactions::move_effect(&mut c, &q, &[], &catalog()).unwrap());
}
#[test]
fn interactions_unseen_fist_uses_frozen_action_identity_after_damage() {
    use crate::combat::{interactions, special};
    let (mut c, a, t) = setup("Iron Head");
    let state = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    state.reviewed_rules = true;
    state.ability_changes.enabled = true;
    ability(&mut c, &a, "Unseen Fist");
    let mut c = damage(&pass(&hit(&c, &a, &t, "Iron Head")));
    let q = f(&c).queue[0].clone();
    assert!(q.damage_context.is_some());
    ability(&mut c, &a, "");
    let m = catalog().find("moves", "Name", "Iron Head").unwrap();
    assert!(interactions::unseen(&c, fighter(&c, &a), &m));
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .special
        .shields
        .insert(
            t.clone(),
            special::Shield {
                name: "Spiky Shield".into(),
                source: t,
                round: 1,
                armed_turn: 0,
                available: true,
            },
        );
    assert_eq!(special::shield_reduction(&c, &q, &m), 0);
}
