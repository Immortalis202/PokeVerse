use super::targeting_tests::pass;
use super::*;

fn setup() -> (Campaign, String, String) {
    let (mut c, a, t) = super::special_tests::setup("Spite");
    let b = c.battle.as_mut().unwrap();
    b.flow.as_mut().unwrap().spite = true;
    let rt = &mut b.combatants[1].runtime;
    rt.pain_ignored = 2;
    rt.chance_round = b.round;
    rt.fate_round = b.round;
    rt.fate_pending = true;
    (c, a, t)
}
fn resolve(c: &Campaign) -> Action {
    Action::ResolveEffect {
        sequence: c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sequence,
        reason: "Resolve Spite card".into(),
    }
}
fn target(c: &Campaign) -> &CombatState {
    &c.battle.as_ref().unwrap().combatants[1].runtime
}

fn drain_setup(name: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = super::special_tests::setup(name);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .will_drains = true;
    super::sequencing_tests::loyal(&mut c, &t, 3);
    (c, a, t)
}
fn drain(c: &Campaign, a: &str, t: &str, name: &str) -> Campaign {
    let mut pending = pass(&hit(c, a, t, name));
    if name == "Eerie Spell" {
        pending = super::targeting_tests::roll(
            &pending,
            "gm",
            &super::targeting_tests::damage_command(&pending),
            1,
        );
    }
    decision(&pending, "gm", resolve(&pending))
}
fn check(t: &str) -> Action {
    Action::SpecialMove {
        command: crate::combat::special::SpecialCommand::GrudgeCheck {
            target_id: t.into(),
        },
        reason: "Recurring Grudge Loyalty".into(),
    }
}
fn end_phase(c: &Campaign) -> Campaign {
    let mut c = c.clone();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .order
        .clear();
    decision(&c, "gm", Action::NextRound)
}
#[test]
fn grudge_faints_user_drains_resets_and_recurs_with_increasing_difficulty() {
    let (mut c, a, t) = drain_setup("Grudge");
    let rt = &mut c.battle.as_mut().unwrap().combatants[1].runtime;
    rt.pain_ignored = 2;
    rt.fate_pending = true;
    let c = drain(&c, &a, &t, "Grudge");
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].hp, 0);
    assert_eq!(target(&c).will, 0);
    assert_eq!(target(&c).pain_ignored, 0);
    assert!(!target(&c).fate_pending);
    assert!(target(&c).chance_cancelled);
    assert!(rules::dice_count(&c, &check(&t), &catalog()).is_err());
    let mut c = end_phase(&c);
    for difficulty in 1..=3 {
        assert_eq!(
            c.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .special
                .grudges[&t]
                .difficulty,
            difficulty
        );
        let n = rules::dice_count(&c, &check(&t), &catalog()).unwrap();
        assert_eq!(n, 3);
        let before = c.clone();
        c = battle::transition(&c, "gm", &check(&t), vec![6; 3], &catalog())
            .unwrap()
            .0;
        assert!(c.battle.as_ref().unwrap().combatants[1].hp > 0);
        assert!(battle::transition(&c, "gm", &check(&t), vec![6; 3], &catalog()).is_err());
        let restored = decision(
            &c,
            "gm",
            Action::RollbackBattle {
                event_sequence: c.battle_undo.last().unwrap().event_sequence,
                reason: "Undo Grudge check".into(),
            },
        );
        assert_eq!(
            serde_json::to_value(
                restored
                    .battle
                    .as_ref()
                    .unwrap()
                    .flow
                    .as_ref()
                    .unwrap()
                    .special
                    .grudges
                    .clone()
            )
            .unwrap(),
            serde_json::to_value(
                before
                    .battle
                    .as_ref()
                    .unwrap()
                    .flow
                    .as_ref()
                    .unwrap()
                    .special
                    .grudges
                    .clone()
            )
            .unwrap()
        );
        c = decision(&c, "gm", Action::NextRound);
        c = end_phase(&c);
    }
    let c = battle::transition(&c, "gm", &check(&t), vec![6; 3], &catalog())
        .unwrap()
        .0;
    assert_eq!(c.battle.as_ref().unwrap().combatants[1].hp, 0);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .special
        .grudges
        .is_empty());
}
#[test]
fn grudge_survives_source_faint_and_switching_and_requires_target_owner() {
    let (c, a, t) = drain_setup("Grudge");
    let mut c = drain(&c, &a, &t, "Grudge");
    c.battle.as_mut().unwrap().combatants[1].deployed = false;
    let c = end_phase(&c);
    let c: Campaign = serde_json::from_value(serde_json::to_value(&c).unwrap()).unwrap();
    assert_eq!(rules::dice_count(&c, &check(&t), &catalog()).unwrap(), 3);
    assert!(battle::transition(&c, "player", &check(&t), vec![6; 3], &catalog()).is_err());
    let next = battle::transition(&c, "other", &check(&t), vec![1; 3], &catalog())
        .unwrap()
        .0;
    assert_eq!(next.battle.as_ref().unwrap().combatants[1].hp, 0);
}
#[test]
fn eerie_spell_removes_only_three_will_and_clamps_at_zero() {
    for remaining in [0, 1, 2, 3, 6] {
        let (mut c, a, t) = drain_setup("Eerie Spell");
        let rt = &mut c.battle.as_mut().unwrap().combatants[1].runtime;
        rt.will = remaining;
        rt.pain_ignored = 2;
        rt.fate_pending = true;
        let next = drain(&c, &a, &t, "Eerie Spell");
        assert_eq!(target(&next).will, (remaining - 3).max(0));
        assert_eq!(target(&next).pain_ignored, 2);
        assert!(target(&next).fate_pending);
        assert!(!target(&next).chance_cancelled);
        assert!(next
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .special
            .grudges
            .is_empty());
    }
}
#[test]
fn drains_historical_gates_and_grudge_missing_loyalty_are_explicit() {
    for name in ["Grudge", "Eerie Spell"] {
        let (mut c, a, t) = drain_setup(name);
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .will_drains = false;
        let mut c = pass(&hit(&c, &a, &t, name));
        if name == "Eerie Spell" {
            c = super::targeting_tests::roll(
                &c,
                "gm",
                &super::targeting_tests::damage_command(&c),
                1,
            );
        }
        assert_eq!(
            battle::transition(&c, "gm", &resolve(&c), vec![], &catalog())
                .unwrap_err()
                .code,
            "manual_rule"
        );
    }
    let (mut c, a, t) = drain_setup("Grudge");
    let sid = c.battle.as_ref().unwrap().combatants[1].sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().workflow = None;
    let c = pass(&hit(&c, &a, &t, "Grudge"));
    assert_eq!(
        battle::transition(&c, "gm", &resolve(&c), vec![], &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
    assert!(c.battle.as_ref().unwrap().combatants[0].hp > 0);
}

#[test]
fn eerie_spell_repeated_hits_are_independent_and_soundproof_and_dark_block_drain() {
    let (c, a, t) = drain_setup("Eerie Spell");
    let mut c = drain(&c, &a, &t, "Eerie Spell");
    assert_eq!(target(&c).will, 3);
    c = end_phase(&c);
    c = decision(&c, "gm", Action::StartTurns);
    let c = drain(&c, &a, &t, "Eerie Spell");
    assert_eq!(target(&c).will, 0);
    for ability in ["Soundproof", ""] {
        let (mut c, a, t) = drain_setup("Eerie Spell");
        c.battle.as_mut().unwrap().combatants[1].selected_ability = ability.into();
        if ability.is_empty() {
            let id = catalog().find("species", "Name", "Umbreon").unwrap()["AppId"]
                .as_str()
                .unwrap()
                .to_owned();
            let sid = c.battle.as_ref().unwrap().combatants[1].sheet_id.clone();
            c.sheets.get_mut(&sid).unwrap().species_id = Some(id);
        }
        let c = drain(&c, &a, &t, "Eerie Spell");
        assert_eq!(target(&c).will, 6);
    }
}
#[test]
fn grudge_pending_blocks_early_actions_and_faint_removes_it_without_round_double_tick() {
    let (c, a, t) = drain_setup("Grudge");
    let c = drain(&c, &a, &t, "Grudge");
    let c = end_phase(&c);
    let round = c.battle.as_ref().unwrap().round;
    let again = decision(&c, "gm", Action::NextRound);
    assert_eq!(again.battle.as_ref().unwrap().round, round);
    assert!(battle::transition(&c, "gm", &Action::StartTurns, vec![], &catalog()).is_err());
    let mut c = c;
    c.battle.as_mut().unwrap().combatants[1].hp = 0;
    crate::combat::will::lifecycle(&mut c);
    assert!(!crate::combat::will::pending(&c));
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .special
        .grudges
        .is_empty());
}
#[test]
fn spite_drains_and_clears_live_benefits_without_refunding_history() {
    for will in [0, 1, 6] {
        let (mut c, a, t) = setup();
        c.battle.as_mut().unwrap().combatants[1].runtime.will = will;
        let before = pass(&hit(&c, &a, &t, "Spite"));
        let next = decision(&before, "gm", resolve(&before));
        let rt = target(&next);
        assert_eq!(rt.will, will.min(1));
        assert_eq!(rt.pain_ignored, 0);
        assert!(!rt.fate_pending);
        assert!(rt.chance_cancelled);
        assert_eq!(rt.chance_round, target(&before).chance_round);
        assert_eq!(rt.fate_round, target(&before).fate_round);
        assert_eq!(rt.will_enabled, target(&before).will_enabled);
        assert_eq!(
            serde_json::to_value(&next.sheets).unwrap(),
            serde_json::to_value(&before.sheets).unwrap()
        );
        let saved: Campaign = serde_json::from_value(serde_json::to_value(&next).unwrap()).unwrap();
        assert!(target(&saved).chance_cancelled);
        assert!(battle::transition(&next, "gm", &resolve(&before), vec![], &catalog()).is_err());
        let restored = decision(
            &next,
            "gm",
            Action::RollbackBattle {
                event_sequence: next.battle_undo.last().unwrap().event_sequence,
                reason: "Undo Spite".into(),
            },
        );
        assert_eq!(
            serde_json::to_value(target(&restored)).unwrap(),
            serde_json::to_value(target(&before)).unwrap()
        );
    }
}
#[test]
fn spite_historical_gate_and_ownership_remain_enforced() {
    let (c, a, t) = setup();
    let before = pass(&hit(&c, &a, &t, "Spite"));
    assert!(battle::transition(&before, "other", &resolve(&before), vec![], &catalog()).is_err());
    let mut old = before.clone();
    old.battle.as_mut().unwrap().flow.as_mut().unwrap().spite = false;
    let err = battle::transition(&old, "gm", &resolve(&old), vec![], &catalog()).unwrap_err();
    assert_eq!(err.code, "manual_rule");
    let mut value = serde_json::to_value(&c).unwrap();
    value["battle"]["flow"]
        .as_object_mut()
        .unwrap()
        .remove("spite");
    let old: Campaign = serde_json::from_value(value).unwrap();
    assert!(!old.battle.unwrap().flow.unwrap().spite);
}
#[test]
fn spite_respects_aroma_veil() {
    let (mut c, a, t) = setup();
    c.battle.as_mut().unwrap().combatants[1].selected_ability = "Aroma Veil".into();
    let before = pass(&hit(&c, &a, &t, "Spite"));
    let next = decision(&before, "gm", resolve(&before));
    assert_eq!(target(&next).will, 6);
    assert_eq!(target(&next).pain_ignored, 2);
    assert!(!target(&next).chance_cancelled);
}

#[test]
fn spite_cancels_chance_dice_and_preserves_same_round_spending_limits() {
    let (c, a, t) = setup();
    let before = pass(&hit(&c, &a, &t, "Spite"));
    let mut next = decision(&before, "gm", resolve(&before));
    let f = next.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.cursor = f.order.iter().position(|id| id == &t).unwrap();
    f.turn += 1;
    let action = Action::Accuracy {
        combatant_id: t.clone(),
        target_id: Some(a),
        move_name: "Tackle".into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Check cancelled Chance".into(),
    };
    let cancelled = rules::dice_count(&next, &action, &catalog()).unwrap();
    next.battle.as_mut().unwrap().combatants[1]
        .runtime
        .chance_cancelled = false;
    assert_eq!(
        rules::dice_count(&next, &action, &catalog()).unwrap(),
        cancelled + 1
    );
    next.battle.as_mut().unwrap().combatants[1]
        .runtime
        .chance_cancelled = true;
    for mode in ["chance", "fate"] {
        assert!(battle::transition(
            &next,
            "gm",
            &Action::SpendWill {
                combatant_id: t.clone(),
                mode: mode.into(),
                reason: "No reset of spending history".into()
            },
            vec![],
            &catalog()
        )
        .is_err());
    }
}
#[test]
fn spite_decoy_blocks_the_entire_drain() {
    let (mut c, a, t) = setup();
    let state = &mut c.battle.as_mut().unwrap().flow.as_mut().unwrap().decoys;
    state.enabled = true;
    state.active.insert(
        t.clone(),
        crate::combat::decoys::Decoy {
            hp: 2,
            defenses: [2, 2],
            source: t.clone(),
            move_name: "Substitute".into(),
        },
    );
    let before = pass(&hit(&c, &a, &t, "Spite"));
    let next = decision(&before, "gm", resolve(&before));
    assert_eq!(target(&next).will, 6);
    assert!(!target(&next).chance_cancelled);
}
#[tokio::test]
async fn spite_room_retry_reconnect_persistence_and_rollback() {
    let hd = tempfile::tempdir().unwrap();
    let pd = tempfile::tempdir().unwrap();
    let hs = Arc::new(Mutex::new(Store::open(hd.path()).unwrap()));
    let ps = Arc::new(Mutex::new(Store::open(pd.path()).unwrap()));
    let host_id = hs.lock().unwrap().player_id.clone();
    let gm_id = ps.lock().unwrap().player_id.clone();
    let (c, a, t) = setup();
    let mut c = pass(&hit(&c, &a, &t, "Spite"));
    c.host_id = host_id;
    c.gm_id = gm_id.clone();
    hs.lock().unwrap().put_campaign(&c).unwrap();
    let cat = Arc::new(catalog());
    let notify: room::Notify = Arc::new(|_, _| {});
    let mut hosted = room::start(hs.clone(), cat.clone(), c.id.clone(), 0, notify.clone())
        .await
        .unwrap();
    let request = JoinRequest {
        address: hosted.info.address.replace("<host-ip>", "127.0.0.1"),
        token: hosted.info.token.clone(),
    };
    let (remote, _) = room::join(ps.clone(), cat.clone(), request.clone(), notify.clone())
        .await
        .unwrap();
    let cmd = Command {
        id: id(),
        expected_revision: c.revision,
        action: resolve(&c),
    };
    let (tx, rx) = tokio::sync::oneshot::channel();
    remote.sender.send((cmd.clone(), tx)).await.unwrap();
    let next = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(target(&next).will, 1);
    assert!(target(&next).chance_cancelled);
    assert_eq!(
        hosted.host.apply(&gm_id, &cmd).unwrap().revision,
        next.revision
    );
    remote.task.abort();
    let (replacement, reconnected) = room::join(ps, cat, request, notify).await.unwrap();
    assert_eq!(target(&reconnected).will, 1);
    assert!(target(&reconnected).chance_cancelled);
    let saved = Store::open(hd.path())
        .unwrap()
        .campaigns()
        .unwrap()
        .into_iter()
        .find(|s| s.id == c.id)
        .unwrap();
    assert_eq!(target(&saved).will, 1);
    assert!(target(&saved).chance_cancelled);
    let latest = hosted.host.snapshot().unwrap();
    let undo = Command {
        id: id(),
        expected_revision: latest.revision,
        action: Action::RollbackBattle {
            event_sequence: latest.battle_undo.last().unwrap().event_sequence,
            reason: "Undo Spite".into(),
        },
    };
    let restored = hosted.host.apply(&gm_id, &undo).unwrap();
    assert_eq!(target(&restored).will, 6);
    assert!(!target(&restored).chance_cancelled);
    replacement.task.abort();
    if let Some(stop) = hosted.stop.take() {
        let _ = stop.send(());
    }
}

#[tokio::test]
async fn grudge_check_room_retry_reconnect_persistence_and_rollback() {
    let hd = tempfile::tempdir().unwrap();
    let pd = tempfile::tempdir().unwrap();
    let hs = Arc::new(Mutex::new(Store::open(hd.path()).unwrap()));
    let ps = Arc::new(Mutex::new(Store::open(pd.path()).unwrap()));
    let host_id = hs.lock().unwrap().player_id.clone();
    let gm_id = ps.lock().unwrap().player_id.clone();
    let (mut c, a, t) = drain_setup("Grudge");
    super::sequencing_tests::loyal(&mut c, &t, 0);
    let c = drain(&c, &a, &t, "Grudge");
    let mut c = end_phase(&c);
    c.host_id = host_id;
    c.gm_id = gm_id.clone();
    hs.lock().unwrap().put_campaign(&c).unwrap();
    let cat = Arc::new(catalog());
    let notify: room::Notify = Arc::new(|_, _| {});
    let mut hosted = room::start(hs.clone(), cat.clone(), c.id.clone(), 0, notify.clone())
        .await
        .unwrap();
    let request = JoinRequest {
        address: hosted.info.address.replace("<host-ip>", "127.0.0.1"),
        token: hosted.info.token.clone(),
    };
    let (remote, _) = room::join(ps.clone(), cat.clone(), request.clone(), notify.clone())
        .await
        .unwrap();
    let cmd = Command {
        id: id(),
        expected_revision: c.revision,
        action: check(&t),
    };
    let (tx, rx) = tokio::sync::oneshot::channel();
    remote.sender.send((cmd.clone(), tx)).await.unwrap();
    let next = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(next.battle.as_ref().unwrap().combatants[1].hp, 0);
    assert!(target(&next).chance_cancelled);
    assert_eq!(
        hosted.host.apply(&gm_id, &cmd).unwrap().revision,
        next.revision
    );
    remote.task.abort();
    let (replacement, reconnected) = room::join(ps, cat, request, notify).await.unwrap();
    assert_eq!(reconnected.battle.as_ref().unwrap().combatants[1].hp, 0);
    assert!(target(&reconnected).chance_cancelled);
    let saved = Store::open(hd.path())
        .unwrap()
        .campaigns()
        .unwrap()
        .into_iter()
        .find(|s| s.id == c.id)
        .unwrap();
    assert_eq!(saved.battle.as_ref().unwrap().combatants[1].hp, 0);
    assert!(target(&saved).chance_cancelled);
    let latest = hosted.host.snapshot().unwrap();
    let undo = Command {
        id: id(),
        expected_revision: latest.revision,
        action: Action::RollbackBattle {
            event_sequence: latest.battle_undo.last().unwrap().event_sequence,
            reason: "Undo Spite".into(),
        },
    };
    let restored = hosted.host.apply(&gm_id, &undo).unwrap();
    assert!(restored.battle.as_ref().unwrap().combatants[1].hp > 0);
    assert!(crate::combat::will::pending(&restored));
    replacement.task.abort();
    if let Some(stop) = hosted.stop.take() {
        let _ = stop.send(());
    }
}
