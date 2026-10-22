use super::*;
use crate::combat::{abilities, extra, special, stat_channels};
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
fn setup(ability: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = super::special_tests::setup("Snatch");
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.stat_channels.enabled = true;
    f.reviewed_rules = true;
    f.tactics.enabled = true;
    pm(&mut c, &a).selected_ability = ability.into();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .interaction_rules = true;
    (c, a, t)
}
fn sync(c: &mut Campaign) {
    let before = c.clone();
    stat_channels::lifecycle(&before, c).unwrap();
}
fn add(c: &mut Campaign, id: &str) -> String {
    let mut q = p(c, id).clone();
    q.id = crate::model::id();
    q.field_slot = Some(1);
    let id = q.id.clone();
    c.battle.as_mut().unwrap().combatants.push(q);
    id
}
fn stat(c: &Campaign, id: &str, key: &str) -> i32 {
    rules::effective_sheet(c, p(c, id)).unwrap().stats[key]
}
fn decision(c: &Campaign, choice: &str) -> Action {
    Action::ResolveAbility {
        event_id: c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events[0]
            .id
            .clone(),
        choice: choice.into(),
        reason: "Choose the ally snapshot".into(),
    }
}
#[test]
fn stat_channels_costar_copies_positive_move_and_ability_channels_without_sheets_or_decreases() {
    let (mut c, a, t) = setup("Costar");
    let side = p(&c, &a).side.clone();
    pm(&mut c, &t).side = side;
    extra::change_stat(&mut c, &t, &t, "Strength", 2).unwrap();
    extra::change_stat(&mut c, &t, &t, "Dexterity", -1).unwrap();
    extra::boost(&mut c, &t, "Strength", 1).unwrap();
    extra::boost(&mut c, &t, "Defense", 2).unwrap();
    let sheets = serde_json::to_value(&c.sheets).unwrap();
    sync(&mut c);
    assert_eq!(stat(&c, &a, "Strength"), 7);
    assert_eq!(stat(&c, &a, "Dexterity"), 4);
    assert_eq!(extra::defense(&c, p(&c, &a), true), 2);
    assert_eq!(pm(&mut c, &a).selected_ability, "Costar");
    assert_eq!(serde_json::to_value(&c.sheets).unwrap(), sheets);
    let before = c.clone();
    extra::boost(&mut c, &t, "Strength", 3).unwrap();
    stat_channels::lifecycle(&before, &mut c).unwrap();
    assert_eq!(
        stat(&c, &a, "Strength"),
        7,
        "entry snapshot is not a live link"
    );
}
#[test]
fn stat_channels_costar_choice_is_frozen_authorized_and_consumed_once() {
    let (mut c, a, t) = setup("Costar");
    let side = p(&c, &a).side.clone();
    pm(&mut c, &t).side = side;
    let other = add(&mut c, &t);
    extra::boost(&mut c, &t, "Strength", 2).unwrap();
    extra::boost(&mut c, &other, "Dexterity", 1).unwrap();
    sync(&mut c);
    let action = decision(&c, &t);
    let before = c.clone();
    assert!(battle::transition(&c, "intruder", &action, vec![], &catalog()).is_err());
    extra::boost(&mut c, &t, "Strength", 3).unwrap();
    let mut next = c.clone();
    abilities::resolve(&c, &mut next, "gm", &action, &[], &catalog()).unwrap();
    assert_eq!(stat(&next, &a, "Strength"), 6);
    assert!(abilities::resolve(&next, &mut next.clone(), "gm", &action, &[], &catalog()).is_err());
    let json = serde_json::to_string(&before).unwrap();
    let resumed: Campaign = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_value(
            &resumed
                .battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .stat_channels
        )
        .unwrap(),
        serde_json::to_value(
            &before
                .battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .stat_channels
        )
        .unwrap()
    );
}
#[test]
fn stat_channels_opportunist_keeps_highest_separate_channels_and_ignores_unchanged_foes() {
    let (mut c, a, t) = setup("Opportunist");
    sync(&mut c);
    let before = c.clone();
    extra::change_stat(&mut c, &t, &t, "Special", 2).unwrap();
    extra::boost(&mut c, &t, "Special", 1).unwrap();
    stat_channels::lifecycle(&before, &mut c).unwrap();
    assert_eq!(stat(&c, &a, "Special"), 7);
    let entries = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .stat_channels
        .grants
        .len();
    sync(&mut c);
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .stat_channels
            .grants
            .len(),
        entries
    );
    let before = c.clone();
    extra::boost(&mut c, &t, "Special", 3).unwrap();
    stat_channels::lifecycle(&before, &mut c).unwrap();
    assert_eq!(stat(&c, &a, "Special"), 9);
    let before = c.clone();
    extra::change_stat(&mut c, &t, &t, "Special", 1).unwrap();
    stat_channels::lifecycle(&before, &mut c).unwrap();
    assert_eq!(stat(&c, &a, "Special"), 9);
}
#[test]
fn stat_channels_reciprocal_opportunists_do_not_loop_or_add_channels_twice() {
    let (mut c, a, t) = setup("Opportunist");
    pm(&mut c, &t).selected_ability = "Opportunist".into();
    sync(&mut c);
    let before = c.clone();
    extra::change_stat(&mut c, &t, &t, "Strength", 2).unwrap();
    stat_channels::lifecycle(&before, &mut c).unwrap();
    for _ in 0..4 {
        sync(&mut c);
    }
    assert_eq!(stat(&c, &a, "Strength"), 6);
    assert_eq!(stat(&c, &t, "Strength"), 6);
}
#[test]
fn stat_channels_snatch_removes_both_ability_signs_preserves_provenance_and_rearms_conditions() {
    let (mut c, a, t) = setup("");
    pm(&mut c, &t).selected_ability = "Anger Shell".into();
    pm(&mut c, &t).hp = 5;
    extra::boost(&mut c, &t, "SpecialDefense", -2).unwrap();
    stat_channels::transfer(&mut c, &t, &a, "Snatch").unwrap();
    assert_eq!(stat_channels::channel(&c, p(&c, &t), "Strength"), (0, 0));
    assert_eq!(stat_channels::channel(&c, p(&c, &a), "Strength"), (1, 0));
    assert_eq!(extra::defense(&c, p(&c, &a), true), -1);
    assert_eq!(extra::defense(&c, p(&c, &a), false), -2);
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .stat_channels
        .grants
        .iter()
        .all(|g| g.source == t && g.cause == "Snatch"));
    sync(&mut c);
    assert_eq!(stat_channels::channel(&c, p(&c, &t), "Strength"), (0, 0));
    pm(&mut c, &t).hp = 20;
    sync(&mut c);
    pm(&mut c, &t).hp = 5;
    sync(&mut c);
    assert_eq!(stat_channels::channel(&c, p(&c, &t), "Strength"), (1, 0));
    pm(&mut c, &t).selected_ability = "Speed Boost".into();
    pm(&mut c, &t)
        .runtime
        .ability
        .counters
        .insert("Speed Boost".into(), 2);
    stat_channels::transfer(&mut c, &t, &a, "Snatch").unwrap();
    sync(&mut c);
    assert_eq!(stat_channels::channel(&c, p(&c, &t), "Dexterity"), (0, 0));
    pm(&mut c, &t)
        .runtime
        .ability
        .counters
        .insert("Speed Boost".into(), 1);
    sync(&mut c);
    assert_eq!(
        stat_channels::channel(&c, p(&c, &t), "Dexterity"),
        (1, 0),
        "a later trigger grants only the new point, not the stolen count"
    );
}
#[test]
fn stat_channels_baton_transfers_only_positive_attributes_and_lock_applies_to_ability_gains() {
    let (mut c, a, t) = setup("");
    extra::boost(&mut c, &a, "Strength", 2).unwrap();
    extra::boost(&mut c, &a, "Defense", 2).unwrap();
    extra::boost(&mut c, &a, "Dexterity", -1).unwrap();
    stat_channels::transfer(&mut c, &a, &t, "Baton Pass").unwrap();
    assert_eq!(stat_channels::channel(&c, p(&c, &t), "Strength"), (2, 0));
    assert_eq!(extra::defense(&c, p(&c, &t), true), 0);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .tactics
        .baton
        .insert(a.clone(), t.clone());
    assert_eq!(stat_channels::channel(&c, p(&c, &a), "Strength"), (0, 0));
    pm(&mut c, &t).hp = 0;
    assert_eq!(stat_channels::channel(&c, p(&c, &a), "Strength"), (2, 0));
}
#[test]
fn stat_channels_withdrawal_clears_snapshots_and_historical_engine_is_unchanged() {
    let (mut c, a, t) = setup("");
    extra::boost(&mut c, &t, "Strength", 2).unwrap();
    stat_channels::transfer(&mut c, &t, &a, "Snatch").unwrap();
    let before = c.clone();
    pm(&mut c, &a).deployed = false;
    stat_channels::lifecycle(&before, &mut c).unwrap();
    pm(&mut c, &a).deployed = true;
    assert_eq!(stat(&c, &a, "Strength"), 4);
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .stat_channels
        .enabled = false;
    c.engine = REVIEWED_ENGINE.into();
    extra::boost(&mut c, &t, "Strength", 1).unwrap();
    stat_channels::transfer(&mut c, &t, &a, "Snatch").unwrap();
    assert_eq!(stat(&c, &a, "Strength"), 4);
    let action = Action::BattleStat {
        source_id: t.clone(),
        combatant_id: t,
        stat: "Special".into(),
        change: 1,
        reason: "Historical host dispatch".into(),
    };
    assert!(battle::transition(&c, "gm", &action, vec![], &catalog()).is_ok());
}
#[test]
fn stat_channels_copied_move_values_are_not_transformed_twice_and_clear_with_moves() {
    let (mut c, a, t) = setup("Costar");
    let side = p(&c, &a).side.clone();
    pm(&mut c, &t).side = side;
    pm(&mut c, &t).selected_ability = "Simple".into();
    extra::change_stat(&mut c, &t, &t, "Strength", 1).unwrap();
    sync(&mut c);
    assert_eq!(stat(&c, &a, "Strength"), 6);
    pm(&mut c, &a).selected_ability = "Simple".into();
    assert_eq!(stat(&c, &a, "Strength"), 6);
    special::clear_modifiers(&mut c, &a);
    assert_eq!(stat(&c, &a, "Strength"), 4);
}

#[test]
fn stat_channels_host_action_and_rollback_restore_copies_and_history() {
    let (mut c, a, t) = setup("Opportunist");
    c.revision = 10;
    sync(&mut c);
    let before = serde_json::to_value(c.battle.as_ref().unwrap()).unwrap();
    let action = Action::BattleStat {
        source_id: t.clone(),
        combatant_id: t,
        stat: "Strength".into(),
        change: 2,
        reason: "A foe increases Strength".into(),
    };
    let count = rules::dice_count(&c, &action, &catalog()).unwrap();
    assert_eq!(count, 0);
    let c = battle::transition(&c, "gm", &action, vec![], &catalog())
        .unwrap()
        .0;
    assert_eq!(stat(&c, &a, "Strength"), 6);
    let sequence = c.battle_undo.last().unwrap().event_sequence;
    let c = battle::transition(
        &c,
        "gm",
        &Action::RollbackBattle {
            event_sequence: sequence,
            reason: "Undo copied increase".into(),
        },
        vec![],
        &catalog(),
    )
    .unwrap()
    .0;
    assert_eq!(
        serde_json::to_value(c.battle.as_ref().unwrap()).unwrap(),
        before
    );
}
#[test]
fn stat_channels_permanent_sheet_bonus_is_not_stolen_or_copied() {
    let (mut c, a, t) = setup("Costar");
    let side = p(&c, &a).side.clone();
    pm(&mut c, &t).side = side;
    pm(&mut c, &t).selected_ability = "Huge Power".into();
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().ability = "Huge Power".into();
    sync(&mut c);
    assert_eq!(stat(&c, &a, "Strength"), 4);
    stat_channels::transfer(&mut c, &t, &a, "Snatch").unwrap();
    assert_eq!(stat(&c, &a, "Strength"), 4);
    assert_eq!(stat(&c, &t, "Strength"), 4);
}

#[tokio::test]
async fn stat_channels_room_costar_owner_retry_reconnect_persistence_and_gm_rollback() {
    let hd = tempfile::tempdir().unwrap();
    let gd = tempfile::tempdir().unwrap();
    let od = tempfile::tempdir().unwrap();
    let mut hs = Store::open(hd.path()).unwrap();
    let gs = Arc::new(Mutex::new(Store::open(gd.path()).unwrap()));
    let os = Arc::new(Mutex::new(Store::open(od.path()).unwrap()));
    let gm_id = gs.lock().unwrap().player_id.clone();
    let owner_id = os.lock().unwrap().player_id.clone();
    let (mut c, a, t) = setup("Costar");
    let side = p(&c, &a).side.clone();
    pm(&mut c, &t).side = side;
    let other = add(&mut c, &t);
    extra::boost(&mut c, &t, "Strength", 2).unwrap();
    extra::boost(&mut c, &other, "Dexterity", 1).unwrap();
    sync(&mut c);
    c.host_id = hs.player_id.clone();
    c.gm_id = gm_id.clone();
    pm(&mut c, &a).owner_id = owner_id.clone();
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().owner_id = owner_id.clone();
    hs.put_campaign(&c).unwrap();
    let shared = Arc::new(Mutex::new(hs));
    let cat = Arc::new(catalog());
    let notify: room::Notify = Arc::new(|_, _| {});
    let mut host = room::start(shared.clone(), cat.clone(), c.id.clone(), 0, notify.clone())
        .await
        .unwrap();
    let join = JoinRequest {
        address: host.info.address.replace("<host-ip>", "127.0.0.1"),
        token: host.info.token.clone(),
    };
    let (gm, _) = room::join(gs, cat.clone(), join.clone(), notify.clone())
        .await
        .unwrap();
    let (owner, entry) = room::join(os.clone(), cat.clone(), join.clone(), notify.clone())
        .await
        .unwrap();
    assert_eq!(
        entry
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .stat_channels
            .choices
            .len(),
        1
    );
    let command = Command {
        id: id(),
        expected_revision: c.revision,
        action: decision(&c, &t),
    };
    assert!(host.host.apply(&c.host_id, &command).is_err());
    let (tx, rx) = tokio::sync::oneshot::channel();
    owner.sender.send((command.clone(), tx)).await.unwrap();
    let next = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(stat(&next, &a, "Strength"), 6);
    assert_eq!(
        host.host.apply(&owner_id, &command).unwrap().revision,
        next.revision
    );
    let mut stale = command.clone();
    stale.id = id();
    assert!(host.host.apply(&owner_id, &stale).is_err());
    owner.task.abort();
    let (owner, snapshot) = room::join(os, cat, join, notify).await.unwrap();
    assert_eq!(
        serde_json::to_value(
            &snapshot
                .battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .stat_channels
        )
        .unwrap(),
        serde_json::to_value(
            &next
                .battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .stat_channels
        )
        .unwrap()
    );
    assert_eq!(
        serde_json::to_value(&shared.lock().unwrap().campaigns().unwrap()[0].battle).unwrap(),
        serde_json::to_value(&next.battle).unwrap()
    );
    let undo = Command {
        id: id(),
        expected_revision: next.revision,
        action: Action::RollbackBattle {
            event_sequence: next.battle_undo.last().unwrap().event_sequence,
            reason: "Restore Costar entry choice".into(),
        },
    };
    let (tx, rx) = tokio::sync::oneshot::channel();
    gm.sender.send((undo, tx)).await.unwrap();
    let restored = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(stat(&restored, &a, "Strength"), 4);
    assert_eq!(
        restored
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .stat_channels
            .choices
            .len(),
        1
    );
    assert_eq!(
        host.host.apply(&owner_id, &command).unwrap().revision,
        restored.revision,
        "receipt retry after rollback must not reapply the copy"
    );
    owner.task.abort();
    gm.task.abort();
    let _ = host.stop.take().unwrap().send(());
}

#[test]
fn stat_channels_new_ability_activation_does_not_inherit_a_stolen_channel_mask() {
    let (mut c, a, t) = setup("");
    pm(&mut c, &t).selected_ability = "Guts".into();
    pm(&mut c, &t).runtime.statuses.push(crate::combat::Status {
        kind: "Poison".into(),
        detail: "".into(),
        rounds: None,
        progress: 0,
    });
    stat_channels::transfer(&mut c, &t, &a, "Snatch").unwrap();
    sync(&mut c);
    assert_eq!(stat_channels::channel(&c, p(&c, &t), "Strength"), (0, 0));
    pm(&mut c, &t).selected_ability = "Toxic Boost".into();
    sync(&mut c);
    assert_eq!(stat_channels::channel(&c, p(&c, &t), "Strength"), (2, 0));
}

#[test]
fn stat_channels_end_scene_clears_live_copies_and_old_snapshots_default_disabled() {
    let (mut c, a, t) = setup("");
    extra::boost(&mut c, &t, "Strength", 2).unwrap();
    stat_channels::transfer(&mut c, &t, &a, "Snatch").unwrap();
    let c = battle::transition(&c, "gm", &Action::EndBattle, vec![], &catalog())
        .unwrap()
        .0;
    let state = &c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .stat_channels;
    assert!(state.grants.is_empty());
    assert!(state.removed.is_empty());
    assert!(!state.history.is_empty());
    let (c, _, _) = setup("");
    let mut value = serde_json::to_value(c).unwrap();
    value["battle"]["flow"]
        .as_object_mut()
        .unwrap()
        .remove("stat_channels");
    let old: Campaign = serde_json::from_value(value).unwrap();
    assert!(!stat_channels::enabled(&old));
}

#[test]
fn stat_channels_copied_moves_keep_stronger_legacy_modifiers_and_separate_reductions() {
    let (mut c, a, t) = setup("Costar");
    let side = p(&c, &a).side.clone();
    pm(&mut c, &t).side = side;
    pm(&mut c, &a)
        .runtime
        .modifiers
        .insert("Strength".into(), 3);
    extra::change_stat(&mut c, &t, &t, "Strength", 1).unwrap();
    sync(&mut c);
    assert_eq!(stat(&c, &a, "Strength"), 7);
    let (mut c, a, t) = setup("Costar");
    let side = p(&c, &a).side.clone();
    pm(&mut c, &t).side = side;
    pm(&mut c, &a)
        .runtime
        .modifiers
        .insert("Strength".into(), -1);
    extra::change_stat(&mut c, &t, &t, "Strength", 2).unwrap();
    sync(&mut c);
    assert_eq!(stat(&c, &a, "Strength"), 5);
}
#[test]
fn interactions_costar_opportunist_copy_reviewed_traits_without_stacking_or_permanent_edits() {
    use crate::combat::interactions;
    for ab in ["Costar", "Opportunist"] {
        let (mut c, a, t) = setup(ab);
        if ab == "Costar" {
            pm(&mut c, &t).side = p(&c, &a).side.clone();
        }
        let accuracy_before = rules::accuracy(&c, &a, "Tackle", 0, &catalog()).unwrap().0;
        let before = c.clone();
        for key in ["AccuracyPool", "HighCritical"] {
            special::record_modifier(&mut c, &t, &t, key, 1).unwrap();
            pm(&mut c, &t).runtime.modifiers.insert(key.into(), 1);
        }
        stat_channels::transfer(&mut c, &t, &a, "Baton Pass").unwrap();
        assert_eq!(
            interactions::trait_value(&c, p(&c, &a), "AccuracyPool"),
            0,
            "Baton Pass transfers attributes, not traits"
        );
        assert_eq!(interactions::trait_value(&c, p(&c, &a), "HighCritical"), 0);
        let sheets = serde_json::to_value(&c.sheets).unwrap();
        stat_channels::lifecycle(&before, &mut c).unwrap();
        assert_eq!(interactions::trait_value(&c, p(&c, &a), "AccuracyPool"), 1);
        assert_eq!(interactions::trait_value(&c, p(&c, &a), "HighCritical"), 1);
        assert_eq!(
            rules::accuracy(&c, &a, "Tackle", 0, &catalog()).unwrap().0,
            accuracy_before + 1
        );
        let m = catalog().find("moves", "Name", "Tackle").unwrap();
        assert_eq!(extra::critical_margin(&c, p(&c, &a), &m), 2);
        let count = c
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .stat_channels
            .grants
            .len();
        let before = c.clone();
        stat_channels::lifecycle(&before, &mut c).unwrap();
        assert_eq!(
            c.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .stat_channels
                .grants
                .len(),
            count
        );
        assert_eq!(serde_json::to_value(&c.sheets).unwrap(), sheets);
        c.engine = STANDALONE_ENGINE.into();
        c.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .interaction_rules = false;
        assert_eq!(interactions::trait_value(&c, p(&c, &a), "HighCritical"), 0);
        assert_eq!(extra::critical_margin(&c, p(&c, &a), &m), 3);
    }
}
