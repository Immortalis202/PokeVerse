//! MOVE-A59 copy moves: Copycat, Mirror Move, Instruct, Assist and Me First cast a copied
//! template through the shared copied-cast pipeline; Mimic and Sketch replace themselves.
use super::*;

fn op(c: &Campaign, name: &str) -> Option<&'static str> {
    completion::definition(c, name)
        .filter(|(kind, _)| *kind == "copy-op")
        .and_then(|(_, d)| d["op"].as_str())
}
/// Unique, Max/Z, Sappy Seed and other copy moves cannot be copied.
fn copyable(m: &serde_json::Value, support_allowed: bool) -> bool {
    m["Attributes"]["Unique"] != true
        && m["Attributes"]["MaxMove"] != true
        && m["Attributes"]["ZMove"] != true
        && m["Attributes"]["CopyMove"] != true
        && !m["Effect"].as_str().unwrap_or("").contains("Unique")
        && m["Name"] != "Sappy Seed"
        && (support_allowed || m["Category"] != "Support")
}
/// The last Pokemon move the combatant performed in this battle.
pub fn last_move(c: &Campaign, id: &str) -> Option<String> {
    flow(c)
        .ok()?
        .completion
        .uses
        .iter()
        .rev()
        .find(|u| u.actor == id)
        .map(|u| u.name.clone())
}
/// Moves known by allies on the field and by the user's party, active or not.
fn party_moves(c: &Campaign, id: &str) -> Result<BTreeSet<String>> {
    let p = fighter(c, id)?;
    let trainer = c.sheets[&p.sheet_id].trainer_id.clone();
    let mut moves = BTreeSet::new();
    for q in &battle(c)?.combatants {
        if q.id != p.id && q.side == p.side {
            moves.extend(rules::effective_sheet(c, q)?.moves);
        }
    }
    if let Some(trainer) = trainer {
        for (sid, s) in &c.sheets {
            if *sid != p.sheet_id
                && s.kind == "pokemon"
                && s.trainer_id.as_deref() == Some(&trainer)
            {
                moves.extend(s.moves.iter().cloned());
            }
        }
    }
    Ok(moves)
}

/// Validate a copied cast and build the action it runs.
fn cast_input(
    c: &Campaign,
    id: &str,
    origin: &str,
    source: &str,
    template: &str,
    targets: &[String],
    cat: &Catalog,
) -> Result<(Campaign, Action)> {
    let Some(op) = op(c, origin) else {
        return Err(error(
            "Choose Copycat, Mirror Move, Instruct, Assist or Me First",
        ));
    };
    if !["copy-last", "assist", "me-first"].contains(&op) {
        return Err(error("Mimic and Sketch are cast normally"));
    }
    let p = fighter(c, id)?;
    if !rules::effective_sheet(c, p)?
        .moves
        .iter()
        .any(|m| m == origin)
    {
        return Err(error("The user does not know this copy move"));
    }
    let m = move_entry(cat, template)?;
    if !copyable(&m, op != "me-first") {
        return Err(error("That move cannot be copied"));
    }
    match op {
        "copy-last" => {
            if fighter(c, source)?.side == p.side && origin != "Instruct" {
                return Err(error("Copy a foe's move"));
            }
            if last_move(c, source).as_deref() != Some(template) {
                return Err(error("Copy the last move that combatant performed"));
            }
        }
        "assist" => {
            if !party_moves(c, id)?.contains(template) {
                return Err(error(
                    "Assist copies a move known by an ally or party member",
                ));
            }
        }
        _ => {
            let incoming = incoming(c)?;
            if incoming.actor_id != source || incoming.move_name != template {
                return Err(error("Me First copies the incoming move it reacts to"));
            }
        }
    }
    let mut input = c.clone();
    flow_mut(&mut input).completion.copied_cast = Some(copies::CopiedCast {
        actor_id: id.into(),
        template: template.into(),
        origin: origin.into(),
    });
    let action = if op == "me-first" {
        Action::Tactics {
            command: tactics::TacticsCommand::Reaction {
                combatant_id: id.into(),
                move_name: template.into(),
                target_ids: targets.to_vec(),
            },
            reason: format!("{origin} copies {template}"),
        }
    } else {
        Action::AccuracyTargets {
            combatant_id: id.into(),
            target_ids: targets.to_vec(),
            move_name: template.into(),
            pool_modifier: 0,
            success_modifier: 0,
            reason: format!("{origin} copies {template}"),
        }
    };
    Ok((input, action))
}
pub fn dice(c: &Campaign, cmd: &special::SpecialCommand, cat: &Catalog) -> Result<u32> {
    let special::SpecialCommand::CastCopy {
        combatant_id,
        origin,
        source_id,
        template,
        target_ids,
    } = cmd
    else {
        return Err(error("Not a copy command"));
    };
    let (input, action) = cast_input(
        c,
        combatant_id,
        origin,
        source_id,
        template,
        target_ids,
        cat,
    )?;
    super::dice_count(&input, &action, cat)
}
pub fn transition(
    c: &Campaign,
    actor: &str,
    cmd: &special::SpecialCommand,
    why: &str,
    faces: &[u8],
    cat: &Catalog,
) -> Result<(Campaign, String)> {
    let special::SpecialCommand::CastCopy {
        combatant_id,
        origin,
        source_id,
        template,
        target_ids,
    } = cmd
    else {
        return Err(error("Not a copy command"));
    };
    reason(why)?;
    owner(c, actor, combatant_id)?;
    let used = fighter(c, combatant_id)?.used_moves.clone();
    let (input, action) = cast_input(
        c,
        combatant_id,
        origin,
        source_id,
        template,
        target_ids,
        cat,
    )?;
    let (mut done, log) = super::transition(&input, actor, &action, faces.to_vec(), cat)?;
    let p = fighter_mut(&mut done, combatant_id)?;
    p.used_moves = used;
    p.used_moves.push(origin.clone());
    let log = format!("{origin} copies {template}: {log}. {why}");
    done.battle_undo = c.battle_undo.clone();
    checkpoint(c, &mut done, &log);
    Ok((done, log))
}

/// Mimic replaces itself with the copied move for the scene; Sketch does so permanently.
pub fn grant(c: &Campaign, p: &Combatant, sheet: &mut Sheet) {
    if let Some(template) = flow(c).ok().and_then(|f| f.completion.mimicked.get(&p.id)) {
        if let Some(slot) = sheet.moves.iter().position(|m| m == "Mimic") {
            sheet.moves[slot] = template.clone();
            sheet.moves.retain(|m| m != "Mimic");
        }
    }
}
pub fn guard(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> Result<()> {
    if ["copy-last", "assist", "me-first"]
        .contains(&op(c, m["Name"].as_str().unwrap_or("")).unwrap_or(""))
    {
        return Err(error(
            "Use the copy-move control: choose the copied move and its targets",
        ));
    }
    let _ = p;
    Ok(())
}
pub fn effect(c: &mut Campaign, q: &QueuedAction, cat: &Catalog) -> Result<()> {
    if q.ice || completion::immune_target(c, q, cat)? {
        return Ok(());
    }
    let op = op(c, &q.move_name).unwrap_or("");
    if !["mimic", "sketch"].contains(&op) {
        return Ok(());
    }
    let Some(template) = last_move(c, &q.target_id) else {
        return Ok(());
    };
    if !copyable(&move_entry(cat, &template)?, true) {
        return Ok(());
    }
    let p = fighter(c, &q.actor_id)?.clone();
    if rules::effective_sheet(c, &p)?.moves.contains(&template) {
        return Ok(());
    }
    if op == "mimic" {
        flow_mut(c)
            .completion
            .mimicked
            .insert(q.actor_id.clone(), template);
    } else if let Some(sheet) = c.sheets.get_mut(&p.sheet_id) {
        if let Some(slot) = sheet.moves.iter().position(|m| m == "Sketch") {
            sheet.moves[slot] = template;
        }
    }
    Ok(())
}
pub fn lifecycle(c: &mut Campaign) -> Result<()> {
    let active: BTreeSet<String> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    flow_mut(c)
        .completion
        .mimicked
        .retain(|id, _| active.contains(id));
    Ok(())
}
