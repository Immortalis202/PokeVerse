//! Owned held-item units and staged encounter effects.
use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct HeldItem {
    pub id: String,
    pub item_id: String,
    pub name: String,
    pub origin_sheet_id: String,
    /// Persistent usage, e.g. Leftovers' three points of total healing.
    pub uses: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub flavor: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct ItemState {
    pub enabled: bool,
    pub initialized: bool,
    pub finalized: bool,
    /// GM choice: return surviving units after an official match or keep transfers.
    pub return_policy: Option<String>,
    pub units: BTreeMap<String, BattleItem>,
    pub history: Vec<ItemEvent>,
    #[serde(default)]
    pub unburdened: BTreeSet<String>,
    #[serde(default)]
    pub symbiosis_chains: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub thefts: BTreeMap<String, BTreeMap<String, TheftOffer>>,
    #[serde(default)]
    pub theft_seen: BTreeSet<String>,
    #[serde(default)]
    pub end_round: u32,
    #[serde(default)]
    pub redacted: bool,
    #[serde(default)]
    pub frisk_active: BTreeSet<String>,
    #[serde(default)]
    pub reveals: Vec<ItemReveal>,
    #[serde(default)]
    pub natural_gift: BTreeMap<String, String>,
    #[serde(default)]
    pub berry_moves: BTreeMap<String, BerryMove>,
    #[serde(default)]
    pub fling: BTreeMap<String, String>,
    #[serde(default)]
    pub fling_moves: BTreeMap<String, FlingMove>,
    #[serde(default)]
    pub fling_hits: BTreeSet<String>,
    #[serde(default)]
    pub berry_hits: BTreeMap<String, String>,
    #[serde(default)]
    pub recycle: BTreeMap<String, String>,
    #[serde(default)]
    pub recycle_uses: BTreeMap<String, u32>,
    #[serde(default)]
    pub recycle_attempts: BTreeSet<String>,
    #[serde(default)]
    pub recycle_moves: BTreeMap<String, String>,
    #[serde(default)]
    pub chew_charges: BTreeMap<String, ChewCharge>,
    #[serde(default)]
    pub chew_serial: u32,
    /// Reviewed catalog stages keyed by species identity, never inferred from evolution edges.
    #[serde(default)]
    pub stages: BTreeMap<String, u32>,
    #[serde(default)]
    pub life_orb_moves: BTreeMap<String, bool>,
    #[serde(default)]
    pub chance_moves: BTreeMap<String, u32>,
    #[serde(default)]
    pub fetches: BTreeMap<String, FetchAttempt>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct ChewCharge {
    pub combatant_id: String,
    pub unit: HeldItem,
    pub source: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct FlingMove {
    pub unit_id: Option<String>,
    pub item_bonus: i32,
    pub ripen_bonus: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct BerryMove {
    pub unit_id: String,
    pub move_type: String,
    pub ripen_bonus: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct ItemReveal {
    pub owner_id: String,
    pub source_id: String,
    pub target_id: String,
    pub items: Vec<String>,
    pub round: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct FetchAttempt {
    pub trainer_id: String,
    pub fetcher_id: String,
    pub target_id: String,
    pub unit: HeldItem,
    pub round: u32,
    pub returned: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct TheftOffer {
    pub thief_id: String,
    pub target_id: String,
    pub stolen_id: String,
    pub outgoing_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct BattleItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub review: Option<ItemReview>,
    pub unit: HeldItem,
    pub initial_holder: String,
    #[serde(default)]
    pub initial_slot: u32,
    pub holder: String,
    pub slot: u32,
    /// held, consumed, ground, or stored after packing at battle end.
    pub state: String,
    /// Harvest was active when this held berry was consumed.
    #[serde(default)]
    pub harvest_pending: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct ItemReview {
    pub scope: String,
    pub note: String,
}

fn effect_review(cat: &Catalog, unit: &HeldItem) -> Result<ItemReview> {
    let item = definition(cat, unit)?;
    let (scope, note) = if item["Category"] == "MegaStone" {
        ("transformation", "Reviewed Mega mappings activate in engine 0.19. Klutz does not suppress Mega Evolution items.")
    } else if item["Category"] == "ZCrystal" {
        (
            "excluded",
            "Z-Moves remain outside battle scope; Klutz's printed exception does not enable them.",
        )
    } else if unit.name == "Berry Juice" {
        (
            "restricted",
            "Berry Juice explicitly cannot be used during battle.",
        )
    } else if [
        "Oran Berry",
        "Sitrus Berry",
        "Pecha Berry",
        "Rawst Berry",
        "Aspear Berry",
        "Cheri Berry",
        "Chesto Berry",
        "Persim Berry",
        "Lum Berry",
    ]
    .contains(&unit.name.as_str())
    {
        (
            "controlled",
            "Reviewed held-berry effects use the owner's berry controls; consumption is optional.",
        )
    } else if [
        "Leftovers",
        "Wide Lens",
        "Zoom Lens",
        "Bright Powder",
        "Quick Claw",
        "Iron Ball",
        "Eviolite",
        "Expert Belt",
        "Life Orb",
        "King's Rock",
        "Razor Fang",
    ]
    .contains(&unit.name.as_str())
        || (item["Category"] == "TypeBoosting" && item["Boost"] == "Damage" && item["Value"] == 1)
    {
        (
            "supported",
            "Reviewed battle effect; printed conditions and suppression rules still apply.",
        )
    } else if recyclable(&item) {
        ("manual", "Original medicine use and its action cost require GM adjudication. Reviewed Recycle can reuse a recorded spent unit.")
    } else if item["Pocket"] == "Pokeballs" {
        ("manual", "Capture rolls require GM adjudication. Ball Fetch protects recorded failed throws from the trainer's owned balls.")
    } else {
        ("manual", "This item's effects are not automated. Resolve them with the GM, including Klutz/Magic Room and printed form/type-changing exceptions.")
    };
    Ok(ItemReview {
        scope: scope.into(),
        note: note.into(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct ItemEvent {
    pub unit_id: String,
    pub from: String,
    pub to: Option<String>,
    pub kind: String,
    pub reason: String,
    pub round: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct EquipmentResources {
    pub held_items: Option<Vec<HeldItem>>,
    pub stored_items: Option<Vec<HeldItem>>,
    pub inventory: BTreeMap<String, i32>,
}
impl EquipmentResources {
    pub fn snapshot(s: &Sheet) -> Self {
        Self {
            held_items: s.held_items.clone(),
            stored_items: s.stored_items.clone(),
            inventory: s.inventory.clone(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/bindings/")]
pub enum ItemCommand {
    FailedCapture {
        attempt_id: String,
        trainer_id: String,
        fetcher_id: String,
        target_id: String,
        ball: String,
    },
    ChewAgain {
        charge_id: String,
    },
    Recycle {
        combatant_id: String,
        unit_id: String,
    },
    Spend {
        combatant_id: String,
        unit_id: String,
    },
    Fling {
        combatant_id: String,
        unit_id: Option<String>,
    },
    Retrieve {
        combatant_id: String,
        unit_id: String,
    },
    BerryFlavor {
        unit_id: String,
        flavor: String,
    },
    NaturalGift {
        combatant_id: String,
        unit_id: String,
    },
    ConsumeRipenedBerry {
        combatant_id: String,
        unit_id: String,
        extra_healing: i32,
    },
    ConsumeBerry {
        combatant_id: String,
        unit_id: String,
    },
    ReturnPolicy {
        policy: String,
    },
    /// Explicit GM state ruling; automatic move/ability theft uses separate triggers.
    Transfer {
        from_id: String,
        to_id: String,
        unit_id: String,
        override_sticky_hold: bool,
    },
    Swap {
        first_id: String,
        second_id: String,
        first_unit_id: String,
        second_unit_id: String,
        override_sticky_hold: bool,
    },
    Lose {
        combatant_id: String,
        unit_id: String,
        override_sticky_hold: bool,
    },
    Recover {
        combatant_id: String,
        unit_id: String,
    },
}

pub fn enabled(c: &Campaign) -> bool {
    c.battle
        .as_ref()
        .and_then(|b| b.flow.as_ref())
        .is_some_and(|f| f.items.enabled)
}
fn state(c: &Campaign) -> &ItemState {
    &c.battle.as_ref().unwrap().flow.as_ref().unwrap().items
}
fn state_mut(c: &mut Campaign) -> &mut ItemState {
    &mut flow_mut(c).items
}

/// Include noncombatant trainers whose inventory participates in an item checkpoint.
pub fn resource_sheets(current: &Campaign, next: &Campaign) -> BTreeSet<String> {
    let mut ids: BTreeSet<_> = current
        .battle
        .iter()
        .flat_map(|b| &b.combatants)
        .map(|p| p.sheet_id.clone())
        .collect();
    if enabled(current) || enabled(next) {
        for campaign in [current, next] {
            if let Some(items) = campaign
                .battle
                .as_ref()
                .and_then(|b| b.flow.as_ref())
                .map(|f| &f.items)
            {
                ids.extend(
                    items
                        .fetches
                        .values()
                        .map(|attempt| attempt.trainer_id.clone()),
                );
            }
        }
    }
    ids
}

fn failed_capture(c: &mut Campaign, command: &ItemCommand, cat: &Catalog) -> Result<()> {
    let ItemCommand::FailedCapture {
        attempt_id,
        trainer_id,
        fetcher_id,
        target_id,
        ball,
    } = command
    else {
        unreachable!()
    };
    if attempt_id.is_empty()
        || attempt_id.len() > 100
        || !attempt_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_:".contains(&b))
    {
        return Err(error(
            "Record a stable capture-attempt ID (letters, numbers, dash, underscore or colon)",
        ));
    }
    if state(c).end_round == battle(c)?.round {
        return Err(error("This round's item returns have already resolved; record a throw during its actual round"));
    }
    if state(c).fetches.contains_key(attempt_id) {
        return Err(error("This failed capture attempt is already recorded"));
    }
    let fetcher = fighter(c, fetcher_id)?;
    let target = fighter(c, target_id)?;
    let trainer = c
        .sheets
        .get(trainer_id)
        .ok_or_else(|| error("Capture trainer missing"))?;
    if !fetcher.deployed
        || fetcher.hp == 0
        || !fetcher.has_ability("Ball Fetch")
        || trainer.kind != "trainer"
        || trainer.owner_id != fetcher.owner_id
        || c.sheets[&fetcher.sheet_id].trainer_id.as_deref() != Some(trainer_id)
        || !target.deployed
        || target.side == fetcher.side
        || c.sheets[&target.sheet_id].kind != "pokemon"
        || c.sheets[&target.sheet_id].trainer_id.is_some()
    {
        return Err(error("GM must record a failed throw at a wild foe for this trainer's conscious deployed Ball Fetch Pokemon"));
    }
    if field::has(c, "Magic Room", fetcher) {
        return Err(error("Magic Room prevents using capture balls"));
    }
    let pending = state(c)
        .fetches
        .values()
        .filter(|f| f.trainer_id == *trainer_id && !f.returned)
        .count();
    let mut sheet = trainer.clone();
    let unit = if let Some(index) = sheet
        .stored_items
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .position(|u| u.id == *ball)
    {
        let unit = sheet.stored_items.as_mut().unwrap().remove(index);
        if sheet.stored_items.as_ref().unwrap().is_empty() {
            sheet.stored_items = None;
        }
        unit
    } else {
        if sheet.inventory.get(ball).copied().unwrap_or(0) < 1 {
            return Err(error("Select an owned capture ball or stored ball unit"));
        }
        let item = cat
            .find("items", "Name", ball)
            .ok_or_else(|| error("Unknown capture ball"))?;
        *sheet.inventory.get_mut(ball).unwrap() -= 1;
        sheet.inventory.retain(|_, count| *count > 0);
        let id = format!("fetch:{}:{attempt_id}", battle(c)?.id);
        if c.sheets.values().any(|s| {
            s.held_items
                .iter()
                .flatten()
                .chain(s.stored_items.iter().flatten())
                .any(|u| u.id == id)
        }) || state(c).fetches.values().any(|f| f.unit.id == id)
            || state(c).units.contains_key(&id)
        {
            return Err(error("Capture-ball unit identity already exists"));
        }
        HeldItem {
            id,
            item_id: item["AppId"].as_str().unwrap_or("").into(),
            name: ball.clone(),
            origin_sheet_id: trainer_id.clone(),
            uses: 0,
            flavor: None,
        }
    };
    if definition(cat, &unit)?["Pocket"] != "Pokeballs" {
        return Err(error("Select a catalog capture ball"));
    }
    if sheet.stored_items.as_deref().unwrap_or(&[]).len() + pending >= 100 {
        return Err(error(
            "Reserve storage space for the protected capture ball's return",
        ));
    }
    sheet.revision = sheet
        .revision
        .checked_add(1)
        .ok_or_else(|| error("Sheet revision exhausted"))?;
    let round = battle(c)?.round;
    c.sheets.insert(trainer_id.clone(), sheet);
    state_mut(c).fetches.insert(
        attempt_id.clone(),
        FetchAttempt {
            trainer_id: trainer_id.clone(),
            fetcher_id: fetcher_id.clone(),
            target_id: target_id.clone(),
            unit: unit.clone(),
            round,
            returned: false,
        },
    );
    record(
        c,
        &unit.id,
        fetcher_id,
        None,
        "ball_fetch_pending",
        "GM confirmed an unsuccessful wild capture; the owned ball is protected until round end",
    );
    Ok(())
}

fn return_fetches(c: &mut Campaign, cat: &Catalog) -> Result<()> {
    let pending: Vec<_> = state(c)
        .fetches
        .iter()
        .filter(|(_, f)| !f.returned && f.round <= battle(c).unwrap().round)
        .map(|(id, f)| (id.clone(), f.clone()))
        .collect();
    for (id, attempt) in pending {
        if c.sheets.values().any(|s| {
            s.held_items
                .iter()
                .flatten()
                .chain(s.stored_items.iter().flatten())
                .any(|u| u.id == attempt.unit.id)
        }) || state(c).units.contains_key(&attempt.unit.id)
        {
            return Err(error(
                "Protected capture ball already exists elsewhere in equipment",
            ));
        }
        let sheet = c
            .sheets
            .get_mut(&attempt.trainer_id)
            .ok_or_else(|| error("Capture trainer missing"))?;
        if sheet
            .stored_items
            .iter()
            .flatten()
            .chain(sheet.held_items.iter().flatten())
            .any(|u| u.id == attempt.unit.id)
        {
            return Err(error("Protected capture ball is already in equipment"));
        }
        sheet
            .stored_items
            .get_or_insert_default()
            .push(attempt.unit.clone());
        validate_equipment(sheet, cat)?;
        sheet.revision = sheet
            .revision
            .checked_add(1)
            .ok_or_else(|| error("Sheet revision exhausted"))?;
        state_mut(c).fetches.get_mut(&id).unwrap().returned = true;
        record(
            c,
            &attempt.unit.id,
            &attempt.fetcher_id,
            Some(attempt.trainer_id),
            "ball_fetch_return",
            "Ball Fetch returned the same protected ball to its trainer at round end",
        );
    }
    Ok(())
}

pub fn initialize(c: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let mut stages = BTreeMap::new();
    for p in &battle(c)?.combatants {
        let sheet = &c.sheets[&p.sheet_id];
        let stage = sheet
            .species_id
            .as_deref()
            .and_then(|id| cat.find("species", "AppId", id))
            .and_then(|v| v["Creation"]["stage"].as_u64());
        if let (Some(id), Some(stage @ 1..=3)) = (&sheet.species_id, stage) {
            stages.insert(id.clone(), stage as u32);
        }
        let equipped = if state(c).initialized {
            held(c, &p.id)
        } else {
            sheet.held_items.clone().unwrap_or_default()
        };
        if equipped.iter().any(|unit| unit.name == "Eviolite") && !matches!(stage, Some(1..=3)) {
            return Err(AppError::new("manual_rule", "Eviolite requires a reviewed catalog evolution stage; resolve the species metadata or unequip it before automated battle use"));
        }
    }
    let mut protected = BTreeSet::new();
    for attempt in state(c).fetches.values().filter(|f| !f.returned) {
        if !protected.insert(&attempt.unit.id)
            || c.sheets.values().any(|s| {
                s.held_items
                    .iter()
                    .flatten()
                    .chain(s.stored_items.iter().flatten())
                    .any(|unit| unit.id == attempt.unit.id)
            })
        {
            return Err(error("A protected capture-ball unit cannot also exist in equipment or another pending throw"));
        }
    }
    state_mut(c).stages = stages;
    if state(c).initialized {
        let reviews: Vec<_> = state(c)
            .units
            .iter()
            .map(|(id, unit)| Ok((id.clone(), effect_review(cat, &unit.unit)?)))
            .collect::<Result<_>>()?;
        for (id, review) in reviews {
            state_mut(c).units.get_mut(&id).unwrap().review = Some(review);
        }
        return Ok(());
    }
    let mut identities = BTreeSet::new();
    for s in c.sheets.values() {
        for unit in s
            .held_items
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .chain(s.stored_items.as_deref().unwrap_or(&[]))
        {
            if !identities.insert(unit.id.clone()) {
                return Err(error("An item unit cannot belong to two sheets"));
            }
        }
    }
    let mut units = BTreeMap::new();
    for p in &battle(c)?.combatants {
        let s = &c.sheets[&p.sheet_id];
        validate_equipment(s, cat)?;
        for (slot, unit) in s.held_items.as_deref().unwrap_or(&[]).iter().enumerate() {
            if units.contains_key(&unit.id) {
                return Err(error(
                    "One equipped item unit cannot be assigned to multiple combatants",
                ));
            }
            units.insert(
                unit.id.clone(),
                BattleItem {
                    review: Some(effect_review(cat, unit)?),
                    unit: unit.clone(),
                    initial_holder: p.id.clone(),
                    initial_slot: slot as u32,
                    holder: p.id.clone(),
                    slot: slot as u32,
                    state: "held".into(),
                    harvest_pending: false,
                },
            );
        }
    }
    state_mut(c).units = units;
    state_mut(c).initialized = true;
    Ok(())
}

pub fn frisk_lifecycle(c: &mut Campaign) -> Result<()> {
    if !enabled(c) || !state(c).initialized {
        return Ok(());
    }
    let active: Vec<_> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0 && p.has_ability("Frisk"))
        .cloned()
        .collect();
    let previous = state(c).frisk_active.clone();
    for source in active.iter().filter(|p| !previous.contains(&p.id)) {
        let choices: Vec<_> = battle(c)?
            .combatants
            .iter()
            .filter(|p| p.deployed && p.hp > 0 && p.side != source.side)
            .map(|p| p.id.clone())
            .collect();
        if choices.is_empty() {
            continue;
        }
        let event_id = format!(
            "item-frisk:{}:{}:{}",
            c.revision + 1,
            flow(c)?.ability_events.len(),
            source.id
        );
        flow_mut(c).ability_events.push(abilities::AbilityEvent {
            id: event_id,
            combatant_id: source.id.clone(),
            target_id: source.id.clone(),
            ability: "Frisk".into(),
            kind: "item-frisk".into(),
            move_name: "battlefield entry".into(),
            choices,
        });
    }
    state_mut(c).frisk_active = active.iter().map(|p| p.id.clone()).collect();
    Ok(())
}

pub fn resolve_frisk(
    c: &mut Campaign,
    event: &abilities::AbilityEvent,
    choice: &str,
) -> Result<()> {
    if choice == "manual" {
        return Ok(());
    }
    let source = fighter(c, &event.combatant_id)?;
    let target = fighter(c, choice)?;
    if !source.deployed
        || source.hp == 0
        || !source.has_ability("Frisk")
        || !target.deployed
        || target.hp == 0
        || source.side == target.side
    {
        return Err(error(
            "Frisk entry reveal is no longer eligible; GM must adjudicate the expired trigger",
        ));
    }
    let owner_id = c
        .sheets
        .get(&source.sheet_id)
        .and_then(|s| s.trainer_id.as_ref())
        .and_then(|trainer| c.sheets.get(trainer))
        .map_or_else(
            || source.owner_id.clone(),
            |trainer| trainer.owner_id.clone(),
        );
    let reveal = ItemReveal {
        owner_id,
        source_id: source.id.clone(),
        target_id: target.id.clone(),
        items: held(c, choice).into_iter().map(|u| u.name).collect(),
        round: battle(c)?.round,
    };
    state_mut(c).reveals.push(reveal);
    Ok(())
}

pub fn prune_offers(c: &mut Campaign) {
    if !enabled(c) {
        return;
    }
    let live: BTreeSet<_> = flow(c)
        .unwrap()
        .ability_events
        .iter()
        .map(|e| e.id.clone())
        .collect();
    let pending: BTreeSet<_> = flow(c)
        .unwrap()
        .queue
        .iter()
        .map(|q| format!("{}:{}", q.actor_id, q.sequence))
        .collect();
    let released: Vec<_> = state(c)
        .recycle_moves
        .iter()
        .filter(|(key, _)| !pending.contains(*key))
        .map(|(_, unit)| unit.clone())
        .collect();
    for unit in released {
        if let Some(unit) = state_mut(c)
            .units
            .get_mut(&unit)
            .filter(|u| u.state == "recycle_pending")
        {
            unit.state = "consumed".into();
        }
    }
    state_mut(c)
        .recycle_moves
        .retain(|key, _| pending.contains(key));
    let canceled: Vec<_> = state(c)
        .berry_hits
        .iter()
        .filter(|(id, _)| !live.contains(*id))
        .map(|(_, unit)| unit.clone())
        .collect();
    for unit in canceled {
        if let Some(unit) = state_mut(c)
            .units
            .get_mut(&unit)
            .filter(|u| u.state == "berry_hit_pending")
        {
            unit.state = "ground".into();
        }
    }
    state_mut(c).berry_hits.retain(|id, _| live.contains(id));
    state_mut(c).thefts.retain(|id, _| live.contains(id));
    state_mut(c)
        .symbiosis_chains
        .retain(|id, _| live.contains(id));
}

pub fn held(c: &Campaign, id: &str) -> Vec<HeldItem> {
    if !enabled(c) || !state(c).initialized {
        return vec![];
    }
    let mut units: Vec<_> = state(c)
        .units
        .values()
        .filter(|u| u.holder == id && u.state == "held")
        .collect();
    units.sort_by_key(|u| (u.slot, &u.unit.id));
    units.into_iter().map(|u| u.unit.clone()).collect()
}

fn has_effect(c: &Campaign, id: &str, name: &str) -> bool {
    active_effects(c, id).unwrap_or(false) && held(c, id).iter().any(|unit| unit.name == name)
}

pub fn iron_ball(c: &Campaign, id: &str) -> bool {
    has_effect(c, id, "Iron Ball")
}

pub fn stat_increase(c: &Campaign, p: &Combatant, key: &str) -> i32 {
    if !["Defense", "SpecialDefense"].contains(&key) || !has_effect(c, &p.id, "Eviolite") {
        return 0;
    }
    let stage = c
        .sheets
        .get(&p.sheet_id)
        .and_then(|s| s.species_id.as_ref())
        .and_then(|id| state(c).stages.get(id));
    i32::from(matches!(stage, Some(1 | 2)))
}

pub fn stat_reduction(c: &Campaign, p: &Combatant, key: &str) -> i32 {
    if key == "Dexterity" && iron_ball(c, &p.id) {
        -1
    } else {
        0
    }
}

pub fn chance_base(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> u32 {
    u32::from(
        (m["Category"] == "Physical" && has_effect(c, &p.id, "Razor Fang"))
            || (m["Category"] == "Special" && has_effect(c, &p.id, "King's Rock")),
    )
}

pub fn chance_dice(c: &Campaign, q: &QueuedAction, cat: &Catalog) -> Result<u32> {
    if !enabled(c) || q.ice || (provenance::enabled(c) && q.sheer_force) {
        return Ok(0);
    }
    let source = reviewed::source(c, q)?;
    let target = fighter(c, &q.target_id)?;
    if target.hp == 0 || target.runtime.flinch_round == battle(c)?.round {
        return Ok(0);
    }
    let m = field::move_data(c, &source, &move_entry(cat, &q.move_name)?);
    let base = state(c)
        .chance_moves
        .get(&format!("{}:{}", q.actor_id, q.sequence))
        .copied()
        .unwrap_or_else(|| chance_base(c, &source, &m));
    if base == 0 {
        return Ok(0);
    }
    let extreme = source.side != target.side && reviewed::extreme(c, &source);
    if (!extreme && special::target_effects_blocked(c, q, &m))
        || (!extreme && source.side != target.side && target.has_ability("Shield Dust"))
    {
        return Ok(0);
    }
    let (effect, immune) = if q.move_name == "Sonic Boom" {
        (0, false)
    } else {
        rules::type_matchup(c, &source, target, m["Type"].as_str().unwrap_or(""), cat)?
    };
    if immune
        || abilities::damage_immune_from(c, &source, target, &m)
        || extra::source_immune(c, &source, target, effect)
    {
        return Ok(0);
    }
    let serene = 2 * u32::from(if interactions::enabled(c) {
        source.has_ability("Serene Grace")
    } else {
        fighter(c, &q.actor_id)?.has_ability("Serene Grace")
    });
    let wonder = 2 * u32::from(!extreme && target.has_ability("Wonder Skin"));
    let native = special::chance(&q.move_name)
        .or_else(|| chance_effect(&q.move_name))
        .or_else(|| interactions::native_flinch(c, &q.move_name))
        .filter(|(kind, _)| *kind == "Flinch")
        .map_or(0, |(_, n)| n);
    Ok((native + base + serene).saturating_sub(wonder))
}

/// The wrapper replaces a native Flinch pool with the merged item/native pool.
pub fn native_item_flinch_dice(c: &Campaign, q: &QueuedAction, cat: &Catalog) -> Result<u32> {
    if !enabled(c) {
        return Ok(0);
    }
    let source = reviewed::source(c, q)?;
    let m = field::move_data(c, &source, &move_entry(cat, &q.move_name)?);
    let item = state(c)
        .chance_moves
        .get(&format!("{}:{}", q.actor_id, q.sequence))
        .copied()
        .unwrap_or_else(|| chance_base(c, &source, &m));
    if item == 0 {
        return Ok(0);
    }
    let native = special::chance(&q.move_name)
        .or_else(|| chance_effect(&q.move_name))
        .or_else(|| interactions::native_flinch(c, &q.move_name))
        .filter(|(kind, _)| *kind == "Flinch")
        .map_or(0, |(_, n)| n);
    if native == 0 {
        return Ok(0);
    }
    let serene = 2 * u32::from(if interactions::enabled(c) {
        source.has_ability("Serene Grace")
    } else {
        fighter(c, &q.actor_id)?.has_ability("Serene Grace")
    });
    let wonder = 2 * u32::from(fighter(c, &q.target_id)?.has_ability("Wonder Skin"));
    Ok((native + serene).saturating_sub(wonder))
}

pub fn apply_chance(c: &mut Campaign, q: &QueuedAction, dice: &[u8], cat: &Catalog) -> Result<()> {
    check_count(dice, chance_dice(c, q, cat)?)?;
    if dice.contains(&6) {
        abilities::inflict(c, &q.actor_id, &q.target_id, "Flinch", "", None, cat)?;
    }
    Ok(())
}

pub fn life_orb(c: &Campaign, id: &str, sequence: u32) -> bool {
    enabled(c)
        && state(c)
            .life_orb_moves
            .get(&format!("{id}:{sequence}"))
            .copied()
            .unwrap_or_else(|| has_effect(c, id, "Life Orb"))
}

pub fn capture_damage(c: &mut Campaign, q: &QueuedAction, cat: &Catalog) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let key = format!("{}:{}", q.actor_id, q.sequence);
    if !state(c).chance_moves.contains_key(&key) {
        let p = fighter(c, &q.actor_id)?;
        let m = field::move_data(c, p, &move_entry(cat, &q.move_name)?);
        let base = chance_base(c, p, &m);
        state_mut(c).chance_moves.insert(key.clone(), base);
    }
    if !state(c).life_orb_moves.contains_key(&key) {
        let m = field::move_data(c, fighter(c, &q.actor_id)?, &move_entry(cat, &q.move_name)?);
        let active = ["Physical", "Special"]
            .iter()
            .any(|kind| m["Category"] == *kind)
            && has_effect(c, &q.actor_id, "Life Orb");
        state_mut(c).life_orb_moves.insert(key, active);
    }
    Ok(())
}

pub fn damage_dice_bonus(
    c: &Campaign,
    p: &Combatant,
    sequence: u32,
    m: &serde_json::Value,
    cat: &Catalog,
) -> Result<i32> {
    let ordinary = ["Physical", "Special"]
        .iter()
        .any(|kind| m["Category"] == *kind);
    Ok(type_damage_bonus(c, p, m, cat)? + 2 * i32::from(ordinary && life_orb(c, &p.id, sequence)))
}

/// Added item recoil shares the existing raw-success basis; native Recoil is not duplicated.
pub fn recoil_dice(c: &Campaign, q: &QueuedAction, cat: &Catalog) -> Result<u32> {
    if !life_orb(c, &q.actor_id, q.sequence)
        || fighter(c, &q.actor_id)?.hp == 0
        || move_entry(cat, &q.move_name)?["Attributes"]["Recoil"] == true
        || ["Rock Head", "Magic Guard", "Wonder Guard"]
            .contains(&fighter(c, &q.actor_id)?.selected_ability.as_str())
    {
        return Ok(0);
    }
    // Shared damage is one attack: resolve its added recoil after the final target,
    // using all successes in the largest shared roll rather than repeating prefixes.
    if q.targets.iter().any(|target| {
        target.target_id != q.target_id
            && !target.resolved
            && !target.avoided
            && fighter(c, &target.target_id).is_ok_and(|p| p.hp > 0)
    }) {
        return Ok(0);
    }
    Ok(if q.shared_dice.is_empty() {
        q.raw_damage_successes
    } else {
        rules::successes(&q.shared_dice) as u32
    })
}

pub fn apply_recoil(c: &mut Campaign, q: &QueuedAction, dice: &[u8], cat: &Catalog) -> Result<()> {
    check_count(dice, recoil_dice(c, q, cat)?)?;
    let p = fighter_mut(c, &q.actor_id)?;
    p.hp = (p.hp - rules::successes(dice)).max(0);
    Ok(())
}

pub fn expert_belt(c: &Campaign, id: &str) -> bool {
    has_effect(c, id, "Expert Belt")
}

pub fn extra_damage(
    c: &Campaign,
    source: &str,
    target: &str,
    sequence: u32,
    matchup: (i32, bool),
    cat: &Catalog,
) -> Result<i32> {
    if !enabled(c) || matchup.0 <= 0 || matchup.1 {
        return Ok(0);
    }
    let target = fighter(c, target)?;
    if field::weather(c, Some(target)) == Some(field::Weather::StrongWinds)
        && field::types(c, target, cat)?
            .iter()
            .any(|kind| kind == "Flying")
    {
        return Ok(0);
    }
    let active = abilities::context(c, source, sequence)
        .map_or_else(|| expert_belt(c, source), |ctx| ctx.expert_belt);
    Ok(i32::from(active))
}

/// Reviewed type boosters add dice after the ordinary damage-pool floor.
pub fn type_damage_bonus(
    c: &Campaign,
    p: &Combatant,
    m: &serde_json::Value,
    cat: &Catalog,
) -> Result<i32> {
    if !active_effects(c, &p.id)?
        || !["Physical", "Special"]
            .iter()
            .any(|category| m["Category"] == *category)
    {
        return Ok(0);
    }
    let mut bonus = 0;
    for unit in held(c, &p.id) {
        let item = definition(cat, &unit)?;
        if item["Category"] != "TypeBoosting" || item["Boost"] != "Damage" || item["Value"] != 1 {
            continue;
        }
        // Black Belt's supplied metadata says Fighting; the rules use Fight.
        let kind = match item["ForTypes"].as_str().unwrap_or("") {
            "Fighting" => "Fight",
            kind => kind,
        };
        if m["Type"] == kind {
            bonus = 1;
        }
    }
    Ok(bonus)
}

pub fn accuracy_bonus(c: &Campaign, id: &str) -> i32 {
    if has_effect(c, id, "Wide Lens") {
        2
    } else {
        0
    }
}

pub fn accuracy_penalty(c: &Campaign, a: &Combatant, t: &Combatant, penalty: i32) -> i32 {
    let penalty = penalty - i32::from(has_effect(c, &t.id, "Bright Powder"));
    reduce_low_accuracy(c, &a.id, penalty)
}

pub fn reduce_low_accuracy(c: &Campaign, id: &str, penalty: i32) -> i32 {
    if has_effect(c, id, "Zoom Lens") {
        (penalty + 2).min(0)
    } else {
        penalty
    }
}

/// Printed Quick Claw bonus applies to the initiative roll, not its tie-break base.
pub fn initiative_bonus(c: &Campaign, id: &str) -> i32 {
    if !active_effects(c, id).unwrap_or(false) {
        return 0;
    }
    if held(c, id).iter().any(|unit| unit.name == "Quick Claw") {
        2
    } else {
        0
    }
}

fn record(c: &mut Campaign, unit_id: &str, from: &str, to: Option<String>, kind: &str, why: &str) {
    let round = c.battle.as_ref().unwrap().round;
    state_mut(c).history.push(ItemEvent {
        unit_id: unit_id.into(),
        from: from.into(),
        to,
        kind: kind.into(),
        reason: why.into(),
        round,
    });
}

fn after_loss(c: &mut Campaign, id: &str, cat: &Catalog) -> Result<()> {
    let p = fighter(c, id)?;
    if p.hp > 0
        && p.has_ability("Unburden")
        && held(c, id).is_empty()
        && !state(c).unburdened.contains(id)
    {
        state_mut(c).unburdened.insert(id.into());
        extra::once(c, id, "Unburden", &[("Dexterity", 2)])?;
    }
    offer_symbiosis(c, id, &[], cat)?;
    Ok(())
}

fn can_receive(c: &Campaign, target: &str, unit: &HeldItem, cat: &Catalog) -> Result<bool> {
    if forms::slot_reserved(c, target) {
        return Ok(false);
    }
    let p = fighter(c, target)?;
    let mut capacity = c.sheets[&p.sheet_id].clone();
    capacity.ability = if p.has_ability("Gluttony") {
        "Gluttony".into()
    } else {
        p.selected_ability.clone()
    };
    let mut units = held(c, target);
    units.push(unit.clone());
    capacity.held_items = Some(units);
    Ok(validate_equipment(&capacity, cat).is_ok())
}

fn offer_symbiosis(
    c: &mut Campaign,
    recipient: &str,
    chain: &[String],
    cat: &Catalog,
) -> Result<()> {
    let target = fighter(c, recipient)?;
    if !target.deployed || target.hp == 0 {
        return Ok(());
    }
    let mut offers = Vec::new();
    let mut cycle = false;
    for donor in &battle(c)?.combatants {
        if donor.id == recipient
            || !donor.deployed
            || donor.hp == 0
            || donor.side != target.side
            || !donor.has_ability("Symbiosis")
        {
            continue;
        }
        for unit in held(c, &donor.id) {
            if can_receive(c, recipient, &unit, cat)? {
                if chain.contains(&donor.id) {
                    cycle = true;
                } else {
                    offers.push((donor.id.clone(), unit.id));
                }
            }
        }
    }
    if offers.is_empty() && !cycle {
        return Ok(());
    }
    let donors: BTreeSet<_> = offers.iter().map(|(donor, _)| donor.clone()).collect();
    let source = if donors.len() == 1 {
        donors.first().unwrap().clone()
    } else {
        recipient.into()
    };
    let kind = if offers.is_empty() {
        "item-symbiosis-cycle"
    } else if donors.len() > 1 {
        "item-symbiosis-order"
    } else {
        "item-symbiosis"
    };
    let event_id = format!(
        "item-symbiosis:{}:{}:{}",
        c.revision + 1,
        flow(c)?.ability_events.len(),
        recipient
    );
    flow_mut(c).ability_events.push(abilities::AbilityEvent {
        id: event_id.clone(),
        combatant_id: source,
        target_id: recipient.into(),
        ability: "Symbiosis".into(),
        kind: kind.into(),
        move_name: "Held-item loss".into(),
        choices: if offers.is_empty() {
            vec!["manual".into()]
        } else {
            offers
                .into_iter()
                .map(|(_, unit)| format!("give:{unit}"))
                .collect()
        },
    });
    state_mut(c)
        .symbiosis_chains
        .insert(event_id, chain.to_vec());
    Ok(())
}

pub fn resolve(
    c: &mut Campaign,
    event: &abilities::AbilityEvent,
    choice: &str,
    cat: &Catalog,
) -> Result<()> {
    let mut chain = state_mut(c)
        .symbiosis_chains
        .remove(&event.id)
        .unwrap_or_default();
    if choice == "manual" {
        return Ok(());
    }
    let unit_id = choice
        .strip_prefix("give:")
        .ok_or_else(|| error("Choose a recorded Symbiosis item"))?;
    let item = state(c)
        .units
        .get(unit_id)
        .filter(|u| u.state == "held")
        .ok_or_else(|| error("Symbiosis donor no longer holds the offered item"))?
        .clone();
    let donor = fighter(c, &item.holder)?.clone();
    let target = fighter(c, &event.target_id)?;
    if !donor.has_ability("Symbiosis")
        || !donor.deployed
        || donor.hp == 0
        || !target.deployed
        || target.hp == 0
        || donor.id == target.id
        || donor.side != target.side
        || chain.contains(&donor.id)
        || !can_receive(c, &target.id, &item.unit, cat)?
    {
        return Err(error(
            "Symbiosis handoff is no longer eligible; GM must adjudicate the expired trigger",
        ));
    }
    let recipient = target.id.clone();
    let slot = state(c)
        .units
        .values()
        .filter(|u| u.holder == recipient && u.state == "held")
        .map(|u| u.slot)
        .max()
        .map_or(0, |slot| slot.saturating_add(1));
    let unit = state_mut(c).units.get_mut(unit_id).unwrap();
    unit.holder = recipient.clone();
    unit.slot = slot;
    record(
        c,
        unit_id,
        &donor.id,
        Some(recipient),
        "symbiosis",
        "Free ally item handoff after held-item loss",
    );
    // A donor may itself have a Symbiosis ally. Preserve the causal chain and
    // surface a repeated-donor cycle as a GM decision rather than looping forever.
    chain.push(donor.id.clone());
    offer_symbiosis(c, &donor.id, &chain, cat)?;
    Ok(())
}

fn fling_berry_hit(
    c: &mut Campaign,
    q: &QueuedAction,
    m: &serde_json::Value,
    cat: &Catalog,
) -> Result<()> {
    if m["Name"] != "Fling"
        || !fighter(c, &q.target_id)?.has_ability("Cud Chew")
        || fighter(c, &q.target_id)?.hp == 0
        || field::has(c, "Magic Room", fighter(c, &q.target_id)?)
    {
        return Ok(());
    }
    let key = format!("{}:{}", q.actor_id, q.sequence);
    let Some(unit_id) = state(c)
        .fling_moves
        .get(&key)
        .and_then(|input| input.unit_id.clone())
    else {
        return Ok(());
    };
    let Some(unit) = state(c)
        .units
        .get(&unit_id)
        .filter(|u| u.state == "ground")
        .map(|u| u.unit.clone())
    else {
        return Ok(());
    };
    let item = definition(cat, &unit)?;
    if item["Category"] != "Berry" {
        return Ok(());
    }
    let source = reviewed::source(c, q)?;
    let target = fighter(c, &q.target_id)?;
    let (effective, immune) =
        rules::type_matchup(c, &source, target, m["Type"].as_str().unwrap_or(""), cat)?;
    if immune
        || abilities::damage_immune_from(c, &source, target, m)
        || extra::source_immune(c, &source, target, effective)
    {
        return Ok(());
    }
    if !state_mut(c).fling_hits.insert(key.clone()) {
        return Ok(());
    }
    let event_id = format!("item-cud-chew:{key}");
    flow_mut(c).ability_events.push(abilities::AbilityEvent {
        id: event_id.clone(),
        combatant_id: q.target_id.clone(),
        target_id: q.target_id.clone(),
        ability: "Cud Chew".into(),
        kind: "item-cud-chew".into(),
        move_name: "Fling".into(),
        choices: vec!["activate".into()],
    });
    state_mut(c).units.get_mut(&unit_id).unwrap().state = "berry_hit_pending".into();
    state_mut(c).berry_hits.insert(event_id, unit_id);
    Ok(())
}

pub fn resolve_berry_hit(
    c: &mut Campaign,
    event: &abilities::AbilityEvent,
    choice: &str,
    cat: &Catalog,
) -> Result<()> {
    let unit_id = state_mut(c)
        .berry_hits
        .remove(&event.id)
        .ok_or_else(|| error("Thrown berry trigger expired"))?;
    let unit = state(c)
        .units
        .get(&unit_id)
        .filter(|u| u.state == "berry_hit_pending")
        .ok_or_else(|| error("Thrown berry is no longer available"))?
        .clone();
    if choice == "manual" {
        state_mut(c).units.get_mut(&unit_id).unwrap().state = "ground".into();
        return Ok(());
    }
    let p = fighter(c, &event.combatant_id)?;
    if p.hp == 0 || !p.has_ability("Cud Chew") || field::has(c, "Magic Room", p) {
        return Err(error(
            "Cud Chew's berry trigger is no longer eligible; GM must adjudicate it",
        ));
    }
    consumable_effects(
        c,
        &event.combatant_id,
        &event.combatant_id,
        &definition(cat, &unit.unit)?,
        0,
        false,
    )?;
    grant_chew(c, &event.combatant_id, &unit.unit, "hit")?;
    state_mut(c).units.get_mut(&unit_id).unwrap().state = "consumed".into();
    record(
        c,
        &unit_id,
        &unit.holder,
        Some(event.combatant_id.clone()),
        "berry_hit",
        "Cud Chew activated the thrown berry twice",
    );
    Ok(())
}

pub fn hit(c: &mut Campaign, q: &QueuedAction, m: &serde_json::Value, cat: &Catalog) -> Result<()> {
    if !enabled(c) || !state(c).initialized || decoys::intercepted(c, q) {
        return Ok(());
    }
    fling_berry_hit(c, q, m, cat)?;
    let source = reviewed::source(c, q)?;
    let target = fighter(c, &q.target_id)?.clone();
    let physical = m["Category"] == "Physical";
    if fighter(c, &q.actor_id)?.hp == 0
        || source.side == target.side
        || source.id == target.id
        || !["Physical", "Special"]
            .iter()
            .any(|category| m["Category"] == *category)
        || !source.has_any_ability(&["Magician", "Pickpocket"])
        || target.has_ability("Sticky Hold")
    {
        return Ok(());
    }
    let own = held(c, &source.id);
    if source.has_ability("Pickpocket") && (!physical || !abilities::contact(m) || !own.is_empty())
    {
        return Ok(());
    }
    let (effective, immune) =
        rules::type_matchup(c, &source, &target, m["Type"].as_str().unwrap_or(""), cat)?;
    if immune
        || abilities::damage_immune_from(c, &source, &target, m)
        || extra::source_immune(c, &source, &target, effective)
    {
        return Ok(());
    }
    let key = format!(
        "{}:{}:{}:{}:{}",
        battle(c)?.id,
        q.sequence,
        q.actor_id,
        q.target_id,
        q.special_index
    );
    if !state_mut(c).theft_seen.insert(key.clone()) {
        return Ok(());
    }
    let mut choices = BTreeMap::new();
    for stolen in held(c, &target.id)
        .into_iter()
        .filter(|u| !forms::locked_item(c, u, cat))
    {
        if own.is_empty() && can_receive(c, &source.id, &stolen, cat)? {
            let choice = format!("take:{}", choices.len());
            choices.insert(
                choice,
                TheftOffer {
                    thief_id: source.id.clone(),
                    target_id: target.id.clone(),
                    stolen_id: stolen.id,
                    outgoing_id: None,
                },
            );
        } else if source.has_ability("Magician") {
            for outgoing in own.iter().filter(|u| !forms::locked_item(c, u, cat)) {
                let mut thief_sheet = c.sheets[&source.sheet_id].clone();
                thief_sheet.ability = source.selected_ability.clone();
                let mut thief_units = own.clone();
                thief_units.retain(|u| u.id != outgoing.id);
                thief_units.push(stolen.clone());
                thief_sheet.held_items = Some(thief_units);
                let mut target_sheet = c.sheets[&target.sheet_id].clone();
                target_sheet.ability = target.selected_ability.clone();
                let mut target_units = held(c, &target.id);
                target_units.retain(|u| u.id != stolen.id);
                target_units.push(outgoing.clone());
                target_sheet.held_items = Some(target_units);
                if validate_equipment(&thief_sheet, cat).is_ok()
                    && validate_equipment(&target_sheet, cat).is_ok()
                {
                    let choice = format!("swap:{}", choices.len());
                    choices.insert(
                        choice,
                        TheftOffer {
                            thief_id: source.id.clone(),
                            target_id: target.id.clone(),
                            stolen_id: stolen.id.clone(),
                            outgoing_id: Some(outgoing.id.clone()),
                        },
                    );
                }
            }
        }
    }
    if choices.is_empty() {
        return Ok(());
    }
    let event_id = format!("item-theft:{key}");
    let mut options: Vec<_> = choices.keys().cloned().collect();
    if !own.is_empty() {
        options.push("decline".into());
    }
    flow_mut(c).ability_events.push(abilities::AbilityEvent {
        id: event_id.clone(),
        combatant_id: source.id,
        target_id: target.id,
        ability: source.selected_ability,
        kind: "item-theft".into(),
        move_name: q.move_name.clone(),
        choices: options,
    });
    state_mut(c).thefts.insert(event_id, choices);
    Ok(())
}

pub fn resolve_theft(
    c: &mut Campaign,
    event: &abilities::AbilityEvent,
    choice: &str,
    cat: &Catalog,
) -> Result<()> {
    let offers = state_mut(c)
        .thefts
        .remove(&event.id)
        .ok_or_else(|| error("Held-item theft offer expired"))?;
    if ["manual", "decline"].contains(&choice) {
        return Ok(());
    }
    let offer = offers
        .get(choice)
        .ok_or_else(|| error("Choose a recorded theft option"))?;
    let thief = fighter(c, &offer.thief_id)?.clone();
    let target = fighter(c, &offer.target_id)?.clone();
    if thief.hp == 0
        || !thief.deployed
        || thief.side == target.side
        || target.has_ability("Sticky Hold")
    {
        return Err(error(
            "Theft is no longer eligible; GM must adjudicate the expired trigger",
        ));
    }
    let stolen = state(c)
        .units
        .get(&offer.stolen_id)
        .filter(|u| u.holder == target.id && u.state == "held")
        .ok_or_else(|| error("Foe no longer holds the offered item"))?
        .clone();
    let outgoing = offer
        .outgoing_id
        .as_ref()
        .map(|id| {
            state(c)
                .units
                .get(id)
                .filter(|u| u.holder == thief.id && u.state == "held")
                .cloned()
                .ok_or_else(|| error("Thief no longer holds the offered exchange item"))
        })
        .transpose()?;
    if outgoing.is_none() && !held(c, &thief.id).is_empty() {
        return Err(error(
            "Theft requires an empty held slot or the recorded Magician exchange",
        ));
    }
    let unit = state_mut(c).units.get_mut(&offer.stolen_id).unwrap();
    unit.holder = thief.id.clone();
    unit.slot = outgoing.as_ref().map_or(0, |u| u.slot);
    if let Some(outgoing) = outgoing {
        let unit = state_mut(c).units.get_mut(&outgoing.unit.id).unwrap();
        unit.holder = target.id.clone();
        unit.slot = stolen.slot;
        record(
            c,
            &outgoing.unit.id,
            &thief.id,
            Some(target.id.clone()),
            "magician_exchange",
            "Owner exchanged held items after a hit",
        );
    }
    // Validate final ownership atomically, including temporary slot capacities.
    for p in [&thief, &target] {
        let mut sheet = c.sheets[&p.sheet_id].clone();
        sheet.ability = p.selected_ability.clone();
        sheet.held_items = Some(held(c, &p.id));
        validate_equipment(&sheet, cat)?;
    }
    record(
        c,
        &offer.stolen_id,
        &target.id,
        Some(thief.id.clone()),
        "ability_theft",
        &format!("{} after {}", event.ability, event.move_name),
    );
    after_loss(c, &target.id, cat)?;
    if offer.outgoing_id.is_some() {
        after_loss(c, &thief.id, cat)?;
    }
    Ok(())
}

pub fn finalize(c: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !enabled(c) || !state(c).initialized || state(c).finalized {
        return Ok(());
    }
    if state(c).fetches.values().any(|f| !f.returned) {
        return Err(error(
            "Resolve the round end to return Ball Fetch's protected balls before ending the battle",
        ));
    }
    // Harvest is earned by using the held berry, then resolves only at battle end.
    // Reuse the tracked unit so neither inventory counts nor provenance are duplicated.
    let regrow: Vec<_> = state(c)
        .units
        .values()
        .filter(|u| u.state == "consumed" && u.harvest_pending)
        .map(|u| (u.unit.id.clone(), u.holder.clone()))
        .collect();
    for (unit_id, holder) in regrow {
        let unit = state_mut(c).units.get_mut(&unit_id).unwrap();
        unit.state = "held".into();
        unit.harvest_pending = false;
        record(
            c,
            &unit_id,
            &holder,
            Some(holder.clone()),
            "harvest",
            "Harvest regrew the consumed held berry at battle end",
        );
    }
    let needs_policy = state(c)
        .units
        .values()
        .any(|u| u.state != "consumed" && (u.holder != u.initial_holder || u.state == "ground"));
    if needs_policy && state(c).return_policy.is_none() {
        return Err(error("GM must choose post-battle item recovery before ending a battle with surviving lost or transferred units"));
    }
    if state(c).return_policy.as_deref() == Some("return_surviving") {
        for u in state_mut(c)
            .units
            .values_mut()
            .filter(|u| u.state != "consumed")
        {
            u.holder = u.initial_holder.clone();
            u.slot = u.initial_slot;
            u.state = "held".into();
        }
    }
    let ps = c.battle.as_ref().unwrap().combatants.clone();
    for p in ps {
        let units = held(c, &p.id);
        let s = c
            .sheets
            .get_mut(&p.sheet_id)
            .ok_or_else(|| error("Missing item owner sheet"))?;
        let mut next = s.clone();
        next.held_items = None;
        for unit in units {
            let mut candidate = next.clone();
            candidate
                .held_items
                .get_or_insert_default()
                .push(unit.clone());
            if validate_equipment(&candidate, cat).is_ok() {
                next = candidate;
            } else {
                next.stored_items.get_or_insert_default().push(unit);
            }
        }
        validate_equipment(&next, cat)?;
        if s.held_items != next.held_items || s.stored_items != next.stored_items {
            s.held_items = next.held_items;
            s.stored_items = next.stored_items;
            s.revision = s
                .revision
                .checked_add(1)
                .ok_or_else(|| error("Sheet revision exhausted"))?;
        }
        let equipped_ids: BTreeSet<_> = s
            .held_items
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .map(|unit| unit.id.clone())
            .collect();
        for unit in state_mut(c)
            .units
            .values_mut()
            .filter(|unit| unit.holder == p.id && unit.state == "held")
        {
            if !equipped_ids.contains(&unit.unit.id) {
                unit.state = "stored".into();
            }
        }
    }
    state_mut(c).finalized = true;
    let b = c.battle.as_ref().unwrap().clone();
    if let Some(archived) = c.finished_battles.last_mut().filter(|b2| b2.id == b.id) {
        *archived = b;
    }
    Ok(())
}

pub fn flavor_type(flavor: &str) -> Option<&'static str> {
    Some(match flavor {
        "Bitter" => "Dark",
        "Dry" => "Ground",
        "Effervescent" => "Flying",
        "Fresh" => "Grass",
        "Frozen" => "Ice",
        "Half-eaten" => "Bug",
        "Juicy" => "Water",
        "Numbing" => "Psychic",
        "Oily" => "Fight",
        "Rotten" => "Poison",
        "Salty" => "Steel",
        "Spicy" => "Fire",
        "Sour" => "Electric",
        "Sugary" => "Fairy",
        "Tough" => "Rock",
        "Uneatable" => "Dragon",
        "Withered" => "Ghost",
        _ => return None,
    })
}
fn natural_input(c: &Campaign, id: &str) -> Result<BerryMove> {
    let unit_id = state(c)
        .natural_gift
        .get(id)
        .ok_or_else(|| error("Select a held berry for Natural Gift first"))?;
    if !held(c, id).iter().any(|u| u.id == *unit_id) {
        return Err(error("Natural Gift's selected berry is no longer held"));
    }
    let move_type = state(c)
        .units
        .get(unit_id)
        .and_then(|unit| unit.unit.flavor.as_deref())
        .and_then(flavor_type)
        .ok_or_else(|| error("Natural Gift's berry needs a GM-recorded flavor"))?;
    Ok(BerryMove {
        unit_id: unit_id.clone(),
        move_type: move_type.into(),
        ripen_bonus: 2 * i32::from(fighter(c, id)?.has_ability("Ripen")),
    })
}
fn fling_input(c: &Campaign, id: &str, cat: &Catalog) -> Result<FlingMove> {
    if held(c, id).is_empty() {
        return Ok(FlingMove::default());
    }
    if fighter(c, id)?.has_ability("Sticky Hold") {
        return Err(error("Sticky Hold prevents removing its held item with Fling; a GM must adjudicate an exception"));
    }
    if held(c, id).iter().any(|u| forms::locked_item(c, u, cat)) {
        return Err(error(
            "Mega stones cannot be thrown or removed during battle",
        ));
    }
    let unit_id = state(c)
        .fling
        .get(id)
        .ok_or_else(|| error("Select a held item for Fling first"))?;
    let unit = held(c, id)
        .into_iter()
        .find(|u| u.id == *unit_id)
        .ok_or_else(|| error("Fling's selected item is no longer held"))?;
    let item = definition(cat, &unit)?;
    let berry = item["Category"] == "Berry";
    if unit.name == "Berry Juice" {
        return Err(error("Berry Juice explicitly cannot be used in battle"));
    }
    let bonus = if berry {
        match item["TrainerPrice"].as_str() {
            Some("Common") => 1,
            Some("Uncommon") => 2,
            Some("Rare") => 3,
            _ => {
                return Err(AppError::new(
                    "manual_rule",
                    "Fling needs a reviewed berry rarity",
                ))
            }
        }
    } else {
        4
    };
    Ok(FlingMove {
        unit_id: Some(unit.id),
        item_bonus: bonus,
        ripen_bonus: 2 * i32::from(berry && fighter(c, id)?.has_ability("Ripen")),
    })
}
fn commit_fling(
    before: &Campaign,
    next: &mut Campaign,
    id: &str,
    sequence: u32,
    cat: &Catalog,
) -> Result<()> {
    let key = format!("{id}:{sequence}");
    if state(next).fling_moves.contains_key(&key) {
        return Ok(());
    }
    let input = fling_input(before, id, cat)?;
    if let Some(unit_id) = &input.unit_id {
        let berry = definition(cat, &state(before).units[unit_id].unit)?["Category"] == "Berry";
        let unit = state_mut(next)
            .units
            .get_mut(unit_id)
            .ok_or_else(|| error("Fling's item disappeared"))?;
        if unit.state != "held" || unit.holder != id {
            return Err(error("Fling's item is no longer held"));
        }
        unit.state = "ground".into();
        unit.harvest_pending = berry && fighter(before, id)?.has_ability("Harvest");
        record(
            next,
            unit_id,
            id,
            None,
            "fling",
            "Held item thrown onto the battlefield on a Fling attempt",
        );
        after_loss(next, id, cat)?;
    }
    state_mut(next).fling.remove(id);
    state_mut(next).fling_moves.insert(key, input);
    Ok(())
}
pub fn retrieve_guard(c: &Campaign, id: &str, unit_id: &str, cat: &Catalog) -> Result<()> {
    if !enabled(c) || !state(c).initialized {
        return Err(error("Upgrade held-item rules before retrieving items"));
    }
    ready(c, id)?;
    turn(c, id)?;
    sequencing::move_guard(c, id, None)?;
    let unit = state(c)
        .units
        .get(unit_id)
        .filter(|u| u.state == "ground")
        .ok_or_else(|| {
            error("Select an existing ground item; consumed items cannot be retrieved")
        })?;
    if !can_receive(c, id, &unit.unit, cat)? {
        return Err(error(
            "The retrieved item does not fit this Pokemon's held slots",
        ));
    }
    Ok(())
}

fn recyclable(item: &serde_json::Value) -> bool {
    matches!(
        item["Name"].as_str(),
        Some(
            "Oran Berry"
                | "Sitrus Berry"
                | "Pecha Berry"
                | "Rawst Berry"
                | "Aspear Berry"
                | "Cheri Berry"
                | "Chesto Berry"
                | "Persim Berry"
                | "Lum Berry"
                | "Potion"
                | "Super Potion"
                | "Hyper Potion"
                | "Antidote"
                | "Awakening"
                | "Burn Heal"
                | "Ice Heal"
                | "Paralyze Heal"
                | "Full Heal"
        )
    )
}
fn unnerve(c: &Campaign, id: &str) -> Result<bool> {
    let p = fighter(c, id)?;
    Ok(battle(c)?
        .combatants
        .iter()
        .any(|foe| foe.deployed && foe.hp > 0 && foe.side != p.side && foe.has_ability("Unnerve")))
}
fn recycle_input(c: &Campaign, id: &str, cat: &Catalog) -> Result<String> {
    if state(c).recycle_uses.get(id).copied().unwrap_or(0) >= 3 {
        return Err(error(
            "Recycle may be attempted at most three times per battle by its user",
        ));
    }
    let unit_id = state(c)
        .recycle
        .get(id)
        .ok_or_else(|| error("Select a spent item for Recycle first"))?;
    let unit = state(c)
        .units
        .get(unit_id)
        .filter(|u| u.state == "consumed")
        .ok_or_else(|| {
            error("Recycle requires an item already spent in this battle, from either side")
        })?;
    let item = definition(cat, &unit.unit)?;
    if !recyclable(&item) {
        return Err(AppError::new("manual_rule", "This item's Recycle effect still requires a reviewed definition; use explicit GM adjudication"));
    }
    if field::has(c, "Magic Room", fighter(c, id)?) {
        return Err(error("Magic Room prevents reusing items with Recycle"));
    }
    if item["Category"] == "Berry" && unnerve(c, id)? {
        return Err(error("Unnerve prevents consuming recycled berries"));
    }
    Ok(unit_id.clone())
}
fn commit_recycle(
    before: &Campaign,
    next: &mut Campaign,
    id: &str,
    sequence: u32,
    cat: &Catalog,
) -> Result<()> {
    let key = format!("{id}:{sequence}");
    if state(next).recycle_attempts.contains(&key) {
        return Ok(());
    }
    let unit_id = recycle_input(before, id, cat)?;
    state_mut(next).recycle_attempts.insert(key.clone());
    *state_mut(next).recycle_uses.entry(id.into()).or_default() += 1;
    state_mut(next).recycle.remove(id);
    if flow(next)?
        .queue
        .iter()
        .any(|q| q.actor_id == id && q.sequence == sequence && q.move_name == "Recycle")
    {
        state_mut(next).units.get_mut(&unit_id).unwrap().state = "recycle_pending".into();
        state_mut(next).recycle_moves.insert(key, unit_id.clone());
    }
    record(
        next,
        &unit_id,
        id,
        None,
        "recycle_attempt",
        "Recycle attempted; counts toward the user's three battle uses",
    );
    Ok(())
}
pub fn recycle_effect(c: &mut Campaign, q: &QueuedAction, cat: &Catalog) -> Result<bool> {
    if !enabled(c) || q.move_name != "Recycle" {
        return Ok(false);
    }
    let actor = tactics::payer(c, &q.actor_id);
    let key = format!("{actor}:{}", q.sequence);
    let unit_id = state_mut(c)
        .recycle_moves
        .remove(&key)
        .ok_or_else(|| error("Recycle's reserved spent item is unavailable"))?;
    let unit = state(c)
        .units
        .get(&unit_id)
        .filter(|u| u.state == "recycle_pending")
        .ok_or_else(|| error("Recycle's item changed before effect resolution"))?
        .unit
        .clone();
    let item = definition(cat, &unit)?;
    // This is a use of the spent item, not creation/equipping of another copy.
    state_mut(c).units.get_mut(&unit_id).unwrap().state = "consumed".into();
    if field::has(c, "Magic Room", fighter(c, &actor)?)
        || (item["Category"] == "Berry" && unnerve(c, &actor)?)
    {
        record(
            c,
            &unit_id,
            &actor,
            None,
            "recycle_blocked",
            "Item reuse prevented at effect resolution",
        );
        return Ok(true);
    }
    let recipient = if special::enabled(c) {
        flow(c)?
            .special
            .effects
            .iter()
            .find(|e| e.name == "Snatch" && e.target == q.target_id)
            .map(|e| e.source.clone())
            .unwrap_or_else(|| q.target_id.clone())
    } else {
        q.target_id.clone()
    };
    consumable_effects(c, &actor, &recipient, &item, 0, false)?;
    if item["Category"] == "Berry" {
        grant_chew(c, &actor, &unit, "recycle")?;
    }
    record(
        c,
        &unit_id,
        &actor,
        Some(recipient),
        "recycle",
        "Spent consumable reused once by Recycle; physical unit remains spent",
    );
    Ok(true)
}

pub fn guard_move(c: &Campaign, id: &str, m: &serde_json::Value, cat: &Catalog) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    if m["Name"] == "Recycle" {
        recycle_input(c, id, cat)?;
        return Ok(());
    }
    if m["Name"] == "Fling" {
        let input = fling_input(c, id, cat)?;
        if input.unit_id.is_some() && field::has(c, "Magic Room", fighter(c, id)?) {
            return Err(error("Magic Room prevents using items through Fling"));
        }
        return Ok(());
    }
    if m["Name"] != "Natural Gift" {
        return Ok(());
    }
    natural_input(c, id)?;
    let source = fighter(c, id)?;
    let resolved = field::move_data(c, source, m);
    field::guard_move(c, source, &resolved)?;
    if field::has(c, "Magic Room", source) {
        return Err(error(
            "Magic Room prevents item use, including Natural Gift's berry",
        ));
    }
    if battle(c)?
        .combatants
        .iter()
        .any(|p| p.deployed && p.hp > 0 && p.side != source.side && p.has_ability("Unnerve"))
    {
        return Err(error(
            "Unnerve prevents consuming Natural Gift's held berry",
        ));
    }
    Ok(())
}
pub fn move_data(c: &Campaign, p: &Combatant, m: &mut serde_json::Value) {
    if !enabled(c) || m["Name"] != "Natural Gift" {
        return;
    }
    let frozen = flow(c)
        .ok()
        .and_then(|f| {
            f.queue
                .iter()
                .find(|q| q.actor_id == p.id && q.move_name == "Natural Gift")
        })
        .and_then(|q| {
            state(c)
                .berry_moves
                .get(&format!("{}:{}", p.id, q.sequence))
        });
    if let Some(input) = frozen.cloned().or_else(|| natural_input(c, &p.id).ok()) {
        m["Type"] = serde_json::json!(input.move_type);
    }
}
pub fn damage_bonus(c: &Campaign, id: &str, sequence: u32) -> i32 {
    if !enabled(c) {
        return 0;
    }
    state(c)
        .berry_moves
        .get(&format!("{id}:{sequence}"))
        .map_or(0, |input| input.ripen_bonus)
        + state(c)
            .fling_moves
            .get(&format!("{id}:{sequence}"))
            .map_or(0, |input| input.item_bonus + input.ripen_bonus)
}
pub fn commit_item_move(
    before: &Campaign,
    next: &mut Campaign,
    id: &str,
    name: &str,
    sequence: u32,
    cat: &Catalog,
) -> Result<()> {
    if !enabled(before) {
        return Ok(());
    }
    if name == "Recycle" {
        return commit_recycle(before, next, id, sequence, cat);
    }
    if name == "Fling" {
        return commit_fling(before, next, id, sequence, cat);
    }
    if name != "Natural Gift" {
        return Ok(());
    }
    let input = natural_input(before, id)?;
    let key = format!("{id}:{sequence}");
    if state(next).berry_moves.contains_key(&key) {
        return Ok(());
    }
    let harvest = fighter(before, id)?.has_ability("Harvest");
    let unit = state_mut(next)
        .units
        .get_mut(&input.unit_id)
        .ok_or_else(|| error("Natural Gift's berry disappeared"))?;
    if unit.state != "held" || unit.holder != id {
        return Err(error("Natural Gift's berry is no longer held"));
    }
    unit.state = "consumed".into();
    unit.harvest_pending = harvest;
    record(
        next,
        &input.unit_id,
        id,
        None,
        "natural_gift",
        "Held berry spent on a Natural Gift attempt",
    );
    state_mut(next).natural_gift.remove(id);
    state_mut(next).berry_moves.insert(key, input);
    after_loss(next, id, cat)?;
    Ok(())
}

pub fn resolved_effects(c: &Campaign, q: &QueuedAction) -> bool {
    if !enabled(c) {
        return false;
    }
    let key = format!("{}:{}", q.actor_id, q.sequence);
    (q.move_name == "Natural Gift" && state(c).berry_moves.contains_key(&key))
        || (q.move_name == "Fling" && state(c).fling_moves.contains_key(&key))
}

fn heal(c: &mut Campaign, id: &str, amount: i32) -> Result<i32> {
    if special::healing_blocked(c, id) || fighter(c, id)?.hp == 0 {
        return Ok(0);
    }
    let debt = if special::enabled(c) {
        flow(c)?
            .special
            .effects
            .iter()
            .filter(|e| e.name == "Shadow Force" && e.target == id)
            .map(|e| e.amount)
            .sum::<i32>()
    } else {
        0
    };
    let p = fighter_mut(c, id)?;
    let actual = amount
        .max(0)
        .min((p.max_hp - debt - p.hp).max(0))
        .min((3 - p.runtime.healed).max(0));
    p.hp += actual;
    p.runtime.healed += actual;
    Ok(actual)
}

pub fn end_round(c: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !enabled(c) || !state(c).initialized || state(c).end_round == battle(c)?.round {
        return Ok(());
    }
    state_mut(c).end_round = battle(c)?.round;
    return_fetches(c, cat)?;
    let holders: Vec<_> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    for id in holders {
        if !active_effects(c, &id)? {
            continue;
        }
        for item in held(c, &id) {
            if item.name != "Leftovers" || item.uses >= 3 {
                continue;
            }
            if heal(c, &id, 1)? == 0 {
                continue;
            }
            let unit = state_mut(c).units.get_mut(&item.id).unwrap();
            unit.unit.uses += 1;
            let consumed = unit.unit.uses == 3;
            if consumed {
                unit.state = "consumed".into();
            }
            record(
                c,
                &item.id,
                &id,
                None,
                "leftovers_heal",
                "Leftovers healed one HP at round end",
            );
            if consumed {
                record(
                    c,
                    &item.id,
                    &id,
                    None,
                    "consume_leftovers",
                    "Leftovers consumed after healing three HP total",
                );
                after_loss(c, &id, cat)?;
            }
        }
    }
    Ok(())
}

fn active_effects(c: &Campaign, id: &str) -> Result<bool> {
    let p = fighter(c, id)?;
    Ok(!p.has_ability("Klutz") && !field::has(c, "Magic Room", p))
}

fn grant_chew(c: &mut Campaign, id: &str, unit: &HeldItem, source: &str) -> Result<()> {
    if !fighter(c, id)?.has_ability("Cud Chew") {
        return Ok(());
    }
    let serial = state(c)
        .chew_serial
        .checked_add(1)
        .ok_or_else(|| error("Cud Chew charge sequence exhausted"))?;
    state_mut(c).chew_serial = serial;
    state_mut(c).chew_charges.insert(
        format!("chew:{serial}"),
        ChewCharge {
            combatant_id: id.into(),
            unit: unit.clone(),
            source: source.into(),
        },
    );
    Ok(())
}

fn consumable_effects(
    c: &mut Campaign,
    id: &str,
    recipient: &str,
    item: &serde_json::Value,
    extra_healing: i32,
    held: bool,
) -> Result<bool> {
    let p = fighter(c, id)?.clone();
    let healing = item["HealthRestored"].as_i64().unwrap_or(0) as i32;
    let cure = item["Cures"].as_str().unwrap_or("");
    if healing <= 0
        && ![
            "Poison",
            "Burn 1/2",
            "Burn",
            "Frozen Solid",
            "Frozen",
            "Paralysis",
            "Sleep",
            "Confusion",
            "All Status",
        ]
        .contains(&cure)
    {
        return Err(AppError::new(
            "manual_rule",
            "This berry's effect still requires a reviewed definition",
        ));
    }
    let berry = item["Category"] == "Berry";
    let active = !held || active_effects(c, id)?;
    if active {
        heal(c, recipient, healing + extra_healing)?;
        let subject = fighter_mut(c, recipient)?;
        subject.runtime.statuses.retain(|status| !match cure {
            "Poison" => {
                status.kind == "Poison"
                    || (item["Name"] == "Antidote" && status.kind == "Badly Poisoned")
            }
            "Burn 1/2" => ["Burn", "Burn 2"].contains(&status.kind.as_str()),
            "Burn" => ["Burn", "Burn 2", "Burn 3"].contains(&status.kind.as_str()),
            "Frozen" | "Frozen Solid" => status.kind == "Frozen Solid",
            "Paralysis" | "Sleep" | "Confusion" => status.kind == cure,
            "All Status" => [
                "Burn",
                "Burn 2",
                "Burn 3",
                "Poison",
                "Badly Poisoned",
                "Frozen Solid",
                "Paralysis",
                "Sleep",
                "Confusion",
                "Flinch",
                "In Love",
            ]
            .contains(&status.kind.as_str()),
            _ => false,
        });
    }
    // Klutz negates the held berry, not a separate eating ability. Magic Room forbids use above.
    if berry && healing == 0 && p.has_ability("Cheek Pouch") {
        heal(c, id, 2)?;
    }
    Ok(active)
}

fn consume_berry(
    c: &mut Campaign,
    actor: &str,
    id: &str,
    unit_id: &str,
    extra_healing: i32,
    why: &str,
    cat: &Catalog,
) -> Result<String> {
    owner(c, actor, id)?;
    let p = fighter(c, id)?.clone();
    if p.hp <= 0 {
        return Err(error("A fainted Pokemon cannot consume its held berry"));
    }
    if field::has(c, "Magic Room", &p) {
        return Err(error("Magic Room prevents item use"));
    }
    if battle(c)?
        .combatants
        .iter()
        .any(|foe| foe.deployed && foe.hp > 0 && foe.side != p.side && foe.has_ability("Unnerve"))
    {
        return Err(error(
            "Unnerve prevents foes from consuming held or recycled berries",
        ));
    }
    let held = state(c)
        .units
        .get(unit_id)
        .filter(|unit| unit.holder == id && unit.state == "held")
        .ok_or_else(|| error("Select a berry currently held by this Pokemon"))?
        .unit
        .clone();
    let item = definition(cat, &held)?;
    if item["Category"] != "Berry" {
        return Err(error("Select a held berry"));
    }
    if held.name == "Berry Juice" {
        return Err(error("Berry Juice explicitly cannot be used in battle"));
    }
    if ![
        "Oran Berry",
        "Sitrus Berry",
        "Pecha Berry",
        "Rawst Berry",
        "Aspear Berry",
        "Cheri Berry",
        "Chesto Berry",
        "Persim Berry",
        "Lum Berry",
    ]
    .contains(&held.name.as_str())
    {
        return Err(AppError::new(
            "manual_rule",
            "This berry still requires a reviewed battle definition",
        ));
    }
    let active = consumable_effects(c, id, id, &item, extra_healing, true)?;
    grant_chew(c, id, &held, "held")?;
    let consumed = state_mut(c).units.get_mut(unit_id).unwrap();
    consumed.state = "consumed".into();
    consumed.harvest_pending = p.has_ability("Harvest");
    record(c, unit_id, id, None, "consume_berry", why);
    after_loss(c, id, cat)?;
    Ok(format!(
        "{} consumed {}; no action spent. Berry effects {}. {why}",
        p.name,
        held.name,
        if active { "applied" } else { "negated" }
    ))
}

pub fn transition(
    c: &Campaign,
    actor: &str,
    command: &ItemCommand,
    why: &str,
    dice: &[u8],
    cat: &Catalog,
) -> Result<(Campaign, String)> {
    if !enabled(c) {
        return Err(error(
            "Upgrade held-item rules before using encounter items",
        ));
    }
    if forms::enabled(c) {
        let mut ids: Vec<&String> = vec![];
        match command {
            ItemCommand::Transfer { unit_id, .. }
            | ItemCommand::Lose { unit_id, .. }
            | ItemCommand::Spend { unit_id, .. } => ids.push(unit_id),
            ItemCommand::Swap {
                first_unit_id,
                second_unit_id,
                ..
            } => {
                ids.push(first_unit_id);
                ids.push(second_unit_id);
            }
            _ => {}
        }
        if ids.iter().any(|id| {
            state(c)
                .units
                .get(*id)
                .is_some_and(|u| forms::locked_item(c, &u.unit, cat))
        }) {
            return Err(error(
                "Mega stones cannot be stolen, switched or removed during battle",
            ));
        }
    }
    battle(c)?;
    reason(why)?;
    check_count(dice, 0)?;
    if abilities::pending(c) {
        return Err(error(
            "Resolve the pending ability trigger before using an item",
        ));
    }
    let mut next = c.clone();
    initialize(&mut next, cat)?;
    let log = match command {
        ItemCommand::FailedCapture { .. } => {
            gm(c, actor)?;
            failed_capture(&mut next, command, cat)?;
            format!("GM recorded a failed wild capture and protected ball; adjudicated capture action cost is unchanged. {why}")
        }
        ItemCommand::ChewAgain { charge_id } => {
            let charge = state(&next)
                .chew_charges
                .get(charge_id)
                .ok_or_else(|| {
                    error("This Cud Chew activation was already used or is unavailable")
                })?
                .clone();
            owner(c, actor, &charge.combatant_id)?;
            let p = fighter(c, &charge.combatant_id)?;
            if p.hp <= 0 || !p.has_ability("Cud Chew") {
                return Err(error(
                    "A conscious Cud Chew holder must activate its saved berry effect",
                ));
            }
            if field::has(c, "Magic Room", p) {
                return Err(error(
                    "Magic Room prevents activating the saved berry effect",
                ));
            }
            if charge.source != "hit" && unnerve(c, &charge.combatant_id)? {
                return Err(error(
                    "Unnerve prevents consuming this held or recycled berry again",
                ));
            }
            consumable_effects(
                &mut next,
                &charge.combatant_id,
                &charge.combatant_id,
                &definition(cat, &charge.unit)?,
                0,
                false,
            )?;
            state_mut(&mut next).chew_charges.remove(charge_id);
            record(
                &mut next,
                &charge.unit.id,
                &charge.combatant_id,
                Some(charge.combatant_id.clone()),
                "cud_chew",
                why,
            );
            format!(
                "{} activated its saved {} effect; no action spent. {why}",
                p.name, charge.unit.name
            )
        }

        ItemCommand::Recycle {
            combatant_id,
            unit_id,
        } => {
            owner(c, actor, combatant_id)?;
            if flow(c)?.queue.iter().any(|q| q.actor_id == *combatant_id) {
                return Err(error(
                    "Select Recycle's spent item before declaring this Pokemon's move",
                ));
            }
            state_mut(&mut next)
                .recycle
                .insert(combatant_id.clone(), unit_id.clone());
            recycle_input(&next, combatant_id, cat)?;
            format!(
                "{} selected a spent consumable for Recycle. {why}",
                fighter(c, combatant_id)?.name
            )
        }
        ItemCommand::Spend {
            combatant_id,
            unit_id,
        } => {
            gm(c, actor)?;
            let unit = state_mut(&mut next)
                .units
                .get_mut(unit_id)
                .filter(|u| u.holder == *combatant_id && u.state == "held")
                .ok_or_else(|| error("Select a tracked unit currently held by this Pokemon"))?;
            unit.state = "consumed".into();
            unit.harvest_pending = fighter(c, combatant_id)?.has_ability("Harvest")
                && definition(cat, &unit.unit)?["Category"] == "Berry";
            record(&mut next, unit_id, combatant_id, None, "gm_spent", why);
            after_loss(&mut next, combatant_id, cat)?;
            format!("GM recorded an owned item spent after adjudication; item effects and action cost are part of that ruling. {why}")
        }

        ItemCommand::Fling {
            combatant_id,
            unit_id,
        } => {
            owner(c, actor, combatant_id)?;
            if flow(c)?.queue.iter().any(|q| q.actor_id == *combatant_id) {
                return Err(error(
                    "Select Fling's item before declaring this Pokemon's action",
                ));
            }
            if let Some(unit_id) = unit_id {
                if !held(&next, combatant_id).iter().any(|u| u.id == *unit_id) {
                    return Err(error("Select this Pokemon's held item"));
                }
                state_mut(&mut next)
                    .fling
                    .insert(combatant_id.clone(), unit_id.clone());
                fling_input(&next, combatant_id, cat)?;
            } else {
                if !held(&next, combatant_id).is_empty() {
                    return Err(error(
                        "A Pokemon holding an item must select an item for Fling",
                    ));
                }
                state_mut(&mut next).fling.remove(combatant_id);
            }
            format!(
                "{} selected its Fling ingredient. {why}",
                fighter(c, combatant_id)?.name
            )
        }
        ItemCommand::Retrieve {
            combatant_id,
            unit_id,
        } => {
            owner(c, actor, combatant_id)?;
            retrieve_guard(c, combatant_id, unit_id, cat)?;
            let slot = state(&next)
                .units
                .values()
                .filter(|u| u.holder == *combatant_id && u.state == "held")
                .map(|u| u.slot)
                .max()
                .map_or(0, |slot| slot.saturating_add(1));
            let unit = state_mut(&mut next).units.get_mut(unit_id).unwrap();
            let previous = unit.holder.clone();
            unit.holder = combatant_id.clone();
            unit.slot = slot;
            unit.state = "held".into();
            fighter_mut(&mut next, combatant_id)?.actions += 1;
            record(
                &mut next,
                unit_id,
                &previous,
                Some(combatant_id.clone()),
                "retrieve",
                why,
            );
            advance(&mut next);
            sequencing::lifecycle(&mut next, cat)?;
            format!(
                "{} retrieved a ground item; one action spent. {why}",
                fighter(c, combatant_id)?.name
            )
        }

        ItemCommand::BerryFlavor { unit_id, flavor } => {
            gm(c, actor)?;
            let unit = state(&next)
                .units
                .get(unit_id)
                .filter(|u| u.state == "held")
                .ok_or_else(|| error("Choose a currently held berry"))?;
            if definition(cat, &unit.unit)?["Category"] != "Berry"
                || unit.unit.name == "Berry Juice"
                || flavor_type(flavor).is_none()
            {
                return Err(error(
                    "Record a supported battle berry and its printed Natural Gift flavor",
                ));
            }
            state_mut(&mut next)
                .units
                .get_mut(unit_id)
                .unwrap()
                .unit
                .flavor = Some(flavor.clone());
            // Flavor details belong to the holder's private item state, not the shared log.
            format!("GM recorded a held berry's flavor. {why}")
        }
        ItemCommand::NaturalGift {
            combatant_id,
            unit_id,
        } => {
            owner(c, actor, combatant_id)?;
            if flow(c)?.queue.iter().any(|q| q.actor_id == *combatant_id) {
                return Err(error(
                    "Choose Natural Gift's berry before declaring this Pokemon's action",
                ));
            }
            let unit = held(&next, combatant_id)
                .into_iter()
                .find(|u| u.id == *unit_id)
                .ok_or_else(|| error("Select this Pokemon's held berry"))?;
            if definition(cat, &unit)?["Category"] != "Berry"
                || unit.name == "Berry Juice"
                || unit.flavor.as_deref().and_then(flavor_type).is_none()
            {
                return Err(error(
                    "Natural Gift requires a battle berry with a GM-recorded flavor",
                ));
            }
            state_mut(&mut next)
                .natural_gift
                .insert(combatant_id.clone(), unit_id.clone());
            format!(
                "{} selected its held berry for Natural Gift. {why}",
                fighter(c, combatant_id)?.name
            )
        }
        ItemCommand::ConsumeRipenedBerry {
            combatant_id,
            unit_id,
            extra_healing,
        } => {
            gm(c, actor)?;
            let p = fighter(c, combatant_id)?;
            let unit = held(&next, combatant_id)
                .into_iter()
                .find(|u| u.id == *unit_id)
                .ok_or_else(|| error("Select this Pokemon's held healing berry"))?;
            if !p.has_ability("Ripen")
                || definition(cat, &unit)?["HealthRestored"]
                    .as_i64()
                    .unwrap_or(0)
                    <= 0
                || !(0..=100).contains(extra_healing)
            {
                return Err(error("Ripen healing requires its active holder, an HP-healing berry and a GM bonus of 0 to 100 HP"));
            }
            let result = consume_berry(
                &mut next,
                actor,
                combatant_id,
                unit_id,
                *extra_healing,
                why,
                cat,
            )?;
            format!("{result}; GM Ripen healing increase {extra_healing} HP")
        }

        ItemCommand::ConsumeBerry {
            combatant_id,
            unit_id,
        } => consume_berry(&mut next, actor, combatant_id, unit_id, 0, why, cat)?,
        ItemCommand::ReturnPolicy { policy } => {
            gm(c, actor)?;
            if !["return_surviving", "keep_transfers"].contains(&policy.as_str()) {
                return Err(error("Choose a supported post-battle item policy"));
            }
            state_mut(&mut next).return_policy = Some(policy.clone());
            format!("GM selected post-battle item recovery {policy}: {why}")
        }
        ItemCommand::Transfer {
            from_id,
            to_id,
            unit_id,
            override_sticky_hold,
        } => {
            gm(c, actor)?;
            let source = fighter(c, from_id)?;
            let target = fighter(c, to_id)?;
            if forms::slot_reserved(c, to_id) {
                return Err(error(
                    "The recipient held slot is reserved by a recorded Disc or mask",
                ));
            }
            if from_id == to_id {
                return Err(error("Choose a different recipient"));
            }
            if source.has_ability("Sticky Hold") && !override_sticky_hold {
                return Err(error(
                    "Sticky Hold prevents removing, stealing or swapping this held item",
                ));
            }
            let unit = state(&next)
                .units
                .get(unit_id)
                .filter(|unit| unit.holder == *from_id && unit.state == "held")
                .ok_or_else(|| error("Source no longer holds that unit"))?
                .unit
                .clone();
            let mut capacity = c.sheets[&target.sheet_id].clone();
            capacity.ability = if target.has_ability("Gluttony") {
                "Gluttony".into()
            } else {
                target.selected_ability.clone()
            };
            capacity.held_items = Some(held(&next, to_id));
            capacity.held_items.as_mut().unwrap().push(unit);
            validate_equipment(&capacity, cat)?;
            let slot = state(&next)
                .units
                .values()
                .filter(|u| u.holder == *to_id && u.state == "held")
                .map(|u| u.slot)
                .max()
                .map_or(0, |slot| slot.saturating_add(1));
            let unit = state_mut(&mut next).units.get_mut(unit_id).unwrap();
            unit.holder = to_id.clone();
            unit.slot = slot;
            record(
                &mut next,
                unit_id,
                from_id,
                Some(to_id.clone()),
                "gm_transfer",
                why,
            );
            after_loss(&mut next, from_id, cat)?;
            format!("GM transferred held unit {unit_id} from {} to {}; Sticky Hold override {override_sticky_hold}. {why}", source.name, target.name)
        }
        ItemCommand::Swap {
            first_id,
            second_id,
            first_unit_id,
            second_unit_id,
            override_sticky_hold,
        } => {
            gm(c, actor)?;
            if forms::slot_reserved(c, first_id) || forms::slot_reserved(c, second_id) {
                return Err(error("A recorded Disc/mask occupies the held slot"));
            }
            if first_id == second_id || first_unit_id == second_unit_id {
                return Err(error("A swap requires two different holders and units"));
            }
            let first = fighter(c, first_id)?;
            let second = fighter(c, second_id)?;
            if !override_sticky_hold
                && (first.has_ability("Sticky Hold") || second.has_ability("Sticky Hold"))
            {
                return Err(error("Sticky Hold prevents swapping a held item"));
            }
            let a = state(&next)
                .units
                .get(first_unit_id)
                .filter(|u| u.holder == *first_id && u.state == "held")
                .ok_or_else(|| error("First holder no longer holds that unit"))?
                .clone();
            let b = state(&next)
                .units
                .get(second_unit_id)
                .filter(|u| u.holder == *second_id && u.state == "held")
                .ok_or_else(|| error("Second holder no longer holds that unit"))?
                .clone();
            for (holder, outgoing, incoming) in [
                (first, first_unit_id, &b.unit),
                (second, second_unit_id, &a.unit),
            ] {
                let mut capacity = c.sheets[&holder.sheet_id].clone();
                capacity.ability = if holder.has_ability("Gluttony") {
                    "Gluttony".into()
                } else {
                    holder.selected_ability.clone()
                };
                let mut units = held(&next, &holder.id);
                units.retain(|unit| unit.id != *outgoing);
                units.push(incoming.clone());
                capacity.held_items = Some(units);
                validate_equipment(&capacity, cat)?;
            }
            let unit = state_mut(&mut next).units.get_mut(first_unit_id).unwrap();
            unit.holder = second_id.clone();
            unit.slot = b.slot;
            let unit = state_mut(&mut next).units.get_mut(second_unit_id).unwrap();
            unit.holder = first_id.clone();
            unit.slot = a.slot;
            record(
                &mut next,
                first_unit_id,
                first_id,
                Some(second_id.clone()),
                "gm_swap",
                why,
            );
            record(
                &mut next,
                second_unit_id,
                second_id,
                Some(first_id.clone()),
                "gm_swap",
                why,
            );
            // Evaluate loss abilities only after both units have moved atomically.
            after_loss(&mut next, first_id, cat)?;
            after_loss(&mut next, second_id, cat)?;
            format!("GM swapped held units between {} and {}; Sticky Hold override {override_sticky_hold}. {why}", first.name, second.name)
        }
        ItemCommand::Recover {
            combatant_id,
            unit_id,
        } => {
            gm(c, actor)?;
            let target = fighter(c, combatant_id)?;
            if forms::slot_reserved(c, combatant_id) {
                return Err(error("A recorded Disc/mask occupies the held slot"));
            }
            let unit = state(&next).units.get(unit_id).filter(|u| u.state == "ground").ok_or_else(|| error("Only an existing lost unit can be recovered; consumed items cannot be recreated"))?.unit.clone();
            let mut capacity = c.sheets[&target.sheet_id].clone();
            capacity.ability = if target.has_ability("Gluttony") {
                "Gluttony".into()
            } else {
                target.selected_ability.clone()
            };
            capacity.held_items = Some(held(&next, combatant_id));
            capacity.held_items.as_mut().unwrap().push(unit);
            validate_equipment(&capacity, cat)?;
            let slot = state(&next)
                .units
                .values()
                .filter(|u| u.holder == *combatant_id && u.state == "held")
                .map(|u| u.slot)
                .max()
                .map_or(0, |slot| slot.saturating_add(1));
            let unit = state_mut(&mut next).units.get_mut(unit_id).unwrap();
            let previous = unit.holder.clone();
            unit.holder = combatant_id.clone();
            unit.slot = slot;
            unit.state = "held".into();
            record(
                &mut next,
                unit_id,
                &previous,
                Some(combatant_id.clone()),
                "gm_recovery",
                why,
            );
            format!(
                "GM recorded recovery of an existing item unit by {}. {why}",
                target.name
            )
        }
        ItemCommand::Lose {
            combatant_id,
            unit_id,
            override_sticky_hold,
        } => {
            gm(c, actor)?;
            let p = fighter(c, combatant_id)?;
            if p.has_ability("Sticky Hold") && !override_sticky_hold {
                return Err(error("Sticky Hold prevents removing this held item"));
            }
            let unit = state_mut(&mut next)
                .units
                .get_mut(unit_id)
                .filter(|unit| unit.holder == *combatant_id && unit.state == "held")
                .ok_or_else(|| error("Pokemon no longer holds that unit"))?;
            unit.state = "ground".into();
            record(&mut next, unit_id, combatant_id, None, "gm_loss", why);
            after_loss(&mut next, combatant_id, cat)?;
            format!("GM recorded held-item loss for {}; Sticky Hold override {override_sticky_hold}. {why}", p.name)
        }
    };
    extra::lifecycle(c, &mut next, cat)?;
    refresh_initiative(c, &mut next)?;
    sync_labels(&mut next);
    information::lifecycle(&mut next, cat)?;
    standalone::lifecycle(&mut next)?;
    checkpoint(c, &mut next, &log);
    Ok((next, log))
}

fn definition(cat: &Catalog, unit: &HeldItem) -> Result<serde_json::Value> {
    let item = cat
        .find("items", "AppId", &unit.item_id)
        .ok_or_else(|| error("Held item has no exact catalog identity"))?;
    if item["Name"].as_str() != Some(&unit.name) {
        return Err(error("Held item name and catalog identity disagree"));
    }
    Ok(item)
}

fn berry_effect(item: &serde_json::Value) -> Option<String> {
    if item["Category"] != "Berry" {
        return None;
    }
    if item["HealthRestored"].as_i64().is_some_and(|n| n > 0) {
        return Some("heal-hp".into());
    }
    item["Cures"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(|s| format!("cure:{}", s.to_lowercase()))
}

pub fn validate_equipment(sheet: &Sheet, cat: &Catalog) -> Result<()> {
    let items = sheet.held_items.as_deref().unwrap_or(&[]);
    let stored = sheet.stored_items.as_deref().unwrap_or(&[]);
    if stored.len() > 100 {
        return Err(error(
            "A sheet can store at most 100 individually tracked item units",
        ));
    }
    if (!items.is_empty() && sheet.kind != "pokemon")
        || items.len() > if sheet.ability == "Gluttony" { 3 } else { 1 }
    {
        return Err(error(
            "A Pokemon holds one item; Gluttony permits up to three different-effect berries",
        ));
    }
    let mut ids = BTreeSet::new();
    let mut effects = BTreeSet::new();
    for unit in items.iter().chain(stored) {
        if unit.id.is_empty()
            || unit.id.len() > 200
            || unit.origin_sheet_id.is_empty()
            || unit.origin_sheet_id.len() > 200
            || !ids.insert(&unit.id)
        {
            return Err(error(
                "Held-item units require unique identities and recorded origins",
            ));
        }
        let item = definition(cat, unit)?;
        if unit
            .flavor
            .as_deref()
            .is_some_and(|flavor| item["Category"] != "Berry" || flavor_type(flavor).is_none())
        {
            return Err(error(
                "A recorded berry flavor must use the reviewed Natural Gift table",
            ));
        }
    }
    for unit in items {
        let item = definition(cat, unit)?;
        if items.len() > 1 {
            let effect = berry_effect(&item).ok_or_else(|| {
                AppError::new(
                    "manual_rule",
                    "Gluttony requires reviewed distinct berry effects",
                )
            })?;
            if !effects.insert(effect) {
                return Err(error("Gluttony berries must have different effects"));
            }
        }
    }
    Ok(())
}

pub fn equip(sheet: &Sheet, name: &str, operation_id: &str, cat: &Catalog) -> Result<Sheet> {
    if sheet.kind != "pokemon" {
        return Err(error("Equip an owned inventory item on a Pokemon"));
    }
    let mut next = sheet.clone();
    if let Some(index) = sheet
        .stored_items
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .position(|unit| unit.id == name)
    {
        let stored = next.stored_items.as_mut().unwrap();
        let unit = stored.remove(index);
        if stored.is_empty() {
            next.stored_items = None;
        }
        next.held_items.get_or_insert_default().push(unit);
        validate_equipment(&next, cat)?;
        return Ok(next);
    }
    if sheet.inventory.get(name).copied().unwrap_or(0) < 1 {
        return Err(error(
            "Equip an owned inventory item or a recorded stored unit",
        ));
    }
    let item = cat
        .find("items", "Name", name)
        .ok_or_else(|| error("Select an exact catalog item"))?;
    let item_id = item["AppId"]
        .as_str()
        .ok_or_else(|| error("Missing catalog item identity"))?;
    next.held_items.get_or_insert_default().push(HeldItem {
        id: operation_id.into(),
        item_id: item_id.into(),
        name: name.into(),
        origin_sheet_id: sheet.id.clone(),
        uses: 0,
        flavor: None,
    });
    validate_equipment(&next, cat)?;
    *next.inventory.get_mut(name).unwrap() -= 1;
    next.inventory.retain(|_, count| *count > 0);
    Ok(next)
}

pub fn unequip(sheet: &Sheet, unit_id: &str, cat: &Catalog) -> Result<Sheet> {
    let mut next = sheet.clone();
    let units = next
        .held_items
        .as_mut()
        .ok_or_else(|| error("No equipped item"))?;
    let index = units
        .iter()
        .position(|unit| unit.id == unit_id)
        .ok_or_else(|| error("Select an equipped unit"))?;
    let unit = units.remove(index);
    definition(cat, &unit)?;
    if units.is_empty() {
        next.held_items = None;
    }
    next.stored_items.get_or_insert_default().push(unit);
    validate_equipment(&next, cat)?;
    Ok(next)
}
