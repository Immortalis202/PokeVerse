//! Versioned, saved block adoption for the remaining exact-name move definitions.
use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct MoveCompletionState {
    pub blocks: BTreeSet<String>,
    pub uses: Vec<CompletionMoveUse>,
    pub entered: BTreeMap<String, u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct CompletionMoveUse {
    pub actor: String,
    pub name: String,
    pub round: u32,
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
        ..Default::default()
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
                .or_else(|| {
                    b["moves"].as_object()?.values().find(|d| {
                        d["runtime_names"]
                            .as_array()
                            .is_some_and(|names| names.iter().any(|v| v == name))
                    })
                })
                .map(|d| (b["kind"].as_str().unwrap(), d))
        })
}
pub fn effect_dice(c: &Campaign, q: &QueuedAction) -> Result<Option<u32>> {
    if let Some(("cost-stat", d)) = definition(c, &q.move_name) {
        let p = fighter(c, &q.actor_id)?;
        return Ok(Some(if q.ice {
            0
        } else {
            match d["damage_mode"].as_str() {
                Some("half-current") => (p.hp / 2).max(0) as u32,
                Some("half-total") => (p.max_hp / 2).max(0) as u32,
                _ => 0,
            }
        }));
    }

    if definition(c, &q.move_name).is_some_and(|(kind, _)| kind == "stat") {
        return damaging_stats::effect_dice(c, q);
    }
    if let Some((_, d)) =
        definition(c, &q.move_name).filter(|(kind, _)| ["accuracy", "stack-stat"].contains(kind))
    {
        let n = d["dice"].as_u64().unwrap() as u32;
        if n == 0 || q.ice || (provenance::enabled(c) && q.sheer_force) {
            return Ok(Some(0));
        }
        let source = reviewed::source(c, q)?;
        let target = fighter(c, &q.target_id)?;
        let bypass = source.side != target.side && reviewed::extreme(c, &source);
        return Ok(Some(
            (n + items::completion::loaded(c, &q.actor_id)
                + 2 * u32::from(source.has_ability("Serene Grace")))
            .saturating_sub(2 * u32::from(!bypass && target.has_ability("Wonder Skin"))),
        ));
    }
    Ok(definition(c, &q.move_name).map(|_| 0))
}
fn immune_target(c: &Campaign, q: &QueuedAction, cat: &Catalog) -> Result<bool> {
    immune_target_with_fainted(c, q, cat, false)
}
fn immune_target_with_fainted(
    c: &Campaign,
    q: &QueuedAction,
    cat: &Catalog,
    allow_fainted: bool,
) -> Result<bool> {
    if q.ice {
        return Ok(true);
    }
    let source = reviewed::source(c, q)?;
    let target = fighter(c, &q.target_id)?;
    let mut m = move_entry(cat, &q.move_name)?;
    special::move_data(c, &source, &mut m);
    let m = field::move_data(c, &source, &m);
    if (!allow_fainted && target.hp == 0)
        || special::target_effects_blocked(c, q, &m)
        || (!reviewed::extreme(c, &source)
            && (extra::effects_immune(c, target, &m)
                || (!ignores_type_immunity(c, &m)
                    && !target_immunity_removed(c, target, &m)
                    && abilities::move_immune(c, target, &m))))
    {
        return Ok(true);
    }
    if m["Category"] != "Support" {
        let (n, immune) =
            rules::type_matchup(c, &source, target, m["Type"].as_str().unwrap_or(""), cat)?;
        let n = effectiveness(c, &m, target, cat, n)?;
        return Ok(
            !(ignores_type_immunity(c, &m) || target_immunity_removed(c, target, &m))
                && (immune
                    || abilities::damage_immune_from(c, &source, target, &m)
                    || extra::source_immune(c, &source, target, n)),
        );
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
        "ordinary" | "damage-rules" | "damage-context" => {}
        "history-gated" => {
            history_effect(c, q, d, cat)?;
        }
        "support-stat" => {
            support_stat(c, q, d, cat)?;
        }
        "stat" => {
            return damaging_stats::effect(c, q, _dice, cat);
        }
        "stack-stat" => {
            if !immune_target(c, q, cat)? && !(provenance::enabled(c) && q.sheer_force) {
                let source = reviewed::source(c, q)?;
                let target = fighter(c, &q.target_id)?;
                let chance = d["dice"].as_u64().unwrap() > 0;
                if !chance
                    || (_dice.contains(&6)
                        && !(source.side != target.side
                            && target.has_ability("Shield Dust")
                            && !reviewed::extreme(c, &source)))
                {
                    let recipient = if d["user"] == true {
                        &q.actor_id
                    } else {
                        &q.target_id
                    };
                    for key in d["keys"].as_array().unwrap() {
                        extra::stack_stat(
                            c,
                            &q.actor_id,
                            recipient,
                            key.as_str().unwrap(),
                            d["amount"].as_i64().unwrap() as i32,
                            &q.move_name,
                        )?;
                    }
                }
            }
        }
        "accuracy" => {
            accuracy_effect(c, q, d, _dice, cat)?;
        }
        "field-cleanse" => {
            if !immune_target_with_fainted(c, q, cat, true)? {
                if d["spin"] == true {
                    extra::change_stat(c, &q.actor_id, &q.actor_id, "Dexterity", 1)?;
                    compound::cleanse(c, &q.actor_id)?;
                } else {
                    flow_mut(c)
                        .battlefield
                        .rules
                        .retain(|r| !r.kind.ends_with("Terrain"));
                    flow_mut(c)
                        .field_effects
                        .retain(|r| !r.move_name.ends_with("Terrain"));
                    flow_mut(c).battlefield.events.retain(|e| {
                        !["terrain-ruling", "terrain-contest"].contains(&e.kind.as_str())
                    });
                }
            }
        }
        "cost-stat" => {
            if !immune_target(c, q, cat)? {
                let loss = if d["damage_mode"].is_string() {
                    rules::successes(_dice)
                } else {
                    d["flat_damage"].as_i64().unwrap_or(0) as i32
                };
                if loss > 0 {
                    field::hurt(c, &q.actor_id, loss, "Neutral", cat)?;
                }
                for (key, n) in d["changes"].as_object().unwrap() {
                    extra::change_stat(
                        c,
                        &q.actor_id,
                        &q.target_id,
                        key,
                        n.as_i64().unwrap() as i32,
                    )?;
                }
                if d["block"] == true {
                    install(c, q, &q.actor_id, &q.target_id)?;
                }
            }
        }
        "restriction" => {
            if !immune_target(c, q, cat)? {
                let name = q.move_name.clone();
                let started = battle(c)?.round;
                flow_mut(c).special.effects.retain(|e| {
                    e.name != name
                        || e.target != q.target_id
                        || !["move-restriction", "persistent-restriction"]
                            .contains(&e.slot.as_str())
                });
                flow_mut(c).special.effects.push(special::TimedEffect {
                    name,
                    source: q.actor_id.clone(),
                    target: q.target_id.clone(),
                    rounds: d["rounds"].as_u64().map(|n| n as u32),
                    amount: 0,
                    started,
                    slot: if d["persistent"] == true {
                        "persistent-restriction".into()
                    } else {
                        "move-restriction".into()
                    },
                });
            }
        }
        "bypass-attack" => {
            if !immune_target_with_fainted(c, q, cat, true)? {
                for (key, n) in d["changes"].as_object().unwrap() {
                    extra::change_stat(
                        c,
                        &q.actor_id,
                        &q.actor_id,
                        key,
                        n.as_i64().unwrap() as i32,
                    )?;
                }
            }
        }
        "reach-attack" => {
            if d["grounding"] == true && !immune_target(c, q, cat)? {
                let name = format!("grounded:{}", q.move_name);
                let started = battle(c)?.round;
                flow_mut(c)
                    .special
                    .effects
                    .retain(|e| e.target != q.target_id || e.name != name);
                flow_mut(c).special.effects.push(special::TimedEffect {
                    name,
                    source: q.actor_id.clone(),
                    target: q.target_id.clone(),
                    rounds: None,
                    amount: 0,
                    started,
                    slot: "ground-immunity".into(),
                });
            }
        }
        "complete-existing" => {
            if d["mode"] == "item" && !items::resolved_effects(c, q) {
                return Err(error("This item move needs its committed accuracy/item context before effect resolution"));
            }
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
    definition(c, name).is_some_and(|(kind, d)| {
        [
            "ordinary",
            "damage-rules",
            "damage-context",
            "history-gated",
        ]
        .contains(&kind)
            || (kind == "bypass-attack" && d["changes"].as_object().is_some_and(|v| v.is_empty()))
            || (kind == "reach-attack" && d["grounding"] != true)
            || (kind == "complete-existing"
                && (d["mode"] == "ordinary"
                    || flow(c).is_ok_and(|f| {
                        f.queue
                            .first()
                            .is_some_and(|q| q.move_name == name && items::resolved_effects(c, q))
                    })))
    })
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

pub fn physical_defense(c: &Campaign, m: &serde_json::Value) -> bool {
    definition(c, m["Name"].as_str().unwrap_or(""))
        .is_some_and(|(_, d)| d["physical_defense"] == true)
}
pub fn move_data(c: &Campaign, p: &Combatant, m: &mut serde_json::Value) {
    contextual_type(c, p, m);
    if definition(c, m["Name"].as_str().unwrap_or(""))
        .is_some_and(|(_, d)| d["ignore_defense"] == true)
    {
        m["Attributes"]["IgnoreDefenses"] = true.into();
    }
}

pub fn accuracy_loss(c: &Campaign, id: &str) -> i32 {
    flow(c).map_or(0, |f| {
        f.special
            .effects
            .iter()
            .filter(|e| e.target == id && e.name.starts_with("accuracy-loss:"))
            .map(|e| e.amount)
            .max()
            .unwrap_or(0)
    })
}
fn accuracy_effect(
    c: &mut Campaign,
    q: &QueuedAction,
    d: &serde_json::Value,
    dice: &[u8],
    cat: &Catalog,
) -> Result<()> {
    if immune_target(c, q, cat)? || (provenance::enabled(c) && q.sheer_force) {
        return Ok(());
    }
    let source = reviewed::source(c, q)?;
    let target = fighter(c, &q.target_id)?.clone();
    let chance = d["dice"].as_u64().unwrap() > 0;
    let hostile = source.side != target.side;
    let bypass = hostile && reviewed::extreme(c, &source);
    if chance && (!dice.contains(&6) || (hostile && target.has_ability("Shield Dust") && !bypass)) {
        return Ok(());
    }
    let mut recipient = q.target_id.clone();
    let mut caster = q.actor_id.clone();
    if hostile && target.has_ability("Mirror Armor") && !house::ring(c, &target) {
        if bypass {
            return Err(AppError::new(
                "manual_rule",
                "Conflicting unique Mirror Armor and Teravolt/Turboblaze require a GM ruling",
            ));
        }
        recipient = q.actor_id.clone();
        caster = q.target_id.clone();
    }
    if recipient != caster && items::completion::has(c, &recipient, "Clear Amulet") && !bypass {
        return Ok(());
    }
    let name = format!("accuracy-loss:{}", q.move_name);
    let amount = d["amount"].as_i64().unwrap() as i32;
    let started = battle(c)?.round;
    flow_mut(c)
        .special
        .effects
        .retain(|e| e.name != name || e.target != recipient);
    flow_mut(c).special.effects.push(special::TimedEffect {
        name,
        source: caster,
        target: recipient,
        rounds: None,
        amount,
        started,
        slot: "accuracy-success".into(),
    });
    Ok(())
}

pub fn applied_accuracy_loss(c: &Campaign, id: &str) -> i32 {
    if reviewed::enabled(c)
        && fighter(c, id).is_ok_and(|p| reviewed::effective_subject(c, p).has_ability("Keen Eye"))
    {
        0
    } else {
        accuracy_loss(c, id)
    }
}

pub fn stack_values(c: &Campaign, p: &Combatant, key: &str) -> Vec<i32> {
    flow(c).map_or_else(
        |_| vec![],
        |f| {
            f.special
                .effects
                .iter()
                .filter(|e| {
                    e.target == p.id
                        && e.slot == "stacking-stat"
                        && e.name.ends_with(&format!(":{key}"))
                })
                .map(|e| match p.ability_for(&["Contrary", "Simple"]) {
                    "Contrary" => -e.amount,
                    "Simple" => 2 * e.amount,
                    _ => e.amount,
                })
                .collect()
        },
    )
}

pub fn guard_move(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> Result<()> {
    restriction_guard(c, p, m)?;
    history_guard(c, p, m)?;
    if definition(c, m["Name"].as_str().unwrap_or(""))
        .is_some_and(|(_, d)| d["requires_terrain"] == true)
        && !flow(c)?
            .battlefield
            .rules
            .iter()
            .any(|r| r.kind.ends_with("Terrain"))
    {
        return Err(error("Steel Roller requires an active Terrain"));
    }
    Ok(())
}

fn terrain_type(c: &Campaign, p: &Combatant) -> Option<&'static str> {
    flow(c)
        .ok()?
        .battlefield
        .rules
        .iter()
        .rev()
        .filter(|r| field::applies(&r.scope, p))
        .find_map(|r| match r.kind.as_str() {
            "Electric Terrain" => Some("Electric"),
            "Grassy Terrain" => Some("Grass"),
            "Misty Terrain" => Some("Fairy"),
            "Psychic Terrain" => Some("Psychic"),
            _ => None,
        })
}
fn contextual_type(c: &Campaign, p: &Combatant, m: &mut serde_json::Value) {
    let Some((_, d)) = definition(c, m["Name"].as_str().unwrap_or("")) else {
        return;
    };
    if let Some(choice) = d["source_type"].as_str() {
        let types = field::cached_types(c, p);
        if let Some(t) = if choice == "secondary" {
            types.get(1).or(types.first())
        } else {
            types.first()
        } {
            m["Type"] = t.clone().into();
        }
    }
    if d["terrain_type"] == true {
        if let Some(t) = terrain_type(c, p) {
            m["Type"] = t.into();
        }
    }
}
pub fn damage_bonus(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> i32 {
    let Some((_, d)) = definition(c, m["Name"].as_str().unwrap_or("")) else {
        return 0;
    };
    let mut n = 0;
    if terrain_type(c, p).is_some() {
        n += d["terrain_bonus"].as_i64().unwrap_or(0) as i32;
    }
    if field::has(c, "Electric Terrain", p) {
        n += d["electric_terrain_bonus"].as_i64().unwrap_or(0) as i32;
    }
    if !items::held(c, &p.id).is_empty() {
        n += d["held_penalty"].as_i64().unwrap_or(0) as i32;
    }
    n
}

fn restriction_guard(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> Result<()> {
    let Some(f) = c.battle.as_ref().and_then(|b| b.flow.as_ref()) else {
        return Ok(());
    };
    for e in &f.special.effects {
        if !["move-restriction", "persistent-restriction"].contains(&e.slot.as_str()) {
            continue;
        }

        if e.target == p.id && m["Type"] != "Typeless" {
            if e.name
                .strip_prefix("Disable:")
                .is_some_and(|n| m["Name"] == n)
            {
                return Err(error("This move is Disabled"));
            }
            if e.name == "Torment"
                && f.completion.uses.iter().any(|u| {
                    u.actor == p.id
                        && m["Name"] == u.name
                        && u.round + 1 == battle(c).unwrap().round
                })
            {
                return Err(error(
                    "Torment prevents moves performed during the last round",
                ));
            }
        }
        if e.target == p.id
            && e.name == "Taunt"
            && m["Category"] == "Support"
            && m["Type"] != "Typeless"
        {
            return Err(error(
                "Taunt prevents Support moves; Support Maneuvers remain available",
            ));
        }
        if e.target == p.id && e.name == "Throat Chop" && m["Attributes"]["SoundMove"] == true {
            return Err(error("Throat Chop prevents Sound moves for this scene"));
        }
        if e.name == "Imprison"
            && m["Type"] != "Typeless"
            && special::registered(c, &e.source).is_ok_and(|source| {
                source.deployed
                    && source.hp > 0
                    && source.side != p.side
                    && rules::effective_sheet(c, source)
                        .is_ok_and(|s| s.moves.iter().any(|n| m["Name"] == n.as_str()))
            })
        {
            return Err(error(
                "Imprison prevents a foe from using this active caster's known move",
            ));
        }
    }
    Ok(())
}
pub fn clear_restriction(c: &mut Campaign, id: &str, name: &str) {
    if flow(c).is_ok_and(|f| f.completion.blocks.contains("MOVE-A15")) {
        flow_mut(c).special.effects.retain(|e| {
            e.target != id
                || e.name != name
                || !["move-restriction", "persistent-restriction"].contains(&e.slot.as_str())
        });
    }
}

pub fn observe(before: &Campaign, next: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !flow(next).is_ok_and(|f| f.completion.blocks.contains("MOVE-A16")) {
        return Ok(());
    }
    let after = battle(next)?;
    if !after.active {
        return Ok(());
    }
    let old = before.battle.as_ref().filter(|b| b.id == after.id);
    let round = after.round;
    let entered: Vec<String> = after
        .combatants
        .iter()
        .filter(|p| {
            p.deployed
                && p.hp > 0
                && !old.is_some_and(|b| b.combatants.iter().any(|v| v.id == p.id && v.deployed))
        })
        .map(|p| p.id.clone())
        .collect();
    let mut uses = vec![];
    for p in &after.combatants {
        let previous = old
            .filter(|b| b.round == round)
            .and_then(|b| b.combatants.iter().find(|v| v.id == p.id));
        let prior = previous.map_or(&[][..], |v| v.used_moves.as_slice());
        if p.used_moves.starts_with(prior) {
            for name in &p.used_moves[prior.len()..] {
                if cat
                    .find("moves", "Name", name)
                    .is_some_and(|m| m["Type"] != "Typeless")
                {
                    uses.push(CompletionMoveUse {
                        actor: p.id.clone(),
                        name: name.clone(),
                        round,
                    });
                }
            }
        }
    }
    let f = flow_mut(next);
    for id in entered {
        f.completion.entered.insert(id, round);
    }
    f.completion.uses.extend(uses);
    Ok(())
}
fn history_guard(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> Result<()> {
    let Some(("history-gated", d)) = definition(c, m["Name"].as_str().unwrap_or("")) else {
        return Ok(());
    };
    match d["mode"].as_str().unwrap() {
        "previous-round"
            if flow(c)?.completion.uses.iter().any(|u| {
                u.actor == p.id && m["Name"] == u.name && u.round + 1 == battle(c).unwrap().round
            }) =>
        {
            return Err(error(
                "This move fails because it was performed during the previous round",
            ))
        }
        _ => {}
    }
    Ok(())
}
pub fn target_guard(c: &Campaign, id: &str, m: &serde_json::Value) -> Result<()> {
    if definition(c, m["Name"].as_str().unwrap_or(""))
        .is_some_and(|(kind, d)| kind == "history-gated" && d["mode"] == "disable")
        && !flow(c)?.completion.uses.iter().any(|u| u.actor == id)
    {
        return Err(error(
            "Disable requires a previously performed Pokemon move",
        ));
    }
    Ok(())
}
fn history_effect(
    c: &mut Campaign,
    q: &QueuedAction,
    d: &serde_json::Value,
    cat: &Catalog,
) -> Result<()> {
    let mode = d["mode"].as_str().unwrap();
    if !["disable", "torment"].contains(&mode) || immune_target(c, q, cat)? {
        return Ok(());
    }
    let name = if mode == "disable" {
        let last = flow(c)?
            .completion
            .uses
            .iter()
            .rev()
            .find(|u| u.actor == q.target_id)
            .ok_or_else(|| error("Disable requires a previously performed Pokemon move"))?;
        format!("Disable:{}", last.name)
    } else {
        "Torment".into()
    };
    flow_mut(c).special.effects.retain(|e| {
        e.target != q.target_id
            || if mode == "disable" {
                !e.name.starts_with("Disable:")
            } else {
                e.name != "Torment"
            }
    });
    let started = battle(c)?.round;
    flow_mut(c).special.effects.push(special::TimedEffect {
        name,
        source: q.actor_id.clone(),
        target: q.target_id.clone(),
        rounds: if mode == "torment" { Some(4) } else { None },
        amount: 0,
        started,
        slot: "move-restriction".into(),
    });
    Ok(())
}

pub fn force_field_bypass(c: &Campaign, m: &serde_json::Value) -> bool {
    definition(c, m["Name"].as_str().unwrap_or("")).is_some_and(|(_, d)| d["force_field"] == true)
}

pub fn ignores_type_immunity(c: &Campaign, m: &serde_json::Value) -> bool {
    definition(c, m["Name"].as_str().unwrap_or("")).is_some_and(|(_, d)| {
        d["ignore_type_immunity"]
            .as_str()
            .is_some_and(|t| m["Type"] == t)
    })
}
pub fn target_immunity_removed(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> bool {
    m["Type"] == "Ground" && grounded(c, p)
}
pub fn grounded(c: &Campaign, p: &Combatant) -> bool {
    p.deployed
        && p.hp > 0
        && flow(c).is_ok_and(|f| {
            f.special
                .effects
                .iter()
                .any(|e| e.target == p.id && e.slot == "ground-immunity")
        })
}
pub fn hits_hidden(c: &Campaign, name: &str, hidden: &str) -> bool {
    definition(c, name).is_some_and(|(_, d)| {
        d["hidden_any"] == true
            || d["hidden"]
                .as_array()
                .is_some_and(|v| v.iter().any(|n| n == hidden))
    })
}
pub fn vulnerable_hidden(c: &Campaign, name: &str, hidden: &str) -> bool {
    definition(c, name).is_some_and(|(_, d)| {
        d["vulnerable_hidden"]
            .as_array()
            .is_some_and(|v| v.iter().any(|n| n == hidden))
    })
}
