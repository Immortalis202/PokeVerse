use super::targeting_tests::{damage_command, pass, roll};
use super::*;
use crate::combat::{completion, decoys, special};

pub(super) fn p<'a>(c: &'a Campaign, id: &str) -> &'a Combatant {
    c.battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .find(|p| p.id == id)
        .unwrap()
}
pub(super) fn pm<'a>(c: &'a mut Campaign, id: &str) -> &'a mut Combatant {
    c.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == id)
        .unwrap()
}
pub(super) fn setup(name: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = special_tests::setup(name);
    c.engine = ENGINE.into();
    for p in &mut c.battle.as_mut().unwrap().combatants {
        p.hp = 100;
        p.max_hp = 100;
    }
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.completion = Box::new(completion::initial(ENGINE));
    f.reviewed_rules = true;
    f.interaction_rules = true;
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some(
        catalog().find("species", "Name", "Shuckle").unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
    (c, a, t)
}
pub(super) fn q(c: &Campaign) -> crate::combat::QueuedAction {
    c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].clone()
}
pub(super) fn cmd(c: &Campaign) -> Action {
    Action::ResolveEffect {
        sequence: q(c).sequence,
        reason: "Complete source-reviewed move".into(),
    }
}
pub(super) fn pending(c: &Campaign, a: &str, t: &str, name: &str) -> Campaign {
    let action = Action::Accuracy {
        combatant_id: a.into(),
        target_id: Some(t.into()),
        move_name: name.into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Owner move accuracy".into(),
    };
    let mut c = roll(c, &p(c, a).owner_id, &action, 6);
    if c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
        c = pass(&c);
    }
    if catalog().find("moves", "Name", name).unwrap()["Category"] != "Support" {
        c = roll(&c, "gm", &damage_command(&c), 1);
    }
    c
}
pub(super) fn resolve(c: &Campaign) -> Campaign {
    roll(c, "gm", &cmd(c), 6)
}
pub(super) fn turn(c: &mut Campaign, a: &str, name: &str) {
    let sid = p(c, a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().moves.push(name.into());
    pm(c, a).used_moves.clear();
    pm(c, a).actions = 0;
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.queue.clear();
    f.window = false;
    f.order = vec![a.into()];
    f.cursor = 0;
    f.turn += 1;
}
pub(super) fn reserve(c: &mut Campaign, id_: &str) -> String {
    let mut new = p(c, id_).clone();
    let mut s = c.sheets[&new.sheet_id].clone();
    s.id = id();
    s.trainer_id = Some("move-trainer".into());
    c.sheets.get_mut(&new.sheet_id).unwrap().trainer_id = s.trainer_id.clone();
    new.sheet_id = s.id.clone();
    new.id = id();
    new.deployed = false;
    new.field_slot = None;
    c.sheets.insert(s.id.clone(), s);
    let key = new.id.clone();
    let b = c.battle.as_mut().unwrap();
    b.slots_per_side = 1;
    b.combatants.push(new);
    key
}

#[test]
fn completion_a03_all_seven_traps_use_public_effect_commands_and_reject_bad_authority() {
    for name in [
        "Anchor Shot",
        "Block",
        "Jaw Lock",
        "Mean Look",
        "Snap Trap",
        "Spirit Shackle",
        "Thousand Waves",
    ] {
        let (c, a, t) = setup(name);
        let c = pending(&c, &a, &t, name);
        assert_eq!(rules::dice_count(&c, &cmd(&c), &catalog()).unwrap(), 0);
        assert!(battle::transition(&c, "intruder", &cmd(&c), vec![], &catalog()).is_err());
        assert!(battle::transition(&c, "gm", &cmd(&c), vec![6], &catalog()).is_err());
        let hp = p(&c, &t).hp;
        let sheets = serde_json::to_value(&c.sheets).unwrap();
        let c = resolve(&c);
        assert!(completion::blocked(&c, &t), "{name}");
        assert_eq!(completion::blocked(&c, &a), name == "Jaw Lock", "{name}");
        assert_eq!(p(&c, &t).hp, hp);
        assert_eq!(serde_json::to_value(&c.sheets).unwrap(), sheets);
        assert!(
            !c.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .special
                .effects
                .iter()
                .any(|e| e.name == "Snap Trap"),
            "no invented ongoing tick"
        );
    }
}
#[test]
fn completion_a03_switch_and_forced_switch_are_blocked_with_explicit_gm_override() {
    let (mut c, a, t) = setup("Block");
    let incoming = reserve(&mut c, &t);
    let mut c = resolve(&pending(&c, &a, &t, "Block"));
    let before = serde_json::to_value(&c).unwrap();
    let action = Action::SwitchPokemon {
        outgoing_id: t.clone(),
        incoming_id: incoming.clone(),
        gm_override: false,
        reason: "Attempt recall".into(),
    };
    assert!(battle::transition(&c, "gm", &action, vec![], &catalog()).is_err());
    assert_eq!(serde_json::to_value(&c).unwrap(), before);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .tactics
        .enabled = true;
    turn(&mut c, &a, "Roar");
    let c = resolve(&pending(&c, &a, &t, "Roar"));
    assert!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .tactics
            .switches
            .is_empty(),
        "Blocking stops forced Switcher effects"
    );
    let action = Action::SwitchPokemon {
        outgoing_id: t.clone(),
        incoming_id: incoming.clone(),
        gm_override: true,
        reason: "Explicit GM escape ruling".into(),
    };
    let done = decision(&c, "gm", action);
    assert!(!p(&done, &t).deployed);
    assert!(p(&done, &incoming).deployed);
}
#[test]
fn completion_a03_run_away_shields_decoys_and_type_immunity() {
    for name in ["Block", "Anchor Shot"] {
        let (mut c, a, t) = setup(name);
        pm(&mut c, &t).selected_ability = "Run Away".into();
        let c = resolve(&pending(&c, &a, &t, name));
        assert!(!completion::blocked(&c, &t));
        for shield in [false, true] {
            let (c, a, t) = setup(name);
            let mut c = pending(&c, &a, &t, name);
            let round = c.battle.as_ref().unwrap().round;
            let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
            if shield {
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
            } else {
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
            }
            assert!(!completion::blocked(&resolve(&c), &t));
        }
    }
    let (mut c, a, t) = setup("Block");
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some(
        catalog().find("species", "Name", "Gengar").unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
    assert!(
        completion::blocked(&resolve(&pending(&c, &a, &t, "Block")), &t),
        "no imported blanket Ghost trapping immunity"
    );
    let (mut c, a, t) = setup("Thousand Waves");
    pm(&mut c, &t).selected_ability = "Levitate".into();
    let c = resolve(&pending(&c, &a, &t, "Thousand Waves"));
    assert!(!completion::blocked(&c, &t));
}
#[test]
fn completion_a03_source_departure_jaw_pair_release_save_and_rollback() {
    for name in ["Mean Look", "Jaw Lock"] {
        let (c, a, t) = setup(name);
        let c = pending(&c, &a, &t, name);
        let before = serde_json::to_value(&c.battle).unwrap();
        let c = resolve(&c);
        let c: Campaign = serde_json::from_value(serde_json::to_value(&c).unwrap()).unwrap();
        assert!(completion::blocked(&c, &t));
        let restored = decision(
            &c,
            "gm",
            Action::RollbackBattle {
                event_sequence: c.battle_undo.last().unwrap().event_sequence,
                reason: "Undo trap".into(),
            },
        );
        assert_eq!(serde_json::to_value(&restored.battle).unwrap(), before);
        let mut departed = c.clone();
        pm(&mut departed, &a).deployed = false;
        special::lifecycle(&c, &mut departed, &catalog()).unwrap();
        assert_eq!(completion::blocked(&departed, &t), name != "Jaw Lock");
        let mut fainted = c.clone();
        pm(&mut fainted, &a).hp = 0;
        special::lifecycle(&c, &mut fainted, &catalog()).unwrap();
        assert_eq!(completion::blocked(&fainted, &t), name != "Jaw Lock");
    }
}
#[test]
fn completion_a03_area_recipients_reflection_and_historical_adoption() {
    let (mut c, a, t) = setup("Thousand Waves");
    let mut third = p(&c, &t).clone();
    third.id = id();
    third.selected_ability = "Run Away".into();
    let u = third.id.clone();
    c.battle.as_mut().unwrap().combatants.push(third);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .tactics
        .enabled = true;
    let action = targeting_tests::attack(&a, vec![t.clone(), u.clone()], "Thousand Waves");
    let mut c = roll(&c, "gm", &action, 6);
    c = pass(&pass(&c));
    for _ in 0..2 {
        c = roll(&c, "gm", &damage_command(&c), 1);
        c = resolve(&c);
    }
    assert!(completion::blocked(&c, &t));
    assert!(!completion::blocked(&c, &u));
    let (mut c, a, t) = setup("Mean Look");
    pm(&mut c, &t).selected_ability = "Magic Bounce".into();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .tactics
        .enabled = true;
    let c = resolve(&pending(&c, &a, &t, "Mean Look"));
    assert!(completion::blocked(&c, &a));
    assert!(!completion::blocked(&c, &t));
    let (mut c, a, t) = setup("Block");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .completion
        .blocks
        .clear();
    let c = pending(&c, &a, &t, "Block");
    assert_eq!(
        battle::transition(&c, "gm", &cmd(&c), vec![], &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
    let mut data = serde_json::to_value(&c).unwrap();
    data["battle"]["flow"]
        .as_object_mut()
        .unwrap()
        .remove("completion");
    let old: Campaign = serde_json::from_value(data).unwrap();
    assert!(old
        .battle
        .unwrap()
        .flow
        .unwrap()
        .completion
        .blocks
        .is_empty());
}

fn ongoing(c: &Campaign, t: &str) -> (usize, crate::combat::special::TimedEffect) {
    c.battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .special
        .effects
        .iter()
        .enumerate()
        .find(|(_, e)| e.target == t && e.name.starts_with("ongoing:"))
        .map(|(i, e)| (i, e.clone()))
        .unwrap()
}
fn tick_action(c: &Campaign, t: &str) -> Action {
    Action::SpecialMove {
        command: crate::combat::special::SpecialCommand::Tick {
            index: ongoing(c, t).0 as u32,
        },
        reason: "Resolve due ongoing damage".into(),
    }
}
fn round_end(c: &Campaign) -> Campaign {
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
#[test]
fn completion_a04_all_ten_moves_tick_once_per_round_and_expire_after_four_ticks() {
    for name in [
        "Bind",
        "Clamp",
        "Fire Spin",
        "Infestation",
        "Sand Tomb",
        "Thunder Cage",
        "Whirlpool",
        "Wrap",
        "Ceaseless Edge",
        "Stone Axe",
    ] {
        let (c, a, t) = setup(name);
        let c = pending(&c, &a, &t, name);
        let hp = p(&c, &t).hp;
        let mut c = resolve(&c);
        assert_eq!(ongoing(&c, &t).1.rounds, Some(4));
        assert_eq!(
            completion::blocked(&c, &t),
            !["Ceaseless Edge", "Stone Axe"].contains(&name)
        );
        assert_eq!(completion::blocked(&c, &a), name == "Clamp");
        assert!(
            rules::dice_count(&c, &tick_action(&c, &t), &catalog()).is_err(),
            "no early ticks"
        );
        for i in 0..4 {
            let round = c.battle.as_ref().unwrap().round;
            c = round_end(&c);
            assert_eq!(
                c.battle.as_ref().unwrap().round,
                round,
                "end phase must wait for damage"
            );
            let action = tick_action(&c, &t);
            assert_eq!(rules::dice_count(&c, &action, &catalog()).unwrap(), 2);
            assert!(battle::transition(&c, "intruder", &action, vec![6, 1], &catalog()).is_err());
            assert!(battle::transition(&c, "gm", &action, vec![6], &catalog()).is_err());
            c = battle::transition(&c, "gm", &action, vec![6, 1], &catalog())
                .unwrap()
                .0;
            assert_eq!(
                p(&c, &t).hp,
                hp - i - 1,
                "{name}: two raw damage dice, no defense subtraction"
            );
            assert!(
                battle::transition(&c, "gm", &action, vec![6, 6], &catalog()).is_err(),
                "no second tick"
            );
            c = round_end(&c);
            assert_eq!(c.battle.as_ref().unwrap().round, round + 1);
        }
        assert!(!c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .special
            .effects
            .iter()
            .any(|e| e.target == t && e.name.starts_with("ongoing:")));
        assert!(!completion::blocked(&c, &t));
        assert!(!completion::blocked(&c, &a));
    }
}
#[test]
fn completion_a04_first_source_priority_cleanup_and_unrelated_scene_block() {
    let (c, a, t) = setup("Wrap");
    let mut c = resolve(&pending(&c, &a, &t, "Wrap"));
    turn(&mut c, &a, "Fire Spin");
    c = resolve(&pending(&c, &a, &t, "Fire Spin"));
    assert_eq!(ongoing(&c, &t).1.name, "ongoing:Wrap");
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .special
            .effects
            .iter()
            .filter(|e| e.name.starts_with("ongoing:"))
            .count(),
        1
    );
    turn(&mut c, &a, "Mean Look");
    c = resolve(&pending(&c, &a, &t, "Mean Look"));
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .compound
        .enabled = true;
    turn(&mut c, &t, "Mortal Spin");
    c = resolve(&pending(&c, &t, &a, "Mortal Spin"));
    let effects = &c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .special
        .effects;
    assert!(!effects.iter().any(|e| e.target == t
        && (e.name.starts_with("ongoing:")
            || ["move-block:Wrap", "move-block:Fire Spin"].contains(&e.name.as_str()))));
    assert!(
        completion::blocked(&c, &t),
        "Mean Look is not an ongoing-damage effect"
    );
    let (c, a, t) = setup("Clamp");
    let mut c = resolve(&pending(&c, &a, &t, "Clamp"));
    completion::cleanse(&mut c, &t);
    assert!(!completion::blocked(&c, &a));
    assert!(!completion::blocked(&c, &t));
}
#[test]
fn completion_a04_damage_immunity_is_live_and_caster_can_leave_without_cancelling_ticks() {
    for ability in ["Magic Guard", "Wonder Guard", ""] {
        let (c, a, t) = setup("Bind");
        let mut c = resolve(&pending(&c, &a, &t, "Bind"));
        pm(&mut c, &t).selected_ability = ability.into();
        pm(&mut c, &a).deployed = false;
        let before = c.clone();
        special::lifecycle(&before, &mut c, &catalog()).unwrap();
        let c: Campaign = serde_json::from_value(serde_json::to_value(&c).unwrap()).unwrap();
        let c = round_end(&c);
        let hp = p(&c, &t).hp;
        let before = serde_json::to_value(&c.battle).unwrap();
        let action = tick_action(&c, &t);
        let done = battle::transition(&c, "player", &action, vec![6, 6], &catalog())
            .unwrap()
            .0;
        assert_eq!(
            p(&done, &t).hp,
            hp - if ability.is_empty() { 2 } else { 0 },
            "{ability}"
        );
        let restored = decision(
            &done,
            "gm",
            Action::RollbackBattle {
                event_sequence: done.battle_undo.last().unwrap().event_sequence,
                reason: "Undo ongoing tick".into(),
            },
        );
        assert_eq!(serde_json::to_value(&restored.battle).unwrap(), before);
    }
    let (mut c, a, t) = setup("Bind");
    pm(&mut c, &t).selected_ability = "Run Away".into();
    let c = resolve(&pending(&c, &a, &t, "Bind"));
    assert!(!completion::blocked(&c, &t));
    assert!(ongoing(&c, &t).1.name.ends_with("Bind"));
}
#[test]
fn completion_a04_sheer_force_recognizes_metadata_empty_ongoing_and_old_blocks_stay_old() {
    let (mut c, a, t) = setup("Ceaseless Edge");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .provenance_rules = true;
    pm(&mut c, &a).selected_ability = "Sheer Force".into();
    c = crate::combat::provenance::choose_sheer_force(
        &c,
        "player",
        &a,
        true,
        "Trade effects for damage",
        &[],
    )
    .unwrap()
    .0;
    let c = hit(&c, &a, &t, "Ceaseless Edge");
    assert!(q(&c).sheer_force);
    let c = pass(&c);
    let c = roll(&c, "gm", &damage_command(&c), 1);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .is_empty());
    assert!(!c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .special
        .effects
        .iter()
        .any(|e| e.name.starts_with("ongoing:")));
    assert_eq!(
        completion::initial(BASIC_BLOCK_ENGINE)
            .blocks
            .into_iter()
            .collect::<Vec<_>>(),
        vec!["MOVE-A03"]
    );
    let (mut c, a, t) = setup("Bind");
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
        Box::new(completion::initial(BASIC_BLOCK_ENGINE));
    let c = pending(&c, &a, &t, "Bind");
    assert_eq!(
        battle::transition(&c, "gm", &cmd(&c), vec![], &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
}

fn set_species(c: &mut Campaign, t: &str, name: &str) {
    let sid = p(c, t).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some(
        catalog().find("species", "Name", name).unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
}
#[test]
fn completion_a05_magma_storm_has_three_typed_dice_and_live_fire_immunity() {
    for ability in ["", "Flash Fire", "Magic Guard"] {
        let (c, a, t) = setup("Magma Storm");
        let mut c = resolve(&pending(&c, &a, &t, "Magma Storm"));
        assert!(completion::blocked(&c, &t));
        assert_eq!(ongoing(&c, &t).1.rounds, Some(4));
        pm(&mut c, &t).selected_ability = ability.into();
        let c = round_end(&c);
        let hp = p(&c, &t).hp;
        let action = tick_action(&c, &t);
        assert_eq!(rules::dice_count(&c, &action, &catalog()).unwrap(), 3);
        let c = battle::transition(&c, "gm", &action, vec![6, 6, 1], &catalog())
            .unwrap()
            .0;
        assert_eq!(
            p(&c, &t).hp,
            hp - if ability.is_empty() { 2 } else { 0 },
            "{ability}"
        );
    }
}
#[test]
fn completion_a05_salt_cure_overrides_initial_typing_and_adds_one_flat_ongoing_damage() {
    for (species, adjustment, extra) in [
        ("Mawile", 1, 1),
        ("Mudkip", 1, 1),
        ("Empoleon", 1, 1),
        ("Eevee", 0, 0),
        ("Charizard", 2, 0),
    ] {
        let (mut c, a, t) = setup("Salt Cure");
        set_species(&mut c, &t, species);
        let c = hit(&c, &a, &t, "Salt Cure");
        let c = pass(&c);
        let c = roll(&c, "gm", &damage_command(&c), 6);
        assert_eq!(
            q(&c).damage,
            q(&c).raw_damage_successes as i32 + adjustment,
            "{species}"
        );
        let c = resolve(&c);
        assert!(!completion::blocked(&c, &t));
        let c = round_end(&c);
        let hp = p(&c, &t).hp;
        let action = tick_action(&c, &t);
        assert_eq!(rules::dice_count(&c, &action, &catalog()).unwrap(), 2);
        let c = battle::transition(&c, "gm", &action, vec![1, 1], &catalog())
            .unwrap()
            .0;
        assert_eq!(
            p(&c, &t).hp,
            hp - extra,
            "{species}: fixed extra damage is not another die and both types do not stack"
        );
    }
}
#[test]
fn completion_a05_salt_cure_uses_live_types_and_retains_earlier_encounter_typing() {
    let (mut c, a, t) = setup("Salt Cure");
    set_species(&mut c, &t, "Mudkip");
    let c = resolve(&pending(&c, &a, &t, "Salt Cure"));
    let c = round_end(&c);
    let c = decision(
        &c,
        "gm",
        Action::Battlefield {
            command: crate::combat::field::FieldCommand::Typing {
                combatant_id: t.clone(),
                mode: "replace".into(),
                types: vec!["Normal".into()],
            },
            reason: "Change live typing before ongoing damage".into(),
        },
    );
    let hp = p(&c, &t).hp;
    let action = tick_action(&c, &t);
    let c = battle::transition(&c, "gm", &action, vec![1, 1], &catalog())
        .unwrap()
        .0;
    assert_eq!(p(&c, &t).hp, hp);
    let (mut c, a, t) = setup("Salt Cure");
    set_species(&mut c, &t, "Mawile");
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
        Box::new(completion::initial(ONGOING_MOVES_ENGINE));
    let c = hit(&c, &a, &t, "Salt Cure");
    let c = pass(&c);
    let c = roll(&c, "gm", &damage_command(&c), 6);
    assert_eq!(q(&c).damage, (q(&c).raw_damage_successes as i32 - 1).max(0));
    assert_eq!(
        battle::transition(&c, "gm", &cmd(&c), vec![], &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
}
#[test]
fn completion_a06_all_68_ordinary_attacks_finish_after_damage_without_sheet_mutation() {
    let registry: serde_json::Value =
        serde_json::from_str(include_str!("../../../rules/move-completion.json")).unwrap();
    let moves = registry["blocks"]["MOVE-A06"]["moves"].as_object().unwrap();
    assert_eq!(moves.len(), 68);
    for name in moves.keys() {
        let (c, a, t) = setup(name);
        let sheets = serde_json::to_value(&c.sheets).unwrap();
        let c = pending(&c, &a, &t, name);
        assert!(
            c.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .queue
                .is_empty(),
            "{name}"
        );
        assert_eq!(p(&c, &a).actions, 1, "{name}");
        assert_eq!(serde_json::to_value(&c.sheets).unwrap(), sheets, "{name}");
    }
}
#[test]
fn completion_a06_retains_historical_manual_route_and_optional_item_chance() {
    let (mut c, a, t) = setup("Pound");
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
        Box::new(completion::initial(ONGOING_EXCEPTIONS_ENGINE));
    let old = pending(&c, &a, &t, "Pound");
    assert!(!old
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .is_empty());
    let (mut c, a, t) = setup("Pound");
    let sid = p(&c, &a).sheet_id.clone();
    let mut s = c.sheets[&sid].clone();
    s.inventory.insert("Razor Fang".into(), 1);
    c.sheets.insert(
        sid,
        crate::combat::items::equip(&s, "Razor Fang", "ordinary-item", &catalog()).unwrap(),
    );
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.items.enabled = true;
    f.items.completion.enabled = true;
    crate::combat::items::initialize(&mut c, &catalog()).unwrap();
    crate::combat::items::completion::initialize(&mut c, &catalog());
    let c = pending(&c, &a, &t, "Pound");
    assert_eq!(rules::dice_count(&c, &cmd(&c), &catalog()).unwrap(), 1);
    let c = resolve(&c);
    assert!(p(&c, &t)
        .runtime
        .statuses
        .iter()
        .any(|s| s.kind == "Flinch"));
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
#[test]
fn completion_a07_ten_stat_attacks_use_exact_pools_recipients_and_six_only_success() {
    let registry: serde_json::Value =
        serde_json::from_str(include_str!("../../../rules/move-completion.json")).unwrap();
    let moves = registry["blocks"]["MOVE-A07"]["moves"].as_object().unwrap();
    assert_eq!(moves.len(), 10);
    for (name, d) in moves {
        let (c, a, t) = setup(name);
        let c = pending(&c, &a, &t, name);
        let n = rules::dice_count(&c, &cmd(&c), &catalog()).unwrap();
        assert_eq!(n, d["dice"].as_u64().unwrap() as u32, "{name}");
        assert!(
            battle::transition(&c, "intruder", &cmd(&c), vec![6; n as usize], &catalog()).is_err()
        );
        assert!(
            battle::transition(&c, "gm", &cmd(&c), vec![6; n as usize + 1], &catalog()).is_err()
        );
        for face in [5, 6] {
            let after = roll(&c, "gm", &cmd(&c), face);
            let recipient = if d["user"] == true { &a } else { &t };
            let other = if d["user"] == true { &t } else { &a };
            for key in d["keys"].as_array().unwrap() {
                let key = key.as_str().unwrap();
                assert_eq!(
                    p(&after, recipient)
                        .runtime
                        .modifiers
                        .get(key)
                        .copied()
                        .unwrap_or(0),
                    if n == 0 || face == 6 {
                        d["amount"].as_i64().unwrap() as i32
                    } else {
                        0
                    },
                    "{name}"
                );
                assert_eq!(
                    p(&after, other)
                        .runtime
                        .modifiers
                        .get(key)
                        .copied()
                        .unwrap_or(0),
                    0,
                    "{name}"
                );
            }
            assert_eq!(p(&after, &t).hp, p(&c, &t).hp);
            assert_eq!(
                serde_json::to_value(&after.sheets).unwrap(),
                serde_json::to_value(&c.sheets).unwrap()
            );
        }
    }
}
#[test]
fn completion_a07_stat_protections_chance_modifiers_and_historical_adoption() {
    for (name, ability, expected) in [
        ("Lumina Crash", "Clear Body", 0),
        ("Lumina Crash", "Contrary", 2),
        ("Lumina Crash", "Simple", -3),
        ("Liquidation", "Shield Dust", 0),
    ] {
        let (mut c, a, t) = setup(name);
        pm(&mut c, &t).selected_ability = ability.into();
        let c = resolve(&pending(&c, &a, &t, name));
        let key = if name == "Liquidation" {
            "Defense"
        } else {
            "SpecialDefense"
        };
        assert_eq!(
            special::modifier(&c, p(&c, &t), key).unwrap_or(0),
            expected,
            "{ability}"
        );
    }
    let (mut c, a, t) = setup("Moonblast");
    pm(&mut c, &a).selected_ability = "Serene Grace".into();
    let c = pending(&c, &a, &t, "Moonblast");
    assert_eq!(rules::dice_count(&c, &cmd(&c), &catalog()).unwrap(), 5);
    let (mut c, a, t) = setup("Moonblast");
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
        Box::new(completion::initial(ORDINARY_ATTACKS_ENGINE));
    assert!(crate::combat::damaging_stats::definition(&c, "Moonblast").is_none());
    let c = pending(&c, &a, &t, "Moonblast");
    assert_eq!(
        battle::transition(&c, "gm", &cmd(&c), vec![], &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
}
fn support_cast(c: &Campaign, a: &str, targets: Vec<String>, name: &str) -> Campaign {
    let action = super::targeting_tests::attack(a, targets, name);
    let mut c = roll(c, &p(c, a).owner_id, &action, 6);
    while c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
        c = pass(&c);
    }
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
        c = resolve(&c);
    }
    c
}
fn stat(c: &Campaign, id: &str, key: &str) -> i32 {
    special::modifier(c, p(c, id), key).unwrap_or(0)
}
#[test]
fn completion_a08_self_buffs_follow_prose_and_non_ghost_condition() {
    for (name, changes) in [
        (
            "Meditate",
            vec![("Special", 1), ("SpecialDefense", 1), ("Strength", 0)],
        ),
        ("Growth", vec![("Strength", 1), ("Special", 1)]),
        (
            "Curse (Non-Ghost)",
            vec![("Strength", 1), ("Defense", 1), ("Dexterity", -1)],
        ),
    ] {
        let (c, a, _) = setup(name);
        let after = support_cast(&c, &a, vec![a.clone()], name);
        for (key, n) in changes {
            assert_eq!(stat(&after, &a, key), n, "{name}: {key}");
        }
        assert_eq!(
            serde_json::to_value(&c.sheets).unwrap(),
            serde_json::to_value(&after.sheets).unwrap()
        );
    }
    let (mut c, a, _) = setup("Growth");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .natural = Some(crate::combat::field::Weather::Sunny);
    let c = support_cast(&c, &a, vec![a.clone()], "Growth");
    assert_eq!(stat(&c, &a, "Strength"), 2);
    assert_eq!(stat(&c, &a, "Special"), 2);
    let (mut c, a, _) = setup("Curse (Non-Ghost)");
    set_species(&mut c, &a, "Gengar");
    let c = support_cast(&c, &a, vec![a.clone()], "Curse (Non-Ghost)");
    assert_eq!(stat(&c, &a, "Strength"), 0);
}
#[test]
fn completion_a08_type_restricted_allies_and_coaching_exclude_the_user() {
    for (name, key) in [
        ("Gear Up", "Strength"),
        ("Magnetic Flux", "Defense"),
        ("Coaching", "Strength"),
    ] {
        let (mut c, a, t) = setup(name);
        let side = p(&c, &a).side.clone();
        pm(&mut c, &t).side = side;
        set_species(&mut c, &t, "Mawile");
        let targets = if name == "Magnetic Flux" {
            vec![a.clone(), t.clone()]
        } else {
            vec![t.clone()]
        };
        let after = support_cast(&c, &a, targets, name);
        assert_eq!(stat(&after, &t, key), 1, "{name}");
        assert_eq!(stat(&after, &a, key), 0, "{name}");
        if name == "Gear Up" {
            set_species(&mut c, &t, "Eevee");
            let after = support_cast(&c, &a, vec![t.clone()], name);
            assert_eq!(stat(&after, &t, key), 0);
        }
        if name == "Coaching" {
            let action = super::targeting_tests::attack(&a, vec![a.clone()], name);
            assert!(rules::dice_count(&c, &action, &catalog()).is_err());
        }
    }
}
#[test]
fn completion_a08_battlefield_grass_effects_include_both_sides_and_retain_historical_gate() {
    for (name, keys) in [
        ("Flower Shield", vec!["Defense"]),
        ("Rototiller", vec!["Strength", "Special"]),
    ] {
        let (mut c, a, t) = setup(name);
        set_species(&mut c, &a, "Bulbasaur");
        set_species(&mut c, &t, "Oddish");
        let after = support_cast(&c, &a, vec![], name);
        for key in keys {
            assert_eq!(stat(&after, &a, key), 1);
            assert_eq!(stat(&after, &t, key), 1);
        }
        set_species(&mut c, &t, "Eevee");
        let after = support_cast(&c, &a, vec![], name);
        assert_eq!(stat(&after, &t, "Defense"), 0);
        assert_eq!(stat(&after, &t, "Strength"), 0);
    }
    let (mut c, a, _) = setup("Meditate");
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
        Box::new(completion::initial(REMAINING_STAT_ATTACKS_ENGINE));
    assert!(completion::definition(&c, "Meditate").is_none());
    assert!(!completion::excludes_self(&c, "Coaching"));
    assert_eq!(stat(&c, &a, "Special"), 0);
}
fn queued_damage_pool(c: &Campaign) -> rules::DamagePool {
    let mut temp = c.clone();
    let q = q(c);
    pm(&mut temp, &q.actor_id).pending_damage = Some(PendingDamage {
        sequence: q.sequence,
        target_id: q.target_id.clone(),
        move_name: q.move_name.clone(),
        critical: q.critical,
    });
    rules::damage_pool(&temp, &q.actor_id, q.sequence, 0, &catalog()).unwrap()
}
fn ordinary_accuracy(c: &Campaign, a: &str, t: &str, name: &str) -> Campaign {
    let m = catalog().find("moves", "Name", name).unwrap();
    let action = Action::Accuracy {
        combatant_id: a.into(),
        target_id: Some(t.into()),
        move_name: name.into(),
        pool_modifier: 0,
        success_modifier: 1 - m["Attributes"]["AccuracyReduction"].as_i64().unwrap_or(0) as i32,
        reason: "Exactly one success after printed difficulty".into(),
    };
    let mut c = roll(c, "gm", &action, 1);
    if c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
        c = pass(&c);
    }
    c
}
#[test]
fn completion_a09_all_ten_damage_rules_finish_without_manual_effects() {
    for name in [
        "Darkest Lariat",
        "Sacred Sword",
        "Psyshock",
        "Psystrike",
        "Secret Sword",
        "Frost Breath",
        "Storm Throw",
        "Wicked Blow",
        "Flower Trick",
        "Weather Ball",
    ] {
        let (c, a, t) = setup(name);
        let c = ordinary_accuracy(&c, &a, &t, name);
        assert_eq!(q(&c).successes, 1, "{name}");
        let expected =
            ["Frost Breath", "Storm Throw", "Wicked Blow", "Flower Trick"].contains(&name);
        assert_eq!(q(&c).critical, expected, "{name}");
        let pool = queued_damage_pool(&c);
        assert_eq!(pool.critical > 0, expected, "{name}");
        if ["Darkest Lariat", "Sacred Sword"].contains(&name) {
            assert_eq!(pool.defense, 0, "{name}");
        }
        let after = roll(&c, "gm", &damage_command(&c), 4);
        assert!(
            after
                .battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .queue
                .is_empty(),
            "{name}"
        );
        assert_eq!(p(&after, &a).actions, 1);
    }
}
#[test]
fn completion_a09_special_attacks_resist_with_physical_defense_without_category_change() {
    for name in ["Psyshock", "Psystrike", "Secret Sword"] {
        let (mut c, a, t) = setup(name);
        let sid = p(&c, &t).sheet_id.clone();
        c.sheets
            .get_mut(&sid)
            .unwrap()
            .stats
            .insert("Vitality".into(), 5);
        c.sheets
            .get_mut(&sid)
            .unwrap()
            .stats
            .insert("Insight".into(), 1);
        let queued = ordinary_accuracy(&c, &a, &t, name);
        assert_eq!(q(&queued).defense_key, "Defense");
        let pool = queued_damage_pool(&queued);
        let target = rules::effective_sheet(&queued, p(&queued, &t)).unwrap();
        assert_eq!(
            pool.defense,
            crate::creation::profile_stats(&target)["Defense"]
        );
        assert_eq!(
            catalog().find("moves", "Name", name).unwrap()["Category"],
            "Special"
        );
        c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
            Box::new(completion::initial(SUPPORT_STAT_MOVES_ENGINE));
        let old = ordinary_accuracy(&c, &a, &t, name);
        assert_eq!(q(&old).defense_key, "SpecialDefense");
        let oldpool = queued_damage_pool(&old);
        assert_eq!(
            oldpool.defense,
            crate::creation::profile_stats(&target)["SpecialDefense"]
        );
    }
}
#[test]
fn completion_a09_weather_ball_preserves_all_weather_types_and_two_bonus_dice() {
    use crate::combat::field::{self, Weather};
    for (w, expected) in [
        (Weather::Sunny, "Fire"),
        (Weather::Rain, "Water"),
        (Weather::Snow, "Ice"),
        (Weather::Hail, "Ice"),
        (Weather::Sandstorm, "Rock"),
        (Weather::StrongWinds, "Flying"),
    ] {
        let (mut c, a, t) = setup("Weather Ball");
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .battlefield
            .natural = Some(w);
        let m = catalog().find("moves", "Name", "Weather Ball").unwrap();
        let resolved = field::move_data(&c, p(&c, &a), &m);
        assert_eq!(resolved["Type"], expected);
        let mut control = resolved.clone();
        control["Name"] = "Weather control".into();
        assert_eq!(
            field::bonus(&c, p(&c, &a), &resolved) - field::bonus(&c, p(&c, &a), &control),
            2
        );
        let c = pending(&c, &a, &t, "Weather Ball");
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
fn completion_a10_all_eleven_losses_use_exact_chance_and_remove_successes_not_dice() {
    let r: serde_json::Value =
        serde_json::from_str(include_str!("../../../rules/move-completion.json")).unwrap();
    for (name, d) in r["blocks"]["MOVE-A10"]["moves"].as_object().unwrap() {
        let (c, a, t) = setup(name);
        let c = pending(&c, &a, &t, name);
        let n = rules::dice_count(&c, &cmd(&c), &catalog()).unwrap();
        assert_eq!(n, d["dice"].as_u64().unwrap() as u32, "{name}");
        for face in [5, 6] {
            let after = roll(&c, "gm", &cmd(&c), face);
            let loss = completion::accuracy_loss(&after, &t);
            assert_eq!(
                loss,
                if n == 0 || face == 6 {
                    d["amount"].as_i64().unwrap() as i32
                } else {
                    0
                },
                "{name}"
            );
            assert_eq!(p(&after, &t).hp, p(&c, &t).hp);
            let mut affected = after.clone();
            turn(&mut affected, &t, "Tackle");
            let mut control = affected.clone();
            control
                .battle
                .as_mut()
                .unwrap()
                .flow
                .as_mut()
                .unwrap()
                .special
                .effects
                .retain(|e| !e.name.starts_with("accuracy-loss:"));
            let action = super::targeting_tests::attack(&t, vec![a.clone()], "Tackle");
            assert_eq!(
                rules::dice_count(&affected, &action, &catalog()).unwrap(),
                rules::dice_count(&control, &action, &catalog()).unwrap(),
                "{name}: pool dice unchanged"
            );
            let outcome = ordinary_accuracy(&affected, &t, &a, "Tackle");
            assert_eq!(
                outcome
                    .battle
                    .as_ref()
                    .unwrap()
                    .flow
                    .as_ref()
                    .unwrap()
                    .queue
                    .is_empty(),
                loss > 0,
                "{name}: exactly one success"
            );
        }
    }
}
#[test]
fn completion_a10_losses_do_not_stack_and_keen_eye_protection_remains_live() {
    let (c, a, t) = setup("Sand Attack");
    let mut c = resolve(&pending(&c, &a, &t, "Sand Attack"));
    turn(&mut c, &a, "Mud Slap");
    let mut c = resolve(&pending(&c, &a, &t, "Mud Slap"));
    assert_eq!(completion::accuracy_loss(&c, &t), 1);
    turn(&mut c, &a, "Spacial Rend");
    let mut c = resolve(&pending(&c, &a, &t, "Spacial Rend"));
    assert_eq!(completion::accuracy_loss(&c, &t), 5);
    pm(&mut c, &t).selected_ability = "Keen Eye".into();
    turn(&mut c, &t, "Tackle");
    let after = ordinary_accuracy(&c, &t, &a, "Tackle");
    assert!(!after
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .is_empty());
    let saved: Campaign = serde_json::from_value(serde_json::to_value(&c).unwrap()).unwrap();
    assert_eq!(completion::accuracy_loss(&saved, &t), 5);
    pm(&mut c, &t).selected_ability.clear();
    pm(&mut c, &t).deployed = false;
    special::lifecycle(&saved, &mut c, &catalog()).unwrap();
    assert_eq!(completion::accuracy_loss(&c, &t), 0);
}
#[test]
fn completion_a10_reflection_shield_dust_chance_modifiers_and_historical_gate() {
    let (mut c, a, t) = setup("Mud Slap");
    pm(&mut c, &t).selected_ability = "Mirror Armor".into();
    let c = resolve(&pending(&c, &a, &t, "Mud Slap"));
    assert_eq!(completion::accuracy_loss(&c, &a), 1);
    assert_eq!(completion::accuracy_loss(&c, &t), 0);
    let (mut c, a, t) = setup("Leaf Tornado");
    pm(&mut c, &t).selected_ability = "Shield Dust".into();
    let c = resolve(&pending(&c, &a, &t, "Leaf Tornado"));
    assert_eq!(completion::accuracy_loss(&c, &t), 0);
    let (mut c, a, t) = setup("Leaf Tornado");
    pm(&mut c, &a).selected_ability = "Serene Grace".into();
    pm(&mut c, &t).selected_ability = "Wonder Skin".into();
    let c = pending(&c, &a, &t, "Leaf Tornado");
    assert_eq!(rules::dice_count(&c, &cmd(&c), &catalog()).unwrap(), 5);
    let (mut c, a, t) = setup("Sand Attack");
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
        Box::new(completion::initial(DAMAGE_RULE_ATTACKS_ENGINE));
    let c = pending(&c, &a, &t, "Sand Attack");
    assert_eq!(
        battle::transition(&c, "gm", &cmd(&c), vec![], &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
}
#[test]
fn completion_a10_clear_amulet_sheer_force_and_independent_area_chance() {
    let (mut c, a, t) = setup("Mud Slap");
    let sid = p(&c, &t).sheet_id.clone();
    let mut s = c.sheets[&sid].clone();
    s.inventory.insert("Clear Amulet".into(), 1);
    c.sheets.insert(
        sid,
        crate::combat::items::equip(&s, "Clear Amulet", "accuracy-amulet", &catalog()).unwrap(),
    );
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.items.enabled = true;
    f.items.completion.enabled = true;
    crate::combat::items::initialize(&mut c, &catalog()).unwrap();
    crate::combat::items::completion::initialize(&mut c, &catalog());
    let c = resolve(&pending(&c, &a, &t, "Mud Slap"));
    assert_eq!(completion::accuracy_loss(&c, &t), 0);
    let (mut c, a, t) = setup("Spacial Rend");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .provenance_rules = true;
    pm(&mut c, &a).selected_ability = "Sheer Force".into();
    let c = decision(
        &c,
        "gm",
        Action::ChooseSheerForce {
            combatant_id: a.clone(),
            ignore_added_effects: true,
            reason: "Retain damage instead of accuracy loss".into(),
        },
    );
    let c = pending(&c, &a, &t, "Spacial Rend");
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .is_empty());
    assert_eq!(completion::accuracy_loss(&c, &t), 0);
    let (mut c, a, t) = setup("Muddy Water");
    let b = c.battle.as_mut().unwrap();
    let mut third = b.combatants[1].clone();
    third.id = id();
    let other = third.id.clone();
    b.combatants.push(third);
    b.flow.as_mut().unwrap().tactics.enabled = true;
    let action = super::targeting_tests::attack(&a, vec![t.clone(), other.clone()], "Muddy Water");
    let mut c = roll(&c, "gm", &action, 6);
    c = pass(&pass(&c));
    let first = q(&c).target_id;
    for face in [5, 6] {
        c = roll(&c, "gm", &damage_command(&c), 1);
        c = roll(&c, "gm", &cmd(&c), face);
    }
    assert_eq!(completion::accuracy_loss(&c, &first), 0);
    assert_eq!(
        completion::accuracy_loss(&c, if first == t { &other } else { &t }),
        1
    );
}
#[test]
fn completion_a10_tactics_reaction_uses_success_loss_and_live_keen_eye() {
    for keen in [false, true] {
        let (mut c, a, t) = setup("Sand Attack");
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .tactics
            .enabled = true;
        let mut c = resolve(&pending(&c, &a, &t, "Sand Attack"));
        turn(&mut c, &a, "Tackle");
        let sid = p(&c, &t).sheet_id.clone();
        c.sheets
            .get_mut(&sid)
            .unwrap()
            .moves
            .push("Quick Attack".into());
        if keen {
            pm(&mut c, &t).selected_ability = "Keen Eye".into();
        }
        let incoming = Action::Accuracy {
            combatant_id: a.clone(),
            target_id: Some(t.clone()),
            move_name: "Tackle".into(),
            pool_modifier: 0,
            success_modifier: 1,
            reason: "Incoming move with one success".into(),
        };
        let c = roll(&c, "gm", &incoming, 1);
        let reaction = Action::Tactics {
            command: crate::combat::tactics::TacticsCommand::Reaction {
                combatant_id: t.clone(),
                move_name: "Quick Attack".into(),
                target_ids: vec![a.clone()],
            },
            reason: "One-success move reaction".into(),
        };
        let n = rules::dice_count(&c, &reaction, &catalog()).unwrap();
        let mut faces = vec![1; n as usize];
        faces[0] = 6;
        let after = battle::transition(&c, "gm", &reaction, faces, &catalog())
            .unwrap()
            .0;
        assert_eq!(
            after
                .battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .queue
                .iter()
                .any(|q| q.actor_id == t),
            keen
        );
    }
}
#[test]
fn completion_a11_all_four_explicit_stacks_stop_at_three_and_keep_chance_pools() {
    let r: serde_json::Value =
        serde_json::from_str(include_str!("../../../rules/move-completion.json")).unwrap();
    for (name, d) in r["blocks"]["MOVE-A11"]["moves"].as_object().unwrap() {
        let (mut c, a, t) = setup(name);
        let target = if d["user"] == true {
            a.clone()
        } else {
            t.clone()
        };
        if d["dice"].as_u64().unwrap() > 0 {
            let queued = pending(&c, &a, &target, name);
            assert_eq!(
                rules::dice_count(&queued, &cmd(&queued), &catalog()).unwrap(),
                5
            );
            c = roll(&queued, "gm", &cmd(&queued), 5);
            assert_eq!(stat(&c, &target, d["keys"][0].as_str().unwrap()), 0);
            turn(&mut c, &a, name);
        }
        for cast in 1..=4 {
            c = resolve(&pending(&c, &a, &target, name));
            assert_eq!(
                stat(&c, &target, d["keys"][0].as_str().unwrap()),
                cast.min(3) * d["amount"].as_i64().unwrap() as i32,
                "{name} cast {cast}"
            );
            turn(&mut c, &a, name);
        }
    }
}
#[test]
fn completion_a11_stacking_protections_reflection_and_each_simple_increment() {
    for (ability, multiplier) in [("Simple", -2), ("Contrary", 1), ("Mirror Armor", -1)] {
        let (mut c, a, t) = setup("Syrup Bomb");
        pm(&mut c, &t).selected_ability = ability.into();
        for cast in 1..=3 {
            c = resolve(&pending(&c, &a, &t, "Syrup Bomb"));
            let recipient = if ability == "Mirror Armor" { &a } else { &t };
            assert_eq!(
                stat(&c, recipient, "Dexterity"),
                cast * multiplier,
                "{ability} cast {cast}"
            );
            if ability == "Mirror Armor" {
                assert_eq!(stat(&c, &t, "Dexterity"), 0);
            }
            turn(&mut c, &a, "Syrup Bomb");
        }
    }
    let (mut c, a, t) = setup("Syrup Bomb");
    pm(&mut c, &t).selected_ability = "Clear Body".into();
    c = resolve(&pending(&c, &a, &t, "Syrup Bomb"));
    assert_eq!(stat(&c, &t, "Dexterity"), 0);
    pm(&mut c, &t).selected_ability.clear();
    turn(&mut c, &a, "Syrup Bomb");
    c = resolve(&pending(&c, &a, &t, "Syrup Bomb"));
    assert_eq!(stat(&c, &t, "Dexterity"), -1);
}
#[test]
fn completion_a11_save_and_clear_preserve_named_counters_without_changing_historical_states() {
    let (c, a, t) = setup("Syrup Bomb");
    let c = resolve(&pending(&c, &a, &t, "Syrup Bomb"));
    let mut saved: Campaign = serde_json::from_value(serde_json::to_value(&c).unwrap()).unwrap();
    turn(&mut saved, &a, "Syrup Bomb");
    let mut saved = resolve(&pending(&saved, &a, &t, "Syrup Bomb"));
    assert_eq!(stat(&saved, &t, "Dexterity"), -2);
    special::clear_modifiers(&mut saved, &t);
    pm(&mut saved, &t).runtime.modifiers.clear();
    turn(&mut saved, &a, "Syrup Bomb");
    let saved = resolve(&pending(&saved, &a, &t, "Syrup Bomb"));
    assert_eq!(stat(&saved, &t, "Dexterity"), -1);
    let (mut c, a, t) = setup("Syrup Bomb");
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
        Box::new(completion::initial(ACCURACY_LOSS_MOVES_ENGINE));
    let c = pending(&c, &a, &t, "Syrup Bomb");
    assert_eq!(
        battle::transition(&c, "gm", &cmd(&c), vec![], &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
}
#[test]
fn completion_a11_stat_transfer_captures_effective_simple_stacks_once() {
    let (mut c, a, t) = setup("Sharpen");
    pm(&mut c, &a).selected_ability = "Simple".into();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .stat_channels
        .enabled = true;
    for _ in 0..3 {
        c = resolve(&pending(&c, &a, &a, "Sharpen"));
        turn(&mut c, &a, "Sharpen");
    }
    assert_eq!(stat(&c, &a, "Strength"), 6);
    let side = p(&c, &a).side.clone();
    pm(&mut c, &t).side = side;
    crate::combat::stat_channels::transfer(&mut c, &a, &t, "Baton Pass").unwrap();
    assert_eq!(stat(&c, &t, "Strength"), 6);
}
fn saved_field_rule(
    c: &mut Campaign,
    kind: &str,
    source: &str,
    scope: &str,
    rounds: Option<u32>,
    parameter: i32,
) {
    let started_round = c.battle.as_ref().unwrap().round;
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .rules
        .push(crate::combat::field::FieldRule {
            id: id(),
            kind: kind.into(),
            source: source.into(),
            scope: scope.into(),
            rounds,
            parameter,
            started_round,
        });
}
#[test]
fn completion_a12_rapid_spin_cleans_own_side_and_ongoing_without_removing_scene_blocks() {
    use crate::combat::field;
    let (c, a, t) = setup("Bind");
    let mut c = resolve(&pending(&c, &a, &t, "Bind"));
    turn(&mut c, &a, "Mean Look");
    let mut c = resolve(&pending(&c, &a, &t, "Mean Look"));
    let own = format!("side:{}", p(&c, &t).side);
    let other = format!("side:{}", p(&c, &a).side);
    saved_field_rule(&mut c, "Spikes", "spin-fixture", "all", None, 0);
    saved_field_rule(&mut c, "Sticky Web", "spin-fixture", &own, None, 0);
    saved_field_rule(&mut c, "Stealth Rock", "spin-fixture", &other, None, 0);
    saved_field_rule(&mut c, "Grassy Terrain", "spin-fixture", "all", Some(4), 0);
    saved_field_rule(&mut c, "Reflect", "spin-fixture", &own, Some(4), 0);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .special
        .effects
        .push(special::TimedEffect {
            name: "Leech Seed".into(),
            source: a.clone(),
            target: t.clone(),
            rounds: None,
            amount: 2,
            started: 0,
            slot: String::new(),
        });
    turn(&mut c, &t, "Rapid Spin");
    let c = resolve(&pending(&c, &t, &a, "Rapid Spin"));
    assert_eq!(stat(&c, &t, "Dexterity"), 1);
    assert!(completion::blocked(&c, &t), "Mean Look remains");
    assert!(!c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .special
        .effects
        .iter()
        .any(|e| e.target == t && (e.name.starts_with("ongoing:") || e.name == "Leech Seed")));
    assert!(!field::has(&c, "Spikes", p(&c, &t)));
    assert!(!field::has(&c, "Sticky Web", p(&c, &t)));
    assert!(
        field::has(&c, "Spikes", p(&c, &a)),
        "global hazard retained on other side"
    );
    assert!(field::has(&c, "Stealth Rock", p(&c, &a)));
    assert!(!field::has(&c, "Grassy Terrain", p(&c, &t)));
    assert!(field::has(&c, "Grassy Terrain", p(&c, &a)));
    assert!(field::has(&c, "Reflect", p(&c, &t)));
}
#[test]
fn completion_a12_terrain_attacks_validate_declaration_and_remove_all_terrain_scopes() {
    use crate::combat::field;
    for name in ["Ice Spinner", "Steel Roller"] {
        let (mut c, a, t) = setup(name);
        let action = super::targeting_tests::attack(&a, vec![t.clone()], name);
        assert_eq!(
            rules::dice_count(&c, &action, &catalog()).is_err(),
            name == "Steel Roller"
        );
        let own = format!("side:{}", p(&c, &a).side);
        let other = format!("side:{}", p(&c, &t).side);
        saved_field_rule(
            &mut c,
            "Grassy Terrain",
            "terrain-fixture",
            &own,
            Some(4),
            0,
        );
        saved_field_rule(
            &mut c,
            "Electric Terrain",
            "terrain-fixture",
            &other,
            Some(3),
            0,
        );
        saved_field_rule(&mut c, "Reflect", "terrain-fixture", &own, Some(2), 0);
        let c = resolve(&pending(&c, &a, &t, name));
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
            .any(|r| r.kind.ends_with("Terrain")));
        assert!(field::has(&c, "Reflect", p(&c, &a)));
    }
}
#[test]
fn completion_a12_terrain_cleanup_survives_target_faint_and_old_encounters_keep_fallback() {
    use crate::combat::field;
    let (mut c, a, t) = setup("Ice Spinner");
    pm(&mut c, &t).hp = 1;
    saved_field_rule(
        &mut c,
        "Grassy Terrain",
        "terrain-fixture",
        "all",
        Some(4),
        0,
    );
    let c = pending(&c, &a, &t, "Ice Spinner");
    assert_eq!(p(&c, &t).hp, 0);
    let c = resolve(&c);
    assert!(!field::has(&c, "Grassy Terrain", p(&c, &a)));
    let (mut c, a, t) = setup("Ice Spinner");
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
        Box::new(completion::initial(STACK_STAT_MOVES_ENGINE));
    let c = pending(&c, &a, &t, "Ice Spinner");
    assert_eq!(
        battle::transition(&c, "gm", &cmd(&c), vec![], &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
}
#[test]
fn completion_a12_immunity_and_sheer_force_preserve_terrain_when_effect_is_blocked() {
    use crate::combat::field;
    let (mut c, a, t) = setup("Ice Spinner");
    pm(&mut c, &t).selected_ability = "Wonder Guard".into();
    saved_field_rule(
        &mut c,
        "Grassy Terrain",
        "terrain-fixture",
        "all",
        Some(4),
        0,
    );
    let c = resolve(&pending(&c, &a, &t, "Ice Spinner"));
    assert!(field::has(&c, "Grassy Terrain", p(&c, &a)));
    let (mut c, a, t) = setup("Ice Spinner");
    pm(&mut c, &a).selected_ability = "Sheer Force".into();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .provenance_rules = true;
    saved_field_rule(
        &mut c,
        "Grassy Terrain",
        "terrain-fixture",
        "all",
        Some(4),
        0,
    );
    let c = decision(
        &c,
        "gm",
        Action::ChooseSheerForce {
            combatant_id: a.clone(),
            ignore_added_effects: true,
            reason: "Trade terrain removal for damage".into(),
        },
    );
    let c = pending(&c, &a, &t, "Ice Spinner");
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .is_empty());
    assert!(field::has(&c, "Grassy Terrain", p(&c, &a)));
}
fn refresh_field(c: &mut Campaign) {
    let before = c.clone();
    crate::combat::field::lifecycle(&before, c, &catalog()).unwrap();
}
#[test]
fn completion_a13_source_type_attacks_use_current_main_or_secondary_type_with_stab() {
    use crate::combat::field;
    for name in [
        "Ivy Cudgel",
        "Raging Bull",
        "Multi-Attack",
        "Revelation Dance",
    ] {
        let (mut c, a, t) = setup(name);
        set_species(&mut c, &a, "Charizard");
        set_species(&mut c, &t, "Mawile");
        refresh_field(&mut c);
        let secondary = ["Ivy Cudgel", "Raging Bull"].contains(&name);
        let m = catalog().find("moves", "Name", name).unwrap();
        let resolved = field::move_data(&c, p(&c, &a), &m);
        assert_eq!(
            resolved["Type"],
            if secondary { "Flying" } else { "Fire" },
            "{name}"
        );
        let queued = ordinary_accuracy(&c, &a, &t, name);
        let pool = queued_damage_pool(&queued);
        assert_eq!(pool.stab, 1, "{name}");
        assert_eq!(pool.effectiveness, if secondary { -1 } else { 1 }, "{name}");
        let after = roll(&queued, "gm", &damage_command(&queued), 1);
        assert!(after
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .queue
            .is_empty());
        set_species(&mut c, &a, "Eevee");
        refresh_field(&mut c);
        assert_eq!(
            field::move_data(&c, p(&c, &a), &m)["Type"],
            "Normal",
            "{name} single-type fallback"
        );
        let command = Action::Battlefield {
            command: field::FieldCommand::Typing {
                combatant_id: a.clone(),
                mode: "replace".into(),
                types: vec!["Water".into(), "Electric".into()],
            },
            reason: "Confirm current battle typing".into(),
        };
        let c = decision(&c, "gm", command);
        assert_eq!(
            field::move_data(&c, p(&c, &a), &m)["Type"],
            if secondary { "Electric" } else { "Water" }
        );
    }
}
#[test]
fn completion_a13_terrain_pulse_and_psyblade_use_exact_context_bonuses() {
    use crate::combat::field;
    for (terrain, tpe) in [
        ("Electric Terrain", "Electric"),
        ("Grassy Terrain", "Grass"),
        ("Misty Terrain", "Fairy"),
        ("Psychic Terrain", "Psychic"),
    ] {
        let (mut c, a, t) = setup("Terrain Pulse");
        let scope = format!("side:{}", p(&c, &a).side);
        saved_field_rule(&mut c, terrain, "context-fixture", &scope, Some(4), 0);
        refresh_field(&mut c);
        let m = catalog().find("moves", "Name", "Terrain Pulse").unwrap();
        let m = field::move_data(&c, p(&c, &a), &m);
        assert_eq!(m["Type"], tpe);
        let mut control = m.clone();
        control["Name"] = "Terrain control".into();
        assert_eq!(
            field::bonus(&c, p(&c, &a), &m) - field::bonus(&c, p(&c, &a), &control),
            2
        );
        let after = pending(&c, &a, &t, "Terrain Pulse");
        assert!(after
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .queue
            .is_empty());
    }
    let (mut c, a, t) = setup("Psyblade");
    let m = catalog().find("moves", "Name", "Psyblade").unwrap();
    assert_eq!(completion::damage_bonus(&c, p(&c, &a), &m), 0);
    let scope = format!("side:{}", p(&c, &t).side);
    saved_field_rule(
        &mut c,
        "Electric Terrain",
        "context-fixture",
        &scope,
        Some(4),
        0,
    );
    assert_eq!(
        completion::damage_bonus(&c, p(&c, &a), &m),
        0,
        "opposing terrain does not reach user"
    );
    saved_field_rule(
        &mut c,
        "Electric Terrain",
        "context-fixture",
        "all",
        Some(4),
        0,
    );
    assert_eq!(completion::damage_bonus(&c, p(&c, &a), &m), 2);
    let after = pending(&c, &a, &t, "Psyblade");
    assert!(after
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
fn completion_a13_acrobatics_uses_held_item_presence_and_old_encounters_keep_original_type() {
    use crate::combat::{field, items};
    let (mut c, a, t) = setup("Acrobatics");
    let m = catalog().find("moves", "Name", "Acrobatics").unwrap();
    assert_eq!(completion::damage_bonus(&c, p(&c, &a), &m), 0);
    let sid = p(&c, &a).sheet_id.clone();
    let mut s = c.sheets[&sid].clone();
    s.inventory.insert("Leftovers".into(), 1);
    c.sheets.insert(
        sid,
        items::equip(&s, "Leftovers", "acrobatics-item", &catalog()).unwrap(),
    );
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.items.enabled = true;
    f.items.completion.enabled = true;
    items::initialize(&mut c, &catalog()).unwrap();
    items::completion::initialize(&mut c, &catalog());
    assert_eq!(completion::damage_bonus(&c, p(&c, &a), &m), -2);
    let after = pending(&c, &a, &t, "Acrobatics");
    assert!(after
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .is_empty());
    let (mut c, a, _) = setup("Revelation Dance");
    set_species(&mut c, &a, "Charizard");
    refresh_field(&mut c);
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
        Box::new(completion::initial(FIELD_CLEANSE_MOVES_ENGINE));
    let m = catalog().find("moves", "Name", "Revelation Dance").unwrap();
    assert_eq!(field::move_data(&c, p(&c, &a), &m)["Type"], m["Type"]);
}
