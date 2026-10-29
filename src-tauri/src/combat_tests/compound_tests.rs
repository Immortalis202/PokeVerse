use super::targeting_tests::{damage_command, pass, roll};
use super::*;
use crate::combat::{compound, field, items, special, Status};

fn pm<'a>(c: &'a mut Campaign, id: &str) -> &'a mut Combatant {
    c.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == id)
        .unwrap()
}
fn p<'a>(c: &'a Campaign, id: &str) -> &'a Combatant {
    c.battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .find(|p| p.id == id)
        .unwrap()
}
fn setup(name: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = special_tests::setup(name);
    let round = c.battle.as_ref().unwrap().round;
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.compound.enabled = true;
    f.compound.entered.insert(a.clone(), round);
    f.compound.environments = vec!["City/Building".into()];
    f.status_escalation = true;
    f.reviewed_rules = true;
    f.interaction_rules = true;
    for id in [&a, &t] {
        pm(&mut c, id).hp = 100;
        pm(&mut c, id).max_hp = 100;
    }
    species(&mut c, &t, "Shuckle");
    (c, a, t)
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
fn q(c: &Campaign) -> crate::combat::QueuedAction {
    c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].clone()
}
fn effect(c: &Campaign) -> Action {
    Action::ResolveEffect {
        sequence: q(c).sequence,
        reason: "Reviewed compound effect".into(),
    }
}
fn pending(c: &Campaign, a: &str, t: &str, name: &str) -> Campaign {
    let mut c = hit(c, a, t, name);
    if c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
        c = pass(&c);
    }
    if catalog().find("moves", "Name", name).unwrap()["Category"] != "Support" {
        c = roll(&c, "gm", &damage_command(&c), 1);
    }
    c
}
fn resolve(c: &Campaign, faces: Vec<u8>) -> Campaign {
    battle::transition(c, "gm", &effect(c), faces, &catalog())
        .unwrap()
        .0
}
fn has(c: &Campaign, id: &str, kind: &str) -> bool {
    p(c, id).runtime.statuses.iter().any(|s| s.kind == kind)
}
fn status(c: &mut Campaign, id: &str, kind: &str) {
    pm(c, id).runtime.statuses.push(Status {
        kind: kind.into(),
        detail: "Fixture".into(),
        rounds: None,
        progress: 0,
        elapsed_rounds: 0,
    });
}
fn command(command: compound::CompoundCommand) -> Action {
    Action::CompoundMove {
        command,
        reason: "Reviewed owner/GM decision".into(),
    }
}

#[test]
fn compound_all_28_cards_resolve_printed_status_and_stat_clauses() {
    let moves = compound::registry()["moves"].as_object().unwrap();
    assert_eq!(moves.len(), 28);
    for (name, d) in moves {
        let (mut c, a, t) = setup(name);
        if name == "Burning Jealousy" {
            pm(&mut c, &t)
                .runtime
                .modifiers
                .insert("Strength".into(), 1);
        }
        if name == "Venom Drench" {
            status(&mut c, &t, "Poison");
        }
        let mut c = pending(&c, &a, &t, name);
        if name == "Upper Hand" {
            c.battle.as_mut().unwrap().flow.as_mut().unwrap().queue[0].response_to = Some(123);
        }
        let n = rules::dice_count(&c, &effect(&c), &catalog()).unwrap();
        let c = resolve(&c, vec![6; n as usize]);
        for pool in d["effects"].as_array().unwrap() {
            let kind = if pool["kind"] == "environment" {
                "Paralysis"
            } else {
                pool["kind"].as_str().unwrap()
            };
            assert!(has(&c, &t, kind), "{name}: {kind}");
        }
        for change in d["changes"].as_array().unwrap() {
            assert_eq!(
                p(&c, &t)
                    .runtime
                    .modifiers
                    .get(change["key"].as_str().unwrap())
                    .copied(),
                Some(change["amount"].as_i64().unwrap() as i32),
                "{name}"
            );
        }
        if name == "Venom Drench" {
            for key in ["Strength", "Special", "Dexterity"] {
                assert_eq!(p(&c, &t).runtime.modifiers[key], -1);
            }
        }
    }
}

#[test]
fn compound_independent_pools_use_separate_slices_and_can_all_trigger() {
    for (name, kinds) in [
        ("Dire Claw", vec!["Poison", "Paralysis", "Sleep"]),
        ("Tri Attack", vec!["Burn", "Frozen Solid", "Paralysis"]),
        ("Fire Fang", vec!["Flinch", "Burn"]),
        ("Ice Fang", vec!["Flinch", "Frozen Solid"]),
        ("Thunder Fang", vec!["Flinch", "Paralysis"]),
    ] {
        let (c, a, t) = setup(name);
        let c = pending(&c, &a, &t, name);
        assert_eq!(
            rules::dice_count(&c, &effect(&c), &catalog()).unwrap(),
            2 * kinds.len() as u32
        );
        for selected in 0..kinds.len() {
            let mut dice = vec![5; 2 * kinds.len()];
            dice[2 * selected] = 6;
            let done = resolve(&c, dice);
            for (i, kind) in kinds.iter().enumerate() {
                assert_eq!(has(&done, &t, kind), i == selected, "{name}: {kind}");
            }
        }
    }
}

#[test]
fn compound_venom_drench_watches_same_round_poison_once_and_expires() {
    let (c, a, t) = setup("Venom Drench");
    let c = pending(&c, &a, &t, "Venom Drench");
    let c = resolve(&c, vec![]);
    assert!(!p(&c, &t).runtime.modifiers.contains_key("Strength"));
    let poison = Action::SetStatus {
        combatant_id: t.clone(),
        kind: "Poison".into(),
        detail: "Later in round".into(),
        rounds: None,
        remove: false,
        reason: "Poison later".into(),
    };
    let done = decision(&c, "gm", poison.clone());
    assert_eq!(p(&done, &t).runtime.modifiers["Strength"], -1);
    let done = decision(&done, "gm", poison.clone());
    assert_eq!(p(&done, &t).runtime.modifiers["Strength"], -1);
    let mut later = c.clone();
    later.battle.as_mut().unwrap().round += 1;
    let later = decision(&later, "gm", poison);
    assert!(!p(&later, &t).runtime.modifiers.contains_key("Strength"));
}

#[test]
fn compound_entry_round_and_final_evolution_guards_use_current_identity() {
    for name in [
        "Buzzy Buzz",
        "Sizzly Slide",
        "Floaty Fall",
        "Splishy Splash",
    ] {
        let (mut c, a, _) = setup(name);
        assert!(compound::guard(&c, &a, name, &catalog()).is_ok());
        species(&mut c, &a, "Venusaur");
        assert!(compound::guard(&c, &a, name, &catalog()).is_err());
    }
    let (mut c, a, _) = setup("Fake Out");
    assert!(compound::guard(&c, &a, "Fake Out", &catalog()).is_ok());
    c.battle.as_mut().unwrap().round = 2;
    assert!(compound::guard(&c, &a, "Fake Out", &catalog()).is_err());
    let before = c.clone();
    pm(&mut c, &a).deployed = false;
    compound::lifecycle(&before, &mut c, &Action::NextRound, &catalog()).unwrap();
    let before = c.clone();
    pm(&mut c, &a).deployed = true;
    compound::lifecycle(&before, &mut c, &Action::NextRound, &catalog()).unwrap();
    assert!(compound::guard(&c, &a, "Fake Out", &catalog()).is_ok());
}

#[test]
fn compound_axe_kick_miss_requires_owned_four_dice_and_is_rollbackable() {
    let (c, a, t) = setup("Axe Kick");
    let accuracy = Action::Accuracy {
        combatant_id: a.clone(),
        target_id: Some(t),
        move_name: "Axe Kick".into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Miss deliberately".into(),
    };
    let c = roll(&c, "player", &accuracy, 1);
    let action = command(compound::CompoundCommand::Crash {
        combatant_id: a.clone(),
    });
    assert_eq!(rules::dice_count(&c, &action, &catalog()).unwrap(), 4);
    assert!(rules::dice_count(&c, &Action::NextRound, &catalog()).is_err());
    assert!(battle::transition(&c, "other", &action, vec![6; 4], &catalog()).is_err());
    let reloaded: Campaign = serde_json::from_value(serde_json::to_value(&c).unwrap()).unwrap();
    let done = roll(&reloaded, "player", &action, 6);
    assert_eq!(p(&done, &a).hp, 96);
    let restored = decision(
        &done,
        "gm",
        Action::RollbackBattle {
            event_sequence: done.battle_undo.last().unwrap().event_sequence,
            reason: "Undo crash".into(),
        },
    );
    assert_eq!(p(&restored, &a).hp, 100);
    assert_eq!(
        restored
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .compound
            .crashes,
        vec![a]
    );
}

#[test]
fn compound_secret_power_uses_printed_environment_and_trainer_alignment() {
    let (mut c, a, t) = setup("Secret Power");
    c = decision(
        &c,
        "gm",
        command(compound::CompoundCommand::Environment {
            sources: vec!["Lake/River/Sea".into(), "Forest/Jungle".into()],
        }),
    );
    assert!(compound::guard(&c, &a, "Secret Power", &catalog()).is_err());
    assert!(battle::transition(
        &c,
        "other",
        &command(compound::CompoundCommand::Align {
            combatant_id: a.clone(),
            source: "Forest/Jungle".into()
        }),
        vec![],
        &catalog()
    )
    .is_err());
    c = decision(
        &c,
        "player",
        command(compound::CompoundCommand::Align {
            combatant_id: a.clone(),
            source: "Lake/River/Sea".into(),
        }),
    );
    let queued = pending(&c, &a, &t, "Secret Power");
    let done = resolve(&queued, vec![6; 3]);
    assert_eq!(p(&done, &t).runtime.modifiers["Strength"], -2);
    let mut terrain = c.clone();
    let f = terrain.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.compound.environments.clear();
    f.compound.alignments.clear();
    f.battlefield.rules.push(field::FieldRule {
        id: id(),
        kind: "Psychic Terrain".into(),
        source: a.clone(),
        scope: "all".into(),
        rounds: None,
        parameter: 0,
        started_round: 1,
    });
    let done = resolve(&pending(&terrain, &a, &t, "Secret Power"), vec![6; 3]);
    assert!(has(&done, &t, "Confusion"));
}

#[test]
fn compound_lovely_kiss_missing_beauty_and_hypnosis_dark_target() {
    let (mut c, a, t) = setup("Lovely Kiss");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .stats
        .insert("Beauty".into(), 0);
    let action = Action::Accuracy {
        combatant_id: a.clone(),
        target_id: Some(t.clone()),
        move_name: "Lovely Kiss".into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Missing Beauty pool".into(),
    };
    assert_eq!(rules::dice_count(&c, &action, &catalog()).unwrap(), 5);
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .stats
        .insert("Beauty".into(), 5);
    assert_eq!(rules::dice_count(&c, &action, &catalog()).unwrap(), 0);
    let (mut c, a, t) = setup("Hypnosis");
    species(&mut c, &t, "Umbreon");
    let c = pending(&c, &a, &t, "Hypnosis");
    let c = resolve(&c, vec![]);
    assert!(has(&c, &t, "Sleep"));
}

#[test]
fn compound_shell_side_arm_category_changes_both_accuracy_and_damage_and_hydro_steam_ignores_sun_penalty(
) {
    let (mut c, a, _) = setup("Shell Side Arm");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .stats
        .insert("Strength".into(), 5);
    let raw = catalog().find("moves", "Name", "Shell Side Arm").unwrap();
    let m = field::move_data(&c, p(&c, &a), &raw);
    assert_eq!(m["Category"], "Physical");
    assert_eq!(m["Accuracy1"], "Strength");
    assert_eq!(m["Accuracy2"], "Brawl");
    assert_eq!(m["Damage1"], "Strength");
    let (mut c, a, t) = setup("Hydro Steam");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .natural = Some(field::Weather::Sunny);
    let raw = catalog().find("moves", "Name", "Hydro Steam").unwrap();
    assert_eq!(field::bonus(&c, p(&c, &a), &raw), 2);
    assert_eq!(field::reduction(&c, p(&c, &a), p(&c, &t), &raw), 0);
}

#[test]
fn compound_mortal_spin_cleans_only_users_side_and_retains_ailments() {
    let (mut c, a, t) = setup("Mortal Spin");
    species(&mut c, &t, "Magnemite");
    status(&mut c, &a, "Burn");
    let side = p(&c, &a).side.clone();
    let enemy = p(&c, &t).side.clone();
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    for (kind, scope) in [
        ("Toxic Spikes", format!("side:{side}")),
        ("Spikes", format!("side:{enemy}")),
        ("Grassy Terrain", "all".into()),
    ] {
        f.battlefield.rules.push(field::FieldRule {
            id: id(),
            kind: kind.into(),
            scope,
            source: t.clone(),
            rounds: None,
            parameter: 0,
            started_round: 1,
        });
    }
    f.special.effects.push(special::TimedEffect {
        amount: 0,
        name: "Leech Seed".into(),
        source: t.clone(),
        target: a.clone(),
        rounds: None,
        started: 1,
        slot: "".into(),
    });
    let c = pending(&c, &a, &t, "Mortal Spin");
    let c = resolve(&c, vec![]);
    let f = c.battle.as_ref().unwrap().flow.as_ref().unwrap();
    assert_eq!(f.battlefield.rules.len(), 1);
    assert_eq!(f.battlefield.rules[0].scope, format!("side:{enemy}"));
    assert!(f.special.effects.is_empty());
    assert!(has(&c, &a, "Burn"));
    let (mut c, a, t) = setup("Mortal Spin");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .rules
        .push(field::FieldRule {
            id: id(),
            kind: "Misty Terrain".into(),
            source: a.clone(),
            scope: "all".into(),
            rounds: None,
            parameter: 0,
            started_round: 1,
        });
    let c = pending(&c, &a, &t, "Mortal Spin");
    let c = resolve(&c, vec![]);
    assert!(
        !has(&c, &t, "Poison"),
        "Poison resolves before terrain removal"
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
        .is_empty());
    let (mut c, a, t) = setup("Mortal Spin");
    let mut third = p(&c, &t).clone();
    let mut sheet = c.sheets[&third.sheet_id].clone();
    sheet.id = id();
    third.sheet_id = sheet.id.clone();
    third.id = id();
    let third_id = third.id.clone();
    c.sheets.insert(sheet.id.clone(), sheet);
    c.battle.as_mut().unwrap().combatants.push(third);
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.tactics.enabled = true;
    f.battlefield.rules.push(field::FieldRule {
        id: id(),
        kind: "Misty Terrain".into(),
        source: a.clone(),
        scope: "all".into(),
        rounds: None,
        parameter: 0,
        started_round: 1,
    });
    let action = targeting_tests::attack(&a, vec![t.clone(), third_id.clone()], "Mortal Spin");
    let c = roll(&c, "player", &action, 4);
    let mut c = pass(&pass(&c));
    for i in 0..2 {
        c = roll(&c, "gm", &damage_command(&c), 1);
        c = resolve(&c, vec![]);
        assert_eq!(
            c.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .battlefield
                .rules
                .is_empty(),
            i == 1
        );
    }
    assert!(!has(&c, &t, "Poison"));
    assert!(!has(&c, &third_id, "Poison"));
}

#[test]
fn compound_relic_song_form_choice_preserves_library_and_does_not_heal() {
    let (mut c, a, t) = setup("Relic Song");
    species(&mut c, &a, "Meloetta");
    let sid = p(&c, &a).sheet_id.clone();
    let dex = catalog().find("species", "Name", "Meloetta").unwrap();
    let sheet = c.sheets.get_mut(&sid).unwrap();
    sheet.rank = "Ace".into();
    sheet.workflow = None;
    for key in crate::creation::ATTRS {
        sheet
            .stats
            .insert(key.into(), dex[key].as_i64().unwrap() as i32);
    }
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .forms
        .enabled = true;
    let original = serde_json::to_value(&c.sheets[&sid]).unwrap();
    pm(&mut c, &a).selected_ability = "Serene Grace".into();
    pm(&mut c, &a).hp = 90;
    let c = pending(&c, &a, &t, "Relic Song");
    let c = resolve(&c, vec![6; 3]);
    let c = decision(
        &c,
        "player",
        command(compound::CompoundCommand::Relic {
            combatant_id: a.clone(),
            change: true,
        }),
    );
    assert_eq!(
        rules::base_sheet(&c, p(&c, &a))
            .unwrap()
            .species_id
            .as_deref(),
        Some("0648F1:meloetta-pirouette-form")
    );
    assert_eq!(p(&c, &a).hp, 90);
    assert_eq!(serde_json::to_value(&c.sheets[&sid]).unwrap(), original);
}

fn equip(c: &mut Campaign, a: &str, item: &str) {
    let sid = p(c, a).sheet_id.clone();
    let mut s = c.sheets[&sid].clone();
    s.inventory.insert(item.into(), 1);
    c.sheets.insert(
        sid,
        items::equip(&s, item, "compound-item", &catalog()).unwrap(),
    );
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.items.enabled = true;
    f.items.completion.enabled = true;
    items::initialize(c, &catalog()).unwrap();
    items::completion::initialize(c, &catalog());
}
#[test]
fn compound_pool_modifiers_and_item_flinch_keep_other_pool_slices_independent() {
    let (mut c, a, t) = setup("Dire Claw");
    equip(&mut c, &a, "Loaded Dice");
    pm(&mut c, &a).selected_ability = "Serene Grace".into();
    pm(&mut c, &t).selected_ability = "Wonder Skin".into();
    let c = pending(&c, &a, &t, "Dire Claw");
    assert_eq!(rules::dice_count(&c, &effect(&c), &catalog()).unwrap(), 12);
    let mut dice = vec![1; 12];
    dice[4] = 6;
    let done = resolve(&c, dice);
    assert!(has(&done, &t, "Paralysis"));
    assert!(!has(&done, &t, "Poison"));
    assert!(!has(&done, &t, "Sleep"));
    let (mut c, a, t) = setup("Fire Fang");
    equip(&mut c, &a, "Razor Fang");
    let c = pending(&c, &a, &t, "Fire Fang");
    assert_eq!(rules::dice_count(&c, &effect(&c), &catalog()).unwrap(), 5);
    let done = resolve(&c, vec![6, 1, 1, 1, 1]);
    assert!(has(&done, &t, "Flinch"));
    assert!(!has(&done, &t, "Burn"));
    let done = resolve(&c, vec![1, 1, 1, 6, 1]);
    assert!(!has(&done, &t, "Flinch"));
    assert!(has(&done, &t, "Burn"));
    let (mut c, a, t) = setup("Secret Power");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .compound
        .environments = vec!["Cave/Mountain".into()];
    equip(&mut c, &a, "Razor Fang");
    let c = pending(&c, &a, &t, "Secret Power");
    assert_eq!(rules::dice_count(&c, &effect(&c), &catalog()).unwrap(), 4);
    assert!(has(&resolve(&c, vec![6; 4]), &t, "Flinch"));
}
#[test]
fn compound_predicates_and_protections_leave_independent_guaranteed_changes() {
    let (c, a, t) = setup("Burning Jealousy");
    let c = pending(&c, &a, &t, "Burning Jealousy");
    assert!(!has(&resolve(&c, vec![]), &t, "Burn"));
    let (c, a, t) = setup("Upper Hand");
    let c = pending(&c, &a, &t, "Upper Hand");
    assert!(!has(&resolve(&c, vec![]), &t, "Flinch"));
    let (mut c, a, t) = setup("Flatter");
    pm(&mut c, &t).selected_ability = "Own Tempo".into();
    let c = pending(&c, &a, &t, "Flatter");
    let c = resolve(&c, vec![]);
    assert_eq!(p(&c, &t).runtime.modifiers["Special"], 1);
    assert!(!has(&c, &t, "Confusion"));
    let (mut c, a, t) = setup("Fire Fang");
    pm(&mut c, &t).selected_ability = "Shield Dust".into();
    let c = pending(&c, &a, &t, "Fire Fang");
    let c = resolve(&c, vec![6; 4]);
    assert!(p(&c, &t).runtime.statuses.is_empty());
    let (mut c, a, t) = setup("Stomp");
    c = decision(
        &c,
        "gm",
        command(compound::CompoundCommand::Minimized {
            combatant_id: t.clone(),
            active: true,
        }),
    );
    assert!(special::vulnerable(&c, &t, "Stomp"));
    let raw = catalog().find("moves", "Name", "Stomp").unwrap();
    assert_eq!(field::reduction(&c, p(&c, &a), p(&c, &t), &raw), -2);
}
#[test]
fn compound_old_flows_retain_manual_route_and_guards_are_atomic() {
    let (mut c, a, t) = setup("Dire Claw");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .compound
        .enabled = false;
    let c = pending(&c, &a, &t, "Dire Claw");
    assert_eq!(
        battle::transition(&c, "gm", &effect(&c), vec![], &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
    let (mut c, a, t) = setup("Sizzly Slide");
    species(&mut c, &a, "Venusaur");
    let before = serde_json::to_value(&c).unwrap();
    let action = Action::Accuracy {
        combatant_id: a,
        target_id: Some(t),
        move_name: "Sizzly Slide".into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Final stage cannot use this".into(),
    };
    assert!(battle::transition(&c, "player", &action, vec![6; 9], &catalog()).is_err());
    assert_eq!(serde_json::to_value(&c).unwrap(), before);
}

#[test]
fn compound_failed_axe_kick_clash_also_queues_crash() {
    let (mut c, a, t) = setup("Tackle");
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .moves
        .push("Axe Kick".into());
    let c = hit(&c, &a, &t, "Tackle");
    let clash = Action::React {
        combatant_id: t.clone(),
        kind: "clash".into(),
        move_name: "Axe Kick".into(),
        reason: "Failed Axe Kick Clash".into(),
    };
    let c = roll(&c, "other", &clash, 1);
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .compound
            .crashes,
        vec![t]
    );
}

#[test]
fn compound_crash_can_resolve_before_an_immediate_switch_request() {
    let (c, a, t) = setup("Axe Kick");
    let accuracy = Action::Accuracy {
        combatant_id: a.clone(),
        target_id: Some(t),
        move_name: "Axe Kick".into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Miss with an immediate switch pending".into(),
    };
    let mut c = roll(&c, "player", &accuracy, 1);
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.tactics.enabled = true;
    f.tactics
        .switches
        .push(crate::combat::tactics::SwitchRequest {
            id: id(),
            source: a.clone(),
            outgoing: a.clone(),
            name: "Emergency Exit".into(),
            round: 1,
            immediate: true,
            forced: false,
            ability: true,
        });
    let c = roll(
        &c,
        "player",
        &command(compound::CompoundCommand::Crash {
            combatant_id: a.clone(),
        }),
        6,
    );
    assert_eq!(p(&c, &a).hp, 96);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .compound
        .crashes
        .is_empty());
}

#[test]
fn compound_upper_hand_records_real_nested_tactics_reaction() {
    let (mut c, a, t) = setup("Tackle");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .moves
        .push("Upper Hand".into());
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .tactics
        .enabled = true;
    let c = hit(&c, &a, &t, "Tackle");
    let quick = Action::Tactics {
        command: crate::combat::tactics::TacticsCommand::Reaction {
            combatant_id: t.clone(),
            move_name: "Quick Attack".into(),
            target_ids: vec![a.clone()],
        },
        reason: "First reaction".into(),
    };
    let c = roll(&c, "other", &quick, 4);
    let incoming = q(&c).sequence;
    let upper = Action::Tactics {
        command: crate::combat::tactics::TacticsCommand::Reaction {
            combatant_id: a.clone(),
            move_name: "Upper Hand".into(),
            target_ids: vec![t.clone()],
        },
        reason: "Reaction to reaction".into(),
    };
    let mut c = roll(&c, "player", &upper, 4);
    assert_eq!(q(&c).move_name, "Upper Hand");
    assert_eq!(q(&c).response_to, Some(incoming));
    if c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
        c = pass(&c);
    }
    c = roll(&c, "gm", &damage_command(&c), 1);
    let c = resolve(&c, vec![]);
    assert!(has(&c, &t, "Flinch"));
}
