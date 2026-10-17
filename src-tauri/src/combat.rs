//! Versioned turn/reaction engine. Sources: corebook pp.28,57-59,70-87.
use crate::{catalog::Catalog, model::*, rules};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use ts_rs::TS;
pub(crate) mod abilities;
pub(crate) mod ability_changes;
pub(crate) mod decoys;
pub(crate) mod extra;
pub(crate) mod field;
pub(crate) mod special;
pub(crate) mod tactics;
mod targeting;
pub(crate) mod transfers;
use targeting::*;
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct CombatState {
    #[serde(default)]
    pub ability: abilities::AbilityState,
    pub will: i32,
    pub pain_ignored: i32,
    pub chance_round: u32,
    pub fate_round: u32,
    pub fate_pending: bool,
    pub statuses: Vec<Status>,
    pub confusion_checked: u32,
    pub confusion_resisted: u32,
    pub love_resisted_turn: u32,
    pub flinch_round: u32,
    pub shield: i32,
    pub shield_round: u32,
    pub shield_streak: u32,
    pub healed: i32,
    pub recharge_actions: u32,
    pub recharge_round: u32,
    pub love_checked_turn: u32,
    pub will_enabled: bool,
    pub adjudicated_round: u32,
    pub modifiers: BTreeMap<String, i32>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Status {
    pub kind: String,
    pub detail: String,
    pub rounds: Option<u32>,
    pub progress: i32,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct TurnFlow {
    #[serde(default)]
    pub transfers: transfers::TransferState,
    #[serde(default)]
    pub ability_changes: ability_changes::AbilityChanges,
    #[serde(default)]
    pub decoys: decoys::DecoyState,
    #[serde(default)]
    pub special: special::SpecialState,
    #[serde(default)]
    pub tactics: tactics::TacticsState,
    #[serde(default)]
    pub extra: extra::ExtraState,
    #[serde(default)]
    pub battlefield: field::Battlefield,
    #[serde(default)]
    pub ability_rules: bool,
    #[serde(default)]
    pub ability_events: Vec<abilities::AbilityEvent>,
    #[serde(default)]
    pub intimidate: BTreeMap<String, Vec<String>>,
    pub started: bool,
    pub order: Vec<String>,
    pub cursor: usize,
    pub turn: u32,
    pub held: BTreeSet<String>,
    pub reacted: BTreeSet<String>,
    pub queue: Vec<QueuedAction>,
    pub window: bool,
    #[serde(default)]
    pub deferred: Option<String>,
    #[serde(default)]
    pub ranges: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub covers: BTreeMap<String, String>,
    #[serde(default)]
    pub field_effects: Vec<FieldEffect>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct QueuedAction {
    #[serde(default)]
    pub special_index: u32,
    #[serde(default)]
    pub damage_context: Option<abilities::DamageContext>,
    pub actor_id: String,
    pub target_id: String,
    pub move_name: String,
    pub sequence: u32,
    pub successes: i32,
    pub critical: bool,
    pub timing: String,
    pub priority: i32,
    pub cancelled: bool,
    pub damage_done: bool,
    pub damage: i32,
    #[serde(default)]
    pub raw_damage_successes: u32,
    #[serde(default)]
    pub ice: bool,
    #[serde(default)]
    pub targets: Vec<TargetImpact>,
    #[serde(default)]
    pub field_scope: Option<String>,
    #[serde(default)]
    pub shared_dice: Vec<u8>,
    #[serde(default)]
    pub damage_modifiers: Option<[i32; 3]>,
    #[serde(default)]
    pub defense_key: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct ActionPreview {
    pub dice: u32,
    pub warnings: Vec<String>,
    pub source: String,
}
fn error(text: &str) -> AppError {
    AppError::new("battle", text)
}
fn battle(c: &Campaign) -> Result<&Battle> {
    c.battle
        .as_ref()
        .filter(|b| b.active)
        .ok_or_else(|| error("No active battle"))
}
fn fighter<'a>(c: &'a Campaign, id: &str) -> Result<&'a Combatant> {
    battle(c)?
        .combatants
        .iter()
        .find(|p| p.id == id && p.deployed)
        .ok_or_else(|| error("Unknown active combatant"))
}
fn fighter_mut<'a>(c: &'a mut Campaign, id: &str) -> Result<&'a mut Combatant> {
    c.battle
        .as_mut()
        .unwrap()
        .combatants
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or_else(|| error("Missing combatant"))
}
fn flow(c: &Campaign) -> Result<&TurnFlow> {
    battle(c)?
        .flow
        .as_ref()
        .ok_or_else(|| error("This battle uses legacy sequencing"))
}
fn flow_mut(c: &mut Campaign) -> &mut TurnFlow {
    c.battle.as_mut().unwrap().flow.as_mut().unwrap()
}
fn owner(c: &Campaign, actor: &str, id: &str) -> Result<()> {
    if actor != c.gm_id && fighter(c, id)?.owner_id != actor {
        Err(AppError::new(
            "permission",
            "Only control your own combatant",
        ))
    } else {
        Ok(())
    }
}
fn gm(c: &Campaign, actor: &str) -> Result<()> {
    if c.gm_id == actor {
        Ok(())
    } else {
        Err(AppError::new(
            "permission",
            "Only the appointed GM can adjudicate",
        ))
    }
}
fn reason(text: &str) -> Result<()> {
    if text.trim().is_empty() || text.len() > 1000 {
        Err(error("Record a reason, up to 1000 characters"))
    } else {
        Ok(())
    }
}
fn old(c: &Campaign) -> Campaign {
    let mut c = c.clone();
    c.engine = PREVIOUS_ENGINE.into();
    c
}
fn status(p: &Combatant, kind: &str) -> bool {
    p.runtime.statuses.iter().any(|s| s.kind == kind)
}
fn confusion(c: &Campaign, p: &Combatant) -> i32 {
    if status(p, "Confusion") && p.runtime.confusion_resisted != battle(c).unwrap().round {
        match crate::creation::rank(&c.sheets[&p.sheet_id].rank).unwrap_or(0) {
            0..=2 => 1,
            3..=5 => 2,
            _ => 3,
        }
    } else {
        0
    }
}
fn ready(c: &Campaign, id: &str) -> Result<()> {
    if special::carried(c, id) {
        return Err(error(
            "This participant is carried out of range by Sky Drop",
        ));
    }
    if field::pending(c) {
        return Err(error("Resolve pending battlefield checks first"));
    }
    if abilities::pending(c) {
        return Err(error(
            "Resolve pending ability triggers before another action",
        ));
    }
    let b = battle(c)?;
    let p = fighter(c, id)?;
    crate::teams::ready(p, b.round)?;
    if !b.automatic_validation {
        return Ok(());
    }
    if p.hp == 0 || p.actions >= 5 {
        return Err(error("Fainted or all five actions spent"));
    }
    if (status(p, "Sleep") && !(extra::enabled(c) && p.selected_ability == "Comatose"))
        || status(p, "Frozen Solid")
        || status(p, "Flinch")
        || (p.runtime.recharge_actions > 0 && p.runtime.recharge_round <= b.round)
    {
        return Err(error(
            "Resolve sleep, ice, flinch or recharge before acting",
        ));
    }
    if status(p, "Confusion") && p.runtime.confusion_checked != b.round {
        return Err(error("Check Confusion at the beginning of the round"));
    }
    Ok(())
}
fn turn(c: &Campaign, id: &str) -> Result<()> {
    if !battle(c)?.automatic_validation {
        return Ok(());
    }
    let f = flow(c)?;
    if !f.started || f.order.get(f.cursor).is_none_or(|v| v != id) || !f.queue.is_empty() {
        return Err(error("Wait for your turn and resolve the pending chain"));
    }
    Ok(())
}
fn advance(c: &mut Campaign) {
    if special::preparing(c) {
        return;
    }
    let Some(f) = c.battle.as_ref().and_then(|b| b.flow.as_ref()) else {
        return;
    };
    let n = f.order.len();
    if n == 0 {
        return;
    }
    let start = f.cursor;
    let mut choice = f
        .deferred
        .as_ref()
        .and_then(|id| f.order.iter().position(|v| v == id))
        .filter(|index| {
            c.battle.as_ref().unwrap().combatants.iter().any(|p| {
                p.id == f.order[*index]
                    && p.hp > 0
                    && p.deployed
                    && p.actions < 5
                    && !f.held.contains(&p.id)
            })
        });
    for offset in 1..=if choice.is_none() { n } else { 0 } {
        let index = (start + offset) % n;
        let id = &f.order[index];
        if c.battle.as_ref().unwrap().combatants.iter().any(|p| {
            &p.id == id
                && p.deployed
                && p.hp > 0
                && p.actions < 5
                && p.available_round <= c.battle.as_ref().unwrap().round
        }) && !f.held.contains(id)
        {
            choice = Some(index);
            break;
        }
    }
    let f = flow_mut(c);
    f.turn += 1;
    f.deferred = None;
    f.reacted.clear();
    if let Some(index) = choice {
        f.cursor = index;
    } else {
        f.order.clear();
        f.cursor = 0;
    }
}
fn finish(c: &mut Campaign) {
    if finish_target(c) {
        return;
    }
    flow_mut(c).queue.remove(0);
    while flow(c).unwrap().queue.first().is_some_and(|q| {
        q.cancelled
            || (q.damage_context.is_none() && fighter(c, &q.actor_id).is_ok_and(|p| p.hp == 0))
    }) {
        flow_mut(c).queue.remove(0);
    }
    if flow(c).unwrap().queue.is_empty() {
        if !special::continue_series(c) {
            advance(c)
        }
    } else {
        prepare_targets(c);
    }
}
fn close(c: &mut Campaign) {
    let f = flow_mut(c);
    f.window = false;
    f.queue.sort_by_key(|q| match q.timing.as_str() {
        "before" => (0, -q.priority),
        "main" => (1, 0),
        _ => (2, q.priority),
    });
    prepare_targets(c);
    while flow(c).unwrap().queue.first().is_some_and(|q| q.cancelled) {
        finish(c);
        if flow(c).unwrap().queue.is_empty() {
            break;
        }
    }
}
fn incoming(c: &Campaign) -> Result<&QueuedAction> {
    let f = flow(c)?;
    (if tactics::enabled(c) {
        f.queue.first()
    } else {
        f.queue.last()
    })
    .ok_or_else(|| error("No incoming action"))
}
fn current(c: &Campaign) -> Result<&QueuedAction> {
    if abilities::pending(c) {
        return Err(error(
            "Resolve pending ability triggers before continuing the move",
        ));
    }
    let f = flow(c)?;
    if f.window {
        return Err(error("The target must react or pass before resolution"));
    }
    f.queue.first().ok_or_else(|| error("No pending action"))
}
fn move_entry(catalog: &Catalog, name: &str) -> Result<serde_json::Value> {
    catalog
        .find("moves", "Name", name)
        .ok_or_else(|| error("Unknown move"))
}
pub fn apply_stats(c: &Campaign, p: &Combatant, s: &mut Sheet) {
    if extra::enabled(c) {
        extra::apply_stats(c, p, s);
        return;
    }
    for (key, amount) in &p.runtime.modifiers {
        if let Some(v) = s.stats.get_mut(key) {
            *v = (*v + amount).clamp(1, 10);
        }
    }
    if status(p, "Paralysis") {
        if let Some(v) = s.stats.get_mut("Dexterity") {
            *v = (*v - 2).max(0);
        }
    }
    abilities::stats(c, p, s);
}
fn check_count(dice: &[u8], count: u32) -> Result<()> {
    if dice.len() != count as usize || dice.iter().any(|d| !(1..=6).contains(d)) {
        Err(error("Invalid host dice results"))
    } else {
        Ok(())
    }
}
fn action_success(c: &mut Campaign, id: &str, dice: &[u8], base: u32) -> i32 {
    let p = fighter(c, id).unwrap();
    let chance = p.runtime.chance_round == battle(c).unwrap().round;
    let raw = rules::successes(&dice[..base as usize])
        + i32::from(
            chance
                && dice[..base as usize].iter().any(|d| *d < 4)
                && dice.get(base as usize).is_some_and(|d| *d >= 4),
        );
    let penalty = confusion(c, p) + (rules::pain(p.hp, p.max_hp) - p.runtime.pain_ignored).max(0);
    fighter_mut(c, id).unwrap().runtime.fate_pending = false;
    (raw - penalty).max(0)
}
fn shield_attempt(c: &mut Campaign, id: &str, m: &serde_json::Value) -> Result<i32> {
    if m["Attributes"]["ShieldMove"] != true {
        return Ok(0);
    }
    let round = battle(c)?.round;
    let rt = &mut fighter_mut(c, id)?.runtime;
    if rt.shield_round == round {
        return Err(error("Only one shield attempt per round"));
    }
    let penalty = if rt.shield_round > 0 && rt.shield_round + 1 == round {
        2_i32.saturating_pow(rt.shield_streak.min(7))
    } else {
        0
    };
    rt.shield_streak = if rt.shield_round + 1 == round {
        rt.shield_streak + 1
    } else {
        1
    };
    rt.shield_round = round;
    Ok(penalty)
}
pub fn type_ability(c: &Campaign, p: &Combatant, move_type: &str) -> bool {
    c.battle.as_ref().is_some_and(|b| b.flow.is_some())
        && matches!(
            (p.selected_ability.as_str(), move_type),
            ("Blaze", "Fire") | ("Overgrow", "Grass") | ("Swarm", "Bug")
        )
}
fn pain_exempt(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> Result<bool> {
    let s = rules::effective_sheet(c, p)?;
    let key = m["Accuracy1"].as_str().unwrap_or("");
    Ok(type_ability(c, p, m["Type"].as_str().unwrap_or(""))
        || key.split('/').any(|a| {
            ["Vitality", "Will"].contains(&a.trim())
                && rules::score(&s, a.trim()).ok() == rules::score(&s, key).ok()
        }))
}
fn sync_labels(c: &mut Campaign) {
    if let Some(b) = c.battle.as_mut() {
        for p in &mut b.combatants {
            if p.hp == 0 {
                p.runtime
                    .statuses
                    .retain(|s| !s.kind.starts_with("Burn") && !s.kind.contains("Poison"));
            }
            p.conditions.retain(|v| !known_status(v));
            p.conditions
                .extend(p.runtime.statuses.iter().map(|s| s.kind.clone()));
        }
    }
}
fn refresh_initiative(before: &Campaign, after: &mut Campaign) -> Result<()> {
    let Some(old) = before.battle.as_ref().filter(|b| b.active) else {
        return Ok(());
    };
    let Some(new) = after.battle.as_ref() else {
        return Ok(());
    };
    let mut updates = Vec::new();
    for p in new.combatants.iter().filter(|p| p.initiative_rolled) {
        if let Some(previous) = old
            .combatants
            .iter()
            .find(|v| v.id == p.id && v.initiative_rolled)
        {
            let old_stats =
                crate::creation::profile_stats(&rules::effective_sheet(before, previous)?);
            let new_stats = crate::creation::profile_stats(&rules::effective_sheet(after, p)?);
            let delta = new_stats["Initiative"] - old_stats["Initiative"];
            updates.push((p.id.clone(), delta));
        }
    }
    for (id, delta) in updates {
        let p = fighter_mut(after, &id)?;
        p.initiative = p
            .initiative
            .checked_add(delta)
            .ok_or_else(|| error("Initiative overflow"))?;
    }
    Ok(())
}
fn known_status(k: &str) -> bool {
    [
        "Burn",
        "Burn 2",
        "Burn 3",
        "Poison",
        "Badly Poisoned",
        "Paralysis",
        "Sleep",
        "Confusion",
        "Flinch",
        "Disable",
        "Frozen Solid",
        "In Love",
    ]
    .contains(&k)
}
fn reaction_pool(c: &Campaign, id: &str, kind: &str, name: &str, cat: &Catalog) -> Result<u32> {
    special::reaction_guard(c, id, kind, name, cat)?;
    let p = fighter(c, id)?;
    field::reaction(c, p, kind, cat)?;
    extra::reaction(c, p, kind, None)?;
    ready(c, id)?;
    let f = flow(c)?;
    // Table rules: Evasion shares the turn's reaction slot with reaction moves.
    // Clash can answer each eligible incoming attack independently of that slot.
    if kind != "clash" && f.reacted.contains(id) {
        return Err(error(if kind == "evade" {
            "Evasion is limited to once per turn and shares the reaction slot with reaction moves"
        } else {
            "One reaction per turn; your reaction slot has already been spent"
        }));
    }
    if !f.window {
        return Err(error("Reaction window must be open"));
    }
    let incoming = incoming(c)?;
    if incoming.actor_id == id
        || (incoming.target_id != id && !(tactics::enabled(c) && kind == "move"))
    {
        return Err(error("Maneuvers answer attacks targeting you; reaction moves can intervene during another participant's action"));
    }
    let m = field::move_data(
        c,
        fighter(c, &incoming.actor_id)?,
        &move_entry(cat, &incoming.move_name)?,
    );
    if kind == "evade" || kind == "clash" {
        if incoming.timing == "after" || incoming.priority >= 6 {
            return Err(error(
                "Reaction 6 cannot answer a late or higher/equal reaction",
            ));
        }
        if !["Foe", "All Foes", "Random Foe", "Area"].contains(&m["Target"].as_str().unwrap_or(""))
            || incoming.field_scope.is_some()
        {
            return Err(AppError::new(
                "manual_rule",
                "This target pattern requires adjudication",
            ));
        }
        if kind == "evade" && m["Attributes"]["NeverMiss"].as_bool() == Some(true) {
            return Err(error("Never Miss cannot be evaded"));
        }
        if crate::creation::SOCIALS.contains(&m["Accuracy1"].as_str().unwrap_or_default()) {
            return Err(error("Social-attribute moves cannot be evaded or clashed"));
        }
        let s = rules::effective_sheet(c, p)?;
        let source = if kind == "evade" {
            "Dexterity"
        } else {
            let cm = move_entry(cat, name)?;
            if m["Target"] == "Area" && cm["Target"] != "Area" {
                return Err(error("Area attacks can only be Clashed with an Area move"));
            }
            if m["Category"] == "Support"
                || m["Attributes"]["IgnoreDefenses"] == true
                || !c.sheets[&p.sheet_id].moves.iter().any(|v| v == name)
                || (p.used_moves.iter().any(|v| v == name)
                    && !special::repeat(c, id, name)
                    && !(special::enabled(c)
                        && flow(c)?
                            .special
                            .charges
                            .get(id)
                            .is_some_and(|q| q.action.move_name == name)))
            {
                return Err(error(
                    "Clash requires an unused learned damaging move against ordinary damage",
                ));
            }
            match cm["Category"].as_str() {
                Some("Physical") => "Strength",
                Some("Special") => "Special",
                _ => return Err(error("Use a Physical or Special move to Clash")),
            }
        };
        let n = rules::score(&s, source)?
            + rules::score(&s, if kind == "evade" { "Evasion" } else { "Clash" })?
            + if crate::creation::rank(&s.rank).is_some_and(|r| r >= 6) {
                2
            } else {
                0
            };
        return u32::try_from(n)
            .ok()
            .filter(|v| *v > 0 && *v <= 100)
            .ok_or_else(|| error("Reaction pool requires manual adjudication"));
    }
    if kind != "move" {
        return Err(error("Choose evade, clash or a reaction move"));
    }
    let m = field::move_data(c, p, &move_entry(cat, name)?);
    extra::reaction(c, p, kind, Some(&m))?;
    let number = |key| {
        m["Attributes"][key]
            .as_str()
            .and_then(|s| s.parse::<i32>().ok())
            .or_else(|| m["Attributes"][key].as_i64().map(|n| n as i32))
    };
    tactics::priority_guard(c, p, &m)?;
    let (timing, n) = if let Some(n) = number("Reaction") {
        ("before", n)
    } else if let Some(n) = number("LateReaction") {
        ("after", n)
    } else {
        return Err(error("This move is not a reaction"));
    };
    if (incoming.timing == "after" && timing == "before")
        || (incoming.timing == timing && n <= incoming.priority)
    {
        return Err(error(
            "Reaction priority must increase; early reactions cannot answer late reactions",
        ));
    }
    if !tactics::enabled(c)
        && m["Target"] != "Self"
        && m["Target"] != "Foe"
        && !(special::enabled(c) && special::ally_shield(&m))
    {
        return Err(AppError::new(
            "manual_rule",
            "Reaction target pattern requires GM adjudication",
        ));
    }
    if p.runtime
        .statuses
        .iter()
        .any(|s| s.kind == "Disable" && s.detail == name)
    {
        return Err(error("This reaction move is disabled"));
    }
    if m["Attributes"]["ShieldMove"] == true && p.runtime.shield_round == battle(c)?.round {
        return Err(error("Only one shield attempt per round"));
    }
    Ok(rules::accuracy(&old(c), id, name, 0, cat)?.0)
}
fn ice_attack(c: &Campaign, id: &str, target: &str, name: &str, cat: &Catalog) -> Result<u32> {
    let p = fighter(c, id)?;
    let t = fighter(c, target)?;
    let b = battle(c)?;
    if !flow(c)?.started
        || !flow(c)?.order.is_empty()
        || !flow(c)?.queue.is_empty()
        || !status(t, "Frozen Solid")
    {
        return Err(error(
            "Ice attempts take place after all turns, before advancing the round",
        ));
    }
    if p.hp == 0
        || p.actions >= 5
        || (id != target && p.side != t.side)
        || status(p, "Sleep")
        || status(p, "Flinch")
        || (p.runtime.recharge_actions > 0 && p.runtime.recharge_round <= b.round)
    {
        return Err(error(
            "Only the frozen subject or an ally with an available action can attack ice",
        ));
    }
    if status(p, "Confusion") && p.runtime.confusion_checked != b.round {
        return Err(error("Check Confusion before attacking ice"));
    }
    let m = move_entry(cat, name)?;
    if !["Physical", "Special"].contains(&m["Category"].as_str().unwrap_or(""))
        || m["Power"].as_i64().is_none_or(|n| n <= 0)
        || m["Damage1"]
            .as_str()
            .is_none_or(|k| k == "Varies" || k == "SetDamage")
    {
        return Err(AppError::new("manual_rule","Ice attack requires a reviewed ordinary damage formula; GM can record ice removal for other effects"));
    }
    if p.runtime
        .statuses
        .iter()
        .any(|s| s.kind == "Disable" && s.detail == name)
    {
        return Err(error("Move disabled"));
    }
    Ok(rules::accuracy(&old(c), id, name, 0, cat)?.0)
}
fn ice_damage_pool(c: &Campaign, q: &QueuedAction, modifier: i32, cat: &Catalog) -> Result<u32> {
    let p = fighter(c, &q.actor_id)?;
    let s = rules::effective_sheet(c, p)?;
    let m = move_entry(cat, &q.move_name)?;
    let stab = i32::from(field::types(c, p, cat)?.iter().any(|t| m["Type"] == *t));
    let conditional = type_ability(c, p, m["Type"].as_str().unwrap_or(""));
    let n = (rules::score(&s, m["Damage1"].as_str().unwrap_or(""))?
        + m["Power"].as_i64().unwrap_or(0) as i32
        - if field::weather(c, Some(fighter(c, &q.target_id)?)) == Some(field::Weather::Snow) {
            3
        } else {
            2
        })
    .max(1)
        + stab
        + if q.critical { 2 } else { 0 }
        + if conditional && p.hp <= p.max_hp / 2 {
            2
        } else {
            0
        }
        + modifier;
    if !(1..=100).contains(&n) {
        return Err(error("Ice damage pool outside 1-100 dice"));
    }
    Ok(n as u32)
}
pub fn dice_count(c: &Campaign, a: &Action, cat: &Catalog) -> Result<u32> {
    if let Action::AbilityTransfer { command, .. } = a {
        return transfers::dice(c, command);
    }
    tactics::phase(c, a)?;
    abilities::phase(c, a)?;
    field::phase(c, a)?;
    let base = match a {
        Action::Tactics { command, .. } => return tactics::dice(c, command, cat),
        Action::SpecialMove { command, .. } => return special::dice(c, command, cat),
        Action::Battlefield { command, .. } => return field::dice(c, command),
        Action::ResolveAbility {
            event_id, choice, ..
        } => return abilities::dice(c, event_id, choice),
        Action::CoverAlly {
            combatant_id,
            ally_id,
            ..
        } => cover_pool(c, combatant_id, ally_id, cat)?,
        Action::BreakIce {
            combatant_id,
            target_id,
            move_name,
            ..
        } => ice_attack(c, combatant_id, target_id, move_name, cat)?,
        Action::React {
            combatant_id,
            kind,
            move_name,
            ..
        } => reaction_pool(c, combatant_id, kind, move_name, cat)?,
        Action::StatusCheck {
            combatant_id, kind, ..
        } => {
            let p = fighter(c, combatant_id)?;
            if extra::enabled(c) && p.selected_ability == "Comatose" && kind == "sleep" {
                return Err(error(
                    "Comatose sleep is permanent and does not prevent acting",
                ));
            }
            let s = rules::effective_sheet(c, p)?;
            let n = match kind.as_str() {
                "burn" => {
                    rules::score(&s, "Dexterity")?
                        + rules::score(&s, "Athletic")?
                        + if crate::creation::rank(&s.rank).is_some_and(|r| r >= 6) {
                            2
                        } else {
                            0
                        }
                }
                "sleep" | "confusion" => rules::score(&s, "Insight")?,
                "love" => {
                    rules::score(&s, "Insight")?.max(s.workflow.as_ref().map_or(0, |b| b.loyalty))
                }
                _ => return Err(error("Unknown status check")),
            };
            n.max(0) as u32
        }
        Action::ResolveEffect { .. } => {
            let q = current(c)?;
            effect_dice(c, q, cat)?
        }
        Action::Accuracy { combatant_id, .. } | Action::AccuracyTargets { combatant_id, .. } => {
            let name = match a {
                Action::Accuracy { move_name, .. } | Action::AccuracyTargets { move_name, .. } => {
                    move_name
                }
                _ => unreachable!(),
            };
            special::ready_move(c, combatant_id, name)?;
            turn(c, combatant_id)?;
            if matches!(a, Action::AccuracyTargets { .. }) {
                plan(c, combatant_id, &selected(a), &move_entry(cat, name)?)?;
            }
            rules::dice_count(&old(c), &accuracy_command(a), cat)?
        }
        Action::Damage {
            combatant_id,
            sequence,
            ..
        } => {
            let q = current(c)?;
            if q.actor_id != *combatant_id || q.sequence != *sequence || q.damage_done {
                return Err(error("Resolve the first pending action"));
            }
            if q.field_scope.is_some() {
                return Err(error(
                    "Battlefield actions require a recorded field ruling, not Pokemon damage",
                ));
            }
            if q.ice {
                if let Action::Damage { pool_modifier, .. } = a {
                    return ice_damage_pool(c, q, *pool_modifier, cat);
                }
            }
            if q.move_name == "Sonic Boom" {
                return Ok(0);
            }
            if special::enabled(c) && special::formula_name(&q.move_name) {
                let Action::Damage { pool_modifier, .. } = a else {
                    unreachable!()
                };
                let mut temp = old(c);
                install_pending(&mut temp, q);
                return Ok(rules::damage_pool(
                    &temp,
                    &q.actor_id,
                    q.sequence,
                    *pool_modifier,
                    cat,
                )?
                .dice);
            }
            if !q.targets.is_empty() {
                if let Action::Damage {
                    pool_modifier,
                    success_modifier,
                    damage_modifier,
                    ..
                } = a
                {
                    let modifiers = [*pool_modifier, *success_modifier, *damage_modifier];
                    if q.damage_modifiers.is_some_and(|v| v != modifiers) {
                        return Err(error(
                            "Shared attack modifiers are fixed after the first damage roll",
                        ));
                    }
                    let total = damage_dice(c, q, *pool_modifier, cat)?;
                    return Ok(total.saturating_sub(q.shared_dice.len() as u32));
                }
            }
            let mut temp = old(c);
            install_pending(&mut temp, q);
            rules::dice_count(&temp, a, cat)?
        }
        _ => rules::dice_count(&old(c), a, cat)?,
    };
    let id = match a {
        Action::Accuracy { combatant_id, .. }
        | Action::AccuracyTargets { combatant_id, .. }
        | Action::CoverAlly { combatant_id, .. }
        | Action::React { combatant_id, .. }
        | Action::BreakIce { combatant_id, .. } => Some(combatant_id),
        Action::StatusCheck {
            combatant_id, kind, ..
        } if ["sleep", "burn"].contains(&kind.as_str()) => Some(combatant_id),
        _ => None,
    };
    Ok(base
        + u32::from(id.is_some_and(|id| {
            fighter(c, id).is_ok_and(|p| p.runtime.chance_round == battle(c).unwrap().round)
        })))
}
fn holding_back(c: &Campaign, q: &QueuedAction) -> bool {
    let Ok(p) = fighter(c, &q.actor_id) else {
        return false;
    };
    let resisted = q
        .damage_context
        .as_ref()
        .map_or(p.runtime.love_resisted_turn, |ctx| ctx.love_resisted_turn);
    if resisted == flow(c).map_or(0, |f| f.turn) {
        return false;
    }
    q.damage_context
        .as_ref()
        .map_or(&p.runtime.statuses, |ctx| &ctx.statuses)
        .iter()
        .filter(|s| s.kind == "In Love")
        .any(|s| {
            fighter(c, &s.detail).is_ok_and(|beloved| {
                beloved.hp > 0 && fighter(c, &q.target_id).is_ok_and(|t| t.side == beloved.side)
            })
        })
}
fn install_pending(c: &mut Campaign, q: &QueuedAction) {
    fighter_mut(c, &q.actor_id).unwrap().pending_damage = Some(PendingDamage {
        sequence: q.sequence,
        target_id: q.target_id.clone(),
        move_name: q.move_name.clone(),
        critical: q.critical && !holding_back(c, q),
    });
}
pub fn preview(c: &Campaign, a: &Action, cat: &Catalog) -> Result<ActionPreview> {
    let dice = rules::dice_count(c, a, cat)?;
    if matches!(a, Action::AbilityTransfer { .. }) {
        return Ok(ActionPreview{dice,warnings:vec!["Campaign game-time clock; 24 hours = 1440 minutes. Selection never changes permanent sheets. During combat the GM confirms selection timing; advance time between battles.".into()],source:"MASTER 534,560,563".into()});
    }
    let mut warnings=vec!["Preview rolls no dice and spends no resources. The host validates the current revision on commit.".into()];
    warnings.extend(special::preview_notes(c, a, cat)?);
    if decoys::enabled(c) {
        warnings.push("Decoys: 2 HP; frozen defenses; move effects blocked, environment and existing status still apply. Dynamax, Max Moves and Z-Moves are out of scope.".into());
    }
    if ability_changes::enabled(c) {
        warnings.push("Temporary abilities: scene assignments persist through switching; Trace ends on exit; Gas negates foes in rank-limited range. See ability history for originals and current effects.".into());
    }
    warnings.extend(tactics::preview_notes(c, a, cat)?);
    if let Action::AccuracyTargets {
        combatant_id,
        move_name,
        ..
    } = a
    {
        let m = field::move_data(c, fighter(c, combatant_id)?, &move_entry(cat, move_name)?);
        let (ids, scope) = plan(c, combatant_id, &selected(a), &m)?;
        if let Some(scope) = scope {
            warnings.push(format!(
                "Field scope: {scope}. Reviewed battlefield effects can resolve automatically; unsupported effects require GM adjudication."
            ));
        } else {
            let names = redirect(
                c,
                combatant_id,
                ids,
                m["Attributes"]["SoundMove"] == true
                    || ["Phantom Force", "Shadow Force"].contains(&move_name.as_str()),
            )
            .iter()
            .map(|id| fighter(c, id).map(|p| p.name.clone()))
            .collect::<Result<Vec<_>>>()?;
            warnings.push(format!("Affected Pokemon: {}. One accuracy/action for the move; per-target responses, shared damage dice.", names.join(", ")));
        }
    }
    if field::enabled(c) {
        warnings.push(format!(
            "Effective weather: {:?}; arena effects: {}",
            field::weather(c, None),
            flow(c)?
                .battlefield
                .rules
                .iter()
                .map(|r| format!("{} ({})", r.kind, r.scope))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        if let Action::Damage {
            combatant_id,
            sequence,
            pool_modifier,
            ..
        } = a
        {
            let q = current(c)?;
            let mut temp = old(c);
            install_pending(&mut temp, q);
            let pool = rules::damage_pool(&temp, combatant_id, *sequence, *pool_modifier, cat)?;
            let source = fighter(c, combatant_id)?;
            let target = fighter(c, &q.target_id)?;
            let m = field::move_data(c, source, &move_entry(cat, &q.move_name)?);
            warnings.push(format!("Damage pool {}: attribute {}, power {}, defense {}, STAB {}, critical {}; field dice {:+}, final field reduction {}",pool.dice,pool.attribute,pool.power,pool.defense,pool.stab,pool.critical,field::bonus(c,source,&m)+field::target_bonus(c,target,&m),field::reduction(c,source,target,&m)));
        }
    }
    if let Some(b) = c.battle.as_ref() {
        for p in b.combatants.iter().filter(|p| p.deployed) {
            if ability_changes::enabled(c) && flow(c)?.ability_changes.suppressed.contains(&p.id) {
                warnings.push(format!(
                    "{}: battle ability negated; original {}",
                    p.name,
                    flow(c)?
                        .ability_changes
                        .originals
                        .get(&p.id)
                        .map_or("unresolved", String::as_str)
                ));
                continue;
            }
            if abilities::enabled(c) {
                warnings.push(format!(
                    "{} / {}: {}",
                    p.name,
                    p.selected_ability,
                    abilities::coverage(c, &p.selected_ability)
                        .unwrap_or("Ability mechanics require explicit GM adjudication")
                ));
                continue;
            }
            if let Some(v) = cat.find("abilities", "Name", &p.selected_ability) {
                if let Some(note) = v["Review"]["notes"].as_str() {
                    warnings.push(format!("{} / {}: {note}", p.name, p.selected_ability));
                }
            }
            if ![
                "Blaze",
                "Overgrow",
                "Swarm",
                "Own Tempo",
                "Oblivious",
                "Thick Fat",
                "Huge Power",
                "Pure Power",
            ]
            .contains(&p.selected_ability.as_str())
            {
                warnings.push(format!(
                    "{}: {} ability effects require GM adjudication",
                    p.name, p.selected_ability
                ));
            }
        }
    }
    warnings.push("Unreviewed added effects require recorded GM adjudication. Trainer-target damage, lethal wounds and automatic Will recovery are excluded.".into());
    Ok(ActionPreview {
        dice,
        warnings,
        source: "Pokerole 3.0 reviewed definitions; see rules/coverage.json".into(),
    })
}
fn checkpoint(current: &Campaign, next: &mut Campaign, description: &str) {
    if let Some(b) = current.battle.as_ref().filter(|b| b.active) {
        next.battle_undo.push(BattleCheckpoint {
            event_sequence: current.revision + 1,
            description: description.into(),
            battle: b.clone(),
            ability_inheritance: Some(current.ability_inheritance.clone()),
            resources: b
                .combatants
                .iter()
                .filter_map(|p| {
                    current.sheets.get(&p.sheet_id).map(|s| {
                        (
                            s.id.clone(),
                            BattleResources {
                                hp: s.hp,
                                conditions: s.conditions.clone(),
                                will: s.workflow.as_ref().map(|b| b.will),
                            },
                        )
                    })
                })
                .collect(),
            finished_count: current.finished_battles.len() as u32,
        });
        if next.battle_undo.len() > 20 {
            next.battle_undo.remove(0);
        }
    }
}
pub fn transition(
    c: &Campaign,
    actor: &str,
    a: &Action,
    dice: Vec<u8>,
    cat: &Catalog,
) -> Result<(Campaign, String)> {
    if let Action::AbilityTransfer { command, reason } = a {
        check_count(&dice, 0)?;
        let mut next = c.clone();
        let log = transfers::command(&mut next, actor, command, reason, cat)?;
        if next.battle.as_ref().is_some_and(|b| b.active) {
            ability_changes::lifecycle(c, &mut next, cat)?;
            field::lifecycle(c, &mut next, cat)?;
            abilities::lifecycle(c, &mut next)?;
            extra::lifecycle(c, &mut next, cat)?;
            refresh_initiative(c, &mut next)?;
        }
        if !next.battle.as_ref().is_some_and(|b| b.active) {
            next.battle_undo.clear();
        }
        checkpoint(c, &mut next, &log);
        return Ok((next, log));
    }
    tactics::phase(c, a)?;
    abilities::phase(c, a)?;
    field::phase(c, a)?;
    let mut next = c.clone();
    if let Action::SwitchPokemon {
        outgoing_id,
        gm_override,
        ..
    } = a
    {
        if let Ok(p) = fighter(c, outgoing_id) {
            if !*gm_override
                && (if extra::enabled(c) {
                    extra::blocked(c, p, cat) || special::blocked(c, &p.id)
                } else {
                    field::has(c, "mud", p) && field::grounded(c, p, cat)
                })
            {
                return Err(error(
                    "Muddy terrain blocks grounded Pokemon from switching",
                ));
            }
        }
    }
    if let Some(description) = targeting_action(c, &mut next, actor, a, &dice, cat)? {
        if ability_changes::enabled(&next) {
            ability_changes::lifecycle(c, &mut next, cat)?;
            field::lifecycle(c, &mut next, cat)?;
            abilities::lifecycle(c, &mut next)?;
            extra::lifecycle(c, &mut next, cat)?;
            refresh_initiative(c, &mut next)?;
            sync_labels(&mut next);
        }
        checkpoint(c, &mut next, &description);
        return Ok((next, description));
    }
    let description = match a {
        Action::Tactics { command, reason } => {
            tactics::command(&mut next, actor, command, &dice, reason, cat)?
        }
        Action::React {
            combatant_id,
            kind,
            move_name,
            reason,
        } if tactics::enabled(c) && kind == "move" => tactics::react(
            &mut next,
            actor,
            combatant_id,
            move_name,
            None,
            &dice,
            reason,
            cat,
        )?,
        Action::SpecialMove { command, reason } => {
            special::command(&mut next, actor, command, &dice, reason, cat)?
        }
        Action::BattleStat {
            source_id,
            combatant_id,
            stat,
            change,
            reason: r,
        } => {
            gm(c, actor)?;
            reason(r)?;
            if !extra::enabled(c) {
                return Err(error("Upgrade ability rules between battles"));
            }
            if !flow(c)?.queue.is_empty() {
                return Err(error("Resolve the pending attack before a stat ruling"));
            }
            extra::change_stat(&mut next, source_id, combatant_id, stat, *change)?;
            format!("Sourced stat change {source_id} -> {combatant_id}, {stat} {change:+}: {r}")
        }
        Action::Battlefield { command, reason } => {
            field::command(&mut next, actor, command, &dice, reason, cat)?
        }
        Action::ResolveAbility { .. } => abilities::resolve(c, &mut next, actor, a, &dice, cat)?,
        Action::InflictStatus {
            combatant_id,
            source_id,
            kind,
            detail,
            rounds,
            reason: r,
        } => {
            gm(c, actor)?;
            reason(r)?;
            if !abilities::enabled(c) {
                return Err(error("Upgrade the engine to track ability status sources"));
            }
            abilities::inflict(
                &mut next,
                source_id,
                combatant_id,
                kind,
                detail,
                *rounds,
                cat,
            )?;
            format!("GM recorded {kind} from {source_id} to {combatant_id}: {r}")
        }
        Action::SetBattleRange { .. }
        | Action::CoverAlly { .. }
        | Action::AdjudicateField { .. }
        | Action::RemoveFieldEffect { .. } => unreachable!(),
        Action::BreakIce {
            combatant_id,
            target_id,
            move_name,
            reason: r,
        } => {
            owner(c, actor, combatant_id)?;
            reason(r)?;
            let base = ice_attack(c, combatant_id, target_id, move_name, cat)?;
            check_count(&dice, dice_count(c, a, cat)?)?;
            let p = fighter(c, combatant_id)?;
            let m = move_entry(cat, move_name)?;
            let net = (action_success(&mut next, combatant_id, &dice, base)
                + m["Attributes"]["AccuracyReduction"]
                    .as_i64()
                    .unwrap_or(0)
                    .min(0) as i32)
                .max(0)
                + i32::from(p.runtime.fate_pending);
            let required = p.actions + 1;
            let pp = fighter_mut(&mut next, combatant_id)?;
            pp.actions += 1;
            pp.used_moves.push(move_name.clone());
            if net >= required as i32 {
                if rules::type_effect(m["Type"].as_str().unwrap_or(""), "Ice")? > 0 {
                    fighter_mut(&mut next, target_id)?
                        .runtime
                        .statuses
                        .retain(|s| s.kind != "Frozen Solid");
                } else {
                    flow_mut(&mut next).queue.push(QueuedAction {
                        actor_id: combatant_id.clone(),
                        target_id: target_id.clone(),
                        special_index: 0,
                        move_name: move_name.clone(),
                        sequence: c.revision + 1,
                        successes: net,
                        critical: net >= required as i32 + extra::critical_margin(c, p, &m),
                        timing: "main".into(),
                        priority: 0,
                        cancelled: false,
                        damage_done: false,
                        damage: 0,
                        raw_damage_successes: 0,
                        ice: true,
                        targets: vec![],
                        field_scope: None,
                        shared_dice: vec![],
                        damage_modifiers: None,
                        damage_context: None,
                        defense_key: String::new(),
                    });
                    flow_mut(&mut next).window = false;
                }
            } else if confusion(c, p) > 0 {
                fighter_mut(&mut next, combatant_id)?.hp = (p.hp - 1).max(0);
            }
            format!("Ice attack {move_name}: {dice:?}; {net} successes, requires {required}. {r}")
        }
        Action::AdjustWill {
            combatant_id,
            will,
            reason: r,
        } => {
            gm(c, actor)?;
            reason(r)?;
            let p = fighter(c, combatant_id)?;
            let max = rules::score(&rules::effective_sheet(c, p)?, "Will")?;
            if !(0..=max).contains(will) {
                return Err(error(
                    "Will correction must stay within the derived maximum",
                ));
            }
            fighter_mut(&mut next, combatant_id)?.runtime.will = *will;
            format!("GM corrected encounter Will to {will}: {r}")
        }
        Action::AuthorizeWill {
            combatant_id,
            enabled,
            reason: r,
        } => {
            gm(c, actor)?;
            reason(r)?;
            fighter(c, combatant_id)?;
            fighter_mut(&mut next, combatant_id)?.runtime.will_enabled = *enabled;
            format!("GM authorized Will={enabled}: {r}")
        }
        Action::AdjudicateStatus {
            combatant_id,
            damage,
            reason: r,
        } => {
            gm(c, actor)?;
            reason(r)?;
            if !(0..=999).contains(damage) {
                return Err(error("Status damage must be 0-999"));
            }
            let round = battle(c)?.round;
            if battle(c)?.automatic_validation
                && (!flow(c)?.order.is_empty() || !flow(c)?.queue.is_empty())
            {
                return Err(error(
                    "Record severe status outcomes at round end, after all turns and chains",
                ));
            }
            let p = fighter_mut(&mut next, combatant_id)?;
            if p.runtime.adjudicated_round == round {
                return Err(error("Status outcome already recorded this round"));
            }
            if !p
                .runtime
                .statuses
                .iter()
                .any(|s| ["Burn 3", "Badly Poisoned"].contains(&s.kind.as_str()))
            {
                return Err(error("No ambiguous severe status to adjudicate"));
            }
            if abilities::enabled(c)
                && *damage > 0
                && (p.selected_ability == "Magic Guard"
                    || (p.selected_ability == "Immunity"
                        && !p.runtime.statuses.iter().any(|s| s.kind == "Burn 3")))
            {
                return Err(error("Immunity prevents poison damage; use an explicit resource override for another damage source"));
            }
            p.runtime.adjudicated_round = round;
            p.hp = (p.hp - damage).max(0);
            format!("GM resolved ambiguous severe status for round {round}: {damage} damage. {r}")
        }
        Action::TakeHeldTurn {
            combatant_id,
            reason: r,
        } => {
            owner(c, actor, combatant_id)?;
            ready(c, combatant_id)?;
            reason(r)?;
            let f = flow(c)?;
            if !f.started
                || !f.queue.is_empty()
                || f.deferred.is_some()
                || !f.held.contains(combatant_id)
                || f.order.is_empty()
            {
                return Err(error("A held turn can resume between completed action chains while the round is active"));
            }
            let index = f
                .order
                .iter()
                .position(|id| id == combatant_id)
                .ok_or_else(|| error("Held participant is no longer active"))?;
            let deferred = f.order.get(f.cursor).cloned();
            let f = flow_mut(&mut next);
            f.deferred = deferred;
            f.held.remove(combatant_id);
            f.cursor = index;
            f.turn += 1;
            f.reacted.clear();
            format!("Held turn resumed: {r}")
        }
        Action::StartTurns => {
            gm(c, actor)?;
            let b = battle(c)?;
            if flow(c)?.started || !flow(c)?.queue.is_empty() || abilities::pending(c) {
                return Err(error("Turns already started"));
            }
            let mut order = b
                .combatants
                .iter()
                .filter(|p| p.deployed && p.hp > 0 && p.available_round <= b.round)
                .collect::<Vec<_>>();
            if order.iter().any(|p| !p.initiative_rolled) {
                return Err(error("Roll initiative for every active combatant"));
            }
            let base = |p: &Combatant| -> Result<i32> {
                let s = rules::effective_sheet(c, p)?;
                Ok(rules::score(&s, "Dexterity")?
                    + rules::score(&s, "Alert")?
                    + if crate::creation::rank(&s.rank).is_some_and(|r| r >= 6) {
                        3
                    } else {
                        0
                    })
            };
            for p in &order {
                base(p)?;
            }
            order.sort_by_key(|p| (-p.initiative, -base(p).unwrap()));
            if order.windows(2).any(|v| {
                extra::order_group(c, v[0]) == extra::order_group(c, v[1])
                    && v[0].initiative == v[1].initiative
                    && base(v[0]).unwrap() == base(v[1]).unwrap()
            }) {
                return Err(error(
                    "Unresolved initiative tie; GM rerolls the tied combatants",
                ));
            }
            if field::enabled(c)
                && order
                    .first()
                    .is_some_and(|p| field::has(c, "Trick Room", p))
            {
                order.reverse();
            }
            extra::order(c, &mut order);
            let f = flow_mut(&mut next);
            f.order = order.iter().map(|p| p.id.clone()).collect();
            f.started = true;
            f.turn += 1;
            f.cursor = 0;
            field::start(&mut next, cat)?;
            extra::round_start(&mut next)?;
            "Initiative order confirmed; turns started".into()
        }
        Action::PassTurn {
            combatant_id,
            reason: r,
        } => {
            owner(c, actor, combatant_id)?;
            turn(c, combatant_id)?;
            reason(r)?;
            let round = battle(c)?.round;
            let p = fighter_mut(&mut next, combatant_id)?;
            if p.hp == 0 || p.actions >= 5 {
                return Err(error("No active turn available"));
            }
            if status(p, "Flinch") {
                p.actions += 1;
                p.runtime.statuses.retain(|s| s.kind != "Flinch");
            } else if p.runtime.recharge_actions > 0 && p.runtime.recharge_round <= round {
                p.actions += 1;
                p.runtime.recharge_actions -= 1;
            } else {
                if status(p, "Sleep")
                    && battle(c)?.automatic_validation
                    && !(extra::enabled(c) && p.selected_ability == "Comatose")
                {
                    return Err(error(
                        "Check Sleep at the start of this turn; waking consumes an action",
                    ));
                }
                flow_mut(&mut next).held.insert(combatant_id.clone());
            }
            advance(&mut next);
            format!("Held action: {r}")
        }
        Action::Accuracy {
            combatant_id,
            move_name,
            pool_modifier,
            success_modifier,
            reason: r,
            ..
        }
        | Action::AccuracyTargets {
            combatant_id,
            move_name,
            pool_modifier,
            success_modifier,
            reason: r,
            ..
        } => {
            owner(c, actor, combatant_id)?;
            turn(c, combatant_id)?;
            special::ready_move(c, combatant_id, move_name)?;
            reason(r)?;
            if actor != c.gm_id && (*pool_modifier != 0 || *success_modifier != 0) {
                return Err(AppError::new("permission", "Modifiers require GM"));
            }
            if !(-20..=20).contains(success_modifier) {
                return Err(error("Success modifier outside bounds"));
            }
            let p = fighter(c, combatant_id)?;
            let m = field::move_data(c, p, &move_entry(cat, move_name)?);
            if matches!(a, Action::AccuracyTargets { .. }) {
                require_targeting(c)?;
            }
            let (targets, field_scope) = plan(c, combatant_id, &selected(a), &m)?;
            if m["Target"] == "Random Foe" && actor != c.gm_id {
                return Err(error("GM must record the randomly selected foe"));
            }
            break_cover(&mut next, combatant_id);
            let targets = redirect(
                &next,
                combatant_id,
                targets,
                m["Attributes"]["SoundMove"] == true
                    || ["Phantom Force", "Shadow Force"].contains(&move_name.as_str()),
            );
            let targets = tactics::redirect_move(c, combatant_id, targets, &m, cat)?;
            let target = targets
                .first()
                .cloned()
                .unwrap_or_else(|| combatant_id.clone());
            if p.runtime
                .statuses
                .iter()
                .any(|s| s.kind == "Disable" && s.detail == *move_name)
            {
                return Err(error("This move is disabled"));
            }
            let (base, _) = rules::accuracy(&old(c), combatant_id, move_name, *pool_modifier, cat)?;
            check_count(&dice, dice_count(c, a, cat)?)?;
            let low = m["Attributes"]["AccuracyReduction"]
                .as_i64()
                .unwrap_or(0)
                .min(0) as i32
                - abilities::low_accuracy(c, p, &m)
                - field::low_accuracy(c, p, &m);
            let low = low - special::low_accuracy(c, combatant_id);
            let shield_penalty = shield_attempt(&mut next, combatant_id, &m)?;
            let pain_exempt = pain_exempt(c, p, &m)?;
            let saved_pain = fighter_mut(&mut next, combatant_id)?.runtime.pain_ignored;
            if pain_exempt {
                fighter_mut(&mut next, combatant_id)?.runtime.pain_ignored = 3;
            }
            let net =
                (action_success(&mut next, combatant_id, &dice, base) + low + success_modifier
                    - shield_penalty)
                    .max(0)
                    + i32::from(p.runtime.fate_pending);
            fighter_mut(&mut next, combatant_id)?.runtime.pain_ignored = saved_pain;
            let required = p.actions + 1;
            let net = if move_name == "Sky Uppercut" && special::vulnerable(c, &target, move_name) {
                net.max(required as i32)
            } else {
                net
            };
            extra::accuracy_attempt(&mut next, combatant_id, &m, cat)?;
            fighter_mut(&mut next, combatant_id)?.actions += 1;
            fighter_mut(&mut next, combatant_id)?
                .used_moves
                .push(move_name.clone());
            if net >= required as i32 {
                let critical = m["Category"] != "Support"
                    && net >= required as i32 + extra::critical_margin(c, p, &m);
                flow_mut(&mut next).queue.push(QueuedAction {
                    actor_id: combatant_id.clone(),
                    target_id: target.clone(),
                    special_index: 0,
                    move_name: move_name.clone(),
                    sequence: c.revision + 1,
                    successes: net,
                    critical,
                    timing: "main".into(),
                    priority: 0,
                    cancelled: false,
                    damage_done: false,
                    damage: 0,
                    raw_damage_successes: 0,
                    ice: false,
                    targets: vec![],
                    field_scope: None,
                    shared_dice: vec![],
                    damage_modifiers: None,
                    damage_context: None,
                    defense_key: String::new(),
                });
                let q = flow_mut(&mut next).queue.last_mut().unwrap();
                q.field_scope = field_scope;
                q.defense_key = if tactics::enabled(c) && m["Category"] == "Support" {
                    "Support"
                } else if m["Category"] == "Physical" {
                    "Defense"
                } else {
                    "SpecialDefense"
                }
                .into();
                if targets.len() > 1 || (tactics::enabled(c) && !targets.is_empty()) {
                    q.targets = targets
                        .into_iter()
                        .map(|target_id| TargetImpact {
                            target_id,
                            responded: false,
                            avoided: false,
                            resolved: false,
                        })
                        .collect();
                }
                flow_mut(&mut next).window =
                    target != *combatant_id && special::registered(c, &target)?.hp > 0;
                if flow(&next)?.queue[0].field_scope.is_some() {
                    flow_mut(&mut next).window = false;
                }
                prepare_targets(&mut next);
            } else {
                extra::consume_charge(&mut next, combatant_id, &m);
                if confusion(c, p) > 0 {
                    fighter_mut(&mut next, combatant_id)?.hp = (p.hp - 1).max(0);
                }
                advance(&mut next);
            }
            special::after_accuracy(&mut next, combatant_id, &m, net >= required as i32, cat)?;
            format!("{move_name}: host dice {dice:?}, {net} successes, requires {required}. {r}")
        }
        Action::React {
            combatant_id,
            kind,
            move_name,
            reason: r,
        } => {
            owner(c, actor, combatant_id)?;
            reason(r)?;
            let base = reaction_pool(c, combatant_id, kind, move_name, cat)?;
            check_count(&dice, dice_count(c, a, cat)?)?;
            let incoming = incoming(c)?.clone();
            let p = fighter(c, combatant_id)?;
            let low = if kind == "move" {
                let resolved_move = field::move_data(c, p, &move_entry(cat, move_name)?);
                resolved_move["Attributes"]["AccuracyReduction"]
                    .as_i64()
                    .unwrap_or(0)
                    .min(0) as i32
                    - abilities::low_accuracy(c, p, &resolved_move)
                    - field::low_accuracy(c, p, &resolved_move)
            } else {
                0
            };
            let shield_penalty = if kind == "move" {
                shield_attempt(&mut next, combatant_id, &move_entry(cat, move_name)?)?
            } else {
                0
            };
            let saved_pain = fighter_mut(&mut next, combatant_id)?.runtime.pain_ignored;
            if kind == "move" && pain_exempt(c, p, &move_entry(cat, move_name)?)? {
                fighter_mut(&mut next, combatant_id)?.runtime.pain_ignored = 3;
            }
            let net = (action_success(&mut next, combatant_id, &dice, base) + low - shield_penalty)
                .max(0)
                + i32::from(p.runtime.fate_pending);
            fighter_mut(&mut next, combatant_id)?.runtime.pain_ignored = saved_pain;
            fighter_mut(&mut next, combatant_id)?.actions += 1;
            fighter_mut(&mut next, combatant_id)?.reactions += 1;
            if kind != "clash" {
                flow_mut(&mut next).reacted.insert(combatant_id.clone());
            }
            responded(&mut next);
            break_cover(&mut next, combatant_id);
            if kind == "evade" || kind == "clash" {
                if kind == "clash" {
                    fighter_mut(&mut next, combatant_id)?
                        .used_moves
                        .push(move_name.clone());
                }
                if net >= incoming.successes {
                    avoid(
                        &mut next,
                        kind == "clash"
                            && move_entry(cat, &incoming.move_name)?["Target"] == "Area",
                    );
                    if kind == "clash" {
                        let attack = move_entry(cat, &incoming.move_name)?;
                        let defend = move_entry(cat, move_name)?;
                        let mut to_def = clash_damage(
                            c,
                            &incoming.target_id,
                            attack["Type"].as_str().unwrap_or(""),
                            cat,
                        )?;
                        let mut to_att = clash_damage(
                            c,
                            &incoming.actor_id,
                            defend["Type"].as_str().unwrap_or(""),
                            cat,
                        )?;
                        if abilities::enabled(c) {
                            to_def = abilities::clash_amount(
                                &mut next,
                                &incoming.target_id,
                                &attack,
                                to_def,
                                &incoming.actor_id,
                            )?;
                            if tactics::enabled(c) && incoming.targets.len() > 1 {
                                tactics::queue_clash(&mut next, &incoming, move_name, to_att, cat)?;
                                to_att = 0;
                            } else {
                                to_att = abilities::clash_amount(
                                    &mut next,
                                    &incoming.actor_id,
                                    &defend,
                                    to_att,
                                    &incoming.target_id,
                                )?;
                            }
                        }
                        let attacker_hp = fighter(c, &incoming.actor_id)?.hp;
                        let counter = QueuedAction {
                            actor_id: incoming.target_id.clone(),
                            target_id: incoming.actor_id.clone(),
                            move_name: move_name.clone(),
                            ..incoming.clone()
                        };
                        let deferred_counter = tactics::enabled(c) && incoming.targets.len() > 1;
                        let def_decoy = decoys::damage(&mut next, &incoming, &attack, to_def);
                        let att_decoy = !deferred_counter
                            && decoys::damage(&mut next, &counter, &defend, to_att);
                        // Apply simultaneous damage before either hit callback; never overwrite retaliation HP.
                        if !def_decoy {
                            fighter_mut(&mut next, &incoming.target_id)?.hp =
                                (p.hp - to_def).max(0);
                        }
                        if !deferred_counter && !att_decoy {
                            fighter_mut(&mut next, &incoming.actor_id)?.hp =
                                (attacker_hp - to_att).max(0);
                        }
                        if !def_decoy {
                            abilities::hit(&mut next, &incoming, p.hp, &attack, cat)?;
                        }
                        if !deferred_counter && !att_decoy {
                            abilities::hit(&mut next, &counter, attacker_hp, &defend, cat)?;
                        }
                    }
                }
                close(&mut next);
            } else {
                let m = field::move_data(c, p, &move_entry(cat, move_name)?);
                let (timing, priority) = if let Some(n) = m["Attributes"]["Reaction"]
                    .as_str()
                    .and_then(|n| n.parse::<i32>().ok())
                {
                    ("before", n)
                } else {
                    (
                        "after",
                        m["Attributes"]["LateReaction"]
                            .as_str()
                            .and_then(|n| n.parse::<i32>().ok())
                            .ok_or_else(|| error("Reaction priority unresolved"))?,
                    )
                };
                let required = p.actions + 1;
                fighter_mut(&mut next, combatant_id)?
                    .used_moves
                    .push(move_name.clone());
                extra::accuracy_attempt(&mut next, combatant_id, &m, cat)?;
                if net >= required as i32 {
                    flow_mut(&mut next).queue.push(QueuedAction {
                        actor_id: combatant_id.clone(),
                        target_id: if m["Target"] == "Self"
                            || (special::enabled(c) && special::ally_shield(&m))
                        {
                            combatant_id.clone()
                        } else {
                            incoming.actor_id.clone()
                        },
                        special_index: 0,
                        move_name: move_name.clone(),
                        sequence: c.revision + 1,
                        successes: net,
                        critical: m["Category"] != "Support"
                            && net >= required as i32 + extra::critical_margin(c, p, &m),
                        timing: timing.into(),
                        priority,
                        cancelled: false,
                        damage_done: false,
                        damage: 0,
                        raw_damage_successes: 0,
                        ice: false,
                        targets: vec![],
                        field_scope: None,
                        shared_dice: vec![],
                        damage_modifiers: None,
                        damage_context: None,
                        defense_key: String::new(),
                    });
                    if special::enabled(c) && special::ally_shield(&m) {
                        let targets = special::allies(c, combatant_id)?;
                        flow_mut(&mut next).queue.last_mut().unwrap().targets = targets
                            .into_iter()
                            .map(|target_id| TargetImpact {
                                target_id,
                                responded: true,
                                avoided: false,
                                resolved: false,
                            })
                            .collect();
                    }
                    let can_clash = m["Target"] != "Self"
                        && c.sheets[&fighter(c, &incoming.actor_id)?.sheet_id]
                            .moves
                            .iter()
                            .any(|name| {
                                reaction_pool(&next, &incoming.actor_id, "clash", name, cat).is_ok()
                            });
                    if m["Target"] == "Self"
                        || (special::enabled(c) && special::ally_shield(&m))
                        || (flow(&next)?.reacted.contains(&incoming.actor_id) && !can_clash)
                    {
                        close(&mut next);
                    }
                } else {
                    if confusion(c, p) > 0 {
                        fighter_mut(&mut next, combatant_id)?.hp = (p.hp - 1).max(0);
                    }
                    close(&mut next);
                }
            }
            special::after_reaction(&mut next, combatant_id, kind, cat)?;
            format!(
                "{kind} {move_name}: {dice:?}, {net} successes against {}. {r}",
                incoming.successes
            )
        }
        Action::PassReaction {
            combatant_id,
            reason: r,
        } => {
            owner(c, actor, combatant_id)?;
            reason(r)?;
            let f = flow(c)?;
            if !f.window || incoming(c).map_or(true, |q| q.target_id != *combatant_id) {
                return Err(error("Only the current targeted combatant can pass"));
            }
            responded(&mut next);
            close(&mut next);
            format!("Reaction passed: {r}")
        }
        Action::Damage {
            combatant_id,
            sequence,
            ..
        } => {
            owner(c, actor, combatant_id)?;
            let mut damage_campaign = c.clone();
            if abilities::enabled(c)
                && !current(c)?.targets.is_empty()
                && current(c)?.damage_context.is_none()
            {
                let p = fighter(c, combatant_id)?;
                let context = abilities::DamageContext {
                    special_bonus: if special::enabled(c) {
                        battle(c)?
                            .combatants
                            .iter()
                            .filter(|t| t.deployed && t.hp > 0)
                            .map(|t| {
                                Ok((
                                    t.id.clone(),
                                    special::bonus(
                                        c,
                                        p,
                                        t,
                                        &field::move_data(
                                            c,
                                            p,
                                            &move_entry(cat, &current(c)?.move_name)?,
                                        ),
                                        cat,
                                    )?,
                                ))
                            })
                            .collect::<Result<BTreeMap<_, _>>>()?
                    } else {
                        BTreeMap::new()
                    },
                    ability_sources: if extra::enabled(c) {
                        battle(c)?.combatants.clone()
                    } else {
                        vec![]
                    },
                    field_snapshot: if field::enabled(c) {
                        Some(field::snapshot(c, cat)?)
                    } else {
                        None
                    },
                    stats: rules::effective_sheet(c, p)?.stats,
                    hp: p.hp,
                    pain_ignored: p.runtime.pain_ignored,
                    counters: p.runtime.ability.counters.clone(),
                    statuses: p.runtime.statuses.clone(),
                    love_resisted_turn: p.runtime.love_resisted_turn,
                    types: field::types(c, p, cat)?,
                    field_bonus: field::bonus(c, p, &move_entry(cat, &current(c)?.move_name)?),
                };
                flow_mut(&mut damage_campaign).queue[0].damage_context = Some(context);
            }
            let c = &damage_campaign;
            let q = current(c)?.clone();
            if q.actor_id != *combatant_id || q.sequence != *sequence || q.damage_done {
                return Err(error("Resolve the first pending action"));
            }
            if q.ice {
                if let Action::Damage {
                    pool_modifier,
                    success_modifier,
                    damage_modifier,
                    reason: r,
                    ..
                } = a
                {
                    reason(r)?;
                    if !(-20..=20).contains(pool_modifier)
                        || !(-20..=20).contains(success_modifier)
                        || !(-20..=20).contains(damage_modifier)
                    {
                        return Err(error("Modifiers outside -20 to 20"));
                    }
                    if actor != c.gm_id
                        && (*pool_modifier != 0 || *success_modifier != 0 || *damage_modifier != 0)
                    {
                        return Err(AppError::new("permission", "Modifiers require GM"));
                    }
                    let n = ice_damage_pool(c, &q, *pool_modifier, cat)?;
                    check_count(&dice, n)?;
                    let raw = rules::successes(&dice);
                    let m = move_entry(cat, &q.move_name)?;
                    let p = fighter(c, &q.actor_id)?;
                    let pain = if m["Damage1"] == "Vitality"
                        || type_ability(c, p, m["Type"].as_str().unwrap_or(""))
                    {
                        0
                    } else {
                        (rules::pain(p.hp, p.max_hp) - p.runtime.pain_ignored).max(0)
                    };
                    let amount = ((raw - pain + success_modifier).max(1) + damage_modifier).max(0);
                    let t = fighter_mut(&mut next, &q.target_id)?;
                    let ice = t
                        .runtime
                        .statuses
                        .iter_mut()
                        .find(|s| s.kind == "Frozen Solid")
                        .ok_or_else(|| error("Ice no longer present"))?;
                    ice.progress += amount;
                    if ice.progress
                        >= if field::weather(c, Some(fighter(c, &q.target_id)?))
                            == Some(field::Weather::Snow)
                        {
                            7
                        } else {
                            5
                        }
                    {
                        t.runtime.statuses.retain(|s| s.kind != "Frozen Solid");
                    }
                    let pending = flow_mut(&mut next).queue.first_mut().unwrap();
                    pending.damage_done = true;
                    pending.damage = amount;
                    pending.raw_damage_successes = raw as u32;
                    if !["Submission", "Hyper Beam"].contains(&q.move_name.as_str()) {
                        finish(&mut next)
                    }
                    sync_labels(&mut next);
                    checkpoint(c, &mut next, "Ice block damage");
                    return Ok((
                        next,
                        format!("Ice block took {amount} damage; Pokemon HP unchanged. {r}"),
                    ));
                }
            }
            if special::enabled(c) && special::formula_name(&q.move_name) {
                let log = special::resolve_formula(&mut next, actor, &q, a, &dice, cat)?;
                ability_changes::lifecycle(c, &mut next, cat)?;
                tactics::lifecycle(c, &mut next, cat)?;
                field::lifecycle(c, &mut next, cat)?;
                abilities::lifecycle(c, &mut next)?;
                extra::lifecycle(c, &mut next, cat)?;
                special::lifecycle(c, &mut next, cat)?;
                decoys::lifecycle(&mut next);
                sync_labels(&mut next);
                checkpoint(c, &mut next, &log);
                return Ok((next, log));
            }
            if q.move_name == "Sonic Boom" {
                if let Action::Damage {
                    pool_modifier,
                    success_modifier,
                    damage_modifier,
                    reason: r,
                    ..
                } = a
                {
                    reason(r)?;
                    check_count(&dice, 0)?;
                    if actor != c.gm_id
                        && (*pool_modifier != 0 || *success_modifier != 0 || *damage_modifier != 0)
                    {
                        return Err(AppError::new("permission", "Damage modifiers require GM"));
                    }
                    if *pool_modifier != 0
                        || *success_modifier != 0
                        || !(-20..=20).contains(damage_modifier)
                    {
                        return Err(error("Set damage has no dice/success modifiers; final adjustment must be -20 to 20"));
                    }
                    let amount = (1 + damage_modifier).max(0);
                    let m = move_entry(cat, &q.move_name)?;
                    let decoy_hit = decoys::damage(&mut next, &q, &m, amount);
                    if !decoy_hit {
                        let p = fighter_mut(&mut next, &q.target_id)?;
                        let before_hp = p.hp;
                        p.hp = (p.hp - amount).max(0);
                        abilities::hit(&mut next, &q, before_hp, &m, cat)?;
                    }
                    finish(&mut next);
                    ability_changes::lifecycle(c, &mut next, cat)?;
                    tactics::lifecycle(c, &mut next, cat)?;
                    field::lifecycle(c, &mut next, cat)?;
                    abilities::lifecycle(c, &mut next)?;
                    extra::lifecycle(c, &mut next, cat)?;
                    special::lifecycle(c, &mut next, cat)?;
                    decoys::lifecycle(&mut next);
                    sync_labels(&mut next);
                    refresh_initiative(c, &mut next)?;
                    checkpoint(c, &mut next, "Sonic Boom set damage");
                    return Ok((next,format!("Sonic Boom: {amount} typeless set damage; defenses/shields bypassed. {r}")));
                }
            }
            let dice = if q.targets.is_empty() {
                dice
            } else {
                let Action::Damage {
                    pool_modifier,
                    success_modifier,
                    damage_modifier,
                    ..
                } = a
                else {
                    unreachable!()
                };
                check_count(&dice, dice_count(c, a, cat)?)?;
                let total = damage_dice(c, &q, *pool_modifier, cat)? as usize;
                let mut shared = q.shared_dice.clone();
                shared.extend_from_slice(&dice);
                let pending = flow_mut(&mut next).queue.first_mut().unwrap();
                pending.shared_dice = shared.clone();
                pending.damage_modifiers =
                    Some([*pool_modifier, *success_modifier, *damage_modifier]);
                shared[..total].to_vec()
            };
            let rolled_damage_dice = dice.len() as u32;
            let raw_damage_successes = rules::successes(&dice) as u32;
            let mut temp = old(c);
            if !q.targets.is_empty() {
                let updated = flow(&next)?.queue.first().unwrap();
                let pending = flow_mut(&mut temp).queue.first_mut().unwrap();
                pending.shared_dice = updated.shared_dice.clone();
                pending.damage_modifiers = updated.damage_modifiers;
            }
            install_pending(&mut temp, &q);
            let (mut done, log) = crate::battle::transition(&temp, actor, a, dice, cat)?;
            done.engine = c.engine.clone();
            done.battle_undo = c.battle_undo.clone();
            let before = fighter(c, &q.target_id)?.hp;
            let after = fighter(&done, &q.target_id)?.hp;
            let resolved_move =
                field::move_data(c, fighter(c, &q.actor_id)?, &move_entry(cat, &q.move_name)?);
            let shield = if special::enabled(c) {
                special::shield_reduction(c, &q, &resolved_move)
            } else if extra::enabled(c)
                && fighter(c, &q.actor_id)?.selected_ability == "Infiltrator"
                && fighter(c, &q.actor_id)?.side != fighter(c, &q.target_id)?.side
            {
                0
            } else {
                fighter(c, &q.target_id)?.runtime.shield
            };
            let target_snapshot = ability_changes::frozen_subject(
                c,
                &q.actor_id,
                q.sequence,
                fighter(c, &q.target_id)?,
            );
            let target = &target_snapshot;
            let source_snapshot = ability_changes::frozen_subject(
                c,
                &q.actor_id,
                q.sequence,
                fighter(c, &q.actor_id)?,
            );
            let mut thick_fat = i32::from(
                target.selected_ability == "Thick Fat"
                    && ["Fire", "Ice"].contains(
                        &move_entry(cat, &q.move_name)?["Type"]
                            .as_str()
                            .unwrap_or(""),
                    ),
            );
            let move_data = field::move_data(c, &source_snapshot, &move_entry(cat, &q.move_name)?);
            if abilities::enabled(c) {
                let mut pool_campaign = old(c);
                install_pending(&mut pool_campaign, &q);
                let Action::Damage { pool_modifier, .. } = a else {
                    unreachable!()
                };
                let pool = rules::damage_pool(
                    &pool_campaign,
                    &q.actor_id,
                    q.sequence,
                    *pool_modifier,
                    cat,
                )?;
                thick_fat = abilities::reduction(
                    c,
                    &decoys::damage_subject(c, &q.actor_id, target, &move_data),
                    &move_data,
                    pool.effectiveness,
                    &q.actor_id,
                );
            }
            let field_reduction = field::reduction(c, fighter(c, &q.actor_id)?, target, &move_data);
            let raw_amount = if field::enabled(c) {
                let Action::Damage {
                    pool_modifier,
                    success_modifier,
                    damage_modifier,
                    ..
                } = a
                else {
                    unreachable!()
                };
                let pool = rules::damage_pool(&temp, &q.actor_id, q.sequence, *pool_modifier, cat)?;
                let net = (raw_damage_successes as i32 - pool.pain + success_modifier).max(0);
                if pool.immune {
                    0
                } else {
                    (net.max(1)
                        + if pool.effectiveness > 0 && net == 0 {
                            0
                        } else {
                            pool.effectiveness
                        }
                        + damage_modifier)
                        .max(0)
                }
            } else {
                before - after
            };
            let mut damage = (raw_amount - shield - thick_fat - field_reduction).max(0);
            if field::enabled(c) {
                let Action::Damage { pool_modifier, .. } = a else {
                    unreachable!()
                };
                if rules::damage_pool(&temp, &q.actor_id, q.sequence, *pool_modifier, cat)?.immune {
                    damage = 0;
                }
            }
            if holding_back(c, &q) {
                damage /= 2;
            }
            damage = special::before_damage(&mut done, &q, &move_data, damage, cat)?;
            let decoy_hit = decoys::damage(&mut done, &q, &move_data, damage);
            if !decoy_hit && !abilities::damage_immune(c, target, &move_data) {
                damage = abilities::sturdy(&mut done, &q.target_id, damage, before)?;
            }
            fighter_mut(&mut done, &q.target_id)?.hp = if decoy_hit {
                before
            } else {
                (before - damage).max(0)
            };
            fighter_mut(&mut done, &q.target_id)?.runtime.shield = 0;
            let qa = flow_mut(&mut done).queue.first_mut().unwrap();
            qa.damage_done = true;
            qa.damage = if decoy_hit { 0 } else { damage };
            qa.raw_damage_successes = raw_damage_successes;
            if !decoy_hit {
                abilities::hit(&mut done, &q, before, &move_data, cat)?;
            }
            let actual_damage = (before - fighter(&done, &q.target_id)?.hp).max(0);
            special::hit(
                &mut done,
                &q,
                &move_data,
                rolled_damage_dice,
                actual_damage,
                cat,
            )?;
            extra::consume_charge(
                &mut done,
                &q.actor_id,
                &field::move_data(c, fighter(c, &q.actor_id)?, &move_data),
            );
            next = done;
            if no_effect(&q.move_name, cat)?
                || ((!special::enabled(c) && (shield > 0 || holding_back(c, &q)))
                    && !(["Submission", "Hyper Beam"].contains(&q.move_name.as_str())))
            {
                finish(&mut next);
            }
            format!(
                "{log} Shield reduction {shield}; final damage {damage}. {}",
                if decoy_hit {
                    "Decoy intercepted this hit; Pokemon HP unchanged and target added effects blocked."
                } else {
                    "Additional effects require their reviewed resolution."
                }
            )
        }
        Action::ResolveEffect {
            sequence,
            reason: r,
        }
        | Action::AdjudicateEffect {
            sequence,
            reason: r,
        } => {
            reason(r)?;
            let q = current(c)?.clone();
            if q.sequence != *sequence {
                return Err(error("Pending action changed"));
            }
            if q.field_scope.is_some() && matches!(a, Action::AdjudicateEffect { .. }) {
                return Err(error(
                    "Use the field-effect ruling to preserve its scope and duration",
                ));
            }
            if matches!(a, Action::AdjudicateEffect { .. }) {
                gm(c, actor)?;
            } else {
                owner(c, actor, &q.actor_id)?;
                let m =
                    field::move_data(c, fighter(c, &q.actor_id)?, &move_entry(cat, &q.move_name)?);
                if m["Category"] != "Support" && !q.damage_done {
                    return Err(error("Resolve damage first"));
                }
                check_count(&dice, effect_dice(c, &q, cat)?)?;
                effects(&mut next, &q, &dice, cat)?;
            }
            finish(&mut next);
            format!("Effects for {} resolved: {r}", q.move_name)
        }
        Action::SetStatus {
            combatant_id,
            kind,
            detail,
            rounds,
            remove,
            reason: r,
        } => {
            gm(c, actor)?;
            reason(r)?;
            fighter(c, combatant_id)?;
            if *remove {
                fighter_mut(&mut next, combatant_id)?
                    .runtime
                    .statuses
                    .retain(|s| s.kind != *kind);
            } else {
                set_status(&mut next, combatant_id, kind, detail, *rounds, cat)?;
            }
            format!("GM status {kind}, remove={remove}: {r}")
        }
        Action::SpendWill {
            combatant_id,
            mode,
            reason: r,
        } => {
            owner(c, actor, combatant_id)?;
            reason(r)?;
            let b = battle(c)?;
            let p = fighter(c, combatant_id)?;
            if p.hp == 0 || p.runtime.will <= 0 {
                return Err(error("No Will available or fainted"));
            }
            if !p.runtime.will_enabled && actor != c.gm_id {
                return Err(AppError::new(
                    "approval_required",
                    "GM must authorize the narrative Will prerequisite for this encounter",
                ));
            }
            let rt = &mut fighter_mut(&mut next, combatant_id)?.runtime;
            if actor == c.gm_id {
                rt.will_enabled = true;
            }
            match mode.as_str() {
                "pain" if rt.pain_ignored < 3 && rules::pain(p.hp, p.max_hp) > rt.pain_ignored => {
                    rt.pain_ignored += 1
                }
                "chance" if rt.fate_round != b.round && rt.chance_round != b.round => {
                    rt.chance_round = b.round
                }
                "fate" if rt.chance_round != b.round && !rt.fate_pending => {
                    rt.fate_round = b.round;
                    rt.fate_pending = true
                }
                _ => {
                    return Err(error(
                        "Will mode unavailable, repeated, or conflicts with the other round effect",
                    ))
                }
            }
            rt.will -= 1;
            format!("Spent one Will: {mode}. {r}")
        }
        Action::StatusCheck {
            combatant_id,
            kind,
            reason: r,
        } => {
            owner(c, actor, combatant_id)?;
            reason(r)?;
            let p = fighter(c, combatant_id)?;
            let b = battle(c)?;
            check_count(&dice, dice_count(c, a, cat)?)?;
            if kind != "confusion" {
                turn(c, combatant_id)?;
                if status(p, "Flinch")
                    || (p.runtime.recharge_actions > 0 && p.runtime.recharge_round <= b.round)
                    || (status(p, "Frozen Solid") && kind != "sleep")
                    || (status(p, "Sleep") && kind != "sleep")
                {
                    return Err(error("Resolve blocking condition before this check"));
                }
                if p.actions >= 5 || p.hp == 0 {
                    return Err(error("No actions available"));
                }
            }
            if p.hp == 0 {
                return Err(error("A fainted Pokemon cannot check conditions"));
            }
            let net = if ["burn", "sleep"].contains(&kind.as_str()) {
                let base = dice_count(c, a, cat)? - u32::from(p.runtime.chance_round == b.round);
                action_success(&mut next, combatant_id, &dice, base)
                    + i32::from(p.runtime.fate_pending)
            } else {
                (rules::successes(&dice) - confusion(c, p)).max(0)
            };
            match kind.as_str() {
                "confusion" if status(p, "Confusion") && p.runtime.confusion_checked != b.round => {
                    let rt = &mut fighter_mut(&mut next, combatant_id)?.runtime;
                    rt.confusion_checked = b.round;
                    if net >= 2 {
                        rt.confusion_resisted = b.round;
                    }
                }
                "sleep" if status(p, "Sleep") => {
                    let p = fighter_mut(&mut next, combatant_id)?;
                    let s = p
                        .runtime
                        .statuses
                        .iter_mut()
                        .find(|s| s.kind == "Sleep")
                        .unwrap();
                    s.progress += net;
                    if s.progress >= 5 {
                        p.runtime.statuses.retain(|s| s.kind != "Sleep");
                    }
                    p.actions += 1;
                    advance(&mut next);
                }
                "burn"
                    if p.runtime
                        .statuses
                        .iter()
                        .any(|s| s.kind.starts_with("Burn")) =>
                {
                    let p = fighter_mut(&mut next, combatant_id)?;
                    for s in &mut p.runtime.statuses {
                        if s.kind.starts_with("Burn") {
                            s.progress += net;
                        }
                    }
                    p.runtime.statuses.retain(|s| {
                        !s.kind.starts_with("Burn")
                            || s.progress
                                < match s.kind.as_str() {
                                    "Burn" => 4,
                                    "Burn 2" => 6,
                                    _ => 8,
                                }
                    });
                    p.actions += 1;
                    advance(&mut next);
                }
                "love" if status(p, "In Love") && p.runtime.love_checked_turn != flow(c)?.turn => {
                    fighter_mut(&mut next, combatant_id)?
                        .runtime
                        .love_checked_turn = flow(c)?.turn;
                    if net >= 3 {
                        fighter_mut(&mut next, combatant_id)?
                            .runtime
                            .love_resisted_turn = flow(c)?.turn;
                    }
                }
                _ => return Err(error("Status check unavailable or already performed")),
            }
            format!("{kind} check: {dice:?}, {net} successes. {r}")
        }
        Action::NextRound => {
            gm(c, actor)?;
            if abilities::pending(c)
                || !flow(c)?.queue.is_empty()
                || (battle(c)?.automatic_validation && !flow(c)?.order.is_empty())
            {
                return Err(error(
                    "Resolve all pending actions and turns before advancing the round",
                ));
            }
            let mut temp = old(c);
            let already_ended =
                field::enabled(c) && flow(c)?.battlefield.end_round == battle(c)?.round;
            if !already_ended {
                end_round(&mut temp, cat)?;
                abilities::end_round(&mut temp)?;
                extra::end_round(&mut temp)?;
                field::end(&mut temp, cat)?;
                special::end_round(&mut temp)?;
            }
            if field::pending(&temp) || abilities::pending(&temp) || special::ticks_pending(&temp) {
                temp.engine = c.engine.clone();
                ability_changes::lifecycle(c, &mut temp, cat)?;
                tactics::lifecycle(c, &mut temp, cat)?;
                field::lifecycle(c, &mut temp, cat)?;
                abilities::lifecycle(c, &mut temp)?;
                extra::lifecycle(c, &mut temp, cat)?;
                special::lifecycle(c, &mut temp, cat)?;
                sync_labels(&mut temp);
                checkpoint(c, &mut temp, "Round-end battlefield checks pending");
                return Ok((
                    temp,
                    "Resolve round-end battlefield checks before expiry and round advancement"
                        .into(),
                ));
            }
            field::expire(&mut temp)?;
            special::expire(&mut temp)?;
            let (mut done, log) = crate::battle::transition(&temp, actor, a, dice, cat)?;
            done.engine = c.engine.clone();
            done.battle_undo = c.battle_undo.clone();
            let ids = done
                .battle
                .as_ref()
                .unwrap()
                .combatants
                .iter()
                .filter(|p| p.deployed && p.hp > 0)
                .map(|p| p.id.clone())
                .collect::<Vec<_>>();
            let f = flow_mut(&mut done);
            f.started = false;
            f.deferred = None;
            f.order.clear();
            f.held.clear();
            f.reacted.clear();
            for effect in &mut f.field_effects {
                if let Some(rounds) = effect.rounds.as_mut() {
                    *rounds = rounds.saturating_sub(1);
                }
            }
            f.field_effects.retain(|effect| effect.rounds != Some(0));
            for p in &mut done.battle.as_mut().unwrap().combatants {
                p.runtime.healed = 0;
                p.runtime.fate_pending = false;
                p.runtime.shield = 0;
            }
            next = done;
            format!("{log}; status/end-of-round effects applied; confirm initiative order for {} active combatants",ids.len())
        }
        _ => {
            if battle(c).is_ok_and(|b| b.automatic_validation)
                && matches!(a, Action::Roll { .. } | Action::Resolve { .. })
            {
                return Err(error("Use structured actions while automatic validation is enabled; GM may disable it with a reason"));
            }
            if matches!(
                a,
                Action::EndBattle | Action::SwitchPokemon { .. } | Action::Initiative { .. }
            ) && c
                .battle
                .as_ref()
                .and_then(|b| b.flow.as_ref())
                .is_some_and(|f| !f.queue.is_empty() || !f.ability_events.is_empty())
            {
                return Err(error("Resolve the complete pending chain first"));
            }
            if matches!(a, Action::Initiative { .. })
                && c.battle
                    .as_ref()
                    .and_then(|b| b.flow.as_ref())
                    .is_some_and(|f| f.started)
            {
                return Err(error(
                    "Initiative changes require next round or rollback of the confirmed order",
                ));
            }
            let (mut done, log) = crate::battle::transition(&old(c), actor, a, dice, cat)?;
            if !matches!(a, Action::UpgradeRules) {
                done.engine = c.engine.clone();
            }
            if matches!(
                a,
                Action::StartBattle { .. } | Action::StartTeamBattle { .. }
            ) {
                let b = done.battle.as_mut().unwrap();
                b.flow = Some(TurnFlow {
                    transfers: transfers::TransferState {
                        enabled: c.engine == ENGINE,
                        ..Default::default()
                    },
                    ability_changes: ability_changes::AbilityChanges {
                        enabled: [ENGINE, ASSIGNMENT_ENGINE].contains(&c.engine.as_str()),
                        ..Default::default()
                    },
                    decoys: decoys::DecoyState {
                        enabled: [ENGINE, ASSIGNMENT_ENGINE, DECOY_ENGINE]
                            .contains(&c.engine.as_str()),
                        ..Default::default()
                    },
                    tactics: tactics::TacticsState {
                        enabled: [ENGINE, ASSIGNMENT_ENGINE, DECOY_ENGINE, TACTICS_ENGINE]
                            .contains(&c.engine.as_str()),
                        ..Default::default()
                    },
                    special: special::SpecialState {
                        enabled: [
                            ENGINE,
                            ASSIGNMENT_ENGINE,
                            DECOY_ENGINE,
                            TACTICS_ENGINE,
                            SPECIAL_ENGINE,
                        ]
                        .contains(&c.engine.as_str()),
                        ..Default::default()
                    },
                    ability_rules: [
                        ENGINE,
                        ASSIGNMENT_ENGINE,
                        DECOY_ENGINE,
                        TACTICS_ENGINE,
                        SPECIAL_ENGINE,
                        EXTRA_ENGINE,
                        FIELD_ENGINE,
                        ABILITY_ENGINE,
                    ]
                    .contains(&c.engine.as_str()),
                    extra: extra::ExtraState {
                        enabled: [
                            ENGINE,
                            ASSIGNMENT_ENGINE,
                            DECOY_ENGINE,
                            TACTICS_ENGINE,
                            SPECIAL_ENGINE,
                            EXTRA_ENGINE,
                        ]
                        .contains(&c.engine.as_str()),
                        ..Default::default()
                    },
                    battlefield: field::Battlefield {
                        enabled: [
                            ENGINE,
                            ASSIGNMENT_ENGINE,
                            DECOY_ENGINE,
                            TACTICS_ENGINE,
                            SPECIAL_ENGINE,
                            EXTRA_ENGINE,
                            FIELD_ENGINE,
                        ]
                        .contains(&c.engine.as_str()),
                        sunlight: true,
                        ..Default::default()
                    },
                    ..TurnFlow::default()
                });
                for p in &mut b.combatants {
                    p.runtime.will = done.sheets[&p.sheet_id]
                        .workflow
                        .as_ref()
                        .map_or(0, |w| w.will);
                    let names = p.conditions.clone();
                    for name in names {
                        if [
                            "Burn",
                            "Burn 2",
                            "Burn 3",
                            "Badly Poisoned",
                            "Poison",
                            "Paralysis",
                            "Sleep",
                            "Confusion",
                            "Flinch",
                            "Frozen Solid",
                        ]
                        .contains(&name.as_str())
                        {
                            p.runtime.statuses.push(Status {
                                kind: name,
                                detail: String::new(),
                                rounds: None,
                                progress: 0,
                            });
                        }
                    }
                }
            }
            if matches!(a, Action::EndBattle) {
                if let Some(b) = done.battle.as_ref() {
                    for p in &b.combatants {
                        if let Some(s) = done.sheets.get_mut(&p.sheet_id) {
                            if let Some(w) = s.workflow.as_mut() {
                                w.will = p.runtime.will;
                            }
                            s.conditions = p
                                .runtime
                                .statuses
                                .iter()
                                .filter(|s| {
                                    !["Confusion", "Flinch", "Disable", "In Love"]
                                        .contains(&s.kind.as_str())
                                })
                                .map(|s| s.kind.clone())
                                .chain(
                                    p.conditions
                                        .iter()
                                        .filter(|name| {
                                            ![
                                                "Burn",
                                                "Burn 2",
                                                "Burn 3",
                                                "Badly Poisoned",
                                                "Poison",
                                                "Paralysis",
                                                "Sleep",
                                                "Confusion",
                                                "Flinch",
                                                "Frozen Solid",
                                            ]
                                            .contains(&name.as_str())
                                        })
                                        .cloned(),
                                )
                                .collect();
                            if p.runtime.will == 0 {
                                s.hp = 0;
                            }
                        }
                    }
                }
            }
            if matches!(a, Action::SwitchPokemon { .. }) {
                let b = done.battle.as_mut().unwrap();
                for p in b.combatants.iter_mut().filter(|p| !p.deployed) {
                    p.runtime.statuses.retain(|s| {
                        !["Confusion", "Flinch", "Disable", "In Love"].contains(&s.kind.as_str())
                    });
                    p.runtime.modifiers.clear();
                    p.runtime.pain_ignored = 0;
                }
                let current_id = b.flow.as_ref().and_then(|f| f.order.get(f.cursor)).cloned();
                b.flow
                    .as_mut()
                    .unwrap()
                    .order
                    .retain(|id| b.combatants.iter().any(|p| &p.id == id && p.deployed));
                let f = b.flow.as_mut().unwrap();
                f.cursor = current_id
                    .and_then(|id| f.order.iter().position(|v| v == &id))
                    .unwrap_or_else(|| f.cursor.min(f.order.len().saturating_sub(1)));
            }
            if let Action::Override {
                combatant_id,
                conditions,
                ..
            } = a
            {
                let p = fighter_mut(&mut done, combatant_id)?;
                p.runtime.statuses.retain(|s| conditions.contains(&s.kind));
                for kind in conditions.iter().filter(|k| known_status(k)) {
                    if !p.runtime.statuses.iter().any(|s| &s.kind == kind) {
                        if ["Disable", "In Love"].contains(&kind.as_str()) {
                            return Err(error("Use structured status controls to specify disabled move or beloved"));
                        }
                        p.runtime.statuses.push(Status {
                            kind: kind.clone(),
                            detail: String::new(),
                            rounds: None,
                            progress: 0,
                        });
                    }
                }
            }
            if !matches!(a, Action::RollbackBattle { .. } | Action::Override { .. }) {
                refresh_initiative(c, &mut done)?;
            }
            if !matches!(a, Action::RollbackBattle { .. }) {
                ability_changes::lifecycle(c, &mut done, cat)?;
                tactics::lifecycle(c, &mut done, cat)?;
                field::lifecycle(c, &mut done, cat)?;
            }
            abilities::lifecycle(c, &mut done)?;
            if !matches!(a, Action::RollbackBattle { .. }) {
                extra::lifecycle(c, &mut done, cat)?;
                special::lifecycle(c, &mut done, cat)?;
                decoys::lifecycle(&mut done);
            }
            cleanup_covers(c, &mut done, a);
            sync_labels(&mut done);
            return Ok((done, log));
        }
    };
    ability_changes::lifecycle(c, &mut next, cat)?;
    tactics::lifecycle(c, &mut next, cat)?;
    field::lifecycle(c, &mut next, cat)?;
    abilities::lifecycle(c, &mut next)?;
    extra::lifecycle(c, &mut next, cat)?;
    special::lifecycle(c, &mut next, cat)?;
    decoys::lifecycle(&mut next);
    cleanup_covers(c, &mut next, a);
    refresh_initiative(c, &mut next)?;
    sync_labels(&mut next);
    checkpoint(c, &mut next, &description);
    Ok((next, description))
}
fn clash_damage(c: &Campaign, id: &str, move_type: &str, cat: &Catalog) -> Result<i32> {
    let p = fighter(c, id)?;
    let mut damage = 1;
    let move_type = if move_type == "Fighting" {
        "Fight"
    } else {
        move_type
    };
    for t in field::types(c, p, cat)? {
        let n = rules::type_effect(move_type, &t)?;
        if n == -100 {
            if move_type == "Ground" && t == "Flying" && field::has(c, "Gravity", p) {
                continue;
            }
            return Ok(0);
        }
        damage += n;
    }
    if field::weather(c, Some(p)) == Some(field::Weather::StrongWinds)
        && field::types(c, p, cat)?.iter().any(|t| t == "Flying")
    {
        damage = damage.min(1);
    }
    Ok(damage.max(0))
}

fn set_status(
    c: &mut Campaign,
    id: &str,
    kind: &str,
    detail: &str,
    rounds: Option<u32>,
    cat: &Catalog,
) -> Result<()> {
    set_status_from(c, id, kind, detail, rounds, cat, None)
}
fn set_status_from(
    c: &mut Campaign,
    id: &str,
    kind: &str,
    detail: &str,
    rounds: Option<u32>,
    cat: &Catalog,
    source: Option<&str>,
) -> Result<()> {
    if ![
        "Burn",
        "Burn 2",
        "Burn 3",
        "Poison",
        "Badly Poisoned",
        "Paralysis",
        "Sleep",
        "Confusion",
        "Flinch",
        "Disable",
        "Frozen Solid",
        "In Love",
    ]
    .contains(&kind)
        || detail.len() > 200
        || rounds.is_some_and(|r| r == 0 || r > 100)
    {
        return Err(error("Invalid structured condition"));
    }
    if kind == "Disable" && detail.is_empty() {
        return Err(error("Specify the disabled move"));
    }
    if kind == "In Love" && fighter(c, detail).is_err() {
        return Err(error("Specify the beloved combatant ID"));
    }
    let p = fighter(c, id)?;
    let species = cat.find(
        "species",
        "AppId",
        c.sheets[&p.sheet_id].species_id.as_deref().unwrap_or(""),
    );
    let immune = species.is_some_and(|s| {
        ["Type1", "Type2"]
            .iter()
            .filter_map(|k| s[*k].as_str())
            .any(|t| {
                (kind.starts_with("Burn") && t == "Fire")
                    || (kind == "Paralysis" && t == "Electric")
                    || (kind.contains("Poison") && ["Poison", "Steel"].contains(&t))
                    || (kind == "Frozen Solid" && t == "Ice")
            })
    });
    let corrosion = extra::enabled(c)
        && kind.contains("Poison")
        && source.is_some_and(|id| fighter(c, id).is_ok_and(|p| p.selected_ability == "Corrosion"));
    if !corrosion
        && ((kind == "Confusion" && p.selected_ability == "Own Tempo")
            || (kind == "In Love" && p.selected_ability == "Oblivious")
            || abilities::status_immune(c, p, kind)
            || extra::status_immune(c, p, kind, source))
    {
        return Err(error("Selected ability is immune to this condition"));
    }
    let immune = if field::enabled(c) {
        field::types(c, p, cat)?.iter().any(|t| {
            (kind.starts_with("Burn") && t == "Fire")
                || (kind == "Paralysis" && t == "Electric")
                || (kind.contains("Poison") && ["Poison", "Steel"].contains(&t.as_str()))
                || (kind == "Frozen Solid" && t == "Ice")
        })
    } else {
        immune
    };
    if field::prevents_status(c, p, kind)
        || (kind == "Burn"
            && p.runtime
                .statuses
                .iter()
                .any(|s| s.kind.starts_with("Burn"))
            && field::weather(c, Some(p)) == Some(field::Weather::Rain))
    {
        return Err(error("Battlefield prevents this condition"));
    }
    if immune && !corrosion {
        return Err(error("Species type is immune to this condition"));
    }
    let round = battle(c)?.round;
    if kind == "Flinch" && p.runtime.flinch_round == round {
        return Err(error("Only one Flinch per subject per round"));
    }
    let p = fighter_mut(c, id)?;
    if kind == "Flinch" {
        p.runtime.flinch_round = round;
    }
    let mut kind = kind.to_string();
    if kind.starts_with("Burn") {
        if let Some(previous) = p
            .runtime
            .statuses
            .iter()
            .find(|s| s.kind.starts_with("Burn"))
        {
            kind = match previous.kind.as_str() {
                "Burn" => "Burn 2",
                _ => "Burn 3",
            }
            .into();
        }
        p.runtime.statuses.retain(|s| !s.kind.starts_with("Burn"));
    } else if kind.contains("Poison") {
        if p.runtime.statuses.iter().any(|s| s.kind.contains("Poison")) {
            kind = "Badly Poisoned".into();
        }
        p.runtime.statuses.retain(|s| !s.kind.contains("Poison"));
    } else if kind != "Disable" && p.runtime.statuses.iter().any(|s| s.kind == kind) {
        return Ok(());
    }
    p.runtime
        .statuses
        .retain(|s| s.kind != kind || (kind == "Disable" && s.detail != detail));
    let applied = kind.clone();
    p.runtime.statuses.push(Status {
        kind,
        detail: detail.into(),
        rounds,
        progress: 0,
    });
    extra::after_status(c, id, &applied)?;
    Ok(())
}
fn end_round(c: &mut Campaign, _cat: &Catalog) -> Result<()> {
    let abilities_on = abilities::enabled(c);
    let extended = extra::enabled(c);
    let b = c.battle.as_mut().unwrap();
    for p in &mut b.combatants {
        if p.hp == 0 {
            p.runtime
                .statuses
                .retain(|s| !s.kind.starts_with("Burn") && !s.kind.contains("Poison"));
            continue;
        }
        if p.runtime.statuses.iter().any(|s| {
            ["Burn 3", "Badly Poisoned"].contains(&s.kind.as_str())
                && !(abilities::status_damage_immune(abilities_on, p, &s.kind)
                    || (extended && extra::status_damage_immune(p, &s.kind)))
                && p.runtime.adjudicated_round != b.round
        }) {
            return Err(AppError::new("manual_rule","Severe status escalation conflicts between pp.84-86; record the GM severe-status outcome before advancing"));
        }
        for s in &p.runtime.statuses {
            let damage = if abilities::status_damage_immune(abilities_on, p, &s.kind)
                || (extended && extra::status_damage_immune(p, &s.kind))
            {
                0
            } else {
                match s.kind.as_str() {
                    "Burn" => 1,
                    "Burn 2" | "Poison" => 2,
                    _ => 0,
                }
            };
            p.hp = (p.hp - damage).max(0);
        }
        for s in &mut p.runtime.statuses {
            if let Some(r) = s.rounds.as_mut() {
                *r = r.saturating_sub(1);
            }
        }
        p.runtime.statuses.retain(|s| {
            s.rounds != Some(0)
                && (p.hp > 0 || !["Burn", "Burn 2", "Poison"].contains(&s.kind.as_str()))
        });
    }
    Ok(())
}
// Reviewed definitions are exact-name whitelists; all others retain visible adjudication.
fn no_effect(name: &str, cat: &Catalog) -> Result<bool> {
    let m = move_entry(cat, name)?;
    Ok([
        "Tackle",
        "Swift",
        "Slash",
        "Quick Attack",
        "Extreme Speed",
        "Splash",
    ]
    .contains(&name)
        || (m["Effect"].as_str() == Some("Single Target.")
            && m["AddedEffects"].as_object().is_some_and(|v| v.is_empty())
            && m["Attributes"].as_object().is_some_and(|v| v.is_empty())))
}
fn chance_effect(name: &str) -> Option<(&str, u32)> {
    match name {
        "Thunder Shock" | "Thunderbolt" | "Thunder Punch" => Some(("Paralysis", 1)),
        "Spark" => Some(("Paralysis", 3)),
        "Confusion" => Some(("Confusion", 1)),
        "Ice Beam" | "Ice Punch" | "Blizzard" | "Powder Snow" => Some(("Frozen Solid", 1)),
        "Thunder" => Some(("Paralysis", 3)),
        "Hurricane" => Some(("Confusion", 3)),
        "Hyper Fang" => Some(("Flinch", 1)),
        _ => None,
    }
}
fn effect_dice(c: &Campaign, q: &QueuedAction, _cat: &Catalog) -> Result<u32> {
    if let Some(n) = special::effect_dice(c, q, _cat)? {
        return Ok(n);
    }
    if !field::enabled(c)
        && [
            "Ice Beam",
            "Ice Punch",
            "Blizzard",
            "Powder Snow",
            "Thunder",
            "Hurricane",
        ]
        .contains(&q.move_name.as_str())
    {
        return Ok(0);
    }
    Ok(if q.move_name == "Submission" {
        if abilities::enabled(c)
            && (fighter(c, &q.actor_id)?.selected_ability == "Magic Guard"
                || (extra::enabled(c) && fighter(c, &q.actor_id)?.selected_ability == "Rock Head"))
        {
            0
        } else {
            q.raw_damage_successes
        }
    } else {
        chance_effect(&q.move_name).map_or(0, |(kind, n)| {
            let mut n = n + u32::from(
                kind == "Frozen Solid"
                    && matches!(
                        field::weather(c, None),
                        Some(field::Weather::Snow | field::Weather::Hail)
                    ),
            );
            if extra::enabled(c) {
                if fighter(c, &q.actor_id).is_ok_and(|p| p.selected_ability == "Serene Grace") {
                    n += 2;
                }
                if fighter(c, &q.target_id).is_ok_and(|p| p.selected_ability == "Wonder Skin") {
                    n = n.saturating_sub(2);
                }
            }
            n
        })
    })
}
fn effects(c: &mut Campaign, q: &QueuedAction, dice: &[u8], cat: &Catalog) -> Result<()> {
    let m = move_entry(cat, &q.move_name)?;
    if m["Category"] == "Support" && decoys::blocks_effects(c, q, &m) {
        return Ok(());
    }
    if let Some(redirected) = tactics::effect_target(c, q, cat)? {
        if redirected.cancelled {
            return Ok(());
        }
        let result = effects_inner(c, &redirected, dice, cat);
        tactics::clear_payer(c);
        return result;
    }
    effects_inner(c, q, dice, cat)
}
fn effects_inner(c: &mut Campaign, q: &QueuedAction, dice: &[u8], cat: &Catalog) -> Result<()> {
    if tactics::move_effect(c, q, cat)? {
        return Ok(());
    }
    if special::effects(c, q, dice, cat)? {
        return Ok(());
    }
    let data = move_entry(cat, &q.move_name)?;
    if decoys::blocks_effects(c, q, &data) {
        if ["Burn Up", "Double Shock"].contains(&q.move_name.as_str()) {
            field::move_effect(c, q, cat)?;
        } else if decoys::unreviewed_user_clause(&data["AddedEffects"]) {
            return Err(AppError::new("manual_rule", "Decoy blocks target effects, but this move has an unsupported user effect; record its GM resolution explicitly"));
        }
        return Ok(());
    }
    if !field::enabled(c)
        && [
            "Ice Beam",
            "Ice Punch",
            "Blizzard",
            "Powder Snow",
            "Thunder",
            "Hurricane",
        ]
        .contains(&q.move_name.as_str())
    {
        return Err(AppError::new(
            "manual_rule",
            "This effect requires the battlefield rules upgrade",
        ));
    }
    if extra::effects_immune(
        c,
        fighter(c, &q.target_id)?,
        &move_entry(cat, &q.move_name)?,
    ) {
        return Ok(());
    }
    if ability_changes::move_effect(c, q, cat)? {
        return Ok(());
    }
    if extra::move_effect(c, q)? {
        return Ok(());
    }
    if field::move_effect(c, q, cat)? {
        return Ok(());
    }
    let data = move_entry(cat, &q.move_name)?;
    if abilities::move_immune(c, fighter(c, &q.target_id)?, &data) {
        return Ok(());
    }
    match q.move_name.as_str() {
        "Protect" => {
            fighter_mut(c, &q.actor_id)?.runtime.shield = 3;
        }
        "Recover" | "Slack Off" | "Soft Boiled" => {
            let p = fighter_mut(c, &q.actor_id)?;
            if p.runtime.will < 1 {
                return Err(error("Healing requires one Will"));
            }
            let amount = 3.min(3 - p.runtime.healed).min(p.max_hp - p.hp);
            if p.hp == 0 {
                return Err(error("Healing cannot revive a fainted combatant"));
            }
            p.runtime.will -= 1;
            p.hp += amount;
            p.runtime.healed += amount;
        }
        "Agility" => {
            fighter_mut(c, &q.actor_id)?
                .runtime
                .modifiers
                .insert("Dexterity".into(), 2);
        }
        "Hyper Beam" => {
            let round = battle(c)?.round;
            let rt = &mut fighter_mut(c, &q.actor_id)?.runtime;
            rt.recharge_actions += 1;
            rt.recharge_round = round + 1;
        }
        "Submission" => {
            let damage = if abilities::enabled(c)
                && (fighter(c, &q.actor_id)?.selected_ability == "Magic Guard"
                    || (extra::enabled(c)
                        && fighter(c, &q.actor_id)?.selected_ability == "Rock Head"))
            {
                0
            } else {
                rules::successes(dice)
            };
            let p = fighter_mut(c, &q.actor_id)?;
            p.hp = (p.hp - damage).max(0);
        }
        name if chance_effect(name).is_some() => {
            let (kind, _) = chance_effect(name).unwrap();
            let already_flinched = kind == "Flinch"
                && fighter(c, &q.target_id)?.runtime.flinch_round == battle(c)?.round;
            let dust = abilities::enabled(c)
                && fighter(c, &q.target_id)?.selected_ability == "Shield Dust"
                && fighter(c, &q.actor_id)?.side != fighter(c, &q.target_id)?.side;
            if dice.contains(&6) && !already_flinched && !dust {
                if let Err(e) =
                    abilities::inflict(c, &q.actor_id, &q.target_id, kind, "", None, cat)
                {
                    if ![
                        "Species type is immune to this condition",
                        "Selected ability is immune to this condition",
                        "Battlefield prevents this condition",
                    ]
                    .contains(&e.message.as_str())
                    {
                        return Err(e);
                    }
                }
            }
        }
        _ if no_effect(&q.move_name, cat)? => {}
        _ => {
            return Err(AppError::new(
                "manual_rule",
                "Added effect has no reviewed executable definition; GM must adjudicate explicitly",
            ))
        }
    }
    Ok(())
}
