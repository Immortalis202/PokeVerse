//! MOVE-A56 battlefield cards: Court Change, the three Pledges, Mist and Lucky Chant.
use super::*;

fn op(c: &Campaign, name: &str) -> Option<&'static serde_json::Value> {
    completion::definition(c, name)
        .filter(|(kind, _)| *kind == "field-op")
        .map(|(_, d)| d)
}
fn op_name(c: &Campaign, name: &str) -> Option<&'static str> {
    op(c, name).and_then(|d| d["op"].as_str())
}
fn active_rule(c: &Campaign, name: &str, p: &Combatant) -> bool {
    op(c, name).is_some() && field::has(c, name, p)
}

/// Water Pledge adds 2 dice to every Chance pool on the battlefield.
pub fn water_pledge(c: &Campaign) -> u32 {
    let active = op(c, "Water Pledge").is_some()
        && flow(c).is_ok_and(|f| f.battlefield.rules.iter().any(|r| r.kind == "Water Pledge"));
    2 * u32::from(active)
}
/// Mist prevents others from reducing its targets' Attributes and Traits.
pub fn misted(c: &Campaign, p: &Combatant) -> bool {
    active_rule(c, "Mist", p)
}
/// Lucky Chant lets its targets reroll one failed die, like a Chance reroll.
pub fn lucky(c: &Campaign, p: &Combatant) -> bool {
    active_rule(c, "Lucky Chant", p)
}
/// One failed die may be rerolled: a spent Chance this round, or Lucky Chant.
pub fn reroll(c: &Campaign, p: &Combatant) -> bool {
    (battle(c).is_ok_and(|b| p.runtime.chance_round == b.round) && !p.runtime.chance_cancelled)
        || lucky(c, p)
}
/// Fire Pledge's round-end damage reuses the ongoing-damage ticks.
pub fn tick_definition(c: &Campaign, name: &str) -> Option<&'static serde_json::Value> {
    op(c, name).filter(|d| d["op"] == "fire-pledge")
}

fn side_scope(c: &Campaign, id: &str) -> Result<String> {
    Ok(format!("side:{}", fighter(c, id)?.side))
}
fn install(c: &mut Campaign, name: &str, scope: &str, rounds: Option<u32>, source: &str) {
    flow_mut(c)
        .battlefield
        .rules
        .retain(|r| r.kind != name || r.scope != scope);
    field::insert(c, name, source, scope, rounds, 0);
}
fn burning(c: &Campaign, id: &str) -> bool {
    flow(c).is_ok_and(|f| {
        f.special
            .effects
            .iter()
            .any(|e| e.target == id && e.name == "ongoing:Fire Pledge")
    })
}
/// Everyone grounded on the battlefield burns each round end while Fire Pledge lasts.
fn ignite(c: &mut Campaign, source: &str, cat: &Catalog) -> Result<()> {
    let rule = flow(c)?
        .battlefield
        .rules
        .iter()
        .find(|r| r.kind == "Fire Pledge")
        .cloned();
    let Some(rule) = rule else { return Ok(()) };
    let ids: Vec<String> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0 && field::grounded(c, p, cat))
        .map(|p| p.id.clone())
        .collect();
    for id in ids {
        if !burning(c, &id) {
            flow_mut(c).special.effects.push(special::TimedEffect {
                name: "ongoing:Fire Pledge".into(),
                source: source.into(),
                target: id,
                rounds: rule.rounds,
                amount: 2,
                started: 0,
                slot: String::new(),
            });
        }
    }
    Ok(())
}

pub fn effect(c: &mut Campaign, q: &QueuedAction, cat: &Catalog) -> Result<()> {
    if q.ice {
        return Ok(());
    }
    let (a, t) = (q.actor_id.clone(), q.target_id.clone());
    let source = format!("field-user:{a}");
    match op_name(c, &q.move_name).unwrap_or("") {
        "fire-pledge" => {
            if !flow(c)?
                .battlefield
                .rules
                .iter()
                .any(|r| r.kind == "Fire Pledge")
            {
                install(c, "Fire Pledge", "all", Some(4), &source);
            }
            ignite(c, &a, cat)?;
        }
        "water-pledge" => {
            if !flow(c)?
                .battlefield
                .rules
                .iter()
                .any(|r| r.kind == "Water Pledge")
            {
                install(c, "Water Pledge", "all", Some(4), &source);
            }
        }
        "grass-pledge" => {
            if !completion::immune_target(c, q, cat)? {
                extra::change_stat(c, &a, &t, "Dexterity", -1)?;
            }
            let scope = format!("opponents-of:{}", fighter(c, &a)?.side);
            install(c, "Grass Pledge", &scope, None, &format!("hazard:{a}"));
        }
        "mist" => {
            let scope = side_scope(c, &a)?;
            if !flow(c)?
                .battlefield
                .rules
                .iter()
                .any(|r| r.kind == "Mist" && r.scope == scope)
            {
                install(c, "Mist", &scope, Some(4), &source);
            }
        }
        "lucky-chant" => {
            let scope = side_scope(c, &a)?;
            install(c, "Lucky Chant", &scope, None, &source);
        }
        "court-change" => {
            let mine = fighter(c, &a)?.side.clone();
            let theirs: Vec<String> = battle(c)?
                .combatants
                .iter()
                .map(|p| p.side.clone())
                .filter(|s| *s != mine)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let [foe] = theirs.as_slice() else {
                return Err(AppError::new(
                    "manual_rule",
                    "Court Change swaps two sides; adjudicate battles with more sides explicitly",
                ));
            };
            let swap = |scope: &str| -> Option<String> {
                for prefix in ["side:", "opponents-of:"] {
                    if scope == format!("{prefix}{mine}") {
                        return Some(format!("{prefix}{foe}"));
                    }
                    if scope == format!("{prefix}{foe}") {
                        return Some(format!("{prefix}{mine}"));
                    }
                }
                None
            };
            for r in &mut flow_mut(c).battlefield.rules {
                if let Some(s) = swap(&r.scope) {
                    r.scope = s;
                }
            }
        }
        _ => {}
    }
    Ok(())
}
pub fn lifecycle(c: &mut Campaign, cat: &Catalog) -> Result<()> {
    let active = flow(c)?
        .battlefield
        .rules
        .iter()
        .any(|r| r.kind == "Fire Pledge");
    if active {
        let source = flow(c)?
            .battlefield
            .rules
            .iter()
            .find(|r| r.kind == "Fire Pledge")
            .and_then(|r| r.source.strip_prefix("field-user:"))
            .unwrap_or_default()
            .to_string();
        ignite(c, &source, cat)?;
    } else {
        flow_mut(c)
            .special
            .effects
            .retain(|e| e.name != "ongoing:Fire Pledge");
    }
    Ok(())
}
