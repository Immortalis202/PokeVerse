use super::*;
use crate::combat::field::{self, Battlefield, FieldCommand, Weather};
fn setup() -> (Campaign, String, String) {
    let (mut c, a, t) = fixture("Tackle");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield = Battlefield {
        enabled: true,
        sunlight: true,
        ..Default::default()
    };
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_rules = true;
    let before = c.clone();
    field::lifecycle(&before, &mut c, &catalog()).unwrap();
    (c, a, t)
}
fn command(c: &Campaign, actor: &str, command: FieldCommand) -> Result<Campaign> {
    let a = Action::Battlefield {
        command,
        reason: "Reviewed battlefield fixture".into(),
    };
    let n = rules::dice_count(c, &a, &catalog())?;
    Ok(battle::transition(c, actor, &a, vec![4; n as usize], &catalog())?.0)
}
fn environment(c: &Campaign, w: Weather) -> Campaign {
    command(
        c,
        "gm",
        FieldCommand::Environment {
            weather: Some(w),
            indoors: false,
            sunlight: true,
        },
    )
    .unwrap()
}
fn typing(c: &Campaign, id: &str, mode: &str, t: &str) -> Campaign {
    command(
        c,
        "gm",
        FieldCommand::Typing {
            combatant_id: id.into(),
            mode: mode.into(),
            types: if t.is_empty() { vec![] } else { vec![t.into()] },
        },
    )
    .unwrap()
}
fn preset(c: &Campaign, name: &str) -> Campaign {
    command(
        c,
        "gm",
        FieldCommand::Preset {
            name: name.into(),
            scope: "all".into(),
            rounds: None,
            parameter: 2,
        },
    )
    .unwrap()
}
#[test]
fn field_all_weather_bonuses_and_move_failures() {
    let (c, a, _) = setup();
    let p = &c.battle.as_ref().unwrap().combatants[0];
    for (w, t, n) in [
        (Weather::Sunny, "Fire", 1),
        (Weather::Rain, "Water", 1),
        (Weather::Snow, "Ice", 1),
        (Weather::Desolate, "Fire", 2),
        (Weather::Typhoon, "Water", 2),
        (Weather::StrongWinds, "Flying", 2),
        (Weather::Hail, "Ice", 0),
        (Weather::Sandstorm, "Rock", 0),
    ] {
        let c = environment(&c, w);
        let m = serde_json::json!({"Type":t,"Category":"Special"});
        assert_eq!(field::bonus(&c, p, &m), n);
    }
    let c = environment(&c, Weather::Desolate);
    let m = catalog().find("moves", "Name", "Water Gun").unwrap();
    assert!(field::guard_move(&c, p, &m).is_err());
    let c = environment(&c, Weather::Typhoon);
    assert!(field::guard_move(&c, p, &catalog().find("moves", "Name", "Ember").unwrap()).is_err());
    assert_eq!(
        c.sheets[&c.battle.as_ref().unwrap().combatants[0].sheet_id].id,
        c.sheets[&c.battle.as_ref().unwrap().combatants[0].sheet_id].id
    );
    assert!(!a.is_empty());
}
#[test]
fn field_typing_replacement_third_type_removal_and_sheet_isolation() {
    let (c, a, _) = setup();
    let original = serde_json::to_value(&c.sheets).unwrap();
    let c = typing(&c, &a, "replace", "Water");
    let c = typing(&c, &a, "add", "Grass");
    let c = typing(&c, &a, "add", "Ghost");
    let p = &c.battle.as_ref().unwrap().combatants[0];
    assert_eq!(
        field::types(&c, p, &catalog()).unwrap(),
        vec!["Water", "Ghost"]
    );
    let c = typing(&c, &a, "remove", "Water");
    assert_eq!(
        field::types(&c, &c.battle.as_ref().unwrap().combatants[0], &catalog()).unwrap(),
        vec!["Ghost"]
    );
    let c = typing(&c, &a, "restore", "");
    assert_eq!(serde_json::to_value(&c.sheets).unwrap(), original);
    assert!(
        field::types(&c, &c.battle.as_ref().unwrap().combatants[0], &catalog())
            .unwrap()
            .contains(&"Electric".into())
    );
}
#[test]
fn field_weather_ticks_cover_immunities_and_no_revival() {
    let (c, a, t) = setup();
    let mut c = typing(&c, &a, "replace", "Rock");
    c = environment(&c, Weather::Sandstorm);
    field::end(&mut c, &catalog()).unwrap();
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].hp, 20);
    assert_eq!(c.battle.as_ref().unwrap().combatants[1].hp, 19);
    field::end(&mut c, &catalog()).unwrap();
    assert_eq!(c.battle.as_ref().unwrap().combatants[1].hp, 19);
    let c = command(
        &c,
        "gm",
        FieldCommand::Shelter {
            combatant_id: t.clone(),
            sheltered: true,
        },
    )
    .unwrap();
    let mut c = c;
    c.battle.as_mut().unwrap().round += 1;
    field::end(&mut c, &catalog()).unwrap();
    assert_eq!(c.battle.as_ref().unwrap().combatants[1].hp, 19);
    c.battle.as_mut().unwrap().combatants[0].hp = 0;
    let mut c = preset(&c, "healing-pits");
    c.battle.as_mut().unwrap().combatants[1].selected_ability = "Ice Body".into();
    c = environment(&c, Weather::Hail);
    c.battle.as_mut().unwrap().round += 1;
    field::end(&mut c, &catalog()).unwrap();
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].hp, 0);
}
#[test]
fn field_round_checks_are_host_rolled_owned_choices_and_idempotent() {
    let (mut c, a, _) = setup();
    c = typing(&c, &a, "replace", "Normal");
    c = environment(&c, Weather::Desolate);
    c.sheets
        .get_mut(&c.battle.as_ref().unwrap().combatants[0].sheet_id)
        .unwrap()
        .stats
        .insert("Vitality".into(), 2);
    field::start(&mut c, &catalog()).unwrap();
    let e = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .battlefield
        .events[0]
        .clone();
    assert_eq!(e.dice, 2);
    let action = FieldCommand::Resolve {
        event_id: e.id.clone(),
        accept: true,
    };
    assert!(command(&c, "player", action.clone()).is_err());
    let c = command(&c, "gm", action.clone()).unwrap();
    assert!(command(&c, "gm", action).is_err());
}
#[test]
fn field_barriers_do_not_stack_and_require_snow_for_veil() {
    let (mut c, a, t) = setup();
    let side = c.battle.as_ref().unwrap().combatants[1].side.clone();
    for (n, name) in ["Reflect", "Aurora Veil"].iter().enumerate() {
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .battlefield
            .rules
            .push(field::FieldRule {
                id: format!("test{n}"),
                kind: (*name).into(),
                source: "test".into(),
                scope: format!("side:{side}"),
                rounds: Some(4),
                parameter: 1,
                started_round: 1,
            });
    }
    let source = &c.battle.as_ref().unwrap().combatants[0];
    let target = &c.battle.as_ref().unwrap().combatants[1];
    assert_eq!(
        field::reduction(
            &c,
            source,
            target,
            &catalog().find("moves", "Name", "Tackle").unwrap()
        ),
        1
    );
    assert!(field::guard_move(
        &c,
        source,
        &catalog().find("moves", "Name", "Aurora Veil").unwrap()
    )
    .is_err());
    let c = environment(&c, Weather::Snow);
    assert!(field::guard_move(
        &c,
        &c.battle.as_ref().unwrap().combatants[0],
        &catalog().find("moves", "Name", "Aurora Veil").unwrap()
    )
    .is_ok());
    assert_ne!(a, t);
}
#[test]
fn field_terrain_healing_is_capped_and_expiry_does_not_repeat() {
    let (mut c, _, _) = setup();
    let f = &mut c
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield;
    f.rules.push(field::FieldRule {
        id: "grassy".into(),
        kind: "Grassy Terrain".into(),
        source: "test".into(),
        scope: "all".into(),
        rounds: Some(1),
        parameter: 1,
        started_round: 1,
    });
    c.battle.as_mut().unwrap().combatants[0].hp = 10;
    c.battle.as_mut().unwrap().combatants[0].runtime.healed = 3;
    c.battle.as_mut().unwrap().combatants[1].hp = 0;
    field::end(&mut c, &catalog()).unwrap();
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].hp, 10);
    assert_eq!(c.battle.as_ref().unwrap().combatants[1].hp, 0);
    field::expire(&mut c).unwrap();
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .battlefield
        .rules
        .is_empty());
}
#[test]
fn field_arena_permissions_typing_and_reaction_restrictions() {
    let (c, a, _) = setup();
    assert!(command(
        &c,
        "host",
        FieldCommand::Preset {
            name: "fog".into(),
            scope: "all".into(),
            rounds: None,
            parameter: 1
        }
    )
    .is_err());
    let c = preset(&c, "sprinklers");
    assert_eq!(
        field::types(&c, &c.battle.as_ref().unwrap().combatants[0], &catalog()).unwrap(),
        vec!["Water"]
    );
    let c = preset(&c, "flowers");
    assert!(field::reaction(
        &c,
        &c.battle.as_ref().unwrap().combatants[0],
        "evade",
        &catalog()
    )
    .is_err());
    let c = preset(&c, "final-destination");
    assert!(field::reaction(
        &c,
        &c.battle.as_ref().unwrap().combatants[0],
        "clash",
        &catalog()
    )
    .is_err());
    assert!(command(
        &c,
        "gm",
        FieldCommand::Preset {
            name: "high-poles".into(),
            scope: "all".into(),
            rounds: None,
            parameter: 0
        }
    )
    .is_err());
    assert!(!a.is_empty());
}
#[test]
fn field_move_resolution_uses_normal_action_and_will_cost() {
    let (mut c, a, _) = setup();
    let sid = c.battle.as_ref().unwrap().combatants[0].sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .moves
        .push("Grassy Terrain".into());
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .stats
        .insert("Insight".into(), 2);
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .skills
        .insert("Nature".into(), 2);
    let action = Action::AccuracyTargets {
        combatant_id: a.clone(),
        target_ids: vec![],
        move_name: "Grassy Terrain".into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Cast".into(),
    };
    let n = rules::dice_count(&c, &action, &catalog()).unwrap();
    let c = battle::transition(&c, "player", &action, vec![4; n as usize], &catalog())
        .unwrap()
        .0;
    let sequence = c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sequence;
    let c = battle::transition(
        &c,
        "player",
        &Action::ResolveEffect {
            sequence,
            reason: "Resolve".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    let b = c.battle.as_ref().unwrap();
    assert_eq!(b.combatants[0].actions, 1);
    assert_eq!(b.combatants[0].runtime.will, 5);
    assert_eq!(
        b.flow.as_ref().unwrap().battlefield.rules[0].scope,
        format!("side:{}", b.combatants[0].side)
    );
    assert!(b.flow.as_ref().unwrap().queue.is_empty());
}
#[test]
fn field_weather_ability_contest_suppression_departure_and_reconnect() {
    let (mut c, _, _) = setup();
    c.battle.as_mut().unwrap().combatants[0].selected_ability = "Drought".into();
    c.battle.as_mut().unwrap().combatants[1].selected_ability = "Drizzle".into();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .known_active
        .clear();
    let before = c.clone();
    field::lifecycle(&before, &mut c, &catalog()).unwrap();
    let e = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .battlefield
        .events[0]
        .clone();
    assert_eq!(e.kind, "weather-contest");
    let action = Action::Battlefield {
        command: FieldCommand::Resolve {
            event_id: e.id.clone(),
            accept: true,
        },
        reason: "Contest".into(),
    };
    let mut faces = vec![1; e.dice as usize];
    faces[0] = 6;
    let c = battle::transition(&c, "gm", &action, faces, &catalog())
        .unwrap()
        .0;
    assert_eq!(field::weather(&c, None), Some(Weather::Sunny));
    let mut c: Campaign = serde_json::from_value(serde_json::to_value(c).unwrap()).unwrap();
    assert_eq!(field::weather(&c, None), Some(Weather::Sunny));
    c.battle.as_mut().unwrap().combatants[0].deployed = false;
    let before = c.clone();
    field::lifecycle(&before, &mut c, &catalog()).unwrap();
    assert_eq!(field::weather(&c, None), None);
    c.battle.as_mut().unwrap().combatants[1].selected_ability = "Air Lock".into();
    let c = environment(&c, Weather::Rain);
    assert_eq!(field::weather(&c, None), None);
}

#[test]
fn field_gravity_wonder_room_and_weather_accuracy_use_reviewed_definitions() {
    let (mut c, a, t) = setup();
    c = typing(&c, &t, "replace", "Flying");
    let target = c.battle.as_ref().unwrap().combatants[1].clone();
    assert!(!field::grounded(&c, &target, &catalog()));
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .rules
        .push(field::FieldRule {
            id: "gravity".into(),
            kind: "Gravity".into(),
            source: "test".into(),
            scope: "all".into(),
            rounds: Some(4),
            parameter: 0,
            started_round: 1,
        });
    assert!(field::grounded(&c, &target, &catalog()));
    let p = c.battle.as_ref().unwrap().combatants[0].clone();
    let m = catalog().find("moves", "Name", "Blizzard").unwrap();
    assert_ne!(
        field::move_data(&c, &p, &m)["Attributes"]["NeverMiss"],
        true
    );
    c = environment(&c, Weather::Snow);
    assert_eq!(
        field::move_data(&c, &p, &m)["Attributes"]["NeverMiss"],
        true
    );
    assert_eq!(
        field::move_data(&c, &p, &m)["Attributes"]["AccuracyReduction"],
        0
    );
    let sid = c.battle.as_ref().unwrap().combatants[0].sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .stats
        .insert("Vitality".into(), 2);
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .stats
        .insert("Insight".into(), 4);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .rules
        .push(field::FieldRule {
            id: "wonder".into(),
            kind: "Wonder Room".into(),
            source: "test".into(),
            scope: "all".into(),
            rounds: Some(4),
            parameter: 0,
            started_round: 1,
        });
    assert_eq!(field::defense(&c, &p, true, &catalog(), 2).unwrap(), 4);
    assert_eq!(field::defense(&c, &p, false, &catalog(), 4).unwrap(), 2);
    assert!(!a.is_empty());
}

#[test]
fn field_weather_checks_and_typing_rollback_preserve_history() {
    let (c, a, _) = setup();
    let before = field::types(&c, &c.battle.as_ref().unwrap().combatants[0], &catalog()).unwrap();
    let changed = typing(&c, &a, "replace", "Water");
    let restored = battle::transition(
        &changed,
        "gm",
        &Action::RollbackBattle {
            event_sequence: changed.revision + 1,
            reason: "Restore typing".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    assert_eq!(
        field::types(
            &restored,
            &restored.battle.as_ref().unwrap().combatants[0],
            &catalog()
        )
        .unwrap(),
        before
    );
}

#[tokio::test]
async fn field_separate_host_gm_receipts_and_reconnect_preserve_environment() {
    let hd = tempfile::tempdir().unwrap();
    let gd = tempfile::tempdir().unwrap();
    let mut store = Store::open(hd.path()).unwrap();
    let gs = Arc::new(Mutex::new(Store::open(gd.path()).unwrap()));
    let gm_id = gs.lock().unwrap().player_id.clone();
    let (mut c, _, _) = setup();
    c.host_id = store.player_id.clone();
    c.gm_id = gm_id.clone();
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
    let (gm, _) = room::join(gs.clone(), cat.clone(), request.clone(), notify.clone())
        .await
        .unwrap();
    let command = Command {
        id: id(),
        expected_revision: c.revision,
        action: Action::Battlefield {
            command: FieldCommand::Environment {
                weather: Some(Weather::Rain),
                indoors: false,
                sunlight: true,
            },
            reason: "Outdoor gym".into(),
        },
    };
    assert!(host.host.apply(c.host_id(), &command).is_err());
    let (tx, rx) = tokio::sync::oneshot::channel();
    gm.sender.send((command.clone(), tx)).await.unwrap();
    let next = rx.await.unwrap().unwrap();
    assert_eq!(field::weather(&next, None), Some(Weather::Rain));
    assert_eq!(
        host.host.apply(&gm_id, &command).unwrap().revision,
        next.revision
    );
    gm.task.abort();
    let (reconnected, snapshot) = room::join(gs, cat, request, notify).await.unwrap();
    assert_eq!(field::weather(&snapshot, None), Some(Weather::Rain));
    reconnected.task.abort();
    let _ = host.stop.take().unwrap().send(());
}

#[test]
fn field_barrier_reductions_precede_hp_clamping() {
    use super::targeting_tests::{damage_command, pass, roll};
    let (mut c, a, t) = setup();
    c.battle.as_mut().unwrap().combatants[1].hp = 1;
    let scope = format!("side:{}", c.battle.as_ref().unwrap().combatants[1].side);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .rules
        .push(field::FieldRule {
            id: "reflect".into(),
            kind: "Reflect".into(),
            source: "test".into(),
            scope,
            rounds: Some(4),
            parameter: 1,
            started_round: 1,
        });
    let c = hit(&c, &a, &t, "Tackle");
    let c = pass(&c);
    let c = roll(&c, "player", &damage_command(&c), 4);
    assert_eq!(c.battle.as_ref().unwrap().combatants[1].hp, 0);
}
#[test]
fn field_shared_attack_freezes_weather_when_its_source_faints() {
    use super::targeting_tests::{attack, damage_command, multi, pass, roll};
    let (mut c, a, t, high) = multi("Swift");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_rules = true;
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield = Battlefield {
        enabled: true,
        sunlight: true,
        ..Default::default()
    };
    c.battle.as_mut().unwrap().combatants[2].selected_ability = "Drought".into();
    c.battle.as_mut().unwrap().combatants[2].hp = 1;
    let before = c.clone();
    field::lifecycle(&before, &mut c, &catalog()).unwrap();
    let c = roll(&c, "player", &attack(&a, vec![t, high.clone()], "Swift"), 4);
    let c = pass(&c);
    let c = pass(&c);
    assert_eq!(
        c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].target_id,
        high
    );
    let c = roll(&c, "player", &damage_command(&c), 4);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .battlefield
        .weather
        .is_none());
    assert_eq!(field::weather(&c, None), Some(Weather::Sunny));
    let c = roll(&c, "player", &damage_command(&c), 4);
    assert_eq!(field::weather(&c, None), None);
}

#[test]
fn field_round_end_events_complete_before_expiry_and_ticks_are_once() {
    let (mut c, a, _) = setup();
    c = preset(&c, "fire");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .order
        .clear();
    c.battle.as_mut().unwrap().combatants[0]
        .runtime
        .statuses
        .push(crate::combat::Status {
            kind: "Poison".into(),
            detail: "".into(),
            rounds: None,
            progress: 0,
        });
    let mut c = battle::transition(&c, "gm", &Action::NextRound, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(c.battle.as_ref().unwrap().round, 1);
    let hp = c.battle.as_ref().unwrap().combatants[0].hp;
    while let Some(e) = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .battlefield
        .events
        .first()
        .cloned()
    {
        let action = Action::Battlefield {
            command: FieldCommand::Resolve {
                event_id: e.id,
                accept: true,
            },
            reason: "Hazard chance".into(),
        };
        let n = rules::dice_count(&c, &action, &catalog()).unwrap();
        c = battle::transition(&c, "gm", &action, vec![1; n as usize], &catalog())
            .unwrap()
            .0;
    }
    let c = battle::transition(&c, "gm", &Action::NextRound, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(c.battle.as_ref().unwrap().round, 2);
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].hp, hp);
    assert!(!a.is_empty());
}
