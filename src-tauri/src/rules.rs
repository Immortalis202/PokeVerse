//! Pokerole 3.0 corebook: pp. 35–36, 56–57, 61–62, 79–80.
//! Shared ordinary pools with versioned reviewed battle hooks; unsupported interactions require GM adjudication.
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
    let mut s = base_sheet(c, p)?;
    crate::combat::apply_stats(c, p, &mut s);
    crate::combat::field::stats(c, p, &mut s);
    crate::combat::sequencing::apply_stats(c, p, &mut s);
    for key in crate::creation::ATTRS {
        if let Some(v) = s.stats.get_mut(key) {
            *v += crate::combat::forms::stat_bonus(c, p, key);
        }
    }
    Ok(s)
}
/// Permanent attributes plus explicit GM overrides, before battle modifiers.
pub fn base_sheet(c: &Campaign, p: &Combatant) -> Result<Sheet> {
    let mut s = c
        .sheets
        .get(&p.sheet_id)
        .cloned()
        .ok_or_else(|| AppError::new("missing", "Missing combatant sheet"))?;
    crate::combat::forms::sheet(c, p, &mut s);
    s.stats.extend(p.stat_overrides.clone());
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
    crate::combat::abilities::move_guard(c, name)?;
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
    crate::combat::sequencing::move_guard(c, key, Some(name))?;
    if automatic
        && p.used_moves.iter().any(|m| m == name)
        && !crate::combat::special::repeat(c, key, name)
        && !crate::combat::sequencing::repeat(c, key, name)
    {
        return Err(AppError::new(
            "battle",
            "Move already attempted this round; special repeat effects need GM adjudication",
        ));
    }
    crate::combat::special::accuracy_guard(c, key, name)?;
    let s = effective_sheet(c, p)?;
    crate::combat::abilities::guard(c, p)?;
    if automatic && !s.moves.iter().any(|m| m == name) {
        return Err(AppError::new(
            "validation",
            "Combatant has not learned this move",
        ));
    }
    let m = catalog
        .find("moves", "Name", name)
        .ok_or_else(|| AppError::new("missing", "Unknown move"))?;
    crate::combat::forms::guard_move(c, key, &m)?;
    crate::combat::field::guard_move(c, p, &m)?;
    crate::combat::special::guard(c, key, &m, catalog)?;
    let m = crate::combat::field::move_data(c, p, &m);
    crate::combat::standalone::torrent(c, p, &m, p.hp, None)?;
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
        .checked_add(crate::combat::extra::accuracy_bonus(c, p, &m))
        .and_then(|v| {
            v.checked_add(crate::combat::interactions::trait_value(
                c,
                p,
                "AccuracyPool",
            ))
        })
        .and_then(|v| v.checked_add(crate::combat::items::accuracy_bonus(c, key)))
        .and_then(|v| {
            v.checked_add(
                if crate::combat::special::enabled(c)
                    && p.has_ability("Skill Link")
                    && crate::combat::special::limit(&m) > 1
                {
                    2
                } else {
                    0
                },
            )
        })
        .and_then(|v| v.checked_add(second))
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
    if [
        ENGINE,
        STANDALONE_ENGINE,
        INFORMATION_ENGINE,
        FORM_ENGINE,
        ITEM_ENGINE,
        EVENT_ENGINE,
        PROVENANCE_ENGINE,
        SEQUENCING_ENGINE,
        STAT_ENGINE,
        REVIEWED_ENGINE,
        TRANSFER_ENGINE,
        ASSIGNMENT_ENGINE,
        DECOY_ENGINE,
        TACTICS_ENGINE,
        SPECIAL_ENGINE,
        EXTRA_ENGINE,
        FIELD_ENGINE,
        ABILITY_ENGINE,
        TARGET_ENGINE,
        TURN_ENGINE,
    ]
    .contains(&c.engine.as_str())
    {
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
    let frozen_attacker =
        crate::combat::ability_changes::frozen_subject(c, key, sequence, attacker);
    let frozen_target = crate::combat::ability_changes::frozen_subject(c, key, sequence, target);
    let attacker = &frozen_attacker;
    let target = &frozen_target;
    if b.automatic_validation
        && ((attacker.hp == 0 && crate::combat::abilities::context(c, key, sequence).is_none())
            || target.hp == 0)
    {
        return Err(AppError::new(
            "battle",
            "A combatant fainted; GM must cancel or adjudicate this attack",
        ));
    }
    let mut source = effective_sheet(c, attacker)?;
    crate::combat::abilities::guard(c, attacker)?;
    let context = crate::combat::abilities::context(c, key, sequence);
    if let Some(ctx) = context {
        source.stats = ctx.stats.clone();
    }
    if crate::combat::provenance::ignores_attributes(c, target, attacker) {
        source.stats = context
            .filter(|ctx| !ctx.base_stats.is_empty())
            .map_or(base_sheet(c, attacker)?.stats, |ctx| ctx.base_stats.clone());
    }
    let attack_hp = context.map_or(attacker.hp, |ctx| ctx.hp);
    let ignore_defender = crate::combat::provenance::ignores_attributes(c, attacker, target);
    let defender = if ignore_defender {
        base_sheet(c, target)?
    } else {
        effective_sheet(c, target)?
    };
    let m = catalog
        .find("moves", "Name", &hit.move_name)
        .ok_or_else(|| AppError::new("missing", "Unknown move"))?;
    let m = crate::combat::field::move_data(c, attacker, &m);
    if let Some(mut pool) =
        crate::combat::special::formula(c, attacker, target, &m, sequence, modifier, catalog)?
    {
        if crate::combat::forms::tera_shell(c, target, &m) {
            pool.effectiveness = -1;
            pool.immune = false;
        }
        return Ok(pool);
    }
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
    let saved_rivalry = context.and_then(|ctx| ctx.rivalry.get(&target.id)).copied();
    let mut rivalry_source = source.clone();
    if attribute_source
        .split('/')
        .any(|key| key.trim() == "Strength")
    {
        let base = score(&source, "Strength")?;
        let delta = crate::combat::standalone::strength_delta(
            c,
            attacker,
            target,
            "Strength",
            base,
            saved_rivalry,
        )?;
        rivalry_source.stats.insert("Strength".into(), base + delta);
    }
    let attribute = score(&rivalry_source, attribute_source)?;
    if (source.rank == "Champion" && source.workflow.is_none())
        || (defender.rank == "Champion" && defender.workflow.is_none())
    {
        return Err(AppError::new(
            "manual_rule",
            "Champion modifiers require GM adjudication",
        ));
    }
    let lost_type = crate::combat::field::enabled(c)
        && c.battle
            .as_ref()
            .and_then(|b| b.flow.as_ref())
            .is_some_and(|f| {
                f.battlefield.rules.iter().any(|r| {
                    r.scope == format!("combatant:{}", attacker.id)
                        && r.kind == format!("typing:remove:{}", m["Type"].as_str().unwrap_or(""))
                })
            });
    let power = if lost_type
        || m["Type"] == "Dragon" && crate::combat::field::has(c, "Misty Terrain", attacker)
    {
        0
    } else {
        m["Power"].as_i64().unwrap() as i32
    };
    let defense = crate::creation::profile_stats(&defender)[if defense_source == "Vitality" {
        "Defense"
    } else {
        "SpecialDefense"
    }] + crate::combat::abilities::defense(c, target, defense_source == "Vitality");
    let defense = crate::combat::field::damage_defense(
        c,
        target,
        defense_source == "Vitality",
        catalog,
        defense,
        ignore_defender,
    )?;
    let defense = context
        .and_then(|ctx| ctx.field_snapshot.as_ref())
        .and_then(|f| {
            if ignore_defender {
                f.base_attribute_defenses.get(&target.id)
            } else {
                f.defenses.get(&target.id)
            }
        })
        .map_or(defense, |d| d[usize::from(defense_source != "Vitality")])
        .max(1);
    let defense =
        crate::combat::decoys::pool_defense(c, key, &target.id, &m, defense_source == "Vitality")
            .unwrap_or(defense);
    let defense = if crate::combat::special::enabled(c) && m["Attributes"]["IgnoreDefenses"] == true
    {
        0
    } else {
        defense
    };
    let move_type = m["Type"].as_str().unwrap_or_default();
    let stab = i32::from(
        source.kind == "pokemon"
            && (if crate::combat::field::enabled(c) {
                context.map_or_else(
                    || crate::combat::field::types(c, attacker, catalog),
                    |ctx| Ok(ctx.types.clone()),
                )?
            } else {
                crate::combat::field::types(c, attacker, catalog)?
            })
            .iter()
            .any(|t| t == move_type),
    );
    let critical = if hit.critical
        || (crate::combat::extra::enabled(c)
            && attacker.has_ability("Merciless")
            && target
                .runtime
                .statuses
                .iter()
                .any(|s| s.kind.contains("Poison")))
    {
        crate::combat::abilities::critical(c, attacker, target)
    } else {
        0
    };
    let (mut effectiveness, mut immune) = type_matchup(c, attacker, target, move_type, catalog)?;
    if crate::combat::forms::tera_shell(c, target, &m) {
        effectiveness = -1;
        immune = false;
    }
    crate::combat::reviewed::unique_damage_guard(c, attacker, target, effectiveness)?;
    if crate::combat::field::weather(c, Some(target))
        == Some(crate::combat::field::Weather::StrongWinds)
        && crate::combat::field::types(c, target, catalog)?
            .iter()
            .any(|t| t == "Flying")
    {
        effectiveness = effectiveness.min(0);
    }
    let conditional = crate::combat::type_ability(c, attacker, move_type);
    let corrosion = crate::combat::extra::enabled(c)
        && attacker.has_ability("Corrosion")
        && move_type == "Poison";
    immune |= !crate::combat::decoys::protected_subject(c, key, &target.id, &m)
        && !corrosion
        && (crate::combat::abilities::damage_immune_from(c, attacker, target, &m)
            || crate::combat::extra::source_immune(c, attacker, target, effectiveness));
    let ability_bonus = if conditional
        && !(crate::combat::standalone::enabled(c)
            && attacker.has_ability("Torrent")
            && move_type == "Water")
        && attack_hp <= attacker.max_hp / 2
    {
        2
    } else {
        0
    };
    let original_bonus = crate::combat::abilities::damage_bonus(
        c,
        attacker,
        &m,
        stab > 0,
        context.map_or(&attacker.runtime.ability.counters, |ctx| &ctx.counters),
    );
    let additional = crate::combat::extra::bonus(c, attacker, target, &m);
    let extra = if additional < 0 {
        additional
    } else {
        original_bonus.max(additional)
    };
    let pool = (attribute + power - defense).max(1)
        + crate::combat::provenance::sheer_force_bonus(c, key, sequence)
        + crate::combat::items::damage_bonus(c, key, sequence)
        + match context {
            Some(ctx) => ctx.item_damage_bonus,
            None => crate::combat::items::damage_dice_bonus(c, attacker, sequence, &m, catalog)?,
        }
        + stab
        + critical
        + modifier
        + ability_bonus
        + crate::combat::standalone::torrent(
            c,
            attacker,
            &m,
            attack_hp,
            context.and_then(|ctx| ctx.torrent_bonus),
        )?
        + extra
        + if let Some(n) = context.and_then(|ctx| ctx.special_bonus.get(&target.id)) {
            *n
        } else {
            crate::combat::special::bonus(c, attacker, target, &m, catalog)?
        }
        + crate::combat::field::target_bonus(c, target, &m)
        + context.map_or_else(
            || crate::combat::field::bonus(c, attacker, &m),
            |ctx| ctx.field_bonus,
        );
    let pool = if crate::combat::extra::enabled(c) {
        pool.max(1)
    } else {
        pool
    };
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
            (pain(attack_hp, attacker.max_hp)
                - context.map_or(attacker.runtime.pain_ignored, |ctx| ctx.pain_ignored))
            .max(0)
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

/// Shared type-immunity calculation for damage and qualifying contact hooks.
pub fn type_matchup(
    c: &Campaign,
    attacker: &Combatant,
    target: &Combatant,
    move_type: &str,
    catalog: &Catalog,
) -> Result<(i32, bool)> {
    let mut effectiveness = 0;
    let mut immune = false;
    let mut ignore_resistance = crate::combat::provenance::ignore_resistance(c, attacker, target);
    for t in crate::combat::field::types(c, target, catalog)? {
        let effect = if crate::combat::extra::ignores_ghost(c, attacker, move_type, &t) {
            0
        } else {
            type_effect(move_type, &t)?
        };
        let effect = if crate::combat::reviewed::type_bypass(c, attacker, target) && effect == -100
        {
            0
        } else {
            effect
        };
        let effect = if crate::combat::extra::enabled(c)
            && attacker.has_ability("Corrosion")
            && move_type == "Poison"
            && effect == -100
        {
            0
        } else {
            effect
        };
        immune |= effect == -100
            && !(move_type == "Ground"
                && t == "Flying"
                && (crate::combat::field::has(c, "Gravity", target)
                    || crate::combat::special::roost(c, &target.id)
                    || crate::combat::items::iron_ball(c, &target.id)));
        if effect == -1 && ignore_resistance {
            ignore_resistance = false;
        } else if effect != -100 {
            effectiveness += effect;
        }
    }
    Ok((effectiveness, immune))
}
