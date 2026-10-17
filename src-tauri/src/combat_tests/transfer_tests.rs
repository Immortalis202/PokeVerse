use super::targeting_tests::{damage_command, pass, roll};
use super::*;
use crate::combat::{ability_changes, transfers, CombatState};

fn setup(name: &str, source: &str, target: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = super::special_tests::setup(name);
    let b = c.battle.as_mut().unwrap();
    let f = b.flow.as_mut().unwrap();
    f.transfers.enabled = true;
    f.ability_changes.enabled = true;
    f.tactics.enabled = true;
    b.combatants[0].selected_ability = source.into();
    b.combatants[1].selected_ability = target.into();
    let before = c.clone();
    ability_changes::lifecycle(&before, &mut c, &catalog()).unwrap();
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
fn add(c: &mut Campaign, template: &str, ability: &str) -> String {
    let mut p = p(c, template).clone();
    let mut sheet = c.sheets[&p.sheet_id].clone();
    p.id = id();
    sheet.id = id();
    p.sheet_id = sheet.id.clone();
    p.name = format!("{ability} recipient");
    p.selected_ability = ability.into();
    p.field_slot = Some(1);
    p.runtime = CombatState::default();
    p.runtime.will = 6;
    c.sheets.insert(sheet.id.clone(), sheet);
    let id = p.id.clone();
    c.battle.as_mut().unwrap().combatants.push(p);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .order
        .push(id.clone());
    let before = c.clone();
    ability_changes::lifecycle(&before, c, &catalog()).unwrap();
    id
}
fn damage(c: &Campaign, a: &str, t: &str, name: &str) -> Campaign {
    let c = pass(&hit(c, a, t, name));
    roll(&c, "gm", &damage_command(&c), 4)
}
fn resolve(c: &Campaign, actor: &str, choice: &str) -> Result<(Campaign, String)> {
    let e = &c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0];
    battle::transition(
        c,
        actor,
        &Action::ResolveAbility {
            event_id: e.id.clone(),
            choice: choice.into(),
            reason: "Resolve fainted ally inheritance".into(),
        },
        vec![],
        &catalog(),
    )
}
fn faint(c: &Campaign, donors: &[String]) -> Campaign {
    let mut next = c.clone();
    for id in donors {
        next.battle
            .as_mut()
            .unwrap()
            .combatants
            .iter_mut()
            .find(|p| p.id == *id)
            .unwrap()
            .hp = 0;
    }
    ability_changes::lifecycle(c, &mut next, &catalog()).unwrap();
    next
}
fn transfer(command: transfers::TransferCommand) -> Action {
    Action::AbilityTransfer {
        command,
        reason: "Reviewed game-time and ability selection".into(),
    }
}
#[test]
fn transfer_mummy_and_aroma_spread_on_both_contact_directions_without_sheet_edits() {
    for name in ["Mummy", "Lingering Aroma"] {
        for (source, target) in [(name, "Run Away"), ("Run Away", name)] {
            let (c, a, t) = setup("Tackle", source, target);
            let permanent = serde_json::to_value(&c.sheets).unwrap();
            let c = damage(&c, &a, &t, "Tackle");
            assert_eq!(p(&c, &a).selected_ability, name);
            assert_eq!(p(&c, &t).selected_ability, name);
            assert_eq!(serde_json::to_value(&c.sheets).unwrap(), permanent);
            assert_eq!(
                c.battle
                    .as_ref()
                    .unwrap()
                    .flow
                    .as_ref()
                    .unwrap()
                    .transfers
                    .history
                    .len(),
                1
            );
        }
    }
}
#[test]
fn transfer_wandering_spirit_swaps_only_when_its_user_hits() {
    for (source, target, expected) in [
        ("Wandering Spirit", "Run Away", true),
        ("Run Away", "Wandering Spirit", false),
    ] {
        let (c, a, t) = setup("Tackle", source, target);
        let c = damage(&c, &a, &t, "Tackle");
        assert_eq!(
            p(&c, &a).selected_ability,
            if expected { target } else { source }
        );
        assert_eq!(
            p(&c, &t).selected_ability,
            if expected { source } else { target }
        );
    }
}
#[test]
fn transfer_qualifying_contact_excludes_ranged_special_immunity_and_unique_abilities() {
    for (name, ability) in [
        ("Swift", "Run Away"),
        ("Rock Throw", "Run Away"),
        ("Tackle", "Multitype"),
        ("Tackle", "Long Reach"),
    ] {
        let (c, a, t) = setup(name, ability, "Mummy");
        let c = damage(&c, &a, &t, name);
        assert_eq!(p(&c, &a).selected_ability, ability, "{name}/{ability}");
    }
    let (mut c, a, t) = setup("Tackle", "Mummy", "Run Away");
    let species = catalog().find("species", "Name", "Gastly").unwrap()["AppId"]
        .as_str()
        .unwrap()
        .to_string();
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some(species);
    let c = damage(&c, &a, &t, "Tackle");
    assert_eq!(p(&c, &t).selected_ability, "Run Away");
}
#[test]
fn transfer_decoys_do_not_invent_contact() {
    let (mut c, a, t) = setup("Tackle", "Mummy", "Run Away");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .decoys
        .enabled = true;
    crate::combat::decoys::cast(&mut c, &t, "Substitute", &catalog()).unwrap();
    let c = damage(&c, &a, &t, "Tackle");
    assert_eq!(p(&c, &t).selected_ability, "Run Away");
}
#[test]
fn transfer_recall_restores_underlying_scene_assignment() {
    let (mut c, a, t) = setup("Tackle", "Mummy", "Run Away");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_changes
        .assignments
        .insert(t.clone(), "Insomnia".into());
    let before = c.clone();
    ability_changes::lifecycle(&before, &mut c, &catalog()).unwrap();
    let c = damage(&c, &a, &t, "Tackle");
    assert_eq!(p(&c, &t).selected_ability, "Mummy");
    let mut next = c.clone();
    next.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == t)
        .unwrap()
        .deployed = false;
    ability_changes::lifecycle(&c, &mut next, &catalog()).unwrap();
    assert_eq!(p(&next, &t).selected_ability, "Insomnia");
}
#[test]
fn transfer_receiver_offers_once_is_owner_controlled_and_preserves_donor_snapshot() {
    let (mut c, _, t) = setup("Tackle", "Run Away", "Levitate");
    let r = add(&mut c, &t, "Receiver");
    let c = faint(&c, std::slice::from_ref(&t));
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
    let mut repeat = c.clone();
    ability_changes::lifecycle(&c, &mut repeat, &catalog()).unwrap();
    assert_eq!(
        repeat
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events
            .len(),
        1
    );
    assert!(resolve(&c, "player", "activate:0").is_err());
    let c = resolve(&c, "other", "activate:0").unwrap().0;
    assert_eq!(p(&c, &r).selected_ability, "Levitate");
    assert_eq!(c.ability_inheritance.copies.len(), 1);
    let copy = c.ability_inheritance.copies.values().next().unwrap();
    assert_eq!(copy.donor_sheet_id, p(&c, &t).sheet_id);
    assert_eq!(copy.expires - copy.acquired, 1440);
}
#[test]
fn transfer_receiver_chooses_one_simultaneous_donor_and_alchemy_can_store_all() {
    for mode in ["Receiver", "Power of Alchemy"] {
        let (mut c, _, t) = setup("Tackle", "Run Away", "Levitate");
        let second = add(&mut c, &t, "Insomnia");
        let r = add(&mut c, &t, mode);
        let c = faint(&c, &[t, second]);
        let choice = if mode == "Receiver" {
            "activate:1"
        } else {
            "store"
        };
        let c = resolve(&c, "other", choice).unwrap().0;
        assert_eq!(
            c.ability_inheritance.copies.len(),
            if mode == "Receiver" { 1 } else { 2 }
        );
        if mode == "Power of Alchemy" {
            assert_eq!(p(&c, &r).selected_ability, mode);
            assert!(c.ability_inheritance.selected.is_empty());
        }
    }
}
#[test]
fn transfer_time_is_gm_only_between_battles_and_expiry_is_exact() {
    let (mut c, _, t) = setup("Tackle", "Run Away", "Levitate");
    let r = add(&mut c, &t, "Receiver");
    let c = resolve(&faint(&c, &[t]), "other", "activate:0").unwrap().0;
    let action = transfer(transfers::TransferCommand::Advance { minutes: 1 });
    assert!(battle::transition(&c, "gm", &action, vec![], &catalog()).is_err());
    let mut c = c;
    c.battle.as_mut().unwrap().active = false;
    assert!(battle::transition(&c, "player", &action, vec![], &catalog()).is_err());
    let c = battle::transition(
        &c,
        "gm",
        &transfer(transfers::TransferCommand::Advance { minutes: 1439 }),
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    assert!(transfers::active_copy(&c, &p(&c, &r).sheet_id).is_some());
    let c = battle::transition(&c, "gm", &action, vec![], &catalog())
        .unwrap()
        .0;
    assert!(transfers::active_copy(&c, &p(&c, &r).sheet_id).is_none());
    assert_eq!(c.ability_inheritance.copies.len(), 1);
}
#[test]
fn transfer_inheritance_and_contact_are_restored_by_rollback() {
    let (mut c, a, t) = setup("Tackle", "Mummy", "Run Away");
    c.revision = 10;
    let c = pass(&hit(&c, &a, &t, "Tackle"));
    let before = serde_json::to_value(c.battle.as_ref().unwrap()).unwrap();
    let c = roll(&c, "gm", &damage_command(&c), 4);
    let step = c.battle_undo.last().unwrap().event_sequence;
    let c = battle::transition(
        &c,
        "gm",
        &Action::RollbackBattle {
            event_sequence: step,
            reason: "Undo contact transfer".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    assert_eq!(
        serde_json::to_value(c.battle.as_ref().unwrap()).unwrap(),
        before
    );
}
#[test]
fn transfer_old_campaigns_default_to_empty_inheritance_and_old_engines_are_not_enabled() {
    let (mut c, _, _) = setup("Tackle", "Run Away", "Levitate");
    c.engine = ASSIGNMENT_ENGINE.into();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .transfers
        .enabled = false;
    let mut value = serde_json::to_value(c).unwrap();
    value.as_object_mut().unwrap().remove("ability_inheritance");
    value["battle"]["flow"]
        .as_object_mut()
        .unwrap()
        .remove("transfers");
    let c: Campaign = serde_json::from_value(value).unwrap();
    assert!(!transfers::enabled(&c));
    assert!(c.ability_inheritance.copies.is_empty());
    assert!(rules::dice_count(
        &c,
        &transfer(transfers::TransferCommand::Advance { minutes: 60 }),
        &catalog()
    )
    .is_err());
}

#[test]
fn transfer_suppressed_contact_abilities_and_unique_targets_are_respected() {
    let (c, a, t) = setup("Tackle", "Mummy", "Neutralizing Gas");
    assert!(p(&c, &a).selected_ability.is_empty());
    let c = damage(&c, &a, &t, "Tackle");
    assert_eq!(p(&c, &t).selected_ability, "Neutralizing Gas");
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .transfers
        .history
        .is_empty());
    for ability in ["Mummy", "Lingering Aroma", "Wandering Spirit"] {
        let (c, a, t) = setup("Tackle", ability, "Multitype");
        let c = damage(&c, &a, &t, "Tackle");
        assert_eq!(p(&c, &a).selected_ability, ability);
        assert_eq!(p(&c, &t).selected_ability, "Multitype");
    }
}

#[test]
fn transfer_inheritance_selection_checks_owner_expiry_and_combat_timing() {
    let (mut c, _, t) = setup("Tackle", "Run Away", "Levitate");
    let r = add(&mut c, &t, "Power of Alchemy");
    let c = resolve(&faint(&c, &[t]), "other", "store").unwrap().0;
    let copy = c
        .ability_inheritance
        .copies
        .values()
        .next()
        .unwrap()
        .clone();
    let select = transfer(transfers::TransferCommand::Select {
        sheet_id: copy.sheet_id.clone(),
        copy_id: Some(copy.id.clone()),
    });
    assert!(battle::transition(&c, "other", &select, vec![], &catalog()).is_err());
    let c = battle::transition(&c, "gm", &select, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(p(&c, &r).selected_ability, "Levitate");
    let mut c = c;
    c.battle.as_mut().unwrap().active = false;
    assert!(battle::transition(&c, "player", &select, vec![], &catalog()).is_err());
    let c = battle::transition(&c, "other", &select, vec![], &catalog())
        .unwrap()
        .0;
    let c = battle::transition(
        &c,
        "gm",
        &transfer(transfers::TransferCommand::Advance { minutes: 1440 }),
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    let snapshot = serde_json::to_value(&c).unwrap();
    assert!(battle::transition(&c, "other", &select, vec![], &catalog()).is_err());
    assert_eq!(serde_json::to_value(&c).unwrap(), snapshot);
    let restored: Campaign = serde_json::from_value(snapshot).unwrap();
    assert_eq!(restored.ability_inheritance.copies.len(), 1);
}

#[test]
fn transfer_inherited_ledger_rolls_back_with_the_ability_event() {
    let (mut c, _, t) = setup("Tackle", "Run Away", "Levitate");
    let _r = add(&mut c, &t, "Receiver");
    let c = faint(&c, &[t]);
    let before = serde_json::to_value(&c.ability_inheritance).unwrap();
    let c = resolve(&c, "other", "activate:0").unwrap().0;
    assert_eq!(c.ability_inheritance.copies.len(), 1);
    let event = c.battle_undo.last().unwrap().event_sequence;
    let c = battle::transition(
        &c,
        "gm",
        &Action::RollbackBattle {
            event_sequence: event,
            reason: "Undo copied ability and its 24-hour record".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    assert_eq!(
        serde_json::to_value(&c.ability_inheritance).unwrap(),
        before
    );
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events[0]
            .kind,
        "transfer-inherit"
    );
}

#[test]
fn transfer_unresolved_donor_metadata_is_a_gm_review_instead_of_rejected_damage() {
    let (mut c, _, t) = setup("Tackle", "Run Away", "Unresolved ability");
    let _r = add(&mut c, &t, "Receiver");
    let c = faint(&c, &[t]);
    let e = &c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0];
    assert_eq!(e.kind, "transfer-review");
    assert!(resolve(&c, "other", "manual").is_err());
    let c = resolve(&c, "gm", "manual").unwrap().0;
    assert!(c.ability_inheritance.copies.is_empty());
}

#[test]
fn transfer_receiver_cannot_acquire_a_second_copy_while_the_first_is_unexpired() {
    let (mut c, _, t) = setup("Tackle", "Run Away", "Levitate");
    let second = add(&mut c, &t, "Insomnia");
    let r = add(&mut c, &t, "Receiver");
    let c = resolve(&faint(&c, &[t]), "other", "activate:0").unwrap().0;
    let sid = p(&c, &r).sheet_id.clone();
    let c = battle::transition(
        &c,
        "gm",
        &transfer(transfers::TransferCommand::Select {
            sheet_id: sid,
            copy_id: None,
        }),
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    assert_eq!(p(&c, &r).selected_ability, "Receiver");
    let c = faint(&c, &[second]);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events
        .is_empty());
    assert_eq!(c.ability_inheritance.copies.len(), 1);
}

#[test]
fn transfer_clock_corrections_are_logged_and_campaign_edits_invalidate_old_battle_undo() {
    let (mut c, _, t) = setup("Tackle", "Run Away", "Levitate");
    let _r = add(&mut c, &t, "Receiver");
    let mut c = resolve(&faint(&c, &[t]), "other", "activate:0").unwrap().0;
    assert!(!c.battle_undo.is_empty());
    c.battle.as_mut().unwrap().active = false;
    let c = battle::transition(
        &c,
        "gm",
        &transfer(transfers::TransferCommand::Advance { minutes: 1440 }),
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    assert!(c.battle_undo.is_empty());
    let command = transfer(transfers::TransferCommand::CorrectClock { minutes: 0 });
    assert!(battle::transition(&c, "other", &command, vec![], &catalog()).is_err());
    let c = battle::transition(&c, "gm", &command, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(c.ability_inheritance.minutes, 0);
    assert!(c
        .ability_inheritance
        .history
        .last()
        .unwrap()
        .contains("CorrectClock"));
    assert_eq!(c.ability_inheritance.copies.len(), 1);
    for command in [
        transfers::TransferCommand::Advance { minutes: 0 },
        transfers::TransferCommand::CorrectClock { minutes: u32::MAX },
    ] {
        assert!(battle::transition(&c, "gm", &transfer(command), vec![], &catalog()).is_err());
    }
}
#[test]
fn transfer_inheritance_rejects_foes_old_faints_and_a_recipient_who_faints_before_accepting() {
    let (mut c, a, t) = setup("Tackle", "Run Away", "Levitate");
    let r = add(&mut c, &t, "Receiver");
    let c = faint(&c, &[a]);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events
        .is_empty());
    let c = faint(&c, &[t]);
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
    let c = faint(&c, &[r]);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events
        .is_empty());
    let mut c = c;
    let before = c.clone();
    ability_changes::lifecycle(&before, &mut c, &catalog()).unwrap();
    assert!(c.ability_inheritance.copies.is_empty());
}

#[test]
fn transfer_contact_replacement_preserves_static_triggered_by_that_same_hit() {
    let (c, a, t) = setup("Tackle", "Mummy", "Static");
    let c = damage(&c, &a, &t, "Tackle");
    assert_eq!(p(&c, &t).selected_ability, "Mummy");
    let event = &c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0];
    assert_eq!(event.ability, "Static");
    let action = Action::ResolveAbility {
        event_id: event.id.clone(),
        choice: "roll".into(),
        reason: "Static triggered before the contact replacement".into(),
    };
    let n = rules::dice_count(&c, &action, &catalog()).unwrap();
    let c = battle::transition(&c, "other", &action, vec![6; n as usize], &catalog())
        .unwrap()
        .0;
    assert!(p(&c, &a)
        .runtime
        .statuses
        .iter()
        .any(|s| s.kind == "Paralysis"));
}

#[test]
fn transfer_contact_replacement_preserves_already_triggered_rough_skin_retaliation() {
    let (c, a, t) = setup("Tackle", "Mummy", "Rough Skin");
    let hp = p(&c, &a).hp;
    let c = damage(&c, &a, &t, "Tackle");
    assert_eq!(p(&c, &t).selected_ability, "Mummy");
    let event = &c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0];
    assert_eq!(event.ability, "Rough Skin");
    let action = Action::ResolveAbility {
        event_id: event.id.clone(),
        choice: "neutral".into(),
        reason: "Resolve retaliation already triggered by the hit".into(),
    };
    let c = battle::transition(&c, "other", &action, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(p(&c, &a).hp, hp - 1);
}

#[test]
fn transfer_contact_replacement_preserves_poison_touch_from_the_same_hit() {
    let (c, a, t) = setup("Tackle", "Poison Touch", "Mummy");
    let c = damage(&c, &a, &t, "Tackle");
    assert_eq!(p(&c, &a).selected_ability, "Mummy");
    let event = &c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0];
    assert_eq!(event.ability, "Poison Touch");
    let action = Action::ResolveAbility {
        event_id: event.id.clone(),
        choice: "roll".into(),
        reason: "Poison Touch already triggered before Mummy".into(),
    };
    let n = rules::dice_count(&c, &action, &catalog()).unwrap();
    let c = battle::transition(&c, "player", &action, vec![6; n as usize], &catalog())
        .unwrap()
        .0;
    assert!(p(&c, &t)
        .runtime
        .statuses
        .iter()
        .any(|s| s.kind == "Poison"));
}
