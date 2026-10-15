//! Pokerole 3.0 corebook: pp. 35–36, 56–57, 61–62, 79–80.
//! Base rolls only: abilities, status effects, Will and battlefield modifiers require GM input.
use crate::{catalog::Catalog, model::*};
use serde_json::Value;

pub fn successes(dice: &[u8]) -> i32 {
    dice.iter().filter(|&&d| d >= 4).count() as i32
}
pub fn pain(hp: i32, max_hp: i32) -> i32 {
    if hp == max_hp {
        0
    } else if hp <= 1 {
        2
    } else if hp <= max_hp / 2 {
        1
    } else {
        0
    }
}
pub fn effective_sheet(c: &Campaign, p: &Combatant) -> Result<Sheet> {
    let mut s = c
        .sheets
        .get(&p.sheet_id)
        .cloned()
        .ok_or_else(|| AppError::new("missing", "Missing combatant sheet"))?;
    s.stats.extend(p.stat_overrides.clone());
    crate::combat::apply_stats(c, p, &mut s);
    s.skills.extend(p.skill_overrides.clone());
    s.ability = p.selected_ability.clone();
    if let Some(b) = s.workflow.as_mut() {
        b.profile = p.rules_profile.clone();
    }
    Ok(s)
}
pub fn score(sheet: &Sheet, source: &str) -> Result<i32> {
    // A slash is an Attribute choice, not an addition (corebook p.57).
    source
        .split('/')
        .map(|key| {
            let key = key.trim();
            if key == "Will" {
                Ok(sheet.workflow.as_ref().map_or(0, |b| b.max_will))
            } else if ["Strength", "Dexterity", "Vitality", "Special", "Insight"].contains(&key)
                || crate::creation::SOCIALS.contains(&key)
            {
                Ok(*sheet.stats.get(key).unwrap_or(&0))
            } else if [
                "Brawl",
                "Channel",
                "Clash",
                "Evasion",
                "Alert",
                "Athletic",
                "Nature",
                "Stealth",
                "Allure",
                "Etiquette",
                "Intimidate",
                "Perform",
                "Crafts",
                "Lore",
                "Medicine",
                "Science",
                "Charm",
                "Empathy",
                "Throw",
                "Weapons",
            ]
            .contains(&key)
            {
                Ok(*sheet.skills.get(key).unwrap_or(&0))
            } else {
                Err(AppError::new(
                    "manual_rule",
                    format!("{key} requires manual adjudication"),
                ))
            }
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .max()
        .ok_or_else(|| AppError::new("manual_rule", "Missing roll formula"))
}
pub fn accuracy(
    c: &Campaign,
    key: &str,
    name: &str,
    modifier: i32,
    catalog: &Catalog,
) -> Result<(u32, Value)> {
    if !core_engine(&c.engine) {
        return Err(AppError::new(
            "version",
            "Upgrade this campaign between battles to enable core rolls",
        ));
    }
    if !(-20..=20).contains(&modifier) {
        return Err(AppError::new(
            "validation",
            "Pool modifier must be between -20 and 20",
        ));
    }
    let p = c
        .battle
        .as_ref()
        .filter(|b| b.active)
        .and_then(|b| b.combatants.iter().find(|p| p.id == key && p.deployed))
        .ok_or_else(|| AppError::new("missing", "Unknown active combatant"))?;
    crate::teams::ready(p, c.battle.as_ref().unwrap().round)?;
    let automatic = c.battle.as_ref().is_some_and(|b| b.automatic_validation);
    if p.actions >= 1000 {
        return Err(AppError::new("limit", "Action counter limit reached"));
    }
    if automatic && (p.hp == 0 || p.actions >= 5) {
        return Err(AppError::new(
            "battle",
            "Combatant is fainted or has used all five actions",
        ));
    }
    if automatic && p.used_moves.iter().any(|m| m == name) {
        return Err(AppError::new(
            "battle",
            "Move already attempted this round; special repeat effects need GM adjudication",
        ));
    }
    let s = effective_sheet(c, p)?;
    if automatic && !s.moves.iter().any(|m| m == name) {
        return Err(AppError::new(
            "validation",
            "Combatant has not learned this move",
        ));
    }
    let m = catalog
        .find("moves", "Name", name)
        .ok_or_else(|| AppError::new("missing", "Unknown move"))?;
    let first = score(&s, m["Accuracy1"].as_str().unwrap_or_default())?;
    let second = match m["Accuracy2"].as_str().unwrap_or_default() {
        "" => 0,
        source => score(&s, source)?,
    };
    if s.rank == "Champion" && s.workflow.is_none() {
        return Err(AppError::new(
            "manual_rule",
            "Champion rank modifiers require GM adjudication",
        ));
    }
    let rank_bonus = if crate::creation::rank(&s.rank).is_some_and(|r| r >= 6)
        && !m["Accuracy2"].as_str().unwrap_or_default().is_empty()
    {
        2
    } else {
        0
    };
    let pool = first
        .checked_add(second)
        .and_then(|v| v.checked_add(rank_bonus))
        .and_then(|v| v.checked_add(modifier))
        .ok_or_else(|| AppError::new("limit", "Pool overflow"))?;
    if !(1..=100).contains(&pool) {
        return Err(AppError::new(
            "manual_rule",
            "Pool outside 1–100 dice; resolve manually",
        ));
    }
    Ok((pool as u32, m.clone()))
}

pub fn dice_count(c: &Campaign, action: &Action, catalog: &Catalog) -> Result<u32> {
    if [ENGINE, TURN_ENGINE].contains(&c.engine.as_str()) {
        return crate::combat::dice_count(c, action, catalog);
    }
    match action {
        Action::Initiative { .. } => Ok(1),
        Action::Accuracy {
            combatant_id,
            move_name,
            pool_modifier,
            ..
        } => Ok(accuracy(c, combatant_id, move_name, *pool_modifier, catalog)?.0),
        Action::Roll { dice, .. } | Action::Resolve { dice, .. } => Ok(*dice),
        Action::Damage {
            combatant_id,
            sequence,
            pool_modifier,
            ..
        } => Ok(damage_pool(c, combatant_id, *sequence, *pool_modifier, catalog)?.dice),
        _ => Ok(0),
    }
}

pub fn pending<'a>(c: &'a Campaign, key: &str, sequence: u32) -> Result<&'a PendingDamage> {
    c.battle
        .as_ref()
        .filter(|b| b.active)
        .and_then(|b| b.combatants.iter().find(|p| p.id == key && p.deployed))
        .and_then(|p| p.pending_damage.as_ref())
        .filter(|p| p.sequence == sequence)
        .ok_or_else(|| AppError::new("stale", "No matching successful attack awaiting damage"))
}

pub struct DamagePool {
    pub dice: u32,
    pub attribute: i32,
    pub power: i32,
    pub defense: i32,
    pub stab: i32,
    pub critical: i32,
    pub pain: i32,
    pub effectiveness: i32,
    pub immune: bool,
}

pub fn damage_pool(
    c: &Campaign,
    key: &str,
    sequence: u32,
    modifier: i32,
    catalog: &Catalog,
) -> Result<DamagePool> {
    if !core_engine(&c.engine) || !(-20..=20).contains(&modifier) {
        return Err(AppError::new(
            "validation",
            "Core rules and a pool modifier between -20 and 20 are required",
        ));
    }
    let hit = pending(c, key, sequence)?;
    let b = c.battle.as_ref().unwrap();
    let attacker = b
        .combatants
        .iter()
        .find(|p| p.id == key && p.deployed)
        .unwrap();
    let target = b
        .combatants
        .iter()
        .find(|p| p.id == hit.target_id && p.deployed)
        .ok_or_else(|| AppError::new("missing", "Missing attack target"))?;
    if b.automatic_validation && (attacker.hp == 0 || target.hp == 0) {
        return Err(AppError::new(
            "battle",
            "A combatant fainted; GM must cancel or adjudicate this attack",
        ));
    }
    let source = effective_sheet(c, attacker)?;
    let defender = effective_sheet(c, target)?;
    let m = catalog
        .find("moves", "Name", &hit.move_name)
        .ok_or_else(|| AppError::new("missing", "Unknown move"))?;
    let defense_source = match m["Category"].as_str() {
        Some("Physical") => "Vitality",
        Some("Special") => "Insight",
        _ => {
            return Err(AppError::new(
                "manual_rule",
                "This move has no ordinary damage pool",
            ))
        }
    };
    let attribute_source = m["Damage1"].as_str().unwrap_or_default();
    if !m["Damage2"].as_str().unwrap_or_default().is_empty()
        || m["Power"].as_i64().is_none_or(|p| p <= 0 || p > 100)
    {
        return Err(AppError::new(
            "manual_rule",
            "Fixed, varied and special damage formulas require GM adjudication",
        ));
    }
    let attribute = score(&source, attribute_source)?;
    if (source.rank == "Champion" && source.workflow.is_none())
        || (defender.rank == "Champion" && defender.workflow.is_none())
    {
        return Err(AppError::new(
            "manual_rule",
            "Champion modifiers require GM adjudication",
        ));
    }
    let power = m["Power"].as_i64().unwrap() as i32;
    let defense = crate::creation::profile_stats(&defender)[if defense_source == "Vitality" {
        "Defense"
    } else {
        "SpecialDefense"
    }];
    let types = |s: &Sheet| -> Result<Vec<String>> {
        if s.kind == "trainer" {
            return Err(AppError::new("manual_rule", "Trainer damage requires GM adjudication; Pokemon typing rules cannot be assigned to humans"));
        }
        let species = catalog
            .find(
                "species",
                "AppId",
                s.species_id.as_deref().unwrap_or_default(),
            )
            .ok_or_else(|| AppError::new("manual_rule", "Missing species typing"))?;
        let mut result = vec![];
        for key in ["Type1", "Type2"] {
            if let Some(t) = species[key].as_str().filter(|t| !t.is_empty()) {
                if !result.iter().any(|v| v == t) {
                    result.push(t.to_string());
                }
            }
        }
        Ok(result)
    };
    let move_type = m["Type"].as_str().unwrap_or_default();
    let stab =
        i32::from(source.kind == "pokemon" && types(&source)?.iter().any(|t| t == move_type));
    let critical = if hit.critical { 2 } else { 0 };
    let mut effectiveness = 0;
    let mut immune = false;
    for t in types(&defender)? {
        let effect = type_effect(move_type, &t)?;
        immune |= effect == -100;
        if effect != -100 {
            effectiveness += effect;
        }
    }
    let conditional = crate::combat::type_ability(c, attacker, move_type);
    let ability_bonus = if conditional && attacker.hp <= attacker.max_hp / 2 {
        2
    } else {
        0
    };
    let pool = (attribute + power - defense).max(1) + stab + critical + modifier + ability_bonus;
    if !(1..=100).contains(&pool) {
        return Err(AppError::new(
            "manual_rule",
            "Damage pool outside 1–100 dice",
        ));
    }
    Ok(DamagePool {
        dice: if immune { 0 } else { pool as u32 },
        attribute,
        power,
        defense,
        stab,
        critical,
        pain: if attribute_source == "Vitality" || conditional {
            0
        } else {
            (pain(attacker.hp, attacker.max_hp) - attacker.runtime.pain_ignored).max(0)
        },
        effectiveness,
        immune,
    })
}

// Defensive chart, corebook pp.52–53: flat HP adjustments, never multipliers.
pub fn type_effect(attack: &str, defense: &str) -> Result<i32> {
    let (resists, weak, immune) = match defense {
        "Normal" => ("", "Fight", "Ghost"),
        "Bug" => ("Fight Grass Ground", "Fire Flying Rock", ""),
        "Dark" => ("Dark Ghost", "Bug Fairy Fight", "Psychic"),
        "Dragon" => ("Electric Fire Grass Water", "Dragon Fairy Ice", ""),
        "Electric" => ("Electric Flying Steel", "Ground", ""),
        "Fairy" => ("Bug Dark Fight", "Poison Steel", "Dragon"),
        "Fight" => ("Bug Dark Rock", "Fairy Flying Psychic", ""),
        "Fire" => ("Bug Fairy Fire Grass Ice Steel", "Ground Rock Water", ""),
        "Flying" => ("Bug Fight Grass", "Electric Ice Rock", "Ground"),
        "Ghost" => ("Bug Poison", "Dark Ghost", "Fight Normal"),
        "Grass" => (
            "Electric Grass Ground Water",
            "Bug Fire Flying Ice Poison",
            "",
        ),
        "Ground" => ("Poison Rock", "Grass Ice Water", "Electric"),
        "Ice" => ("Ice", "Fight Fire Rock Steel", ""),
        "Poison" => ("Bug Fairy Fight Grass Poison", "Ground Psychic", ""),
        "Psychic" => ("Fight Psychic", "Bug Dark Ghost", ""),
        "Rock" => (
            "Fire Flying Normal Poison",
            "Fight Grass Ground Steel Water",
            "",
        ),
        "Steel" => (
            "Bug Dragon Fairy Flying Grass Ice Normal Psychic Rock Steel",
            "Fight Fire Ground",
            "Poison",
        ),
        "Water" => ("Fire Ice Steel Water", "Electric Grass", ""),
        _ => return Err(AppError::new("manual_rule", "Unknown defensive type")),
    };
    if ![
        "Normal", "Bug", "Dark", "Dragon", "Electric", "Fairy", "Fight", "Fire", "Flying", "Ghost",
        "Grass", "Ground", "Ice", "Poison", "Psychic", "Rock", "Steel", "Water",
    ]
    .contains(&attack)
    {
        return Err(AppError::new("manual_rule", "Unknown attack type"));
    }
    let contains = |s: &str| s.split_whitespace().any(|t| t == attack);
    Ok(if contains(immune) {
        -100
    } else if contains(weak) {
        1
    } else if contains(resists) {
        -1
    } else {
        0
    })
}
