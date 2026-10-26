use super::*;
use crate::combat::{house, standalone, tactics};
fn setup_house(item: &str, ability: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = setup6(item, ability);
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.house_rules = true;
    f.standalone.enabled = true;
    (c, a, t)
}
fn force_turn(c: &mut Campaign, id: &str) {
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    if !f.order.iter().any(|v| v == id) {
        f.order.push(id.into());
    }
    f.cursor = f.order.iter().position(|v| v == id).unwrap();
    f.queue.clear();
    f.window = false;
    f.turn += 1;
    f.reacted.clear();
}
fn attempt(c: &Campaign, a: &str, t: &str, name: &str, n: u8) -> Campaign {
    let mut c = c.clone();
    force_turn(&mut c, a);
    learn(&mut c, a, name);
    roll(
        &c,
        "gm",
        &Action::Accuracy {
            combatant_id: a.into(),
            target_id: Some(t.into()),
            move_name: name.into(),
            pool_modifier: 0,
            success_modifier: 0,
            reason: "Group rule fixture".into(),
        },
        n,
    )
}
fn effect(c: &Campaign) -> Campaign {
    let c = if c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
        pass(c)
    } else {
        c.clone()
    };
    let seq = c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sequence;
    battle::transition(
        &c,
        "gm",
        &Action::ResolveEffect {
            sequence: seq,
            reason: "Resolve supported group effect".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap()
    .0
}

#[test]
fn house_torrent_matches_starter_boosts_without_a_ruling_and_freezes_zero() {
    let (mut c, a, _) = setup_house("Leftovers", "Torrent");
    let m = catalog().find("moves", "Name", "Water Gun").unwrap();
    let max = p(&c, &a).max_hp;
    assert_eq!(
        standalone::torrent(&c, p(&c, &a), &m, max / 2, None).unwrap(),
        2
    );
    assert_eq!(
        standalone::torrent(&c, p(&c, &a), &m, max / 2 + 1, None).unwrap(),
        0
    );
    assert_eq!(
        standalone::torrent(&c, p(&c, &a), &m, 1, Some(0)).unwrap(),
        0
    );
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .house_rules = false;
    assert!(standalone::torrent(&c, p(&c, &a), &m, 1, None).is_err());
}

#[test]
fn house_rivalry_uses_explicit_gender_and_rejects_the_other_policy() {
    let (mut c, a, t) = setup_house("Leftovers", "Rivalry");
    for (id, gender) in [(&a, "male"), (&t, "male")] {
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .standalone
            .profiles
            .insert(
                id.clone(),
                standalone::BattleProfile {
                    species_id: None,
                    gender: gender.into(),
                    preference: "female".into(),
                    weight_kg: None,
                    metal_weight_kg: None,
                },
            );
    }
    assert_eq!(standalone::rivalry(&c, p(&c, &a), p(&c, &t)).unwrap(), 1);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .standalone
        .profiles
        .get_mut(&t)
        .unwrap()
        .gender = "female".into();
    assert_eq!(standalone::rivalry(&c, p(&c, &a), p(&c, &t)).unwrap(), -1);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .standalone
        .profiles
        .get_mut(&t)
        .unwrap()
        .gender = "unknown".into();
    assert_eq!(standalone::rivalry(&c, p(&c, &a), p(&c, &t)).unwrap(), 0);
    assert!(battle::transition(
        &c,
        "gm",
        &Action::Standalone {
            command: standalone::StandaloneCommand::RivalryBasis {
                basis: "preference".into()
            },
            reason: "Cannot override fixed group rule".into()
        },
        vec![],
        &catalog()
    )
    .is_err());
}

#[test]
fn house_full_revives_and_recycle_ignore_round_cap_without_minting_stock() {
    for name in ["Max Revive", "Revival Herb"] {
        let (mut c, a, _) = setup_house("Leftovers", "Gluttony");
        stock(&mut c, &a, name);
        pm(&mut c, &a).hp = 0;
        pm(&mut c, &a).runtime.healed = 3;
        // A conscious allied payer restores the fainted recipient.
        let mut payer = p(&c, &a).clone();
        payer.id = "payer".into();
        payer.hp = 20;
        payer.runtime = Default::default();
        c.battle.as_mut().unwrap().combatants.push(payer.clone());
        force_turn(&mut c, &payer.id);
        let cmd = use_med(&c, &payer.id, &a, name, 1);
        let next = run(&c, "gm", cmd);
        assert_eq!(p(&next, &a).hp, p(&next, &a).max_hp, "{name}");
        let mut recycled = c.clone();
        let item = catalog().find("items", "Name", name).unwrap();
        assert!(ci::recycle_medicine(&mut recycled, &a, &item).unwrap());
        assert_eq!(p(&recycled, &a).hp, p(&recycled, &a).max_hp);
    }
}

#[test]
fn house_ring_target_removes_type_and_unique_ability_immunities_but_keeps_items() {
    for (ability, ty) in [
        ("Levitate", "Ground"),
        ("Flash Fire", "Fire"),
        ("Water Absorb", "Water"),
        ("Soundproof", "Normal"),
        ("Lightning Rod", "Electric"),
        ("Wonder Guard", "Normal"),
    ] {
        let (mut c, a, t) = setup_house("Ring Target", ability);
        let m = serde_json::json!({"Type":ty,"Category":"Special","Attributes":{"SoundMove":true}});
        assert!(
            !abilities::damage_immune_from(&c, p(&c, &t), p(&c, &a), &m),
            "{ability}"
        );
        assert!(
            !extra::source_immune(&c, p(&c, &t), p(&c, &a), 0),
            "{ability}"
        );
        pm(&mut c, &t).selected_ability = "Teravolt".into();
        assert!(crate::combat::reviewed::unique_damage_guard(&c, p(&c, &t), p(&c, &a), 0).is_ok());
    }
    let (mut c, a, t) = setup_house("Ring Target", "Comatose");
    field::change_type(&mut c, &a, "replace", &["Fire".into()]).unwrap();
    abilities::inflict(&mut c, &t, &a, "Burn", "", None, &catalog()).unwrap();
    assert!(p(&c, &a).runtime.statuses.iter().any(|s| s.kind == "Burn"));
    let (c, a, t) = setup_house("Air Balloon", "");
    let m = serde_json::json!({"Type":"Ground","Category":"Physical"});
    assert!(abilities::damage_immune_from(&c, p(&c, &t), p(&c, &a), &m));
}

#[test]
fn house_mycelium_supports_are_never_miss_late_and_scarf_overrides_late() {
    let (mut c, a, _) = setup_house("Choice Scarf", "Prankster");
    learn(&mut c, &a, "Roar");
    let c = select(&c, &a, "Roar");
    let m = catalog().find("moves", "Name", "Roar").unwrap();
    assert!(tactics::priority_guard(&c, p(&c, &a), &m).is_ok());
    assert_eq!(
        field::move_data(&c, p(&c, &a), &m)["Attributes"]["Reaction"],
        "5"
    );

    let (mut c, a, _) = setup_house("Choice Scarf", "Mycelium Might");
    learn(&mut c, &a, "Magic Coat");
    let c = select(&c, &a, "Magic Coat");
    let m = catalog().find("moves", "Name", "Magic Coat").unwrap();
    assert!(tactics::priority_guard(&c, p(&c, &a), &m).is_ok());
    let resolved = field::move_data(&c, p(&c, &a), &m);
    assert_eq!(resolved["Attributes"]["NeverMiss"], true);
    assert_eq!(resolved["Attributes"]["Reaction"], "5");
    assert!(resolved["Attributes"]["LateReaction"].is_null());
    let (c, a, _) = setup_house("Leftovers", "Mycelium Might");
    let resolved = field::move_data(&c, p(&c, &a), &m);
    assert_eq!(resolved["Attributes"]["NeverMiss"], true);
    assert_eq!(resolved["Attributes"]["LateReaction"], "1");
    assert!(resolved["Attributes"]["Reaction"].is_null());
}

#[test]
fn house_magic_coat_survives_foe_actions_and_expires_on_own_action_with_rollback() {
    let (c, a, t) = setup_house("Leftovers", "");
    let mut c = effect(&attempt(&c, &a, &a, "Magic Coat", 6));
    // Retained completed queue entries must not keep renewing an old Coat.
    let mut next = c.clone();
    next.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == a)
        .unwrap()
        .actions += 1;
    house::lifecycle(&c, &mut next);
    assert!(!next
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .tactics
        .lures
        .iter()
        .any(|l| l.name == "Magic Coat"));

    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .tactics
        .lures
        .iter()
        .any(|l| l.name == "Magic Coat"));
    c = effect(&attempt(&c, &t, &a, "Growl", 6));
    assert_eq!(p(&c, &t).runtime.modifiers["Strength"], -1);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .tactics
        .lures
        .iter()
        .any(|l| l.name == "Magic Coat"));
    let c = attempt(&c, &a, &t, "Tackle", 6);
    assert!(!c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .tactics
        .lures
        .iter()
        .any(|l| l.name == "Magic Coat"));
    let undo = Action::RollbackBattle {
        event_sequence: c.battle_undo.last().unwrap().event_sequence,
        reason: "Restore coat before next action".into(),
    };
    let restored = battle::transition(&c, "gm", &undo, vec![], &catalog())
        .unwrap()
        .0;
    assert!(restored
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .tactics
        .lures
        .iter()
        .any(|l| l.name == "Magic Coat"));
}

#[test]
fn house_bide_uses_users_received_damage_and_last_attacker_including_a_miss() {
    let (c, a, t) = setup_house("Leftovers", "");
    let mut c = effect(&attempt(&c, &a, &t, "Bide", 6));
    let mut other = p(&c, &t).clone();
    other.id = "second-attacker".into();
    other.field_slot = Some(1);
    c.battle.as_mut().unwrap().slots_per_side = 3;
    c.battle.as_mut().unwrap().combatants.push(other.clone());
    let initial = p(&c, &a).hp;
    for source in [&t, &other.id] {
        let next = pass(&attempt(&c, source, &a, "Tackle", 6));
        c = roll(&next, "gm", &damage_command(&next), 1);
    }
    let damage = initial - p(&c, &a).hp;
    let b = &c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .special
        .bides[&a];
    assert_eq!(b.target_id, other.id);
    assert_eq!(b.basis_id, a);
    assert_eq!(b.damage, damage);
    c = attempt(&c, &t, &a, "Swift", 1);
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .special
            .bides[&a]
            .target_id,
        t
    );
    let c = attempt(&c, &a, &other.id, "Bide", 6);
    assert_eq!(
        c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].target_id,
        t
    );
}

#[test]
fn house_missing_flag_stays_historical_in_serialization() {
    let (c, _, _) = setup_house("Leftovers", "");
    let mut json = serde_json::to_value(&c).unwrap();
    json["battle"]["flow"]
        .as_object_mut()
        .unwrap()
        .remove("house_rules");
    let old: Campaign = serde_json::from_value(json).unwrap();
    assert!(!house::enabled(&old));
}

#[tokio::test]
async fn house_room_rules_persist_through_owner_receipts_reconnect_and_rollback() {
    super::room_fixture(true).await;
}

#[test]
fn house_ring_preserves_contact_retaliation_allows_drops_and_coat_survives_round_change() {
    let (c, a, t) = setup_house("Ring Target", "Rough Skin");
    let before = p(&c, &t).hp;
    let next = foe_attack(c, &a, &t, "Tackle");
    let event = next
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events
        .iter()
        .find(|e| e.kind == "retaliate")
        .unwrap();
    let retaliation = Action::ResolveAbility {
        event_id: event.id.clone(),
        choice: "neutral".into(),
        reason: "Ring Target does not erase Rough Skin".into(),
    };
    let next = battle::transition(&next, "gm", &retaliation, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(p(&next, &t).hp, before - 1);
    let (c, a, t) = setup_house("Ring Target", "Good As Gold");
    let c = effect(&attempt(&c, &t, &a, "Growl", 6));
    assert_eq!(p(&c, &a).runtime.modifiers["Strength"], -1);
    let (c, a, t) = setup_house("Ring Target", "Clear Body");
    let c = effect(&attempt(&c, &t, &a, "Growl", 6));
    assert_eq!(p(&c, &a).runtime.modifiers["Strength"], -1);
    let (mut c, a, _) = setup_house("Ring Target", "Disguise");
    assert_eq!(extra::disguise(&mut c, &a, 3).unwrap(), 3);
    let (mut c, a, _) = setup_house("Ring Target", "");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .tactics
        .lures
        .push(tactics::Lure {
            source: a.clone(),
            target: a.clone(),
            name: "Magic Coat".into(),
            round: 0,
        });
    let before = c.clone();
    c.battle.as_mut().unwrap().round += 1;
    pm(&mut c, &a).actions = 0;
    house::lifecycle(&before, &mut c);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .tactics
        .lures
        .iter()
        .any(|l| l.name == "Magic Coat"));
}

#[test]
fn house_ring_removes_status_tick_and_recoil_immunity() {
    for (item, damage) in [("Ring Target", 2), ("Air Balloon", 0)] {
        let (mut c, a, t) = setup_house(item, "Magic Guard");
        abilities::inflict(&mut c, &t, &a, "Poison", "", None, &catalog()).unwrap();
        let hp = p(&c, &a).hp;
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .order
            .clear();
        let next = battle::transition(&c, "gm", &Action::NextRound, vec![], &catalog())
            .unwrap()
            .0;
        assert_eq!(p(&next, &a).hp, hp - damage);
    }
    let (c, a, t) = setup_house("Ring Target", "Rock Head");
    let mut c = attempt(&c, &a, &t, "Flare Blitz", 6);
    let mut q = c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].clone();
    q.raw_damage_successes = 3;
    q.damage_done = true;
    let with_ring = special::effect_dice(&c, &q, &catalog()).unwrap().unwrap();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .house_rules = false;
    let historical = special::effect_dice(&c, &q, &catalog()).unwrap().unwrap();
    assert_eq!(with_ring, historical + 3);
}

fn mirror_response(id: &str) -> Action {
    Action::React {
        combatant_id: id.into(),
        kind: "move".into(),
        move_name: "Mirror Coat".into(),
        reason: "Answer the incoming Special move".into(),
    }
}
#[test]
fn house_mirror_coat_is_a_late_response_using_the_exact_incoming_pool() {
    let (mut c, a, t) = setup_house("Leftovers", "");
    learn(&mut c, &t, "Mirror Coat");
    for id in [&a, &t] {
        let p = pm(&mut c, id);
        p.hp = 100;
        p.max_hp = 100;
    }
    c.revision = 10;
    let mut c = attempt(&c, &a, &t, "Swift", 6);
    let incoming_sequence = c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sequence;
    c.revision += 1;
    let response = mirror_response(&t);
    let c = roll(&c, "gm", &response, 6);
    let f = c.battle.as_ref().unwrap().flow.as_ref().unwrap();
    assert_eq!(f.queue[0].move_name, "Swift");
    let q = f
        .queue
        .iter()
        .find(|q| q.move_name == "Mirror Coat")
        .unwrap();
    assert_eq!(
        (&*q.timing, q.priority, q.response_to),
        ("after", 5, Some(incoming_sequence))
    );
    assert_eq!(p(&c, &t).actions, 1);
    assert!(f.reacted.contains(&t));
    let c: Campaign = serde_json::from_value(serde_json::to_value(&c).unwrap()).unwrap();
    let n = rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap();
    let c = roll(&c, "gm", &damage_command(&c), 1);
    let c = if c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].move_name == "Swift" {
        effect(&c)
    } else {
        c
    };
    assert_eq!(
        c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].move_name,
        "Mirror Coat"
    );
    let c = if c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
        pass(&c)
    } else {
        c
    };
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    let m = catalog().find("moves", "Name", "Mirror Coat").unwrap();
    let pool = special::formula(&c, p(&c, &t), p(&c, &a), &m, q.sequence, 0, &catalog())
        .unwrap()
        .unwrap();
    assert_eq!(
        rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap(),
        n + 2
    );
    assert_eq!(pool.dice, n + 2);
    assert_eq!(pool.defense, 0);
    let mut altered = c.clone();
    altered
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .special
        .history
        .last_mut()
        .unwrap()
        .dice += 3;
    assert_eq!(
        rules::dice_count(&altered, &damage_command(&altered), &catalog()).unwrap(),
        n + 5
    );
    let mut unrelated = c.clone();
    unrelated
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .special
        .history
        .last_mut()
        .unwrap()
        .sequence += 100;
    assert!(rules::dice_count(&unrelated, &damage_command(&unrelated), &catalog()).is_err());
    let c = roll(&c, "gm", &damage_command(&c), 4);
    assert!(p(&c, &a).hp < 100);
}
#[test]
fn house_mirror_coat_rejects_ordinary_physical_ally_and_spent_reaction_uses() {
    let (mut c, a, t) = setup_house("Leftovers", "");
    learn(&mut c, &t, "Mirror Coat");
    force_turn(&mut c, &t);
    let ordinary = Action::Accuracy {
        combatant_id: t.clone(),
        target_id: Some(a.clone()),
        move_name: "Mirror Coat".into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Not a stored shield".into(),
    };
    assert!(rules::dice_count(&c, &ordinary, &catalog()).is_err());
    let c = attempt(&c, &a, &t, "Tackle", 6);
    assert!(rules::dice_count(&c, &mirror_response(&t), &catalog()).is_err());
    let c = attempt(&c, &a, &t, "Swift", 6);
    let mut spent = c.clone();
    spent
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .reacted
        .insert(t.clone());
    assert!(rules::dice_count(&spent, &mirror_response(&t), &catalog()).is_err());
    let mut other = c.clone();
    other.battle.as_mut().unwrap().flow.as_mut().unwrap().queue[0].target_id = a.clone();
    assert!(rules::dice_count(&other, &mirror_response(&t), &catalog()).is_err());
    let wrong = Action::Tactics {
        command: tactics::TacticsCommand::Reaction {
            combatant_id: t.clone(),
            move_name: "Mirror Coat".into(),
            target_ids: vec![t.clone()],
        },
        reason: "Cannot redirect the counter".into(),
    };
    assert!(rules::dice_count(&c, &wrong, &catalog()).is_err());
    let mut late = c.clone();
    let q = &mut late.battle.as_mut().unwrap().flow.as_mut().unwrap().queue[0];
    q.timing = "after".into();
    q.priority = 5;
    assert!(rules::dice_count(&late, &mirror_response(&t), &catalog()).is_err());
}
#[test]
fn house_mirror_coat_failed_accuracy_spends_the_slot_and_rollback_restores_it() {
    let (mut c, a, t) = setup_house("Leftovers", "");
    learn(&mut c, &t, "Mirror Coat");
    let c = attempt(&c, &a, &t, "Swift", 6);
    let before = serde_json::to_value(&c).unwrap();
    let failed = roll(&c, "gm", &mirror_response(&t), 1);
    assert!(failed
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .reacted
        .contains(&t));
    assert_eq!(p(&failed, &t).actions, 1);
    assert!(!failed
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .iter()
        .any(|q| q.move_name == "Mirror Coat"));
    let undo = Action::RollbackBattle {
        event_sequence: failed.battle_undo.last().unwrap().event_sequence,
        reason: "Undo failed Mirror Coat".into(),
    };
    let restored = battle::transition(&failed, "gm", &undo, vec![], &catalog())
        .unwrap()
        .0;
    assert!(!restored
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .reacted
        .contains(&t));
    assert_eq!(p(&restored, &t).actions, 0);
    assert_eq!(serde_json::to_value(&c).unwrap(), before);
    assert!(rules::dice_count(&restored, &mirror_response(&t), &catalog()).is_ok());
}
