use super::*;
use crate::combat::{CombatState, TurnFlow};
use crate::rules;
#[path = "combat_tests/targeting_tests.rs"]
mod targeting_tests;
fn fixture(name: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = damage_fixture(name, "Charmander", false);
    c.engine = ENGINE.into();
    let b = c.battle.as_mut().unwrap();
    b.flow = Some(TurnFlow::default());
    for (i, p) in b.combatants.iter_mut().enumerate() {
        p.pending_damage = None;
        p.runtime = CombatState::default();
        p.runtime.will = 6;
        p.runtime.will_enabled = true;
        p.initiative_rolled = true;
        p.initiative = 10 - i as i32;
        p.side = if i == 0 { "Players" } else { "Opponents" }.into();
        p.hp = 20;
        p.max_hp = 20;
    }
    let sheet_id = b.combatants[1].sheet_id.clone();
    c.sheets.get_mut(&sheet_id).unwrap().moves = vec![
        "Tackle".into(),
        "Quick Attack".into(),
        "Extreme Speed".into(),
        "Protect".into(),
    ];
    c.sheets
        .get_mut(&sheet_id)
        .unwrap()
        .skills
        .insert("Evasion".into(), 4);
    c.sheets
        .get_mut(&sheet_id)
        .unwrap()
        .skills
        .insert("Clash".into(), 4);
    c = battle::transition(&c, "gm", &Action::StartTurns, vec![], &catalog())
        .unwrap()
        .0;
    (c, a, t)
}
fn hit(c: &Campaign, a: &str, t: &str, name: &str) -> Campaign {
    let action = Action::Accuracy {
        combatant_id: a.into(),
        target_id: Some(t.into()),
        move_name: name.into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Attack".into(),
    };
    let n = rules::dice_count(c, &action, &catalog()).unwrap();
    battle::transition(c, "player", &action, vec![4; n as usize], &catalog())
        .unwrap()
        .0
}
fn react(c: &Campaign, t: &str, kind: &str, name: &str) -> Campaign {
    let a = Action::React {
        combatant_id: t.into(),
        kind: kind.into(),
        move_name: name.into(),
        reason: "React".into(),
    };
    let n = rules::dice_count(c, &a, &catalog()).unwrap();
    battle::transition(c, "other", &a, vec![4; n as usize], &catalog())
        .unwrap()
        .0
}
#[test]
fn turn_order_and_reaction_budget_are_host_enforced() {
    let (c, a, t) = fixture("Tackle");
    let mut wrong = core_accuracy(&t, "Tackle");
    if let Action::Accuracy { target_id, .. } = &mut wrong {
        *target_id = Some(a.clone());
    }
    assert!(battle::transition(&c, "other", &wrong, vec![4; 7], &catalog()).is_err());
    let c = hit(&c, &a, &t, "Tackle");
    assert!(rules::dice_count(&c, &damage_action(&a), &catalog()).is_err());
    let c = react(&c, &t, "evade", "");
    let b = c.battle.unwrap();
    assert_eq!(b.combatants[0].actions, 1);
    assert_eq!(b.combatants[1].actions, 1);
    assert!(b.flow.as_ref().unwrap().queue.is_empty());
    assert_eq!(
        b.flow.as_ref().unwrap().order[b.flow.as_ref().unwrap().cursor],
        t
    );
}
#[test]
fn never_miss_forbids_evasion_but_allows_clash() {
    let (c, a, t) = fixture("Aerial Ace");
    let c = hit(&c, &a, &t, "Aerial Ace");
    assert!(rules::dice_count(
        &c,
        &Action::React {
            combatant_id: t.clone(),
            kind: "evade".into(),
            move_name: String::new(),
            reason: "Evade".into()
        },
        &catalog()
    )
    .is_err());
    let c = react(&c, &t, "clash", "Tackle");
    assert!(c.battle.unwrap().flow.unwrap().queue.is_empty());
}
#[test]
fn evasion_failed_attempt_spends_turn_slot_but_next_turn_restores_it() {
    let (mut c, a, t) = fixture("Tackle");
    let sheet_id = c.battle.as_ref().unwrap().combatants[0].sheet_id.clone();
    c.sheets
        .get_mut(&sheet_id)
        .unwrap()
        .moves
        .push("Slash".into());
    let c = hit(&c, &a, &t, "Tackle");
    let evade = Action::React {
        combatant_id: t.clone(),
        kind: "evade".into(),
        move_name: String::new(),
        reason: "Evade".into(),
    };
    let n = rules::dice_count(&c, &evade, &catalog()).unwrap();
    let c = battle::transition(&c, "other", &evade, vec![1; n as usize], &catalog())
        .unwrap()
        .0;
    assert_eq!(c.battle.as_ref().unwrap().combatants[1].actions, 1);
    let before = serde_json::to_value(&c).unwrap();
    let err = battle::transition(&c, "other", &evade, vec![4; n as usize], &catalog()).unwrap_err();
    assert!(err.message.contains("Evasion is limited to once per turn"));
    assert_eq!(serde_json::to_value(&c).unwrap(), before);
    let clash = Action::React {
        combatant_id: t.clone(),
        kind: "clash".into(),
        move_name: "Tackle".into(),
        reason: "Clash instead".into(),
    };
    assert!(rules::dice_count(&c, &clash, &catalog()).is_err());
    let c = damage(&c, "player", 1);
    let c = hold(&c, &t);
    let c = hit(&c, &a, &t, "Slash");
    assert_eq!(c.battle.as_ref().unwrap().round, 1);
    assert!(rules::dice_count(&c, &evade, &catalog()).is_ok());
    let c = react(&c, &t, "evade", "");
    assert_eq!(c.battle.as_ref().unwrap().combatants[1].actions, 2);
}

#[test]
fn evasion_can_spend_fifth_action_but_cannot_exceed_round_budget() {
    let (c, a, t) = fixture("Tackle");
    let mut c = hit(&c, &a, &t, "Tackle");
    c.battle.as_mut().unwrap().combatants[1].actions = 4;
    let evade = Action::React {
        combatant_id: t.clone(),
        kind: "evade".into(),
        move_name: String::new(),
        reason: "Last action".into(),
    };
    let spent = react(&c, &t, "evade", "");
    assert_eq!(spent.battle.as_ref().unwrap().combatants[1].actions, 5);
    assert_eq!(spent.battle.as_ref().unwrap().round, 1);
    c.battle.as_mut().unwrap().combatants[1].actions = 5;
    assert!(rules::dice_count(&c, &evade, &catalog()).is_err());
    assert!(battle::transition(&c, "other", &evade, vec![], &catalog()).is_err());
}
#[test]
fn reaction_moves_resolve_before_main_and_cannot_repeat_per_turn() {
    let (c, a, t) = fixture("Tackle");
    let c = hit(&c, &a, &t, "Tackle");
    let c = react(&c, &t, "move", "Quick Attack");
    assert!(rules::dice_count(
        &c,
        &Action::React {
            combatant_id: t.clone(),
            kind: "move".into(),
            move_name: "Quick Attack".into(),
            reason: "Again".into()
        },
        &catalog()
    )
    .is_err());
    let c = battle::transition(
        &c,
        "player",
        &Action::PassReaction {
            combatant_id: a.clone(),
            reason: "Take hit".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    assert_eq!(
        c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].move_name,
        "Quick Attack"
    );
    let q = c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].clone();
    let damage = Action::Damage {
        combatant_id: t,
        sequence: q.sequence,
        pool_modifier: 0,
        success_modifier: 0,
        damage_modifier: 0,
        reason: "Damage".into(),
    };
    let n = rules::dice_count(&c, &damage, &catalog()).unwrap();
    let c = battle::transition(&c, "other", &damage, vec![1; n as usize], &catalog())
        .unwrap()
        .0;
    assert_eq!(c.battle.unwrap().flow.unwrap().queue[0].move_name, "Tackle");
}

#[test]
fn clash_can_answer_a_new_counterattack_after_reacting_in_the_same_turn() {
    let (mut c, a, t) = fixture("Tackle");
    let sheet_id = c.battle.as_ref().unwrap().combatants[0].sheet_id.clone();
    c.sheets
        .get_mut(&sheet_id)
        .unwrap()
        .moves
        .push("Extreme Speed".into());
    let c = hit(&c, &a, &t, "Tackle");
    let c = react(&c, &t, "move", "Quick Attack");
    let counter = Action::React {
        combatant_id: a.clone(),
        kind: "move".into(),
        move_name: "Extreme Speed".into(),
        reason: "Counterattack".into(),
    };
    let n = rules::dice_count(&c, &counter, &catalog()).unwrap();
    let c = battle::transition(&c, "player", &counter, vec![4; n as usize], &catalog())
        .unwrap()
        .0;
    let turn = c.battle.as_ref().unwrap().flow.as_ref().unwrap().turn;
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .reacted
        .contains(&t));
    let evade = Action::React {
        combatant_id: t.clone(),
        kind: "evade".into(),
        move_name: String::new(),
        reason: "Already reacted".into(),
    };
    assert!(rules::dice_count(&c, &evade, &catalog()).is_err());
    let clash = Action::React {
        combatant_id: t.clone(),
        kind: "clash".into(),
        move_name: "Tackle".into(),
        reason: "Clash counterattack".into(),
    };
    let n = rules::dice_count(&c, &clash, &catalog()).unwrap();
    let c = battle::transition(&c, "other", &clash, vec![1; n as usize], &catalog())
        .unwrap()
        .0;
    assert_eq!(c.battle.as_ref().unwrap().combatants[1].actions, 2);
    assert_eq!(c.battle.as_ref().unwrap().flow.as_ref().unwrap().turn, turn);
    assert!(rules::dice_count(&c, &clash, &catalog()).is_err());
}

#[test]
fn clash_does_not_spend_the_turn_slot_and_still_spends_the_last_action() {
    let (c, a, t) = fixture("Tackle");
    let mut c = hit(&c, &a, &t, "Tackle");
    c.battle.as_mut().unwrap().combatants[1].actions = 4;
    let clash = Action::React {
        combatant_id: t.clone(),
        kind: "clash".into(),
        move_name: "Tackle".into(),
        reason: "Last action".into(),
    };
    let n = rules::dice_count(&c, &clash, &catalog()).unwrap();
    let spent = battle::transition(&c, "other", &clash, vec![1; n as usize], &catalog())
        .unwrap()
        .0;
    assert_eq!(spent.battle.as_ref().unwrap().combatants[1].actions, 5);
    assert!(!spent
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .reacted
        .contains(&t));
    c.battle.as_mut().unwrap().combatants[1].actions = 5;
    assert!(rules::dice_count(&c, &clash, &catalog()).is_err());
    assert!(battle::transition(&c, "other", &clash, vec![], &catalog()).is_err());
}
#[test]
fn will_fate_survives_penalties_and_cannot_mix_with_chance() {
    let (mut c, a, t) = fixture("Thunder");
    let p = &mut c.battle.as_mut().unwrap().combatants[0];
    p.hp = 1;
    c = battle::transition(
        &c,
        "player",
        &Action::SpendWill {
            combatant_id: a.clone(),
            mode: "fate".into(),
            reason: "Push".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    assert!(battle::transition(
        &c,
        "player",
        &Action::SpendWill {
            combatant_id: a.clone(),
            mode: "chance".into(),
            reason: "Conflict".into()
        },
        vec![],
        &catalog()
    )
    .is_err());
    let action = Action::Accuracy {
        combatant_id: a.clone(),
        target_id: Some(t),
        move_name: "Thunder".into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Fate".into(),
    };
    let n = rules::dice_count(&c, &action, &catalog()).unwrap();
    let c = battle::transition(&c, "player", &action, vec![1; n as usize], &catalog())
        .unwrap()
        .0;
    assert_eq!(c.battle.unwrap().flow.unwrap().queue[0].successes, 1);
}
#[test]
fn structured_status_immunity_and_round_damage() {
    let (mut c, a, t) = fixture("Tackle");
    let status = |id: String, kind: &str| Action::SetStatus {
        combatant_id: id,
        kind: kind.into(),
        detail: String::new(),
        rounds: None,
        remove: false,
        reason: "GM condition".into(),
    };
    assert!(battle::transition(&c, "gm", &status(t.clone(), "Burn"), vec![], &catalog()).is_err()); // Charmander.
    c = battle::transition(&c, "gm", &status(a.clone(), "Poison"), vec![], &catalog())
        .unwrap()
        .0;
    c = battle::transition(
        &c,
        "player",
        &Action::PassTurn {
            combatant_id: a,
            reason: "Hold".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    c = battle::transition(
        &c,
        "other",
        &Action::PassTurn {
            combatant_id: t,
            reason: "Hold".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    let before = c.battle.as_ref().unwrap().combatants[0].hp;
    c = battle::transition(&c, "gm", &Action::NextRound, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(c.battle.unwrap().combatants[0].hp, before - 2);
}
#[test]
fn rollback_restores_turn_state_statuses_and_will() {
    let (c, a, _) = fixture("Tackle");
    let next = battle::transition(
        &c,
        "player",
        &Action::SpendWill {
            combatant_id: a,
            mode: "chance".into(),
            reason: "Spend".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    let rolled = battle::transition(
        &next,
        "gm",
        &Action::RollbackBattle {
            event_sequence: c.revision + 1,
            reason: "Undo".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    assert_eq!(
        rolled.battle.as_ref().unwrap().combatants[0].runtime.will,
        6
    );
    assert_eq!(
        rolled.battle.unwrap().flow.unwrap().turn,
        c.battle.unwrap().flow.unwrap().turn
    );
}

fn decision(c: &Campaign, actor: &str, a: Action) -> Campaign {
    battle::transition(c, actor, &a, vec![], &catalog())
        .unwrap()
        .0
}
fn condition(c: &Campaign, id: &str, kind: &str, rounds: Option<u32>) -> Campaign {
    decision(
        c,
        "gm",
        Action::SetStatus {
            combatant_id: id.into(),
            kind: kind.into(),
            detail: String::new(),
            rounds,
            remove: false,
            reason: "Reviewed condition".into(),
        },
    )
}
fn hold(c: &Campaign, id: &str) -> Campaign {
    decision(
        c,
        "gm",
        Action::PassTurn {
            combatant_id: id.into(),
            reason: "Hold".into(),
        },
    )
}
fn round(c: &Campaign, a: &str, t: &str) -> Campaign {
    let c = hold(c, a);
    let c = hold(&c, t);
    decision(&c, "gm", Action::NextRound)
}
fn damage(c: &Campaign, actor: &str, face: u8) -> Campaign {
    let q = c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].clone();
    let action = Action::Damage {
        combatant_id: q.actor_id,
        sequence: q.sequence,
        pool_modifier: 0,
        success_modifier: 0,
        damage_modifier: 0,
        reason: "Reviewed damage".into(),
    };
    let n = rules::dice_count(c, &action, &catalog()).unwrap();
    battle::transition(c, actor, &action, vec![face; n as usize], &catalog())
        .unwrap()
        .0
}
fn effect(c: &Campaign, face: u8) -> Campaign {
    let action = Action::ResolveEffect {
        sequence: c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sequence,
        reason: "Reviewed effect".into(),
    };
    let n = rules::dice_count(c, &action, &catalog()).unwrap();
    battle::transition(c, "gm", &action, vec![face; n as usize], &catalog())
        .unwrap()
        .0
}
#[test]
fn protect_uses_will_without_pain_and_reduces_damage_by_three() {
    let (mut c, a, t) = fixture("Tackle");
    c.battle.as_mut().unwrap().combatants[1].hp = 10;
    let c = hit(&c, &a, &t, "Tackle");
    let c = react(&c, &t, "move", "Protect");
    assert_eq!(
        c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].successes,
        6
    );
    let c = effect(&c, 1);
    let action = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    assert_eq!(action.move_name, "Tackle");
    let c = damage(&c, "player", 1);
    assert_eq!(c.battle.as_ref().unwrap().combatants[1].hp, 10);
    assert_eq!(c.battle.as_ref().unwrap().combatants[1].runtime.shield, 0);
    assert_eq!(
        c.battle.as_ref().unwrap().combatants[1]
            .runtime
            .shield_round,
        1
    );
}
#[test]
fn healing_debits_will_caps_recovery_and_cannot_revive() {
    let (mut c, a, _) = fixture("Recover");
    c.battle.as_mut().unwrap().combatants[0].hp = 10;
    let c = hit(&c, &a, &a, "Recover");
    let c = effect(&c, 1);
    let p = &c.battle.as_ref().unwrap().combatants[0];
    assert_eq!((p.hp, p.runtime.will, p.runtime.healed), (13, 5, 3));
    let mut dead = c.clone();
    dead.battle.as_mut().unwrap().combatants[0].hp = 0;
    assert!(rules::dice_count(&dead, &core_accuracy(&a, "Recover"), &catalog()).is_err());
}
#[test]
fn reapplication_escalates_poison_and_does_not_reset_active_duration() {
    let (c, a, t) = fixture("Tackle");
    let c = condition(&c, &a, "Poison", None);
    let c = condition(&c, &a, "Poison", None);
    assert_eq!(
        c.battle.as_ref().unwrap().combatants[0].runtime.statuses[0].kind,
        "Badly Poisoned"
    );
    let c = condition(&c, &t, "Paralysis", Some(2));
    let c = condition(&c, &t, "Paralysis", Some(4));
    assert_eq!(
        c.battle.as_ref().unwrap().combatants[1]
            .runtime
            .statuses
            .iter()
            .find(|s| s.kind == "Paralysis")
            .unwrap()
            .rounds,
        Some(2)
    );
    let c = hold(&c, &a);
    let c = hold(&c, &t);
    let c = decision(
        &c,
        "gm",
        Action::AdjudicateStatus {
            combatant_id: a.clone(),
            damage: 2,
            reason: "Choose one printed severe-status interpretation".into(),
        },
    );
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].hp, 18);
    assert!(battle::transition(
        &c,
        "gm",
        &Action::AdjudicateStatus {
            combatant_id: a,
            damage: 2,
            reason: "Again".into()
        },
        vec![],
        &catalog()
    )
    .is_err());
}
#[test]
fn status_duration_and_pending_fate_expire_without_reusing_turn_identity() {
    let (c, a, t) = fixture("Tackle");
    let original_turn = c.battle.as_ref().unwrap().flow.as_ref().unwrap().turn;
    let c = condition(&c, &a, "Confusion", Some(1));
    let c = decision(
        &c,
        "player",
        Action::SpendWill {
            combatant_id: a.clone(),
            mode: "fate".into(),
            reason: "Prepare".into(),
        },
    );
    let c = round(&c, &a, &t);
    assert!(c.battle.as_ref().unwrap().combatants[0]
        .runtime
        .statuses
        .is_empty());
    assert!(
        !c.battle.as_ref().unwrap().combatants[0]
            .runtime
            .fate_pending
    );
    let c = decision(&c, "gm", Action::StartTurns);
    assert!(c.battle.unwrap().flow.unwrap().turn > original_turn);
}
#[test]
fn frozen_subject_can_break_ice_at_round_end_without_losing_its_hp() {
    let (c, a, t) = fixture("Tackle");
    let c = condition(&c, &a, "Frozen Solid", None);
    assert!(rules::dice_count(&c, &core_accuracy(&a, "Tackle"), &catalog()).is_err());
    let c = hold(&c, &a);
    let c = hold(&c, &t);
    let action = Action::BreakIce {
        combatant_id: a.clone(),
        target_id: a.clone(),
        move_name: "Tackle".into(),
        reason: "Break free".into(),
    };
    let n = rules::dice_count(&c, &action, &catalog()).unwrap();
    let c = battle::transition(&c, "player", &action, vec![4; n as usize], &catalog())
        .unwrap()
        .0;
    let c = damage(&c, "player", 4);
    let p = &c.battle.as_ref().unwrap().combatants[0];
    assert_eq!(p.hp, 20);
    assert!(!p.runtime.statuses.iter().any(|s| s.kind == "Frozen Solid"));
    assert_eq!(p.actions, 1);
}
#[test]
fn sonic_boom_ignores_shield_and_recoil_is_based_on_raw_damage_successes() {
    let (mut c, a, t) = fixture("Sonic Boom");
    let sid = c.battle.as_ref().unwrap().combatants[1].sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some(
        catalog().find("species", "Name", "Gastly").unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
    let c = hit(&c, &a, &t, "Sonic Boom");
    let c = react(&c, &t, "move", "Protect");
    let c = effect(&c, 1);
    let c = damage(&c, "player", 1);
    assert_eq!(c.battle.as_ref().unwrap().combatants[1].hp, 19);
    let (c, a, t) = fixture("Submission");
    let c = hit(&c, &a, &t, "Submission");
    let c = decision(
        &c,
        "other",
        Action::PassReaction {
            combatant_id: t,
            reason: "Accept attack".into(),
        },
    );
    let c = damage(&c, "player", 4);
    let n = c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].raw_damage_successes;
    assert!(n > 0);
    let c = effect(&c, 4);
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].hp, 20 - n as i32);
}
#[test]
fn conditional_ability_snapshot_removes_pain_and_adds_two_damage_dice() {
    let (mut c, a, t) = fixture("Ember");
    let p = &mut c.battle.as_mut().unwrap().combatants[0];
    p.selected_ability = "Blaze".into();
    p.hp = 10;
    let c = hit(&c, &a, &t, "Ember");
    let c = decision(
        &c,
        "other",
        Action::PassReaction {
            combatant_id: t,
            reason: "Accept".into(),
        },
    );
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    let mut temp = c.clone();
    temp.engine = PREVIOUS_ENGINE.into();
    temp.battle.as_mut().unwrap().combatants[0].pending_damage = Some(PendingDamage {
        sequence: q.sequence,
        target_id: q.target_id.clone(),
        move_name: q.move_name.clone(),
        critical: q.critical,
    });
    let with = rules::damage_pool(&temp, &a, q.sequence, 0, &catalog()).unwrap();
    assert_eq!(with.pain, 0);
    temp.battle.as_mut().unwrap().combatants[0].selected_ability = "Unreviewed ability".into();
    let without = rules::damage_pool(&temp, &a, q.sequence, 0, &catalog()).unwrap();
    assert_eq!(with.dice, without.dice + 2);
    assert_eq!(without.pain, 1);
}
#[test]
fn champion_inherits_master_skill_bonus_and_new_state_survives_storage() {
    let (mut c, a, _) = fixture("Tackle");
    let sheet_id = c.battle.as_ref().unwrap().combatants[0].sheet_id.clone();
    c.sheets.get_mut(&sheet_id).unwrap().rank = "Master".into();
    let master = rules::dice_count(&c, &core_accuracy(&a, "Tackle"), &catalog()).unwrap();
    c.sheets.get_mut(&sheet_id).unwrap().rank = "Champion".into();
    assert_eq!(
        master,
        rules::dice_count(&c, &core_accuracy(&a, "Tackle"), &catalog()).unwrap()
    );
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store.put_campaign(&c).unwrap();
    let restored = store
        .campaigns()
        .unwrap()
        .into_iter()
        .find(|v| v.id == c.id)
        .unwrap();
    assert_eq!(
        serde_json::to_value(c.battle).unwrap(),
        serde_json::to_value(restored.battle).unwrap()
    );
}

#[test]
fn held_turn_can_resume_and_then_returns_to_the_delayed_actor() {
    let (c, a, t) = fixture("Tackle");
    let c = hold(&c, &a);
    let c = decision(
        &c,
        "player",
        Action::TakeHeldTurn {
            combatant_id: a.clone(),
            reason: "Ally has finished; resume".into(),
        },
    );
    assert_eq!(
        c.battle.as_ref().unwrap().flow.as_ref().unwrap().deferred,
        Some(t.clone())
    );
    let c = hit(&c, &a, &t, "Tackle");
    let c = decision(
        &c,
        "other",
        Action::PassReaction {
            combatant_id: t.clone(),
            reason: "Pass".into(),
        },
    );
    let c = damage(&c, "player", 1);
    let f = c.battle.unwrap().flow.unwrap();
    assert_eq!(f.order[f.cursor], t);
    assert!(f.deferred.is_none());
}

#[test]
fn sleep_recovery_consumes_actions_and_flinch_spends_the_next_turn() {
    let (mut c, a, t) = fixture("Tackle");
    let sid = c.battle.as_ref().unwrap().combatants[0].sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .stats
        .insert("Insight".into(), 2);
    let c = condition(&c, &a, "Sleep", None);
    assert!(battle::transition(
        &c,
        "player",
        &Action::PassTurn {
            combatant_id: a.clone(),
            reason: "Skip sleep".into()
        },
        vec![],
        &catalog()
    )
    .is_err());
    let action = Action::StatusCheck {
        combatant_id: a.clone(),
        kind: "sleep".into(),
        reason: "Wake".into(),
    };
    let n = rules::dice_count(&c, &action, &catalog()).unwrap();
    let c = battle::transition(&c, "player", &action, vec![4; n as usize], &catalog())
        .unwrap()
        .0;
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].actions, 1);
    let c = hold(&c, &t);
    let c = battle::transition(&c, "player", &action, vec![4; n as usize], &catalog())
        .unwrap()
        .0;
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].actions, 2);
    assert_eq!(
        c.battle.as_ref().unwrap().combatants[0].runtime.statuses[0].progress,
        4
    );
    let c = battle::transition(&c, "player", &action, vec![4; n as usize], &catalog())
        .unwrap()
        .0;
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].actions, 3);
    assert!(c.battle.as_ref().unwrap().combatants[0]
        .runtime
        .statuses
        .is_empty());
    let (c, a, _) = fixture("Tackle");
    let c = condition(&c, &a, "Flinch", None);
    let c = hold(&c, &a);
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].actions, 1);
    assert!(c.battle.as_ref().unwrap().combatants[0]
        .runtime
        .statuses
        .is_empty());
    assert!(battle::transition(
        &c,
        "gm",
        &Action::SetStatus {
            combatant_id: a,
            kind: "Flinch".into(),
            detail: String::new(),
            rounds: None,
            remove: false,
            reason: "Twice".into()
        },
        vec![],
        &catalog()
    )
    .is_err());
}
#[test]
fn recharge_is_paid_next_round_and_does_not_spend_will() {
    let (c, a, t) = fixture("Hyper Beam");
    let c = hit(&c, &a, &t, "Hyper Beam");
    let c = decision(
        &c,
        "other",
        Action::PassReaction {
            combatant_id: t.clone(),
            reason: "Pass".into(),
        },
    );
    let c = damage(&c, "player", 1);
    let c = effect(&c, 1);
    let c = hold(&c, &t);
    let c = hold(&c, &a);
    let c = decision(&c, "gm", Action::NextRound);
    let c = decision(&c, "gm", Action::StartTurns);
    assert!(rules::dice_count(&c, &core_accuracy(&a, "Hyper Beam"), &catalog()).is_err());
    let c = hold(&c, &a);
    let p = &c.battle.as_ref().unwrap().combatants[0];
    assert_eq!(
        (p.actions, p.runtime.recharge_actions, p.runtime.will),
        (1, 0, 6)
    );
}
#[tokio::test]
async fn separate_host_gm_and_player_preserve_core_state_and_receipts() {
    let hd = tempfile::tempdir().unwrap();
    let gd = tempfile::tempdir().unwrap();
    let pd = tempfile::tempdir().unwrap();
    let mut store = Store::open(hd.path()).unwrap();
    let host_id = store.player_id.clone();
    let gs = Arc::new(Mutex::new(Store::open(gd.path()).unwrap()));
    let ps = Arc::new(Mutex::new(Store::open(pd.path()).unwrap()));
    let gm_id = gs.lock().unwrap().player_id.clone();
    let player = ps.lock().unwrap().player_id.clone();
    let (mut c, a, t) = fixture("Tackle");
    c.host_id = host_id;
    c.gm_id = gm_id.clone();
    c.battle.as_mut().unwrap().combatants[0].owner_id = player.clone();
    let sid = c.battle.as_ref().unwrap().combatants[0].sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().owner_id = player.clone();
    let cid = c.id.clone();
    store.put_campaign(&c).unwrap();
    let shared = Arc::new(Mutex::new(store));
    let cat = Arc::new(catalog());
    let notify: room::Notify = Arc::new(|_, _| {});
    let mut host = room::start(shared.clone(), cat.clone(), cid, 0, notify.clone())
        .await
        .unwrap();
    let request = JoinRequest {
        address: host.info.address.replace("<host-ip>", "127.0.0.1"),
        token: host.info.token.clone(),
    };
    let (gm_remote, _) = room::join(gs, cat.clone(), request.clone(), notify.clone())
        .await
        .unwrap();
    let (remote, _) = room::join(ps, cat, request, notify).await.unwrap();
    let status = Command {
        id: id(),
        expected_revision: 7,
        action: Action::SetStatus {
            combatant_id: a.clone(),
            kind: "Poison".into(),
            detail: String::new(),
            rounds: None,
            remove: false,
            reason: "GM ruling".into(),
        },
    };
    assert!(host.host.apply(&player, &status).is_err());
    let (tx, rx) = tokio::sync::oneshot::channel();
    gm_remote.sender.send((status, tx)).await.unwrap();
    let accepted = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(accepted.revision, 8);
    let spend = Command {
        id: id(),
        expected_revision: 8,
        action: Action::SpendWill {
            combatant_id: a.clone(),
            mode: "chance".into(),
            reason: "Player chooses accepted Will use".into(),
        },
    };
    let (tx, rx) = tokio::sync::oneshot::channel();
    remote.sender.send((spend.clone(), tx)).await.unwrap();
    let accepted = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(
        accepted.battle.as_ref().unwrap().combatants[0].runtime.will,
        5
    );
    assert_eq!(host.host.apply(&player, &spend).unwrap().revision, 9);
    let accuracy = Command {
        id: id(),
        expected_revision: 9,
        action: Action::Accuracy {
            combatant_id: a.clone(),
            target_id: Some(t),
            move_name: "Tackle".into(),
            pool_modifier: 0,
            success_modifier: 0,
            reason: "Player uses the new core engine".into(),
        },
    };
    let (tx, rx) = tokio::sync::oneshot::channel();
    remote.sender.send((accuracy.clone(), tx)).await.unwrap();
    let attack = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(attack.events.last().unwrap().dice.len(), 7);
    assert_eq!(attack.battle.as_ref().unwrap().combatants[0].actions, 1);
    let recovered = host
        .host
        .apply(
            &gm_id,
            &Command {
                id: id(),
                expected_revision: 10,
                action: Action::RollbackBattle {
                    event_sequence: 10,
                    reason: "Undo reaction window/failed action".into(),
                },
            },
        )
        .unwrap();
    assert_eq!(recovered.battle.as_ref().unwrap().combatants[0].actions, 0);
    assert_eq!(
        recovered.battle.as_ref().unwrap().combatants[0]
            .runtime
            .will,
        5
    );
    assert_eq!(host.host.apply(&player, &accuracy).unwrap().revision, 11);
    let rollback = Command {
        id: id(),
        expected_revision: 11,
        action: Action::RollbackBattle {
            event_sequence: 9,
            reason: "Review correction".into(),
        },
    };
    let rolled = host.host.apply(&gm_id, &rollback).unwrap();
    assert_eq!(
        rolled.battle.as_ref().unwrap().combatants[0].runtime.will,
        6
    );
    assert_eq!(host.host.apply(&player, &spend).unwrap().revision, 12);
    assert_eq!(
        shared.lock().unwrap().campaigns().unwrap()[0]
            .battle
            .as_ref()
            .unwrap()
            .combatants[0]
            .runtime
            .will,
        6
    );
    remote.task.abort();
    gm_remote.task.abort();
    let _ = host.host.shutdown.send(true);
    let _ = host.stop.take().unwrap().send(());
}

#[test]
fn chance_effects_trigger_only_on_six_and_agility_does_not_edit_the_sheet() {
    let (c, a, t) = fixture("Thunder Shock");
    let c = hit(&c, &a, &t, "Thunder Shock");
    let c = decision(
        &c,
        "other",
        Action::PassReaction {
            combatant_id: t.clone(),
            reason: "Pass".into(),
        },
    );
    let c = damage(&c, "player", 1);
    let failed = effect(&c, 4);
    assert!(failed.battle.as_ref().unwrap().combatants[1]
        .runtime
        .statuses
        .is_empty());
    let success = effect(&c, 6);
    let p = &success.battle.as_ref().unwrap().combatants[1];
    assert_eq!(p.runtime.statuses[0].kind, "Paralysis");
    let base = c.sheets[&p.sheet_id].stats["Dexterity"];
    assert_eq!(
        rules::effective_sheet(&success, p).unwrap().stats["Dexterity"],
        (base - 2).max(0)
    );
    let (c, a, _) = fixture("Agility");
    let sid = c.battle.as_ref().unwrap().combatants[0].sheet_id.clone();
    let base = c.sheets[&sid].stats["Dexterity"];
    let c = hit(&c, &a, &a, "Agility");
    let c = effect(&c, 1);
    let p = &c.battle.as_ref().unwrap().combatants[0];
    assert_eq!(
        rules::effective_sheet(&c, p).unwrap().stats["Dexterity"],
        (base + 2).min(10)
    );
    assert_eq!(c.sheets[&sid].stats["Dexterity"], base);
    assert_eq!(p.initiative, 12);
}

#[test]
fn zero_will_faints_at_scene_end_and_rollback_restores_sheet_resources() {
    let (mut c, _, _) = fixture("Tackle");
    let sid = c.battle.as_ref().unwrap().combatants[0].sheet_id.clone();
    let original = c.sheets[&sid].clone();
    c.battle.as_mut().unwrap().combatants[0].runtime.will = 0;
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].hp, 20);
    let ended = decision(&c, "gm", Action::EndBattle);
    assert_eq!(ended.sheets[&sid].hp, 0);
    assert_eq!(ended.sheets[&sid].workflow.as_ref().unwrap().will, 0);
    let rolled = decision(
        &ended,
        "gm",
        Action::RollbackBattle {
            event_sequence: c.revision + 1,
            reason: "Restore scene".into(),
        },
    );
    assert_eq!(rolled.sheets[&sid].hp, original.hp);
    assert_eq!(
        rolled.sheets[&sid].workflow.as_ref().unwrap().will,
        original.workflow.as_ref().unwrap().will
    );
    assert!(rolled.sheets[&sid].revision > ended.sheets[&sid].revision);
}
#[test]
fn old_battles_without_runtime_or_flow_remain_readable() {
    let (c, _, _) = fixture("Tackle");
    let mut json = serde_json::to_value(c.battle.unwrap()).unwrap();
    json.as_object_mut().unwrap().remove("flow");
    for p in json["combatants"].as_array_mut().unwrap() {
        p.as_object_mut().unwrap().remove("runtime");
    }
    let b: Battle = serde_json::from_value(json).unwrap();
    assert!(b.flow.is_none());
    assert!(b
        .combatants
        .iter()
        .all(|p| p.runtime.statuses.is_empty() && p.runtime.will == 0));
}

#[test]
fn will_corrections_require_gm_and_enforce_the_derived_cap() {
    let (c, a, _) = fixture("Tackle");
    let action = |will| Action::AdjustWill {
        combatant_id: a.clone(),
        will,
        reason: "Correct recorded battle resource".into(),
    };
    assert!(battle::transition(&c, "player", &action(2), vec![], &catalog()).is_err());
    assert!(battle::transition(&c, "gm", &action(999), vec![], &catalog()).is_err());
    let next = decision(&c, "gm", action(2));
    assert_eq!(next.battle.as_ref().unwrap().combatants[0].runtime.will, 2);
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].runtime.will, 6);
}

#[test]
fn reviewed_status_immunities_and_thick_fat_use_the_frozen_battle_ability() {
    let (mut c, a, t) = fixture("Confusion");
    c.battle.as_mut().unwrap().combatants[1].selected_ability = "Own Tempo".into();
    assert!(battle::transition(
        &c,
        "gm",
        &Action::SetStatus {
            combatant_id: t.clone(),
            kind: "Confusion".into(),
            detail: String::new(),
            rounds: None,
            remove: false,
            reason: "Test immunity".into()
        },
        vec![],
        &catalog()
    )
    .is_err());
    let c = hit(&c, &a, &t, "Confusion");
    let c = decision(
        &c,
        "other",
        Action::PassReaction {
            combatant_id: t,
            reason: "Pass".into(),
        },
    );
    let c = damage(&c, "player", 1);
    let c = effect(&c, 6);
    assert!(c.battle.as_ref().unwrap().combatants[1]
        .runtime
        .statuses
        .is_empty());
    let (mut c, a, t) = fixture("Ember");
    let sid = c.battle.as_ref().unwrap().combatants[1].sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some(
        catalog().find("species", "Name", "Bulbasaur").unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
    let c = hit(&c, &a, &t, "Ember");
    let c = decision(
        &c,
        "other",
        Action::PassReaction {
            combatant_id: t,
            reason: "Pass".into(),
        },
    );
    let normal = damage(&c, "player", 4);
    let mut protected = c.clone();
    protected.battle.as_mut().unwrap().combatants[1].selected_ability = "Thick Fat".into();
    let reduced = damage(&protected, "player", 4);
    assert_eq!(
        reduced.battle.as_ref().unwrap().combatants[1].hp,
        normal.battle.as_ref().unwrap().combatants[1].hp + 1
    );
}

#[test]
fn shield_first_attempt_is_free_then_consecutive_penalties_are_two_four_eight() {
    let (mut c, a, t) = fixture("Protect");
    for expected in [6, 4, 2] {
        c = hit(&c, &a, &a, "Protect");
        assert_eq!(
            c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].successes,
            expected
        );
        c = effect(&c, 1);
        c = hold(&c, &t);
        c = hold(&c, &a);
        c = decision(&c, "gm", Action::NextRound);
        c = decision(&c, "gm", Action::StartTurns);
    }
    let c = hit(&c, &a, &a, "Protect");
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .is_empty());
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].actions, 1);
    assert_eq!(
        c.battle.as_ref().unwrap().combatants[0]
            .runtime
            .shield_round,
        4
    );
}
