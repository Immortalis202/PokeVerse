//! MASTER 549,553,556,559,564,574 and the supplied v3.0 ability/move cards.
use super::*;
const ATTRIBUTES: [&str; 5] = ["Strength", "Dexterity", "Vitality", "Special", "Insight"];
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct StandaloneState {
    pub enabled: bool,
    pub profiles: BTreeMap<String, BattleProfile>,
    pub rivalry_basis: Option<String>,
    pub torrent_bonus: Option<i32>,
    pub perish: BTreeMap<String, PerishCountdown>,
    pub ended_round: u32,
    pub history: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct BattleProfile {
    pub species_id: Option<String>,
    pub gender: String,
    pub preference: String,
    pub weight_kg: Option<f64>,
    pub metal_weight_kg: Option<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct PerishCountdown {
    pub source_id: String,
    pub due_round: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "operation", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/bindings/")]
pub enum StandaloneCommand {
    Profile {
        combatant_id: String,
        gender: String,
        preference: String,
        weight_kg: Option<f64>,
        metal_weight_kg: Option<f64>,
    },
    RivalryBasis {
        basis: String,
    },
    Torrent {
        bonus: i32,
    },
}
pub fn enabled(c: &Campaign) -> bool {
    c.battle
        .as_ref()
        .and_then(|b| b.flow.as_ref())
        .is_some_and(|f| f.standalone.enabled)
}
pub fn torrent_choice(c: &Campaign) -> Option<i32> {
    c.battle.as_ref()?.flow.as_ref()?.standalone.torrent_bonus
}
pub fn torrent(
    c: &Campaign,
    p: &Combatant,
    m: &serde_json::Value,
    hp: i32,
    saved: Option<i32>,
) -> Result<i32> {
    if !enabled(c)
        || !p.has_ability("Torrent")
        || m["Type"] != "Water"
        || m["Category"] == "Support"
    {
        return Ok(0);
    }
    // A shared attack saves the effective bonus, including zero above the threshold.
    // Later form/max-HP changes must not reclassify an already saved attack.
    if let Some(n) = saved {
        return Ok(n);
    }
    if hp > p.max_hp / 2 {
        return Ok(0);
    }
    torrent_choice(c).ok_or_else(|| {
        AppError::new(
            "manual_rule",
            "Torrent text grants two dice but icon one: GM must record the encounter bonus",
        )
    })
}

pub fn rivalry(c: &Campaign, a: &Combatant, t: &Combatant) -> Result<i32> {
    if !enabled(c) || !a.has_ability("Rivalry") || a.side == t.side {
        return Ok(0);
    }
    let state = &flow(c)?.standalone;
    let basis = state.rivalry_basis.as_deref().ok_or_else(|| {
        AppError::new(
            "manual_rule",
            "Rivalry: GM must choose supplied-card gender or printed preference wording",
        )
    })?;
    let group = |id: &str| {
        state
            .profiles
            .get(id)
            .map(|p| {
                if basis == "gender" {
                    p.gender.as_str()
                } else {
                    p.preference.as_str()
                }
            })
            .unwrap_or("unknown")
    };
    let (a, t) = (group(&a.id), group(&t.id));
    if !["male", "female"].contains(&a) || !["male", "female"].contains(&t) {
        return Ok(0);
    }
    Ok(if a == t { 1 } else { -1 })
}
fn attribute_change(c: &Campaign, a: &Combatant, change: i32) -> i32 {
    let change = match a.ability_for(&["Contrary", "Simple"]) {
        "Contrary" => -change,
        "Simple" => change * 2,
        _ => change,
    };
    let (up, down) = stat_channels::channel(c, a, "Strength");
    if change > 0 {
        (change - up).max(0)
    } else {
        (change - down).min(0)
    }
}
pub fn rivalry_snapshot(c: &Campaign, a: &Combatant) -> Result<BTreeMap<String, i32>> {
    if !enabled(c) || !a.has_ability("Rivalry") {
        return Ok(BTreeMap::new());
    }
    let strength = rules::score(&rules::effective_sheet(c, a)?, "Strength")?;
    battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed)
        .map(|t| {
            Ok((
                t.id.clone(),
                strength_delta(c, a, t, "Strength", strength, None)?,
            ))
        })
        .collect()
}
pub fn strength_delta(
    c: &Campaign,
    a: &Combatant,
    t: &Combatant,
    key: &str,
    base: i32,
    saved: Option<i32>,
) -> Result<i32> {
    if key != "Strength" || provenance::ignores_attributes(c, t, a) {
        return Ok(0);
    }
    if let Some(n) = saved {
        return Ok(n);
    }
    let change = attribute_change(c, a, rivalry(c, a, t)?);
    if change == 0 {
        return Ok(0);
    }
    Ok((base + change).clamp(1, forms::stat_cap(c, a)) - base)
}

pub fn accuracy_delta(
    c: &Campaign,
    a: &Combatant,
    t: &Combatant,
    m: &serde_json::Value,
) -> Result<i32> {
    if !enabled(c) {
        return Ok(0);
    }
    let key = m["Accuracy1"].as_str().unwrap_or("");
    if !key.split('/').any(|k| k.trim() == "Strength") {
        return Ok(0);
    }
    let mut sheet = rules::effective_sheet(c, a)?;
    let before = rules::score(&sheet, key)?;
    let value = sheet
        .stats
        .get_mut("Strength")
        .ok_or_else(|| error("Missing Strength"))?;
    *value = (*value + attribute_change(c, a, rivalry(c, a, t)?)).clamp(1, forms::stat_cap(c, a));
    Ok(rules::score(&sheet, key)? - before)
}
pub fn accuracy_plan(
    c: &Campaign,
    action: &Action,
    cat: &Catalog,
) -> Result<(u32, BTreeMap<String, u32>)> {
    let (id, name, modifier) = match action {
        Action::Accuracy {
            combatant_id,
            move_name,
            pool_modifier,
            ..
        }
        | Action::AccuracyTargets {
            combatant_id,
            move_name,
            pool_modifier,
            ..
        } => (combatant_id, move_name, *pool_modifier),
        _ => return Err(error("Expected accuracy action")),
    };
    let a = fighter(c, id)?;
    let (base, m) = rules::accuracy(&old(c), id, name, modifier, cat)?;
    if !enabled(c) {
        return Ok((base, BTreeMap::new()));
    }
    let (targets, _) = plan(c, id, &selected(action), &m)?;
    let targets = redirect(
        c,
        id,
        targets,
        interactions::cover_bypass(c, id, &m)
            || m["Attributes"]["SoundMove"] == true
            || ["Phantom Force", "Shadow Force"].contains(&name.as_str()),
    );
    let targets = tactics::redirect_move(c, id, targets, &m, cat)?;
    let mut pools = BTreeMap::new();
    for id in targets {
        let n = base as i32 + accuracy_delta(c, a, special::registered(c, &id)?, &m)?;
        if !(1..=100).contains(&n) {
            return Err(error(
                "Rivalry accuracy requires GM adjudication outside 1-100 dice",
            ));
        }
        pools.insert(id, n as u32);
    }
    Ok((pools.values().copied().max().unwrap_or(base), pools))
}
pub fn accuracy_success(
    c: &Campaign,
    p: &Combatant,
    dice: &[u8],
    base: u32,
    shared: u32,
    exempt: bool,
) -> i32 {
    let values = &dice[..base as usize];
    let chance = p.runtime.chance_round == battle(c).unwrap().round
        && values.iter().any(|v| *v < 4)
        && dice.get(shared as usize).is_some_and(|v| *v >= 4);
    (rules::successes(values) + i32::from(chance)
        - confusion(c, p)
        - if exempt {
            0
        } else {
            (rules::pain(p.hp, p.max_hp) - p.runtime.pain_ignored).max(0)
        })
    .max(0)
}

pub fn weight(c: &Campaign, p: &Combatant, cat: &Catalog, legacy: Option<f64>) -> Result<f64> {
    let s = rules::base_sheet(c, p)?;
    let profile = flow(c)?.standalone.profiles.get(&p.id);
    let specific = profile.filter(|v| v.species_id == s.species_id);
    let baseline = specific
        .and_then(|v| v.weight_kg)
        .or(legacy)
        .or_else(|| {
            cat.find("species", "AppId", s.species_id.as_deref().unwrap_or(""))
                .and_then(|v| v["Weight"]["Kilograms"].as_f64())
        })
        .filter(|n| n.is_finite() && *n > 0.)
        .ok_or_else(|| AppError::new("manual_rule", "GM must record an encounter weight in kg"))?;
    if p.has_any_ability(&["Heavy Metal", "Light Metal"]) {
        let n = specific.and_then(|v| v.metal_weight_kg).ok_or_else(|| {
            AppError::new(
                "manual_rule",
                "Heavy/Light Metal requires a GM-recorded weight for the exact current form",
            )
        })?;
        if (p.has_ability("Heavy Metal") && n <= baseline)
            || (p.has_ability("Light Metal") && n >= baseline)
        {
            return Err(error(
                "Metal weight must follow the printed increase/reduction direction",
            ));
        }
        return Ok(n);
    }
    Ok(baseline)
}
pub fn transition(
    c: &Campaign,
    actor: &str,
    command: &StandaloneCommand,
    why: &str,
    dice: &[u8],
    cat: &Catalog,
) -> Result<(Campaign, String)> {
    gm(c, actor)?;
    reason(why)?;
    check_count(dice, 0)?;
    if !enabled(c) || !battle(c)?.active {
        return Err(error("Standalone rules require a new active encounter"));
    }
    let mut next = c.clone();
    match command {
        StandaloneCommand::Profile {
            combatant_id,
            gender,
            preference,
            weight_kg,
            metal_weight_kg,
        } => {
            let p = special::registered(c, combatant_id)?;
            if !["male", "female", "genderless", "unknown"].contains(&gender.as_str())
                || !["male", "female", "neutral", "unknown"].contains(&preference.as_str())
            {
                return Err(error(
                    "Choose explicit reviewed gender/comparison groups or unknown/neutral",
                ));
            }
            for n in [weight_kg, metal_weight_kg].into_iter().flatten() {
                if !n.is_finite() || *n <= 0. || *n > 100000. {
                    return Err(error(
                        "Weight must be finite and between zero and 100000 kg",
                    ));
                }
            }
            let profile = BattleProfile {
                species_id: rules::base_sheet(c, p)?.species_id,
                gender: gender.clone(),
                preference: preference.clone(),
                weight_kg: *weight_kg,
                metal_weight_kg: *metal_weight_kg,
            };
            flow_mut(&mut next)
                .standalone
                .profiles
                .insert(combatant_id.clone(), profile);
            if metal_weight_kg.is_some() && p.has_any_ability(&["Heavy Metal", "Light Metal"]) {
                weight(&next, p, cat, None)?;
            }
        }
        StandaloneCommand::RivalryBasis { basis } => {
            if !["gender", "preference"].contains(&basis.as_str()) {
                return Err(error("Choose JSON gender or printed preference wording"));
            }
            flow_mut(&mut next).standalone.rivalry_basis = Some(basis.clone());
        }
        StandaloneCommand::Torrent { bonus } => {
            if ![1, 2].contains(bonus) {
                return Err(error("Torrent's printed alternatives are one or two dice"));
            }
            flow_mut(&mut next).standalone.torrent_bonus = Some(*bonus);
        }
    }
    let log = format!("GM standalone ruling: {why}");
    flow_mut(&mut next)
        .standalone
        .history
        .push(format!("{command:?}: {why}"));
    refresh_initiative(c, &mut next)?;
    checkpoint(c, &mut next, &log);
    Ok((next, log))
}
pub fn hit(
    c: &mut Campaign,
    q: &QueuedAction,
    target: &Combatant,
    attacker: &Combatant,
    m: &serde_json::Value,
    before: i32,
    cat: &Catalog,
) -> Result<()> {
    if !enabled(c)
        || !target.has_ability("Perish Body")
        || before <= 0
        || attacker.hp <= 0
        || attacker.side == target.side
        || !abilities::contact(m)
        || decoys::protected_subject(c, &q.actor_id, &target.id, m)
        || reviewed::damage_bypass(c, &reviewed::source(c, q)?, target)
    {
        return Ok(());
    }
    let (effect, immune) =
        rules::type_matchup(c, attacker, target, m["Type"].as_str().unwrap_or(""), cat)?;
    if immune
        || abilities::damage_immune_from(c, attacker, target, m)
        || extra::source_immune(c, attacker, target, effect)
    {
        return Ok(());
    }
    let due = battle(c)?.round.saturating_add(3);
    if !flow(c)?.standalone.perish.contains_key(&attacker.id) {
        flow_mut(c).standalone.perish.insert(
            attacker.id.clone(),
            PerishCountdown {
                source_id: target.id.clone(),
                due_round: due,
            },
        );
        flow_mut(c).standalone.history.push(format!("Perish Body: {} cursed {}; remaining-HP damage at end of round {due}, canceled by departure",target.name,attacker.name));
    }
    Ok(())
}
pub fn lifecycle(c: &mut Campaign) -> Result<()> {
    if !enabled(c) || c.battle.as_ref().is_none_or(|b| !b.active) {
        return Ok(());
    }
    let active: BTreeSet<_> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    flow_mut(c)
        .standalone
        .perish
        .retain(|id, _| active.contains(id));
    let moody: BTreeSet<_> = battle(c)?
        .combatants
        .iter()
        .filter(|p| active.contains(&p.id) && p.has_ability("Moody"))
        .map(|p| p.id.clone())
        .collect();
    flow_mut(c)
        .stat_channels
        .grants
        .retain(|g| g.cause != "Moody" || g.source != g.target || moody.contains(&g.target));
    let stale: BTreeSet<_> = flow(c)?
        .ability_events
        .iter()
        .filter(|e| {
            e.kind == "standalone-moody"
                && (!active.contains(&e.combatant_id)
                    || !fighter(c, &e.combatant_id).is_ok_and(|p| p.has_ability("Moody")))
        })
        .map(|e| e.id.clone())
        .collect();
    flow_mut(c)
        .ability_events
        .retain(|e| !stale.contains(&e.id));
    Ok(())
}
pub fn end_round(c: &mut Campaign) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let round = battle(c)?.round;
    if flow(c)?.standalone.ended_round == round {
        return Ok(());
    }
    flow_mut(c).standalone.ended_round = round;
    lifecycle(c)?;
    let due: Vec<_> = flow(c)?
        .standalone
        .perish
        .iter()
        .filter(|(_, p)| p.due_round <= round)
        .map(|(id, _)| id.clone())
        .collect();
    for id in due {
        let p = fighter(c, &id)?.clone();
        let damage = if p.has_any_ability(&["Magic Guard", "Wonder Guard"]) {
            0
        } else {
            p.hp
        };
        fighter_mut(c, &id)?.hp -= damage;
        flow_mut(c).standalone.perish.remove(&id);
        flow_mut(c).standalone.history.push(format!(
            "Perish Body: {} received {damage} remaining-HP ability damage",
            p.name
        ));
    }
    let moody: Vec<_> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0 && p.has_ability("Moody"))
        .cloned()
        .collect();
    for p in moody {
        flow_mut(c)
            .stat_channels
            .grants
            .retain(|g| !(g.target == p.id && g.source == p.id && g.cause == "Moody"));
        let event_id = format!("moody:{}:{round}:{}", c.revision + 1, p.id);
        flow_mut(c).ability_events.push(abilities::AbilityEvent {
            id: event_id,
            combatant_id: p.id.clone(),
            target_id: p.id,
            ability: "Moody".into(),
            kind: "standalone-moody".into(),
            move_name: "round end; previous Moody channels reset".into(),
            choices: vec!["roll".into()],
        });
    }
    Ok(())
}
pub fn resolve_moody(
    c: &mut Campaign,
    e: &abilities::AbilityEvent,
    choice: &str,
    dice: &[u8],
) -> Result<bool> {
    if choice == "manual" {
        return Ok(true);
    }
    let p = fighter(c, &e.combatant_id)?.clone();
    if !enabled(c) || !p.deployed || p.hp <= 0 || !p.has_ability("Moody") {
        return Err(error("Moody is no longer eligible"));
    }
    check_count(dice, 2)?;
    let sample = (dice[0] as usize - 1) * 6 + dice[1] as usize - 1;
    if sample >= 20 {
        flow_mut(c).standalone.history.push(format!(
            "Moody: {} rejected d6 sample {dice:?}; pending reroll retained",
            p.name
        ));
        return Ok(false);
    }
    let down = sample / 4;
    let index = sample % 4;
    let up = if index >= down { index + 1 } else { index };
    for (key, n) in [(ATTRIBUTES[down], -1), (ATTRIBUTES[up], 1)] {
        let n = match p.ability_for(&["Contrary", "Simple"]) {
            "Contrary" => -n,
            "Simple" => n * 2,
            _ => n,
        };
        flow_mut(c)
            .stat_channels
            .grants
            .push(stat_channels::StatGrant {
                source: p.id.clone(),
                target: p.id.clone(),
                cause: "Moody".into(),
                key: key.into(),
                channel: "ability".into(),
                positive: n.max(0),
                negative: n.min(0),
            });
    }
    flow_mut(c).standalone.history.push(format!(
        "Moody: {} host dice {dice:?}; {} -1, {} +1 (before Contrary/Simple)",
        p.name, ATTRIBUTES[down], ATTRIBUTES[up]
    ));
    Ok(true)
}
