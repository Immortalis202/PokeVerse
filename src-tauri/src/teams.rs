//! Registered teams and Trainer Area switching (corebook pp.64–65).
use crate::{battle, catalog::Catalog, model::*};
use std::collections::{BTreeMap, BTreeSet};

pub fn start(
    c: &Campaign,
    actor: &str,
    roster: &[String],
    sides: &BTreeMap<String, String>,
    starters: &[String],
    slots: u32,
    cat: &Catalog,
) -> Result<(Campaign, String)> {
    if !(1..=6).contains(&slots)
        || starters.is_empty()
        || starters.len() != starters.iter().collect::<BTreeSet<_>>().len()
    {
        return Err(AppError::new(
            "validation",
            "Choose 1–6 active slots per side and unique starters",
        ));
    }
    let mut counts = BTreeMap::<String, u32>::new();
    for id in roster {
        let p = c
            .sheets
            .get(id)
            .filter(|p| p.kind == "pokemon" && !p.archived)
            .ok_or_else(|| AppError::new("validation", "Register campaign Pokémon only"))?;
        let t = p
            .trainer_id
            .as_ref()
            .and_then(|id| c.sheets.get(id))
            .filter(|t| t.kind == "trainer" && !t.archived && t.owner_id == p.owner_id)
            .ok_or_else(|| {
                AppError::new(
                    "validation",
                    "Every roster Pokémon needs its active campaign trainer",
                )
            })?;
        let _ = t;
        let side = sides
            .get(id)
            .filter(|s| ["Players", "Opponents"].contains(&s.as_str()))
            .ok_or_else(|| {
                AppError::new(
                    "validation",
                    "Assign every roster Pokémon to Players or Opponents",
                )
            })?;
        counts.entry(side.clone()).or_default();
        if starters.contains(id) {
            if p.hp == 0 {
                return Err(AppError::new(
                    "validation",
                    "A fainted Pokémon cannot start in an active slot",
                ));
            }
            *counts.get_mut(side).unwrap() += 1;
        }
    }
    if counts.len() != 2
        || counts.values().any(|count| *count == 0 || *count > slots)
        || starters.iter().any(|id| !roster.contains(id))
    {
        return Err(AppError::new("validation","Both teams need 1 to the configured number of active Pokémon; starters must belong to the roster"));
    }
    let (mut next, _) = battle::transition(
        c,
        actor,
        &Action::StartBattle {
            environment: None,
            sheet_ids: roster.to_vec(),
            sides: sides.clone(),
        },
        vec![],
        cat,
    )?;
    let b = next.battle.as_mut().unwrap();
    b.slots_per_side = slots;
    let mut next_slot = BTreeMap::<String, u32>::new();
    for p in &mut b.combatants {
        p.deployed = starters.contains(&p.sheet_id);
        if p.deployed {
            let n = next_slot.entry(p.side.clone()).or_default();
            p.field_slot = Some(*n);
            *n += 1;
        }
    }
    Ok((next,format!("Registered {} Pokémon with up to {slots} active slots per side; reserves cannot act or be targeted",roster.len())))
}

pub fn switch(
    c: &Campaign,
    actor: &str,
    outgoing: &str,
    incoming: &str,
    exception: bool,
    reason: &str,
) -> Result<(Campaign, String)> {
    if crate::combat::forms::joined_component(c, incoming) {
        return Err(AppError::new(
            "battle",
            "A component of an active As One union cannot enter separately",
        ));
    }
    if !exception && crate::combat::forms::blocks_recall(c, outgoing) {
        return Err(AppError::new(
            "battle",
            "Commander blocks this Pokemon for the scene",
        ));
    }
    if reason.trim().is_empty() || reason.len() > 1000 {
        return Err(AppError::new("validation", "Record a switching reason"));
    }
    let b = c
        .battle
        .as_ref()
        .filter(|b| b.active && b.slots_per_side > 0)
        .ok_or_else(|| AppError::new("battle", "Switching requires a registered team battle"))?;
    let out = b
        .combatants
        .iter()
        .find(|p| p.id == outgoing && p.deployed)
        .ok_or_else(|| AppError::new("battle", "Select an active outgoing Pokémon"))?;
    let entry = b
        .combatants
        .iter()
        .find(|p| p.id == incoming && !p.deployed && p.hp > 0)
        .ok_or_else(|| AppError::new("battle", "Select a conscious reserve"))?;
    let trainer = c
        .sheets
        .get(&out.sheet_id)
        .and_then(|s| s.trainer_id.as_ref())
        .ok_or_else(|| AppError::new("missing", "Outgoing trainer link is missing"))?;
    if out.side != entry.side
        || out.owner_id != entry.owner_id
        || c.sheets
            .get(&entry.sheet_id)
            .and_then(|s| s.trainer_id.as_ref())
            != Some(trainer)
    {
        return Err(AppError::new(
            "ownership",
            "A reserve must belong to the same trainer and side as the outgoing Pokémon",
        ));
    }
    let gm = actor == c.gm_id;
    if (!gm && out.owner_id != actor) || (exception && !gm) {
        return Err(AppError::new(
            "permission",
            "Only the owner or GM can switch; rule overrides require the GM",
        ));
    }
    if b.combatants.iter().any(|p| p.pending_damage.is_some()) {
        return Err(AppError::new(
            "battle",
            "Resolve or cancel pending attacks before switching",
        ));
    }
    let previous = *b.trainer_switches.get(trainer).unwrap_or(&0);
    let actions = *b.trainer_actions.get(trainer).unwrap_or(&0);
    let cost = if previous == 0 {
        0
    } else if previous == 1 {
        1
    } else {
        2
    };
    if !exception && (previous > 998 || actions.saturating_add(cost) > 5) {
        return Err(AppError::new(
            "battle",
            "Trainer has no switching actions left this round",
        ));
    }
    let mut next = c.clone();
    let b = next.battle.as_mut().unwrap();
    if !exception {
        b.trainer_switches.insert(trainer.clone(), previous + 2);
        b.trainer_actions.insert(trainer.clone(), actions + cost);
    }
    let slot = out.field_slot;
    let p = b.combatants.iter_mut().find(|p| p.id == outgoing).unwrap();
    p.deployed = false;
    p.field_slot = None;
    let p = b.combatants.iter_mut().find(|p| p.id == incoming).unwrap();
    p.deployed = true;
    p.field_slot = slot;
    p.available_round = if exception {
        b.round
    } else {
        b.round
            .checked_add(1)
            .ok_or_else(|| AppError::new("limit", "Round limit reached"))?
    };
    Ok((
        next,
        format!(
            "{} recalled; {} sent out. {}. {reason}",
            out.name,
            entry.name,
            if exception {
                "GM overrides switching timing/cost".into()
            } else {
                format!("{cost} trainer actions; incoming Pokémon can act next round")
            }
        ),
    ))
}

pub fn ready(p: &Combatant, round: u32) -> Result<()> {
    if !p.deployed || p.available_round > round {
        return Err(AppError::new(
            "battle",
            "Reserve or newly switched Pokémon cannot act yet",
        ));
    }
    Ok(())
}
