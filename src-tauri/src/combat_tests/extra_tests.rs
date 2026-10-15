use super::targeting_tests::{damage_command, pass, roll};
use super::*;
use crate::combat::{extra, field};
fn setup(name: &str, source: &str, target: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = fixture(name);
    let b = c.battle.as_mut().unwrap();
    let f = b.flow.as_mut().unwrap();
    f.ability_rules = true;
    f.extra.enabled = true;
    f.battlefield.enabled = true;
    f.battlefield.sunlight = true;
    for (i, p) in b.combatants.iter_mut().enumerate() {
        p.selected_ability = if i == 0 { source } else { target }.into();
        let s = c.sheets.get_mut(&p.sheet_id).unwrap();
        s.rank = "Standard".into();
        s.species_id = Some(
            catalog().find("species", "Name", "Eevee").unwrap()["AppId"]
                .as_str()
                .unwrap()
                .into(),
        );
        s.stats.extend(BTreeMap::from([
            ("Strength".into(), 4),
            ("Dexterity".into(), 4),
            ("Vitality".into(), 2),
            ("Special".into(), 4),
            ("Insight".into(), 3),
        ]));
        s.skills.extend(BTreeMap::from([
            ("Brawl".into(), 3),
            ("Channel".into(), 3),
            ("Nature".into(), 3),
        ]));
        s.moves.push("Protect".into());
    }
    (c, a, t)
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
fn score(c: &Campaign, id: &str, key: &str) -> i32 {
    rules::score(&rules::effective_sheet(c, p(c, id)).unwrap(), key).unwrap()
}
fn pending(name: &str, source: &str, target: &str) -> (Campaign, String, String) {
    let (c, a, t) = setup(name, source, target);
    let c = hit(&c, &a, &t, name);
    (pass(&c), a, t)
}
fn trigger(c: &Campaign, actor: &str, choice: &str, faces: Vec<u8>) -> Result<Campaign> {
    let e = &c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0];
    Ok(battle::transition(
        c,
        actor,
        &Action::ResolveAbility {
            event_id: e.id.clone(),
            choice: choice.into(),
            reason: "Reviewed trigger".into(),
        },
        faces,
        &catalog(),
    )?
    .0)
}
#[test]
fn extra_damage_families_use_reviewed_types_tags_and_final_reductions() {
    for (ability, tag, t, n) in [
        ("Dragon Maw", "", "Dragon", 2),
        ("Transistor", "", "Electric", 2),
        ("Water Bubble", "", "Water", 2),
        ("Rocky Payload", "", "Rock", 1),
        ("Steelworker", "", "Steel", 1),
        ("Strong Jaw", "BiteMove", "Normal", 1),
        ("Sharpness", "CutterMove", "Normal", 1),
        ("Wind Rider", "WindMove", "Normal", 1),
        ("Tough Claws", "", "Normal", 1),
        ("Punk Rock", "SoundMove", "Normal", 1),
    ] {
        let (c, a, tgt) = setup("Tackle", ability, "");
        let mut m =
            serde_json::json!({"Name":"Tackle","Type":t,"Category":"Physical","Attributes":{}});
        if !tag.is_empty() {
            m["Attributes"][tag] = serde_json::json!(true);
        }
        assert_eq!(extra::bonus(&c, p(&c, &a), p(&c, &tgt), &m), n, "{ability}");
    }
    let (c, a, t) = pending("Ember", "", "Ice Scales");
    let c = roll(&c, "player", &damage_command(&c), 4);
    assert_eq!(p(&c, &t).hp, 17);
    assert_eq!(p(&c, &a).hp, 20);
    let (c, a, t) = pending("Ember", "", "Fluffy");
    let c = roll(&c, "player", &damage_command(&c), 4);
    assert_eq!(p(&c, &t).hp, 13);
    assert_eq!(p(&c, &a).hp, 20);
    let (mut c, a, t) = pending("Bite", "Dark Aura", "Aura Break");
    c.battle.as_mut().unwrap().combatants[0]
        .stat_overrides
        .insert("Strength".into(), 1);
    c.battle.as_mut().unwrap().combatants[1]
        .stat_overrides
        .insert("Vitality".into(), 10);
    let seq = c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sequence;
    assert_eq!(test_pool(&c, &a, seq).unwrap().dice, 1);
    assert_eq!(p(&c, &t).hp, 20);
}
#[test]
fn extra_conversion_and_scrappy_apply_to_damage_stab_and_immunity() {
    for (ability, t) in [
        ("Aerilate", "Flying"),
        ("Pixilate", "Fairy"),
        ("Galvanize", "Electric"),
        ("Refrigerate", "Ice"),
        ("Normalize", "Normal"),
    ] {
        let (c, a, _) = setup("Tackle", ability, "");
        let m = field::move_data(
            &c,
            p(&c, &a),
            &catalog().find("moves", "Name", "Tackle").unwrap(),
        );
        assert_eq!(m["Type"], t);
    }
    let (mut c, a, t) = setup("Tackle", "Scrappy", "");
    field::change_type(&mut c, &t, "replace", &["Ghost".into()]).unwrap();
    let c = hit(&c, &a, &t, "Tackle");
    let c = pass(&c);
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    assert!(!test_pool(&c, &a, q.sequence).unwrap().immune);
    let (c, a, t) = pending("Tackle", "Aerilate", "");
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    assert_eq!(test_pool(&c, &a, q.sequence).unwrap().stab, 0);
    assert_eq!(p(&c, &t).hp, 20);
}
#[test]
fn extra_conditional_stats_and_ability_channels_do_not_duplicate_bonuses() {
    for (ability, kind, key, want) in [
        ("Berserk", "", "Special", 6),
        ("Anger Shell", "", "Strength", 5),
        ("Flare Boost", "Burn", "Special", 6),
        ("Toxic Boost", "Poison", "Strength", 6),
        ("Quick Feet", "Paralysis", "Dexterity", 6),
    ] {
        let (mut c, a, _) = setup("Tackle", ability, "");
        let p = c.battle.as_mut().unwrap().combatants.first_mut().unwrap();
        p.hp = 10;
        if !kind.is_empty() {
            p.runtime.statuses.push(crate::combat::Status {
                kind: kind.into(),
                detail: "".into(),
                rounds: None,
                progress: 0,
            });
        }
        assert_eq!(score(&c, &a, key), want, "{ability}");
    }
    let (mut c, a, t) = setup("Tackle", "Guts", "Flower Gift");
    c.battle.as_mut().unwrap().combatants[1].side = "Players".into();
    c.battle.as_mut().unwrap().combatants[0]
        .runtime
        .statuses
        .push(crate::combat::Status {
            kind: "Poison".into(),
            detail: "".into(),
            rounds: None,
            progress: 0,
        });
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .natural = Some(field::Weather::Sunny);
    assert_eq!(score(&c, &a, "Strength"), 6);
    assert_eq!(score(&c, &t, "Strength"), 6);
}
#[test]
fn extra_sourced_stats_trigger_protection_reflection_and_once_only_retaliation() {
    for (ability, want) in [
        ("Defiant", 5),
        ("Hyper Cutter", 4),
        ("Clear Body", 4),
        ("Contrary", 5),
        ("Simple", 2),
        ("Guard Dog", 6),
    ] {
        let (mut c, a, t) = setup("Tackle", "", ability);
        extra::change_stat(&mut c, &a, &t, "Strength", -1).unwrap();
        assert_eq!(score(&c, &t, "Strength"), want, "{ability}");
    }
    let (mut c, a, t) = setup("Tackle", "", "Mirror Armor");
    extra::change_stat(&mut c, &a, &t, "Strength", -1).unwrap();
    assert_eq!(score(&c, &a, "Strength"), 3);
    assert_eq!(score(&c, &t, "Strength"), 4);
    let (mut c, a, t) = setup("Tackle", "", "Competitive");
    extra::change_stat(&mut c, &t, &t, "Strength", -1).unwrap();
    assert_eq!(score(&c, &t, "Special"), 4);
    extra::change_stat(&mut c, &a, &t, "Strength", -1).unwrap();
    assert_eq!(score(&c, &t, "Special"), 6);
    extra::change_stat(&mut c, &a, &t, "Strength", -1).unwrap();
    assert_eq!(score(&c, &t, "Special"), 6);
}
#[test]
fn extra_hit_reactions_cover_immunity_defenses_disguise_and_fainting() {
    for (ability, name, key, n) in [
        ("Water Compaction", "Water Gun", "boost:Defense", 2),
        ("Well-Baked Body", "Ember", "boost:Defense", 2),
        ("Thermal Exchange", "Ember", "boost:Strength", 1),
        ("Stamina", "Tackle", "boost:Defense", 1),
        ("Steam Engine", "Ember", "boost:Dexterity", 3),
        ("Weak Armor", "Tackle", "debuff:Defense", -1),
    ] {
        let (c, _, t) = pending(name, "", ability);
        let c = roll(&c, "player", &damage_command(&c), 4);
        assert_eq!(
            p(&c, &t).runtime.ability.counters.get(key),
            Some(&n),
            "{ability}"
        );
        if ["Water Compaction", "Well-Baked Body"].contains(&ability) {
            assert_eq!(p(&c, &t).hp, 20);
        }
    }
    let (c, _, t) = pending("Tackle", "", "Disguise");
    let c = roll(&c, "player", &damage_command(&c), 4);
    assert_eq!(p(&c, &t).hp, 20);
    assert!(p(&c, &t).runtime.ability.flags.contains("Disguise"));
    let (mut c, a, t) = pending("Tackle", "", "Innards Out");
    c.battle.as_mut().unwrap().combatants[1].hp = 2;
    let c = roll(&c, "player", &damage_command(&c), 4);
    assert_eq!(p(&c, &a).hp, 18);
    assert_eq!(p(&c, &t).hp, 0);
}
#[test]
fn extra_poison_heal_ticks_once_cures_after_three_rounds_and_does_not_revive() {
    let (mut c, a, _) = setup("Tackle", "Poison Heal", "");
    c.battle.as_mut().unwrap().combatants[0].hp = 10;
    c.battle.as_mut().unwrap().combatants[0]
        .runtime
        .statuses
        .push(crate::combat::Status {
            kind: "Badly Poisoned".into(),
            detail: "".into(),
            rounds: None,
            progress: 0,
        });
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .order
        .clear();
    for _ in 0..3 {
        c = battle::transition(&c, "gm", &Action::NextRound, vec![], &catalog())
            .unwrap()
            .0;
    }
    assert_eq!(p(&c, &a).hp, 13);
    assert!(p(&c, &a).runtime.statuses.is_empty());
    c.battle.as_mut().unwrap().combatants[0].hp = 0;
    extra::end_round(&mut c).unwrap();
    assert_eq!(p(&c, &a).hp, 0);
}
#[test]
fn extra_chance_modifiers_and_effect_spore_selector_do_not_share_dice() {
    for (source, target, n) in [
        ("Serene Grace", "", 3),
        ("", "Wonder Skin", 0),
        ("Serene Grace", "Wonder Skin", 1),
    ] {
        let (c, _, _) = pending("Thunder Shock", source, target);
        let c = roll(&c, "player", &damage_command(&c), 4);
        let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
        assert_eq!(
            rules::dice_count(
                &c,
                &Action::ResolveEffect {
                    sequence: q.sequence,
                    reason: "chance".into()
                },
                &catalog()
            )
            .unwrap(),
            n
        );
    }
    let (c, a, _) = pending("Tackle", "Wonder Skin", "Effect Spore");
    let c = roll(&c, "player", &damage_command(&c), 4);
    let e = &c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0];
    assert_eq!(
        crate::combat::abilities::dice(&c, &e.id, "roll").unwrap(),
        2
    );
    let c = trigger(&c, "other", "roll", vec![6, 6]).unwrap();
    assert!(p(&c, &a).runtime.statuses.iter().any(|s| s.kind == "Sleep"));
}
#[test]
fn extra_no_guard_choice_disables_defenses_and_low_accuracy_for_one_round() {
    let (mut c, a, _) = setup("Blizzard", "No Guard", "");
    extra::round_start(&mut c).unwrap();
    let c = trigger(&c, "player", "activate", vec![]).unwrap();
    let m = field::move_data(
        &c,
        p(&c, &a),
        &catalog().find("moves", "Name", "Blizzard").unwrap(),
    );
    assert_eq!(m["Attributes"]["AccuracyReduction"], 0);
    assert!(extra::reaction(&c, p(&c, &a), "evade", None).is_err());
    assert!(rules::accuracy(&c, &a, "Protect", 0, &catalog()).is_err());
}
#[test]
fn extra_entry_decisions_enforce_gm_role_and_hospitality_will() {
    let (mut c, a, _) = setup("Tackle", "Download", "");
    let before = c.clone();
    extra::lifecycle(&before, &mut c, &catalog()).unwrap();
    assert!(trigger(&c, "player", "Strength", vec![]).is_err());
    let c = trigger(&c, "gm", "Strength", vec![]).unwrap();
    assert_eq!(score(&c, &a, "Strength"), 5);
    let (mut c, a, t) = setup("Tackle", "Hospitality", "");
    c.battle.as_mut().unwrap().combatants[1].side = "Players".into();
    c.battle.as_mut().unwrap().combatants[1].hp = 10;
    let before = c.clone();
    extra::lifecycle(&before, &mut c, &catalog()).unwrap();
    let c = trigger(&c, "player", &t, vec![]).unwrap();
    assert_eq!(p(&c, &t).hp, 13);
    assert_eq!(p(&c, &a).runtime.will, 5);
}
#[test]
fn extra_end_round_cure_precedes_round_expiry_without_duplicate_status_damage() {
    let (mut c, a, _) = setup("Tackle", "Natural Cure", "");
    c.battle.as_mut().unwrap().combatants[0]
        .runtime
        .statuses
        .push(crate::combat::Status {
            kind: "Poison".into(),
            detail: "".into(),
            rounds: None,
            progress: 0,
        });
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .order
        .clear();
    let c = battle::transition(&c, "gm", &Action::NextRound, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(c.battle.as_ref().unwrap().round, 1);
    assert_eq!(p(&c, &a).hp, 18);
    let c = trigger(&c, "player", "roll", vec![6, 1, 1]).unwrap();
    let c = battle::transition(&c, "gm", &Action::NextRound, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(c.battle.as_ref().unwrap().round, 2);
    assert_eq!(p(&c, &a).hp, 18);
}
#[test]
fn extra_corrosion_and_protective_auras_keep_self_infliction_distinct() {
    let (mut c, a, t) = setup("Tackle", "Corrosion", "");
    field::change_type(&mut c, &t, "replace", &["Steel".into()]).unwrap();
    crate::combat::abilities::inflict(&mut c, &a, &t, "Poison", "", None, &catalog()).unwrap();
    assert!(p(&c, &t)
        .runtime
        .statuses
        .iter()
        .any(|s| s.kind == "Poison"));
    let (mut c, a, t) = setup("Tackle", "Flower Veil", "");
    c.battle.as_mut().unwrap().combatants[1].side = "Players".into();
    crate::combat::abilities::inflict(&mut c, &a, &t, "Poison", "", None, &catalog()).unwrap();
    assert!(p(&c, &t).runtime.statuses.is_empty());
    crate::combat::abilities::inflict(&mut c, &t, &t, "Poison", "", None, &catalog()).unwrap();
    assert!(p(&c, &t)
        .runtime
        .statuses
        .iter()
        .any(|s| s.kind == "Poison"));
}
#[test]
fn extra_typing_traps_and_infiltrator_use_encounter_state() {
    let (mut c, a, t) = setup("Tackle", "Shadow Tag", "");
    assert!(extra::blocked(&c, p(&c, &t), &catalog()));
    field::change_type(&mut c, &t, "replace", &["Ghost".into()]).unwrap();
    assert!(!extra::blocked(&c, p(&c, &t), &catalog()));
    assert!(!extra::blocked(&c, p(&c, &a), &catalog()));
    let (mut c, a, t) = pending("Tackle", "Infiltrator", "");
    c.battle.as_mut().unwrap().combatants[1].runtime.shield = 3;
    let c = roll(&c, "player", &damage_command(&c), 4);
    assert_eq!(p(&c, &t).hp, 13);
    assert_eq!(p(&c, &a).hp, 20);
}
#[test]
fn extra_mimicry_preserves_secondary_type_and_highest_attribute_ties_need_a_choice() {
    let (mut c, a, _) = setup("Tackle", "Mimicry", "");
    field::change_type(&mut c, &a, "add", &["Steel".into()]).unwrap();
    let f = &mut c
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield;
    f.rules.insert(
        0,
        field::FieldRule {
            id: "electric".into(),
            kind: "Electric Terrain".into(),
            source: "test".into(),
            scope: "all".into(),
            rounds: Some(4),
            parameter: 1,
            started_round: 1,
        },
    );
    assert_eq!(
        field::types(&c, p(&c, &a), &catalog()).unwrap(),
        vec!["Electric", "Steel"]
    );
    let (mut c, a, _) = setup("Tackle", "Protosynthesis", "");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .natural = Some(field::Weather::Sunny);
    let before = c.clone();
    extra::lifecycle(&before, &mut c, &catalog()).unwrap();
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0]
        .choices
        .contains(&"Dexterity".into()));
    let c = trigger(&c, "player", "Dexterity", vec![]).unwrap();
    assert_eq!(score(&c, &a, "Dexterity"), 6);
    assert_eq!(score(&c, &a, "Strength"), 4);
}

fn test_pool(c: &Campaign, a: &str, seq: u32) -> Result<rules::DamagePool> {
    let mut c = c.clone();
    let q = c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].clone();
    let source = c
        .battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == a)
        .unwrap();
    source.pending_damage = Some(PendingDamage {
        sequence: seq,
        target_id: q.target_id,
        move_name: q.move_name,
        critical: q.critical,
    });
    rules::damage_pool(&c, a, seq, 0, &catalog())
}

#[test]
fn extra_color_change_restores_underlying_scene_typing_and_libero_changes_before_damage() {
    let (mut c, a, t) = setup("Tackle", "", "Color Change");
    field::change_type(&mut c, &t, "replace", &["Water".into()]).unwrap();
    let c = hit(&c, &a, &t, "Tackle");
    let c = pass(&c);
    let mut c = roll(&c, "player", &damage_command(&c), 4);
    assert_eq!(
        field::types(&c, p(&c, &t), &catalog()).unwrap(),
        vec!["Normal"]
    );
    field::change_type_inactive(&mut c, &t);
    assert_eq!(
        field::types(&c, p(&c, &t), &catalog()).unwrap(),
        vec!["Water"]
    );
    let (c, a, t) = pending("Ember", "Libero", "");
    assert_eq!(
        field::types(&c, p(&c, &a), &catalog()).unwrap(),
        vec!["Fire"]
    );
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    assert_eq!(test_pool(&c, &a, q.sequence).unwrap().stab, 1);
    assert_eq!(p(&c, &t).hp, 20);
}
#[test]
fn extra_cursed_body_tracks_distinct_disabled_moves_and_shed_skin_selects_without_bias() {
    let (mut c, a, t) = pending("Tackle", "", "Cursed Body");
    c = roll(&c, "player", &damage_command(&c), 4);
    let mut c = trigger(&c, "other", "roll", vec![6, 1, 1]).unwrap();
    assert!(p(&c, &a)
        .runtime
        .statuses
        .iter()
        .any(|s| s.kind == "Disable" && s.detail == "Tackle"));
    crate::combat::abilities::inflict(&mut c, &t, &a, "Disable", "Quick Attack", None, &catalog())
        .unwrap();
    assert_eq!(
        p(&c, &a)
            .runtime
            .statuses
            .iter()
            .filter(|s| s.kind == "Disable")
            .count(),
        2
    );
    let (mut c, a, _) = setup("Tackle", "Shed Skin", "");
    for kind in ["Poison", "Burn", "Disable", "Confusion"] {
        c.battle.as_mut().unwrap().combatants[0]
            .runtime
            .statuses
            .push(crate::combat::Status {
                kind: kind.into(),
                detail: "Tackle".into(),
                rounds: None,
                progress: 0,
            });
    }
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .order
        .clear();
    let c = battle::transition(&c, "gm", &Action::NextRound, vec![], &catalog())
        .unwrap()
        .0;
    let c = trigger(&c, "player", "roll", vec![6, 1, 1]).unwrap();
    let c = trigger(&c, "player", "roll", vec![6]).unwrap();
    assert_eq!(p(&c, &a).runtime.statuses.len(), 4);
    let c = trigger(&c, "player", "roll", vec![1]).unwrap();
    assert_eq!(p(&c, &a).runtime.statuses.len(), 3);
}
#[test]
fn extra_quick_draw_and_slow_start_preserve_round_order_rules() {
    let (mut c, a, t) = setup("Tackle", "Quick Draw", "Slow Start");
    let before = c.clone();
    extra::lifecycle(&before, &mut c, &catalog()).unwrap();
    extra::round_start(&mut c).unwrap();
    let c = trigger(&c, "player", "roll", vec![6, 1, 1]).unwrap();
    assert_eq!(
        c.battle.as_ref().unwrap().flow.as_ref().unwrap().order[0],
        a
    );
    assert_eq!(extra::order_group(&c, p(&c, &t)), 1);
    let mut c = c;
    c.battle.as_mut().unwrap().round += 3;
    assert_eq!(extra::order_group(&c, p(&c, &t)), -1);
    assert_eq!(score(&c, &t, "Strength"), 6);
    assert_eq!(score(&c, &t, "Dexterity"), 6);
}
#[test]
fn extra_beast_boost_requires_a_tie_choice_and_sand_spit_does_not_refresh() {
    let (mut c, a, t) = pending("Tackle", "Beast Boost", "");
    c.battle.as_mut().unwrap().combatants[1].hp = 1;
    let c = roll(&c, "player", &damage_command(&c), 4);
    assert_eq!(p(&c, &t).hp, 0);
    let c = trigger(&c, "player", "Strength", vec![]).unwrap();
    assert_eq!(score(&c, &a, "Strength"), 5);
    assert_eq!(
        p(&c, &a).runtime.ability.counters.get("Beast Boost total"),
        Some(&1)
    );
    let (c, _, t) = pending("Tackle", "", "Sand Spit");
    let mut c = roll(&c, "player", &damage_command(&c), 4);
    assert_eq!(field::weather(&c, None), Some(field::Weather::Sandstorm));
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .weather
        .as_mut()
        .unwrap()
        .rounds = Some(2);
    field::sand_spit(&mut c, &t).unwrap();
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .battlefield
            .weather
            .as_ref()
            .unwrap()
            .rounds,
        Some(2)
    );
    c.battle.as_mut().unwrap().combatants[1].deployed = false;
    let before = c.clone();
    field::lifecycle(&before, &mut c, &catalog()).unwrap();
    assert_eq!(field::weather(&c, None), Some(field::Weather::Sandstorm));
}
#[test]
fn extra_sourced_stat_api_requires_gm_and_mega_launcher_uses_complete_healing() {
    let (c, a, t) = setup("Tackle", "", "Defiant");
    let action = Action::BattleStat {
        source_id: a,
        combatant_id: t.clone(),
        stat: "Strength".into(),
        change: -1,
        reason: "Reviewed Growl ruling".into(),
    };
    assert!(battle::transition(&c, "host", &action, vec![], &catalog()).is_err());
    let c = battle::transition(&c, "gm", &action, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(score(&c, &t, "Strength"), 5);
    let (mut c, a, t) = setup("Heal Pulse", "Mega Launcher", "");
    c.battle.as_mut().unwrap().combatants[1].side = "Players".into();
    c.battle.as_mut().unwrap().combatants[1].hp = 10;
    let sid = p(&c, &a).sheet_id.clone();
    let mut w=c.sheets[&sid].workflow.clone().unwrap_or_else(||serde_json::from_value(serde_json::json!({"version":1,"status":"accepted","origin":"captured","age":"","allocations":{},"socials":{},"happiness":2,"loyalty":2,"training_points":0,"legacy_xp":0,"will":6,"max_will":6,"profile":{"version":1,"defenses":"split","hp":"vitality"},"hp_attribute":"Vitality","available_abilities":[],"learned":{},"forgotten":{},"overranked":null,"evolution_ready":null,"ledger":[]})).unwrap());
    w.max_will = 6;
    c.sheets.get_mut(&sid).unwrap().workflow = Some(w);
    let c = hit(&c, &a, &t, "Heal Pulse");
    let c = pass(&c);
    let q = c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].clone();
    let c = battle::transition(
        &c,
        "player",
        &Action::ResolveEffect {
            sequence: q.sequence,
            reason: "Complete Heal Pulse".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    assert_eq!(p(&c, &t).hp, 15);
    assert_eq!(p(&c, &a).runtime.will, 5);
}

#[test]
fn extra_catalog_abilities_preserve_sheets_and_reconnect_without_repeating_entry_events() {
    let registry: serde_json::Value =
        serde_json::from_str(include_str!("../../../rules/ability-coverage.json")).unwrap();
    for (name, definition) in registry["entries"].as_object().unwrap() {
        if definition["status"] == "manual" {
            continue;
        }
        let (mut c, a, _) = setup("Tackle", name, name);
        let before_sheets = serde_json::to_value(&c.sheets).unwrap();
        let before = c.clone();
        extra::lifecycle(&before, &mut c, &catalog())
            .unwrap_or_else(|e| panic!("{name}: {}", e.message));
        for fighter in &c.battle.as_ref().unwrap().combatants {
            let s = rules::effective_sheet(&c, fighter).unwrap();
            for attr in ["Strength", "Dexterity", "Vitality", "Special", "Insight"] {
                assert!((0..=10).contains(&s.stats[attr]), "{name}: {attr}");
            }
        }
        let serialized = serde_json::to_value(&c).unwrap();
        let mut resumed: Campaign = serde_json::from_value(serialized.clone()).unwrap();
        extra::lifecycle(&c, &mut resumed, &catalog()).unwrap();
        assert_eq!(
            serde_json::to_value(&resumed).unwrap(),
            serialized,
            "{name}: repeated entry"
        );
        assert_eq!(
            serde_json::to_value(&resumed.sheets).unwrap(),
            before_sheets,
            "{name}: permanent mutation"
        );
        assert_eq!(p(&resumed, &a).selected_ability, *name);
    }
}

#[test]
fn extra_paralysis_and_weather_immunity_do_not_rewrite_historical_engines() {
    let (mut c, a, _) = setup("Tackle", "Overcoat", "");
    c.sheets
        .get_mut(&p(&c, &a).sheet_id.clone())
        .unwrap()
        .stats
        .insert("Dexterity".into(), 1);
    c.battle.as_mut().unwrap().combatants[0]
        .runtime
        .statuses
        .push(crate::combat::Status {
            kind: "Paralysis".into(),
            detail: "".into(),
            rounds: None,
            progress: 0,
        });
    assert_eq!(score(&c, &a, "Dexterity"), 0);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield
        .natural = Some(field::Weather::Hail);
    let mut historical = c.clone();
    historical
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .extra
        .enabled = false;
    field::end(&mut historical, &catalog()).unwrap();
    field::end(&mut c, &catalog()).unwrap();
    assert_eq!(p(&historical, &a).hp, 19);
    assert_eq!(p(&c, &a).hp, 20);
}

#[test]
fn extra_converted_hits_use_the_same_type_for_reduction_reactions_and_immunity() {
    let (c, _, _) = pending("Tackle", "Long Reach", "Sand Spit");
    let c = roll(&c, "player", &damage_command(&c), 4);
    assert_eq!(field::weather(&c, None), None);
    let (c, _, t) = pending("Tackle", "Refrigerate", "");
    let c = roll(&c, "player", &damage_command(&c), 4);
    let unprotected = p(&c, &t).hp;
    let (c, _, t) = pending("Tackle", "Refrigerate", "Thick Fat");
    let c = roll(&c, "player", &damage_command(&c), 4);
    assert_eq!(p(&c, &t).hp, unprotected + 1);
    let (c, _, t) = pending("Tackle", "Refrigerate", "Color Change");
    let c = roll(&c, "player", &damage_command(&c), 4);
    assert_eq!(
        field::types(&c, p(&c, &t), &catalog()).unwrap(),
        vec!["Ice"]
    );
    let (mut c, a, t) = pending("Poison Sting", "Corrosion", "Wonder Guard");
    field::change_type(&mut c, &t, "replace", &["Steel".into()]).unwrap();
    assert!(
        !test_pool(
            &c,
            &a,
            c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sequence
        )
        .unwrap()
        .immune
    );
}

#[test]
fn extra_pending_round_choices_preserve_knockout_provenance() {
    let (mut c, a, _) = setup("Tackle", "", "Natural Cure");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .order
        .clear();
    for (i, p) in c.battle.as_mut().unwrap().combatants.iter_mut().enumerate() {
        if i == 0 {
            p.hp = 1;
        }
        p.runtime.statuses.push(crate::combat::Status {
            kind: "Poison".into(),
            detail: "".into(),
            rounds: None,
            progress: 0,
        });
    }
    let c = battle::transition(&c, "gm", &Action::NextRound, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(p(&c, &a).hp, 0);
    let b = c.battle.as_ref().unwrap();
    assert_eq!(b.round, 1);
    assert!(!b.flow.as_ref().unwrap().ability_events.is_empty());
    assert!(b.flow.as_ref().unwrap().extra.fainted.contains(&a));
}
