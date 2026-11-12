//! Source-preserving copied casts and committed Metronome draws.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct MetronomeTable {
    pub physical_faces: Vec<u8>,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct MetronomeDraw {
    pub actor_id: String,
    pub token: u32,
    pub turn: u32,
    pub faces: Vec<u8>,
    pub category: String,
    pub power: i32,
    pub candidates: Vec<String>,
    pub selected: Option<String>,
    pub attempt_limit: u32,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct CopiedCast {
    pub actor_id: String,
    pub template: String,
    pub origin: String,
}
fn enabled(c: &Campaign) -> bool {
    special::enabled(c) && completion::definition(c, "Metronome").is_some()
}
fn require(c: &Campaign) -> Result<()> {
    if !enabled(c) {
        return Err(error("Adopt Metronome rules in a new encounter first"));
    }
    Ok(())
}
fn draw(c: &Campaign, token: u32) -> Result<&MetronomeDraw> {
    require(c)?;
    let d = flow(c)?
        .completion
        .metronome_draw
        .as_ref()
        .filter(|d| d.token == token)
        .ok_or_else(|| error("Metronome draw changed"))?;
    turn(c, &d.actor_id)?;
    ready(c, &d.actor_id)?;
    if d.turn != flow(c)?.turn {
        return Err(error("Metronome draw belongs to a different turn"));
    }
    Ok(d)
}
fn draw_guard(c: &Campaign, id: &str, cat: &Catalog) -> Result<()> {
    require(c)?;
    turn(c, id)?;
    ready(c, id)?;
    if flow(c)?.completion.metronome_table.is_none() {
        return Err(error(
            "GM must record Metronome's category die table before rolling",
        ));
    }
    if flow(c)?.completion.metronome_draw.is_some() || flow(c)?.completion.copied_cast.is_some() {
        return Err(error("Resolve the committed copied move first"));
    }
    rules::accuracy(c, id, "Metronome", 0, cat)?;
    Ok(())
}
fn eligible(m: &serde_json::Value) -> bool {
    m["Attributes"]["Unique"] != true
        && m["Attributes"]["MaxMove"] != true
        && m["Attributes"]["ZMove"] != true
        && !m["Effect"].as_str().unwrap_or("").contains("Unique")
        && m["Name"] != "Sappy Seed"
}
pub fn command_matches(cmd: &special::SpecialCommand) -> bool {
    matches!(
        cmd,
        special::SpecialCommand::MetronomeTable { .. }
            | special::SpecialCommand::DrawMetronome { .. }
            | special::SpecialCommand::SelectMetronome { .. }
            | special::SpecialCommand::CastMetronome { .. }
            | special::SpecialCommand::DiscardMetronome { .. }
    )
}
pub fn dice(c: &Campaign, cmd: &special::SpecialCommand, cat: &Catalog) -> Result<u32> {
    use special::SpecialCommand::*;
    require(c)?;
    match cmd {
        MetronomeTable { physical_faces } => {
            let distinct: BTreeSet<_> = physical_faces.iter().copied().collect();
            if physical_faces.is_empty()
                || physical_faces.len() >= 6
                || distinct.len() != physical_faces.len()
                || physical_faces.iter().any(|v| !(1..=6).contains(v))
            {
                return Err(error(
                    "Choose one to five distinct Physical die faces from 1 to 6",
                ));
            }
            if !flow(c)?.queue.is_empty()
                || flow(c)?.completion.metronome_draw.is_some()
                || flow(c)?.completion.copied_cast.is_some()
            {
                return Err(error(
                    "Cannot change Metronome's table during a committed move",
                ));
            }
            Ok(0)
        }
        DrawMetronome { combatant_id } => {
            draw_guard(c, combatant_id, cat)?;
            Ok(2)
        }
        SelectMetronome { token, move_name } => {
            let d = draw(c, *token)?;
            if d.selected.is_some() {
                return Err(error("Metronome template already selected"));
            }
            let m = move_entry(cat, move_name)?;
            if !d.candidates.contains(move_name)
                || m["Category"] != d.category
                || m["Power"] != d.power
                || !eligible(&m)
            {
                return Err(error(
                    "Choose a non-Unique, in-scope move matching the committed category and Power",
                ));
            }
            Ok(0)
        }
        CastMetronome {
            token,
            target_id,
            attempts,
        } => {
            let (input, action) = cast_input(c, *token, target_id, attempts.unwrap_or(1), cat)?;
            super::dice_count(&input, &action, cat)
        }
        DiscardMetronome { token } => {
            draw(c, *token)?;
            Ok(0)
        }
        _ => Err(error("Not a copied-move command")),
    }
}
fn cast_input(
    c: &Campaign,
    token: u32,
    target: &str,
    attempts: u32,
    cat: &Catalog,
) -> Result<(Campaign, Action)> {
    let d = draw(c, token)?.clone();
    let template = d
        .selected
        .as_ref()
        .ok_or_else(|| error("GM must select the drawn Metronome template first"))?;
    let m = move_entry(cat, template)?;
    if attempts < 1 || attempts > special::limit(&m) {
        return Err(error("Choose a legal number of copied successive actions"));
    }
    if m["Category"] != d.category || m["Power"] != d.power || !eligible(&m) {
        return Err(error("Selected template no longer matches the saved draw"));
    }
    let mut input = c.clone();
    flow_mut(&mut input).completion.metronome_draw = None;
    flow_mut(&mut input).completion.copied_cast = Some(CopiedCast {
        actor_id: d.actor_id.clone(),
        template: template.clone(),
        origin: "Metronome".into(),
    });
    if attempts > 1 {
        return Ok((
            input,
            Action::SpecialMove {
                command: special::SpecialCommand::Successive {
                    reaction: false,
                    combatant_id: d.actor_id,
                    target_id: target.into(),
                    move_name: template.clone(),
                    attempts,
                },
                reason: "Copied Metronome successive actions using original Accuracy".into(),
            },
        ));
    }
    Ok((
        input,
        Action::Accuracy {
            combatant_id: d.actor_id,
            target_id: Some(target.into()),
            move_name: template.clone(),
            pool_modifier: 0,
            success_modifier: 0,
            reason: "Metronome copied cast using Insight + Perform".into(),
        },
    ))
}
pub fn transition(
    c: &Campaign,
    actor: &str,
    cmd: &special::SpecialCommand,
    why: &str,
    faces: &[u8],
    cat: &Catalog,
) -> Result<(Campaign, String)> {
    use special::SpecialCommand::*;
    reason(why)?;
    check_count(faces, dice(c, cmd, cat)?)?;
    let mut next = c.clone();
    let description = match cmd {
        MetronomeTable { physical_faces } => {
            gm(c, actor)?;
            flow_mut(&mut next).completion.metronome_table = Some(self::MetronomeTable {
                physical_faces: physical_faces.clone(),
                reason: why.into(),
            });
            format!("GM recorded Metronome Physical faces {physical_faces:?}; remaining faces mean Special")
        }
        DrawMetronome { combatant_id } => {
            owner(c, actor, combatant_id)?;
            let table = flow(c)?.completion.metronome_table.as_ref().unwrap();
            let category = if table.physical_faces.contains(&faces[0]) {
                "Physical"
            } else {
                "Special"
            };
            let candidates = cat
                .damage_templates(category, faces[1] as i32)?
                .into_iter()
                .filter(eligible)
                .filter_map(|m| m["Name"].as_str().map(str::to_owned))
                .collect();
            let token = c.revision + 1;
            let turn = flow(c)?.turn;
            flow_mut(&mut next).completion.metronome_draw = Some(MetronomeDraw {
                actor_id: combatant_id.clone(),
                token,
                turn,
                faces: faces.to_vec(),
                category: category.into(),
                power: faces[1] as i32,
                candidates,
                selected: None,
                attempt_limit: 1,
                reason: why.into(),
            });
            format!("Metronome #{token} committed {category}, Power {} from dice {faces:?}; Accuracy and action not spent yet", faces[1])
        }
        SelectMetronome { token, move_name } => {
            gm(c, actor)?;
            let limit = special::limit(&move_entry(cat, move_name)?);
            let d = flow_mut(&mut next)
                .completion
                .metronome_draw
                .as_mut()
                .unwrap();
            d.selected = Some(move_name.clone());
            d.attempt_limit = limit;
            d.reason = why.into();
            format!("GM selected {move_name} for committed Metronome #{token}")
        }
        CastMetronome {
            token,
            target_id,
            attempts,
        } => {
            let d = draw(c, *token)?;
            owner(c, actor, &d.actor_id)?;
            let used = fighter(c, &d.actor_id)?.used_moves.clone();
            let (input, action) = cast_input(c, *token, target_id, attempts.unwrap_or(1), cat)?;
            let (mut done, log) = super::transition(&input, actor, &action, faces.to_vec(), cat)?;
            let p = fighter_mut(&mut done, &d.actor_id)?;
            p.used_moves = used;
            p.used_moves.push("Metronome".into());
            next = done;
            format!("Metronome #{token}: {log}")
        }
        DiscardMetronome { token } => {
            gm(c, actor)?;
            let id = draw(c, *token)?.actor_id.clone();
            flow_mut(&mut next).completion.metronome_draw = None;
            let p = fighter_mut(&mut next, &id)?;
            p.actions += 1;
            p.used_moves.push("Metronome".into());
            advance(&mut next);
            format!("GM recorded Metronome #{token} as a failed attempt; one action spent")
        }
        _ => return Err(error("Not a copied-move command")),
    };
    next.battle_undo = c.battle_undo.clone();
    let log = format!("{description}. {why}");
    if matches!(cmd, special::SpecialCommand::CastMetronome { .. }) {
        checkpoint(c, &mut next, &log);
    }
    Ok((next, log))
}
pub fn phase(c: &Campaign, action: &Action) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    if matches!(action, Action::Accuracy { move_name, .. } | Action::AccuracyTargets { move_name, .. } if move_name == "Metronome")
    {
        return Err(error(
            "Draw Metronome's category and Power before casting its selected move",
        ));
    }
    let factual_input = matches!(
        action,
        Action::SpecialMove {
            command: special::SpecialCommand::Parameters { .. },
            ..
        } | Action::CompoundMove {
            command: compound::CompoundCommand::Environment { .. }
                | compound::CompoundCommand::Align { .. },
            ..
        }
    );
    let berry_input = flow(c)?.completion.metronome_draw.as_ref().is_some_and(|d| d.selected.as_deref() == Some("Natural Gift")
        && matches!(action, Action::HeldItem { command: items::ItemCommand::NaturalGift { combatant_id, .. }, .. } if combatant_id == &d.actor_id))
        || flow(c)?.completion.metronome_draw.as_ref().is_some_and(|d| d.selected.as_deref() == Some("Natural Gift") && matches!(action, Action::HeldItem { command: items::ItemCommand::BerryFlavor { .. }, .. }));
    if flow(c)?.completion.metronome_draw.is_some()
        && !factual_input
        && !berry_input
        && !matches!(action, Action::SpecialMove { command, .. } if command_matches(command))
        && !matches!(
            action,
            Action::RollbackBattle { .. }
                | Action::EndBattle
                | Action::ResolveAbility { .. }
                | Action::Battlefield { .. }
        )
    {
        return Err(error(
            "Resolve or discard the committed Metronome draw first",
        ));
    }
    Ok(())
}
pub fn matches(c: &Campaign, id: &str, name: &str) -> bool {
    enabled(c)
        && flow(c)
            .ok()
            .and_then(|f| f.completion.copied_cast.as_ref())
            .is_some_and(|s| s.actor_id == id && s.template == name)
}
pub fn identity<'a>(c: &'a Campaign, id: &str, name: &'a str) -> &'a str {
    if matches(c, id, name) {
        flow(c)
            .unwrap()
            .completion
            .copied_cast
            .as_ref()
            .unwrap()
            .origin
            .as_str()
    } else {
        name
    }
}
pub fn grant(c: &Campaign, p: &Combatant, sheet: &mut Sheet) {
    if let Some(s) = flow(c)
        .ok()
        .and_then(|f| f.completion.copied_cast.as_ref())
        .filter(|s| s.actor_id == p.id && enabled(c))
    {
        if !sheet.moves.contains(&s.template) {
            sheet.moves.push(s.template.clone());
        }
    }
}
pub fn move_data(c: &Campaign, p: &Combatant, m: &mut serde_json::Value) {
    if !matches(c, &p.id, m["Name"].as_str().unwrap_or("")) {
        return;
    }
    match identity(c, &p.id, m["Name"].as_str().unwrap_or("")) {
        "Metronome" => {
            m["Accuracy1"] = "Insight".into();
            m["Accuracy2"] = "Perform".into();
            // Metronome explicitly retains a single target, including area templates.
            m["Target"] = "Foe".into();
        }
        // Me First copies the incoming move as its own Reaction 1.
        "Me First" => {
            if let Some(attrs) = m["Attributes"].as_object_mut() {
                attrs.remove("LateReaction");
                attrs.insert("Reaction".into(), "1".into());
            } else {
                m["Attributes"] = serde_json::json!({"Reaction": "1"});
            }
        }
        _ => {}
    }
}
pub fn lifecycle(c: &mut Campaign) {
    let keep = flow(c).is_ok_and(|f| {
        f.completion.copied_cast.as_ref().is_some_and(|s| {
            f.queue.iter().any(|q| q.actor_id == s.actor_id)
                || f.special
                    .charges
                    .get(&s.actor_id)
                    .is_some_and(|q| q.action.move_name == s.template)
                || f.special.series.as_ref().is_some_and(|series| {
                    series.actor_id == s.actor_id && series.name == s.template
                })
        })
    });
    if !keep {
        if let Some(f) = c.battle.as_mut().and_then(|b| b.flow.as_mut()) {
            f.completion.copied_cast = None;
        }
    }
}
