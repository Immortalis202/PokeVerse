//! Versioned, saved block adoption for the remaining exact-name move definitions.
use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct MoveCompletionState {
    pub judgment_prepared: BTreeMap<String, judgment::JudgmentProfile>,
    pub judgment_casts: BTreeMap<u32, judgment::JudgmentCast>,
    pub metronome_table: Option<copies::MetronomeTable>,
    pub metronome_draw: Option<copies::MetronomeDraw>,
    pub copied_cast: Option<copies::CopiedCast>,
    pub hold_hands: BTreeMap<u32, choices::HoldHandsDecision>,
    /// Scene base replacements from splits and swaps, by combatant and key.
    pub stat_bases: BTreeMap<String, BTreeMap<String, i32>>,
    /// Saved owner choices for the current cast, by queue sequence.
    pub move_options: BTreeMap<u32, String>,
    /// Choices the current cast still offers, refreshed after every transition.
    pub open_options: BTreeMap<u32, Vec<String>>,
    /// Spectral Thief casts that already stole before damage.
    pub stolen: BTreeSet<u32>,
    /// Conversion 2 types each combatant may still choose after being hit.
    pub conversion_choices: BTreeMap<String, Vec<String>>,
    /// Moves each combatant has performed with a successful Accuracy roll this battle.
    pub performed: BTreeMap<String, BTreeSet<String>>,
    /// Whether each combatant's last move Accuracy roll succeeded.
    pub last_accuracy: BTreeMap<String, bool>,
    /// Round in which each combatant last had an Attribute or Trait lowered.
    pub lowered: BTreeMap<String, u32>,
    /// Round in which each combatant last lost HP.
    pub damaged: BTreeMap<String, u32>,
    /// Targets already hit by Ruination this battle.
    pub ruined: BTreeSet<String>,
    /// Temper Flare/Stomping Tantrum casts following a failed Accuracy roll.
    pub tantrum: BTreeSet<u32>,
    /// After You (first) and Quash (last) initiative marks, by target.
    pub initiative: BTreeMap<String, initiative_ops::InitiativeMark>,
    /// Successful Ambushes awaiting or having taken their surprise action, by user.
    pub ambush: BTreeMap<String, initiative_ops::AmbushState>,
    /// Moves declared this round and last round, for Encore.
    pub move_log: BTreeMap<String, Vec<initiative_ops::LoggedMove>>,
    /// Round in which each Encored combatant must repeat its previous round.
    pub encored: BTreeMap<String, u32>,
    /// Round in which each user last cast Encore.
    pub encore_casts: BTreeMap<String, u32>,
    /// Sucker Punch/Thunderclap timing choices ("early"/"late"), by "combatant:move".
    pub reaction_timing: BTreeMap<String, String>,
    /// Evasions taken this round by Double Team/Minimize users: (round, count).
    pub evasions: BTreeMap<String, (u32, u32)>,
    /// Helping Hand bonuses still waiting for the next Accuracy and damage rolls.
    pub helping_accuracy: BTreeSet<String>,
    pub helping_damage: BTreeSet<String>,
    /// Help Another dice for the next Accuracy roll: (dice, helpers).
    pub help_dice: BTreeMap<String, (u32, u32)>,
    /// Allies who spent an action joining each Beat Up cast.
    pub beat_up: BTreeMap<u32, Vec<String>>,
    /// Combatants whose Round this round sings together with an ally.
    pub round_chorus: BTreeMap<String, u32>,
    /// Future Sight/Doom Desire damage waiting for the next round.
    pub delayed: Vec<delayed_ops::DelayedHit>,
    /// Round in which each user used Rage / Destiny Bond.
    pub rage: BTreeMap<String, u32>,
    pub destiny_bond: BTreeMap<String, u32>,
    pub destiny_casts: BTreeMap<String, u32>,
    /// Targets hit by Pursuit, with the Pursuit user.
    pub pursued: BTreeMap<String, String>,
    /// Glaive Rush users vulnerable to the next damaging move.
    pub glaive: BTreeSet<String>,
    /// Powdered combatants, and Powder damage dice owed after declaring a Fire move.
    pub powdered: BTreeSet<String>,
    pub powder_due: BTreeMap<String, u32>,
    /// Embargoed combatants: no items may be used on them this scene.
    pub embargoed: BTreeSet<String>,
    /// Memento reductions that persist across switching, by target.
    pub memento: BTreeMap<String, Vec<String>>,
    /// Pay Day / Make It Rain money owed at battle end.
    pub money: Vec<scene_ops::MoneyAward>,
    /// Moves Mimic currently replaces itself with, by user.
    pub mimicked: BTreeMap<String, String>,
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
        hold_hands: BTreeMap::new(),
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
                + 2 * u32::from(source.has_ability("Serene Grace"))
                + field_ops::water_pledge(c))
            .saturating_sub(2 * u32::from(!bypass && target.has_ability("Wonder Skin"))),
        ));
    }
    if let Some(n) = stat_ops::effect_dice(c, q)
        .or_else(|| type_ops::effect_dice(c, q))
        .or_else(|| damage_ops::effect_dice(c, q))
        .or_else(|| trigger_ops::effect_dice(c, q))
        .or_else(|| scene_ops::effect_dice(c, q))
    {
        return Ok(Some(n));
    }
    if let Some(n) = initiative_ops::effect_dice(c, q)? {
        return Ok(Some(n));
    }
    Ok(definition(c, &q.move_name).map(|_| 0))
}
/// Owner choices the current cast needs before its effect resolves.
pub fn options(c: &Campaign, q: &QueuedAction) -> Result<Vec<String>> {
    let mut options = stat_ops::options(c, q)?;
    options.extend(type_ops::options(c, q)?);
    options.extend(damage_ops::options(c, q)?);
    options.extend(trigger_ops::options(c, q)?);
    options.extend(scene_ops::options(c, q)?);
    Ok(options)
}
pub(super) fn immune_target(c: &Campaign, q: &QueuedAction, cat: &Catalog) -> Result<bool> {
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
        let (n, immune) = rules::move_matchup(c, &source, target, &m, cat)?;
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
    install_named(c, &q.move_name, source, target)
}
pub(super) fn install_named(
    c: &mut Campaign,
    move_name: &str,
    source: &str,
    target: &str,
) -> Result<()> {
    let p = fighter(c, target)?;
    if p.hp == 0 || (p.has_ability("Run Away") && !house::ring(c, p)) {
        return Ok(());
    }
    let name = format!("move-block:{}", move_name);
    if flow(c)?
        .special
        .effects
        .iter()
        .any(|e| e.name == name && e.target == target)
    {
        return Ok(());
    }
    let started = battle(c)?.round;
    let (_, d) = definition(c, move_name).unwrap();
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
        "judgment" => return Err(error("Judgment requires its saved GM added-effect ruling after damage")),
        "metronome" => return Err(error("Metronome requires a committed category/Power draw and GM template selection before casting")),
        "hold-hands" => {
            if !immune_target(c, q, cat)? {
                return Err(error(
                    "Hold Hands requires the saved target resistance and GM ruling workflow",
                ));
            }
        }
        "ordinary" | "damage-rules" | "damage-context" => {}
        "history-gated" => {
            history_effect(c, q, d, cat)?;
        }
        "vulnerability-support" => {
            if !immune_target(c, q, cat)? {
                extra::change_stat(c, &q.actor_id, &q.target_id, "Dexterity", -1)?;
                let started = battle(c)?.round;
                flow_mut(c)
                    .special
                    .effects
                    .retain(|e| e.name != "Tar Shot" || e.target != q.target_id);
                flow_mut(c).special.effects.push(special::TimedEffect {
                    name: "Tar Shot".into(),
                    source: q.actor_id.clone(),
                    target: q.target_id.clone(),
                    rounds: None,
                    amount: 0,
                    started,
                    slot: "fire-vulnerability".into(),
                });
            }
        }
        "entry-hazard" => {
            entry_hazards::effect(c, q)?;
        }
        "battlefield-support" => {
            let scope = if d["scope"] == "side" {
                format!("side:{}", fighter(c, &q.actor_id)?.side)
            } else {
                "all".into()
            };
            // RULING p.77: an active round-duration move cannot restart its counter.
            if !flow(c)?
                .battlefield
                .rules
                .iter()
                .any(|r| r.kind == q.move_name && r.scope == scope)
            {
                field::insert(
                    c,
                    &q.move_name,
                    &format!("field-user:{}", q.actor_id),
                    &scope,
                    Some(d["rounds"].as_u64().unwrap() as u32),
                    1,
                );
            }
        }
        "cover-stat" => {
            if fighter(c, &q.actor_id)?.hp > 0 && !immune_target_with_fainted(c, q, cat, true)? {
                let prefix = format!("order-up-cast:{}:", q.sequence);
                let key = flow(c)?
                    .special
                    .effects
                    .iter()
                    .find(|e| {
                        e.name == q.move_name
                            && e.target == q.actor_id
                            && e.slot.starts_with(&prefix)
                    })
                    .and_then(|e| e.slot.strip_prefix(&prefix))
                    .map(str::to_owned);
                if let Some(key) = key {
                    extra::change_stat(c, &q.actor_id, &q.actor_id, &key, 1)?;
                }
            }
        }
        "electrify" => {
            if !immune_target(c, q, cat)? {
                flow_mut(c).special.effects.retain(|e| {
                    e.name != q.move_name || e.target != q.target_id || e.slot != "self-support"
                });
                let started = battle(c)?.round;
                flow_mut(c).special.effects.push(special::TimedEffect {
                    name: q.move_name.clone(),
                    source: q.actor_id.clone(),
                    target: q.target_id.clone(),
                    rounds: None,
                    amount: 0,
                    started,
                    slot: "self-support".into(),
                });
            }
        }
        "type-conversion" => {
            if d["field"] == true {
                if !flow(c)?
                    .battlefield
                    .rules
                    .iter()
                    .any(|r| r.kind == q.move_name && r.scope == "all")
                {
                    field::insert(
                        c,
                        &q.move_name,
                        &format!("field-user:{}", q.actor_id),
                        "all",
                        None,
                        0,
                    );
                }
            } else if !immune_target_with_fainted(c, q, cat, true)? {
                flow_mut(c)
                    .special
                    .effects
                    .retain(|e| e.name != q.move_name || e.target != q.actor_id);
                let started = battle(c)?.round;
                flow_mut(c).special.effects.push(special::TimedEffect {
                    name: q.move_name.clone(),
                    source: q.actor_id.clone(),
                    target: q.actor_id.clone(),
                    rounds: None,
                    amount: 0,
                    started,
                    slot: "self-support".into(),
                });
            }
        }
        "type-change" => {
            let source = reviewed::source(c, q)?;
            let target = fighter(c, &q.target_id)?;
            let grass = field::types(c, target, cat)?.iter().any(|t| t == "Grass")
                && !reviewed::type_bypass(c, &source, target);
            if !immune_target(c, q, cat)? && !grass {
                field::change_type(
                    c,
                    &q.target_id,
                    "replace",
                    &[d["type"].as_str().unwrap().into()],
                )?;
                let scope = format!("combatant:{}", q.target_id);
                for r in &mut flow_mut(c).battlefield.rules {
                    if r.scope == scope && r.kind.starts_with("typing:") && r.source == "typing" {
                        r.source = "completion-type".into();
                    }
                }
            }
        }
        "detection" => {
            if !immune_target(c, q, cat)? {
                if d["rounds"].as_u64().is_some()
                    && flow(c)?
                        .special
                        .effects
                        .iter()
                        .any(|e| e.name == q.move_name && e.target == q.target_id)
                {
                    return Ok(true);
                }
                let started = battle(c)?.round;
                flow_mut(c).special.effects.retain(|e| {
                    e.name != q.move_name
                        || e.target != q.target_id
                        || (d["detection"] == true && e.source != q.actor_id)
                });
                flow_mut(c).special.effects.push(special::TimedEffect {
                    name: q.move_name.clone(),
                    source: q.actor_id.clone(),
                    target: q.target_id.clone(),
                    rounds: d["rounds"].as_u64().map(|n| n as u32),
                    amount: 0,
                    started,
                    slot: "detection".into(),
                });
            }
        }
        "stat-reset" => {
            if d["field"] == true {
                let ids: Vec<_> = battle(c)?
                    .combatants
                    .iter()
                    .filter(|p| p.deployed && p.hp > 0)
                    .map(|p| p.id.clone())
                    .collect();
                for id in ids {
                    reset_stats(c, &id, true)?;
                }
            } else if !immune_target(c, q, cat)? {
                reset_stats(c, &q.target_id, d["traits"] == true)?;
            }
        }
        "support-stat" => {
            support_stat(c, q, d, cat)?;
        }
        "stat-op" => {
            stat_ops::effect(c, q, _dice, d, cat)?;
        }
        "type-op" => {
            type_ops::effect(c, q, _dice, cat)?;
        }
        "damage-op" => {
            damage_ops::effect(c, q, _dice, cat)?;
        }
        "condition-op" => {
            condition_ops::effect(c, q)?;
        }
        "initiative-op" => {
            initiative_ops::effect(c, q, _dice, cat)?;
        }
        "allied-op" => {
            allied_ops::effect(c, q, cat)?;
        }
        "delayed-op" => {
            delayed_ops::effect(c, q)?;
        }
        "trigger-op" => {
            trigger_ops::effect(c, q, _dice, cat)?;
        }
        "field-op" => {
            field_ops::effect(c, q, cat)?;
        }
        "item-op" => {
            item_ops::effect(c, q, cat)?;
        }
        "scene-op" => {
            scene_ops::effect(c, q, _dice, cat)?;
        }
        "copy-op" => {
            copy_ops::effect(c, q, cat)?;
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
        "partner-attack" => {
            if let Some(kind) = d["field"].as_str() {
                if q.damage > 0 && !immune_target_with_fainted(c, q, cat, true)? {
                    let scope = format!("side:{}", fighter(c, &q.actor_id)?.side);
                    flow_mut(c)
                        .battlefield
                        .rules
                        .retain(|r| r.kind != kind || r.scope != scope);
                    field::insert(c, kind, &format!("move:{}", q.sequence), &scope, Some(4), 1);
                }
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
    if definition(c, "Fairy Lock").is_some() && field::has(c, "Fairy Lock", p) {
        return true;
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
    if let Some(d) = field_ops::tick_definition(c, name) {
        return d["dice"].as_u64().map(|n| n as u32);
    }
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
    let d = field_ops::tick_definition(c, name).unwrap_or_else(|| definition(c, name).unwrap().1);
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
    let n = if let Some((_, d)) = definition(c, m["Name"].as_str().unwrap_or("")) {
        if matches_types(c, target, cat, &d["super_effective_types"])? {
            1
        } else {
            base
        }
    } else {
        base
    };
    let n = damage_ops::effectiveness(c, m, target, cat, n)?;
    Ok(sport_effectiveness(
        c,
        m,
        typed_effectiveness(c, m, target, n),
    ))
}
/// Sports alter the printed Power and positive effectiveness, not total damage.
pub fn sport(c: &Campaign, m: &serde_json::Value) -> bool {
    if !["Physical", "Special"].contains(&m["Category"].as_str().unwrap_or(""))
        || !field::enabled(c)
        || forms::zero_active(c)
    {
        return false;
    }
    let name = match m["Type"].as_str().unwrap_or("") {
        "Electric" => "Mud Sport",
        "Fire" => "Water Sport",
        _ => return false,
    };
    definition(c, name).is_some()
        && flow(c).is_ok_and(|f| {
            f.battlefield
                .rules
                .iter()
                .any(|r| r.kind == name && r.scope == "all")
        })
}
pub fn sport_effectiveness(c: &Campaign, m: &serde_json::Value, n: i32) -> i32 {
    if sport(c, m) {
        n.min(0)
    } else {
        n
    }
}
pub fn final_move_data(c: &Campaign, p: &Combatant, m: &mut serde_json::Value) {
    if ["Physical", "Special"].contains(&m["Category"].as_str().unwrap_or(""))
        && electrified(c, p, m)
    {
        m["Type"] = "Electric".into();
    }
    if m["Type"] == "Normal"
        && (field::normal_electric(c, &p.id)
            || (definition(c, "Ion Deluge").is_some()
                && field::has(c, "Ion Deluge", p)
                && ["Physical", "Special"].contains(&m["Category"].as_str().unwrap_or(""))))
    {
        m["Type"] = "Electric".into();
    }
    if sport(c, m) {
        m["Power"] = 0.into();
    }
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
            || (kind == "partner-attack" && d["field"].is_null())
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
            if let Some(types) = d["stat_types"].as_array() {
                let actual = field::types(c, fighter(c, &target)?, cat)?;
                if !types.iter().any(|t| actual.iter().any(|v| t == v.as_str())) {
                    continue;
                }
            }

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
        if d["auxiliary"] == true {
            let started = battle(c)?.round;
            flow_mut(c)
                .special
                .effects
                .retain(|e| e.name != q.move_name || e.target != target);
            flow_mut(c).special.effects.push(special::TimedEffect {
                name: q.move_name.clone(),
                source: q.actor_id.clone(),
                target: target.clone(),
                rounds: None,
                amount: 0,
                started,
                slot: "self-support".into(),
            });
        }
    }
    Ok(())
}

pub fn physical_defense(c: &Campaign, m: &serde_json::Value) -> bool {
    definition(c, m["Name"].as_str().unwrap_or(""))
        .is_some_and(|(_, d)| d["physical_defense"] == true)
}
pub fn move_data(c: &Campaign, p: &Combatant, m: &mut serde_json::Value) {
    damage_ops::move_data(c, p, m);
    initiative_ops::move_data(c, p, m);
    scene_ops::move_data(c, p, m);
    if ["Physical", "Special"].contains(&m["Category"].as_str().unwrap_or(""))
        && self_support(c, p, "Dragon Cheer")
    {
        m["Attributes"]["HighCritical"] = true.into();
    }
    if let Some(terrain) = definition(c, m["Name"].as_str().unwrap_or(""))
        .and_then(|(_, d)| d["reaction_terrain"].as_str())
    {
        if field::has(c, terrain, p) {
            m["Attributes"]["Reaction"] = "1".into();
        } else if let Some(attrs) = m["Attributes"].as_object_mut() {
            attrs.remove("Reaction");
        }
    }
    contextual_type(c, p, m);
    if let Some(target) = definition(c, m["Name"].as_str().unwrap_or(""))
        .and_then(|(_, d)| d["target_override"].as_str())
    {
        m["Target"] = target.into();
    }
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
                .chain(scene_ops::memento_values(c, &p.id, key))
                .collect()
        },
    )
}

pub fn guard_move(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> Result<()> {
    restriction_guard(c, p, m)?;
    history_guard(c, p, m)?;
    condition_ops::guard(c, p, m)?;
    allied_ops::guard(c, m)?;
    delayed_ops::guard(c, p, m)?;
    item_ops::guard(c, p, m)?;
    copy_ops::guard(c, p, m)?;
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
    let mut n = i32::from(abilities::contact(m) && self_support(c, p, "Victory Dance"))
        + condition_ops::damage_bonus(c, p, m)
        + allied_ops::damage_bonus(c, p, m);
    if m["Type"] == "Electric"
        && ["Physical", "Special"].contains(&m["Category"].as_str().unwrap_or(""))
        && definition(c, "Charge").is_some()
    {
        let cast = flow(c)
            .ok()
            .and_then(|f| f.queue.first())
            .filter(|q| q.actor_id == p.id && m["Name"] == q.move_name)
            .is_some_and(|q| {
                flow(c).unwrap().special.effects.iter().any(|e| {
                    e.name == "Charge"
                        && e.target == p.id
                        && e.slot == format!("charge-cast:{}", q.sequence)
                })
            });
        if cast || self_support(c, p, "Charge") {
            n += 2;
        }
    }
    let Some((_, d)) = definition(c, m["Name"].as_str().unwrap_or("")) else {
        return n;
    };
    if m["Name"] == "Misty Explosion" && field::has(c, "Misty Terrain", p) {
        n += 4;
    }
    if d["happiness_bonus"] == true {
        n += c.sheets[&p.sheet_id]
            .workflow
            .as_ref()
            .map_or(2, |w| w.happiness)
            .clamp(0, 5);
    }
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
    if flow(next).is_ok_and(|f| f.completion.blocks.contains("MOVE-A48"))
        && battle(next).is_ok_and(|b| b.active)
    {
        stat_ops::lifecycle(next)?;
        stat_ops::before_damage(next, cat)?;
    }
    if flow(next).is_ok_and(|f| f.completion.blocks.contains("MOVE-A49"))
        && battle(next).is_ok_and(|b| b.active)
    {
        type_ops::lifecycle(next)?;
    }
    if flow(next).is_ok_and(|f| f.completion.blocks.contains("MOVE-A50"))
        && battle(next).is_ok_and(|b| b.active)
    {
        damage_ops::lifecycle(next)?;
    }
    if flow(next).is_ok_and(|f| f.completion.blocks.contains("MOVE-A51")) {
        condition_ops::observe(before, next)?;
    }
    if flow(next).is_ok_and(|f| f.completion.blocks.contains("MOVE-A52"))
        && battle(next).is_ok_and(|b| b.active)
    {
        initiative_ops::lifecycle(next)?;
    }
    if flow(next).is_ok_and(|f| f.completion.blocks.contains("MOVE-A53"))
        && battle(next).is_ok_and(|b| b.active)
    {
        allied_ops::lifecycle(next)?;
    }
    if flow(next).is_ok_and(|f| f.completion.blocks.contains("MOVE-A54"))
        && battle(next).is_ok_and(|b| b.active)
    {
        delayed_ops::observe(before, next)?;
        delayed_ops::lifecycle(next)?;
    }
    if flow(next).is_ok_and(|f| f.completion.blocks.contains("MOVE-A55"))
        && battle(next).is_ok_and(|b| b.active)
    {
        trigger_ops::lifecycle(next)?;
    }
    if flow(next).is_ok_and(|f| f.completion.blocks.contains("MOVE-A56"))
        && battle(next).is_ok_and(|b| b.active)
    {
        field_ops::lifecycle(next, cat)?;
    }
    if flow(next).is_ok_and(|f| f.completion.blocks.contains("MOVE-A59"))
        && battle(next).is_ok_and(|b| b.active)
    {
        copy_ops::lifecycle(next)?;
    }
    if definition(next, "Charge").is_some() {
        let sequences: BTreeSet<_> = flow(next)?
            .queue
            .iter()
            .map(|q| (q.actor_id.clone(), q.sequence))
            .collect();
        flow_mut(next).special.effects.retain(|e| {
            let cast = if e.name == "Charge" {
                e.slot.strip_prefix("charge-cast:")
            } else if e.name == "Order Up" {
                e.slot
                    .strip_prefix("order-up-cast:")
                    .and_then(|s| s.split_once(':').map(|v| v.0))
            } else if e.name == "Electrify" {
                e.slot.strip_prefix("electrify-cast:")
            } else if e.name == "Laser Focus" {
                e.slot.strip_prefix("focus-cast:")
            } else {
                None
            };
            cast.is_none_or(|s| {
                s.parse::<u32>()
                    .is_ok_and(|seq| sequences.contains(&(e.target.clone(), seq)))
            })
        });
    }
    let after = battle(next)?;
    if !after.active {
        return Ok(());
    }
    let old = before.battle.as_ref().filter(|b| b.id == after.id);
    let round = after.round;
    let initial_entry =
        flow(next)?.completion.blocks.contains("MOVE-A24") && old.is_none_or(|b| b.flow.is_none());
    let entered: Vec<String> = after
        .combatants
        .iter()
        .filter(|p| {
            p.deployed
                && p.hp > 0
                && (initial_entry
                    || !old
                        .is_some_and(|b| b.combatants.iter().any(|v| v.id == p.id && v.deployed)))
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
    if d["mode"] == "first-eligible" {
        let entered=flow(c)?.completion.entered.get(&p.id).copied().ok_or_else(||AppError::new("manual_rule","First Impression requires recorded deployment history to determine its first eligible round"))?;
        if battle(c)?.round != entered.max(p.available_round) {
            return Err(error("First Impression works only during the first round its user can act after deployment"));
        }
    }
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
    condition_ops::target_guard(c, id, m)
}
fn history_effect(
    c: &mut Campaign,
    q: &QueuedAction,
    d: &serde_json::Value,
    cat: &Catalog,
) -> Result<()> {
    let mode = d["mode"].as_str().unwrap();
    if mode == "first-eligible" {
        return history_guard(c, fighter(c, &q.actor_id)?, &move_entry(cat, &q.move_name)?);
    }

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

pub fn partner_rule(c: &Campaign, name: &str) -> bool {
    definition(c, name).is_some_and(|(_, d)| d["final_stage_fail"] == true)
        || (flow(c).is_ok_and(|f| f.completion.blocks.contains("MOVE-A20"))
            && ["Bouncy Bubble", "Veevee Volley", "Sparkly Swirl"].contains(&name))
}
pub fn partner_guard(c: &Campaign, id: &str, cat: &Catalog) -> Result<()> {
    let p = fighter(c, id)?;
    let species = cat
        .find(
            "species",
            "AppId",
            c.sheets[&p.sheet_id].species_id.as_deref().unwrap_or(""),
        )
        .ok_or_else(|| {
            AppError::new(
                "manual_rule",
                "Partner moves require a reviewed species evolution identity",
            )
        })?;
    let number = species["Number"]
        .as_u64()
        .ok_or_else(|| error("Partner evolution identity has no Number"))?;
    if let Some(edges) = species["Evolutions"].as_array() {
        for edge in edges {
            if ["Mega", "Form"].contains(&edge["Kind"].as_str().unwrap_or("")) {
                continue;
            }
            let Some(to) = edge["To"].as_str() else {
                continue;
            };
            let target=cat.find("species","Name",to).ok_or_else(||AppError::new("manual_rule","Partner move has an unresolved evolution target; the GM must review its identity"))?;
            let target_number = target["Number"]
                .as_u64()
                .ok_or_else(|| error("Partner evolution target has no Number"))?;
            if number != target_number {
                return Ok(());
            }
        }
    }
    Err(error(
        "This partner move fails at its user's final ordinary evolution stage",
    ))
}

/// Live move-channel values, so a field boost does not add to Agility or copied boosts.
pub fn field_stat_values(c: &Campaign, p: &Combatant, key: &str) -> Vec<i32> {
    if key != "Dexterity"
        || !p.deployed
        || p.hp == 0
        || definition(c, "Tailwind").is_none()
        || !field::has(c, "Tailwind", p)
    {
        return vec![];
    }
    let allowed = flow(c).unwrap().battlefield.rules.iter().any(|r| {
        r.kind == "Tailwind"
            && field::applies(&r.scope, p)
            && (!p.has_ability("Clear Body")
                || house::ring(c, p)
                || r.source == format!("field-user:{}", p.id))
    });
    if !allowed {
        return vec![];
    }
    vec![match p.ability_for(&["Contrary", "Simple"]) {
        "Contrary" => -2,
        "Simple" => 3,
        _ => 2,
    }]
}

/// Guaranteed target status clauses use the same sport-adjusted immunity as damage.
pub(super) fn sport_effect_blocked(c: &Campaign, q: &QueuedAction, cat: &Catalog) -> Result<bool> {
    let source = reviewed::source(c, q)?;
    let m = field::move_data(c, &source, &move_entry(cat, &q.move_name)?);
    Ok(sport(c, &m) && immune_target(c, q, cat)?)
}

/// An applied Fire vulnerability changes move effectiveness, not static species typing.
pub fn typed_effectiveness(c: &Campaign, m: &serde_json::Value, t: &Combatant, n: i32) -> i32 {
    if m["Type"] == "Fire"
        && ["Physical", "Special"].contains(&m["Category"].as_str().unwrap_or(""))
        && definition(c, "Tar Shot").is_some()
        && field::fire_vulnerable(c, &t.id)
    {
        1
    } else {
        n
    }
}

fn self_support(c: &Campaign, p: &Combatant, name: &str) -> bool {
    p.deployed
        && p.hp > 0
        && definition(c, name).is_some()
        && flow(c).is_ok_and(|f| {
            f.special
                .effects
                .iter()
                .any(|e| e.name == name && e.target == p.id && e.slot == "self-support")
        })
}
pub fn weight(c: &Campaign, p: &Combatant, n: f64) -> f64 {
    let n = if self_support(c, p, "Autotomize") {
        n / 2.0
    } else {
        n
    };
    if initiative_ops::minimized(c, &p.id) {
        n / 2.0
    } else {
        n
    }
}

/// Bind a stored Charge to the attempted cast before hit/miss branching.
pub(super) fn charge_attempt(
    c: &mut Campaign,
    id: &str,
    m: &serde_json::Value,
    cat: &Catalog,
) -> Result<()> {
    let p = fighter(c, id)?.clone();
    if !self_support(c, &p, "Charge") {
        return Ok(());
    }
    let m = field::move_data(c, &p, m);
    if m["Type"] != "Electric"
        || !["Physical", "Special"].contains(&m["Category"].as_str().unwrap_or(""))
    {
        return Ok(());
    }
    let sequence = c.revision + 1;
    let started = battle(c)?.round;
    flow_mut(c)
        .special
        .effects
        .retain(|e| e.name != "Charge" || e.target != id || e.slot != "self-support");
    flow_mut(c).special.effects.push(special::TimedEffect {
        name: "Charge".into(),
        source: id.into(),
        target: id.into(),
        rounds: None,
        amount: 2,
        started,
        slot: format!("charge-cast:{sequence}"),
    });
    let _ = cat;
    Ok(())
}

pub(super) fn laser_attempt(
    c: &mut Campaign,
    id: &str,
    m: &serde_json::Value,
    required: u32,
) -> Result<()> {
    if !["Physical", "Special"].contains(&m["Category"].as_str().unwrap_or(""))
        || !self_support(c, fighter(c, id)?, "Laser Focus")
    {
        return Ok(());
    }
    let started = battle(c)?.round;
    let sequence = c.revision + 1;
    flow_mut(c)
        .special
        .effects
        .retain(|e| e.name != "Laser Focus" || e.target != id || e.slot != "self-support");
    flow_mut(c).special.effects.push(special::TimedEffect {
        name: "Laser Focus".into(),
        source: id.into(),
        target: id.into(),
        rounds: None,
        amount: required as i32 + 5,
        started,
        slot: format!("focus-cast:{sequence}"),
    });
    Ok(())
}
pub fn laser_defenses(c: &Campaign, id: &str, sequence: u32, target: &str) -> bool {
    if definition(c, "Laser Focus").is_none() {
        return false;
    }
    let Ok(f) = flow(c) else { return false };
    let Some(q) = f
        .queue
        .first()
        .filter(|q| q.actor_id == id && q.sequence == sequence)
    else {
        return false;
    };
    let successes = q
        .targets
        .iter()
        .find(|v| v.target_id == target)
        .and_then(|v| v.successes)
        .unwrap_or(q.successes);
    f.special.effects.iter().any(|e| {
        e.name == "Laser Focus"
            && e.target == id
            && e.slot == format!("focus-cast:{sequence}")
            && successes >= e.amount
    })
}

pub fn shelter_loss(c: &Campaign, a: &Combatant, t: &Combatant) -> i32 {
    i32::from(a.side != t.side && self_support(c, t, "Shelter"))
}

/// Reset applied stat changes, preserving active passives under the user's ruling.
pub(super) fn reset_stats(c: &mut Campaign, id: &str, traits: bool) -> Result<()> {
    let mut keys: BTreeSet<&str> = ["Strength", "Dexterity", "Vitality", "Special", "Insight"]
        .into_iter()
        .collect();
    if traits {
        keys.extend([
            "Defense",
            "SpecialDefense",
            "AccuracyPool",
            "AccuracySuccess",
            "Evasion",
            "EvasionPool",
            "Will",
        ]);
    }
    reset_keys(c, id, &keys, traits)
}
/// Clear applied move, stacking, copied and triggered-ability changes for `keys`.
/// Passive ability and field bonuses are preserved; `traits` also clears Accuracy losses.
pub(super) fn reset_keys(
    c: &mut Campaign,
    id: &str,
    keys: &BTreeSet<&str>,
    traits: bool,
) -> Result<()> {
    flow_mut(c).special.effects.retain(|e| {
        if e.target != id {
            return true;
        }
        if traits && (e.name.starts_with("accuracy-loss:") || e.name == "Lunar accuracy") {
            return false;
        }
        if e.name.starts_with("buff:") || e.name.starts_with("debuff:") || e.slot == "stacking-stat"
        {
            return !e
                .name
                .rsplit(':')
                .next()
                .is_some_and(|key| keys.contains(key));
        }
        true
    });
    let f = flow_mut(c);
    f.stat_channels
        .grants
        .retain(|g| g.target != id || !keys.contains(g.key.as_str()));
    if let Some(removed) = f.stat_channels.removed.get_mut(id) {
        removed.retain(|key, _| !keys.contains(key.as_str()));
    }
    if f.stat_channels
        .removed
        .get(id)
        .is_some_and(|v| v.is_empty())
    {
        f.stat_channels.removed.remove(id);
        f.stat_channels.removed_abilities.remove(id);
    }
    if keys.contains("Strength") {
        for targets in f.intimidate.values_mut() {
            targets.retain(|target| target != id);
        }
    }
    let rt = &mut fighter_mut(c, id)?.runtime;
    rt.modifiers.retain(|key, _| !keys.contains(key.as_str()));
    rt.ability.counters.retain(|key, _| {
        if let Some(stat) = key
            .strip_prefix("boost:")
            .or_else(|| key.strip_prefix("debuff:"))
        {
            return !keys.contains(stat);
        }
        let stat = match key.as_str() {
            "Moxie" | "Sap Sipper" => "Strength",
            "Lightning Rod" | "Storm Drain" => "Special",
            "Speed Boost" => "Dexterity",
            "Beast Boost total" => "Strength",
            _ => "",
        };
        !keys.contains(stat)
    });
    rt.ability.flags.retain(|flag| {
        let affected: &[&str] = match flag.as_str() {
            "Defiant" | "Thermal Exchange" => &["Strength"],
            "Competitive" => &["Special"],
            "Steadfast" | "Rattled" | "Steam Engine" | "Unburden" => &["Dexterity"],
            "Weak Armor" => &["Dexterity", "Defense"],
            "Stamina" => &["Defense", "SpecialDefense"],
            "Water Compaction" | "Well-Baked Body" => &["Defense"],
            _ => &[],
        };
        !affected.iter().any(|key| keys.contains(key))
    });
    Ok(())
}

pub fn cannot_evade(c: &Campaign, source: &str, target: &str) -> bool {
    if type_ops::miracle_eye(c, source) {
        return true;
    }
    flow(c).is_ok_and(|f| {
        f.special.effects.iter().any(|e| {
            e.target == target
                && definition(c, &e.name).is_some_and(|(kind, d)| {
                    kind == "detection" && (d["detection"] != true || e.source == source)
                })
        })
    })
}
pub fn ghost_pair(
    c: &Campaign,
    a: &Combatant,
    t: &Combatant,
    move_type: &str,
    target_type: &str,
) -> bool {
    (((["Normal", "Fight", "Fighting"].contains(&move_type) && target_type == "Ghost")
        || (move_type == "Ghost" && target_type == "Normal"))
        && field::detected(c, &a.id, &t.id))
        || (move_type == "Psychic" && target_type == "Dark" && type_ops::miracle_eye(c, &a.id))
}

fn lifted_by(c: &Campaign, p: &Combatant, name: &str) -> bool {
    p.deployed
        && p.hp > 0
        && definition(c, name).is_some()
        && !grounded(c, p)
        && !field::has(c, "Gravity", p)
        && !items::iron_ball(c, &p.id)
        && !items::completion::ring(c, p, true)
        && flow(c).is_ok_and(|f| {
            f.special
                .effects
                .iter()
                .any(|e| e.name == name && e.target == p.id)
        })
}

pub fn telekinetic(c: &Campaign, p: &Combatant) -> bool {
    lifted_by(c, p, "Telekinesis")
}
pub fn lifted(c: &Campaign, p: &Combatant) -> bool {
    telekinetic(c, p) || lifted_by(c, p, "Magnet Rise")
}

pub fn non_fainting_damage(c: &Campaign, name: &str, before: i32, damage: i32) -> i32 {
    if name == "False Swipe" && definition(c, name).is_some() && before > 0 {
        damage.min(before - 1).max(0)
    } else {
        damage
    }
}

/// Finish self-fainting only once the whole cast leaves the pending queue.
pub fn self_faint(before: &Campaign, next: &mut Campaign) -> Result<bool> {
    let Some(b) = next.battle.as_ref().filter(|b| b.active) else {
        return Ok(false);
    };
    if !flow(next).is_ok_and(|f| f.completion.blocks.contains("MOVE-A38")) {
        return Ok(false);
    }
    let old = before.battle.as_ref().filter(|v| v.id == b.id);
    let explosive = |name: &str| ["Explosion", "Self Destruct", "Misty Explosion"].contains(&name);
    let mut finished = BTreeSet::new();
    for p in &b.combatants {
        let prior = old
            .filter(|v| v.round == b.round)
            .and_then(|v| v.combatants.iter().find(|v| v.id == p.id))
            .map_or(&[][..], |v| v.used_moves.as_slice());
        if p.used_moves.starts_with(prior)
            && p.used_moves[prior.len()..].iter().any(|n| explosive(n))
        {
            finished.insert(p.id.clone());
        }
    }
    if let Some(f) = old.and_then(|v| v.flow.as_ref()) {
        for q in &f.queue {
            if explosive(&q.move_name)
                && !flow(next)?
                    .queue
                    .iter()
                    .any(|v| v.actor_id == q.actor_id && v.sequence == q.sequence)
            {
                finished.insert(q.actor_id.clone());
            }
        }
    }
    let mut changed = false;
    for id in finished {
        if !flow(next)?
            .queue
            .iter()
            .any(|q| q.actor_id == id && explosive(&q.move_name))
            && fighter(next, &id)?.hp > 0
        {
            fighter_mut(next, &id)?.hp = 0;
            changed = true;
        }
    }
    Ok(changed)
}

pub fn clear_inactive_types(c: &mut Campaign) {
    let Some(b) = c.battle.as_ref() else { return };
    let active: BTreeSet<_> = b
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| format!("combatant:{}", p.id))
        .collect();
    if flow(c).is_ok() {
        flow_mut(c)
            .battlefield
            .rules
            .retain(|r| r.source != "completion-type" || active.contains(&r.scope));
    }
}

pub fn plasma(c: &Campaign, p: &Combatant) -> bool {
    self_support(c, p, "Plasma Fists")
}

fn electrified(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> bool {
    if definition(c, "Electrify").is_none() {
        return false;
    }
    if self_support(c, p, "Electrify") {
        return true;
    }
    flow(c).is_ok_and(|f| {
        f.queue.first().is_some_and(|q| {
            q.actor_id == p.id
                && m["Name"] == q.move_name
                && f.special.effects.iter().any(|e| {
                    e.name == "Electrify"
                        && e.target == p.id
                        && e.slot == format!("electrify-cast:{}", q.sequence)
                })
        })
    })
}
pub(super) fn electrify_attempt(c: &mut Campaign, id: &str, m: &serde_json::Value) -> Result<()> {
    if !["Physical", "Special"].contains(&m["Category"].as_str().unwrap_or(""))
        || !self_support(c, fighter(c, id)?, "Electrify")
    {
        return Ok(());
    }
    let seq = c.revision + 1;
    let started = battle(c)?.round;
    flow_mut(c)
        .special
        .effects
        .retain(|e| e.name != "Electrify" || e.target != id || e.slot != "self-support");
    flow_mut(c).special.effects.push(special::TimedEffect {
        name: "Electrify".into(),
        source: id.into(),
        target: id.into(),
        rounds: None,
        amount: 0,
        started,
        slot: format!("electrify-cast:{seq}"),
    });
    Ok(())
}

pub(super) fn order_up_attempt(
    c: &mut Campaign,
    id: &str,
    m: &serde_json::Value,
    cat: &Catalog,
) -> Result<()> {
    let name = m["Name"].as_str().unwrap_or("");
    let Some(("cover-stat", d)) = definition(c, name) else {
        return Ok(());
    };
    let side = fighter(c, id)?.side.clone();
    let mut key = None;
    for (ally, coverer) in &flow(c)?.covers {
        if coverer != id {
            continue;
        }
        let p = fighter(c, ally)?;
        if !p.deployed || p.hp == 0 || p.side != side {
            continue;
        }
        let sheet = rules::base_sheet(c, p)?;
        if let Some(species) = sheet
            .species_id
            .as_deref()
            .and_then(|v| cat.find("species", "AppId", v))
        {
            if species["Number"].as_i64() == Some(978) {
                key = species["DexID"]
                    .as_str()
                    .and_then(|id| d["forms"][id].as_str())
                    .map(str::to_owned);
            }
        }
    }
    if let Some(key) = key {
        let sequence = c.revision + 1;
        let started = battle(c)?.round;
        flow_mut(c).special.effects.push(special::TimedEffect {
            name: name.into(),
            source: id.into(),
            target: id.into(),
            rounds: None,
            amount: 0,
            started,
            slot: format!("order-up-cast:{sequence}:{key}"),
        });
    }
    Ok(())
}
