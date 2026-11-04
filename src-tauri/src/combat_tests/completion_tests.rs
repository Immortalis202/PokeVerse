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
