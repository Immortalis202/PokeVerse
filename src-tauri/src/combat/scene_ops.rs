//! MOVE-A58 Memento, Springtide Storm, Synchronoise, Pay Day, Make It Rain, Camouflage
//! and Nature Power.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct MoneyAward {
    pub trainer_sheet: String,
    pub amount: i32,
    pub move_name: String,
}

const TYPES: [&str; 18] = [
    "Normal", "Fire", "Water", "Electric", "Grass", "Ice", "Fight", "Poison", "Ground", "Flying",
    "Psychic", "Bug", "Rock", "Ghost", "Dragon", "Dark", "Steel", "Fairy",
];
const MEMENTO: [&str; 5] = [
    "Strength",
    "Dexterity",
    "Special",
    "Defense",
    "SpecialDefense",
];

fn op(c: &Campaign, name: &str) -> Option<&'static str> {
    completion::definition(c, name)
        .filter(|(kind, _)| *kind == "scene-op")
        .and_then(|(_, d)| d["op"].as_str())
}
fn saved(c: &Campaign, sequence: u32) -> Option<&String> {
    flow(c).ok()?.completion.move_options.get(&sequence)
}
fn queued<'a>(c: &'a Campaign, actor: &str, name: &str) -> Option<&'a QueuedAction> {
    flow(c)
        .ok()?
        .queue
        .iter()
        .find(|q| q.actor_id == actor && q.move_name == name)
}

// ----- Environment energy (corebook p.530) -----

fn energy(source: &str) -> &'static [&'static str] {
    match source {
        "City/Building" => &["Electric", "Steel"],
        "Plains/Grasslands" | "Grassy Terrain" => &["Grass"],
        "Forest/Jungle" => &["Bug", "Poison"],
        "Lake/River/Sea" => &["Water"],
        "Cave/Mountain" => &["Fight", "Rock"],
        "Desert/Volcano" => &["Fire", "Ground"],
        "Ice/Snow" => &["Ice"],
        "Electric Terrain" => &["Electric"],
        "Misty Terrain" => &["Fairy"],
        "Psychic Terrain" => &["Psychic"],
        _ => &[],
    }
}
/// Types the user may align with: every recorded surrounding and active terrain.
fn energies(c: &Campaign, id: &str) -> Vec<String> {
    let mut types: Vec<String> = compound::choices(c, id)
        .iter()
        .flat_map(|s| energy(s).iter().map(|t| t.to_string()))
        .collect();
    types.sort();
    types.dedup();
    types
}
fn aligned_type(c: &Campaign, q: &QueuedAction) -> Option<String> {
    let types = energies(c, &q.actor_id);
    match types.len() {
        0 => None,
        1 => Some(types[0].clone()),
        _ => saved(c, q.sequence).filter(|t| types.contains(t)).cloned(),
    }
}

// ----- Choices, move data and damage -----

pub fn options(c: &Campaign, q: &QueuedAction) -> Result<Vec<String>> {
    Ok(match op(c, &q.move_name) {
        Some("springtide") => vec!["Raise the user".into(), "Lower the targets".into()],
        Some("synchronoise") if !q.damage_done => TYPES.iter().map(|t| t.to_string()).collect(),
        Some("nature-power") if !q.damage_done => {
            let types = energies(c, &q.actor_id);
            if types.len() > 1 {
                types
            } else {
                vec![]
            }
        }
        Some("camouflage") => {
            let types = energies(c, &q.actor_id);
            if types.len() > 1 {
                types
            } else {
                vec![]
            }
        }
        _ => vec![],
    })
}
pub fn move_data(c: &Campaign, p: &Combatant, m: &mut serde_json::Value) {
    let name = m["Name"].as_str().unwrap_or("").to_string();
    if op(c, &name) != Some("nature-power") {
        return;
    }
    if let Some(t) = queued(c, &p.id, &name).and_then(|q| aligned_type(c, q)) {
        m["Type"] = t.into();
    }
}
pub fn damage_guard(c: &Campaign, key: &str, sequence: u32, m: &serde_json::Value) -> Result<()> {
    let name = m["Name"].as_str().unwrap_or("");
    if op(c, name).is_none() {
        return Ok(());
    }
    let q = flow(c)?.queue.iter().find(|q| q.sequence == sequence);
    match op(c, name) {
        Some("synchronoise") if saved(c, sequence).is_none() => Err(error(
            "Save Synchronoise's chosen type with the Save choice control first",
        )),
        Some("nature-power") if q.is_some_and(|q| aligned_type(c, q).is_none()) => {
            if energies(c, key).is_empty() {
                Err(AppError::new(
                    "manual_rule",
                    "Nature Power needs the GM to record the surroundings (or an active Terrain)",
                ))
            } else {
                Err(error("Save which energy Nature Power aligns with first"))
            }
        }
        _ => Ok(()),
    }
}
/// Synchronoise only affects targets that have the chosen type.
pub fn synchronoise_immune(
    c: &Campaign,
    sequence: u32,
    target: &Combatant,
    m: &serde_json::Value,
    cat: &Catalog,
) -> bool {
    op(c, m["Name"].as_str().unwrap_or("")) == Some("synchronoise")
        && saved(c, sequence).is_some_and(|t| {
            !field::types(c, target, cat).is_ok_and(|types| types.iter().any(|v| v == t))
        })
}

// ----- Memento -----

/// Memento's reductions persist across switching for the scene.
pub fn memento_values(c: &Campaign, id: &str, key: &str) -> Vec<i32> {
    flow(c).map_or_else(
        |_| vec![],
        |f| {
            f.completion
                .memento
                .get(id)
                .filter(|keys| keys.iter().any(|k| k == key))
                .map_or_else(Vec::new, |_| vec![-2])
        },
    )
}
/// Memento's user faints even when its Accuracy roll fails.
pub fn accuracy(c: &mut Campaign, id: &str, name: &str, success: bool) -> Result<()> {
    if !success && op(c, name) == Some("memento") {
        fighter_mut(c, id)?.hp = 0;
    }
    Ok(())
}

// ----- Money -----

fn rank_money(rank: &str) -> i32 {
    match rank {
        "Starter" | "Rookie" => 100,
        "Standard" => 250,
        "Advanced" => 500,
        _ => 1000,
    }
}
fn award(c: &mut Campaign, pokemon: &str, name: &str) -> Result<()> {
    let sheet = &c.sheets[&fighter(c, pokemon)?.sheet_id];
    let Some(trainer) = sheet
        .trainer_id
        .clone()
        .filter(|t| c.sheets.contains_key(t))
    else {
        return Ok(());
    };
    let amount = rank_money(&c.sheets[&trainer].rank);
    let f = flow_mut(c);
    if name == "Make It Rain"
        && f.completion
            .money
            .iter()
            .any(|m| m.trainer_sheet == trainer && m.move_name == name)
    {
        return Ok(());
    }
    f.completion.money.push(MoneyAward {
        trainer_sheet: trainer,
        amount,
        move_name: name.into(),
    });
    Ok(())
}
/// Battle-end receipt for Pay Day and Make It Rain: the app does not track money.
pub fn end_log(c: &Campaign) -> Option<String> {
    let awards = &flow(c).ok()?.completion.money;
    if awards.is_empty() {
        return None;
    }
    let mut totals: BTreeMap<&str, i32> = BTreeMap::new();
    for a in awards {
        *totals.entry(&a.trainer_sheet).or_default() += a.amount;
    }
    Some(format!(
        "Money from Pay Day/Make It Rain (record it manually): {}",
        totals
            .iter()
            .map(|(id, n)| format!("{} {n}", c.sheets.get(*id).map_or(*id, |s| s.name.as_str())))
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

// ----- Effects -----

pub fn effect_dice(c: &Campaign, q: &QueuedAction) -> Option<u32> {
    let op = op(c, &q.move_name)?;
    if q.ice || op != "springtide" {
        return Some(0);
    }
    let source = fighter(c, &q.actor_id).ok()?;
    Some(3 + 2 * u32::from(source.has_ability("Serene Grace")) + field_ops::water_pledge(c))
}
pub fn effect(c: &mut Campaign, q: &QueuedAction, dice: &[u8], cat: &Catalog) -> Result<()> {
    if q.ice {
        return Ok(());
    }
    let (a, t) = (q.actor_id.clone(), q.target_id.clone());
    match op(c, &q.move_name).unwrap_or("") {
        "memento" => {
            if !completion::immune_target(c, q, cat)? {
                let target = fighter(c, &t)?.clone();
                let mut lowered = vec![];
                for key in MEMENTO {
                    let before = special::modifier(c, &target, key).unwrap_or(0);
                    extra::change_stat(c, &a, &t, key, -2)?;
                    if special::modifier(c, fighter(c, &t)?, key).unwrap_or(0) < before {
                        lowered.push(key.to_string());
                    }
                }
                if !lowered.is_empty() {
                    flow_mut(c).completion.memento.insert(t, lowered);
                }
            }
            fighter_mut(c, &a)?.hp = 0;
        }
        "springtide" => {
            let choice = saved(c, q.sequence).cloned().ok_or_else(|| {
                error("Choose whether Springtide Storm raises the user or lowers the targets")
            })?;
            if dice.contains(&6) {
                if choice == "Raise the user" {
                    for key in MEMENTO {
                        extra::change_stat(c, &a, &a, key, 1)?;
                    }
                } else if !completion::immune_target(c, q, cat)? {
                    extra::change_stat(c, &a, &t, "Defense", -2)?;
                    extra::change_stat(c, &a, &t, "SpecialDefense", -2)?;
                }
            }
        }
        "pay-day" => {
            if q.damage > 0 {
                award(c, &a, &q.move_name)?;
            }
        }
        "make-it-rain" => {
            extra::change_stat(c, &a, &a, "Special", -2)?;
            award(c, &a, &q.move_name)?;
            if q.damage > 0 {
                award(c, &t, &q.move_name)?;
            }
        }
        "camouflage" => {
            let types = energies(c, &a);
            let chosen =
                match types.len() {
                    0 => return Err(AppError::new(
                        "manual_rule",
                        "Camouflage needs the GM to record the surroundings (or an active Terrain)",
                    )),
                    1 => types[0].clone(),
                    _ => saved(c, q.sequence)
                        .filter(|t| types.contains(t))
                        .cloned()
                        .ok_or_else(|| error("Choose which surrounding type Camouflage takes"))?,
                };
            field::change_type(c, &a, "replace", &[chosen])?;
            let scope = format!("combatant:{a}");
            for r in &mut flow_mut(c).battlefield.rules {
                if r.scope == scope && r.kind.starts_with("typing:") && r.source == "typing" {
                    r.source = "completion-type".into();
                }
            }
        }
        _ => {}
    }
    flow_mut(c).completion.move_options.remove(&q.sequence);
    Ok(())
}
