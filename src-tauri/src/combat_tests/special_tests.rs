use super::targeting_tests::{damage_command, pass, roll};
use super::*;
use crate::combat::{field, special};
pub(super) fn setup(name: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = fixture(name);
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.special.enabled = true;
    f.extra.enabled = true;
    f.ability_rules = true;
    f.battlefield.enabled = true;
    f.battlefield.sunlight = true;
    let species = catalog().find("species", "Name", "Eevee").unwrap()["AppId"]
        .as_str()
        .unwrap()
        .to_string();
    for p in &mut c.battle.as_mut().unwrap().combatants {
        p.selected_ability.clear();
        let s = c.sheets.get_mut(&p.sheet_id).unwrap();
        s.species_id = Some(species.clone());
        s.rank = "Standard".into();
        s.moves.extend(
            [
                "Tackle",
                "Protect",
                "Rest",
                "Solar Beam",
                "Double Kick",
                "Thrash",
            ]
            .map(String::from),
        );
        for key in ["Strength", "Dexterity", "Vitality", "Special", "Insight"] {
            s.stats.insert(key.into(), 4);
        }
        for key in ["Brawl", "Channel", "Nature", "Athletic", "Charm", "Perform"] {
            s.skills.insert(key.into(), 5);
        }
        for key in ["Cool", "Cute", "Clever", "Beauty", "Tough"] {
            s.stats.insert(key.into(), 4);
        }
    }
    (c, a, t)
}
fn cmd(command: special::SpecialCommand) -> Action {
    Action::SpecialMove {
        command,
        reason: "Reviewed special fixture".into(),
    }
}
fn effect(c: &Campaign) -> Campaign {
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    roll(
        c,
        "gm",
        &Action::ResolveEffect {
            sequence: q.sequence,
            reason: "Resolve reviewed effect".into(),
        },
        6,
    )
}
fn attack(c: &Campaign, a: &str, t: &str, name: &str) -> Campaign {
    roll(
        c,
        "gm",
        &Action::Accuracy {
            combatant_id: a.into(),
            target_id: Some(t.into()),
            move_name: name.into(),
            pool_modifier: 0,
            success_modifier: 0,
            reason: "Reviewed special accuracy".into(),
        },
        6,
    )
}
fn hp(c: &Campaign, id: &str) -> i32 {
    c.battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .find(|p| p.id == id)
        .unwrap()
        .hp
}
fn force_turn(c: &mut Campaign, id: &str) {
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.cursor = f.order.iter().position(|v| v == id).unwrap();
    f.queue.clear();
    f.window = false;
    f.turn += 1;
    f.reacted.clear();
}

#[test]
fn special_charge_stores_accuracy_spends_two_actions_and_opens_reactions_only_on_release() {
    let (c, a, t) = setup("Solar Beam");
    let c = attack(&c, &a, &t, "Solar Beam");
    let b = c.battle.as_ref().unwrap();
    assert_eq!(b.combatants[0].actions, 1);
    assert!(b.flow.as_ref().unwrap().queue.is_empty());
    assert!(b.flow.as_ref().unwrap().special.charges.contains_key(&a));
    let mut c = c;
    force_turn(&mut c, &a);
    let c = roll(
        &c,
        "gm",
        &cmd(special::SpecialCommand::Release {
            combatant_id: a.clone(),
        }),
        6,
    );
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].actions, 2);
    assert!(c.battle.as_ref().unwrap().flow.as_ref().unwrap().window);
    let c = pass(&c);
    let c = roll(&c, "gm", &damage_command(&c), 4);
    assert!(hp(&c, &t) < 20);
}
#[test]
fn special_weather_charge_skip_and_extra_preparation_obey_the_manual() {
    for (weather, expected) in [(field::Weather::Sunny, false), (field::Weather::Rain, true)] {
        let (mut c, a, t) = setup("Solar Beam");
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .battlefield
            .natural = Some(weather);
        let c = attack(&c, &a, &t, "Solar Beam");
        assert_eq!(
            c.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .special
                .charges
                .contains_key(&a),
            expected
        );
        if expected {
            assert_eq!(
                c.battle
                    .as_ref()
                    .unwrap()
                    .flow
                    .as_ref()
                    .unwrap()
                    .special
                    .charges[&a]
                    .remaining,
                1
            );
        }
    }
}
#[test]
fn special_successive_rolls_commit_budget_and_keep_one_turn_and_evasion_slot() {
    let (c, a, t) = setup("Double Kick");
    let turn = c.battle.as_ref().unwrap().flow.as_ref().unwrap().turn;
    let c = roll(
        &c,
        "gm",
        &cmd(special::SpecialCommand::Successive {
            reaction: false,
            combatant_id: a.clone(),
            target_id: t.clone(),
            move_name: "Double Kick".into(),
            attempts: 2,
        }),
        4,
    );
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].actions, 2);
    let c = pass(&c);
    let c = roll(&c, "gm", &damage_command(&c), 4);
    let c = effect(&c);
    assert_eq!(c.battle.as_ref().unwrap().flow.as_ref().unwrap().turn, turn);
    assert_eq!(
        c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].special_index,
        2
    );
    assert!(c.battle.as_ref().unwrap().flow.as_ref().unwrap().window);
    let c = pass(&c);
    let c = roll(&c, "gm", &damage_command(&c), 4);
    let c = effect(&c);
    assert_eq!(
        c.battle.as_ref().unwrap().flow.as_ref().unwrap().turn,
        turn + 1
    );
}
#[test]
fn special_rampage_repeats_and_ends_with_confusion() {
    let (c, a, t) = setup("Thrash");
    let c = attack(&c, &a, &t, "Thrash");
    let c = pass(&c);
    let c = roll(&c, "gm", &damage_command(&c), 1);
    let mut c = effect(&c);
    assert!(special::repeat(&c, &a, "Thrash"));
    force_turn(&mut c, &a);
    let c = roll(
        &c,
        "gm",
        &cmd(special::SpecialCommand::EndRampage {
            combatant_id: a.clone(),
        }),
        6,
    );
    assert!(c.battle.as_ref().unwrap().combatants[0]
        .runtime
        .statuses
        .iter()
        .any(|s| s.kind == "Confusion"));
    assert!(!special::repeat(&c, &a, "Thrash"));
}
#[test]
fn special_fixed_and_rank_damage_use_reviewed_pools_and_type_immunity() {
    for (name, dice) in [
        ("Dragon Rage", 0),
        ("Night Shade", 4),
        ("Seismic Toss", 4),
        ("Psywave", 4),
        ("Final Gambit", 24),
    ] {
        let (mut c, a, t) = setup(name);
        field::change_type(&mut c, &t, "replace", &["Water".into()]).unwrap();
        let c = attack(&c, &a, &t, name);
        let c = pass(&c);
        assert_eq!(
            rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap(),
            dice,
            "{name}"
        );
    }
    let (mut c, a, t) = setup("Dragon Rage");
    field::change_type(&mut c, &t, "replace", &["Fairy".into()]).unwrap();
    let c = attack(&c, &a, &t, "Dragon Rage");
    let c = pass(&c);
    let c = roll(&c, "gm", &damage_command(&c), 6);
    assert_eq!(hp(&c, &t), 20);
}
#[test]
fn special_healing_caps_will_and_rest_does_not_invent_a_status_cure() {
    let (mut c, a, _) = setup("Rest");
    c.battle.as_mut().unwrap().combatants[0].hp = 10;
    c.battle.as_mut().unwrap().combatants[0]
        .runtime
        .statuses
        .push(crate::combat::Status {
            kind: "Poison".into(),
            detail: "".into(),
            rounds: None,
            progress: 0,
        });
    let c = attack(&c, &a, &a, "Rest");
    let c = effect(&c);
    let p = &c.battle.as_ref().unwrap().combatants[0];
    assert_eq!(p.hp, 15);
    assert_eq!(p.runtime.will, 5);
    assert!(p.runtime.statuses.iter().any(|s| s.kind == "Poison"));
    assert!(p.runtime.statuses.iter().any(|s| s.kind == "Sleep"));
}
#[test]
fn special_shield_reduction_and_contact_retaliation_are_separate() {
    let (mut c, a, t) = setup("King's Shield");
    c.sheets
        .get_mut(&c.battle.as_ref().unwrap().combatants[1].sheet_id.clone())
        .unwrap()
        .moves
        .push("King's Shield".into());
    let c = attack(&c, &a, &t, "Tackle");
    let c = roll(
        &c,
        "gm",
        &Action::React {
            combatant_id: t.clone(),
            kind: "move".into(),
            move_name: "King's Shield".into(),
            reason: "Shield reaction".into(),
        },
        6,
    );
    let c = effect(&c);
    let c = roll(&c, "gm", &damage_command(&c), 4);
    assert_eq!(
        c.battle.as_ref().unwrap().combatants[0]
            .runtime
            .modifiers
            .get("Strength"),
        Some(&-2)
    );
    assert!(hp(&c, &t) > 0);
}

#[test]
fn special_all_basic_heal_definitions_charge_once_and_respect_round_caps() {
    for name in [
        "Recover",
        "Slack Off",
        "Soft Boiled",
        "Milk Drink",
        "Heal Order",
        "Roost",
        "Shore Up",
        "Synthesis",
        "Morning Sun",
        "Moonlight",
    ] {
        let (mut c, a, _) = setup(name);
        c.battle.as_mut().unwrap().combatants[0].hp = 10;
        c.battle.as_mut().unwrap().combatants[0].runtime.healed = 2;
        let c = attack(&c, &a, &a, name);
        let c = effect(&c);
        assert_eq!(hp(&c, &a), 11, "{name}");
        assert_eq!(
            c.battle.as_ref().unwrap().combatants[0].runtime.will,
            5,
            "{name}"
        );
    }
}
#[test]
fn special_wish_and_ongoing_recovery_apply_once_at_the_correct_boundary() {
    let (mut c, a, _) = setup("Wish");
    c.battle.as_mut().unwrap().combatants[0].hp = 10;
    let c = attack(&c, &a, &a, "Wish");
    let mut c = effect(&c);
    assert_eq!(hp(&c, &a), 10);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .order
        .clear();
    let c = roll(&c, "gm", &Action::NextRound, 4);
    assert_eq!(hp(&c, &a), 13);
    assert!(!c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .special
        .effects
        .iter()
        .any(|e| e.name == "Wish"));
    let (mut c, a, _) = setup("Aqua Ring");
    c.battle.as_mut().unwrap().combatants[0].hp = 10;
    let c = attack(&c, &a, &a, "Aqua Ring");
    let mut c = effect(&c);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .order
        .clear();
    let c = roll(&c, "gm", &Action::NextRound, 4);
    assert_eq!(hp(&c, &a), 11);
}
#[test]
fn special_drain_is_capped_and_liquid_ooze_reverses_only_the_recovery() {
    for ooze in [false, true] {
        let (mut c, a, t) = setup("Absorb");
        c.battle.as_mut().unwrap().combatants[0].hp = 10;
        if ooze {
            c.battle.as_mut().unwrap().combatants[1].selected_ability = "Liquid Ooze".into();
        }
        let c = attack(&c, &a, &t, "Absorb");
        let c = pass(&c);
        let c = roll(&c, "gm", &damage_command(&c), 4);
        let amount = c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].damage / 2;
        let c = effect(&c);
        assert_eq!(
            hp(&c, &a),
            if ooze {
                10 - amount
            } else {
                10 + amount.min(3)
            }
        );
    }
}
#[test]
fn special_revives_a_registered_reserve_without_granting_an_active_slot() {
    let (mut c, a, t) = setup("Revival Blessing");
    let p = &mut c.battle.as_mut().unwrap().combatants[1];
    p.hp = 0;
    p.deployed = false;
    p.field_slot = None;
    p.side = "Players".into();
    let c = attack(&c, &a, &t, "Revival Blessing");
    let c = effect(&c);
    assert_eq!(hp(&c, &t), 2);
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].runtime.will, 0);
    assert!(!c.battle.as_ref().unwrap().combatants[1].deployed);
}
#[test]
fn special_healing_wish_transfers_slot_actions_and_identity_without_editing_sheets() {
    let (mut c, a, t) = setup("Healing Wish");
    c.battle.as_mut().unwrap().combatants[0].field_slot = Some(0);
    let p = &mut c.battle.as_mut().unwrap().combatants[1];
    p.hp = 1;
    p.deployed = false;
    p.side = "Players".into();
    p.field_slot = None;
    let sheets = serde_json::to_value(&c.sheets).unwrap();
    let c = attack(&c, &a, &a, "Healing Wish");
    let seq = c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sequence;
    let c = roll(
        &c,
        "gm",
        &cmd(special::SpecialCommand::Select {
            sequence: seq,
            recipient_id: t.clone(),
            secondary_id: String::new(),
        }),
        4,
    );
    let c = effect(&c);
    assert_eq!(hp(&c, &a), 0);
    assert_eq!(hp(&c, &t), 20);
    let target = &c.battle.as_ref().unwrap().combatants[1];
    assert!(target.deployed);
    assert_eq!(target.field_slot, Some(0));
    assert_eq!(target.actions, 1);
    assert_eq!(serde_json::to_value(&c.sheets).unwrap(), sheets);
}
#[test]
fn special_leech_seed_requires_round_end_and_cannot_tick_twice() {
    let (mut c, a, t) = setup("Leech Seed");
    c.battle.as_mut().unwrap().combatants[0].hp = 10;
    let c = attack(&c, &a, &t, "Leech Seed");
    let c = pass(&c);
    let mut c = effect(&c);
    let index = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .special
        .effects
        .iter()
        .position(|e| e.name == "Leech Seed")
        .unwrap() as u32;
    let tick = cmd(special::SpecialCommand::Tick { index });
    assert!(rules::dice_count(&c, &tick, &catalog()).is_err());
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .order
        .clear();
    let c = roll(&c, "gm", &Action::NextRound, 4);
    assert_eq!(c.battle.as_ref().unwrap().round, 1);
    let c = roll(&c, "gm", &tick, 4);
    assert_eq!(hp(&c, &t), 18);
    assert_eq!(hp(&c, &a), 12);
    assert!(rules::dice_count(&c, &tick, &catalog()).is_err());
    let c = roll(&c, "gm", &Action::NextRound, 4);
    assert_eq!(c.battle.as_ref().unwrap().round, 2);
}
#[test]
fn special_move_modifiers_keep_highest_buffs_and_debuffs_in_separate_channels() {
    let (mut c, a, t) = setup("Tackle");
    crate::combat::extra::change_stat(&mut c, &a, &a, "Strength", 2).unwrap();
    crate::combat::extra::change_stat(&mut c, &a, &a, "Strength", 2).unwrap();
    crate::combat::extra::change_stat(&mut c, &t, &a, "Strength", -1).unwrap();
    assert_eq!(
        rules::effective_sheet(&c, &c.battle.as_ref().unwrap().combatants[0])
            .unwrap()
            .stats["Strength"],
        5
    );
    c.battle.as_mut().unwrap().combatants[0].selected_ability = "Simple".into();
    assert_eq!(
        rules::effective_sheet(&c, &c.battle.as_ref().unwrap().combatants[0])
            .unwrap()
            .stats["Strength"],
        5
    );
}
#[test]
fn special_weight_hp_dexterity_and_happiness_formulas_have_exact_caps() {
    let (mut c, a, t) = setup("Tackle");
    let p = &mut c.battle.as_mut().unwrap().combatants[0];
    p.hp = 12;
    p.stat_overrides.insert("Dexterity".into(), 10);
    let p = &mut c.battle.as_mut().unwrap().combatants[1];
    p.stat_overrides.insert("Dexterity".into(), 2);
    let c = roll(
        &c,
        "gm",
        &cmd(special::SpecialCommand::Parameters {
            combatant_id: t.clone(),
            weight_kg: Some(300.),
            lift_kg: None,
            night: false,
        }),
        4,
    );
    for (name, expected) in [
        ("Flail", 5),
        ("Water Spout", -5),
        ("Electro Ball", 4),
        ("Low Kick", 4),
        ("Return", 2),
        ("Frustration", 3),
    ] {
        let m = catalog().find("moves", "Name", name).unwrap();
        assert_eq!(
            special::bonus(
                &c,
                &c.battle.as_ref().unwrap().combatants[0],
                &c.battle.as_ref().unwrap().combatants[1],
                &m,
                &catalog()
            )
            .unwrap(),
            expected,
            "{name}"
        );
    }
    let _ = a;
}
#[test]
fn special_counter_uses_the_recorded_pool_not_damage_successes() {
    let (c, a, t) = setup("Tackle");
    let c = attack(&c, &a, &t, "Tackle");
    let c = pass(&c);
    let n = rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap();
    let mut c = roll(&c, "gm", &damage_command(&c), 1);
    force_turn(&mut c, &t);
    c.sheets
        .get_mut(&c.battle.as_ref().unwrap().combatants[1].sheet_id.clone())
        .unwrap()
        .moves
        .push("Counter".into());
    let c = attack(&c, &t, &a, "Counter");
    let c = pass(&c);
    assert_eq!(
        rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap(),
        n + 2
    );
}
#[test]
fn special_reconnect_and_rollback_preserve_charges_and_command_authority() {
    let (c, a, t) = setup("Solar Beam");
    let c = attack(&c, &a, &t, "Solar Beam");
    let before = serde_json::to_value(&c).unwrap();
    let mut copy: Campaign = serde_json::from_value(before.clone()).unwrap();
    force_turn(&mut copy, &a);
    let action = cmd(special::SpecialCommand::Release {
        combatant_id: a.clone(),
    });
    assert!(battle::transition(&copy, "outsider", &action, vec![], &catalog()).is_err());
    let released = roll(&copy, "gm", &action, 4);
    assert!(released
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .special
        .charges
        .is_empty());
    assert_eq!(serde_json::to_value(&c).unwrap(), before);
    let restored = decision(
        &released,
        "gm",
        Action::RollbackBattle {
            event_sequence: released.revision + 1,
            reason: "Restore charged state".into(),
        },
    );
    assert!(restored
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .special
        .charges
        .contains_key(&a));
}

#[test]
fn special_successive_reactions_resume_the_original_attack_after_every_hit() {
    let (mut c, a, t) = setup("Tackle");
    c.sheets
        .get_mut(&c.battle.as_ref().unwrap().combatants[1].sheet_id.clone())
        .unwrap()
        .moves
        .push("Water Shuriken".into());
    let c = attack(&c, &a, &t, "Tackle");
    let turn = c.battle.as_ref().unwrap().flow.as_ref().unwrap().turn;
    let c = roll(
        &c,
        "gm",
        &cmd(special::SpecialCommand::Successive {
            reaction: true,
            combatant_id: t.clone(),
            target_id: a.clone(),
            move_name: "Water Shuriken".into(),
            attempts: 2,
        }),
        4,
    );
    let c = pass(&c);
    let c = roll(&c, "gm", &damage_command(&c), 1);
    let c = effect(&c);
    let c = pass(&c);
    let c = roll(&c, "gm", &damage_command(&c), 1);
    let c = effect(&c);
    let f = c.battle.as_ref().unwrap().flow.as_ref().unwrap();
    assert_eq!(f.queue[0].actor_id, a);
    assert_eq!(f.turn, turn);
    assert!(!f.window);
    assert!(f.reacted.contains(&t));
    let c = roll(&c, "gm", &damage_command(&c), 1);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .is_empty());
    assert_eq!(c.battle.as_ref().unwrap().combatants[1].actions, 2);
}
#[test]
fn special_bide_requires_gm_basis_then_records_two_real_hits() {
    let (mut c, a, t) = setup("Bide");
    let tid = c.battle.as_ref().unwrap().combatants[1].sheet_id.clone();
    c.sheets.get_mut(&tid).unwrap().moves.push("Swift".into());
    let c = attack(&c, &a, &t, "Bide");
    let c = pass(&c);
    let seq = c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sequence;
    let selection = cmd(special::SpecialCommand::Select {
        sequence: seq,
        recipient_id: String::new(),
        secondary_id: a.clone(),
    });
    assert!(battle::transition(&c, "player", &selection, vec![], &catalog()).is_err());
    let c = roll(&c, "gm", &selection, 4);
    let mut c = effect(&c);
    for name in ["Tackle", "Swift"] {
        force_turn(&mut c, &t);
        let next = attack(&c, &t, &a, name);
        let next = pass(&next);
        c = roll(&next, "gm", &damage_command(&next), 1);
    }
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .special
            .bides[&a]
            .received,
        2
    );
    force_turn(&mut c, &a);
    let c = attack(&c, &a, &t, "Bide");
    let c = pass(&c);
    assert_eq!(
        rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap(),
        4
    );
}
#[test]
fn special_sky_drop_checks_lifting_and_transfers_out_of_range_to_the_carried_target() {
    let (c, a, t) = setup("Sky Drop");
    let c = attack(&c, &a, &t, "Sky Drop");
    assert!(special::carried(&c, &t));
    assert!(special::target_guard(
        &c,
        &a,
        &t,
        &catalog().find("moves", "Name", "Tackle").unwrap()
    )
    .is_err());
    assert!(special::vulnerable(&c, &t, "Thunder"));
    assert!(special::vulnerable(&c, &a, "Thunder"));
    let mut released = c.clone();
    force_turn(&mut released, &a);
    let released = roll(
        &released,
        "gm",
        &cmd(special::SpecialCommand::Release {
            combatant_id: a.clone(),
        }),
        6,
    );
    assert!(!special::carried(&released, &t));
    assert!(
        released
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .window
    );
    let (mut c, a, t) = setup("Sky Drop");
    field::change_type(&mut c, &t, "replace", &["Flying".into()]).unwrap();
    let action = Action::Accuracy {
        combatant_id: a,
        target_id: Some(t),
        move_name: "Sky Drop".into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Try lifting Flying target".into(),
    };
    let n = rules::dice_count(&c, &action, &catalog()).unwrap();
    assert!(battle::transition(&c, "gm", &action, vec![6; n as usize], &catalog()).is_err());
}
#[test]
fn special_skill_link_adds_accuracy_dice_without_changing_hit_limit() {
    let (mut c, a, t) = setup("Double Kick");
    let action = cmd(special::SpecialCommand::Successive {
        reaction: false,
        combatant_id: a,
        target_id: t,
        move_name: "Double Kick".into(),
        attempts: 2,
    });
    let ordinary = rules::dice_count(&c, &action, &catalog()).unwrap();
    c.battle.as_mut().unwrap().combatants[0].selected_ability = "Skill Link".into();
    assert_eq!(
        rules::dice_count(&c, &action, &catalog()).unwrap(),
        ordinary + 4
    );
}
#[tokio::test]
async fn special_room_host_validates_gm_parameters_receipts_stale_commands_and_reconnect() {
    let hd = tempfile::tempdir().unwrap();
    let gd = tempfile::tempdir().unwrap();
    let mut store = Store::open(hd.path()).unwrap();
    let gs = Arc::new(Mutex::new(Store::open(gd.path()).unwrap()));
    let gm_id = gs.lock().unwrap().player_id.clone();
    let (mut c, a, _) = setup("Tackle");
    c.host_id = store.player_id.clone();
    c.gm_id = gm_id.clone();
    store.put_campaign(&c).unwrap();
    let shared = Arc::new(Mutex::new(store));
    let cat = Arc::new(catalog());
    let notify: room::Notify = Arc::new(|_, _| {});
    let mut host = room::start(shared.clone(), cat.clone(), c.id.clone(), 0, notify.clone())
        .await
        .unwrap();
    let request = JoinRequest {
        address: host.info.address.replace("<host-ip>", "127.0.0.1"),
        token: host.info.token.clone(),
    };
    let (gm, _) = room::join(gs.clone(), cat.clone(), request.clone(), notify.clone())
        .await
        .unwrap();
    let command = Command {
        id: id(),
        expected_revision: c.revision,
        action: cmd(special::SpecialCommand::Parameters {
            combatant_id: a.clone(),
            weight_kg: Some(200.),
            lift_kg: Some(300.),
            night: true,
        }),
    };
    assert!(host.host.apply(c.host_id(), &command).is_err());
    let (tx, rx) = tokio::sync::oneshot::channel();
    gm.sender.send((command.clone(), tx)).await.unwrap();
    let next = rx.await.unwrap().unwrap();
    assert_eq!(
        next.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .special
            .parameters[&a]
            .weight_kg,
        Some(200.)
    );
    assert_eq!(
        host.host.apply(&gm_id, &command).unwrap().revision,
        next.revision
    );
    let mut stale = command.clone();
    stale.id = id();
    assert!(host.host.apply(&gm_id, &stale).is_err());
    gm.task.abort();
    let (client, snapshot) = room::join(gs, cat, request, notify).await.unwrap();
    assert_eq!(
        snapshot
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .special
            .parameters[&a]
            .lift_kg,
        Some(300.)
    );
    client.task.abort();
    let _ = host.stop.take().unwrap().send(());
}

#[test]
fn special_charge_defense_is_temporary_and_matcha_can_break_its_own_ice() {
    let (c, a, t) = setup("Skull Bash");
    let base = rules::effective_sheet(&c, &c.battle.as_ref().unwrap().combatants[0]).unwrap();
    let c = attack(&c, &a, &t, "Skull Bash");
    assert_eq!(
        special::modifier(&c, &c.battle.as_ref().unwrap().combatants[0], "Defense"),
        Some(1)
    );
    let c = roll(
        &c,
        "gm",
        &cmd(special::SpecialCommand::CancelCharge {
            combatant_id: a.clone(),
        }),
        6,
    );
    assert_eq!(
        special::modifier(&c, &c.battle.as_ref().unwrap().combatants[0], "Defense"),
        None
    );
    assert_eq!(
        rules::effective_sheet(&c, &c.battle.as_ref().unwrap().combatants[0])
            .unwrap()
            .stats,
        base.stats
    );
    let (mut c, a, t) = setup("Matcha Gotcha");
    c.battle.as_mut().unwrap().combatants[0]
        .runtime
        .statuses
        .push(crate::combat::Status {
            kind: "Frozen Solid".into(),
            detail: "ice".into(),
            rounds: None,
            progress: 0,
        });
    let c = attack(&c, &a, &t, "Matcha Gotcha");
    assert!(!c.battle.as_ref().unwrap().combatants[0]
        .runtime
        .statuses
        .iter()
        .any(|s| s.kind == "Frozen Solid"));
}
#[test]
fn special_roost_changes_ground_immunity_only_for_the_encounter_duration() {
    let (mut c, a, t) = setup("Roost");
    field::change_type(&mut c, &a, "replace", &["Flying".into()]).unwrap();
    c.battle.as_mut().unwrap().combatants[0].hp = 10;
    let c = attack(&c, &a, &a, "Roost");
    let mut c = effect(&c);
    assert!(special::roost(&c, &a));
    force_turn(&mut c, &t);
    c.sheets
        .get_mut(&c.battle.as_ref().unwrap().combatants[1].sheet_id.clone())
        .unwrap()
        .moves
        .push("Earthquake".into());
    let c = attack(&c, &t, &a, "Earthquake");
    let c = pass(&c);
    assert!(rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap() > 0);
    assert_eq!(
        field::types(&c, &c.battle.as_ref().unwrap().combatants[0], &catalog()).unwrap(),
        vec!["Flying"]
    );
}
#[test]
fn special_heal_block_prevents_direct_recovery_without_changing_will() {
    let (c, a, t) = setup("Heal Block");
    let mut c = attack(&c, &a, &t, "Heal Block");
    c = effect(&pass(&c));
    let before = c.battle.as_ref().unwrap().combatants[1].runtime.will;
    c.battle.as_mut().unwrap().combatants[1].hp = 10;
    special::heal(&mut c, &t, 3, false).unwrap();
    assert_eq!(hp(&c, &t), 10);
    assert_eq!(
        c.battle.as_ref().unwrap().combatants[1].runtime.will,
        before
    );
}
#[test]
fn special_endure_rearms_on_its_users_turn_and_never_on_another_users_turn() {
    let (mut c, a, t) = setup("Endure");
    let turn = c.battle.as_ref().unwrap().flow.as_ref().unwrap().turn;
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .special
        .shields
        .insert(
            a.clone(),
            special::Shield {
                name: "Endure".into(),
                source: a.clone(),
                round: 1,
                armed_turn: turn,
                available: false,
            },
        );
    force_turn(&mut c, &t);
    let before = c.clone();
    special::lifecycle(&before, &mut c, &catalog()).unwrap();
    assert!(
        !c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .special
            .shields[&a]
            .available
    );
    force_turn(&mut c, &a);
    let before = c.clone();
    special::lifecycle(&before, &mut c, &catalog()).unwrap();
    assert!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .special
            .shields[&a]
            .available
    );
}

#[test]
fn special_sky_uppercut_always_hits_its_printed_airborne_charge_targets() {
    let (c, a, t) = setup("Fly");
    let mut c = attack(&c, &a, &t, "Fly");
    force_turn(&mut c, &t);
    c.sheets
        .get_mut(&c.battle.as_ref().unwrap().combatants[1].sheet_id.clone())
        .unwrap()
        .moves
        .push("Sky Uppercut".into());
    let c = roll(
        &c,
        "gm",
        &Action::Accuracy {
            combatant_id: t,
            target_id: Some(a),
            move_name: "Sky Uppercut".into(),
            pool_modifier: 0,
            success_modifier: 0,
            reason: "Printed airborne exception".into(),
        },
        1,
    );
    assert!(!c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .is_empty());
}
