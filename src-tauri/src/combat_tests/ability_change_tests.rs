use super::targeting_tests::pass;
use super::*;
use crate::combat::{abilities, ability_changes as changes};

fn setup(name: &str, ua: &str, ta: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = super::special_tests::setup(name);
    let b = c.battle.as_mut().unwrap();
    b.flow.as_mut().unwrap().ability_changes.enabled = true;
    b.flow.as_mut().unwrap().tactics.enabled = true;
    b.combatants[0].selected_ability = ua.into();
    b.combatants[1].selected_ability = ta.into();
    // The permanent sheets remain unchanged throughout these tests.
    let before = c.clone();
    changes::lifecycle(&before, &mut c, &catalog()).unwrap();
    (c, a, t)
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
fn refresh(c: &Campaign) -> Campaign {
    let mut next = c.clone();
    changes::lifecycle(c, &mut next, &catalog()).unwrap();
    next
}
fn effect(c: &Campaign) -> Campaign {
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    battle::transition(
        c,
        "gm",
        &Action::ResolveEffect {
            sequence: q.sequence,
            reason: "Reviewed ability change".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap()
    .0
}
#[test]
fn ability_changes_moves_use_normal_accuracy_and_persist_through_switching() {
    for (name, expected) in [
        ("Worry Seed", "Insomnia"),
        ("Simple Beam", "Simple"),
        ("Gastro Acid", ""),
    ] {
        let (c, a, t) = setup(name, "Run Away", "Levitate");
        let sheets = serde_json::to_value(&c.sheets).unwrap();
        let c = effect(&pass(&hit(&c, &a, &t, name)));
        assert_eq!(p(&c, &t).selected_ability, expected);
        assert_eq!(p(&c, &a).actions, 1);
        let mut away = c.clone();
        away.battle.as_mut().unwrap().combatants[1].deployed = false;
        let away = refresh(&away);
        assert_eq!(p(&away, &t).selected_ability, expected);
        assert_eq!(serde_json::to_value(&away.sheets).unwrap(), sheets);
        let restored: Campaign =
            serde_json::from_value(serde_json::to_value(&away).unwrap()).unwrap();
        assert_eq!(p(&restored, &t).selected_ability, expected);
    }
}
#[test]
fn ability_changes_skill_swap_and_role_play_preserve_permanent_abilities() {
    for name in ["Skill Swap", "Role Play"] {
        let (c, a, t) = setup(name, "Run Away", "Levitate");
        let c = effect(&pass(&hit(&c, &a, &t, name)));
        assert_eq!(p(&c, &a).selected_ability, "Levitate");
        assert_eq!(
            p(&c, &t).selected_ability,
            if name == "Skill Swap" {
                "Run Away"
            } else {
                "Levitate"
            }
        );
        assert_ne!(c.sheets[&p(&c, &a).sheet_id].ability, "Levitate");
        let mut ended = c.clone();
        ended.battle.as_mut().unwrap().active = false;
        changes::lifecycle(&c, &mut ended, &catalog()).unwrap();
        assert_eq!(p(&ended, &a).selected_ability, "Run Away");
        assert_eq!(p(&ended, &t).selected_ability, "Levitate");
    }
}
#[test]
fn ability_changes_unique_copy_is_explicit_and_atomic() {
    let (c, a, t) = setup("Role Play", "Run Away", "Multitype");
    let c = pass(&hit(&c, &a, &t, "Role Play"));
    let before = serde_json::to_value(&c).unwrap();
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    let err = battle::transition(
        &c,
        "gm",
        &Action::ResolveEffect {
            sequence: q.sequence,
            reason: "Try unique".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap_err();
    assert_eq!(err.code, "manual_rule");
    assert_eq!(serde_json::to_value(&c).unwrap(), before);
}
#[test]
fn ability_changes_gas_is_foe_only_ranged_and_restores_on_source_departure() {
    let (mut c, a, t) = setup("Tackle", "Neutralizing Gas", "Levitate");
    assert_eq!(p(&c, &t).selected_ability, "");
    assert_eq!(p(&c, &a).selected_ability, "Neutralizing Gas");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ranges
        .insert(a.clone(), vec![]);
    let mut c = refresh(&c);
    assert_eq!(p(&c, &t).selected_ability, "Levitate");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ranges
        .clear();
    let mut c = refresh(&c);
    assert_eq!(p(&c, &t).selected_ability, "");
    c.battle.as_mut().unwrap().combatants[0].hp = 0;
    let c = refresh(&c);
    assert_eq!(p(&c, &t).selected_ability, "Levitate");
    assert!(abilities::guard(&c, p(&c, &t)).is_ok());
}
#[test]
fn ability_changes_gas_preserves_unique_abilities_and_ally_abilities() {
    let (mut c, a, t) = setup("Tackle", "Neutralizing Gas", "Multitype");
    assert_eq!(p(&c, &t).selected_ability, "Multitype");
    c.battle.as_mut().unwrap().combatants[1].side = p(&c, &a).side.clone();
    c.battle.as_mut().unwrap().combatants[1].selected_ability = "Levitate".into();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_changes
        .originals
        .insert(t.clone(), "Levitate".into());
    let c = refresh(&c);
    assert_eq!(p(&c, &t).selected_ability, "Levitate");
}
#[test]
fn ability_changes_trace_uses_host_dice_owner_permissions_and_restores_on_exit() {
    let (c, a, t) = setup("Tackle", "Trace", "Levitate");
    let e = &c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0];
    let action = Action::ResolveAbility {
        event_id: e.id.clone(),
        choice: "roll".into(),
        reason: "Random foe".into(),
    };
    assert_eq!(rules::dice_count(&c, &action, &catalog()).unwrap(), 1);
    assert!(battle::transition(&c, "other", &action, vec![6], &catalog()).is_err());
    let c = battle::transition(&c, "player", &action, vec![6], &catalog())
        .unwrap()
        .0;
    assert_eq!(p(&c, &a).selected_ability, "Levitate");
    assert_eq!(p(&c, &t).selected_ability, "Levitate");
    let mut away = c.clone();
    away.battle.as_mut().unwrap().combatants[0].deployed = false;
    changes::lifecycle(&c, &mut away, &catalog()).unwrap();
    assert_eq!(p(&away, &a).selected_ability, "Trace");
    away.battle.as_mut().unwrap().combatants[0].deployed = true;
    let away = refresh(&away);
    assert_eq!(
        away.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events[0]
            .kind,
        "ability-trace"
    );
}
#[test]
fn ability_changes_huge_power_bonus_tracks_the_effective_ability_once() {
    let (mut c, a, t) = setup("Role Play", "Run Away", "Huge Power");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().ability = "Run Away".into();
    let before = rules::effective_sheet(&c, p(&c, &a)).unwrap().stats["Strength"];
    let mut c = effect(&pass(&hit(&c, &a, &t, "Role Play")));
    assert_eq!(
        rules::effective_sheet(&c, p(&c, &a)).unwrap().stats["Strength"],
        before + 2
    );
    c = refresh(&c);
    assert_eq!(
        rules::effective_sheet(&c, p(&c, &a)).unwrap().stats["Strength"],
        before + 2
    );
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_changes
        .negated
        .insert(a.clone());
    c = refresh(&c);
    assert_eq!(
        rules::effective_sheet(&c, p(&c, &a)).unwrap().stats["Strength"],
        before
    );
}
#[test]
fn ability_changes_old_engines_have_no_new_state() {
    let (mut c, _, _) = fixture("Tackle");
    c.engine = DECOY_ENGINE.into();
    assert!(!changes::enabled(&c));
    let mut value = serde_json::to_value(c).unwrap();
    value["battle"]["flow"]
        .as_object_mut()
        .unwrap()
        .remove("ability_changes");
    let c: Campaign = serde_json::from_value(value).unwrap();
    assert!(!changes::enabled(&c));
}

#[test]
fn ability_changes_decoy_blocks_ability_replacement() {
    let (mut c, a, t) = setup("Worry Seed", "Run Away", "Levitate");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .decoys
        .enabled = true;
    crate::combat::decoys::cast(&mut c, &t, "Substitute", &catalog()).unwrap();
    let c = effect(&pass(&hit(&c, &a, &t, "Worry Seed")));
    assert_eq!(p(&c, &t).selected_ability, "Levitate");
}

#[test]
fn ability_changes_copied_entry_weather_is_applied_and_removed_on_loss() {
    let (c, a, t) = setup("Role Play", "Run Away", "Drought");
    let c = effect(&pass(&hit(&c, &a, &t, "Role Play")));
    let f = c.battle.as_ref().unwrap().flow.as_ref().unwrap();
    assert_eq!(
        f.battlefield.weather.as_ref().unwrap().source,
        format!("ability:{a}")
    );
    let mut next = c.clone();
    next.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_changes
        .negated
        .insert(a.clone());
    changes::lifecycle(&c, &mut next, &catalog()).unwrap();
    assert!(next
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .battlefield
        .weather
        .is_none());
}

#[test]
fn ability_changes_trace_does_not_enqueue_twice_and_cancels_when_negated() {
    let (c, a, _) = setup("Tackle", "Trace", "Levitate");
    let mut c = refresh(&c);
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events
            .len(),
        1
    );
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_changes
        .negated
        .insert(a.clone());
    let c = refresh(&c);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events
        .is_empty());
    assert_eq!(p(&c, &a).selected_ability, "");
}

#[test]
fn ability_changes_opposing_gas_requires_gm_decision() {
    let (c, _, _) = setup("Tackle", "Neutralizing Gas", "Neutralizing Gas");
    let e = &c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0];
    assert_eq!(e.kind, "ability-gas");
    let action = Action::ResolveAbility {
        event_id: e.id.clone(),
        choice: "simultaneous".into(),
        reason: "Both gases continue emitting in this arena".into(),
    };
    assert!(battle::transition(&c, "player", &action, vec![], &catalog()).is_err());
    let c = battle::transition(&c, "gm", &action, vec![], &catalog())
        .unwrap()
        .0;
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events
        .is_empty());
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_changes
            .gas_rulings
            .len(),
        1
    );
}

#[test]
fn ability_changes_range_command_recalculates_gas_immediately() {
    let (c, a, t) = setup("Tackle", "Neutralizing Gas", "Levitate");
    let action = Action::SetBattleRange {
        combatant_id: a,
        in_range_ids: vec![],
        reason: "Foe left gas range".into(),
    };
    let c = battle::transition(&c, "gm", &action, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(p(&c, &t).selected_ability, "Levitate");
}

#[test]
fn ability_changes_copy_negated_identity_without_copying_negation() {
    let (mut c, a, t) = setup("Role Play", "Run Away", "Levitate");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_changes
        .negated
        .insert(t.clone());
    let c = refresh(&c);
    assert_eq!(p(&c, &t).selected_ability, "");
    let c = effect(&pass(&hit(&c, &a, &t, "Role Play")));
    assert_eq!(p(&c, &a).selected_ability, "Levitate");
    assert_eq!(p(&c, &t).selected_ability, "");
}

#[test]
fn ability_changes_negation_removes_only_ability_generated_comatose_sleep() {
    let (mut c, a, t) = setup("Tackle", "Comatose", "Levitate");
    let before = c.clone();
    crate::combat::extra::lifecycle(&before, &mut c, &catalog()).unwrap();
    assert!(p(&c, &a)
        .runtime
        .statuses
        .iter()
        .any(|s| s.detail == "Comatose"));
    c.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == t)
        .unwrap()
        .runtime
        .statuses
        .push(crate::combat::Status {
            kind: "Sleep".into(),
            detail: "Sleep Powder".into(),
            rounds: None,
            progress: 0,
        });
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_changes
        .negated
        .insert(a.clone());
    let mut c = refresh(&c);
    assert!(!p(&c, &a).runtime.statuses.iter().any(|s| s.kind == "Sleep"));
    assert!(p(&c, &t)
        .runtime
        .statuses
        .iter()
        .any(|s| s.detail == "Sleep Powder"));
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_changes
        .negated
        .clear();
    let before = c.clone();
    let mut c = refresh(&c);
    crate::combat::extra::lifecycle(&before, &mut c, &catalog()).unwrap();
    assert!(p(&c, &a)
        .runtime
        .statuses
        .iter()
        .any(|s| s.detail == "Comatose"));
}

#[test]
fn ability_changes_trace_exit_restores_underlying_scene_assignment() {
    let (mut c, a, _) = setup("Tackle", "Run Away", "Levitate");
    let state = &mut c
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_changes;
    state.assignments.insert(a.clone(), "Trace".into());
    state.trace.insert(a.clone(), "Levitate".into());
    c.battle.as_mut().unwrap().combatants[0].selected_ability = "Levitate".into();
    c.battle.as_mut().unwrap().combatants[0].deployed = false;
    let c = refresh(&c);
    assert_eq!(p(&c, &a).selected_ability, "Trace");
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_changes
            .originals[&a],
        "Run Away"
    );
}
