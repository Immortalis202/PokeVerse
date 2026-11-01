use super::targeting_tests::{damage_command, pass, roll};
use super::*;
use crate::combat::{damaging_stats, decoys, items, provenance, special};

fn cards() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../rules/damaging-stat-moves.json")).unwrap()
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
fn value(c: &Campaign, id: &str, key: &str) -> i32 {
    *c.battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .find(|p| p.id == id)
        .unwrap()
        .runtime
        .modifiers
        .get(key)
        .unwrap_or(&0)
}
fn setup(name: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = special_tests::setup(name);
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.damaging_stats.enabled = true;
    f.reviewed_rules = true;
    f.interaction_rules = true;
    for p in &mut c.battle.as_mut().unwrap().combatants {
        p.hp = 100;
        p.max_hp = 100;
    }
    let sid = pm(&mut c, &t).sheet_id.clone();
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
fn command(c: &Campaign) -> Action {
    Action::ResolveEffect {
        sequence: q(c).sequence,
        reason: "Reviewed damaging stats".into(),
    }
}
fn pending(c: &Campaign, a: &str, t: &str, name: &str) -> Campaign {
    let mut c = hit(c, a, t, name);
    if c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
        c = pass(&c);
    }
    roll(&c, "gm", &damage_command(&c), 1)
}
fn resolve(c: &Campaign, face: u8) -> Campaign {
    let n = rules::dice_count(c, &command(c), &catalog()).unwrap();
    battle::transition(c, "gm", &command(c), vec![face; n as usize], &catalog())
        .unwrap()
        .0
}

#[test]
fn damaging_stats_all_66_exact_pools_and_recipients_through_public_commands() {
    assert_eq!(cards()["moves"].as_object().unwrap().len(), 66);
    for (name, d) in cards()["moves"].as_object().unwrap() {
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
        let passed = resolve(&c, 6);
        let failed = resolve(&c, 5);
        let recipient = if d["user"] == true { &a } else { &t };
        let other = if d["user"] == true { &t } else { &a };
        for key in d["keys"].as_array().unwrap() {
            let key = key.as_str().unwrap();
            let amount = d["amount"].as_i64().unwrap() as i32;
            assert_eq!(value(&passed, recipient, key), amount, "{name}: {key}");
            assert_eq!(value(&passed, other, key), 0, "{name}: wrong recipient");
            assert_eq!(
                value(&failed, recipient, key),
                if n == 0 { amount } else { 0 },
                "{name}: Chance needs six"
            );
        }
        assert_eq!(
            serde_json::to_value(&passed.sheets).unwrap(),
            serde_json::to_value(&c.sheets).unwrap(),
            "{name}: permanent sheets"
        );
        for (before, after) in c
            .battle
            .as_ref()
            .unwrap()
            .combatants
            .iter()
            .zip(&passed.battle.as_ref().unwrap().combatants)
        {
            assert_eq!(before.hp, after.hp, "{name}: effect does not repeat damage");
        }
    }
}

#[test]
fn damaging_stats_immunities_contrary_simple_and_nonstacking() {
    for (name, ability, expected) in [
        ("Acid Spray", "Clear Body", 0),
        ("Acid Spray", "Contrary", 2),
        ("Acid Spray", "Simple", -3),
        ("Acid Spray", "Shield Dust", -2),
        ("Shadow Ball", "Shield Dust", 0),
    ] {
        let (mut c, a, t) = setup(name);
        pm(&mut c, &t).selected_ability = ability.into();
        let mut c = pending(&c, &a, &t, name);
        let queued = q(&c);
        let n = damaging_stats::effect_dice(&c, &queued).unwrap().unwrap();
        damaging_stats::effect(&mut c, &queued, &vec![6; n as usize], &catalog()).unwrap();
        damaging_stats::effect(&mut c, &queued, &vec![6; n as usize], &catalog()).unwrap();
        assert_eq!(
            special::modifier(
                &c,
                c.battle
                    .as_ref()
                    .unwrap()
                    .combatants
                    .iter()
                    .find(|p| p.id == t)
                    .unwrap(),
                "SpecialDefense"
            )
            .unwrap_or(0),
            expected,
            "{ability}: nonstacking"
        );
    }
    let (mut c, a, t) = setup("Shadow Ball");
    pm(&mut c, &t).selected_ability = "Mirror Armor".into();
    let c = resolve(&pending(&c, &a, &t, "Shadow Ball"), 6);
    assert_eq!(value(&c, &a, "SpecialDefense"), -1);
    assert_eq!(value(&c, &t, "SpecialDefense"), 0);
}

#[test]
fn damaging_stats_chance_modifiers_items_and_sheer_force() {
    let (mut c, a, t) = setup("Shadow Ball");
    let sid = pm(&mut c, &a).sheet_id.clone();
    let mut sheet = c.sheets[&sid].clone();
    sheet.inventory.insert("Loaded Dice".into(), 1);
    c.sheets.insert(
        sid,
        items::equip(&sheet, "Loaded Dice", "stat-item", &catalog()).unwrap(),
    );
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.items.enabled = true;
    f.items.completion.enabled = true;
    items::initialize(&mut c, &catalog()).unwrap();
    items::completion::initialize(&mut c, &catalog());
    pm(&mut c, &a).selected_ability = "Serene Grace".into();
    pm(&mut c, &t).selected_ability = "Wonder Skin".into();
    let c = pending(&c, &a, &t, "Shadow Ball");
    assert_eq!(rules::dice_count(&c, &command(&c), &catalog()).unwrap(), 3);
    let (mut c, a, t) = setup("Shadow Ball");
    pm(&mut c, &t).selected_ability = "Wonder Skin".into();
    let c = pending(&c, &a, &t, "Shadow Ball");
    assert_eq!(rules::dice_count(&c, &command(&c), &catalog()).unwrap(), 0);
    assert_eq!(value(&resolve(&c, 6), &t, "SpecialDefense"), 0);
    for name in ["Shadow Ball", "Acid Spray", "Aqua Step"] {
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
        let c = hit(&c, &a, &t, name);
        assert!(q(&c).sheer_force, "{name}");
        let c = roll(&pass(&c), "gm", &damage_command(&pass(&c)), 1);
        assert!(c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .queue
            .is_empty());
        assert_eq!(value(&c, &a, "Dexterity"), 0);
        assert_eq!(value(&c, &t, "SpecialDefense"), 0);
    }
}

#[test]
fn damaging_stats_decoys_sound_shields_and_historical_fallback() {
    for name in ["Acid Spray", "Snarl"] {
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
        let c = resolve(&c, 6);
        assert_eq!(
            value(
                &c,
                &t,
                if name == "Snarl" {
                    "Special"
                } else {
                    "SpecialDefense"
                }
            ),
            if name == "Snarl" { -1 } else { 0 }
        );
    }
    let (c, a, t) = setup("Acid Spray");
    let mut c = pending(&c, &a, &t, "Acid Spray");
    let round = c.battle.as_ref().unwrap().round;
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .special
        .shields
        .insert(
            t.clone(),
            special::Shield {
                name: "Protect".into(),
                source: t.clone(),
                round,
                armed_turn: 0,
                available: true,
            },
        );
    assert_eq!(value(&resolve(&c, 6), &t, "SpecialDefense"), 0);
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.special.shields.clear();
    f.damaging_stats.enabled = false;
    let e = battle::transition(&c, "gm", &command(&c), vec![], &catalog()).unwrap_err();
    assert_eq!(e.code, "manual_rule");
    let mut serialized = serde_json::to_value(&c).unwrap();
    serialized["battle"]["flow"]
        .as_object_mut()
        .unwrap()
        .remove("damaging_stats");
    let old: Campaign = serde_json::from_value(serialized).unwrap();
    assert!(!old.battle.unwrap().flow.unwrap().damaging_stats.enabled);
}

#[test]
fn damaging_stats_user_pool_consumed_once_and_survives_serialization() {
    let (c, a, t) = setup("Diamond Storm");
    let mut c = pending(&c, &a, &t, "Diamond Storm");
    let queued = q(&c);
    damaging_stats::effect(&mut c, &queued, &[5; 5], &catalog()).unwrap();
    let mut c: Campaign = serde_json::from_value(serde_json::to_value(c).unwrap()).unwrap();
    assert_eq!(damaging_stats::effect_dice(&c, &queued).unwrap(), Some(0));
    damaging_stats::effect(&mut c, &queued, &[], &catalog()).unwrap();
    assert_eq!(
        value(&c, &a, "Defense"),
        0,
        "later area target cannot reroll failed cast pool"
    );
    let (c, a, t) = setup("Diamond Storm");
    let c = pending(&c, &a, &t, "Diamond Storm");
    let before = serde_json::to_value(&c.battle).unwrap();
    let done = resolve(&c, 6);
    assert_eq!(value(&done, &a, "Defense"), 2);
    let restored = decision(
        &done,
        "gm",
        Action::RollbackBattle {
            event_sequence: done.battle_undo.last().unwrap().event_sequence,
            reason: "Undo damaging stat cast".into(),
        },
    );
    assert_eq!(serde_json::to_value(&restored.battle).unwrap(), before);
}

#[test]
fn damaging_stats_area_targets_get_independent_pools_and_user_gets_one_pool() {
    for name in ["Acid", "Diamond Storm"] {
        let (mut c, a, t) = setup(name);
        let b = c.battle.as_mut().unwrap();
        let mut third = b.combatants[1].clone();
        third.id = id();
        let third_id = third.id.clone();
        b.combatants.push(third);
        b.flow.as_mut().unwrap().tactics.enabled = true;
        let mut accuracy = targeting_tests::attack(&a, vec![t.clone(), third_id.clone()], name);
        if let Action::AccuracyTargets { pool_modifier, .. } = &mut accuracy {
            *pool_modifier = 10;
        }
        let mut c = roll(&c, "gm", &accuracy, 6);
        c = pass(&pass(&c));
        let first = q(&c).target_id;
        for face in [5, 6] {
            c = roll(&c, "gm", &damage_command(&c), 1);
            if name == "Diamond Storm" && face == 6 {
                assert_eq!(rules::dice_count(&c, &command(&c), &catalog()).unwrap(), 0);
            }
            c = resolve(&c, face);
        }
        if name == "Acid" {
            assert_eq!(value(&c, &first, "SpecialDefense"), 0);
            let last = if first == t { third_id } else { t };
            assert_eq!(value(&c, &last, "SpecialDefense"), -1);
        } else {
            assert_eq!(value(&c, &a, "Defense"), 0);
            assert!(c
                .battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .damaging_stats
                .completed
                .is_empty());
        }
    }
}
