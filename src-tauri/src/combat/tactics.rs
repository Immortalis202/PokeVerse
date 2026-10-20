//! Targeting, reaction scheduling and controlled effect switches (RULING 68–72,83).
use super::*;
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct TacticsState {
    pub enabled: bool,
    pub lures: Vec<Lure>,
    pub switches: Vec<SwitchRequest>,
    pub switchers: BTreeMap<String, u32>,
    pub redirect_choices: BTreeMap<String, String>,
    pub priority_choices: BTreeMap<String, String>,
    pub coat_rules: BTreeMap<String, String>,
    pub clash_damage: BTreeMap<u32, i32>,
    pub clash_pending: BTreeMap<u32, (QueuedAction, i32)>,
    pub baton: BTreeMap<String, String>,
    pub history: Vec<String>,
    pub payer: Option<(String, String)>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Lure {
    pub source: String,
    pub target: String,
    pub name: String,
    pub round: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct SwitchRequest {
    pub id: String,
    pub source: String,
    pub outgoing: String,
    pub name: String,
    pub round: u32,
    pub immediate: bool,
    pub forced: bool,
    pub ability: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export, export_to = "../../src/bindings/")]
pub enum TacticsCommand {
    Reaction {
        combatant_id: String,
        move_name: String,
        target_ids: Vec<String>,
    },
    Switch {
        request_id: String,
        incoming_id: Option<String>,
        ruling: String,
    },
    Redirect {
        combatant_id: String,
        recipient_id: String,
    },
    CoatDuration {
        combatant_id: String,
        duration: String,
    },
    Priority {
        combatant_id: String,
        move_name: String,
        mode: String,
    },
}
pub fn enabled(c: &Campaign) -> bool {
    c.battle
        .as_ref()
        .filter(|b| b.active)
        .and_then(|b| b.flow.as_ref())
        .is_some_and(|f| f.tactics.enabled)
}
fn state(c: &Campaign) -> &TacticsState {
    &flow(c).unwrap().tactics
}
fn state_mut(c: &mut Campaign) -> &mut TacticsState {
    &mut flow_mut(c).tactics
}
fn record(c: &mut Campaign, text: String) {
    let log = &mut state_mut(c).history;
    log.push(text);
    if log.len() > 300 {
        log.remove(0);
    }
}
fn in_range(c: &Campaign, a: &str, t: &str) -> bool {
    a == t
        || flow(c)
            .unwrap()
            .ranges
            .get(a)
            .is_none_or(|ids| ids.iter().any(|id| id == t))
}
fn living(c: &Campaign) -> impl Iterator<Item = &Combatant> {
    c.battle
        .as_ref()
        .into_iter()
        .flat_map(|b| b.combatants.iter())
        .filter(|p| p.deployed && p.hp > 0)
}
fn bypass(c: &Campaign, id: &str) -> bool {
    enabled(c)
        && fighter(c, id).is_ok_and(|p| {
            reviewed::extreme(c, p)
                || ["Stalwart", "Propeller Tail"].contains(&p.selected_ability.as_str())
        })
}
fn number(m: &serde_json::Value, key: &str) -> Option<i32> {
    m["Attributes"][key]
        .as_str()
        .and_then(|s| s.parse().ok())
        .or_else(|| m["Attributes"][key].as_i64().map(|n| n as i32))
}
fn ability_priority(p: &Combatant, m: &serde_json::Value) -> bool {
    ((p.selected_ability == "Prankster"
        || (p.selected_ability == "Triage"
            && (m["Effect"]
                .as_str()
                .is_some_and(|e| e.contains("Basic Heal"))
                || [
                    "Heal Bell",
                    "Aromatherapy",
                    "Refresh",
                    "Purify",
                    "Jungle Healing",
                    "Take Heart",
                ]
                .contains(&m["Name"].as_str().unwrap_or("")))))
        && m["Category"] == "Support")
        || (p.selected_ability == "Gale Wings" && m["Type"] == "Flying")
}
pub fn move_data(c: &Campaign, p: &Combatant, m: &mut serde_json::Value) {
    if !enabled(c) || !ability_priority(p, m) {
        return;
    }
    let key = format!("{}:{}", p.id, m["Name"].as_str().unwrap_or(""));
    let mode = state(c).priority_choices.get(&key).map(String::as_str);
    if mode == Some("printed") {
        return;
    }
    if number(m, "LateReaction").is_some() && mode != Some("ability") {
        return;
    }
    let n = number(m, "Reaction").unwrap_or(1).max(1);
    m["Attributes"]["Reaction"] = serde_json::json!(n.to_string());
    if mode == Some("ability") {
        m["Attributes"]
            .as_object_mut()
            .unwrap()
            .remove("LateReaction");
    }
}
pub fn priority_guard(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> Result<()> {
    if reviewed::enabled(c)
        && p.selected_ability == "Mycelium Might"
        && m["Category"] == "Support"
        && number(m, "Reaction").is_some()
    {
        return Err(AppError::new("manual_rule","Mycelium Might Late Reaction 1 conflicts with printed Reaction; GM adjudication required"));
    }
    if enabled(c)
        && ability_priority(p, m)
        && number(m, "LateReaction").is_some()
        && !state(c).priority_choices.contains_key(&format!(
            "{}:{}",
            p.id,
            m["Name"].as_str().unwrap_or("")
        ))
    {
        return Err(AppError::new("manual_rule","Ability Reaction 1 conflicts with this card's Late Reaction. The GM must record printed timing or ability timing before use"));
    }
    Ok(())
}
fn guard_targets(c: &Campaign, id: &str, targets: &[String], m: &serde_json::Value) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let source = fighter(c, id)?;
    priority_guard(c, source, m)?;
    if number(m, "Reaction").is_none() && number(m, "LateReaction").is_none() {
        return Ok(());
    }
    for key in targets {
        let t = fighter(c, key)?;
        if t.side != source.side && reviewed::extreme(c, source) {
            continue;
        }
        if t.side != source.side
            && ["Dazzling", "Queenly Majesty"].contains(&t.selected_ability.as_str())
        {
            return Err(error("Dazzling/Queenly Majesty prevents reaction moves against this foe; Evasion and Clash remain maneuvers"));
        }
        if m["Category"] != "Support"
            && living(c).any(|p| {
                p.side == t.side && p.selected_ability == "Armor Tail" && in_range(c, &p.id, key)
            })
        {
            return Err(error(
                "Armor Tail prevents Physical/Special reaction moves against this participant",
            ));
        }
    }
    Ok(())
}
pub fn redirect_move(
    c: &Campaign,
    id: &str,
    ids: Vec<String>,
    m: &serde_json::Value,
    cat: &Catalog,
) -> Result<Vec<String>> {
    if !enabled(c) {
        return Ok(ids);
    }
    let source = fighter(c, id)?;
    let powder_immune = source.selected_ability == "Overcoat"
        || field::types(c, source, cat)?.iter().any(|t| t == "Grass");
    let mut result = ids;
    if !bypass(c, id)
        && result.len() == 1
        && ["Foe", "Ally", "Random Foe"].contains(&m["Target"].as_str().unwrap_or(""))
    {
        let mut candidates: Vec<String> = living(c)
            .filter(|p| {
                p.id != id
                    && special::target_guard(c, id, &p.id, m).is_ok()
                    && ((p.selected_ability == "Lightning Rod" && m["Type"] == "Electric")
                        || (p.selected_ability == "Storm Drain" && m["Type"] == "Water"))
            })
            .map(|p| p.id.clone())
            .collect();
        if m["Target"] != "Ally" {
            candidates.extend(
                state(c)
                    .lures
                    .iter()
                    .filter(|l| {
                        (l.round == 0 || l.round == battle(c).unwrap().round)
                            && l.name != "Magic Coat"
                            && (l.name != "Rage Powder"
                                || (m["Category"] != "Support" && !powder_immune))
                    })
                    .filter_map(|l| {
                        fighter(c, &l.target)
                            .ok()
                            .filter(|p| {
                                p.hp > 0
                                    && p.side != source.side
                                    && in_range(c, id, &p.id)
                                    && special::target_guard(c, id, &p.id, m).is_ok()
                            })
                            .map(|p| p.id.clone())
                    }),
            );
        }
        candidates.sort();
        candidates.dedup();
        if !candidates.is_empty() {
            let chosen = if candidates.len() == 1 {
                candidates[0].clone()
            } else {
                state(c)
                    .redirect_choices
                    .get(id)
                    .filter(|key| candidates.contains(key))
                    .cloned()
                    .ok_or_else(|| {
                        AppError::new(
                            "manual_rule",
                            format!(
                                "Competing redirectors require a GM choice: {}",
                                candidates
                                    .iter()
                                    .map(|id| fighter(c, id).unwrap().name.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        )
                    })?
            };
            result = vec![chosen];
        }
    }
    guard_targets(c, id, &result, m)?;
    Ok(result)
}
pub fn reaction_targets(
    c: &Campaign,
    id: &str,
    m: &serde_json::Value,
    selected: Option<&[String]>,
    cat: &Catalog,
) -> Result<(Vec<String>, Option<String>)> {
    if !enabled(c) {
        return Ok((vec![], None));
    }
    let incoming = incoming(c)?;
    if special::ally_shield(m) && selected.is_none() {
        return Ok((special::allies(c, id)?, None));
    }
    let ids = selected
        .map(Vec::from)
        .unwrap_or_else(|| match m["Target"].as_str().unwrap_or("") {
            "Self" => vec![id.into()],
            "Foe" | "Random Foe" => vec![incoming.actor_id.clone()],
            "Ally" => vec![incoming.target_id.clone()],
            _ => vec![],
        });
    let (ids, scope) = plan(c, id, &ids, m)?;
    let ids = redirect_move(c, id, ids, m, cat)?;
    Ok((ids, scope))
}
pub fn reaction_dice(
    c: &Campaign,
    id: &str,
    name: &str,
    selected: Option<&[String]>,
    cat: &Catalog,
) -> Result<u32> {
    if abilities::pending(c) || field::pending(c) {
        return Err(error(
            "Resolve pending ability and battlefield events first",
        ));
    }
    let base = reaction_pool(c, id, "move", name, cat)?;
    let m = field::move_data(c, fighter(c, id)?, &move_entry(cat, name)?);
    reaction_targets(c, id, &m, selected, cat)?;
    Ok(base + u32::from(fighter(c, id)?.runtime.chance_round == battle(c)?.round))
}
#[allow(clippy::too_many_arguments)]
pub fn react(
    c: &mut Campaign,
    actor: &str,
    id: &str,
    name: &str,
    selected: Option<&[String]>,
    faces: &[u8],
    r: &str,
    cat: &Catalog,
) -> Result<String> {
    owner(c, actor, id)?;
    reason(r)?;
    let count = reaction_dice(c, id, name, selected, cat)?;
    check_count(faces, count)?;
    let before = c.clone();
    let p = fighter(c, id)?.clone();
    let m = field::move_data(c, &p, &move_entry(cat, name)?);
    let (targets, scope) = reaction_targets(c, id, &m, selected, cat)?;
    let base = count - u32::from(p.runtime.chance_round == battle(c)?.round);
    let penalty = shield_attempt(c, id, &m)?;
    let saved = p.runtime.pain_ignored;
    if pain_exempt(&before, &p, &m)? {
        fighter_mut(c, id)?.runtime.pain_ignored = 3;
    }
    let net = (action_success(c, id, faces, base)
        + m["Attributes"]["AccuracyReduction"]
            .as_i64()
            .unwrap_or(0)
            .min(0) as i32
        - abilities::low_accuracy(&before, &p, &m)
        - field::low_accuracy(&before, &p, &m)
        - special::low_accuracy(&before, id)
        - penalty)
        .max(0)
        + i32::from(p.runtime.fate_pending);
    fighter_mut(c, id)?.runtime.pain_ignored = saved;
    fighter_mut(c, id)?.actions += 1;
    fighter_mut(c, id)?.reactions += 1;
    fighter_mut(c, id)?.used_moves.push(name.into());
    flow_mut(c).reacted.insert(id.into());
    if incoming(&before)?.target_id == id {
        responded(c);
    }
    break_cover(c, id);
    extra::accuracy_attempt(c, id, &m, cat)?;
    if net >= (p.actions + 1) as i32 {
        let (timing, priority) = number(&m, "Reaction")
            .map(|n| ("before", n))
            .or_else(|| number(&m, "LateReaction").map(|n| ("after", n)))
            .ok_or_else(|| error("No reaction timing"))?;
        let target = targets.first().cloned().unwrap_or_else(|| id.into());
        flow_mut(c).queue.push(QueuedAction {
            actor_id: id.into(),
            target_id: target,
            move_name: name.into(),
            sequence: before.revision + 1,
            successes: net,
            critical: m["Category"] != "Support"
                && net >= (p.actions + 1) as i32 + extra::critical_margin(&before, &p, &m),
            timing: timing.into(),
            priority,
            cancelled: false,
            damage_done: false,
            damage: 0,
            raw_damage_successes: 0,
            ice: false,
            special_index: 0,
            targets: targets
                .into_iter()
                .map(|target_id| TargetImpact {
                    target_id,
                    responded: false,
                    avoided: false,
                    resolved: false,
                })
                .collect(),
            field_scope: scope,
            shared_dice: vec![],
            damage_modifiers: None,
            damage_context: None,
            defense_key: if m["Category"] == "Support" {
                "Support"
            } else if m["Category"] == "Physical" {
                "Defense"
            } else {
                "SpecialDefense"
            }
            .into(),
        });
    } else if confusion(&before, &p) > 0 {
        fighter_mut(c, id)?.hp = (p.hp - 1).max(0);
    }
    special::after_reaction(c, id, "move", cat)?;
    close(c);
    Ok(format!(
        "Reaction {name}: {faces:?}; {net} successes, requires {}. {r}",
        p.actions + 1
    ))
}
pub fn dice(c: &Campaign, cmd: &TacticsCommand, cat: &Catalog) -> Result<u32> {
    if !enabled(c) {
        return Err(error("Adopt targeting/reaction rules between battles"));
    }
    match cmd {
        TacticsCommand::Reaction {
            combatant_id,
            move_name,
            target_ids,
        } => reaction_dice(c, combatant_id, move_name, Some(target_ids), cat),
        _ => Ok(0),
    }
}
pub fn phase(c: &Campaign, a: &Action) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let blocking = state(c)
        .switches
        .iter()
        .any(|s| s.immediate || flow(c).unwrap().queue.is_empty());
    if blocking
        && !matches!(
            a,
            Action::Tactics {
                command: TacticsCommand::Switch { .. },
                ..
            } | Action::ResolveAbility { .. }
                | Action::Battlefield {
                    command: field::FieldCommand::Resolve { .. },
                    ..
                }
                | Action::RollbackBattle { .. }
                | Action::EndBattle
                | Action::Override { .. }
        )
    {
        return Err(error(
            "Resolve the pending switch/replacement before continuing",
        ));
    }
    Ok(())
}
pub fn command(
    c: &mut Campaign,
    actor: &str,
    cmd: &TacticsCommand,
    faces: &[u8],
    r: &str,
    cat: &Catalog,
) -> Result<String> {
    reason(r)?;
    check_count(faces, dice(c, cmd, cat)?)?;
    match cmd {
        TacticsCommand::Reaction {
            combatant_id,
            move_name,
            target_ids,
        } => {
            return react(
                c,
                actor,
                combatant_id,
                move_name,
                Some(target_ids),
                faces,
                r,
                cat,
            )
        }
        TacticsCommand::Redirect {
            combatant_id,
            recipient_id,
        } => {
            gm(c, actor)?;
            fighter(c, combatant_id)?;
            fighter(c, recipient_id)?;
            if !flow(c)?.queue.is_empty() {
                return Err(error("Choose competing redirectors before an action chain"));
            }
            state_mut(c)
                .redirect_choices
                .insert(combatant_id.clone(), recipient_id.clone());
        }
        TacticsCommand::CoatDuration {
            combatant_id,
            duration,
        } => {
            gm(c, actor)?;
            fighter(c, combatant_id)?;
            if !["next_effect", "round", "scene"].contains(&duration.as_str()) {
                return Err(error(
                    "Choose next_effect, round or scene for the unstated Magic Coat duration",
                ));
            }
            if flow(c)?
                .queue
                .iter()
                .any(|q| q.damage_context.is_some() || q.damage_done)
            {
                return Err(error("Record duration before damage commits"));
            }
            state_mut(c)
                .coat_rules
                .insert(combatant_id.clone(), duration.clone());
        }
        TacticsCommand::Priority {
            combatant_id,
            move_name,
            mode,
        } => {
            gm(c, actor)?;
            let p = fighter(c, combatant_id)?;
            let m = move_entry(cat, move_name)?;
            if !ability_priority(p, &m) || !["printed", "ability"].contains(&mode.as_str()) {
                return Err(error(
                    "Choose printed or ability timing for this priority conflict",
                ));
            }
            if flow(c)?
                .queue
                .iter()
                .any(|q| q.damage_context.is_some() || q.damage_done)
            {
                return Err(error("Record timing choices before damage commits"));
            }
            state_mut(c)
                .priority_choices
                .insert(format!("{combatant_id}:{move_name}"), mode.clone());
        }
        TacticsCommand::Switch {
            request_id,
            incoming_id,
            ruling,
        } => resolve_switch(c, actor, request_id, incoming_id.as_deref(), ruling, cat)?,
    }
    let text = format!("Targeting decision {cmd:?}: {r}");
    record(c, text.clone());
    Ok(text)
}
pub fn automatic_response(c: &Campaign, q: &QueuedAction, id: &str) -> bool {
    if !enabled(c) {
        return false;
    }
    if id == q.actor_id {
        return true;
    }
    let cat_type = q.defense_key == "Support";
    cat_type
        && fighter(c, id).is_ok_and(|p| {
            p.selected_ability == "Good As Gold"
                || (p.selected_ability == "Magic Bounce"
                    && fighter(c, &q.actor_id).is_ok_and(|a| a.side != p.side)
                    && !bypass(c, &q.actor_id))
        })
}
pub fn queue_clash(
    c: &mut Campaign,
    incoming: &QueuedAction,
    name: &str,
    raw: i32,
    cat: &Catalog,
) -> Result<()> {
    let counter = QueuedAction {
        actor_id: incoming.target_id.clone(),
        target_id: incoming.actor_id.clone(),
        move_name: name.into(),
        ..incoming.clone()
    };
    let m = move_entry(cat, name)?;
    let reduced = abilities::clash_reduced(c, &counter.target_id, &m, raw, &counter.actor_id)?;
    let old = state(c)
        .clash_pending
        .get(&incoming.sequence)
        .map(|(q, raw)| {
            abilities::clash_reduced(
                c,
                &q.target_id,
                &move_entry(cat, &q.move_name)?,
                *raw,
                &q.actor_id,
            )
        })
        .transpose()?
        .unwrap_or(-1);
    if reduced > old {
        state_mut(c)
            .clash_pending
            .insert(incoming.sequence, (counter, raw));
    }
    Ok(())
}
fn flush_clashes(c: &mut Campaign, cat: &Catalog) -> Result<()> {
    let pending = state(c)
        .clash_pending
        .iter()
        .filter(|(seq, _)| {
            flow(c)
                .unwrap()
                .queue
                .iter()
                .find(|q| q.sequence == **seq)
                .is_none_or(|q| q.targets.iter().all(|t| t.responded || t.avoided))
        })
        .map(|(id, v)| (*id, v.clone()))
        .collect::<Vec<_>>();
    for (seq, (q, raw)) in pending {
        state_mut(c).clash_pending.remove(&seq);
        if fighter(c, &q.target_id).is_err() || fighter(c, &q.actor_id).is_err() {
            continue;
        }
        let m = move_entry(cat, &q.move_name)?;
        let hp = fighter(c, &q.target_id)?.hp;
        let amount = abilities::clash_amount(c, &q.target_id, &m, raw, &q.actor_id)?;
        let decoy_hit = decoys::damage(c, &q, &m, amount);
        if !decoy_hit {
            fighter_mut(c, &q.target_id)?.hp = (hp - amount).max(0);
        }
        state_mut(c).clash_damage.insert(seq, amount);
        if !decoy_hit {
            abilities::hit(c, &q, hp, &m, cat)?;
        }
        if fighter(c, &q.target_id)?.hp == 0 {
            for pending in &mut flow_mut(c).queue {
                if pending.actor_id == q.target_id && pending.damage_context.is_none() {
                    pending.cancelled = true;
                }
            }
            close(c);
        }

        record(
            c,
            format!(
                "Multiple Clashes against sequence {seq}: highest counter-damage only, {amount} HP"
            ),
        );
    }
    Ok(())
}
pub fn damage_immune(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> bool {
    enabled(c)
        && ((p.selected_ability == "Lightning Rod" && m["Type"] == "Electric")
            || (p.selected_ability == "Storm Drain" && m["Type"] == "Water"))
}
pub fn hit(c: &mut Campaign, q: &QueuedAction, m: &serde_json::Value) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let p = fighter(c, &q.target_id)?.clone();
    if !reviewed::damage_bypass(c, &reviewed::source(c, q)?, &p)
        && damage_immune(c, &p, m)
        && !p.runtime.ability.flags.contains(&p.selected_ability)
    {
        let rt = &mut fighter_mut(c, &p.id)?.runtime.ability;
        rt.flags.insert(p.selected_ability.clone());
        rt.counters.insert(p.selected_ability.clone(), 1);
        record(
            c,
            format!(
                "{} absorbed {}: damage immunity, first-hit Special +1",
                p.name, q.move_name
            ),
        );
    }
    Ok(())
}
pub fn effect_target(
    c: &mut Campaign,
    q: &QueuedAction,
    cat: &Catalog,
) -> Result<Option<QueuedAction>> {
    if !enabled(c) {
        return Ok(None);
    }
    let m = field::move_data(c, fighter(c, &q.actor_id)?, &move_entry(cat, &q.move_name)?);
    if m["Category"] != "Support" {
        return Ok(None);
    }
    let source = fighter(c, &q.actor_id)?.clone();
    let target = fighter(c, &q.target_id)?.clone();
    let mut next = q.clone();
    if q.field_scope.is_none()
        && target.id != source.id
        && target.selected_ability == "Good As Gold"
    {
        next.cancelled = true;
        record(
            c,
            format!("Good As Gold: {} blocks {}", target.name, q.move_name),
        );
        return Ok(Some(next));
    }
    let field_side = q
        .field_scope
        .as_deref()
        .and_then(|s| s.strip_prefix("opponents-of:"))
        .filter(|side| *side == source.side);
    let reflector = if bypass(c, &source.id) {
        None
    } else if field_side.is_some() {
        let mut choices: Vec<_> = living(c)
            .filter(|p| {
                p.side != source.side
                    && (p.selected_ability == "Magic Bounce"
                        || state(c).lures.iter().any(|l| {
                            l.name == "Magic Coat"
                                && l.target == p.id
                                && (l.round == 0 || l.round == battle(c).unwrap().round)
                        }))
            })
            .map(|p| p.id.clone())
            .collect();
        choices.sort();
        if choices.len() > 1 {
            Some(state(c).redirect_choices.get(&source.id).filter(|id|choices.contains(id)).cloned().ok_or_else(||AppError::new("manual_rule","Several Magic Bounce/Coat sources protect this side; record the GM redirector choice"))?)
        } else {
            choices.first().cloned()
        }
    } else if target.side != source.side
        && (target.selected_ability == "Magic Bounce"
            || state(c).lures.iter().any(|l| {
                l.name == "Magic Coat"
                    && l.target == target.id
                    && (l.round == 0 || l.round == battle(c).unwrap().round)
            }))
    {
        Some(target.id.clone())
    } else {
        None
    };
    if let Some(reflector) = reflector {
        if state(c)
            .coat_rules
            .get(&reflector)
            .is_some_and(|mode| mode == "next_effect")
        {
            state_mut(c)
                .lures
                .retain(|l| l.name != "Magic Coat" || l.target != reflector);
        }
        next.actor_id = reflector.clone();
        next.target_id = source.id.clone();
        if next.field_scope.is_some() {
            next.field_scope = Some(format!("side:{}", source.side));
        }
        state_mut(c).payer = Some((reflector.clone(), source.id.clone()));
        record(
            c,
            format!(
                "{} reflects {} back to {}; no new accuracy/action and no reflection loop",
                fighter(c, &reflector)?.name,
                q.move_name,
                source.name
            ),
        );
        if source.selected_ability == "Good As Gold" && next.field_scope.is_none() {
            next.cancelled = true;
            clear_payer(c);
        }
        return Ok(Some(next));
    }
    Ok(None)
}
pub fn clear_payer(c: &mut Campaign) {
    if enabled(c) {
        state_mut(c).payer = None;
    }
}
pub fn payer(c: &Campaign, id: &str) -> String {
    if enabled(c) {
        if let Some((a, p)) = &state(c).payer {
            if a == id {
                return p.clone();
            }
        }
    }
    id.into()
}
fn trainer(c: &Campaign, id: &str) -> Result<String> {
    let p = special::registered(c, id)?;
    c.sheets[&p.sheet_id]
        .trainer_id
        .clone()
        .ok_or_else(|| error("A controlled switch needs the campaign trainer link"))
}
fn switch_available(c: &Campaign, out: &str) -> Result<()> {
    let trainer = trainer(c, out)?;
    if state(c).switchers.get(&trainer) == Some(&battle(c)?.round) {
        return Err(error(
            "Only one switcher effect per trainer per round; a GM can record an exception",
        ));
    }
    Ok(())
}
fn request(
    c: &mut Campaign,
    q: &QueuedAction,
    out: &str,
    name: &str,
    forced: bool,
    ability: bool,
) -> Result<()> {
    let p = special::registered(c, out)?;
    if !p.deployed || p.hp == 0 {
        return Ok(());
    }
    if state(c).switches.iter().any(|s| s.outgoing == out) {
        return Ok(());
    }
    if !ability || trainer(c, out).is_ok() {
        switch_available(c, out)?;
    }
    let round = battle(c)?.round;
    state_mut(c).switches.push(SwitchRequest {
        id: format!("{}:{}:{name}", q.sequence, out),
        source: q.actor_id.clone(),
        outgoing: out.into(),
        name: name.into(),
        round,
        immediate: !ability && q.timing != "main",
        forced,
        ability,
    });
    Ok(())
}
pub fn move_effect(c: &mut Campaign, q: &QueuedAction, cat: &Catalog) -> Result<bool> {
    if !enabled(c) {
        return Ok(false);
    }
    let m = move_entry(cat, &q.move_name)?;
    if q.move_name == "Air Cutter" {
        return Ok(true);
    }
    if ["Follow Me", "Rage Powder", "Spotlight", "Magic Coat"].contains(&q.move_name.as_str()) {
        if special::target_effects_blocked(c, q, &m) {
            return Ok(true);
        }
        let target = if q.move_name == "Spotlight" {
            q.target_id.clone()
        } else {
            q.actor_id.clone()
        };
        let round = if q.move_name == "Magic Coat" {
            match state(c).coat_rules.get(&q.actor_id).map(String::as_str) { Some("scene")=>0, Some("next_effect"|"round")=>battle(c)?.round, _=>return Err(AppError::new("manual_rule","Magic Coat has no stated duration; GM must record next effect, round or scene duration")) }
        } else {
            battle(c)?.round
        };
        state_mut(c)
            .lures
            .retain(|l| l.target != target || l.name != q.move_name);
        state_mut(c).lures.push(Lure {
            source: q.actor_id.clone(),
            target,
            name: q.move_name.clone(),
            round,
        });
        // A successful early lure can redirect the original pending single-target attack.
        let work = flow(c)?
            .queue
            .iter()
            .filter(|p| {
                p.sequence != q.sequence && p.damage_context.is_none() && p.timing != "before"
            })
            .cloned()
            .collect::<Vec<_>>();
        for pending in work {
            let data = field::move_data(
                c,
                fighter(c, &pending.actor_id)?,
                &move_entry(cat, &pending.move_name)?,
            );
            if pending.field_scope.is_some() || pending.targets.len() > 1 || data["Target"] != "Foe"
            {
                continue;
            }
            let ids = redirect_move(
                c,
                &pending.actor_id,
                vec![pending.target_id.clone()],
                &data,
                cat,
            )?;
            if let Some(target) = ids.first() {
                if *target != pending.target_id {
                    let item = flow_mut(c)
                        .queue
                        .iter_mut()
                        .find(|p| p.sequence == pending.sequence)
                        .unwrap();
                    item.target_id = target.clone();
                    item.targets = vec![TargetImpact {
                        target_id: target.clone(),
                        responded: false,
                        avoided: false,
                        resolved: false,
                    }];
                }
            }
        }
        return Ok(true);
    }
    let switcher = m["Attributes"]["SwitcherMove"].clone();
    if switcher.is_null() || switcher == false {
        return Ok(false);
    }
    if q.move_name == "Shed Tail" && !decoys::enabled(c) {
        return Err(AppError::new(
            "manual_rule",
            "Shed Tail requires the current decoy engine",
        ));
    }
    let forced = switcher == "Foe";
    if special::target_effects_blocked(c, q, &m)
        && !(decoys::intercepted(c, q) && !forced && m["Category"] != "Support")
    {
        return Ok(true);
    }
    let out = if forced { &q.target_id } else { &q.actor_id };
    let p = fighter(c, out)?.clone();
    if forced && p.side != fighter(c, &q.actor_id)?.side && p.selected_ability == "Suction Cups" {
        record(
            c,
            format!("{} prevents forced switching with Suction Cups", p.name),
        );
        return Ok(true);
    }
    if !forced && (extra::blocked(c, &p, cat) || special::blocked(c, out)) {
        return Err(error(
            "This participant is blocked from switching; record a GM exception",
        ));
    }
    if q.move_name == "Parting Shot" {
        for key in ["Strength", "Special"] {
            extra::change_stat(c, &q.actor_id, &q.target_id, key, -1)?;
        }
    }
    if q.move_name == "Shed Tail" {
        decoys::cast(c, out, &q.move_name, cat)?;
    }
    request(c, q, out, &q.move_name, forced, false)?;
    Ok(true)
}
pub fn reserves(c: &Campaign, out: &str) -> Vec<String> {
    let Ok(p) = special::registered(c, out) else {
        return vec![];
    };
    let Ok(t) = trainer(c, out) else {
        return vec![];
    };
    c.battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .filter(|v| {
            !v.deployed
                && v.hp > 0
                && v.side == p.side
                && v.owner_id == p.owner_id
                && c.sheets[&v.sheet_id].trainer_id.as_ref() == Some(&t)
        })
        .map(|v| v.id.clone())
        .collect()
}
fn resolve_switch(
    c: &mut Campaign,
    actor: &str,
    id: &str,
    into: Option<&str>,
    ruling: &str,
    cat: &Catalog,
) -> Result<()> {
    if abilities::pending(c) || field::pending(c) {
        return Err(error("Resolve pending ability/field events first"));
    }
    let s = state(c)
        .switches
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or_else(|| error("Switch request is no longer pending"))?;
    if !s.immediate && !flow(c)?.queue.is_empty() {
        return Err(error(
            "Finish the attack chain before replacing this participant",
        ));
    }
    let out = special::registered(c, &s.outgoing)?.clone();
    if actor != out.owner_id && actor != c.gm_id {
        return Err(error(
            "Only the outgoing owner or appointed GM may choose its reserve",
        ));
    }
    if ruling != "replace" {
        gm(c, actor)?;
        if !["withdraw", "waive"].contains(&ruling) {
            return Err(error("Choose replace, withdraw or a recorded GM waiver"));
        }
    }
    if ruling == "waive" {
        state_mut(c).switches.retain(|v| v.id != id);
        return Ok(());
    }
    let entry = if ruling == "replace" {
        let key = into.ok_or_else(|| error("Choose a conscious registered reserve"))?;
        if !reserves(c, &s.outgoing).contains(&key.to_owned()) {
            return Err(error(
                "Reserve must belong to the same trainer, owner and side",
            ));
        }
        Some(special::registered(c, key)?.clone())
    } else {
        if !reserves(c, &s.outgoing).is_empty() {
            return Err(error(
                "Conscious reserves exist; select one or record a GM waiver",
            ));
        }
        None
    };
    let trainer = trainer(c, &out.id)?;
    switch_available(c, &out.id)?;
    let boosts: Vec<_> = if s.name == "Baton Pass" {
        ["Strength", "Dexterity", "Vitality", "Special", "Insight"]
            .into_iter()
            .filter_map(|key| {
                let n = special::modifier(c, &out, key)
                    .unwrap_or_else(|| out.runtime.modifiers.get(key).copied().unwrap_or(0))
                    .max(0);
                if n > 0 {
                    Some((key.to_owned(), n))
                } else {
                    None
                }
            })
            .collect()
    } else {
        vec![]
    };
    if s.name == "Baton Pass" {
        if let Some(incoming) = &entry {
            stat_channels::transfer(c, &out.id, &incoming.id, "Baton Pass")?;
        }
    }
    decoys::switch(c, &out.id, entry.as_ref().map(|p| p.id.as_str()), &s.name);
    let cursor = flow(c)?.order.get(flow(c)?.cursor).cloned();
    let old_index = flow(c)?.order.iter().position(|v| v == &out.id);
    let round = battle(c)?.round;
    let p = fighter_mut(c, &out.id)?;
    if s.name == "Ally Switch" {
        p.actions = p.actions.saturating_sub(1);
    }
    p.deployed = false;
    p.field_slot = None;
    p.runtime.modifiers.clear();
    p.runtime.pain_ignored = 0;
    p.runtime
        .statuses
        .retain(|s| !["Confusion", "Flinch", "Disable", "In Love"].contains(&s.kind.as_str()));
    if let Some(entry) = entry {
        let incoming = entry.id.clone();
        let p = fighter_mut(c, &incoming)?;
        p.deployed = true;
        p.field_slot = out.field_slot;
        p.available_round = round;
        p.actions = out
            .actions
            .saturating_sub(u32::from(s.name == "Ally Switch"));
        p.initiative = out.initiative;
        p.initiative_rolled = out.initiative_rolled;
        if s.name == "Baton Pass" {
            if !stat_channels::enabled(c) {
                for (key, n) in &boosts {
                    let value = special::record_modifier(c, &out.id, &incoming, key, *n)?;
                    fighter_mut(c, &incoming)?
                        .runtime
                        .modifiers
                        .insert(key.clone(), value);
                }
            }
            state_mut(c).baton.insert(out.id.clone(), incoming.clone());
        }
        if let Some(index) = old_index {
            flow_mut(c).order[index] = incoming.clone();
        }
        // Ally Switch explicitly permits the incoming ally to react to the pending attack.
        if s.immediate {
            for q in &mut flow_mut(c).queue {
                if q.damage_context.is_none() && !q.damage_done {
                    if q.target_id == out.id {
                        q.target_id = incoming.clone();
                    }
                    for t in &mut q.targets {
                        if t.target_id == out.id && !t.resolved {
                            t.target_id = incoming.clone();
                            t.responded = false;
                            t.avoided = false;
                        }
                    }
                }
            }
            flow_mut(c).reacted.remove(&incoming);
        }
        if cursor.as_deref() == Some(&out.id) {
            flow_mut(c).deferred = Some(incoming.clone());
        }
        record(
            c,
            format!(
                "{}: {} -> {}; inherits slot, initiative and remaining action budget",
                s.name, out.name, entry.name
            ),
        );
    } else {
        flow_mut(c).order.retain(|v| v != &out.id);
        record(
            c,
            format!(
                "{} withdrew {} without a reserve; GM decides whether the battle ends",
                s.name, out.name
            ),
        );
    }
    let f = flow_mut(c);
    f.cursor = cursor
        .and_then(|id| f.order.iter().position(|v| v == &id))
        .unwrap_or_else(|| f.cursor.min(f.order.len().saturating_sub(1)));
    state_mut(c).switchers.insert(trainer, round);
    state_mut(c).switches.retain(|v| v.id != id);
    prepare_targets(c);
    let _ = cat;
    Ok(())
}
pub fn baton_blocked(c: &Campaign, source: &str, target: &str, key: &str, n: i32) -> bool {
    enabled(c)
        && n > 0
        && source == target
        && ["Strength", "Dexterity", "Vitality", "Special", "Insight"].contains(&key)
        && state(c)
            .baton
            .get(target)
            .is_some_and(|id| special::registered(c, id).is_ok_and(|p| p.hp > 0))
}
pub fn lifecycle(before: &Campaign, c: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    flush_clashes(c, cat)?;
    let round = battle(c)?.round;
    let active: BTreeSet<_> = living(c).map(|p| p.id.clone()).collect();
    state_mut(c).lures.retain(|l| {
        (l.round == 0 || l.round == round)
            && active.contains(&l.source)
            && active.contains(&l.target)
    });
    state_mut(c)
        .switches
        .retain(|s| active.contains(&s.outgoing));
    let candidates = living(c)
        .filter(|p| {
            ["Emergency Exit", "Wimp Out"].contains(&p.selected_ability.as_str())
                && p.hp <= p.max_hp / 2
                && before.battle.as_ref().is_some_and(|b| {
                    b.combatants
                        .iter()
                        .any(|old| old.id == p.id && old.hp > old.max_hp / 2)
                })
        })
        .cloned()
        .collect::<Vec<_>>();
    for p in candidates {
        if trainer(c, &p.id).is_ok_and(|trainer| state(c).switchers.get(&trainer) == Some(&round)) {
            record(
                c,
                format!(
                    "{} cannot trigger a second switcher effect this round; GM ruling required",
                    p.name
                ),
            );
            continue;
        }
        let q = QueuedAction {
            actor_id: p.id.clone(),
            target_id: p.id.clone(),
            move_name: p.selected_ability.clone(),
            sequence: c.revision + 1,
            successes: 0,
            critical: false,
            timing: "main".into(),
            priority: 0,
            cancelled: false,
            damage_done: true,
            damage: 0,
            raw_damage_successes: 0,
            ice: false,
            special_index: 0,
            targets: vec![],
            field_scope: None,
            shared_dice: vec![],
            damage_modifiers: None,
            damage_context: None,
            defense_key: String::new(),
        };
        request(c, &q, &p.id, &p.selected_ability, false, true)?;
    }
    let _ = cat;
    Ok(())
}
pub fn preview_notes(c: &Campaign, a: &Action, cat: &Catalog) -> Result<Vec<String>> {
    if !enabled(c) {
        return Ok(vec![]);
    }
    let mut out = vec![];
    if let Action::Accuracy {
        combatant_id,
        move_name,
        ..
    }
    | Action::AccuracyTargets {
        combatant_id,
        move_name,
        ..
    } = a
    {
        let m = field::move_data(c, fighter(c, combatant_id)?, &move_entry(cat, move_name)?);
        let (ids, _) = plan(c, combatant_id, &selected(a), &m)?;
        let actual = redirect_move(c, combatant_id, ids.clone(), &m, cat)?;
        if ids != actual {
            out.push(format!(
                "Redirection: {}",
                actual
                    .iter()
                    .map(|id| fighter(c, id).unwrap().name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        for id in actual {
            let p = fighter(c, &id)?;
            if ["Magic Bounce", "Good As Gold"].contains(&p.selected_ability.as_str())
                && m["Category"] == "Support"
            {
                out.push(format!(
                    "{}: {} applies to incoming support effects",
                    p.name, p.selected_ability
                ));
            }
        }
    }
    out.push("Targeting/reaction rules preserve one Evasion per turn. Reaction moves share its slot; Clash may answer each eligible incoming hit. Switch effects use registered reserves and require an owner choice, not host approval.".into());
    Ok(out)
}
