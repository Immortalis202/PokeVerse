use super::targeting_tests::{attack, damage_command, multi, pass, roll};
use super::*;
use crate::combat::abilities;

fn enable(c: &mut Campaign) {
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_rules = true;
}
fn selected(c: &mut Campaign, id: &str, name: &str) {
    c.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == id)
        .unwrap()
        .selected_ability = name.into();
}
fn neutral(c: &mut Campaign, id: &str) {
    let sid = c
        .battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .find(|p| p.id == id)
        .unwrap()
        .sheet_id
        .clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some(
        catalog().find("species", "Name", "Eevee").unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
}
fn pending(name: &str, source_ability: &str, target_ability: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = fixture(name);
    enable(&mut c);
    neutral(&mut c, &a);
    neutral(&mut c, &t);
    selected(&mut c, &a, source_ability);
    selected(&mut c, &t, target_ability);
    let c = hit(&c, &a, &t, name);
    let c = pass(&c);
    (c, a, t)
}
fn trigger(c: &Campaign, choice: &str, faces: Vec<u8>) -> Campaign {
    let id = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0]
        .id
        .clone();
    battle::transition(
        c,
        "other",
        &Action::ResolveAbility {
            event_id: id,
            choice: choice.into(),
            reason: "Resolve reviewed ability".into(),
        },
        faces,
        &catalog(),
    )
    .unwrap()
    .0
}

#[test]
fn ability_damage_pools_use_reviewed_modifiers_and_critical_immunity() {
    let (base, a, _) = pending("Fire Punch", "", " ");
    let normal = rules::dice_count(&base, &damage_command(&base), &catalog()).unwrap();
    for (ability, extra) in [("Iron Fist", 1), ("Hustle", 2), ("Technician", 0)] {
        let mut c = base.clone();
        selected(&mut c, &a, ability);
        assert_eq!(
            rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap(),
            normal + extra,
            "{ability}"
        );
    }
    let (base, a, t) = pending("Tackle", "", "");
    let normal = rules::dice_count(&base, &damage_command(&base), &catalog()).unwrap();
    for ability in ["Adaptability", "Technician"] {
        let mut c = base.clone();
        selected(&mut c, &a, ability);
        assert_eq!(
            rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap(),
            normal + 1
        );
    }
    let mut c = base.clone();
    selected(&mut c, &a, "Sniper");
    assert_eq!(
        rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap(),
        normal + 1
    );
    for armor in ["Battle Armor", "Shell Armor"] {
        let mut c = c.clone();
        selected(&mut c, &t, armor);
        assert_eq!(
            rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap(),
            normal - 2
        );
    }
}

#[test]
fn ability_status_immunities_and_guts_are_derived_without_sheet_mutations() {
    for (ability, kind) in [
        ("Limber", "Paralysis"),
        ("Insomnia", "Sleep"),
        ("Vital Spirit", "Sleep"),
        ("Water Veil", "Burn 2"),
        ("Magma Armor", "Frozen Solid"),
        ("Inner Focus", "Flinch"),
    ] {
        let (mut c, _, t) = fixture("Tackle");
        enable(&mut c);
        neutral(&mut c, &t);
        selected(&mut c, &t, ability);
        let a = Action::SetStatus {
            combatant_id: t,
            kind: kind.into(),
            detail: "".into(),
            rounds: None,
            remove: false,
            reason: "Immunity".into(),
        };
        assert!(
            battle::transition(&c, "gm", &a, vec![], &catalog()).is_err(),
            "{ability}"
        );
    }
    let (mut c, a, _) = fixture("Tackle");
    enable(&mut c);
    neutral(&mut c, &a);
    selected(&mut c, &a, "Guts");
    let sid = c.battle.as_ref().unwrap().combatants[0].sheet_id.clone();
    let before = c.sheets[&sid].stats.clone();
    let c = condition(&c, &a, "Poison", None);
    assert_eq!(
        rules::effective_sheet(&c, &c.battle.as_ref().unwrap().combatants[0])
            .unwrap()
            .stats["Strength"],
        before["Strength"] + 2
    );
    assert_eq!(c.sheets[&sid].stats, before);
}

#[test]
fn contact_triggers_use_host_chance_dice_and_no_additional_action() {
    for (ability, condition_name) in [
        ("Static", "Paralysis"),
        ("Poison Point", "Poison"),
        ("Flame Body", "Burn 2"),
        ("Cute Charm", "In Love"),
        ("Effect Spore", "Sleep"),
    ] {
        let (c, a, _) = pending("Tackle", "", ability);
        let c = damage(&c, "player", 1);
        let event = &c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events[0];
        assert_eq!(event.ability, ability);
        let forbidden = Action::ResolveAbility {
            event_id: event.id.clone(),
            choice: "roll".into(),
            reason: "Not owned".into(),
        };
        assert!(battle::transition(
            &c,
            "player",
            &forbidden,
            vec![6; if ability == "Effect Spore" { 4 } else { 3 }],
            &catalog()
        )
        .is_err());
        let fail = trigger(
            &c,
            "roll",
            vec![4; if ability == "Effect Spore" { 4 } else { 3 }],
        );
        assert!(fail.battle.as_ref().unwrap().combatants[0]
            .runtime
            .statuses
            .is_empty());
        let success = trigger(
            &c,
            "roll",
            vec![6; if ability == "Effect Spore" { 4 } else { 3 }],
        );
        let p = &success.battle.as_ref().unwrap().combatants[0];
        assert!(
            p.runtime.statuses.iter().any(|s| s.kind == condition_name),
            "{ability}"
        );
        assert_eq!(p.actions, 1);
        assert_eq!(p.id, a);
        assert_eq!(success.battle.as_ref().unwrap().combatants[1].actions, 0);
    }
    let (c, _, _) = pending("Earthquake", "", "Static");
    assert!(damage(&c, "player", 1)
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events
        .is_empty());
}

#[test]
fn immunity_prevents_damage_and_absorption_healing_is_an_explicit_choice() {
    for (name, ability) in [
        ("Water Gun", "Water Absorb"),
        ("Thunder Shock", "Volt Absorb"),
    ] {
        let (mut c, _, _) = pending(name, "", ability);
        c.battle.as_mut().unwrap().combatants[1].hp = 12;
        assert_eq!(
            rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap(),
            0
        );
        let c = damage(&c, "player", 1);
        assert_eq!(c.battle.as_ref().unwrap().combatants[1].hp, 12);
        assert_eq!(
            trigger(&c, "decline", vec![])
                .battle
                .as_ref()
                .unwrap()
                .combatants[1]
                .hp,
            12
        );
        assert_eq!(
            trigger(&c, "heal", vec![])
                .battle
                .as_ref()
                .unwrap()
                .combatants[1]
                .hp,
            13
        );
    }
    for (name, ability) in [
        ("Earthquake", "Levitate"),
        ("Hyper Voice", "Soundproof"),
        ("Ember", "Flash Fire"),
        ("Vine Whip", "Sap Sipper"),
    ] {
        let (c, _, _) = pending(name, "", ability);
        assert_eq!(
            rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap(),
            0,
            "{ability}"
        );
        let c = damage(&c, "player", 1);
        assert_eq!(c.battle.as_ref().unwrap().combatants[1].hp, 20);
    }
}

#[test]
fn rough_skin_choice_sturdy_and_aftermath_use_distinct_damage_sources() {
    for ability in ["Rough Skin", "Iron Barbs"] {
        let (c, _, _) = pending("Tackle", "", ability);
        let c = damage(&c, "player", 1);
        assert_eq!(
            trigger(&c, "neutral", vec![])
                .battle
                .as_ref()
                .unwrap()
                .combatants[0]
                .hp,
            19
        );
        assert_eq!(
            trigger(&c, "roll", vec![4, 4])
                .battle
                .as_ref()
                .unwrap()
                .combatants[0]
                .hp,
            18
        );
    }
    let (mut c, _, _) = pending("Tackle", "", "Sturdy");
    c.battle.as_mut().unwrap().combatants[1].hp = 1;
    let saved = damage(&c, "player", 4);
    assert_eq!(saved.battle.as_ref().unwrap().combatants[1].hp, 1);
    assert!(saved.battle.as_ref().unwrap().combatants[1]
        .runtime
        .ability
        .flags
        .contains("Sturdy"));
    c.battle.as_mut().unwrap().combatants[1]
        .runtime
        .ability
        .flags
        .insert("Sturdy".into());
    assert_eq!(
        damage(&c, "player", 4).battle.as_ref().unwrap().combatants[1].hp,
        0
    );
    let (mut c, _, _) = pending("Tackle", "Sturdy", "Aftermath");
    c.battle.as_mut().unwrap().combatants[0].hp = 1;
    c.battle.as_mut().unwrap().combatants[1].hp = 1;
    assert_eq!(
        damage(&c, "player", 4).battle.as_ref().unwrap().combatants[0].hp,
        0
    );
}

#[test]
fn synchronize_reflects_once_and_shield_dust_blocks_foe_chance_effects() {
    let (c, _, _) = pending("Thunder Shock", "Synchronize", "Synchronize");
    let c = damage(&c, "player", 1);
    let c = effect(&c, 6);
    for p in &c.battle.as_ref().unwrap().combatants {
        assert_eq!(
            p.runtime
                .statuses
                .iter()
                .filter(|s| s.kind == "Paralysis")
                .count(),
            1
        );
    }
    let (c, _, _) = pending("Thunder Shock", "Limber", "Synchronize");
    let c = effect(&damage(&c, "player", 1), 6);
    assert!(c.battle.as_ref().unwrap().combatants[0]
        .runtime
        .statuses
        .is_empty());
    let (c, _, _) = pending("Thunder Shock", "", "Shield Dust");
    let c = effect(&damage(&c, "player", 1), 6);
    assert!(c.battle.as_ref().unwrap().combatants[1]
        .runtime
        .statuses
        .is_empty());
}

#[test]
fn intimidate_entries_do_not_stack_and_expire_when_sources_leave() {
    let (mut c, a, t, high) = multi("Tackle");
    enable(&mut c);
    selected(&mut c, &t, "Intimidate");
    selected(&mut c, &high, "Intimidate");
    let mut before = c.clone();
    before.battle.as_mut().unwrap().active = false;
    abilities::lifecycle(&before, &mut c).unwrap();
    assert_eq!(
        rules::effective_sheet(&c, &c.battle.as_ref().unwrap().combatants[0])
            .unwrap()
            .stats["Strength"],
        3
    );
    let before = c.clone();
    c.battle.as_mut().unwrap().combatants[1].deployed = false;
    abilities::lifecycle(&before, &mut c).unwrap();
    assert_eq!(
        rules::effective_sheet(&c, &c.battle.as_ref().unwrap().combatants[0])
            .unwrap()
            .stats["Strength"],
        3
    );
    let before = c.clone();
    c.battle.as_mut().unwrap().combatants[2].hp = 0;
    abilities::lifecycle(&before, &mut c).unwrap();
    assert_eq!(
        rules::effective_sheet(&c, &c.battle.as_ref().unwrap().combatants[0])
            .unwrap()
            .stats["Strength"],
        4
    );
    selected(&mut c, &a, "Inner Focus");
}

#[test]
fn moxie_does_not_change_damage_dice_midway_through_an_area_move() {
    let (mut c, a, t, high) = multi("Earthquake");
    enable(&mut c);
    selected(&mut c, &a, "Moxie");
    neutral(&mut c, &t);
    neutral(&mut c, &high);
    c.battle.as_mut().unwrap().combatants[2].hp = 1;
    let c = roll(&c, "player", &attack(&a, vec![], "Earthquake"), 4);
    let c = pass(&pass(&c));
    let mut low = c.clone();
    low.battle.as_mut().unwrap().flow.as_mut().unwrap().queue[0].target_id = t.clone();
    let total_low = rules::dice_count(&low, &damage_command(&low), &catalog()).unwrap();
    let c = damage(&c, "player", 4);
    assert_eq!(
        c.battle.as_ref().unwrap().combatants[0]
            .runtime
            .ability
            .counters["Moxie"],
        1
    );
    let q = c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].clone();
    let retained = q.shared_dice.len() as u32;
    let c = decision(
        &c,
        "gm",
        Action::AdjudicateEffect {
            sequence: q.sequence,
            reason: "Other Earthquake effects resolved".into(),
        },
    );
    assert_eq!(
        rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap(),
        total_low - retained
    );
}

#[test]
fn speed_boost_caps_and_pending_triggers_survive_rollback() {
    let (mut c, a, _) = fixture("Tackle");
    enable(&mut c);
    selected(&mut c, &a, "Speed Boost");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .order
        .clear();
    for _ in 0..4 {
        c = decision(&c, "gm", Action::NextRound);
    }
    assert_eq!(
        c.battle.as_ref().unwrap().combatants[0]
            .runtime
            .ability
            .counters["Speed Boost"],
        3
    );
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].initiative, 13);
    c.battle.as_mut().unwrap().combatants[0].hp = 0;
    let before = c.clone();
    c.battle.as_mut().unwrap().combatants[0].deployed = false;
    abilities::lifecycle(&before, &mut c).unwrap();
    assert!(c.battle.as_ref().unwrap().combatants[0]
        .runtime
        .ability
        .counters
        .is_empty());
    let (c, _, _) = pending("Tackle", "", "Rough Skin");
    let c = damage(&c, "player", 1);
    let restored: Campaign = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
    let next = trigger(&restored, "neutral", vec![]);
    let undo = decision(
        &next,
        "gm",
        Action::RollbackBattle {
            event_sequence: c.revision + 1,
            reason: "Undo only ability reaction".into(),
        },
    );
    assert_eq!(undo.battle.as_ref().unwrap().combatants[0].hp, 20);
    assert_eq!(
        undo.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events
            .len(),
        1
    );
}

#[test]
fn reductions_marvel_scale_and_hustle_apply_to_the_correct_rolls() {
    for (name, ability, reduction) in [
        ("Tackle", "Fur Coat", 2),
        ("Tackle", "Multiscale", 1),
        ("Water Gun", "Filter", 1),
        ("Water Gun", "Solid Rock", 1),
    ] {
        let (mut base, _, t) = pending(name, "", "");
        if name == "Water Gun" {
            let sid = base.battle.as_ref().unwrap().combatants[1].sheet_id.clone();
            base.sheets.get_mut(&sid).unwrap().species_id = Some(
                catalog().find("species", "Name", "Charmander").unwrap()["AppId"]
                    .as_str()
                    .unwrap()
                    .into(),
            );
        }
        let mut protected = base.clone();
        selected(&mut protected, &t, ability);
        let ordinary = damage(&base, "player", 4);
        let protected = damage(&protected, "player", 4);
        assert_eq!(
            protected.battle.as_ref().unwrap().combatants[1].hp,
            ordinary.battle.as_ref().unwrap().combatants[1].hp + reduction,
            "{ability}"
        );
    }
    for (name, delta) in [("Tackle", 2), ("Water Gun", 0)] {
        let (mut c, _, t) = pending(name, "", "");
        let base = rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap();
        selected(&mut c, &t, "Marvel Scale");
        c.battle.as_mut().unwrap().combatants[1]
            .runtime
            .statuses
            .push(crate::combat::Status {
                kind: "Poison".into(),
                detail: "".into(),
                rounds: None,
                progress: 0,
            });
        assert_eq!(
            rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap(),
            base - delta
        );
    }
    let (mut c, a, t) = fixture("Tackle");
    enable(&mut c);
    let normal = hit(&c, &a, &t, "Tackle")
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue[0]
        .successes;
    selected(&mut c, &a, "Hustle");
    assert_eq!(
        hit(&c, &a, &t, "Tackle")
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .queue[0]
            .successes,
        normal - 1
    );
}

#[test]
fn poison_immunity_prevents_ticks_but_keeps_the_condition() {
    let (mut c, a, _) = fixture("Tackle");
    enable(&mut c);
    neutral(&mut c, &a);
    selected(&mut c, &a, "Immunity");
    let c = condition(&c, &a, "Poison", None);
    let mut c = condition(&c, &a, "Poison", None);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .order
        .clear();
    let next = decision(&c, "gm", Action::NextRound);
    assert_eq!(next.battle.as_ref().unwrap().combatants[0].hp, 20);
    assert!(next.battle.as_ref().unwrap().combatants[0]
        .runtime
        .statuses
        .iter()
        .any(|s| s.kind == "Badly Poisoned"));
}

#[test]
fn first_hit_flags_and_pending_phase_guards_are_explicit() {
    for (name, ability) in [("Ember", "Flash Fire"), ("Vine Whip", "Sap Sipper")] {
        let (c, _, t) = pending(name, "", ability);
        let c = damage(&c, "player", 1);
        assert!(c.battle.as_ref().unwrap().combatants[1]
            .runtime
            .ability
            .flags
            .contains(ability));
        if ability == "Sap Sipper" {
            assert_eq!(
                rules::effective_sheet(&c, &c.battle.as_ref().unwrap().combatants[1])
                    .unwrap()
                    .stats["Strength"],
                5
            );
        }
        assert_eq!(c.battle.as_ref().unwrap().combatants[1].id, t);
    }
    let (c, _, t) = pending("Tackle", "", "Static");
    let c = damage(&c, "player", 1);
    for a in [
        Action::NextRound,
        Action::EndBattle,
        Action::PassTurn {
            combatant_id: t,
            reason: "Cannot skip trigger".into(),
        },
    ] {
        assert!(battle::transition(&c, "gm", &a, vec![], &catalog()).is_err());
    }
    let (mut c, _, _) = pending("Earthquake", "", "Levitate");
    c.battle.as_mut().unwrap().combatants[0].selected_ability = "Mold Breaker".into();
    assert_eq!(
        rules::dice_count(&c, &damage_command(&c), &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
}

#[tokio::test]
async fn ability_owner_receipts_reconnect_and_gm_rollback_are_host_validated() {
    let hd = tempfile::tempdir().unwrap();
    let pd = tempfile::tempdir().unwrap();
    let gd = tempfile::tempdir().unwrap();
    let mut store = Store::open(hd.path()).unwrap();
    let ps = Arc::new(Mutex::new(Store::open(pd.path()).unwrap()));
    let gs = Arc::new(Mutex::new(Store::open(gd.path()).unwrap()));
    let player = ps.lock().unwrap().player_id.clone();
    let gm_id = gs.lock().unwrap().player_id.clone();
    let (c, _, _) = pending("Tackle", "", "Rough Skin");
    let mut c = damage(&c, "player", 1);
    c.host_id = store.player_id.clone();
    c.gm_id = gm_id.clone();
    c.battle.as_mut().unwrap().combatants[1].owner_id = player.clone();
    let campaign_id = c.id.clone();
    let event = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0]
        .id
        .clone();
    store.put_campaign(&c).unwrap();
    let shared = Arc::new(Mutex::new(store));
    let cat = Arc::new(catalog());
    let notify: room::Notify = Arc::new(|_, _| {});
    let mut host = room::start(shared.clone(), cat.clone(), campaign_id, 0, notify.clone())
        .await
        .unwrap();
    let request = JoinRequest {
        address: host.info.address.replace("<host-ip>", "127.0.0.1"),
        token: host.info.token.clone(),
    };
    let (remote, snapshot) = room::join(ps.clone(), cat.clone(), request.clone(), notify.clone())
        .await
        .unwrap();
    assert_eq!(
        snapshot
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events[0]
            .id,
        event
    );
    let command = Command {
        id: id(),
        expected_revision: c.revision,
        action: Action::ResolveAbility {
            event_id: event.clone(),
            choice: "neutral".into(),
            reason: "Owner selected neutral damage".into(),
        },
    };
    assert!(host.host.apply(c.host_id(), &command).is_err());
    let (tx, rx) = tokio::sync::oneshot::channel();
    remote.sender.send((command.clone(), tx)).await.unwrap();
    let next = rx.await.unwrap().unwrap();
    assert_eq!(next.battle.as_ref().unwrap().combatants[0].hp, 19);
    assert_eq!(
        host.host.apply(&player, &command).unwrap().revision,
        next.revision
    );
    remote.task.abort();
    let (reconnected, snapshot) = room::join(ps, cat.clone(), request.clone(), notify.clone())
        .await
        .unwrap();
    assert!(snapshot
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events
        .is_empty());
    let (gm, _) = room::join(gs, cat, request, notify).await.unwrap();
    let undo = Command {
        id: id(),
        expected_revision: next.revision,
        action: Action::RollbackBattle {
            event_sequence: next.revision,
            reason: "GM corrects ability choice".into(),
        },
    };
    let (tx, rx) = tokio::sync::oneshot::channel();
    gm.sender.send((undo, tx)).await.unwrap();
    let restored = rx.await.unwrap().unwrap();
    assert_eq!(restored.battle.as_ref().unwrap().combatants[0].hp, 20);
    assert_eq!(
        restored
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events[0]
            .id,
        event
    );
    assert_eq!(
        host.host.apply(&player, &command).unwrap().revision,
        restored.revision
    );
    assert_eq!(
        shared.lock().unwrap().campaigns().unwrap()[0].revision,
        restored.revision
    );
    gm.task.abort();
    reconnected.task.abort();
    let _ = host.host.shutdown.send(true);
    let _ = host.stop.take().unwrap().send(());
}

#[test]
fn damp_long_reach_magic_guard_and_set_damage_interactions_are_explicit() {
    let (c, _, _) = pending("Tackle", "Long Reach", "Static");
    assert!(damage(&c, "player", 4)
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events
        .is_empty());
    let (mut c, _, _) = pending("Tackle", "Damp", "Aftermath");
    c.battle.as_mut().unwrap().combatants[1].hp = 1;
    assert_eq!(
        damage(&c, "player", 4).battle.as_ref().unwrap().combatants[0].hp,
        20
    );
    let (c, _, _) = pending("Tackle", "Magic Guard", "Rough Skin");
    let c = damage(&c, "player", 4);
    assert_eq!(
        trigger(&c, "neutral", vec![])
            .battle
            .as_ref()
            .unwrap()
            .combatants[0]
            .hp,
        20
    );
    let (c, _, _) = pending("Submission", "Magic Guard", "");
    let c = damage(&c, "player", 4);
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    assert_eq!(
        rules::dice_count(
            &c,
            &Action::ResolveEffect {
                sequence: q.sequence,
                reason: "No recoil".into()
            },
            &catalog()
        )
        .unwrap(),
        0
    );
    assert_eq!(effect(&c, 4).battle.as_ref().unwrap().combatants[0].hp, 20);
    let (mut c, a, t) = fixture("Explosion");
    enable(&mut c);
    selected(&mut c, &t, "Damp");
    assert!(rules::dice_count(
        &c,
        &Action::Accuracy {
            combatant_id: a,
            target_id: Some(t),
            move_name: "Explosion".into(),
            pool_modifier: 0,
            success_modifier: 0,
            reason: "Damp forbids".into()
        },
        &catalog()
    )
    .is_err());
    let (mut c, _, _) = pending("Sonic Boom", "Moxie", "Sturdy");
    c.battle.as_mut().unwrap().combatants[1].hp = 1;
    let c = damage(&c, "player", 4);
    assert_eq!(c.battle.as_ref().unwrap().combatants[1].hp, 0);
    assert_eq!(
        c.battle.as_ref().unwrap().combatants[0]
            .runtime
            .ability
            .counters["Moxie"],
        1
    );
}

#[test]
fn multi_target_damage_finishes_even_if_contact_retaliation_faints_its_attacker() {
    let (mut c, a, t, high) = multi("Brutal Swing");
    enable(&mut c);
    neutral(&mut c, &a);
    neutral(&mut c, &t);
    neutral(&mut c, &high);
    selected(&mut c, &high, "Rough Skin");
    c.battle.as_mut().unwrap().combatants[0].hp = 1;
    let c = roll(
        &c,
        "player",
        &attack(&a, vec![t.clone(), high], "Brutal Swing"),
        4,
    );
    let c = pass(&pass(&c));
    let c = damage(&c, "player", 4);
    let c = trigger(&c, "neutral", vec![]);
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].hp, 0);
    let q = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0];
    let c = decision(
        &c,
        "gm",
        Action::AdjudicateEffect {
            sequence: q.sequence,
            reason: "Manual Brutal Swing effects".into(),
        },
    );
    assert_eq!(
        c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].target_id,
        t
    );
    let c = damage(&c, "player", 4);
    assert!(c.battle.as_ref().unwrap().combatants[1].hp < 20);
    assert_eq!(c.battle.as_ref().unwrap().combatants[0].hp, 0);
}
