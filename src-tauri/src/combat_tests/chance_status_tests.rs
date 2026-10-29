use super::targeting_tests::{attack, damage_command, pass, roll};
use super::*;
use crate::combat::{decoys, field, items, provenance, special};

fn cards() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../rules/chance-status-moves.json")).unwrap()
}
fn setup(name: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = special_tests::setup(name);
    let b = c.battle.as_mut().unwrap();
    let f = b.flow.as_mut().unwrap();
    f.chance_status = true;
    f.status_escalation = true;
    f.interaction_rules = true;
    f.reviewed_rules = true;
    for p in &mut b.combatants {
        p.hp = 100;
        p.max_hp = 100;
    }
    let sid = b.combatants[1].sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some(
        catalog().find("species", "Name", "Shuckle").unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
    (c, a, t)
}
fn q(c: &Campaign) -> crate::combat::QueuedAction {
    c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].clone()
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
fn status(c: &Campaign, id: &str, kind: &str) -> bool {
    c.battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .find(|p| p.id == id)
        .unwrap()
        .runtime
        .statuses
        .iter()
        .any(|s| s.kind == kind)
}
fn command(c: &Campaign) -> Action {
    Action::ResolveEffect {
        sequence: q(c).sequence,
        reason: "Reviewed Chance status".into(),
    }
}
fn pending(c: &Campaign, a: &str, t: &str, name: &str) -> Campaign {
    let mut c = hit(c, a, t, name);
    if c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
        c = pass(&c);
    }
    roll(&c, "gm", &damage_command(&c), 1)
}
fn resolve(c: &Campaign, dice: Vec<u8>) -> Campaign {
    battle::transition(c, "gm", &command(c), dice, &catalog())
        .unwrap()
        .0
}
fn equip(c: &mut Campaign, a: &str, name: &str) {
    let sid = pm(c, a).sheet_id.clone();
    let mut s = c.sheets[&sid].clone();
    s.inventory.insert(name.into(), 1);
    c.sheets.insert(
        sid,
        items::equip(&s, name, "chance-item", &catalog()).unwrap(),
    );
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.items.enabled = true;
    f.items.completion.enabled = true;
    items::initialize(c, &catalog()).unwrap();
    items::completion::initialize(c, &catalog());
}

#[test]
fn chance_status_all_47_require_sixes_and_exact_pools_without_changing_damage() {
    let cards = cards();
    assert_eq!(cards["moves"].as_object().unwrap().len(), 47);
    for (name, d) in cards["moves"].as_object().unwrap() {
        let (c, a, t) = setup(name);
        let c = pending(&c, &a, &t, name);
        let n = rules::dice_count(&c, &command(&c), &catalog()).unwrap();
        assert_eq!(n, d["dice"].as_u64().unwrap() as u32, "{name}");
        assert!(
            battle::transition(&c, "gm", &command(&c), vec![6; n as usize + 1], &catalog())
                .is_err()
        );
        assert!(
            battle::transition(&c, "other", &command(&c), vec![6; n as usize], &catalog()).is_err()
        );
        let hp = c.battle.as_ref().unwrap().combatants[1].hp;
        let failed = resolve(&c, vec![5; n as usize]); // successes alone do not trigger Chance
        let kind = d["status"].as_str().unwrap();
        assert!(!status(&failed, &t, kind), "{name}");
        let mut dice = vec![1; n as usize];
        *dice.last_mut().unwrap() = 6;
        let passed = resolve(&c, dice);
        assert!(status(&passed, &t, kind), "{name}: {kind}");
        assert_eq!(
            passed.battle.as_ref().unwrap().combatants[1].hp,
            hp,
            "{name}"
        );
        assert!(!status(&passed, &a, kind), "{name}");
    }
}

#[test]
fn chance_status_pool_modifiers_zero_pool_and_native_flinch_item_merge() {
    for (name, base) in [
        ("Fire Punch", 1),
        ("Poison Fang", 5),
        ("Aqua Tail", 3),
        ("Fiery Wrath", 2),
    ] {
        let (mut c, a, t) = setup(name);
        equip(&mut c, &a, "Loaded Dice");
        pm(&mut c, &a).selected_ability = "Serene Grace".into();
        pm(&mut c, &t).selected_ability = "Wonder Skin".into();
        let c = pending(&c, &a, &t, name);
        assert_eq!(
            rules::dice_count(&c, &command(&c), &catalog()).unwrap(),
            base + 2,
            "{name}"
        );
    }
    for (name, item, expected) in [
        ("Fiery Wrath", "King's Rock", 3),
        ("Aqua Tail", "Razor Fang", 4),
    ] {
        let (mut c, a, t) = setup(name);
        equip(&mut c, &a, item);
        pm(&mut c, &a).selected_ability = "Serene Grace".into();
        pm(&mut c, &t).selected_ability = "Wonder Skin".into();
        let c = pending(&c, &a, &t, name);
        assert_eq!(
            rules::dice_count(&c, &command(&c), &catalog()).unwrap(),
            expected,
            "{item}: merge item and native flinch into one pool"
        );
        let c = resolve(&c, vec![6; expected as usize]);
        assert_eq!(
            c.battle.as_ref().unwrap().combatants[1]
                .runtime
                .statuses
                .iter()
                .filter(|s| s.kind == "Flinch")
                .count(),
            1
        );
    }
    let (mut c, a, t) = setup("Fire Punch");
    pm(&mut c, &t).selected_ability = "Wonder Skin".into();
    let c = pending(&c, &a, &t, "Fire Punch");
    assert_eq!(rules::dice_count(&c, &command(&c), &catalog()).unwrap(), 0);
    assert!(!status(&resolve(&c, vec![]), &t, "Burn 2"));
    let (mut c, a, t) = setup("Aqua Tail");
    equip(&mut c, &a, "Razor Fang");
    pm(&mut c, &a).selected_ability = "Teravolt".into();
    pm(&mut c, &t).selected_ability = "Wonder Skin".into();
    let c = pending(&c, &a, &t, "Aqua Tail");
    assert_eq!(
        rules::dice_count(&c, &command(&c), &catalog()).unwrap(),
        4,
        "Teravolt bypasses Wonder Skin without leaving a duplicate native Flinch pool"
    );
    assert!(status(&resolve(&c, vec![6; 4]), &t, "Flinch"));
}

#[test]
fn chance_status_sheer_force_includes_missing_metadata_and_retains_recoil() {
    for name in [
        "Blazing Torque",
        "Combat Torque",
        "Mountain Gale",
        "Wicked Torque",
        "Wild Charge",
    ] {
        let (mut c, a, t) = setup(name);
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .provenance_rules = true;
        pm(&mut c, &a).selected_ability = "Sheer Force".into();
        c = provenance::choose_sheer_force(&c, "player", &a, true, "Choose damage", &[])
            .unwrap()
            .0;
        let mut c = hit(&c, &a, &t, name);
        assert!(q(&c).sheer_force, "{name}");
        c = pass(&c);
        c = roll(&c, "gm", &damage_command(&c), 1);
        if name != "Wild Charge" {
            assert!(
                c.battle
                    .as_ref()
                    .unwrap()
                    .flow
                    .as_ref()
                    .unwrap()
                    .queue
                    .is_empty(),
                "{name}: no effect phase after Sheer Force"
            );
            assert!(c.battle.as_ref().unwrap().combatants[1]
                .runtime
                .statuses
                .is_empty());
            continue;
        }
        let expected = if name == "Wild Charge" {
            c.battle.as_mut().unwrap().flow.as_mut().unwrap().queue[0].raw_damage_successes = 3;
            3
        } else {
            0
        };
        assert_eq!(
            rules::dice_count(&c, &command(&c), &catalog()).unwrap(),
            expected,
            "{name}"
        );
        let c = resolve(&c, vec![6; expected as usize]);
        assert!(
            c.battle.as_ref().unwrap().combatants[1]
                .runtime
                .statuses
                .is_empty(),
            "{name}"
        );
        assert_eq!(
            c.battle.as_ref().unwrap().combatants[0].hp,
            100 - expected as i32
        );
    }
}

#[test]
fn chance_status_wild_charge_recoil_and_status_use_separate_dice() {
    let (c, a, t) = setup("Wild Charge");
    let mut c = pending(&c, &a, &t, "Wild Charge");
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().queue[0].raw_damage_successes = 2;
    assert_eq!(rules::dice_count(&c, &command(&c), &catalog()).unwrap(), 3);
    let recoil_only = resolve(&c, vec![6, 6, 1]);
    assert_eq!(recoil_only.battle.as_ref().unwrap().combatants[0].hp, 98);
    assert!(!status(&recoil_only, &t, "Paralysis"));
    let status_only = resolve(&c, vec![1, 1, 6]);
    assert_eq!(status_only.battle.as_ref().unwrap().combatants[0].hp, 100);
    assert!(status(&status_only, &t, "Paralysis"));
    for ability in ["Rock Head", "Magic Guard"] {
        let mut guarded = c.clone();
        pm(&mut guarded, &a).selected_ability = ability.into();
        assert_eq!(
            rules::dice_count(&guarded, &command(&guarded), &catalog()).unwrap(),
            1
        );
        assert!(status(&resolve(&guarded, vec![6]), &t, "Paralysis"));
    }
    let mut old = c.clone();
    old.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .chance_status = false;
    assert_eq!(
        rules::dice_count(&old, &command(&old), &catalog()).unwrap(),
        2
    );
    assert!(!status(&resolve(&old, vec![6, 6]), &t, "Paralysis"));
}

#[test]
fn chance_status_guards_type_immunity_conditions_and_once_per_round_flinch() {
    for (name, ability) in [
        ("Fire Punch", "Shield Dust"),
        ("Chatter", "Soundproof"),
        ("Lick", "Limber"),
        ("Heart Stamp", "Inner Focus"),
    ] {
        let (mut c, a, t) = setup(name);
        pm(&mut c, &t).selected_ability = ability.into();
        let c = pending(&c, &a, &t, name);
        let n = rules::dice_count(&c, &command(&c), &catalog()).unwrap();
        let c = resolve(&c, vec![6; n as usize]);
        assert!(
            c.battle.as_ref().unwrap().combatants[1]
                .runtime
                .statuses
                .is_empty(),
            "{name}: {ability}"
        );
    }
    for (name, species) in [
        ("Wild Charge", "Sandshrew"),
        ("Poison Fang", "Magnemite"),
        ("Fire Punch", "Charmander"),
        ("Freezing Glare", "Sneasel"),
    ] {
        let (mut c, a, t) = setup(name);
        let sid = pm(&mut c, &t).sheet_id.clone();
        c.sheets.get_mut(&sid).unwrap().species_id = Some(
            catalog().find("species", "Name", species).unwrap()["AppId"]
                .as_str()
                .unwrap()
                .into(),
        );
        let c = pending(&c, &a, &t, name);
        let n = rules::dice_count(&c, &command(&c), &catalog()).unwrap();
        let c = resolve(&c, vec![6; n as usize]);
        assert!(
            c.battle.as_ref().unwrap().combatants[1]
                .runtime
                .statuses
                .is_empty(),
            "{name}: {species}"
        );
    }
    for name in ["Aqua Tail", "Chatter"] {
        let (c, a, t) = setup(name);
        let mut c = pending(&c, &a, &t, name);
        let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
        f.decoys.enabled = true;
        f.decoys.active.insert(
            t.clone(),
            decoys::Decoy {
                hp: 5,
                defenses: [2, 2],
                source: t.clone(),
                move_name: "Substitute".into(),
            },
        );
        let n = rules::dice_count(&c, &command(&c), &catalog()).unwrap();
        let c = resolve(&c, vec![6; n as usize]);
        assert_eq!(
            c.battle.as_ref().unwrap().combatants[1]
                .runtime
                .statuses
                .is_empty(),
            name != "Chatter"
        );
    }
    let (c, a, t) = setup("Aqua Tail");
    let mut c = pending(&c, &a, &t, "Aqua Tail");
    let queued = q(&c);
    crate::combat::audit_effect_route(&mut c, &queued, &catalog()).unwrap();
    pm(&mut c, &t).runtime.statuses.clear();
    crate::combat::audit_effect_route(&mut c, &queued, &catalog()).unwrap();
    assert!(!status(&c, &t, "Flinch"));
}

#[test]
fn chance_status_old_flows_keep_fallback_and_weather_adds_freeze_die() {
    let (c, a, t) = special_tests::setup("Fire Punch");
    let c = pending(&c, &a, &t, "Fire Punch");
    assert_eq!(
        battle::transition(&c, "gm", &command(&c), vec![], &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
    let (mut c, a, t) = setup("Freezing Glare");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .natural = Some(field::Weather::Snow);
    let c = pending(&c, &a, &t, "Freezing Glare");
    assert_eq!(rules::dice_count(&c, &command(&c), &catalog()).unwrap(), 2);
}

#[test]
fn chance_status_area_and_all_foes_keep_independent_target_rolls() {
    for (name, kind, area) in [
        ("Bleakwind Storm", "Frozen Solid", true),
        ("Sandsear Storm", "Burn 2", true),
        ("Wildbolt Storm", "Paralysis", true),
        ("Fiery Wrath", "Flinch", false),
        ("Heat Wave", "Burn", false),
        ("Searing Shot", "Burn 2", false),
    ] {
        let (mut c, a, t) = setup(name);
        let mut third = c.battle.as_ref().unwrap().combatants[1].clone();
        let mut sheet = c.sheets[&third.sheet_id].clone();
        sheet.id = id();
        third.sheet_id = sheet.id.clone();
        third.id = id();
        if area {
            third.side = "Players".into();
        }
        let third_id = third.id.clone();
        c.sheets.insert(sheet.id.clone(), sheet);
        let b = c.battle.as_mut().unwrap();
        b.combatants.push(third);
        b.flow.as_mut().unwrap().tactics.enabled = true;
        let targets = if area {
            vec![]
        } else {
            vec![t.clone(), third_id.clone()]
        };
        let mut accuracy = attack(&a, targets, name);
        if let Action::AccuracyTargets { pool_modifier, .. } = &mut accuracy {
            *pool_modifier = 10;
        }
        let mut c = roll(&c, "gm", &accuracy, 6);
        assert_eq!(q(&c).targets.len(), 2, "{name}");
        c = pass(&pass(&c));
        let first = q(&c).target_id;
        for face in [1, 6] {
            c = roll(&c, "gm", &damage_command(&c), 1);
            let n = rules::dice_count(&c, &command(&c), &catalog()).unwrap();
            c = resolve(&c, vec![face; n as usize]);
        }
        assert!(
            !status(&c, &first, kind),
            "{name}: failed Chance stays local"
        );
        let last = if first == t { third_id } else { t };
        assert!(status(&c, &last, kind), "{name}: second target succeeds");
    }
}

#[test]
fn chance_status_shields_block_effect_but_preserve_wild_charge_recoil_and_rollback() {
    let (c, a, t) = setup("Wild Charge");
    let mut c = pending(&c, &a, &t, "Wild Charge");
    let round = c.battle.as_ref().unwrap().round;
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.queue[0].raw_damage_successes = 2;
    f.special.shields.insert(
        t.clone(),
        special::Shield {
            name: "Protect".into(),
            source: t.clone(),
            round,
            armed_turn: 0,
            available: true,
        },
    );
    let before = serde_json::to_value(&c.battle).unwrap();
    let done = resolve(&c, vec![6, 6, 6]);
    assert!(!status(&done, &t, "Paralysis"));
    assert_eq!(done.battle.as_ref().unwrap().combatants[0].hp, 98);
    let restored = decision(
        &done,
        "gm",
        Action::RollbackBattle {
            event_sequence: done.battle_undo.last().unwrap().event_sequence,
            reason: "Undo chance and recoil".into(),
        },
    );
    assert_eq!(serde_json::to_value(&restored.battle).unwrap(), before);
}
