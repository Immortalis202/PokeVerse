use super::targeting_tests::{damage_command, pass, roll};
use super::*;
use crate::combat::{cure_protection, field, Status};

fn pm<'a>(c: &'a mut Campaign, id: &str) -> &'a mut Combatant {
    c.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == id)
        .unwrap()
}
fn p<'a>(c: &'a Campaign, id: &str) -> &'a Combatant {
    c.battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .find(|p| p.id == id)
        .unwrap()
}
fn setup(name: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = special_tests::setup(name);
    let round = c.battle.as_ref().unwrap().round;
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.compound.enabled = true;
    f.recurring.enabled = true;
    f.cure_protection = true;
    f.direct_status = true;
    f.compound.entered.insert(a.clone(), round);
    f.compound.environments = vec!["City/Building".into()];
    f.status_escalation = true;
    f.reviewed_rules = true;
    f.interaction_rules = true;
    for id in [&a, &t] {
        pm(&mut c, id).hp = 100;
        pm(&mut c, id).max_hp = 100;
    }
    species(&mut c, &t, "Shuckle");
    (c, a, t)
}
fn species(c: &mut Campaign, id: &str, name: &str) {
    let sid = p(c, id).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some(
        catalog().find("species", "Name", name).unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
}
fn q(c: &Campaign) -> crate::combat::QueuedAction {
    c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].clone()
}
fn effect(c: &Campaign) -> Action {
    Action::ResolveEffect {
        sequence: q(c).sequence,
        reason: "Reviewed compound effect".into(),
    }
}
fn pending_attack(c: &Campaign, a: &str, t: &str, name: &str) -> Campaign {
    let mut c = roll(
        c,
        "gm",
        &Action::Accuracy {
            combatant_id: a.into(),
            target_id: Some(t.into()),
            move_name: name.into(),
            pool_modifier: 0,
            success_modifier: 0,
            reason: "Reviewed explicit target".into(),
        },
        4,
    );
    if c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
        c = pass(&c);
    }
    if catalog().find("moves", "Name", name).unwrap()["Category"] != "Support"
        && !(name == "Sparkling Aria"
            && p(&c, t)
                .runtime
                .statuses
                .iter()
                .any(|s| s.kind.starts_with("Burn")))
    {
        c = roll(&c, "gm", &damage_command(&c), 6);
    }
    c
}
fn resolve(c: &Campaign, faces: Vec<u8>) -> Campaign {
    battle::transition(c, "gm", &effect(c), faces, &catalog())
        .unwrap()
        .0
}
fn has(c: &Campaign, id: &str, kind: &str) -> bool {
    p(c, id).runtime.statuses.iter().any(|s| s.kind == kind)
}
fn status(c: &mut Campaign, id: &str, kind: &str) {
    pm(c, id).runtime.statuses.push(Status {
        kind: kind.into(),
        detail: "Fixture".into(),
        rounds: None,
        progress: 0,
        elapsed_rounds: 0,
    });
}

fn cast(c: &Campaign, a: &str, name: &str) -> Campaign {
    let action = Action::AccuracyTargets {
        combatant_id: a.into(),
        target_ids: vec![],
        move_name: name.into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Cast field move".into(),
    };
    let c = roll(c, "gm", &action, 6);
    assert!(q(&c).field_scope.is_some());
    resolve(&c, vec![])
}
fn set(id: &str, kind: &str, detail: &str) -> Action {
    Action::SetStatus {
        combatant_id: id.into(),
        kind: kind.into(),
        detail: detail.into(),
        rounds: None,
        remove: false,
        reason: "Condition fixture".into(),
    }
}
fn next(c: &Campaign) -> Campaign {
    let mut c = c.clone();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .order
        .clear();
    battle::transition(&c, "gm", &Action::NextRound, vec![], &catalog())
        .unwrap()
        .0
}
#[test]
fn cure_protection_safeguard_preserves_existing_conditions_blocks_all_new_conditions_on_own_side() {
    let (mut c, a, t) = setup("Safeguard");
    status(&mut c, &a, "Poison");
    let c = cast(&c, &a, "Safeguard");
    assert!(has(&c, &a, "Poison"));
    assert!(field::has(&c, "Safeguard", p(&c, &a)));
    assert!(!field::has(&c, "Safeguard", p(&c, &t)));
    for kind in [
        "Burn",
        "Burn 2",
        "Burn 3",
        "Poison",
        "Badly Poisoned",
        "Paralysis",
        "Sleep",
        "Confusion",
        "Flinch",
        "Disable",
        "Frozen Solid",
        "In Love",
    ] {
        let detail = if kind == "Disable" {
            "Tackle"
        } else if kind == "In Love" {
            &t
        } else {
            ""
        };
        let action = set(&a, kind, detail);
        let err = battle::transition(&c, "gm", &action, vec![], &catalog()).unwrap_err();
        assert!(
            err.message.contains("Battlefield prevents"),
            "{kind}: {}",
            err.message
        );
    }
    let c = battle::transition(&c, "gm", &set(&t, "Sleep", ""), vec![], &catalog())
        .unwrap()
        .0;
    assert!(has(&c, &t, "Sleep"));
}
#[test]
fn cure_protection_safeguard_four_round_clock_and_removal_are_shared_field_rules() {
    let (c, a, _) = setup("Safeguard");
    let mut c = cast(&c, &a, "Safeguard");
    for remaining in (1..=4).rev() {
        assert!(field::has(&c, "Safeguard", p(&c, &a)));
        assert_eq!(
            c.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .battlefield
                .rules
                .iter()
                .find(|r| r.kind == "Safeguard")
                .unwrap()
                .rounds,
            Some(remaining)
        );
        c = next(&c);
    }
    assert!(!field::has(&c, "Safeguard", p(&c, &a)));
    let c = battle::transition(&c, "gm", &set(&a, "Sleep", ""), vec![], &catalog())
        .unwrap()
        .0;
    assert!(has(&c, &a, "Sleep"));
    let (c, a, _) = setup("Safeguard");
    let mut c = cast(&c, &a, "Safeguard");
    field::screen_cleaner(&mut c);
    assert!(!field::has(&c, "Safeguard", p(&c, &a)));
}
#[test]
fn cure_protection_safeguard_respects_infiltrator_and_sound_force_field_bypass() {
    for (name, infiltrator, expected) in [
        ("Spore", false, false),
        ("Spore", true, true),
        ("Sing", false, true),
    ] {
        let (mut c, a, t) = setup("Safeguard");
        let sid = p(&c, &t).sheet_id.clone();
        c.sheets.get_mut(&sid).unwrap().moves.push(name.into());
        let mut c = cast(&c, &a, "Safeguard");
        if infiltrator {
            pm(&mut c, &t).selected_ability = "Infiltrator".into();
        }
        let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
        f.order = vec![t.clone()];
        f.cursor = 0;
        f.started = true;
        let c = pending_attack(&c, &t, &a, name);
        let c = resolve(&c, vec![]);
        assert_eq!(has(&c, &a, "Sleep"), expected, "{name}");
    }
}
#[test]
fn cure_protection_sparkling_aria_resolves_burned_and_unburned_targets() {
    for kind in [None, Some("Burn"), Some("Burn 2"), Some("Burn 3")] {
        let (mut c, a, t) = setup("Sparkling Aria");
        if let Some(k) = kind {
            status(&mut c, &t, k);
        }
        let hp = p(&c, &t).hp;
        let c = pending_attack(&c, &a, &t, "Sparkling Aria");
        assert_eq!(rules::dice_count(&c, &effect(&c), &catalog()).unwrap(), 0);
        let c = resolve(&c, vec![]);
        assert!(!p(&c, &t)
            .runtime
            .statuses
            .iter()
            .any(|s| s.kind.starts_with("Burn")));
        if kind.is_some() {
            assert_eq!(p(&c, &t).hp, hp);
        } else {
            assert!(p(&c, &t).hp < hp);
        }
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
}
#[test]
fn cure_protection_uproar_cures_all_sides_and_new_sleep_for_only_current_round() {
    let (mut c, a, t) = setup("Uproar");
    status(&mut c, &t, "Sleep");
    let c = pending_attack(&c, &a, &t, "Uproar");
    let mut c = resolve(&c, vec![]);
    assert!(!has(&c, &t, "Sleep"));
    let c2 = battle::transition(&c, "gm", &set(&a, "Sleep", ""), vec![], &catalog())
        .unwrap()
        .0;
    assert!(!has(&c2, &a, "Sleep"));
    status(&mut c, &a, "Sleep");
    cure_protection::cure_sleep(&mut c, &catalog()).unwrap();
    assert!(!has(&c, &a, "Sleep"));
    c = next(&c);
    let c = battle::transition(&c, "gm", &set(&t, "Sleep", ""), vec![], &catalog())
        .unwrap()
        .0;
    assert!(has(&c, &t, "Sleep"));
}
#[test]
fn cure_protection_uproar_respects_soundproof_comatose_and_does_not_cure_burn() {
    let (mut c, a, t) = setup("Uproar");
    status(&mut c, &t, "Sleep");
    status(&mut c, &t, "Burn");
    let c = pending_attack(&c, &a, &t, "Uproar");
    let mut c = resolve(&c, vec![]);
    assert!(has(&c, &t, "Burn"));
    for ability in ["Soundproof", "Comatose"] {
        pm(&mut c, &t).selected_ability = ability.into();
        status(&mut c, &t, "Sleep");
        cure_protection::cure_sleep(&mut c, &catalog()).unwrap();
        assert!(has(&c, &t, "Sleep"));
        pm(&mut c, &t)
            .runtime
            .statuses
            .retain(|s| s.kind != "Sleep");
    }
}
#[test]
fn cure_protection_toxic_spikes_cast_reuses_persistent_entry_chance_and_immunities() {
    for face in [1, 6] {
        let (c, a, t) = setup("Toxic Spikes");
        let mut c = cast(&c, &a, "Toxic Spikes");
        assert!(!has(&c, &t, "Poison"));
        assert!(field::has(&c, "Toxic Spikes", p(&c, &t)));
        assert!(!field::has(&c, "Toxic Spikes", p(&c, &a)));
        assert_eq!(
            c.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .battlefield
                .rules[0]
                .rounds,
            None
        );
        let before = c.clone();
        pm(&mut c, &t).deployed = false;
        field::lifecycle(&before, &mut c, &catalog()).unwrap();
        let before = c.clone();
        pm(&mut c, &t).deployed = true;
        field::lifecycle(&before, &mut c, &catalog()).unwrap();
        let e = c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .battlefield
            .events
            .iter()
            .find(|e| e.kind == "toxic-spikes")
            .unwrap()
            .clone();
        assert_eq!(e.dice, 1);
        let cmd = field::FieldCommand::Resolve {
            event_id: e.id,
            accept: true,
        };
        let action = Action::Battlefield {
            command: cmd,
            reason: "Hazard entry".into(),
        };
        let c = battle::transition(&c, "gm", &action, vec![face], &catalog())
            .unwrap()
            .0;
        assert!(has(
            &c,
            &t,
            if face == 6 {
                "Badly Poisoned"
            } else {
                "Poison"
            }
        ));
    }
}
#[test]
fn cure_protection_historical_flag_preserves_missing_paths() {
    for name in ["Safeguard", "Toxic Spikes", "Uproar", "Sparkling Aria"] {
        let (mut c, a, t) = setup(name);
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .cure_protection = false;
        assert!(!cure_protection::handles(&c, name));
        let c = if ["Safeguard", "Toxic Spikes"].contains(&name) {
            roll(
                &c,
                "gm",
                &Action::AccuracyTargets {
                    combatant_id: a,
                    target_ids: vec![],
                    move_name: name.into(),
                    pool_modifier: 0,
                    success_modifier: 0,
                    reason: "Historical".into(),
                },
                6,
            )
        } else {
            pending_attack(&c, &a, &t, name)
        };
        assert!(
            battle::transition(&c, "gm", &effect(&c), vec![], &catalog()).is_err(),
            "{name}"
        );
    }
}
#[test]
fn cure_protection_field_state_survives_serialization_and_rollback_with_permissions() {
    let (c, a, _) = setup("Safeguard");
    let action = Action::AccuracyTargets {
        combatant_id: a.clone(),
        target_ids: vec![],
        move_name: "Safeguard".into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Protection".into(),
    };
    let n = rules::dice_count(&c, &action, &catalog()).unwrap();
    assert!(battle::transition(&c, "intruder", &action, vec![6; n as usize], &catalog()).is_err());
    let c = cast(&c, &a, "Safeguard");
    let c: Campaign = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
    assert!(field::has(&c, "Safeguard", p(&c, &a)));
    let c = battle::transition(
        &c,
        "gm",
        &Action::RollbackBattle {
            event_sequence: c.battle_undo.last().unwrap().event_sequence,
            reason: "Undo field".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    assert!(!field::has(&c, "Safeguard", p(&c, &a)));
}

#[test]
fn cure_protection_toxic_spikes_ignores_flying_and_levitate_entries_and_recasting_does_not_stack() {
    for flying in [true, false] {
        let (c, a, t) = setup("Toxic Spikes");
        let mut c = cast(&c, &a, "Toxic Spikes");
        c = next(&c);
        let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
        f.started = true;
        f.order = vec![a.clone()];
        f.cursor = 0;
        c = cast(&c, &a, "Toxic Spikes");
        assert_eq!(
            c.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .battlefield
                .rules
                .iter()
                .filter(|r| r.kind == "Toxic Spikes")
                .count(),
            1
        );
        if flying {
            species(&mut c, &t, "Pidgey");
        } else {
            pm(&mut c, &t).selected_ability = "Levitate".into();
        }
        let before = c.clone();
        pm(&mut c, &t).deployed = false;
        field::lifecycle(&before, &mut c, &catalog()).unwrap();
        let before = c.clone();
        pm(&mut c, &t).deployed = true;
        field::lifecycle(&before, &mut c, &catalog()).unwrap();
        assert!(!c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .battlefield
            .events
            .iter()
            .any(|e| e.kind == "toxic-spikes"));
    }
}
#[test]
fn cure_protection_defog_removes_safeguard_and_toxic_spikes() {
    for name in ["Safeguard", "Toxic Spikes"] {
        let (mut c, a, _) = setup(name);
        let sid = p(&c, &a).sheet_id.clone();
        c.sheets.get_mut(&sid).unwrap().moves.push("Defog".into());
        let mut c = cast(&c, &a, name);
        let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
        f.started = true;
        f.order = vec![a.clone()];
        f.cursor = 0;
        c = cast(&c, &a, "Defog");
        assert!(!c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .battlefield
            .rules
            .iter()
            .any(|r| r.kind == name));
    }
}
#[test]
fn cure_protection_sparkling_aria_mixed_area_cures_burned_target_and_damages_other() {
    let (mut c, a, t) = setup("Sparkling Aria");
    let mut other = p(&c, &t).clone();
    other.id = "unburned-area-target".into();
    let u = other.id.clone();
    c.battle.as_mut().unwrap().combatants.push(other);
    status(&mut c, &t, "Burn 3");
    let action = Action::AccuracyTargets {
        combatant_id: a,
        target_ids: vec![],
        move_name: "Sparkling Aria".into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Mixed area targets".into(),
    };
    let mut c = roll(&c, "gm", &action, 6);
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
        while c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
            c = pass(&c);
        }
        let target = q(&c).target_id;
        if target == u {
            c = roll(&c, "gm", &damage_command(&c), 6);
        }
        c = resolve(&c, vec![]);
    }
    assert!(!has(&c, &t, "Burn 3"));
    assert_eq!(p(&c, &t).hp, 100);
    assert!(p(&c, &u).hp < 100);
}
