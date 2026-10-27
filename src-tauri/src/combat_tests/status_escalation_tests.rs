use super::*;
use crate::combat::Status;

fn prepared() -> (Campaign, String, String) {
    let (mut c, a, t) = fixture("Tackle");
    let species = catalog().find("species", "Name", "Eevee").unwrap()["AppId"]
        .as_str()
        .unwrap()
        .to_string();
    for p in &mut c.battle.as_mut().unwrap().combatants {
        c.sheets.get_mut(&p.sheet_id).unwrap().species_id = Some(species.clone());
        p.selected_ability = "Run Away".into();
        p.hp = 100;
        p.max_hp = 100;
    }
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .status_escalation = true;
    (c, a, t)
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
fn tick(c: &Campaign, a: &str, t: &str) -> Campaign {
    let c = if c.battle.as_ref().unwrap().flow.as_ref().unwrap().started {
        c.clone()
    } else {
        decision(c, "gm", Action::StartTurns)
    };
    round(&c, a, t)
}
#[test]
fn status_escalation_uses_detailed_manual_pools_and_separate_condition_ages() {
    let (c, a, t) = prepared();
    let c = condition(
        &condition(&c, &a, "Burn 3", None),
        &t,
        "Badly Poisoned",
        None,
    );
    let c = tick(&c, &a, &t);
    assert_eq!((p(&c, &a).hp, p(&c, &t).hp), (97, 98));
    assert_eq!(p(&c, &a).runtime.statuses[0].elapsed_rounds, 1);
    let c = tick(&c, &a, &t);
    assert_eq!((p(&c, &a).hp, p(&c, &t).hp), (92, 94));
    let c: Campaign = serde_json::from_value(serde_json::to_value(&c).unwrap()).unwrap();
    let c = tick(&c, &a, &t);
    assert_eq!((p(&c, &a).hp, p(&c, &t).hp), (85, 88));
}
#[test]
fn status_escalation_reinfliction_keeps_age_and_cumulative_treatment() {
    let (c, a, t) = prepared();
    let mut c = condition(&c, &a, "Burn", None);
    pm(&mut c, &a).runtime.statuses[0].progress = 3;
    let c = condition(&c, &a, "Burn", None);
    assert_eq!(p(&c, &a).runtime.statuses[0].kind, "Burn 2");
    let c = condition(&c, &a, "Burn", None);
    let c = condition(&condition(&c, &t, "Poison", None), &t, "Poison", None);
    let c = tick(&c, &a, &t);
    let c = condition(&condition(&c, &a, "Burn", None), &t, "Poison", None);
    assert_eq!(p(&c, &a).runtime.statuses[0].progress, 3);
    assert_eq!(p(&c, &a).runtime.statuses[0].elapsed_rounds, 1);
    assert_eq!(p(&c, &t).runtime.statuses[0].elapsed_rounds, 1);
    let c = tick(&c, &a, &t);
    assert_eq!((p(&c, &a).hp, p(&c, &t).hp), (92, 94));
}
#[test]
fn status_escalation_cure_expiry_and_faint_clear_the_condition_clock() {
    let (c, a, t) = prepared();
    let c = condition(
        &condition(&c, &a, "Burn 3", Some(1)),
        &t,
        "Badly Poisoned",
        Some(1),
    );
    let c = tick(&c, &a, &t);
    assert!(p(&c, &a).runtime.statuses.is_empty());
    assert!(p(&c, &t).runtime.statuses.is_empty());
    let mut c = condition(
        &condition(&c, &a, "Burn 3", None),
        &t,
        "Badly Poisoned",
        None,
    );
    pm(&mut c, &a).hp = 3;
    pm(&mut c, &t).hp = 2;
    let c = tick(&c, &a, &t);
    assert_eq!((p(&c, &a).hp, p(&c, &t).hp), (0, 0));
    assert!(p(&c, &a).runtime.statuses.is_empty());
    assert!(p(&c, &t).runtime.statuses.is_empty());
}
#[test]
fn status_escalation_immunity_prevents_damage_without_rewinding_elapsed_rounds() {
    let (c, a, t) = prepared();
    let mut c = condition(
        &condition(&c, &a, "Burn 3", None),
        &t,
        "Badly Poisoned",
        None,
    );
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_rules = true;
    pm(&mut c, &a).selected_ability = "Magic Guard".into();
    pm(&mut c, &t).selected_ability = "Immunity".into();
    let mut c = tick(&c, &a, &t);
    assert_eq!((p(&c, &a).hp, p(&c, &t).hp), (100, 100));
    pm(&mut c, &a).selected_ability = "Run Away".into();
    pm(&mut c, &t).selected_ability = "Run Away".into();
    let c = tick(&c, &a, &t);
    assert_eq!((p(&c, &a).hp, p(&c, &t).hp), (95, 96));
}
#[test]
fn status_escalation_does_not_change_fixed_ailments_or_historical_adjudication() {
    let (c, a, t) = prepared();
    for (kind, damage) in [("Burn", 1), ("Burn 2", 2), ("Poison", 2)] {
        let c = condition(&c, &a, kind, None);
        let c = tick(&c, &a, &t);
        let c = tick(&c, &a, &t);
        assert_eq!(p(&c, &a).hp, 100 - 2 * damage);
        assert_eq!(p(&c, &a).runtime.statuses[0].elapsed_rounds, 0);
    }
    let mut c = condition(&c, &a, "Burn 3", None);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .status_escalation = false;
    let c = hold(&hold(&c, &a), &t);
    assert_eq!(
        battle::transition(&c, "gm", &Action::NextRound, vec![], &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
    let c = decision(
        &c,
        "gm",
        Action::AdjudicateStatus {
            combatant_id: a.clone(),
            damage: 4,
            reason: "Historical choice".into(),
        },
    );
    let c = decision(&c, "gm", Action::NextRound);
    assert_eq!(p(&c, &a).hp, 96);
    let mut json = serde_json::to_value(&c).unwrap();
    json["battle"]["flow"]
        .as_object_mut()
        .unwrap()
        .remove("status_escalation");
    json["battle"]["combatants"][0]["runtime"]["statuses"][0]
        .as_object_mut()
        .unwrap()
        .remove("elapsed_rounds");
    let loaded: Campaign = serde_json::from_value(json).unwrap();
    assert!(
        !loaded
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .status_escalation
    );
    assert_eq!(p(&loaded, &a).runtime.statuses[0].elapsed_rounds, 0);
}
#[test]
fn status_escalation_permissions_rollback_and_manual_double_damage_guard() {
    let (c, a, t) = prepared();
    let c = condition(&c, &a, "Burn 3", None);
    let before = hold(&hold(&c, &a), &t);
    for actor in ["player", "gm"] {
        assert!(battle::transition(
            &before,
            actor,
            &Action::AdjudicateStatus {
                combatant_id: a.clone(),
                damage: 3,
                reason: "Do not double apply".into()
            },
            vec![],
            &catalog()
        )
        .is_err());
    }
    assert!(battle::transition(&before, "player", &Action::NextRound, vec![], &catalog()).is_err());
    let next = decision(&before, "gm", Action::NextRound);
    let restored = decision(
        &next,
        "gm",
        Action::RollbackBattle {
            event_sequence: next.battle_undo.last().unwrap().event_sequence,
            reason: "Undo status tick".into(),
        },
    );
    assert_eq!(p(&restored, &a).hp, 100);
    assert_eq!(p(&restored, &a).runtime.statuses[0].elapsed_rounds, 0);
    let again = decision(&restored, "gm", Action::NextRound);
    assert_eq!(p(&again, &a).hp, 97);
}
#[test]
fn status_escalation_pool_saturates_and_direct_severe_burn_never_downgrades() {
    let (c, a, _) = prepared();
    let c = condition(&condition(&c, &a, "Burn", None), &a, "Burn 3", None);
    assert_eq!(p(&c, &a).runtime.statuses[0].kind, "Burn 3");
    let s = Status {
        kind: "Burn 3".into(),
        detail: String::new(),
        rounds: None,
        progress: 7,
        elapsed_rounds: u32::MAX,
    };
    assert_eq!(crate::combat::severe_status_damage(&s), i32::MAX);
}

#[test]
fn status_escalation_burn_treatment_uses_4_6_8_successes_and_cure_restarts_age() {
    for (kind, threshold) in [("Burn", 4), ("Burn 2", 6), ("Burn 3", 8)] {
        let (mut c, a, t) = prepared();
        let sid = p(&c, &a).sheet_id.clone();
        c.sheets
            .get_mut(&sid)
            .unwrap()
            .stats
            .insert("Dexterity".into(), 1);
        c.sheets
            .get_mut(&sid)
            .unwrap()
            .skills
            .insert("Athletic".into(), 0);
        let mut c = condition(&c, &a, kind, None);
        pm(&mut c, &a).runtime.statuses[0].progress = threshold - 1;
        pm(&mut c, &a).runtime.statuses[0].elapsed_rounds = 2;
        let action = Action::StatusCheck {
            combatant_id: a.clone(),
            kind: "burn".into(),
            reason: "Treat cumulative burn".into(),
        };
        let n = rules::dice_count(&c, &action, &catalog()).unwrap();
        assert_eq!(n, 1);
        let c = battle::transition(&c, "player", &action, vec![4], &catalog())
            .unwrap()
            .0;
        assert!(p(&c, &a).runtime.statuses.is_empty());
        let c = condition(&c, &a, "Burn 3", None);
        assert_eq!(p(&c, &a).runtime.statuses[0].elapsed_rounds, 0);
        let c = hold(&c, &t);
        let c = hold(&c, &a);
        let c = decision(&c, "gm", Action::NextRound);
        assert_eq!(p(&c, &a).hp, 97);
    }
}
#[test]
fn status_escalation_poison_heal_preserves_existing_healing_and_finite_cure() {
    let (c, a, t) = prepared();
    let mut c = condition(&c, &a, "Badly Poisoned", None);
    pm(&mut c, &a).hp = 50;
    pm(&mut c, &a).selected_ability = "Poison Heal".into();
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.ability_rules = true;
    f.extra.enabled = true;
    let c = tick(&c, &a, &t);
    assert_eq!(p(&c, &a).hp, 51);
    assert_eq!(p(&c, &a).runtime.statuses[0].elapsed_rounds, 1);
    let mut c = tick(&c, &a, &t);
    assert_eq!(p(&c, &a).hp, 52);
    let cured = tick(&c, &a, &t);
    assert_eq!(p(&cured, &a).hp, 53);
    assert!(p(&cured, &a).runtime.statuses.is_empty());
    pm(&mut c, &a).selected_ability = "Run Away".into();
    let c = tick(&c, &a, &t);
    assert_eq!(p(&c, &a).hp, 46);
}
#[tokio::test]
async fn status_escalation_room_retry_reconnect_persistence_and_rollback() {
    let hd = tempfile::tempdir().unwrap();
    let pd = tempfile::tempdir().unwrap();
    let hs = Arc::new(Mutex::new(Store::open(hd.path()).unwrap()));
    let ps = Arc::new(Mutex::new(Store::open(pd.path()).unwrap()));
    let host_id = hs.lock().unwrap().player_id.clone();
    let gm_id = ps.lock().unwrap().player_id.clone();
    let (c, a, t) = prepared();
    let c = condition(
        &condition(&c, &a, "Burn 3", None),
        &t,
        "Badly Poisoned",
        None,
    );
    let mut c = hold(&hold(&c, &a), &t);
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
        action: Action::NextRound,
    };
    let (tx, rx) = tokio::sync::oneshot::channel();
    remote.sender.send((cmd.clone(), tx)).await.unwrap();
    let next = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!((p(&next, &a).hp, p(&next, &t).hp), (97, 98));
    assert_eq!(
        hosted.host.apply(&gm_id, &cmd).unwrap().revision,
        next.revision
    );
    assert_eq!(
        p(&hosted.host.snapshot().unwrap(), &a).runtime.statuses[0].elapsed_rounds,
        1
    );
    remote.task.abort();
    let (replacement, reconnected) = room::join(ps, cat, request, notify).await.unwrap();
    assert_eq!(p(&reconnected, &a).runtime.statuses[0].elapsed_rounds, 1);
    let saved = Store::open(hd.path())
        .unwrap()
        .campaigns()
        .unwrap()
        .into_iter()
        .find(|s| s.id == c.id)
        .unwrap();
    assert_eq!(p(&saved, &t).hp, 98);
    assert_eq!(p(&saved, &t).runtime.statuses[0].elapsed_rounds, 1);
    let latest = hosted.host.snapshot().unwrap();
    let undo = Command {
        id: id(),
        expected_revision: latest.revision,
        action: Action::RollbackBattle {
            event_sequence: latest.battle_undo.last().unwrap().event_sequence,
            reason: "Undo escalation round".into(),
        },
    };
    let restored = hosted.host.apply(&gm_id, &undo).unwrap();
    assert_eq!((p(&restored, &a).hp, p(&restored, &t).hp), (100, 100));
    assert_eq!(p(&restored, &t).runtime.statuses[0].elapsed_rounds, 0);
    replacement.task.abort();
    if let Some(stop) = hosted.stop.take() {
        let _ = stop.send(());
    }
}
