//! MASTER 534,553,557,560,563,577. No permanent ability mutation.
use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Inheritance {
    pub minutes: u32,
    pub copies: BTreeMap<String, InheritedAbility>,
    pub selected: BTreeMap<String, String>,
    pub history: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct InheritedAbility {
    pub id: String,
    pub sheet_id: String,
    pub donor_sheet_id: String,
    pub donor_name: String,
    pub ability: String,
    pub source: String,
    pub battle_id: String,
    pub acquired: u32,
    pub expires: u32,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct TransferState {
    pub enabled: bool,
    pub contact_overrides: BTreeMap<String, String>,
    pub seen: BTreeSet<String>,
    pub offers: BTreeMap<String, InheritanceOffer>,
    pub history: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct InheritanceOffer {
    pub recipient_sheet_id: String,
    pub source: String,
    pub donors: Vec<InheritanceDonor>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct InheritanceDonor {
    pub combatant_id: String,
    pub sheet_id: String,
    pub name: String,
    pub ability: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/bindings/")]
pub enum TransferCommand {
    Advance {
        minutes: u32,
    },
    CorrectClock {
        minutes: u32,
    },
    Select {
        sheet_id: String,
        copy_id: Option<String>,
    },
}
pub fn enabled(c: &Campaign) -> bool {
    c.battle
        .as_ref()
        .and_then(|b| b.flow.as_ref())
        .is_some_and(|f| f.transfers.enabled)
}
fn state(c: &Campaign) -> &TransferState {
    &c.battle.as_ref().unwrap().flow.as_ref().unwrap().transfers
}
fn state_mut(c: &mut Campaign) -> &mut TransferState {
    &mut flow_mut(c).transfers
}
pub fn active_copy<'a>(c: &'a Campaign, sheet: &str) -> Option<&'a InheritedAbility> {
    if !enabled(c) {
        return None;
    }
    let i = &c.ability_inheritance;
    i.selected
        .get(sheet)
        .and_then(|id| i.copies.get(id))
        .filter(|r| r.sheet_id == sheet && r.expires > i.minutes)
}
pub fn dice(c: &Campaign, command: &TransferCommand) -> Result<u32> {
    if ![ENGINE, STANDALONE_ENGINE, INFORMATION_ENGINE, FORM_ENGINE].contains(&c.engine.as_str()) {
        return Err(AppError::new(
            "version",
            "Adopt the current ability-transfer engine between battles",
        ));
    }
    if matches!(command, TransferCommand::Advance{minutes} if *minutes==0 || *minutes>525600) {
        return Err(error("Advance game time by 1–525600 minutes"));
    }
    if matches!(command,TransferCommand::CorrectClock{minutes} if *minutes>u32::MAX-1440) {
        return Err(error(
            "Corrected game time must leave room for a 24-hour copy",
        ));
    }
    Ok(0)
}
pub fn command(
    c: &mut Campaign,
    actor: &str,
    command: &TransferCommand,
    r: &str,
    cat: &Catalog,
) -> Result<String> {
    dice(c, command)?;
    reason(r)?;
    let active = c.battle.as_ref().is_some_and(|b| b.active);
    match command {
        TransferCommand::Advance { minutes } | TransferCommand::CorrectClock { minutes } => {
            gm(c, actor)?;
            if active {
                return Err(error("Advance the ability clock between battles"));
            }
            c.ability_inheritance.minutes =
                if matches!(command, TransferCommand::CorrectClock { .. }) {
                    *minutes
                } else {
                    c.ability_inheritance
                        .minutes
                        .checked_add(*minutes)
                        .filter(|m| *m <= u32::MAX - 1440)
                        .ok_or_else(|| error("Ability clock exhausted"))?
                };
            let live: BTreeSet<_> = c
                .ability_inheritance
                .copies
                .values()
                .filter(|r| r.expires > c.ability_inheritance.minutes)
                .map(|r| r.id.clone())
                .collect();
            c.ability_inheritance
                .selected
                .retain(|_, id| live.contains(id));
        }
        TransferCommand::Select { sheet_id, copy_id } => {
            let s = c
                .sheets
                .get(sheet_id)
                .ok_or_else(|| error("Missing recipient sheet"))?;
            if s.owner_id != actor && c.gm_id != actor {
                return Err(error(
                    "Only the owner or GM can select an inherited ability",
                ));
            }
            if active {
                gm(c, actor)?;
                if !flow(c)?.queue.is_empty()
                    || abilities::pending(c)
                    || field::pending(c)
                    || special::ticks_pending(c)
                    || !flow(c)?.tactics.switches.is_empty()
                {
                    return Err(error(
                        "Resolve pending actions and choices before a GM ability-selection ruling",
                    ));
                }
            }
            if let Some(id) = copy_id {
                let record = c
                    .ability_inheritance
                    .copies
                    .get(id)
                    .ok_or_else(|| error("Unknown inherited ability"))?;
                if record.sheet_id != *sheet_id || record.expires <= c.ability_inheritance.minutes {
                    return Err(error(
                        "Inherited ability belongs to another sheet or has expired",
                    ));
                }
                if ability_changes::unique(cat, &record.ability)? {
                    return Err(AppError::new(
                        "manual_rule",
                        "Unique abilities cannot be inherited",
                    ));
                }
                c.ability_inheritance
                    .selected
                    .insert(sheet_id.clone(), id.clone());
            } else {
                c.ability_inheritance.selected.remove(sheet_id);
            }
            if active {
                let ids: Vec<_> = battle(c)?
                    .combatants
                    .iter()
                    .filter(|p| p.sheet_id == *sheet_id)
                    .map(|p| p.id.clone())
                    .collect();
                for id in ids {
                    ability_changes::clear_assignment(c, &id);
                }
            }
        }
    }
    let log = format!(
        "Ability inheritance: {actor} {command:?}, campaign minute {}. {r}",
        c.ability_inheritance.minutes
    );
    c.ability_inheritance.history.push(log.clone());
    Ok(log)
}
pub fn hit(c: &mut Campaign, q: &QueuedAction, m: &serde_json::Value, cat: &Catalog) -> Result<()> {
    if !enabled(c) || !abilities::contact(m) || decoys::intercepted(c, q) {
        return Ok(());
    }
    let source = fighter(c, &q.actor_id)?.clone();
    let target = fighter(c, &q.target_id)?.clone();
    if source.side == target.side || source.has_ability("Long Reach") {
        return Ok(());
    }
    let (_, immune) =
        rules::type_matchup(c, &source, &target, m["Type"].as_str().unwrap_or(""), cat)?;
    if immune || abilities::damage_immune_from(c, &source, &target, m) {
        return Ok(());
    }
    let key = format!(
        "hit:{}:{}:{}:{}:{}",
        battle(c)?.id,
        q.sequence,
        q.actor_id,
        q.target_id,
        q.special_index
    );
    if !state_mut(c).seen.insert(key) {
        return Ok(());
    }
    // Read both pre-contact abilities once. No newly acquired ability chains on
    // this same hit; two independent contamination directions remain simultaneous.
    let mut assignments = BTreeMap::new();
    if source.has_any_ability(&["Mummy", "Lingering Aroma"])
        && !forms::union_active(c, &target.id)
        && !ability_changes::unique(cat, &target.selected_ability)?
    {
        assignments.insert(
            target.id.clone(),
            source.ability_for(&["Mummy", "Lingering Aroma"]).into(),
        );
    }
    if target.has_any_ability(&["Mummy", "Lingering Aroma"])
        && !forms::union_active(c, &source.id)
        && !ability_changes::unique(cat, &source.selected_ability)?
    {
        assignments.insert(
            source.id.clone(),
            target.ability_for(&["Mummy", "Lingering Aroma"]).into(),
        );
    }
    if source.has_ability("Wandering Spirit")
        && !forms::union_active(c, &target.id)
        && !ability_changes::unique(cat, &target.selected_ability)?
    {
        assignments.insert(source.id.clone(), ability_changes::identity(c, &target));
        assignments.insert(target.id.clone(), source.selected_ability.clone());
    }
    for (id, ability) in assignments {
        let p = fighter(c, &id)?.clone();
        if p.selected_ability != ability {
            ability_changes::assign(c, &id, &ability);
            state_mut(c).history.push(format!(
                "{}: {} -> {ability}; contact {} -> {} ({})",
                p.name, p.selected_ability, source.name, target.name, q.move_name
            ));
        }
    }
    Ok(())
}
pub fn lifecycle(before: &Campaign, c: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let ps = c.battle.as_ref().unwrap().combatants.clone();
    let ended = !c.battle.as_ref().unwrap().active;
    for p in &ps {
        if (ended || !p.deployed || p.hp == 0)
            && state_mut(c).contact_overrides.remove(&p.id).is_some()
        {
            state_mut(c)
                .history
                .push(format!("{}: contact ability ended on removal", p.name));
        }
    }
    if ended {
        return Ok(());
    }
    flow_mut(c).ability_events.retain(|e| {
        e.kind != "transfer-inherit"
            || ps.iter().any(|p| {
                p.id == e.combatant_id && p.deployed && p.hp > 0 && p.has_ability(&e.ability)
            })
    });
    let old = before
        .battle
        .as_ref()
        .filter(|b| b.id == c.battle.as_ref().unwrap().id && b.active);
    let mut donors = vec![];
    let mut reviews = vec![];
    for p in ps.iter().filter(|p| p.deployed && p.hp == 0) {
        if let Some(was) = old.and_then(|b| {
            b.combatants
                .iter()
                .find(|q| q.id == p.id && q.deployed && q.hp > 0)
        }) {
            let key = format!("faint:{}:{}:{}", battle(c)?.id, before.revision + 1, p.id);
            if state_mut(c).seen.insert(key) {
                let ability = if ability_changes::enabled(before) {
                    ability_changes::identity(before, was)
                } else {
                    was.selected_ability.clone()
                };
                let unique = if ability.is_empty() {
                    true
                } else {
                    match ability_changes::unique(cat, &ability) {
                        Ok(value) => value,
                        Err(e) => {
                            reviews.push((
                                p.side.clone(),
                                p.id.clone(),
                                format!("{} / {}: {}", p.name, ability, e.message),
                            ));
                            true
                        }
                    }
                };
                if !unique {
                    donors.push((
                        p.side.clone(),
                        InheritanceDonor {
                            combatant_id: p.id.clone(),
                            sheet_id: p.sheet_id.clone(),
                            name: p.name.clone(),
                            ability,
                        },
                    ));
                }
            }
        }
    }
    donors.sort_by(|a, b| a.1.combatant_id.cmp(&b.1.combatant_id));
    for p in ps
        .iter()
        .filter(|p| p.deployed && p.hp > 0 && p.has_any_ability(&["Receiver", "Power of Alchemy"]))
    {
        if p.has_ability("Receiver")
            && c.ability_inheritance
                .copies
                .values()
                .any(|r| r.sheet_id == p.sheet_id && r.expires > c.ability_inheritance.minutes)
        {
            continue;
        }
        for (_, donor, note) in reviews.iter().filter(|(side, _, _)| *side == p.side) {
            let id = format!(
                "inherit-review:{}:{}:{}:{}",
                battle(c)?.id,
                before.revision + 1,
                p.id,
                donor
            );
            flow_mut(c).ability_events.push(abilities::AbilityEvent {
                id,
                combatant_id: p.id.clone(),
                target_id: donor.clone(),
                ability: p.selected_ability.clone(),
                kind: "transfer-review".into(),
                move_name: note.clone(),
                choices: vec!["manual".into()],
            });
            state_mut(c)
                .history
                .push(format!("Inheritance requires GM metadata review: {note}"));
        }
        let candidates: Vec<_> = donors
            .iter()
            .filter(|(side, _)| *side == p.side)
            .map(|(_, d)| d.clone())
            .collect();
        if candidates.is_empty() {
            continue;
        }
        let id = format!("inherit:{}:{}:{}", battle(c)?.id, before.revision + 1, p.id);
        let mut choices: Vec<_> = (0..candidates.len())
            .map(|i| format!("activate:{i}"))
            .collect();
        if p.has_ability("Power of Alchemy") {
            choices.push("store".into());
        }
        let event = abilities::AbilityEvent {
            id: id.clone(),
            combatant_id: p.id.clone(),
            target_id: candidates[0].combatant_id.clone(),
            ability: p.selected_ability.clone(),
            kind: "transfer-inherit".into(),
            move_name: "copy fainted ally ability for 24 game hours".into(),
            choices,
        };
        state_mut(c).offers.insert(
            id,
            InheritanceOffer {
                recipient_sheet_id: p.sheet_id.clone(),
                source: p.selected_ability.clone(),
                donors: candidates,
            },
        );
        flow_mut(c).ability_events.push(event);
    }
    Ok(())
}
pub fn resolve(
    c: &mut Campaign,
    event: &abilities::AbilityEvent,
    choice: &str,
    cat: &Catalog,
) -> Result<()> {
    let recipient = fighter(c, &event.combatant_id)?;
    if !recipient.deployed || recipient.hp == 0 || !recipient.has_ability(&event.ability) {
        return Err(error("Inheritance recipient changed"));
    }
    let offer = state(c)
        .offers
        .get(&event.id)
        .cloned()
        .ok_or_else(|| error("Inheritance offer changed"))?;
    let selected = choice
        .strip_prefix("activate:")
        .map(|s| {
            s.parse::<usize>()
                .map_err(|_| error("Unknown inheritance choice"))
        })
        .transpose()?;
    if selected.is_some_and(|i| i >= offer.donors.len())
        || (selected.is_none() && choice != "store")
    {
        return Err(error("Choose a recorded inheritance option"));
    }
    if offer.source == "Receiver"
        && (selected.is_none()
            || c.ability_inheritance.copies.values().any(|r| {
                r.sheet_id == offer.recipient_sheet_id && r.expires > c.ability_inheritance.minutes
            }))
    {
        return Err(error("Receiver supports only one unexpired copied ability"));
    }
    let mut activate = None;
    for (i, d) in offer.donors.iter().enumerate() {
        if offer.source == "Receiver" && selected != Some(i) {
            continue;
        }
        if ability_changes::unique(cat, &d.ability)? {
            return Err(AppError::new(
                "manual_rule",
                "Unique ability cannot be inherited",
            ));
        }
        let record = InheritedAbility {
            id: format!("{}:copy:{i}", event.id),
            sheet_id: offer.recipient_sheet_id.clone(),
            donor_sheet_id: d.sheet_id.clone(),
            donor_name: d.name.clone(),
            ability: d.ability.clone(),
            source: offer.source.clone(),
            battle_id: battle(c)?.id.clone(),
            acquired: c.ability_inheritance.minutes,
            expires: c
                .ability_inheritance
                .minutes
                .checked_add(1440)
                .ok_or_else(|| error("Ability clock exhausted"))?,
        };
        if selected == Some(i) {
            activate = Some(record.id.clone());
        }
        c.ability_inheritance.history.push(format!(
            "{}: {} copied {} from {} until minute {}",
            offer.source, offer.recipient_sheet_id, d.ability, d.name, record.expires
        ));
        c.ability_inheritance
            .copies
            .entry(record.id.clone())
            .or_insert(record);
    }
    if let Some(id) = activate {
        c.ability_inheritance
            .selected
            .insert(offer.recipient_sheet_id, id);
    }
    Ok(())
}

pub(super) fn contact_copy<'a>(c: &'a Campaign, id: &str) -> Option<&'a String> {
    if enabled(c) {
        state(c).contact_overrides.get(id)
    } else {
        None
    }
}
pub(super) fn clear_contact(c: &mut Campaign, id: &str) {
    if enabled(c) {
        state_mut(c).contact_overrides.remove(id);
    }
}
