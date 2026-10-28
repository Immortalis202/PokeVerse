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
