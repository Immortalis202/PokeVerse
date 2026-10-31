use crate::room;
use crate::{battle, catalog::Catalog, creation as rules, model::*, store::Store, sync};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
#[path = "encounter_tests.rs"]
mod encounter_tests;
#[path = "held_item_tests.rs"]
mod held_item_tests;
#[path = "point_tests.rs"]
mod point_tests;
fn catalog() -> Catalog {
    Catalog::bundled().unwrap()
}

#[test]
fn all_rank_prices_and_champion_limits_match_reviewed_manual_fixtures() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../rules/manual-fixtures.json")).unwrap();
    let t = trainer("Champion");
    for (i, r) in rules::RANKS.iter().enumerate() {
        let mut p = award(&pokemon(&t, "Pikachu"), &t, 1000);
        p.rank = (*r).into();
        p.workflow.as_mut().unwrap().origin = "captured".into();
        let v = rules::preview(
            Some(&p),
            Some(&t),
            &request(&p, "retrain", Some(&t)),
            &catalog(),
        );
        assert!(v.errors.is_empty(), "{r}: {:?}", v.errors);
        assert_eq!(
            v.cost as i64,
            fixture["retraining_costs"][i].as_i64().unwrap()
        );
        if i < 7 {
            let v = rules::preview(
                Some(&p),
                Some(&t),
                &request(&p, "rank_up", Some(&t)),
                &catalog(),
            );
            assert!(v.errors.is_empty(), "{r}: {:?}", v.errors);
            assert_eq!(v.cost as i64, fixture["rank_up_costs"][i].as_i64().unwrap());
            assert_eq!(v.sheet.rank, rules::RANKS[i + 1]);
        }
        let mut req = request(&p, "allocate", Some(&t));
        let limit = catalog().find("species", "Name", "Pikachu").unwrap()["MaxStrength"]
            .as_i64()
            .unwrap() as i32;
        req.sheet.workflow.as_mut().unwrap().allocations.insert(
            "Strength".into(),
            limit - p.stats["Strength"] + if i == 7 { 2 } else { 0 },
        );
        let v = rules::preview(Some(&p), Some(&t), &req, &catalog());
        assert!(
            !v.errors
                .iter()
                .any(|e| e.starts_with("Strength allocation")),
            "{r}: {:?}",
            v.errors
        );
        *req.sheet
            .workflow
            .as_mut()
            .unwrap()
            .allocations
            .get_mut("Strength")
            .unwrap() += 1;
        assert!(rules::preview(Some(&p), Some(&t), &req, &catalog())
            .errors
            .iter()
            .any(|e| e.starts_with("Strength allocation")));
    }
}

#[test]
fn atomic_database_failure_cannot_spend_points_or_leave_a_receipt() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let mut t = trainer("Rookie");
    t.owner_id = store.player_id.clone();
    t.revision = 0;
    t.workflow.as_mut().unwrap().ledger.clear();
    let t = store
        .commit_workflow(&request(&t, "create", None), &catalog())
        .unwrap();
    let mut req = request(&t, "allocate", None);
    req.sheet.skills.insert("Alert".into(), 1);
    store.library.execute_batch("CREATE TRIGGER fail_history BEFORE INSERT ON history BEGIN SELECT RAISE(ABORT,'injected storage failure'); END;").unwrap();
    assert!(store.commit_workflow(&req, &catalog()).is_err());
    assert_eq!(store.sheet(&t.id).unwrap().unwrap(), t);
    let count: i32 = store
        .library
        .query_row(
            "SELECT count(*) FROM receipts WHERE id=?1",
            [&req.operation_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
    store
        .library
        .execute_batch("DROP TRIGGER fail_history;")
        .unwrap();
    let saved = store.commit_workflow(&req, &catalog()).unwrap();
    assert_eq!(saved.skills["Alert"], 1);
}

#[test]
fn approved_pull_preserves_unrelated_drafts_and_exception_decisions() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let mut t = trainer("Rookie");
    t.owner_id = store.player_id.clone();
    t.revision = 0;
    t.workflow.as_mut().unwrap().ledger.clear();
    let t = store
        .commit_workflow(&request(&t, "create", None), &catalog())
        .unwrap();
    let mut c = Campaign::new("Review".into(), "gm".into(), catalog().hash);
    c.sheets.insert(t.id.clone(), t.clone());
    store
        .accept_campaign(
            &c,
            std::slice::from_ref(&t.id),
            &BTreeMap::new(),
            &catalog(),
        )
        .unwrap();
    let local = store.sheet(&t.id).unwrap().unwrap();
    let mut req = request(&local, "exception", None);
    req.sheet
        .workflow
        .as_mut()
        .unwrap()
        .allocations
        .insert("Strength".into(), 7);
    let mut unrelated = request(&local, "bond", None);
    unrelated.reason = "Still pending".into();
    store.save_workflow_draft(&req).unwrap();
    store.save_workflow_draft(&unrelated).unwrap();
    let (proposed, _) = battle::transition(
        &c,
        &store.player_id,
        &Action::ProposeWorkflow {
            request: Box::new(req.clone()),
        },
        vec![],
        &catalog(),
    )
    .unwrap();
    let (accepted, _) = battle::transition(
        &proposed,
        "gm",
        &Action::ApproveWorkflow {
            proposal_id: req.operation_id,
            accept: true,
            reason: "Specific Strength exception approved".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap();
    let record = accepted.sheets[&t.id]
        .workflow
        .as_ref()
        .unwrap()
        .ledger
        .last()
        .unwrap();
    assert!(record
        .adjudications
        .iter()
        .any(|e| e.contains("Strength allocation")));
    assert_eq!(
        record.approval_reason.as_deref(),
        Some("Specific Strength exception approved")
    );
    store
        .accept_campaign(&accepted, &[t.id], &BTreeMap::new(), &catalog())
        .unwrap();
    let drafts = store.workflow_drafts().unwrap();
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].operation_id, unrelated.operation_id);
}

#[test]
fn forget_and_failed_evolution_do_not_silently_discard_allocations_or_moves() {
    let t = trainer("Rookie");
    let p = pokemon(&t, "Caterpie");
    let mut req = request(&p, "learn", Some(&t));
    req.move_name = "Tackle".into();
    let p = rules::apply(Some(&p), Some(&t), &req, &catalog(), None).unwrap();
    let mut req = request(&p, "forget", Some(&t));
    req.move_name = "Tackle".into();
    let p = rules::apply(Some(&p), Some(&t), &req, &catalog(), None).unwrap();
    assert!(p.moves.is_empty());
    assert_eq!(p.workflow.as_ref().unwrap().forgotten["Tackle"], "ordinary");
    let mut req = request(&p, "learn", Some(&t));
    req.move_name = "Tackle".into();
    assert!(rules::preview(Some(&p), Some(&t), &req, &catalog()).cost > 0);
    let mut p = award(&p, &t, 100);
    p.workflow
        .as_mut()
        .unwrap()
        .allocations
        .insert("Dexterity".into(), 2);
    let mut req = request(&p, "prepare_evolution", Some(&t));
    req.target_species = "0011:metapod".into();
    let p = rules::apply(Some(&p), Some(&t), &req, &catalog(), None).unwrap();
    let mut altered = catalog();
    let mut sp = altered.find("species", "AppId", "0011:metapod").unwrap();
    sp["MaxDexterity"] = serde_json::json!(1);
    altered.replace_for_test(sp);
    let mut req = request(&p, "evolve", Some(&t));
    req.target_species = "0011:metapod".into();
    assert!(rules::apply(Some(&p), Some(&t), &req, &altered, Some(("test", "gm"))).is_err());
    let v = rules::preview(Some(&p), Some(&t), &req, &altered);
    assert_eq!(
        v.sheet.workflow.unwrap().allocations,
        p.workflow.unwrap().allocations
    );
}

#[test]
fn migration_preserves_balances_identity_membership_history_and_runs_once() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    let mut t = trainer("Rookie");
    t.owner_id = store.player_id.clone();
    t.workflow = None;
    t.xp = 77;
    let mut p = pokemon(&t, "Pikachu");
    p.workflow = None;
    p.xp = 123;
    p.hp = 0;
    let mut c = Campaign::new("Legacy campaign".into(), "gm".into(), catalog().source_hash);
    c.sheets.insert(t.id.clone(), t.clone());
    c.sheets.insert(p.id.clone(), p.clone());
    for s in [&t, &p] {
        store
            .library
            .execute(
                "INSERT INTO sheets VALUES (?1,?2)",
                rusqlite::params![s.id, serde_json::to_string(s).unwrap()],
            )
            .unwrap();
    }
    store
        .library
        .execute(
            "INSERT INTO membership VALUES (?1,?2,?3)",
            rusqlite::params![p.id, c.id, serde_json::to_string(&p).unwrap()],
        )
        .unwrap();
    store
        .campaigns
        .execute(
            "INSERT INTO campaigns VALUES (?1,?2)",
            rusqlite::params![c.id, serde_json::to_string(&c).unwrap()],
        )
        .unwrap();
    store
        .library
        .execute("DELETE FROM metadata WHERE key='workflow_v1'", [])
        .unwrap();
    let identity = store.player_id.clone();
    drop(store);
    let mut store = Store::open(dir.path()).unwrap();
    assert_eq!(store.player_id, identity);
    assert_eq!(store.campaigns().unwrap()[0].catalog_hash, c.catalog_hash);
    let migrated = store.sheet(&p.id).unwrap().unwrap();
    let b = migrated.workflow.as_ref().unwrap();
    assert_eq!(
        (b.training_points, b.legacy_xp, migrated.xp),
        (123, 123, 123)
    );
    assert_eq!(b.status, "review");
    assert_eq!(migrated.hp, 0);
    assert_eq!(store.sheet(&t.id).unwrap().unwrap().xp, 77);
    assert_eq!(
        store
            .sheet(&t.id)
            .unwrap()
            .unwrap()
            .workflow
            .unwrap()
            .training_points,
        0
    );
    assert_eq!(store.library().unwrap().history.len(), 2);
    let h = store
        .library()
        .unwrap()
        .history
        .into_iter()
        .find(|h| h.sheet_id == p.id)
        .unwrap();
    assert_eq!(h.before, p);
    assert_eq!(h.after, migrated);
    assert_eq!(
        store.campaigns().unwrap()[0].sheets[&p.id]
            .workflow
            .as_ref()
            .unwrap()
            .training_points,
        123
    );
    let membership: String = store
        .library
        .query_row(
            "SELECT campaign_id FROM membership WHERE sheet_id=?1",
            [&p.id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(membership, c.id);
    let backups: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains("pre-workflow"))
        .collect();
    assert!(backups.len() >= 2);
    let original = rusqlite::Connection::open(
        backups
            .iter()
            .find(|e| {
                e.file_name().to_string_lossy().starts_with("library")
                    && rusqlite::Connection::open(e.path())
                        .unwrap()
                        .query_row("SELECT count(*) FROM sheets", [], |r| r.get::<_, i32>(0))
                        .unwrap()
                        == 2
            })
            .unwrap()
            .path(),
    )
    .unwrap();
    let text: String = original
        .query_row("SELECT data FROM sheets WHERE id=?1", [&p.id], |r| r.get(0))
        .unwrap();
    assert_eq!(serde_json::from_str::<Sheet>(&text).unwrap(), p);
    let backup = store.backup().unwrap();
    store.library.execute("DELETE FROM sheets", []).unwrap();
    store
        .restore_backup(
            std::path::Path::new(&backup)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
        )
        .unwrap();
    assert_eq!(store.sheet(&p.id).unwrap().unwrap(), migrated);
    drop(store);
    let reopened = Store::open(dir.path()).unwrap();
    assert_eq!(reopened.library().unwrap().history.len(), 2);
    assert_eq!(
        reopened
            .sheet(&p.id)
            .unwrap()
            .unwrap()
            .workflow
            .unwrap()
            .training_points,
        123
    );
    assert_eq!(
        reopened
            .library
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i32>(0))
            .unwrap(),
        4
    );
}

#[tokio::test]
async fn offline_progression_reconnects_through_real_host_and_gm_approval() {
    let host_dir = tempfile::tempdir().unwrap();
    let player_dir = tempfile::tempdir().unwrap();
    let mut host_store = Store::open(host_dir.path()).unwrap();
    let gm = host_store.player_id.clone();
    let c = Campaign::new("Progression room".into(), gm.clone(), catalog().hash);
    host_store.put_campaign(&c).unwrap();
    let host_store = Arc::new(Mutex::new(host_store));
    let mut player_store = Store::open(player_dir.path()).unwrap();
    let mut t = trainer("Rookie");
    t.owner_id = player_store.player_id.clone();
    t.revision = 0;
    t.workflow.as_mut().unwrap().ledger.clear();
    let t = player_store
        .commit_workflow(&request(&t, "create", None), &catalog())
        .unwrap();
    let mut p = pokemon(&t, "Pikachu");
    p.revision = 0;
    p.workflow.as_mut().unwrap().ledger.clear();
    let p = player_store
        .commit_workflow(&request(&p, "create", Some(&t)), &catalog())
        .unwrap();
    let player = player_store.player_id.clone();
    let player_store = Arc::new(Mutex::new(player_store));
    let cat = Arc::new(catalog());
    let notify: room::Notify = Arc::new(|_, _| {});
    let mut hosted = room::start(
        host_store.clone(),
        cat.clone(),
        c.id.clone(),
        0,
        notify.clone(),
    )
    .await
    .unwrap();
    let join = JoinRequest {
        address: hosted.info.address.replace("<host-ip>", "127.0.0.1"),
        token: hosted.info.token.clone(),
    };
    let (client, _) = room::join(
        player_store.clone(),
        cat.clone(),
        join.clone(),
        notify.clone(),
    )
    .await
    .unwrap();
    let action = player_store
        .lock()
        .unwrap()
        .sync_payload(&[t.id.clone(), p.id.clone()], &c.id, BTreeMap::new())
        .unwrap();
    let (tx, rx) = tokio::sync::oneshot::channel();
    client
        .sender
        .send((
            Command {
                id: id(),
                expected_revision: 0,
                action: Action::Sync { sheets: action },
            },
            tx,
        ))
        .await
        .unwrap();
    let accepted = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    player_store
        .lock()
        .unwrap()
        .accept_campaign(
            &accepted,
            &[t.id.clone(), p.id.clone()],
            &BTreeMap::new(),
            &cat,
        )
        .unwrap();
    let local = player_store.lock().unwrap().sheet(&p.id).unwrap().unwrap();
    let local_t = player_store.lock().unwrap().sheet(&t.id).unwrap().unwrap();
    let mut award = request(&local, "award", Some(&local_t));
    award.amount = 25;
    player_store
        .lock()
        .unwrap()
        .save_workflow_draft(&award)
        .unwrap();
    let proposed = hosted
        .host
        .apply(
            &player,
            &Command {
                id: id(),
                expected_revision: accepted.revision,
                action: Action::ProposeWorkflow {
                    request: Box::new(award.clone()),
                },
            },
        )
        .unwrap();
    assert_eq!(proposed.sheets[&p.id].xp, 0);
    let awarded = hosted
        .host
        .apply(
            &gm,
            &Command {
                id: id(),
                expected_revision: proposed.revision,
                action: Action::ApproveWorkflow {
                    proposal_id: award.operation_id,
                    accept: true,
                    reason: "Confirmed session award".into(),
                },
            },
        )
        .unwrap();
    player_store
        .lock()
        .unwrap()
        .accept_campaign(
            &awarded,
            std::slice::from_ref(&p.id),
            &BTreeMap::new(),
            &cat,
        )
        .unwrap();
    assert!(player_store
        .lock()
        .unwrap()
        .workflow_drafts()
        .unwrap()
        .is_empty());
    client.task.abort();
    let local = player_store.lock().unwrap().sheet(&p.id).unwrap().unwrap();
    let mut learn = request(&local, "learn", Some(&local_t));
    learn.move_name = "Thunder Shock".into();
    let learned = player_store
        .lock()
        .unwrap()
        .commit_workflow(&learn, &cat)
        .unwrap();
    assert_eq!(learned.xp, 25);
    let (client, snapshot) = room::join(player_store.clone(), cat.clone(), join, notify)
        .await
        .unwrap();
    let action = player_store
        .lock()
        .unwrap()
        .sync_payload(std::slice::from_ref(&p.id), &c.id, BTreeMap::new())
        .unwrap();
    let (tx, rx) = tokio::sync::oneshot::channel();
    client
        .sender
        .send((
            Command {
                id: id(),
                expected_revision: snapshot.revision,
                action: Action::Sync { sheets: action },
            },
            tx,
        ))
        .await
        .unwrap();
    let synced = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(synced.sheets[&p.id].moves, vec!["Thunder Shock"]);
    let battled = hosted
        .host
        .apply(
            &gm,
            &Command {
                id: id(),
                expected_revision: synced.revision,
                action: Action::StartBattle {
                    environment: None,
                    sheet_ids: vec![p.id.clone()],
                    sides: BTreeMap::new(),
                },
            },
        )
        .unwrap();
    assert_eq!(
        battled.battle.unwrap().combatants[0].selected_ability,
        "Static"
    );
    client.task.abort();
    let _ = hosted.stop.take().unwrap().send(());
}
fn sheet(kind: &str) -> Sheet {
    Sheet {
        gender: None,
        medical: None,
        id: id(),
        owner_id: "player".into(),
        revision: 0,
        kind: kind.into(),
        name: "Test".into(),
        trainer_id: None,
        species_id: None,
        rank: "Rookie".into(),
        nature: "Adamant".into(),
        ability: String::new(),
        stats: BTreeMap::new(),
        skills: BTreeMap::new(),
        moves: vec![],
        hp: 1,
        max_hp: 1,
        xp: 0,
        inventory: BTreeMap::new(),
        held_items: None,
        stored_items: None,
        notes: String::new(),
        archived: false,
        squad_order: 0,
        conditions: vec![],
        workflow: None,
    }
}
fn request(s: &Sheet, kind: &str, t: Option<&Sheet>) -> WorkflowRequest {
    WorkflowRequest {
        operation_id: id(),
        expected_revision: s.revision,
        trainer_revision: t.map(|t| t.revision),
        rules_version: 1,
        catalog_hash: catalog().hash,
        kind: kind.into(),
        sheet: s.clone(),
        move_name: String::new(),
        replacement: String::new(),
        method: "ordinary".into(),
        target_species: String::new(),
        amount: 0,
        reason: "Reviewed fixture".into(),
    }
}
pub(crate) fn trainer(rank: &str) -> Sheet {
    let mut s = sheet("trainer");
    s.rank = rank.into();
    let mut b = rules::build(&s, &catalog());
    b.origin = "starter".into();
    b.allocations.clear();
    b.age = "Child".into();
    s.workflow = Some(b);
    rules::apply(
        None,
        None,
        &request(&s, "create", None),
        &catalog(),
        Some(("test", "gm")),
    )
    .unwrap()
}
fn pokemon(t: &Sheet, name: &str) -> Sheet {
    let mut s = sheet("pokemon");
    s.owner_id = t.owner_id.clone();
    s.trainer_id = Some(t.id.clone());
    s.rank = t.rank.clone();
    s.species_id = Some(
        catalog().find("species", "Name", name).unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
    let mut b = rules::build(&s, &catalog());
    b.origin = "starter".into();
    b.allocations.clear();
    s.workflow = Some(b);
    rules::apply(
        None,
        Some(t),
        &request(&s, "create", Some(t)),
        &catalog(),
        None,
    )
    .unwrap()
}
fn award(s: &Sheet, t: &Sheet, amount: i32) -> Sheet {
    let mut req = request(s, "award", Some(t));
    req.amount = amount;
    rules::apply(Some(s), Some(t), &req, &catalog(), Some(("test", "gm"))).unwrap()
}
#[test]
fn rank_budgets_age_bonuses_caps_and_species_minima() {
    for (i, r) in rules::RANKS.iter().enumerate() {
        let t = trainer(r);
        let p = pokemon(&t, "Pikachu");
        let v = rules::preview(
            Some(&p),
            Some(&t),
            &request(&p, "describe", Some(&t)),
            &catalog(),
        );
        assert!(v.errors.is_empty(), "{r}: {:?}", v.errors);
        assert_eq!(v.remaining["Attributes"], [0, 2, 4, 6, 8, 10, 10, 14][i]);
        assert_eq!(v.remaining["Skills"], [5, 10, 14, 17, 19, 20, 22, 25][i]);
        let mut req = request(&p, "allocate", Some(&t));
        req.sheet
            .skills
            .insert("Brawl".into(), [1, 2, 3, 4, 5, 5, 5, 5][i] + 1);
        assert!(rules::preview(Some(&p), Some(&t), &req, &catalog())
            .errors
            .iter()
            .any(|e| e.starts_with("Skill Brawl")));
    }
    let t = trainer("Rookie");
    let p = pokemon(&t, "Pikachu");
    let mut req = request(&p, "allocate", Some(&t));
    req.sheet
        .workflow
        .as_mut()
        .unwrap()
        .allocations
        .insert("Vitality".into(), -1);
    assert!(rules::apply(Some(&p), Some(&t), &req, &catalog(), None).is_err());
    for (age, a, social) in [
        ("Child", 0, 0),
        ("Teen", 2, 2),
        ("Adult", 4, 4),
        ("Senior", 3, 6),
    ] {
        let mut req = request(&t, "exception", None);
        req.sheet.workflow.as_mut().unwrap().age = age.into();
        let v = rules::preview(Some(&t), None, &req, &catalog());
        assert_eq!(v.remaining["Attributes"], 2 + a);
        assert_eq!(v.remaining["Socials"], 2 + social);
    }
}
#[test]
fn allocations_cannot_be_deleted_to_retrain_for_free_and_bonuses_are_not_spent() {
    let t = trainer("Rookie");
    let p = pokemon(&t, "Pikachu");
    let mut req = request(&p, "allocate", Some(&t));
    req.sheet
        .workflow
        .as_mut()
        .unwrap()
        .allocations
        .insert("Strength".into(), 2);
    let p = rules::apply(Some(&p), Some(&t), &req, &catalog(), None).unwrap();
    let mut req = request(&p, "allocate", Some(&t));
    req.sheet.workflow.as_mut().unwrap().allocations.clear();
    assert!(rules::preview(Some(&p), Some(&t), &req, &catalog())
        .errors
        .contains(&"Redistribution requires retraining".into()));
    let p = pokemon(&t, "Marill");
    assert_eq!(p.ability, "Thick Fat");
    let mut req = request(&p, "ability", Some(&t));
    req.sheet.ability = "Huge Power".into();
    let powered = rules::apply(Some(&p), Some(&t), &req, &catalog(), None).unwrap();
    assert_eq!(powered.stats["Strength"], p.stats["Strength"] + 1);
    let again = rules::apply(
        Some(&powered),
        Some(&t),
        &request(&powered, "describe", Some(&t)),
        &catalog(),
        None,
    )
    .unwrap();
    assert_eq!(again.stats["Strength"], powered.stats["Strength"]);
}
#[test]
fn resources_profiles_and_rank_up_preserve_damage_and_spent_will() {
    let t = trainer("Standard");
    let mut p = pokemon(&t, "Pikachu");
    p.hp = 0;
    p.workflow.as_mut().unwrap().will = 1;
    let mut req = request(&p, "allocate", Some(&t));
    req.sheet
        .workflow
        .as_mut()
        .unwrap()
        .allocations
        .insert("Vitality".into(), 1);
    req.sheet
        .workflow
        .as_mut()
        .unwrap()
        .allocations
        .insert("Insight".into(), 1);
    let p = rules::apply(Some(&p), Some(&t), &req, &catalog(), None).unwrap();
    assert_eq!(p.hp, 0);
    assert_eq!(p.max_hp, 6);
    assert_eq!(p.workflow.as_ref().unwrap().will, 2);
    let mut p = pokemon(&t, "Pikachu");
    p.hp -= 2;
    p = award(&p, &t, 100);
    let old = p.max_hp - p.hp;
    let mut req = request(&p, "exception", Some(&t));
    req.sheet.workflow.as_mut().unwrap().profile.hp = "highest_limit".into();
    let v = rules::preview(Some(&p), Some(&t), &req, &catalog());
    assert_eq!(v.sheet.workflow.as_ref().unwrap().hp_attribute, "Insight");
    assert_eq!(v.sheet.max_hp - v.sheet.hp, old);
    let mut req = request(&p, "exception", Some(&t));
    req.sheet.workflow.as_mut().unwrap().profile.defenses = "vitality".into();
    let v = rules::preview(Some(&p), Some(&t), &req, &catalog());
    assert_eq!(v.derived["Defense"], v.derived["SpecialDefense"]);
    let low = trainer("Rookie");
    let p = award(&pokemon(&low, "Pikachu"), &low, 100);
    assert!(rules::apply(
        Some(&p),
        Some(&low),
        &request(&p, "rank_up", Some(&low)),
        &catalog(),
        None
    )
    .is_err());
}
#[test]
fn starting_moves_rank_capacity_learning_and_retraining_costs() {
    let t = trainer("Rookie");
    let p = pokemon(&t, "Pikachu");
    let mut req = request(&p, "learn", Some(&t));
    req.move_name = "Thunderbolt".into();
    assert!(rules::preview(Some(&p), Some(&t), &req, &catalog())
        .errors
        .iter()
        .any(|e| e.starts_with("Ordinary learning")));
    let mut p = p;
    for name in [
        "Thunder Shock",
        "Tail Whip",
        "Growl",
        "Play Nice",
        "Thunder Wave",
    ] {
        let mut req = request(&p, "learn", Some(&t));
        req.move_name = name.into();
        p = rules::apply(Some(&p), Some(&t), &req, &catalog(), None).unwrap();
    }
    let mut req = request(&p, "learn", Some(&t));
    req.move_name = "Quick Attack".into();
    assert!(rules::preview(Some(&p), Some(&t), &req, &catalog())
        .errors
        .iter()
        .any(|e| e.starts_with("At move capacity")));
    let p = award(&p, &t, 30);
    let mut req = request(&p, "learn", Some(&t));
    req.move_name = "Quick Attack".into();
    req.replacement = "Growl".into();
    let v = rules::preview(Some(&p), Some(&t), &req, &catalog());
    assert_eq!(v.cost, 4);
    assert!(v.errors.is_empty());
    let next = rules::apply(Some(&p), Some(&t), &req, &catalog(), None).unwrap();
    assert_eq!(next.workflow.as_ref().unwrap().training_points, 26);
    assert_eq!(next.moves.len(), 5);
    assert!(next
        .workflow
        .as_ref()
        .unwrap()
        .forgotten
        .contains_key("Growl"));
    let v = rules::preview(
        Some(&next),
        Some(&t),
        &request(&next, "retrain", Some(&t)),
        &catalog(),
    );
    assert_eq!(v.cost, 10);
    assert_eq!(v.sheet.moves, next.moves);
}
#[test]
fn overranking_and_tm_require_correct_cost_provenance_and_approval() {
    let t = trainer("Rookie");
    let mut p = award(&pokemon(&t, "Pikachu"), &t, 100);
    let mut req = request(&p, "learn", Some(&t));
    req.method = "overrank".into();
    req.move_name = "Thunderbolt".into();
    let v = rules::preview(Some(&p), Some(&t), &req, &catalog());
    assert_eq!(v.cost, 30);
    assert!(v
        .errors
        .iter()
        .any(|e| e.starts_with("Over-ranking requires Happiness")));
    p.workflow.as_mut().unwrap().happiness = 4;
    p.workflow.as_mut().unwrap().loyalty = 3;
    let mut req = request(&p, "learn", Some(&t));
    req.method = "overrank".into();
    req.move_name = "Thunderbolt".into();
    let p = rules::apply(Some(&p), Some(&t), &req, &catalog(), None).unwrap();
    assert_eq!(
        p.workflow.as_ref().unwrap().overranked.as_deref(),
        Some("Thunderbolt")
    );
    let mut req = request(&p, "learn", Some(&t));
    req.method = "overrank".into();
    req.move_name = "Thunder".into();
    let p = rules::apply(Some(&p), Some(&t), &req, &catalog(), None).unwrap();
    assert!(!p.moves.contains(&"Thunderbolt".into()));
    let mut p = p;
    p.inventory.insert("TR01".into(), 1);
    let mut req = request(&p, "learn", Some(&t));
    req.method = "tr".into();
    req.move_name = "Surf".into();
    req.target_species = "TR01".into();
    let v = rules::preview(Some(&p), Some(&t), &req, &catalog());
    assert_eq!(v.cost, 5);
    assert_eq!(v.sheet.inventory["TR01"], 0);
    assert!(!v.approvals.is_empty());
    assert!(rules::apply(Some(&p), Some(&t), &req, &catalog(), None).is_err());
}
#[test]
fn evolution_preparation_and_completion_preserve_identity_moves_and_allocations() {
    let t = trainer("Rookie");
    let mut p = award(&pokemon(&t, "Caterpie"), &t, 100);
    p.moves = vec!["Tackle".into()];
    let mut req = request(&p, "prepare_evolution", Some(&t));
    req.target_species = catalog().find("species", "Name", "Metapod").unwrap()["AppId"]
        .as_str()
        .unwrap()
        .into();
    let prepared = rules::apply(Some(&p), Some(&t), &req, &catalog(), None).unwrap();
    assert_eq!(prepared.workflow.as_ref().unwrap().training_points, 90);
    assert_eq!(prepared.species_id, p.species_id);
    let mut req = request(&prepared, "evolve", Some(&t));
    req.target_species = prepared
        .workflow
        .as_ref()
        .unwrap()
        .evolution_ready
        .clone()
        .unwrap();
    assert!(rules::apply(Some(&prepared), Some(&t), &req, &catalog(), None).is_err());
    let evolved = rules::apply(
        Some(&prepared),
        Some(&t),
        &req,
        &catalog(),
        Some(("test", "gm")),
    )
    .unwrap();
    assert_eq!(evolved.id, p.id);
    assert_eq!(evolved.moves, p.moves);
    assert_eq!(
        evolved.workflow.as_ref().unwrap().allocations,
        p.workflow.as_ref().unwrap().allocations
    );
    assert!(evolved.workflow.as_ref().unwrap().evolution_ready.is_none());
}
#[test]
fn offline_commits_are_atomic_idempotent_and_drafts_do_not_replace_sheets() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let mut t = trainer("Rookie");
    t.owner_id = store.player_id.clone();
    t.revision = 0;
    t.workflow.as_mut().unwrap().ledger.clear();
    let req = request(&t, "create", None);
    let t = store.commit_workflow(&req, &catalog()).unwrap();
    assert_eq!(store.commit_workflow(&req, &catalog()).unwrap(), t);
    let mut req = request(&t, "allocate", None);
    req.sheet
        .workflow
        .as_mut()
        .unwrap()
        .allocations
        .insert("Vitality".into(), 99);
    assert!(store.commit_workflow(&req, &catalog()).is_err());
    store.save_workflow_draft(&req).unwrap();
    assert_eq!(store.sheet(&t.id).unwrap().unwrap(), t);
    assert_eq!(store.workflow_drafts().unwrap().len(), 1);
    let mut forged = t.clone();
    forged.stats.insert("Strength".into(), 5);
    assert!(store
        .save(
            SaveRequest {
                operation_id: id(),
                sheet: forged,
                expected_revision: t.revision,
                xp_cost: 0,
                description: "Bypass".into()
            },
            &catalog()
        )
        .is_err());
}
#[test]
fn sync_replays_purchases_rejects_forgery_and_keeps_progression_indivisible() {
    let t = trainer("Rookie");
    let p = pokemon(&t, "Pikachu");
    rules::verify_sync(None, &p, Some(&t), &catalog()).unwrap();
    let mut forged = p.clone();
    forged.workflow.as_mut().unwrap().training_points = 100;
    forged.xp = 100;
    assert!(rules::verify_sync(None, &forged, Some(&t), &catalog()).is_err());
    let mut req = request(&p, "allocate", Some(&t));
    req.sheet
        .workflow
        .as_mut()
        .unwrap()
        .allocations
        .insert("Strength".into(), 1);
    let local = rules::apply(Some(&p), Some(&t), &req, &catalog(), None).unwrap();
    rules::verify_sync(Some(&p), &local, Some(&t), &catalog()).unwrap();
    let mut remote = p.clone();
    remote.notes = "independent".into();
    let merged = sync::merge(&p, &local, &remote, &BTreeMap::new()).unwrap();
    assert_eq!(merged.notes, "independent");
    remote = award(&p, &t, 10);
    let error = sync::merge(&p, &local, &remote, &BTreeMap::new()).unwrap_err();
    assert_eq!(error.conflicts, vec!["/progression"]);
    let mut changed_trainer = t.clone();
    changed_trainer.rank = "Standard".into();
    assert_eq!(
        rules::verify_sync(Some(&p), &local, Some(&changed_trainer), &catalog())
            .unwrap_err()
            .code,
        "stale"
    );
    let mut forged = local.clone();
    forged.hp = 0;
    assert!(rules::verify_sync(Some(&p), &forged, Some(&t), &catalog()).is_err());
}
#[test]
fn gm_proposals_are_scoped_stale_checked_and_keep_host_role_independent() {
    let t = trainer("Rookie");
    let p = pokemon(&t, "Pikachu");
    let mut c = Campaign::new("test".into(), "gm".into(), catalog().hash);
    c.sheets.insert(t.id.clone(), t.clone());
    c.sheets.insert(p.id.clone(), p.clone());
    let mut req = request(&p, "award", Some(&t));
    req.amount = 25;
    let (proposed, _) = battle::transition(
        &c,
        "player",
        &Action::ProposeWorkflow {
            request: Box::new(req.clone()),
        },
        vec![],
        &catalog(),
    )
    .unwrap();
    assert_eq!(proposed.sheets[&p.id].xp, 0);
    let action = Action::ApproveWorkflow {
        proposal_id: req.operation_id,
        accept: true,
        reason: "Training reward confirmed".into(),
    };
    assert!(battle::transition(&proposed, "player", &action, vec![], &catalog()).is_err());
    let (accepted, _) = battle::transition(&proposed, "gm", &action, vec![], &catalog()).unwrap();
    assert_eq!(accepted.sheets[&p.id].xp, 25);
    assert_eq!(
        accepted.sheets[&p.id]
            .workflow
            .as_ref()
            .unwrap()
            .ledger
            .last()
            .unwrap()
            .campaign_id
            .as_deref(),
        Some(c.id.as_str())
    );
    let mut stale = proposed;
    stale.sheets.get_mut(&p.id).unwrap().revision += 1;
    assert!(battle::transition(&stale, "gm", &action, vec![], &catalog()).is_err());
    assert!(rules::verify_sync(None, &accepted.sheets[&p.id], Some(&t), &catalog()).is_err());
}

#[test]
fn m4_r01_mew_wildcard_enforces_rank_capacity_normal_costs_and_replay() {
    let cat = catalog();
    for (rank, permitted) in [("Expert", false), ("Ace", true), ("Master", true)] {
        let t = trainer(rank);
        let p = pokemon(&t, "Mew");
        let mut req = request(&p, "learn", Some(&t));
        req.move_name = "Roar Of Time".into();
        let preview = rules::preview(Some(&p), Some(&t), &req, &cat);
        if !permitted {
            assert!(preview
                .errors
                .iter()
                .any(|e| e.starts_with("Ordinary learning")));
            continue;
        }
        assert!(preview.errors.is_empty(), "{:?}", preview.errors);
        assert!(preview.approvals.is_empty(), "{:?}", preview.approvals);
        assert_eq!(preview.cost, 0);
        let mut full = p.clone();
        let capacity = full.stats["Insight"] + 3;
        full.moves = cat.find("species", "Name", "Mew").unwrap()["Moves"]
            .as_array()
            .unwrap()
            .iter()
            .take(capacity as usize)
            .map(|m| m["Name"].as_str().unwrap().to_owned())
            .collect();
        let mut at_capacity = request(&full, "learn", Some(&t));
        at_capacity.move_name = "Roar Of Time".into();
        assert!(rules::preview(Some(&full), Some(&t), &at_capacity, &cat)
            .errors
            .iter()
            .any(|e| e.starts_with("At move capacity")));
        let learned = rules::apply(Some(&p), Some(&t), &req, &cat, None).unwrap();
        rules::verify_sync(Some(&p), &learned, Some(&t), &cat).unwrap();
        let funded = award(&learned, &t, 20);
        let mut replace = request(&funded, "learn", Some(&t));
        replace.move_name = "Sacred Fire".into();
        replace.replacement = "Roar Of Time".into();
        let preview = rules::preview(Some(&funded), Some(&t), &replace, &cat);
        assert!(preview.errors.is_empty(), "{:?}", preview.errors);
        assert!(preview.approvals.is_empty());
        let expected = if rank == "Ace" { 6 } else { 3 };
        assert_eq!(preview.cost, expected);
        let replaced = rules::apply(Some(&funded), Some(&t), &replace, &cat, None).unwrap();
        assert_eq!(
            replaced.workflow.as_ref().unwrap().training_points,
            20 - expected
        );
        assert!(replaced.moves.contains(&"Sacred Fire".into()));
        assert!(!replaced.moves.contains(&"Roar Of Time".into()));
        rules::verify_sync(Some(&funded), &replaced, Some(&t), &cat).unwrap();
        let mut invalid = request(&replaced, "learn", Some(&t));
        invalid.move_name = "Any Move".into();
        assert!(!rules::preview(Some(&replaced), Some(&t), &invalid, &cat)
            .errors
            .is_empty());
    }
}

#[test]
fn m4_r01_strength_legacy_spelling_preserves_identity_and_cannot_duplicate_learning() {
    let cat = catalog();
    let canonical = cat.find("moves", "Name", "Strength").unwrap();
    assert_eq!(canonical["AppId"], "strength_:Strength ");
    assert_eq!(cat.find("moves", "Name", "Strength ").unwrap(), canonical);
    assert!(cat.find("moves", "Name", "strength").is_none());
    let t = trainer("Ace");
    let mut p = pokemon(&t, "Mew");
    p.moves = vec!["Strength ".into()];
    cat.validate(&p).unwrap();
    let mut req = request(&p, "learn", Some(&t));
    req.move_name = "Strength".into();
    assert!(rules::preview(Some(&p), Some(&t), &req, &cat)
        .errors
        .iter()
        .any(|e| e.contains("unlearned catalog move")));
    p.moves.push("Strength".into());
    assert!(cat.validate(&p).is_err());
}

#[test]
fn s01_standalone_and_form_learning_use_user_final_stage_costs() {
    let cat = catalog();
    for name in [
        "Articuno",
        "Rotom",
        "Deoxys",
        "Groudon (Primal Form)",
        "Necrozma (Ultra Burst Form)",
    ] {
        let t = trainer("Champion");
        let mut draft = sheet("pokemon");
        draft.owner_id = t.owner_id.clone();
        draft.trainer_id = Some(t.id.clone());
        draft.rank = t.rank.clone();
        draft.species_id = Some(
            cat.find("species", "Name", name).unwrap()["AppId"]
                .as_str()
                .unwrap()
                .to_owned(),
        );
        let mut build = rules::build(&draft, &cat);
        build.origin = "starter".into();
        build.allocations.clear();
        draft.workflow = Some(build);
        // Source-field adjudications remain required; the cost ruling must not certify them.
        let p = rules::apply(
            None,
            Some(&t),
            &request(&draft, "create", Some(&t)),
            &cat,
            Some(("test", "gm")),
        )
        .unwrap();
        let species = cat.find("species", "Name", name).unwrap();
        let candidates = species["Moves"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|m| m["Learned"].as_str().and_then(rules::rank).is_some())
            .take(2)
            .map(|m| m["Name"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        let mut p = p;
        let mut first = request(&p, "learn", Some(&t));
        first.move_name = candidates[0].clone();
        p = rules::apply(Some(&p), Some(&t), &first, &cat, Some(("test", "gm"))).unwrap();
        p = award(&p, &t, 20);
        let mut replace = request(&p, "learn", Some(&t));
        replace.move_name = candidates[1].clone();
        replace.replacement = candidates[0].clone();
        let v = rules::preview(Some(&p), Some(&t), &replace, &cat);
        assert!(v.errors.is_empty(), "{name}: {:?}", v.errors);
        assert!(
            !v.approvals.iter().any(|a| a.contains("Evolution stage")),
            "{name}: {:?}",
            v.approvals
        );
        let learned_rank = species["Moves"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["Name"] == candidates[1])
            .unwrap()["Learned"]
            .as_str()
            .and_then(rules::rank)
            .unwrap();
        let expected = if learned_rank < rules::rank(&p.rank).unwrap() {
            3
        } else {
            6
        };
        assert_eq!(v.cost, expected, "{name}");
        let applied =
            rules::apply(Some(&p), Some(&t), &replace, &cat, Some(("test", "gm"))).unwrap();
        assert_eq!(
            applied.workflow.as_ref().unwrap().training_points,
            20 - expected
        );
    }
}

fn s01_reviewed_fixture(t: &Sheet, name: &str, cat: &Catalog) -> Sheet {
    let mut s = sheet("pokemon");
    s.owner_id = t.owner_id.clone();
    s.trainer_id = Some(t.id.clone());
    s.rank = t.rank.clone();
    s.species_id = Some(
        cat.find("species", "Name", name).unwrap()["AppId"]
            .as_str()
            .unwrap()
            .to_owned(),
    );
    let mut b = rules::build(&s, cat);
    b.origin = "starter".into();
    b.allocations.clear();
    s.workflow = Some(b);
    // Test fixtures explicitly adjudicate unreviewed source fields; production guards stay enabled.
    rules::apply(
        None,
        Some(t),
        &request(&s, "create", Some(t)),
        cat,
        Some(("test", "gm")),
    )
    .unwrap()
}

#[test]
fn s01_special_evolution_guidance_preserves_resources_and_gm_completion() {
    let cat = catalog();
    let t = trainer("Champion");
    for (name, target, guidance) in [
        ("Scyther", "Kleavor", "Black Augurite"),
        ("Eevee", "Vaporeon", "Water Stone"),
        ("Eevee", "Espeon", "Sunlight"),
        ("Eevee", "Sylveon", "Loyalty"),
        ("Golbat", "Crobat", "Happiness"),
    ] {
        let mut p = s01_reviewed_fixture(&t, name, &cat);
        p.workflow.as_mut().unwrap().happiness = 0;
        p.workflow.as_mut().unwrap().loyalty = 0;
        for quantity in [0, 2] {
            p.inventory.insert(guidance.into(), quantity);
            let mut req = request(&p, "prepare_evolution", Some(&t));
            req.target_species = cat.find("species", "Name", target).unwrap()["AppId"]
                .as_str()
                .unwrap()
                .to_owned();
            let preview = rules::preview(Some(&p), Some(&t), &req, &cat);
            assert!(preview.errors.is_empty(), "{target}: {:?}", preview.errors);
            assert!(
                preview.approvals.is_empty(),
                "{target}: {:?}",
                preview.approvals
            );
            assert!(preview.warnings.iter().any(|w| w.contains(guidance)));
            assert_eq!(preview.cost, 0);
            let ready = rules::apply(Some(&p), Some(&t), &req, &cat, None).unwrap();
            assert_eq!(ready.inventory, p.inventory);
            assert_eq!(
                ready.workflow.as_ref().unwrap().training_points,
                p.workflow.as_ref().unwrap().training_points
            );
            rules::verify_sync(Some(&p), &ready, Some(&t), &cat).unwrap();
            let mut finish = request(&ready, "evolve", Some(&t));
            finish.target_species = req.target_species.clone();
            assert!(rules::apply(Some(&ready), Some(&t), &finish, &cat, None).is_err());
            let evolved =
                rules::apply(Some(&ready), Some(&t), &finish, &cat, Some(("test", "gm"))).unwrap();
            assert_eq!(evolved.species_id.as_ref(), Some(&req.target_species));
            assert_eq!(evolved.inventory, p.inventory);
        }
    }
}

#[test]
fn s01_gender_legacy_identity_and_sync_preserve_provenance() {
    let cat = catalog();
    let t = trainer("Standard");
    let p = s01_reviewed_fixture(&t, "Kirlia", &cat);
    let legacy = serde_json::to_value(&p).unwrap();
    assert!(legacy.get("gender").is_none());
    let restored: Sheet = serde_json::from_value(legacy).unwrap();
    assert_eq!(restored.gender, None);
    assert_eq!(
        rules::fingerprint(&p).unwrap(),
        rules::fingerprint(&restored).unwrap()
    );
    for gender in [
        Some(PokemonGender::Male),
        Some(PokemonGender::Female),
        Some(PokemonGender::Genderless),
        None,
    ] {
        let mut req = request(&p, "describe", Some(&t));
        req.sheet.gender = gender;
        let edited = rules::apply(Some(&p), Some(&t), &req, &cat, None).unwrap();
        assert_eq!(edited.gender, gender);
        rules::verify_sync(Some(&p), &edited, Some(&t), &cat).unwrap();
        let mut tampered = edited.clone();
        tampered.gender = if gender == Some(PokemonGender::Male) {
            Some(PokemonGender::Female)
        } else {
            Some(PokemonGender::Male)
        };
        assert!(rules::verify_sync(Some(&p), &tampered, Some(&t), &cat).is_err());
    }
    let mut req = request(&t, "describe", None);
    req.sheet.gender = Some(PokemonGender::Male);
    assert!(rules::apply(Some(&t), None, &req, &cat, None).is_err());
    let mut local_req = request(&p, "describe", Some(&t));
    local_req.sheet.gender = Some(PokemonGender::Male);
    let local = rules::apply(Some(&p), Some(&t), &local_req, &cat, None).unwrap();
    let mut remote_req = request(&p, "describe", Some(&t));
    remote_req.sheet.gender = Some(PokemonGender::Female);
    let remote = rules::apply(Some(&p), Some(&t), &remote_req, &cat, None).unwrap();
    let conflict = sync::merge(&p, &local, &remote, &BTreeMap::new()).unwrap_err();
    assert!(conflict.conflicts.contains(&"/progression".into()));
    for (branch, expected) in [("local", &local), ("remote", &remote)] {
        let merged = sync::merge(
            &p,
            &local,
            &remote,
            &BTreeMap::from([("/progression".into(), branch.into())]),
        )
        .unwrap();
        assert_eq!(merged.gender, expected.gender);
        assert_eq!(merged.workflow, expected.workflow);
        rules::verify_sync(Some(&p), &merged, Some(&t), &cat).unwrap();
    }
    let mut req = request(&p, "prepare_evolution", Some(&t));
    req.sheet.gender = Some(PokemonGender::Male);
    assert!(rules::preview(Some(&p), Some(&t), &req, &cat)
        .errors
        .iter()
        .any(|e| e.contains("identity editing")));
}

#[test]
fn s01_kirlia_branches_enforce_gender_and_medium_cost_with_item_guidance() {
    let cat = catalog();
    let t = trainer("Standard");
    let p = award(&s01_reviewed_fixture(&t, "Kirlia", &cat), &t, 60);
    for (target, cost) in [("Gallade", 0), ("Gardevoir", 30)] {
        let id = cat.find("species", "Name", target).unwrap()["AppId"]
            .as_str()
            .unwrap()
            .to_owned();
        for gender in [
            None,
            Some(PokemonGender::Genderless),
            Some(PokemonGender::Female),
            Some(PokemonGender::Male),
        ] {
            let mut describe = request(&p, "describe", Some(&t));
            describe.sheet.gender = gender;
            let mut edited = rules::apply(Some(&p), Some(&t), &describe, &cat, None).unwrap();
            edited.inventory.insert("Dawn Stone".into(), 2);
            let mut req = request(&edited, "prepare_evolution", Some(&t));
            req.target_species = id.clone();
            let preview = rules::preview(Some(&edited), Some(&t), &req, &cat);
            let eligible = gender == Some(PokemonGender::Male)
                || (target == "Gardevoir" && gender == Some(PokemonGender::Female));
            assert_eq!(
                preview.errors.is_empty(),
                eligible,
                "{target} {gender:?}: {:?}",
                preview.errors
            );
            if !eligible {
                continue;
            }
            assert!(preview.approvals.is_empty(), "{:?}", preview.approvals);
            assert_eq!(preview.cost, cost);
            let ready = rules::apply(Some(&edited), Some(&t), &req, &cat, None).unwrap();
            assert_eq!(ready.inventory["Dawn Stone"], 2);
            assert_eq!(ready.workflow.as_ref().unwrap().training_points, 60 - cost);
            rules::verify_sync(Some(&edited), &ready, Some(&t), &cat).unwrap();
            let mut describe = request(&ready, "describe", Some(&t));
            describe.sheet.gender = Some(PokemonGender::Genderless);
            let changed = rules::apply(Some(&ready), Some(&t), &describe, &cat, None).unwrap();
            let mut finish = request(&changed, "evolve", Some(&t));
            finish.target_species = id.clone();
            assert!(rules::apply(
                Some(&changed),
                Some(&t),
                &finish,
                &cat,
                Some(("test", "gm"))
            )
            .is_err());
        }
    }
    let mut male = p.clone();
    male.gender = Some(PokemonGender::Male);
    let mut req = request(&male, "prepare_evolution", Some(&t));
    req.target_species = "0475:gallade".into();
    let without_stone = rules::preview(Some(&male), Some(&t), &req, &cat);
    assert!(
        without_stone.errors.is_empty(),
        "{:?}",
        without_stone.errors
    );
    assert!(without_stone.approvals.is_empty());
    assert!(without_stone
        .warnings
        .iter()
        .any(|w| w.contains("Dawn Stone")));
    req.target_species = "0282:gardevoir".into();
    male.workflow.as_mut().unwrap().training_points = 29;
    req = WorkflowRequest {
        sheet: male.clone(),
        ..req
    };
    assert!(rules::apply(Some(&male), Some(&t), &req, &cat, None).is_err());
}

#[test]
fn s01_named_move_evolutions_require_known_moves_without_mastery_adjudication() {
    let cat = catalog();
    let t = trainer("Champion");
    for (name, target, move_name) in [
        ("Aipom", "Ambipom", "Double Hit"),
        ("Girafarig", "Farigiraf", "Twin Beam"),
        ("Dunsparce", "Dudunsparce", "Hyper Drill"),
        ("Primeape", "Annihilape", "Rage Fist"),
        ("Stantler", "Wyrdeer", "Psyshield Bash"),
        ("Dipplin", "Hydrapple", "Dragon Cheer"),
    ] {
        let mut p = s01_reviewed_fixture(&t, name, &cat);
        if name == "Dipplin" {
            p.inventory.insert("Syrupy Apple".into(), 1);
        }
        let target_id = cat.find("species", "Name", target).unwrap()["AppId"]
            .as_str()
            .unwrap()
            .to_owned();
        let mut req = request(&p, "prepare_evolution", Some(&t));
        req.target_species = target_id.clone();
        let missing = rules::preview(Some(&p), Some(&t), &req, &cat);
        assert!(
            missing.errors.iter().any(|e| e.contains(move_name)),
            "{name}: {:?}",
            missing.errors
        );
        assert!(rules::apply(Some(&p), Some(&t), &req, &cat, None).is_err());
        let mut learn = request(&p, "learn", Some(&t));
        learn.move_name = move_name.into();
        if name == "Dipplin" {
            learn.method = "tutor".into();
        }
        let learned = rules::apply(Some(&p), Some(&t), &learn, &cat, Some(("test", "gm"))).unwrap();
        let mut req = request(&learned, "prepare_evolution", Some(&t));
        req.target_species = target_id.clone();
        let ready = rules::preview(Some(&learned), Some(&t), &req, &cat);
        assert!(ready.errors.is_empty(), "{name}: {:?}", ready.errors);
        assert!(ready.approvals.is_empty(), "{name}: {:?}", ready.approvals);
        assert_eq!(ready.cost, 0);
        let applied = rules::apply(Some(&learned), Some(&t), &req, &cat, None).unwrap();
        assert_eq!(
            applied.workflow.as_ref().unwrap().evolution_ready.as_ref(),
            Some(&target_id)
        );
        assert_eq!(applied.inventory, learned.inventory);
        rules::verify_sync(Some(&learned), &applied, Some(&t), &cat).unwrap();
        let mut finish = request(&applied, "evolve", Some(&t));
        finish.target_species = target_id.clone();
        assert!(rules::preview(Some(&applied), Some(&t), &finish, &cat)
            .approvals
            .iter()
            .any(|a| a.contains("evolution scene")));
        let mut forget = request(&applied, "forget", Some(&t));
        forget.move_name = move_name.into();
        let forgotten = rules::apply(Some(&applied), Some(&t), &forget, &cat, None).unwrap();
        let mut finish = request(&forgotten, "evolve", Some(&t));
        finish.target_species = target_id;
        let rejected = rules::preview(Some(&forgotten), Some(&t), &finish, &cat);
        assert!(rejected.errors.iter().any(|e| e.contains(move_name)));
        assert!(rules::apply(
            Some(&forgotten),
            Some(&t),
            &finish,
            &cat,
            Some(("test", "gm"))
        )
        .is_err());
    }
}

#[test]
fn s01_applin_item_guidance_preserves_inventory_and_dipplin_second_stage_costs() {
    let cat = catalog();
    let t = trainer("Advanced");
    let applin = s01_reviewed_fixture(&t, "Applin", &cat);
    let target = cat.find("species", "Name", "Dipplin").unwrap()["AppId"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut wrong = applin.clone();
    wrong.inventory.insert("Candy Apple".into(), 1);
    let mut req = request(&wrong, "prepare_evolution", Some(&t));
    req.target_species = target.clone();
    let before = serde_json::to_value(&wrong).unwrap();
    let preview = rules::preview(Some(&wrong), Some(&t), &req, &cat);
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    assert!(preview.warnings.iter().any(|e| e.contains("Syrupy Apple")));
    let prepared = rules::apply(Some(&wrong), Some(&t), &req, &cat, None).unwrap();
    assert_eq!(prepared.inventory, wrong.inventory);
    assert_eq!(serde_json::to_value(&wrong).unwrap(), before);
    let mut owned = applin.clone();
    owned.inventory.insert("Syrupy Apple".into(), 2);
    let mut req = request(&owned, "prepare_evolution", Some(&t));
    req.target_species = target.clone();
    let preview = rules::preview(Some(&owned), Some(&t), &req, &cat);
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    assert!(preview.approvals.is_empty(), "{:?}", preview.approvals);
    assert_eq!(preview.cost, 0);
    let prepared = rules::apply(Some(&owned), Some(&t), &req, &cat, None).unwrap();
    assert_eq!(prepared.inventory["Syrupy Apple"], 2);
    assert_eq!(
        prepared.workflow.as_ref().unwrap().evolution_ready.as_ref(),
        Some(&target)
    );
    let p = s01_reviewed_fixture(&t, "Dipplin", &cat);
    let mut first = request(&p, "learn", Some(&t));
    first.move_name = "Withdraw".into();
    let learned = rules::apply(Some(&p), Some(&t), &first, &cat, Some(("test", "gm"))).unwrap();
    let funded = award(&learned, &t, 20);
    let mut replace = request(&funded, "learn", Some(&t));
    replace.move_name = "Reflect".into();
    replace.replacement = "Withdraw".into();
    let priced = rules::preview(Some(&funded), Some(&t), &replace, &cat);
    assert!(priced.errors.is_empty(), "{:?}", priced.errors);
    assert_eq!(priced.cost, 4);
}
