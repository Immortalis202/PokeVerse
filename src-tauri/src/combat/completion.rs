//! Versioned, saved block adoption for the remaining exact-name move definitions.
use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct MoveCompletionState {
    pub blocks: BTreeSet<String>,
}
fn registry() -> &'static serde_json::Value {
    static DATA: std::sync::LazyLock<serde_json::Value> = std::sync::LazyLock::new(|| {
        serde_json::from_str(include_str!("../../../rules/move-completion.json")).unwrap()
    });
    &DATA
}
pub fn initial(engine: &str) -> MoveCompletionState {
    let blocks = registry()["blocks"].as_object().unwrap();
    let Some(last) = blocks
        .iter()
        .find(|(_, b)| b["engine"] == engine)
        .map(|(id, _)| id)
    else {
        return MoveCompletionState::default();
    };
    MoveCompletionState {
        blocks: blocks.keys().filter(|id| *id <= last).cloned().collect(),
    }
}
pub fn definition(c: &Campaign, name: &str) -> Option<(&'static str, &'static serde_json::Value)> {
    let f = flow(c).ok()?;
    registry()["blocks"]
        .as_object()?
        .iter()
        .filter(|(id, _)| f.completion.blocks.contains(*id))
        .find_map(|(_, b)| {
            b["moves"]
                .get(name)
                .map(|d| (b["kind"].as_str().unwrap(), d))
        })
}
pub fn effect_dice(c: &Campaign, q: &QueuedAction) -> Result<Option<u32>> {
    if definition(c, &q.move_name).is_some_and(|(kind, _)| kind == "stat") {
        return damaging_stats::effect_dice(c, q);
    }
    Ok(definition(c, &q.move_name).map(|_| 0))
}
fn immune_target(c: &Campaign, q: &QueuedAction, cat: &Catalog) -> Result<bool> {
    if q.ice {
        return Ok(true);
    }
    let source = reviewed::source(c, q)?;
    let target = fighter(c, &q.target_id)?;
    let mut m = move_entry(cat, &q.move_name)?;
    special::move_data(c, &source, &mut m);
    let m = field::move_data(c, &source, &m);
    if target.hp == 0
        || special::target_effects_blocked(c, q, &m)
        || (!reviewed::extreme(c, &source)
            && (extra::effects_immune(c, target, &m) || abilities::move_immune(c, target, &m)))
    {
        return Ok(true);
    }
    if m["Category"] != "Support" {
        let (n, immune) =
            rules::type_matchup(c, &source, target, m["Type"].as_str().unwrap_or(""), cat)?;
        let n = effectiveness(c, &m, target, cat, n)?;
        return Ok(immune
            || abilities::damage_immune_from(c, &source, target, &m)
            || extra::source_immune(c, &source, target, n));
    }
    Ok(false)
}
fn install(c: &mut Campaign, q: &QueuedAction, source: &str, target: &str) -> Result<()> {
    let p = fighter(c, target)?;
    if p.hp == 0 || (p.has_ability("Run Away") && !house::ring(c, p)) {
        return Ok(());
    }
    let name = format!("move-block:{}", q.move_name);
    if flow(c)?
        .special
        .effects
        .iter()
        .any(|e| e.name == name && e.target == target)
    {
        return Ok(());
    }
    let started = battle(c)?.round;
    let (_, d) = definition(c, &q.move_name).unwrap();
    let rounds = d["rounds"].as_u64().map(|n| n as u32);
    let paired = d["both"] == true;
    flow_mut(c).special.effects.push(special::TimedEffect {
        name,
        source: source.into(),
        target: target.into(),
        rounds,
        amount: 0,
        started,
        slot: if paired {
            "trap-pair".into()
        } else {
            String::new()
        },
    });
    Ok(())
}
pub fn effect(c: &mut Campaign, q: &QueuedAction, _dice: &[u8], cat: &Catalog) -> Result<bool> {
    let Some((kind, d)) = definition(c, &q.move_name) else {
        return Ok(false);
    };
    match kind {
        "ordinary" => {}
        "support-stat" => {
            support_stat(c, q, d, cat)?;
        }
        "stat" => {
            return damaging_stats::effect(c, q, _dice, cat);
        }
        "trap" => {
            if !immune_target(c, q, cat)? {
                install(c, q, &q.actor_id, &q.target_id)?;
                if d["both"] == true {
                    install(c, q, &q.target_id, &q.actor_id)?;
                }
            }
        }
        "ongoing" => {
            if !immune_target(c, q, cat)? {
                if d["block"] == true {
                    install(c, q, &q.actor_id, &q.target_id)?;
                    if d["both"] == true {
                        install(c, q, &q.target_id, &q.actor_id)?;
                    }
                }
                if !flow(c)?
                    .special
                    .effects
                    .iter()
                    .any(|e| e.target == q.target_id && e.name.starts_with("ongoing:"))
                {
                    flow_mut(c).special.effects.push(special::TimedEffect {
                        name: format!("ongoing:{}", q.move_name),
                        source: q.actor_id.clone(),
                        target: q.target_id.clone(),
                        rounds: Some(d["rounds"].as_u64().unwrap() as u32),
                        amount: d["dice"].as_i64().unwrap() as i32,
                        started: 0,
                        slot: String::new(),
                    });
                }
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}
pub fn blocked(c: &Campaign, id: &str) -> bool {
    if !flow(c).is_ok_and(|f| !f.completion.blocks.is_empty()) {
        return false;
    }
    let Ok(p) = special::registered(c, id) else {
        return false;
    };
    if !p.deployed || p.hp == 0 {
        return false;
    }
    if p.has_ability("Run Away") && !house::ring(c, p) {
        return false;
    }
    flow(c).unwrap().special.effects.iter().any(|e| {
        e.target == id
            && e.name.starts_with("move-block:")
            && ((e.name != "move-block:Jaw Lock" && e.slot != "trap-pair")
                || special::registered(c, &e.source).is_ok_and(|p| p.deployed && p.hp > 0))
    })
}

pub fn tick_dice(c: &Campaign, e: &special::TimedEffect) -> Option<u32> {
    let name = e.name.strip_prefix("ongoing:")?;
    let (kind, d) = definition(c, name)?;
    (kind == "ongoing").then(|| d["dice"].as_u64().unwrap() as u32)
}
pub fn tick(
    c: &mut Campaign,
    e: &special::TimedEffect,
    dice: &[u8],
    cat: &Catalog,
) -> Result<bool> {
    if tick_dice(c, e).is_none() {
        return Ok(false);
    }
    let name = e.name.strip_prefix("ongoing:").unwrap();
    let (_, d) = definition(c, name).unwrap();
    let before = fighter(c, &e.target)?.hp;
    let damage_type = d["damage_type"].as_str().unwrap();
    let target = fighter(c, &e.target)?;
    let immune = damage_type != "Neutral"
        && abilities::damage_immune(
            c,
            target,
            &serde_json::json!({"Type":damage_type,"Category":"Special"}),
        );
    let extra = i32::from(matches_types(c, target, cat, &d["extra_tick_types"])?);
    if !immune {
        field::hurt(
            c,
            &e.target,
            rules::successes(dice) + extra,
            damage_type,
            cat,
        )?;
    }
    let lost = before - fighter(c, &e.target)?.hp;
    ability_changes::record_sourced(c, &e.source, &e.target, lost, name);
    Ok(true)
}

fn matches_types(
    c: &Campaign,
    target: &Combatant,
    cat: &Catalog,
    list: &serde_json::Value,
) -> Result<bool> {
    let Some(list) = list.as_array() else {
        return Ok(false);
    };
    Ok(field::types(c, target, cat)?
        .iter()
        .any(|t| list.iter().any(|v| v.as_str() == Some(t))))
}
pub fn effectiveness(
    c: &Campaign,
    m: &serde_json::Value,
    target: &Combatant,
    cat: &Catalog,
    base: i32,
) -> Result<i32> {
    let Some((_, d)) = definition(c, m["Name"].as_str().unwrap_or("")) else {
        return Ok(base);
    };
    Ok(
        if matches_types(c, target, cat, &d["super_effective_types"])? {
            1
        } else {
            base
        },
    )
}
pub fn cleanse(c: &mut Campaign, id: &str) {
    let paired: Vec<_> = flow(c)
        .unwrap()
        .special
        .effects
        .iter()
        .filter(|e| e.target == id && e.slot == "trap-pair")
        .filter(|e| {
            e.name
                .strip_prefix("move-block:")
                .and_then(|n| definition(c, n))
                .is_some_and(|(k, _)| k == "ongoing")
        })
        .map(|e| (e.name.clone(), e.source.clone()))
        .collect();
    let names: BTreeSet<_> = flow(c)
        .unwrap()
        .special
        .effects
        .iter()
        .filter(|e| e.target == id)
        .filter_map(|e| {
            e.name
                .strip_prefix("move-block:")
                .filter(|n| definition(c, n).is_some_and(|(k, _)| k == "ongoing"))
                .map(str::to_string)
        })
        .collect();
    flow_mut(c).special.effects.retain(|e| {
        !(e.target == id
            && (e.name.starts_with("ongoing:")
                || e.name
                    .strip_prefix("move-block:")
                    .is_some_and(|n| names.contains(n))))
            && !paired
                .iter()
                .any(|(name, source)| e.name == *name && e.target == *source && e.source == id)
    });
}

pub fn ordinary(c: &Campaign, name: &str) -> bool {
    definition(c, name).is_some_and(|(kind, _)| kind == "ordinary")
}

pub fn excludes_self(c: &Campaign, name: &str) -> bool {
    definition(c, name).is_some_and(|(_, d)| d["exclude_self"] == true)
}
fn support_stat(
    c: &mut Campaign,
    q: &QueuedAction,
    d: &serde_json::Value,
    cat: &Catalog,
) -> Result<()> {
    let source = reviewed::source(c, q)?;
    if d["exclude_user_type"].as_str().is_some_and(|t| {
        field::types(c, &source, cat).is_ok_and(|types| types.iter().any(|v| v == t))
    }) {
        return Ok(());
    }
    let targets: Vec<String> = if d["field"] == true {
        battle(c)?
            .combatants
            .iter()
            .filter(|p| p.deployed && p.hp > 0 && p.available_round <= battle(c).unwrap().round)
            .map(|p| p.id.clone())
            .collect()
    } else {
        vec![q.target_id.clone()]
    };
    for target in targets {
        let p = fighter(c, &target)?;
        if d["exclude_self"] == true && (p.id == q.actor_id || p.side != source.side) {
            continue;
        }
        if let Some(types) = d["types"].as_array() {
            let actual = field::types(c, p, cat)?;
            if !types.iter().any(|t| actual.iter().any(|v| t == v.as_str())) {
                continue;
            }
        }
        let mut recipient = q.clone();
        recipient.target_id = target.clone();
        if immune_target(c, &recipient, cat)? {
            continue;
        }
        for (key, amount) in d["changes"].as_object().unwrap() {
            let mut n = amount.as_i64().unwrap() as i32;
            if let Some(sunny) = d["sunny_amount"].as_i64() {
                if matches!(
                    field::weather(c, Some(&source)),
                    Some(field::Weather::Sunny | field::Weather::Desolate)
                ) {
                    n = sunny as i32;
                }
            }
            extra::change_stat(c, &q.actor_id, &target, key, n)?;
        }
    }
    Ok(())
}
