use super::*;
use crate::combat::{
    field::{self, EncounterField, FieldCommand, InitialFieldEffect, Weather},
    special,
};
fn effect(kind: &str, rounds: Option<u32>) -> InitialFieldEffect {
    InitialFieldEffect {
        kind: kind.into(),
        rounds,
        parameter: 1,
    }
}
fn config() -> EncounterField {
    EncounterField {
        night: true,
        weather: Some(Weather::Rain),
        indoors: false,
        sunlight: true,
        terrain: Some(effect("Grassy Terrain", None)),
        effects: vec![effect("Gravity", None), effect("Trick Room", Some(2))],
    }
}
fn fresh() -> (Campaign, Vec<String>) {
    let (mut c, _, _) = fixture("Tackle");
    let ids = c
        .battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .map(|p| p.sheet_id.clone())
        .collect::<Vec<_>>();
    c.battle = None;
    for id in &ids {
        c.sheets.get_mut(id).unwrap().ability.clear();
    }
    (c, ids)
}
fn start(c: &Campaign, ids: &[String], setup: Option<EncounterField>) -> Action {
    let _ = c;
    Action::StartBattle {
        environment: setup,
        sheet_ids: ids.to_vec(),
        sides: ids
            .iter()
            .enumerate()
            .map(|(i, id)| {
                (
                    id.clone(),
                    if i == 0 { "Players" } else { "Opponents" }.into(),
                )
            })
            .collect(),
    }
}
#[test]
fn environment_start_snapshots_all_weather_without_spending_actions_or_touching_wild_encounters() {
    let (c, ids) = fresh();
    let before = serde_json::to_value(&c).unwrap();
    for weather in [
        None,
        Some(Weather::Sunny),
        Some(Weather::Rain),
        Some(Weather::Sandstorm),
        Some(Weather::Snow),
        Some(Weather::Hail),
        Some(Weather::Desolate),
        Some(Weather::Typhoon),
        Some(Weather::StrongWinds),
    ] {
        let mut setup = config();
        setup.weather = weather;
        let (n, log) =
            battle::transition(&c, "gm", &start(&c, &ids, Some(setup)), vec![], &catalog())
                .unwrap();
        let f = &n
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .battlefield;
        assert_eq!(f.natural, weather);
        assert!(f.night && f.initial.is_some());
        assert!(log.contains("GM starting battlefield"));
        assert!(n
            .battle
            .as_ref()
            .unwrap()
            .combatants
            .iter()
            .all(|p| p.actions == 0 && p.reactions == 0));
        assert_eq!(
            serde_json::to_value(&n.training_encounters).unwrap(),
            serde_json::to_value(&c.training_encounters).unwrap()
        );
        let restored: Campaign = serde_json::from_value(serde_json::to_value(&n).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(&restored.battle).unwrap(),
            serde_json::to_value(&n.battle).unwrap()
        );
    }
    assert_eq!(serde_json::to_value(&c).unwrap(), before);
}
#[test]
fn environment_rejects_invalid_setup_unauthorized_starts_and_historical_engines_atomically() {
    let (c, ids) = fresh();
    let action = start(&c, &ids, Some(config()));
    assert!(battle::transition(&c, "player", &action, vec![], &catalog()).is_err());
    let mut old = c.clone();
    old.engine = GROUP_RULES_ENGINE.into();
    assert!(battle::transition(&old, "gm", &action, vec![], &catalog()).is_err());
    for mode in 0..6 {
        let mut s = config();
        match mode {
            0 => s.effects.push(effect("Gravity", None)),
            1 => s.effects[0].kind = "invented".into(),
            2 => s.effects[0].rounds = Some(0),
            3 => s.terrain = Some(effect("Trick Room", None)),
            4 => s.indoors = true,
            _ => s.effects[0].rounds = Some(1001),
        };
        assert!(
            battle::transition(&c, "gm", &start(&c, &ids, Some(s)), vec![], &catalog()).is_err()
        );
    }
    let n = battle::transition(&old, "gm", &start(&old, &ids, None), vec![], &catalog())
        .unwrap()
        .0;
    assert!(
        n.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .house_rules
    );
    assert!(n
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .battlefield
        .initial
        .is_none());
    let mut json = serde_json::to_value(start(&c, &ids, None)).unwrap();
    json.as_object_mut().unwrap().remove("environment");
    let historical: Action = serde_json::from_value(json).unwrap();
    assert!(field::start_setup(&historical).is_none());
}
#[test]
fn environment_rooms_gravity_expiry_and_live_changes_preserve_initial_snapshot_on_rollback() {
    let (c, ids) = fresh();
    let mut c = battle::transition(
        &c,
        "gm",
        &start(&c, &ids, Some(config())),
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    let b = c.battle.as_mut().unwrap();
    let high = b.combatants[0].id.clone();
    let low = b.combatants[1].id.clone();
    for (i, p) in b.combatants.iter_mut().enumerate() {
        p.initiative_rolled = true;
        p.initiative = 10 - i as i32;
    }
    let c = battle::transition(&c, "gm", &Action::StartTurns, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(
        c.battle.as_ref().unwrap().flow.as_ref().unwrap().order[0],
        low
    );
    let mut p = c.battle.as_ref().unwrap().combatants[0].clone();
    p.selected_ability = "Levitate".into();
    assert!(field::grounded(&c, &p, &catalog()));
    let change = Action::Battlefield {
        command: FieldCommand::Night { night: false },
        reason: "Dawn during the match".into(),
    };
    let changed = battle::transition(&c, "gm", &change, vec![], &catalog())
        .unwrap()
        .0;
    let initial = changed
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .battlefield
        .initial
        .as_ref()
        .unwrap();
    assert!(initial.night);
    let undo = Action::RollbackBattle {
        event_sequence: changed.battle_undo.last().unwrap().event_sequence,
        reason: "Undo dawn".into(),
    };
    let mut restored = battle::transition(&changed, "gm", &undo, vec![], &catalog())
        .unwrap()
        .0;
    assert!(
        restored
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .battlefield
            .night
    );
    field::expire(&mut restored).unwrap();
    assert!(field::has(&restored, "Trick Room", &p));
    field::expire(&mut restored).unwrap();
    assert!(!field::has(&restored, "Trick Room", &p));
    assert!(field::has(&restored, "Gravity", &p));
    let _ = high;
}
#[test]
fn environment_entry_abilities_resolve_against_preexisting_conditions() {
    let (mut c, ids) = fresh();
    c.sheets.get_mut(&ids[0]).unwrap().ability = "Drizzle".into();
    let mut setup = config();
    setup.weather = Some(Weather::Sunny);
    let n = battle::transition(&c, "gm", &start(&c, &ids, Some(setup)), vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(field::weather(&n, None), Some(Weather::Rain));
    assert_eq!(
        n.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .battlefield
            .initial
            .as_ref()
            .unwrap()
            .weather,
        Some(Weather::Sunny)
    );
    c.sheets.get_mut(&ids[0]).unwrap().ability = "Electric Surge".into();
    let n = battle::transition(
        &c,
        "gm",
        &start(&c, &ids, Some(config())),
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    assert!(n
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .battlefield
        .events
        .iter()
        .any(|e| e.kind == "terrain-ruling"));
}
#[test]
fn environment_moonlight_uses_live_night_with_explicit_overrides_and_historical_fallback() {
    let (mut c, a, t) = fixture("Tackle");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .enabled = true;
    field::prepare(&mut c, &config()).unwrap();
    assert!(field::moonlight_night(&c, false, None));
    assert!(!field::moonlight_night(&c, false, Some(false)));
    let mut q = hit(&c, &a, &t, "Tackle")
        .battle
        .unwrap()
        .flow
        .unwrap()
        .queue[0]
        .clone();
    q.move_name = "Moonlight".into();
    q.target_id = a.clone();
    q.targets.clear();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .special
        .enabled = true;
    for (override_night, expected) in [(None, 5), (Some(false), 1), (Some(true), 5)] {
        let mut n = c.clone();
        let p = n
            .battle
            .as_mut()
            .unwrap()
            .combatants
            .iter_mut()
            .find(|p| p.id == a)
            .unwrap();
        p.hp = 5;
        p.max_hp = 20;
        p.runtime.healed = 0;
        let command = Action::SpecialMove {
            command: special::SpecialCommand::Parameters {
                combatant_id: a.clone(),
                weight_kg: None,
                lift_kg: None,
                night: false,
                night_override: override_night,
            },
            reason: "Explicit Moonlight context".into(),
        };
        let mut n = battle::transition(&n, "gm", &command, vec![], &catalog())
            .unwrap()
            .0;
        assert!(special::effects(&mut n, &q, &[], &catalog()).unwrap());
        assert_eq!(
            n.battle
                .as_ref()
                .unwrap()
                .combatants
                .iter()
                .find(|p| p.id == a)
                .unwrap()
                .hp,
            5 + expected
        );
    }
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .initial = None;
    assert!(!field::moonlight_night(&c, false, None));
    assert!(field::moonlight_night(&c, true, None));
}
