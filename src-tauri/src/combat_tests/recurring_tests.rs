use super::targeting_tests::{damage_command, pass, roll};
use super::*;
use crate::combat::{recurring, Status};

fn pm<'a>(c: &'a mut Campaign, id: &str) -> &'a mut Combatant {
    c.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == id)
        .unwrap()
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
fn setup(name: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = special_tests::setup(name);
    let round = c.battle.as_ref().unwrap().round;
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.compound.enabled = true;
    f.recurring.enabled = true;
    f.compound.entered.insert(a.clone(), round);
    f.compound.environments = vec!["City/Building".into()];
    f.status_escalation = true;
    f.reviewed_rules = true;
    f.interaction_rules = true;
    for id in [&a, &t] {
        pm(&mut c, id).hp = 100;
        pm(&mut c, id).max_hp = 100;
    }
    species(&mut c, &t, "Shuckle");
    (c, a, t)
}
fn species(c: &mut Campaign, id: &str, name: &str) {
    let sid = p(c, id).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some(
        catalog().find("species", "Name", name).unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
}
fn q(c: &Campaign) -> crate::combat::QueuedAction {
    c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].clone()
}
fn effect(c: &Campaign) -> Action {
    Action::ResolveEffect {
        sequence: q(c).sequence,
        reason: "Reviewed compound effect".into(),
    }
}
fn pending(c: &Campaign, a: &str, t: &str, name: &str) -> Campaign {
    let mut c = hit(c, a, t, name);
    if c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
        c = pass(&c);
    }
    if catalog().find("moves", "Name", name).unwrap()["Category"] != "Support" {
        c = roll(&c, "gm", &damage_command(&c), 1);
    }
    c
}
fn resolve(c: &Campaign, faces: Vec<u8>) -> Campaign {
    battle::transition(c, "gm", &effect(c), faces, &catalog())
        .unwrap()
        .0
}
fn has(c: &Campaign, id: &str, kind: &str) -> bool {
    p(c, id).runtime.statuses.iter().any(|s| s.kind == kind)
}
fn status(c: &mut Campaign, id: &str, kind: &str) {
    pm(c, id).runtime.statuses.push(Status {
        kind: kind.into(),
        detail: "Fixture".into(),
        rounds: None,
        progress: 0,
        elapsed_rounds: 0,
    });
}

fn rc(command: recurring::RecurringCommand) -> Action {
    Action::RecurringMove {
        command,
        reason: "Reviewed recurring decision".into(),
    }
}
fn next(c: &Campaign) -> Campaign {
    let mut c = c.clone();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .order
        .clear();
    battle::transition(&c, "gm", &Action::NextRound, vec![], &catalog())
        .unwrap()
        .0
}
#[test]
fn recurring_yawn_waits_until_next_round_and_departure_cancels() {
    let (c, a, t) = setup("Yawn");
    let c = pending(&c, &a, &t, "Yawn");
    let mut c = resolve(&c, vec![]);
    assert!(!has(&c, &t, "Sleep"));
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .recurring
            .yawn[&t]
            .due_round,
        2
    );
    let awake = next(&c);
    assert!(has(&awake, &t, "Sleep"));
    pm(&mut c, &t).deployed = false;
    recurring::lifecycle(&mut c).unwrap();
    pm(&mut c, &t).deployed = true;
    assert!(!has(&next(&c), &t, "Sleep"));
}
#[test]
fn recurring_perish_waits_three_rounds_and_cancels_on_departure() {
    let (c, a, t) = setup("Perish Song");
    let c = pending(&c, &a, &t, "Perish Song");
    let mut c = resolve(&c, vec![]);
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .recurring
            .perish[&t]
            .due_round,
        4
    );
    c = next(&c);
    c = next(&c);
    c = next(&c);
    assert_eq!(p(&c, &t).hp, 100);
    c = next(&c);
    assert_eq!(p(&c, &t).hp, 0);
    let (c, a, t) = setup("Perish Song");
    let c = pending(&c, &a, &t, "Perish Song");
    let mut c = resolve(&c, vec![]);
    pm(&mut c, &t).deployed = false;
    recurring::lifecycle(&mut c).unwrap();
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .recurring
        .perish
        .is_empty());
}
#[test]
fn recurring_nightmare_damages_only_failed_sleep_attempts_and_clears_on_wake() {
    let (mut c, a, t) = setup("Nightmare");
    status(&mut c, &t, "Sleep");
    let c = pending(&c, &a, &t, "Nightmare");
    let mut c = resolve(&c, vec![]);
    recurring::sleep_attempt(&mut c, &t).unwrap();
    assert_eq!(p(&c, &t).hp, 99);
    recurring::end_round(&mut c).unwrap();
    assert_eq!(p(&c, &t).hp, 99);
    pm(&mut c, &t).runtime.statuses.clear();
    recurring::sleep_attempt(&mut c, &t).unwrap();
    assert_eq!(p(&c, &t).hp, 99);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .recurring
        .nightmares
        .is_empty());
}
#[test]
fn recurring_malignant_chain_rolls_loyalty_changes_side_next_round_and_cure_restores() {
    let (mut c, a, t) = setup("Malignant Chain");
    super::sequencing_tests::loyal(&mut c, &t, 3);
    let owner = p(&c, &t).owner_id.clone();
    let side = p(&c, &t).side.clone();
    let allied = p(&c, &a).side.clone();
    let c = pending(&c, &a, &t, "Malignant Chain");
    assert_eq!(rules::dice_count(&c, &effect(&c), &catalog()).unwrap(), 5);
    let c = resolve(&c, vec![6; 5]);
    assert!(has(&c, &t, "Badly Poisoned"));
    let c = next(&c);
    assert_eq!(c.battle.as_ref().unwrap().round, 1);
    let check = rc(recurring::RecurringCommand::Loyalty {
        combatant_id: t.clone(),
    });
    assert_eq!(rules::dice_count(&c, &check, &catalog()).unwrap(), 3);
    let c = battle::transition(&c, "gm", &check, vec![1; 3], &catalog())
        .unwrap()
        .0;
    assert_eq!(p(&c, &t).side, side);
    let mut c = next(&c);
    assert_eq!(p(&c, &t).side, allied);
    assert_eq!(p(&c, &t).owner_id, owner);
    pm(&mut c, &t).runtime.statuses.clear();
    recurring::lifecycle(&mut c).unwrap();
    assert_eq!(p(&c, &t).side, side);
}
#[test]
fn recurring_chain_success_and_failed_chance_never_change_allegiance() {
    for faces in [vec![1; 5], vec![6; 5]] {
        let (mut c, a, t) = setup("Malignant Chain");
        super::sequencing_tests::loyal(&mut c, &t, 3);
        let side = p(&c, &t).side.clone();
        let c = pending(&c, &a, &t, "Malignant Chain");
        let c = resolve(&c, faces.clone());
        let c = next(&c);
        let c = if faces[0] == 6 {
            battle::transition(
                &c,
                "gm",
                &rc(recurring::RecurringCommand::Loyalty {
                    combatant_id: t.clone(),
                }),
                vec![6; 3],
                &catalog(),
            )
            .unwrap()
            .0
        } else {
            c
        };
        let c = next(&c);
        assert_eq!(p(&c, &t).side, side);
        assert!(c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .recurring
            .malignant
            .is_empty());
    }
}
#[test]
fn recurring_snore_requires_sleep_and_leaves_wake_progress_unchanged() {
    let (mut c, a, t) = setup("Snore");
    assert!(rules::dice_count(
        &c,
        &Action::Accuracy {
            combatant_id: a.clone(),
            target_id: Some(t.clone()),
            move_name: "Snore".into(),
            pool_modifier: 0,
            success_modifier: 0,
            reason: "Test".into()
        },
        &catalog()
    )
    .is_err());
    status(&mut c, &a, "Sleep");
    let c = pending(&c, &a, &t, "Snore");
    let n = rules::dice_count(&c, &effect(&c), &catalog()).unwrap();
    assert_eq!(n, 3);
    let c = resolve(&c, vec![6; 3]);
    assert!(has(&c, &t, "Flinch"));
    assert_eq!(
        p(&c, &a)
            .runtime
            .statuses
            .iter()
            .find(|s| s.kind == "Sleep")
            .unwrap()
            .progress,
        0
    );
}
#[test]
fn recurring_sleep_talk_commits_uniform_draw_before_spending_action() {
    let (mut c, a, _t) = setup("Sleep Talk");
    status(&mut c, &a, "Sleep");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().moves = vec!["Sleep Talk".into(), "Tackle".into()];
    let draw = rc(recurring::RecurringCommand::Draw {
        combatant_id: a.clone(),
    });
    assert_eq!(rules::dice_count(&c, &draw, &catalog()).unwrap(), 6);
    let c = battle::transition(&c, "gm", &draw, vec![1, 1, 2, 1, 1, 1], &catalog())
        .unwrap()
        .0;
    assert_eq!(p(&c, &a).actions, 0);
    assert!(rules::dice_count(&c, &draw, &catalog()).is_err());
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .recurring
            .pending_talk
            .as_ref()
            .unwrap()
            .move_name,
        "Tackle"
    );
    let copy = rc(recurring::RecurringCommand::UseCopy {
        combatant_id: a.clone(),
    });
    let n = rules::dice_count(&c, &copy, &catalog()).unwrap();
    let c = battle::transition(&c, "gm", &copy, vec![6; n as usize], &catalog())
        .unwrap()
        .0;
    assert_eq!(p(&c, &a).actions, 1);
    assert!(has(&c, &a, "Sleep"));
    assert_eq!(q(&c).move_name, "Tackle");
}
#[test]
fn recurring_sleep_talk_self_draw_is_retained_until_gm_consumes_attempt() {
    let (mut c, a, _) = setup("Sleep Talk");
    status(&mut c, &a, "Sleep");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().moves = vec!["Sleep Talk".into()];
    let draw = rc(recurring::RecurringCommand::Draw {
        combatant_id: a.clone(),
    });
    let c = battle::transition(&c, "gm", &draw, vec![1; 6], &catalog())
        .unwrap()
        .0;
    assert!(rules::dice_count(
        &c,
        &rc(recurring::RecurringCommand::UseCopy {
            combatant_id: a.clone()
        }),
        &catalog()
    )
    .is_err());
    let c = battle::transition(
        &c,
        "gm",
        &rc(recurring::RecurringCommand::Discard {
            combatant_id: a.clone(),
        }),
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    assert_eq!(p(&c, &a).actions, 1);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .recurring
        .pending_talk
        .is_none());
}
#[test]
fn recurring_random_selection_rejects_bias_instead_of_favoring_first_moves() {
    for size in 1..=216 {
        let mut counts = vec![0; size];
        for n in 0..216 {
            let faces = [
                (n / 36 + 1) as u8,
                ((n / 6) % 6 + 1) as u8,
                (n % 6 + 1) as u8,
            ];
            if let Some(i) = recurring::sample(&faces, size) {
                counts[i] += 1;
            }
        }
        assert!(counts.iter().all(|n| *n == 216 / size));
    }
}
#[test]
fn recurring_nightmare_uses_actual_cumulative_wake_checks() {
    let (mut c, a, t) = setup("Nightmare");
    status(&mut c, &t, "Sleep");
    let c = pending(&c, &a, &t, "Nightmare");
    let mut c = resolve(&c, vec![]);
    let check = Action::StatusCheck {
        combatant_id: t.clone(),
        kind: "sleep".into(),
        reason: "Attempt waking".into(),
    };
    for wake in [false, true] {
        let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
        f.order = vec![t.clone()];
        f.cursor = 0;
        f.started = true;
        let n = rules::dice_count(&c, &check, &catalog()).unwrap();
        if wake {
            pm(&mut c, &t)
                .runtime
                .statuses
                .iter_mut()
                .find(|s| s.kind == "Sleep")
                .unwrap()
                .progress = 4;
        }
        c = battle::transition(
            &c,
            "gm",
            &check,
            vec![if wake { 6 } else { 1 }; n as usize],
            &catalog(),
        )
        .unwrap()
        .0;
        assert_eq!(p(&c, &t).hp, 99);
        assert_eq!(has(&c, &t, "Sleep"), !wake);
    }
    assert!(!c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .recurring
        .nightmares
        .contains_key(&t));
}
#[test]
fn recurring_yawn_rechecks_immunity_at_delivery_and_source_departure_does_not_cancel() {
    for immune in [false, true] {
        let (c, a, t) = setup("Yawn");
        let c = pending(&c, &a, &t, "Yawn");
        let mut c = resolve(&c, vec![]);
        pm(&mut c, &a).deployed = false;
        if immune {
            pm(&mut c, &t).selected_ability = "Insomnia".into();
        }
        let c = next(&c);
        assert_eq!(has(&c, &t, "Sleep"), !immune);
    }
}
#[test]
fn recurring_chain_requires_reviewed_loyalty_atomically_and_can_cancel_before_activation() {
    let (mut c, a, t) = setup("Malignant Chain");
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().workflow = None;
    let c = pending(&c, &a, &t, "Malignant Chain");
    assert!(rules::dice_count(&c, &effect(&c), &catalog()).is_err());
    assert!(!has(&c, &t, "Badly Poisoned"));
    let (mut c, a, t) = setup("Malignant Chain");
    super::sequencing_tests::loyal(&mut c, &t, 3);
    let side = p(&c, &t).side.clone();
    let c = pending(&c, &a, &t, "Malignant Chain");
    let c = resolve(&c, vec![6; 5]);
    let c = next(&c);
    let mut c = battle::transition(
        &c,
        "gm",
        &rc(recurring::RecurringCommand::Loyalty {
            combatant_id: t.clone(),
        }),
        vec![1; 3],
        &catalog(),
    )
    .unwrap()
    .0;
    pm(&mut c, &t).runtime.statuses.clear();
    let c = next(&c);
    assert_eq!(p(&c, &t).side, side);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .recurring
        .malignant
        .is_empty());
}
#[test]
fn recurring_perish_indirect_immunity_and_nonstacking_are_preserved() {
    let (c, a, t) = setup("Perish Song");
    let c = pending(&c, &a, &t, "Perish Song");
    let mut c = resolve(&c, vec![]);
    pm(&mut c, &t).selected_ability = "Magic Guard".into();
    for _ in 0..4 {
        c = next(&c);
    }
    assert_eq!(p(&c, &t).hp, 100);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .recurring
        .perish
        .is_empty());
}
#[test]
fn recurring_sleep_talk_permissions_and_saved_selection_survive_roundtrip() {
    let (mut c, a, _) = setup("Sleep Talk");
    status(&mut c, &a, "Sleep");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().moves = vec!["Sleep Talk".into(), "Tackle".into()];
    let draw = rc(recurring::RecurringCommand::Draw {
        combatant_id: a.clone(),
    });
    assert!(battle::transition(&c, "intruder", &draw, vec![1; 6], &catalog()).is_err());
    let c = battle::transition(&c, "gm", &draw, vec![1, 1, 2, 1, 1, 1], &catalog())
        .unwrap()
        .0;
    let restored: Campaign = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
    assert_eq!(
        restored
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .recurring
            .pending_talk
            .as_ref()
            .unwrap()
            .move_name,
        "Tackle"
    );
    assert!(battle::transition(
        &restored,
        "intruder",
        &rc(recurring::RecurringCommand::Discard { combatant_id: a }),
        vec![],
        &catalog()
    )
    .is_err());
}
#[test]
fn recurring_sleep_talk_can_repeat_five_times_but_cannot_exceed_action_budget() {
    let (mut c, a, _) = setup("Sleep Talk");
    status(&mut c, &a, "Sleep");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().moves = vec!["Sleep Talk".into(), "Tackle".into()];
    let draw = rc(recurring::RecurringCommand::Draw {
        combatant_id: a.clone(),
    });
    let copy = rc(recurring::RecurringCommand::UseCopy {
        combatant_id: a.clone(),
    });
    for action in 1..=5 {
        let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
        f.order = vec![a.clone()];
        f.cursor = 0;
        f.started = true;
        c = battle::transition(&c, "gm", &draw, vec![1, 1, 2, 1, 1, 1], &catalog())
            .unwrap()
            .0;
        let n = rules::dice_count(&c, &copy, &catalog()).unwrap();
        c = battle::transition(&c, "gm", &copy, vec![1; n as usize], &catalog())
            .unwrap()
            .0;
        assert_eq!(p(&c, &a).actions, action);
        assert!(c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .queue
            .is_empty());
    }
    assert!(rules::dice_count(&c, &draw, &catalog()).is_err());
    assert_eq!(
        p(&c, &a)
            .runtime
            .statuses
            .iter()
            .find(|s| s.kind == "Sleep")
            .unwrap()
            .progress,
        0
    );
}
#[test]
fn recurring_sleep_talk_rollback_restores_the_committed_draw() {
    let (mut c, a, _) = setup("Sleep Talk");
    status(&mut c, &a, "Sleep");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().moves = vec!["Sleep Talk".into(), "Tackle".into()];
    let c = battle::transition(
        &c,
        "gm",
        &rc(recurring::RecurringCommand::Draw {
            combatant_id: a.clone(),
        }),
        vec![1, 1, 2, 1, 1, 1],
        &catalog(),
    )
    .unwrap()
    .0;
    let copy = rc(recurring::RecurringCommand::UseCopy {
        combatant_id: a.clone(),
    });
    let n = rules::dice_count(&c, &copy, &catalog()).unwrap();
    let c = battle::transition(&c, "gm", &copy, vec![1; n as usize], &catalog())
        .unwrap()
        .0;
    let undo = Action::RollbackBattle {
        event_sequence: c.battle_undo.last().unwrap().event_sequence,
        reason: "Restore drawn attempt".into(),
    };
    let c = battle::transition(&c, "gm", &undo, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(p(&c, &a).actions, 0);
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .recurring
            .pending_talk
            .as_ref()
            .unwrap()
            .move_name,
        "Tackle"
    );
}

#[test]
fn recurring_chain_withdrawal_retains_check_and_cure_restores_benched_side() {
    let (mut c, a, t) = setup("Malignant Chain");
    super::sequencing_tests::loyal(&mut c, &t, 3);
    let side = p(&c, &t).side.clone();
    let allied = p(&c, &a).side.clone();
    let c = pending(&c, &a, &t, "Malignant Chain");
    let mut c = resolve(&c, vec![6; 5]);
    pm(&mut c, &t).deployed = false;
    pm(&mut c, &t).owner_id = "benched-owner".into();
    let c = next(&c);
    let c = battle::transition(
        &c,
        "benched-owner",
        &rc(recurring::RecurringCommand::Loyalty {
            combatant_id: t.clone(),
        }),
        vec![1; 3],
        &catalog(),
    )
    .unwrap()
    .0;
    let mut c = next(&c);
    assert_eq!(p(&c, &t).side, allied);
    pm(&mut c, &t).runtime.statuses.clear();
    recurring::lifecycle(&mut c).unwrap();
    assert_eq!(p(&c, &t).side, side);
}
