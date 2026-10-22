use super::*;
use crate::combat::{
    forms,
    information::{self, InformationState},
};
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
fn setup(ability: &str, moves: &[&str]) -> (Campaign, String, String) {
    let (mut c, a, t) = super::special_tests::setup("Tackle");
    let own = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&own).unwrap().species_id = Some(
        catalog().find("species", "Name", "Bulbasaur").unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
    c.sheets.get_mut(&own).unwrap().moves = vec!["Tackle".into()];
    pm(&mut c, &a).selected_ability = ability.into();
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().moves = moves.iter().map(|s| (*s).into()).collect();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .information = InformationState {
        enabled: true,
        ..Default::default()
    };
    information::lifecycle(&mut c, &catalog()).unwrap();
    (c, a, t)
}
fn action(c: &Campaign, choice: &str) -> Action {
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
        reason: "Private source review and trainer Insight confirmed".into(),
    }
}
fn resolve(c: &Campaign, choice: &str) -> Campaign {
    battle::transition(c, "gm", &action(c, choice), vec![], &catalog())
        .unwrap()
        .0
}
#[test]
fn information_anticipation_classifies_lethal_super_effective_support_and_immunity() {
    let (c, a, _) = setup(
        "Anticipation",
        &["Flamethrower", "Guillotine", "Swords Dance", "Tackle"],
    );
    let f = c.battle.as_ref().unwrap().flow.as_ref().unwrap();
    let offer = f.information.offers.values().next().unwrap();
    assert!(offer
        .candidates
        .iter()
        .any(|m| m.move_name == "Flamethrower" && m.super_effective));
    assert!(offer
        .candidates
        .iter()
        .any(|m| m.move_name == "Guillotine" && m.lethal));
    assert!(!offer
        .candidates
        .iter()
        .any(|m| m.move_name == "Swords Dance"));
    assert_eq!(f.ability_events[0].choices, vec!["alert"]);
    assert!(battle::transition(&c, "gm", &action(&c, "clear"), vec![], &catalog()).is_err());
    let n = resolve(&c, "alert");
    let reveal = &n
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .information
        .reveals[0];
    assert!(reveal.alert);
    assert!(reveal.move_name.is_none());
    let mut ghost = c.clone();
    ghost
        .sheets
        .get_mut(&p(&c, &a).sheet_id)
        .unwrap()
        .species_id = Some(
        catalog().find("species", "Name", "Gastly").unwrap()["AppId"]
            .as_str()
            .unwrap()
            .into(),
    );
    ghost
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .information
        .active
        .clear();
    ghost
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_events
        .clear();
    information::lifecycle(&mut ghost, &catalog()).unwrap();
    let f = ghost.battle.as_ref().unwrap().flow.as_ref().unwrap();
    assert!(
        !f.information
            .offers
            .values()
            .last()
            .unwrap()
            .candidates
            .iter()
            .find(|m| m.move_name == "Guillotine")
            .unwrap()
            .lethal
    );
}
#[test]
fn information_forewarn_preserves_ties_variable_cards_and_gm_insight_rulings() {
    let (c, _, _) = setup(
        "Forewarn",
        &[
            "Flamethrower",
            "Fissure",
            "Tackle",
            "Water Gun",
            "Flamethrower",
        ],
    );
    let f = c.battle.as_ref().unwrap().flow.as_ref().unwrap();
    assert_eq!(
        f.information
            .offers
            .values()
            .next()
            .unwrap()
            .candidates
            .len(),
        4
    );
    assert!(f
        .information
        .offers
        .values()
        .next()
        .unwrap()
        .candidates
        .iter()
        .find(|m| m.move_name == "Fissure")
        .unwrap()
        .power
        .is_none());
    let candidates = &f.information.offers.values().next().unwrap().candidates;
    let tied: Vec<_> = candidates
        .iter()
        .filter(|m| m.power == Some(2))
        .map(|m| m.move_name.as_str())
        .collect();
    assert_eq!(tied, vec!["Tackle", "Water Gun"]);
    for choice in ["move:0", "move:1", "move:2", "move:3", "none", "manual"] {
        let n = resolve(&c, choice);
        assert_eq!(
            n.battle
                .as_ref()
                .unwrap()
                .flow
                .as_ref()
                .unwrap()
                .information
                .reveals
                .len(),
            1
        );
        assert!(n
            .events
            .last()
            .is_none_or(|e| !e.description.contains("Flamethrower")));
    }
    assert!(battle::transition(&c, "gm", &action(&c, "move:9"), vec![], &catalog()).is_err());
}
#[test]
fn information_only_gm_resolves_even_guessed_choices_and_stale_inputs_expire() {
    let (mut c, _, t) = setup("Forewarn", &["Solar Beam"]);
    for actor in ["player", "other", "outsider"] {
        for choice in ["move:0", "move:999", "manual", "none"] {
            assert_eq!(
                battle::transition(&c, actor, &action(&c, choice), vec![], &catalog())
                    .unwrap_err()
                    .code,
                "permission"
            );
        }
    }
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().moves.clear();
    assert!(battle::transition(&c, "gm", &action(&c, "move:0"), vec![], &catalog()).is_err());
    let n = resolve(&c, "manual");
    assert_eq!(
        n.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .information
            .reveals[0]
            .decision,
        "manual"
    );
}
#[test]
fn information_reentry_suppression_fainting_and_reserves_do_not_repeat_windows() {
    let (mut c, a, t) = setup("Anticipation", &["Flamethrower"]);
    let once = resolve(&c, "alert");
    c = once;
    information::lifecycle(&mut c, &catalog()).unwrap();
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events
        .is_empty());
    pm(&mut c, &a).selected_ability.clear();
    information::lifecycle(&mut c, &catalog()).unwrap();
    pm(&mut c, &a).selected_ability = "Anticipation".into();
    information::lifecycle(&mut c, &catalog()).unwrap();
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events
            .len(),
        1
    );
    pm(&mut c, &a).deployed = false;
    information::lifecycle(&mut c, &catalog()).unwrap();
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events
        .is_empty());
    pm(&mut c, &a).deployed = true;
    pm(&mut c, &t).hp = 0;
    information::lifecycle(&mut c, &catalog()).unwrap();
    assert!(c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events
        .is_empty());
    pm(&mut c, &t).hp = 1;
    information::lifecycle(&mut c, &catalog()).unwrap();
    assert_eq!(
        c.battle
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
fn information_recipient_projection_hides_candidates_movepools_archives_and_undo() {
    let (mut c, a, t) = setup("Forewarn", &["Solar Beam"]);
    let trainer = c.sheets.values().find(|s| s.kind == "trainer").cloned();
    let mut trainer = trainer.unwrap_or_else(|| c.sheets[&p(&c, &a).sheet_id].clone());
    trainer.id = "secret-trainer".into();
    trainer.kind = "trainer".into();
    trainer.owner_id = "recipient".into();
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().trainer_id = Some(trainer.id.clone());
    c.sheets.insert(trainer.id.clone(), trainer);
    let eid = c
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_events[0]
        .id
        .clone();
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .information
        .offers
        .get_mut(&eid)
        .unwrap()
        .recipient = "recipient".into();
    let n = resolve(&c, "move:0");
    let encoded = serde_json::to_string(&n).unwrap();
    assert!(!encoded.contains("chose move:"));
    assert!(
        serde_json::to_string(&crate::privacy::for_player(&n, "recipient"))
            .unwrap()
            .contains("Solar Beam")
    );
    let view = crate::privacy::for_player(&n, "player");
    assert!(view
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .information
        .reveals
        .is_empty());
    assert!(!serde_json::to_string(&view).unwrap().contains("Solar Beam"));
    let mut archived = n.clone();
    archived.finished_battles.push(n.battle.clone().unwrap());
    let foreign = crate::privacy::for_player(&archived, "observer");
    assert!(!serde_json::to_string(&foreign)
        .unwrap()
        .contains("Solar Beam"));
    assert!(foreign.sheets[&p(&n, &t).sheet_id].moves.is_empty());
    assert!(n.sheets[&p(&n, &t).sheet_id]
        .moves
        .contains(&"Solar Beam".into()));
}
#[test]
fn information_uses_virtual_form_moves_and_historical_engines_keep_old_gates() {
    let (mut c, _, t) = setup("Forewarn", &["Tackle"]);
    let sheet = c.sheets[&p(&c, &t).sheet_id].clone();
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.forms.enabled = true;
    let mut transformed = sheet;
    transformed.moves = vec!["Solar Beam".into()];
    f.forms.current.insert(
        t,
        forms::FormView {
            sheet: transformed,
            types: vec!["Normal".into()],
            ability: String::new(),
            label: "Transform".into(),
            source: "Transform".into(),
            mega: false,
            secondary: None,
            partner: None,
        },
    );
    f.ability_events.clear();
    f.information.active.clear();
    f.information.offers.clear();
    information::lifecycle(&mut c, &catalog()).unwrap();
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .information
            .offers
            .values()
            .next()
            .unwrap()
            .candidates[0]
            .move_name,
        "Solar Beam"
    );
    for engine in [FORM_ENGINE, ITEM_ENGINE, EVENT_ENGINE] {
        let mut old = c.clone();
        old.engine = engine.into();
        old.battle
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .information = Default::default();
        information::lifecycle(&mut old, &catalog()).unwrap();
        assert!(!information::enabled(&old));
    }
    let mut value = serde_json::to_value(&c).unwrap();
    value["battle"]["flow"]
        .as_object_mut()
        .unwrap()
        .remove("information");
    let legacy: Campaign = serde_json::from_value(value).unwrap();
    assert!(!information::enabled(&legacy));
}

#[tokio::test]
async fn information_room_private_trainer_transport_receipts_stale_reconnect_and_rollback() {
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
    let dirs: Vec<_> = (0..5).map(|_| tempfile::tempdir().unwrap()).collect();
    let stores: Vec<_> = dirs
        .iter()
        .map(|d| Arc::new(Mutex::new(Store::open(d.path()).unwrap())))
        .collect();
    let ids: Vec<_> = stores
        .iter()
        .map(|s| s.lock().unwrap().player_id.clone())
        .collect();
    let (mut c, a, _) = setup("Forewarn", &["Solar Beam"]);
    c.host_id = ids[0].clone();
    c.gm_id = ids[2].clone();
    pm(&mut c, &a).owner_id = ids[1].clone();
    let sid = p(&c, &a).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().owner_id = ids[1].clone();
    let mut trainer = c.sheets[&sid].clone();
    trainer.id = "private-trainer".into();
    trainer.kind = "trainer".into();
    trainer.owner_id = ids[3].clone();
    trainer.moves.clear();
    c.sheets.get_mut(&sid).unwrap().trainer_id = Some(trainer.id.clone());
    c.sheets.insert(trainer.id.clone(), trainer);
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.ability_events.clear();
    f.information.active.clear();
    f.information.offers.clear();
    information::lifecycle(&mut c, &catalog()).unwrap();
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
    let req = JoinRequest {
        address: hosted.info.address.replace("<host-ip>", "127.0.0.1"),
        token: hosted.info.token.clone(),
    };
    let mut remotes = Vec::new();
    for store in stores.iter().skip(1) {
        let (r, w) = room::join(store.clone(), cat.clone(), req.clone(), Arc::new(|_, _| {}))
            .await
            .unwrap();
        if w.gm_id != store.lock().unwrap().player_id {
            assert!(!serde_json::to_string(&w).unwrap().contains("Solar Beam"));
        }
        remotes.push(r);
    }
    let command = Command {
        id: "private-warning".into(),
        expected_revision: c.revision,
        action: action(&c, "move:0"),
    };
    let error = send(&remotes[0], command.clone()).await.unwrap_err();
    assert_eq!(error.code, "permission");
    let mut guess = command.clone();
    guess.id = "guess-private".into();
    guess.action = action(&c, "move:999");
    assert_eq!(
        send(&remotes[3], guess).await.unwrap_err().code,
        "permission"
    );
    let changed = send(&remotes[1], command.clone()).await.unwrap();
    assert_eq!(
        send(&remotes[1], command.clone()).await.unwrap().revision,
        changed.revision
    );
    let mut stale = command;
    stale.id = "stale-private".into();
    assert_eq!(send(&remotes[1], stale).await.unwrap_err().code, "stale");
    assert!(changed
        .events
        .iter()
        .all(|e| !e.description.contains("Solar Beam")));
    for r in &remotes {
        r.task.abort();
    }
    for r in &mut remotes {
        let _ = (&mut r.task).await;
    }
    remotes.clear();
    for (i, store) in stores.iter().enumerate().skip(1) {
        let (r, w) = room::join(store.clone(), cat.clone(), req.clone(), Arc::new(|_, _| {}))
            .await
            .unwrap();
        let messages = &w
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .information
            .reveals;
        assert_eq!(messages.len(), usize::from(i == 2 || i == 3));
        if i == 1 || i == 4 {
            assert!(!serde_json::to_string(&w).unwrap().contains("Solar Beam"));
        }
        remotes.push(r);
    }
    let saved = stores[0].lock().unwrap().campaigns().unwrap().remove(0);
    assert_eq!(
        saved
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .information
            .reveals[0]
            .move_name
            .as_deref(),
        Some("Solar Beam")
    );
    let undone = send(
        &remotes[1],
        Command {
            id: "undo-private".into(),
            expected_revision: changed.revision,
            action: Action::RollbackBattle {
                event_sequence: changed.battle_undo.last().unwrap().event_sequence,
                reason: "Restore the private entry decision".into(),
            },
        },
    )
    .await
    .unwrap();
    assert!(undone
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .information
        .reveals
        .is_empty());
    assert_eq!(
        undone
            .battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .information
            .offers
            .len(),
        1
    );
    let _ = hosted.host.shutdown.send(true);
    for r in &remotes {
        r.task.abort();
    }
    for r in remotes {
        let _ = r.task.await;
    }
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
fn information_rechecks_foe_movepools_and_converted_types_without_round_spam() {
    let (mut c, _, t) = setup("Anticipation", &["Tackle"]);
    c = resolve(&c, "clear");
    let sid = p(&c, &t).sheet_id.clone();
    c.sheets.get_mut(&sid).unwrap().moves = vec!["Flamethrower".into()];
    information::lifecycle(&mut c, &catalog()).unwrap();
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events[0]
            .choices,
        vec!["alert"]
    );
    c = resolve(&c, "alert");
    pm(&mut c, &t).selected_ability = "Normalize".into();
    information::lifecycle(&mut c, &catalog()).unwrap();
    assert_eq!(
        c.battle
            .as_ref()
            .unwrap()
            .flow
            .as_ref()
            .unwrap()
            .ability_events[0]
            .choices,
        vec!["clear"]
    );
    c = resolve(&c, "clear");
    information::lifecycle(&mut c, &catalog()).unwrap();
    assert!(c
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
fn information_unknown_cards_remain_private_rulings_and_gm_notes_never_reach_trainers() {
    let (c, _, _) = setup("Anticipation", &["Unresolved secret card"]);
    let n = resolve(&c, "manual");
    let view = crate::privacy::for_player(&n, "player");
    let r = &view
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .information
        .reveals[0];
    assert!(r.reason.is_empty());
    assert_eq!(r.decision, "manual");
    assert!(!serde_json::to_string(&view)
        .unwrap()
        .contains("Unresolved secret card"));
    assert!(!n
        .battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .information
        .reveals[0]
        .reason
        .is_empty());
}

#[test]
fn information_conditional_and_copy_cards_require_review_without_false_safe_results() {
    let (c, _, _) = setup(
        "Anticipation",
        &[
            "Metronome",
            "Nature Power",
            "Assist",
            "Swords Dance",
            "Bide",
        ],
    );
    let f = c.battle.as_ref().unwrap().flow.as_ref().unwrap();
    let candidates = &f.information.offers.values().next().unwrap().candidates;
    for name in ["Metronome", "Assist", "Nature Power"] {
        assert!(
            candidates
                .iter()
                .find(|m| m.move_name == name)
                .unwrap()
                .uncertain
        );
    }
    assert!(candidates.iter().any(|m| m.move_name == "Bide"));
    assert!(!candidates.iter().any(|m| m.move_name == "Swords Dance"));
    assert_eq!(f.ability_events[0].choices, vec!["alert", "clear"]);
}
