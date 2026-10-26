use super::targeting_tests::{damage_command, pass, roll};
use super::*;
use crate::combat::{abilities, extra, field, reviewed};

fn ordinary_pool(c: &Campaign) -> rules::DamagePool {
    let mut temp = c.clone();
    let q = queued(c);
    pm(&mut temp, &q.actor_id).pending_damage = Some(PendingDamage {
        sequence: q.sequence,
        target_id: q.target_id.clone(),
        move_name: q.move_name.clone(),
        critical: q.critical,
    });
    rules::damage_pool(&temp, &q.actor_id, q.sequence, 0, &catalog()).unwrap()
}

fn sheer_choice(id: &str, ignore: bool) -> Action {
    Action::ChooseSheerForce {
        combatant_id: id.into(),
        ignore_added_effects: ignore,
        reason: "Owner chooses the printed Sheer Force tradeoff".into(),
    }
}

#[test]
fn provenance_sheer_force_charge_preserves_original_choice_through_release() {
    for initial in [false, true] {
        let (mut c, a, t) = setup("Bounce", "Sheer Force", "");
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .provenance_rules = true;
        let c = decision(&c, "player", sheer_choice(&a, initial));
        let c = hit(&c, &a, &t, "Bounce");
        let f = c.battle.as_ref().unwrap().flow.as_ref().unwrap();
        assert!(f.queue.is_empty());
        assert_eq!(f.special.charges[&a].action.sheer_force, initial);
        let mut c = decision(&c, "player", sheer_choice(&a, !initial));
        let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
        f.cursor = f.order.iter().position(|id| id == &a).unwrap();
        f.turn += 1;
        let c = decision(
            &c,
            "player",
            Action::SpecialMove {
                command: crate::combat::special::SpecialCommand::Release {
                    combatant_id: a.clone(),
                },
                reason: "Release stored charge".into(),
            },
        );
        assert_eq!(queued(&c).sheer_force, initial);
        let c = pass(&c);
        let done = dmg(&c);
        assert!(p(&done, &t).hp < 20);
        if initial {
            assert!(done
                .battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .queue
                .is_empty());
            assert!(!p(&done, &t)
                .runtime
                .statuses
                .iter()
                .any(|s| s.kind == "Paralysis"));
        }
    }
}

#[test]
fn provenance_sheer_force_shared_targets_retain_choice_after_identity_change() {
    let (mut c, a, t, h) = super::targeting_tests::multi("Discharge");
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.provenance_rules = true;
    f.reviewed_rules = true;
    f.extra.enabled = true;
    f.special.enabled = true;
    f.ability_rules = true;
    f.battlefield.enabled = true;
    pm(&mut c, &a).selected_ability = "Sheer Force".into();
    let c = decision(&c, "player", sheer_choice(&a, true));
    let action = super::targeting_tests::attack(&a, vec![t, h], "Discharge");
    let mut c = roll(&c, "player", &action, 6);
    while c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
        c = pass(&c);
    }
    assert!(queued(&c).sheer_force);
    let mut c = dmg(&c);
    assert!(queued(&c).sheer_force);
    pm(&mut c, &a).selected_ability.clear();
    let boosted = ordinary_pool(&c).dice;
    let mut plain = c.clone();
    plain.battle.as_mut().unwrap().flow.as_mut().unwrap().queue[0].sheer_force = false;
    assert_eq!(boosted, ordinary_pool(&plain).dice + 2);
    let done = dmg(&c);
    assert!(done
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
fn provenance_sheer_force_successive_hits_each_keep_damage_and_suppress_poison() {
    let (mut c, a, t) = setup("Twineedle", "Sheer Force", "");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .provenance_rules = true;
    let c = decision(&c, "player", sheer_choice(&a, true));
    let action = Action::SpecialMove {
        command: crate::combat::special::SpecialCommand::Successive {
            reaction: false,
            combatant_id: a.clone(),
            target_id: t.clone(),
            move_name: "Twineedle".into(),
            attempts: 2,
        },
        reason: "Two reviewed successive attacks".into(),
    };
    let mut c = roll(&c, "player", &action, 6);
    let mut hits = 0;
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
        if c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
            c = pass(&c);
            continue;
        }
        assert!(queued(&c).sheer_force);
        c = dmg(&c);
        hits += 1;
    }
    assert_eq!(hits, 2);
    assert!(!p(&c, &t)
        .runtime
        .statuses
        .iter()
        .any(|s| s.kind.contains("Poison")));
}

#[test]
fn provenance_stakeout_clash_consumes_once_and_preserves_type_immunity() {
    let (mut c, a, t) = setup("Tackle", "Stakeout", "");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .provenance_rules = true;
    let flag = format!("stakeout:{t}");
    pm(&mut c, &a).runtime.ability.flags.insert(flag.clone());
    let data = m(&c, &a, "Tackle");
    let damage = abilities::clash_amount(&mut c, &t, &data, 1, &a, &catalog()).unwrap();
    assert_eq!(damage, 2);
    pm(&mut c, &a).runtime.ability.flags.insert(flag.clone());
    pm(&mut c, &t).selected_ability = "Filter".into();
    assert_eq!(
        abilities::clash_amount(&mut c, &t, &data, 1, &a, &catalog()).unwrap(),
        2
    );
    pm(&mut c, &t).selected_ability.clear();
    assert_eq!(
        abilities::clash_amount(&mut c, &t, &data, 1, &a, &catalog()).unwrap(),
        1
    );
    pm(&mut c, &a).runtime.ability.flags.insert(flag.clone());
    field::change_type(&mut c, &t, "replace", &["Ghost".into()]).unwrap();
    assert_eq!(
        crate::combat::provenance::clash_bonus(&c, &a, &t, &data, &catalog()).unwrap(),
        0
    );
    assert!(p(&c, &a).runtime.ability.flags.contains(&flag));
}

#[test]
fn provenance_sheer_force_suppresses_chance_but_retains_recoil_attribute() {
    let (mut c, a, t) = setup("Volt Tackle", "Sheer Force", "");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .provenance_rules = true;
    let c = decision(&c, "player", sheer_choice(&a, true));
    let c = pending(&c, &a, &t, "Volt Tackle");
    let c = dmg(&c);
    let q = queued(&c);
    assert!(q.sheer_force && q.damage_done);
    let effect = Action::ResolveEffect {
        sequence: q.sequence,
        reason: "Resolve mandatory recoil only".into(),
    };
    assert_eq!(
        rules::dice_count(&c, &effect, &catalog()).unwrap(),
        q.raw_damage_successes
    );
    let before = p(&c, &a).hp;
    let c = roll(&c, "player", &effect, 6);
    assert_eq!(
        p(&c, &a).hp,
        (before - q.raw_damage_successes as i32).max(0)
    );
    assert!(!p(&c, &t)
        .runtime
        .statuses
        .iter()
        .any(|s| s.kind == "Paralysis"));
}

#[tokio::test]
async fn provenance_room_owner_choice_retry_reconnect_persistence_and_gm_rollback() {
    async fn send(remote: &room::Remote, command: Command) -> Campaign {
        let (tx, rx) = tokio::sync::oneshot::channel();
        remote.sender.send((command, tx)).await.unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(5), rx)
            .await
            .unwrap()
            .unwrap()
            .unwrap()
    }
    let dirs: Vec<_> = (0..3).map(|_| tempfile::tempdir().unwrap()).collect();
    let stores: Vec<_> = dirs
        .iter()
        .map(|d| Arc::new(Mutex::new(Store::open(d.path()).unwrap())))
        .collect();
    let ids: Vec<_> = stores
        .iter()
        .map(|s| s.lock().unwrap().player_id.clone())
        .collect();
    let (mut c, a, _) = setup("Ember", "Sheer Force", "");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .provenance_rules = true;
    c.host_id = ids[0].clone();
    c.gm_id = ids[1].clone();
    pm(&mut c, &a).owner_id = ids[2].clone();
    let sheet = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sheet).unwrap().owner_id = ids[2].clone();
    stores[0].lock().unwrap().put_campaign(&c).unwrap();
    let cat = Arc::new(catalog());
    let notify: room::Notify = Arc::new(|_, _| {});
    let mut host = room::start(
        stores[0].clone(),
        cat.clone(),
        c.id.clone(),
        0,
        notify.clone(),
    )
    .await
    .unwrap();
    let join = JoinRequest {
        address: host.info.address.replace("<host-ip>", "127.0.0.1"),
        token: host.info.token.clone(),
    };
    let (gm, _) = room::join(stores[1].clone(), cat.clone(), join.clone(), notify.clone())
        .await
        .unwrap();
    let (owner, _) = room::join(stores[2].clone(), cat.clone(), join.clone(), notify.clone())
        .await
        .unwrap();
    let before = serde_json::to_value(&c.battle).unwrap();
    let choose = Command {
        id: id(),
        expected_revision: c.revision,
        action: sheer_choice(&a, true),
    };
    assert!(host.host.apply(&ids[0], &choose).is_err());
    let c = send(&owner, choose.clone()).await;
    assert!(p(&c, &a)
        .runtime
        .ability
        .flags
        .contains("sheer-force:damage"));
    assert_eq!(
        host.host.apply(&ids[2], &choose).unwrap().revision,
        c.revision
    );
    let mut stale = choose.clone();
    stale.id = id();
    assert!(host.host.apply(&ids[2], &stale).is_err());
    owner.task.abort();
    let (owner, snapshot) = room::join(stores[2].clone(), cat, join, notify)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(&snapshot.battle).unwrap(),
        serde_json::to_value(&c.battle).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&stores[0].lock().unwrap().campaigns().unwrap()[0].battle).unwrap(),
        serde_json::to_value(&c.battle).unwrap()
    );
    let undo = Command {
        id: id(),
        expected_revision: c.revision,
        action: Action::RollbackBattle {
            event_sequence: c.battle_undo.last().unwrap().event_sequence,
            reason: "Restore the previous Sheer Force choice".into(),
        },
    };
    let restored = send(&gm, undo).await;
    assert_eq!(serde_json::to_value(&restored.battle).unwrap(), before);
    assert_eq!(
        host.host.apply(&ids[2], &choose).unwrap().revision,
        restored.revision
    );
    gm.task.abort();
    owner.task.abort();
    let _ = host.stop.take().unwrap().send(());
}

#[test]
fn provenance_sheer_force_is_optional_owned_and_suppresses_added_effects() {
    let (mut c, a, t) = setup("Ember", "Sheer Force", "");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .provenance_rules = true;
    assert!(battle::transition(&c, "other", &sheer_choice(&a, true), vec![], &catalog()).is_err());
    let chosen = decision(&c, "player", sheer_choice(&a, true));
    assert_eq!(p(&c, &a).actions, p(&chosen, &a).actions);
    let normal = pending(&c, &a, &t, "Ember");
    let boosted = pending(&chosen, &a, &t, "Ember");
    assert!(!queued(&normal).sheer_force);
    assert!(queued(&boosted).sheer_force);
    assert_eq!(
        ordinary_pool(&boosted).dice,
        ordinary_pool(&normal).dice + 2
    );
    assert!(battle::transition(
        &boosted,
        "player",
        &sheer_choice(&a, false),
        vec![],
        &catalog()
    )
    .is_err());
    let done = dmg(&boosted);
    assert!(done
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .queue
        .is_empty());
    assert!(p(&done, &t)
        .runtime
        .statuses
        .iter()
        .all(|s| !s.kind.starts_with("Burn")));
    let disabled = decision(&chosen, "player", sheer_choice(&a, false));
    assert!(!queued(&pending(&disabled, &a, &t, "Ember")).sheer_force);
}

#[test]
fn provenance_sheer_force_does_not_boost_moves_without_added_effects() {
    let (mut c, a, t) = setup("Tackle", "Sheer Force", "");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .provenance_rules = true;
    let chosen = decision(&c, "player", sheer_choice(&a, true));
    let ordinary = pending(&c, &a, &t, "Tackle");
    let c = pending(&chosen, &a, &t, "Tackle");
    assert!(!queued(&c).sheer_force);
    assert_eq!(ordinary_pool(&c).dice, ordinary_pool(&ordinary).dice);
}

#[test]
fn provenance_tinted_lens_ignores_exactly_one_resistance_and_preserves_immunity() {
    for (species, ty, original, expected, immune) in [
        ("Squirtle", "Fire", -1, 0, false),
        ("Kingdra", "Fire", -2, -1, false),
        ("Bulbasaur", "Fire", 1, 1, false),
        ("Gengar", "Normal", 0, 0, true),
        // A weakness and resistance cancel normally; the resistance is still ignored.
        ("Gyarados", "Ice", 0, 1, false),
    ] {
        let (mut c, a, t) = setup("Ember", "Tinted Lens", "");
        let sid = p(&c, &t).sheet_id.clone();
        c.sheets.get_mut(&sid).unwrap().species_id = Some(
            catalog().find("species", "Name", species).unwrap()["AppId"]
                .as_str()
                .unwrap()
                .into(),
        );
        let matchup =
            |c: &Campaign| rules::type_matchup(c, p(c, &a), p(c, &t), ty, &catalog()).unwrap();
        assert_eq!(matchup(&c), (original, immune), "historical {species}");
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .provenance_rules = true;
        assert_eq!(matchup(&c), (expected, immune), "{species}");
        pm(&mut c, &a).selected_ability.clear();
        assert_eq!(matchup(&c), (original, immune), "suppressed {species}");
    }
}

#[test]
fn provenance_old_saved_flows_default_to_disabled() {
    let (c, _, _) = setup("Ember", "Tinted Lens", "");
    let mut value = serde_json::to_value(c.battle.unwrap().flow.unwrap()).unwrap();
    value.as_object_mut().unwrap().remove("provenance_rules");
    let flow: TurnFlow = serde_json::from_value(value).unwrap();
    assert!(!flow.provenance_rules);
}

#[test]
fn provenance_new_battles_enable_rules_and_historical_sequencing_engine_stays_compatible() {
    let (c, a, t) = setup("Tackle", "", "");
    let sheets = vec![p(&c, &a).sheet_id.clone(), p(&c, &t).sheet_id.clone()];
    let ended = decision(&c, "gm", Action::EndBattle);
    for (engine, expected) in [
        (ENGINE, true),
        (ITEM_COMPLETION_ENGINE, true),
        (SEQUENCING_ENGINE, false),
    ] {
        let mut c = ended.clone();
        c.engine = engine.into();
        let c = decision(
            &c,
            "gm",
            Action::StartBattle {
                environment: None,
                sheet_ids: sheets.clone(),
                sides: BTreeMap::new(),
            },
        );
        let f = c.battle.as_ref().unwrap().flow.as_ref().unwrap();
        assert_eq!(f.provenance_rules, expected);
        assert!(f.sequencing.enabled);
        assert!(f.stat_channels.enabled && f.reviewed_rules);
    }
}

#[test]
fn provenance_unaware_ignores_foe_attributes_in_both_directions() {
    for (attacker, defender, key, expected_attribute, expected_defense) in [
        ("Unaware", "", "Vitality", 4, 4),
        ("", "Unaware", "Strength", 4, 4),
        ("Unaware", "", "Strength", 6, 4),
    ] {
        let (mut c, a, t) = setup("Tackle", attacker, defender);
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .provenance_rules = true;
        let id = if key == "Vitality" { &t } else { &a };
        pm(&mut c, id).runtime.modifiers.insert(key.into(), 2);
        let c = pending(&c, &a, &t, "Tackle");
        let pool = ordinary_pool(&c);
        assert_eq!(
            pool.attribute, expected_attribute,
            "{attacker}/{defender}/{key}"
        );
        assert_eq!(
            pool.defense, expected_defense,
            "{attacker}/{defender}/{key}"
        );
    }
}

#[test]
fn provenance_unaware_preserves_explicit_defense_channels_and_gm_baselines() {
    let (mut c, a, t) = setup("Tackle", "Unaware", "");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .provenance_rules = true;
    pm(&mut c, &t).stat_overrides.insert("Vitality".into(), 5);
    pm(&mut c, &t)
        .runtime
        .modifiers
        .insert("Vitality".into(), -3);
    pm(&mut c, &t)
        .runtime
        .ability
        .counters
        .insert("boost:Defense".into(), 2);
    let c = pending(&c, &a, &t, "Tackle");
    let pool = ordinary_pool(&c);
    assert_eq!(pool.defense, 7);
    assert_eq!(
        rules::effective_sheet(&c, p(&c, &t)).unwrap().stats["Vitality"],
        2
    );
    assert_eq!(
        rules::base_sheet(&c, p(&c, &t)).unwrap().stats["Vitality"],
        5
    );
}

#[test]
fn provenance_stakeout_tracks_real_replacement_and_adds_damage_only_once() {
    let (mut c, a, t) = setup("Tackle", "Stakeout", "");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .provenance_rules = true;
    let mut reserve = p(&c, &t).clone();
    let mut sheet = c.sheets[&reserve.sheet_id].clone();
    sheet.id = crate::model::id();
    reserve.sheet_id = sheet.id.clone();
    reserve.id = crate::model::id();
    reserve.deployed = false;
    reserve.field_slot = None;
    let incoming = reserve.id.clone();
    c.sheets.insert(sheet.id.clone(), sheet);
    c.battle.as_mut().unwrap().slots_per_side = 1;
    pm(&mut c, &t).field_slot = Some(0);
    c.battle.as_mut().unwrap().combatants.push(reserve);
    let action = Action::SwitchPokemon {
        outgoing_id: t.clone(),
        incoming_id: incoming.clone(),
        gm_override: true,
        reason: "Reviewed replacement timing".into(),
    };
    let c = decision(&c, "gm", action);
    let flag = format!("stakeout:{incoming}");
    assert!(p(&c, &a).runtime.ability.flags.contains(&flag));
    let c = pending(&c, &a, &incoming, "Tackle");
    let mut control = c.clone();
    pm(&mut control, &a).runtime.ability.flags.remove(&flag);
    let normal = dmg(&control);
    let boosted = dmg(&c);
    assert_eq!(p(&normal, &incoming).hp - p(&boosted, &incoming).hp, 1);
    assert!(!p(&boosted, &a).runtime.ability.flags.contains(&flag));
    assert!(p(&c, &a).runtime.ability.flags.contains(&flag));
}

#[test]
fn provenance_unaware_shared_defense_snapshot_preserves_neutral_attributes() {
    let (mut c, a, t) = setup("Tackle", "Unaware", "");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .provenance_rules = true;
    pm(&mut c, &t)
        .runtime
        .modifiers
        .insert("Vitality".into(), 3);
    let snap = field::snapshot(&c, &catalog()).unwrap();
    assert_eq!(snap.defenses[&t][0], 7);
    assert_eq!(snap.base_attribute_defenses[&t][0], 4);
    let c = pending(&c, &a, &t, "Tackle");
    assert_eq!(ordinary_pool(&c).defense, 4);
}

#[test]
fn provenance_unaware_uses_frozen_base_attributes_for_later_shared_targets() {
    let (mut c, a, t, h) = super::targeting_tests::multi("Swift");
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.provenance_rules = true;
    f.reviewed_rules = true;
    f.extra.enabled = true;
    f.special.enabled = true;
    f.ability_rules = true;
    f.battlefield.enabled = true;
    pm(&mut c, &h).selected_ability = "Unaware".into();
    let base = rules::base_sheet(&c, p(&c, &a)).unwrap().stats["Special"];
    pm(&mut c, &a).runtime.modifiers.insert("Special".into(), 2);
    let action = super::targeting_tests::attack(&a, vec![t.clone(), h.clone()], "Swift");
    let mut c = roll(&c, "player", &action, 6);
    while c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
        c = pass(&c);
    }
    let first = queued(&c).target_id;
    let last = if first == t { &h } else { &t };
    pm(&mut c, &first).selected_ability.clear();
    pm(&mut c, last).selected_ability = "Unaware".into();
    let mut c = dmg(&c);
    assert!(queued(&c).damage_context.is_some());
    pm(&mut c, &a).stat_overrides.insert("Special".into(), 9);
    assert_eq!(ordinary_pool(&c).attribute, base);
}
fn setup(name: &str, a: &str, t: &str) -> (Campaign, String, String) {
    let (mut c, u, v) = super::special_tests::setup(name);
    let b = c.battle.as_mut().unwrap();
    b.combatants[0].selected_ability = a.into();
    b.combatants[1].selected_ability = t.into();
    b.flow.as_mut().unwrap().tactics.enabled = true;
    b.flow.as_mut().unwrap().reviewed_rules = true;
    (c, u, v)
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
fn pm<'a>(c: &'a mut Campaign, id: &str) -> &'a mut Combatant {
    c.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == id)
        .unwrap()
}
fn m(c: &Campaign, id: &str, name: &str) -> serde_json::Value {
    field::move_data(c, p(c, id), &catalog().find("moves", "Name", name).unwrap())
}
fn queued(c: &Campaign) -> crate::combat::QueuedAction {
    c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].clone()
}
fn pending(c: &Campaign, a: &str, t: &str, name: &str) -> Campaign {
    pass(&hit(c, a, t, name))
}
fn dmg(c: &Campaign) -> Campaign {
    roll(c, "gm", &damage_command(c), 4)
}
#[test]
fn reviewed_mold_breaker_bypasses_type_and_damage_immunity_without_mutating_identity() {
    for ability in ["Mold Breaker", "Teravolt", "Turboblaze"] {
        let (mut c, a, t) = setup("Thunder Shock", ability, "Volt Absorb");
        let sid = p(&c, &t).sheet_id.clone();
        c.sheets.get_mut(&sid).unwrap().species_id = Some(
            catalog().find("species", "Name", "Sandshrew").unwrap()["AppId"]
                .as_str()
                .unwrap()
                .into(),
        );
        let sheets = serde_json::to_value(&c.sheets).unwrap();
        let c = pending(&c, &a, &t, "Thunder Shock");
        let dice = rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap();
        assert!(dice > 0, "{ability}");
        let c = dmg(&c);
        assert!(p(&c, &t).hp < 20);
        assert_eq!(p(&c, &t).selected_ability, "Volt Absorb");
        assert!(c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events
            .is_empty());
        assert_eq!(serde_json::to_value(&c.sheets).unwrap(), sheets);
    }
}
#[test]
fn reviewed_bypass_does_not_remove_damage_reductions_or_ally_immunity() {
    let (c, a, t) = setup("Ember", "Mold Breaker", "Thick Fat");
    let data = m(&c, &a, "Ember");
    assert_eq!(abilities::reduction(&c, p(&c, &t), &data, 0, &a), 1);
    let mut ally = p(&c, &t).clone();
    ally.side = p(&c, &a).side.clone();
    ally.selected_ability = "Volt Absorb".into();
    assert!(abilities::damage_immune_from(
        &c,
        p(&c, &a),
        &ally,
        &serde_json::json!({"Type":"Electric"})
    ));
}
#[test]
fn reviewed_historical_engine_keeps_bypass_guard_and_old_accuracy() {
    let (mut c, a, t) = setup("Tackle", "Mold Breaker", "Tangled Feet");
    c.engine = TRANSFER_ENGINE.into();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .reviewed_rules = false;
    assert!(abilities::guard(&c, p(&c, &a)).is_err());
    assert_eq!(reviewed::accuracy_penalty(&c, p(&c, &a), p(&c, &t), -1), -1);
    assert!(!reviewed::damage_bypass(&c, p(&c, &a), p(&c, &t)));
}
#[test]
fn reviewed_keen_eye_and_tangled_feet_adjust_successes_not_action_budget() {
    let (mut c, a, t) = setup("Tackle", "Keen Eye", "Tangled Feet");
    pm(&mut c, &t).runtime.statuses.push(crate::combat::Status {
        kind: "Confusion".into(),
        detail: String::new(),
        rounds: None,
        progress: 0,
    });
    assert_eq!(reviewed::accuracy_penalty(&c, p(&c, &a), p(&c, &t), -3), 0);
    pm(&mut c, &a).selected_ability.clear();
    assert_eq!(reviewed::accuracy_penalty(&c, p(&c, &a), p(&c, &t), -3), -5);
    assert_eq!(p(&c, &a).actions, 0);
}
#[test]
fn reviewed_minds_eye_night_and_ghost_identity() {
    let (mut c, a, t) = setup("Tackle", "Mind's Eye", "");
    assert_ne!(m(&c, &a, "Tackle")["Attributes"]["NeverMiss"], true);
    assert!(extra::ignores_ghost(&c, p(&c, &a), "Normal", "Ghost"));
    let action = Action::Battlefield {
        command: field::FieldCommand::Night { night: true },
        reason: "Night gym".into(),
    };
    assert!(battle::transition(&c, "player", &action, vec![], &catalog()).is_err());
    c = battle::transition(&c, "gm", &action, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(m(&c, &a, "Tackle")["Attributes"]["NeverMiss"], true);
    let c = hit(&c, &a, &t, "Tackle");
    assert!(battle::transition(
        &c,
        "gm",
        &Action::React {
            combatant_id: t,
            kind: "evade".into(),
            move_name: "".into(),
            reason: "Evade".into()
        },
        vec![6; 7],
        &catalog()
    )
    .is_err());
}
#[test]
fn reviewed_victory_star_only_damaging_moves_and_range() {
    let (mut c, a, t) = setup("Tackle", "Victory Star", "");
    assert_eq!(m(&c, &a, "Tackle")["Attributes"]["NeverMiss"], true);
    assert_ne!(m(&c, &a, "Growl")["Attributes"]["NeverMiss"], true);
    pm(&mut c, &t).side = p(&c, &a).side.clone();
    assert_eq!(m(&c, &t, "Tackle")["Attributes"]["NeverMiss"], true);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ranges
        .insert(a.clone(), vec![]);
    assert_ne!(m(&c, &t, "Tackle")["Attributes"]["NeverMiss"], true);
    pm(&mut c, &a).selected_ability.clear();
    assert_ne!(m(&c, &a, "Tackle")["Attributes"]["NeverMiss"], true);
}
#[test]
fn reviewed_mycelium_support_gets_late_reaction_without_bypass() {
    let (c, a, t) = setup("Growl", "Mycelium Might", "");
    let data = m(&c, &a, "Growl");
    assert_eq!(data["Attributes"]["NeverMiss"], true);
    assert_eq!(data["Attributes"]["LateReaction"], "1");
    assert!(!reviewed::damage_bypass(&c, p(&c, &a), p(&c, &t)));
    assert_ne!(m(&c, &a, "Tackle")["Attributes"]["NeverMiss"], true);
}
#[test]
fn reviewed_supersweet_syrup_blocks_only_foe_evasion_in_range() {
    let (mut c, a, t) = setup("Tackle", "Supersweet Syrup", "");
    assert!(reviewed::syrup(&c, p(&c, &a), p(&c, &t)));
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ranges
        .insert(a.clone(), vec![]);
    assert!(!reviewed::syrup(&c, p(&c, &a), p(&c, &t)));
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ranges
        .clear();
    pm(&mut c, &t).side = p(&c, &a).side.clone();
    assert!(!reviewed::syrup(&c, p(&c, &a), p(&c, &t)));
}
#[test]
fn reviewed_motor_drive_immunity_and_once_boost_are_bypassed_by_mold_breaker() {
    for bypass in [false, true] {
        let (c, a, t) = setup(
            "Thunder Shock",
            if bypass { "Mold Breaker" } else { "" },
            "Motor Drive",
        );
        let c = pending(&c, &a, &t, "Thunder Shock");
        let q = queued(&c);
        let mut out = dmg(&c);
        assert_eq!(p(&out, &t).hp == 20, !bypass);
        assert_eq!(
            p(&out, &t)
                .runtime
                .ability
                .counters
                .get("boost:Dexterity")
                .copied()
                .unwrap_or(0),
            i32::from(!bypass)
        );
        reviewed::hit(&mut out, &q, &m(&c, &a, "Thunder Shock"), &catalog()).unwrap();
        assert_eq!(
            p(&out, &t)
                .runtime
                .ability
                .counters
                .get("boost:Dexterity")
                .copied()
                .unwrap_or(0),
            i32::from(!bypass)
        );
    }
}
#[test]
fn reviewed_justified_caps_at_three_and_requires_dark_hit() {
    let (c, a, t) = setup("Bite", "", "Justified");
    let mut out = pending(&c, &a, &t, "Bite");
    let q = queued(&out);
    for _ in 0..5 {
        reviewed::hit(&mut out, &q, &m(&c, &a, "Bite"), &catalog()).unwrap();
    }
    assert_eq!(p(&out, &t).runtime.ability.counters["boost:Strength"], 3);
}
#[test]
fn reviewed_cotton_down_uses_ability_channel_and_skips_ranged_contact() {
    let (c, a, t) = setup("Tackle", "", "Cotton Down");
    let mut out = pending(&c, &a, &t, "Tackle");
    let q = queued(&out);
    let mut data = m(&c, &a, "Tackle");
    data["Attributes"]["Ranged"] = serde_json::json!(true);
    reviewed::hit(&mut out, &q, &data, &catalog()).unwrap();
    assert!(!p(&out, &a)
        .runtime
        .ability
        .counters
        .contains_key("debuff:Dexterity"));
    reviewed::hit(&mut out, &q, &m(&c, &a, "Tackle"), &catalog()).unwrap();
    assert_eq!(p(&out, &a).runtime.ability.counters["debuff:Dexterity"], -1);
    assert!(p(&out, &a).runtime.modifiers.is_empty());
    assert_eq!(
        rules::effective_sheet(&out, p(&out, &a)).unwrap().stats["Dexterity"],
        rules::effective_sheet(&c, p(&c, &a)).unwrap().stats["Dexterity"] - 1
    );
}
#[test]
fn reviewed_seed_sower_is_four_rounds_does_not_refresh_and_overlap_requires_ruling() {
    let (mut c, _a, t) = setup("Tackle", "", "Seed Sower");
    let target = p(&c, &t).clone();
    field::seed_sower(&mut c, &target).unwrap();
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .battlefield
            .rules[0]
            .rounds,
        Some(4)
    );
    field::expire(&mut c).unwrap();
    field::seed_sower(&mut c, &target).unwrap();
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .battlefield
            .rules[0]
            .rounds,
        Some(3)
    );
    let f = &mut c
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield;
    f.rules[0].kind = "Electric Terrain".into();
    field::seed_sower(&mut c, &target).unwrap();
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .battlefield
            .events[0]
            .kind,
        "terrain-ruling"
    );
}
#[test]
fn reviewed_screen_cleaner_removes_both_sides_force_fields_without_erasing_weather() {
    let (mut c, _, _) = setup("Tackle", "Screen Cleaner", "");
    let f = &mut c
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .battlefield;
    f.natural = Some(field::Weather::Rain);
    for (i, name) in ["Reflect", "Light Screen", "Aurora Veil", "Trick Room"]
        .iter()
        .enumerate()
    {
        f.rules.push(field::FieldRule {
            id: i.to_string(),
            kind: (*name).into(),
            source: "test".into(),
            scope: "all".into(),
            rounds: Some(4),
            parameter: 1,
            started_round: 1,
        });
    }
    let before = c.clone();
    extra::lifecycle(&before, &mut c, &catalog()).unwrap();
    let f = &c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .battlefield;
    assert_eq!(f.rules.len(), 1);
    assert_eq!(f.rules[0].kind, "Trick Room");
    assert_eq!(f.natural, Some(field::Weather::Rain));
}
#[test]
fn reviewed_toxic_debris_places_one_persistent_hazard_on_attackers_side() {
    let (c, a, t) = setup("Tackle", "", "Toxic Debris");
    let mut out = pending(&c, &a, &t, "Tackle");
    let q = queued(&out);
    for _ in 0..2 {
        reviewed::hit(&mut out, &q, &m(&c, &a, "Tackle"), &catalog()).unwrap();
    }
    let f = &out
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .battlefield;
    assert_eq!(f.rules.len(), 1);
    assert_eq!(f.rules[0].kind, "Toxic Spikes");
    assert_eq!(f.rules[0].scope, format!("side:{}", p(&c, &a).side));
    assert_eq!(f.rules[0].rounds, None);
}
#[test]
fn reviewed_extreme_bypasses_status_immunity_and_decoys_without_negating_target() {
    let (mut c, a, t) = setup("Toxic", "Teravolt", "Immunity");
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some(
        catalog().find("species", "Name", "Magnemite").unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
    abilities::inflict(&mut c, &a, &t, "Poison", "", None, &catalog()).unwrap();
    assert!(p(&c, &t)
        .runtime
        .statuses
        .iter()
        .any(|s| s.kind == "Poison"));
    assert_eq!(p(&c, &t).selected_ability, "Immunity");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .decoys
        .enabled = true;
    crate::combat::decoys::cast(&mut c, &t, "Substitute", &catalog()).unwrap();
    assert!(!crate::combat::decoys::protected_subject(
        &c,
        &a,
        &t,
        &m(&c, &a, "Toxic")
    ));
}
#[test]
fn reviewed_bypass_preview_explains_scope() {
    let (c, a, t) = setup("Tackle", "Mold Breaker", "");
    let c = pending(&c, &a, &t, "Tackle");
    let preview = crate::combat::preview(&c, &damage_command(&c), &catalog()).unwrap();
    assert!(preview.warnings.iter().any(|v| v.contains("Mold Breaker")));
}

#[test]
fn reviewed_early_bird_halves_sleep_duration_and_wakes_at_two_except_rest() {
    for rest in [false, true] {
        let (mut c, a, _) = setup("Tackle", "Early Bird", "");
        abilities::inflict(
            &mut c,
            &a,
            &a,
            "Sleep",
            if rest { "Rest" } else { "" },
            Some(5),
            &catalog(),
        )
        .unwrap();
        assert_eq!(
            p(&c, &a).runtime.statuses[0].rounds,
            Some(if rest { 5 } else { 3 })
        );
        let action = Action::StatusCheck {
            combatant_id: a.clone(),
            kind: "sleep".into(),
            reason: "Wake".into(),
        };
        let n = rules::dice_count(&c, &action, &catalog()).unwrap();
        let mut dice = vec![1; n as usize];
        dice[0] = 4;
        dice[1] = 4;
        let next = battle::transition(&c, "player", &action, dice, &catalog())
            .unwrap()
            .0;
        assert_eq!(
            p(&next, &a)
                .runtime
                .statuses
                .iter()
                .any(|s| s.kind == "Sleep"),
            rest
        );
        assert_eq!(p(&next, &a).actions, 1);
    }
}
#[test]
fn reviewed_toxic_spikes_entry_roll_is_required_and_does_not_repeat_on_ability_refresh() {
    for face in [1, 6] {
        let (mut c, a, t) = setup("Tackle", "", "Toxic Debris");
        let target = p(&c, &t).clone();
        let actor = p(&c, &a).clone();
        field::toxic_debris(&mut c, &target, &actor);
        let mut before = c.clone();
        pm(&mut before, &a).deployed = false;
        field::lifecycle(&before, &mut c, &catalog()).unwrap();
        let e = c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .battlefield
            .events[0]
            .clone();
        assert_eq!(e.kind, "toxic-spikes");
        let cmd = field::FieldCommand::Resolve {
            event_id: e.id,
            accept: true,
        };
        assert_eq!(field::dice(&c, &cmd).unwrap(), 1);
        assert!(field::command(&mut c, "player", &cmd, &[face], "Entry", &catalog()).is_err());
        field::command(&mut c, "gm", &cmd, &[face], "Entry", &catalog()).unwrap();
        assert_eq!(
            p(&c, &a).runtime.statuses[0].kind,
            if face == 6 {
                "Badly Poisoned"
            } else {
                "Poison"
            }
        );
        let before = c.clone();
        field::lifecycle(&before, &mut c, &catalog()).unwrap();
        assert!(c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .battlefield
            .events
            .is_empty());
    }
}
#[test]
fn reviewed_unique_damage_stalemate_is_explicit_and_common_bypass_cannot_erase_wonder_guard() {
    let (c, a, t) = setup("Tackle", "Mold Breaker", "Wonder Guard");
    assert!(!reviewed::damage_bypass(&c, p(&c, &a), p(&c, &t)));
    let (mut c, a, t) = setup("Tackle", "Teravolt", "Wonder Guard");
    assert!(reviewed::unique_damage_guard(&c, p(&c, &a), p(&c, &t), 0).is_err());
    assert!(reviewed::unique_damage_guard(&c, p(&c, &a), p(&c, &t), 1).is_ok());
    pm(&mut c, &a).selected_ability.clear();
    assert!(reviewed::unique_damage_guard(&c, p(&c, &a), p(&c, &t), 0).is_ok());
}

#[test]
fn reviewed_gas_negation_and_scene_copy_control_bypass_identity() {
    let (mut c, a, t) = setup("Tackle", "Run Away", "Neutralizing Gas");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_changes
        .enabled = true;
    let old = c.clone();
    crate::combat::ability_changes::lifecycle(&old, &mut c, &catalog()).unwrap();
    assert!(p(&c, &a).selected_ability.is_empty());
    let state = &mut c
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_changes;
    state.assignments.insert(a.clone(), "Mold Breaker".into());
    let old = c.clone();
    crate::combat::ability_changes::lifecycle(&old, &mut c, &catalog()).unwrap();
    assert!(p(&c, &a).selected_ability.is_empty());
    pm(&mut c, &t).deployed = false;
    let old = c.clone();
    crate::combat::ability_changes::lifecycle(&old, &mut c, &catalog()).unwrap();
    assert_eq!(p(&c, &a).selected_ability, "Mold Breaker");
    let mut foe = p(&c, &t).clone();
    foe.side = "Opponents".into();
    assert!(reviewed::damage_bypass(&c, p(&c, &a), &foe));
}
#[test]
fn reviewed_shared_damage_retains_frozen_bypass_source_after_live_identity_changes() {
    let (mut c, a, t, h) = super::targeting_tests::multi("Swift");
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.reviewed_rules = true;
    f.extra.enabled = true;
    f.special.enabled = true;
    f.ability_rules = true;
    f.battlefield.enabled = true;
    f.ability_changes.enabled = true;
    pm(&mut c, &a).selected_ability = "Mold Breaker".into();
    for id in [&t, &h] {
        let sid = p(&c, id).sheet_id.clone();
        c.sheets.get_mut(&sid).unwrap().species_id = Some("0092:gastly".into());
        pm(&mut c, id).selected_ability = "Levitate".into();
    }
    let action = super::targeting_tests::attack(&a, vec![t, h], "Swift");
    let mut c = roll(&c, "player", &action, 6);
    while c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
        c = pass(&c);
    }
    let mut c = dmg(&c);
    assert!(queued(&c).damage_context.is_some());
    pm(&mut c, &a).selected_ability.clear();
    let q = queued(&c);
    assert_eq!(
        reviewed::source(&c, &q).unwrap().selected_ability,
        "Mold Breaker"
    );
    assert!(rules::dice_count(&c, &damage_command(&c), &catalog()).unwrap() > 0);
}

#[test]
fn reviewed_mold_breaker_preserves_reactive_stamina_boost() {
    let (c, a, t) = setup("Tackle", "Mold Breaker", "Stamina");
    let c = dmg(&pending(&c, &a, &t, "Tackle"));
    assert_eq!(p(&c, &t).runtime.ability.counters["boost:Defense"], 1);
    assert_eq!(
        p(&c, &t).runtime.ability.counters["boost:SpecialDefense"],
        1
    );
}
#[test]
fn reviewed_wonder_guard_does_not_prevent_independent_type_bypass() {
    let (mut c, a, t) = setup("Thunder Shock", "Mold Breaker", "Wonder Guard");
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().species_id = Some("0195:quagsire".into());
    let (effect, immune) =
        rules::type_matchup(&c, p(&c, &a), p(&c, &t), "Electric", &catalog()).unwrap();
    assert_eq!(effect, 1);
    assert!(!immune);
    assert!(!extra::source_immune(&c, p(&c, &a), p(&c, &t), effect));
}

#[test]
fn reviewed_tidy_up_and_defog_remove_toxic_debris_hazards() {
    for name in ["Tidy Up", "Defog"] {
        let (mut c, a, t) = setup(name, "", "Toxic Debris");
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .decoys
            .enabled = true;
        let target = p(&c, &t).clone();
        let actor = p(&c, &a).clone();
        field::toxic_debris(&mut c, &target, &actor);
        let action = super::targeting_tests::attack(&a, vec![], name);
        let c = roll(&c, "player", &action, 6);
        let q = queued(&c);
        let c = battle::transition(
            &c,
            "player",
            &Action::ResolveEffect {
                sequence: q.sequence,
                reason: "Clear entry hazards".into(),
            },
            vec![],
            &catalog(),
        )
        .unwrap()
        .0;
        assert!(
            !c.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .battlefield
                .rules
                .iter()
                .any(|r| r.kind == "Toxic Spikes"),
            "{name}"
        );
    }
}

#[test]
fn reviewed_supersweet_syrup_respects_rank_target_caps() {
    let (mut c, a, t) = setup("Tackle", "Supersweet Syrup", "");
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().rank = "Starter".into();
    let mut extra = p(&c, &t).clone();
    extra.id = "000-first-foe".into();
    c.battle.as_mut().unwrap().combatants.push(extra.clone());
    assert!(reviewed::syrup(&c, p(&c, &a), &extra));
    assert!(!reviewed::syrup(&c, p(&c, &a), p(&c, &t)));
}
