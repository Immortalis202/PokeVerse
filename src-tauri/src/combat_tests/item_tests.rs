use super::*;
use crate::combat::items::{self, ItemCommand};

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
fn action(command: ItemCommand) -> Action {
    Action::HeldItem {
        command,
        reason: "Reviewed battle item fixture".into(),
    }
}
fn use_berry(id: &str) -> Action {
    action(ItemCommand::ConsumeBerry {
        combatant_id: id.into(),
        unit_id: "berry".into(),
    })
}
fn setup(name: &str, ability: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = super::special_tests::setup("Tackle");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .enabled = true;
    pm(&mut c, &a).selected_ability = ability.into();
    pm(&mut c, &a).hp = 12;
    let sid = p(&c, &a).sheet_id.clone();
    let mut s = c.sheets[&sid].clone();
    s.inventory.insert(name.into(), 1);
    let equipped = items::equip(&s, name, "berry", &catalog()).unwrap();
    c.sheets.insert(sid, equipped);
    items::initialize(&mut c, &catalog()).unwrap();
    (c, a, t)
}

#[test]
fn items_held_berry_consumption_is_free_owned_and_cannot_repeat() {
    let (c, a, _) = setup("Oran Berry", "");
    let before = serde_json::to_value(&c.battle).unwrap();
    let command = use_berry(&a);
    assert_eq!(rules::dice_count(&c, &command, &catalog()).unwrap(), 0);
    assert!(battle::transition(&c, "other", &command, vec![], &catalog()).is_err());
    let done = decision(&c, "player", command.clone());
    assert_eq!(p(&done, &a).hp, 13);
    assert_eq!(p(&done, &a).actions, 0);
    assert!(items::held(&done, &a).is_empty());
    assert!(battle::transition(&done, "player", &command, vec![], &catalog()).is_err());
    let restored = decision(
        &done,
        "gm",
        Action::RollbackBattle {
            event_sequence: done.battle_undo.last().unwrap().event_sequence,
            reason: "Restore the consumed unit and HP".into(),
        },
    );
    assert_eq!(serde_json::to_value(&restored.battle).unwrap(), before);
}

#[test]
fn items_berry_consumption_persists_at_end_and_end_rollback_restores_equipment() {
    let (c, a, _) = setup("Pecha Berry", "Cheek Pouch");
    let c = decision(&c, "player", use_berry(&a));
    assert_eq!(p(&c, &a).hp, 14);
    let sid = p(&c, &a).sheet_id.clone();
    assert!(c.sheets[&sid].held_items.is_some());
    let ended = decision(&c, "gm", Action::EndBattle);
    assert!(ended.sheets[&sid].held_items.is_none());
    assert!(
        ended
            .finished_battles
            .last()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .finalized
    );
    let restored = decision(
        &ended,
        "gm",
        Action::RollbackBattle {
            event_sequence: ended.battle_undo.last().unwrap().event_sequence,
            reason: "Restore pre-end equipment resources".into(),
        },
    );
    assert_eq!(restored.sheets[&sid].held_items, c.sheets[&sid].held_items);
    assert_eq!(
        serde_json::to_value(&restored.battle).unwrap(),
        serde_json::to_value(&c.battle).unwrap()
    );
}

#[test]
fn items_unnerve_blocks_consumption_and_klutz_negates_effects_without_minting_items() {
    let (mut c, a, t) = setup("Oran Berry", "");
    pm(&mut c, &t).selected_ability = "Unnerve".into();
    assert!(battle::transition(&c, "player", &use_berry(&a), vec![], &catalog()).is_err());
    assert_eq!(items::held(&c, &a).len(), 1);
    pm(&mut c, &t).deployed = false;
    pm(&mut c, &a).selected_ability = "Klutz".into();
    let c = decision(&c, "player", use_berry(&a));
    assert_eq!(p(&c, &a).hp, 12);
    assert!(items::held(&c, &a).is_empty());
}

#[test]
fn items_transfers_require_gm_and_explicit_surviving_item_recovery_policy() {
    for policy in ["return_surviving", "keep_transfers"] {
        let (c, a, t) = setup("Oran Berry", "");
        let transfer = action(ItemCommand::Transfer {
            from_id: a.clone(),
            to_id: t.clone(),
            unit_id: "berry".into(),
            override_sticky_hold: false,
        });
        assert!(battle::transition(&c, "player", &transfer, vec![], &catalog()).is_err());
        let c = decision(&c, "gm", transfer);
        assert!(items::held(&c, &a).is_empty());
        assert_eq!(items::held(&c, &t)[0].origin_sheet_id, p(&c, &a).sheet_id);
        assert!(battle::transition(&c, "gm", &Action::EndBattle, vec![], &catalog()).is_err());
        let c = decision(
            &c,
            "gm",
            action(ItemCommand::ReturnPolicy {
                policy: policy.into(),
            }),
        );
        let a_sheet = p(&c, &a).sheet_id.clone();
        let t_sheet = p(&c, &t).sheet_id.clone();
        let ended = decision(&c, "gm", Action::EndBattle);
        assert_eq!(
            ended.sheets[&a_sheet].held_items.is_some(),
            policy == "return_surviving"
        );
        assert_eq!(
            ended.sheets[&t_sheet].held_items.is_some(),
            policy == "keep_transfers"
        );
    }
}

#[test]
fn items_sticky_hold_and_duplicate_unit_ownership_are_enforced() {
    let (c, a, t) = setup("Oran Berry", "Sticky Hold");
    let transfer = action(ItemCommand::Transfer {
        from_id: a.clone(),
        to_id: t.clone(),
        unit_id: "berry".into(),
        override_sticky_hold: false,
    });
    assert!(battle::transition(&c, "gm", &transfer, vec![], &catalog()).is_err());
    let mut duplicate = c.clone();
    let a_sheet = p(&c, &a).sheet_id.clone();
    let t_sheet = p(&c, &t).sheet_id.clone();
    duplicate.sheets.get_mut(&t_sheet).unwrap().held_items = c.sheets[&a_sheet].held_items.clone();
    duplicate
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .initialized = false;
    assert!(items::initialize(&mut duplicate, &catalog()).is_err());
}

#[test]
fn items_berry_cures_follow_the_printed_degree_and_status_boundaries() {
    for (berry, kind, cured) in [
        ("Rawst Berry", "Burn 2", true),
        ("Rawst Berry", "Burn 3", false),
        ("Aspear Berry", "Frozen Solid", true),
        ("Pecha Berry", "Badly Poisoned", false),
        ("Lum Berry", "Badly Poisoned", true),
    ] {
        let (mut c, a, _) = setup(berry, "");
        pm(&mut c, &a).runtime.statuses.push(crate::combat::Status {
            kind: kind.into(),
            detail: "".into(),
            rounds: None,
            progress: 0,
        });
        let done = decision(&c, "player", use_berry(&a));
        assert_eq!(
            !p(&done, &a).runtime.statuses.iter().any(|s| s.kind == kind),
            cured,
            "{berry}/{kind}"
        );
        assert_eq!(p(&done, &a).actions, 0);
    }
}

#[test]
fn items_unburden_grants_two_dexterity_once_after_the_last_held_item_is_spent() {
    let (mut c, a, t) = setup("Oran Berry", "Unburden");
    let sid = p(&c, &t).sheet_id.clone();
    let mut sheet = c.sheets[&sid].clone();
    sheet.inventory.insert("Oran Berry".into(), 1);
    c.sheets.insert(
        sid,
        items::equip(&sheet, "Oran Berry", "second-berry", &catalog()).unwrap(),
    );
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .initialized = false;
    items::initialize(&mut c, &catalog()).unwrap();
    let original = rules::effective_sheet(&c, p(&c, &a)).unwrap().stats["Dexterity"];
    let c = decision(&c, "player", use_berry(&a));
    assert_eq!(
        rules::effective_sheet(&c, p(&c, &a)).unwrap().stats["Dexterity"],
        original + 2
    );
    let c = decision(
        &c,
        "gm",
        action(ItemCommand::Transfer {
            from_id: t,
            to_id: a.clone(),
            unit_id: "second-berry".into(),
            override_sticky_hold: false,
        }),
    );
    let c = decision(
        &c,
        "player",
        action(ItemCommand::ConsumeBerry {
            combatant_id: a.clone(),
            unit_id: "second-berry".into(),
        }),
    );
    assert_eq!(
        rules::effective_sheet(&c, p(&c, &a)).unwrap().stats["Dexterity"],
        original + 2
    );
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .unburdened
        .contains(&a));
}

#[test]
fn items_magic_room_forbids_berry_use_without_consumption_or_cheek_pouch_healing() {
    for (berry, ability) in [("Oran Berry", ""), ("Pecha Berry", "Cheek Pouch")] {
        let (mut c, a, _) = setup(berry, ability);
        pm(&mut c, &a).runtime.statuses.push(crate::combat::Status {
            kind: "Poison".into(),
            detail: "".into(),
            rounds: None,
            progress: 0,
        });
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .battlefield
            .rules
            .push(crate::combat::field::FieldRule {
                id: "room".into(),
                kind: "Magic Room".into(),
                source: a.clone(),
                scope: "all".into(),
                rounds: Some(5),
                parameter: 0,
                started_round: 1,
            });
        assert!(battle::transition(&c, "player", &use_berry(&a), vec![], &catalog()).is_err());
        assert_eq!(p(&c, &a).hp, 12);
        assert_eq!(items::held(&c, &a).len(), 1);
        assert!(p(&c, &a)
            .runtime
            .statuses
            .iter()
            .any(|s| s.kind == "Poison"));
    }
}

#[test]
fn items_berry_juice_retains_its_printed_out_of_battle_restriction() {
    let (c, a, _) = setup("Berry Juice", "");
    let error = battle::transition(&c, "player", &use_berry(&a), vec![], &catalog()).unwrap_err();
    assert!(error.message.contains("cannot be used in battle"));
    assert_eq!(items::held(&c, &a).len(), 1);
    assert_eq!(p(&c, &a).hp, 12);
}

#[test]
fn items_atomic_swap_does_not_trigger_unburden_via_an_intermediate_empty_slot() {
    let (mut c, a, t) = setup("Oran Berry", "Unburden");
    let sid = p(&c, &t).sheet_id.clone();
    let mut sheet = c.sheets[&sid].clone();
    sheet.inventory.insert("Pecha Berry".into(), 1);
    c.sheets.insert(
        sid,
        items::equip(&sheet, "Pecha Berry", "other-unit", &catalog()).unwrap(),
    );
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .initialized = false;
    items::initialize(&mut c, &catalog()).unwrap();
    let dexterity = rules::effective_sheet(&c, p(&c, &a)).unwrap().stats["Dexterity"];
    let c = decision(
        &c,
        "gm",
        action(ItemCommand::Swap {
            first_id: a.clone(),
            second_id: t.clone(),
            first_unit_id: "berry".into(),
            second_unit_id: "other-unit".into(),
            override_sticky_hold: false,
        }),
    );
    assert_eq!(items::held(&c, &a)[0].id, "other-unit");
    assert_eq!(items::held(&c, &t)[0].id, "berry");
    assert_eq!(
        rules::effective_sheet(&c, p(&c, &a)).unwrap().stats["Dexterity"],
        dexterity
    );
    assert!(!c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .unburdened
        .contains(&a));
}

#[test]
fn items_recorded_recovery_restores_only_a_lost_unit_and_never_recreates_consumed_berries() {
    let (c, a, _) = setup("Oran Berry", "");
    let c = decision(
        &c,
        "gm",
        action(ItemCommand::Lose {
            combatant_id: a.clone(),
            unit_id: "berry".into(),
            override_sticky_hold: false,
        }),
    );
    let recover = action(ItemCommand::Recover {
        combatant_id: a.clone(),
        unit_id: "berry".into(),
    });
    assert!(battle::transition(&c, "player", &recover, vec![], &catalog()).is_err());
    let c = decision(&c, "gm", recover.clone());
    assert_eq!(items::held(&c, &a)[0].id, "berry");
    let c = decision(&c, "player", use_berry(&a));
    assert!(battle::transition(&c, "gm", &recover, vec![], &catalog()).is_err());
    assert!(items::held(&c, &a).is_empty());
}

#[test]
fn items_cud_chew_activates_one_consumed_berry_twice_without_duplicating_the_unit() {
    let (c, a, _) = setup("Sitrus Berry", "Cud Chew");
    let done = decision(&c, "player", use_berry(&a));
    assert_eq!(p(&done, &a).hp, 15);
    assert_eq!(p(&done, &a).actions, 0);
    let state = &done.battle.as_ref().unwrap().flow.as_ref().unwrap().items;
    assert_eq!(state.units.len(), 1);
    assert_eq!(state.units["berry"].state, "consumed");
    assert_eq!(state.history.len(), 1);
    let restored = decision(
        &done,
        "gm",
        Action::RollbackBattle {
            event_sequence: done.battle_undo.last().unwrap().event_sequence,
            reason: "Restore doubled berry activation".into(),
        },
    );
    assert_eq!(p(&restored, &a).hp, 12);
    assert_eq!(items::held(&restored, &a).len(), 1);
}

#[test]
fn items_harvest_regrows_at_battle_end_once_and_end_rollback_restores_consumption() {
    let (c, a, _) = setup("Oran Berry", "Harvest");
    let used = decision(&c, "player", use_berry(&a));
    assert!(items::held(&used, &a).is_empty());
    let sid = p(&used, &a).sheet_id.clone();
    let ended = decision(&used, "gm", Action::EndBattle);
    let units = ended.sheets[&sid].held_items.as_ref().unwrap();
    assert_eq!(units.len(), 1);
    assert_eq!(units[0].id, "berry");
    assert_eq!(units[0].origin_sheet_id, sid);
    let state = &ended.battle.as_ref().unwrap().flow.as_ref().unwrap().items;
    assert_eq!(
        state.history.iter().filter(|e| e.kind == "harvest").count(),
        1
    );
    assert!(!state.units["berry"].harvest_pending);
    let restored = decision(
        &ended,
        "gm",
        Action::RollbackBattle {
            event_sequence: ended.battle_undo.last().unwrap().event_sequence,
            reason: "Undo Harvest's end-of-battle regeneration".into(),
        },
    );
    assert_eq!(
        serde_json::to_value(&restored.battle).unwrap(),
        serde_json::to_value(&used.battle).unwrap()
    );
    let ended_again = decision(&restored, "gm", Action::EndBattle);
    assert_eq!(
        ended_again.sheets[&sid].held_items.as_ref().unwrap().len(),
        1
    );
}

#[test]
fn items_harvest_regrowth_obeys_surviving_transfer_policy_and_retains_provenance() {
    for policy in ["return_surviving", "keep_transfers"] {
        let (mut c, a, t) = setup("Oran Berry", "");
        pm(&mut c, &t).selected_ability = "Harvest".into();
        let c = decision(
            &c,
            "gm",
            action(ItemCommand::Transfer {
                from_id: a.clone(),
                to_id: t.clone(),
                unit_id: "berry".into(),
                override_sticky_hold: false,
            }),
        );
        let used = decision(&c, "gm", use_berry(&t));
        assert!(battle::transition(&used, "gm", &Action::EndBattle, vec![], &catalog()).is_err());
        let c = decision(
            &used,
            "gm",
            action(ItemCommand::ReturnPolicy {
                policy: policy.into(),
            }),
        );
        let ended = decision(&c, "gm", Action::EndBattle);
        let original = &p(&ended, &a).sheet_id;
        let recipient = &p(&ended, &t).sheet_id;
        let holder = if policy == "return_surviving" {
            original
        } else {
            recipient
        };
        let other = if holder == original {
            recipient
        } else {
            original
        };
        let item = &ended.sheets[holder].held_items.as_ref().unwrap()[0];
        assert_eq!(item.id, "berry");
        assert_eq!(&item.origin_sheet_id, original);
        assert!(ended.sheets[other].held_items.is_none());
    }
}

fn symbiosis_setup(recipient_ability: &str) -> (Campaign, String, String) {
    symbiosis_setup_item("Oran Berry", recipient_ability)
}
fn symbiosis_setup_item(item: &str, recipient_ability: &str) -> (Campaign, String, String) {
    let (mut c, a, donor) = setup(item, recipient_ability);
    pm(&mut c, &donor).side = p(&c, &a).side.clone();
    pm(&mut c, &donor).selected_ability = "Symbiosis".into();
    pm(&mut c, &donor).owner_id = "ally-owner".into();
    let sid = p(&c, &donor).sheet_id.clone();
    let mut sheet = c.sheets[&sid].clone();
    sheet.inventory.insert("Pecha Berry".into(), 1);
    c.sheets.insert(
        sid,
        items::equip(&sheet, "Pecha Berry", "gift", &catalog()).unwrap(),
    );
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .initialized = false;
    items::initialize(&mut c, &catalog()).unwrap();
    (c, a, donor)
}
fn symbiosis_choice(c: &Campaign, unit: &str) -> Action {
    Action::ResolveAbility {
        event_id: c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events[0]
            .id
            .clone(),
        choice: if unit == "manual" {
            "manual".into()
        } else {
            format!("give:{unit}")
        },
        reason: "Resolve the free Symbiosis handoff".into(),
    }
}
#[test]
fn items_symbiosis_uses_the_donor_owner_and_preserves_units_actions_and_rollback() {
    let (c, a, donor) = symbiosis_setup("Unburden");
    let used = decision(&c, "player", use_berry(&a));
    let event = &used
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0];
    assert_eq!(event.combatant_id, donor);
    assert_eq!(event.target_id, a);
    assert_eq!(event.choices, vec!["give:gift"]);
    let command = symbiosis_choice(&used, "gift");
    assert_eq!(rules::dice_count(&used, &command, &catalog()).unwrap(), 0);
    assert!(battle::transition(&used, "player", &command, vec![], &catalog()).is_err());
    assert!(battle::transition(&used, "host", &command, vec![], &catalog()).is_err());
    let done = decision(&used, "ally-owner", command);
    assert_eq!(items::held(&done, &a)[0].id, "gift");
    assert!(items::held(&done, &donor).is_empty());
    assert_eq!(p(&done, &a).actions, 0);
    assert_eq!(p(&done, &donor).actions, 0);
    assert_eq!(
        done.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .units
            .len(),
        2
    );
    let restored = decision(
        &done,
        "gm",
        Action::RollbackBattle {
            event_sequence: done.battle_undo.last().unwrap().event_sequence,
            reason: "Restore Symbiosis offer and original ownership".into(),
        },
    );
    assert_eq!(
        serde_json::to_value(&restored.battle).unwrap(),
        serde_json::to_value(&used.battle).unwrap()
    );
}
#[test]
fn items_symbiosis_repeated_donor_cycle_requires_an_explicit_gm_ruling() {
    let (c, a, donor) = symbiosis_setup("Symbiosis");
    let used = decision(&c, "player", use_berry(&a));
    let done = decision(&used, "ally-owner", symbiosis_choice(&used, "gift"));
    let back = decision(&done, "player", symbiosis_choice(&done, "gift"));
    assert_eq!(items::held(&back, &donor)[0].id, "gift");
    let event = &back
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0];
    assert_eq!(event.kind, "item-symbiosis-cycle");
    let command = symbiosis_choice(&back, "manual");
    assert!(battle::transition(&back, "player", &command, vec![], &catalog()).is_err());
    let ruled = decision(&back, "gm", command);
    assert!(ruled
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events
        .is_empty());
    assert!(ruled
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .symbiosis_chains
        .is_empty());
}

#[test]
fn items_symbiosis_multiple_donors_require_gm_ordering_without_spending_an_action() {
    let (mut c, a, donor) = symbiosis_setup("");
    let mut other = p(&c, &donor).clone();
    other.id = "second-donor".into();
    other.sheet_id = "second-donor-sheet".into();
    other.owner_id = "second-owner".into();
    let mut sheet = c.sheets[&p(&c, &donor).sheet_id].clone();
    sheet.id = other.sheet_id.clone();
    sheet.held_items = None;
    sheet.inventory.insert("Pecha Berry".into(), 1);
    c.sheets.insert(
        sheet.id.clone(),
        items::equip(&sheet, "Pecha Berry", "gift2", &catalog()).unwrap(),
    );
    c.battle.as_mut().unwrap().combatants.push(other);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .initialized = false;
    items::initialize(&mut c, &catalog()).unwrap();
    let used = decision(&c, "player", use_berry(&a));
    let event = &used
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0];
    assert_eq!(event.kind, "item-symbiosis-order");
    assert_eq!(event.choices.len(), 2);
    let command = symbiosis_choice(&used, "gift2");
    for actor in ["player", "ally-owner", "second-owner", "host"] {
        assert!(battle::transition(&used, actor, &command, vec![], &catalog()).is_err());
    }
    let done = decision(&used, "gm", command);
    assert_eq!(items::held(&done, &a)[0].id, "gift2");
    assert_eq!(items::held(&done, &donor)[0].id, "gift");
    assert_eq!(p(&done, "second-donor").actions, 0);
}
#[test]
fn items_symbiosis_requires_a_conscious_deployed_ally_with_an_item() {
    for mode in ["foe", "fainted", "reserve", "empty", "negated"] {
        let (mut c, a, donor) = symbiosis_setup("");
        match mode {
            "foe" => pm(&mut c, &donor).side = "Other".into(),
            "fainted" => pm(&mut c, &donor).hp = 0,
            "reserve" => pm(&mut c, &donor).deployed = false,
            "negated" => pm(&mut c, &donor).selected_ability.clear(),
            "empty" => {
                c.battle
                    .as_mut()
                    .unwrap()
                    .flow
                    .as_mut()
                    .unwrap()
                    .items
                    .units
                    .remove("gift");
            }
            _ => unreachable!(),
        }
        let used = decision(&c, "player", use_berry(&a));
        assert!(
            used.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .ability_events
                .is_empty(),
            "{mode}"
        );
        assert!(items::held(&used, &a).is_empty());
    }
}

fn theft_setup(name: &str, ability: &str, own_item: bool) -> (Campaign, String, String) {
    let (mut c, a, t) = super::special_tests::setup(name);
    pm(&mut c, &a).selected_ability = ability.into();
    for (who, item, unit_id) in [(&t, "Oran Berry", "loot"), (&a, "Pecha Berry", "own")] {
        if who == &a && !own_item {
            continue;
        }
        let sid = p(&c, who).sheet_id.clone();
        let mut sheet = c.sheets[&sid].clone();
        sheet.inventory.insert(item.into(), 1);
        c.sheets.insert(
            sid,
            items::equip(&sheet, item, unit_id, &catalog()).unwrap(),
        );
    }
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .enabled = true;
    items::initialize(&mut c, &catalog()).unwrap();
    (c, a, t)
}
fn theft_damage(c: &Campaign, a: &str, t: &str, name: &str) -> Campaign {
    let c = super::targeting_tests::pass(&hit(c, a, t, name));
    super::targeting_tests::roll(&c, "gm", &super::targeting_tests::damage_command(&c), 4)
}
fn theft_choice(c: &Campaign, choice: &str) -> Action {
    Action::ResolveAbility {
        event_id: c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events[0]
            .id
            .clone(),
        choice: choice.into(),
        reason: "Resolve hit-triggered held-item theft".into(),
    }
}
#[test]
fn items_magician_and_pickpocket_steal_only_after_their_users_hit_and_rollback_restores_the_offer()
{
    for ability in ["Magician", "Pickpocket"] {
        let (c, a, t) = theft_setup("Tackle", ability, false);
        let hit = theft_damage(&c, &a, &t, "Tackle");
        let event = &hit
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events[0];
        assert_eq!(event.ability, ability);
        assert_eq!(event.combatant_id, a);
        assert_eq!(event.choices, vec!["take:0"]);
        let command = theft_choice(&hit, "take:0");
        assert_eq!(rules::dice_count(&hit, &command, &catalog()).unwrap(), 0);
        assert!(battle::transition(&hit, "other", &command, vec![], &catalog()).is_err());
        let done = decision(&hit, "player", command);
        assert_eq!(items::held(&done, &a)[0].id, "loot");
        assert_eq!(
            items::held(&done, &a)[0].origin_sheet_id,
            p(&done, &t).sheet_id
        );
        assert!(items::held(&done, &t).is_empty());
        assert_eq!(p(&done, &a).actions, p(&hit, &a).actions);
        let restored = decision(
            &done,
            "gm",
            Action::RollbackBattle {
                event_sequence: done.battle_undo.last().unwrap().event_sequence,
                reason: "Undo theft and restore the pending hit trigger".into(),
            },
        );
        assert_eq!(
            serde_json::to_value(&restored.battle).unwrap(),
            serde_json::to_value(&hit.battle).unwrap()
        );
    }
}
#[test]
fn items_magician_can_exchange_or_keep_its_existing_item_without_an_empty_slot() {
    for choice in ["swap:0", "decline"] {
        let (c, a, t) = theft_setup("Tackle", "Magician", true);
        let hit = theft_damage(&c, &a, &t, "Tackle");
        let done = decision(&hit, "player", theft_choice(&hit, choice));
        assert_eq!(
            items::held(&done, &a)[0].id,
            if choice == "decline" { "own" } else { "loot" }
        );
        assert_eq!(
            items::held(&done, &t)[0].id,
            if choice == "decline" { "loot" } else { "own" }
        );
        assert_eq!(
            done.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .items
                .units
                .len(),
            2
        );
    }
}
#[test]
fn items_theft_respects_physical_ranged_empty_slot_foe_and_sticky_hold_boundaries() {
    for (ability, name, own, blocker, expected) in [
        ("Magician", "Water Gun", false, "", true),
        ("Pickpocket", "Water Gun", false, "", false),
        ("Pickpocket", "Rock Throw", false, "", false),
        ("Pickpocket", "Tackle", true, "", false),
        ("Magician", "Tackle", false, "Sticky Hold", false),
        ("Pickpocket", "Tackle", false, "Sticky Hold", false),
        ("Magician", "Tackle", false, "ally", false),
    ] {
        let (mut c, a, t) = theft_setup(name, ability, own);
        if blocker == "ally" {
            pm(&mut c, &t).side = p(&c, &a).side.clone();
        } else {
            pm(&mut c, &t).selected_ability = blocker.into();
        }
        let done = theft_damage(&c, &a, &t, name);
        assert_eq!(
            done.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .ability_events
                .iter()
                .any(|e| e.kind == "item-theft"),
            expected,
            "{ability} {name} {blocker}"
        );
    }
}
#[test]
fn items_theft_offer_survives_same_hit_mummy_replacement_without_granting_retroactive_triggers() {
    let (mut c, a, t) = theft_setup("Tackle", "Pickpocket", false);
    pm(&mut c, &t).selected_ability = "Mummy".into();
    let flow = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    flow.transfers.enabled = true;
    flow.ability_changes.enabled = true;
    let hit = theft_damage(&c, &a, &t, "Tackle");
    assert_eq!(p(&hit, &a).selected_ability, "Mummy");
    let done = decision(&hit, "player", theft_choice(&hit, "take:0"));
    assert_eq!(items::held(&done, &a)[0].id, "loot");
    assert_eq!(p(&done, &a).selected_ability, "Mummy");
}

#[test]
fn items_theft_cannot_take_items_through_a_decoy_immunity_or_missed_attack() {
    for mode in ["decoy", "type_immune", "ability_immune", "miss"] {
        let (mut c, a, t) = theft_setup("Tackle", "Magician", false);
        if mode == "decoy" {
            c.battle
                .as_mut()
                .unwrap()
                .flow
                .as_mut()
                .unwrap()
                .decoys
                .enabled = true;
            crate::combat::decoys::cast(&mut c, &t, "Substitute", &catalog()).unwrap();
        } else if mode == "type_immune" {
            let species = catalog().find("species", "Name", "Gastly").unwrap()["AppId"]
                .as_str()
                .unwrap()
                .to_owned();
            let sid = p(&c, &t).sheet_id.clone();
            c.sheets.get_mut(&sid).unwrap().species_id = Some(species);
        } else if mode == "ability_immune" {
            pm(&mut c, &t).selected_ability = "Wonder Guard".into();
        }
        let done = if mode == "miss" {
            super::targeting_tests::roll(
                &c,
                "player",
                &super::targeting_tests::attack(&a, vec![t.clone()], "Tackle"),
                1,
            )
        } else {
            theft_damage(&c, &a, &t, "Tackle")
        };
        assert!(
            !done
                .battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .ability_events
                .iter()
                .any(|e| e.kind == "item-theft"),
            "{mode}"
        );
        assert_eq!(items::held(&done, &t)[0].id, "loot");
        assert!(items::held(&done, &a).is_empty());
    }
}

fn next_item_round(c: &Campaign) -> Campaign {
    let mut c = c.clone();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .order
        .clear();
    decision(&c, "gm", Action::NextRound)
}
fn restrict_healing(c: &mut Campaign, id: &str, name: &str, amount: i32) {
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .special
        .effects
        .push(crate::combat::special::TimedEffect {
            name: name.into(),
            source: id.into(),
            target: id.into(),
            amount,
            rounds: None,
            started: 1,
            slot: "".into(),
        });
}
#[test]
fn items_leftovers_heals_three_actual_hp_then_consumes_and_triggers_unburden() {
    let (mut c, a, _) = setup("Leftovers", "Unburden");
    let dexterity = rules::effective_sheet(&c, p(&c, &a)).unwrap().stats["Dexterity"];
    for used in 1..=3 {
        c = next_item_round(&c);
        assert_eq!(p(&c, &a).hp, 12 + used as i32);
        let unit = &c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .units["berry"];
        assert_eq!(unit.unit.uses, used);
        assert_eq!(unit.state, if used == 3 { "consumed" } else { "held" });
    }
    assert_eq!(
        rules::effective_sheet(&c, p(&c, &a)).unwrap().stats["Dexterity"],
        dexterity + 2
    );
    let more = next_item_round(&c);
    assert_eq!(p(&more, &a).hp, 15);
    let sid = p(&more, &a).sheet_id.clone();
    let ended = decision(&more, "gm", Action::EndBattle);
    assert!(ended.sheets[&sid].held_items.is_none());
}
#[test]
fn items_leftovers_partial_usage_persists_at_battle_end_and_round_rollback_restores_it() {
    let (mut c, a, _) = setup("Leftovers", "");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .order
        .clear();
    let healed = decision(&c, "gm", Action::NextRound);
    let restored = decision(
        &healed,
        "gm",
        Action::RollbackBattle {
            event_sequence: healed.battle_undo.last().unwrap().event_sequence,
            reason: "Undo Leftovers healing, usage and round checkpoint".into(),
        },
    );
    assert_eq!(
        serde_json::to_value(&restored.battle).unwrap(),
        serde_json::to_value(&c.battle).unwrap()
    );
    let sid = p(&healed, &a).sheet_id.clone();
    let ended = decision(&healed, "gm", Action::EndBattle);
    let item = &ended.sheets[&sid].held_items.as_ref().unwrap()[0];
    assert_eq!(item.id, "berry");
    assert_eq!(item.uses, 1);
}
#[test]
fn items_leftovers_does_not_spend_usage_when_full_fainted_recalled_suppressed_or_healing_blocked() {
    for mode in [
        "full",
        "fainted",
        "reserve",
        "Klutz",
        "Magic Room",
        "Heal Block",
        "Shadow Force",
    ] {
        let (mut c, a, _) = setup("Leftovers", "");
        match mode {
            "full" => pm(&mut c, &a).hp = 20,
            "fainted" => pm(&mut c, &a).hp = 0,
            "reserve" => pm(&mut c, &a).deployed = false,
            "Klutz" => pm(&mut c, &a).selected_ability = "Klutz".into(),
            "Magic Room" => c
                .battle
                .as_mut()
                .unwrap()
                .flow
                .as_mut()
                .unwrap()
                .battlefield
                .rules
                .push(crate::combat::field::FieldRule {
                    id: "room".into(),
                    kind: "Magic Room".into(),
                    source: a.clone(),
                    scope: "all".into(),
                    rounds: None,
                    parameter: 0,
                    started_round: 1,
                }),
            "Heal Block" => restrict_healing(&mut c, &a, mode, 0),
            "Shadow Force" => restrict_healing(&mut c, &a, mode, 8),
            _ => unreachable!(),
        }
        let hp = p(&c, &a).hp;
        let done = next_item_round(&c);
        assert_eq!(p(&done, &a).hp, hp, "{mode}");
        assert_eq!(items::held(&done, &a)[0].uses, 0, "{mode}");
    }
}
#[test]
fn items_leftovers_round_end_symbiosis_does_not_repeat_healing_during_pending_decisions() {
    let (mut c, a, _) = symbiosis_setup_item("Leftovers", "");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .units
        .get_mut("berry")
        .unwrap()
        .unit
        .uses = 2;
    let healed = next_item_round(&c);
    assert_eq!(p(&healed, &a).hp, 13);
    assert_eq!(healed.battle.as_ref().unwrap().round, 1);
    let given = decision(&healed, "ally-owner", symbiosis_choice(&healed, "gift"));
    let advanced = next_item_round(&given);
    assert_eq!(advanced.battle.as_ref().unwrap().round, 2);
    assert_eq!(p(&advanced, &a).hp, 13);
    assert_eq!(items::held(&advanced, &a)[0].id, "gift");
    assert_eq!(
        advanced
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .units["berry"]
            .unit
            .uses,
        3
    );
}
#[test]
fn items_berry_healing_cannot_restore_shadow_force_damage() {
    let (mut c, a, _) = setup("Sitrus Berry", "Cud Chew");
    restrict_healing(&mut c, &a, "Shadow Force", 7);
    let done = decision(&c, "player", use_berry(&a));
    assert_eq!(p(&done, &a).hp, 13);
    assert!(items::held(&done, &a).is_empty());
}

fn frisk_setup() -> (Campaign, String, String) {
    let (mut c, a, t) = theft_setup("Tackle", "Frisk", false);
    items::frisk_lifecycle(&mut c).unwrap();
    (c, a, t)
}
fn frisk_choice(c: &Campaign, target: &str) -> Action {
    Action::ResolveAbility {
        event_id: c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events[0]
            .id
            .clone(),
        choice: target.into(),
        reason: "GM reveals one foe's held equipment to the Frisk trainer".into(),
    }
}
#[test]
fn items_frisk_reveals_one_foe_to_its_trainer_and_rollback_restores_the_entry_decision() {
    let (mut c, a, t) = frisk_setup();
    items::frisk_lifecycle(&mut c).unwrap();
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
    let command = frisk_choice(&c, &t);
    assert!(battle::transition(&c, "player", &command, vec![], &catalog()).is_err());
    let done = decision(&c, "gm", command);
    let reveal = &done
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .reveals[0];
    assert_eq!(reveal.owner_id, "player");
    assert_eq!(reveal.source_id, a);
    assert_eq!(reveal.target_id, t);
    assert_eq!(reveal.items, vec!["Oran Berry"]);
    let undone = decision(
        &done,
        "gm",
        Action::RollbackBattle {
            event_sequence: done.battle_undo.last().unwrap().event_sequence,
            reason: "Undo the private reveal and restore the GM choice".into(),
        },
    );
    assert_eq!(
        serde_json::to_value(&undone.battle).unwrap(),
        serde_json::to_value(&c.battle).unwrap()
    );
    let mut recalled = done;
    pm(&mut recalled, &a).deployed = false;
    items::frisk_lifecycle(&mut recalled).unwrap();
    pm(&mut recalled, &a).deployed = true;
    items::frisk_lifecycle(&mut recalled).unwrap();
    assert_eq!(
        recalled
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
}
#[test]
fn items_private_views_hide_equipment_reveals_archives_and_undo_from_unrelated_players() {
    let (c, _, t) = frisk_setup();
    let done = decision(&c, "gm", frisk_choice(&c, &t));
    let actor_view = crate::privacy::for_player(&done, "player");
    assert_eq!(
        actor_view
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .reveals[0]
            .items,
        vec!["Oran Berry"]
    );
    let target_sheet = &p(&done, &t).sheet_id;
    assert!(actor_view.sheets[target_sheet].held_items.is_none());
    assert!(!actor_view
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .units
        .contains_key("loot"));
    for viewer in ["host", "unrelated"] {
        let view = crate::privacy::for_player(&done, viewer);
        assert!(
            !serde_json::to_string(&view).unwrap().contains("Oran Berry"),
            "{viewer}"
        );
    }
    assert_eq!(
        serde_json::to_value(crate::privacy::for_player(&done, "gm")).unwrap(),
        serde_json::to_value(&done).unwrap()
    );
    let ended = decision(&done, "gm", Action::EndBattle);
    let view = crate::privacy::for_player(&ended, "unrelated");
    assert!(!serde_json::to_string(&view).unwrap().contains("Oran Berry"));
    assert!(
        crate::privacy::for_player(&ended, "player").finished_battles[0]
            .flow
            .as_ref()
            .unwrap()
            .items
            .reveals
            .len()
            == 1
    );
}
#[test]
fn items_frisk_can_reveal_no_item_and_theft_decisions_disclose_only_the_needed_units() {
    let (mut c, _, t) = frisk_setup();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .units
        .remove("loot");
    let done = decision(&c, "gm", frisk_choice(&c, &t));
    assert!(done
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .reveals[0]
        .items
        .is_empty());
    let (c, a, t) = theft_setup("Tackle", "Magician", false);
    let pending = theft_damage(&c, &a, &t, "Tackle");
    let own_view = crate::privacy::for_player(&pending, "player");
    assert_eq!(
        own_view
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .units["loot"]
            .unit
            .name,
        "Oran Berry"
    );
    let observer = crate::privacy::for_player(&pending, "unrelated");
    assert!(!serde_json::to_string(&observer)
        .unwrap()
        .contains("Oran Berry"));
    assert!(observer
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .thefts
        .is_empty());
}

#[tokio::test]
async fn items_frisk_room_views_remain_private_on_welcome_broadcast_receipt_and_reconnect() {
    use crate::{room, store::Store};
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };
    fn notify(tx: tokio::sync::mpsc::UnboundedSender<Campaign>) -> room::Notify {
        Arc::new(move |_, value| {
            if let Ok(
                ServerMessage::Snapshot { campaign } | ServerMessage::Accepted { campaign, .. },
            ) = serde_json::from_value(value)
            {
                let _ = tx.send(campaign);
            }
        })
    }
    async fn receive(
        rx: &mut tokio::sync::mpsc::UnboundedReceiver<Campaign>,
        revision: u32,
    ) -> Campaign {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let c = rx.recv().await.unwrap();
                if c.revision >= revision {
                    return c;
                }
            }
        })
        .await
        .unwrap()
    }
    async fn send(remote: &room::Remote, command: Command) -> Campaign {
        let (tx, rx) = tokio::sync::oneshot::channel();
        remote.sender.send((command, tx)).await.unwrap();
        tokio::time::timeout(Duration::from_secs(5), rx)
            .await
            .unwrap()
            .unwrap()
            .unwrap()
    }
    let dirs: Vec<_> = (0..4).map(|_| tempfile::tempdir().unwrap()).collect();
    let stores: Vec<_> = dirs
        .iter()
        .map(|dir| Arc::new(Mutex::new(Store::open(dir.path()).unwrap())))
        .collect();
    let ids: Vec<_> = stores
        .iter()
        .map(|store| store.lock().unwrap().player_id.clone())
        .collect();
    let (mut c, a, t) = frisk_setup();
    c.host_id = ids[0].clone();
    c.gm_id = ids[3].clone();
    pm(&mut c, &a).owner_id = ids[1].clone();
    let sid = p(&c, &a).sheet_id.clone();
    let trainer = c.sheets[&sid].trainer_id.clone();
    c.sheets.get_mut(&sid).unwrap().owner_id = ids[1].clone();
    if let Some(trainer) = trainer {
        c.sheets.get_mut(&trainer).unwrap().owner_id = ids[1].clone();
    }
    stores[0].lock().unwrap().put_campaign(&c).unwrap();
    let cat = Arc::new(catalog());
    let (host_tx, mut host_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut hosted = room::start(
        stores[0].clone(),
        cat.clone(),
        c.id.clone(),
        0,
        notify(host_tx),
    )
    .await
    .unwrap();
    let request = JoinRequest {
        address: hosted.info.address.replace("<host-ip>", "127.0.0.1"),
        token: hosted.info.token.clone(),
    };
    let (owner_tx, mut owner_rx) = tokio::sync::mpsc::unbounded_channel();
    let (owner, welcome) = room::join(
        stores[1].clone(),
        cat.clone(),
        request.clone(),
        notify(owner_tx),
    )
    .await
    .unwrap();
    assert!(!serde_json::to_string(&welcome)
        .unwrap()
        .contains("Oran Berry"));
    let (observer_tx, mut observer_rx) = tokio::sync::mpsc::unbounded_channel();
    let (observer, welcome) = room::join(
        stores[2].clone(),
        cat.clone(),
        request.clone(),
        notify(observer_tx),
    )
    .await
    .unwrap();
    assert!(!serde_json::to_string(&welcome)
        .unwrap()
        .contains("Oran Berry"));
    let (gm, _) = room::join(
        stores[3].clone(),
        cat.clone(),
        request.clone(),
        Arc::new(|_, _| {}),
    )
    .await
    .unwrap();
    let command = Command {
        id: id(),
        expected_revision: c.revision,
        action: frisk_choice(&c, &t),
    };
    let accepted = send(&gm, command.clone()).await;
    assert_eq!(
        accepted
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .reveals[0]
            .items,
        vec!["Oran Berry"]
    );
    let owner_view = receive(&mut owner_rx, accepted.revision).await;
    assert_eq!(
        owner_view
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .reveals[0]
            .items,
        vec!["Oran Berry"]
    );
    for view in [
        receive(&mut observer_rx, accepted.revision).await,
        receive(&mut host_rx, accepted.revision).await,
    ] {
        assert!(!serde_json::to_string(&view).unwrap().contains("Oran Berry"));
    }
    let receipt = send(&gm, command).await;
    assert_eq!(receipt.revision, accepted.revision);
    owner.task.abort();
    tokio::time::timeout(Duration::from_secs(5), async {
        while hosted.host.connected.lock().unwrap().contains(&ids[1]) {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    let (rejoined, snapshot) = room::join(stores[1].clone(), cat, request, Arc::new(|_, _| {}))
        .await
        .unwrap();
    assert_eq!(
        snapshot
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .reveals[0]
            .items,
        vec!["Oran Berry"]
    );
    assert!(snapshot.sheets[&p(&c, &t).sheet_id].held_items.is_none());
    let cached = stores[2]
        .lock()
        .unwrap()
        .available_campaigns()
        .unwrap()
        .into_iter()
        .find(|view| view.id == c.id)
        .unwrap();
    assert!(!serde_json::to_string(&cached)
        .unwrap()
        .contains("Oran Berry"));
    assert_eq!(
        hosted.host.snapshot().unwrap().sheets[&p(&c, &t).sheet_id]
            .held_items
            .as_ref()
            .unwrap()[0]
            .name,
        "Oran Berry"
    );
    rejoined.task.abort();
    observer.task.abort();
    gm.task.abort();
    let _ = hosted.host.shutdown.send(true);
    if let Some(stop) = hosted.stop.take() {
        let _ = stop.send(());
    }
}

fn gift_setup(ability: &str, flavor: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = setup("Oran Berry", ability);
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .moves
        .push("Natural Gift".into());
    let c = decision(
        &c,
        "gm",
        action(ItemCommand::BerryFlavor {
            unit_id: "berry".into(),
            flavor: flavor.into(),
        }),
    );
    let c = decision(
        &c,
        "player",
        action(ItemCommand::NaturalGift {
            combatant_id: a.clone(),
            unit_id: "berry".into(),
        }),
    );
    (c, a, t)
}
#[test]
fn items_natural_gift_spends_one_berry_on_hit_or_miss_and_rollback_restores_selection() {
    for face in [1, 4] {
        let (c, a, t) = gift_setup("", "Spicy");
        let before = serde_json::to_value(&c.battle).unwrap();
        let done = super::targeting_tests::roll(
            &c,
            "player",
            &super::targeting_tests::attack(&a, vec![t], "Natural Gift"),
            face,
        );
        assert!(items::held(&done, &a).is_empty());
        assert_eq!(p(&done, &a).hp, 12);
        assert!(done
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .natural_gift
            .is_empty());
        assert_eq!(
            done.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .items
                .units["berry"]
                .state,
            "consumed"
        );
        let restored = decision(
            &done,
            "gm",
            Action::RollbackBattle {
                event_sequence: done.battle_undo.last().unwrap().event_sequence,
                reason: "Restore the spent ingredient and selected berry".into(),
            },
        );
        assert_eq!(serde_json::to_value(&restored.battle).unwrap(), before);
    }
}
#[test]
fn items_ripen_natural_gift_adds_two_damage_dice_and_freezes_its_type_and_bonus() {
    let mut pools = vec![];
    for ability in ["", "Ripen"] {
        let (c, a, t) = gift_setup(ability, "Spicy");
        let hit = hit(&c, &a, &t, "Natural Gift");
        let mut ready = super::targeting_tests::pass(&hit);
        let m = crate::combat::field::move_data(
            &ready,
            p(&ready, &a),
            &catalog().find("moves", "Name", "Natural Gift").unwrap(),
        );
        assert_eq!(m["Type"], "Fire");
        let pool = rules::dice_count(
            &ready,
            &super::targeting_tests::damage_command(&ready),
            &catalog(),
        )
        .unwrap();
        pools.push(pool);
        pm(&mut ready, &a).selected_ability.clear();
        assert_eq!(
            rules::dice_count(
                &ready,
                &super::targeting_tests::damage_command(&ready),
                &catalog()
            )
            .unwrap(),
            pool
        );
        let result = super::targeting_tests::roll(
            &ready,
            "gm",
            &super::targeting_tests::damage_command(&ready),
            4,
        );
        assert!(p(&result, &t).hp < 20);
        let sequence = result.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sequence;
        let finished = decision(
            &result,
            "player",
            Action::ResolveEffect {
                sequence,
                reason: "Finish Natural Gift's reviewed ingredient effects".into(),
            },
        );
        assert!(finished
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .queue
            .is_empty());
    }
    assert_eq!(pools[1], pools[0] + 2);
}
#[test]
fn items_natural_gift_requires_a_real_selected_flavor_and_respects_unnerve() {
    let (mut c, a, t) = setup("Oran Berry", "Ripen");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .moves
        .push("Natural Gift".into());
    let command = super::targeting_tests::attack(&a, vec![t.clone()], "Natural Gift");
    assert!(rules::dice_count(&c, &command, &catalog()).is_err());
    let flavor = action(ItemCommand::BerryFlavor {
        unit_id: "berry".into(),
        flavor: "Spicy".into(),
    });
    assert!(battle::transition(&c, "player", &flavor, vec![], &catalog()).is_err());
    let (mut c, a, t) = gift_setup("Ripen", "Spicy");
    pm(&mut c, &t).selected_ability = "Unnerve".into();
    assert!(rules::dice_count(
        &c,
        &super::targeting_tests::attack(&a, vec![t], "Natural Gift"),
        &catalog()
    )
    .is_err());
    assert_eq!(items::held(&c, &a).len(), 1);
    for (flavor, ty) in [
        ("Bitter", "Dark"),
        ("Dry", "Ground"),
        ("Effervescent", "Flying"),
        ("Fresh", "Grass"),
        ("Frozen", "Ice"),
        ("Half-eaten", "Bug"),
        ("Juicy", "Water"),
        ("Numbing", "Psychic"),
        ("Oily", "Fight"),
        ("Rotten", "Poison"),
        ("Salty", "Steel"),
        ("Spicy", "Fire"),
        ("Sour", "Electric"),
        ("Sugary", "Fairy"),
        ("Tough", "Rock"),
        ("Uneatable", "Dragon"),
        ("Withered", "Ghost"),
    ] {
        assert_eq!(items::flavor_type(flavor), Some(ty));
    }
    assert_eq!(items::flavor_type("Normal"), None);
}
#[test]
fn items_ripen_healing_increase_is_an_explicit_gm_ruling_not_automatic_doubling() {
    let (c, a, _) = setup("Oran Berry", "Ripen");
    assert_eq!(p(&decision(&c, "player", use_berry(&a)), &a).hp, 13);
    let command = action(ItemCommand::ConsumeRipenedBerry {
        combatant_id: a.clone(),
        unit_id: "berry".into(),
        extra_healing: 4,
    });
    assert!(battle::transition(&c, "player", &command, vec![], &catalog()).is_err());
    let done = decision(&c, "gm", command.clone());
    assert_eq!(p(&done, &a).hp, 15);
    assert!(items::held(&done, &a).is_empty());
    let (c, _, _) = setup("Oran Berry", "");
    assert!(battle::transition(&c, "gm", &command, vec![], &catalog()).is_err());
}

#[test]
fn items_recorded_berry_flavor_persists_with_the_unspent_unit_and_stays_private() {
    let (c, a, _) = gift_setup("Ripen", "Spicy");
    let sid = p(&c, &a).sheet_id.clone();
    let view = crate::privacy::for_player(&c, "unrelated");
    assert!(!serde_json::to_string(&view).unwrap().contains("Spicy"));
    let ended = decision(&c, "gm", Action::EndBattle);
    let unit = &ended.sheets[&sid].held_items.as_ref().unwrap()[0];
    assert_eq!(unit.flavor.as_deref(), Some("Spicy"));
    assert_eq!(unit.id, "berry");
    let stored = items::unequip(&ended.sheets[&sid], "berry", &catalog()).unwrap();
    let equipped = items::equip(&stored, "berry", "do-not-mint-another-unit", &catalog()).unwrap();
    assert_eq!(
        equipped.held_items.as_ref().unwrap()[0].flavor.as_deref(),
        Some("Spicy")
    );
}

#[test]
fn items_natural_gift_clash_spends_the_defenders_berry_once() {
    let (c, a, t) = gift_setup("Ripen", "Spicy");
    let c = decision(
        &c,
        "player",
        Action::PassTurn {
            combatant_id: a.clone(),
            reason: "Allow the foe to act before the berry clash".into(),
        },
    );
    let attack = super::targeting_tests::attack(&t, vec![a.clone()], "Tackle");
    let incoming = super::targeting_tests::roll(&c, "other", &attack, 4);
    let action = Action::React {
        combatant_id: a.clone(),
        kind: "clash".into(),
        move_name: "Natural Gift".into(),
        reason: "Throw the selected berry during Clash".into(),
    };
    let done = super::targeting_tests::roll(&incoming, "player", &action, 4);
    assert!(items::held(&done, &a).is_empty());
    assert_eq!(
        done.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .history
            .iter()
            .filter(|e| e.kind == "natural_gift")
            .count(),
        1
    );
}
#[test]
fn items_magic_room_blocks_natural_gift_without_spending_its_prepared_berry() {
    let (mut c, a, t) = gift_setup("Ripen", "Spicy");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .rules
        .push(crate::combat::field::FieldRule {
            id: "room".into(),
            kind: "Magic Room".into(),
            source: a.clone(),
            scope: "all".into(),
            rounds: None,
            parameter: 0,
            started_round: 1,
        });
    let command = super::targeting_tests::attack(&a, vec![t], "Natural Gift");
    assert!(rules::dice_count(&c, &command, &catalog()).is_err());
    assert_eq!(items::held(&c, &a)[0].flavor.as_deref(), Some("Spicy"));
}

#[test]
fn items_natural_gift_weather_guards_use_the_resolved_flavor_and_ability_type() {
    for (ability, flavor, allowed) in [
        ("Ripen", "Spicy", false),
        ("Ripen", "Sugary", true),
        ("Normalize", "Spicy", true),
    ] {
        let (mut c, a, t) = gift_setup(ability, flavor);
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .battlefield
            .natural = Some(crate::combat::field::Weather::Typhoon);
        let command = super::targeting_tests::attack(&a, vec![t], "Natural Gift");
        assert_eq!(
            rules::dice_count(&c, &command, &catalog()).is_ok(),
            allowed,
            "{ability}/{flavor}"
        );
        assert_eq!(items::held(&c, &a).len(), 1);
    }
}

fn fling_setup(item: &str, ability: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = setup(item, ability);
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().moves.push("Fling".into());
    let c = decision(
        &c,
        "player",
        action(ItemCommand::Fling {
            combatant_id: a.clone(),
            unit_id: Some("berry".into()),
        }),
    );
    (c, a, t)
}
#[test]
fn items_fling_freezes_rarity_bonus_loses_one_unit_and_restores_selection_on_rollback() {
    for (item, bonus) in [
        ("Chesto Berry", 1),
        ("Oran Berry", 2),
        ("Sitrus Berry", 3),
        ("Leftovers", 4),
    ] {
        let (c, a, t) = fling_setup(item, "");
        let hit = hit(&c, &a, &t, "Fling");
        assert!(items::held(&hit, &a).is_empty());
        let state = &hit.battle.as_ref().unwrap().flow.as_ref().unwrap().items;
        assert_eq!(state.units["berry"].state, "ground");
        let input = state.fling_moves.values().next().unwrap();
        assert_eq!(input.item_bonus, bonus);
        assert_eq!(input.ripen_bonus, 0);
        let undone = decision(
            &hit,
            "gm",
            Action::RollbackBattle {
                event_sequence: hit.battle_undo.last().unwrap().event_sequence,
                reason: "Restore thrown unit and Fling selection".into(),
            },
        );
        assert_eq!(
            serde_json::to_value(&undone.battle).unwrap(),
            serde_json::to_value(&c.battle).unwrap()
        );
    }
    let (c, a, t) = fling_setup("Oran Berry", "Unburden");
    let missed = super::targeting_tests::roll(
        &c,
        "player",
        &super::targeting_tests::attack(&a, vec![t], "Fling"),
        1,
    );
    assert_eq!(
        missed
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .units["berry"]
            .state,
        "ground"
    );
    assert!(missed
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .unburdened
        .contains(&a));
}
#[test]
fn items_fling_ripen_adds_two_dice_without_changing_item_bonus_and_can_be_unarmed() {
    let mut pools = vec![];
    for ability in ["", "Ripen"] {
        let (c, a, t) = fling_setup("Oran Berry", ability);
        let ready = super::targeting_tests::pass(&hit(&c, &a, &t, "Fling"));
        let command = super::targeting_tests::damage_command(&ready);
        pools.push(rules::dice_count(&ready, &command, &catalog()).unwrap());
        assert_eq!(
            ready
                .battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .items
                .fling_moves
                .values()
                .next()
                .unwrap()
                .item_bonus,
            2
        );
    }
    assert_eq!(pools[1], pools[0] + 2);
    let (mut c, a, t) = super::special_tests::setup("Fling");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .enabled = true;
    items::initialize(&mut c, &catalog()).unwrap();
    let done = hit(&c, &a, &t, "Fling");
    assert!(done
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .fling_moves
        .values()
        .next()
        .unwrap()
        .unit_id
        .is_none());
}
#[test]
fn items_fling_item_retrieval_costs_an_action_and_cannot_retrieve_consumed_berries() {
    let (c, a, t) = fling_setup("Leftovers", "");
    let mut landed = theft_damage(&c, &a, &t, "Fling");
    assert!(battle::transition(
        &landed,
        "player",
        &action(ItemCommand::Retrieve {
            combatant_id: a.clone(),
            unit_id: "berry".into()
        }),
        vec![],
        &catalog()
    )
    .is_err());
    let sequence = landed.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sequence;
    landed = decision(
        &landed,
        "player",
        Action::ResolveEffect {
            sequence,
            reason: "Finish the Fling chain before retrieving its item".into(),
        },
    );
    let flow = landed.battle.as_mut().unwrap().flow.as_mut().unwrap();
    flow.cursor = flow.order.iter().position(|id| id == &a).unwrap();
    let retrieve = action(ItemCommand::Retrieve {
        combatant_id: a.clone(),
        unit_id: "berry".into(),
    });
    assert!(battle::transition(&landed, "other", &retrieve, vec![], &catalog()).is_err());
    let count = p(&landed, &a).actions;
    let done = decision(&landed, "player", retrieve.clone());
    assert_eq!(p(&done, &a).actions, count + 1);
    assert_eq!(items::held(&done, &a)[0].id, "berry");
    let restored = decision(
        &done,
        "gm",
        Action::RollbackBattle {
            event_sequence: done.battle_undo.last().unwrap().event_sequence,
            reason: "Undo paid item retrieval".into(),
        },
    );
    assert_eq!(
        serde_json::to_value(&restored.battle).unwrap(),
        serde_json::to_value(&landed.battle).unwrap()
    );
    let (c, a, _) = setup("Oran Berry", "");
    let used = decision(&c, "player", use_berry(&a));
    assert!(battle::transition(
        &used,
        "player",
        &action(ItemCommand::Retrieve {
            combatant_id: a,
            unit_id: "berry".into()
        }),
        vec![],
        &catalog()
    )
    .is_err());
}
#[test]
fn items_paid_retrieval_obeys_truant_loyalty_and_gorilla_tactics() {
    for face in [1, 4] {
        let (mut c, a, _) = setup("Leftovers", "Truant");
        c = decision(
            &c,
            "gm",
            action(ItemCommand::Lose {
                combatant_id: a.clone(),
                unit_id: "berry".into(),
                override_sticky_hold: false,
            }),
        );
        super::sequencing_tests::loyal(&mut c, &a, 2);
        let flow = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
        flow.sequencing.enabled = true;
        flow.sequencing.visits.insert(a.clone(), 2);
        let command = action(ItemCommand::Retrieve {
            combatant_id: a.clone(),
            unit_id: "berry".into(),
        });
        assert_eq!(rules::dice_count(&c, &command, &catalog()).unwrap(), 2);
        let done = super::targeting_tests::roll(&c, "player", &command, face);
        assert_eq!(items::held(&done, &a).len(), usize::from(face == 4));
        assert_eq!(p(&done, &a).actions, u32::from(face == 4));
    }
    let (mut c, a, _) = setup("Leftovers", "Gorilla Tactics");
    c = decision(
        &c,
        "gm",
        action(ItemCommand::Lose {
            combatant_id: a.clone(),
            unit_id: "berry".into(),
            override_sticky_hold: false,
        }),
    );
    let flow = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    flow.sequencing.enabled = true;
    flow.sequencing.gorilla.insert(a.clone(), "Tackle".into());
    assert!(rules::dice_count(
        &c,
        &action(ItemCommand::Retrieve {
            combatant_id: a,
            unit_id: "berry".into()
        }),
        &catalog()
    )
    .is_err());
}
#[test]
fn items_cud_chew_berry_hit_resolves_after_damage_and_consumes_only_the_thrown_unit() {
    let (mut c, a, t) = fling_setup("Oran Berry", "");
    pm(&mut c, &t).selected_ability = "Cud Chew".into();
    let hit = theft_damage(&c, &a, &t, "Fling");
    let hp_after_damage = p(&hit, &t).hp;
    let event = &hit
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0];
    assert_eq!(event.kind, "item-cud-chew");
    let command = Action::ResolveAbility {
        event_id: event.id.clone(),
        choice: "activate".into(),
        reason: "Activate the thrown berry twice after damage".into(),
    };
    assert!(battle::transition(&hit, "player", &command, vec![], &catalog()).is_err());
    let first = decision(&hit, "other", command);
    assert_eq!(p(&first, &t).hp, (hp_after_damage + 1).min(20));
    let charge_id = first
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .chew_charges
        .keys()
        .next()
        .unwrap()
        .clone();
    let done = decision(
        &first,
        "other",
        action(ItemCommand::ChewAgain { charge_id }),
    );
    assert_eq!(p(&done, &t).hp, (hp_after_damage + 2).min(20));
    assert_eq!(
        done.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .units["berry"]
            .state,
        "consumed"
    );
    assert_eq!(
        done.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .units
            .len(),
        1
    );
    assert_eq!(items::held(&done, &a).len(), 0);
    let undo = decision(
        &done,
        "gm",
        Action::RollbackBattle {
            event_sequence: done.battle_undo.last().unwrap().event_sequence,
            reason: "Restore the unactivated berry hit".into(),
        },
    );
    assert_eq!(
        serde_json::to_value(&undo.battle).unwrap(),
        serde_json::to_value(&first.battle).unwrap()
    );
}

#[test]
fn items_paid_retrieval_uses_defeatists_loyalty_check_without_inventing_a_damage_roll() {
    let (mut c, a, _) = setup("Leftovers", "Defeatist");
    c = decision(
        &c,
        "gm",
        action(ItemCommand::Lose {
            combatant_id: a.clone(),
            unit_id: "berry".into(),
            override_sticky_hold: false,
        }),
    );
    pm(&mut c, &a).hp = 9;
    super::sequencing_tests::loyal(&mut c, &a, 2);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .sequencing
        .enabled = true;
    let command = action(ItemCommand::Retrieve {
        combatant_id: a.clone(),
        unit_id: "berry".into(),
    });
    assert_eq!(rules::dice_count(&c, &command, &catalog()).unwrap(), 2);
    let done = super::targeting_tests::roll(&c, "player", &command, 1);
    assert_eq!(items::held(&done, &a)[0].id, "berry");
    assert_eq!(p(&done, &a).actions, 1);
}

#[test]
fn items_fling_clash_records_the_projectile_before_cud_chews_hit_trigger() {
    let (mut c, a, t) = fling_setup("Oran Berry", "Ripen");
    pm(&mut c, &t).selected_ability = "Cud Chew".into();
    let c = decision(
        &c,
        "player",
        Action::PassTurn {
            combatant_id: a.clone(),
            reason: "Let the foe act before the Fling clash".into(),
        },
    );
    let incoming_action = super::targeting_tests::attack(&t, vec![a.clone()], "Tackle");
    let mut faces = vec![1; rules::dice_count(&c, &incoming_action, &catalog()).unwrap() as usize];
    faces[0] = 4;
    let incoming = battle::transition(&c, "other", &incoming_action, faces, &catalog())
        .unwrap()
        .0;
    let command = Action::React {
        combatant_id: a.clone(),
        kind: "clash".into(),
        move_name: "Fling".into(),
        reason: "Throw the held berry in Clash".into(),
    };
    let done = super::targeting_tests::roll(&incoming, "player", &command, 4);
    assert!(items::held(&done, &a).is_empty());
    assert_eq!(
        done.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .units["berry"]
            .state,
        "berry_hit_pending"
    );
    assert!(done
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events
        .iter()
        .any(|e| e.kind == "item-cud-chew" && e.combatant_id == t));
}

#[test]
fn items_fling_cannot_remove_sticky_hold_items_or_use_items_in_magic_room() {
    for blocker in ["Sticky Hold", "Magic Room"] {
        let (mut c, a, t) = fling_setup("Leftovers", "");
        if blocker == "Sticky Hold" {
            pm(&mut c, &a).selected_ability = blocker.into();
        } else {
            c.battle
                .as_mut()
                .unwrap()
                .flow
                .as_mut()
                .unwrap()
                .battlefield
                .rules
                .push(crate::combat::field::FieldRule {
                    id: "room".into(),
                    kind: "Magic Room".into(),
                    source: a.clone(),
                    scope: "all".into(),
                    rounds: None,
                    parameter: 0,
                    started_round: 1,
                });
        }
        let command = super::targeting_tests::attack(&a, vec![t], "Fling");
        assert!(
            rules::dice_count(&c, &command, &catalog()).is_err(),
            "{blocker}"
        );
        assert_eq!(items::held(&c, &a)[0].id, "berry");
    }
}

fn recycle_setup(name: &str, ability: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = super::special_tests::setup("Recycle");
    pm(&mut c, &a).selected_ability = ability.into();
    pm(&mut c, &a).hp = 12;
    let sid = p(&c, &t).sheet_id.clone();
    let mut sheet = c.sheets[&sid].clone();
    sheet.inventory.insert(name.into(), 1);
    c.sheets.insert(
        sid,
        items::equip(&sheet, name, "spent", &catalog()).unwrap(),
    );
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .enabled = true;
    items::initialize(&mut c, &catalog()).unwrap();
    let c = decision(
        &c,
        "gm",
        action(ItemCommand::Spend {
            combatant_id: t.clone(),
            unit_id: "spent".into(),
        }),
    );
    (c, a, t)
}
fn select_recycle(c: &Campaign, id: &str, unit: &str) -> Campaign {
    decision(
        c,
        "player",
        action(ItemCommand::Recycle {
            combatant_id: id.into(),
            unit_id: unit.into(),
        }),
    )
}
fn finish_recycle(c: &Campaign, id: &str) -> Campaign {
    let sequence = c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sequence;
    decision(
        c,
        if p(c, id).owner_id == "player" {
            "player"
        } else {
            "other"
        },
        Action::ResolveEffect {
            sequence,
            reason: "Reuse the reserved spent item".into(),
        },
    )
}
fn chew(c: &Campaign, actor: &str) -> Campaign {
    let charge_id = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .chew_charges
        .keys()
        .next()
        .unwrap()
        .clone();
    decision(c, actor, action(ItemCommand::ChewAgain { charge_id }))
}
#[test]
fn items_recycle_reuses_a_foes_spent_berry_without_creating_or_equipping_another_unit() {
    let (c, a, _) = recycle_setup("Oran Berry", "Cud Chew");
    let chosen = select_recycle(&c, &a, "spent");
    let pending = hit(&chosen, &a, &a, "Recycle");
    assert_eq!(
        pending
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .units["spent"]
            .state,
        "recycle_pending"
    );
    let done = finish_recycle(&pending, &a);
    assert_eq!(p(&done, &a).hp, 13);
    assert_eq!(p(&done, &a).actions, 1);
    let done = chew(&done, "player");
    assert_eq!(p(&done, &a).hp, 14);
    assert!(items::held(&done, &a).is_empty());
    let state = &done.battle.as_ref().unwrap().flow.as_ref().unwrap().items;
    assert_eq!(state.units.len(), 1);
    assert_eq!(state.units["spent"].state, "consumed");
    assert_eq!(state.recycle_uses[&a], 1);
    assert!(state.chew_charges.is_empty());
}
#[test]
fn items_cud_chew_saves_its_second_activation_for_a_later_round_and_shares_the_healing_cap() {
    let (mut c, a, _) = setup("Sitrus Berry", "Cud Chew");
    pm(&mut c, &a).runtime.healed = 2;
    let first = decision(&c, "player", use_berry(&a));
    assert_eq!(p(&first, &a).hp, 13);
    assert_eq!(p(&first, &a).runtime.healed, 3);
    let mut blocked = first.clone();
    crate::combat::special::heal(&mut blocked, &a, 1, false).unwrap();
    assert_eq!(p(&blocked, &a).hp, 13);
    let later = next_item_round(&first);
    let done = chew(&later, "player");
    assert_eq!(p(&done, &a).hp, 16);
    assert_eq!(p(&done, &a).actions, 0);
    assert!(done
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .chew_charges
        .is_empty());
    assert_eq!(
        done.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .units
            .len(),
        1
    );
}
#[test]
fn items_recycle_limit_is_three_attempts_even_with_gorilla_tactics_repeat_or_misses() {
    for face in [1, 4] {
        let (mut c, a, t) = recycle_setup("Oran Berry", "Gorilla Tactics");
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .sequencing
            .enabled = true;
        crate::combat::sequencing::lifecycle(&mut c, &catalog()).unwrap();
        let event_id = c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events[0]
            .id
            .clone();
        c = decision(
            &c,
            "player",
            Action::ResolveAbility {
                event_id,
                choice: "Recycle".into(),
                reason: "Choose Recycle as Gorilla Tactics' entry move".into(),
            },
        );
        for used in 1..=3 {
            if c.battle.as_ref().unwrap().flow.as_ref().unwrap().order
                [c.battle.as_ref().unwrap().flow.as_ref().unwrap().cursor]
                == t
            {
                c = decision(
                    &c,
                    "other",
                    Action::PassTurn {
                        combatant_id: t.clone(),
                        reason: "Hold while the opponent recycles".into(),
                    },
                );
            }
            c = select_recycle(&c, &a, "spent");
            c.revision += 1; // Match the host's committed-command sequence.
            c = super::targeting_tests::roll(
                &c,
                "player",
                &super::targeting_tests::attack(&a, vec![a.clone()], "Recycle"),
                face,
            );
            c.revision += 1;
            if face == 4 {
                c = finish_recycle(&c, &a);
                c.revision += 1;
            }
            assert_eq!(
                c.battle
                    .as_ref()
                    .unwrap()
                    .flow
                    .as_ref()
                    .unwrap()
                    .items
                    .recycle_uses[&a],
                used
            );
        }
        let fourth = action(ItemCommand::Recycle {
            combatant_id: a.clone(),
            unit_id: "spent".into(),
        });
        assert!(battle::transition(&c, "player", &fourth, vec![], &catalog()).is_err());
        assert_eq!(p(&c, &a).actions, 3);
    }
}
#[test]
fn items_recycle_reservation_effect_and_saved_chew_roll_back_together() {
    let (c, a, _) = recycle_setup("Pecha Berry", "Cud Chew");
    let selected = select_recycle(&c, &a, "spent");
    let pending = hit(&selected, &a, &a, "Recycle");
    let done = finish_recycle(&pending, &a);
    let undone = decision(
        &done,
        "gm",
        Action::RollbackBattle {
            event_sequence: done.battle_undo.last().unwrap().event_sequence,
            reason: "Restore the pending Recycle resource and remove its saved chew".into(),
        },
    );
    assert_eq!(
        serde_json::to_value(&undone.battle).unwrap(),
        serde_json::to_value(&pending.battle).unwrap()
    );
    let sequence = pending
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue[0]
        .sequence;
    let waived = decision(
        &pending,
        "gm",
        Action::AdjudicateEffect {
            sequence,
            reason: "Explicit GM waiver releases the spent item reservation".into(),
        },
    );
    assert_eq!(
        waived
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .units["spent"]
            .state,
        "consumed"
    );
    assert!(waived
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .recycle_moves
        .is_empty());
    assert_eq!(
        waived
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .recycle_uses[&a],
        1
    );
}
#[test]
fn items_recycled_medicine_obeys_printed_cures_cap_and_does_not_trigger_berry_abilities() {
    for (medicine, status, hp, cured) in [
        ("Potion", "", 14, false),
        ("Hyper Potion", "", 15, false),
        ("Antidote", "Badly Poisoned", 12, true),
        ("Burn Heal", "Burn 3", 12, true),
        ("Ice Heal", "Frozen Solid", 12, true),
        ("Full Heal", "Paralysis", 12, true),
    ] {
        let (mut c, a, _) = recycle_setup(medicine, "Cud Chew");
        // A disabled turn/status gate permits testing an effect's cure boundaries;
        // the native item effect still applies the shared healing cap.
        if !status.is_empty() {
            pm(&mut c, &a).runtime.statuses.push(crate::combat::Status {
                kind: status.into(),
                detail: "".into(),
                rounds: None,
                progress: 0,
            });
        }
        c.battle.as_mut().unwrap().automatic_validation = false;
        c = select_recycle(&c, &a, "spent");
        let pending = hit(&c, &a, &a, "Recycle");
        let done = finish_recycle(&pending, &a);
        assert_eq!(p(&done, &a).hp, hp, "{medicine}");
        if cured {
            assert!(
                !p(&done, &a)
                    .runtime
                    .statuses
                    .iter()
                    .any(|s| s.kind == status),
                "{medicine}"
            );
        }
        assert!(
            done.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .items
                .chew_charges
                .is_empty(),
            "{medicine}"
        );
    }
}
#[test]
fn items_unnerve_blocks_recycled_berries_but_not_recycled_medicine() {
    for (item, blocked) in [("Oran Berry", true), ("Potion", false)] {
        let (mut c, a, t) = recycle_setup(item, "");
        pm(&mut c, &t).selected_ability = "Unnerve".into();
        let choose = action(ItemCommand::Recycle {
            combatant_id: a.clone(),
            unit_id: "spent".into(),
        });
        assert_eq!(
            battle::transition(&c, "player", &choose, vec![], &catalog()).is_err(),
            blocked
        );
    }
}
#[test]
fn items_recycle_snatch_redirects_the_move_effect_without_redirecting_the_saved_ability_charge() {
    let (mut c, a, t) = recycle_setup("Oran Berry", "Cud Chew");
    pm(&mut c, &t).hp = 10;
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .special
        .effects
        .push(crate::combat::special::TimedEffect {
            name: "Snatch".into(),
            source: t.clone(),
            target: a.clone(),
            amount: 0,
            rounds: Some(1),
            started: 1,
            slot: "".into(),
        });
    c = select_recycle(&c, &a, "spent");
    let pending = hit(&c, &a, &a, "Recycle");
    let done = finish_recycle(&pending, &a);
    assert_eq!(p(&done, &a).hp, 12);
    assert_eq!(p(&done, &t).hp, 11);
    let done = chew(&done, "player");
    assert_eq!(p(&done, &a).hp, 13);
    assert_eq!(p(&done, &t).hp, 11);
}

#[test]
fn items_recycle_stacks_saved_chews_while_another_item_is_held() {
    let (mut c, a, _) = symbiosis_setup("Cud Chew");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().moves.push("Recycle".into());
    let used = decision(&c, "player", use_berry(&a));
    let given = decision(&used, "ally-owner", symbiosis_choice(&used, "gift"));
    let selected = select_recycle(&given, &a, "berry");
    let done = finish_recycle(&hit(&selected, &a, &a, "Recycle"), &a);
    assert_eq!(items::held(&done, &a)[0].id, "gift");
    let state = &done.battle.as_ref().unwrap().flow.as_ref().unwrap().items;
    assert_eq!(state.chew_charges.len(), 2);
    assert_eq!(state.units.len(), 2);
    assert_eq!(state.units["berry"].state, "consumed");
    assert!(crate::privacy::for_player(&done, "unrelated")
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .chew_charges
        .is_empty());
}
#[test]
fn items_saved_chew_keeps_ownership_and_unnerve_checks_and_unreviewed_recycle_stays_manual() {
    let (c, a, t) = setup("Oran Berry", "Cud Chew");
    let mut used = decision(&c, "player", use_berry(&a));
    let charge_id = used
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .chew_charges
        .keys()
        .next()
        .unwrap()
        .clone();
    let command = action(ItemCommand::ChewAgain { charge_id });
    assert!(battle::transition(&used, "other", &command, vec![], &catalog()).is_err());
    pm(&mut used, &t).selected_ability = "Unnerve".into();
    assert!(battle::transition(&used, "player", &command, vec![], &catalog()).is_err());
    assert_eq!(
        used.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .chew_charges
            .len(),
        1
    );
    let (c, a, _) = recycle_setup("Max Potion", "");
    let error = battle::transition(
        &c,
        "player",
        &action(ItemCommand::Recycle {
            combatant_id: a,
            unit_id: "spent".into(),
        }),
        vec![],
        &catalog(),
    )
    .unwrap_err();
    assert_eq!(error.code, "manual_rule");
}

#[test]
fn items_quick_claw_roll_loss_rollback_and_suppression() {
    let (mut c, a, _) = setup("Quick Claw", "");
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().started = false;
    pm(&mut c, &a).initiative_rolled = false;
    let roll = Action::Initiative {
        combatant_id: a.clone(),
    };
    let rolled = battle::transition(&c, "gm", &roll, vec![4], &catalog())
        .unwrap()
        .0;
    let mut legacy = c.clone();
    legacy
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .enabled = false;
    let ordinary = battle::transition(&legacy, "gm", &roll, vec![4], &catalog())
        .unwrap()
        .0;
    assert_eq!(p(&rolled, &a).initiative, p(&ordinary, &a).initiative + 2);
    let lost = decision(
        &rolled,
        "gm",
        action(ItemCommand::Lose {
            combatant_id: a.clone(),
            unit_id: "berry".into(),
            override_sticky_hold: false,
        }),
    );
    assert_eq!(p(&lost, &a).initiative, p(&ordinary, &a).initiative);
    let restored = decision(
        &lost,
        "gm",
        Action::RollbackBattle {
            event_sequence: lost.battle_undo.last().unwrap().event_sequence,
            reason: "Restore claw and initiative together".into(),
        },
    );
    assert_eq!(p(&restored, &a).initiative, p(&rolled, &a).initiative);
    pm(&mut c, &a).selected_ability = "Klutz".into();
    assert_eq!(items::initiative_bonus(&c, &a), 0);
    pm(&mut c, &a).selected_ability.clear();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .rules
        .push(crate::combat::field::FieldRule {
            id: "room".into(),
            kind: "Magic Room".into(),
            source: a.clone(),
            scope: "all".into(),
            rounds: Some(5),
            parameter: 0,
            started_round: 1,
        });
    assert_eq!(items::initiative_bonus(&c, &a), 0);
}

#[test]
fn items_accuracy_lenses_stack_with_abilities_and_respect_suppression() {
    let (mut c, a, _) = setup("Wide Lens", "");
    let boosted = rules::accuracy(&c, &a, "Tackle", 0, &catalog()).unwrap().0;
    let mut ordinary = c.clone();
    ordinary
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .enabled = false;
    let base = rules::accuracy(&ordinary, &a, "Tackle", 0, &catalog())
        .unwrap()
        .0;
    assert_eq!(boosted, base + 2);
    pm(&mut c, &a).selected_ability = "Klutz".into();
    assert_eq!(
        rules::accuracy(&c, &a, "Tackle", 0, &catalog()).unwrap().0,
        base
    );
    let (mut c, a, t) = setup("Zoom Lens", "");
    assert_eq!(
        crate::combat::reviewed::accuracy_penalty(&c, p(&c, &a), p(&c, &t), -5),
        -3
    );
    assert_eq!(
        crate::combat::reviewed::accuracy_penalty(&c, p(&c, &a), p(&c, &t), -1),
        0
    );
    assert_eq!(
        crate::combat::reviewed::accuracy_penalty(&c, p(&c, &a), p(&c, &t), 0),
        0
    );
    pm(&mut c, &a).selected_ability = "Klutz".into();
    assert_eq!(
        crate::combat::reviewed::accuracy_penalty(&c, p(&c, &a), p(&c, &t), -5),
        -5
    );
    // Zoom Lens corrects successes without erasing a move's printed Low Accuracy:
    // Compound Eyes can still add its separate ability dice.
    pm(&mut c, &a).selected_ability = "Compound Eyes".into();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .extra
        .enabled = true;
    let m = catalog().find("moves", "Name", "Thunder").unwrap();
    assert_eq!(crate::combat::extra::accuracy_bonus(&c, p(&c, &a), &m), 2);
    let _ = t;
}

#[test]
fn items_bright_powder_freezes_separate_target_accuracy() {
    let (mut c, a, t, h) = super::targeting_tests::multi("Discharge");
    let sid = p(&c, &h).sheet_id.clone();
    let mut sheet = c.sheets[&sid].clone();
    sheet.inventory.insert("Bright Powder".into(), 1);
    let sheet = items::equip(&sheet, "Bright Powder", "powder", &catalog()).unwrap();
    c.sheets.insert(sid, sheet);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .enabled = true;
    items::initialize(&mut c, &catalog()).unwrap();
    let attack = super::targeting_tests::attack(&a, vec![t.clone(), h.clone()], "Discharge");
    let n = rules::dice_count(&c, &attack, &catalog()).unwrap() as usize;
    let mut faces = vec![1; n];
    faces[0] = 4;
    let done = battle::transition(&c, "player", &attack, faces, &catalog())
        .unwrap()
        .0;
    let q = &done.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    assert_eq!(
        q.targets
            .iter()
            .find(|v| v.target_id == t)
            .unwrap()
            .successes,
        Some(1)
    );
    assert_eq!(
        q.targets
            .iter()
            .find(|v| v.target_id == h)
            .unwrap()
            .successes,
        Some(0)
    );
    assert!(q.targets.iter().find(|v| v.target_id == h).unwrap().avoided);
    pm(&mut c, &h).selected_ability = "Klutz".into();
    assert_eq!(
        crate::combat::reviewed::accuracy_penalty(&c, p(&c, &a), p(&c, &h), 0),
        0
    );
    pm(&mut c, &h).selected_ability.clear();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .reviewed_rules = true;
    pm(&mut c, &a).selected_ability = "Keen Eye".into();
    assert_eq!(
        crate::combat::reviewed::accuracy_penalty(&c, p(&c, &a), p(&c, &h), -3),
        0
    );
}

#[test]
fn items_iron_ball_reduction_shares_move_channel_and_ignores_simple_contrary() {
    for ability in ["", "Simple", "Contrary"] {
        let (mut c, a, _) = setup("Iron Ball", ability);
        let base = rules::base_sheet(&c, p(&c, &a)).unwrap().stats["Dexterity"];
        assert_eq!(
            rules::effective_sheet(&c, p(&c, &a)).unwrap().stats["Dexterity"],
            (base - 1).max(1)
        );
        // An item is not inverted or amplified by move-modifying abilities.
        assert_eq!(
            crate::combat::special::modifier(&c, p(&c, &a), "Dexterity"),
            Some(-1)
        );
        pm(&mut c, &a)
            .runtime
            .modifiers
            .insert("Dexterity".into(), -2);
        let expected = match ability {
            "Simple" => -3,
            "Contrary" => 1,
            _ => -2,
        };
        assert_eq!(
            crate::combat::special::modifier(&c, p(&c, &a), "Dexterity"),
            Some(expected)
        );
    }
}

#[test]
fn items_iron_ball_removes_flying_and_levitate_immunity_until_suppressed_or_lost() {
    let (mut c, a, t) = setup("Iron Ball", "Levitate");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some(
        catalog().find("species", "Name", "Pidgey").unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .species_types
        .insert(a.clone(), vec!["Flying".into()]);
    let ground = catalog().find("moves", "Name", "Earthquake").unwrap();
    assert!(
        !rules::type_matchup(&c, p(&c, &t), p(&c, &a), "Ground", &catalog())
            .unwrap()
            .1
    );
    assert!(!crate::combat::abilities::move_immune(
        &c,
        p(&c, &a),
        &ground
    ));
    assert!(crate::combat::field::grounded(&c, p(&c, &a), &catalog()));
    let fissure = catalog().find("moves", "Name", "Fissure").unwrap();
    assert!(
        !crate::combat::special::formula(&c, p(&c, &t), p(&c, &a), &fissure, 0, 0, &catalog())
            .unwrap()
            .unwrap()
            .immune
    );
    let lost = decision(
        &c,
        "gm",
        action(ItemCommand::Lose {
            combatant_id: a.clone(),
            unit_id: "berry".into(),
            override_sticky_hold: false,
        }),
    );
    assert!(
        rules::type_matchup(&lost, p(&lost, &t), p(&lost, &a), "Ground", &catalog())
            .unwrap()
            .1
    );
    assert!(crate::combat::abilities::move_immune(
        &lost,
        p(&lost, &a),
        &ground
    ));
    assert!(!crate::combat::field::grounded(
        &lost,
        p(&lost, &a),
        &catalog()
    ));
    pm(&mut c, &a).selected_ability = "Klutz".into();
    assert!(
        rules::type_matchup(&c, p(&c, &t), p(&c, &a), "Ground", &catalog())
            .unwrap()
            .1
    );
    assert!(!items::iron_ball(&c, &a));
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .enabled = false;
    pm(&mut c, &a).selected_ability = "Levitate".into();
    assert!(crate::combat::abilities::move_immune(
        &c,
        p(&c, &a),
        &ground
    ));
}

#[test]
fn items_eviolite_uses_reviewed_stage_and_combines_move_and_ability_channels() {
    for (name, stage, bonus) in [("Bulbasaur", 1, 1), ("Ivysaur", 2, 1), ("Arbok", 3, 0)] {
        let (mut c, a, _) = setup("Eviolite", "Marvel Scale");
        let species = catalog().find("species", "Name", name).unwrap();
        assert_eq!(species["Creation"]["stage"], stage);
        let sid = p(&c, &a).sheet_id.clone();
        c.sheets.get_mut(&sid).unwrap().species_id =
            Some(species["AppId"].as_str().unwrap().into());
        items::initialize(&mut c, &catalog()).unwrap();
        pm(&mut c, &a).runtime.statuses.push(crate::combat::Status {
            kind: "Poison".into(),
            detail: "".into(),
            rounds: None,
            progress: 0,
        });
        assert_eq!(
            crate::combat::abilities::defense(&c, p(&c, &a), true),
            2 + bonus
        );
        assert_eq!(
            crate::combat::abilities::defense(&c, p(&c, &a), false),
            bonus
        );
        // A stronger move bonus replaces Eviolite in its channel; Marvel Scale still adds.
        pm(&mut c, &a).runtime.modifiers.insert("Defense".into(), 2);
        assert_eq!(crate::combat::abilities::defense(&c, p(&c, &a), true), 4);
        pm(&mut c, &a).selected_ability = "Klutz".into();
        assert_eq!(crate::combat::abilities::defense(&c, p(&c, &a), false), 0);
    }
}

#[test]
fn items_eviolite_is_not_inverted_or_doubled_and_is_saved_in_defense_snapshots() {
    for ability in ["Simple", "Contrary"] {
        let (mut c, a, _) = setup("Eviolite", ability);
        assert_eq!(
            crate::combat::special::modifier(&c, p(&c, &a), "Defense"),
            Some(1)
        );
        let saved = crate::combat::field::snapshot(&c, &catalog()).unwrap();
        let lost = decision(
            &c,
            "gm",
            action(ItemCommand::Lose {
                combatant_id: a.clone(),
                unit_id: "berry".into(),
                override_sticky_hold: false,
            }),
        );
        let after = crate::combat::field::snapshot(&lost, &catalog()).unwrap();
        for index in [0, 1] {
            assert_eq!(saved.defenses[&a][index], after.defenses[&a][index] + 1);
        }
        pm(&mut c, &a)
            .runtime
            .modifiers
            .insert("Defense".into(), -2);
        assert_eq!(
            crate::combat::special::modifier(&c, p(&c, &a), "Defense"),
            Some(if ability == "Simple" { -2 } else { 2 })
        );
    }
    let (mut c, a, _) = setup("Eviolite", "");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = None;
    assert_eq!(
        items::initialize(&mut c, &catalog()).unwrap_err().code,
        "manual_rule"
    );
}

#[test]
fn items_all_type_boosters_match_resolved_type_and_respect_klutz() {
    for (name, kind) in [
        ("Black Belt", "Fight"),
        ("Black Glasses", "Dark"),
        ("Charcoal", "Fire"),
        ("Dragon Fang", "Dragon"),
        ("Fairy Wings", "Fairy"),
        ("Hard Stone", "Rock"),
        ("Magnet", "Electric"),
        ("Metal Coat", "Steel"),
        ("Miracle Seed", "Grass"),
        ("Mystic Water", "Water"),
        ("Never-Melt Ice", "Ice"),
        ("Poison Barb", "Poison"),
        ("Sharp Beak", "Flying"),
        ("Silk Scarf", "Normal"),
        ("Silver Powder", "Bug"),
        ("Soft Sand", "Ground"),
        ("Spell Tag", "Ghost"),
        ("Twisted Spoon", "Psychic"),
    ] {
        let (mut c, a, _) = setup(name, "");
        let mut m = catalog().find("moves", "Name", "Tackle").unwrap();
        m["Type"] = serde_json::json!(kind);
        assert_eq!(
            items::type_damage_bonus(&c, p(&c, &a), &m, &catalog()).unwrap(),
            1,
            "{name}"
        );
        m["Type"] = serde_json::json!(if kind == "Normal" { "Fire" } else { "Normal" });
        assert_eq!(
            items::type_damage_bonus(&c, p(&c, &a), &m, &catalog()).unwrap(),
            0
        );
        m["Type"] = serde_json::json!(kind);
        pm(&mut c, &a).selected_ability = "Klutz".into();
        assert_eq!(
            items::type_damage_bonus(&c, p(&c, &a), &m, &catalog()).unwrap(),
            0
        );
    }
    let (mut c, a, _) = setup("Silk Scarf", "Normalize");
    let fire = catalog().find("moves", "Name", "Ember").unwrap();
    let normal = crate::combat::field::move_data(&c, p(&c, &a), &fire);
    assert_eq!(normal["Type"], "Normal");
    assert_eq!(
        items::type_damage_bonus(&c, p(&c, &a), &normal, &catalog()).unwrap(),
        1
    );
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .enabled = false;
    assert_eq!(
        items::type_damage_bonus(&c, p(&c, &a), &normal, &catalog()).unwrap(),
        0
    );
}

#[test]
fn items_type_boost_is_saved_across_multi_target_item_loss() {
    use super::targeting_tests::{attack, damage_command, pass, roll};
    let (mut c, a, t, high) = super::targeting_tests::multi("Swift");
    let sid = p(&c, &a).sheet_id.clone();
    let mut sheet = c.sheets[&sid].clone();
    sheet.inventory.insert("Silk Scarf".into(), 1);
    let equipped = items::equip(&sheet, "Silk Scarf", "scarf", &catalog()).unwrap();
    c.sheets.insert(sid, equipped);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .enabled = true;
    items::initialize(&mut c, &catalog()).unwrap();
    let c = roll(&c, "player", &attack(&a, vec![t, high], "Swift"), 4);
    let c = pass(&pass(&c));
    let c = roll(&c, "player", &damage_command(&c), 4);
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    assert_eq!(q.damage_context.as_ref().unwrap().item_damage_bonus, 1);
    let before = rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap();
    let mut without_bonus = c.clone();
    without_bonus
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .queue[0]
        .damage_context
        .as_mut()
        .unwrap()
        .item_damage_bonus = 0;
    assert_eq!(
        before,
        rules::dice_count(&without_bonus, &damage_command(&without_bonus), &catalog()).unwrap() + 1
    );
    let lost = decision(
        &c,
        "gm",
        action(ItemCommand::Lose {
            combatant_id: a.clone(),
            unit_id: "scarf".into(),
            override_sticky_hold: false,
        }),
    );
    assert!(items::held(&lost, &a).is_empty());
    assert_eq!(
        rules::dice_count(&lost, &damage_command(&lost), &catalog()).unwrap(),
        before
    );
}

#[test]
fn items_expert_belt_adds_damage_not_dice_to_ordinary_and_special_formulas() {
    for (name, species) in [("Ember", "Bulbasaur"), ("Night Shade", "Gastly")] {
        let mut outcomes = vec![];
        for enabled in [false, true] {
            let (mut c, a, t) = setup("Expert Belt", "");
            let sid = p(&c, &a).sheet_id.clone();
            c.sheets.get_mut(&sid).unwrap().moves.push(name.into());
            let sid = p(&c, &t).sheet_id.clone();
            c.sheets.get_mut(&sid).unwrap().species_id = Some(
                catalog().find("species", "Name", species).unwrap()["AppId"]
                    .as_str()
                    .unwrap()
                    .into(),
            );
            c.battle
                .as_mut()
                .unwrap()
                .flow
                .as_mut()
                .unwrap()
                .items
                .enabled = enabled;
            let c = hit(&c, &a, &t, name);
            let c = super::targeting_tests::pass(&c);
            let command = super::targeting_tests::damage_command(&c);
            let dice = rules::dice_count(&c, &command, &catalog()).unwrap();
            let done = super::targeting_tests::roll(&c, "player", &command, 4);
            outcomes.push((dice, p(&done, &t).hp));
        }
        assert_eq!(outcomes[0].0, outcomes[1].0, "{name} must not gain dice");
        assert_eq!(outcomes[0].1, outcomes[1].1 + 1, "{name} gains one damage");
    }
}

#[test]
fn items_expert_belt_clash_and_suppression_boundaries() {
    let mut damage = vec![];
    for ability in ["", "Klutz"] {
        let (mut c, a, t) = setup("Expert Belt", ability);
        let sid = p(&c, &a).sheet_id.clone();
        c.sheets.get_mut(&sid).unwrap().moves.push("Ember".into());
        c.sheets
            .get_mut(&sid)
            .unwrap()
            .skills
            .insert("Clash".into(), 5);
        let sid = p(&c, &t).sheet_id.clone();
        c.sheets.get_mut(&sid).unwrap().species_id = Some(
            catalog().find("species", "Name", "Bulbasaur").unwrap()["AppId"]
                .as_str()
                .unwrap()
                .into(),
        );
        assert_eq!(
            items::extra_damage(&c, &a, &t, 0, (0, false), &catalog()).unwrap(),
            0
        );
        assert_eq!(
            items::extra_damage(&c, &a, &t, 0, (1, true), &catalog()).unwrap(),
            0
        );
        let c = decision(
            &c,
            "player",
            Action::PassTurn {
                combatant_id: a.clone(),
                reason: "Let foe attack".into(),
            },
        );
        let c = super::targeting_tests::roll(
            &c,
            "other",
            &super::targeting_tests::attack(&t, vec![a.clone()], "Tackle"),
            4,
        );
        let command = Action::React {
            combatant_id: a.clone(),
            kind: "clash".into(),
            move_name: "Ember".into(),
            reason: "Counter with a super-effective move".into(),
        };
        let done = super::targeting_tests::roll(&c, "player", &command, 4);
        damage.push(p(&done, &t).hp);
    }
    assert_eq!(damage[0] + 1, damage[1]);
}

#[test]
fn items_expert_belt_shared_damage_keeps_saved_item_after_loss() {
    use super::targeting_tests::{attack, damage_command, pass, roll};
    let (mut c, a, t, high) = super::targeting_tests::multi("Discharge");
    let sid = p(&c, &a).sheet_id.clone();
    let mut sheet = c.sheets[&sid].clone();
    sheet.inventory.insert("Expert Belt".into(), 1);
    c.sheets.insert(
        sid,
        items::equip(&sheet, "Expert Belt", "belt", &catalog()).unwrap(),
    );
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .enabled = true;
    for target in [&t, &high] {
        let sid = p(&c, target).sheet_id.clone();
        c.sheets.get_mut(&sid).unwrap().species_id = Some(
            catalog().find("species", "Name", "Squirtle").unwrap()["AppId"]
                .as_str()
                .unwrap()
                .into(),
        );
    }
    items::initialize(&mut c, &catalog()).unwrap();
    let c = roll(&c, "player", &attack(&a, vec![t, high], "Discharge"), 4);
    let c = pass(&pass(&c));
    let c = roll(&c, "player", &damage_command(&c), 4);
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    assert!(q.damage_context.as_ref().unwrap().expert_belt);
    let sequence = q.sequence;
    let target = q.target_id.clone();
    let mut saved = c.clone();
    let copy = q.clone();
    let f = saved.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.special.charges.insert(
        a.clone(),
        crate::combat::special::Charge {
            action: copy.clone(),
            remaining: 1,
            turn: 1,
        },
    );
    f.special.series = Some(crate::combat::special::Series {
        follow_up: vec![copy.clone()],
        actor_id: a.clone(),
        target_id: target.clone(),
        name: "Discharge".into(),
        attempts: 1,
        start_actions: 0,
        preparing: false,
        pending: vec![copy.clone()],
        landed: 1,
    });
    f.tactics.clash_pending.insert(sequence, (copy.clone(), 1));
    f.sequencing.pending.push(copy.clone());
    f.sequencing.dances.insert(a.clone(), copy);
    let mut archive = saved.battle.as_ref().unwrap().clone();
    archive.active = false;
    saved.finished_battles.push(archive);
    let mut checkpoint = saved.battle_undo.last().unwrap().clone();
    checkpoint.battle = saved.battle.as_ref().unwrap().clone();
    saved.battle_undo.push(checkpoint);
    let authoritative = serde_json::to_string(&saved).unwrap();
    assert!(authoritative.contains("\"expert_belt\":true"));
    for viewer in ["other", "observer"] {
        let projected = crate::privacy::for_player(&saved, viewer);
        assert!(!serde_json::to_string(&projected)
            .unwrap()
            .contains("\"expert_belt\":true"));
    }
    for viewer in ["player", "gm"] {
        assert!(
            serde_json::to_string(&crate::privacy::for_player(&saved, viewer))
                .unwrap()
                .contains("\"expert_belt\":true")
        );
    }
    assert_eq!(serde_json::to_string(&saved).unwrap(), authoritative);
    let lost = decision(
        &c,
        "gm",
        action(ItemCommand::Lose {
            combatant_id: a.clone(),
            unit_id: "belt".into(),
            override_sticky_hold: false,
        }),
    );
    assert!(!items::expert_belt(&lost, &a));
    let matchup = rules::type_matchup(
        &lost,
        p(&lost, &a),
        p(&lost, &target),
        "Electric",
        &catalog(),
    )
    .unwrap();
    assert_eq!(matchup, (1, false));
    assert_eq!(
        items::extra_damage(&lost, &a, &target, sequence, matchup, &catalog()).unwrap(),
        1
    );
}

#[test]
fn items_expert_belt_respects_strong_winds_and_magic_room() {
    let (mut c, a, t) = setup("Expert Belt", "");
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some(
        catalog().find("species", "Name", "Pidgey").unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
    assert_eq!(
        items::extra_damage(&c, &a, &t, 0, (1, false), &catalog()).unwrap(),
        1
    );
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .natural = Some(crate::combat::field::Weather::StrongWinds);
    assert_eq!(
        items::extra_damage(&c, &a, &t, 0, (1, false), &catalog()).unwrap(),
        0
    );
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .natural = None;
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .rules
        .push(crate::combat::field::FieldRule {
            id: "room".into(),
            kind: "Magic Room".into(),
            source: a.clone(),
            scope: "all".into(),
            rounds: Some(5),
            parameter: 0,
            started_round: 1,
        });
    assert_eq!(
        items::extra_damage(&c, &a, &t, 0, (1, false), &catalog()).unwrap(),
        0
    );
}

fn orb_damage(name: &str, ability: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = setup("Life Orb", ability);
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().moves.push(name.into());
    if name == "Night Shade" {
        let sid = p(&c, &t).sheet_id.clone();
        c.sheets.get_mut(&sid).unwrap().species_id = Some(
            catalog().find("species", "Name", "Gastly").unwrap()["AppId"]
                .as_str()
                .unwrap()
                .into(),
        );
    }
    let c = hit(&c, &a, &t, name);
    let c = super::targeting_tests::pass(&c);
    let done =
        super::targeting_tests::roll(&c, "player", &super::targeting_tests::damage_command(&c), 4);
    (done, a, t)
}
fn orb_effect(c: &Campaign) -> Action {
    Action::ResolveEffect {
        sequence: c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sequence,
        reason: "Resolve the committed recoil and original move effects".into(),
    }
}
#[test]
fn items_life_orb_boosts_two_dice_and_recoil_survives_loss_with_rollback_privacy() {
    let (mut c, a, t) = setup("Life Orb", "");
    let move_data = catalog().find("moves", "Name", "Tackle").unwrap();
    assert_eq!(
        items::damage_dice_bonus(&c, p(&c, &a), 1, &move_data, &catalog()).unwrap(),
        2
    );
    pm(&mut c, &a).selected_ability = "Klutz".into();
    assert_eq!(
        items::damage_dice_bonus(&c, p(&c, &a), 1, &move_data, &catalog()).unwrap(),
        0
    );
    let (c, a, target) = orb_damage("Tackle", "");
    let (ordinary, _, other_target) = orb_damage("Tackle", "Klutz");
    assert_eq!(p(&ordinary, &other_target).hp, p(&c, &target).hp + 2);
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    let expected = q.raw_damage_successes;
    let command = orb_effect(&c);
    assert_eq!(
        rules::dice_count(&c, &command, &catalog()).unwrap(),
        expected
    );
    let projected = crate::privacy::for_player(&c, "other");
    assert!(projected
        .battle
        .unwrap()
        .flow
        .unwrap()
        .items
        .life_orb_moves
        .is_empty());
    assert!(!crate::privacy::for_player(&c, "player")
        .battle
        .unwrap()
        .flow
        .unwrap()
        .items
        .life_orb_moves
        .is_empty());
    let lost = decision(
        &c,
        "gm",
        action(ItemCommand::Lose {
            combatant_id: a.clone(),
            unit_id: "berry".into(),
            override_sticky_hold: false,
        }),
    );
    assert!(items::held(&lost, &a).is_empty());
    assert_eq!(
        rules::dice_count(&lost, &orb_effect(&lost), &catalog()).unwrap(),
        expected
    );
    let done = super::targeting_tests::roll(&lost, "player", &orb_effect(&lost), 4);
    assert_eq!(p(&done, &a).hp, (12 - expected as i32).max(0));
    let restored = decision(
        &done,
        "gm",
        Action::RollbackBattle {
            event_sequence: done.battle_undo.last().unwrap().event_sequence,
            reason: "Restore recoil and its pending resolution".into(),
        },
    );
    assert_eq!(
        serde_json::to_value(&restored.battle).unwrap(),
        serde_json::to_value(&lost.battle).unwrap()
    );
    let _ = t;
}
#[test]
fn items_life_orb_recoil_protections_and_native_recoil_are_not_duplicated() {
    for ability in ["Klutz", "Rock Head", "Magic Guard", "Wonder Guard"] {
        let (c, a, _) = orb_damage("Tackle", ability);
        assert_eq!(p(&c, &a).hp, 12, "{ability}");
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
    let (c, a, _) = orb_damage("Submission", "");
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    assert_eq!(
        rules::dice_count(&c, &orb_effect(&c), &catalog()).unwrap(),
        q.raw_damage_successes
    );
    let expected = q.raw_damage_successes as i32;
    let done = super::targeting_tests::roll(&c, "player", &orb_effect(&c), 4);
    assert_eq!(p(&done, &a).hp, (12 - expected).max(0));
}
#[test]
fn items_life_orb_preserves_original_effects_recharge_and_sheer_force_rules() {
    let (c, a, t) = orb_damage("Thunderbolt", "");
    let command = orb_effect(&c);
    let n = rules::dice_count(&c, &command, &catalog()).unwrap() as usize;
    let mut faces = vec![1; n];
    faces[n - 1] = 6;
    let done = battle::transition(&c, "player", &command, faces, &catalog())
        .unwrap()
        .0;
    assert_eq!(p(&done, &a).hp, 12);
    assert!(p(&done, &t)
        .runtime
        .statuses
        .iter()
        .any(|status| status.kind == "Paralysis"));
    let (c, a, _) = orb_damage("Hyper Beam", "");
    let done = super::targeting_tests::roll(&c, "player", &orb_effect(&c), 1);
    assert_eq!(p(&done, &a).runtime.recharge_actions, 1);
    let (mut c, a, t) = setup("Life Orb", "Sheer Force");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .provenance_rules = true;
    pm(&mut c, &a)
        .runtime
        .ability
        .flags
        .insert("sheer-force:damage".into());
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().moves.push("Ember".into());
    let c = hit(&c, &a, &t, "Ember");
    let c = super::targeting_tests::pass(&c);
    let c =
        super::targeting_tests::roll(&c, "player", &super::targeting_tests::damage_command(&c), 4);
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    assert!(q.sheer_force);
    assert_eq!(
        rules::dice_count(&c, &orb_effect(&c), &catalog()).unwrap(),
        q.raw_damage_successes
    );
    let done = super::targeting_tests::roll(&c, "player", &orb_effect(&c), 1);
    assert!(!p(&done, &t)
        .runtime
        .statuses
        .iter()
        .any(|status| status.kind.starts_with("Burn")));
}
#[test]
fn items_life_orb_shared_roll_has_one_recoil_obligation_after_the_last_target() {
    use super::targeting_tests::{attack, damage_command, pass, roll};
    let (mut c, a, t, high) = super::targeting_tests::multi("Swift");
    let sid = p(&c, &a).sheet_id.clone();
    let mut sheet = c.sheets[&sid].clone();
    sheet.inventory.insert("Life Orb".into(), 1);
    c.sheets.insert(
        sid,
        items::equip(&sheet, "Life Orb", "orb", &catalog()).unwrap(),
    );
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .enabled = true;
    items::initialize(&mut c, &catalog()).unwrap();
    let c = roll(&c, "player", &attack(&a, vec![t, high], "Swift"), 4);
    let c = pass(&pass(&c));
    let c = roll(&c, "player", &damage_command(&c), 4);
    assert_eq!(p(&c, &a).hp, 20);
    let c = decision(
        &c,
        "gm",
        action(ItemCommand::Lose {
            combatant_id: a.clone(),
            unit_id: "orb".into(),
            override_sticky_hold: false,
        }),
    );
    let c = roll(&c, "player", &damage_command(&c), 4);
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    let expected = rules::successes(&q.shared_dice) as u32;
    assert_eq!(
        rules::dice_count(&c, &orb_effect(&c), &catalog()).unwrap(),
        expected
    );
    let done = roll(&c, "player", &orb_effect(&c), 4);
    assert_eq!(p(&done, &a).hp, (20 - expected as i32).max(0));
    assert!(done
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .is_empty());
}
#[test]
fn items_life_orb_varied_formula_gets_dice_but_set_damage_has_no_damage_roll() {
    for name in ["Night Shade", "Dragon Rage", "Final Gambit"] {
        let (c, a, _) = orb_damage(name, "");
        if name != "Night Shade" {
            assert_eq!(p(&c, &a).hp, if name == "Final Gambit" { 0 } else { 12 });
            assert!(c
                .battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .queue
                .is_empty());
        } else {
            let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
            assert!(q.raw_damage_successes >= 2);
            let done = super::targeting_tests::roll(&c, "player", &orb_effect(&c), 1);
            assert_eq!(p(&done, &a).hp, 12);
        }
    }
}

fn chance_setup(
    item: &str,
    name: &str,
    ability: &str,
    target_ability: &str,
) -> (Campaign, String, String) {
    let (mut c, a, t) = setup(item, ability);
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().moves.push(name.into());
    pm(&mut c, &t).selected_ability = target_ability.into();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .reviewed_rules = true;
    (c, a, t)
}
fn chance_damage(c: &Campaign, a: &str, t: &str, name: &str) -> Campaign {
    let c = hit(c, a, t, name);
    let c = super::targeting_tests::pass(&c);
    super::targeting_tests::roll(&c, "player", &super::targeting_tests::damage_command(&c), 4)
}
#[test]
fn items_flinch_items_have_different_categories_and_saved_private_effects() {
    for (item, name, eligible) in [
        ("Razor Fang", "Tackle", true),
        ("Razor Fang", "Swift", false),
        ("King's Rock", "Tackle", false),
        ("King's Rock", "Swift", true),
    ] {
        let (c, a, t) = chance_setup(item, name, "", "");
        let c = chance_damage(&c, &a, &t, name);
        if !eligible {
            assert!(c
                .battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .queue
                .is_empty());
            continue;
        }
        assert_eq!(
            rules::dice_count(&c, &orb_effect(&c), &catalog()).unwrap(),
            1
        );
        let view = crate::privacy::for_player(&c, "other");
        assert!(view
            .battle
            .unwrap()
            .flow
            .unwrap()
            .items
            .chance_moves
            .is_empty());
        let lost = decision(
            &c,
            "gm",
            action(ItemCommand::Lose {
                combatant_id: a.clone(),
                unit_id: "berry".into(),
                override_sticky_hold: false,
            }),
        );
        let done = super::targeting_tests::roll(&lost, "player", &orb_effect(&lost), 6);
        assert!(p(&done, &t)
            .runtime
            .statuses
            .iter()
            .any(|status| status.kind == "Flinch"));
        let restored = decision(
            &done,
            "gm",
            Action::RollbackBattle {
                event_sequence: done.battle_undo.last().unwrap().event_sequence,
                reason: "Restore the saved item chance and unflinched target".into(),
            },
        );
        assert_eq!(
            serde_json::to_value(&restored.battle).unwrap(),
            serde_json::to_value(&lost.battle).unwrap()
        );
    }
}
#[test]
fn items_item_only_and_combined_added_effects_enable_sheer_force_once() {
    for name in ["Tackle", "Hyper Fang"] {
        let mut results = vec![];
        for ignore in [false, true] {
            let (mut c, a, t) = chance_setup("Razor Fang", name, "Sheer Force", "");
            c.battle
                .as_mut()
                .unwrap()
                .flow
                .as_mut()
                .unwrap()
                .provenance_rules = true;
            if ignore {
                pm(&mut c, &a)
                    .runtime
                    .ability
                    .flags
                    .insert("sheer-force:damage".into());
            }
            let c = hit(&c, &a, &t, name);
            assert_eq!(
                c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sheer_force,
                ignore
            );
            let c = super::targeting_tests::pass(&c);
            let command = super::targeting_tests::damage_command(&c);
            let dice = rules::dice_count(&c, &command, &catalog()).unwrap();
            let done = super::targeting_tests::roll(&c, "player", &command, 4);
            if ignore {
                assert!(done
                    .battle
                    .as_ref()
                    .unwrap()
                    .flow
                    .as_ref()
                    .unwrap()
                    .queue
                    .is_empty());
            } else {
                assert!(rules::dice_count(&done, &orb_effect(&done), &catalog()).unwrap() > 0);
            }
            results.push(dice);
        }
        assert_eq!(
            results[1],
            results[0] + 2,
            "{name}: item and move effects must not stack Sheer Force bonuses"
        );
    }
}
#[test]
fn items_flinch_pool_merges_without_duplicating_serene_grace() {
    for (source, target, expected) in [
        ("", "", 2),
        ("Serene Grace", "", 4),
        ("Serene Grace", "Wonder Skin", 2),
        ("", "Wonder Skin", 0),
    ] {
        let (c, a, t) = chance_setup("Razor Fang", "Hyper Fang", source, target);
        let c = chance_damage(&c, &a, &t, "Hyper Fang");
        assert_eq!(
            rules::dice_count(&c, &orb_effect(&c), &catalog()).unwrap(),
            expected
        );
        let done = super::targeting_tests::roll(&c, "player", &orb_effect(&c), 6);
        assert_eq!(
            p(&done, &t)
                .runtime
                .statuses
                .iter()
                .filter(|status| status.kind == "Flinch")
                .count(),
            usize::from(expected > 0)
        );
    }
    // Distinct paralysis and item-Flinch pools preserve their own chance rolls.
    for flinch in [false, true] {
        let (c, a, t) = chance_setup("King's Rock", "Thunderbolt", "Serene Grace", "");
        let c = chance_damage(&c, &a, &t, "Thunderbolt");
        assert_eq!(
            rules::dice_count(&c, &orb_effect(&c), &catalog()).unwrap(),
            6
        );
        let mut faces = vec![1; 6];
        faces[if flinch { 0 } else { 3 }] = 6;
        let done = battle::transition(&c, "player", &orb_effect(&c), faces, &catalog())
            .unwrap()
            .0;
        assert_eq!(
            p(&done, &t)
                .runtime
                .statuses
                .iter()
                .any(|s| s.kind == "Flinch"),
            flinch
        );
        assert_eq!(
            p(&done, &t)
                .runtime
                .statuses
                .iter()
                .any(|s| s.kind == "Paralysis"),
            !flinch
        );
    }
}
#[test]
fn items_flinch_respects_blockers_and_extreme_ability_bypass() {
    for (source, target, expected) in [
        ("", "Shield Dust", false),
        ("Mold Breaker", "Shield Dust", false),
        ("Teravolt", "Shield Dust", true),
        ("Turboblaze", "Wonder Skin", true),
        ("", "Inner Focus", false),
        ("Teravolt", "Inner Focus", true),
    ] {
        let (c, a, t) = chance_setup("Razor Fang", "Tackle", source, target);
        let c = chance_damage(&c, &a, &t, "Tackle");
        let done = if c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .queue
            .is_empty()
        {
            c
        } else {
            super::targeting_tests::roll(&c, "player", &orb_effect(&c), 6)
        };
        assert_eq!(
            p(&done, &t)
                .runtime
                .statuses
                .iter()
                .any(|s| s.kind == "Flinch"),
            expected,
            "{source} vs {target}"
        );
    }
    for decoy in [false, true] {
        let (mut c, a, t) = chance_setup("Razor Fang", "Tackle", "", "");
        if decoy {
            c.battle
                .as_mut()
                .unwrap()
                .flow
                .as_mut()
                .unwrap()
                .decoys
                .enabled = true;
            crate::combat::decoys::cast(&mut c, &t, "Substitute", &catalog()).unwrap();
        } else {
            let sid = p(&c, &t).sheet_id.clone();
            c.sheets.get_mut(&sid).unwrap().species_id = Some(
                catalog().find("species", "Name", "Gastly").unwrap()["AppId"]
                    .as_str()
                    .unwrap()
                    .into(),
            );
        }
        let done = chance_damage(&c, &a, &t, "Tackle");
        assert!(done
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .queue
            .is_empty());
        assert!(!p(&done, &t)
            .runtime
            .statuses
            .iter()
            .any(|s| s.kind == "Flinch"));
    }
}
#[test]
fn items_flinch_shared_targets_and_typeless_set_damage_use_normal_effect_resolution() {
    use super::targeting_tests::{attack, damage_command, pass, roll};
    let (mut c, a, t, high) = super::targeting_tests::multi("Swift");
    let sid = p(&c, &a).sheet_id.clone();
    let mut sheet = c.sheets[&sid].clone();
    sheet.inventory.insert("King's Rock".into(), 1);
    c.sheets.insert(
        sid,
        items::equip(&sheet, "King's Rock", "rock", &catalog()).unwrap(),
    );
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .enabled = true;
    items::initialize(&mut c, &catalog()).unwrap();
    let c = roll(
        &c,
        "player",
        &attack(&a, vec![t.clone(), high.clone()], "Swift"),
        4,
    );
    let c = pass(&pass(&c));
    let c = roll(&c, "player", &damage_command(&c), 4);
    let c = roll(&c, "player", &orb_effect(&c), 6);
    let c = decision(
        &c,
        "gm",
        action(ItemCommand::Lose {
            combatant_id: a.clone(),
            unit_id: "rock".into(),
            override_sticky_hold: false,
        }),
    );
    let c = roll(&c, "player", &damage_command(&c), 4);
    let done = roll(&c, "player", &orb_effect(&c), 6);
    for target in [&t, &high] {
        assert!(p(&done, target)
            .runtime
            .statuses
            .iter()
            .any(|s| s.kind == "Flinch"));
    }
    let (mut c, a, t) = chance_setup("King's Rock", "Sonic Boom", "", "");
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some(
        catalog().find("species", "Name", "Gastly").unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
    let c = chance_damage(&c, &a, &t, "Sonic Boom");
    assert_eq!(p(&c, &t).hp, 19);
    let done = roll(&c, "player", &orb_effect(&c), 6);
    assert!(p(&done, &t)
        .runtime
        .statuses
        .iter()
        .any(|s| s.kind == "Flinch"));
}

#[test]
fn items_flinch_suppression_and_clash_do_not_create_added_effects() {
    for (ability, room) in [("Klutz", false), ("", true)] {
        let (mut c, a, t) = chance_setup("Razor Fang", "Tackle", ability, "");
        if room {
            c.battle
                .as_mut()
                .unwrap()
                .flow
                .as_mut()
                .unwrap()
                .battlefield
                .rules
                .push(crate::combat::field::FieldRule {
                    id: "room".into(),
                    kind: "Magic Room".into(),
                    source: a.clone(),
                    scope: "all".into(),
                    rounds: Some(5),
                    parameter: 0,
                    started_round: 1,
                });
        }
        let done = chance_damage(&c, &a, &t, "Tackle");
        assert!(done
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .queue
            .is_empty());
    }
    let (mut c, a, t) = chance_setup("Razor Fang", "Tackle", "", "");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .skills
        .insert("Clash".into(), 5);
    let c = decision(
        &c,
        "player",
        Action::PassTurn {
            combatant_id: a.clone(),
            reason: "Let the foe attack before clashing".into(),
        },
    );
    let c = super::targeting_tests::roll(
        &c,
        "other",
        &super::targeting_tests::attack(&t, vec![a.clone()], "Tackle"),
        4,
    );
    let command = Action::React {
        combatant_id: a.clone(),
        kind: "clash".into(),
        move_name: "Tackle".into(),
        reason: "Clash does not trigger move added effects".into(),
    };
    let done = super::targeting_tests::roll(&c, "player", &command, 4);
    assert!(done
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .is_empty());
    assert!(!p(&done, &t)
        .runtime
        .statuses
        .iter()
        .any(|status| status.kind == "Flinch"));
}

#[test]
fn items_item_sheer_force_does_not_suppress_bide_preparation() {
    let (mut c, a, t) = chance_setup("Razor Fang", "Bide", "Sheer Force", "");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .provenance_rules = true;
    pm(&mut c, &a)
        .runtime
        .ability
        .flags
        .insert("sheer-force:damage".into());
    let c = hit(&c, &a, &t, "Bide");
    assert!(!c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sheer_force);
    let c = super::targeting_tests::pass(&c);
    let sequence = c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sequence;
    let c = decision(
        &c,
        "gm",
        Action::SpecialMove {
            command: crate::combat::special::SpecialCommand::Select {
                sequence,
                recipient_id: String::new(),
                secondary_id: a.clone(),
            },
            reason: "GM retains the existing explicit Bide damage-basis ruling".into(),
        },
    );
    let done = super::targeting_tests::roll(&c, "player", &orb_effect(&c), 1);
    assert!(done
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .special
        .bides
        .contains_key(&a));
}

#[test]
fn items_sheer_force_varied_damage_adds_dice_and_fixed_damage_retains_chance() {
    for name in ["Night Shade", "Sonic Boom"] {
        let mut pools = vec![];
        for ignore in [false, true] {
            let (mut c, a, t) = chance_setup("King's Rock", name, "Sheer Force", "");
            c.battle
                .as_mut()
                .unwrap()
                .flow
                .as_mut()
                .unwrap()
                .provenance_rules = true;
            if ignore {
                pm(&mut c, &a)
                    .runtime
                    .ability
                    .flags
                    .insert("sheer-force:damage".into());
            }
            let sid = p(&c, &t).sheet_id.clone();
            c.sheets.get_mut(&sid).unwrap().species_id = Some(
                catalog().find("species", "Name", "Gastly").unwrap()["AppId"]
                    .as_str()
                    .unwrap()
                    .into(),
            );
            let c = hit(&c, &a, &t, name);
            assert_eq!(
                c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sheer_force,
                ignore && name == "Night Shade"
            );
            let c = super::targeting_tests::pass(&c);
            let action = super::targeting_tests::damage_command(&c);
            pools.push(rules::dice_count(&c, &action, &catalog()).unwrap());
            let done = super::targeting_tests::roll(&c, "player", &action, 4);
            assert_eq!(
                done.battle
                    .as_ref()
                    .unwrap()
                    .flow
                    .as_ref()
                    .unwrap()
                    .queue
                    .is_empty(),
                ignore && name == "Night Shade"
            );
        }
        assert_eq!(
            pools[1],
            pools[0] + if name == "Night Shade" { 2 } else { 0 }
        );
    }
}

fn fetch_setup() -> (Campaign, String, String, String) {
    let (mut c, a, t) = setup("Oran Berry", "Ball Fetch");
    let mut human = trainer("player");
    human.inventory.insert("Pokeball".into(), 1);
    let trainer_id = human.id.clone();
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().trainer_id = Some(trainer_id.clone());
    c.sheets.insert(trainer_id.clone(), human);
    let wild_sheet = p(&c, &t).sheet_id.clone();
    c.sheets.get_mut(&wild_sheet).unwrap().trainer_id = None;
    (c, a, t, trainer_id)
}
fn fetch_command(a: &str, t: &str, trainer: &str, attempt: &str, ball: &str) -> Action {
    action(ItemCommand::FailedCapture {
        attempt_id: attempt.into(),
        trainer_id: trainer.into(),
        fetcher_id: a.into(),
        target_id: t.into(),
        ball: ball.into(),
    })
}
#[test]
fn items_ball_fetch_tracks_stock_return_reuse_and_noncombatant_trainer_rollback() {
    let (c, a, t, trainer) = fetch_setup();
    let command = fetch_command(&a, &t, &trainer, "capture-1", "Pokeball");
    assert!(battle::transition(&c, "player", &command, vec![], &catalog()).is_err());
    let captured = decision(&c, "gm", command.clone());
    assert!(!captured.sheets[&trainer].inventory.contains_key("Pokeball"));
    assert_eq!(items::held(&captured, &a)[0].name, "Oran Berry");
    assert_eq!(p(&captured, &a).actions, p(&c, &a).actions);
    assert!(battle::transition(&captured, "gm", &command, vec![], &catalog()).is_err());
    assert!(battle::transition(
        &captured,
        "gm",
        &fetch_command(&a, &t, &trainer, "capture-2", "Pokeball"),
        vec![],
        &catalog()
    )
    .is_err());
    assert!(battle::transition(&captured, "gm", &Action::EndBattle, vec![], &catalog()).is_err());
    let undone = decision(
        &captured,
        "gm",
        Action::RollbackBattle {
            event_sequence: captured.battle_undo.last().unwrap().event_sequence,
            reason: "Undo the recorded throw and trainer stock together".into(),
        },
    );
    assert_eq!(
        undone.sheets[&trainer].inventory,
        c.sheets[&trainer].inventory
    );
    assert!(undone
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .fetches
        .is_empty());
    let before_end = hold(&hold(&captured, &a), &t);
    let returned = decision(&before_end, "gm", Action::NextRound);
    let unit = returned.sheets[&trainer].stored_items.as_ref().unwrap()[0].clone();
    assert_eq!(unit.origin_sheet_id, trainer);
    assert!(
        returned
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .fetches["capture-1"]
            .returned
    );
    let restored = decision(
        &returned,
        "gm",
        Action::RollbackBattle {
            event_sequence: returned.battle_undo.last().unwrap().event_sequence,
            reason: "Undo round-end return and restore the pending ball".into(),
        },
    );
    assert!(restored.sheets[&trainer].stored_items.is_none());
    assert!(
        !restored
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .fetches["capture-1"]
            .returned
    );
    let mut reuse = returned.clone();
    let retry = fetch_command(&a, &t, &trainer, "capture-2", &unit.id);
    reuse = decision(&reuse, "gm", retry);
    assert!(reuse.sheets[&trainer].stored_items.is_none());
    items::end_round(&mut reuse, &catalog()).unwrap();
    items::end_round(&mut reuse, &catalog()).unwrap();
    assert_eq!(
        reuse.sheets[&trainer].stored_items.as_ref().unwrap(),
        &vec![unit]
    );
}
#[test]
fn items_ball_fetch_validates_ability_wild_target_stock_and_private_views() {
    let (c, a, t, trainer) = fetch_setup();
    let command = fetch_command(&a, &t, &trainer, "capture-1", "Pokeball");
    for condition in [
        "wrong_ability",
        "recalled",
        "fainted",
        "trained",
        "same_side",
        "room",
    ] {
        let mut invalid = c.clone();
        match condition {
            "wrong_ability" => pm(&mut invalid, &a).selected_ability = "Pickup".into(),
            "recalled" => pm(&mut invalid, &a).deployed = false,
            "fainted" => pm(&mut invalid, &a).hp = 0,
            "trained" => {
                let sid = p(&invalid, &t).sheet_id.clone();
                invalid.sheets.get_mut(&sid).unwrap().trainer_id = Some(trainer.clone());
            }
            "same_side" => pm(&mut invalid, &t).side = p(&c, &a).side.clone(),
            _ => invalid
                .battle
                .as_mut()
                .unwrap()
                .flow
                .as_mut()
                .unwrap()
                .battlefield
                .rules
                .push(crate::combat::field::FieldRule {
                    id: "room".into(),
                    kind: "Magic Room".into(),
                    source: a.clone(),
                    scope: "all".into(),
                    rounds: Some(5),
                    parameter: 0,
                    started_round: 1,
                }),
        }
        assert!(
            battle::transition(&invalid, "gm", &command, vec![], &catalog()).is_err(),
            "{condition}"
        );
    }
    let captured = decision(&c, "gm", command);
    let other = crate::privacy::for_player(&captured, "other");
    assert!(other
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .fetches
        .is_empty());
    assert!(other.sheets[&trainer].inventory.is_empty());
    assert!(!crate::privacy::for_player(&captured, "player")
        .battle
        .unwrap()
        .flow
        .unwrap()
        .items
        .fetches
        .is_empty());
    let mut earned = captured;
    pm(&mut earned, &a).selected_ability = "Mummy".into();
    items::end_round(&mut earned, &catalog()).unwrap();
    assert_eq!(
        earned.sheets[&trainer].stored_items.as_ref().unwrap().len(),
        1
    );
    pm(&mut earned, &a).selected_ability = "Ball Fetch".into();
    let stored_id = earned.sheets[&trainer].stored_items.as_ref().unwrap()[0]
        .id
        .clone();
    assert!(battle::transition(
        &earned,
        "gm",
        &fetch_command(&a, &t, &trainer, "late", &stored_id),
        vec![],
        &catalog()
    )
    .is_err());
}

#[test]
fn items_ball_fetch_pending_unit_identity_cannot_be_duplicated() {
    let (c, a, t, trainer) = fetch_setup();
    let captured = decision(
        &c,
        "gm",
        fetch_command(&a, &t, &trainer, "capture-1", "Pokeball"),
    );
    let unit = captured
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .fetches["capture-1"]
        .unit
        .clone();
    let mut forged = captured.clone();
    let sid = p(&forged, &a).sheet_id.clone();
    forged.sheets.get_mut(&sid).unwrap().stored_items = Some(vec![unit]);
    assert!(items::initialize(&mut forged, &catalog()).is_err());
    let mut alias = captured;
    let colliding = format!("fetch:{}:collision", alias.battle.as_ref().unwrap().id);
    alias
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .fetches
        .get_mut("capture-1")
        .unwrap()
        .unit
        .id = colliding;
    alias
        .sheets
        .get_mut(&trainer)
        .unwrap()
        .inventory
        .insert("Pokeball".into(), 1);
    assert!(battle::transition(
        &alias,
        "gm",
        &fetch_command(&a, &t, &trainer, "collision", "Pokeball"),
        vec![],
        &catalog()
    )
    .is_err());
}

#[test]
fn items_effect_review_exposes_manual_boundaries_without_private_preview_names() {
    for (name, scope) in [
        ("Choice Band", "manual"),
        ("Leftovers", "supported"),
        ("Oran Berry", "controlled"),
        ("Berry Juice", "restricted"),
        ("Charizardite X", "transformation"),
        ("Waterium Z", "excluded"),
        ("Potion", "manual"),
        ("Pokeball", "manual"),
    ] {
        let (mut c, a, t) = setup(name, "");
        let review = c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .units["berry"]
            .review
            .as_ref()
            .unwrap();
        assert_eq!(review.scope, scope, "{name}");
        if scope == "transformation" {
            assert!(review.note.contains("Klutz does not suppress"));
        }
        let action = super::targeting_tests::attack(&a, vec![t], "Tackle");
        let preview = crate::combat::preview(&c, &action, &catalog()).unwrap();
        assert!(preview
            .warnings
            .iter()
            .any(|w| w.contains("Only reviewed held-item effects")));
        assert!(
            !preview.warnings.iter().any(|w| w.contains(name)),
            "private item name leaked into preview: {name}"
        );
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .items
            .units
            .get_mut("berry")
            .unwrap()
            .review = None;
        let legacy = serde_json::to_value(
            &c.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .items
                .units["berry"],
        )
        .unwrap();
        assert!(legacy.get("review").is_none());
        let restored: items::BattleItem = serde_json::from_value(legacy).unwrap();
        assert!(restored.review.is_none());
        items::initialize(&mut c, &catalog()).unwrap();
        assert_eq!(
            c.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .items
                .units["berry"]
                .review
                .as_ref()
                .unwrap()
                .scope,
            scope
        );
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .items
            .enabled = false;
        assert!(!crate::combat::preview(&c, &action, &catalog())
            .unwrap()
            .warnings
            .iter()
            .any(|w| w.contains("Only reviewed held-item effects")));
    }
}

#[test]
fn items_release_activates_new_battles_and_preserves_historical_engines() {
    let (c, a, t) = setup("Oran Berry", "");
    let sheets = vec![p(&c, &a).sheet_id.clone(), p(&c, &t).sheet_id.clone()];
    let ended = decision(&c, "gm", Action::EndBattle);
    for (engine, enabled, events) in [
        (ENGINE, true, true),
        (ITEM_COMPLETION_ENGINE, true, true),
        (EVENT_ENGINE, false, true),
        (PROVENANCE_ENGINE, false, false),
    ] {
        let mut old = ended.clone();
        old.engine = engine.into();
        let mut started = decision(
            &old,
            "gm",
            Action::StartBattle {
                sheet_ids: sheets.clone(),
                sides: BTreeMap::new(),
            },
        );
        assert_eq!(items::enabled(&started), enabled);
        let flow = started.battle.as_ref().unwrap().flow.as_ref().unwrap();
        assert_eq!(flow.events.enabled, events);
        assert!(flow.provenance_rules && flow.sequencing.enabled && flow.reviewed_rules);
        let id = started.battle.as_ref().unwrap().combatants[0].id.clone();
        pm(&mut started, &id).hp -= 1;
        if enabled {
            assert_eq!(items::held(&started, &id)[0].id, "berry");
            let done = decision(&started, "player", use_berry(&id));
            assert!(items::held(&done, &id).is_empty());
            assert_eq!(p(&done, &id).hp, p(&started, &id).hp + 1);
            let restored = decision(
                &done,
                "gm",
                Action::RollbackBattle {
                    event_sequence: done.battle_undo.last().unwrap().event_sequence,
                    reason: "Restore released item engine state".into(),
                },
            );
            assert_eq!(items::held(&restored, &id)[0].id, "berry");
        } else {
            assert!(
                battle::transition(&started, "player", &use_berry(&id), vec![], &catalog())
                    .is_err()
            );
            assert!(crate::combat::abilities::coverage(&started, "Cheek Pouch")
                .unwrap()
                .contains("newer encounter"));
        }
        // Changing a campaign label never activates an existing saved encounter.
        let mut value = serde_json::to_value(&started).unwrap();
        value["engine"] = ENGINE.into();
        value["battle"]["flow"]
            .as_object_mut()
            .unwrap()
            .remove("items");
        let loaded: Campaign = serde_json::from_value(value).unwrap();
        assert!(!items::enabled(&loaded));
    }
}
#[test]
fn interactions_new_native_flinch_merges_item_pool_once_and_sheer_force_removes_both() {
    for (source, target, expected) in [
        ("", "", 4),
        ("Serene Grace", "", 6),
        ("Serene Grace", "Wonder Skin", 4),
    ] {
        let (mut c, a, t) = chance_setup("Razor Fang", "Iron Head", source, target);
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .interaction_rules = true;
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .ability_changes
            .enabled = true;
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .tactics
            .enabled = true;
        let attack = super::targeting_tests::attack(&a, vec![t.clone()], "Iron Head");
        let c = super::targeting_tests::roll(&c, "player", &attack, 4);
        let c = super::targeting_tests::pass(&c);
        let c = super::targeting_tests::roll(
            &c,
            "player",
            &super::targeting_tests::damage_command(&c),
            4,
        );
        assert!(c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0]
            .damage_context
            .is_some());
        assert_eq!(
            rules::dice_count(&c, &orb_effect(&c), &catalog()).unwrap(),
            expected
        );
        let mut changed = c.clone();
        pm(&mut changed, &a).selected_ability = if source == "Serene Grace" {
            "Stench"
        } else {
            "Serene Grace"
        }
        .into();
        assert_eq!(
            rules::dice_count(&changed, &orb_effect(&changed), &catalog()).unwrap(),
            expected,
            "Frozen source chance modifiers"
        );
        let done = super::targeting_tests::roll(&c, "player", &orb_effect(&c), 6);
        assert_eq!(
            p(&done, &t)
                .runtime
                .statuses
                .iter()
                .filter(|s| s.kind == "Flinch")
                .count(),
            1
        );
    }
    let (mut c, a, t) = chance_setup("Razor Fang", "Iron Head", "Sheer Force", "");
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.interaction_rules = true;
    f.provenance_rules = true;
    pm(&mut c, &a)
        .runtime
        .ability
        .flags
        .insert("sheer-force:damage".into());
    let c = chance_damage(&c, &a, &t, "Iron Head");
    // Sheer Force completes the attack without an added-effect obligation.
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .is_empty());
    assert!(p(&c, &t).runtime.statuses.is_empty());
}
#[path = "item_completion_tests.rs"]
mod completion_tests;
