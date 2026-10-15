use super::*;
use crate::encounter::*;
fn setup() -> (Campaign, Sheet, TrainingEncounter) {
    let cat = catalog();
    let t = trainer("Rookie");
    let mut c = Campaign::new("Training".into(), t.owner_id.clone(), cat.hash.clone());
    c.sheets.insert(t.id.clone(), t.clone());
    let mut e = generate(&c.id, &t.id, "grassland", "Rookie", 2, "fixture", &cat).unwrap();
    for p in &mut e.individuals {
        p.species_id = "0025:pikachu".into();
        p.ability = "Static".into();
    }
    (c, t, e)
}
#[test]
fn generation_is_repeatable_bounded_and_uses_accepted_pools() {
    let (_, t, e) = setup();
    let cat = catalog();
    let a = generate("campaign", &t.id, "grassland", "Rookie", 50, "same", &cat).unwrap();
    let b = generate("campaign", &t.id, "grassland", "Rookie", 50, "same", &cat).unwrap();
    assert_eq!(
        serde_json::to_value(&a).unwrap(),
        serde_json::to_value(b).unwrap()
    );
    let pool = cat.habitat_members("grassland").unwrap();
    assert!(a.individuals.iter().all(|p| pool.contains(&p.species_id)));
    assert_eq!(
        a.individuals
            .iter()
            .map(|p| &p.id)
            .collect::<std::collections::HashSet<_>>()
            .len(),
        50
    );
    assert!(generate("c", &t.id, "unknown", "Rookie", 5, "seed", &cat).is_err());
    for n in [0, 51] {
        assert!(generate("c", &t.id, "grassland", "Rookie", n, "seed", &cat).is_err());
    }
    let changed = generate(
        "campaign",
        &t.id,
        "grassland",
        "Rookie",
        50,
        "changed",
        &cat,
    )
    .unwrap();
    assert_ne!(a.id, changed.id);
    let mut outside = e;
    outside.individuals[0].species_id = "invalid".into();
    assert!(validate(&outside, &cat).is_err());
}
#[test]
fn balanced_capture_generation_all_ranks_is_legal_and_does_not_mutate_bases() {
    let (c, t, mut e) = setup();
    let cat = catalog();
    for rank in rules::RANKS {
        e.individuals[0].rank = rank.into();
        let a = capture_preview(&e, &e.individuals[0], &t, &c, &cat, None).unwrap();
        assert!(
            a.preview.errors.is_empty(),
            "{rank}: {:?}",
            a.preview.errors
        );
        assert!(a.preview.remaining.values().all(|n| *n >= 0));
        assert!(a.preview.sheet.moves.len() <= a.preview.derived["MoveCapacity"] as usize);
        assert_eq!(a.preview.sheet.hp, a.preview.sheet.max_hp);
        assert_eq!(
            a.preview.sheet.workflow.as_ref().unwrap().training_points,
            0
        );
        let b = capture_preview(&e, &e.individuals[0], &t, &c, &cat, None).unwrap();
        assert_eq!(a.preview.sheet.stats, b.preview.sheet.stats);
        assert_eq!(a.preview.sheet.moves, b.preview.sheet.moves);
    }
}

#[test]
fn capture_modifiers_edits_and_higher_rank_adjudication_use_shared_rules() {
    let (c, t, mut e) = setup();
    let mut cat = catalog();
    let mut sp = cat.find("species", "Name", "Pikachu").unwrap();
    sp["Ability1"] = serde_json::json!("Huge Power");
    sp["Ability2"] = serde_json::json!("Static");
    sp["Creation"]["unresolved_fields"] = serde_json::json!(["BaseHP"]);
    cat.replace_for_test(sp);
    e.individuals[0].ability = "Huge Power".into();
    e.individuals[0].rank = "Advanced".into();
    let p = &e.individuals[0];
    let a = capture_preview(&e, p, &t, &c, &cat, None).unwrap();
    assert!(a.preview.errors.is_empty(), "{:?}", a.preview.errors);
    assert!(a
        .preview
        .warnings
        .iter()
        .any(|w| w.contains("disobedience")));
    assert!(a.preview.approvals.iter().any(|w| w.contains("BaseHP")));
    let b = a.preview.sheet.workflow.as_ref().unwrap();
    assert_eq!(
        a.preview.sheet.stats["Strength"],
        a.preview.baselines["Strength"]
            + b.allocations.get("Strength").copied().unwrap_or(0)
            + a.preview.modifiers["Strength"]
    );
    let refreshed = capture_preview(&e, p, &t, &c, &cat, Some(a.request.clone())).unwrap();
    assert_eq!(a.preview.sheet.stats, refreshed.preview.sheet.stats);
    let mut edited = a.request.clone();
    edited
        .sheet
        .workflow
        .as_mut()
        .unwrap()
        .allocations
        .insert("Strength".into(), 100);
    assert!(!capture_preview(&e, p, &t, &c, &cat, Some(edited))
        .unwrap()
        .preview
        .errors
        .is_empty());
    let mut edited = a.request;
    edited.sheet.owner_id = "forged".into();
    assert!(capture_preview(&e, p, &t, &c, &cat, Some(edited)).is_err());
}

#[test]
fn published_updates_and_replacement_gm_preserve_preparation_and_reject_stale_previews() {
    let (c, t, e) = setup();
    let cat = catalog();
    let (mut c, _) = transition(
        &c,
        &c.gm_id,
        &Action::PublishTraining { encounter: e },
        &cat,
    )
    .unwrap();
    let old = c.training_encounters[0].clone();
    let capture = capture_preview(&old, &old.individuals[0], &t, &c, &cat, None).unwrap();
    c.gm_id = "replacement".into();
    let mut update = old.clone();
    update.individuals[0].rank = "Standard".into();
    let (c, _) = transition(
        &c,
        "replacement",
        &Action::PublishTraining { encounter: update },
        &cat,
    )
    .unwrap();
    let stale = Action::ResolveTraining {
        resolution: EncounterResolution {
            encounter_id: old.id,
            encounter_revision: old.revision,
            captures: vec![CaptureChoice {
                individual_id: old.individuals[0].id.clone(),
                request: capture.request,
            }],
            rewards: vec![],
            reason: "Confirmed".into(),
        },
    };
    assert!(transition(&c, "replacement", &stale, &cat).is_err());
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store.put_campaign(&c).unwrap();
    drop(store);
    let store = Store::open(dir.path()).unwrap();
    let restored = store.campaigns().unwrap().remove(0);
    assert_eq!(
        restored.training_encounters[0].individuals[0].rank,
        "Standard"
    );
    assert_eq!(restored.gm_id, "replacement");
}

#[test]
fn old_backup_restoration_migrates_encounter_storage_and_keeps_identity() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    let identity = store.player_id.clone();
    store.library.execute_batch("DROP TABLE encounter_drafts; DROP TABLE campaign_cache; DELETE FROM metadata WHERE key='encounters_v1'; PRAGMA user_version=2;").unwrap();
    store
        .campaigns
        .execute_batch("PRAGMA user_version=2;")
        .unwrap();
    let backup = store.backup().unwrap();
    drop(store);
    let mut store = Store::open(dir.path()).unwrap();
    store
        .restore_backup(
            std::path::Path::new(&backup)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
        )
        .unwrap();
    assert_eq!(store.player_id, identity);
    assert_eq!(
        store
            .library
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i32>(0))
            .unwrap(),
        4
    );
    assert_eq!(
        store
            .campaigns
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i32>(0))
            .unwrap(),
        4
    );
    assert_eq!(
        store
            .library
            .query_row("SELECT count(*) FROM encounter_drafts", [], |r| r
                .get::<_, i32>(0))
            .unwrap(),
        0
    );
}
#[test]
fn capture_resolution_is_atomic_authorized_and_closed_once() {
    let (c, t, e) = setup();
    let cat = catalog();
    let (c, _) = transition(
        &c,
        &c.gm_id,
        &Action::PublishTraining { encounter: e },
        &cat,
    )
    .unwrap();
    let e = &c.training_encounters[0];
    let p = capture_preview(e, &e.individuals[0], &t, &c, &cat, None).unwrap();
    let resolution = EncounterResolution {
        encounter_id: e.id.clone(),
        encounter_revision: e.revision,
        captures: vec![CaptureChoice {
            individual_id: e.individuals[0].id.clone(),
            request: p.request.clone(),
        }],
        rewards: vec![],
        reason: "GM confirms captured origin and catalog review".into(),
    };
    let action = Action::ResolveTraining {
        resolution: resolution.clone(),
    };
    assert!(transition(&c, "other-player", &action, &cat).is_err());
    let mut stale = resolution.clone();
    stale.captures[0].request.trainer_revision = Some(t.revision + 1);
    assert!(transition(
        &c,
        &c.gm_id,
        &Action::ResolveTraining { resolution: stale },
        &cat
    )
    .is_err());
    let mut invalid = resolution.clone();
    invalid.rewards.push(super::request(&t, "award", None));
    assert!(transition(
        &c,
        &c.gm_id,
        &Action::ResolveTraining {
            resolution: invalid
        },
        &cat
    )
    .is_err());
    assert!(!c.sheets.contains_key(&p.request.sheet.id));
    let (done, _) = transition(&c, &c.gm_id, &action, &cat).unwrap();
    let captured = &done.sheets[&p.request.sheet.id];
    assert_eq!(captured.owner_id, t.owner_id);
    assert_eq!(captured.hp, captured.max_hp);
    let b = captured.workflow.as_ref().unwrap();
    assert_eq!(b.will, b.max_will);
    assert_eq!(
        b.ledger.last().unwrap().approved_by.as_deref(),
        Some(c.gm_id.as_str())
    );
    assert!(transition(&done, &done.gm_id, &action, &cat).is_err());
    assert_eq!(done.training_encounters[0].status, "resolved");
    assert!(done.battle.is_none());
    let mut second = t.clone();
    second.id = id();
    second.owner_id = id();
    let mut mixed = c.clone();
    mixed.sheets.insert(second.id.clone(), second.clone());
    let q = capture_preview(e, &e.individuals[1], &second, &mixed, &cat, None).unwrap();
    let mut multiple = resolution;
    multiple.captures.push(CaptureChoice {
        individual_id: e.individuals[1].id.clone(),
        request: q.request.clone(),
    });
    let (done, _) = transition(
        &mixed,
        &mixed.gm_id,
        &Action::ResolveTraining {
            resolution: multiple,
        },
        &cat,
    )
    .unwrap();
    assert_eq!(done.sheets[&q.request.sheet.id].owner_id, second.owner_id);
}
#[test]
fn draft_migration_backup_is_once_and_catalog_adoption_preserves_history() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    let backups = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|p| p.ok())
        .filter(|p| p.file_name().to_string_lossy().contains("pre-encounters"))
        .count();
    assert_eq!(backups, 2);
    let (mut c, _, e) = setup();
    store
        .library
        .execute(
            "INSERT INTO encounter_drafts VALUES (?1,?2)",
            rusqlite::params![e.id, serde_json::to_string(&e).unwrap()],
        )
        .unwrap();
    drop(store);
    let store = Store::open(dir.path()).unwrap();
    let count: i32 = store
        .library
        .query_row("SELECT count(*) FROM encounter_drafts", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
    assert_eq!(
        backups,
        std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|p| p.ok())
            .filter(|p| p.file_name().to_string_lossy().contains("pre-encounters"))
            .count()
    );
    c.catalog_hash = "previous".into();
    let (next, _) = transition(&c, &c.gm_id, &Action::AdoptCatalog, &catalog()).unwrap();
    assert_eq!(next.catalog_hash, catalog().hash);
    assert_eq!(next.sheets.len(), c.sheets.len());
}
#[tokio::test]
async fn separate_host_and_gm_capture_reward_reconnect_and_offline_import() {
    let host_dir = tempfile::tempdir().unwrap();
    let player_dir = tempfile::tempdir().unwrap();
    let mut host_store = Store::open(host_dir.path()).unwrap();
    let player_store = Store::open(player_dir.path()).unwrap();
    let owner = player_store.player_id.clone();
    let host_id = host_store.player_id.clone();
    let cat = Arc::new(catalog());
    let mut t = trainer("Rookie");
    t.owner_id = owner.clone();
    let pokemon = pokemon(&t, "Pikachu");
    let mut c = Campaign::new(
        "Native training room".into(),
        host_id.clone(),
        cat.hash.clone(),
    );
    c.sheets.insert(t.id.clone(), t.clone());
    c.sheets.insert(pokemon.id.clone(), pokemon.clone());
    c.catalog_hash = "previous-catalog".into();
    host_store.put_campaign(&c).unwrap();
    let host_store = Arc::new(Mutex::new(host_store));
    let player_store = Arc::new(Mutex::new(player_store));
    let notify: room::Notify = Arc::new(|_, _| {});
    let mut host = room::start(
        host_store.clone(),
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
    let (remote, _) = room::join(
        player_store.clone(),
        cat.clone(),
        join.clone(),
        notify.clone(),
    )
    .await
    .unwrap();
    let c = host
        .host
        .apply(
            &host_id,
            &Command {
                id: id(),
                expected_revision: c.revision,
                action: Action::AssignGm {
                    player_id: owner.clone(),
                },
            },
        )
        .unwrap();
    let mut e = generate(&c.id, &t.id, "grassland", "Rookie", 1, "native-seed", &cat).unwrap();
    e.individuals[0].species_id = "0025:pikachu".into();
    e.individuals[0].ability = "Static".into();
    player_store
        .lock()
        .unwrap()
        .library
        .execute(
            "INSERT INTO encounter_drafts VALUES (?1,?2)",
            rusqlite::params![e.id, serde_json::to_string(&e).unwrap()],
        )
        .unwrap();
    assert!(host.host.snapshot().unwrap().training_encounters.is_empty());
    async fn send(remote: &room::Remote, cmd: Command) -> Campaign {
        let (tx, rx) = tokio::sync::oneshot::channel();
        remote.sender.send((cmd, tx)).await.unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(5), rx)
            .await
            .unwrap()
            .unwrap()
            .unwrap()
    }
    assert!(host
        .host
        .apply(
            &owner,
            &Command {
                id: id(),
                expected_revision: c.revision,
                action: Action::PublishTraining {
                    encounter: e.clone()
                }
            }
        )
        .is_err());
    let c = send(
        &remote,
        Command {
            id: id(),
            expected_revision: c.revision,
            action: Action::AdoptCatalog,
        },
    )
    .await;
    let c = send(
        &remote,
        Command {
            id: id(),
            expected_revision: c.revision,
            action: Action::PublishTraining { encounter: e },
        },
    )
    .await;
    let e = &c.training_encounters[0];
    let capture = capture_preview(e, &e.individuals[0], &t, &c, &cat, None).unwrap();
    let mut award = super::request(&t, "award", None);
    award.amount = 8;
    let command = Command {
        id: id(),
        expected_revision: c.revision,
        action: Action::ResolveTraining {
            resolution: EncounterResolution {
                encounter_id: e.id.clone(),
                encounter_revision: e.revision,
                captures: vec![CaptureChoice {
                    individual_id: e.individuals[0].id.clone(),
                    request: capture.request.clone(),
                }],
                rewards: vec![award],
                reason: "Training completed; capture and eight TP approved".into(),
            },
        },
    };
    host_store.lock().unwrap().campaigns.execute_batch("CREATE TRIGGER fail_training BEFORE UPDATE ON campaigns BEGIN SELECT RAISE(ABORT,'injected failure'); END;").unwrap();
    assert!(host.host.apply(&owner, &command).is_err());
    assert!(!host
        .host
        .snapshot()
        .unwrap()
        .sheets
        .contains_key(&capture.request.sheet.id));
    assert_eq!(
        host.host.snapshot().unwrap().sheets[&pokemon.id]
            .workflow
            .as_ref()
            .unwrap()
            .training_points,
        0
    );
    assert_eq!(
        host_store
            .lock()
            .unwrap()
            .campaigns
            .query_row(
                "SELECT count(*) FROM receipts WHERE id=?1",
                [&command.id],
                |r| r.get::<_, i32>(0)
            )
            .unwrap(),
        0
    );
    host_store
        .lock()
        .unwrap()
        .campaigns
        .execute_batch("DROP TRIGGER fail_training")
        .unwrap();
    let done = send(&remote, command.clone()).await;
    let repeated = send(&remote, command.clone()).await;
    assert_eq!(repeated.revision, done.revision);
    assert_eq!(
        done.sheets[&t.id]
            .workflow
            .as_ref()
            .unwrap()
            .training_points,
        8
    );
    // A subsequent normal GM award funds a real offline purchase after import.
    // Encounter captures themselves still start with zero Training Points.
    assert_eq!(
        done.sheets[&capture.request.sheet.id]
            .workflow
            .as_ref()
            .unwrap()
            .training_points,
        0
    );
    let done = send(
        &remote,
        Command {
            id: id(),
            expected_revision: done.revision,
            action: Action::Reward {
                sheet_id: t.id.clone(),
                xp: 50,
                reason: "Later approved training award".into(),
            },
        },
    )
    .await;
    remote.task.abort();
    let (remote, reconnected) = room::join(player_store.clone(), cat.clone(), join, notify)
        .await
        .unwrap();
    assert_eq!(reconnected.training_encounters[0].status, "resolved");
    let repeated = send(&remote, command).await;
    assert_eq!(repeated.revision, done.revision);
    let ids = vec![
        t.id.clone(),
        capture.request.sheet.id.clone(),
        pokemon.id.clone(),
    ];
    let cached = Store::open(player_dir.path())
        .unwrap()
        .available_campaigns()
        .unwrap();
    assert_eq!(cached[0].training_encounters[0].status, "resolved");
    assert!(Store::open(player_dir.path())
        .unwrap()
        .campaigns()
        .unwrap()
        .is_empty());
    let (payload, captured, cost) = {
        let mut local = player_store.lock().unwrap();
        local
            .accept_campaign(&done, &ids, &BTreeMap::new(), &cat)
            .unwrap();
        local
            .accept_campaign(&done, &ids, &BTreeMap::new(), &cat)
            .unwrap();
        let captured = local.sheet(&capture.request.sheet.id).unwrap().unwrap();
        let trainer = local.sheet(&t.id).unwrap().unwrap();
        let transfer = PointTransferRequest {
            kind: "point_transfer".into(),
            operation_id: id(),
            trainer_id: trainer.id.clone(),
            pokemon_id: captured.id.clone(),
            trainer_revision: trainer.revision,
            pokemon_revision: captured.revision,
            catalog_hash: cat.hash.clone(),
            amount: 50,
            reason: "Player distribution".into(),
        };
        let transferred = local.commit_point_transfer(&transfer, &cat).unwrap();
        let captured = transferred.pokemon;
        let trainer = transferred.trainer;
        let mut spend = super::request(&captured, "retrain", Some(&trainer));
        spend.expected_revision = captured.revision;
        let v = local.preview_workflow(&spend, &cat).unwrap();
        assert!(v.errors.is_empty(), "{:?}", v.errors);
        assert!(v.cost > 0);
        let purchased = local.commit_workflow(&spend, &cat).unwrap();
        assert_eq!(purchased.workflow.unwrap().training_points, 50 - v.cost);
        let payload = local
            .sync_payload(
                std::slice::from_ref(&captured.id),
                &done.id,
                BTreeMap::new(),
            )
            .unwrap();
        assert_eq!(payload.len(), 2);
        (payload, captured, v.cost)
    };
    let synced = send(
        &remote,
        Command {
            id: id(),
            expected_revision: done.revision,
            action: Action::Sync { sheets: payload },
        },
    )
    .await;
    assert_eq!(
        synced.sheets[&t.id]
            .workflow
            .as_ref()
            .unwrap()
            .training_points,
        8
    );
    assert_eq!(
        synced.sheets[&captured.id]
            .workflow
            .as_ref()
            .unwrap()
            .training_points,
        50 - cost
    );
    for _ in 0..2 {
        player_store
            .lock()
            .unwrap()
            .accept_campaign(
                &synced,
                std::slice::from_ref(&captured.id),
                &BTreeMap::new(),
                &cat,
            )
            .unwrap();
    }
    assert_eq!(
        player_store
            .lock()
            .unwrap()
            .sheet(&t.id)
            .unwrap()
            .unwrap()
            .workflow
            .unwrap()
            .training_points,
        8
    );
    assert!(host.host.snapshot().unwrap().battle.is_none());
    remote.task.abort();
    if let Some(stop) = host.stop.take() {
        let _ = stop.send(());
    }
}
