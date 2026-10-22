use super::targeting_tests::{damage_command, pass, roll};
use super::*;
use crate::combat::{
    special,
    standalone::{self, StandaloneCommand, StandaloneState},
    stat_channels,
};
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
fn ability(c: &mut Campaign, id: &str, name: &str) {
    pm(c, id).selected_ability = name.into();
    let sid = p(c, id).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().ability = name.into();
}
fn setup(name: &str, ab: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = super::special_tests::setup(name);
    ability(&mut c, &a, ab);
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.standalone = StandaloneState {
        enabled: true,
        ..Default::default()
    };
    f.stat_channels.enabled = true;
    f.ability_changes.enabled = true;
    (c, a, t)
}
fn act(command: StandaloneCommand) -> Action {
    Action::Standalone {
        command,
        reason: "Reviewed printed standalone rule".into(),
    }
}
fn command(c: &Campaign, command: StandaloneCommand) -> Campaign {
    battle::transition(c, "gm", &act(command), vec![], &catalog())
        .unwrap()
        .0
}
fn profile(
    c: &Campaign,
    id: &str,
    gender: &str,
    weight: Option<f64>,
    metal: Option<f64>,
) -> Campaign {
    command(
        c,
        StandaloneCommand::Profile {
            combatant_id: id.into(),
            gender: gender.into(),
            preference: gender.into(),
            weight_kg: weight,
            metal_weight_kg: metal,
        },
    )
}
#[test]
fn standalone_metal_requires_gm_weights_exact_direction_and_suppression_restores_baseline() {
    let (c, a, t) = setup("Heavy Slam", "Heavy Metal");
    let before = c.sheets.clone();
    assert!(special::bonus(
        &c,
        p(&c, &a),
        p(&c, &t),
        &catalog().find("moves", "Name", "Heavy Slam").unwrap(),
        &catalog()
    )
    .is_err());
    let n = profile(&c, &a, "unknown", Some(100.), Some(300.));
    let n = profile(&n, &t, "unknown", Some(99.), None);
    assert_eq!(
        standalone::weight(&n, p(&n, &a), &catalog(), None).unwrap(),
        300.
    );
    for (name, expected) in [("Low Kick", 4), ("Grass Knot", 4)] {
        let m = catalog().find("moves", "Name", name).unwrap();
        assert_eq!(
            special::bonus(&n, p(&n, &t), p(&n, &a), &m, &catalog()).unwrap(),
            expected
        );
    }
    let m = catalog().find("moves", "Name", "Heavy Slam").unwrap();
    assert_eq!(
        special::bonus(&n, p(&n, &a), p(&n, &t), &m, &catalog()).unwrap(),
        4
    );
    let mut suppressed = n.clone();
    ability(&mut suppressed, &a, "");
    assert_eq!(
        standalone::weight(&suppressed, p(&suppressed, &a), &catalog(), None).unwrap(),
        100.
    );
    assert_eq!(n.sheets, before);
    assert!(battle::transition(
        &c,
        "player",
        &act(StandaloneCommand::Torrent { bonus: 2 }),
        vec![],
        &catalog()
    )
    .is_err());
    for value in [f64::NAN, -1., 0., 100001.] {
        let cmd = act(StandaloneCommand::Profile {
            combatant_id: a.clone(),
            gender: "unknown".into(),
            preference: "unknown".into(),
            weight_kg: Some(value),
            metal_weight_kg: None,
        });
        assert!(battle::transition(&c, "gm", &cmd, vec![], &catalog()).is_err());
    }
    let mut light = n.clone();
    ability(&mut light, &a, "Light Metal");
    assert!(standalone::weight(&light, p(&light, &a), &catalog(), None).is_err());
    let light = profile(&light, &a, "unknown", Some(100.), Some(25.));
    assert_eq!(
        standalone::weight(&light, p(&light, &a), &catalog(), None).unwrap(),
        25.
    );
}
#[test]
fn standalone_weight_thresholds_are_literal_fifty_kg_steps_capped_at_four() {
    let (c, a, t) = setup("Low Kick", "");
    for (kg, n) in [
        (49.99, 0),
        (50., 1),
        (99.99, 1),
        (100., 2),
        (200., 4),
        (999., 4),
    ] {
        let c = profile(&c, &t, "unknown", Some(kg), None);
        for name in ["Low Kick", "Grass Knot"] {
            assert_eq!(
                special::bonus(
                    &c,
                    p(&c, &a),
                    p(&c, &t),
                    &catalog().find("moves", "Name", name).unwrap(),
                    &catalog()
                )
                .unwrap(),
                n
            );
        }
    }
}
#[test]
fn standalone_rivalry_requires_wording_and_uses_individual_neutral_groups_and_caps() {
    let (c, a, t) = setup("Tackle", "Rivalry");
    assert!(standalone::rivalry(&c, p(&c, &a), p(&c, &t)).is_err());
    let c = command(
        &c,
        StandaloneCommand::RivalryBasis {
            basis: "gender".into(),
        },
    );
    let c = profile(&c, &a, "male", None, None);
    let same = profile(&c, &t, "male", None, None);
    assert_eq!(
        standalone::rivalry(&same, p(&same, &a), p(&same, &t)).unwrap(),
        1
    );
    let opposite = profile(&c, &t, "female", None, None);
    assert_eq!(
        standalone::rivalry(&opposite, p(&opposite, &a), p(&opposite, &t)).unwrap(),
        -1
    );
    let neutral = profile(&c, &t, "unknown", None, None);
    assert_eq!(
        standalone::rivalry(&neutral, p(&neutral, &a), p(&neutral, &t)).unwrap(),
        0
    );
    assert_eq!(
        standalone::strength_delta(&same, p(&same, &a), p(&same, &t), "Strength", 10, None)
            .unwrap(),
        0
    );
    assert_eq!(
        standalone::strength_delta(
            &opposite,
            p(&opposite, &a),
            p(&opposite, &t),
            "Strength",
            1,
            None
        )
        .unwrap(),
        0
    );
    let preferred = command(
        &same,
        StandaloneCommand::RivalryBasis {
            basis: "preference".into(),
        },
    );
    assert_eq!(
        standalone::rivalry(&preferred, p(&preferred, &a), p(&preferred, &t)).unwrap(),
        1
    );
}
#[test]
fn standalone_rivalry_changes_host_accuracy_and_damage_without_mutating_sheets() {
    let (mut c, a, t) = setup("Tackle", "Rivalry");
    pm(&mut c, &a).stat_overrides.insert("Dexterity".into(), 2);
    let c = command(
        &c,
        StandaloneCommand::RivalryBasis {
            basis: "gender".into(),
        },
    );
    let c = profile(&c, &a, "male", None, None);
    let same = profile(&c, &t, "male", None, None);
    let different = profile(&c, &t, "female", None, None);
    let mut attack = core_accuracy(&a, "Tackle");
    if let Action::Accuracy { target_id, .. } = &mut attack {
        *target_id = Some(t.clone());
    }
    assert_eq!(
        rules::dice_count(&same, &attack, &catalog()).unwrap(),
        rules::dice_count(&different, &attack, &catalog()).unwrap() + 2
    );
    let n = roll(&same, "player", &attack, 4);
    let n = pass(&n);
    let n = roll(&n, "player", &damage_command(&n), 4);
    assert_eq!(n.sheets, same.sheets);
}
#[test]
fn standalone_torrent_choice_is_explicit_thresholded_and_exempts_water_pain() {
    let (mut c, a, t) = setup("Water Gun", "Torrent");
    pm(&mut c, &a).hp = 10;
    let m = catalog().find("moves", "Name", "Water Gun").unwrap();
    assert!(standalone::torrent(&c, p(&c, &a), &m, 10, None).is_err());
    let one = command(&c, StandaloneCommand::Torrent { bonus: 1 });
    let two = command(&c, StandaloneCommand::Torrent { bonus: 2 });
    assert_eq!(
        standalone::torrent(&one, p(&one, &a), &m, 10, None).unwrap(),
        1
    );
    assert_eq!(
        standalone::torrent(&two, p(&two, &a), &m, 10, None).unwrap(),
        2
    );
    assert_eq!(
        standalone::torrent(&two, p(&two, &a), &m, 11, None).unwrap(),
        0
    );
    assert_eq!(
        standalone::torrent(&two, p(&two, &a), &m, 10, Some(1)).unwrap(),
        1
    );
    let mut attack = core_accuracy(&a, "Water Gun");
    if let Action::Accuracy { target_id, .. } = &mut attack {
        *target_id = Some(t.clone());
    }
    let wounded = roll(&one, "player", &attack, 4);
    let mut healthy = one.clone();
    pm(&mut healthy, &a).hp = 20;
    let healthy = roll(&healthy, "player", &attack, 4);
    assert_eq!(
        wounded
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .queue[0]
            .successes,
        healthy
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .queue[0]
            .successes
    );
    assert_eq!(
        standalone::torrent(&two, p(&two, &a), &m, 10, Some(0)).unwrap(),
        0
    );
    assert_eq!(
        standalone::torrent(
            &c,
            p(&c, &a),
            &serde_json::json!({"Type":"Water","Category":"Support"}),
            10,
            None
        )
        .unwrap(),
        0
    );
}
fn moody() -> (Campaign, String) {
    let (mut c, a, _) = setup("Tackle", "Moody");
    standalone::end_round(&mut c).unwrap();
    (c, a)
}
#[test]
fn standalone_moody_all_twenty_distinct_pairs_are_unbiased_and_rejections_persist() {
    let (c, a) = moody();
    let e = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0]
        .clone();
    let mut pairs = std::collections::BTreeSet::new();
    let mut rejected = 0;
    for d1 in 1..=6 {
        for d2 in 1..=6 {
            let mut n = c.clone();
            let done = standalone::resolve_moody(&mut n, &e, "roll", &[d1, d2]).unwrap();
            if !done {
                rejected += 1;
                assert!(n
                    .battle
                    .as_ref()
                    .unwrap()
                    .flow
                    .as_ref()
                    .unwrap()
                    .stat_channels
                    .grants
                    .is_empty());
                continue;
            }
            let g = &n
                .battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .stat_channels
                .grants;
            assert_eq!(g.len(), 2);
            assert_ne!(g[0].key, g[1].key);
            assert_eq!(g[0].negative, -1);
            assert_eq!(g[1].positive, 1);
            pairs.insert((g[0].key.clone(), g[1].key.clone()));
        }
    }
    assert_eq!(pairs.len(), 20);
    assert_eq!(rejected, 16);
    let action = Action::ResolveAbility {
        event_id: e.id.clone(),
        choice: "roll".into(),
        reason: "Unbiased host Moody choice".into(),
    };
    let rejected = battle::transition(&c, "player", &action, vec![6, 6], &catalog())
        .unwrap()
        .0;
    assert_eq!(
        rejected
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events[0]
            .id,
        e.id
    );
    let accepted = battle::transition(&rejected, "player", &action, vec![1, 1], &catalog())
        .unwrap()
        .0;
    assert!(accepted
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events
        .is_empty());
    assert_eq!(accepted.sheets, c.sheets);
    assert!(p(&accepted, &a).actions == p(&c, &a).actions);
}
#[test]
fn standalone_moody_resets_only_its_own_grants_and_round_end_is_idempotent() {
    let (mut c, a) = moody();
    let e = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0]
        .clone();
    standalone::resolve_moody(&mut c, &e, "roll", &[1, 1]).unwrap();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_events
        .clear();
    c.battle.as_mut().unwrap().round += 1;
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .stat_channels
        .grants
        .push(stat_channels::StatGrant {
            source: a.clone(),
            target: a.clone(),
            cause: "Other".into(),
            key: "Strength".into(),
            channel: "ability".into(),
            positive: 2,
            negative: 0,
        });
    standalone::end_round(&mut c).unwrap();
    standalone::end_round(&mut c).unwrap();
    let f = c.battle.as_ref().unwrap().flow.as_ref().unwrap();
    assert_eq!(f.ability_events.len(), 1);
    assert_eq!(f.stat_channels.grants.len(), 1);
    assert_eq!(f.stat_channels.grants[0].cause, "Other");
}
#[test]
fn standalone_perish_body_contact_arms_once_killing_hit_and_departure_cancels() {
    let (mut c, a, t) = setup("Tackle", "");
    ability(&mut c, &t, "Perish Body");
    pm(&mut c, &t).hp = 1;
    let mut attack = core_accuracy(&a, "Tackle");
    if let Action::Accuracy { target_id, .. } = &mut attack {
        *target_id = Some(t.clone());
    }
    let c = roll(&c, "player", &attack, 6);
    let c = pass(&c);
    let mut c = roll(&c, "player", &damage_command(&c), 6);
    assert_eq!(p(&c, &t).hp, 0);
    let due = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .standalone
        .perish[&a]
        .due_round;
    assert_eq!(due, 4);
    for round in 1..4 {
        c.battle.as_mut().unwrap().round = round;
        standalone::end_round(&mut c).unwrap();
        assert!(p(&c, &a).hp > 0);
    }
    c.battle.as_mut().unwrap().round = 4;
    standalone::end_round(&mut c).unwrap();
    assert_eq!(p(&c, &a).hp, 0);
    let (mut c, a, _) = setup("Tackle", "");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .standalone
        .perish
        .insert(
            a.clone(),
            standalone::PerishCountdown {
                source_id: "departed source".into(),
                due_round: 4,
            },
        );
    pm(&mut c, &a).deployed = false;
    standalone::lifecycle(&mut c).unwrap();
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .standalone
        .perish
        .is_empty());
}
#[test]
fn standalone_perish_body_rejects_ranged_hits_and_respects_ability_damage_immunity() {
    let (mut c, a, t) = setup("Water Gun", "");
    ability(&mut c, &t, "Perish Body");
    let mut attack = core_accuracy(&a, "Water Gun");
    if let Action::Accuracy { target_id, .. } = &mut attack {
        *target_id = Some(t.clone());
    }
    let n = roll(&c, "player", &attack, 4);
    let n = pass(&n);
    let n = roll(&n, "player", &damage_command(&n), 4);
    assert!(n
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .standalone
        .perish
        .is_empty());
    ability(&mut c, &a, "Magic Guard");
    c.battle.as_mut().unwrap().round = 4;
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .standalone
        .perish
        .insert(
            a.clone(),
            standalone::PerishCountdown {
                source_id: t,
                due_round: 4,
            },
        );
    let hp = p(&c, &a).hp;
    standalone::end_round(&mut c).unwrap();
    assert_eq!(p(&c, &a).hp, hp);
}
#[test]
fn standalone_new_state_defaults_disabled_and_previous_engines_keep_gates() {
    let (c, _, _) = setup("Tackle", "");
    let mut value = serde_json::to_value(&c).unwrap();
    value["battle"]["flow"]
        .as_object_mut()
        .unwrap()
        .remove("standalone");
    let old: Campaign = serde_json::from_value(value).unwrap();
    assert!(!standalone::enabled(&old));
    let old = command(&c, StandaloneCommand::Torrent { bonus: 1 });
    let mut old = old;
    old.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .standalone
        .enabled = false;
    assert!(battle::transition(
        &old,
        "gm",
        &act(StandaloneCommand::Torrent { bonus: 2 }),
        vec![],
        &catalog()
    )
    .is_err());
}

#[test]
fn standalone_rivalry_shared_prefix_and_frozen_target_deltas_survive_profile_changes() {
    let (mut c, a, t, high) = super::targeting_tests::multi("Earthquake");
    ability(&mut c, &a, "Rivalry");
    pm(&mut c, &a).stat_overrides.insert("Strength".into(), 4);
    pm(&mut c, &a).stat_overrides.insert("Dexterity".into(), 2);
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.standalone = StandaloneState {
        enabled: true,
        ..Default::default()
    };
    f.special.enabled = true;
    f.extra.enabled = true;
    f.ability_rules = true;
    f.ability_changes.enabled = true;
    f.stat_channels.enabled = true;
    f.battlefield.enabled = true;
    let c = command(
        &c,
        StandaloneCommand::RivalryBasis {
            basis: "gender".into(),
        },
    );
    let c = profile(&c, &a, "male", None, None);
    let c = profile(&c, &t, "male", None, None);
    let c = profile(&c, &high, "female", None, None);
    let attack = super::targeting_tests::attack(&a, vec![t.clone(), high.clone()], "Earthquake");
    let (_, pools) = standalone::accuracy_plan(&c, &attack, &catalog()).unwrap();
    assert_eq!(pools[&t], pools[&high] + 2);
    let n = roll(&c, "player", &attack, 4);
    let q = &n.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    let get = |id: &str| {
        q.targets
            .iter()
            .find(|v| v.target_id == id)
            .unwrap()
            .successes
            .unwrap()
    };
    assert_eq!(get(&t), get(&high) + 2);
    let n = pass(&n);
    let n = pass(&n);
    let n = roll(&n, "player", &damage_command(&n), 4);
    let context = n.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0]
        .damage_context
        .as_ref()
        .unwrap();
    assert_eq!(context.rivalry[&t], 1);
    assert_eq!(context.rivalry[&high], -1);
    let n = if n.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].damage_done {
        let seq = n.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sequence;
        decision(
            &n,
            "gm",
            Action::AdjudicateEffect {
                sequence: seq,
                reason: "No additional ordinary area effect in this fixture".into(),
            },
        )
    } else {
        n
    };
    let changed = profile(&n, &t, "female", None, None);
    let context = changed
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue[0]
        .damage_context
        .as_ref()
        .unwrap();
    assert_eq!(context.rivalry[&t], 1);
    let completed = roll(&changed, "player", &damage_command(&changed), 4);
    let completed = if completed
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .first()
        .is_some_and(|q| q.damage_done)
    {
        let seq = completed
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .queue[0]
            .sequence;
        decision(
            &completed,
            "gm",
            Action::AdjudicateEffect {
                sequence: seq,
                reason: "No additional ordinary area effect in this fixture".into(),
            },
        )
    } else {
        completed
    };
    assert!(completed
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .is_empty());
}
#[tokio::test]
async fn standalone_room_gm_rulings_owner_permissions_retry_reconnect_and_rollback() {
    use crate::{room, store::Store};
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };
    async fn send(remote: &room::Remote, cmd: Command) -> Result<Campaign> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        remote.sender.send((cmd, tx)).await.unwrap();
        tokio::time::timeout(Duration::from_secs(5), rx)
            .await
            .unwrap()
            .unwrap()
    }
    let dirs: Vec<_> = (0..4).map(|_| tempfile::tempdir().unwrap()).collect();
    let stores: Vec<_> = dirs
        .iter()
        .map(|d| Arc::new(Mutex::new(Store::open(d.path()).unwrap())))
        .collect();
    let ids: Vec<_> = stores
        .iter()
        .map(|s| s.lock().unwrap().player_id.clone())
        .collect();
    let (mut c, a, _) = setup("Water Gun", "Torrent");
    c.host_id = ids[0].clone();
    c.gm_id = ids[2].clone();
    pm(&mut c, &a).owner_id = ids[1].clone();
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().owner_id = ids[1].clone();
    stores[0].lock().unwrap().put_campaign(&c).unwrap();
    let cat = Arc::new(catalog());
    let mut hosted = room::start(
        stores[0].clone(),
        cat.clone(),
        c.id.clone(),
        0,
        Arc::new(|_, _| {}),
    )
    .await
    .unwrap();
    let request = JoinRequest {
        address: hosted.info.address.replace("<host-ip>", "127.0.0.1"),
        token: hosted.info.token.clone(),
    };
    let (owner, _) = room::join(
        stores[1].clone(),
        cat.clone(),
        request.clone(),
        Arc::new(|_, _| {}),
    )
    .await
    .unwrap();
    let (gm, _) = room::join(
        stores[2].clone(),
        cat.clone(),
        request.clone(),
        Arc::new(|_, _| {}),
    )
    .await
    .unwrap();
    let (observer, _) = room::join(
        stores[3].clone(),
        cat.clone(),
        request.clone(),
        Arc::new(|_, _| {}),
    )
    .await
    .unwrap();
    let command = Command {
        id: "torrent-one".into(),
        expected_revision: c.revision,
        action: act(StandaloneCommand::Torrent { bonus: 1 }),
    };
    assert_eq!(
        send(&owner, command.clone()).await.unwrap_err().code,
        "permission"
    );
    let changed = send(&gm, command.clone()).await.unwrap();
    assert_eq!(standalone::torrent_choice(&changed), Some(1));
    assert_eq!(
        send(&gm, command.clone()).await.unwrap().revision,
        changed.revision
    );
    let mut stale = command.clone();
    stale.id = "stale-ruling".into();
    assert_eq!(send(&gm, stale).await.unwrap_err().code, "stale");
    owner.task.abort();
    let _ = owner.task.await;
    let (owner, reconnected) = room::join(stores[1].clone(), cat, request, Arc::new(|_, _| {}))
        .await
        .unwrap();
    assert_eq!(standalone::torrent_choice(&reconnected), Some(1));
    assert_eq!(
        standalone::torrent_choice(&stores[0].lock().unwrap().campaigns().unwrap().remove(0)),
        Some(1)
    );
    let rolled = send(
        &gm,
        Command {
            id: "undo-ruling".into(),
            expected_revision: changed.revision,
            action: Action::RollbackBattle {
                event_sequence: changed.battle_undo.last().unwrap().event_sequence,
                reason: "Restore the unresolved printed rule".into(),
            },
        },
    )
    .await
    .unwrap();
    assert_eq!(standalone::torrent_choice(&rolled), None);
    let _ = hosted.host.shutdown.send(true);
    owner.task.abort();
    gm.task.abort();
    observer.task.abort();
    let _ = owner.task.await;
    let _ = gm.task.await;
    let _ = observer.task.await;
    if let Some(stop) = hosted.stop.take() {
        let _ = stop.send(());
    }
    drop(hosted);
    tokio::time::timeout(Duration::from_secs(5), async {
        while stores.iter().any(|s| Arc::strong_count(s) > 1) {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
}

#[test]
fn standalone_moody_suppression_clears_own_channels_but_not_copied_provenance() {
    let (mut c, a) = moody();
    let e = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0]
        .clone();
    standalone::resolve_moody(&mut c, &e, "roll", &[1, 1]).unwrap();
    let mut copy = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .stat_channels
        .grants[1]
        .clone();
    copy.cause = "Costar".into();
    copy.target = c.battle.as_ref().unwrap().combatants[1].id.clone();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .stat_channels
        .grants
        .push(copy);
    ability(&mut c, &a, "");
    standalone::lifecycle(&mut c).unwrap();
    let grants = &c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .stat_channels
        .grants;
    assert_eq!(grants.len(), 1);
    assert_eq!(grants[0].cause, "Costar");
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events
        .is_empty());
}
