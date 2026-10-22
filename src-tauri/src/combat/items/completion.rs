//! RULING 73,118-120,127-129: remaining held effects and medicine.
use super::*;
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct CompletionState {
    pub enabled: bool,
    pub choices: BTreeMap<String, String>,
    pub rulings: BTreeMap<String, String>,
    pub used: BTreeSet<String>,
    pub entries: BTreeSet<String>,
    pub frozen: BTreeMap<String, Vec<HeldItem>>,
    pub payers: BTreeSet<String>,
    pub species: BTreeMap<String, String>,
    pub recycled: BTreeMap<String, BTreeMap<String, u32>>,
    pub recycle_recipients: BTreeMap<String, String>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct MedicalRecord {
    pub healing_until: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "operation", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/bindings/")]
pub enum CompletionCommand {
    RecycleRecipient {
        combatant_id: String,
        target_id: String,
    },
    Select {
        combatant_id: String,
        unit_id: String,
        value: String,
    },
    Rule {
        key: String,
        value: String,
    },
    Medicine {
        combatant_id: String,
        target_id: String,
        supply_sheet_id: String,
        unit_id: Option<String>,
        name: String,
        dose: u32,
        trainer_on_field: bool,
    },
}
fn state(c: &Campaign) -> &CompletionState {
    &super::state(c).completion
}
fn state_mut(c: &mut Campaign) -> &mut CompletionState {
    &mut super::state_mut(c).completion
}
pub fn enabled(c: &Campaign) -> bool {
    super::enabled(c) && state(c).enabled
}
fn units(c: &Campaign, id: &str) -> Vec<HeldItem> {
    if !enabled(c) {
        return vec![];
    }
    if let Some(q) = flow(c)
        .ok()
        .and_then(|f| f.queue.first())
        .filter(|q| q.actor_id == id)
    {
        if let Some(units) = state(c).frozen.get(&format!("{id}:{}", q.sequence)) {
            return units.clone();
        }
    }
    if !active_effects(c, id).unwrap_or(false) {
        return vec![];
    }
    super::held(c, id)
}
fn reuse_active(c: &Campaign, id: &str, name: &str) -> bool {
    enabled(c)
        && fighter(c, id).is_ok_and(|p| !field::has(c, "Magic Room", p))
        && state(c)
            .recycled
            .get(id)
            .and_then(|v| v.get(name))
            .is_some_and(|n| *n > 0)
}
pub fn has(c: &Campaign, id: &str, name: &str) -> bool {
    units(c, id).iter().any(|u| u.name == name) || reuse_active(c, id, name)
}
pub fn live(c: &Campaign, id: &str, name: &str) -> bool {
    (enabled(c) && super::has_effect(c, id, name)) || reuse_active(c, id, name)
}
pub fn ring(c: &Campaign, p: &Combatant, wide: bool) -> bool {
    live(c, &p.id, "Ring Target")
        && state(c)
            .rulings
            .get(&format!("ring:{}", p.id))
            .is_some_and(|v| v == "reviewed" || (!wide && v == "types"))
}
pub fn healing_locked(c: &Campaign, id: &str) -> bool {
    special::registered(c, id)
        .ok()
        .and_then(|p| c.sheets.get(&p.sheet_id))
        .and_then(|s| s.medical.as_ref())
        .is_some_and(|m| m.healing_until > c.ability_inheritance.minutes)
}
pub fn capture(c: &mut Campaign, q: &QueuedAction) {
    if !enabled(c) {
        return;
    }
    let key = format!("{}:{}", q.actor_id, q.sequence);
    if !state(c).frozen.contains_key(&key) {
        let saved = if active_effects(c, &q.actor_id).unwrap_or(false) {
            super::held(c, &q.actor_id)
        } else {
            vec![]
        };
        state_mut(c).frozen.insert(key, saved);
    }
}
pub fn species(c: &Campaign, p: &Combatant) -> String {
    let mut sheet = c.sheets[&p.sheet_id].clone();
    forms::sheet(c, p, &mut sheet);
    sheet
        .species_id
        .as_ref()
        .and_then(|id| state(c).species.get(id))
        .cloned()
        .unwrap_or_default()
}
pub fn initialize(c: &mut Campaign, cat: &Catalog) {
    if !enabled(c) {
        return;
    }
    for name in [
        "Pikachu",
        "Farfetch'd",
        "Farfetch'd (Galarian Form)",
        "Cubone",
        "Marowak",
        "Marowak (Alolan Form)",
        "Chansey",
    ] {
        if let Some(v) = cat.find("species", "Name", name) {
            if let Some(id) = v["AppId"].as_str() {
                state_mut(c)
                    .species
                    .insert(id.into(), name.split(" (").next().unwrap().into());
            }
        }
    }
}
pub fn stat(c: &Campaign, p: &Combatant, key: &str) -> i32 {
    if !enabled(c) {
        return 0;
    }
    let species = species(c, p);
    units(c, &p.id)
        .iter()
        .map(|u| match u.name.as_str() {
            "Light Ball" if species == "Pikachu" && ["Strength", "Special"].contains(&key) => 1,
            "Thick Club"
                if ["Cubone", "Marowak"].contains(&species.as_str()) && key == "Strength" =>
            {
                2
            }
            "Lucky Punch" if species == "Chansey" && key == "Strength" => 2,
            "Power Increasers" if key == "Dexterity" => -1,
            "Power Increasers" if state(c).choices.get(&u.id).is_some_and(|k| k == key) => 2,
            _ => 0,
        })
        .sum()
}
pub fn high_critical(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> bool {
    if m["Category"] == "Support" {
        return false;
    }
    has(c, &p.id, "Razor Claw")
        || (has(c, &p.id, "Leek") && species(c, p) == "Farfetch'd")
        || (has(c, &p.id, "Lucky Punch")
            && species(c, p) == "Chansey"
            && m["Category"] == "Physical")
}
pub fn loaded(c: &Campaign, id: &str) -> u32 {
    2 * u32::from(has(c, id, "Loaded Dice"))
}
pub fn guard(c: &Campaign, id: &str, m: &serde_json::Value) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    for p in battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
    {
        if live(c, &p.id, "Ring Target")
            && !state(c).rulings.contains_key(&format!("ring:{}", p.id))
        {
            return Err(AppError::new(
                "manual_rule",
                "GM must record Ring Target's immunity scope before automated actions",
            ));
        }
    }
    for u in units(c, id) {
        if [
            "Choice Band",
            "Choice Specs",
            "Choice Scarf",
            "Power Increasers",
        ]
        .contains(&u.name.as_str())
            && !state(c).choices.contains_key(&u.id)
        {
            return Err(error("Select this held item's move or trait before acting"));
        }
        if u.name == "Choice Scarf"
            && state(c).choices.get(&u.id).is_some_and(|n| m["Name"] == *n)
            && !m["Attributes"]["LateReaction"].is_null()
            && !state(c).rulings.contains_key(&format!("scarf:{}", u.id))
        {
            return Err(AppError::new("manual_rule","Choice Scarf Reaction 5 conflicts with printed Late Reaction; record a GM timing ruling"));
        }
    }
    Ok(())
}
pub fn move_data(c: &Campaign, p: &Combatant, m: &mut serde_json::Value) {
    for u in units(c, &p.id) {
        if u.name == "Choice Scarf" && state(c).choices.get(&u.id).is_some_and(|n| m["Name"] == *n)
        {
            let mode = state(c)
                .rulings
                .get(&format!("scarf:{}", u.id))
                .map(String::as_str);
            if !m["Attributes"]["LateReaction"].is_null() && mode != Some("item") {
                continue;
            }
            m["Attributes"]["Reaction"] = serde_json::json!("5");
            if mode == Some("item") {
                m["Attributes"]
                    .as_object_mut()
                    .unwrap()
                    .remove("LateReaction");
            }
        }
    }
}
pub fn damage_bonus(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> i32 {
    if m["Category"] == "Support" {
        return 0;
    }
    let name = m["Name"].as_str().unwrap_or("");
    let mut n = 0;
    for u in units(c, &p.id) {
        if ["Choice Band", "Choice Specs", "Choice Scarf"].contains(&u.name.as_str()) {
            if let Some(chosen) = state(c).choices.get(&u.id) {
                n += if chosen == name {
                    if u.name == "Choice Scarf" {
                        0
                    } else {
                        3
                    }
                } else {
                    -3
                };
            }
        }
        if u.name == "Metronome"
            && (special::limit(m) > 1
                || m["Attributes"]["SuccessiveActions"] == true
                || ["Rollout", "Ice Ball", "Fury Cutter", "Echoed Voice"].contains(&name))
        {
            n += 1;
        }
    }
    n
}
pub fn consume(c: &mut Campaign, id: &str, name: &str, cat: &Catalog) -> Result<bool> {
    if reuse_active(c, id, name) {
        *state_mut(c)
            .recycled
            .get_mut(id)
            .unwrap()
            .get_mut(name)
            .unwrap() -= 1;
        return Ok(true);
    }
    if !live(c, id, name) {
        return Ok(false);
    }
    let unit = super::held(c, id)
        .into_iter()
        .find(|u| u.name == name)
        .unwrap();
    super::state_mut(c).units.get_mut(&unit.id).unwrap().state = "consumed".into();
    record(c, &unit.id, id, None, "consume", name);
    after_loss(c, id, cat)?;
    Ok(true)
}
pub fn power_herb(c: &mut Campaign, id: &str, cat: &Catalog) -> Result<bool> {
    consume(c, id, "Power Herb", cat)
}
pub fn pads(c: &Campaign, id: &str) -> bool {
    has(c, id, "Protective Pads")
}
pub fn added_immune(c: &Campaign, id: &str, m: &serde_json::Value) -> bool {
    has(c, id, "Safety Goggles")
        && ["Spore", "Powder", "Pollen"]
            .iter()
            .any(|w| m["Name"].as_str().unwrap_or("").contains(w))
}
pub fn survive(c: &mut Campaign, q: &QueuedAction, before: i32, damage: i32) -> i32 {
    if reviewed::source(c, q).is_ok_and(|source| {
        reviewed::extreme(c, &source)
            && fighter(c, &q.target_id).is_ok_and(|target| target.side != source.side)
    }) {
        return damage;
    }
    if before > 0 && damage >= before && live(c, &q.target_id, "Focus Sash") {
        if let Some(unit) = super::held(c, &q.target_id)
            .iter()
            .find(|u| u.name == "Focus Sash" && !state(c).used.contains(&format!("sash:{}", u.id)))
            .cloned()
        {
            state_mut(c).used.insert(format!("sash:{}", unit.id));
            record(
                c,
                &unit.id,
                &q.target_id,
                None,
                "survive",
                "Focus Sash's first move-fainting prevention",
            );
            return before - 1;
        }
    }
    damage
}
fn hurt(c: &mut Campaign, id: &str, n: i32) -> Result<()> {
    let p = fighter(c, id)?;
    if p.has_ability("Wonder Guard") || (p.has_ability("Magic Guard") && !ring(c, p, true)) {
        return Ok(());
    }
    let p = fighter_mut(c, id)?;
    p.hp = (p.hp - n).max(0);
    Ok(())
}
pub fn hit(c: &mut Campaign, q: &QueuedAction, m: &serde_json::Value, cat: &Catalog) -> Result<()> {
    if !enabled(c) || q.actor_id == q.target_id {
        return Ok(());
    }
    let contact = abilities::contact(m)
        && !pads(c, &q.actor_id)
        && !fighter(c, &q.actor_id)?.has_ability("Long Reach");
    if contact && live(c, &q.target_id, "Rocky Helmet") {
        hurt(c, &q.actor_id, 1)?;
    }
    if contact && live(c, &q.target_id, "Sticky Barb") && fighter(c, &q.actor_id)?.hp > 0 {
        let unit = super::held(c, &q.target_id)
            .into_iter()
            .find(|u| u.name == "Sticky Barb")
            .unwrap();
        if (!fighter(c, &q.target_id)?.has_ability("Sticky Hold")
            || reviewed::extreme(c, &reviewed::source(c, q)?))
            && can_receive(c, &q.actor_id, &unit, cat)?
        {
            let slot = super::held(c, &q.actor_id).len() as u32;
            let item = super::state_mut(c).units.get_mut(&unit.id).unwrap();
            item.holder = q.actor_id.clone();
            item.slot = slot;
            record(
                c,
                &unit.id,
                &q.target_id,
                Some(q.actor_id.clone()),
                "contact_transfer",
                "Sticky Barb",
            );
            after_loss(c, &q.target_id, cat)?;
        }
    }
    let received = flow(c)?
        .queue
        .iter()
        .find(|v| v.sequence == q.sequence && v.actor_id == q.actor_id)
        .map_or(q.damage, |v| v.damage);
    if received <= 0 {
        return Ok(());
    }
    consume(c, &q.target_id, "Air Balloon", cat)?;
    let source = reviewed::source(c, q)?;
    let target = fighter(c, &q.target_id)?.clone();
    let (effect, immune) =
        rules::type_matchup(c, &source, &target, m["Type"].as_str().unwrap_or(""), cat)?;
    if effect > 0 && !immune && live(c, &q.target_id, "Weakness Policy") {
        extra::boost(c, &q.target_id, "Strength", 1)?;
        extra::boost(c, &q.target_id, "Special", 1)?;
    }
    for (item, out, forced) in [
        ("Eject Button", q.target_id.clone(), false),
        ("Red Card", q.actor_id.clone(), true),
    ] {
        let unit_id = super::held(c, &q.target_id)
            .into_iter()
            .find(|u| u.name == item)
            .map(|u| u.id)
            .unwrap_or_default();
        let key = format!("switch:{unit_id}:{}", battle(c)?.round);
        if live(c, &q.target_id, item) && !state(c).used.contains(&key) && fighter(c, &out)?.hp > 0
        {
            if forced
                && (fighter(c, &out)?.has_any_ability(&["Guard Dog", "Suction Cups"])
                    || forms::blocks_recall(c, &out))
            {
                continue;
            }
            if !forced && (extra::blocked(c, fighter(c, &out)?, cat) || special::blocked(c, &out)) {
                continue;
            }
            state_mut(c).used.insert(key);
            tactics::item_switch(c, q, &out, item, forced)?;
        }
    }
    Ok(())
}
pub fn used_move(c: &mut Campaign, id: &str, m: &serde_json::Value) -> Result<()> {
    if m["Attributes"]["SoundMove"] == true && has(c, id, "Throat Spray") {
        extra::boost(c, id, "Special", 1)?;
    }
    Ok(())
}
pub fn lifecycle(before: &Campaign, c: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !enabled(c) || !battle(c).is_ok_and(|b| b.active) {
        return Ok(());
    }
    let active: BTreeSet<_> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    let damaged: Vec<_> = battle(c)?
        .combatants
        .iter()
        .filter(|p| {
            p.deployed
                && before.battle.as_ref().is_some_and(|b| {
                    b.combatants
                        .iter()
                        .any(|old| old.id == p.id && p.hp < old.hp)
                })
        })
        .map(|p| p.id.clone())
        .collect();
    for id in damaged {
        consume(c, &id, "Air Balloon", cat)?;
    }
    for id in &active {
        if !state(c).entries.contains(id) {
            for (item, kind) in [("Flame Orb", "Burn"), ("Toxic Orb", "Poison")] {
                if live(c, id, item) {
                    match set_status_from(c, id, kind, "", None, cat, Some(id)) {
                        Err(e)
                            if [
                                "Species type is immune to this condition",
                                "Selected ability is immune to this condition",
                                "Battlefield prevents this condition",
                            ]
                            .contains(&e.message.as_str()) => {}
                        other => other?,
                    }
                }
            }
        }
        if live(c, id, "White Herb") {
            let p = fighter(c, id)?.clone();
            let contrary = p.has_ability("Contrary");
            let lowered = p
                .runtime
                .modifiers
                .values()
                .any(|n| if contrary { *n > 0 } else { *n < 0 })
                || p.runtime
                    .ability
                    .counters
                    .iter()
                    .any(|(k, n)| k.starts_with("debuff:") && *n < 0)
                || flow(c)?
                    .stat_channels
                    .grants
                    .iter()
                    .any(|g| g.target == *id && g.negative < 0)
                || flow(c)?.intimidate.values().any(|v| v.contains(id));
            if lowered && consume(c, id, "White Herb", cat)? {
                restore_lowered(c, id)?;
            }
        }
    }
    state_mut(c).entries = active;
    Ok(())
}
pub fn end_round(c: &mut Campaign) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let ids: Vec<_> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    for id in ids {
        if live(c, &id, "Sticky Barb") {
            hurt(c, &id, 1)?;
        }
    }
    Ok(())
}
pub fn command(
    c: &mut Campaign,
    actor: &str,
    cmd: &CompletionCommand,
    why: &str,
    cat: &Catalog,
) -> Result<String> {
    if !enabled(c) {
        return Err(error("Upgrade remaining item rules between battles"));
    }
    match cmd {
        CompletionCommand::RecycleRecipient {
            combatant_id,
            target_id,
        } => {
            owner(c, actor, combatant_id)?;
            if !flow(c)?.queue.is_empty() {
                return Err(error("Select a Recycle recipient before the action chain"));
            }
            let p = fighter(c, combatant_id)?;
            let t = special::registered(c, target_id)?;
            if p.side != t.side || p.owner_id != t.owner_id {
                return Err(error(
                    "Choose an owned registered ally for the reused medicine",
                ));
            }
            state_mut(c)
                .recycle_recipients
                .insert(combatant_id.clone(), target_id.clone());
        }
        CompletionCommand::Select {
            combatant_id,
            unit_id,
            value,
        } => {
            owner(c, actor, combatant_id)?;
            if !flow(c)?.queue.is_empty() {
                return Err(error("Choose item effects between action chains"));
            }
            let unit = super::held(c, combatant_id)
                .into_iter()
                .find(|u| u.id == *unit_id)
                .ok_or_else(|| error("Select an owned held unit"))?;
            if state(c).choices.contains_key(unit_id) && actor != c.gm_id {
                return Err(error(
                    "Changing an encounter item choice requires a GM timing ruling",
                ));
            }
            if unit.name == "Power Increasers" {
                if ![
                    "Strength",
                    "Vitality",
                    "Special",
                    "Insight",
                    "Defense",
                    "SpecialDefense",
                    "AccuracyPool",
                    "EvasionPool",
                    "Initiative",
                ]
                .contains(&value.as_str())
                {
                    return Err(error("Choose a supported other attribute or numeric trait"));
                }
            } else if ["Choice Band", "Choice Scarf", "Choice Specs"].contains(&unit.name.as_str())
            {
                if !rules::effective_sheet(c, fighter(c, combatant_id)?)?
                    .moves
                    .contains(value)
                {
                    return Err(error("Choose a currently learned move"));
                }
                let m = move_entry(cat, value)?;
                if (unit.name == "Choice Band" && m["Category"] != "Physical")
                    || (unit.name == "Choice Specs" && m["Category"] != "Special")
                {
                    return Err(error("Choice item requires its printed move category"));
                }
            } else {
                return Err(error("This item has no selection"));
            }
            state_mut(c).choices.insert(unit_id.clone(), value.clone());
        }
        CompletionCommand::Rule { key, value } => {
            gm(c, actor)?;
            if !flow(c)?.queue.is_empty() {
                return Err(error("Record item rulings between action chains"));
            }
            let valid = if key == "recycle-balloon" {
                value == "until-damage"
            } else if key == "revive" {
                ["full", "capped"].contains(&value.as_str())
            } else if let Some(id) = key.strip_prefix("pocket:") {
                c.sheets.contains_key(id) && value == "accessible"
            } else if let Some(id) = key.strip_prefix("ring:") {
                fighter(c, id)?;
                ["types", "reviewed"].contains(&value.as_str())
            } else if let Some(id) = key.strip_prefix("herbal:") {
                special::registered(c, id)?;
                value == "accepted"
            } else if let Some(id) = key.strip_prefix("scarf:") {
                super::state(c).units.contains_key(id)
                    && ["printed", "item"].contains(&value.as_str())
            } else {
                false
            };
            if !valid {
                return Err(error("Unknown printed item ruling or value"));
            }
            state_mut(c).rulings.insert(key.clone(), value.clone());
        }
        CompletionCommand::Medicine { .. } => medicine(c, actor, cmd, cat)?,
    }
    Ok(format!("Reviewed item decision recorded: {why}"))
}
// Ordinary medicine uses the same tracked physical consumable as later Recycle.
fn medicine(c: &mut Campaign, actor: &str, cmd: &CompletionCommand, cat: &Catalog) -> Result<()> {
    let CompletionCommand::Medicine {
        combatant_id,
        target_id,
        supply_sheet_id,
        unit_id,
        name,
        dose,
        trainer_on_field,
    } = cmd
    else {
        unreachable!()
    };
    owner(c, actor, combatant_id)?;
    if !*trainer_on_field {
        ready(c, combatant_id)?;
    }
    turn(c, combatant_id)?;
    if *dose == 0 || *dose > 99 {
        return Err(error("Choose 1-99 medicine units"));
    }
    let payer = fighter(c, combatant_id)?.clone();
    let target = special::registered(c, target_id)?.clone();
    if target.deployed
        && target_id != combatant_id
        && flow(c)?
            .ranges
            .get(combatant_id)
            .is_some_and(|ids| !ids.contains(target_id))
    {
        return Err(error("The medicine recipient is outside confirmed range"));
    }
    if payer.side != target.side || (actor != c.gm_id && payer.owner_id != target.owner_id) {
        return Err(error(
            "Medicine requires an owned allied recipient or a GM ruling",
        ));
    }
    let supply = c
        .sheets
        .get(supply_sheet_id)
        .ok_or_else(|| error("Medicine supply sheet is missing"))?;
    let trainer = c.sheets[&payer.sheet_id].trainer_id.as_ref();
    if supply.owner_id != payer.owner_id
        || (supply.id != payer.sheet_id && trainer != Some(&supply.id))
    {
        return Err(error(
            "Use the acting Pokemon's or linked trainer's supplies",
        ));
    }
    if actor != c.gm_id
        && state(c)
            .rulings
            .get(&format!("pocket:{supply_sheet_id}"))
            .map(String::as_str)
            != Some("accessible")
    {
        return Err(error(
            "GM must confirm the supplies are accessible in the small battle pocket",
        ));
    }
    if *trainer_on_field {
        gm(c, actor)?;
        if supply.kind != "trainer" {
            return Err(error(
                "Only a GM-confirmed trainer on the battlefield can pay this cost",
            ));
        }
        if battle(c)?
            .trainer_actions
            .get(supply_sheet_id)
            .copied()
            .unwrap_or(0)
            >= 5
        {
            return Err(error("Trainer has no actions remaining"));
        }
    }
    let item = cat
        .find("items", "Name", name)
        .ok_or_else(|| error("Unknown medicine"))?;
    if !medicine_names().contains(&name.as_str()) {
        return Err(error("This is not a reviewed in-battle medicine"));
    }
    if field::has(c, "Magic Room", &payer) || field::has(c, "Magic Room", &target) {
        return Err(error("Magic Room prevents using medicine"));
    }
    let herbal = ["Energy Root", "Remedy", "Heal Powder", "Revival Herb"].contains(&name.as_str());
    if herbal
        && !target.has_ability("Gluttony")
        && state(c)
            .rulings
            .get(&format!("herbal:{target_id}"))
            .map(String::as_str)
            != Some("accepted")
    {
        return Err(AppError::new("manual_rule","GM must record herbal acceptance and the unspecified happiness consequences before use"));
    }
    let revived = item["Cures"] == "Fainted";
    if (revived && target.hp > 0) || (!revived && target.hp == 0) {
        return Err(error(
            "Only revival medicines restore consciousness; use them on a fainted recipient",
        ));
    }
    let max = ["Max Potion", "Full Restore"].contains(&name.as_str());
    let full_revive = ["Max Revive", "Revival Herb"].contains(&name.as_str());
    let full = if full_revive {
        match state(c).rulings.get("revive").map(String::as_str) {Some("full")=>true,Some("capped")=>false,_=>return Err(AppError::new("manual_rule","Full-HP revival wording conflicts with the listed three-HP exceptions; record full or capped healing"))}
    } else {
        max
    };
    let heal = item["HealthRestored"].as_u64().unwrap_or(0) as u32;
    let multiple = [
        "Potion",
        "Super Potion",
        "Hyper Potion",
        "Energy Root",
        "Remedy",
    ]
    .contains(&name.as_str());
    if !multiple && *dose != 1 {
        return Err(error("This medicine is single-use, not a dosed bottle"));
    }
    if heal > 0 && (healing_locked(c, target_id) || special::healing_blocked(c, target_id)) {
        return Err(error(
            "The recipient cannot recover HP until the healing restriction expires",
        ));
    }
    let amount = if full {
        recoverable(c, &target)
    } else {
        (if multiple { *dose as i32 } else { heal as i32 })
            .min((3 - target.runtime.healed).max(0))
            .min(recoverable(c, &target))
    };
    if heal > 0 && amount <= 0 {
        return Err(error(
            "No HP can be recovered within this round's healing allowance",
        ));
    }
    if multiple && amount != *dose as i32 {
        return Err(error(
            "Choose a dose within missing HP and the shared three-HP allowance",
        ));
    }
    let mut unit = if let Some(unit_id) = unit_id {
        supply
            .stored_items
            .as_ref()
            .and_then(|v| v.iter().find(|u| u.id == *unit_id && u.name == *name))
            .cloned()
            .ok_or_else(|| error("Select a stored partial medicine unit"))?
    } else {
        if supply.inventory.get(name).copied().unwrap_or(0) <= 0 {
            return Err(error("Medicine is not present in the owner's supplies"));
        }
        HeldItem {
            id: format!(
                "medicine:{}:{}:{}",
                battle(c)?.id,
                supply_sheet_id,
                c.revision + 1
            ),
            item_id: item["AppId"].as_str().unwrap_or("").into(),
            name: name.clone(),
            origin_sheet_id: supply_sheet_id.clone(),
            uses: 0,
            flavor: None,
        }
    };
    let capacity = if multiple { heal } else { 1 };
    if unit.uses.checked_add(*dose).is_none_or(|n| n > capacity) {
        return Err(error("This physical medicine unit has insufficient doses"));
    }
    let supply = c.sheets.get_mut(supply_sheet_id).unwrap();
    if unit_id.is_none() {
        *supply.inventory.get_mut(name).unwrap() -= 1;
    }
    if let Some(stored) = supply.stored_items.as_mut() {
        stored.retain(|u| u.id != unit.id);
    }
    unit.uses += *dose;
    if unit.uses < capacity {
        supply
            .stored_items
            .get_or_insert_default()
            .push(unit.clone());
    }
    supply.revision = supply
        .revision
        .checked_add(1)
        .ok_or_else(|| error("Sheet revision exhausted"))?;
    state_mut(c).payers.insert(supply_sheet_id.clone());
    if unit.uses == capacity {
        super::state_mut(c).units.insert(
            unit.id.clone(),
            BattleItem {
                review: Some(ItemReview {
                    scope: "supported".into(),
                    note: "Spent ordinary medicine; eligible for reviewed Recycle".into(),
                }),
                unit: unit.clone(),
                initial_holder: combatant_id.clone(),
                initial_slot: 0,
                holder: combatant_id.clone(),
                slot: 0,
                state: "consumed".into(),
                harvest_pending: false,
            },
        );
    }
    apply_medicine(c, target_id, &item, amount, full, revived)?;
    if *trainer_on_field {
        *c.battle
            .as_mut()
            .unwrap()
            .trainer_actions
            .entry(supply_sheet_id.clone())
            .or_default() += 1;
    } else {
        fighter_mut(c, combatant_id)?.actions += 1;
        advance(c);
    }
    record(
        c,
        &unit.id,
        combatant_id,
        Some(target_id.clone()),
        "medicine",
        "Owned medicine used; dose, action and healing limits enforced",
    );
    Ok(())
}
pub fn medicine_names() -> &'static [&'static str] {
    &[
        "Potion",
        "Super Potion",
        "Hyper Potion",
        "Max Potion",
        "Full Restore",
        "Antidote",
        "Awakening",
        "Burn Heal",
        "Ice Heal",
        "Paralyze Heal",
        "Full Heal",
        "Energy Root",
        "Remedy",
        "Heal Powder",
        "Revive",
        "Max Revive",
        "Revival Herb",
    ]
}
fn recoverable(c: &Campaign, p: &Combatant) -> i32 {
    let debt = if special::enabled(c) {
        flow(c)
            .map(|f| {
                f.special
                    .effects
                    .iter()
                    .filter(|e| e.name == "Shadow Force" && e.target == p.id)
                    .map(|e| e.amount)
                    .sum::<i32>()
            })
            .unwrap_or(0)
    } else {
        0
    };
    (p.max_hp - debt - p.hp).max(0)
}
fn apply_medicine(
    c: &mut Campaign,
    target: &str,
    item: &serde_json::Value,
    amount: i32,
    full: bool,
    revive: bool,
) -> Result<()> {
    let p = fighter_mut(c, target)?;
    if revive || p.hp > 0 {
        p.hp = (p.hp + amount).min(p.max_hp);
        p.runtime.healed += amount;
    }
    let cure = item["Cures"].as_str().unwrap_or("");
    p.runtime.statuses.retain(|s| !match cure {
        "All Status" => [
            "Burn",
            "Burn 2",
            "Burn 3",
            "Poison",
            "Badly Poisoned",
            "Frozen Solid",
            "Paralysis",
            "Sleep",
        ]
        .contains(&s.kind.as_str()),
        "Poison" => s.kind.contains("Poison"),
        "Burn" | "Burn 1/2" => s.kind.starts_with("Burn"),
        "Frozen" | "Frozen Solid" => s.kind == "Frozen Solid",
        _ => s.kind == cure,
    });
    let sheet_id = p.sheet_id.clone();
    if full && ["Max Potion", "Full Restore"].contains(&item["Name"].as_str().unwrap_or("")) {
        let sheet = c.sheets.get_mut(&sheet_id).unwrap();
        sheet.revision = sheet
            .revision
            .checked_add(1)
            .ok_or_else(|| error("Sheet revision exhausted"))?;
        sheet.medical = Some(MedicalRecord {
            healing_until: (c.ability_inheritance.minutes / 1440 + 1).saturating_mul(1440),
        });
    }
    Ok(())
}
pub fn recycle_medicine(
    c: &mut Campaign,
    recipient: &str,
    item: &serde_json::Value,
) -> Result<bool> {
    if !enabled(c) || !medicine_names().contains(&item["Name"].as_str().unwrap_or("")) {
        return Ok(false);
    }
    let p = special::registered(c, recipient)?.clone();
    let heal = item["HealthRestored"].as_i64().unwrap_or(0) as i32;
    if heal > 0 && (healing_locked(c, recipient) || special::healing_blocked(c, recipient)) {
        return Ok(true);
    }
    let revive = item["Cures"] == "Fainted";
    if (revive && p.hp > 0) || (!revive && p.hp == 0) {
        return Err(error(
            "Recycled medicine cannot bypass fainting eligibility",
        ));
    }
    let name = item["Name"].as_str().unwrap_or("");
    if ["Energy Root", "Remedy", "Heal Powder", "Revival Herb"].contains(&name)
        && !p.has_ability("Gluttony")
        && state(c)
            .rulings
            .get(&format!("herbal:{recipient}"))
            .map(String::as_str)
            != Some("accepted")
    {
        return Err(AppError::new(
            "manual_rule",
            "Recycled herbs need the same recorded herbal ruling",
        ));
    }
    let full = match name {
        "Max Potion" | "Full Restore" => true,
        "Max Revive" | "Revival Herb" => match state(c).rulings.get("revive").map(String::as_str) {
            Some("full") => true,
            Some("capped") => false,
            _ => {
                return Err(error(
                    "Record full revival versus capped healing before Recycle",
                ))
            }
        },
        _ => false,
    };
    let n = if full {
        recoverable(c, &p)
    } else {
        heal.min((3 - p.runtime.healed).max(0))
            .min(recoverable(c, &p))
    };
    apply_medicine(c, recipient, item, n, full, revive)?;
    Ok(true)
}
pub fn apply_stats(c: &Campaign, p: &Combatant, s: &mut Sheet) {
    if !enabled(c) {
        return;
    }
    for key in ["Strength", "Dexterity", "Vitality", "Special", "Insight"] {
        let delta = stat(c, p, key);
        if delta != 0 {
            if let Some(value) = s.stats.get_mut(key) {
                let minimum =
                    if key == "Dexterity" && status(p, "Paralysis") && !p.has_ability("Quick Feet")
                    {
                        0
                    } else {
                        1
                    };
                *value = (*value + delta).clamp(minimum, forms::stat_cap(c, p));
            }
        }
    }
    *s.skills.entry("Evasion".into()).or_default() += stat(c, p, "EvasionPool");
}
pub fn definitions() -> &'static [&'static str] {
    &[
        "Air Balloon",
        "Choice Band",
        "Choice Scarf",
        "Choice Specs",
        "Clear Amulet",
        "Destiny Knot",
        "Eject Button",
        "Flame Orb",
        "Focus Sash",
        "Heavy-Duty Boots",
        "Leek",
        "Light Ball",
        "Loaded Dice",
        "Lucky Punch",
        "Metronome",
        "Power Herb",
        "Power Increasers",
        "Protective Pads",
        "Razor Claw",
        "Red Card",
        "Ring Target",
        "Rocky Helmet",
        "Safety Goggles",
        "Sticky Barb",
        "Thick Club",
        "Throat Spray",
        "Toxic Orb",
        "Umbrella",
        "Weakness Policy",
        "White Herb",
        "Potion",
        "Super Potion",
        "Hyper Potion",
        "Max Potion",
        "Full Restore",
        "Antidote",
        "Awakening",
        "Burn Heal",
        "Ice Heal",
        "Paralyze Heal",
        "Full Heal",
        "Energy Root",
        "Remedy",
        "Heal Powder",
        "Revive",
        "Max Revive",
        "Revival Herb",
    ]
}

fn restore_lowered(c: &mut Campaign, id: &str) -> Result<()> {
    let contrary = fighter(c, id)?.has_ability("Contrary");
    fighter_mut(c, id)?
        .runtime
        .modifiers
        .retain(|_, n| if contrary { *n <= 0 } else { *n >= 0 });
    fighter_mut(c, id)?
        .runtime
        .ability
        .counters
        .retain(|k, n| !(k.starts_with("debuff:") && *n < 0));
    flow_mut(c).special.effects.retain(|e| {
        e.target != id
            || !(e.name.starts_with("buff:") || e.name.starts_with("debuff:"))
            || if contrary {
                e.amount <= 0
            } else {
                e.amount >= 0
            }
    });
    for g in &mut flow_mut(c).stat_channels.grants {
        if g.target == id {
            g.negative = 0;
        }
    }
    for ids in flow_mut(c).intimidate.values_mut() {
        ids.retain(|v| v != id);
    }
    Ok(())
}
pub fn recyclable_held(name: &str) -> bool {
    ["Power Herb", "White Herb", "Air Balloon"].contains(&name)
}
pub fn recycle_held(c: &mut Campaign, id: &str, unit: &HeldItem) -> Result<bool> {
    if !enabled(c) || !recyclable_held(&unit.name) {
        return Ok(false);
    }
    if unit.name == "Air Balloon"
        && state(c).rulings.get("recycle-balloon").map(String::as_str) != Some("until-damage")
    {
        return Err(AppError::new("manual_rule","GM must approve Recycle balloon immunity lasting until damage; no physical unit is recreated"));
    }
    if unit.name == "White Herb" {
        restore_lowered(c, id)?;
    } else {
        *state_mut(c)
            .recycled
            .entry(id.into())
            .or_default()
            .entry(unit.name.clone())
            .or_default() = 1;
    }
    Ok(true)
}

pub fn recycle_recipient(c: &Campaign, id: &str) -> Option<String> {
    state(c).recycle_recipients.get(id).cloned()
}
