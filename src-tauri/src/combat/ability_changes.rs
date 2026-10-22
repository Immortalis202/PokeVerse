//! MASTER pp.480,500,507,515,534,557,575. Encounter-only ability assignments.
//! selected_ability is the materialized effective ability used by every battle hook;
//! originals and assignments live here, never in permanent sheets.
use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct AbilityChanges {
    pub enabled: bool,
    pub originals: BTreeMap<String, String>,
    pub assignments: BTreeMap<String, String>,
    pub negated: BTreeSet<String>,
    pub suppressed: BTreeSet<String>,
    pub trace: BTreeMap<String, String>,
    pub active: BTreeSet<String>,
    pub gas_rulings: BTreeSet<String>,
    pub history: Vec<String>,
}
pub fn enabled(c: &Campaign) -> bool {
    c.battle
        .as_ref()
        .and_then(|b| b.flow.as_ref())
        .is_some_and(|f| f.ability_changes.enabled)
}
fn state(c: &Campaign) -> &AbilityChanges {
    &c.battle
        .as_ref()
        .unwrap()
        .flow
        .as_ref()
        .unwrap()
        .ability_changes
}
fn state_mut(c: &mut Campaign) -> &mut AbilityChanges {
    &mut c
        .battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .ability_changes
}
fn assigned(c: &Campaign, p: &Combatant) -> String {
    state(c)
        .trace
        .get(&p.id)
        .or_else(|| transfers::contact_copy(c, &p.id))
        .or_else(|| state(c).assignments.get(&p.id))
        .or_else(|| transfers::active_copy(c, &p.sheet_id).map(|r| &r.ability))
        .or_else(|| forms::identity(c, &p.id))
        .or_else(|| state(c).originals.get(&p.id))
        .unwrap_or(&p.selected_ability)
        .clone()
}
// Catalog exact references only. A missing definition is adjudicated, never guessed.
pub(super) fn unique(cat: &Catalog, name: &str) -> Result<bool> {
    if name.is_empty() {
        return Ok(false);
    }
    let entry = cat.find("abilities", "Name", name).ok_or_else(|| {
        AppError::new(
            "manual_rule",
            "Ability definition is unresolved; record a GM ruling",
        )
    })?;
    Ok(entry["Effect"]
        .as_str()
        .unwrap_or("")
        .to_lowercase()
        .contains("unique ability"))
}
pub fn move_effect(c: &mut Campaign, q: &QueuedAction, cat: &Catalog) -> Result<bool> {
    if ![
        "Skill Swap",
        "Role Play",
        "Worry Seed",
        "Simple Beam",
        "Gastro Acid",
    ]
    .contains(&q.move_name.as_str())
    {
        return Ok(false);
    }
    if !enabled(c) {
        return Err(AppError::new(
            "manual_rule",
            "Ability changes require the new encounter engine",
        ));
    }
    let user = fighter(c, &q.actor_id)?.clone();
    let target = fighter(c, &q.target_id)?.clone();
    let ua = assigned(c, &user);
    let ta = assigned(c, &target);
    if forms::union_active(c, &target.id)
        || forms::union_active(c, &user.id)
        || unique(cat, &ta)?
        || (["Skill Swap", "Role Play"].contains(&q.move_name.as_str()) && unique(cat, &ua)?)
    {
        return Err(AppError::new(
            "manual_rule",
            "Unique ability restriction: this change requires an explicit GM resolution",
        ));
    }
    match q.move_name.as_str() {
        "Skill Swap" => {
            state_mut(c).assignments.insert(user.id.clone(), ta.clone());
            state_mut(c)
                .assignments
                .insert(target.id.clone(), ua.clone());
            state_mut(c).trace.remove(&user.id);
            state_mut(c).trace.remove(&target.id);
        }
        "Role Play" => {
            state_mut(c).assignments.insert(user.id.clone(), ta.clone());
            state_mut(c).trace.remove(&user.id);
        }
        "Worry Seed" | "Simple Beam" => {
            state_mut(c).assignments.insert(
                target.id.clone(),
                if q.move_name == "Worry Seed" {
                    "Insomnia"
                } else {
                    "Simple"
                }
                .into(),
            );
            state_mut(c).trace.remove(&target.id);
        }
        "Gastro Acid" => {
            state_mut(c).negated.insert(target.id.clone());
        }
        _ => unreachable!(),
    }
    match q.move_name.as_str() {
        "Role Play" => {
            transfers::clear_contact(c, &user.id);
        }
        "Skill Swap" => {
            transfers::clear_contact(c, &user.id);
            transfers::clear_contact(c, &target.id);
        }
        "Worry Seed" | "Simple Beam" => {
            transfers::clear_contact(c, &target.id);
        }
        _ => {}
    }
    state_mut(c).history.push(format!(
        "{}: {} ({ua}) -> {} ({ta}); whole scene",
        q.move_name, user.name, target.name
    ));
    Ok(true)
}
pub fn lifecycle(before: &Campaign, c: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    forms::redirect_losses(before, c)?;
    transfers::lifecycle(before, c, cat)?;
    let ps = c.battle.as_ref().unwrap().combatants.clone();
    let active: BTreeSet<_> = ps
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    let ended = !c.battle.as_ref().unwrap().active;
    for p in &ps {
        state_mut(c)
            .originals
            .entry(p.id.clone())
            .or_insert_with(|| p.selected_ability.clone());
        if ended || !active.contains(&p.id) {
            state_mut(c).trace.remove(&p.id);
        }
    }
    if ended {
        for p in &mut c.battle.as_mut().unwrap().combatants {
            p.selected_ability = before
                .battle
                .as_ref()
                .and_then(|b| b.flow.as_ref())
                .and_then(|f| f.ability_changes.originals.get(&p.id))
                .cloned()
                .unwrap_or_else(|| p.selected_ability.clone());
        }
        let s = state_mut(c);
        s.assignments.clear();
        s.negated.clear();
        s.suppressed.clear();
        s.active.clear();
        s.trace.clear();
        return Ok(());
    }
    // Opposing gases are a stalemate under MASTER p.534; require a recorded GM decision.
    let gases: Vec<_> = ps
        .iter()
        .filter(|p| {
            active.contains(&p.id)
                && assigned(c, p) == "Neutralizing Gas"
                && !state(c).negated.contains(&p.id)
        })
        .cloned()
        .collect();
    for (i, g) in gases.iter().enumerate() {
        for h in gases.iter().skip(i + 1) {
            if !(h.side != g.side
                && (flow(c)
                    .unwrap()
                    .ranges
                    .get(&g.id)
                    .is_none_or(|ids| ids.contains(&h.id))
                    || flow(c)
                        .unwrap()
                        .ranges
                        .get(&h.id)
                        .is_none_or(|ids| ids.contains(&g.id))))
            {
                continue;
            }
            let mut pair = vec![g.id.clone(), h.id.clone()];
            pair.sort();
            let key = serde_json::to_string(&pair).unwrap();
            if !state(c).gas_rulings.contains(&key)
                && !flow(c)?
                    .ability_events
                    .iter()
                    .any(|e| e.kind == "ability-gas" && e.target_id == key)
            {
                flow_mut(c).ability_events.push(abilities::AbilityEvent {
                    id: format!("gas:{key}"),
                    combatant_id: g.id.clone(),
                    target_id: key,
                    ability: "Neutralizing Gas".into(),
                    kind: "ability-gas".into(),
                    move_name: "Opposing Gas stalemate: keep both emitters active".into(),
                    choices: vec!["simultaneous".into()],
                });
            }
        }
    }
    let mut suppressed = state(c).negated.clone();
    if forms::zero_active(c) {
        for p in &ps {
            if !forms::union_active(c, &p.id) && !unique(cat, &assigned(c, p))? {
                suppressed.insert(p.id.clone());
            }
        }
    }
    for g in &gases {
        let cap = [1, 2, 3, 4, 5, 6, 8, 10][crate::creation::rank(&c.sheets[&g.sheet_id].rank)
            .ok_or_else(|| error("Unknown Gas range rank"))?];
        let mut targets: Vec<_> = ps
            .iter()
            .filter(|p| {
                active.contains(&p.id)
                    && g.side != p.side
                    && flow(c)
                        .unwrap()
                        .ranges
                        .get(&g.id)
                        .is_none_or(|ids| ids.contains(&p.id))
            })
            .collect();
        targets.sort_by_key(|p| p.id.clone());
        targets.truncate(cap);
        for p in targets {
            if !forms::union_active(c, &p.id) && !unique(cat, &assigned(c, p))? {
                suppressed.insert(p.id.clone());
            }
        }
    }
    for p in &ps {
        let a = if suppressed.contains(&p.id) {
            String::new()
        } else {
            assigned(c, p)
        };
        if a != p.selected_ability {
            state_mut(c).history.push(format!(
                "{}: {} -> {}",
                p.name,
                p.selected_ability,
                if a.is_empty() { "negated" } else { &a }
            ));
            let contact_changed = transfers::enabled(c)
                && transfers::contact_copy(c, &p.id).is_some()
                && transfers::contact_copy(c, &p.id)
                    != before
                        .battle
                        .as_ref()
                        .and_then(|b| b.flow.as_ref())
                        .and_then(|f| f.transfers.contact_overrides.get(&p.id));
            let b = c.battle.as_mut().unwrap();
            if p.has_ability("Comatose") && a != "Comatose" {
                b.combatants
                    .iter_mut()
                    .find(|v| v.id == p.id)
                    .unwrap()
                    .runtime
                    .statuses
                    .retain(|s| s.kind != "Sleep" || s.detail != "Comatose");
            }
            b.combatants
                .iter_mut()
                .find(|v| v.id == p.id)
                .unwrap()
                .selected_ability = a;
            let f = b.flow.as_mut().unwrap();
            f.intimidate.remove(&p.id);
            f.extra.active.remove(&p.id);
            f.battlefield.known_active.remove(&p.id);
            // A contact replacement does not undo a reaction already triggered
            // by that same hit. Other obsolete events retain the existing cleanup.
            f.ability_events.retain(|e| {
                e.combatant_id != p.id
                    || e.kind == "ability-gas"
                    || (contact_changed
                        && ["contact", "retaliate", "extra-poison", "item-theft"]
                            .contains(&e.kind.as_str()))
            });
            if f.battlefield
                .weather
                .as_ref()
                .is_some_and(|w| w.source == format!("ability:{}", p.id))
            {
                f.battlefield.weather = None;
            }
        }
    }
    state_mut(c).suppressed = suppressed;
    let ps = c.battle.as_ref().unwrap().combatants.clone();
    for p in &ps {
        if active.contains(&p.id)
            && (!state(c).active.contains(&p.id)
                || before
                    .battle
                    .as_ref()
                    .and_then(|b| b.combatants.iter().find(|v| v.id == p.id))
                    .is_some_and(|v| !v.has_ability("Trace")))
            && p.has_ability("Trace")
            && !flow(c)?
                .ability_events
                .iter()
                .any(|e| e.combatant_id == p.id && e.kind == "ability-trace")
        {
            let choices: Vec<String> = ps
                .iter()
                .filter(|t| {
                    t.deployed && t.hp > 0 && t.side != p.side && !assigned(c, t).is_empty()
                })
                .filter_map(|t| match unique(cat, &assigned(c, t)) {
                    Ok(false) => Some(Ok(t.id.clone())),
                    Ok(true) => None,
                    Err(e) => Some(Err(e)),
                })
                .collect::<Result<_>>()?;
            if !choices.is_empty() {
                // Candidate IDs are frozen in target_id; only host dice choose.
                let event_id = format!("trace:{}:{}", p.id, battle(c)?.round);
                flow_mut(c).ability_events.push(abilities::AbilityEvent {
                    id: event_id,
                    combatant_id: p.id.clone(),
                    target_id: serde_json::to_string(&choices).unwrap(),
                    ability: "Trace".into(),
                    kind: "ability-trace".into(),
                    move_name: "entry".into(),
                    choices: vec!["roll".into()],
                });
            }
        }
    }
    state_mut(c).active = active;
    Ok(())
}
pub fn trace_dice(e: &abilities::AbilityEvent) -> u32 {
    let n: Vec<String> = serde_json::from_str(&e.target_id).unwrap_or_default();
    let (mut size, mut dice) = (6usize, 1);
    while size < n.len() {
        size *= 6;
        dice += 1;
    }
    dice
}
pub fn resolve_trace(
    c: &mut Campaign,
    e: &abilities::AbilityEvent,
    faces: &[u8],
    cat: &Catalog,
) -> Result<()> {
    let candidates: Vec<String> =
        serde_json::from_str(&e.target_id).map_err(|_| error("Invalid Trace candidates"))?;
    if candidates.is_empty() {
        return Err(error("Trace has no eligible foe"));
    }
    let value = faces
        .iter()
        .fold(0usize, |n, d| n * 6 + usize::from(*d - 1));
    let size = 6usize.pow(faces.len() as u32);
    if value >= size - size % candidates.len() {
        // Keep the same event for an unbiased reroll (the caller normally removes it).
        return Err(AppError::new(
            "reroll",
            "Trace random selection tied its rejection range; roll again",
        ));
    }
    let p = fighter(c, &candidates[value % candidates.len()])?.clone();
    let copied = assigned(c, &p);
    if !p.deployed || p.hp == 0 || copied.is_empty() || unique(cat, &copied)? {
        return Err(error("Trace candidate changed; GM resolution required"));
    }
    state_mut(c)
        .trace
        .insert(e.combatant_id.clone(), copied.clone());
    state_mut(c).history.push(format!(
        "Trace: {} copied {} from {}; host dice {faces:?}",
        e.combatant_id, copied, p.name
    ));
    Ok(())
}

pub fn frozen_subject(c: &Campaign, source: &str, sequence: u32, p: &Combatant) -> Combatant {
    let mut result = p.clone();
    if enabled(c) {
        if let Some(old) = abilities::context(c, source, sequence)
            .and_then(|ctx| ctx.ability_sources.iter().find(|v| v.id == p.id))
        {
            result.selected_ability = old.selected_ability.clone();
            result
                .runtime
                .ability
                .flags
                .retain(|f| !f.starts_with("as-one:"));
            result.runtime.ability.flags.extend(
                old.runtime
                    .ability
                    .flags
                    .iter()
                    .filter(|f| f.starts_with("as-one:"))
                    .cloned(),
            );
        }
    }
    result
}
pub fn gas_ruling(c: &mut Campaign, e: &abilities::AbilityEvent, actor: &str) -> Result<()> {
    gm(c, actor)?;
    state_mut(c).gas_rulings.insert(e.target_id.clone());
    state_mut(c).history.push(format!(
        "GM {actor}: opposing Gas emitters remain active, foe effects negated; {}",
        e.target_id
    ));
    Ok(())
}

pub(super) fn assign(c: &mut Campaign, id: &str, ability: &str) {
    let original = c
        .battle
        .as_ref()
        .unwrap()
        .combatants
        .iter()
        .find(|p| p.id == id)
        .unwrap()
        .selected_ability
        .clone();
    state_mut(c).originals.entry(id.into()).or_insert(original);
    state_mut(c).trace.remove(id);
    flow_mut(c)
        .transfers
        .contact_overrides
        .insert(id.into(), ability.into());
}
pub(super) fn clear_assignment(c: &mut Campaign, id: &str) {
    state_mut(c).assignments.remove(id);
    state_mut(c).trace.remove(id);
    transfers::clear_contact(c, id);
}

pub(super) fn identity(c: &Campaign, p: &Combatant) -> String {
    assigned(c, p)
}
