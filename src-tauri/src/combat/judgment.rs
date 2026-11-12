//! Judgment: saved GM parameters and receipts for applied mechanical consequences.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct JudgmentProfile {
    pub move_type: String,
    pub power: i32,
    pub bypass_unique: bool,
    pub unique_abilities: Vec<String>,
    pub turn: u32,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct JudgmentConsequence {
    pub kind: JudgmentConsequenceKind,
    pub revision: u32,
    pub description: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub enum JudgmentConsequenceKind {
    CombatantState,
    Condition,
    Stat,
    Field,
    Equipment,
    Will,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct JudgmentCast {
    pub actor_id: String,
    pub profile: JudgmentProfile,
    pub recording: bool,
    pub consequences: Vec<JudgmentConsequence>,
    pub complete: bool,
    pub reason: String,
}
pub fn active(c: &Campaign, m: &serde_json::Value) -> bool {
    m["Name"] == "Judgment" && completion::definition(c, "Judgment").is_some()
}
fn profile<'a>(c: &'a Campaign, id: &str) -> Option<&'a JudgmentProfile> {
    let f = flow(c).ok()?;
    if let Some(q) = f
        .queue
        .iter()
        .find(|q| q.actor_id == id && q.move_name == "Judgment")
    {
        if let Some(cast) = f.completion.judgment_casts.get(&q.sequence) {
            return Some(&cast.profile);
        }
    }
    f.completion
        .judgment_prepared
        .get(id)
        .filter(|p| p.turn == f.turn)
}
pub fn guard(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> Result<()> {
    if active(c, m) && profile(c, &p.id).is_none() {
        return Err(error(
            "GM must prepare Judgment's Type, Power and Unique-protection ruling for this turn",
        ));
    }
    Ok(())
}
pub fn move_data(c: &Campaign, p: &Combatant, m: &mut serde_json::Value) {
    if active(c, m) {
        if let Some(profile) = profile(c, &p.id) {
            m["Type"] = profile.move_type.clone().into();
            m["Power"] = profile.power.into();
        }
    }
}
pub fn preview_note(c: &Campaign, id: &str) -> Option<String> {
    let p = profile(c, id)?;
    Some(format!("Judgment: saved {} Type, Power {}; defenses, shields, decoys and ordinary damage immunities are bypassed. Unique target abilities are {} by the recorded GM ruling. Accuracy and action eligibility still apply; the GM resolves extra mechanical effects after damage.", p.move_type, p.power, if p.bypass_unique { "bypassed" } else { "preserved" }))
}
pub fn retained_target(
    c: &Campaign,
    source: &str,
    target: &Combatant,
    m: &serde_json::Value,
) -> Combatant {
    if !active(c, m) {
        return target.clone();
    }
    let Some(profile) = profile(c, source) else {
        return target.clone();
    };
    let keep: Vec<_> = target
        .ability_names()
        .into_iter()
        .filter(|name| {
            (!profile.bypass_unique && profile.unique_abilities.contains(name))
                || (m["Type"] == "Fire" && ["Dry Skin", "Fluffy"].contains(&name.as_str()))
        })
        .collect();
    let mut copy = target.clone();
    copy.selected_ability = keep.first().cloned().unwrap_or_default();
    copy.runtime
        .ability
        .flags
        .retain(|name| !name.starts_with("as-one:"));
    copy.runtime.ability.flags.extend(
        keep.into_iter()
            .skip(1)
            .map(|name| format!("as-one:{name}")),
    );
    copy
}
pub fn survival(
    c: &mut Campaign,
    q: &QueuedAction,
    m: &serde_json::Value,
    target: &Combatant,
    mut amount: i32,
) -> Result<i32> {
    let kept = retained_target(c, &q.actor_id, target, m);
    if kept.has_ability("Disguise") {
        amount = extra::disguise(c, &q.target_id, amount)?;
    }
    if kept.has_ability("Ice Face") {
        amount = forms::damage_guard(c, &q.target_id, amount)?;
    }
    Ok(amount)
}
pub fn awaiting(c: &Campaign) -> bool {
    flow(c).is_ok_and(|f| {
        !f.window
            && f.queue.first().is_some_and(|q| {
                q.move_name == "Judgment"
                    && q.damage_done
                    && f.completion
                        .judgment_casts
                        .get(&q.sequence)
                        .is_some_and(|j| !j.complete)
            })
    })
}
fn current_cast(c: &Campaign, sequence: u32) -> Result<&JudgmentCast> {
    let q = current(c)?;
    if q.sequence != sequence
        || q.move_name != "Judgment"
        || !q.damage_done
        || completion::definition(c, "Judgment").is_none()
    {
        return Err(error(
            "Resolve the matching Judgment damage before its GM ruling",
        ));
    }
    flow(c)?
        .completion
        .judgment_casts
        .get(&sequence)
        .filter(|j| !j.complete)
        .ok_or_else(|| error("No unresolved saved Judgment ruling"))
}
pub fn dice(c: &Campaign, cmd: &special::SpecialCommand) -> Result<u32> {
    use special::SpecialCommand::*;
    if !special::enabled(c) || completion::definition(c, "Judgment").is_none() {
        return Err(error(
            "Judgment decisions require a newly adopting encounter",
        ));
    }
    match cmd {
        PrepareJudgment {
            combatant_id,
            move_type,
            power,
            ..
        } => {
            let p = fighter(c, combatant_id)?;
            if !p.deployed || p.hp <= 0 || !flow(c)?.started {
                return Err(error(
                    "Prepare Judgment for an active combatant after turns begin",
                ));
            }
            if flow(c)?
                .queue
                .iter()
                .any(|q| q.actor_id == *combatant_id && q.move_name == "Judgment")
            {
                return Err(error("Cannot change an in-flight Judgment ruling"));
            }
            if !rules::effective_sheet(c, fighter(c, combatant_id)?)?
                .moves
                .contains(&"Judgment".into())
            {
                return Err(error("Choose a combatant that knows Judgment"));
            }
            rules::type_effect(move_type, "Normal")?;
            if !(0..=100).contains(power) {
                return Err(error(
                    "Judgment Power must be between 0 and 100; ordinary pool limits still apply",
                ));
            }
        }
        BeginJudgmentEffects { sequence } => {
            if current_cast(c, *sequence)?.recording {
                return Err(error("Judgment consequences are already being recorded"));
            }
        }
        RuleJudgment {
            sequence,
            no_extra_effects,
        } => {
            let j = current_cast(c, *sequence)?;
            if *no_extra_effects && !j.consequences.is_empty() {
                return Err(error(
                    "Applied Judgment consequences cannot be declared absent",
                ));
            }
            if !*no_extra_effects && (!j.recording || j.consequences.is_empty()) {
                return Err(error("Apply a mechanical consequence through GM battle controls before finishing with extra effects"));
            }
        }
        _ => return Err(error("Not a Judgment decision command")),
    }
    Ok(0)
}
pub fn command(
    c: &mut Campaign,
    actor: &str,
    cmd: &special::SpecialCommand,
    why: &str,
    cat: &Catalog,
) -> Result<String> {
    use special::SpecialCommand::*;
    gm(c, actor)?;
    reason(why)?;
    dice(c, cmd)?;
    match cmd {
        PrepareJudgment {
            combatant_id,
            move_type,
            power,
            bypass_unique,
        } => {
            let profile = JudgmentProfile {
                move_type: move_type.clone(),
                power: *power,
                bypass_unique: *bypass_unique,
                unique_abilities: cat.unique_ability_names()?,
                turn: flow(c)?.turn,
                reason: why.into(),
            };
            flow_mut(c)
                .completion
                .judgment_prepared
                .insert(combatant_id.clone(), profile);
        }
        BeginJudgmentEffects { sequence } => {
            let j = flow_mut(c)
                .completion
                .judgment_casts
                .get_mut(sequence)
                .unwrap();
            j.recording = true;
            j.reason = why.into();
        }
        RuleJudgment {
            sequence,
            no_extra_effects,
        } => {
            let j = flow_mut(c)
                .completion
                .judgment_casts
                .get_mut(sequence)
                .unwrap();
            j.complete = true;
            j.reason = why.into();
            finish(c);
            return Ok(format!(
                "GM completed Judgment #{sequence}; no extra effects={no_extra_effects}: {why}"
            ));
        }
        _ => return Err(error("Not a Judgment decision command")),
    }
    Ok(format!("Judgment GM decision {cmd:?}: {why}"))
}
fn scrub(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for key in ["history", "reason", "ruling", "corrections", "revision"] {
                map.remove(key);
            }
            for value in map.values_mut() {
                scrub(value);
            }
        }
        serde_json::Value::Array(list) => {
            for value in list {
                scrub(value);
            }
        }
        _ => {}
    }
}

fn statuses(c: &Campaign) -> Option<serde_json::Value> {
    Some(serde_json::json!(c
        .battle
        .as_ref()?
        .combatants
        .iter()
        .map(|p| (
            &p.id,
            p.runtime
                .statuses
                .iter()
                .map(|s| (
                    &s.kind,
                    s.rounds,
                    s.progress,
                    s.elapsed_rounds,
                    if ["Disable", "In Love", "Vulnerable", "Blocked"].contains(&s.kind.as_str()) {
                        s.detail.as_str()
                    } else {
                        ""
                    }
                ))
                .collect::<Vec<_>>()
        ))
        .collect::<Vec<_>>()))
}
fn mechanical(c: &Campaign, action: &Action) -> Option<serde_json::Value> {
    let value = match action {
        Action::Override { .. } => serde_json::json!([
            c.battle
                .as_ref()?
                .combatants
                .iter()
                .map(|p| (&p.id, p.hp, p.actions, p.reactions, p.initiative))
                .collect::<Vec<_>>(),
            statuses(c)?
        ]),
        Action::SetStatus { .. } | Action::InflictStatus { .. } => statuses(c)?,
        Action::BattleStat { .. } => serde_json::json!(c
            .battle
            .as_ref()?
            .combatants
            .iter()
            .map(|p| (
                &p.id,
                p.runtime
                    .modifiers
                    .iter()
                    .filter(|(_, value)| **value != 0)
                    .collect::<BTreeMap<_, _>>()
            ))
            .collect::<Vec<_>>()),
        Action::BattleStats { combatant_id, .. } => {
            let p = fighter(c, combatant_id).ok()?;
            serde_json::json!([p.stat_overrides, p.skill_overrides])
        }
        Action::Battlefield { .. } | Action::RemoveFieldEffect { .. } => {
            serde_json::to_value(&flow(c).ok()?.battlefield).ok()?
        }
        Action::HeldItem { .. } => serde_json::json!([flow(c).ok()?.items, c.sheets]),
        Action::AdjustWill { combatant_id, .. } => {
            serde_json::json!(fighter(c, combatant_id).ok()?.runtime.will)
        }
        _ => return None,
    };
    let mut value = value;
    scrub(&mut value);
    Some(value)
}
fn consequence_kind(action: &Action) -> Option<(JudgmentConsequenceKind, &'static str)> {
    use JudgmentConsequenceKind::*;
    Some(match action {
        Action::Override { .. } => (CombatantState, "GM applied a combatant-state consequence"),
        Action::SetStatus { .. } | Action::InflictStatus { .. } => {
            (Condition, "GM applied a condition consequence")
        }
        Action::BattleStat { .. } | Action::BattleStats { .. } => {
            (Stat, "GM applied a stat consequence")
        }
        Action::Battlefield { .. } | Action::RemoveFieldEffect { .. } => {
            (Field, "GM applied a battlefield consequence")
        }
        Action::HeldItem { .. } => (Equipment, "GM applied an equipment/resource consequence"),
        Action::AdjustWill { .. } => (Will, "GM applied an explicit Will correction"),
        _ => return None,
    })
}

pub fn observe(
    before: &Campaign,
    next: &mut Campaign,
    actor: &str,
    action: &Action,
    _log: &str,
    cat: &Catalog,
) -> Result<()> {
    if completion::definition(before, "Judgment").is_none() {
        return Ok(());
    }
    if let Action::Accuracy {
        combatant_id,
        move_name,
        ..
    }
    | Action::AccuracyTargets {
        combatant_id,
        move_name,
        ..
    }
    | Action::React {
        combatant_id,
        move_name,
        ..
    } = action
    {
        if move_name == "Judgment" {
            if let Some(mut p) = flow_mut(next)
                .completion
                .judgment_prepared
                .remove(combatant_id)
            {
                p.unique_abilities = cat.unique_ability_names()?;
                if let Some(q) = flow(next)?
                    .queue
                    .iter()
                    .find(|q| q.actor_id == *combatant_id && q.move_name == "Judgment")
                {
                    let sequence = q.sequence;
                    flow_mut(next).completion.judgment_casts.insert(
                        sequence,
                        JudgmentCast {
                            actor_id: combatant_id.clone(),
                            profile: p,
                            recording: false,
                            consequences: vec![],
                            complete: false,
                            reason: String::new(),
                        },
                    );
                }
            }
        }
    }
    if actor != before.gm_id || !awaiting(before) {
        return Ok(());
    }
    let sequence = flow(before)?.queue[0].sequence;
    if !flow(before)?.completion.judgment_casts[&sequence].recording {
        return Ok(());
    }
    if let (Some(old), Some(new)) = (mechanical(before, action), mechanical(next, action)) {
        if old != new {
            if let Some(j) = flow_mut(next).completion.judgment_casts.get_mut(&sequence) {
                j.consequences.push(JudgmentConsequence {
                    kind: consequence_kind(action).unwrap().0,
                    revision: before.revision + 1,
                    description: consequence_kind(action).unwrap().1.into(),
                });
            }
        }
    }
    Ok(())
}
