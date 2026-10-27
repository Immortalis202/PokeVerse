use super::*;
fn prepared(name: &str, ua: &str, ta: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = setup(name, ua, ta);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .identity_moves = true;
    (c, a, t)
}
fn resolved(c: &Campaign, a: &str, t: &str, name: &str) -> Campaign {
    effect(&pass(&hit(c, a, t, name)))
}
fn assert_scene(c: &Campaign, id: &str, expected: &str) {
    let mut away = c.clone();
    away.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == id)
        .unwrap()
        .deployed = false;
    let mut away = refresh(&away);
    assert_eq!(p(&away, id).selected_ability, expected);
    away.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == id)
        .unwrap()
        .deployed = true;
    let away = refresh(&away);
    assert_eq!(p(&away, id).selected_ability, expected);
    let copy: Campaign = serde_json::from_value(serde_json::to_value(&away).unwrap()).unwrap();
    assert_eq!(p(&copy, id).selected_ability, expected);
}
#[test]
fn identity_entrainment_replaces_only_target_for_scene_and_rollback_restores_it() {
    let (c, a, t) = prepared("Entrainment", "Run Away", "Levitate");
    let sheets = serde_json::to_value(&c.sheets).unwrap();
    let before = pass(&hit(&c, &a, &t, "Entrainment"));
    let n = effect(&before);
    assert_eq!(p(&n, &t).selected_ability, "Run Away");
    assert_eq!(p(&n, &a).selected_ability, "Run Away");
    assert_scene(&n, &t, "Run Away");
    assert_eq!(serde_json::to_value(&n.sheets).unwrap(), sheets);
    let undo = Action::RollbackBattle {
        event_sequence: n.battle_undo.last().unwrap().event_sequence,
        reason: "Undo Entrainment".into(),
    };
    let restored = battle::transition(&n, "gm", &undo, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(p(&restored, &t).selected_ability, "Levitate");
    let mut ended = n.clone();
    ended.battle.as_mut().unwrap().active = false;
    changes::lifecycle(&n, &mut ended, &catalog()).unwrap();
    assert_eq!(p(&ended, &t).selected_ability, "Levitate");
}
#[test]
fn identity_entrainment_protects_unique_identities_and_does_not_pass_source_negation() {
    for (ua, ta) in [("Multitype", "Levitate"), ("Run Away", "Multitype")] {
        let (c, a, t) = prepared("Entrainment", ua, ta);
        let n = resolved(&c, &a, &t, "Entrainment");
        assert_eq!(p(&n, &t).selected_ability, ta);
    }
    let (mut c, a, t) = prepared("Entrainment", "Run Away", "Levitate");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_changes
        .negated
        .insert(a.clone());
    let c = refresh(&c);
    let n = resolved(&c, &a, &t, "Entrainment");
    assert_eq!(p(&n, &t).selected_ability, "Run Away");
    assert_eq!(p(&n, &a).selected_ability, "");
}

#[test]
fn identity_doodle_copies_only_user_and_retains_the_copy_for_scene() {
    let (mut c, a, t) = prepared("Doodle", "Run Away", "Levitate");
    let mut ally = p(&c, &a).clone();
    ally.id = "unaffected-ally".into();
    ally.selected_ability = "Pickup".into();
    let ally_id = ally.id.clone();
    c.battle.as_mut().unwrap().combatants.push(ally);
    let sheets = serde_json::to_value(&c.sheets).unwrap();
    let n = resolved(&c, &a, &t, "Doodle");
    assert_eq!(p(&n, &a).selected_ability, "Levitate");
    assert_eq!(p(&n, &t).selected_ability, "Levitate");
    assert_eq!(p(&n, &ally_id).selected_ability, "Pickup");
    assert_scene(&n, &a, "Levitate");
    assert_eq!(serde_json::to_value(&n.sheets).unwrap(), sheets);
    let mut ended = n.clone();
    ended.battle.as_mut().unwrap().active = false;
    changes::lifecycle(&n, &mut ended, &catalog()).unwrap();
    assert_eq!(p(&ended, &a).selected_ability, "Run Away");
}
#[test]
fn identity_doodle_copies_current_identity_without_copying_source_negation() {
    for (ua, ta) in [("Run Away", "Multitype"), ("Multitype", "Levitate")] {
        let (c, a, t) = prepared("Doodle", ua, ta);
        let n = resolved(&c, &a, &t, "Doodle");
        assert_eq!(p(&n, &a).selected_ability, ua);
    }
    let (mut c, a, t) = prepared("Doodle", "Run Away", "Levitate");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_changes
        .negated
        .insert(t.clone());
    let c = refresh(&c);
    let n = resolved(&c, &a, &t, "Doodle");
    assert_eq!(p(&n, &a).selected_ability, "Levitate");
    assert_eq!(p(&n, &t).selected_ability, "");
    let (mut c, a, t) = prepared("Doodle", "Run Away", "Levitate");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_changes
        .trace
        .insert(t.clone(), "Huge Power".into());
    let c = refresh(&c);
    let n = resolved(&c, &a, &t, "Doodle");
    assert_eq!(p(&n, &a).selected_ability, "Huge Power");
    assert_eq!(p(&refresh(&n), &a).selected_ability, "Huge Power");
}

fn core_history(c: &mut Campaign, a: &str, t: &str, damage: i32, round: u32) {
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .special
        .history
        .push(crate::combat::special::Hit {
            sequence: 0,
            source: t.into(),
            target: a.into(),
            name: "Tackle".into(),
            category: "Physical".into(),
            dice: 4,
            damage,
            round,
            turn: 0,
        });
}
fn core(c: &Campaign, a: &str, t: &str) -> Campaign {
    let c = pass(&hit(c, a, t, "Core Enforcer"));
    let cmd = super::super::targeting_tests::damage_command(&c);
    let c = super::super::targeting_tests::roll(&c, "gm", &cmd, 4);
    effect(&c)
}
#[test]
fn identity_core_enforcer_checks_actual_damage_from_that_target_this_round() {
    let (mut c, a, t) = prepared("Core Enforcer", "Run Away", "Levitate");
    {
        let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
        f.order = vec![t.clone(), a.clone()];
        f.cursor = 0;
        f.turn += 1;
    }
    let incoming = Action::Accuracy {
        combatant_id: t.clone(),
        target_id: Some(a.clone()),
        move_name: "Tackle".into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Actual incoming damage".into(),
    };
    let c = pass(&super::super::targeting_tests::roll(&c, "gm", &incoming, 4));
    let cmd = super::super::targeting_tests::damage_command(&c);
    let c = super::super::targeting_tests::roll(&c, "gm", &cmd, 4);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .special
        .history
        .iter()
        .any(|h| h.source == t && h.target == a && h.damage > 0));
    let n = core(&c, &a, &t);
    assert_eq!(p(&n, &t).selected_ability, "");
    assert_scene(&n, &t, "");
    let mut ended = n.clone();
    ended.battle.as_mut().unwrap().active = false;
    changes::lifecycle(&n, &mut ended, &catalog()).unwrap();
    assert_eq!(p(&ended, &t).selected_ability, "Levitate");
}
#[test]
fn identity_core_enforcer_does_not_confuse_actions_old_hits_or_other_attackers_with_qualifying_damage(
) {
    for mode in 0..4 {
        let (mut c, a, t) = prepared("Core Enforcer", "Run Away", "Levitate");
        let round = c.battle.as_ref().unwrap().round;
        match mode {
            0 => {}
            1 => core_history(&mut c, &a, &t, 0, round),
            2 => core_history(&mut c, &a, &t, 2, round.saturating_sub(1)),
            _ => core_history(&mut c, &a, "other-attacker", 2, round),
        }
        let hp = p(&c, &t).hp;
        let n = core(&c, &a, &t);
        assert_eq!(p(&n, &t).selected_ability, "Levitate");
        assert!(p(&n, &t).hp < hp);
    }
}
#[test]
fn identity_core_enforcer_preserves_unique_and_type_immunity_and_allows_unique_user() {
    let (mut c, a, t) = prepared("Core Enforcer", "Run Away", "Multitype");
    let round = c.battle.as_ref().unwrap().round;
    core_history(&mut c, &a, &t, 2, round);
    let n = core(&c, &a, &t);
    assert_eq!(p(&n, &t).selected_ability, "Multitype");
    let (mut c, a, t) = prepared("Core Enforcer", "Multitype", "Levitate");
    let round = c.battle.as_ref().unwrap().round;
    core_history(&mut c, &a, &t, 2, round);
    let n = core(&c, &a, &t);
    assert_eq!(p(&n, &t).selected_ability, "");
    let (mut c, a, t) = prepared("Core Enforcer", "Run Away", "Levitate");
    let round = c.battle.as_ref().unwrap().round;
    core_history(&mut c, &a, &t, 2, round);
    let change = Action::Battlefield {
        command: crate::combat::field::FieldCommand::Typing {
            combatant_id: t.clone(),
            mode: "replace".into(),
            types: vec!["Fairy".into()],
        },
        reason: "Fairy immunity fixture".into(),
    };
    let c = battle::transition(&c, "gm", &change, vec![], &catalog())
        .unwrap()
        .0;
    let n = core(&c, &a, &t);
    assert_eq!(p(&n, &t).selected_ability, "Levitate");
}

#[test]
fn identity_moves_keep_permissions_historical_flags_and_effect_rollback() {
    for name in ["Entrainment", "Doodle", "Core Enforcer"] {
        let (mut c, a, t) = prepared(name, "Run Away", "Levitate");
        if name == "Core Enforcer" {
            let round = c.battle.as_ref().unwrap().round;
            core_history(&mut c, &a, &t, 2, round);
        }
        let mut pending = pass(&hit(&c, &a, &t, name));
        if name == "Core Enforcer" {
            let cmd = super::super::targeting_tests::damage_command(&pending);
            pending = super::super::targeting_tests::roll(&pending, "gm", &cmd, 4);
        }
        let q = pending
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .queue[0]
            .clone();
        let action = Action::ResolveEffect {
            sequence: q.sequence,
            reason: "Reviewed identity effect".into(),
        };
        assert!(battle::transition(&pending, "outsider", &action, vec![], &catalog()).is_err());
        let mut historical = pending.clone();
        historical
            .battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .identity_moves = false;
        assert!(!changes::move_effect(&mut historical, &q, &catalog()).unwrap());
        let changed = effect(&pending);
        let undo = Action::RollbackBattle {
            event_sequence: changed.battle_undo.last().unwrap().event_sequence,
            reason: "Undo scene identity".into(),
        };
        let restored = battle::transition(&changed, "gm", &undo, vec![], &catalog())
            .unwrap()
            .0;
        assert_eq!(
            p(&restored, &a).selected_ability,
            p(&pending, &a).selected_ability
        );
        assert_eq!(
            p(&restored, &t).selected_ability,
            p(&pending, &t).selected_ability
        );
        let mut value = serde_json::to_value(&pending).unwrap();
        value["battle"]["flow"]
            .as_object_mut()
            .unwrap()
            .remove("identity_moves");
        let loaded: Campaign = serde_json::from_value(value).unwrap();
        assert!(!loaded.battle.unwrap().flow.unwrap().identity_moves);
    }
}

#[tokio::test]
async fn identity_moves_room_receipts_reconnect_and_rollback_preserve_scene_changes() {
    for name in ["Entrainment", "Doodle", "Core Enforcer"] {
        let hd = tempfile::tempdir().unwrap();
        let pd = tempfile::tempdir().unwrap();
        let hs = Arc::new(Mutex::new(Store::open(hd.path()).unwrap()));
        let ps = Arc::new(Mutex::new(Store::open(pd.path()).unwrap()));
        let host_id = hs.lock().unwrap().player_id.clone();
        let player_id = ps.lock().unwrap().player_id.clone();
        let (mut c, a, t) = prepared(name, "Run Away", "Levitate");
        if name == "Core Enforcer" {
            let round = c.battle.as_ref().unwrap().round;
            core_history(&mut c, &a, &t, 2, round);
        }
        let mut c = pass(&hit(&c, &a, &t, name));
        if name == "Core Enforcer" {
            let cmd = super::super::targeting_tests::damage_command(&c);
            c = super::super::targeting_tests::roll(&c, "gm", &cmd, 4);
        }
        c.host_id = host_id.clone();
        c.gm_id = host_id.clone();
        let sid = p(&c, &a).sheet_id.clone();
        c.sheets.get_mut(&sid).unwrap().owner_id = player_id.clone();
        c.battle
            .as_mut()
            .unwrap()
            .combatants
            .iter_mut()
            .find(|p| p.id == a)
            .unwrap()
            .owner_id = player_id.clone();
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
        async fn send(r: &room::Remote, cmd: Command) -> Campaign {
            let (tx, rx) = tokio::sync::oneshot::channel();
            r.sender.send((cmd, tx)).await.unwrap();
            tokio::time::timeout(std::time::Duration::from_secs(5), rx)
                .await
                .unwrap()
                .unwrap()
                .unwrap()
        }
        let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
        let command = Command {
            id: id(),
            expected_revision: c.revision,
            action: Action::ResolveEffect {
                sequence: q.sequence,
                reason: "Owner resolves reviewed identity move".into(),
            },
        };
        let next = send(&remote, command.clone()).await;
        assert_eq!(send(&remote, command).await.revision, next.revision);
        let expected = if name == "Core Enforcer" {
            ""
        } else if name == "Doodle" {
            "Levitate"
        } else {
            "Run Away"
        };
        let recipient = if name == "Doodle" { &a } else { &t };
        assert_eq!(p(&next, recipient).selected_ability, expected);
        remote.task.abort();
        let (replacement, reconnected) =
            room::join(ps.clone(), cat, request, notify).await.unwrap();
        assert_eq!(p(&reconnected, recipient).selected_ability, expected);
        let saved = Store::open(hd.path())
            .unwrap()
            .campaigns()
            .unwrap()
            .into_iter()
            .find(|s| s.id == c.id)
            .unwrap();
        assert_eq!(
            serde_json::to_value(crate::privacy::for_player(&saved, &player_id).battle).unwrap(),
            serde_json::to_value(reconnected.battle).unwrap()
        );
        let latest = hosted.host.snapshot().unwrap();
        let undo = Command {
            id: id(),
            expected_revision: latest.revision,
            action: Action::RollbackBattle {
                event_sequence: latest.battle_undo.last().unwrap().event_sequence,
                reason: "GM rolls back identity move".into(),
            },
        };
        let restored = hosted.host.apply(&host_id, &undo).unwrap();
        assert_eq!(
            p(&restored, &a).selected_ability,
            p(&c, &a).selected_ability
        );
        assert_eq!(
            p(&restored, &t).selected_ability,
            p(&c, &t).selected_ability
        );
        replacement.task.abort();
        if let Some(stop) = hosted.stop.take() {
            let _ = stop.send(());
        }
    }
}

#[test]
fn identity_core_enforcer_counts_actual_contact_retaliation_without_reusing_counter_pools() {
    let (c, a, t) = prepared("Core Enforcer", "Run Away", "Rough Skin");
    let c = pass(&hit(&c, &a, &t, "Tackle"));
    let command = super::super::targeting_tests::damage_command(&c);
    let c = super::super::targeting_tests::roll(&c, "gm", &command, 4);
    let event = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0]
        .clone();
    assert_eq!(event.kind, "retaliate");
    let action = Action::ResolveAbility {
        event_id: event.id,
        choice: "neutral".into(),
        reason: "Contact retaliation has an explicit source".into(),
    };
    let mut c = battle::transition(&c, "gm", &action, vec![], &catalog())
        .unwrap()
        .0;
    let h = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .special
        .history
        .last()
        .unwrap();
    assert_eq!(
        (&*h.source, &*h.target, &*h.category, h.damage),
        (&*t, &*a, "Indirect", 1)
    );
    {
        let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
        f.queue.clear();
        f.window = false;
        f.cursor = f.order.iter().position(|id| id == &a).unwrap();
        f.reacted.clear();
        f.turn += 1;
    }
    let n = core(&c, &a, &t);
    assert_eq!(p(&n, &t).selected_ability, "");
}
