use super::targeting_tests::{damage_command, pass, roll};
use super::*;
use crate::combat::{abilities, events};

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
fn setup(name: &str, source: &str, target: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = super::special_tests::setup(name);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .events
        .enabled = true;
    pm(&mut c, &a).selected_ability = source.into();
    pm(&mut c, &t).selected_ability = target.into();
    (c, a, t)
}

#[test]
fn events_stench_rolls_one_chance_die_on_nonranged_physical_hits() {
    for (name, expected) in [("Tackle", true), ("Swift", false), ("Ember", false)] {
        let (c, a, t) = setup(name, "", "Stench");
        let c = pass(&hit(&c, &a, &t, name));
        let c = roll(&c, "player", &damage_command(&c), 4);
        let pending = &c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events;
        assert_eq!(!pending.is_empty(), expected, "{name}");
        if expected {
            assert_eq!(pending[0].ability, "Stench");
            let command = Action::ResolveAbility {
                event_id: pending[0].id.clone(),
                choice: "roll".into(),
                reason: "Resolve battle Stench".into(),
            };
            assert_eq!(rules::dice_count(&c, &command, &catalog()).unwrap(), 1);
            assert!(battle::transition(&c, "player", &command, vec![6], &catalog()).is_err());
            let c = roll(&c, "other", &command, 6);
            assert!(p(&c, &a)
                .runtime
                .statuses
                .iter()
                .any(|s| s.kind == "Flinch"));
        }
    }
}

#[test]
fn events_stench_respects_long_reach_and_wonder_skin() {
    for (source, expected_dice) in [("Long Reach", None), ("Wonder Skin", Some(0))] {
        let (c, a, t) = setup("Tackle", source, "Stench");
        let c = pass(&hit(&c, &a, &t, "Tackle"));
        let c = roll(&c, "player", &damage_command(&c), 4);
        let pending = &c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events;
        if let Some(n) = expected_dice {
            assert_eq!(abilities::dice(&c, &pending[0].id, "roll").unwrap(), n);
        } else {
            assert!(pending.is_empty());
        }
    }
}

#[test]
fn events_pressure_halves_remaining_will_rounding_up_without_refreshing() {
    for will in 0..=6 {
        let (mut c, a, t) = setup("Tackle", "Pressure", "");
        pm(&mut c, &t).runtime.will = will;
        events::lifecycle(&mut c).unwrap();
        assert_eq!(p(&c, &t).runtime.will, will / 2);
        assert_eq!(p(&c, &a).runtime.will, 6);
        events::lifecycle(&mut c).unwrap();
        assert_eq!(p(&c, &t).runtime.will, will / 2);
        assert_eq!(
            c.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .events
                .history
                .len(),
            1
        );
    }
}

#[test]
fn events_pressure_requires_entry_and_respects_explicit_range() {
    let (mut c, a, t) = setup("Tackle", "Pressure", "");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ranges
        .insert(a.clone(), vec![]);
    events::lifecycle(&mut c).unwrap();
    assert_eq!(p(&c, &t).runtime.will, 6);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ranges
        .remove(&a);
    events::lifecycle(&mut c).unwrap();
    assert_eq!(p(&c, &t).runtime.will, 6);
    pm(&mut c, &a).deployed = false;
    events::lifecycle(&mut c).unwrap();
    pm(&mut c, &a).deployed = true;
    events::lifecycle(&mut c).unwrap();
    assert_eq!(p(&c, &t).runtime.will, 3);
}

#[test]
fn events_pressure_does_not_stack_overlapping_entries_and_rearms_after_all_leave() {
    let (mut c, a, t) = setup("Tackle", "Pressure", "");
    events::lifecycle(&mut c).unwrap();
    let mut other = p(&c, &a).clone();
    other.id = crate::model::id();
    let second = other.id.clone();
    c.battle.as_mut().unwrap().combatants.push(other);
    events::lifecycle(&mut c).unwrap();
    assert_eq!(p(&c, &t).runtime.will, 3);
    pm(&mut c, &a).deployed = false;
    events::lifecycle(&mut c).unwrap();
    assert_eq!(p(&c, &t).runtime.will, 3);
    pm(&mut c, &second).deployed = false;
    events::lifecycle(&mut c).unwrap();
    pm(&mut c, &a).deployed = true;
    events::lifecycle(&mut c).unwrap();
    assert_eq!(p(&c, &t).runtime.will, 1);
}

fn mixed() -> (Campaign, String, String, String) {
    let (mut c, a, t, h) = super::targeting_tests::multi("Discharge");
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.events.enabled = true;
    f.reviewed_rules = true;
    f.extra.enabled = true;
    f.tactics.enabled = true;
    f.ability_rules = true;
    pm(&mut c, &h).selected_ability = "Tangled Feet".into();
    pm(&mut c, &h).runtime.statuses.push(crate::combat::Status {
        kind: "Confusion".into(),
        detail: "".into(),
        rounds: None,
        progress: 0,
    });
    (c, a, t, h)
}

#[test]
fn events_tangled_feet_mixed_targets_can_miss_only_the_confused_holder() {
    let (c, a, t, h) = mixed();
    let action = super::targeting_tests::attack(&a, vec![t.clone(), h.clone()], "Discharge");
    let n = rules::dice_count(&c, &action, &catalog()).unwrap() as usize;
    let mut faces = vec![1; n];
    faces[..2].fill(4);
    let mut c = battle::transition(&c, "player", &action, faces, &catalog())
        .unwrap()
        .0;
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    assert_eq!(
        q.targets
            .iter()
            .find(|p| p.target_id == h)
            .unwrap()
            .successes,
        Some(0)
    );
    assert!(q.targets.iter().find(|p| p.target_id == h).unwrap().avoided);
    assert_eq!(q.target_id, t);
    assert_eq!(q.successes, 2);
    while c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
        c = pass(&c);
    }
    c = roll(&c, "player", &damage_command(&c), 4);
    assert!(p(&c, &t).hp < 20);
    assert_eq!(p(&c, &h).hp, 20);
}

#[test]
fn events_tangled_feet_freezes_individual_criticals_and_keen_eye_prevents_the_penalty() {
    for keen in [false, true] {
        let (mut c, a, t, h) = mixed();
        if keen {
            pm(&mut c, &a).selected_ability = "Keen Eye".into();
        }
        let action = super::targeting_tests::attack(&a, vec![t.clone(), h.clone()], "Discharge");
        let n = rules::dice_count(&c, &action, &catalog()).unwrap() as usize;
        let mut faces = vec![1; n];
        faces[..4].fill(4);
        let mut c = battle::transition(&c, "player", &action, faces, &catalog())
            .unwrap()
            .0;
        let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
        assert_eq!(
            q.targets
                .iter()
                .find(|p| p.target_id == t)
                .unwrap()
                .critical,
            Some(true)
        );
        assert_eq!(
            q.targets
                .iter()
                .find(|p| p.target_id == h)
                .unwrap()
                .critical,
            Some(keen)
        );
        pm(&mut c, &h).runtime.statuses.clear();
        while c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
            let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
            assert_eq!(q.critical, q.target_id != h || keen);
            c = pass(&c);
        }
    }
}

#[test]
fn events_next_electric_support_move_consumes_charge_without_granting_support_dice() {
    for ability in ["Wind Power", "Electromorphosis"] {
        let (mut c, a, t) = setup("Thunder Wave", ability, "");
        pm(&mut c, &a)
            .runtime
            .ability
            .counters
            .insert("charged-electric".into(), 2);
        let c = hit(&c, &a, &t, "Thunder Wave");
        assert!(!p(&c, &a)
            .runtime
            .ability
            .counters
            .contains_key("charged-electric"));
    }
}

#[test]
fn events_mixed_accuracy_results_round_trip_and_rollback_with_damage() {
    let (c, a, t, h) = mixed();
    let action = super::targeting_tests::attack(&a, vec![t, h], "Discharge");
    let mut faces = vec![1; rules::dice_count(&c, &action, &catalog()).unwrap() as usize];
    faces[..4].fill(4);
    let mut c = battle::transition(&c, "player", &action, faces, &catalog())
        .unwrap()
        .0;
    while c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
        c = pass(&c);
    }
    let serialized = serde_json::to_value(&c).unwrap();
    let c: Campaign = serde_json::from_value(serialized).unwrap();
    let before = serde_json::to_value(&c.battle).unwrap();
    let after = roll(&c, "player", &damage_command(&c), 4);
    let undo = Action::RollbackBattle {
        event_sequence: after.battle_undo.last().unwrap().event_sequence,
        reason: "Restore target-specific success and critical results".into(),
    };
    let restored = decision(&after, "gm", undo);
    assert_eq!(serde_json::to_value(&restored.battle).unwrap(), before);
}

#[test]
fn events_old_target_and_flow_records_load_without_new_rules() {
    let (c, a, t, h) = mixed();
    let action = super::targeting_tests::attack(&a, vec![t, h], "Discharge");
    let c = roll(&c, "player", &action, 6);
    let mut value = serde_json::to_value(&c).unwrap();
    value["battle"]["flow"]
        .as_object_mut()
        .unwrap()
        .remove("events");
    for impact in value["battle"]["flow"]["queue"][0]["targets"]
        .as_array_mut()
        .unwrap()
    {
        impact.as_object_mut().unwrap().remove("successes");
        impact.as_object_mut().unwrap().remove("critical");
    }
    let loaded: Campaign = serde_json::from_value(value).unwrap();
    let f = loaded.battle.as_ref().unwrap().flow.as_ref().unwrap();
    assert!(!f.events.enabled);
    assert!(f.queue[0]
        .targets
        .iter()
        .all(|t| t.successes.is_none() && t.critical.is_none()));
}

#[test]
fn events_new_battles_enable_rules_and_preserve_provenance_engine_behavior() {
    let (c, a, t) = setup("Tackle", "", "");
    let sheets = vec![p(&c, &a).sheet_id.clone(), p(&c, &t).sheet_id.clone()];
    let ended = decision(&c, "gm", Action::EndBattle);
    for (engine, expected) in [
        (ENGINE, true),
        (ITEM_COMPLETION_ENGINE, true),
        (EVENT_ENGINE, true),
        (PROVENANCE_ENGINE, false),
    ] {
        let mut c = ended.clone();
        c.engine = engine.into();
        let c = decision(
            &c,
            "gm",
            Action::StartBattle {
                sheet_ids: sheets.clone(),
                sides: BTreeMap::new(),
            },
        );
        let f = c.battle.as_ref().unwrap().flow.as_ref().unwrap();
        assert_eq!(f.events.enabled, expected);
        assert!(f.provenance_rules && f.sequencing.enabled);
    }
}

#[test]
fn events_historical_mixed_targets_retain_explicit_ruling() {
    let (mut c, a, t, h) = mixed();
    c.engine = PROVENANCE_ENGINE.into();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .events
        .enabled = false;
    let action = super::targeting_tests::attack(&a, vec![t, h], "Discharge");
    let n = rules::dice_count(&c, &action, &catalog()).unwrap();
    let error =
        battle::transition(&c, "player", &action, vec![4; n as usize], &catalog()).unwrap_err();
    assert_eq!(error.code, "manual_rule");
}

#[test]
fn events_dancer_preserves_individual_source_results_without_collapsing_to_last_target() {
    for actions in [0, 2] {
        let (mut c, a, t, h) = mixed();
        pm(&mut c, &a).selected_ability = "Dancer".into();
        pm(&mut c, &a).actions = actions;
        let sid = p(&c, &a).sheet_id.clone();
        c.sheets
            .get_mut(&sid)
            .unwrap()
            .moves
            .push("Teeter Dance".into());
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .sequencing
            .enabled = true;
        crate::combat::sequencing::lifecycle(&mut c, &catalog()).unwrap();
        let action = super::targeting_tests::attack(&a, vec![], "Teeter Dance");
        let mut faces = vec![1; rules::dice_count(&c, &action, &catalog()).unwrap() as usize];
        faces[..2].fill(4);
        let mut c = battle::transition(&c, "player", &action, faces, &catalog())
            .unwrap()
            .0;
        while !c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .queue
            .is_empty()
        {
            if c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
                c = pass(&c);
            } else {
                let sequence = c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sequence;
                c = decision(
                    &c,
                    "gm",
                    Action::AdjudicateEffect {
                        sequence,
                        reason: "Record the unsupported dance added effect explicitly".into(),
                    },
                );
            }
        }
        let event = c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events[0]
            .clone();
        assert_eq!(event.kind, "sequence-dancer");
        let c = decision(
            &c,
            "player",
            Action::ResolveAbility {
                event_id: event.id,
                choice: "copy".into(),
                reason: "Copy the original target-specific accuracy".into(),
            },
        );
        let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
        let normal = q
            .targets
            .iter()
            .find(|target| target.target_id == t)
            .unwrap();
        let confused = q
            .targets
            .iter()
            .find(|target| target.target_id == h)
            .unwrap();
        assert_eq!(normal.successes, Some(2));
        assert_eq!(confused.successes, Some(0));
        assert!(confused.avoided);
        assert_eq!(p(&c, &a).actions, actions + 1);
    }
}

#[tokio::test]
async fn events_room_pressure_entry_retry_reconnect_persistence_and_rollback() {
    async fn send(remote: &room::Remote, command: Command) -> Campaign {
        let (tx, rx) = tokio::sync::oneshot::channel();
        remote.sender.send((command, tx)).await.unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(5), rx)
            .await
            .unwrap()
            .unwrap()
            .unwrap()
    }
    let dirs: Vec<_> = (0..3).map(|_| tempfile::tempdir().unwrap()).collect();
    let stores: Vec<_> = dirs
        .iter()
        .map(|d| Arc::new(Mutex::new(Store::open(d.path()).unwrap())))
        .collect();
    let ids: Vec<_> = stores
        .iter()
        .map(|s| s.lock().unwrap().player_id.clone())
        .collect();
    let (mut c, a, t) = setup("Tackle", "", "");
    c.host_id = ids[0].clone();
    c.gm_id = ids[1].clone();
    pm(&mut c, &a).owner_id = ids[2].clone();
    pm(&mut c, &a).field_slot = Some(0);
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().owner_id = ids[2].clone();
    let mut reserve = p(&c, &a).clone();
    let mut sheet = c.sheets[&sid].clone();
    sheet.id = id();
    sheet.ability = "Pressure".into();
    reserve.id = id();
    reserve.sheet_id = sheet.id.clone();
    reserve.selected_ability = "Pressure".into();
    reserve.deployed = false;
    reserve.field_slot = None;
    let incoming = reserve.id.clone();
    c.sheets.insert(sheet.id.clone(), sheet);
    c.battle.as_mut().unwrap().combatants.push(reserve);
    c.battle.as_mut().unwrap().slots_per_side = 1;
    pm(&mut c, &t).runtime.will = 5;
    let before = serde_json::to_value(&c.battle).unwrap();
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
    let (gm, _) = room::join(stores[1].clone(), cat.clone(), join.clone(), notify.clone())
        .await
        .unwrap();
    let (owner, _) = room::join(stores[2].clone(), cat.clone(), join.clone(), notify.clone())
        .await
        .unwrap();
    let enter = Command {
        id: id(),
        expected_revision: c.revision,
        action: Action::SwitchPokemon {
            outgoing_id: a.clone(),
            incoming_id: incoming.clone(),
            gm_override: false,
            reason: "Owner sends out Pressure".into(),
        },
    };
    assert!(host.host.apply(&ids[0], &enter).is_err());
    let c = send(&owner, enter.clone()).await;
    assert_eq!(p(&c, &t).runtime.will, 2);
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .events
            .history
            .len(),
        1
    );
    assert_eq!(
        host.host.apply(&ids[2], &enter).unwrap().revision,
        c.revision
    );
    let mut stale = enter.clone();
    stale.id = id();
    assert!(host.host.apply(&ids[2], &stale).is_err());
    owner.task.abort();
    let (owner, snapshot) = room::join(stores[2].clone(), cat, join, notify)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(&snapshot.battle).unwrap(),
        serde_json::to_value(&c.battle).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&stores[0].lock().unwrap().campaigns().unwrap()[0].battle).unwrap(),
        serde_json::to_value(&c.battle).unwrap()
    );
    let undo = Command {
        id: id(),
        expected_revision: c.revision,
        action: Action::RollbackBattle {
            event_sequence: c.battle_undo.last().unwrap().event_sequence,
            reason: "Restore Pressure entry and spent Will".into(),
        },
    };
    let restored = send(&gm, undo).await;
    assert_eq!(serde_json::to_value(&restored.battle).unwrap(), before);
    assert_eq!(
        host.host.apply(&ids[2], &enter).unwrap().revision,
        restored.revision
    );
    let retry = Command {
        id: id(),
        expected_revision: restored.revision,
        action: enter.action,
    };
    let retried = send(&owner, retry).await;
    assert_eq!(p(&retried, &t).runtime.will, 2);
    gm.task.abort();
    owner.task.abort();
    let _ = host.stop.take().unwrap().send(());
}
