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
    let mut c = hit(c, a, t, name);
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
