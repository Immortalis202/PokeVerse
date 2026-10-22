use super::*;
use crate::combat::{sequencing, special};
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
fn f(c: &Campaign) -> &TurnFlow {
    c.battle.as_ref().unwrap().flow.as_ref().unwrap()
}
fn fm(c: &mut Campaign) -> &mut TurnFlow {
    c.battle.as_mut().unwrap().flow.as_mut().unwrap()
}
fn learn(c: &mut Campaign, id: &str, name: &str) {
    let sid = p(c, id).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().moves.push(name.into());
}
pub(super) fn loyal(c: &mut Campaign, id: &str, n: i32) {
    let sid = p(c, id).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().workflow = Some(CharacterBuild {
        version: 1,
        status: "accepted".into(),
        origin: "captured".into(),
        age: "".into(),
        allocations: BTreeMap::new(),
        socials: BTreeMap::new(),
        happiness: 2,
        loyalty: n,
        training_points: 0,
        legacy_xp: 0,
        will: 6,
        max_will: 6,
        profile: RulesProfile::default(),
        hp_attribute: "Vitality".into(),
        available_abilities: vec![],
        learned: BTreeMap::new(),
        forgotten: BTreeMap::new(),
        overranked: None,
        evolution_ready: None,
        ledger: vec![],
    });
}
fn setup(name: &str, ability: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = super::special_tests::setup(name);
    fm(&mut c).sequencing.enabled = true;
    fm(&mut c).tactics.enabled = true;
    fm(&mut c).reviewed_rules = true;
    fm(&mut c).stat_channels.enabled = true;
    pm(&mut c, &a).selected_ability = ability.into();
    loyal(&mut c, &a, 2);
    sequencing::lifecycle(&mut c, &catalog()).unwrap();
    (c, a, t)
}
fn add(c: &mut Campaign, id: &str, ability: &str) -> String {
    let mut q = p(c, id).clone();
    let mut sheet = c.sheets[&q.sheet_id].clone();
    sheet.id = crate::model::id();
    q.sheet_id = sheet.id.clone();
    q.id = crate::model::id();
    q.name = format!("Second {ability}");
    q.selected_ability = ability.into();
    q.field_slot = Some(1);
    let id = q.id.clone();
    c.sheets.insert(sheet.id.clone(), sheet);
    c.battle.as_mut().unwrap().combatants.push(q);
    fm(c).order.push(id.clone());
    id
}
fn force(c: &mut Campaign, id: &str) {
    fm(c).cursor = f(c).order.iter().position(|n| n == id).unwrap();
    fm(c).turn += 1;
    fm(c).window = false;
    fm(c).reacted.clear();
    sequencing::lifecycle(c, &catalog()).unwrap();
}
fn action(id: &str, target: &str, name: &str) -> Action {
    Action::Accuracy {
        combatant_id: id.into(),
        target_id: Some(target.into()),
        move_name: name.into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Reviewed sequencing action".into(),
    }
}
fn run(c: &Campaign, a: &Action, faces: Vec<u8>) -> Campaign {
    assert_eq!(
        faces.len(),
        rules::dice_count(c, a, &catalog()).unwrap() as usize
    );
    let mut next = battle::transition(c, "gm", a, faces, &catalog()).unwrap().0;
    next.revision += 1;
    next
}
fn roll(c: &Campaign, a: &Action) -> Campaign {
    run(
        c,
        a,
        vec![6; rules::dice_count(c, a, &catalog()).unwrap() as usize],
    )
}
fn choose(c: &Campaign, choice: &str) -> Action {
    Action::ResolveAbility {
        event_id: f(c).ability_events[0].id.clone(),
        choice: choice.into(),
        reason: "Reviewed sequencing choice".into(),
    }
}
fn effect(c: &Campaign) -> Campaign {
    roll(
        c,
        &Action::ResolveEffect {
            sequence: f(c).queue[0].sequence,
            reason: "Complete the move".into(),
        },
    )
}
fn damage(c: &Campaign) -> Action {
    super::targeting_tests::damage_command(c)
}
fn settle(mut c: Campaign) -> Campaign {
    for _ in 0..12 {
        if f(&c).queue.is_empty() || !f(&c).ability_events.is_empty() {
            return c;
        }
        if f(&c).window {
            c = super::targeting_tests::pass(&c);
        } else if !f(&c).queue[0].damage_done && f(&c).queue[0].defense_key != "Support" {
            c = roll(&c, &damage(&c));
        } else {
            c = effect(&c);
        }
    }
    panic!("action chain did not settle")
}
#[test]
fn sequencing_gorilla_entry_choice_is_owned_and_allows_five_identical_moves() {
    let (mut c, a, _) = setup("Swords Dance", "Gorilla Tactics");
    assert_eq!(f(&c).ability_events[0].kind, "sequence-gorilla");
    let choice = choose(&c, "Swords Dance");
    assert!(battle::transition(&c, "intruder", &choice, vec![], &catalog()).is_err());
    c = roll(&c, &choice);
    assert_eq!(
        rules::effective_sheet(&c, p(&c, &a)).unwrap().stats["Strength"],
        5
    );
    assert!(sequencing::move_guard(&c, &a, None).is_err());
    assert!(sequencing::move_guard(&c, &a, Some("Tackle")).is_err());
    for i in 1..=5 {
        force(&mut c, &a);
        c = settle(roll(&c, &action(&a, &a, "Swords Dance")));
        assert_eq!(p(&c, &a).actions, i);
    }
    force(&mut c, &a);
    assert!(rules::dice_count(&c, &action(&a, &a, "Swords Dance"), &catalog()).is_err());
}
#[test]
fn sequencing_gorilla_withdrawal_and_ability_changes_require_a_new_choice() {
    let (c, a, _) = setup("Tackle", "Gorilla Tactics");
    let mut c = roll(&c, &choose(&c, "Tackle"));
    pm(&mut c, &a).deployed = false;
    sequencing::lifecycle(&mut c, &catalog()).unwrap();
    assert!(!f(&c).sequencing.gorilla.contains_key(&a));
    pm(&mut c, &a).deployed = true;
    sequencing::lifecycle(&mut c, &catalog()).unwrap();
    assert_eq!(f(&c).ability_events[0].kind, "sequence-gorilla");
}
#[test]
fn sequencing_defeatist_roll_is_bound_to_one_action_and_frozen_for_damage() {
    let (mut c, a, t) = setup("Tackle", "Defeatist");
    pm(&mut c, &a).hp = 10;
    let sheets = serde_json::to_value(&c.sheets).unwrap();
    let cmd = action(&a, &t, "Tackle");
    let n = rules::dice_count(&c, &cmd, &catalog()).unwrap() as usize;
    let mut faces = vec![6; n];
    faces[..2].fill(1);
    let mut c = run(&c, &cmd, faces);
    assert!(f(&c).queue[0].sequencing_penalty);
    assert_eq!(p(&c, &a).actions, 1);
    assert!(f(&c).sequencing.penalties.is_empty());
    pm(&mut c, &a).hp = 20;
    assert_eq!(
        rules::effective_sheet(&c, p(&c, &a)).unwrap().stats["Strength"],
        2
    );
    c = settle(c);
    assert_eq!(
        rules::effective_sheet(&c, p(&c, &a)).unwrap().stats["Strength"],
        4
    );
    assert_eq!(serde_json::to_value(&c.sheets).unwrap(), sheets);
    learn(&mut c, &a, "Pound");
    force(&mut c, &a);
    c = roll(&c, &action(&a, &t, "Pound"));
    assert!(!f(&c).queue[0].sequencing_penalty);
}
#[test]
fn sequencing_defeatist_threshold_zero_loyalty_and_missing_loyalty_are_explicit() {
    let (mut c, a, t) = setup("Tackle", "Defeatist");
    let cmd = action(&a, &t, "Tackle");
    let plain = rules::dice_count(&c, &cmd, &catalog()).unwrap();
    pm(&mut c, &a).hp = 11;
    assert_eq!(rules::dice_count(&c, &cmd, &catalog()).unwrap(), plain);
    pm(&mut c, &a).hp = 10;
    assert_eq!(rules::dice_count(&c, &cmd, &catalog()).unwrap(), plain + 2);
    loyal(&mut c, &a, 0);
    let c = roll(&c, &cmd);
    assert!(f(&c).queue[0].sequencing_penalty);
    let (mut c, a, t) = setup("Tackle", "Defeatist");
    pm(&mut c, &a).hp = 10;
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().workflow = None;
    assert_eq!(
        rules::dice_count(&c, &action(&a, &t, "Tackle"), &catalog())
            .unwrap_err()
            .code,
        "manual_rule"
    );
}
#[test]
fn sequencing_defeatist_successive_actions_have_independent_loyalty_and_penalties() {
    let (mut c, a, t) = setup("Double Kick", "Defeatist");
    pm(&mut c, &a).hp = 10;
    let cmd = Action::SpecialMove {
        command: special::SpecialCommand::Successive {
            reaction: false,
            combatant_id: a.clone(),
            target_id: t,
            move_name: "Double Kick".into(),
            attempts: 2,
        },
        reason: "Two independently checked actions".into(),
    };
    let n = rules::dice_count(&c, &cmd, &catalog()).unwrap() as usize;
    let mut faces = vec![6; n];
    faces[..2].fill(1);
    let c = run(&c, &cmd, faces);
    assert_eq!(p(&c, &a).actions, 2);
    assert!(f(&c).queue[0].sequencing_penalty);
    let pending = &f(&c).special.series.as_ref().unwrap().pending;
    assert!(pending.iter().all(|q| !q.sequencing_penalty));
    assert_eq!(
        f(&c)
            .sequencing
            .history
            .iter()
            .filter(|s| s.starts_with("Defeatist:"))
            .count(),
        2
    );
}
#[test]
fn sequencing_truant_checks_every_second_personal_turn_and_forfeits_only_failed_turn() {
    let (mut c, a, t) = setup("Tackle", "Truant");
    let cmd = action(&a, &t, "Tackle");
    let first = rules::dice_count(&c, &cmd, &catalog()).unwrap();
    force(&mut c, &t);
    force(&mut c, &a);
    assert_eq!(f(&c).sequencing.visits[&a], 2);
    assert_eq!(rules::dice_count(&c, &cmd, &catalog()).unwrap(), first + 2);
    let mut faces = vec![6; (first + 2) as usize];
    faces[..2].fill(1);
    let mut c = run(&c, &cmd, faces);
    assert!(f(&c).queue.is_empty());
    assert_eq!(p(&c, &a).actions, 0);
    assert!(p(&c, &a).used_moves.is_empty());
    assert_eq!(f(&c).order[f(&c).cursor], t);
    force(&mut c, &a);
    assert_eq!(f(&c).sequencing.visits[&a], 3);
    assert_eq!(rules::dice_count(&c, &cmd, &catalog()).unwrap(), first);
    c = roll(&c, &cmd);
    assert_eq!(p(&c, &a).actions, 1);
}
#[test]
fn sequencing_parental_bond_rolls_two_pools_and_applies_only_one_hit() {
    let (c, a, t) = setup("Tackle", "Parental Bond");
    let c = super::targeting_tests::pass(&roll(&c, &action(&a, &t, "Tackle")));
    let cmd = damage(&c);
    let n = rules::dice_count(&c, &cmd, &catalog()).unwrap() as usize;
    assert_eq!(n % 2, 0);
    let mut plain = c.clone();
    pm(&mut plain, &a).selected_ability.clear();
    let ordinary = rules::dice_count(&plain, &cmd, &catalog()).unwrap() as usize;
    assert_eq!(n, ordinary * 2);
    let expected = run(&plain, &cmd, vec![6; ordinary]);
    let mut faces = vec![1; ordinary];
    faces.extend(vec![6; ordinary]);
    let c = run(&c, &cmd, faces);
    assert_eq!(p(&c, &t).hp, p(&expected, &t).hp);
    assert_eq!(p(&c, &a).actions, 1);
    assert!(f(&c)
        .sequencing
        .history
        .last()
        .unwrap()
        .contains("keep pool 2 once"));
}
#[test]
fn sequencing_parental_bond_reuses_both_lanes_for_shared_targets() {
    let (mut c, a, t) = setup("Swift", "Parental Bond");
    let original_sid = p(&c, &t).sheet_id.clone();
    c.sheets
        .get_mut(&original_sid)
        .unwrap()
        .stats
        .insert("Insight".into(), 1);
    let high = add(&mut c, &t, "");
    let sid = p(&c, &high).sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .stats
        .insert("Insight".into(), 7);
    let cmd = Action::AccuracyTargets {
        combatant_id: a.clone(),
        target_ids: vec![t.clone(), high],
        move_name: "Swift".into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "Shared roll with distinct defenses".into(),
    };
    let mut c = roll(&c, &cmd);
    while f(&c).window {
        c = super::targeting_tests::pass(&c);
    }
    let first = damage(&c);
    let n = rules::dice_count(&c, &first, &catalog()).unwrap() as usize;
    let mut faces = vec![1; n];
    faces[0] = 6;
    c = run(&c, &first, faces);
    if f(&c).queue[0].damage_done {
        c = effect(&c);
    }
    let second = damage(&c);
    let n = rules::dice_count(&c, &second, &catalog()).unwrap() as usize;
    assert!(n >= 4, "second pool needs extra prefix dice");
    let mut faces = vec![1; n / 2];
    faces.extend(vec![6; n / 2]);
    c = run(&c, &second, faces);
    assert!(f(&c)
        .sequencing
        .history
        .last()
        .unwrap()
        .contains("keep pool 2 once"));
}
#[test]
fn sequencing_dancer_free_self_copy_reuses_accuracy_without_spending_actions_or_learning() {
    let (mut c, a, _) = setup("Swords Dance", "");
    let dancer = add(&mut c, &a, "Dancer");
    pm(&mut c, &dancer).actions = 5;
    let sid = p(&c, &dancer).sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .moves
        .retain(|n| n != "Swords Dance");
    sequencing::lifecycle(&mut c, &catalog()).unwrap();
    let c = roll(&c, &action(&a, &a, "Swords Dance"));
    let successes = f(&c).queue[0].successes;
    let turn = f(&c).turn;
    let c = effect(&c);
    assert_eq!(f(&c).turn, turn);
    assert_eq!(f(&c).ability_events[0].kind, "sequence-dancer");
    let c = roll(&c, &choose(&c, "copy"));
    assert_eq!(f(&c).queue[0].actor_id, dancer);
    assert_eq!(f(&c).queue[0].successes, successes);
    assert_eq!(p(&c, &dancer).actions, 5);
    let c = effect(&c);
    assert!(f(&c).queue.is_empty());
    assert!(f(&c).ability_events.is_empty());
    assert_eq!(p(&c, &dancer).actions, 5);
    assert!(!c.sheets[&sid].moves.iter().any(|n| n == "Swords Dance"));
}
#[test]
fn sequencing_dancer_declines_without_stalling_and_can_accept_a_later_trigger() {
    let (mut c, a, _) = setup("Swords Dance", "Gorilla Tactics");
    c = roll(&c, &choose(&c, "Swords Dance"));
    let dancer = add(&mut c, &a, "Dancer");
    sequencing::lifecycle(&mut c, &catalog()).unwrap();
    let c = effect(&roll(&c, &action(&a, &a, "Swords Dance")));
    let turn = f(&c).turn;
    let mut c = roll(&c, &choose(&c, "decline"));
    assert!(f(&c).queue.is_empty());
    assert!(f(&c).turn > turn);
    assert!(!f(&c).sequencing.dancer_round.contains_key(&dancer));
    learn(&mut c, &a, "Swords Dance");
    force(&mut c, &a);
    let c = effect(&roll(&c, &action(&a, &a, "Swords Dance")));
    assert!(!f(&c).ability_events.is_empty());
}
#[test]
fn sequencing_dancer_random_rampage_requires_gm_and_does_not_start_a_rampage() {
    let (mut c, a, t) = setup("Petal Dance", "");
    let dancer = add(&mut c, &a, "Dancer");
    sequencing::lifecycle(&mut c, &catalog()).unwrap();
    let mut c = settle(roll(&c, &action(&a, &t, "Petal Dance")));
    assert_eq!(f(&c).ability_events[0].kind, "sequence-dancer-random");
    let decision = choose(&c, &t);
    assert!(battle::transition(&c, "player", &decision, vec![], &catalog()).is_err());
    c = roll(&c, &decision);
    assert_eq!(f(&c).queue[0].actor_id, dancer);
    assert!(!f(&c).special.rampages.contains_key(&dancer));
}
#[test]
fn sequencing_preview_history_serialization_and_rollback_preserve_original_checks() {
    let (mut c, a, t) = setup("Tackle", "Defeatist");
    c.revision = 10;
    pm(&mut c, &a).hp = 10;
    let before = serde_json::to_value(c.battle.as_ref().unwrap()).unwrap();
    let cmd = action(&a, &t, "Tackle");
    let preview = crate::combat::preview(&c, &cmd, &catalog()).unwrap();
    assert!(preview
        .warnings
        .iter()
        .any(|w| w.contains("Loyalty dice first")));
    assert_eq!(
        serde_json::to_value(c.battle.as_ref().unwrap()).unwrap(),
        before
    );
    let c = roll(&c, &cmd);
    let restored: Campaign = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
    assert_eq!(f(&c).sequencing.history, f(&restored).sequencing.history);
    let c = battle::transition(
        &c,
        "gm",
        &Action::RollbackBattle {
            event_sequence: c.battle_undo.last().unwrap().event_sequence,
            reason: "Undo the complete Loyalty/action command".into(),
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
fn sequencing_historical_engines_default_disabled_and_keep_single_damage_rolls() {
    let (mut c, a, t) = setup("Tackle", "Parental Bond");
    fm(&mut c).sequencing.enabled = false;
    c.engine = STAT_ENGINE.into();
    let c = super::targeting_tests::pass(&roll(&c, &action(&a, &t, "Tackle")));
    let n = rules::dice_count(&c, &damage(&c), &catalog()).unwrap();
    let mut blank = c.clone();
    pm(&mut blank, &a).selected_ability.clear();
    assert_eq!(
        n,
        rules::dice_count(&blank, &damage(&blank), &catalog()).unwrap()
    );
    let mut json = serde_json::to_value(c).unwrap();
    json["battle"]["flow"]
        .as_object_mut()
        .unwrap()
        .remove("sequencing");
    let restored: Campaign = serde_json::from_value(json).unwrap();
    assert!(!sequencing::enabled(&restored));
}

#[test]
fn sequencing_defeatist_reaction_and_charged_release_keep_their_own_action_checks() {
    let (mut c, a, t) = setup("Tackle", "");
    pm(&mut c, &t).selected_ability = "Defeatist".into();
    pm(&mut c, &t).hp = 10;
    loyal(&mut c, &t, 2);
    sequencing::lifecycle(&mut c, &catalog()).unwrap();
    let c = roll(&c, &action(&a, &t, "Tackle"));
    let cmd = Action::Tactics {
        command: crate::combat::tactics::TacticsCommand::Reaction {
            combatant_id: t.clone(),
            move_name: "Quick Attack".into(),
            target_ids: vec![a],
        },
        reason: "A checked reaction move".into(),
    };
    let n = rules::dice_count(&c, &cmd, &catalog()).unwrap() as usize;
    let mut faces = vec![6; n];
    faces[..2].fill(1);
    let c = run(&c, &cmd, faces);
    assert!(f(&c)
        .queue
        .iter()
        .any(|q| q.actor_id == t && q.sequencing_penalty));
    assert_eq!(p(&c, &t).actions, 1);
    let (mut c, a, t) = setup("Solar Beam", "Defeatist");
    pm(&mut c, &a).hp = 10;
    let cmd = action(&a, &t, "Solar Beam");
    let n = rules::dice_count(&c, &cmd, &catalog()).unwrap() as usize;
    let mut faces = vec![6; n];
    faces[..2].fill(1);
    let mut c = run(&c, &cmd, faces);
    assert!(f(&c).special.charges[&a].action.sequencing_penalty);
    pm(&mut c, &a).hp = 20;
    force(&mut c, &a);
    let cmd = Action::SpecialMove {
        command: special::SpecialCommand::Release {
            combatant_id: a.clone(),
        },
        reason: "A new full-health release action".into(),
    };
    let c = roll(&c, &cmd);
    assert!(!f(&c).queue[0].sequencing_penalty);
    assert_eq!(p(&c, &a).actions, 2);
}
#[test]
fn sequencing_dancer_missed_source_and_battlefield_copies_do_not_roll_accuracy() {
    let (mut c, a, _) = setup("Swords Dance", "");
    let dancer = add(&mut c, &a, "Dancer");
    sequencing::lifecycle(&mut c, &catalog()).unwrap();
    let cmd = action(&a, &a, "Swords Dance");
    let c = run(
        &c,
        &cmd,
        vec![1; rules::dice_count(&c, &cmd, &catalog()).unwrap() as usize],
    );
    assert_eq!(f(&c).ability_events[0].ability, "Dancer");
    let c = roll(&c, &choose(&c, "copy"));
    assert!(f(&c).queue.is_empty());
    assert_eq!(p(&c, &dancer).actions, 0);
    assert_eq!(f(&c).sequencing.dancer_round[&dancer], 1);
    let (mut c, a, _) = setup("Rain Dance", "");
    let dancer = add(&mut c, &a, "Dancer");
    sequencing::lifecycle(&mut c, &catalog()).unwrap();
    let cmd = Action::AccuracyTargets {
        combatant_id: a,
        move_name: "Rain Dance".into(),
        target_ids: vec![],
        pool_modifier: 0,
        success_modifier: 0,
        reason: "A battlefield dance".into(),
    };
    let c = settle(roll(&c, &cmd));
    let c = roll(&c, &choose(&c, "copy"));
    assert_eq!(f(&c).queue[0].actor_id, dancer);
    assert_eq!(f(&c).queue[0].field_scope.as_deref(), Some("all"));
}
#[test]
fn sequencing_parental_bond_set_damage_has_no_extra_roll_and_rollback_removes_both_lanes() {
    let (c, a, t) = setup("Sonic Boom", "Parental Bond");
    let c = super::targeting_tests::pass(&roll(&c, &action(&a, &t, "Sonic Boom")));
    let cmd = damage(&c);
    assert_eq!(rules::dice_count(&c, &cmd, &catalog()).unwrap(), 0);
    let before = p(&c, &t).hp;
    let c = roll(&c, &cmd);
    assert_eq!(p(&c, &t).hp, before - 1);
    let (mut c, a, t) = setup("Tackle", "Parental Bond");
    c.revision = 10;
    let c = super::targeting_tests::pass(&roll(&c, &action(&a, &t, "Tackle")));
    let before = serde_json::to_value(c.battle.as_ref().unwrap()).unwrap();
    let c = roll(&c, &damage(&c));
    let c = battle::transition(
        &c,
        "gm",
        &Action::RollbackBattle {
            event_sequence: c.battle_undo.last().unwrap().event_sequence,
            reason: "Undo the complete pair of damage pools".into(),
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

#[tokio::test]
async fn sequencing_room_distinct_owners_choose_copy_retry_reconnect_and_rollback() {
    async fn send(remote: &room::Remote, cmd: Command) -> Campaign {
        let (tx, rx) = tokio::sync::oneshot::channel();
        remote.sender.send((cmd, tx)).await.unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(5), rx)
            .await
            .unwrap()
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
    let (mut c, a, _) = setup("Swords Dance", "Gorilla Tactics");
    let dancer = add(&mut c, &a, "Dancer");
    c.host_id = ids[0].clone();
    c.gm_id = ids[1].clone();
    for (who, role) in [(&a, 2usize), (&dancer, 3usize)] {
        pm(&mut c, who).owner_id = ids[role].clone();
        let sid = p(&c, who).sheet_id.clone();
        c.sheets.get_mut(&sid).unwrap().owner_id = ids[role].clone();
    }
    sequencing::lifecycle(&mut c, &catalog()).unwrap();
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
    let (dancer_owner, _) =
        room::join(stores[3].clone(), cat.clone(), join.clone(), notify.clone())
            .await
            .unwrap();
    let pick = Command {
        id: id(),
        expected_revision: c.revision,
        action: choose(&c, "Swords Dance"),
    };
    assert!(host.host.apply(&ids[0], &pick).is_err());
    c = send(&owner, pick.clone()).await;
    assert_eq!(f(&c).sequencing.gorilla[&a], "Swords Dance");
    assert_eq!(
        host.host.apply(&ids[2], &pick).unwrap().revision,
        c.revision
    );
    let attack = Command {
        id: id(),
        expected_revision: c.revision,
        action: action(&a, &a, "Swords Dance"),
    };
    c = send(&owner, attack).await;
    if !f(&c).queue.is_empty() {
        let effect = Command {
            id: id(),
            expected_revision: c.revision,
            action: Action::ResolveEffect {
                sequence: f(&c).queue[0].sequence,
                reason: "Resolve the original dance".into(),
            },
        };
        c = send(&gm, effect).await;
    }
    assert_eq!(f(&c).ability_events[0].combatant_id, dancer);
    let source_successes = f(&c).sequencing.dances[&f(&c).ability_events[0].id].successes;
    let before = serde_json::to_value(c.battle.as_ref().unwrap()).unwrap();
    let copy = Command {
        id: id(),
        expected_revision: c.revision,
        action: choose(&c, "copy"),
    };
    assert!(host.host.apply(&ids[2], &copy).is_err());
    c = send(&dancer_owner, copy.clone()).await;
    assert_eq!(p(&c, &dancer).actions, 0);
    assert_eq!(f(&c).sequencing.dancer_round[&dancer], 1);
    if source_successes > 0 {
        assert_eq!(f(&c).queue[0].successes, source_successes);
    }
    assert_eq!(
        host.host.apply(&ids[3], &copy).unwrap().revision,
        c.revision
    );
    let mut stale = copy.clone();
    stale.id = id();
    assert!(host.host.apply(&ids[3], &stale).is_err());
    dancer_owner.task.abort();
    let (dancer_owner, snapshot) = room::join(stores[3].clone(), cat, join, notify)
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
            reason: "Restore the free-copy owner choice".into(),
        },
    };
    c = send(&gm, undo).await;
    assert_eq!(
        serde_json::to_value(c.battle.as_ref().unwrap()).unwrap(),
        before
    );
    assert_eq!(
        host.host.apply(&ids[3], &copy).unwrap().revision,
        c.revision
    );
    gm.task.abort();
    owner.task.abort();
    dancer_owner.task.abort();
    let _ = host.stop.take().unwrap().send(());
}

#[test]
fn sequencing_dancer_self_trigger_is_once_per_round_and_rearms_in_next_round() {
    let (c, a, _) = setup("Swords Dance", "Dancer");
    let c = effect(&roll(&c, &action(&a, &a, "Swords Dance")));
    assert_eq!(f(&c).ability_events[0].combatant_id, a);
    let c = roll(&c, &choose(&c, "copy"));
    let mut c = effect(&c);
    assert!(f(&c).ability_events.is_empty());
    assert_eq!(p(&c, &a).actions, 1);
    crate::combat::sequencing::missed(&mut c, &a, "Swords Dance", 0);
    sequencing::lifecycle(&mut c, &catalog()).unwrap();
    assert!(f(&c).ability_events.is_empty());
    c.battle.as_mut().unwrap().round += 1;
    crate::combat::sequencing::missed(&mut c, &a, "Swords Dance", 0);
    sequencing::lifecycle(&mut c, &catalog()).unwrap();
    assert!(!f(&c).ability_events.is_empty());
}
#[test]
fn sequencing_mid_turn_truant_activation_starts_at_one_and_end_scene_clears_live_state() {
    let (mut c, a, t) = setup("Tackle", "");
    pm(&mut c, &a).selected_ability = "Truant".into();
    sequencing::lifecycle(&mut c, &catalog()).unwrap();
    assert_eq!(f(&c).sequencing.visits[&a], 1);
    force(&mut c, &t);
    force(&mut c, &a);
    assert_eq!(f(&c).sequencing.visits[&a], 2);
    let c = battle::transition(&c, "gm", &Action::EndBattle, vec![], &catalog())
        .unwrap()
        .0;
    let state = &c.battle.as_ref().unwrap().flow.as_ref().unwrap().sequencing;
    assert!(state.visits.is_empty());
    assert!(state.gorilla.is_empty());
    assert!(state.dances.is_empty());
}

#[test]
fn sequencing_parental_bond_shared_attack_keeps_its_frozen_ability_after_contact_change() {
    let (mut c, a, t) = setup("Swift", "Parental Bond");
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .stats
        .insert("Insight".into(), 1);
    let high = add(&mut c, &t, "");
    let sid = p(&c, &high).sheet_id.clone();
    c.sheets
        .get_mut(&sid)
        .unwrap()
        .stats
        .insert("Insight".into(), 7);
    fm(&mut c).ability_changes.enabled = true;
    let cmd = Action::AccuracyTargets {
        combatant_id: a.clone(),
        target_ids: vec![t, high],
        move_name: "Swift".into(),
        pool_modifier: 0,
        success_modifier: 0,
        reason: "An action with frozen ability identity".into(),
    };
    let mut c = roll(&c, &cmd);
    while f(&c).window {
        c = super::targeting_tests::pass(&c);
    }
    c = roll(&c, &damage(&c));
    pm(&mut c, &a).selected_ability = "Mummy".into();
    fm(&mut c)
        .ability_changes
        .assignments
        .insert(a.clone(), "Mummy".into());
    let next = damage(&c);
    let doubled = rules::dice_count(&c, &next, &catalog()).unwrap();
    let mut old = c.clone();
    fm(&mut old).sequencing.enabled = false;
    assert_eq!(
        doubled,
        rules::dice_count(&old, &next, &catalog()).unwrap() * 2
    );
    assert!(doubled > 0);
    assert_eq!(p(&c, &a).selected_ability, "Mummy");
}
