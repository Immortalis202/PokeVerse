use super::*;
fn setup(owners: [&str; 4]) -> (Campaign, Action) {
    let mut c = campaign(owners[0]);
    c.gm_id = owners[3].into();
    c.engine = PREVIOUS_ENGINE.into();
    let mut roster = vec![];
    let mut starters = vec![];
    let mut sides = BTreeMap::new();
    for (i, owner) in owners.iter().enumerate() {
        let t = trainer(owner);
        c.sheets.insert(t.id.clone(), t.clone());
        for n in 0..if i == 3 { 6 } else { 2 } {
            let mut p = pokemon(owner, &t.id);
            p.name = format!("Trainer {i} Pokémon {n}");
            let mut b = crate::creation::build(&p, &catalog());
            b.status = "exception".into();
            p.workflow = Some(b);
            if n == 0 || i == 3 && n < 3 {
                starters.push(p.id.clone());
            }
            sides.insert(
                p.id.clone(),
                if i == 3 { "Opponents" } else { "Players" }.into(),
            );
            roster.push(p.id.clone());
            c.sheets.insert(p.id.clone(), p);
        }
    }
    (
        c,
        Action::StartTeamBattle {
            environment: None,
            sheet_ids: roster,
            sides,
            active_sheet_ids: starters,
            slots_per_side: 3,
        },
    )
}
fn start() -> Campaign {
    let (c, a) = setup(["host", "p2", "p3", "gm"]);
    battle::transition(&c, "gm", &a, vec![], &catalog())
        .unwrap()
        .0
}
fn pair(c: &Campaign, owner: &str) -> (String, String) {
    let b = c.battle.as_ref().unwrap();
    (
        b.combatants
            .iter()
            .find(|p| p.owner_id == owner && p.deployed)
            .unwrap()
            .id
            .clone(),
        b.combatants
            .iter()
            .find(|p| p.owner_id == owner && !p.deployed)
            .unwrap()
            .id
            .clone(),
    )
}
fn switch(out: &str, incoming: &str, exception: bool) -> Action {
    Action::SwitchPokemon {
        outgoing_id: out.into(),
        incoming_id: incoming.into(),
        gm_override: exception,
        reason: "Trainer confirms switch".into(),
    }
}
#[test]
fn twelve_registered_six_active_and_reserves_are_not_combatants() {
    let c = start();
    let b = c.battle.as_ref().unwrap();
    assert_eq!(b.combatants.len(), 12);
    assert_eq!(b.combatants.iter().filter(|p| p.deployed).count(), 6);
    for side in ["Players", "Opponents"] {
        assert_eq!(
            b.combatants
                .iter()
                .filter(|p| p.deployed && p.side == side)
                .count(),
            3
        );
    }
    let (out, incoming) = pair(&c, "p2");
    for a in [
        Action::Initiative {
            combatant_id: incoming.clone(),
        },
        Action::Roll {
            combatant_id: incoming.clone(),
            dice: 1,
            reason: "Reserve".into(),
            use_action: false,
        },
        Action::Resolve {
            combatant_id: out,
            target_id: incoming,
            move_name: "".into(),
            dice: 0,
            hp_delta: -1,
            condition: None,
            use_action: false,
            reason: "Bench target".into(),
        },
    ] {
        let dice = if matches!(a, Action::Resolve { .. }) {
            vec![]
        } else {
            vec![1]
        };
        assert!(battle::transition(&c, "gm", &a, dice, &catalog()).is_err());
    }
    let (mut invalid, a) = setup(["host", "p2", "p3", "gm"]);
    let Action::StartTeamBattle {
        active_sheet_ids,
        slots_per_side,
        ..
    } = a.clone()
    else {
        unreachable!()
    };
    invalid.sheets.get_mut(&active_sheet_ids[0]).unwrap().hp = 0;
    assert!(battle::transition(&invalid, "gm", &a, vec![], &catalog()).is_err());
    let (mut invalid, a) = setup(["host", "p2", "p3", "gm"]);
    if let Action::StartTeamBattle {
        sheet_ids,
        mut sides,
        active_sheet_ids,
        ..
    } = a
    {
        for id in &sheet_ids {
            sides.insert(id.clone(), "Players".into());
        }
        assert!(battle::transition(
            &invalid,
            "gm",
            &Action::StartTeamBattle {
                environment: None,
                sheet_ids,
                sides,
                active_sheet_ids,
                slots_per_side
            },
            vec![],
            &catalog()
        )
        .is_err());
    }
    invalid.gm_id = "gm".into();
    assert!(battle::transition(&invalid, "host", &Action::EndBattle, vec![], &catalog()).is_err());
}
#[test]
fn switches_enforce_ownership_waiting_costs_round_reset_and_preserve_resources() {
    let c = start();
    let (out, incoming) = pair(&c, "p2");
    let cmd = switch(&out, &incoming, false);
    assert!(battle::transition(&c, "host", &cmd, vec![], &catalog()).is_err());
    assert!(
        battle::transition(&c, "p2", &switch(&out, &incoming, true), vec![], &catalog()).is_err()
    );
    let (mut next, _) = battle::transition(&c, "p2", &cmd, vec![], &catalog()).unwrap();
    assert_eq!(
        next.battle
            .as_ref()
            .unwrap()
            .combatants
            .iter()
            .filter(|p| p.deployed)
            .count(),
        6
    );
    let b = next.battle.as_ref().unwrap();
    let p = b.combatants.iter().find(|p| p.id == incoming).unwrap();
    assert_eq!(p.available_round, 2);
    assert!(battle::transition(
        &next,
        "p2",
        &Action::Initiative {
            combatant_id: incoming.clone()
        },
        vec![1],
        &catalog()
    )
    .is_err());
    let actor = pair(&next, "host").0;
    next = battle::transition(
        &next,
        "gm",
        &Action::Resolve {
            combatant_id: actor,
            target_id: incoming.clone(),
            move_name: "".into(),
            dice: 0,
            hp_delta: -2,
            condition: Some("test condition".into()),
            use_action: false,
            reason: "Incoming can be targeted".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    next = battle::transition(&next, "gm", &Action::NextRound, vec![], &catalog())
        .unwrap()
        .0;
    next = battle::transition(
        &next,
        "p2",
        &Action::Initiative {
            combatant_id: incoming.clone(),
        },
        vec![3],
        &catalog(),
    )
    .unwrap()
    .0;
    next = battle::transition(
        &next,
        "p2",
        &switch(&incoming, &out, false),
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    let b = next.battle.as_ref().unwrap();
    let p = b.combatants.iter().find(|p| p.id == incoming).unwrap();
    assert_eq!(p.hp, 8);
    assert!(p.initiative_rolled);
    assert_eq!(p.conditions, vec!["test condition"]);
    next = battle::transition(
        &next,
        "p2",
        &switch(&out, &incoming, false),
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    let t = next.sheets[&next
        .battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .find(|p| p.id == out)
        .unwrap()
        .sheet_id]
        .trainer_id
        .as_ref()
        .unwrap();
    assert_eq!(next.battle.as_ref().unwrap().trainer_actions[t], 2);
    next = battle::transition(
        &next,
        "p2",
        &switch(&incoming, &out, false),
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    assert!(battle::transition(
        &next,
        "p2",
        &switch(&out, &incoming, false),
        vec![],
        &catalog()
    )
    .is_err());
    next = battle::transition(
        &next,
        "gm",
        &switch(&out, &incoming, true),
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    assert_eq!(
        next.battle
            .as_ref()
            .unwrap()
            .combatants
            .iter()
            .find(|p| p.id == incoming)
            .unwrap()
            .available_round,
        2
    );
    next = battle::transition(&next, "gm", &Action::EndBattle, vec![], &catalog())
        .unwrap()
        .0;
    let p = next.finished_battles[0]
        .combatants
        .iter()
        .find(|p| p.id == incoming)
        .unwrap();
    assert_eq!(next.sheets[&p.sheet_id].hp, 8);
    assert_eq!(next.sheets[&p.sheet_id].conditions, vec!["test condition"]);
}
#[test]
fn fainted_replacement_pending_attack_locks_and_rollback() {
    let mut c = start();
    let (out, incoming) = pair(&c, "p2");
    c.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == out)
        .unwrap()
        .pending_damage = Some(PendingDamage {
        sequence: 1,
        target_id: incoming.clone(),
        move_name: "Thunderbolt".into(),
        critical: false,
    });
    assert!(
        battle::transition(&c, "gm", &switch(&out, &incoming, true), vec![], &catalog()).is_err()
    );
    let p = c
        .battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == out)
        .unwrap();
    p.pending_damage = None;
    p.hp = 0;
    let (next, _) = battle::transition(
        &c,
        "p2",
        &switch(&out, &incoming, false),
        vec![],
        &catalog(),
    )
    .unwrap();
    let (undone, _) = battle::transition(
        &next,
        "gm",
        &Action::RollbackBattle {
            event_sequence: c.revision + 1,
            reason: "Undo replacement".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(undone.battle).unwrap(),
        serde_json::to_value(c.battle).unwrap()
    );
}

#[tokio::test]
async fn separate_gm_three_trainers_receipts_reconnect_and_persistence() {
    let dirs: Vec<_> = (0..4).map(|_| tempfile::tempdir().unwrap()).collect();
    let stores: Vec<_> = dirs
        .iter()
        .map(|d| Arc::new(Mutex::new(Store::open(d.path()).unwrap())))
        .collect();
    let owners: Vec<_> = stores
        .iter()
        .map(|s| s.lock().unwrap().player_id.clone())
        .collect();
    let (mut c, mut action) = setup([&owners[0], &owners[1], &owners[2], &owners[3]]);
    c.engine = ENGINE.into();
    if let Action::StartTeamBattle { environment, .. } = &mut action {
        *environment = Some(crate::combat::field::EncounterField {
            night: true,
            weather: Some(crate::combat::field::Weather::Rain),
            indoors: false,
            sunlight: true,
            terrain: Some(crate::combat::field::InitialFieldEffect {
                kind: "Grassy Terrain".into(),
                rounds: None,
                parameter: 1,
            }),
            effects: vec![crate::combat::field::InitialFieldEffect {
                kind: "Gravity".into(),
                rounds: None,
                parameter: 1,
            }],
        });
    }
    stores[0].lock().unwrap().put_campaign(&c).unwrap();
    let cat = Arc::new(catalog());
    let notify: room::Notify = Arc::new(|_, _| {});
    let mut host = room::start(
        stores[0].clone(),
        cat.clone(),
        c.id.clone(),
        0,
        notify.clone(),
    )
    .await
    .unwrap();
    let join = JoinRequest {
        address: host.info.address.replace("<host-ip>", "127.0.0.1"),
        token: host.info.token.clone(),
    };
    let mut remotes = vec![];
    for store in &stores[1..] {
        remotes.push(
            room::join(store.clone(), cat.clone(), join.clone(), notify.clone())
                .await
                .unwrap()
                .0,
        );
    }
    async fn send(remote: &room::Remote, cmd: Command) -> Result<Campaign> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        remote.sender.send((cmd, tx)).await.unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(5), rx)
            .await
            .unwrap()
            .unwrap()
    }
    let start_command = Command {
        id: id(),
        expected_revision: c.revision,
        action: action.clone(),
    };
    assert!(host.host.apply(&owners[0], &start_command).is_err());
    assert!(send(&remotes[0], start_command.clone()).await.is_err());
    let mut next = send(&remotes[2], start_command.clone()).await.unwrap();
    assert_eq!(
        send(&remotes[2], start_command).await.unwrap().revision,
        next.revision
    );
    let initial = &next
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .battlefield;
    assert!(initial.night && initial.initial.is_some());
    assert_eq!(initial.natural, Some(crate::combat::field::Weather::Rain));
    assert!(initial.rules.iter().any(|r| r.kind == "Gravity"));
    let (out, incoming) = pair(&next, &owners[1]);
    let command = Command {
        id: id(),
        expected_revision: next.revision,
        action: switch(&out, &incoming, false),
    };
    assert!(host.host.apply(&owners[0], &command).is_err());
    next = send(&remotes[0], command.clone()).await.unwrap();
    let first_revision = next.revision;
    assert_eq!(
        send(&remotes[0], command).await.unwrap().revision,
        first_revision
    );
    let cmd = Command {
        id: id(),
        expected_revision: next.revision,
        action: switch(&incoming, &out, false),
    };
    next = send(&remotes[0], cmd.clone()).await.unwrap();
    let trainer = next.sheets[&next
        .battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .find(|p| p.id == out)
        .unwrap()
        .sheet_id]
        .trainer_id
        .as_ref()
        .unwrap();
    assert_eq!(next.battle.as_ref().unwrap().trainer_actions[trainer], 2);
    assert_eq!(
        send(&remotes[0], cmd).await.unwrap().revision,
        next.revision
    );
    assert!(send(
        &remotes[0],
        Command {
            id: id(),
            expected_revision: next.revision,
            action: Action::Initiative {
                combatant_id: out.clone()
            }
        }
    )
    .await
    .is_err());
    remotes[0].task.abort();
    let (replacement, reconnected) = room::join(stores[1].clone(), cat, join, notify)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(&next.battle).unwrap(),
        serde_json::to_value(&reconnected.battle).unwrap()
    );
    next = send(
        &remotes[2],
        Command {
            id: id(),
            expected_revision: next.revision,
            action: Action::NextRound,
        },
    )
    .await
    .unwrap();
    next = send(
        &replacement,
        Command {
            id: id(),
            expected_revision: next.revision,
            action: Action::Initiative { combatant_id: out },
        },
    )
    .await
    .unwrap();
    assert!(next.battle.as_ref().unwrap().trainer_actions.is_empty());
    let saved = Store::open(dirs[0].path())
        .unwrap()
        .campaigns()
        .unwrap()
        .into_iter()
        .find(|saved| saved.id == c.id)
        .unwrap();
    assert_eq!(
        serde_json::to_value(crate::privacy::for_player(&saved, &owners[1]).battle).unwrap(),
        serde_json::to_value(next.battle).unwrap()
    );
    replacement.task.abort();
    for remote in remotes {
        remote.task.abort();
    }
    if let Some(stop) = host.stop.take() {
        let _ = stop.send(());
    }
}

#[test]
fn legacy_battles_default_active_and_team_migration_is_idempotent() {
    let mut value = serde_json::to_value(start().battle.unwrap()).unwrap();
    for key in ["slots_per_side", "trainer_switches", "trainer_actions"] {
        value.as_object_mut().unwrap().remove(key);
    }
    for p in value["combatants"].as_array_mut().unwrap() {
        for key in ["deployed", "field_slot", "available_round"] {
            p.as_object_mut().unwrap().remove(key);
        }
    }
    let battle: Battle = serde_json::from_value(value).unwrap();
    assert_eq!(battle.slots_per_side, 0);
    assert!(battle
        .combatants
        .iter()
        .all(|p| p.deployed && p.available_round == 0));
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    let owner = store.player_id.clone();
    store
        .library
        .execute("DELETE FROM metadata WHERE key='teams_v1'", [])
        .unwrap();
    store
        .library
        .execute_batch("PRAGMA user_version=3")
        .unwrap();
    store
        .campaigns
        .execute_batch("PRAGMA user_version=3")
        .unwrap();
    drop(store);
    let count = || {
        std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|p| p.ok())
            .filter(|p| p.file_name().to_string_lossy().contains("pre-teams"))
            .count()
    };
    let before = count();
    let store = Store::open(dir.path()).unwrap();
    assert_eq!(store.player_id, owner);
    assert_eq!(
        store
            .library
            .query_row("PRAGMA user_version", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        4
    );
    assert_eq!(count(), before + 2);
    drop(store);
    drop(Store::open(dir.path()).unwrap());
    assert_eq!(count(), before + 2);
}
