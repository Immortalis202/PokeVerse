use crate::{battle, catalog::Catalog, model::*, room, store::Store, sync};
#[path = "combat_tests.rs"]
mod combat_tests;
#[path = "scenario_3v3_tests.rs"]
mod scenario_3v3_tests;
#[path = "team_tests.rs"]
mod team_tests;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

fn catalog() -> Catalog {
    Catalog::bundled().unwrap()
}
fn trainer(owner: &str) -> Sheet {
    Sheet {
        medical: None,
        id: id(),
        owner_id: owner.into(),
        revision: 0,
        kind: "trainer".into(),
        name: "Maya".into(),
        trainer_id: None,
        species_id: None,
        rank: "Starter".into(),
        nature: "".into(),
        ability: "".into(),
        stats: BTreeMap::new(),
        skills: BTreeMap::new(),
        moves: vec![],
        hp: 10,
        max_hp: 10,
        xp: 10,
        inventory: BTreeMap::new(),
        held_items: None,
        stored_items: None,
        notes: "".into(),
        archived: false,
        squad_order: 0,
        conditions: vec![],
        workflow: None,
    }
}
fn pokemon(owner: &str, trainer_id: &str) -> Sheet {
    let mut s = trainer(owner);
    s.kind = "pokemon".into();
    s.name = "Pika".into();
    s.trainer_id = Some(trainer_id.into());
    s.species_id = Some("0025:pikachu".into());
    s.stats.insert("Strength".into(), 2);
    s.moves = vec!["Thunderbolt".into()];
    s.ability = "Static".into();
    s
}
fn request(s: Sheet) -> SaveRequest {
    SaveRequest {
        operation_id: id(),
        expected_revision: s.revision,
        sheet: s,
        xp_cost: 0,
        description: "Test edit".into(),
    }
}
fn campaign(owner: &str) -> Campaign {
    // Legacy fixtures exercise the retained manual compatibility path.
    let mut c = Campaign::new("Test campaign".into(), owner.into(), catalog().hash);
    c.engine = LEGACY_ENGINE.into();
    c
}
fn sync_action(sheets: Vec<Sheet>) -> Action {
    Action::Sync {
        sheets: sheets
            .into_iter()
            .map(|sheet| SyncSheet {
                sheet,
                baseline: None,
                resolutions: BTreeMap::new(),
            })
            .collect(),
    }
}

fn core_battle(move_name: &str) -> (Campaign, String) {
    let t = trainer("player");
    let mut p = pokemon("player", &t.id);
    p.moves = vec![move_name.into()];
    p.stats.insert("Dexterity".into(), 3);
    p.stats.insert("Strength".into(), 4);
    p.stats.insert("Special".into(), 3);
    p.skills.insert("Channel".into(), 3);
    p.skills.insert("Athletic".into(), 2);
    p.skills.insert("Brawl".into(), 3);
    p.skills.insert("Alert".into(), 2);
    let mut c = campaign("gm");
    c.engine = PREVIOUS_ENGINE.into();
    let mut build = crate::creation::build(&p, &catalog());
    build.status = "exception".into();
    p.workflow = Some(build); // Accepted GM exception fixture: tests isolate battle formulas.
    c.sheets.insert(t.id.clone(), t);
    c.sheets.insert(p.id.clone(), p.clone());
    let (c, _) = battle::transition(
        &c,
        "gm",
        &Action::StartBattle {
            sheet_ids: vec![p.id],
            sides: BTreeMap::new(),
        },
        vec![],
        &catalog(),
    )
    .unwrap();
    let key = c.battle.as_ref().unwrap().combatants[0].id.clone();
    (c, key)
}

#[test]
fn legacy_host_stays_owner_when_gm_changes_and_can_reclaim_role() {
    let c = campaign("host");
    let mut json = serde_json::to_value(c).unwrap();
    json.as_object_mut().unwrap().remove("host_id");
    let legacy: Campaign = serde_json::from_value(json).unwrap();
    assert_eq!(legacy.host_id(), "host");
    let (assigned, _) = battle::transition(
        &legacy,
        "host",
        &Action::AssignGm {
            player_id: "remote".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap();
    assert_eq!(assigned.host_id(), "host");
    assert_eq!(assigned.gm_id, "remote");
    assert!(battle::transition(
        &assigned,
        "remote",
        &Action::AssignGm {
            player_id: "remote".into()
        },
        vec![],
        &catalog()
    )
    .is_err());
    let (reclaimed, _) = battle::transition(
        &assigned,
        "host",
        &Action::AssignGm {
            player_id: "host".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap();
    assert_eq!(reclaimed.host_id(), "host");
    assert_eq!(reclaimed.gm_id, "host");
}
fn core_accuracy(key: &str, name: &str) -> Action {
    Action::Accuracy {
        combatant_id: key.into(),
        target_id: None,
        move_name: name.into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Base roll; GM checks effects".into(),
    }
}

fn damage_fixture(
    move_name: &str,
    species_name: &str,
    critical: bool,
) -> (Campaign, String, String) {
    let (mut c, key) = core_battle(move_name);
    let species = catalog()
        .find("species", "Name", species_name)
        .unwrap()
        .clone();
    let mut s = c
        .sheets
        .values()
        .find(|s| s.kind == "pokemon")
        .unwrap()
        .clone();
    s.id = id();
    s.owner_id = "other".into();
    s.species_id = Some(species["AppId"].as_str().unwrap().into());
    s.stats.insert("Vitality".into(), 2);
    s.stats.insert("Insight".into(), 2);
    let mut p = c.battle.as_ref().unwrap().combatants[0].clone();
    p.id = id();
    p.sheet_id = s.id.clone();
    p.owner_id = s.owner_id.clone();
    p.pending_damage = None;
    let target = p.id.clone();
    c.sheets.insert(s.id.clone(), s);
    let b = c.battle.as_mut().unwrap();
    b.combatants.push(p);
    b.combatants[0].pending_damage = Some(PendingDamage {
        sequence: 7,
        target_id: target.clone(),
        move_name: move_name.into(),
        critical,
    });
    c.revision = 7;
    (c, key, target)
}

fn damage_action(key: &str) -> Action {
    Action::Damage {
        combatant_id: key.into(),
        sequence: 7,
        pool_modifier: 0,
        success_modifier: 0,
        damage_modifier: 0,
        reason: "After reactions".into(),
    }
}

#[test]
fn automatic_damage_uses_stab_defense_critical_pain_and_minimum_pool() {
    let (mut c, key, _) = damage_fixture("Thunderbolt", "Squirtle", true);
    let pool = crate::rules::damage_pool(&c, &key, 7, 0, &catalog()).unwrap();
    assert_eq!(
        (
            pool.attribute,
            pool.power,
            pool.defense,
            pool.stab,
            pool.critical,
            pool.dice
        ),
        (3, 3, 2, 1, 2, 7)
    );
    assert_eq!(pool.effectiveness, 1);
    c.battle.as_mut().unwrap().combatants[0].hp = 1;
    assert_eq!(
        crate::rules::damage_pool(&c, &key, 7, 0, &catalog())
            .unwrap()
            .pain,
        2
    );
    c.battle.as_mut().unwrap().combatants[1]
        .stat_overrides
        .insert("Insight".into(), 99);
    assert_eq!(
        crate::rules::damage_pool(&c, &key, 7, 0, &catalog())
            .unwrap()
            .dice,
        4
    ); // min 1 + STAB + critical
    c.sheets
        .get_mut(&c.battle.as_ref().unwrap().combatants[1].sheet_id)
        .unwrap()
        .rank = "Master".into();
    assert_eq!(
        crate::rules::damage_pool(&c, &key, 7, 0, &catalog())
            .unwrap()
            .defense,
        102
    );
}

#[test]
fn damage_type_adjustments_require_success_and_can_prevent_all_damage() {
    for (species, dice, expected) in [
        ("Squirtle", vec![1; 5], 1),
        ("Squirtle", vec![6, 1, 1, 1, 1], 2),
        ("Pikachu", vec![1; 5], 0),
        ("Gyarados", vec![6, 1, 1, 1, 1], 3),
        ("Larvitar", vec![], 0),
    ] {
        let (c, key, target) = damage_fixture("Thunderbolt", species, false);
        let (after, _) =
            battle::transition(&c, "player", &damage_action(&key), dice, &catalog()).unwrap();
        let b = after.battle.unwrap();
        assert_eq!(
            b.combatants.iter().find(|p| p.id == target).unwrap().hp,
            10 - expected,
            "{species}"
        );
        assert!(b.combatants[0].pending_damage.is_none());
        assert_eq!(b.combatants[0].actions, 0); // damage consumes no second action
    }
    assert_eq!(crate::rules::type_effect("Ice", "Steel").unwrap(), -1);
    assert_eq!(crate::rules::type_effect("Ice", "Water").unwrap(), -1);
    assert_eq!(crate::rules::type_effect("Normal", "Ghost").unwrap(), -100);
    assert!(crate::rules::type_effect("Invalid", "Water").is_err());
}

#[test]
fn damage_modifiers_and_pending_cancellation_keep_gm_authority() {
    let (c, key, _) = damage_fixture("Thunderbolt", "Squirtle", false);
    let mut action = damage_action(&key);
    if let Action::Damage {
        damage_modifier, ..
    } = &mut action
    {
        *damage_modifier = -3;
    }
    assert_eq!(
        battle::transition(&c, "player", &action, vec![1; 5], &catalog())
            .unwrap_err()
            .code,
        "permission"
    );
    let (after, _) = battle::transition(&c, "gm", &action, vec![1; 5], &catalog()).unwrap();
    assert_eq!(after.battle.unwrap().combatants[1].hp, 10); // shield can negate minimum damage
    assert!(battle::transition(&c, "gm", &Action::NextRound, vec![], &catalog()).is_err());
    assert!(battle::transition(&c, "gm", &Action::EndBattle, vec![], &catalog()).is_err());
    let cancel = Action::CancelDamage {
        combatant_id: key,
        sequence: 7,
        reason: "Evaded".into(),
    };
    assert_eq!(
        battle::transition(&c, "player", &cancel, vec![], &catalog())
            .unwrap_err()
            .code,
        "permission"
    );
    let (cancelled, _) = battle::transition(&c, "gm", &cancel, vec![], &catalog()).unwrap();
    assert!(cancelled.battle.as_ref().unwrap().combatants[0]
        .pending_damage
        .is_none());
    assert!(battle::transition(&cancelled, "gm", &Action::NextRound, vec![], &catalog()).is_ok());
}

#[tokio::test]
async fn remote_damage_is_host_rolled_persisted_and_idempotent_after_rollback() {
    let host_dir = tempfile::tempdir().unwrap();
    let player_dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(host_dir.path()).unwrap();
    let gm = store.player_id.clone();
    let player_store = Arc::new(Mutex::new(Store::open(player_dir.path()).unwrap()));
    let player = player_store.lock().unwrap().player_id.clone();
    let (mut c, key, _) = damage_fixture("Thunderbolt", "Squirtle", false);
    c.gm_id = gm.clone();
    c.host_id = gm.clone();
    c.battle.as_mut().unwrap().combatants[0].owner_id = player.clone();
    let source_id = c.battle.as_ref().unwrap().combatants[0].sheet_id.clone();
    c.sheets.get_mut(&source_id).unwrap().owner_id = player.clone();
    store.put_campaign(&c).unwrap();
    let shared = Arc::new(Mutex::new(store));
    let catalog = Arc::new(catalog());
    let notify: room::Notify = Arc::new(|_, _| {});
    let mut host = room::start(shared.clone(), catalog.clone(), c.id, 0, notify.clone())
        .await
        .unwrap();
    let (remote, _) = room::join(
        player_store,
        catalog,
        JoinRequest {
            address: host.info.address.replace("<host-ip>", "127.0.0.1"),
            token: host.info.token.clone(),
        },
        notify,
    )
    .await
    .unwrap();
    let command = Command {
        id: id(),
        expected_revision: 7,
        action: damage_action(&key),
    };
    let (tx, rx) = tokio::sync::oneshot::channel();
    remote.sender.send((command.clone(), tx)).await.unwrap();
    let resolved = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(resolved.revision, 8);
    assert_eq!(resolved.events.last().unwrap().dice.len(), 5);
    assert!(resolved.battle.as_ref().unwrap().combatants[1].hp < 10);
    assert!(resolved.battle.as_ref().unwrap().combatants[0]
        .pending_damage
        .is_none());
    assert_eq!(host.host.apply(&player, &command).unwrap().revision, 8);
    let restored = host
        .host
        .apply(
            &gm,
            &Command {
                id: id(),
                expected_revision: 8,
                action: Action::RollbackBattle {
                    event_sequence: 8,
                    reason: "Correction".into(),
                },
            },
        )
        .unwrap();
    assert_eq!(restored.battle.as_ref().unwrap().combatants[1].hp, 10);
    let retry = host.host.apply(&player, &command).unwrap();
    assert_eq!(retry.revision, 9);
    assert_eq!(retry.battle.as_ref().unwrap().combatants[1].hp, 10);
    assert_eq!(retry.events[0].dice, resolved.events[0].dice);
    assert_eq!(shared.lock().unwrap().campaigns().unwrap()[0].revision, 9);
    remote.task.abort();
    let _ = host.host.shutdown.send(true);
    let _ = host.stop.take().unwrap().send(());
}

#[test]
fn damage_requires_successful_accuracy_and_is_single_use_owned_and_rollbackable() {
    let (mut c, key, target) = damage_fixture("Tackle", "Squirtle", false);
    c.battle.as_mut().unwrap().combatants[0].pending_damage = None;
    let mut accuracy = core_accuracy(&key, "Tackle");
    if let Action::Accuracy { target_id, .. } = &mut accuracy {
        *target_id = Some(target);
    }
    let (miss, _) = battle::transition(&c, "player", &accuracy, vec![1; 6], &catalog()).unwrap();
    assert!(miss.battle.unwrap().combatants[0].pending_damage.is_none());
    let (hit, _) = battle::transition(&c, "player", &accuracy, vec![6; 6], &catalog()).unwrap();
    assert!(
        hit.battle.as_ref().unwrap().combatants[0]
            .pending_damage
            .as_ref()
            .unwrap()
            .critical
    );
    let action = Action::Damage {
        combatant_id: key.clone(),
        sequence: 8,
        pool_modifier: 0,
        success_modifier: 0,
        damage_modifier: 0,
        reason: "Attack".into(),
    };
    assert_eq!(
        battle::transition(&hit, "other", &action, vec![6; 6], &catalog())
            .unwrap_err()
            .code,
        "permission"
    );
    assert_eq!(
        battle::transition(&hit, "player", &damage_action(&key), vec![6; 6], &catalog())
            .unwrap_err()
            .code,
        "stale"
    );
    let count = crate::rules::dice_count(&hit, &action, &catalog()).unwrap();
    let (mut after, _) =
        battle::transition(&hit, "player", &action, vec![6; count as usize], &catalog()).unwrap();
    assert!(battle::transition(
        &after,
        "player",
        &action,
        vec![6; count as usize],
        &catalog()
    )
    .is_err());
    after.revision = 8;
    let (restored, _) = battle::transition(
        &after,
        "gm",
        &Action::RollbackBattle {
            event_sequence: 8,
            reason: "Correction".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap();
    assert_eq!(restored.battle.as_ref().unwrap().combatants[1].hp, 10);
    assert!(restored.battle.unwrap().combatants[0]
        .pending_damage
        .is_some());
    let (fixed, fixed_key, _) = damage_fixture("Dragon Rage", "Squirtle", false);
    assert_eq!(
        crate::rules::dice_count(&fixed, &damage_action(&fixed_key), &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
    let target_sheet = c.battle.as_ref().unwrap().combatants[1].sheet_id.clone();
    c.sheets.get_mut(&target_sheet).unwrap().kind = "trainer".into();
    let trainer_key = c.battle.as_ref().unwrap().combatants[0].id.clone();
    let trainer_target = c.battle.as_ref().unwrap().combatants[1].id.clone();
    c.battle.as_mut().unwrap().combatants[0].pending_damage = Some(PendingDamage {
        sequence: 7,
        target_id: trainer_target,
        move_name: "Tackle".into(),
        critical: false,
    });
    assert_eq!(
        crate::rules::damage_pool(&c, &trainer_key, 7, 0, &catalog())
            .err()
            .unwrap()
            .code,
        "manual_rule"
    );
}

#[test]
fn battle_validation_toggle_allows_exceptions_but_preserves_permissions() {
    let (mut c, key) = core_battle("Tackle");
    let p = &mut c.battle.as_mut().unwrap().combatants[0];
    p.actions = 5;
    p.used_moves.push("Tackle".into());
    let roll = core_accuracy(&key, "Tackle");
    assert!(crate::rules::dice_count(&c, &roll, &catalog()).is_err());
    let config = Action::BattleConfig {
        automatic_validation: false,
        reason: "Allow a special extra action".into(),
    };
    assert!(battle::transition(&c, "player", &config, vec![], &catalog()).is_err());
    let (manual, _) = battle::transition(&c, "gm", &config, vec![], &catalog()).unwrap();
    assert_eq!(
        crate::rules::dice_count(&manual, &roll, &catalog()).unwrap(),
        6
    );
    let (rolled, _) = battle::transition(&manual, "player", &roll, vec![6; 6], &catalog()).unwrap();
    assert_eq!(rolled.battle.as_ref().unwrap().combatants[0].actions, 6);
    assert!(battle::transition(&manual, "stranger", &roll, vec![6; 6], &catalog()).is_err());
    let bad = Action::Roll {
        combatant_id: key,
        dice: 101,
        reason: "Too many dice".into(),
        use_action: false,
    };
    assert!(battle::transition(&manual, "player", &bad, vec![6; 101], &catalog()).is_err());
    let (automatic, _) = battle::transition(
        &rolled,
        "gm",
        &Action::BattleConfig {
            automatic_validation: true,
            reason: "Restore checks".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap();
    assert!(crate::rules::dice_count(&automatic, &roll, &catalog()).is_err());
}

#[test]
fn battle_only_stats_feed_rolls_and_do_not_change_permanent_attributes() {
    let (c, key) = core_battle("Tackle");
    let sheet_id = c.battle.as_ref().unwrap().combatants[0].sheet_id.clone();
    let correction = Action::BattleStats {
        combatant_id: key.clone(),
        stats: BTreeMap::from([("Strength".into(), 8)]),
        skills: BTreeMap::from([("Athletic".into(), 3)]),
        max_hp: 20,
        reason: "Correct imported combat values".into(),
    };
    assert!(battle::transition(&c, "player", &correction, vec![], &catalog()).is_err());
    let (corrected, _) = battle::transition(&c, "gm", &correction, vec![], &catalog()).unwrap();
    assert_eq!(
        crate::rules::dice_count(&corrected, &core_accuracy(&key, "Tackle"), &catalog()).unwrap(),
        11
    );
    assert_eq!(corrected.sheets[&sheet_id].stats["Strength"], 4);
    let heal = Action::Resolve {
        combatant_id: key.clone(),
        target_id: key.clone(),
        move_name: String::new(),
        dice: 0,
        hp_delta: 10,
        condition: None,
        use_action: false,
        reason: "Heal battle HP".into(),
    };
    let (healed, _) = battle::transition(&corrected, "gm", &heal, vec![], &catalog()).unwrap();
    assert_eq!(healed.battle.as_ref().unwrap().combatants[0].hp, 20);
    let (ended, _) =
        battle::transition(&healed, "gm", &Action::EndBattle, vec![], &catalog()).unwrap();
    assert_eq!(ended.sheets[&sheet_id].hp, 10);
    assert_eq!(ended.sheets[&sheet_id].max_hp, 10);
    assert_eq!(ended.sheets[&sheet_id].stats["Strength"], 4);
    let reset = Action::BattleStats {
        combatant_id: key.clone(),
        stats: BTreeMap::new(),
        skills: BTreeMap::new(),
        max_hp: 10,
        reason: "Clear correction".into(),
    };
    let (reset, _) = battle::transition(&corrected, "gm", &reset, vec![], &catalog()).unwrap();
    assert_eq!(
        crate::rules::dice_count(&reset, &core_accuracy(&key, "Tackle"), &catalog()).unwrap(),
        6
    );
}

#[test]
fn battle_rollback_restores_actions_rounds_and_end_state_without_rewinding_revisions() {
    let (c, key) = core_battle("Tackle");
    let undo = Action::RollbackBattle {
        event_sequence: 1,
        reason: "Wrong action selected".into(),
    };
    let (rolled, _) = battle::transition(
        &c,
        "player",
        &core_accuracy(&key, "Tackle"),
        vec![6; 6],
        &catalog(),
    )
    .unwrap();
    assert!(battle::transition(&rolled, "player", &undo, vec![], &catalog()).is_err());
    let (restored, _) = battle::transition(&rolled, "gm", &undo, vec![], &catalog()).unwrap();
    assert_eq!(restored.battle.as_ref().unwrap().combatants[0].actions, 0);
    assert!(restored.battle.as_ref().unwrap().combatants[0]
        .used_moves
        .is_empty());
    let (round, _) =
        battle::transition(&rolled, "gm", &Action::NextRound, vec![], &catalog()).unwrap();
    let (round_undo, _) = battle::transition(&round, "gm", &undo, vec![], &catalog()).unwrap();
    assert_eq!(round_undo.battle.as_ref().unwrap().round, 1);
    assert_eq!(round_undo.battle.as_ref().unwrap().combatants[0].actions, 1);
    let hit = Action::Resolve {
        combatant_id: key.clone(),
        target_id: key.clone(),
        move_name: String::new(),
        dice: 0,
        hp_delta: -3,
        condition: Some("Burn".into()),
        use_action: false,
        reason: "Damage".into(),
    };
    let (hurt, _) = battle::transition(&rolled, "gm", &hit, vec![], &catalog()).unwrap();
    let (ended, _) =
        battle::transition(&hurt, "gm", &Action::EndBattle, vec![], &catalog()).unwrap();
    let sheet = &hurt.battle.as_ref().unwrap().combatants[0].sheet_id;
    assert_eq!(ended.sheets[sheet].hp, 7);
    let (reopened, _) = battle::transition(&ended, "gm", &undo, vec![], &catalog()).unwrap();
    assert!(reopened.battle.as_ref().unwrap().active);
    assert_eq!(reopened.battle.as_ref().unwrap().combatants[0].hp, 7);
    assert_eq!(reopened.sheets[sheet].hp, 10);
    assert!(reopened.finished_battles.is_empty());
    assert!(reopened.sheets[sheet].revision > ended.sheets[sheet].revision);
    // This assertion exercises legacy reward invalidation of undo history;
    // strict workflow rewards are covered by the progression socket tests.
    let mut legacy_reward = ended.clone();
    let t = crate::workflow_tests::trainer("Rookie");
    let reward_id = t.id.clone();
    legacy_reward.sheets.insert(t.id.clone(), t);
    let (rewarded, _) = battle::transition(
        &legacy_reward,
        "gm",
        &Action::Reward {
            sheet_id: reward_id,
            xp: 1,
            reason: "Reward".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap();
    assert!(battle::transition(&rewarded, "gm", &undo, vec![], &catalog()).is_err());
    let mut bounded = c;
    for _ in 0..25 {
        bounded = battle::transition(
            &bounded,
            "gm",
            &Action::SetPhase {
                phase: "A new phase".into(),
            },
            vec![],
            &catalog(),
        )
        .unwrap()
        .0;
    }
    assert_eq!(bounded.battle_undo.len(), 20);
    let restored: Campaign =
        serde_json::from_str(&serde_json::to_string(&bounded).unwrap()).unwrap();
    assert!(battle::transition(&restored, "gm", &undo, vec![], &catalog()).is_ok());
}

#[test]
fn core_initiative_uses_numeric_die_and_is_not_an_action() {
    let (c, key) = core_battle("Tackle");
    let action = Action::Initiative { combatant_id: key };
    let (next, _) = battle::transition(&c, "player", &action, vec![6], &catalog()).unwrap();
    let p = &next.battle.as_ref().unwrap().combatants[0];
    assert_eq!(p.initiative, 11); // Dexterity 3 + Alert 2 + die 6 (p.56).
    assert_eq!(p.actions, 0);
    assert!(battle::transition(&next, "player", &action, vec![1], &catalog()).is_err());
    assert!(battle::transition(&next, "gm", &action, vec![1], &catalog()).is_ok());
}

#[test]
fn core_accuracy_uses_attribute_choice_and_counts_failed_attempts() {
    let (c, key) = core_battle("Tackle");
    let action = core_accuracy(&key, "Tackle");
    assert_eq!(
        crate::rules::dice_count(&c, &action, &catalog()).unwrap(),
        6
    ); // max(Dexterity, Strength) + Athletic (p.57).
    let (failed, text) = battle::transition(&c, "player", &action, vec![1; 6], &catalog()).unwrap();
    assert!(text.contains("accuracy failed"));
    assert_eq!(failed.battle.as_ref().unwrap().combatants[0].actions, 1);
    assert!(battle::transition(&failed, "player", &action, vec![6; 6], &catalog()).is_err());
    let (round, _) =
        battle::transition(&failed, "gm", &Action::NextRound, vec![], &catalog()).unwrap();
    assert!(battle::transition(&round, "player", &action, vec![6; 6], &catalog()).is_ok());
    let mut exhausted = round;
    exhausted.battle.as_mut().unwrap().combatants[0].actions = 5;
    assert!(crate::rules::dice_count(&exhausted, &action, &catalog()).is_err());
}

#[test]
fn core_low_accuracy_and_pain_remove_successes_not_dice() {
    let (mut c, key) = core_battle("Thunder");
    c.battle.as_mut().unwrap().combatants[0].hp = 5;
    let action = core_accuracy(&key, "Thunder");
    assert_eq!(
        crate::rules::dice_count(&c, &action, &catalog()).unwrap(),
        6
    );
    let (_, text) =
        battle::transition(&c, "player", &action, vec![4, 5, 6, 4, 1, 2], &catalog()).unwrap();
    assert!(text.contains("= 1; requires 1: accuracy succeeded")); // 4 raw - 1 pain - 2 low accuracy (pp.62,79).
    assert_eq!(crate::rules::pain(1, 10), 2);
    assert_eq!(crate::rules::pain(6, 10), 0);
    assert_eq!(crate::rules::pain(1, 1), 0);
    let mut modified = action;
    if let Action::Accuracy {
        success_modifier, ..
    } = &mut modified
    {
        *success_modifier = 1;
    }
    assert_eq!(
        battle::transition(&c, "player", &modified, vec![6; 6], &catalog())
            .unwrap_err()
            .code,
        "permission"
    );
}

#[test]
fn core_critical_margins_and_never_miss_still_require_accuracy() {
    let (c, key) = core_battle("Slash");
    let (_, text) = battle::transition(
        &c,
        "player",
        &core_accuracy(&key, "Slash"),
        vec![4, 5, 6, 1, 1, 1],
        &catalog(),
    )
    .unwrap();
    assert!(text.contains("critical hit")); // HighCritical: requirement + 2 (p.79).
    let (mut c, key) = core_battle("Swift");
    c.battle.as_mut().unwrap().combatants[0].actions = 2;
    let (_, text) = battle::transition(
        &c,
        "player",
        &core_accuracy(&key, "Swift"),
        vec![1; 6],
        &catalog(),
    )
    .unwrap();
    assert!(text.contains("critical failure")); // Never Miss prevents Evasion, not roll failure (p.80).
    let (c, key) = core_battle("Protect");
    assert_eq!(
        crate::rules::dice_count(&c, &core_accuracy(&key, "Protect"), &catalog()).unwrap(),
        6
    ); // Will source is now supported.
}

#[test]
fn core_vitality_accuracy_is_exempt_from_pain() {
    let (mut c, key) = core_battle("Recycle");
    let p = &mut c.battle.as_mut().unwrap().combatants[0];
    p.hp = 1;
    let sheet = c.sheets.get_mut(&p.sheet_id).unwrap();
    sheet.stats.insert("Vitality".into(), 3);
    sheet.skills.insert("Nature".into(), 3);
    let (_, text) = battle::transition(
        &c,
        "player",
        &core_accuracy(&key, "Recycle"),
        vec![4, 1, 1, 1, 1, 1],
        &catalog(),
    )
    .unwrap();
    assert!(text.contains("= 1; requires 1: accuracy succeeded")); // pp.62,434.
}

#[test]
fn core_master_rank_bonuses_and_legacy_upgrade_are_explicit() {
    let (mut c, key) = core_battle("Tackle");
    let sheet = c.battle.as_ref().unwrap().combatants[0].sheet_id.clone();
    c.sheets.get_mut(&sheet).unwrap().rank = "Master".into();
    assert_eq!(
        crate::rules::dice_count(&c, &core_accuracy(&key, "Tackle"), &catalog()).unwrap(),
        8
    ); // p.31.
    let (c, _) = battle::transition(
        &c,
        "player",
        &Action::Initiative {
            combatant_id: key.clone(),
        },
        vec![1],
        &catalog(),
    )
    .unwrap();
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].initiative, 9);
    let mut legacy = c.clone();
    legacy.engine = LEGACY_ENGINE.into();
    assert!(crate::rules::dice_count(&legacy, &core_accuracy(&key, "Tackle"), &catalog()).is_err());
    assert!(battle::transition(&legacy, "gm", &Action::UpgradeRules, vec![], &catalog()).is_err());
    let (legacy, _) =
        battle::transition(&legacy, "gm", &Action::EndBattle, vec![], &catalog()).unwrap();
    assert!(
        battle::transition(&legacy, "player", &Action::UpgradeRules, vec![], &catalog()).is_err()
    );
    let (upgraded, _) =
        battle::transition(&legacy, "gm", &Action::UpgradeRules, vec![], &catalog()).unwrap();
    assert_eq!(upgraded.engine, ENGINE);
    assert_eq!(
        serde_json::to_value(&upgraded.finished_battles).unwrap(),
        serde_json::to_value(&legacy.finished_battles).unwrap()
    );
}

#[test]
fn catalog_preserves_forms_and_rejects_invalid_sheet() {
    let c = catalog();
    let summaries = c.bootstrap().unwrap();
    let species = summaries["species"].as_array().unwrap();
    assert_eq!(species.len(), 1200);
    let forms: Vec<_> = species.iter().filter(|s| s["DexID"] == "0916").collect();
    assert_eq!(forms.len(), 2);
    assert_ne!(forms[0]["AppId"], forms[1]["AppId"]);
    let mut p = pokemon("player", "trainer");
    c.validate(&p).unwrap();
    p.stats.insert("Strength".into(), 99);
    assert_eq!(c.validate(&p).unwrap_err().code, "validation");
    p.stats.insert("Strength".into(), 2);
    p.moves.push("Not a move".into());
    assert!(c.validate(&p).is_err());
}
#[test]
fn saving_is_persistent_transactional_and_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let c = catalog();
    let initial = trainer(&store.player_id);
    let request = request(initial.clone());
    let saved = store.save(request.clone(), &c).unwrap();
    assert_eq!(saved.revision, 1);
    assert_eq!(store.save(request, &c).unwrap(), saved);
    assert_eq!(store.library().unwrap().history.len(), 1);
    let mut edit = saved.clone();
    edit.stats.insert("Strength".into(), 1);
    let mut req = request_for(edit);
    req.xp_cost = 3;
    let trained = store.save(req.clone(), &c).unwrap();
    assert_eq!(trained.xp, 7);
    assert_eq!(store.save(req, &c).unwrap().xp, 7);
    assert_eq!(
        store.save(request_for(saved), &c).unwrap_err().code,
        "stale"
    );
    let player = store.player_id.clone();
    drop(store);
    let reopened = Store::open(dir.path()).unwrap();
    assert_eq!(reopened.player_id, player);
    assert_eq!(reopened.sheet(&initial.id).unwrap().unwrap().xp, 7);
}
fn request_for(s: Sheet) -> SaveRequest {
    request(s)
}
#[test]
fn invalid_saves_leave_database_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    let c = catalog();
    let mut sheet = trainer(&s.player_id);
    sheet.hp = 100;
    assert!(s.save(request(sheet), &c).is_err());
    assert!(s.library().unwrap().sheets.is_empty());
    let p = pokemon(&s.player_id, "missing-trainer");
    assert!(s.save(request(p), &c).is_err());
    assert!(s.library().unwrap().history.is_empty());
}
#[test]
fn merge_preserves_independent_changes_and_requires_conflict_resolution() {
    let base = trainer("player");
    let mut local = base.clone();
    let mut remote = base.clone();
    local.notes = "offline notes".into();
    remote.hp = 5;
    remote.revision = 2;
    let merged = sync::merge(&base, &local, &remote, &BTreeMap::new()).unwrap();
    assert_eq!(merged.notes, "offline notes");
    assert_eq!(merged.hp, 5);
    local.hp = 8;
    let error = sync::merge(&base, &local, &remote, &BTreeMap::new()).unwrap_err();
    assert_eq!(error.conflicts, vec!["/hp"]);
    let choices = BTreeMap::from([("/hp".into(), "remote".into())]);
    assert_eq!(sync::merge(&base, &local, &remote, &choices).unwrap().hp, 5);
}
#[test]
fn ownership_cannot_change_during_sync() {
    let base = trainer("player");
    let mut forged = base.clone();
    forged.owner_id = "attacker".into();
    assert!(sync::merge(&base, &forged, &base, &BTreeMap::new()).is_err());
    assert!(battle::transition(
        &campaign("gm"),
        "attacker",
        &sync_action(vec![base]),
        vec![],
        &catalog()
    )
    .is_err());
}
#[test]
fn battle_lifecycle_persists_resources_and_freezes_participating_sheets() {
    let c = catalog();
    let t = trainer("gm");
    let p = pokemon("gm", &t.id);
    let (campaign, _) = battle::transition(
        &campaign("gm"),
        "gm",
        &sync_action(vec![t, p.clone()]),
        vec![],
        &c,
    )
    .unwrap();
    let (active, _) = battle::transition(
        &campaign,
        "gm",
        &Action::StartBattle {
            sheet_ids: vec![p.id.clone()],
            sides: BTreeMap::new(),
        },
        vec![],
        &c,
    )
    .unwrap();
    assert_eq!(
        battle::transition(&active, "gm", &sync_action(vec![p.clone()]), vec![], &c)
            .unwrap_err()
            .code,
        "battle_locked"
    );
    let key = active.battle.as_ref().unwrap().combatants[0].id.clone();
    let action = Action::Resolve {
        combatant_id: key.clone(),
        target_id: key,
        move_name: "Thunderbolt".into(),
        dice: 2,
        hp_delta: -3,
        condition: Some("Paralyze".into()),
        use_action: true,
        reason: "GM ruling".into(),
    };
    let (resolved, _) = battle::transition(&active, "gm", &action, vec![2, 6], &c).unwrap();
    assert_eq!(resolved.battle.as_ref().unwrap().combatants[0].actions, 1);
    let (round, _) = battle::transition(&resolved, "gm", &Action::NextRound, vec![], &c).unwrap();
    assert_eq!(round.battle.as_ref().unwrap().round, 2);
    assert_eq!(round.battle.as_ref().unwrap().combatants[0].actions, 0);
    let (ended, _) = battle::transition(&round, "gm", &Action::EndBattle, vec![], &c).unwrap();
    assert_eq!(ended.sheets[&p.id].hp, 7);
    assert_eq!(ended.sheets[&p.id].conditions, vec!["Paralyze"]);
}
#[test]
fn player_cannot_resolve_damage_or_roll_another_players_combatant() {
    let c = catalog();
    let t = trainer("gm");
    let mut campaign = campaign("gm");
    campaign.sheets.insert(t.id.clone(), t.clone());
    let (active, _) = battle::transition(
        &campaign,
        "gm",
        &Action::StartBattle {
            sheet_ids: vec![t.id],
            sides: BTreeMap::new(),
        },
        vec![],
        &c,
    )
    .unwrap();
    let key = active.battle.as_ref().unwrap().combatants[0].id.clone();
    assert!(battle::transition(
        &active,
        "player",
        &Action::Roll {
            combatant_id: key,
            dice: 1,
            reason: "roll".into(),
            use_action: true
        },
        vec![3],
        &c
    )
    .is_err());
    assert!(battle::transition(&active, "player", &Action::NextRound, vec![], &c).is_err());
}
#[test]
fn invalid_resolution_is_atomic() {
    let c = catalog();
    let t = trainer("gm");
    let mut state = campaign("gm");
    state.sheets.insert(t.id.clone(), t.clone());
    let (active, _) = battle::transition(
        &state,
        "gm",
        &Action::StartBattle {
            sheet_ids: vec![t.id],
            sides: BTreeMap::new(),
        },
        vec![],
        &c,
    )
    .unwrap();
    let key = active.battle.as_ref().unwrap().combatants[0].id.clone();
    assert!(battle::transition(
        &active,
        "gm",
        &Action::Resolve {
            combatant_id: key,
            target_id: "missing".into(),
            move_name: "".into(),
            dice: 0,
            hp_delta: -2,
            condition: None,
            use_action: true,
            reason: "manual".into()
        },
        vec![],
        &c
    )
    .is_err());
    assert_eq!(active.battle.unwrap().combatants[0].actions, 0);
}
#[test]
fn export_import_creates_distinct_ids_and_preserves_trainer_links() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    let c = catalog();
    let t = s.save(request(trainer(&s.player_id)), &c).unwrap();
    let p = s.save(request(pokemon(&s.player_id, &t.id)), &c).unwrap();
    let export = serde_json::json!({"version":1,"library":s.library().unwrap()}).to_string();
    s.import_library(&export, &c).unwrap();
    let sheets = s.library().unwrap().sheets;
    assert_eq!(sheets.len(), 4);
    let copied = sheets
        .iter()
        .find(|sheet| sheet.kind == "pokemon" && sheet.id != p.id)
        .unwrap();
    assert_ne!(copied.trainer_id, Some(t.id));
    assert!(sheets
        .iter()
        .any(|sheet| Some(&sheet.id) == copied.trainer_id.as_ref()));
}
#[test]
fn backup_contains_committed_library_and_campaign() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    s.save(request(trainer(&s.player_id)), &catalog()).unwrap();
    let c = campaign(&s.player_id);
    s.put_campaign(&c).unwrap();
    let backup = s.backup().unwrap();
    let restored = Store::open(std::path::Path::new(&backup)).unwrap();
    assert_eq!(restored.library().unwrap().sheets.len(), 1);
    assert_eq!(restored.campaigns().unwrap()[0].id, c.id);
}

#[test]
fn backup_restore_replaces_both_databases_and_rejects_path_traversal() {
    let directory = tempfile::tempdir().unwrap();
    let mut s = Store::open(directory.path()).unwrap();
    let t = s.save(request(trainer(&s.player_id)), &catalog()).unwrap();
    let c = campaign(&s.player_id);
    s.put_campaign(&c).unwrap();
    let backup = s.backup().unwrap();
    let backup_id = std::path::Path::new(&backup)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    let mut updated = t.clone();
    updated.name = "Changed".into();
    s.save(request(updated), &catalog()).unwrap();
    let mut changed = c.clone();
    changed.revision = 99;
    s.put_campaign(&changed).unwrap();
    s.restore_backup(backup_id).unwrap();
    assert_eq!(s.sheet(&t.id).unwrap().unwrap().name, t.name);
    assert_eq!(s.campaigns().unwrap()[0].revision, 0);
    assert_eq!(
        s.restore_backup("../../outside").unwrap_err().code,
        "validation"
    );
}

#[test]
fn reconciliation_preserves_offline_notes_and_applies_rewards_once() {
    let directory = tempfile::tempdir().unwrap();
    let mut s = Store::open(directory.path()).unwrap();
    let catalog = catalog();
    let mut t = crate::workflow_tests::trainer("Rookie");
    t.owner_id = s.player_id.clone();
    t.xp = 10;
    let mut initial = campaign("gm");
    initial.sheets.insert(t.id.clone(), t.clone());
    let accepted = initial;
    s.accept_campaign(
        &accepted,
        std::slice::from_ref(&t.id),
        &BTreeMap::new(),
        &catalog,
    )
    .unwrap();
    let mut local = s.sheet(&t.id).unwrap().unwrap();
    local.notes = "Offline training notes".into();
    s.save(request(local), &catalog).unwrap();
    let (rewarded, _) = battle::transition(
        &accepted,
        "gm",
        &Action::Reward {
            sheet_id: t.id.clone(),
            xp: 5,
            reason: "Reward".into(),
        },
        vec![],
        &catalog,
    )
    .unwrap();
    for _ in 0..2 {
        s.accept_campaign(
            &rewarded,
            std::slice::from_ref(&t.id),
            &BTreeMap::new(),
            &catalog,
        )
        .unwrap();
    }
    let final_sheet = s.sheet(&t.id).unwrap().unwrap();
    assert_eq!(final_sheet.notes, "Offline training notes");
    assert_eq!(final_sheet.xp, 10);
    assert_eq!(final_sheet.workflow.unwrap().training_points, 5);
    assert_eq!(
        s.sync_payload(&[t.id], "another-campaign", BTreeMap::new())
            .unwrap_err()
            .code,
        "membership"
    );
}

#[test]
fn reconciliation_can_remove_inventory_keys_without_null_quantities() {
    let mut base = trainer("owner");
    base.inventory.insert("Potion".into(), 1);
    let mut local = base.clone();
    local.inventory.remove("Potion");
    let mut remote = base.clone();
    remote.notes = "Campaign note".into();
    let merged = sync::merge(&base, &local, &remote, &BTreeMap::new()).unwrap();
    assert!(!merged.inventory.contains_key("Potion"));
    assert_eq!(merged.notes, "Campaign note");
}
#[tokio::test]
async fn two_clients_reconnect_and_duplicate_commands_do_not_duplicate_rewards() {
    let host_dir = tempfile::tempdir().unwrap();
    let player_dir = tempfile::tempdir().unwrap();
    let mut host_store = Store::open(host_dir.path()).unwrap();
    let gm = host_store.player_id.clone();
    let c = campaign(&gm);
    host_store.put_campaign(&c).unwrap();
    let shared = Arc::new(Mutex::new(host_store));
    let player_store = Arc::new(Mutex::new(Store::open(player_dir.path()).unwrap()));
    let player = player_store.lock().unwrap().player_id.clone();
    let catalog = Arc::new(catalog());
    let notify: room::Notify = Arc::new(|_, _| {});
    let mut hosted = room::start(
        shared.clone(),
        catalog.clone(),
        c.id.clone(),
        0,
        notify.clone(),
    )
    .await
    .unwrap();
    let address = hosted.info.address.replace("<host-ip>", "127.0.0.1");
    let join_request = JoinRequest {
        address: address.clone(),
        token: hosted.info.token.clone(),
    };
    let (remote, _) = room::join(
        player_store.clone(),
        catalog.clone(),
        join_request.clone(),
        notify.clone(),
    )
    .await
    .unwrap();
    let mut t = crate::workflow_tests::trainer("Rookie");
    t.owner_id = player.clone();
    let mut source = t.clone();
    source.workflow = Some(crate::creation::build(&source, &catalog));
    source.workflow.as_mut().unwrap().allocations.clear();
    source.workflow.as_mut().unwrap().origin = "starter".into();
    let req = WorkflowRequest {
        operation_id: id(),
        expected_revision: 0,
        trainer_revision: None,
        rules_version: 1,
        catalog_hash: catalog.hash.clone(),
        kind: "create".into(),
        sheet: source,
        move_name: String::new(),
        replacement: String::new(),
        method: String::new(),
        target_species: String::new(),
        amount: 0,
        reason: "Socket fixture".into(),
    };
    let t = crate::creation::apply(None, None, &req, &catalog, None).unwrap();
    let cmd = Command {
        id: id(),
        expected_revision: 0,
        action: sync_action(vec![t.clone()]),
    };
    let (tx, rx) = tokio::sync::oneshot::channel();
    remote.sender.send((cmd, tx)).await.unwrap();
    let imported = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(imported.revision, 1);
    let reward = Command {
        id: id(),
        expected_revision: 1,
        action: Action::Reward {
            sheet_id: t.id.clone(),
            xp: 5,
            reason: "Session reward".into(),
        },
    };
    assert_eq!(
        hosted.host.apply(&gm, &reward).unwrap().sheets[&t.id]
            .workflow
            .as_ref()
            .unwrap()
            .training_points,
        5
    );
    assert_eq!(
        hosted.host.apply(&gm, &reward).unwrap().sheets[&t.id]
            .workflow
            .as_ref()
            .unwrap()
            .training_points,
        5
    );
    assert_eq!(
        hosted.host.apply(&player, &reward).unwrap_err().code,
        "ownership"
    );
    remote.task.abort();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let (reconnected, snapshot) = room::join(
        player_store.clone(),
        catalog.clone(),
        JoinRequest {
            address: join_request.address.replace("127.0.0.1", "localhost"),
            token: join_request.token,
        },
        notify.clone(),
    )
    .await
    .unwrap();
    assert_eq!(
        snapshot.sheets[&t.id]
            .workflow
            .as_ref()
            .unwrap()
            .training_points,
        5
    );
    assert!(hosted.host.members().unwrap().contains(&player));
    let unknown = Command {
        id: id(),
        expected_revision: snapshot.revision,
        action: Action::AssignGm { player_id: id() },
    };
    assert_eq!(
        hosted.host.apply(&gm, &unknown).unwrap_err().code,
        "validation"
    );
    let assignment = Command {
        id: id(),
        expected_revision: snapshot.revision,
        action: Action::AssignGm {
            player_id: player.clone(),
        },
    };
    assert_eq!(
        hosted.host.apply(&player, &assignment).unwrap_err().code,
        "permission"
    );
    let appointed = hosted.host.apply(&gm, &assignment).unwrap();
    assert_eq!(appointed.host_id(), gm);
    assert_eq!(appointed.gm_id, player);
    assert_eq!(
        hosted.host.apply(&gm, &assignment).unwrap().revision,
        appointed.revision
    );
    let gm_reward = Command {
        id: id(),
        expected_revision: appointed.revision,
        action: Action::Reward {
            sheet_id: t.id.clone(),
            xp: 2,
            reason: "Remote GM reward".into(),
        },
    };
    assert_eq!(
        hosted.host.apply(&gm, &gm_reward).unwrap_err().code,
        "permission"
    );
    let (tx, rx) = tokio::sync::oneshot::channel();
    reconnected.sender.send((gm_reward, tx)).await.unwrap();
    let validated = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(
        validated.sheets[&t.id]
            .workflow
            .as_ref()
            .unwrap()
            .training_points,
        7
    );
    assert_eq!(shared.lock().unwrap().campaigns().unwrap()[0].gm_id, player);
    reconnected.task.abort();
    let token = hosted.info.token.clone();
    let _ = hosted.host.shutdown.send(true);
    let _ = hosted.stop.take().unwrap().send(());
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let mut restarted = room::start(shared, catalog, c.id, 0, notify).await.unwrap();
    assert_eq!(restarted.info.token, token);
    let reopened = restarted.host.snapshot().unwrap();
    assert_eq!(reopened.revision, validated.revision);
    assert_eq!(reopened.host_id(), gm);
    assert_eq!(reopened.gm_id, player);
    let _ = restarted.host.shutdown.send(true);
    let _ = restarted.stop.take().unwrap().send(());
}
#[tokio::test]
async fn host_automatically_accepts_player_accuracy_and_rollback_keeps_receipts_and_dice() {
    let host_dir = tempfile::tempdir().unwrap();
    let player_dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(host_dir.path()).unwrap();
    let gm = store.player_id.clone();
    let c = campaign(&gm);
    store.put_campaign(&c).unwrap();
    let shared = Arc::new(Mutex::new(store));
    let catalog = Arc::new(catalog());
    let notify: room::Notify = Arc::new(|_, _| {});
    let mut host = room::start(shared.clone(), catalog.clone(), c.id, 0, notify.clone())
        .await
        .unwrap();
    let player_store = Arc::new(Mutex::new(Store::open(player_dir.path()).unwrap()));
    let player = player_store.lock().unwrap().player_id.clone();
    let (remote, _) = room::join(
        player_store,
        catalog.clone(),
        JoinRequest {
            address: host.info.address.replace("<host-ip>", "127.0.0.1"),
            token: host.info.token.clone(),
        },
        notify,
    )
    .await
    .unwrap();
    let t = trainer(&player);
    let mut p = pokemon(&player, &t.id);
    p.stats.insert("Dexterity".into(), 3);
    p.skills.insert("Channel".into(), 3);
    host.host
        .apply(
            &player,
            &Command {
                id: id(),
                expected_revision: 0,
                action: sync_action(vec![t, p.clone()]),
            },
        )
        .unwrap();
    // Seed previously accepted manual sheets for the isolated core-roll test.
    let mut seeded = host.host.snapshot().unwrap();
    seeded.engine = PREVIOUS_ENGINE.into();
    for sheet in seeded.sheets.values_mut() {
        let mut build = crate::creation::build(sheet, &catalog);
        build.status = "exception".into();
        sheet.workflow = Some(build);
    }
    shared.lock().unwrap().put_campaign(&seeded).unwrap();
    let started = host
        .host
        .apply(
            &gm,
            &Command {
                id: id(),
                expected_revision: 1,
                action: Action::StartBattle {
                    sheet_ids: vec![p.id],
                    sides: BTreeMap::new(),
                },
            },
        )
        .unwrap();
    let key = started.battle.as_ref().unwrap().combatants[0].id.clone();
    let roll = Command {
        id: id(),
        expected_revision: 2,
        action: core_accuracy(&key, "Thunderbolt"),
    };
    let (tx, rx) = tokio::sync::oneshot::channel();
    remote.sender.send((roll.clone(), tx)).await.unwrap();
    let rolled = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(rolled.battle.as_ref().unwrap().combatants[0].actions, 1);
    assert_eq!(rolled.events.last().unwrap().dice.len(), 6);
    let rollback = Command {
        id: id(),
        expected_revision: 3,
        action: Action::RollbackBattle {
            event_sequence: 3,
            reason: "Correct mistaken roll".into(),
        },
    };
    assert_eq!(
        host.host.apply(&player, &rollback).unwrap_err().code,
        "permission"
    );
    let restored = host.host.apply(&gm, &rollback).unwrap();
    assert_eq!(restored.revision, 4);
    assert_eq!(restored.battle.as_ref().unwrap().combatants[0].actions, 0);
    assert_eq!(restored.events[2].dice, rolled.events[2].dice);
    assert_eq!(restored.events.len(), 4);
    assert!(restored.battle_undo.is_empty());
    let phase = host
        .host
        .apply(
            &gm,
            &Command {
                id: id(),
                expected_revision: 4,
                action: Action::SetPhase {
                    phase: "New step after rollback".into(),
                },
            },
        )
        .unwrap();
    let stale_undo = Command {
        id: id(),
        expected_revision: phase.revision,
        action: Action::RollbackBattle {
            event_sequence: 3,
            reason: "Stale recovery selection".into(),
        },
    };
    assert_eq!(host.host.apply(&gm, &stale_undo).unwrap_err().code, "stale");
    let retry = host.host.apply(&player, &roll).unwrap();
    assert_eq!(retry.revision, 5);
    assert_eq!(retry.battle.as_ref().unwrap().combatants[0].actions, 0);
    assert_eq!(host.host.apply(&gm, &rollback).unwrap().revision, 5);
    assert_eq!(
        shared.lock().unwrap().campaigns().unwrap()[0].events.len(),
        5
    );
    remote.task.abort();
    let _ = host.host.shutdown.send(true);
    let _ = host.stop.take().unwrap().send(());
}

#[tokio::test]
async fn malformed_and_mismatched_handshakes_are_rejected() {
    use futures_util::{SinkExt, StreamExt};
    let directory = tempfile::tempdir().unwrap();
    let mut s = Store::open(directory.path()).unwrap();
    let c = campaign(&s.player_id);
    s.put_campaign(&c).unwrap();
    let mut host = room::start(
        Arc::new(Mutex::new(s)),
        Arc::new(catalog()),
        c.id,
        0,
        Arc::new(|_, _| {}),
    )
    .await
    .unwrap();
    let address = host.info.address.replace("<host-ip>", "127.0.0.1");
    let (mut socket, _) = tokio_tungstenite::connect_async(&address).await.unwrap();
    let hello = Hello {
        player_id: id(),
        token: host.info.token.clone(),
        credential: None,
        protocol: 999,
        build_version: env!("CARGO_PKG_VERSION").into(),
        catalog_hash: catalog().hash,
        engine: ENGINE.into(),
    };
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            serde_json::to_string(&hello).unwrap().into(),
        ))
        .await
        .unwrap();
    let text = socket.next().await.unwrap().unwrap().into_text().unwrap();
    let error: ServerMessage = serde_json::from_str(&text).unwrap();
    assert!(matches!(error,ServerMessage::Error{error:AppError{code,..},..} if code=="version"));
    let _ = host.host.shutdown.send(true);
    let _ = host.stop.take().unwrap().send(());
}
