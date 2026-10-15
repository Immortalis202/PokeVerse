use super::*;
use crate::points;
fn funded(owner: &str) -> Sheet {
    let mut t = trainer("Rookie");
    t.owner_id = owner.into();
    t.xp = 77;
    let mut req = request(&t, "award", None);
    req.amount = 100;
    rules::apply(Some(&t), None, &req, &catalog(), Some(("campaign", "gm"))).unwrap()
}
fn transfer(t: &Sheet, p: &Sheet, amount: i32) -> PointTransferRequest {
    PointTransferRequest {
        kind: "point_transfer".into(),
        operation_id: id(),
        trainer_id: t.id.clone(),
        pokemon_id: p.id.clone(),
        trainer_revision: t.revision,
        pokemon_revision: p.revision,
        catalog_hash: catalog().hash,
        amount,
        reason: "Player distribution".into(),
    }
}
fn map(sheets: &[Sheet]) -> BTreeMap<String, Sheet> {
    sheets.iter().map(|s| (s.id.clone(), s.clone())).collect()
}
#[test]
fn pool_distribution_conserves_points_and_preserves_legacy_resources() {
    let cat = catalog();
    let t = funded("player");
    let mut p = pokemon(&t, "Pikachu");
    p.hp = 0;
    p.workflow.as_mut().unwrap().will = 0;
    let q = pokemon(&t, "Charmander");
    let a = points::apply(&t, &p, &transfer(&t, &p, 40), &cat).unwrap();
    let b = points::apply(&a.trainer, &q, &transfer(&a.trainer, &q, 30), &cat).unwrap();
    assert_eq!(b.trainer.workflow.as_ref().unwrap().training_points, 30);
    assert_eq!(a.pokemon.workflow.as_ref().unwrap().training_points, 40);
    assert_eq!(b.pokemon.workflow.as_ref().unwrap().training_points, 30);
    assert_eq!(b.trainer.xp, 77);
    assert_eq!(a.pokemon.hp, 0);
    assert_eq!(a.pokemon.workflow.as_ref().unwrap().will, 0);
    assert_eq!(a.pokemon.stats, p.stats);
    assert_eq!(a.pokemon.moves, p.moves);
    for amount in [-1, 0, 101, 100001] {
        assert!(points::apply(&t, &p, &transfer(&t, &p, amount), &cat).is_err());
    }
    let mut wrong = p.clone();
    wrong.owner_id = "other".into();
    assert!(points::apply(&t, &wrong, &transfer(&t, &wrong, 1), &cat).is_err());
    wrong = p.clone();
    wrong.trainer_id = Some(id());
    assert!(points::apply(&t, &wrong, &transfer(&t, &wrong, 1), &cat).is_err());
    let mut stale = transfer(&t, &p, 1);
    stale.trainer_revision += 1;
    assert_eq!(
        points::apply(&t, &p, &stale, &cat).unwrap_err().code,
        "stale"
    );
    let mut stale = transfer(&t, &p, 1);
    stale.pokemon_revision += 1;
    assert_eq!(
        points::apply(&t, &p, &stale, &cat).unwrap_err().code,
        "stale"
    );
    let mut stale = transfer(&t, &p, 1);
    stale.catalog_hash = "old".into();
    assert!(points::apply(&t, &p, &stale, &cat).is_err());
}
#[test]
fn local_transfer_is_atomic_idempotent_and_restorable() {
    let dir = tempfile::tempdir().unwrap();
    let cat = catalog();
    let mut store = Store::open(dir.path()).unwrap();
    let t = funded(&store.player_id);
    let p = pokemon(&t, "Pikachu");
    for s in [&t, &p] {
        store
            .library
            .execute(
                "INSERT INTO sheets VALUES (?1,?2)",
                rusqlite::params![s.id, serde_json::to_string(s).unwrap()],
            )
            .unwrap();
    }
    let req = transfer(&t, &p, 40);
    store.library.execute_batch("CREATE TRIGGER fail_credit BEFORE UPDATE ON sheets WHEN json_extract(NEW.data,'$.kind')='pokemon' BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(store.commit_point_transfer(&req, &cat).is_err());
    assert_eq!(
        store
            .sheet(&t.id)
            .unwrap()
            .unwrap()
            .workflow
            .unwrap()
            .training_points,
        100
    );
    assert_eq!(
        store
            .sheet(&p.id)
            .unwrap()
            .unwrap()
            .workflow
            .unwrap()
            .training_points,
        0
    );
    assert_eq!(
        store
            .library
            .query_row("SELECT count(*) FROM receipts", [], |r| r.get::<_, i32>(0))
            .unwrap(),
        0
    );
    store
        .library
        .execute_batch("DROP TRIGGER fail_credit")
        .unwrap();
    let result = store.commit_point_transfer(&req, &cat).unwrap();
    store.commit_point_transfer(&req, &cat).unwrap();
    let mut reused = req.clone();
    reused.amount = 20;
    assert_eq!(
        store.commit_point_transfer(&reused, &cat).unwrap_err().code,
        "duplicate"
    );
    assert_eq!(
        store
            .library
            .query_row("SELECT count(*) FROM history", [], |r| r.get::<_, i32>(0))
            .unwrap(),
        2
    );
    let backup = dir.path().join("restored.db");
    store
        .library
        .execute("VACUUM INTO ?1", [backup.to_str().unwrap()])
        .unwrap();
    let restored = rusqlite::Connection::open(backup).unwrap();
    let text: String = restored
        .query_row("SELECT data FROM sheets WHERE id=?1", [&t.id], |r| r.get(0))
        .unwrap();
    assert_eq!(
        serde_json::from_str::<Sheet>(&text)
            .unwrap()
            .workflow
            .unwrap()
            .training_points,
        60
    );
    drop(restored);
    let backup_folder = store.backup().unwrap();
    let extra = transfer(&result.trainer, &result.pokemon, 10);
    store.commit_point_transfer(&extra, &cat).unwrap();
    store
        .restore_backup(
            std::path::Path::new(&backup_folder)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
        )
        .unwrap();
    assert_eq!(
        store
            .sheet(&t.id)
            .unwrap()
            .unwrap()
            .workflow
            .unwrap()
            .training_points,
        60
    );
    assert_eq!(
        store
            .sheet(&p.id)
            .unwrap()
            .unwrap()
            .workflow
            .unwrap()
            .training_points,
        40
    );
    store.commit_point_transfer(&req, &cat).unwrap();
    drop(store);
    let mut reopened = Store::open(dir.path()).unwrap();
    assert_eq!(
        reopened
            .commit_point_transfer(&req, &cat)
            .unwrap()
            .pokemon
            .revision,
        result.pokemon.revision
    );
}
#[test]
fn sync_replays_allocation_transfer_purchase_and_rejects_orphan_or_forgery() {
    let cat = catalog();
    let t = funded("player");
    let p = pokemon(&t, "Pikachu");
    let mut alloc = request(&p, "allocate", Some(&t));
    alloc.sheet.skills.insert("Brawl".into(), 1);
    let allocated = rules::apply(Some(&p), Some(&t), &alloc, &cat, None).unwrap();
    let result = points::apply(&t, &allocated, &transfer(&t, &allocated, 40), &cat).unwrap();
    let retrain = request(&result.pokemon, "retrain", Some(&result.trainer));
    let purchased = rules::apply(
        Some(&result.pokemon),
        Some(&result.trainer),
        &retrain,
        &cat,
        None,
    )
    .unwrap();
    let old = map(&[t.clone(), p.clone()]);
    let proposed = map(&[result.trainer.clone(), purchased.clone()]);
    let ids = vec![t.id.clone(), p.id.clone()];
    points::verify_batch(&old, &proposed, &ids, &cat).unwrap();
    // Repeated import/sync has no new ledger operations.
    points::verify_batch(&proposed, &proposed, &ids, &cat).unwrap();
    assert!(points::verify_batch(&old, &proposed, std::slice::from_ref(&p.id), &cat).is_err());
    assert!(points::verify_batch(&old, &proposed, std::slice::from_ref(&t.id), &cat).is_err());
    let mut forged = proposed.clone();
    forged
        .get_mut(&p.id)
        .unwrap()
        .workflow
        .as_mut()
        .unwrap()
        .training_points += 1;
    assert!(points::verify_batch(&old, &forged, &ids, &cat).is_err());
    let mut changed = old.clone();
    changed
        .get_mut(&t.id)
        .unwrap()
        .workflow
        .as_mut()
        .unwrap()
        .training_points += 10;
    assert!(points::verify_batch(&changed, &proposed, &ids, &cat).is_err());
    let branch = points::apply(&t, &p, &transfer(&t, &p, 30), &cat).unwrap();
    let accepted = map(&[branch.trainer, branch.pokemon]);
    assert!(points::verify_batch(&accepted, &proposed, &ids, &cat).is_err());
}
#[test]
fn synchronization_expands_linked_sheets_and_imports_only_once() {
    let dir = tempfile::tempdir().unwrap();
    let cat = catalog();
    let mut store = Store::open(dir.path()).unwrap();
    let t = funded(&store.player_id);
    let p = pokemon(&t, "Pikachu");
    let mut c = Campaign::new("Points".into(), "gm".into(), cat.hash.clone());
    c.sheets = map(&[t.clone(), p.clone()]);
    store
        .accept_campaign(&c, &[t.id.clone(), p.id.clone()], &BTreeMap::new(), &cat)
        .unwrap();
    let t = store.sheet(&t.id).unwrap().unwrap();
    let p = store.sheet(&p.id).unwrap().unwrap();
    let result = store
        .commit_point_transfer(&transfer(&t, &p, 40), &cat)
        .unwrap();
    let payload = store
        .sync_payload(std::slice::from_ref(&p.id), &c.id, BTreeMap::new())
        .unwrap();
    assert_eq!(payload.len(), 2);
    let (accepted, _) = battle::transition(
        &c,
        &store.player_id,
        &Action::Sync { sheets: payload },
        vec![],
        &cat,
    )
    .unwrap();
    for _ in 0..2 {
        store
            .accept_campaign(
                &accepted,
                std::slice::from_ref(&p.id),
                &BTreeMap::new(),
                &cat,
            )
            .unwrap();
    }
    assert_eq!(
        store
            .sheet(&t.id)
            .unwrap()
            .unwrap()
            .workflow
            .unwrap()
            .training_points,
        60
    );
    assert_eq!(
        store
            .sheet(&p.id)
            .unwrap()
            .unwrap()
            .workflow
            .unwrap()
            .training_points,
        40
    );
    assert_eq!(accepted.sheets[&p.id].xp, result.pokemon.xp);
    assert!(battle::transition(
        &accepted,
        "gm",
        &Action::Reward {
            sheet_id: p.id.clone(),
            xp: 10,
            reason: "Wrong recipient".into()
        },
        vec![],
        &cat
    )
    .is_err());
}

#[test]
fn sync_preserves_purchases_between_multiple_distributions_regardless_of_id_order() {
    let cat = catalog();
    let mut t = funded("player");
    t.id = "a-trainer".into();
    let mut p = pokemon(&t, "Pikachu");
    p.id = "z-pokemon".into();
    let mut q = pokemon(&t, "Charmander");
    q.id = "b-pokemon".into();
    let first = points::apply(&t, &p, &transfer(&t, &p, 40), &cat).unwrap();
    let purchase = rules::apply(
        Some(&first.pokemon),
        Some(&first.trainer),
        &request(&first.pokemon, "retrain", Some(&first.trainer)),
        &cat,
        None,
    )
    .unwrap();
    let second =
        points::apply(&first.trainer, &q, &transfer(&first.trainer, &q, 30), &cat).unwrap();
    let old = map(&[t.clone(), p.clone(), q.clone()]);
    let new = map(&[second.trainer, purchase, second.pokemon]);
    points::verify_batch(&old, &new, &[t.id, p.id, q.id], &cat).unwrap();
}

#[test]
fn reconciliation_selects_complete_chain_and_never_leaves_half_a_transfer() {
    let dir = tempfile::tempdir().unwrap();
    let cat = catalog();
    let mut store = Store::open(dir.path()).unwrap();
    let t = funded(&store.player_id);
    let p = pokemon(&t, "Pikachu");
    let mut c = Campaign::new("Conflict".into(), "gm".into(), cat.hash.clone());
    c.sheets = map(&[t.clone(), p.clone()]);
    store
        .accept_campaign(&c, &[t.id.clone(), p.id.clone()], &BTreeMap::new(), &cat)
        .unwrap();
    let t = store.sheet(&t.id).unwrap().unwrap();
    let p = store.sheet(&p.id).unwrap().unwrap();
    store
        .commit_point_transfer(&transfer(&t, &p, 40), &cat)
        .unwrap();
    // An explicit GM legacy Pokemon balance correction conflicts with a local transfer.
    let mut correction = request(&c.sheets[&p.id], "award", Some(&c.sheets[&t.id]));
    correction.amount = 5;
    let corrected = rules::apply(
        Some(&c.sheets[&p.id]),
        Some(&c.sheets[&t.id]),
        &correction,
        &cat,
        Some((&c.id, "gm")),
    )
    .unwrap();
    c.sheets.insert(p.id.clone(), corrected);
    assert_eq!(
        store
            .accept_campaign(&c, std::slice::from_ref(&p.id), &BTreeMap::new(), &cat)
            .unwrap_err()
            .code,
        "conflict"
    );
    assert_eq!(
        store
            .sheet(&t.id)
            .unwrap()
            .unwrap()
            .workflow
            .unwrap()
            .training_points,
        60
    );
    let choices = BTreeMap::from([("/progression".into(), "remote".into())]);
    store
        .accept_campaign(&c, std::slice::from_ref(&p.id), &choices, &cat)
        .unwrap();
    assert_eq!(
        store
            .sheet(&t.id)
            .unwrap()
            .unwrap()
            .workflow
            .unwrap()
            .training_points,
        100
    );
    assert_eq!(
        store
            .sheet(&p.id)
            .unwrap()
            .unwrap()
            .workflow
            .unwrap()
            .training_points,
        5
    );
    let payload = store
        .sync_payload(std::slice::from_ref(&p.id), &c.id, BTreeMap::new())
        .unwrap();
    assert_eq!(payload.len(), 1);
}
