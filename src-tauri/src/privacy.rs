//! Per-player projections. The authoritative campaign is never replaced by a view.
use crate::model::*;
use std::collections::BTreeSet;

fn equipment(sheet: &mut Sheet, player: &str) {
    if sheet.owner_id != player {
        sheet.held_items = None;
        sheet.stored_items = None;
        sheet.inventory.clear();
    }
}
fn learned_moves(sheet: &mut Sheet, player: &str) {
    if sheet.owner_id != player {
        sheet.moves.clear();
        if let Some(build) = sheet.workflow.as_mut() {
            build.learned.clear();
            build.forgotten.clear();
            build.ledger.clear();
        }
    }
}
fn action_view(action: &mut crate::combat::QueuedAction, owned: &BTreeSet<String>) {
    if owned.contains(&action.actor_id) {
        return;
    }
    if let Some(context) = action.damage_context.as_mut() {
        context.expert_belt = false;
    }
}
fn battle_view(battle: &mut Battle, player: &str, owned_sheets: &BTreeSet<String>) {
    let Some(flow) = battle.flow.as_mut() else {
        return;
    };
    let owned: BTreeSet<_> = battle
        .combatants
        .iter()
        .filter(|p| p.owner_id == player)
        .map(|p| p.id.clone())
        .collect();
    if flow.information.enabled {
        flow.information.offers.clear();
        flow.information.active.clear();
        flow.information.reveals.retain(|r| r.recipient == player);
        for message in &mut flow.information.reveals {
            message.reason.clear();
            if message.move_name.is_some() {
                message.decision = "reveal".into();
            }
        }
        for event in &mut flow.ability_events {
            if event.kind.starts_with("info-")
                || (event.kind == "sequence-gorilla" && !owned.contains(&event.combatant_id))
            {
                event.choices.clear();
            }
        }
        for original in flow.forms.originals.values_mut() {
            learned_moves(&mut original.sheet, player);
        }
        for current in flow.forms.current.values_mut() {
            learned_moves(&mut current.sheet, player);
        }
    }
    for original in flow.forms.originals.values_mut() {
        equipment(&mut original.sheet, player);
    }
    for current in flow.forms.current.values_mut() {
        equipment(&mut current.sheet, player);
    }
    flow.forms.plans.retain(|id, _| owned.contains(id));
    for action in &mut flow.queue {
        action_view(action, &owned);
    }
    for charge in flow.special.charges.values_mut() {
        action_view(&mut charge.action, &owned);
    }
    if let Some(series) = flow.special.series.as_mut() {
        for action in series.pending.iter_mut().chain(series.follow_up.iter_mut()) {
            action_view(action, &owned);
        }
    }
    for (action, _) in flow.tactics.clash_pending.values_mut() {
        action_view(action, &owned);
    }
    for action in &mut flow.sequencing.pending {
        action_view(action, &owned);
    }
    for action in flow.sequencing.dances.values_mut() {
        action_view(action, &owned);
    }
    let items = &mut flow.items;
    let tasted: BTreeSet<_> = items
        .history
        .iter()
        .filter(|event| {
            event.kind == "berry_hit" && event.to.as_ref().is_some_and(|id| owned.contains(id))
        })
        .map(|event| event.unit_id.clone())
        .collect();
    let pending_tasted: BTreeSet<_> = flow
        .ability_events
        .iter()
        .filter(|e| e.kind == "item-cud-chew" && owned.contains(&e.combatant_id))
        .filter_map(|e| items.berry_hits.get(&e.id))
        .cloned()
        .collect();
    items.berry_hits.retain(|id, _| {
        flow.ability_events
            .iter()
            .any(|e| e.id == *id && owned.contains(&e.combatant_id))
    });
    items.redacted = items.enabled;
    items.units.retain(|_, unit| {
        ["ground", "consumed", "recycle_pending"].contains(&unit.state.as_str())
            || owned.contains(&unit.holder)
            || owned.contains(&unit.initial_holder)
            || tasted.contains(&unit.unit.id)
            || pending_tasted.contains(&unit.unit.id)
    });
    items.reveals.retain(|reveal| reveal.owner_id == player);
    items
        .fetches
        .retain(|_, attempt| owned_sheets.contains(&attempt.trainer_id));
    items.natural_gift.retain(|id, _| owned.contains(id));
    items
        .chance_moves
        .retain(|key, _| owned.iter().any(|id| key.starts_with(&format!("{id}:"))));
    items
        .life_orb_moves
        .retain(|key, _| owned.iter().any(|id| key.starts_with(&format!("{id}:"))));
    items.fling.retain(|id, _| owned.contains(id));
    items.recycle.retain(|id, _| owned.contains(id));
    items
        .chew_charges
        .retain(|_, charge| owned.contains(&charge.combatant_id));
    items.history.retain(|event| {
        owned.contains(&event.from) || event.to.as_ref().is_some_and(|id| owned.contains(id))
    });
    items.thefts.retain(|event_id, _| {
        flow.ability_events
            .iter()
            .any(|e| e.id == *event_id && owned.contains(&e.combatant_id))
    });
    // Offered items are disclosed only to the owner who must decide on the theft.
    // Their names are copied below from the authoritative units before filtering.
    for event in &mut flow.ability_events {
        if event.kind.starts_with("item-")
            && (!owned.contains(&event.combatant_id)
                || ["item-frisk", "item-symbiosis-order", "item-symbiosis-cycle"]
                    .contains(&event.kind.as_str()))
        {
            event.choices.clear();
        }
    }
    items.symbiosis_chains.clear();
}

pub fn for_player(campaign: &Campaign, player: &str) -> Campaign {
    let mut view = campaign.clone();
    let private_items = campaign
        .sheets
        .values()
        .any(|s| s.held_items.is_some() || s.stored_items.is_some())
        || campaign
            .battle
            .iter()
            .chain(campaign.finished_battles.iter())
            .any(|b| b.flow.as_ref().is_some_and(|f| f.items.enabled));
    let private_moves = campaign
        .battle
        .iter()
        .chain(campaign.finished_battles.iter())
        .any(|b| b.flow.as_ref().is_some_and(|f| f.information.enabled));
    if player == campaign.gm_id || (!private_items && !private_moves) {
        return view;
    }
    let owned_sheets: BTreeSet<_> = campaign
        .sheets
        .values()
        .filter(|s| s.owner_id == player)
        .map(|s| s.id.clone())
        .collect();
    for sheet in view.sheets.values_mut() {
        equipment(sheet, player);
        if private_moves {
            learned_moves(sheet, player);
        }
    }
    view.proposals
        .retain(|p| p.request.sheet.owner_id == player);
    for battle in view
        .battle
        .iter_mut()
        .chain(view.finished_battles.iter_mut())
    {
        // Preserve only the offered units needed for an owned pending decision.
        let offered: Vec<_> = battle
            .flow
            .as_ref()
            .map(|flow| {
                flow.ability_events
                    .iter()
                    .filter(|e| {
                        battle
                            .combatants
                            .iter()
                            .any(|p| p.id == e.combatant_id && p.owner_id == player)
                    })
                    .filter(|e| e.kind == "item-theft")
                    .flat_map(|e| {
                        flow.items
                            .thefts
                            .get(&e.id)
                            .into_iter()
                            .flat_map(|choices| choices.values())
                    })
                    .flat_map(|offer| {
                        std::iter::once(&offer.stolen_id).chain(offer.outgoing_id.iter())
                    })
                    .filter_map(|id| {
                        flow.items
                            .units
                            .get(id)
                            .map(|unit| (id.clone(), unit.clone()))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        battle_view(battle, player, &owned_sheets);
        if let Some(flow) = battle.flow.as_mut() {
            flow.items.units.extend(offered);
        }
    }
    for checkpoint in &mut view.battle_undo {
        battle_view(&mut checkpoint.battle, player, &owned_sheets);
        for (sheet, resources) in &mut checkpoint.resources {
            if !owned_sheets.contains(sheet) {
                resources.equipment = None;
            }
        }
    }
    view
}
