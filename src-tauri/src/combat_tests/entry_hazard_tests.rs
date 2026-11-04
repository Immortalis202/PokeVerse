use super::targeting_tests::roll;
use super::*;
use crate::combat::{entry_hazards, field, items, special};

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
fn setup(name: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = special_tests::setup(name);
    for p in &mut c.battle.as_mut().unwrap().combatants {
        p.hp = 20;
        p.max_hp = 20;
    }
    let active = c
        .battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .map(|p| p.id.clone())
        .collect();
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.entry_hazards = true;
    f.reviewed_rules = true;
    f.interaction_rules = true;
    f.decoys.enabled = true;
    f.battlefield.known_active = active;
    (c, a, t)
}
fn q(c: &Campaign) -> crate::combat::QueuedAction {
    c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].clone()
}
fn effect(c: &Campaign) -> Action {
    Action::ResolveEffect {
        sequence: q(c).sequence,
        reason: "Reviewed entry hazard".into(),
    }
}
fn accuracy(a: &str, name: &str) -> Action {
    Action::AccuracyTargets {
        combatant_id: a.into(),
        target_ids: vec![],
        move_name: name.into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Install field rule".into(),
    }
}
fn reset(c: &mut Campaign, a: &str) {
    pm(c, a).used_moves.clear();
    pm(c, a).actions = 0;
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.queue.clear();
    f.window = false;
    f.started = true;
    f.order = vec![a.into()];
    f.cursor = 0;
    f.turn += 1;
}
fn cast(c: &Campaign, a: &str, name: &str) -> Campaign {
    let mut c = c.clone();
    reset(&mut c, a);
    let c = roll(&c, "gm", &accuracy(a, name), 6);
    let n = rules::dice_count(&c, &effect(&c), &catalog()).unwrap();
    assert_eq!(n, 0);
    battle::transition(&c, "gm", &effect(&c), vec![], &catalog())
        .unwrap()
        .0
}
fn depart(c: &mut Campaign, id: &str) {
    let before = c.clone();
    pm(c, id).deployed = false;
    field::lifecycle(&before, c, &catalog()).unwrap();
}
fn enter(c: &mut Campaign, id: &str) {
    let before = c.clone();
    pm(c, id).deployed = true;
    field::lifecycle(&before, c, &catalog()).unwrap();
    // A second lifecycle in the same command must not replay the entry.
    field::lifecycle(&before, c, &catalog()).unwrap();
}
fn dexterity(c: &Campaign, id: &str) -> i32 {
    special::modifier(c, p(c, id), "Dexterity").unwrap_or(0)
}
fn equip(c: &mut Campaign, id: &str, item: &str) {
    let sid = p(c, id).sheet_id.clone();
    let mut s = c.sheets[&sid].clone();
    s.inventory.insert(item.into(), 1);
    c.sheets.insert(
        sid,
        items::equip(&s, item, "hazard-item", &catalog()).unwrap(),
    );
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.items.enabled = true;
    f.items.completion.enabled = true;
    items::initialize(c, &catalog()).unwrap();
    items::completion::initialize(c, &catalog());
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

#[test]
fn entry_hazards_all_three_cast_scope_permissions_fixed_entry_and_nonstacking() {
    for name in entry_hazards::NAMES {
        let (c, a, t) = setup(name);
        let c0 = roll(&c, "gm", &accuracy(&a, name), 6);
        assert!(battle::transition(&c0, "intruder", &effect(&c0), vec![], &catalog()).is_err());
        assert!(battle::transition(&c0, "gm", &effect(&c0), vec![6], &catalog()).is_err());
        let c = battle::transition(&c0, "gm", &effect(&c0), vec![], &catalog())
            .unwrap()
            .0;
        assert!(field::has(&c, name, p(&c, &t)));
        assert!(!field::has(&c, name, p(&c, &a)));
        assert_eq!(
            p(&c, &t).hp,
            20,
            "placement must not hit existing occupants"
        );
        assert_eq!(dexterity(&c, &t), 0);
        let mut c = cast(&c, &a, name);
        let rules = &c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .battlefield
            .rules;
        assert_eq!(rules.iter().filter(|r| r.kind == name).count(), 1);
        assert_eq!(rules.iter().find(|r| r.kind == name).unwrap().rounds, None);
        // Overlapping side scopes must still produce just one entry effect.
        let mut overlap = rules.iter().find(|r| r.kind == name).unwrap().clone();
        overlap.id = id();
        overlap.scope = format!("side:{}", p(&c, &t).side);
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .battlefield
            .rules
            .push(overlap);
        let sheets = serde_json::to_value(&c.sheets).unwrap();
        depart(&mut c, &t);
        enter(&mut c, &t);
        assert_eq!(
            p(&c, &t).hp,
            if name == "Sticky Web" { 20 } else { 19 },
            "{name}"
        );
        assert_eq!(
            dexterity(&c, &t),
            if name == "Sticky Web" { -1 } else { 0 },
            "{name}"
        );
        depart(&mut c, &t);
        enter(&mut c, &t);
        assert_eq!(p(&c, &t).hp, if name == "Sticky Web" { 20 } else { 18 });
        assert_eq!(
            dexterity(&c, &t),
            if name == "Sticky Web" { -1 } else { 0 },
            "move debuffs do not stack"
        );
        assert_eq!(serde_json::to_value(&c.sheets).unwrap(), sheets);
    }
}

#[test]
fn entry_hazards_boots_ground_immunities_and_gravity_use_live_entry_state() {
    for name in entry_hazards::NAMES {
        let (mut c, a, t) = setup(name);
        equip(&mut c, &t, "Heavy-Duty Boots");
        let mut c = cast(&c, &a, name);
        depart(&mut c, &t);
        enter(&mut c, &t);
        assert_eq!(p(&c, &t).hp, 20);
        assert_eq!(dexterity(&c, &t), 0);
    }
    for name in ["Spikes", "Sticky Web"] {
        for immunity in ["Flying", "Levitate", "Earth Eater", "Air Balloon"] {
            let (mut c, a, t) = setup(name);
            match immunity {
                "Flying" => species(&mut c, &t, "Pidgey"),
                "Air Balloon" => equip(&mut c, &t, "Air Balloon"),
                _ => pm(&mut c, &t).selected_ability = immunity.into(),
            }
            let mut c = cast(&c, &a, name);
            depart(&mut c, &t);
            enter(&mut c, &t);
            assert_eq!(p(&c, &t).hp, 20, "{name}: {immunity}");
            assert_eq!(dexterity(&c, &t), 0, "{name}: {immunity}");
            if immunity != "Air Balloon" {
                let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
                let mut gravity = f.battlefield.rules[0].clone();
                gravity.id = id();
                gravity.kind = "Gravity".into();
                gravity.scope = "all".into();
                gravity.rounds = Some(4);
                f.battlefield.rules.push(gravity);
                depart(&mut c, &t);
                enter(&mut c, &t);
                assert_eq!(p(&c, &t).hp, if name == "Spikes" { 19 } else { 20 });
                assert_eq!(dexterity(&c, &t), if name == "Sticky Web" { -1 } else { 0 });
            }
        }
    }
}

#[test]
fn entry_hazards_rock_uses_fixed_typed_damage_and_indirect_protection() {
    for (species_name, expected) in [("Charizard", 17), ("Eevee", 19), ("Lucario", 20)] {
        let (mut c, a, t) = setup("Stealth Rock");
        species(&mut c, &t, species_name);
        let mut c = cast(&c, &a, "Stealth Rock");
        depart(&mut c, &t);
        enter(&mut c, &t);
        assert_eq!(p(&c, &t).hp, expected, "{species_name}");
    }
    for name in ["Spikes", "Stealth Rock"] {
        for ability in ["Magic Guard", "Wonder Guard"] {
            let (mut c, a, t) = setup(name);
            pm(&mut c, &t).selected_ability = ability.into();
            let mut c = cast(&c, &a, name);
            depart(&mut c, &t);
            enter(&mut c, &t);
            assert_eq!(p(&c, &t).hp, 20, "{name}: {ability}");
        }
    }
    let (c, a, t) = setup("Spikes");
    let mut c = cast(&c, &a, "Spikes");
    pm(&mut c, &t).hp = 1;
    depart(&mut c, &t);
    enter(&mut c, &t);
    assert_eq!(p(&c, &t).hp, 0);
    let before = c.clone();
    field::lifecycle(&before, &mut c, &catalog()).unwrap();
    assert_eq!(p(&c, &t).hp, 0);
    // Recovery in an occupied slot is not another deployment/entry.
    let before = c.clone();
    pm(&mut c, &t).hp = 1;
    field::lifecycle(&before, &mut c, &catalog()).unwrap();
    assert_eq!(p(&c, &t).hp, 1);
}

#[test]
fn entry_hazards_web_stat_protections_reflection_and_caster_departure() {
    for (ability, expected) in [
        ("Clear Body", 0),
        ("Full Metal Body", 0),
        ("White Smoke", 0),
        ("Contrary", 1),
        ("Simple", -2),
    ] {
        let (mut c, a, t) = setup("Sticky Web");
        pm(&mut c, &t).selected_ability = ability.into();
        let mut c = cast(&c, &a, "Sticky Web");
        depart(&mut c, &a);
        depart(&mut c, &t);
        enter(&mut c, &t);
        assert_eq!(
            dexterity(&c, &t),
            expected,
            "{ability}: caster need not remain active"
        );
    }
    let (mut c, a, t) = setup("Sticky Web");
    equip(&mut c, &t, "Clear Amulet");
    let mut c = cast(&c, &a, "Sticky Web");
    depart(&mut c, &t);
    enter(&mut c, &t);
    assert_eq!(dexterity(&c, &t), 0);
    let (mut c, a, t) = setup("Sticky Web");
    pm(&mut c, &t).selected_ability = "Mirror Armor".into();
    let mut c = cast(&c, &a, "Sticky Web");
    depart(&mut c, &t);
    enter(&mut c, &t);
    assert_eq!(dexterity(&c, &t), 0);
    assert_eq!(dexterity(&c, &a), -1);
    let (mut c, a, t) = setup("Sticky Web");
    pm(&mut c, &a).selected_ability = "Teravolt".into();
    pm(&mut c, &t).selected_ability = "Clear Body".into();
    let mut c = cast(&c, &a, "Sticky Web");
    depart(&mut c, &t);
    enter(&mut c, &t);
    assert_eq!(
        dexterity(&c, &t),
        0,
        "a field hazard does not inherit a live attack's bypass"
    );
    for (ability, key) in [("Defiant", "Strength"), ("Competitive", "Special")] {
        let (mut c, a, t) = setup("Sticky Web");
        pm(&mut c, &t).selected_ability = ability.into();
        let mut c = cast(&c, &a, "Sticky Web");
        depart(&mut c, &a);
        depart(&mut c, &t);
        enter(&mut c, &t);
        let s = rules::effective_sheet(&c, p(&c, &t)).unwrap();
        assert_eq!(
            rules::score(&s, key).unwrap(),
            6,
            "{ability}: hostile field stat change"
        );
    }
    let (mut c, a, t) = setup("Sticky Web");
    let u = reserve(&mut c, &t);
    pm(&mut c, &u).selected_ability = "Mirror Armor".into();
    let mut c = cast(&c, &a, "Sticky Web");
    depart(&mut c, &a);
    let before = serde_json::to_value(&c).unwrap();
    let err = battle::transition(&c, "gm", &switch(&t, &u), vec![], &catalog()).unwrap_err();
    assert_eq!(err.code, "manual_rule");
    assert!(err.message.contains("Mirror Armor"));
    assert_eq!(
        serde_json::to_value(&c).unwrap(),
        before,
        "absent-source reflection rejects atomically"
    );
}

#[test]
fn entry_hazards_cleanup_defog_tidy_up_and_mortal_spin_preserve_stat_history() {
    for cleanup in ["Defog", "Tidy Up", "Mortal Spin"] {
        let (mut c, a, t) = setup("Spikes");
        let sid = p(&c, &a).sheet_id.clone();
        c.sheets.get_mut(&sid).unwrap().moves.extend(
            entry_hazards::NAMES
                .into_iter()
                .chain([cleanup])
                .map(str::to_string),
        );
        for name in entry_hazards::NAMES {
            c = cast(&c, &a, name);
        }
        if cleanup != "Mortal Spin" {
            depart(&mut c, &t);
            enter(&mut c, &t);
            assert_eq!(dexterity(&c, &t), -1);
        }
        if cleanup == "Mortal Spin" {
            let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
            f.compound.enabled = true;
            let side = p(&c, &a).side.clone();
            for r in &mut c
                .battle
                .as_mut()
                .unwrap()
                .flow
                .as_mut()
                .unwrap()
                .battlefield
                .rules
            {
                r.scope = format!("side:{side}");
            }
            reset(&mut c, &a);
            let mut c0 = hit(&c, &a, &t, cleanup);
            if c0.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
                c0 = super::targeting_tests::pass(&c0);
            }
            c0 = roll(&c0, "gm", &super::targeting_tests::damage_command(&c0), 1);
            c = roll(&c0, "gm", &effect(&c0), 6);
        } else {
            c = cast(&c, &a, cleanup);
        }
        assert!(
            !c.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .battlefield
                .rules
                .iter()
                .any(|r| entry_hazards::NAMES.contains(&r.kind.as_str())),
            "{cleanup}"
        );
        if cleanup != "Mortal Spin" {
            assert_eq!(
                dexterity(&c, &t),
                -1,
                "cleanup does not undo past stat changes"
            );
        }
    }
}

#[test]
fn entry_hazards_field_reflection_preserves_scope_without_another_action() {
    for name in entry_hazards::NAMES {
        let (mut c, a, t) = setup(name);
        pm(&mut c, &t).selected_ability = "Magic Bounce".into();
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .tactics
            .enabled = true;
        let mut c = cast(&c, &a, name);
        assert!(field::has(&c, name, p(&c, &a)));
        assert!(!field::has(&c, name, p(&c, &t)));
        assert_eq!(p(&c, &a).actions, 1);
        assert_eq!(p(&c, &t).actions, 0);
        depart(&mut c, &a);
        enter(&mut c, &a);
        assert_eq!(p(&c, &a).hp, if name == "Sticky Web" { 20 } else { 19 });
        assert_eq!(dexterity(&c, &a), if name == "Sticky Web" { -1 } else { 0 });
    }
}

fn reserve(c: &mut Campaign, t: &str) -> String {
    let mut newcomer = p(c, t).clone();
    let new_id = id();
    let sid = id();
    let mut sheet = c.sheets[&newcomer.sheet_id].clone();
    sheet.id = sid.clone();
    sheet.trainer_id = Some("hazard-trainer".into());
    c.sheets.get_mut(&newcomer.sheet_id).unwrap().trainer_id = sheet.trainer_id.clone();
    c.sheets.insert(sid.clone(), sheet);
    newcomer.id = new_id.clone();
    newcomer.sheet_id = sid;
    newcomer.deployed = false;
    newcomer.field_slot = None;
    let b = c.battle.as_mut().unwrap();
    b.slots_per_side = 1;
    b.combatants.push(newcomer);
    new_id
}
fn switch(out: &str, into: &str) -> Action {
    Action::SwitchPokemon {
        outgoing_id: out.into(),
        incoming_id: into.into(),
        gm_override: false,
        reason: "Enter hazard side".into(),
    }
}
#[test]
fn entry_hazards_real_switch_serialization_rollback_and_permissions() {
    for name in entry_hazards::NAMES {
        let (mut c, a, t) = setup(name);
        let u = reserve(&mut c, &t);
        let c = cast(&c, &a, name);
        let c: Campaign = serde_json::from_value(serde_json::to_value(&c).unwrap()).unwrap();
        let before = serde_json::to_value(&c.battle).unwrap();
        let action = switch(&t, &u);
        assert!(battle::transition(&c, "intruder", &action, vec![], &catalog()).is_err());
        let done = battle::transition(&c, "gm", &action, vec![], &catalog())
            .unwrap()
            .0;
        assert_eq!(p(&done, &u).hp, if name == "Sticky Web" { 20 } else { 19 });
        assert_eq!(
            dexterity(&done, &u),
            if name == "Sticky Web" { -1 } else { 0 }
        );
        let restored = decision(
            &done,
            "gm",
            Action::RollbackBattle {
                event_sequence: done.battle_undo.last().unwrap().event_sequence,
                reason: "Undo hazard switch".into(),
            },
        );
        assert_eq!(serde_json::to_value(&restored.battle).unwrap(), before);
    }
}
#[test]
fn entry_hazards_missing_saved_flag_and_historical_routes_stay_disabled() {
    let (mut c, a, _) = setup("Spikes");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .entry_hazards = false;
    let c = roll(&c, "gm", &accuracy(&a, "Spikes"), 6);
    assert_eq!(
        battle::transition(&c, "gm", &effect(&c), vec![], &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
    let mut data = serde_json::to_value(&c).unwrap();
    data["battle"]["flow"]
        .as_object_mut()
        .unwrap()
        .remove("entry_hazards");
    let old: Campaign = serde_json::from_value(data).unwrap();
    assert!(!entry_hazards::enabled(&old));
}
