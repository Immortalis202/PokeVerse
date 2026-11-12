//! MOVE-A48 stat operations: scene base replacements, change transfers and chosen increases.
//!
//! Splits and swaps replace scene base values; increases and reductions stay with their
//! owner. Transfers act on applied move/stacking/copied changes and triggered-ability
//! counters, never on passive ability or field bonuses.
use super::*;

/// Keys whose increases and reductions transfer between combatants.
const CHANGE_KEYS: [&str; 9] = [
    "Strength",
    "Dexterity",
    "Vitality",
    "Special",
    "Insight",
    "Defense",
    "SpecialDefense",
    "AccuracyPool",
    "EvasionPool",
];
const ACUPRESSURE: [&str; 7] = [
    "Strength",
    "Dexterity",
    "Special",
    "Defense",
    "SpecialDefense",
    "AccuracyPool",
    "EvasionPool",
];
const ATTRIBUTES: [&str; 5] = ["Strength", "Dexterity", "Vitality", "Special", "Insight"];
const SHEET_BASES: [&str; 3] = ["Strength", "Dexterity", "Special"];
const DEFENSES: [&str; 2] = ["Defense", "SpecialDefense"];

fn definition(c: &Campaign, name: &str) -> Option<&'static serde_json::Value> {
    completion::definition(c, name)
        .filter(|(kind, _)| *kind == "stat-op")
        .map(|(_, d)| d)
}
fn bases<'a>(c: &'a Campaign, id: &str) -> Option<&'a BTreeMap<String, i32>> {
    flow(c).ok()?.completion.stat_bases.get(id)
}

/// Scene Strength/Dexterity/Special replacements, applied before battle modifiers.
pub fn apply_base(c: &Campaign, p: &Combatant, s: &mut Sheet) {
    if let Some(values) = bases(c, &p.id) {
        for key in SHEET_BASES {
            if let Some(v) = values.get(key) {
                s.stats.insert(key.into(), *v);
            }
        }
    }
}
/// Derived profile values with scene Defense/Sp. Defense replacements.
pub fn profile(c: &Campaign, p: &Combatant, s: &Sheet) -> BTreeMap<String, i32> {
    let mut stats = crate::creation::profile_stats(s);
    if let Some(values) = bases(c, &p.id) {
        for key in DEFENSES {
            if let Some(v) = values.get(key) {
                stats.insert(key.into(), *v);
            }
        }
    }
    stats
}
/// Acupressure's Evasion increase and transferred Evasion changes feed the evade pool.
pub fn apply_sheet(c: &Campaign, p: &Combatant, s: &mut Sheet) {
    if flow(c).is_err() {
        return;
    }
    let bonus = trait_bonus(c, p, "EvasionPool");
    if bonus != 0 {
        *s.skills.entry("Evasion".into()).or_default() += bonus;
    }
}
/// Net stacking/copied value for a numeric trait channel.
pub fn trait_bonus(c: &Campaign, p: &Combatant, key: &str) -> i32 {
    let values: Vec<i32> = completion::stack_values(c, p, key)
        .into_iter()
        .chain(if key == "EvasionPool" {
            stat_channels::move_values(c, p, key)
        } else {
            vec![]
        })
        .collect();
    values.iter().copied().max().unwrap_or(0).max(0)
        + values.iter().copied().min().unwrap_or(0).min(0)
}
fn base_value(c: &Campaign, p: &Combatant, key: &str) -> Result<i32> {
    if let Some(v) = bases(c, &p.id).and_then(|v| v.get(key)) {
        return Ok(*v);
    }
    let s = rules::base_sheet(c, p)?;
    Ok(if DEFENSES.contains(&key) {
        crate::creation::profile_stats(&s)[key]
    } else {
        *s.stats.get(key).unwrap_or(&0)
    })
}
fn set_base(c: &mut Campaign, id: &str, key: &str, value: i32) {
    flow_mut(c)
        .completion
        .stat_bases
        .entry(id.into())
        .or_default()
        .insert(key.into(), value);
}
/// Scene replacements end when their owner leaves battle; Spectral Thief markers end with the cast.
pub fn lifecycle(c: &mut Campaign) -> Result<()> {
    let active: BTreeSet<String> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    let queued: BTreeSet<u32> = flow(c)?.queue.iter().map(|q| q.sequence).collect();
    let open = match flow(c)?.queue.first().cloned() {
        Some(q) if !flow(c)?.window => {
            let options = completion::options(c, &q)?;
            (!options.is_empty()).then_some((q.sequence, options))
        }
        _ => None,
    };
    let f = flow_mut(c);
    f.completion.open_options = open.into_iter().collect();
    f.completion.stat_bases.retain(|id, _| active.contains(id));
    f.completion.stolen.retain(|s| queued.contains(s));
    f.completion.move_options.retain(|s, _| queued.contains(s));
    Ok(())
}

/// Applied move/stacking/copied changes plus triggered-ability counters, as (increase, reduction).
fn parts(c: &Campaign, p: &Combatant, key: &str) -> (i32, i32) {
    let numeric = ["AccuracyPool", "EvasionPool"].contains(&key);
    let turn = |n: i32| {
        if numeric {
            n
        } else {
            match p.ability_for(&["Contrary", "Simple"]) {
                "Contrary" => -n,
                "Simple" => n + n.signum(),
                _ => n,
            }
        }
    };
    let names = [format!("buff:{key}"), format!("debuff:{key}")];
    let mut values: Vec<i32> = flow(c).map_or_else(
        |_| vec![],
        |f| {
            f.special
                .effects
                .iter()
                .filter(|e| e.target == p.id && names.contains(&e.name))
                .map(|e| turn(e.amount))
                .collect()
        },
    );
    if values.is_empty() {
        if let Some(n) = p.runtime.modifiers.get(key) {
            values.push(turn(*n));
        }
    }
    values.extend(stat_channels::move_values(c, p, key));
    values.extend(completion::stack_values(c, p, key));
    for prefix in ["boost", "debuff"] {
        if let Some(n) = p.runtime.ability.counters.get(&format!("{prefix}:{key}")) {
            values.push(*n);
        }
    }
    (
        values.iter().copied().max().unwrap_or(0).max(0),
        values.iter().copied().min().unwrap_or(0).min(0),
    )
}
fn grants(
    c: &Campaign,
    from: &str,
    to: &str,
    cause: &str,
    keys: &[&str],
    map: impl Fn(i32, i32) -> (i32, i32),
) -> Result<Vec<stat_channels::StatGrant>> {
    let p = fighter(c, from)?;
    Ok(keys
        .iter()
        .filter_map(|key| {
            let (positive, negative) = map(parts(c, p, key).0, parts(c, p, key).1);
            (positive > 0 || negative < 0).then(|| stat_channels::StatGrant {
                source: from.into(),
                target: to.into(),
                cause: cause.into(),
                key: (*key).into(),
                channel: "move".into(),
                positive,
                negative,
            })
        })
        .collect())
}
fn clear(c: &mut Campaign, id: &str, keys: &[&str]) -> Result<()> {
    completion::reset_keys(c, id, &keys.iter().copied().collect(), false)
}
/// Spectral Thief removes the target's increases and keeps its reductions.
fn steal(c: &mut Campaign, actor: &str, target: &str, cause: &str) -> Result<()> {
    let keys = &CHANGE_KEYS[..7];
    let stolen = grants(c, target, actor, cause, keys, |up, _| (up, 0))?;
    let kept = grants(c, target, target, cause, keys, |_, down| (0, down))?;
    clear(c, target, keys)?;
    stat_channels::apply(c, kept)?;
    stat_channels::apply(c, stolen)
}

fn channels(c: &Campaign) -> Result<()> {
    if stat_channels::enabled(c) {
        Ok(())
    } else {
        Err(AppError::new(
            "manual_rule",
            "Transferring stat changes requires the stat-channel engine; adjudicate this cast explicitly",
        ))
    }
}
/// Owner choices a cast still needs before its effect can resolve.
pub fn options(c: &Campaign, q: &QueuedAction) -> Result<Vec<String>> {
    let Some(d) = definition(c, &q.move_name) else {
        return Ok(vec![]);
    };
    if d["op"] != "lowest" {
        return Ok(vec![]);
    }
    let lowest = lowest(c, fighter(c, &q.actor_id)?)?;
    Ok(if lowest.len() > 1 { lowest } else { vec![] })
}
fn lowest(c: &Campaign, p: &Combatant) -> Result<Vec<String>> {
    let s = rules::effective_sheet(c, p)?;
    let score = |key: &str| *s.stats.get(key).unwrap_or(&0);
    let min = ATTRIBUTES.iter().map(|k| score(k)).min().unwrap_or(0);
    Ok(ATTRIBUTES
        .iter()
        .filter(|k| score(k) == min)
        .map(|k| (*k).to_string())
        .collect())
}

pub fn effect_dice(c: &Campaign, q: &QueuedAction) -> Option<u32> {
    let d = definition(c, &q.move_name)?;
    Some(if d["op"] == "acupressure" && !q.ice {
        2
    } else {
        0
    })
}

pub fn effect(
    c: &mut Campaign,
    q: &QueuedAction,
    dice: &[u8],
    d: &serde_json::Value,
    cat: &Catalog,
) -> Result<()> {
    if q.ice {
        return Ok(());
    }
    let op = d["op"].as_str().unwrap_or("");
    let targeted = !["acupressure", "lowest", "trade"].contains(&op);
    if targeted && completion::immune_target(c, q, cat)? {
        return Ok(());
    }
    let (a, t) = (q.actor_id.clone(), q.target_id.clone());
    if ["copy", "exchange", "invert", "steal"].contains(&op) {
        channels(c)?;
    }
    let keys = |d: &serde_json::Value| -> Vec<String> {
        d["keys"]
            .as_array()
            .map(|v| {
                v.iter()
                    .filter_map(|k| k.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    };
    match op {
        "acupressure" => {
            let value = (dice[0] as usize - 1) * 6 + (dice[1] as usize - 1);
            if value >= 35 {
                return Err(error(
                    "Acupressure rerolls a double six so all seven outcomes stay equally likely; roll the effect again",
                ));
            }
            let key = ACUPRESSURE[value % 7];
            let started = battle(c)?.round;
            let f = flow_mut(c);
            f.special
                .effects
                .retain(|e| e.target != a || !e.name.starts_with("acupressure:"));
            f.special.effects.push(special::TimedEffect {
                name: format!("acupressure:{key}"),
                source: a.clone(),
                target: a.clone(),
                rounds: None,
                amount: 1,
                started,
                slot: "stacking-stat".into(),
            });
            if SHEET_BASES.contains(&key) {
                fighter_mut(c, &a)?
                    .runtime
                    .modifiers
                    .entry(key.into())
                    .or_insert(0);
            }
        }
        "lowest" => {
            let p = fighter(c, &a)?.clone();
            let lowest = lowest(c, &p)?;
            let key = if lowest.len() == 1 {
                lowest[0].clone()
            } else {
                let choice = flow(c)?
                    .completion
                    .move_options
                    .get(&q.sequence)
                    .cloned()
                    .ok_or_else(|| {
                        error("Choose which tied lowest Attribute Mystical Power raises")
                    })?;
                if !lowest.contains(&choice) {
                    return Err(error("The saved choice is no longer a lowest Attribute"));
                }
                choice
            };
            let score = *rules::effective_sheet(c, &p)?.stats.get(&key).unwrap_or(&0);
            if score < d["limit"].as_i64().unwrap_or(6) as i32 {
                extra::change_stat(c, &a, &a, &key, d["amount"].as_i64().unwrap_or(2) as i32)?;
            }
        }
        "split" | "swap" => {
            let (pa, pt) = (fighter(c, &a)?.clone(), fighter(c, &t)?.clone());
            for key in keys(d) {
                let (va, vt) = (base_value(c, &pa, &key)?, base_value(c, &pt, &key)?);
                let (na, nt) = if op == "split" {
                    let avg = (va + vt).div_euclid(2);
                    (avg, avg)
                } else {
                    (vt, va)
                };
                set_base(c, &a, &key, na);
                set_base(c, &t, &key, nt);
            }
        }
        "trade" => {
            let p = fighter(c, &a)?.clone();
            for pair in d["pairs"].as_array().into_iter().flatten() {
                let (x, y) = (pair[0].as_str().unwrap(), pair[1].as_str().unwrap());
                let (vx, vy) = (base_value(c, &p, x)?, base_value(c, &p, y)?);
                set_base(c, &a, x, vy);
                set_base(c, &a, y, vx);
            }
        }
        "copy" => {
            let copied = grants(c, &t, &a, &q.move_name, &CHANGE_KEYS, |u, d| (u, d))?;
            clear(c, &a, &CHANGE_KEYS)?;
            stat_channels::apply(c, copied)?;
        }
        "exchange" => {
            let to_target = grants(c, &a, &t, &q.move_name, &CHANGE_KEYS, |u, d| (u, d))?;
            let to_user = grants(c, &t, &a, &q.move_name, &CHANGE_KEYS, |u, d| (u, d))?;
            clear(c, &a, &CHANGE_KEYS)?;
            clear(c, &t, &CHANGE_KEYS)?;
            stat_channels::apply(c, to_target)?;
            stat_channels::apply(c, to_user)?;
        }
        "invert" => {
            let inverted = grants(c, &t, &t, &q.move_name, &CHANGE_KEYS, |u, d| (-d, -u))?;
            clear(c, &t, &CHANGE_KEYS)?;
            stat_channels::apply(c, inverted)?;
        }
        // Spectral Thief already stole before damage; see `before_damage`.
        "steal" => {}
        _ => return Err(error("Unknown stat operation")),
    }
    flow_mut(c).completion.move_options.remove(&q.sequence);
    Ok(())
}

/// Spectral Thief steals once its reactions close, before the damage pool is rolled.
pub fn before_damage(c: &mut Campaign, cat: &Catalog) -> Result<()> {
    let Some(q) = flow(c)?.queue.first().cloned() else {
        return Ok(());
    };
    let f = flow(c)?;
    if definition(c, &q.move_name).is_none_or(|d| d["op"] != "steal")
        || q.damage_done
        || q.cancelled
        || f.window
        || f.completion.stolen.contains(&q.sequence)
    {
        return Ok(());
    }
    channels(c)?;
    flow_mut(c).completion.stolen.insert(q.sequence);
    if completion::immune_target(c, &q, cat)? {
        return Ok(());
    }
    steal(c, &q.actor_id, &q.target_id, &q.move_name)
}
