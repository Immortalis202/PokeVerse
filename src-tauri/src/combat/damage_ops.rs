//! MOVE-A50 damage-calculation cards: chosen types and categories, borrowed attributes,
//! dual-type effectiveness, random retyping and Conversion 2.
use super::*;

const TYPES: [&str; 18] = [
    "Normal", "Fire", "Water", "Electric", "Grass", "Ice", "Fight", "Poison", "Ground", "Flying",
    "Psychic", "Bug", "Rock", "Ghost", "Dragon", "Dark", "Steel", "Fairy",
];
const DRIVES: [(&str, &str); 4] = [
    ("Burn Drive", "Fire"),
    ("Chill Drive", "Ice"),
    ("Douse Drive", "Water"),
    ("Shock Drive", "Electric"),
];

fn op(c: &Campaign, name: &str) -> Option<&'static str> {
    completion::definition(c, name)
        .filter(|(kind, _)| *kind == "damage-op")
        .and_then(|(_, d)| d["op"].as_str())
}
/// Photon Geyser, Tera Blast and Tera Starstorm print a Physical/Special category.
fn category_choice(c: &Campaign, name: &str) -> bool {
    completion::definition(c, name)
        .is_some_and(|(kind, d)| kind == "damage-op" && d["category_choice"] == true)
}
fn name(m: &serde_json::Value) -> &str {
    m["Name"].as_str().unwrap_or("")
}
/// The saved choice for `p`'s queued cast of `move_name`, if any.
fn choice<'a>(c: &'a Campaign, p: &str, move_name: &str) -> Option<&'a String> {
    let f = flow(c).ok()?;
    let q = f
        .queue
        .iter()
        .find(|q| q.actor_id == p && q.move_name == move_name)?;
    f.completion.move_options.get(&q.sequence)
}

pub fn move_data(c: &Campaign, p: &Combatant, m: &mut serde_json::Value) {
    let move_name = name(m).to_string();
    if category_choice(c, &move_name) {
        if let Some(cat) = choice(c, &p.id, &move_name) {
            let attribute = if cat == "Physical" {
                "Strength"
            } else {
                "Special"
            };
            m["Category"] = cat.clone().into();
            m["Damage1"] = attribute.into();
        }
    }
    match op(c, &move_name) {
        Some("hidden-power") => {
            if let Some(t) = choice(c, &p.id, &move_name).filter(|t| TYPES.contains(&t.as_str())) {
                m["Type"] = t.clone().into();
            }
        }
        Some("techno-blast") => {
            let sheet = &c.sheets[&p.sheet_id];
            let genesect = sheet
                .species_id
                .as_deref()
                .is_some_and(|s| s.starts_with("0649:"));
            if genesect {
                if let Some((_, t)) = DRIVES
                    .iter()
                    .find(|(item, _)| items::completion::has(c, &p.id, item))
                {
                    m["Type"] = (*t).into();
                }
            }
        }
        _ => {}
    }
}
/// Hidden Power's type and the Physical/Special cards' category are chosen per cast.
pub fn options(c: &Campaign, q: &QueuedAction) -> Result<Vec<String>> {
    if q.damage_done {
        return Ok(vec![]);
    }
    if category_choice(c, &q.move_name) {
        return Ok(vec!["Physical".into(), "Special".into()]);
    }
    Ok(match op(c, &q.move_name) {
        Some("hidden-power") => TYPES.iter().map(|t| t.to_string()).collect(),
        _ => vec![],
    })
}
pub fn damage_guard(c: &Campaign, m: &serde_json::Value) -> Result<()> {
    match op(c, name(m)) {
        Some("hidden-power") if m["Type"] == "Varies" => Err(error(
            "Save Hidden Power's type for this cast with the Save choice control first",
        )),
        _ if category_choice(c, name(m))
            && m["Category"].as_str().is_some_and(|v| v.contains('/')) =>
        {
            Err(error(
                "Save Physical or Special for this cast with the Save choice control first",
            ))
        }
        _ => Ok(()),
    }
}
/// Hidden Power is always super-effective; Flying Press adds the Flying matchup.
pub fn effectiveness(
    c: &Campaign,
    m: &serde_json::Value,
    target: &Combatant,
    cat: &Catalog,
    n: i32,
) -> Result<i32> {
    Ok(match op(c, name(m)) {
        Some("hidden-power") => 1,
        Some("dual-type") => {
            let mut total = n;
            for t in field::types(c, target, cat)? {
                let effect = rules::type_effect(m["DualType"].as_str().unwrap_or("Flying"), &t)?;
                if effect != -100 {
                    total += effect;
                }
            }
            total
        }
        _ => n,
    })
}
/// Hidden Power's guaranteed super-effective damage overrides type immunity.
pub fn ignores_immunity(c: &Campaign, m: &serde_json::Value) -> bool {
    op(c, name(m)) == Some("hidden-power")
}
/// Foul Play's damage pool uses the target's attributes.
pub fn borrowed_attribute(c: &Campaign, m: &serde_json::Value) -> bool {
    op(c, name(m)) == Some("foul-play")
}
/// Unown's entire move pool is Hidden Power, so it may repeat it within a round (p.530).
pub fn repeat(c: &Campaign, id: &str, move_name: &str) -> bool {
    op(c, move_name) == Some("hidden-power")
        && fighter(c, id).is_ok_and(|p| {
            c.sheets[&p.sheet_id]
                .species_id
                .as_deref()
                .is_some_and(|s| s.starts_with("0201:"))
        })
}
pub fn effect_dice(c: &Campaign, q: &QueuedAction) -> Option<u32> {
    let op = op(c, &q.move_name)?;
    Some(if op == "retype" && !q.ice { 2 } else { 0 })
}
fn retype(c: &mut Campaign, id: &str, types: &[String]) -> Result<()> {
    field::change_type(c, id, "replace", types)?;
    let scope = format!("combatant:{id}");
    for r in &mut flow_mut(c).battlefield.rules {
        if r.scope == scope && r.kind.starts_with("typing:") && r.source == "typing" {
            r.source = "completion-type".into();
        }
    }
    Ok(())
}
pub fn effect(c: &mut Campaign, q: &QueuedAction, dice: &[u8], cat: &Catalog) -> Result<()> {
    if q.ice {
        return Ok(());
    }
    match op(c, &q.move_name).unwrap_or("") {
        "retype" => {
            if !completion::immune_target(c, q, cat)? && q.damage > 0 {
                let value = (dice[0] as usize - 1) * 6 + (dice[1] as usize - 1);
                retype(c, &q.target_id, &[TYPES[value % TYPES.len()].to_string()])?;
            }
        }
        "conversion-2" => {
            let started = battle(c)?.round;
            let f = flow_mut(c);
            f.special
                .effects
                .retain(|e| e.name != "Conversion 2" || e.target != q.actor_id);
            f.special.effects.push(special::TimedEffect {
                name: "Conversion 2".into(),
                source: q.actor_id.clone(),
                target: q.actor_id.clone(),
                rounds: None,
                amount: 0,
                started,
                slot: "type-op".into(),
            });
        }
        _ => {}
    }
    flow_mut(c).completion.move_options.remove(&q.sequence);
    Ok(())
}
/// Conversion 2 offers the owner every type that resists the damaging move's type.
pub fn after_damage(
    c: &mut Campaign,
    target: &str,
    m: &serde_json::Value,
    damage: i32,
) -> Result<()> {
    if damage <= 0
        || !["Physical", "Special"].contains(&m["Category"].as_str().unwrap_or(""))
        || op(c, "Conversion 2").is_none()
        || !flow(c)?
            .special
            .effects
            .iter()
            .any(|e| e.name == "Conversion 2" && e.target == target)
    {
        return Ok(());
    }
    let move_type = m["Type"].as_str().unwrap_or("Normal");
    let resists: Vec<String> = TYPES
        .iter()
        .filter(|t| rules::type_effect(move_type, t).is_ok_and(|e| e < 0))
        .map(|t| t.to_string())
        .collect();
    if resists.is_empty() {
        return Ok(());
    }
    flow_mut(c)
        .completion
        .conversion_choices
        .insert(target.into(), resists);
    Ok(())
}
/// Apply a pending Conversion 2 choice: the main type changes and the secondary is kept.
pub fn convert(c: &mut Campaign, id: &str, type_name: &str, cat: &Catalog) -> Result<()> {
    let options = flow(c)?
        .completion
        .conversion_choices
        .get(id)
        .cloned()
        .ok_or_else(|| error("No pending Conversion 2 choice for this combatant"))?;
    if !options.iter().any(|t| t == type_name) {
        return Err(error(&format!("Choose one of: {}", options.join(", "))));
    }
    let current = field::types(c, fighter(c, id)?, cat)?;
    let mut types = vec![type_name.to_string()];
    if let Some(second) = current.get(1).filter(|s| *s != type_name) {
        types.push(second.clone());
    }
    retype(c, id, &types)?;
    flow_mut(c).completion.conversion_choices.remove(id);
    Ok(())
}
pub fn lifecycle(c: &mut Campaign) -> Result<()> {
    let active: BTreeSet<String> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    flow_mut(c)
        .completion
        .conversion_choices
        .retain(|id, _| active.contains(id));
    Ok(())
}
