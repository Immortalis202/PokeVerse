use super::*;

fn multi(name: &str) -> (Campaign, String, String, String) {
    let (mut c, a, t) = fixture(name);
    let source = c.battle.as_ref().unwrap().combatants[0].sheet_id.clone();
    c.sheets.get_mut(&source).unwrap().rank = "Standard".into();
    let mut p = c.battle.as_ref().unwrap().combatants[1].clone();
    let mut s = c.sheets[&p.sheet_id].clone();
    s.id = id();
    p.id = id();
    p.sheet_id = s.id.clone();
    p.name = "High defense".into();
    s.stats.insert("Vitality".into(), 5);
    s.stats.insert("Insight".into(), 5);
    let high = p.id.clone();
    c.sheets.insert(s.id.clone(), s);
    c.battle.as_mut().unwrap().combatants.push(p);
    (c, a, t, high)
}
fn attack(a: &str, targets: Vec<String>, name: &str) -> Action {
    Action::AccuracyTargets {
        combatant_id: a.into(),
        target_ids: targets,
        move_name: name.into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Reviewed targeting test".into(),
    }
}
fn roll(c: &Campaign, actor: &str, a: &Action, face: u8) -> Campaign {
    let n = rules::dice_count(c, a, &catalog()).unwrap();
    battle::transition(c, actor, a, vec![face; n as usize], &catalog())
        .unwrap()
        .0
}
fn pass(c: &Campaign) -> Campaign {
    let q = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .last()
        .unwrap();
    decision(
        c,
        "gm",
        Action::PassReaction {
            combatant_id: q.target_id.clone(),
            reason: "Take hit".into(),
        },
    )
}
fn damage_command(c: &Campaign) -> Action {
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    Action::Damage {
        combatant_id: q.actor_id.clone(),
        sequence: q.sequence,
        pool_modifier: 0,
        success_modifier: 0,
        damage_modifier: 0,
        reason: "Shared damage".into(),
    }
}
fn loyal(c: &mut Campaign, id: &str) {
    let sheet = c
        .battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .find(|p| p.id == id)
        .unwrap()
        .sheet_id
        .clone();
    c.sheets.get_mut(&sheet).unwrap().workflow = Some(CharacterBuild {
        version: 1,
        status: "accepted".into(),
        origin: "captured".into(),
        age: "".into(),
        allocations: BTreeMap::new(),
        socials: BTreeMap::new(),
        happiness: 2,
        loyalty: 3,
        training_points: 0,
        legacy_xp: 0,
        will: 6,
        max_will: 6,
        profile: RulesProfile::default(),
        hp_attribute: "Vitality".into(),
        available_abilities: vec![],
        learned: BTreeMap::new(),
        forgotten: BTreeMap::new(),
        overranked: None,
        evolution_ready: None,
        ledger: vec![],
    });
}

#[test]
fn multi_target_accuracy_is_single_and_damage_reuses_prefix_highest_defense_first() {
    let (c, a, t, high) = multi("Swift");
    let c = roll(
        &c,
        "player",
        &attack(&a, vec![t.clone(), high.clone()], "Swift"),
        4,
    );
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].actions, 1);
    let c = pass(&c);
    assert!(c.battle.as_ref().unwrap().flow.as_ref().unwrap().window);
    let c = pass(&c);
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    assert_eq!(q.target_id, high);
    let first = rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap();
    let c = roll(&c, "player", &damage_command(&c), 4);
    let b = c.battle.as_ref().unwrap();
    assert_eq!(b.flow.as_ref().unwrap().queue[0].target_id, t);
    assert_eq!(
        b.flow.as_ref().unwrap().queue[0].shared_dice.len(),
        first as usize
    );
    let extra = rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap();
    assert_eq!(extra, 2);
    let c = roll(&c, "player", &damage_command(&c), 1);
    let b = c.battle.as_ref().unwrap();
    assert!(b.flow.as_ref().unwrap().queue.is_empty());
    assert_eq!(b.combatants[1].hp, b.combatants[2].hp);
    assert_eq!(b.combatants[0].actions, 1);
}

#[test]
fn rank_caps_duplicates_wrong_sides_and_range_are_host_validated() {
    let (mut c, a, t, high) = multi("Swift");
    for targets in [vec![t.clone(), t.clone()], vec![a.clone()]] {
        assert!(rules::dice_count(&c, &attack(&a, targets, "Swift"), &catalog()).is_err());
    }
    let sid = c.battle.as_ref().unwrap().combatants[0].sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().rank = "Starter".into();
    assert!(rules::dice_count(
        &c,
        &attack(&a, vec![t.clone(), high.clone()], "Swift"),
        &catalog()
    )
    .is_err());
    c = decision(
        &c,
        "gm",
        Action::SetBattleRange {
            combatant_id: a.clone(),
            in_range_ids: vec![t.clone()],
            reason: "Only one in range".into(),
        },
    );
    assert!(rules::dice_count(&c, &attack(&a, vec![high], "Swift"), &catalog()).is_err());
    assert!(rules::dice_count(&c, &attack(&a, vec![t], "Swift"), &catalog()).is_ok());
}

#[test]
fn area_fills_rank_capacity_with_foes_then_allies_and_requires_area_clash() {
    let (mut c, a, t, ally) = multi("Earthquake");
    c.battle.as_mut().unwrap().combatants[2].side = "Players".into();
    let sid = c.battle.as_ref().unwrap().combatants[1].sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().moves.push("Surf".into());
    let c = roll(&c, "player", &attack(&a, vec![], "Earthquake"), 4);
    let targets = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].targets;
    assert!(targets.iter().any(|v| v.target_id == ally));
    assert!(targets.iter().any(|v| v.target_id == t));
    // The ally has higher defense and gets its response first.
    let c = pass(&c);
    let bad = Action::React {
        combatant_id: t.clone(),
        kind: "clash".into(),
        move_name: "Tackle".into(),
        reason: "Cannot Clash area".into(),
    };
    assert!(rules::dice_count(&c, &bad, &catalog()).is_err());
    let good = Action::React {
        combatant_id: t,
        kind: "clash".into(),
        move_name: "Surf".into(),
        reason: "Area Clash".into(),
    };
    let c = roll(&c, "other", &good, 4);
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

#[test]
fn field_moves_record_scope_expire_and_cannot_be_evaded() {
    let (c, a, _, _) = multi("Reflect");
    let c = roll(&c, "player", &attack(&a, vec![], "Reflect"), 4);
    let q = c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].clone();
    assert_eq!(q.field_scope.as_deref(), Some("side:Players"));
    assert!(!c.battle.as_ref().unwrap().flow.as_ref().unwrap().window);
    assert!(rules::dice_count(&c, &damage_command(&c), &catalog()).is_err());
    let command = Action::AdjudicateField {
        sequence: q.sequence,
        rounds: Some(1),
        reason: "Reflect mechanics adjudicated by GM".into(),
    };
    assert!(battle::transition(&c, "player", &command, vec![], &catalog()).is_err());
    let mut c = decision(&c, "gm", command);
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .field_effects
            .len(),
        1
    );
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .order
        .clear();
    let c = decision(&c, "gm", Action::NextRound);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .field_effects
        .is_empty());
}

#[test]
fn cover_checks_loyalty_redirects_and_holding_preserves_it() {
    let (mut c, a, t, coverer) = multi("Tackle");
    let incoming = Action::Accuracy {
        combatant_id: a.clone(),
        target_id: Some(t.clone()),
        move_name: "Tackle".into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Target ally".into(),
    };
    let c0 = roll(&c, "player", &incoming, 4);
    let cover = Action::CoverAlly {
        combatant_id: coverer.clone(),
        ally_id: t.clone(),
        reason: "Body shield".into(),
    };
    assert!(rules::dice_count(&c0, &cover, &catalog()).is_err());
    loyal(&mut c, &coverer);
    let c = roll(&c, "player", &incoming, 4);
    assert!(battle::transition(&c, "player", &cover, vec![4; 6], &catalog()).is_err());
    let c = roll(&c, "other", &cover, 4);
    assert_eq!(
        c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].target_id,
        coverer
    );
    let c = roll(&c, "player", &damage_command(&c), 4);
    assert_eq!(c.battle.as_ref().unwrap().combatants[1].hp, 20);
    let c = hold(&c, &t);
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .covers
            .get(&t),
        Some(&coverer)
    );
}

#[test]
fn pending_multi_target_state_survives_serialization_and_rollback() {
    let (c, a, t, high) = multi("Swift");
    let next = roll(&c, "player", &attack(&a, vec![t, high], "Swift"), 4);
    let restored: Campaign = serde_json::from_str(&serde_json::to_string(&next).unwrap()).unwrap();
    assert_eq!(
        restored
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .queue[0]
            .targets
            .len(),
        2
    );
    let undo = decision(
        &restored,
        "gm",
        Action::RollbackBattle {
            event_sequence: c.revision + 1,
            reason: "Undo all targets".into(),
        },
    );
    assert!(undo
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .is_empty());
    assert_eq!(undo.battle.as_ref().unwrap().combatants[0].actions, 0);
}

#[test]
fn target_caps_cover_every_rank_and_allies_include_the_user() {
    let (mut c, a, _, _) = multi("Howl");
    loyal(&mut c, &a);
    let sid = c.battle.as_ref().unwrap().combatants[0].sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .stats
        .insert("Cool".into(), 3);
    for p in &mut c.battle.as_mut().unwrap().combatants {
        p.side = "Players".into();
    }
    let template = c.battle.as_ref().unwrap().combatants[1].clone();
    for _ in 0..8 {
        let mut p = template.clone();
        p.id = id();
        c.battle.as_mut().unwrap().combatants.push(p);
    }
    let all: Vec<_> = c
        .battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .map(|p| p.id.clone())
        .collect();
    for (rank, cap) in [
        "Starter", "Rookie", "Standard", "Advanced", "Expert", "Ace", "Master", "Champion",
    ]
    .into_iter()
    .zip([1, 2, 3, 4, 5, 6, 8, 10])
    {
        c.sheets.get_mut(&sid).unwrap().rank = rank.into();
        assert!(
            rules::dice_count(&c, &attack(&a, all[..cap].to_vec(), "Howl"), &catalog()).is_ok(),
            "{rank}"
        );
        assert!(
            rules::dice_count(&c, &attack(&a, all[..cap + 1].to_vec(), "Howl"), &catalog())
                .is_err(),
            "{rank}"
        );
    }
    assert!(rules::dice_count(&c, &attack(&a, vec![all[1].clone()], "Howl"), &catalog()).is_err());
}

#[test]
fn evading_one_area_target_does_not_cancel_the_other_targets() {
    let (c, a, t, high) = multi("Earthquake");
    let c = roll(&c, "player", &attack(&a, vec![], "Earthquake"), 4);
    let c = react(&c, &high, "evade", "");
    assert!(c.battle.as_ref().unwrap().flow.as_ref().unwrap().window);
    let c = pass(&c);
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    assert_eq!(q.target_id, t);
    assert!(q.targets.iter().any(|v| v.target_id == high && v.avoided));
}

#[test]
fn cover_failure_costs_an_action_and_fifth_action_cannot_be_exceeded() {
    let (mut c, a, t, coverer) = multi("Tackle");
    loyal(&mut c, &coverer);
    let c = hit(&c, &a, &t, "Tackle");
    let command = Action::CoverAlly {
        combatant_id: coverer.clone(),
        ally_id: t.clone(),
        reason: "Attempt cover".into(),
    };
    let failed = roll(&c, "other", &command, 1);
    assert_eq!(failed.battle.as_ref().unwrap().combatants[2].actions, 1);
    assert!(failed
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .covers
        .is_empty());
    assert_eq!(
        failed.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].target_id,
        t
    );
    let mut c = c;
    c.battle.as_mut().unwrap().combatants[2].actions = 5;
    assert!(rules::dice_count(&c, &command, &catalog()).is_err());
}

#[test]
fn previous_turn_engine_is_retained_and_requires_explicit_upgrade() {
    let (mut c, a, t, high) = multi("Swift");
    c.engine = TURN_ENGINE.into();
    assert!(rules::dice_count(&c, &attack(&a, vec![t, high], "Swift"), &catalog()).is_err());
    c.battle.as_mut().unwrap().active = false;
    let next = decision(&c, "gm", Action::UpgradeRules);
    assert_eq!(next.engine, ENGINE);
}

#[tokio::test]
async fn remote_multi_target_damage_receipts_and_reconnect_preserve_shared_dice() {
    let hd = tempfile::tempdir().unwrap();
    let pd = tempfile::tempdir().unwrap();
    let gd = tempfile::tempdir().unwrap();
    let mut store = Store::open(hd.path()).unwrap();
    let player_store = Arc::new(Mutex::new(Store::open(pd.path()).unwrap()));
    let gm_store = Arc::new(Mutex::new(Store::open(gd.path()).unwrap()));
    let player = player_store.lock().unwrap().player_id.clone();
    let gm_id = gm_store.lock().unwrap().player_id.clone();
    let (mut c, a, t, high) = multi("Swift");
    c.host_id = store.player_id.clone();
    c.gm_id = gm_id.clone();
    c.battle.as_mut().unwrap().combatants[0].owner_id = player.clone();
    let sid = c.battle.as_ref().unwrap().combatants[0].sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().owner_id = player.clone();
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
    let (gm, _) = room::join(gm_store, cat.clone(), request.clone(), notify.clone())
        .await
        .unwrap();
    let (remote, _) = room::join(
        player_store.clone(),
        cat.clone(),
        request.clone(),
        notify.clone(),
    )
    .await
    .unwrap();
    let mut action = attack(&a, vec![t, high], "Swift");
    if let Action::AccuracyTargets {
        success_modifier, ..
    } = &mut action
    {
        *success_modifier = 20;
    }
    let accuracy = Command {
        id: id(),
        expected_revision: c.revision,
        action,
    };
    let (tx, rx) = tokio::sync::oneshot::channel();
    gm.sender.send((accuracy.clone(), tx)).await.unwrap();
    let mut accepted = rx.await.unwrap().unwrap();
    assert_eq!(accepted.battle.as_ref().unwrap().combatants[0].actions, 1);
    assert_eq!(
        host.host.apply(&gm_id, &accuracy).unwrap().revision,
        accepted.revision
    );
    for _ in 0..2 {
        let q = accepted
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .queue
            .last()
            .unwrap();
        accepted = host
            .host
            .apply(
                &gm_id,
                &Command {
                    id: id(),
                    expected_revision: accepted.revision,
                    action: Action::PassReaction {
                        combatant_id: q.target_id.clone(),
                        reason: "Take hit".into(),
                    },
                },
            )
            .unwrap();
    }
    let first = Command {
        id: id(),
        expected_revision: accepted.revision,
        action: damage_command(&accepted),
    };
    let (tx, rx) = tokio::sync::oneshot::channel();
    remote.sender.send((first.clone(), tx)).await.unwrap();
    accepted = rx.await.unwrap().unwrap();
    let saved = accepted
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue[0]
        .shared_dice
        .clone();
    assert!(!saved.is_empty());
    assert_eq!(
        host.host.apply(&player, &first).unwrap().revision,
        accepted.revision
    );
    remote.task.abort();
    let (reconnected, snapshot) = room::join(player_store, cat, request, notify)
        .await
        .unwrap();
    assert_eq!(
        snapshot
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .queue[0]
            .shared_dice,
        saved
    );
    let second = Command {
        id: id(),
        expected_revision: accepted.revision,
        action: damage_command(&accepted),
    };
    let (tx, rx) = tokio::sync::oneshot::channel();
    reconnected.sender.send((second, tx)).await.unwrap();
    let finished = rx.await.unwrap().unwrap();
    assert!(finished
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .is_empty());
    let persisted = shared.lock().unwrap().campaigns().unwrap();
    assert_eq!(persisted[0].revision, finished.revision);
    gm.task.abort();
    reconnected.task.abort();
    let _ = host.host.shutdown.send(true);
    let _ = host.stop.take().unwrap().send(());
}
