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
    let round = c.battle.as_ref().unwrap().round;
    let entered = c
        .battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .filter(|p| p.deployed)
        .map(|p| (p.id.clone(), round))
        .collect();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .completion
        .entered = entered;
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
    f.started = true;
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
#[test]
fn completion_a14_self_damage_uses_exact_half_hp_dice_and_flat_damage() {
    for (name, hp, max_hp, dice, loss, keys, amount) in [
        ("Belly Drum", 9, 21, 4, 4, vec!["Strength"], 3),
        (
            "Clangorous Soul",
            9,
            21,
            10,
            10,
            vec![
                "Strength",
                "Dexterity",
                "Special",
                "Defense",
                "SpecialDefense",
            ],
            1,
        ),
        (
            "Fillet Away",
            9,
            21,
            0,
            2,
            vec!["Strength", "Dexterity", "Special"],
            2,
        ),
    ] {
        let (mut c, a, _) = setup(name);
        pm(&mut c, &a).hp = hp;
        pm(&mut c, &a).max_hp = max_hp;
        // Keep the actor conscious to inspect buffs when the whole self-damage pool succeeds.
        if name == "Clangorous Soul" {
            pm(&mut c, &a).hp = 15;
        }
        let c = pending(&c, &a, &a, name);
        let hp = p(&c, &a).hp;
        assert_eq!(
            rules::dice_count(&c, &cmd(&c), &catalog()).unwrap(),
            dice,
            "{name}"
        );
        assert!(
            battle::transition(&c, "gm", &cmd(&c), vec![6; dice as usize + 1], &catalog()).is_err()
        );
        let after = roll(&c, "gm", &cmd(&c), 4);
        assert_eq!(p(&after, &a).hp, (hp - loss).max(0), "{name}");
        for key in keys {
            assert_eq!(stat(&after, &a, key), amount, "{name} {key}");
        }
        assert_eq!(
            serde_json::to_value(&c.sheets).unwrap(),
            serde_json::to_value(&after.sheets).unwrap()
        );
    }
    let (mut c, a, _) = setup("Belly Drum");
    pm(&mut c, &a).hp = 1;
    let c = pending(&c, &a, &a, "Belly Drum");
    assert_eq!(rules::dice_count(&c, &cmd(&c), &catalog()).unwrap(), 0);
    let c = resolve(&c);
    assert_eq!(p(&c, &a).hp, 1);
    assert_eq!(stat(&c, &a, "Strength"), 3);
}
#[test]
fn completion_a14_self_damage_is_not_chance_and_retains_indirect_protection() {
    let (mut c, a, _) = setup("Belly Drum");
    pm(&mut c, &a).selected_ability = "Serene Grace".into();
    pm(&mut c, &a).hp = 9;
    let c = pending(&c, &a, &a, "Belly Drum");
    assert_eq!(rules::dice_count(&c, &cmd(&c), &catalog()).unwrap(), 4);
    let after = roll(&c, "gm", &cmd(&c), 1);
    assert_eq!(p(&after, &a).hp, 9);
    assert_eq!(stat(&after, &a, "Strength"), 3);
    for name in ["Belly Drum", "Clangorous Soul", "Fillet Away"] {
        let (mut c, a, _) = setup(name);
        pm(&mut c, &a).selected_ability = "Magic Guard".into();
        let c = resolve(&pending(&c, &a, &a, name));
        assert_eq!(p(&c, &a).hp, 100, "{name}");
        assert!(stat(&c, &a, "Strength") > 0);
    }
}
#[test]
fn completion_a14_no_retreat_and_octolock_block_independently_of_stat_protection() {
    let (c, a, _) = setup("No Retreat");
    let c = resolve(&pending(&c, &a, &a, "No Retreat"));
    assert!(completion::blocked(&c, &a));
    for key in [
        "Strength",
        "Dexterity",
        "Special",
        "Defense",
        "SpecialDefense",
    ] {
        assert_eq!(stat(&c, &a, key), 1);
    }
    for ability in ["", "Clear Body", "Run Away"] {
        let (mut c, a, t) = setup("Octolock");
        pm(&mut c, &t).selected_ability = ability.into();
        let c = resolve(&pending(&c, &a, &t, "Octolock"));
        assert_eq!(completion::blocked(&c, &t), ability != "Run Away");
        for key in ["Defense", "SpecialDefense"] {
            assert_eq!(
                stat(&c, &t, key),
                if ability == "Clear Body" { 0 } else { -1 }
            );
        }
    }
    let (mut c, a, _) = setup("Belly Drum");
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
        Box::new(completion::initial(CONTEXT_DAMAGE_MOVES_ENGINE));
    let c = pending(&c, &a, &a, "Belly Drum");
    assert_eq!(
        battle::transition(&c, "gm", &cmd(&c), vec![], &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
}
#[test]
fn completion_a15_taunt_blocks_support_moves_preserves_maneuvers_and_expires_after_four_rounds() {
    let (c, a, t) = setup("Taunt");
    let mut c = resolve(&pending(&c, &a, &t, "Taunt"));
    for _ in 0..4 {
        turn(&mut c, &t, "Rest");
        let support = super::targeting_tests::attack(&t, vec![t.clone()], "Rest");
        assert!(rules::dice_count(&c, &support, &catalog()).is_err());
        let attack = super::targeting_tests::attack(&t, vec![a.clone()], "Tackle");
        rules::dice_count(&c, &attack, &catalog()).expect("Taunt must permit Tackle");
        turn(&mut c, &t, "Evasion");
        let maneuver = super::targeting_tests::attack(&t, vec![t.clone()], "Evasion");
        rules::dice_count(&c, &maneuver, &catalog()).expect("Taunt must permit Evasion");
        c = round_end(&c);
    }
    turn(&mut c, &t, "Rest");
    assert!(rules::dice_count(
        &c,
        &super::targeting_tests::attack(&t, vec![t.clone()], "Rest"),
        &catalog()
    )
    .is_ok());
}
#[test]
fn completion_a15_throat_chop_blocks_sound_and_survives_public_switching_until_faint() {
    let (mut c, a, t) = setup("Throat Chop");
    let incoming = reserve(&mut c, &t);
    let mut c = resolve(&pending(&c, &a, &t, "Throat Chop"));
    turn(&mut c, &t, "Hyper Voice");
    let sound = super::targeting_tests::attack(&t, vec![a.clone()], "Hyper Voice");
    assert!(rules::dice_count(&c, &sound, &catalog()).is_err());
    let attack = super::targeting_tests::attack(&t, vec![a.clone()], "Tackle");
    assert!(rules::dice_count(&c, &attack, &catalog()).is_ok());
    let c = decision(
        &c,
        "gm",
        Action::SwitchPokemon {
            outgoing_id: t.clone(),
            incoming_id: incoming.clone(),
            gm_override: false,
            reason: "Switch while silenced".into(),
        },
    );
    assert!(!p(&c, &t).deployed);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .special
        .effects
        .iter()
        .any(|e| e.name == "Throat Chop" && e.target == t));
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
        .any(|e| e.name == "Throat Chop" && e.target == incoming));
    let mut saved: Campaign = serde_json::from_value(serde_json::to_value(&c).unwrap()).unwrap();
    pm(&mut saved, &t).hp = 0;
    special::lifecycle(&c, &mut saved, &catalog()).unwrap();
    assert!(!saved
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .special
        .effects
        .iter()
        .any(|e| e.name == "Throat Chop"));
}
#[test]
fn completion_a15_imprison_uses_current_caster_moves_and_ends_on_caster_departure() {
    let (c, a, t) = setup("Imprison");
    let mut c = resolve(&pending(&c, &a, &a, "Imprison"));
    turn(&mut c, &t, "Tackle");
    let attack = super::targeting_tests::attack(&t, vec![a.clone()], "Tackle");
    assert!(rules::dice_count(&c, &attack, &catalog()).is_err());
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .moves
        .retain(|n| n != "Tackle");
    assert!(rules::dice_count(&c, &attack, &catalog()).is_ok());
    c.sheets.get_mut(&sid).unwrap().moves.push("Tackle".into());
    assert!(rules::dice_count(&c, &attack, &catalog()).is_err());
    let before = c.clone();
    pm(&mut c, &a).deployed = false;
    special::lifecycle(&before, &mut c, &catalog()).unwrap();
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
        .any(|e| e.name == "Imprison"));
}
#[test]
fn completion_a15_protection_gm_clearing_and_historical_states_keep_authority() {
    let (mut c, a, t) = setup("Taunt");
    pm(&mut c, &t).selected_ability = "Aroma Veil".into();
    let c = resolve(&pending(&c, &a, &t, "Taunt"));
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
        .any(|e| e.name == "Taunt"));
    let (c, a, t) = setup("Taunt");
    let c = resolve(&pending(&c, &a, &t, "Taunt"));
    let clear = Action::SetStatus {
        combatant_id: t.clone(),
        kind: "Taunt".into(),
        detail: String::new(),
        rounds: None,
        remove: true,
        reason: "GM ends Taunt".into(),
    };
    assert!(battle::transition(&c, "intruder", &clear, vec![], &catalog()).is_err());
    let mut c = decision(&c, "gm", clear);
    turn(&mut c, &t, "Rest");
    assert!(rules::dice_count(
        &c,
        &super::targeting_tests::attack(&t, vec![t.clone()], "Rest"),
        &catalog()
    )
    .is_ok());
    let (mut c, a, t) = setup("Taunt");
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
        Box::new(completion::initial(COST_STAT_MOVES_ENGINE));
    let c = pending(&c, &a, &t, "Taunt");
    assert_eq!(
        battle::transition(&c, "gm", &cmd(&c), vec![], &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
}
#[test]
fn completion_a16_previous_round_limits_record_public_attempts_and_allow_after_a_gap() {
    for name in ["Blood Moon", "Gigaton Hammer"] {
        let (c, a, t) = setup(name);
        let c = pending(&c, &a, &t, name);
        assert!(c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .queue
            .is_empty());
        assert!(c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .completion
            .uses
            .iter()
            .any(|u| u.actor == a && u.name == name && u.round == 1));
        let mut next = round_end(&c);
        turn(&mut next, &a, name);
        let action = super::targeting_tests::attack(&a, vec![t.clone()], name);
        assert!(
            rules::dice_count(&next, &action, &catalog()).is_err(),
            "{name}"
        );
        let mut next = round_end(&next);
        turn(&mut next, &a, name);
        assert!(
            rules::dice_count(&next, &action, &catalog()).is_ok(),
            "{name}"
        );
    }
}
#[test]
fn completion_a16_failed_clash_counts_as_using_blood_moon() {
    let (mut c, a, t) = setup("Tackle");
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .moves
        .push("Blood Moon".into());
    let action = super::targeting_tests::attack(&a, vec![t.clone()], "Tackle");
    let c = roll(&c, "gm", &action, 6);
    let clash = Action::React {
        combatant_id: t.clone(),
        kind: "clash".into(),
        move_name: "Blood Moon".into(),
        reason: "Attempt Blood Moon Clash".into(),
    };
    let c = roll(&c, "gm", &clash, 1);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .completion
        .uses
        .iter()
        .any(|u| u.actor == t && u.name == "Blood Moon"));
    let c = roll(&c, "gm", &damage_command(&c), 1);
    let mut c = round_end(&c);
    turn(&mut c, &t, "Blood Moon");
    assert!(rules::dice_count(
        &c,
        &super::targeting_tests::attack(&t, vec![a.clone()], "Blood Moon"),
        &catalog()
    )
    .is_err());
}
#[test]
fn completion_a16_disable_selects_only_last_move_and_replaces_the_previous_disable() {
    let (mut c, a, t) = setup("Disable");
    let action = super::targeting_tests::attack(&a, vec![t.clone()], "Disable");
    assert!(rules::dice_count(&c, &action, &catalog()).is_err());
    turn(&mut c, &t, "Tackle");
    let mut c = pending(&c, &t, &a, "Tackle");
    turn(&mut c, &a, "Disable");
    let mut c = resolve(&pending(&c, &a, &t, "Disable"));
    turn(&mut c, &t, "Tackle");
    assert!(rules::dice_count(
        &c,
        &super::targeting_tests::attack(&t, vec![a.clone()], "Tackle"),
        &catalog()
    )
    .is_err());
    turn(&mut c, &t, "Quick Attack");
    let mut c = pending(&c, &t, &a, "Quick Attack");
    turn(&mut c, &a, "Disable");
    let mut c = resolve(&pending(&c, &a, &t, "Disable"));
    let effects = &c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .special
        .effects;
    assert_eq!(
        effects
            .iter()
            .filter(|e| e.target == t && e.name.starts_with("Disable:"))
            .count(),
        1
    );
    assert!(effects
        .iter()
        .any(|e| e.target == t && e.name == "Disable:Quick Attack"));
    turn(&mut c, &t, "Tackle");
    assert!(rules::dice_count(
        &c,
        &super::targeting_tests::attack(&t, vec![a.clone()], "Tackle"),
        &catalog()
    )
    .is_ok());
    turn(&mut c, &t, "Quick Attack");
    assert!(rules::dice_count(
        &c,
        &super::targeting_tests::attack(&t, vec![a.clone()], "Quick Attack"),
        &catalog()
    )
    .is_err());
}
#[test]
fn completion_a16_torment_tracks_the_previous_round_dynamically() {
    let (mut c, a, t) = setup("Torment");
    turn(&mut c, &t, "Tackle");
    let mut c = pending(&c, &t, &a, "Tackle");
    turn(&mut c, &a, "Torment");
    let c = resolve(&pending(&c, &a, &t, "Torment"));
    let mut c = round_end(&c);
    turn(&mut c, &t, "Tackle");
    assert!(rules::dice_count(
        &c,
        &super::targeting_tests::attack(&t, vec![a.clone()], "Tackle"),
        &catalog()
    )
    .is_err());
    turn(&mut c, &t, "Quick Attack");
    let c = pending(&c, &t, &a, "Quick Attack");
    let mut c = round_end(&c);
    turn(&mut c, &t, "Tackle");
    assert!(rules::dice_count(
        &c,
        &super::targeting_tests::attack(&t, vec![a.clone()], "Tackle"),
        &catalog()
    )
    .is_ok());
    turn(&mut c, &t, "Quick Attack");
    assert!(rules::dice_count(
        &c,
        &super::targeting_tests::attack(&t, vec![a.clone()], "Quick Attack"),
        &catalog()
    )
    .is_err());
}
#[test]
fn completion_a16_history_survives_save_and_undo_and_old_states_default_empty() {
    let (c, a, t) = setup("Blood Moon");
    let action = super::targeting_tests::attack(&a, vec![t.clone()], "Blood Moon");
    let c = roll(&c, "gm", &action, 6);
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .completion
            .uses
            .len(),
        1
    );
    let saved: Campaign = serde_json::from_value(serde_json::to_value(&c).unwrap()).unwrap();
    assert_eq!(
        saved
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .completion
            .uses[0]
            .name,
        "Blood Moon"
    );
    let restored = decision(
        &c,
        "gm",
        Action::RollbackBattle {
            event_sequence: c.battle_undo.last().unwrap().event_sequence,
            reason: "Undo move declaration and history".into(),
        },
    );
    assert!(restored
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .completion
        .uses
        .is_empty());
    let state: completion::MoveCompletionState =
        serde_json::from_value(serde_json::json!({"blocks":["MOVE-A15"]})).unwrap();
    assert!(state.uses.is_empty());
    assert!(state.entered.is_empty());
    let (mut c, a, t) = setup("Blood Moon");
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
        Box::new(completion::initial(MOVE_RESTRICTIONS_ENGINE));
    let c = pending(&c, &a, &t, "Blood Moon");
    assert_eq!(
        battle::transition(&c, "gm", &cmd(&c), vec![], &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
}
#[test]
fn completion_a16_public_switch_records_actual_deployment_round_without_guessing_eligibility() {
    let (mut c, a, _) = setup("Blood Moon");
    let incoming = reserve(&mut c, &a);
    let round = c.battle.as_ref().unwrap().round;
    let c = decision(
        &c,
        "gm",
        Action::SwitchPokemon {
            outgoing_id: a.clone(),
            incoming_id: incoming.clone(),
            gm_override: false,
            reason: "Record reserve deployment".into(),
        },
    );
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .completion
            .entered
            .get(&incoming),
        Some(&round)
    );
    assert_eq!(p(&c, &incoming).available_round, round + 1);
}
fn bypass_attack(c: &Campaign, a: &str, t: &str, name: &str) -> Campaign {
    let c = ordinary_accuracy(c, a, t, name);
    let mut c = roll(&c, "gm", &damage_command(&c), 6);
    if !c
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
#[test]
fn completion_a17_all_four_bypass_shields_and_only_printed_force_fields() {
    for name in [
        "Hyper Drill",
        "Hyperspace Hole",
        "Mighty Cleave",
        "Hyperspace Fury",
    ] {
        let (mut c, a, t) = setup(name);
        set_species(&mut c, &t, "Eevee");
        refresh_field(&mut c);
        let baseline = bypass_attack(&c, &a, &t, name);
        let base = 100 - p(&baseline, &t).hp;
        let side = format!("side:{}", p(&c, &t).side);
        saved_field_rule(&mut c, "Reflect", "bypass-fixture", &side, Some(4), 3);
        saved_field_rule(&mut c, "Light Screen", "bypass-fixture", &side, Some(4), 3);
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
                    armed_turn: 1,
                    available: true,
                },
            );
        let after = bypass_attack(&c, &a, &t, name);
        let damage = 100 - p(&after, &t).hp;
        assert_eq!(
            damage,
            if name == "Hyper Drill" {
                (base - 3).max(0)
            } else {
                base
            },
            "{name}"
        );
        assert_eq!(
            stat(&after, &a, "Defense"),
            if name == "Hyperspace Fury" { -1 } else { 0 },
            "{name}"
        );
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
}
#[test]
fn completion_a17_mighty_cleave_retains_environment_reduction_and_earlier_engine_behavior() {
    use crate::combat::field;
    let (mut c, a, t) = setup("Mighty Cleave");
    let side = format!("side:{}", p(&c, &t).side);
    saved_field_rule(&mut c, "Reflect", "bypass-fixture", &side, Some(4), 3);
    saved_field_rule(&mut c, "flowers", "bypass-fixture", "all", Some(4), 0);
    let m = field::move_data(
        &c,
        p(&c, &a),
        &catalog().find("moves", "Name", "Mighty Cleave").unwrap(),
    );
    assert_eq!(
        field::reduction(&c, p(&c, &a), p(&c, &t), &m),
        2,
        "environment still applies"
    );
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
        Box::new(completion::initial(MOVE_HISTORY_ENGINE));
    assert_eq!(
        field::reduction(&c, p(&c, &a), p(&c, &t), &m),
        5,
        "historical field reduction remains"
    );
    let c = pending(&c, &a, &t, "Mighty Cleave");
    assert_eq!(
        battle::transition(&c, "gm", &cmd(&c), vec![], &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
}
#[test]
fn completion_a17_hyperspace_hole_ignores_defense_and_fury_self_drop_survives_target_faint() {
    let (c, a, t) = setup("Hyperspace Hole");
    let queued = ordinary_accuracy(&c, &a, &t, "Hyperspace Hole");
    assert_eq!(queued_damage_pool(&queued).defense, 0);
    let (mut c, a, t) = setup("Hyperspace Fury");
    pm(&mut c, &t).hp = 1;
    let c = pending(&c, &a, &t, "Hyperspace Fury");
    assert_eq!(p(&c, &t).hp, 0);
    let c = resolve(&c);
    assert_eq!(stat(&c, &a, "Defense"), -1);
}
#[test]
fn completion_a18_all_nine_attack_paths_finish_and_install_only_printed_grounding() {
    for name in [
        "Earthquake",
        "Gust",
        "Sky Uppercut",
        "Surf",
        "Freeze Dry",
        "Origin Pulse",
        "Precipice Blades",
        "Smack Down",
        "Thousand Arrows",
    ] {
        let (c, a, t) = setup(name);
        let mut c = pending(&c, &a, &t, name);
        let grounding = ["Smack Down", "Thousand Arrows"].contains(&name);
        if grounding {
            c = resolve(&c);
        }
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
        assert_eq!(completion::grounded(&c, p(&c, &t)), grounding, "{name}");
    }
}
#[test]
fn completion_a18_public_attacks_reach_only_the_reviewed_hidden_charge_targets() {
    for (name, hidden) in [
        ("Earthquake", "Dig"),
        ("Gust", "Fly"),
        ("Sky Uppercut", "Bounce"),
        ("Surf", "Dive"),
        ("Freeze Dry", "Dive"),
        ("Origin Pulse", "Phantom Force"),
        ("Precipice Blades", "Shadow Force"),
        ("Smack Down", "Fly"),
    ] {
        let (c, a, t) = setup(hidden);
        let charge = super::targeting_tests::attack(&a, vec![t.clone()], hidden);
        let mut c = roll(&c, "gm", &charge, 6);
        assert!(
            c.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .special
                .charges
                .contains_key(&a),
            "{hidden}"
        );
        turn(&mut c, &t, name);
        let mut after = pending(&c, &t, &a, name);
        if name == "Smack Down" {
            after = resolve(&after);
        }
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
            "{name} against {hidden}"
        );
    }
    let (c, a, t) = setup("Dive");
    let charge = super::targeting_tests::attack(&a, vec![t.clone()], "Dive");
    let mut c = roll(&c, "gm", &charge, 6);
    turn(&mut c, &t, "Freeze Dry");
    assert!(special::vulnerable(&c, &a, "Freeze Dry"));
    turn(&mut c, &t, "Gust");
    assert!(rules::dice_count(
        &c,
        &super::targeting_tests::attack(&t, vec![a.clone()], "Gust"),
        &catalog()
    )
    .is_err());
}
#[test]
fn completion_a18_water_effectiveness_and_printed_type_immunities_use_real_damage_pools() {
    for species in ["Mudkip", "Gyarados", "Empoleon"] {
        let (mut c, a, t) = setup("Freeze Dry");
        set_species(&mut c, &t, species);
        refresh_field(&mut c);
        let queued = ordinary_accuracy(&c, &a, &t, "Freeze Dry");
        assert_eq!(queued_damage_pool(&queued).effectiveness, 1, "{species}");
    }
    for (name, ability, species) in [
        ("Origin Pulse", "Water Absorb", "Eevee"),
        ("Precipice Blades", "Levitate", "Charizard"),
        ("Thousand Arrows", "Levitate", "Charizard"),
        ("Thousand Arrows", "Wonder Guard", "Eevee"),
    ] {
        let (mut c, a, t) = setup(name);
        set_species(&mut c, &t, species);
        pm(&mut c, &t).selected_ability = ability.into();
        refresh_field(&mut c);
        let queued = ordinary_accuracy(&c, &a, &t, name);
        assert!(!queued_damage_pool(&queued).immune, "{name} {ability}");
    }
}
#[test]
fn completion_a18_grounding_removes_ground_move_immunity_until_public_recall() {
    use crate::combat::field;
    for name in ["Smack Down", "Thousand Arrows"] {
        let (mut c, a, t) = setup(name);
        let incoming = reserve(&mut c, &t);
        set_species(&mut c, &t, "Charizard");
        pm(&mut c, &t).selected_ability = "Levitate".into();
        refresh_field(&mut c);
        let mut c = resolve(&pending(&c, &a, &t, name));
        assert!(field::grounded(&c, p(&c, &t), &catalog()));
        turn(&mut c, &a, "Earthquake");
        let queued = ordinary_accuracy(&c, &a, &t, "Earthquake");
        assert!(!queued_damage_pool(&queued).immune);
        let mut c = roll(&queued, "gm", &damage_command(&queued), 1);
        turn(&mut c, &t, "Tackle");
        let c = decision(
            &c,
            "gm",
            Action::SwitchPokemon {
                outgoing_id: t.clone(),
                incoming_id: incoming,
                gm_override: false,
                reason: "Recall grounded target".into(),
            },
        );
        assert!(!completion::grounded(&c, p(&c, &t)));
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
            .any(|e| e.target == t && e.slot == "ground-immunity"));
    }
}
#[test]
fn completion_a18_shields_still_block_grounding_and_older_encounters_keep_immunity() {
    let (mut c, a, t) = setup("Thousand Arrows");
    set_species(&mut c, &t, "Charizard");
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
                armed_turn: 1,
                available: true,
            },
        );
    let c = resolve(&pending(&c, &a, &t, "Thousand Arrows"));
    assert!(!completion::grounded(&c, p(&c, &t)));
    let (mut c, a, t) = setup("Thousand Arrows");
    set_species(&mut c, &t, "Charizard");
    refresh_field(&mut c);
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
        Box::new(completion::initial(BYPASS_ATTACKS_ENGINE));
    let queued = ordinary_accuracy(&c, &a, &t, "Thousand Arrows");
    assert!(queued_damage_pool(&queued).immune);
}
fn completion_item_command(command: crate::combat::items::ItemCommand) -> Action {
    Action::HeldItem {
        command,
        reason: "Prepare existing move item context".into(),
    }
}
fn existing_item_move(name: &str, prepare: bool) -> (Campaign, String, String, String) {
    use crate::combat::items::{self, ItemCommand};
    let (mut c, a, t) = setup(name);
    let sid = p(&c, &a).sheet_id.clone();
    let mut s = c.sheets[&sid].clone();
    s.inventory.insert("Oran Berry".into(), 1);
    c.sheets.insert(
        sid,
        items::equip(&s, "Oran Berry", "move-completion-item", &catalog()).unwrap(),
    );
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.items.enabled = true;
    f.items.completion.enabled = true;
    items::initialize(&mut c, &catalog()).unwrap();
    items::completion::initialize(&mut c, &catalog());
    let unit = items::held(&c, &a)[0].id.clone();
    if prepare {
        let owner = p(&c, &a).owner_id.clone();
        if name == "Natural Gift" {
            c = decision(
                &c,
                "gm",
                completion_item_command(ItemCommand::BerryFlavor {
                    unit_id: unit.clone(),
                    flavor: "Spicy".into(),
                }),
            );
            c = decision(
                &c,
                &owner,
                completion_item_command(ItemCommand::NaturalGift {
                    combatant_id: a.clone(),
                    unit_id: unit.clone(),
                }),
            );
        } else {
            c = decision(
                &c,
                &owner,
                completion_item_command(ItemCommand::Fling {
                    combatant_id: a.clone(),
                    unit_id: Some(unit.clone()),
                }),
            );
        }
    }
    (c, a, t, unit)
}
#[test]
fn completion_a19_cut_and_both_strength_names_finish_battle_damage_without_narrative_mutation() {
    for name in ["Cut", "Strength", "Strength "] {
        let (c, a, t) = setup(name);
        let sheets = serde_json::to_value(&c.sheets).unwrap();
        let after = pending(&c, &a, &t, name);
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
            "{name:?}"
        );
        assert_eq!(serde_json::to_value(&after.sheets).unwrap(), sheets);
    }
    assert_eq!(
        catalog().find("moves", "Name", "Strength ").unwrap()["Name"],
        "Strength"
    );
}
#[test]
fn completion_a19_prepared_item_moves_finish_after_damage_and_keep_physical_unit_spending() {
    use crate::combat::items;
    for name in ["Fling", "Natural Gift"] {
        let (c, a, t, unit) = existing_item_move(name, true);
        let after = pending(&c, &a, &t, name);
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
        assert!(items::held(&after, &a).is_empty(), "{name}");
        assert_ne!(
            after
                .battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .items
                .units[&unit]
                .state,
            "held"
        );
    }
    let (mut c, a, t) = setup("Fling");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .enabled = true;
    items::initialize(&mut c, &catalog()).unwrap();
    let c = pending(&c, &a, &t, "Fling");
    assert!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .queue
            .is_empty(),
        "unarmed Fling is valid"
    );
}
#[test]
fn completion_a19_item_preparation_and_historical_completion_remain_explicit() {
    for name in ["Fling", "Natural Gift"] {
        let (c, a, t, _) = existing_item_move(name, false);
        let action = super::targeting_tests::attack(&a, vec![t.clone()], name);
        assert!(
            rules::dice_count(&c, &action, &catalog()).is_err(),
            "{name} needs selected ingredient/projectile"
        );
        let (mut c, a, t, _) = existing_item_move(name, true);
        c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
            Box::new(completion::initial(RANGE_IMMUNITY_MOVES_ENGINE));
        let c = pending(&c, &a, &t, name);
        assert!(
            !c.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .queue
                .is_empty(),
            "{name} old extra stage"
        );
        let c = resolve(&c);
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
    let (mut c, a, t) = setup("Cut");
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
        Box::new(completion::initial(RANGE_IMMUNITY_MOVES_ENGINE));
    let c = pending(&c, &a, &t, "Cut");
    assert_eq!(
        battle::transition(&c, "gm", &cmd(&c), vec![], &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
}
#[test]
fn completion_a20_partner_fields_follow_damage_side_and_four_round_clock() {
    use crate::combat::field;
    for (name, kind, category) in [
        ("Baddy Bad", "Reflect", "Physical"),
        ("Glitzy Glow", "Light Screen", "Special"),
    ] {
        let (c, a, t) = setup(name);
        let c = pending(&c, &a, &t, name);
        assert!(q(&c).damage > 0);
        let mut c = resolve(&c);
        assert!(field::has(&c, kind, p(&c, &a)));
        assert!(!field::has(&c, kind, p(&c, &t)));
        assert_eq!(
            field::reduction(
                &c,
                p(&c, &t),
                p(&c, &a),
                &serde_json::json!({"Name":"screen check","Type":"Normal","Category":category})
            ),
            1
        );
        assert!(c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .battlefield
            .rules
            .iter()
            .any(|r| r.kind == kind && r.rounds == Some(4)));
        for _ in 0..4 {
            c = round_end(&c);
        }
        assert!(!field::has(&c, kind, p(&c, &a)));
        let (mut c, a, t) = setup(name);
        pm(&mut c, &t).selected_ability = "Wonder Guard".into();
        let c = pending(&c, &a, &t, name);
        assert_eq!(q(&c).damage, 0);
        let c = resolve(&c);
        assert!(!field::has(&c, kind, p(&c, &a)));
    }
}
#[test]
fn completion_a20_happiness_is_capped_and_zippy_zap_keeps_critical_and_reaction_two() {
    let mut pools = vec![];
    for happy in 0..=5 {
        let (mut c, a, t) = setup("Pika Papow");
        super::sequencing_tests::loyal(&mut c, &a, 3);
        let sid = p(&c, &a).sheet_id.clone();
        c.sheets
            .get_mut(&sid)
            .unwrap()
            .workflow
            .as_mut()
            .unwrap()
            .happiness = happy;
        let queued = ordinary_accuracy(&c, &a, &t, "Pika Papow");
        pools.push(queued_damage_pool(&queued).dice);
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
    }
    assert_eq!(pools[5], pools[0] + 5);
    let (c, a, t) = setup("Zippy Zap");
    let c = ordinary_accuracy(&c, &a, &t, "Zippy Zap");
    assert!(q(&c).critical);
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
    let (mut c, a, t) = setup("Tackle");
    set_species(&mut c, &t, "Eevee");
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .moves
        .push("Zippy Zap".into());
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .tactics
        .enabled = true;
    let c = roll(
        &c,
        "gm",
        &super::targeting_tests::attack(&a, vec![t.clone()], "Tackle"),
        6,
    );
    let reaction = Action::Tactics {
        command: crate::combat::tactics::TacticsCommand::Reaction {
            combatant_id: t.clone(),
            move_name: "Zippy Zap".into(),
            target_ids: vec![a],
        },
        reason: "Use Zippy Zap Reaction 2".into(),
    };
    let c = roll(&c, "gm", &reaction, 6);
    assert_eq!(q(&c).priority, 2);
    assert_eq!(q(&c).actor_id, t);
    assert!(q(&c).critical);
}
#[test]
fn completion_a20_final_stage_uses_forward_ordinary_edges_not_from_or_mega_links() {
    for name in ["Baddy Bad", "Glitzy Glow", "Pika Papow", "Zippy Zap"] {
        for (species, allowed) in [
            ("Eevee", true),
            ("Pikachu", true),
            ("Dunsparce", true),
            ("Ivysaur", true),
            ("Charizard", false),
            ("Vaporeon", false),
            ("Raichu", false),
            ("Mew", false),
        ] {
            let (mut c, a, t) = setup(name);
            set_species(&mut c, &a, species);
            let action = super::targeting_tests::attack(&a, vec![t], name);
            assert_eq!(
                rules::dice_count(&c, &action, &catalog()).is_ok(),
                allowed,
                "{name} {species}"
            );
        }
    }
    let (mut c, a, t) = setup("Bouncy Bubble");
    set_species(&mut c, &a, "Charizard");
    let action = super::targeting_tests::attack(&a, vec![t], "Bouncy Bubble");
    assert!(rules::dice_count(&c, &action, &catalog()).is_err());
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
        Box::new(completion::initial(EXISTING_MOVE_COMPLETION_ENGINE));
    assert!(
        rules::dice_count(&c, &action, &catalog()).is_ok(),
        "historical stage guard preserved"
    );
}
#[test]
fn completion_a20_sheer_force_retains_damage_without_creating_partner_field() {
    let (mut c, a, t) = setup("Baddy Bad");
    pm(&mut c, &a).selected_ability = "Sheer Force".into();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .provenance_rules = true;
    let c = decision(
        &c,
        "gm",
        Action::ChooseSheerForce {
            combatant_id: a.clone(),
            ignore_added_effects: true,
            reason: "Trade partner field for damage".into(),
        },
    );
    let c = pending(&c, &a, &t, "Baddy Bad");
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .is_empty());
    assert!(!crate::combat::field::has(&c, "Reflect", p(&c, &a)));
}

#[test]
fn completion_a21_tailwind_live_side_stats_duration_and_non_additive_channels() {
    use crate::combat::{extra, field};
    let (mut c, a, t) = setup("Tailwind");
    let incoming = reserve(&mut c, &a);
    let before = rules::effective_sheet(&c, p(&c, &a)).unwrap().stats["Dexterity"];
    c = support_cast(&c, &a, vec![], "Tailwind");
    assert!(field::has(&c, "Tailwind", p(&c, &a)));
    assert!(!field::has(&c, "Tailwind", p(&c, &t)));
    assert_eq!(stat(&c, &a, "Dexterity"), 2);
    assert_eq!(
        rules::effective_sheet(&c, p(&c, &a)).unwrap().stats["Dexterity"],
        before + 2
    );
    c = round_end(&c);
    turn(&mut c, &a, "Tailwind");
    c = support_cast(&c, &a, vec![], "Tailwind");
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
            .filter(|r| r.kind == "Tailwind")
            .count(),
        1
    );
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
            .find(|r| r.kind == "Tailwind")
            .unwrap()
            .rounds,
        Some(3),
        "recasting does not restart a round-duration counter"
    );
    extra::change_stat(&mut c, &a, &a, "Dexterity", 2).unwrap();
    assert_eq!(
        stat(&c, &a, "Dexterity"),
        2,
        "Agility-sized move boost does not add to Tailwind"
    );
    let mut copy: Campaign = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
    assert_eq!(stat(&copy, &a, "Dexterity"), 2);
    copy = decision(
        &copy,
        "gm",
        Action::SwitchPokemon {
            outgoing_id: a.clone(),
            incoming_id: incoming.clone(),
            gm_override: false,
            reason: "Tailwind follows the side, not the caster".into(),
        },
    );
    assert_eq!(stat(&copy, &incoming, "Dexterity"), 2);
    assert_eq!(stat(&copy, &a, "Dexterity"), 0);
    for _ in 0..2 {
        copy = round_end(&copy);
        assert_eq!(stat(&copy, &incoming, "Dexterity"), 2);
    }
    copy = round_end(&copy);
    assert_eq!(stat(&copy, &incoming, "Dexterity"), 0);
}
#[test]
fn completion_a21_tailwind_ability_channels_and_transfer_snapshot() {
    use crate::combat::stat_channels;
    for (ability, expected) in [("", 2), ("Simple", 3), ("Contrary", -2), ("Clear Body", 0)] {
        let (mut c, a, t) = setup("Tailwind");
        let side = p(&c, &a).side.clone();
        pm(&mut c, &t).side = side;
        pm(&mut c, &t).selected_ability = ability.into();
        c = support_cast(&c, &a, vec![], "Tailwind");
        assert_eq!(stat(&c, &t, "Dexterity"), expected, "{ability}");
        if ability == "Clear Body" {
            pm(&mut c, &a).selected_ability = ability.into();
            assert_eq!(stat(&c, &a, "Dexterity"), 2);
        }
    }
    let (mut c, a, t) = setup("Tailwind");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .stat_channels
        .enabled = true;
    pm(&mut c, &a).selected_ability = "Simple".into();
    c = support_cast(&c, &a, vec![], "Tailwind");
    stat_channels::transfer(&mut c, &a, &t, "Baton Pass").unwrap();
    assert_eq!(
        stat(&c, &t, "Dexterity"),
        3,
        "snapshot captures effective field boost once"
    );
}
#[test]
fn completion_a21_fairy_lock_blocks_both_sides_and_entrants_for_one_round() {
    let (mut c, a, t) = setup("Fairy Lock");
    let incoming = reserve(&mut c, &t);
    c = support_cast(&c, &a, vec![], "Fairy Lock");
    assert!(completion::blocked(&c, &a));
    assert!(completion::blocked(&c, &t));
    let action = Action::SwitchPokemon {
        outgoing_id: t.clone(),
        incoming_id: incoming.clone(),
        gm_override: false,
        reason: "Fairy Lock check".into(),
    };
    assert!(battle::transition(&c, "gm", &action, vec![], &catalog()).is_err());
    pm(&mut c, &t).selected_ability = "Run Away".into();
    assert!(!completion::blocked(&c, &t));
    pm(&mut c, &t).selected_ability = "".into();
    c = decision(
        &c,
        "gm",
        Action::SwitchPokemon {
            outgoing_id: t.clone(),
            incoming_id: incoming.clone(),
            gm_override: true,
            reason: "GM explicitly overrides Blocking".into(),
        },
    );
    assert!(
        completion::blocked(&c, &incoming),
        "field also affects newly deployed participants"
    );
    c = round_end(&c);
    assert!(!completion::blocked(&c, &a));
    assert!(!completion::blocked(&c, &incoming));
}
#[test]
fn completion_a21_historical_fields_keep_manual_resolution() {
    for name in ["Tailwind", "Fairy Lock"] {
        let (mut c, a, _) = setup(name);
        c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
            Box::new(completion::initial(PARTNER_MOVES_ENGINE));
        let action = super::targeting_tests::attack(&a, vec![], name);
        let mut c = roll(&c, "gm", &action, 6);
        while c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
            c = pass(&c);
        }
        assert_eq!(
            battle::transition(&c, "gm", &cmd(&c), vec![], &catalog())
                .unwrap_err()
                .code,
            "manual_rule"
        );
    }
}

fn spider_setup() -> (Campaign, String, String, String) {
    let (mut c, a, t) = setup("Spider Web");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .entry_hazards = true;
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .compound
        .enabled = true;
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .decoys
        .enabled = true;
    let incoming = reserve(&mut c, &t);
    (c, a, t, incoming)
}
fn spider_switch(c: &Campaign, outgoing: &str, incoming: &str, gm_override: bool) -> Campaign {
    decision(
        c,
        "gm",
        Action::SwitchPokemon {
            outgoing_id: outgoing.into(),
            incoming_id: incoming.into(),
            gm_override,
            reason: "Reviewed Spider Web entry".into(),
        },
    )
}
#[test]
fn completion_a22_spider_web_only_blocks_entrants_persists_and_does_not_stack() {
    use crate::combat::field;
    let (c, a, t, incoming) = spider_setup();
    let mut c = support_cast(&c, &a, vec![], "Spider Web");
    assert!(
        !completion::blocked(&c, &t),
        "existing occupants are not entrants"
    );
    assert!(!field::has(&c, "Spider Web", p(&c, &a)));
    assert!(field::has(&c, "Spider Web", p(&c, &t)));
    turn(&mut c, &a, "Spider Web");
    c = support_cast(&c, &a, vec![], "Spider Web");
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
            .filter(|r| r.kind == "Spider Web")
            .count(),
        1
    );
    // No Ground predicate appears on the Spider Web card.
    pm(&mut c, &incoming).selected_ability = "Levitate".into();
    c = spider_switch(&c, &t, &incoming, false);
    assert!(completion::blocked(&c, &incoming));
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
            .filter(|e| e.name == "move-block:Spider Web" && e.target == incoming)
            .count(),
        1
    );
    let action = Action::SwitchPokemon {
        outgoing_id: incoming.clone(),
        incoming_id: t.clone(),
        gm_override: false,
        reason: "Blocking applies to recall".into(),
    };
    assert!(battle::transition(&c, "gm", &action, vec![], &catalog()).is_err());
    let mut restored: Campaign = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
    pm(&mut restored, &a).deployed = false;
    assert!(
        completion::blocked(&restored, &incoming),
        "scene block does not depend on caster staying"
    );
    for _ in 0..5 {
        c = round_end(&c);
    }
    assert!(completion::blocked(&c, &incoming));
    c = spider_switch(&c, &incoming, &t, true);
    assert!(!completion::blocked(&c, &incoming));
    assert!(
        completion::blocked(&c, &t),
        "replacement is also an entrant"
    );
}
#[test]
fn completion_a22_entry_protection_and_historical_adoption() {
    use crate::combat::items;
    for protection in ["Run Away", "Heavy-Duty Boots"] {
        let (mut c, a, t, incoming) = spider_setup();
        if protection == "Run Away" {
            pm(&mut c, &incoming).selected_ability = protection.into();
        } else {
            let sid = p(&c, &incoming).sheet_id.clone();
            let mut s = c.sheets[&sid].clone();
            s.inventory.insert(protection.into(), 1);
            c.sheets.insert(
                sid,
                items::equip(&s, protection, "spider-boots", &catalog()).unwrap(),
            );
            let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
            f.items.enabled = true;
            f.items.completion.enabled = true;
            items::initialize(&mut c, &catalog()).unwrap();
            items::completion::initialize(&mut c, &catalog());
        }
        c = support_cast(&c, &a, vec![], "Spider Web");
        c = spider_switch(&c, &t, &incoming, false);
        assert!(!completion::blocked(&c, &incoming), "{protection}");
    }
    let (mut c, a, _, _) = spider_setup();
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
        Box::new(completion::initial(BATTLEFIELD_SUPPORT_ENGINE));
    let action = super::targeting_tests::attack(&a, vec![], "Spider Web");
    c = roll(&c, "gm", &action, 6);
    while c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
        c = pass(&c);
    }
    assert_eq!(
        battle::transition(&c, "gm", &cmd(&c), vec![], &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
}
#[test]
fn completion_a22_hazard_cleanup_preserves_previously_applied_scene_blocking() {
    use crate::combat::field;
    for name in ["Rapid Spin", "Mortal Spin", "Defog", "Tidy Up"] {
        let (c, a, t, incoming) = spider_setup();
        let c = support_cast(&c, &a, vec![], "Spider Web");
        let mut c = spider_switch(&c, &t, &incoming, false);
        // Permit the entrant to take its first turn normally next round.
        c = round_end(&c);
        turn(&mut c, &incoming, name);
        let c = match name {
            "Defog" | "Tidy Up" => support_cast(&c, &incoming, vec![], name),
            _ => resolve(&pending(&c, &incoming, &a, name)),
        };
        assert!(!field::has(&c, "Spider Web", p(&c, &incoming)), "{name}");
        assert!(
            completion::blocked(&c, &incoming),
            "{name}: hazard removal does not cure its previously applied scene block"
        );
    }
}

#[test]
fn completion_a23_ordinary_attacks_finish_without_excluded_dynamax_bonuses() {
    for name in [
        "Behemoth Bash",
        "Behemoth Blade",
        "Dynamax Cannon",
        "Grassy Glide",
    ] {
        let (c, a, t) = setup(name);
        let queued = ordinary_accuracy(&c, &a, &t, name);
        let pool = queued_damage_pool(&queued);
        assert_eq!(
            pool.power,
            if name == "Grassy Glide" { 3 } else { 4 },
            "{name}"
        );
        let c = roll(&queued, "gm", &damage_command(&queued), 1);
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
        let (mut c, a, t) = setup(name);
        c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
            Box::new(completion::initial(SPIDER_WEB_ENGINE));
        let c = pending(&c, &a, &t, name);
        assert_eq!(
            battle::transition(&c, "gm", &cmd(&c), vec![], &catalog())
                .unwrap_err()
                .code,
            "manual_rule"
        );
    }
}
#[test]
fn completion_a23_grassy_glide_conditional_priority_uses_live_field_scope_and_clock() {
    use crate::combat::{field, tactics::TacticsCommand};
    for scope in ["none", "all", "opposing"] {
        let (mut c, a, t) = setup("Tackle");
        let sid = p(&c, &t).sheet_id.clone();
        c.sheets
            .get_mut(&sid)
            .unwrap()
            .moves
            .push("Grassy Glide".into());
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .tactics
            .enabled = true;
        if scope != "none" {
            let actual = if scope == "all" {
                "all".to_string()
            } else {
                format!("side:{}", p(&c, &a).side)
            };
            saved_field_rule(&mut c, "Grassy Terrain", "move:review", &actual, Some(1), 0);
        }
        let m = catalog().find("moves", "Name", "Grassy Glide").unwrap();
        let resolved = field::move_data(&c, p(&c, &t), &m);
        assert_eq!(
            resolved["Attributes"]["Reaction"].as_str(),
            if scope == "all" { Some("1") } else { None }
        );
        c = roll(
            &c,
            "gm",
            &super::targeting_tests::attack(&a, vec![t.clone()], "Tackle"),
            6,
        );
        let action = Action::Tactics {
            command: TacticsCommand::Reaction {
                combatant_id: t.clone(),
                move_name: "Grassy Glide".into(),
                target_ids: vec![a],
            },
            reason: "Conditional Grassy Glide reaction".into(),
        };
        if scope == "all" {
            let c = roll(&c, "gm", &action, 6);
            assert_eq!(q(&c).priority, 1);
            assert_eq!(q(&c).actor_id, t);
        } else {
            assert!(battle::transition(&c, "gm", &action, vec![], &catalog()).is_err());
        }
    }
    let (mut c, a, _) = setup("Grassy Glide");
    saved_field_rule(&mut c, "Grassy Terrain", "move:review", "all", Some(1), 0);
    c = round_end(&c);
    let m = catalog().find("moves", "Name", "Grassy Glide").unwrap();
    assert!(field::move_data(&c, p(&c, &a), &m)["Attributes"]["Reaction"].is_null());
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().completion =
        Box::new(completion::initial(SPIDER_WEB_ENGINE));
    assert_eq!(
        field::move_data(&c, p(&c, &a), &m)["Attributes"]["Reaction"],
        "1",
        "historical encounters retain raw printed metadata behavior"
    );
}
