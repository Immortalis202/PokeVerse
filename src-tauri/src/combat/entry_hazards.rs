//! MOVE-A02: persistent, non-stacking hazards from the three exact source cards.
use super::*;

pub const NAMES: [&str; 3] = ["Spikes", "Stealth Rock", "Sticky Web"];
pub fn enabled(c: &Campaign) -> bool {
    flow(c).is_ok_and(|f| f.entry_hazards)
}
pub fn handles(c: &Campaign, name: &str) -> bool {
    enabled(c)
        && (NAMES.contains(&name)
            || completion::definition(c, name).is_some_and(|(kind, _)| kind == "entry-hazard"))
}
pub fn effect(c: &mut Campaign, q: &QueuedAction) -> Result<bool> {
    if !handles(c, &q.move_name) {
        return Ok(false);
    }
    let source = fighter(c, &q.actor_id)?;
    let scope = q
        .field_scope
        .clone()
        .unwrap_or_else(|| format!("opponents-of:{}", source.side));
    flow_mut(c)
        .battlefield
        .rules
        .retain(|r| r.kind != q.move_name || r.scope != scope);
    field::insert(
        c,
        &q.move_name,
        &format!("hazard:{}", q.actor_id),
        &scope,
        None,
        0,
    );
    Ok(true)
}

pub fn lifecycle(before: &Campaign, c: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let entrants: Vec<String> = battle(c)?
        .combatants
        .iter()
        .filter(|p| {
            p.deployed
                && p.hp > 0
                && c.sheets
                    .get(&p.sheet_id)
                    .is_some_and(|s| s.kind == "pokemon")
                && !flow(c).unwrap().battlefield.known_active.contains(&p.id)
                && !before.battle.as_ref().is_some_and(|b| {
                    b.combatants
                        .iter()
                        .any(|old| old.id == p.id && old.deployed)
                })
        })
        .map(|p| p.id.clone())
        .collect();
    for id in entrants {
        let spider = completion::definition(c, "Spider Web").is_some();
        for name in NAMES.into_iter().chain(spider.then_some("Spider Web")) {
            let p = fighter(c, &id)?;
            if p.hp == 0
                || !field::has(c, name, p)
                || items::completion::has(c, &id, "Heavy-Duty Boots")
            {
                continue;
            }
            if !["Stealth Rock", "Spider Web"].contains(&name)
                && (!field::grounded(c, p, cat)
                    || abilities::damage_immune(
                        c,
                        p,
                        &serde_json::json!({"Type":"Ground", "Category":"Physical"}),
                    ))
            {
                continue;
            }
            match name {
                "Spikes" => field::hurt(c, &id, 1, "Neutral", cat)?,
                "Stealth Rock" => field::hurt(c, &id, 1, "Rock", cat)?,
                "Spider Web" => {
                    let source = flow(c)?
                        .battlefield
                        .rules
                        .iter()
                        .find(|r| r.kind == name && field::applies(&r.scope, p))
                        .unwrap()
                        .source
                        .strip_prefix("hazard:")
                        .ok_or_else(|| {
                            AppError::new(
                                "manual_rule",
                                "Spider Web requires a recorded hazard caster",
                            )
                        })?
                        .to_string();
                    special::registered(c, &source)?;
                    completion::install_named(c, name, &source, &id)?;
                }
                "Sticky Web" => {
                    // Overlapping scopes still apply once. Keep the first installed
                    // matching source for the same non-stacking hazard.
                    let rule = flow(c)?
                        .battlefield
                        .rules
                        .iter()
                        .find(|r| r.kind == name && field::applies(&r.scope, p))
                        .unwrap();
                    let source = rule
                        .source
                        .strip_prefix("hazard:")
                        .ok_or_else(|| {
                            AppError::new(
                                "manual_rule",
                                "Sticky Web requires a recorded hazard caster",
                            )
                        })?
                        .to_string();
                    extra::hazard_stat(c, &source, &id, "Dexterity", -1)?;
                }
                _ => unreachable!(),
            }
        }
    }
    Ok(())
}

pub fn spider_enabled(c: &Campaign) -> bool {
    enabled(c) && completion::definition(c, "Spider Web").is_some()
}
