//! MOVE-A57 held-item cards: theft, exchange, removal, destruction and berry eating.
use super::*;

fn op(c: &Campaign, name: &str) -> Option<&'static str> {
    completion::definition(c, name)
        .filter(|(kind, _)| *kind == "item-op")
        .and_then(|(_, d)| d["op"].as_str())
}
/// Held units of a combatant, ordered by slot.
fn units(c: &Campaign, id: &str) -> Vec<String> {
    if !items::enabled(c) {
        return vec![];
    }
    let mut v: Vec<_> = items::state(c)
        .units
        .iter()
        .filter(|(_, u)| u.holder == id && u.state == "held")
        .map(|(k, u)| (u.slot, k.clone()))
        .collect();
    v.sort();
    v.into_iter().map(|(_, k)| k).collect()
}
fn berry(c: &Campaign, unit_id: &str, cat: &Catalog) -> Result<Option<serde_json::Value>> {
    let unit = items::state(c).units[unit_id].unit.clone();
    let item = items::definition(cat, &unit)?;
    Ok((item["Category"] == "Berry").then_some(item))
}
fn sticky(c: &Campaign, id: &str) -> Result<bool> {
    let p = fighter(c, id)?;
    Ok(p.has_ability("Sticky Hold") && !house::ring(c, p))
}
/// Validate both holders' final equipment, like ability thefts do.
fn validate(c: &Campaign, ids: &[&str], cat: &Catalog) -> Result<()> {
    for id in ids {
        let p = fighter(c, id)?;
        let mut sheet = c.sheets[&p.sheet_id].clone();
        sheet.ability = p.selected_ability.clone();
        sheet.held_items = Some(items::held(c, id));
        items::validate_equipment(&sheet, cat)
            .map_err(|_| error("The recipient cannot hold another item"))?;
    }
    Ok(())
}
fn give(c: &mut Campaign, unit_id: &str, from: &str, to: &str, kind: &str, why: &str) {
    let unit = items::state_mut(c).units.get_mut(unit_id).unwrap();
    unit.holder = to.into();
    unit.slot = 0;
    items::record(c, unit_id, from, Some(to.into()), kind, why);
}
fn drop(c: &mut Campaign, unit_id: &str, from: &str, state: &str, why: &str) {
    items::state_mut(c).units.get_mut(unit_id).unwrap().state = state.into();
    items::record(
        c,
        unit_id,
        from,
        None,
        if state == "consumed" {
            "destroyed"
        } else {
            "knocked_off"
        },
        why,
    );
}

/// Knock Off (+2) and Pluck (+1) add damage dice against an item or berry holder.
pub fn damage_bonus(c: &Campaign, target: &Combatant, m: &serde_json::Value, cat: &Catalog) -> i32 {
    match op(c, m["Name"].as_str().unwrap_or("")) {
        Some("knock-off") if !units(c, &target.id).is_empty() => 2,
        Some("pluck")
            if units(c, &target.id)
                .iter()
                .any(|u| berry(c, u, cat).is_ok_and(|b| b.is_some())) =>
        {
            1
        }
        _ => 0,
    }
}
/// Embargo: the target's trainer and partners cannot use items on it this scene.
pub fn embargoed(c: &Campaign, id: &str) -> bool {
    flow(c).is_ok_and(|f| f.completion.embargoed.contains(id))
}
/// Bestow needs an item to give; Thief/Covet need an empty hand to fill.
pub fn guard(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> Result<()> {
    if op(c, m["Name"].as_str().unwrap_or("")) == Some("bestow") && units(c, &p.id).is_empty() {
        return Err(error("Bestow requires the user to hold an item"));
    }
    Ok(())
}

pub fn effect(c: &mut Campaign, q: &QueuedAction, cat: &Catalog) -> Result<()> {
    if q.ice || !items::enabled(c) {
        return Ok(());
    }
    let (a, t) = (q.actor_id.clone(), q.target_id.clone());
    let Some(op) = op(c, &q.move_name) else {
        return Ok(());
    };
    let damaging = ![
        "bestow",
        "trick",
        "corrosive-gas",
        "embargo",
        "stuff-cheeks",
        "teatime",
    ]
    .contains(&op);
    if op != "stuff-cheeks" && completion::immune_target(c, q, cat)? {
        return Ok(());
    }
    if damaging && q.damage <= 0 {
        return Ok(());
    }
    let why = format!("{} effect", q.move_name);
    match op {
        "steal" => {
            let stolen = units(c, &t);
            if units(c, &a).is_empty() && !stolen.is_empty() && !sticky(c, &t)? {
                give(c, &stolen[0], &t, &a, "move_theft", &why);
                validate(c, &[&a, &t], cat)?;
                items::after_loss(c, &t, cat)?;
            }
        }
        "bestow" => {
            let mine = units(c, &a);
            if let Some(unit) = mine.first() {
                give(c, unit, &a, &t, "bestow", &why);
                validate(c, &[&a, &t], cat)?;
                items::after_loss(c, &a, cat)?;
            }
        }
        "trick" => {
            if sticky(c, &t)? {
                return Ok(());
            }
            let (mine, theirs) = (units(c, &a), units(c, &t));
            if let Some(unit) = mine.first() {
                give(c, unit, &a, &t, "trick", &why);
            }
            if let Some(unit) = theirs.first() {
                give(c, unit, &t, &a, "trick", &why);
            }
            validate(c, &[&a, &t], cat)?;
            if theirs.is_empty() && !mine.is_empty() {
                items::after_loss(c, &a, cat)?;
            }
            if mine.is_empty() && !theirs.is_empty() {
                items::after_loss(c, &t, cat)?;
            }
        }
        "knock-off" | "corrosive-gas" | "embargo" => {
            if !sticky(c, &t)? {
                for unit in units(c, &t) {
                    drop(c, &unit, &t, "ground", &why);
                }
                items::after_loss(c, &t, cat)?;
            }
            if op == "embargo" {
                flow_mut(c).completion.embargoed.insert(t);
            }
        }
        "incinerate" => {
            let held = units(c, &t);
            for unit in &held {
                drop(c, unit, &t, "consumed", &why);
            }
            if !held.is_empty() {
                items::after_loss(c, &t, cat)?;
            }
        }
        "eat-berry" | "pluck" => {
            let berries: Vec<String> = units(c, &t)
                .into_iter()
                .filter(|u| berry(c, u, cat).is_ok_and(|b| b.is_some()))
                .collect();
            if let Some(unit) = berries.first() {
                let item = berry(c, unit, cat)?.unwrap();
                let held = items::state(c).units[unit].unit.clone();
                items::consumable_effects(c, &a, &a, &item, 0, false)?;
                items::grant_chew(c, &a, &held, "eaten")?;
                items::state_mut(c).units.get_mut(unit).unwrap().state = "consumed".into();
                items::record(c, unit, &t, Some(a.clone()), "berry_hit", &why);
                items::after_loss(c, &t, cat)?;
            }
        }
        "stuff-cheeks" => {
            let berries: Vec<String> = units(c, &a)
                .into_iter()
                .filter(|u| berry(c, u, cat).is_ok_and(|b| b.is_some()))
                .collect();
            if let Some(unit) = berries.first() {
                // The berry's effect applies, but it stays held and can still be used.
                let item = berry(c, unit, cat)?.unwrap();
                items::consumable_effects(c, &a, &a, &item, 0, true)?;
            }
            extra::change_stat(c, &a, &a, "Defense", 2)?;
        }
        "teatime" => {
            let berries: Vec<String> = units(c, &t)
                .into_iter()
                .filter(|u| berry(c, u, cat).is_ok_and(|b| b.is_some()))
                .collect();
            for unit in &berries {
                items::state_mut(c).units.get_mut(unit).unwrap().state = "consumed".into();
                items::record(c, unit, &t, None, "teatime", "Eaten without effect");
            }
            if !berries.is_empty() {
                items::after_loss(c, &t, cat)?;
            }
        }
        _ => {}
    }
    Ok(())
}
