use super::super::targeting_tests::{damage_command, pass, roll};
use super::*;
use crate::combat::{
    abilities, extra, field,
    items::completion::{self as ci, CompletionCommand},
    special,
};
fn setup6(item: &str, ability: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = setup(item, ability);
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.completion_enable();
    ci::initialize(&mut c, &catalog());
    (c, a, t)
}
trait EnableCompletion {
    fn completion_enable(&mut self);
}
impl EnableCompletion for TurnFlow {
    fn completion_enable(&mut self) {
        self.items.completion.enabled = true;
        self.interaction_rules = true;
        self.reviewed_rules = true;
        self.tactics.enabled = true;
        self.stat_channels.enabled = true;
        self.ability_changes.enabled = true;
    }
}
fn cmd(command: CompletionCommand) -> Action {
    action(ItemCommand::Completion { command })
}
fn run(c: &Campaign, who: &str, command: CompletionCommand) -> Campaign {
    battle::transition(c, who, &cmd(command), vec![], &catalog())
        .unwrap()
        .0
}
fn select(c: &Campaign, a: &str, value: &str) -> Campaign {
    run(
        c,
        "player",
        CompletionCommand::Select {
            combatant_id: a.into(),
            unit_id: "berry".into(),
            value: value.into(),
        },
    )
}
fn rule(c: &Campaign, key: &str, value: &str) -> Campaign {
    run(
        c,
        "gm",
        CompletionCommand::Rule {
            key: key.into(),
            value: value.into(),
        },
    )
}
fn learn(c: &mut Campaign, a: &str, name: &str) {
    let sid = p(c, a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().moves.push(name.into());
}
fn use_med(c: &Campaign, a: &str, t: &str, name: &str, dose: u32) -> CompletionCommand {
    CompletionCommand::Medicine {
        combatant_id: a.into(),
        target_id: t.into(),
        supply_sheet_id: p(c, a).sheet_id.clone(),
        unit_id: None,
        name: name.into(),
        dose,
        trainer_on_field: false,
    }
}
fn stock(c: &mut Campaign, a: &str, name: &str) {
    let sid = p(c, a).sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .inventory
        .insert(name.into(), 2);
}
#[test]
fn completion_choice_items_select_exact_categories_penalize_other_moves_and_keep_historical_gates()
{
    for (item, name, category) in [
        ("Choice Band", "Tackle", "Physical"),
        ("Choice Specs", "Swift", "Special"),
        ("Choice Scarf", "Tackle", "Physical"),
    ] {
        let (mut c, a, t) = setup6(item, "");
        learn(&mut c, &a, "Swift");
        learn(&mut c, &a, "Tackle");
        let m = catalog().find("moves", "Name", name).unwrap();
        assert!(ci::guard(&c, &a, &m).is_err());
        let c = select(&c, &a, name);
        assert!(ci::guard(&c, &a, &m).is_ok());
        assert_eq!(
            ci::damage_bonus(&c, p(&c, &a), &m),
            if item == "Choice Scarf" { 0 } else { 3 }
        );
        let other = catalog()
            .find(
                "moves",
                "Name",
                if category == "Physical" {
                    "Swift"
                } else {
                    "Tackle"
                },
            )
            .unwrap();
        assert_eq!(ci::damage_bonus(&c, p(&c, &a), &other), -3);
        assert!(battle::transition(
            &c,
            "other",
            &cmd(CompletionCommand::Select {
                combatant_id: a.clone(),
                unit_id: "berry".into(),
                value: name.into()
            }),
            vec![],
            &catalog()
        )
        .is_err());
        assert!(battle::transition(
            &c,
            "player",
            &cmd(CompletionCommand::Select {
                combatant_id: a.clone(),
                unit_id: "berry".into(),
                value: name.into()
            }),
            vec![],
            &catalog()
        )
        .is_err());
        if item == "Choice Scarf" {
            let m = field::move_data(&c, p(&c, &a), &m);
            assert_eq!(m["Attributes"]["Reaction"], "5");
            assert_eq!(items::initiative_bonus(&c, &a), 3);
        }
        let mut old = c.clone();
        old.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .items
            .completion
            .enabled = false;
        assert_eq!(ci::damage_bonus(&old, p(&old, &a), &m), 0);
        assert!(!ci::has(&old, &a, item));
        let _ = t;
    }
}
#[test]
fn completion_scarf_late_timing_has_a_real_gm_choice() {
    let (mut c, a, _) = setup6("Choice Scarf", "");
    learn(&mut c, &a, "Roar");
    let c = select(&c, &a, "Roar");
    let m = catalog().find("moves", "Name", "Roar").unwrap();
    assert!(ci::guard(&c, &a, &m).is_err());
    let c = rule(&c, "scarf:berry", "item");
    let m = field::move_data(&c, p(&c, &a), &m);
    assert_eq!(m["Attributes"]["Reaction"], "5");
    assert!(m["Attributes"]["LateReaction"].is_null());
}
#[test]
fn completion_species_stat_and_critical_items_use_exact_species_and_suppression() {
    for (item, species, key, n) in [
        ("Light Ball", "Pikachu", "Special", 1),
        ("Thick Club", "Cubone", "Strength", 2),
        ("Lucky Punch", "Chansey", "Strength", 2),
    ] {
        let (mut c, a, _) = setup6(item, "");
        let sid = p(&c, &a).sheet_id.clone();
        c.sheets.get_mut(&sid).unwrap().species_id = Some(
            catalog().find("species", "Name", species).unwrap()["AppId"]
                .as_str()
                .unwrap()
                .into(),
        );
        ci::initialize(&mut c, &catalog());
        assert_eq!(ci::stat(&c, p(&c, &a), key), n);
        let base = c.sheets[&sid].stats[key];
        assert_eq!(
            rules::effective_sheet(&c, p(&c, &a)).unwrap().stats[key],
            base + n
        );
        pm(&mut c, &a).selected_ability = "Klutz".into();
        assert_eq!(ci::stat(&c, p(&c, &a), key), 0);
    }
    for item in ["Razor Claw", "Leek", "Lucky Punch"] {
        let (mut c, a, _) = setup6(item, "");
        let sid = p(&c, &a).sheet_id.clone();
        let species = if item == "Leek" {
            "Farfetch'd"
        } else {
            "Chansey"
        };
        c.sheets.get_mut(&sid).unwrap().species_id = Some(
            catalog().find("species", "Name", species).unwrap()["AppId"]
                .as_str()
                .unwrap()
                .into(),
        );
        ci::initialize(&mut c, &catalog());
        let m = catalog().find("moves", "Name", "Tackle").unwrap();
        assert!(ci::high_critical(&c, p(&c, &a), &m));
        assert_eq!(extra::critical_margin(&c, p(&c, &a), &m), 2);
        if item == "Lucky Punch" {
            let m = catalog().find("moves", "Name", "Swift").unwrap();
            assert!(!ci::high_critical(&c, p(&c, &a), &m));
        }
    }
}
#[test]
fn completion_power_increasers_use_item_channels_and_clear_amulet_preserves_self_changes() {
    let (c, a, _) = setup6("Power Increasers", "");
    let c = select(&c, &a, "Strength");
    assert_eq!(ci::stat(&c, p(&c, &a), "Strength"), 2);
    assert_eq!(ci::stat(&c, p(&c, &a), "Dexterity"), -1);
    let (mut c, a, t) = setup6("Clear Amulet", "");
    extra::change_stat(&mut c, &t, &a, "Strength", -2).unwrap();
    assert!(!p(&c, &a).runtime.modifiers.contains_key("Strength"));
    extra::change_stat(&mut c, &a, &a, "Strength", -1).unwrap();
    assert_eq!(p(&c, &a).runtime.modifiers["Strength"], -1);
}
#[test]
fn completion_loaded_dice_uses_move_chance_pools_and_merges_native_flinch() {
    for name in ["Ember", "Iron Head", "Flare Blitz"] {
        let (mut c, a, t) = setup6("Loaded Dice", "Serene Grace");
        learn(&mut c, &a, name);
        pm(&mut c, &t).selected_ability = "Wonder Skin".into();
        let c = hit(&c, &a, &t, name);
        let c = pass(&c);
        let c = roll(&c, "player", &damage_command(&c), 4);
        let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
        let expected = match name {
            "Iron Head" => 5,
            "Flare Blitz" => 3 + q.raw_damage_successes,
            _ => 3,
        };
        assert_eq!(
            rules::dice_count(&c, &orb_effect(&c), &catalog()).unwrap(),
            expected
        );
    }
}
#[test]
fn completion_power_herb_is_spent_only_when_it_removes_preparation() {
    let (mut c, a, t) = setup6("Power Herb", "");
    learn(&mut c, &a, "Solar Beam");
    let c = hit(&c, &a, &t, "Solar Beam");
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .special
        .charges
        .is_empty());
    assert!(items::held(&c, &a).is_empty());
    assert_eq!(
        c.battle
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
    let (c, a, _) = setup6("Metronome", "");
    let m = catalog().find("moves", "Name", "Double Kick").unwrap();
    assert_eq!(ci::damage_bonus(&c, p(&c, &a), &m), 1);
    let m = catalog().find("moves", "Name", "Tackle").unwrap();
    assert_eq!(ci::damage_bonus(&c, p(&c, &a), &m), 0);
}
#[test]
fn completion_balloon_ring_goggles_umbrella_and_boots_expose_scoped_immunities() {
    let (c, a, t) = setup6("Air Balloon", "");
    let m = catalog().find("moves", "Name", "Earthquake").unwrap();
    assert!(abilities::damage_immune_from(&c, p(&c, &t), p(&c, &a), &m));
    assert!(!field::grounded(&c, p(&c, &a), &catalog()));
    let (mut c, a, t) = setup6("Ring Target", "");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some(
        catalog().find("species", "Name", "Gastly").unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
    let c = rule(&c, &format!("ring:{a}"), "types");
    assert!(
        !rules::type_matchup(&c, p(&c, &t), p(&c, &a), "Normal", &catalog())
            .unwrap()
            .1
    );
    let (c, a, _) = setup6("Safety Goggles", "");
    let m = catalog().find("moves", "Name", "Powder Snow").unwrap();
    assert!(ci::added_immune(&c, &a, &m));
    let (mut c, a, _) = setup6("Umbrella", "");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .natural = Some(field::Weather::Rain);
    assert_eq!(field::weather(&c, Some(p(&c, &a))), None);
    assert_eq!(field::weather(&c, None), Some(field::Weather::Rain));
    let (c, a, _) = setup6("Heavy-Duty Boots", "");
    assert!(ci::has(&c, &a, "Heavy-Duty Boots"));
}
#[test]
fn completion_orbs_white_herb_and_focus_sash_have_bounded_triggers() {
    for (item, kind) in [("Flame Orb", "Burn"), ("Toxic Orb", "Poison")] {
        let (mut c, a, _) = setup6(item, "");
        let before = c.clone();
        ci::lifecycle(&before, &mut c, &catalog()).unwrap();
        assert!(p(&c, &a).runtime.statuses.iter().any(|s| s.kind == kind));
        let before = c.clone();
        ci::lifecycle(&before, &mut c, &catalog()).unwrap();
        assert_eq!(p(&c, &a).runtime.statuses.len(), 1);
    }
    let (mut c, a, t) = setup6("White Herb", "");
    extra::change_stat(&mut c, &t, &a, "Defense", -2).unwrap();
    let before = c.clone();
    ci::lifecycle(&before, &mut c, &catalog()).unwrap();
    assert!(!p(&c, &a).runtime.modifiers.values().any(|n| *n < 0));
    assert!(items::held(&c, &a).is_empty());
    let (mut c, a, t) = setup6("Focus Sash", "");
    let q = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .first()
        .cloned();
    assert!(q.is_none());
    let target_owner = p(&c, &a).owner_id.clone();
    let (mut qcampaign, source, target) = super::super::special_tests::setup("Tackle");
    let hitc = hit(&qcampaign, &source, &target, "Tackle");
    let mut q = hitc.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].clone();
    q.actor_id = t;
    q.target_id = a.clone();
    assert_eq!(ci::survive(&mut c, &q, 3, 7), 2);
    assert_eq!(ci::survive(&mut c, &q, 3, 7), 7);
    let _ = (&mut qcampaign, target_owner);
}
#[test]
fn completion_medicine_doses_are_owned_action_costed_persistent_and_reversible() {
    let (mut c, a, _) = setup6("Leftovers", "");
    stock(&mut c, &a, "Super Potion");
    let sid = p(&c, &a).sheet_id.clone();
    let before = c.sheets[&sid].clone();
    let command = use_med(&c, &a, &a, "Super Potion", 2);
    assert!(battle::transition(&c, "other", &cmd(command.clone()), vec![], &catalog()).is_err());
    let c = run(&c, "gm", command);
    assert_eq!(p(&c, &a).hp, 14);
    assert_eq!(p(&c, &a).actions, 1);
    assert_eq!(c.sheets[&sid].inventory["Super Potion"], 1);
    assert_eq!(c.sheets[&sid].stored_items.as_ref().unwrap()[0].uses, 2);
    let copy: Campaign = serde_json::from_value(serde_json::to_value(&c).unwrap()).unwrap();
    assert_eq!(copy.sheets[&sid].stored_items, c.sheets[&sid].stored_items);
    let undo = Action::RollbackBattle {
        event_sequence: c.battle_undo.last().unwrap().event_sequence,
        reason: "Undo dose and cost".into(),
    };
    let undone = battle::transition(&c, "gm", &undo, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(undone.sheets[&sid].inventory, before.inventory);
    assert_eq!(undone.sheets[&sid].stored_items, before.stored_items);
    assert_eq!(p(&undone, &a).hp, 12);
}
#[test]
fn completion_full_restore_lockout_survives_end_and_blocks_every_healing_channel() {
    let (mut c, a, _) = setup6("Leftovers", "");
    stock(&mut c, &a, "Full Restore");
    let command = use_med(&c, &a, &a, "Full Restore", 1);
    let c = run(&c, "gm", command);
    assert_eq!(p(&c, &a).hp, 20);
    assert!(ci::healing_locked(&c, &a));
    assert!(special::healing_blocked(&c, &a));
    let sid = p(&c, &a).sheet_id.clone();
    let ended = battle::transition(&c, "gm", &Action::EndBattle, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(
        ended.sheets[&sid].medical.as_ref().unwrap().healing_until,
        1440
    );
    let mut advanced = c.clone();
    advanced.ability_inheritance.minutes = 1440;
    assert!(!ci::healing_locked(&advanced, &a));
}
#[test]
fn completion_revives_preserve_reserve_identity_and_require_the_full_hp_ruling() {
    let (mut c, a, t) = setup6("Leftovers", "");
    stock(&mut c, &a, "Max Revive");
    let side = p(&c, &a).side.clone();
    pm(&mut c, &t).side = side;
    pm(&mut c, &t).hp = 0;
    pm(&mut c, &t).deployed = false;
    let command = use_med(&c, &a, &t, "Max Revive", 1);
    assert!(battle::transition(&c, "gm", &cmd(command.clone()), vec![], &catalog()).is_err());
    let c = rule(&c, "revive", "full");
    let c = run(&c, "gm", command);
    assert_eq!(p(&c, &t).hp, 20);
    assert!(!p(&c, &t).deployed);
}
#[test]
fn completion_herbal_gm_ruling_and_gluttony_do_not_invent_a_happiness_penalty() {
    for ability in ["", "Gluttony"] {
        let (mut c, a, _) = setup6("Leftovers", ability);
        stock(&mut c, &a, "Remedy");
        let command = use_med(&c, &a, &a, "Remedy", 2);
        let c = if ability.is_empty() {
            assert!(
                battle::transition(&c, "gm", &cmd(command.clone()), vec![], &catalog()).is_err()
            );
            rule(&c, &format!("herbal:{a}"), "accepted")
        } else {
            c
        };
        let sid = p(&c, &a).sheet_id.clone();
        let loyalty = c.sheets[&sid].workflow.as_ref().map(|w| w.loyalty);
        let c = run(&c, "gm", command);
        assert_eq!(p(&c, &a).hp, 14);
        assert_eq!(c.sheets[&sid].workflow.as_ref().map(|w| w.loyalty), loyalty);
    }
}
fn foe_attack(mut c: Campaign, a: &str, t: &str, name: &str) -> Campaign {
    learn(&mut c, t, name);
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.queue.clear();
    f.window = false;
    f.reacted.clear();
    f.cursor = f.order.iter().position(|id| id == t).unwrap();
    let attack = super::super::targeting_tests::attack(t, vec![a.into()], name);
    let c = roll(&c, "gm", &attack, 4);
    let c = pass(&c);
    roll(&c, "gm", &damage_command(&c), 4)
}
#[test]
fn completion_move_damage_pops_balloon_and_focus_sash_survives_only_once() {
    let (c, a, t) = setup6("Air Balloon", "");
    let c = foe_attack(c, &a, &t, "Tackle");
    assert!(items::held(&c, &a).is_empty());
    assert_eq!(
        c.battle
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
    for name in ["Tackle", "Sonic Boom", "Dragon Rage"] {
        let (mut c, a, t) = setup6("Focus Sash", "");
        pm(&mut c, &a).hp = 1;
        let c = foe_attack(c, &a, &t, name);
        assert_eq!(p(&c, &a).hp, 1, "{name}");
        assert!(c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .completion
            .used
            .contains("sash:berry"));
    }
}
#[test]
fn completion_contact_helmet_barb_pads_and_policy_follow_real_hits() {
    let (c, a, t) = setup6("Rocky Helmet", "");
    let hp = p(&c, &t).hp;
    let c = foe_attack(c, &a, &t, "Tackle");
    assert_eq!(p(&c, &t).hp, hp - 1);
    let (c, a, t) = setup6("Sticky Barb", "");
    let c = foe_attack(c.clone(), &a, &t, "Tackle");
    assert!(items::held(&c, &a).is_empty());
    assert!(items::held(&c, &t).iter().any(|u| u.id == "berry"));
    let (mut c, a, t) = setup6("Protective Pads", "");
    pm(&mut c, &t).selected_ability = "Static".into();
    let c = hit(&c, &a, &t, "Tackle");
    let c = pass(&c);
    let c = roll(&c, "player", &damage_command(&c), 4);
    assert!(!c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events
        .iter()
        .any(|e| e.kind == "contact"));
    let (mut c, a, t) = setup6("Weakness Policy", "");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some(
        catalog().find("species", "Name", "Eevee").unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
    let c = foe_attack(c, &a, &t, "Karate Chop");
    assert_eq!(p(&c, &a).runtime.ability.counters["boost:Strength"], 1);
    assert_eq!(p(&c, &a).runtime.ability.counters["boost:Special"], 1);
}
#[test]
fn completion_destiny_knot_reflects_sourced_love_once_and_throat_spray_uses_sound() {
    let (mut c, a, t) = setup6("Destiny Knot", "");
    abilities::inflict(&mut c, &t, &a, "In Love", &t, None, &catalog()).unwrap();
    assert!(p(&c, &a)
        .runtime
        .statuses
        .iter()
        .any(|s| s.kind == "In Love"));
    assert!(p(&c, &t)
        .runtime
        .statuses
        .iter()
        .any(|s| s.kind == "In Love"));
    let (mut c, a, t) = setup6("Throat Spray", "");
    learn(&mut c, &a, "Growl");
    let c = hit(&c, &a, &t, "Growl");
    assert_eq!(p(&c, &a).runtime.ability.counters["boost:Special"], 1);
}
#[test]
fn completion_eject_and_red_card_create_once_per_unit_round_requests() {
    for item in ["Eject Button", "Red Card"] {
        let (c, a, t) = setup6(item, "");
        let c = foe_attack(c, &a, &t, "Tackle");
        let switches = &c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .tactics
            .switches;
        assert_eq!(switches.len(), 1, "{item}");
        assert_eq!(
            switches[0].outgoing,
            if item == "Eject Button" { a } else { t }
        );
    }
}
#[test]
fn completion_all_medicine_definitions_respect_cures_doses_and_fainting() {
    for name in ci::medicine_names() {
        let (mut c, a, t) = setup6("Leftovers", "Gluttony");
        stock(&mut c, &a, name);
        let target = if ["Revive", "Max Revive", "Revival Herb"].contains(name) {
            let side = p(&c, &a).side.clone();
            pm(&mut c, &t).side = side;
            pm(&mut c, &t).hp = 0;
            pm(&mut c, &t).deployed = false;
            t.clone()
        } else {
            a.clone()
        };
        pm(&mut c, &target).selected_ability = "Gluttony".into();
        let c = rule(&c, "revive", "full");
        let command = use_med(&c, &a, &target, name, 1);
        let c = run(&c, "gm", command);
        assert!(p(&c, &target).hp > 0, "{name}");
    }
    let (mut c, a, t) = setup6("Leftovers", "");
    stock(&mut c, &a, "Potion");
    let side = p(&c, &a).side.clone();
    pm(&mut c, &t).side = side;
    pm(&mut c, &t).hp = 0;
    let command = use_med(&c, &a, &t, "Potion", 1);
    assert!(battle::transition(&c, "gm", &cmd(command), vec![], &catalog()).is_err());
    let (mut c, a, _) = setup6("Leftovers", "");
    stock(&mut c, &a, "Antidote");
    abilities::inflict(&mut c, &a, &a, "Badly Poisoned", "", None, &catalog()).unwrap();
    let command = use_med(&c, &a, &a, "Antidote", 1);
    let c = run(&c, "gm", command);
    assert!(!p(&c, &a)
        .runtime
        .statuses
        .iter()
        .any(|s| s.kind.contains("Poison")));
}
#[test]
fn completion_small_pocket_and_trainer_costs_are_authoritative() {
    let (mut c, a, _) = setup6("Leftovers", "");
    stock(&mut c, &a, "Potion");
    let command = use_med(&c, &a, &a, "Potion", 1);
    assert!(battle::transition(&c, "player", &cmd(command.clone()), vec![], &catalog()).is_err());
    let key = format!("pocket:{}", p(&c, &a).sheet_id);
    let c = rule(&c, &key, "accessible");
    let c = run(&c, "player", command);
    assert_eq!(p(&c, &a).hp, 13);
    let (mut c, a, _) = setup6("Leftovers", "");
    let sid = p(&c, &a).sheet_id.clone();
    let mut tr = c.sheets[&sid].clone();
    tr.id = "medicine-trainer".into();
    tr.kind = "trainer".into();
    tr.held_items = None;
    tr.inventory.insert("Potion".into(), 1);
    c.sheets.get_mut(&sid).unwrap().trainer_id = Some(tr.id.clone());
    c.sheets.insert(tr.id.clone(), tr);
    let command = CompletionCommand::Medicine {
        combatant_id: a.clone(),
        target_id: a.clone(),
        supply_sheet_id: "medicine-trainer".into(),
        unit_id: None,
        name: "Potion".into(),
        dose: 1,
        trainer_on_field: true,
    };
    assert!(battle::transition(&c, "player", &cmd(command.clone()), vec![], &catalog()).is_err());
    c.battle
        .as_mut()
        .unwrap()
        .trainer_actions
        .insert("medicine-trainer".into(), 3);
    let c = run(&c, "gm", command);
    assert_eq!(p(&c, &a).actions, 0);
    assert_eq!(
        c.battle.as_ref().unwrap().trainer_actions["medicine-trainer"],
        4
    );
}
#[test]
fn completion_recycle_revives_an_owned_reserve_without_creating_new_stock() {
    let (mut c, a, t) = setup6("Leftovers", "");
    stock(&mut c, &a, "Revive");
    learn(&mut c, &a, "Recycle");
    let side = p(&c, &a).side.clone();
    pm(&mut c, &t).side = side;
    pm(&mut c, &t).hp = 0;
    pm(&mut c, &t).deployed = false;
    let owner = p(&c, &a).owner_id.clone();
    pm(&mut c, &t).owner_id = owner.clone();
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().owner_id = owner;
    let command = use_med(&c, &a, &t, "Revive", 1);
    let mut c = run(&c, "gm", command);
    pm(&mut c, &t).hp = 0;
    pm(&mut c, &t).runtime.healed = 0;
    let used = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .units
        .values()
        .find(|u| u.unit.name == "Revive")
        .unwrap()
        .unit
        .id
        .clone();
    let c = run(
        &c,
        "player",
        CompletionCommand::RecycleRecipient {
            combatant_id: a.clone(),
            target_id: t.clone(),
        },
    );
    let mut c = decision(
        &c,
        "player",
        action(ItemCommand::Recycle {
            combatant_id: a.clone(),
            unit_id: used.clone(),
        }),
    );
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.cursor = f.order.iter().position(|id| id == &a).unwrap();
    f.window = false;
    let c = hit(&c, &a, &a, "Recycle");
    let c = roll(&c, "gm", &orb_effect(&c), 6);
    assert_eq!(p(&c, &t).hp, 1);
    assert!(!p(&c, &t).deployed);
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .units[&used]
            .state,
        "consumed"
    );
    assert_eq!(c.sheets[&p(&c, &a).sheet_id].inventory["Revive"], 1);
}
#[test]
fn completion_new_state_is_private_in_snapshots_archives_undo_and_missing_fields() {
    let (mut c, a, t) = setup6("Choice Scarf", "");
    learn(&mut c, &a, "Tackle");
    let c = select(&c, &a, "Tackle");
    let c = rule(&c, "scarf:berry", "item");
    let other = crate::privacy::for_player(&c, "other");
    let own = crate::privacy::for_player(&c, "player");
    assert!(other
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .items
        .completion
        .choices
        .is_empty());
    assert_eq!(
        own.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .completion
            .choices["berry"],
        "Tackle"
    );
    assert_eq!(
        own.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .completion
            .rulings["scarf:berry"],
        "item"
    );
    let mut data = serde_json::to_value(&c).unwrap();
    data["battle"]["flow"]["items"]
        .as_object_mut()
        .unwrap()
        .remove("completion");
    let old: Campaign = serde_json::from_value(data).unwrap();
    assert!(!ci::enabled(&old));
    let _ = t;
}
#[tokio::test]
async fn completion_room_medicine_permissions_receipts_reconnect_and_rollback() {
    room_fixture(false).await;
}
async fn room_fixture(group: bool) {
    use crate::{room, store::Store};
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };
    async fn send(r: &room::Remote, c: Command) -> Result<Campaign> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        r.sender.send((c, tx)).await.unwrap();
        tokio::time::timeout(Duration::from_secs(5), rx)
            .await
            .unwrap()
            .unwrap()
    }
    let dirs: Vec<_> = (0..4).map(|_| tempfile::tempdir().unwrap()).collect();
    let stores: Vec<_> = dirs
        .iter()
        .map(|d| Arc::new(Mutex::new(Store::open(d.path()).unwrap())))
        .collect();
    let ids: Vec<_> = stores
        .iter()
        .map(|s| s.lock().unwrap().player_id.clone())
        .collect();
    let (mut c, a, _) = setup6("Leftovers", "");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .house_rules = group;
    stock(&mut c, &a, "Super Potion");
    c.host_id = ids[0].clone();
    c.gm_id = ids[2].clone();
    pm(&mut c, &a).owner_id = ids[1].clone();
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().owner_id = ids[1].clone();
    stores[0].lock().unwrap().put_campaign(&c).unwrap();
    let cat = Arc::new(catalog());
    let mut hosted = room::start(
        stores[0].clone(),
        cat.clone(),
        c.id.clone(),
        0,
        Arc::new(|_, _| {}),
    )
    .await
    .unwrap();
    let request = JoinRequest {
        address: hosted.info.address.replace("<host-ip>", "127.0.0.1"),
        token: hosted.info.token.clone(),
    };
    let (owner, _) = room::join(
        stores[1].clone(),
        cat.clone(),
        request.clone(),
        Arc::new(|_, _| {}),
    )
    .await
    .unwrap();
    let (gm, _) = room::join(
        stores[2].clone(),
        cat.clone(),
        request.clone(),
        Arc::new(|_, _| {}),
    )
    .await
    .unwrap();
    let (observer, view) = room::join(
        stores[3].clone(),
        cat.clone(),
        request.clone(),
        Arc::new(|_, _| {}),
    )
    .await
    .unwrap();
    assert!(view.sheets[&sid].inventory.is_empty());
    let pocket = Command {
        id: "pocket-ruling".into(),
        expected_revision: c.revision,
        action: cmd(CompletionCommand::Rule {
            key: format!("pocket:{sid}"),
            value: "accessible".into(),
        }),
    };
    assert!(send(&owner, pocket.clone()).await.is_err());
    let ready = send(&gm, pocket).await.unwrap();
    let medicine = Command {
        id: "medicine-once".into(),
        expected_revision: ready.revision,
        action: cmd(use_med(&ready, &a, &a, "Super Potion", 2)),
    };
    assert!(send(&observer, medicine.clone()).await.is_err());
    let done = send(&owner, medicine.clone()).await.unwrap();
    assert_eq!(
        done.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .house_rules,
        group
    );
    assert_eq!(p(&done, &a).hp, 14);
    assert_eq!(done.sheets[&sid].inventory["Super Potion"], 1);
    assert_eq!(
        send(&owner, medicine.clone()).await.unwrap().revision,
        done.revision
    );
    let mut stale = medicine;
    stale.id = "stale-medicine".into();
    assert_eq!(send(&owner, stale).await.unwrap_err().code, "stale");
    owner.task.abort();
    let _ = owner.task.await;
    let (owner, reconnected) = room::join(
        stores[1].clone(),
        cat.clone(),
        request.clone(),
        Arc::new(|_, _| {}),
    )
    .await
    .unwrap();
    assert_eq!(
        reconnected.sheets[&sid].stored_items.as_ref().unwrap()[0].uses,
        2
    );
    assert_eq!(
        reconnected
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .house_rules,
        group
    );
    let saved = stores[0].lock().unwrap().campaigns().unwrap().remove(0);
    assert_eq!(saved.sheets[&sid].inventory["Super Potion"], 1);
    let undone = send(
        &gm,
        Command {
            id: "undo-medicine".into(),
            expected_revision: done.revision,
            action: Action::RollbackBattle {
                event_sequence: done.battle_undo.last().unwrap().event_sequence,
                reason: "Restore doses, HP and payer".into(),
            },
        },
    )
    .await
    .unwrap();
    assert_eq!(p(&undone, &a).hp, 12);
    assert_eq!(undone.sheets[&sid].inventory["Super Potion"], 2);
    let _ = hosted.host.shutdown.send(true);
    owner.task.abort();
    gm.task.abort();
    observer.task.abort();
    let _ = owner.task.await;
    let _ = gm.task.await;
    let _ = observer.task.await;
    if let Some(stop) = hosted.stop.take() {
        let _ = stop.send(());
    }
    drop(hosted);
    tokio::time::timeout(Duration::from_secs(5), async {
        while stores.iter().any(|s| Arc::strong_count(s) > 1) {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
}

#[test]
fn completion_medicine_respects_shadow_force_debt_without_spending_excess_doses() {
    let (mut c, a, _) = setup6("Leftovers", "");
    stock(&mut c, &a, "Super Potion");
    let hp = p(&c, &a).hp;
    let debt = p(&c, &a).max_hp - hp - 1;
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.special.enabled = true;
    f.special.effects.push(special::TimedEffect {
        amount: debt,
        name: "Shadow Force".into(),
        source: a.clone(),
        target: a.clone(),
        rounds: None,
        started: 1,
        slot: String::new(),
    });
    let command = use_med(&c, &a, &a, "Super Potion", 2);
    assert!(battle::transition(&c, "gm", &cmd(command), vec![], &catalog()).is_err());
    let command = use_med(&c, &a, &a, "Super Potion", 1);
    let next = run(&c, "gm", command);
    assert_eq!(p(&next, &a).hp, hp + 1);
    assert_eq!(
        next.sheets[&p(&next, &a).sheet_id]
            .stored_items
            .as_ref()
            .unwrap()[0]
            .uses,
        1
    );
}

#[test]
fn completion_recycled_balloon_does_not_stack_and_fainting_damage_pops_it() {
    let (mut c, a, _) = setup6("Air Balloon", "");
    let unit = items::held(&c, &a)[0].clone();
    c = rule(&c, "recycle-balloon", "until-damage");
    ci::consume(&mut c, &a, "Air Balloon", &catalog()).unwrap();
    ci::recycle_held(&mut c, &a, &unit).unwrap();
    ci::recycle_held(&mut c, &a, &unit).unwrap();
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .items
            .completion
            .recycled[&a]["Air Balloon"],
        1
    );
    let before = c.clone();
    pm(&mut c, &a).hp = 0;
    ci::lifecycle(&before, &mut c, &catalog()).unwrap();
    assert!(!ci::has(&c, &a, "Air Balloon"));
}

#[test]
fn completion_medicine_checkpoints_include_linked_trainers_without_unrelated_inventory() {
    let (mut c, a, _) = setup6("Leftovers", "");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().trainer_id = Some("linked-trainer".into());
    let mut unrelated = c.sheets[&sid].clone();
    unrelated.id = "unrelated".into();
    unrelated.inventory.insert("Potion".into(), 99);
    c.sheets.insert(unrelated.id.clone(), unrelated);
    let ids = items::resource_sheets(&c, &c);
    assert!(ids.contains(&sid));
    assert!(ids.contains("linked-trainer"));
    assert!(!ids.contains("unrelated"));
}

#[path = "house_tests.rs"]
mod house_tests;
