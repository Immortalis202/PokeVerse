use super::*;
use crate::combat::forms::{self, FormBuild, FormCommand};
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
fn setup(species: &str, ability: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = super::special_tests::setup("Tackle");
    let cat = catalog();
    let sp = cat.find("species", "Name", species).unwrap();
    let sid = p(&c, &a).sheet_id.clone();
    let s = c.sheets.get_mut(&sid).unwrap();
    s.species_id = Some(sp["AppId"].as_str().unwrap().into());
    if s.workflow.is_none() {
        s.workflow = Some(crate::creation::build(s, &cat));
    }
    if let Some(b) = s.workflow.as_mut() {
        b.allocations.clear();
    }
    s.ability = ability.into();
    s.moves.push("Transform".into());
    pm(&mut c, &a).selected_ability = ability.into();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .forms
        .enabled = true;
    forms::initialize(&mut c, &cat).unwrap();
    (c, a, t)
}
fn act(command: FormCommand) -> Action {
    Action::Form {
        command,
        reason: "Reviewed form fixture".into(),
    }
}
fn refresh(c: &Campaign) -> Campaign {
    let mut n = c.clone();
    forms::lifecycle(c, &mut n, &Action::StartTurns, &catalog()).unwrap();
    n
}
fn choose(c: &Campaign, id: &str, kind: &str) -> Campaign {
    let event = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events
        .iter()
        .find(|e| e.kind == kind && e.combatant_id == id)
        .unwrap()
        .clone();
    let mut n = c.clone();
    forms::resolve(&mut n, &event, &event.choices[0], &catalog()).unwrap();
    n.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_events
        .retain(|e| e.id != event.id);
    n
}
#[test]
fn forms_registry_all_mega_stones_have_exact_distinct_base_and_form_records() {
    let cat = catalog();
    let r = forms::registry();
    let mega = r["mega"].as_object().unwrap();
    assert_eq!(mega.len(), 47);
    for (name, m) in mega {
        assert_ne!(m["base"], m["form"]);
        assert!(cat.find("items", "Name", name).is_some());
        for id in [m["base"].as_str().unwrap(), m["form"].as_str().unwrap()] {
            assert!(cat.find("species", "AppId", id).is_some());
            assert!(!r["species"][id].is_null());
        }
    }
    assert_eq!(r["species"]["0681:aegislash"]["stats"]["Vitality"], 4);
    assert_eq!(
        r["species"]["0681F1:aegislash-blade-form"]["stats"]["Vitality"],
        2
    );
}
#[test]
fn forms_multitype_is_owned_exact_species_reversible_and_sheet_preserving() {
    let (c, a, t) = setup("Arceus", "Multitype");
    let sheets = serde_json::to_value(&c.sheets).unwrap();
    let command = act(FormCommand::Type {
        combatant_id: a.clone(),
        type_name: "Fire".into(),
    });
    assert!(battle::transition(&c, "other", &command, vec![], &catalog()).is_err());
    let n = decision(&c, "player", command);
    assert_eq!(
        crate::combat::field::live_types(&n, p(&n, &a), &catalog()).unwrap(),
        vec!["Fire"]
    );
    assert_eq!(serde_json::to_value(&n.sheets).unwrap(), sheets);
    let undo = decision(
        &n,
        "gm",
        Action::RollbackBattle {
            event_sequence: n.battle_undo.last().unwrap().event_sequence,
            reason: "Restore type".into(),
        },
    );
    assert!(forms::types(&undo, &a).is_none());
    assert!(battle::transition(
        &c,
        "gm",
        &act(FormCommand::Type {
            combatant_id: t,
            type_name: "Fire".into()
        }),
        vec![],
        &catalog()
    )
    .is_err());
}
#[test]
fn forms_prepared_builds_enforce_budgets_limits_and_preserve_permanent_sheets() {
    let (c, a, _) = setup("Aegislash", "Stance Change");
    let target = "0681F1:aegislash-blade-form".to_string();
    let build = FormBuild {
        attributes: crate::creation::ATTRS
            .into_iter()
            .map(|key| (key.into(), i32::MAX))
            .collect(),
        ..Default::default()
    };
    assert!(battle::transition(
        &c,
        "player",
        &act(FormCommand::Prepare {
            combatant_id: a.clone(),
            species_id: target.clone(),
            build
        }),
        vec![],
        &catalog()
    )
    .is_err());
    let n = decision(
        &c,
        "player",
        act(FormCommand::Prepare {
            combatant_id: a.clone(),
            species_id: target,
            build: FormBuild::default(),
        }),
    );
    assert_eq!(
        serde_json::to_value(&n.sheets).unwrap(),
        serde_json::to_value(&c.sheets).unwrap()
    );
}
#[test]
fn forms_imposter_copies_moves_and_attributes_without_copying_hp_or_equipment() {
    let (mut c, a, t) = setup("Ditto", "Imposter");
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .stats
        .insert("Strength".into(), 6);
    pm(&mut c, &t).selected_ability = "Levitate".into();
    let hp = p(&c, &a).hp;
    let max = p(&c, &a).max_hp;
    let mut before = c.clone();
    before.battle = None;
    forms::lifecycle(&before, &mut c, &Action::StartTurns, &catalog()).unwrap();
    let n = choose(&c, &a, "form-imposter");
    let s = rules::base_sheet(&n, p(&n, &a)).unwrap();
    assert_eq!(s.stats["Strength"], 6);
    assert!(s.moves.iter().any(|m| m == "Transform"));
    assert_eq!(p(&n, &a).selected_ability, "Levitate");
    assert_eq!((p(&n, &a).hp, p(&n, &a).max_hp), (hp, max));
}
#[test]
fn forms_threshold_core_heals_once_and_persists_without_replacing_library_species() {
    let (mut c, a, _) = setup("Minior", "Shields Down");
    pm(&mut c, &a).hp = 1;
    pm(&mut c, &a).runtime.will = 3;
    let c = refresh(&c);
    let n = choose(&c, &a, "form-core");
    assert_eq!(p(&n, &a).hp, p(&n, &a).max_hp);
    assert!(forms::status_immune(&n, p(&n, &a)));
    assert_eq!(p(&n, &a).runtime.will, 3);
    assert_eq!(n.form_memory[&p(&n, &a).sheet_id], "0774F1:minior-core");
    assert_eq!(
        n.sheets[&p(&n, &a).sheet_id].species_id.as_deref(),
        Some("0774:minior")
    );
    assert!(!refresh(&n)
        .battle
        .unwrap()
        .flow
        .unwrap()
        .ability_events
        .iter()
        .any(|e| e.kind == "form-core"));
}
#[test]
fn forms_ice_face_absorbs_two_damage_and_then_changes_form() {
    let (mut c, a, _) = setup("Eiscue", "Ice Face");
    assert_eq!(forms::damage_guard(&mut c, &a, 1).unwrap(), 0);
    assert_eq!(forms::damage_guard(&mut c, &a, 4).unwrap(), 3);
    let n = choose(&c, &a, "form-ice");
    assert_eq!(forms::types(&n, &a).unwrap(), vec!["Ice"]);
    assert_eq!(forms::damage_guard(&mut n.clone(), &a, 2).unwrap(), 2);
}
#[test]
fn forms_school_arrival_requires_gm_and_is_prevented_by_healing_or_recall() {
    let (mut c, a, _) = setup("Wishiwashi", "Schooling");
    pm(&mut c, &a).hp = 1;
    pm(&mut c, &a).runtime.will = 3;
    let c = refresh(&c);
    let command = act(FormCommand::SchoolArrives {
        combatant_id: a.clone(),
    });
    assert!(battle::transition(&c, "player", &command, vec![], &catalog()).is_err());
    let n = decision(&c, "gm", command.clone());
    assert_eq!(rules::base_sheet(&n, p(&n, &a)).unwrap().rank, "Expert");
    assert_eq!(p(&n, &a).hp, p(&n, &a).max_hp);
    assert_eq!(p(&n, &a).runtime.will, 3);
    let mut healed = c.clone();
    pm(&mut healed, &a).hp = pm(&mut healed, &a).max_hp;
    assert!(battle::transition(&healed, "gm", &command, vec![], &catalog()).is_err());
    let mut recalled = c;
    pm(&mut recalled, &a).deployed = false;
    assert!(battle::transition(&recalled, "gm", &command, vec![], &catalog()).is_err());
}
#[test]
fn forms_tera_shell_is_species_and_full_health_scoped() {
    let (mut c, a, _) = setup("Terapagos (Terastal Form)", "Tera Shell");
    let m = catalog().find("moves", "Name", "Tackle").unwrap();
    pm(&mut c, &a).hp = pm(&mut c, &a).max_hp;
    assert!(forms::tera_shell(&c, p(&c, &a), &m));
    pm(&mut c, &a).hp -= 1;
    assert!(!forms::tera_shell(&c, p(&c, &a), &m));
}
#[test]
fn forms_private_projections_redact_saved_original_equipment() {
    let (c, a, _) = setup("Arceus", "Multitype");
    let mut c = decision(
        &c,
        "player",
        act(FormCommand::Type {
            combatant_id: a.clone(),
            type_name: "Normal".into(),
        }),
    );
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .inventory
        .insert("Secret stock".into(), 3);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .forms
        .originals
        .get_mut(&a)
        .unwrap()
        .sheet
        .inventory
        .insert("Secret stock".into(), 3);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .enabled = true;
    assert!(
        !serde_json::to_string(&crate::privacy::for_player(&c, "other"))
            .unwrap()
            .contains("Secret stock")
    );
    assert!(serde_json::to_string(&crate::privacy::for_player(&c, "gm"))
        .unwrap()
        .contains("Secret stock"));
}
fn mega_fixture(name: &str, stone: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = setup(name, "");
    let sid = p(&c, &a).sheet_id.clone();
    let mut trainer = c.sheets[&sid].clone();
    trainer.id = "mega-trainer".into();
    trainer.kind = "trainer".into();
    trainer.ability.clear();
    c.sheets.insert(trainer.id.clone(), trainer);
    let mut s = c.sheets[&sid].clone();
    s.trainer_id = Some("mega-trainer".into());
    s.inventory.insert(stone.into(), 1);
    let s = crate::combat::items::equip(&s, stone, "mega-stone", &catalog()).unwrap();
    c.sheets.insert(sid, s);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .enabled = true;
    crate::combat::items::initialize(&mut c, &catalog()).unwrap();
    pm(&mut c, &a).runtime.will = 5;
    c = decision(
        &c,
        "gm",
        act(FormCommand::KeyStone {
            trainer_id: "mega-trainer".into(),
            present: true,
        }),
    );
    (c, a, t)
}
#[test]
fn forms_all_47_mega_stones_activate_exact_forms_and_restore_original_resources() {
    for (stone, mapping) in forms::registry()["mega"].as_object().unwrap() {
        let name = forms::registry()["species"][mapping["base"].as_str().unwrap()]["name"]
            .as_str()
            .unwrap();
        let (mut c, a, _) = mega_fixture(name, stone);
        pm(&mut c, &a).hp = 2;
        pm(&mut c, &a).runtime.statuses.push(crate::combat::Status {
            kind: "Poison".into(),
            detail: "original body".into(),
            rounds: None,
            progress: 0,
        });
        let n = decision(
            &c,
            "player",
            act(FormCommand::Mega {
                combatant_id: a.clone(),
                stone: stone.clone(),
            }),
        );
        assert_eq!(
            rules::base_sheet(&n, p(&n, &a))
                .unwrap()
                .species_id
                .as_deref(),
            mapping["form"].as_str(),
            "{stone}"
        );
        assert_eq!(p(&n, &a).hp, p(&n, &a).max_hp);
        assert!(p(&n, &a).runtime.statuses.is_empty());
        assert!(battle::transition(
            &n,
            "player",
            &act(FormCommand::Mega {
                combatant_id: a.clone(),
                stone: stone.clone()
            }),
            vec![],
            &catalog()
        )
        .is_err());
        let mut restored = n.clone();
        forms::before_end(&mut restored).unwrap();
        assert_eq!(p(&restored, &a).hp, 2, "{stone}");
        assert_eq!(p(&restored, &a).runtime.will, 4);
        assert_eq!(p(&restored, &a).runtime.statuses[0].kind, "Poison");
        assert_eq!(
            n.sheets[&p(&n, &a).sheet_id].species_id,
            c.sheets[&p(&c, &a).sheet_id].species_id
        );
    }
}
#[test]
fn forms_mega_klutz_exception_stone_lock_and_fainted_end_rollback() {
    let (mut c, a, _) = mega_fixture("Charizard", "Charizardite X");
    pm(&mut c, &a).selected_ability = "Klutz".into();
    let command = Action::HeldItem {
        command: crate::combat::items::ItemCommand::Lose {
            combatant_id: a.clone(),
            unit_id: "mega-stone".into(),
            override_sticky_hold: true,
        },
        reason: "Attempt forbidden removal".into(),
    };
    assert!(battle::transition(&c, "gm", &command, vec![], &catalog()).is_err());
    let mut n = decision(
        &c,
        "player",
        act(FormCommand::Mega {
            combatant_id: a.clone(),
            stone: "Charizardite X".into(),
        }),
    );
    pm(&mut n, &a).hp = 0;
    let ended = decision(&n, "gm", Action::EndBattle);
    assert_eq!(ended.sheets[&p(&ended, &a).sheet_id].hp, 0);
    let undone = decision(
        &ended,
        "gm",
        Action::RollbackBattle {
            event_sequence: ended.battle_undo.last().unwrap().event_sequence,
            reason: "Restore fainted Mega encounter".into(),
        },
    );
    assert_eq!(
        rules::base_sheet(&undone, p(&undone, &a))
            .unwrap()
            .species_id
            .as_deref(),
        Some("0006M2:charizard-mega-x-form")
    );
    assert_eq!(p(&undone, &a).hp, 0);
}
#[test]
fn forms_round_and_switch_triggers_use_printed_conditions() {
    for (name, ability, kind) in [
        ("Darmanitan", "Zen Mode", "form-round"),
        ("Zygarde 10%", "Power Construct", "form-round"),
        ("Morpeko", "Hunger Switch", ""),
    ] {
        let (mut c, a, _) = setup(name, ability);
        pm(&mut c, &a).hp = 1;
        let mut n = c.clone();
        n.battle.as_mut().unwrap().round += 1;
        forms::lifecycle(&c, &mut n, &Action::NextRound, &catalog()).unwrap();
        if kind.is_empty() {
            assert_eq!(
                n.battle
                    .as_ref()
                    .unwrap()
                    .flow
                    .as_ref()
                    .unwrap()
                    .forms
                    .choices[&format!("hunger:{a}")],
                "Hangry"
            );
        } else {
            let n = choose(&n, &a, kind);
            assert_ne!(
                rules::base_sheet(&n, p(&n, &a)).unwrap().species_id,
                c.sheets[&p(&c, &a).sheet_id].species_id
            );
            if ability == "Power Construct" {
                assert_eq!(p(&n, &a).hp, p(&n, &a).max_hp);
            }
        }
    }
    let (c, a, _) = setup("Palafin (Zero Form)", "Zero to Hero");
    let mut n = c.clone();
    pm(&mut n, &a).deployed = false;
    forms::lifecycle(&c, &mut n, &Action::NextRound, &catalog()).unwrap();
    let n = choose(&n, &a, "form-hero");
    assert_eq!(
        n.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .forms
            .current[&a]
            .sheet
            .species_id
            .as_deref(),
        Some("0964F2:palafin-hero-form")
    );
    assert!(!p(&n, &a).deployed);
}
#[test]
fn forms_stance_categories_and_tera_shift_restore_printed_resources() {
    let (mut c, a, _) = setup("Aegislash", "Stance Change");
    let mut before = c.clone();
    before.battle = None;
    forms::lifecycle(&before, &mut c, &Action::StartTurns, &catalog()).unwrap();
    let n = choose(&c, &a, "form-stance");
    let m = catalog().find("moves", "Name", "Protect").unwrap();
    let blade = n
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .forms
        .current[&a]
        .sheet
        .species_id
        .as_deref()
        == Some("0681F1:aegislash-blade-form");
    assert_eq!(forms::guard_move(&n, &a, &m).is_err(), blade);
    let (mut c, a, _) = setup("Terapagos (Normal Form)", "Tera Shift");
    pm(&mut c, &a).hp = 1;
    pm(&mut c, &a).runtime.will = 0;
    let mut before = c.clone();
    before.battle = None;
    forms::lifecycle(&before, &mut c, &Action::StartTurns, &catalog()).unwrap();
    // The species option precedes decline lexically only in some presentations; select the explicit form.
    let e = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events
        .iter()
        .find(|e| e.kind == "form-tera")
        .unwrap()
        .clone();
    let mut n = c.clone();
    forms::resolve(&mut n, &e, "1024F2:terapagos-terastal-form", &catalog()).unwrap();
    assert_eq!(p(&n, &a).selected_ability, "Tera Shell");
    assert_eq!(p(&n, &a).hp, p(&n, &a).max_hp);
    assert!(p(&n, &a).runtime.will > 0);
}
#[test]
fn forms_as_one_combines_moves_two_passive_channels_and_restores_components() {
    let (mut c, a, t) = setup("Calyrex", "As One");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .workflow
        .as_mut()
        .unwrap()
        .available_abilities
        .push("Adaptability".into());
    pm(&mut c, &t).side = p(&c, &a).side.clone();
    pm(&mut c, &t).selected_ability = "Technician".into();
    let command = act(FormCommand::Union {
        combatant_id: a.clone(),
        partner_id: t.clone(),
        species_id: "0898F1:calyrex-ice-rider-form".into(),
        first_ability: "Adaptability".into(),
        second_ability: "Technician".into(),
    });
    assert!(battle::transition(&c, "player", &command, vec![], &catalog()).is_err());
    let n = decision(&c, "gm", command);
    assert!(p(&n, &a).has_ability("Adaptability") && p(&n, &a).has_ability("Technician"));
    assert!(!p(&n, &t).deployed);
    let m = catalog().find("moves", "Name", "Tackle").unwrap();
    assert_eq!(
        crate::combat::abilities::damage_bonus(&n, p(&n, &a), &m, true, &BTreeMap::new()),
        2
    );
    let mut ended = n.clone();
    forms::before_end(&mut ended).unwrap();
    assert!(p(&ended, &t).deployed);
    assert!(!p(&ended, &a).has_ability("Technician"));
}
#[test]
fn forms_commander_redirects_damage_and_blocks_the_allied_dondozo() {
    let (mut c, a, t) = setup("Tatsugiri (Curly Form)", "Commander");
    pm(&mut c, &t).side = p(&c, &a).side.clone();
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some(
        catalog().find("species", "Name", "Dondozo").unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
    let mut before = c.clone();
    before.battle = None;
    forms::lifecycle(&before, &mut c, &Action::StartTurns, &catalog()).unwrap();
    let c = choose(&c, &a, "form-commander");
    assert!(forms::blocks_recall(&c, &t));
    assert_eq!(forms::stat_bonus(&c, p(&c, &t), "Defense"), 2);
    let mut n = c.clone();
    pm(&mut n, &a).hp -= 3;
    forms::redirect_losses(&c, &mut n).unwrap();
    assert_eq!(p(&n, &a).hp, p(&c, &a).hp);
    assert_eq!(p(&n, &t).hp, p(&c, &t).hp - 3);
}
#[test]
fn forms_illusion_breaks_on_damage_and_disguise_remains_once_per_scene() {
    let (mut c, a, t) = setup("Zoroark", "Illusion");
    pm(&mut c, &t).side = p(&c, &a).side.clone();
    pm(&mut c, &t).owner_id = p(&c, &a).owner_id.clone();
    let mut before = c.clone();
    before.battle = None;
    forms::lifecycle(&before, &mut c, &Action::StartTurns, &catalog()).unwrap();
    let c = choose(&c, &a, "form-illusion");
    let mut n = c.clone();
    pm(&mut n, &a).hp -= 1;
    forms::lifecycle(&c, &mut n, &Action::StartTurns, &catalog()).unwrap();
    assert!(!n
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .forms
        .choices
        .contains_key(&format!("illusion:{a}")));
    let (mut c, a, _) = setup("Mimikyu", "Disguise");
    assert_eq!(crate::combat::extra::disguise(&mut c, &a, 5).unwrap(), 0);
    assert_eq!(crate::combat::extra::disguise(&mut c, &a, 5).unwrap(), 5);
}
#[test]
fn forms_rks_and_masks_require_gm_evidence_without_inventing_stock() {
    for (species, ability, kind, value) in [
        ("Silvally", "RKS System", "disc", "Electric"),
        ("Ogerpon (Teal Mask Form)", "Embody Aspect", "mask", "Teal"),
    ] {
        let (c, a, _) = setup(species, ability);
        let command = act(FormCommand::Equipment {
            combatant_id: a.clone(),
            kind: kind.into(),
            value: value.into(),
        });
        assert!(battle::transition(&c, "player", &command, vec![], &catalog()).is_err());
        let n = decision(&c, "gm", command);
        assert_eq!(n.sheets, c.sheets);
        if kind == "disc" {
            assert_eq!(forms::types(&n, &a).unwrap(), vec!["Electric"]);
        } else {
            assert_eq!(forms::stat_bonus(&n, p(&n, &a), "Dexterity"), 2);
        }
    }
}
#[tokio::test]
async fn forms_room_owner_receipts_stale_reconnect_persistence_privacy_and_rollback() {
    use crate::{room, store::Store};
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };
    async fn send(remote: &room::Remote, command: Command) -> Result<Campaign> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        remote.sender.send((command, tx)).await.unwrap();
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
    let (mut c, a, _) = setup("Arceus", "Multitype");
    c.host_id = ids[0].clone();
    c.gm_id = ids[2].clone();
    pm(&mut c, &a).owner_id = ids[1].clone();
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().owner_id = ids[1].clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .inventory
        .insert("Owner stock".into(), 1);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .items
        .enabled = true;
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .forms
        .originals
        .clear();
    forms::initialize(&mut c, &catalog()).unwrap();
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
    let (observer, welcome) = room::join(
        stores[3].clone(),
        cat.clone(),
        request.clone(),
        Arc::new(|_, _| {}),
    )
    .await
    .unwrap();
    assert!(!serde_json::to_string(&welcome)
        .unwrap()
        .contains("Owner stock"));
    let command = Command {
        id: "form-fire".into(),
        expected_revision: c.revision,
        action: act(FormCommand::Type {
            combatant_id: a.clone(),
            type_name: "Fire".into(),
        }),
    };
    let changed = send(&owner, command.clone()).await.unwrap();
    assert_eq!(forms::types(&changed, &a).unwrap(), vec!["Fire"]);
    assert_eq!(
        send(&owner, command.clone()).await.unwrap().revision,
        changed.revision
    );
    let mut foreign = command.clone();
    foreign.id = "foreign-form".into();
    foreign.expected_revision = changed.revision;
    assert_eq!(
        send(&observer, foreign).await.unwrap_err().code,
        "permission"
    );
    let mut stale = command.clone();
    stale.id = "stale-form".into();
    assert_eq!(send(&owner, stale).await.unwrap_err().code, "stale");
    owner.task.abort();
    let _ = owner.task.await;
    let (owner, reconnected) = room::join(stores[1].clone(), cat, request, Arc::new(|_, _| {}))
        .await
        .unwrap();
    assert_eq!(forms::types(&reconnected, &a).unwrap(), vec!["Fire"]);
    let persisted = stores[0].lock().unwrap().campaigns().unwrap().remove(0);
    assert_eq!(forms::types(&persisted, &a).unwrap(), vec!["Fire"]);
    assert_eq!(persisted.sheets[&sid].species_id, c.sheets[&sid].species_id);
    let rolled = send(
        &gm,
        Command {
            id: "undo-form".into(),
            expected_revision: changed.revision,
            action: Action::RollbackBattle {
                event_sequence: changed.battle_undo.last().unwrap().event_sequence,
                reason: "Restore the former battle typing".into(),
            },
        },
    )
    .await
    .unwrap();
    assert!(forms::types(&rolled, &a).is_none());
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
fn forms_new_battle_gates_preserve_item_and_event_engines_and_missing_state() {
    let (c, a, t) = setup("Arceus", "Multitype");
    let sheet_ids = vec![p(&c, &a).sheet_id.clone(), p(&c, &t).sheet_id.clone()];
    let ended = decision(&c, "gm", Action::EndBattle);
    for engine in [
        ENGINE,
        ITEM_COMPLETION_ENGINE,
        INTERACTION_ENGINE,
        STANDALONE_ENGINE,
        INFORMATION_ENGINE,
        FORM_ENGINE,
        ITEM_ENGINE,
        EVENT_ENGINE,
    ] {
        let mut c = ended.clone();
        c.engine = engine.into();
        let n = decision(
            &c,
            "gm",
            Action::StartBattle {
                sheet_ids: sheet_ids.clone(),
                sides: BTreeMap::new(),
            },
        );
        let flow = n.battle.as_ref().unwrap().flow.as_ref().unwrap();
        assert_eq!(
            flow.interaction_rules,
            [ENGINE, ITEM_COMPLETION_ENGINE, INTERACTION_ENGINE].contains(&engine)
        );
        assert_eq!(
            flow.items.completion.enabled,
            [ENGINE, ITEM_COMPLETION_ENGINE].contains(&engine)
        );
        assert_eq!(flow.house_rules, engine == ENGINE);
        assert_eq!(
            flow.forms.enabled,
            [
                ENGINE,
                ITEM_COMPLETION_ENGINE,
                INTERACTION_ENGINE,
                STANDALONE_ENGINE,
                INFORMATION_ENGINE,
                FORM_ENGINE
            ]
            .contains(&engine)
        );
        assert_eq!(
            flow.information.enabled,
            [
                ENGINE,
                ITEM_COMPLETION_ENGINE,
                INTERACTION_ENGINE,
                STANDALONE_ENGINE,
                INFORMATION_ENGINE
            ]
            .contains(&engine)
        );
        assert_eq!(
            flow.standalone.enabled,
            [
                ENGINE,
                ITEM_COMPLETION_ENGINE,
                INTERACTION_ENGINE,
                STANDALONE_ENGINE
            ]
            .contains(&engine)
        );
        assert_eq!(flow.items.enabled, engine != EVENT_ENGINE);
        assert!(flow.events.enabled);
        let mut data = serde_json::to_value(&n).unwrap();
        data["battle"]["flow"]
            .as_object_mut()
            .unwrap()
            .remove("forms");
        data["battle"]["flow"]
            .as_object_mut()
            .unwrap()
            .remove("interaction_rules");
        data.as_object_mut().unwrap().remove("form_memory");
        let old: Campaign = serde_json::from_value(data).unwrap();
        assert!(!forms::enabled(&old));
        assert!(!crate::combat::interactions::enabled(&old));
        assert!(old.form_memory.is_empty());
    }
}
#[test]
fn forms_transform_uses_the_move_pipeline_and_preserves_user_skills_rank_and_resources() {
    use super::targeting_tests::roll;
    let (mut c, a, t) = setup("Ditto", "");
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .stats
        .insert("Strength".into(), 6);
    c.sheets.get_mut(&sid).unwrap().rank = "Master".into();
    let original = rules::base_sheet(&c, p(&c, &a)).unwrap();
    let hp = p(&c, &a).hp;
    let c = decision(
        &c,
        "player",
        act(FormCommand::Choose {
            combatant_id: a.clone(),
            target_id: t,
        }),
    );
    let c = roll(
        &c,
        "player",
        &Action::Accuracy {
            combatant_id: a.clone(),
            target_id: Some(a.clone()),
            move_name: "Transform".into(),
            pool_modifier: 0,
            success_modifier: 0,
            reason: "Transform into the chosen visible model".into(),
        },
        4,
    );
    let sequence = c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].sequence;
    let n = roll(
        &c,
        "player",
        &Action::ResolveEffect {
            sequence,
            reason: "Copy the frozen model attributes and moves".into(),
        },
        4,
    );
    let s = rules::base_sheet(&n, p(&n, &a)).unwrap();
    assert_eq!(s.stats["Strength"], 6);
    assert_eq!(s.skills, original.skills);
    assert_eq!(s.rank, original.rank);
    assert_eq!(p(&n, &a).hp, hp);
}
#[test]
fn forms_gulp_missile_tracks_form_and_gm_recorded_retaliation_once() {
    let (mut c, a, t) = setup("Cramorant", "Gulp Missile");
    let mut q = crate::combat::QueuedAction {
        response_to: None,
        actor_id: a.clone(),
        target_id: t.clone(),
        move_name: "Surf".into(),
        sequence: 1,
        successes: 1,
        critical: false,
        timing: "normal".into(),
        priority: 0,
        cancelled: false,
        damage_done: true,
        damage: 1,
        raw_damage_successes: 1,
        ice: false,
        targets: vec![],
        field_scope: None,
        shared_dice: vec![],
        damage_modifiers: None,
        defense_key: String::new(),
        damage_context: None,
        special_index: 0,
        sequencing_penalty: false,
        sheer_force: false,
    };
    let before = p(&c, &t).hp;
    pm(&mut c, &t).hp -= 1;
    let m = catalog().find("moves", "Name", "Surf").unwrap();
    forms::hit(&mut c, &q, &m, &catalog(), before).unwrap();
    assert_eq!(
        rules::base_sheet(&c, p(&c, &a))
            .unwrap()
            .species_id
            .as_deref(),
        Some("0845F1:cramorant-gulping-form")
    );
    q.actor_id = t.clone();
    q.target_id = a.clone();
    c.battle.as_mut().unwrap().flow.as_mut().unwrap().queue = vec![q];
    let mut damaged = c.clone();
    pm(&mut damaged, &a).hp -= 1;
    forms::lifecycle(
        &c,
        &mut damaged,
        &Action::Damage {
            combatant_id: t.clone(),
            sequence: 1,
            pool_modifier: 0,
            success_modifier: 0,
            damage_modifier: 0,
            reason: "Hit the loaded Cramorant".into(),
        },
        &catalog(),
    )
    .unwrap();
    let command = act(FormCommand::Missile {
        combatant_id: a.clone(),
        target_id: t.clone(),
        status: None,
    });
    let n = battle::transition(&damaged, "gm", &command, vec![4, 4], &catalog())
        .unwrap()
        .0;
    assert_eq!(p(&n, &t).hp, p(&damaged, &t).hp - 2);
    assert_eq!(
        rules::base_sheet(&n, p(&n, &a))
            .unwrap()
            .species_id
            .as_deref(),
        Some("0845:cramorant")
    );
    assert!(battle::transition(&n, "gm", &command, vec![4, 4], &catalog()).is_err());
}
#[test]
fn forms_teraform_zero_removes_field_effects_and_negates_ordinary_abilities() {
    use crate::combat::{ability_changes, field};
    let (mut c, a, t) = setup("Terapagos (Stellar Form)", "Teraform Zero");
    pm(&mut c, &t).selected_ability = "Levitate".into();
    let flow = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    flow.ability_changes.enabled = true;
    flow.battlefield.natural = Some(field::Weather::Sunny);
    let before = c.clone();
    ability_changes::lifecycle(&before, &mut c, &catalog()).unwrap();
    forms::lifecycle(&before, &mut c, &Action::StartTurns, &catalog()).unwrap();
    assert!(p(&c, &t).selected_ability.is_empty());
    assert_eq!(p(&c, &a).selected_ability, "Teraform Zero");
    assert_eq!(field::weather(&c, None), None);
    pm(&mut c, &a).deployed = false;
    let before = c.clone();
    ability_changes::lifecycle(&before, &mut c, &catalog()).unwrap();
    assert_eq!(p(&c, &t).selected_ability, "Levitate");
    assert_eq!(field::weather(&c, None), None);
}
