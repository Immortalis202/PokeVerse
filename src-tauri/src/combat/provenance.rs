//! Damage-provenance milestone, versioned by a saved-battle feature flag.
//! Pokerole 3.0 Tinted Lens: ignore one resistance, including one of two.
use super::*;

pub fn enabled(c: &Campaign) -> bool {
    flow(c).is_ok_and(|f| f.provenance_rules)
}

pub fn choose_sheer_force(
    c: &Campaign,
    actor: &str,
    id: &str,
    ignore: bool,
    why: &str,
    dice: &[u8],
) -> Result<(Campaign, String)> {
    if !enabled(c) {
        return Err(error(
            "Upgrade damage-provenance rules before choosing Sheer Force",
        ));
    }
    owner(c, actor, id)?;
    reason(why)?;
    check_count(dice, 0)?;
    if fighter(c, id)?.selected_ability != "Sheer Force" || fighter(c, id)?.hp <= 0 {
        return Err(error("Choose an active, conscious Sheer Force holder"));
    }
    if !flow(c)?.queue.is_empty() || abilities::pending(c) {
        return Err(error(
            "Choose Sheer Force before declaring an action or resolving a trigger",
        ));
    }
    let mut next = c.clone();
    let flags = &mut fighter_mut(&mut next, id)?.runtime.ability.flags;
    if ignore {
        flags.insert("sheer-force:damage".into());
    } else {
        flags.remove("sheer-force:damage");
    }
    let log = format!(
        "Sheer Force for {}: {}. {why}",
        fighter(c, id)?.name,
        if ignore {
            "ignore added effects for two extra damage dice"
        } else {
            "retain added effects"
        }
    );
    checkpoint(c, &mut next, &log);
    Ok((next, log))
}

pub fn tag_actions(before: &Campaign, after: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !enabled(before) {
        return Ok(());
    }
    let mut existing: BTreeSet<_> = flow(before)?
        .queue
        .iter()
        .map(|q| (q.actor_id.clone(), q.sequence, q.move_name.clone()))
        .collect();
    existing.extend(flow(before)?.special.charges.values().map(|charge| {
        (
            charge.action.actor_id.clone(),
            charge.action.sequence,
            charge.action.move_name.clone(),
        )
    }));
    let selected: BTreeSet<_> = battle(before)?
        .combatants
        .iter()
        .filter(|p| {
            p.selected_ability == "Sheer Force"
                && p.runtime.ability.flags.contains("sheer-force:damage")
        })
        .map(|p| p.id.clone())
        .collect();
    let mut item_input = None;
    if let Some(f) = after.battle.as_mut().and_then(|b| b.flow.as_mut()) {
        for q in &mut f.queue {
            if !existing.contains(&(q.actor_id.clone(), q.sequence, q.move_name.clone())) {
                let raw = move_entry(cat, &q.move_name)?;
                let m = if items::enabled(before) {
                    // Eligibility must use this action's target/category, including
                    // Bide preparation and friendly/healing variants of attacks.
                    let input = item_input.get_or_insert_with(|| before.clone());
                    flow_mut(input).queue = vec![q.clone()];
                    field::move_data(input, fighter(input, &q.actor_id)?, &raw)
                } else {
                    raw
                };
                q.sheer_force = selected.contains(&q.actor_id)
                    && !(items::enabled(before) && special::fixed(&q.move_name))
                    && ["Physical", "Special"].contains(&m["Category"].as_str().unwrap_or(""))
                    && (m["AddedEffects"]
                        .as_object()
                        .is_some_and(|effects| !effects.is_empty())
                        || items::chance_base(before, fighter(before, &q.actor_id)?, &m) > 0);
            }
        }
    }
    Ok(())
}

pub fn sheer_force_bonus(c: &Campaign, source: &str, sequence: u32) -> i32 {
    if !enabled(c) {
        return 0;
    }
    2 * i32::from(flow(c).is_ok_and(|f| {
        f.queue
            .iter()
            .any(|q| q.actor_id == source && q.sequence == sequence && q.sheer_force)
    }))
}

pub fn retained_attributes(q: &QueuedAction, cat: &Catalog) -> Result<bool> {
    let m = move_entry(cat, &q.move_name)?;
    Ok(m["Attributes"]["Recoil"] == true || m["Attributes"]["MustRecharge"] == true)
}

pub fn ignore_resistance(c: &Campaign, attacker: &Combatant, target: &Combatant) -> bool {
    enabled(c) && attacker.side != target.side && attacker.selected_ability == "Tinted Lens"
}

pub fn ignores_attributes(c: &Campaign, holder: &Combatant, foe: &Combatant) -> bool {
    enabled(c) && holder.side != foe.side && holder.selected_ability == "Unaware"
}

pub fn clash_bonus(
    c: &Campaign,
    source: &str,
    target: &str,
    m: &serde_json::Value,
    cat: &Catalog,
) -> Result<i32> {
    if !enabled(c) {
        return Ok(0);
    }
    let a = fighter(c, source)?;
    let t = fighter(c, target)?;
    let (effect, immune) = rules::type_matchup(c, a, t, m["Type"].as_str().unwrap_or(""), cat)?;
    Ok(i32::from(
        !immune
            && !abilities::damage_immune_from(c, a, t, m)
            && !extra::source_immune(c, a, t, effect)
            && a.selected_ability == "Stakeout"
            && a.runtime
                .ability
                .flags
                .contains(&format!("stakeout:{target}")),
    ))
}

/// Observe actual replacements, including switches performed by reviewed move effects.
/// A reserve's first deployment without an outgoing occupant does not qualify.
pub fn lifecycle(before: &Campaign, after: &mut Campaign, action: &Action) {
    if !enabled(before) || matches!(action, Action::RollbackBattle { .. }) {
        return;
    }
    let Some(old) = before.battle.as_ref() else {
        return;
    };
    let Some(new) = after.battle.as_mut() else {
        return;
    };
    let active: BTreeSet<_> = new
        .combatants
        .iter()
        .filter(|p| new.active && p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    let arrivals: Vec<_> = new
        .combatants
        .iter()
        .filter(|p| {
            p.deployed
                && p.hp > 0
                && old.combatants.iter().any(|q| q.id == p.id && !q.deployed)
                && old.combatants.iter().any(|q| {
                    q.deployed
                        && q.side == p.side
                        && q.field_slot == p.field_slot
                        && !active.contains(&q.id)
                })
        })
        .map(|p| (p.id.clone(), p.side.clone()))
        .collect();
    let frozen_sources: BTreeSet<_> = new
        .flow
        .as_ref()
        .into_iter()
        .flat_map(|f| &f.queue)
        .filter(|q| {
            q.damage_context.as_ref().is_some_and(|ctx| {
                ctx.ability_sources
                    .iter()
                    .any(|p| p.id == q.actor_id && p.selected_ability == "Stakeout")
            })
        })
        .map(|q| q.actor_id.clone())
        .collect();
    for p in &mut new.combatants {
        p.runtime.ability.flags.retain(|f| {
            !f.starts_with("stakeout:")
                || ((active.contains(&p.id) || frozen_sources.contains(&p.id))
                    && active.contains(&f[9..])
                    && (p.selected_ability == "Stakeout" || frozen_sources.contains(&p.id)))
        });
        if p.selected_ability == "Stakeout"
            && active.contains(&p.id)
            && old
                .combatants
                .iter()
                .any(|q| q.id == p.id && q.deployed && q.hp > 0 && q.selected_ability == "Stakeout")
        {
            for (id, side) in &arrivals {
                if *side != p.side {
                    p.runtime.ability.flags.insert(format!("stakeout:{id}"));
                }
            }
        }
    }
}

pub fn stakeout_bonus(c: &Campaign, q: &QueuedAction, immune: bool) -> i32 {
    if !enabled(c) || immune || q.successes <= 0 {
        return 0;
    }
    let Ok(source) = reviewed::source(c, q) else {
        return 0;
    };
    i32::from(
        source.selected_ability == "Stakeout"
            && fighter(c, &q.actor_id).is_ok_and(|p| {
                p.runtime
                    .ability
                    .flags
                    .contains(&format!("stakeout:{}", q.target_id))
            }),
    )
}

pub fn consume_stakeout(c: &mut Campaign, q: &QueuedAction, immune: bool) {
    if !enabled(c) || immune || q.successes <= 0 {
        return;
    }
    if let Ok(p) = fighter_mut(c, &q.actor_id) {
        p.runtime
            .ability
            .flags
            .remove(&format!("stakeout:{}", q.target_id));
    }
}
