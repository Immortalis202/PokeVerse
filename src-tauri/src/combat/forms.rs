//! RULING 133; MASTER 505,606 and supplied v3.0 ability/form cards.
//! Form builds and battle resources are encounter state, never replacement library sheets.
use super::*;
use serde_json::Value;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct FormState {
    pub enabled: bool,
    pub originals: BTreeMap<String, FormOriginal>,
    pub current: BTreeMap<String, FormView>,
    pub plans: BTreeMap<String, BTreeMap<String, FormBuild>>,
    pub choices: BTreeMap<String, String>,
    pub seen: BTreeSet<String>,
    pub mega_trainers: BTreeSet<String>,
    pub key_stones: BTreeSet<String>,
    pub equipment: BTreeMap<String, FormEquipment>,
    pub history: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct FormOriginal {
    pub sheet: Sheet,
    pub ability: String,
    pub hp: i32,
    pub max_hp: i32,
    pub will: i32,
    pub statuses: Vec<Status>,
    pub deployed: bool,
    pub field_slot: Option<u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct FormView {
    pub sheet: Sheet,
    pub types: Vec<String>,
    pub ability: String,
    pub label: String,
    pub source: String,
    pub mega: bool,
    pub secondary: Option<String>,
    pub partner: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct FormBuild {
    pub attributes: BTreeMap<String, i32>,
    pub socials: BTreeMap<String, i32>,
    pub skills: BTreeMap<String, i32>,
    pub moves: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct FormEquipment {
    pub kind: String,
    pub value: String,
    pub evidence: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "operation", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/bindings/")]
pub enum FormCommand {
    Prepare {
        combatant_id: String,
        species_id: String,
        build: FormBuild,
    },
    KeyStone {
        trainer_id: String,
        present: bool,
    },
    Equipment {
        combatant_id: String,
        kind: String,
        value: String,
    },
    Mega {
        combatant_id: String,
        stone: String,
    },
    Choose {
        combatant_id: String,
        target_id: String,
    },
    Type {
        combatant_id: String,
        type_name: String,
    },
    SchoolArrives {
        combatant_id: String,
    },
    RestoreCore {
        sheet_id: String,
    },
    Union {
        combatant_id: String,
        partner_id: String,
        species_id: String,
        first_ability: String,
        second_ability: String,
    },
    Missile {
        combatant_id: String,
        target_id: String,
        status: Option<String>,
    },
}
fn fighter<'a>(c: &'a Campaign, id: &str) -> Result<&'a Combatant> {
    c.battle
        .as_ref()
        .and_then(|b| b.combatants.iter().find(|p| p.id == id))
        .ok_or_else(|| error("Unknown registered form participant"))
}
pub fn owner_registered(c: &Campaign, actor: &str, id: &str) -> Result<()> {
    if actor != c.gm_id && fighter(c, id)?.owner_id != actor {
        return Err(AppError::new(
            "permission",
            "Only control your own form participant",
        ));
    }
    Ok(())
}
fn owner(c: &Campaign, actor: &str, id: &str) -> Result<()> {
    owner_registered(c, actor, id)
}
pub fn enabled(c: &Campaign) -> bool {
    c.battle
        .as_ref()
        .and_then(|b| b.flow.as_ref())
        .is_some_and(|f| f.forms.enabled)
}
fn state(c: &Campaign) -> &FormState {
    &c.battle.as_ref().unwrap().flow.as_ref().unwrap().forms
}
fn state_mut(c: &mut Campaign) -> &mut FormState {
    &mut flow_mut(c).forms
}
pub fn registry() -> &'static Value {
    static DATA: std::sync::LazyLock<Value> = std::sync::LazyLock::new(|| {
        serde_json::from_str(include_str!("../../../rules/battle-forms.json"))
            .expect("reviewed form registry")
    });
    &DATA
}
pub fn identity<'a>(c: &'a Campaign, id: &str) -> Option<&'a String> {
    enabled(c)
        .then(|| state(c).current.get(id))
        .flatten()
        .map(|v| &v.ability)
}
pub fn sheet(c: &Campaign, p: &Combatant, base: &mut Sheet) {
    if !enabled(c) {
        return;
    }
    if let Some(v) = state(c).current.get(&p.id) {
        base.species_id = v.sheet.species_id.clone();
        base.rank = v.sheet.rank.clone();
        base.stats = v.sheet.stats.clone();
        base.skills = v.sheet.skills.clone();
        base.moves = v.sheet.moves.clone();
        base.max_hp = p.max_hp;
        if let (Some(b), Some(vb)) = (base.workflow.as_mut(), v.sheet.workflow.as_ref()) {
            b.max_will = vb.max_will;
        }
    }
}
pub fn types(c: &Campaign, id: &str) -> Option<Vec<String>> {
    if !enabled(c) {
        return None;
    }
    state(c).current.get(id).map(|v| v.types.clone())
}
fn species(c: &Campaign, id: &str) -> String {
    state(c)
        .current
        .get(id)
        .and_then(|v| v.sheet.species_id.clone())
        .or_else(|| {
            fighter(c, id)
                .ok()
                .and_then(|p| c.sheets[&p.sheet_id].species_id.clone())
        })
        .unwrap_or_default()
}
fn record(c: &mut Campaign, id: &str, text: &str) {
    state_mut(c).history.push(format!("{id}: {text}"));
}
fn original(c: &mut Campaign, id: &str) -> Result<()> {
    if !state(c).originals.contains_key(id) {
        let p = fighter(c, id)?.clone();
        let mut s = c.sheets[&p.sheet_id].clone();
        s.inventory.clear();
        s.held_items = None;
        s.stored_items = None;
        s.medical = None;
        if let Some(b) = s.workflow.as_mut() {
            b.ledger.clear();
        }
        state_mut(c).originals.insert(
            id.into(),
            FormOriginal {
                sheet: s,
                ability: p.selected_ability,
                hp: p.hp,
                max_hp: p.max_hp,
                will: p.runtime.will,
                statuses: p.runtime.statuses,
                deployed: p.deployed,
                field_slot: p.field_slot,
            },
        );
    }
    Ok(())
}
fn mapped(ability: &str, from: &str) -> Vec<String> {
    registry()["links"][ability][from]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}
fn form_build(
    c: &Campaign,
    id: &str,
    target: &str,
    rank_up: usize,
    mega: bool,
    cat: &Catalog,
) -> Result<Sheet> {
    let p = fighter(c, id)?;
    let source = &c.sheets[&p.sheet_id];
    let definition = &registry()["species"][target];
    if definition.is_null() {
        return Err(AppError::new(
            "manual_rule",
            "This exact form has no reviewed mapping",
        ));
    }
    let mut s = source.clone();
    if let Some(b) = s.workflow.as_mut() {
        b.ledger.clear();
    }
    s.inventory.clear();
    s.held_items = None;
    s.stored_items = None;
    s.medical = None;
    let ri = (crate::creation::rank(&s.rank).ok_or_else(|| error("Unknown form rank"))? + rank_up)
        .min(7);
    s.rank = crate::creation::RANKS[ri].into();
    let plan = state(c).plans.get(id).and_then(|p| p.get(target));
    let base_definition = &registry()["species"][source.species_id.as_deref().unwrap_or("")];
    let allocations: BTreeMap<String, i32> = match plan {
        Some(plan) => plan.attributes.clone(),
        None => crate::creation::ATTRS
            .into_iter()
            .map(|a| {
                let value = source
                    .workflow
                    .as_ref()
                    .map(|b| b.allocations.get(a).copied().unwrap_or(0))
                    .unwrap_or_else(|| {
                        (source.stats.get(a).copied().unwrap_or(0)
                            - base_definition["stats"][a].as_i64().unwrap_or(0) as i32)
                            .max(0)
                    });
                (a.into(), value)
            })
            .collect(),
    };
    if allocations
        .keys()
        .any(|a| !crate::creation::ATTRS.contains(&a.as_str()))
        || allocations.values().any(|n| !(0..=14).contains(n))
        || allocations.values().sum::<i32>() > [0, 2, 4, 6, 8, 10, 10, 14][ri]
    {
        return Err(error(
            "Prepare form attributes within the rank point budget",
        ));
    }
    for a in crate::creation::ATTRS {
        let base = definition["stats"][a]
            .as_i64()
            .ok_or_else(|| error("Missing form baseline"))? as i32;
        let limit = definition["limits"][a]
            .as_i64()
            .ok_or_else(|| error("Missing form limit"))? as i32
            + if ri == 7 { 2 } else { 0 };
        let score = base + allocations.get(a).copied().unwrap_or(0);
        if score > limit {
            return Err(error(
                "Prepare a different rank point distribution for this form's limits",
            ));
        }
        s.stats.insert(a.into(), score);
    }
    if let Some(plan) = plan {
        if !plan.socials.is_empty() {
            if plan
                .socials
                .keys()
                .any(|k| !crate::creation::SOCIALS.contains(&k.as_str()))
                || plan
                    .socials
                    .values()
                    .any(|n| *n < 0 || *n > if ri == 7 { 6 } else { 4 })
                || plan.socials.values().sum::<i32>() > [0, 2, 4, 6, 8, 10, 10, 14][ri]
            {
                return Err(error("Invalid form social allocation"));
            }
            for a in crate::creation::SOCIALS {
                s.stats
                    .insert(a.into(), 1 + plan.socials.get(a).copied().unwrap_or(0));
            }
        }
        if !plan.skills.is_empty() {
            if plan.skills.iter().any(|(k, n)| {
                !source.skills.contains_key(k)
                    && ![
                        "Brawl",
                        "Channel",
                        "Clash",
                        "Evasion",
                        "Alert",
                        "Athletic",
                        "Nature",
                        "Stealth",
                        "Charm",
                        "Etiquette",
                        "Intimidate",
                        "Perform",
                        "Empathy",
                        "Throw",
                        "Weapons",
                        "Crafts",
                        "Lore",
                        "Medicine",
                        "Science",
                    ]
                    .contains(&k.as_str())
                    || *n < 0
                    || *n > [1, 2, 3, 4, 5, 5, 5, 5][ri]
            }) || plan.skills.values().sum::<i32>() > [5, 10, 14, 17, 19, 20, 22, 25][ri]
            {
                return Err(error("Invalid form skill allocation"));
            }
            s.skills = plan.skills.clone();
        }
        if !plan.moves.is_empty() {
            if !mega
                && (plan
                    .moves
                    .iter()
                    .filter(|m| !source.moves.contains(m))
                    .count()
                    > 2
                    || source
                        .moves
                        .iter()
                        .filter(|m| !plan.moves.contains(m))
                        .count()
                        > 2)
            {
                return Err(error("Ordinary forms share their moves, with at most two reviewed form-specific differences"));
            }
            let entry = cat
                .find("species", "AppId", target)
                .ok_or_else(|| error("Unknown form"))?;
            for name in &plan.moves {
                if cat.find("moves", "Name", name).is_none()
                    || (!source.moves.contains(name)
                        && !entry["Moves"].as_array().is_some_and(|m| {
                            m.iter().any(|m| {
                                m["Name"] == *name
                                    && m["Learned"]
                                        .as_str()
                                        .and_then(crate::creation::rank)
                                        .is_some_and(|r| r <= ri)
                            })
                        }))
                {
                    return Err(error(
                        "New form moves must be exact learned moves available at this rank",
                    ));
                }
            }
            let insight = if mega {
                s.stats["Insight"]
            } else {
                s.stats["Insight"].max(source.stats.get("Insight").copied().unwrap_or(0))
            };
            if plan.moves.len() > insight as usize + 3 {
                return Err(error("Form move capacity exceeded"));
            }
            s.moves = plan.moves.clone();
        }
    }
    let hp_attribute = if p.rules_profile.hp == "highest_limit"
        && definition["limits"]["Insight"].as_i64() > definition["limits"]["Vitality"].as_i64()
    {
        "Insight"
    } else {
        "Vitality"
    };
    let bonus = if ri >= 6 { 3 } else { 0 };
    s.max_hp = definition["base_hp"]
        .as_i64()
        .ok_or_else(|| error("Missing form HP"))? as i32
        + s.stats[hp_attribute]
        + bonus;
    if let Some(b) = s.workflow.as_mut() {
        b.max_will = s.stats["Insight"] + 3 + bonus;
    }
    s.species_id = Some(target.into());
    Ok(s)
}
fn set_form(
    c: &mut Campaign,
    id: &str,
    target: &str,
    source: &str,
    resources: (bool, usize, bool),
    cat: &Catalog,
) -> Result<()> {
    let (restore, rank_up, mega) = resources;
    if union_active(c, id) && source != "As One" {
        return Err(AppError::new(
            "manual_rule",
            "Nested changes of a joint As One body require an explicit GM form ruling",
        ));
    }
    let mut s = form_build(c, id, target, rank_up, mega, cat)?;
    original(c, id)?;
    let p = fighter(c, id)?.clone();
    let definition = &registry()["species"][target];
    let ability = if mega || source == "Tera Shift" {
        definition["abilities"][0]
            .as_str()
            .unwrap_or("")
            .to_string()
    } else {
        p.selected_ability.clone()
    };
    let hp = if mega {
        s.max_hp
    } else {
        s.max_hp.max(p.max_hp)
    };
    s.max_hp = hp;
    if !mega {
        if let Some(b) = s.workflow.as_mut() {
            b.max_will = b.max_will.max(
                c.sheets[&p.sheet_id]
                    .workflow
                    .as_ref()
                    .map_or(p.runtime.will, |b| b.max_will),
            );
        }
    }
    let max_will = s.workflow.as_ref().map_or(p.runtime.will, |b| b.max_will);
    state_mut(c).current.insert(
        id.into(),
        FormView {
            sheet: s,
            types: definition["types"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect(),
            ability: ability.clone(),
            label: definition["name"].as_str().unwrap_or("").into(),
            source: source.into(),
            mega,
            secondary: None,
            partner: None,
        },
    );
    ability_changes::clear_assignment(c, id);
    if ability != p.selected_ability {
        flow_mut(c).battlefield.known_active.remove(id);
        flow_mut(c).extra.active.remove(id);
    }
    let p = fighter_mut(c, id)?;
    p.max_hp = hp;
    if restore {
        p.hp = hp;
        if mega || ["Power Construct", "Tera Shift"].contains(&source) {
            p.runtime.will = max_will;
            p.runtime.statuses.clear();
            p.conditions.clear();
        } else if source == "Shields Down" {
            p.runtime.statuses.clear();
            p.conditions.clear();
        }
    } else {
        p.hp = p.hp.min(hp);
    }
    p.selected_ability = ability;
    record(c, id, &format!("{source}: exact form {target}"));
    Ok(())
}
fn enqueue(c: &mut Campaign, id: &str, kind: &str, choices: Vec<String>) -> Result<()> {
    if choices.is_empty() {
        return Ok(());
    }
    let event_id = format!("form:{kind}:{id}:{}", battle(c)?.round);
    if state(c).seen.contains(&event_id) {
        return Ok(());
    }
    state_mut(c).seen.insert(event_id.clone());
    let p = fighter(c, id)?.clone();
    flow_mut(c).ability_events.push(abilities::AbilityEvent {
        id: event_id,
        combatant_id: id.into(),
        target_id: id.into(),
        ability: p.selected_ability,
        kind: format!("form-{kind}"),
        move_name: String::new(),
        choices,
    });
    Ok(())
}
fn same_species(c: &Campaign, id: &str, stem: &str) -> bool {
    species(c, id)
        .split_once(':')
        .is_some_and(|(_, s)| s == stem)
}
pub fn initialize(c: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let ps = battle(c)?.combatants.clone();
    for p in ps {
        if let Some(form) = c.form_memory.get(&p.sheet_id).cloned() {
            if !state(c).current.contains_key(&p.id) {
                set_form(c, &p.id, &form, "Shields Down", (false, 0, false), cat)?;
            }
        }
    }
    Ok(())
}
pub fn guard_move(c: &Campaign, id: &str, m: &Value) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    if m["Name"] == "Fling" && slot_reserved(c, id) {
        return Err(AppError::new(
            "manual_rule",
            "A catalog-absent attested Disc or mask has no reviewed Fling unit definition",
        ));
    }
    if let Some(v) = state(c).current.get(id) {
        if v.source == "Stance Change" {
            let blade = v.sheet.species_id.as_deref() == Some("0681F1:aegislash-blade-form");
            if blade == (m["Category"] == "Support") {
                return Err(error(
                    "This move category is forbidden by the selected Stance Change form",
                ));
            }
        }
    }
    Ok(())
}
pub fn move_data(c: &Campaign, p: &Combatant, m: &mut Value) {
    if !enabled(c) {
        return;
    }
    if m["Name"] == "Aura Wheel"
        && state(c)
            .choices
            .get(&format!("hunger:{}", p.id))
            .is_some_and(|v| v == "Hangry")
    {
        m["Type"] = "Dark".into();
    }
}
pub fn status_immune(c: &Campaign, p: &Combatant) -> bool {
    enabled(c) && p.has_ability("Shields Down") && same_species(c, &p.id, "minior-core")
}
pub fn stat_bonus(c: &Campaign, p: &Combatant, key: &str) -> i32 {
    if !enabled(c) {
        return 0;
    }
    let mut bonus = 0;
    if p.has_ability("Embody Aspect") {
        if let Some(mask) = state(c).equipment.get(&p.id).filter(|m| m.kind == "mask") {
            if key
                == if ["Teal", "Wellspring"].contains(&mask.value.as_str()) {
                    "Dexterity"
                } else {
                    "Strength"
                }
            {
                bonus += 2;
            }
        }
    }
    if state(c)
        .choices
        .values()
        .any(|v| v == &format!("commander:{}", p.id))
    {
        bonus += 2;
    }
    bonus
}
pub fn tera_shell(c: &Campaign, p: &Combatant, m: &Value) -> bool {
    enabled(c)
        && p.has_ability("Tera Shell")
        && species(c, &p.id).starts_with("1024")
        && p.hp == p.max_hp
        && ["Physical", "Special"].contains(&m["Category"].as_str().unwrap_or(""))
}
pub fn damage_guard(c: &mut Campaign, id: &str, amount: i32) -> Result<i32> {
    if !enabled(c) || amount <= 0 {
        return Ok(amount);
    }
    let p = fighter(c, id)?.clone();
    if p.has_ability("Ice Face") && same_species(c, id, "eiscue") {
        let key = format!("ice:{id}");
        let used = state(c)
            .choices
            .get(&key)
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);
        let absorbed = amount.min((2 - used).max(0));
        state_mut(c)
            .choices
            .insert(key, (used + absorbed).to_string());
        if used + absorbed >= 2 {
            enqueue(c, id, "ice", mapped("Ice Face", &species(c, id)))?;
        }
        return Ok(amount - absorbed);
    }
    Ok(amount)
}
pub fn damage_redirect(c: &Campaign, id: &str) -> Option<String> {
    if !enabled(c) {
        return None;
    }
    state(c)
        .choices
        .get(&format!("commander:{id}"))
        .and_then(|v| v.strip_prefix("commander:"))
        .filter(|target| fighter(c, target).is_ok_and(|p| p.deployed && p.hp > 0))
        .map(str::to_owned)
}
pub fn blocks_recall(c: &Campaign, id: &str) -> bool {
    enabled(c)
        && state(c)
            .choices
            .values()
            .any(|v| v == &format!("commander:{id}"))
}
pub fn joined_component(c: &Campaign, id: &str) -> bool {
    enabled(c)
        && state(c)
            .current
            .values()
            .any(|v| v.partner.as_deref() == Some(id))
}
pub fn union_active(c: &Campaign, id: &str) -> bool {
    enabled(c)
        && state(c)
            .current
            .get(id)
            .is_some_and(|v| v.source == "As One")
}
pub fn stat_cap(c: &Campaign, p: &Combatant) -> i32 {
    if enabled(c) && state(c).current.get(&p.id).is_some_and(|v| v.mega) {
        999
    } else {
        10
    }
}
pub fn slot_reserved(c: &Campaign, id: &str) -> bool {
    enabled(c) && state(c).equipment.contains_key(id)
}
pub fn locked_item(c: &Campaign, unit: &items::HeldItem, cat: &Catalog) -> bool {
    enabled(c)
        && cat
            .find("items", "AppId", &unit.item_id)
            .is_some_and(|v| v["Category"] == "MegaStone")
}
pub fn zero_active(c: &Campaign) -> bool {
    enabled(c)
        && c.battle.as_ref().is_some_and(|b| {
            b.active
                && b.combatants.iter().any(|p| {
                    p.deployed
                        && p.hp > 0
                        && p.has_ability("Teraform Zero")
                        && species(c, &p.id).starts_with("1024")
                })
        })
}
pub fn redirect_losses(before: &Campaign, c: &mut Campaign) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let Some(old) = &before.battle else {
        return Ok(());
    };
    for p in &old.combatants {
        let Some(target) = damage_redirect(c, &p.id) else {
            continue;
        };
        let hp = fighter(c, &p.id)?.hp;
        let lost = (p.hp - hp).max(0);
        if lost > 0 {
            fighter_mut(c, &p.id)?.hp = p.hp;
            let receiver = fighter_mut(c, &target)?;
            receiver.hp = (receiver.hp - lost).max(0);
            record(
                c,
                &p.id,
                "Commander redirected damage to the allied Dondozo",
            );
        }
    }
    Ok(())
}
pub fn lifecycle(before: &Campaign, c: &mut Campaign, a: &Action, cat: &Catalog) -> Result<()> {
    if !enabled(c) || matches!(a, Action::RollbackBattle { .. }) {
        return Ok(());
    }
    if !c.battle.as_ref().is_some_and(|b| b.active) {
        return Ok(());
    }
    initialize(c, cat)?;
    let round = battle(c)?.round;
    let boundary = before.battle.as_ref().is_some_and(|b| b.round < round);
    let ps = battle(c)?.combatants.clone();
    for p in ps {
        let prev = before
            .battle
            .as_ref()
            .and_then(|b| b.combatants.iter().find(|v| v.id == p.id));
        if let Some(old) = prev {
            if p.hp > old.hp {
                state_mut(c).choices.remove(&format!("school:{}", p.id));
                state_mut(c)
                    .seen
                    .insert(format!("school-cancelled:{}:{round}", p.id));
            }
            if p.hp < old.hp {
                state_mut(c).choices.remove(&format!("illusion:{}", p.id));
                if p.has_ability("Gulp Missile") && !same_species(c, &p.id, "cramorant") {
                    let source = match a {
                        Action::Damage { .. } => flow(before)
                            .ok()
                            .and_then(|f| f.queue.first())
                            .map(|q| q.actor_id.clone()),
                        _ => None,
                    };
                    if let Some(source) =
                        source.filter(|s| fighter(c, s).is_ok_and(|s| s.side != p.side))
                    {
                        enqueue(c, &p.id, "missile", vec![source])?;
                    }
                }
            }
            if old.deployed && !p.deployed && p.has_ability("Zero to Hero") {
                enqueue(c, &p.id, "hero", mapped("Zero to Hero", &species(c, &p.id)))?;
            }
        }
        if p.hp == 0 || !p.deployed {
            state_mut(c).choices.remove(&format!("school:{}", p.id));
            continue;
        }
        let entered = prev.is_none_or(|old| !old.deployed || old.hp == 0);
        if entered {
            if p.has_ability("Imposter") {
                enqueue(
                    c,
                    &p.id,
                    "imposter",
                    battle(c)?
                        .combatants
                        .iter()
                        .filter(|v| v.deployed && v.hp > 0 && v.id != p.id)
                        .map(|v| v.id.clone())
                        .collect(),
                )?;
            }
            if p.has_ability("Illusion") {
                enqueue(
                    c,
                    &p.id,
                    "illusion",
                    battle(c)?
                        .combatants
                        .iter()
                        .filter(|v| v.side == p.side && v.owner_id == p.owner_id && v.id != p.id)
                        .map(|v| v.id.clone())
                        .collect(),
                )?;
            }
            if p.has_ability("Commander") {
                let ids = battle(c)?
                    .combatants
                    .iter()
                    .filter(|v| {
                        v.side == p.side
                            && v.deployed
                            && v.hp > 0
                            && c.sheets[&v.sheet_id]
                                .species_id
                                .as_deref()
                                .is_some_and(|s| s.ends_with(":dondozo"))
                    })
                    .map(|v| v.id.clone())
                    .collect();
                enqueue(c, &p.id, "commander", ids)?;
            }
        }
        if p.hp * 2 <= p.max_hp {
            if p.has_ability("Shields Down") {
                enqueue(c, &p.id, "core", mapped("Shields Down", &species(c, &p.id)))?;
            }
            if p.has_ability("Schooling")
                && same_species(c, &p.id, "wishiwashi")
                && !state(c)
                    .seen
                    .contains(&format!("school-cancelled:{}:{round}", p.id))
            {
                state_mut(c)
                    .choices
                    .entry(format!("school:{}", p.id))
                    .or_insert(round.to_string());
            }
        } else {
            state_mut(c).choices.remove(&format!("school:{}", p.id));
        }
        if entered || boundary {
            if p.has_ability("Stance Change") {
                let mut choices = mapped("Stance Change", &species(c, &p.id));
                choices.push(species(c, &p.id));
                choices.sort();
                choices.dedup();
                enqueue(c, &p.id, "stance", choices)?;
            }
            if p.has_ability("Tera Shift") {
                let mut options = mapped("Tera Shift", &species(c, &p.id));
                options.push("decline".into());
                enqueue(c, &p.id, "tera", options)?;
            }
        }
        if boundary {
            if p.has_any_ability(&["Power Construct", "Zen Mode"])
                && prev.is_some_and(|v| v.hp > 0 && v.hp * 2 <= v.max_hp)
            {
                enqueue(
                    c,
                    &p.id,
                    "round",
                    mapped(&p.selected_ability, &species(c, &p.id)),
                )?;
            }
            if p.has_ability("Hunger Switch") {
                let key = format!("hunger:{}", p.id);
                let next = if state(c).choices.get(&key).is_some_and(|s| s == "Hangry") {
                    "Full-Belly"
                } else {
                    "Hangry"
                };
                state_mut(c).choices.insert(key, next.into());
                record(c, &p.id, next);
            }
        }
        let exposure = format!("ice-exposure:{}", p.id);
        if p.has_ability("Ice Face")
            && same_species(c, &p.id, "eiscue-no-ice-form")
            && matches!(
                field::weather(c, Some(&p)),
                Some(field::Weather::Hail | field::Weather::Snow)
            )
        {
            if !state(c).choices.contains_key(&exposure) {
                state_mut(c).choices.insert(
                    exposure.clone(),
                    if boundary || entered {
                        round
                    } else {
                        round + 1
                    }
                    .to_string(),
                );
            }
            if boundary
                && state(c)
                    .choices
                    .get(&exposure)
                    .and_then(|s| s.parse::<u32>().ok())
                    .is_some_and(|start| start < round)
                && prev.is_some_and(|v| v.deployed && v.hp > 0)
            {
                enqueue(
                    c,
                    &p.id,
                    "restore-ice",
                    mapped("Ice Face", &species(c, &p.id)),
                )?;
            }
        } else {
            state_mut(c).choices.remove(&exposure);
        }
        if p.has_ability("Teraform Zero") {
            clear_field(c);
        }
    }
    Ok(())
}
fn clear_field(c: &mut Campaign) {
    let f = flow_mut(c);
    f.field_effects.clear();
    f.battlefield.rules.clear();
    f.battlefield.weather = None;
    f.battlefield.natural = None;
    f.battlefield.events.clear();
}
pub fn before_end(c: &mut Campaign) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let forms = state(c).current.clone();
    for (id, v) in forms {
        let old = state(c).originals[&id].clone();
        if v.source == "Shields Down" {
            c.form_memory.insert(
                old.sheet.id.clone(),
                v.sheet.species_id.clone().unwrap_or_default(),
            );
        }
        let p = fighter_mut(c, &id)?;
        p.max_hp = old.max_hp;
        if v.mega {
            p.hp = if p.hp == 0 { 0 } else { old.hp };
            p.runtime.will = old.will;
            p.runtime.statuses = old.statuses;
            p.conditions = old.sheet.conditions.clone();
        } else {
            p.hp = p.hp.min(old.max_hp);
        }
        p.selected_ability = old.ability;
        p.runtime
            .ability
            .flags
            .retain(|f| !f.starts_with("as-one:"));
        if let Some(partner) = &v.partner {
            let original = state(c).originals[partner].clone();
            let p = fighter_mut(c, partner)?;
            p.deployed = original.deployed;
            p.field_slot = original.field_slot;
        }
    }
    Ok(())
}
fn transform(c: &mut Campaign, id: &str, target: &str, cat: &Catalog) -> Result<()> {
    let target = fighter(c, target)?.clone();
    if !target.deployed || target.hp == 0 || id == target.id {
        return Err(error("Choose another visible active Pokemon"));
    }
    original(c, id)?;
    let mut copied = rules::base_sheet(c, &target)?;
    copied.inventory.clear();
    copied.held_items = None;
    copied.stored_items = None;
    copied.medical = None;
    if !copied.moves.iter().any(|m| m == "Transform") {
        copied.moves.push("Transform".into());
    }
    let types = field::live_types(c, &target, cat)?;
    let p = fighter(c, id)?.clone();
    let original_sheet = &c.sheets[&p.sheet_id];
    copied.rank = original_sheet.rank.clone();
    copied.skills = original_sheet.skills.clone();
    copied.workflow = original_sheet.workflow.clone();
    for key in crate::creation::ATTRS {
        if let Some(value) = copied.stats.get_mut(key) {
            *value -= stat_channels::permanent(c, &target, key);
        }
    }
    stat_channels::copy_transform(c, &target, id)?;
    ability_changes::clear_assignment(c, id);
    copied.id = p.sheet_id.clone();
    copied.owner_id = p.owner_id.clone();
    copied.max_hp = p.max_hp;
    state_mut(c).current.insert(
        id.into(),
        FormView {
            sheet: copied,
            types,
            ability: target.selected_ability.clone(),
            label: target.name.clone(),
            source: "Transform".into(),
            mega: false,
            secondary: None,
            partner: None,
        },
    );
    fighter_mut(c, id)?.selected_ability = target.selected_ability;
    fighter_mut(c, id)?.runtime.modifiers = target.runtime.modifiers;
    fighter_mut(c, id)?.runtime.ability.counters = target.runtime.ability.counters;
    fighter_mut(c, id)?
        .runtime
        .ability
        .flags
        .retain(|f| !f.starts_with("as-one:"));
    fighter_mut(c, id)?.runtime.ability.flags.extend(
        target
            .runtime
            .ability
            .flags
            .iter()
            .filter(|f| f.starts_with("as-one:"))
            .cloned(),
    );
    record(
        c,
        id,
        "Transform copied attributes, ability, modifiers and moves for this scene",
    );
    Ok(())
}
pub fn resolve(
    c: &mut Campaign,
    e: &abilities::AbilityEvent,
    choice: &str,
    cat: &Catalog,
) -> Result<()> {
    let id = &e.combatant_id;
    if choice == "decline" || choice == "manual" {
        record(
            c,
            id,
            "Recorded GM form ruling or declined optional transformation",
        );
        return Ok(());
    }
    match e.kind.as_str() {
        "form-imposter" => transform(c, id, choice, cat)?,
        "form-illusion" => {
            state_mut(c)
                .choices
                .insert(format!("illusion:{id}"), choice.into());
        }
        "form-commander" => {
            state_mut(c)
                .choices
                .insert(format!("commander:{id}"), format!("commander:{choice}"));
        }
        "form-missile" => {
            return Err(AppError::new(
                "manual_rule",
                "Record Gulp Missile's unspecified added effect with the dedicated form command",
            ));
        }
        _ => {
            let p = fighter(c, id)?.clone();
            let heal = ["form-core", "form-tera"].contains(&e.kind.as_str())
                || p.has_ability("Power Construct");
            set_form(c, id, choice, &p.selected_ability, (heal, 0, false), cat)?;
            if e.kind == "form-core" {
                c.form_memory.insert(p.sheet_id, choice.into());
            }
            if e.kind == "form-restore-ice" {
                state_mut(c).choices.remove(&format!("ice:{id}"));
            }
        }
    }
    Ok(())
}
pub fn move_effect(c: &mut Campaign, q: &QueuedAction, cat: &Catalog) -> Result<bool> {
    if q.move_name != "Transform" {
        return Ok(false);
    }
    if !enabled(c) {
        return Err(AppError::new(
            "manual_rule",
            "Transform requires the reviewed form engine",
        ));
    }
    let target = state(c)
        .choices
        .get(&format!("transform:{}", q.actor_id))
        .cloned()
        .ok_or_else(|| error("Choose the visible Transform model before attempting the move"))?;
    transform(c, &q.actor_id, &target, cat)?;
    Ok(true)
}
pub fn hit(
    c: &mut Campaign,
    q: &QueuedAction,
    m: &Value,
    cat: &Catalog,
    before_hp: i32,
) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let p = fighter(c, &q.actor_id)?.clone();
    if p.has_ability("Gulp Missile")
        && ["Surf", "Dive"].contains(&q.move_name.as_str())
        && fighter(c, &q.target_id)?.hp < before_hp
        && same_species(c, &p.id, "cramorant")
    {
        let options = mapped("Gulp Missile", &species(c, &p.id));
        if let Some(target) = options.get(usize::from(p.hp * 2 <= p.max_hp)) {
            set_form(c, &p.id, target, "Gulp Missile", (false, 0, false), cat)?;
        }
    }
    let _ = m;
    Ok(())
}
pub fn transition(
    c: &Campaign,
    actor: &str,
    command: &FormCommand,
    why: &str,
    dice: &[u8],
    cat: &Catalog,
) -> Result<(Campaign, String)> {
    if let FormCommand::RestoreCore { sheet_id } = command {
        if ![
            ENGINE,
            GROUP_RULES_ENGINE,
            ITEM_COMPLETION_ENGINE,
            INTERACTION_ENGINE,
            STANDALONE_ENGINE,
            INFORMATION_ENGINE,
            FORM_ENGINE,
        ]
        .contains(&c.engine.as_str())
        {
            return Err(error("Adopt the reviewed form engine between battles"));
        }
        gm(c, actor)?;
        reason(why)?;
        check_count(dice, 0)?;
        if c.battle.as_ref().is_some_and(|b| {
            b.active
                && b.combatants
                    .iter()
                    .any(|p| p.sheet_id == *sheet_id && p.deployed)
        }) {
            return Err(error(
                "Record release and later recapture outside active deployment",
            ));
        }
        if !c.form_memory.contains_key(sheet_id) {
            return Err(error("No persistent Core form to restore"));
        }
        let mut next = c.clone();
        next.form_memory.remove(sheet_id);
        let log = format!("GM confirmed Minior release and later recapture: {why}");
        checkpoint(c, &mut next, &log);
        return Ok((next, log));
    }
    if !enabled(c) {
        return Err(AppError::new(
            "version",
            "Forms require a new engine 0.19 encounter",
        ));
    }
    reason(why)?;
    let mut next = c.clone();
    match command {
        FormCommand::Prepare {
            combatant_id,
            species_id,
            build,
        } => {
            owner(c, actor, combatant_id)?;
            state_mut(&mut next)
                .plans
                .entry(combatant_id.clone())
                .or_default()
                .insert(species_id.clone(), build.clone());
            let p = fighter(c, combatant_id)?;
            let schooling = p.has_ability("Schooling")
                && mapped("Schooling", &species(c, combatant_id)).contains(species_id);
            let mega = registry()["mega"]
                .as_object()
                .unwrap()
                .values()
                .any(|m| m["form"] == *species_id && m["base"] == species(c, combatant_id));
            form_build(
                &next,
                combatant_id,
                species_id,
                if schooling { 2 } else { 0 },
                mega || p.has_ability("As One"),
                cat,
            )?;
        }
        FormCommand::KeyStone {
            trainer_id,
            present,
        } => {
            gm(c, actor)?;
            if c.sheets.get(trainer_id).is_none_or(|s| s.kind != "trainer") {
                return Err(error("Choose a campaign trainer"));
            }
            if *present {
                state_mut(&mut next).key_stones.insert(trainer_id.clone());
            } else {
                state_mut(&mut next).key_stones.remove(trainer_id);
            }
        }
        FormCommand::Equipment {
            combatant_id,
            kind,
            value,
        } => {
            gm(c, actor)?;
            let p = fighter(c, combatant_id)?;
            if !(kind == "disc"
                && [
                    "Normal", "Fire", "Water", "Electric", "Grass", "Ice", "Fight", "Poison",
                    "Ground", "Flying", "Psychic", "Bug", "Rock", "Ghost", "Dragon", "Dark",
                    "Steel", "Fairy",
                ]
                .contains(&value.as_str())
                || kind == "mask"
                    && ["Teal", "Wellspring", "Hearthflame", "Cornerstone"]
                        .contains(&value.as_str()))
            {
                return Err(error("Select a reviewed Disc type or mask"));
            }
            let required = if kind == "disc" {
                "RKS System"
            } else {
                "Embody Aspect"
            };
            if !p.has_ability(required) {
                return Err(error("The selected ability does not use this equipment"));
            }
            if items::held(c, combatant_id).iter().any(|_| true) {
                return Err(error(
                    "A catalog-absent Disc/mask requires an otherwise empty held slot",
                ));
            }
            state_mut(&mut next).equipment.insert(
                combatant_id.clone(),
                FormEquipment {
                    kind: kind.clone(),
                    value: value.clone(),
                    evidence: why.into(),
                },
            );
            if kind == "disc" {
                set_type(&mut next, combatant_id, value)?;
            }
        }
        FormCommand::Mega {
            combatant_id,
            stone,
        } => {
            owner(c, actor, combatant_id)?;
            ready(c, combatant_id)?;
            turn(c, combatant_id)?;
            let p = fighter(c, combatant_id)?;
            let trainer = c.sheets[&p.sheet_id]
                .trainer_id
                .as_ref()
                .ok_or_else(|| error("Mega Evolution requires a linked trainer"))?;
            if !state(c).key_stones.contains(trainer) || state(c).mega_trainers.contains(trainer) {
                return Err(error(
                    "Trainer needs a recorded Key Stone and an unused Mega Evolution this battle",
                ));
            }
            let mapping = &registry()["mega"][stone];
            if mapping["base"] != species(c, combatant_id)
                || !items::held(c, combatant_id)
                    .iter()
                    .any(|u| u.name == *stone)
            {
                return Err(error(
                    "Hold the exact Mega stone for the exact base species",
                ));
            }
            if field::has(c, "Magic Room", p) {
                return Err(error("Magic Room prevents item use"));
            }
            if p.runtime.will < 1 {
                return Err(error("Mega Evolution costs one Will"));
            }
            original(&mut next, combatant_id)?;
            let origin = state_mut(&mut next)
                .originals
                .get_mut(combatant_id)
                .unwrap();
            origin.hp = p.hp;
            origin.max_hp = p.max_hp;
            origin.statuses = p.runtime.statuses.clone();
            origin.sheet.conditions = p.conditions.clone();
            origin.will = p.runtime.will - 1;
            set_form(
                &mut next,
                combatant_id,
                mapping["form"]
                    .as_str()
                    .ok_or_else(|| error("Unreviewed Mega mapping"))?,
                "Mega Evolution",
                (true, 0, true),
                cat,
            )?;
            state_mut(&mut next).mega_trainers.insert(trainer.clone());
        }
        FormCommand::Choose {
            combatant_id,
            target_id,
        } => {
            owner(c, actor, combatant_id)?;
            let p = fighter(c, combatant_id)?;
            if !p.has_ability("Imposter")
                && !rules::base_sheet(c, p)?
                    .moves
                    .iter()
                    .any(|m| m == "Transform")
            {
                return Err(error("This Pokemon has no Transform effect"));
            }
            let t = fighter(c, target_id)?;
            if !t.deployed || t.hp == 0 || !reviewed::in_range(c, p, t) {
                return Err(error("Transform model must be visible and in range"));
            }
            state_mut(&mut next)
                .choices
                .insert(format!("transform:{combatant_id}"), target_id.clone());
        }
        FormCommand::Type {
            combatant_id,
            type_name,
        } => {
            owner(c, actor, combatant_id)?;
            if !fighter(c, combatant_id)?.has_ability("Multitype")
                || !species(c, combatant_id).starts_with("0493:")
            {
                return Err(error("Only Multitype Arceus may change type freely"));
            }
            set_type(&mut next, combatant_id, type_name)?;
        }
        FormCommand::SchoolArrives { combatant_id } => {
            gm(c, actor)?;
            let p = fighter(c, combatant_id)?;
            if !p.has_ability("Schooling")
                || !p.deployed
                || p.hp == 0
                || p.hp * 2 > p.max_hp
                || !state(c)
                    .choices
                    .contains_key(&format!("school:{combatant_id}"))
            {
                return Err(error(
                    "Healing, fainting or recall prevents the called school from arriving",
                ));
            }
            let target = mapped("Schooling", &species(c, combatant_id))
                .first()
                .cloned()
                .ok_or_else(|| error("No exact Schooling form"))?;
            set_form(
                &mut next,
                combatant_id,
                &target,
                "Schooling",
                (true, 2, false),
                cat,
            )?;
            state_mut(&mut next)
                .choices
                .remove(&format!("school:{combatant_id}"));
        }
        FormCommand::RestoreCore { sheet_id } => {
            gm(c, actor)?;
            if battle(c)?
                .combatants
                .iter()
                .any(|p| p.sheet_id == *sheet_id && p.deployed)
            {
                return Err(error("Core restoration requires the recorded release and recapture outside active deployment"));
            }
            next.form_memory.remove(sheet_id);
        }
        FormCommand::Missile {
            combatant_id,
            target_id,
            status,
        } => {
            gm(c, actor)?;
            check_count(dice, 2)?;
            let event = flow(c)?
                .ability_events
                .first()
                .filter(|e| {
                    e.combatant_id == *combatant_id
                        && e.kind == "form-missile"
                        && e.choices.contains(target_id)
                })
                .ok_or_else(|| error("Resolve the saved Gulp Missile retaliation"))?;
            let damage = if abilities::enabled(c)
                && fighter(c, target_id)?.has_any_ability(&["Magic Guard", "Wonder Guard"])
            {
                0
            } else {
                rules::successes(dice)
            };
            let target = fighter_mut(&mut next, target_id)?;
            target.hp = (target.hp - damage).max(0);
            if let Some(status) = status {
                if !["Paralysis", "Confusion"].contains(&status.as_str()) {
                    return Err(error("Unreviewed missile status: record the card's unspecified effect through ordinary GM adjudication"));
                }
                abilities::inflict(&mut next, combatant_id, target_id, status, why, None, cat)?;
            }
            let target = mapped("Gulp Missile", &species(c, combatant_id))
                .first()
                .cloned()
                .ok_or_else(|| error("No regular Cramorant form"))?;
            set_form(
                &mut next,
                combatant_id,
                &target,
                "Gulp Missile",
                (false, 0, false),
                cat,
            )?;
            state_mut(&mut next).seen.insert(event.id.clone());
            flow_mut(&mut next).ability_events.remove(0);
        }
        FormCommand::Union {
            combatant_id,
            partner_id,
            species_id,
            first_ability,
            second_ability,
        } => {
            gm(c, actor)?;
            if !flow(c)?.queue.is_empty() {
                return Err(error("Resolve attacks before joining two allies"));
            }
            let first = fighter(c, combatant_id)?;
            let partner = fighter(c, partner_id)?;
            if !first.has_ability("As One")
                || first.id == partner.id
                || first.side != partner.side
                || !first.deployed
                || !partner.deployed
                || first.hp == 0
                || partner.hp == 0
            {
                return Err(error("As One requires two conscious deployed allies"));
            }
            let converters = [
                "Aerilate",
                "Pixilate",
                "Galvanize",
                "Refrigerate",
                "Normalize",
                "Liquid Voice",
            ];
            if (first_ability != second_ability
                && converters.contains(&first_ability.as_str())
                && converters.contains(&second_ability.as_str()))
                || ([first_ability.as_str(), second_ability.as_str()].contains(&"Contrary")
                    && [first_ability.as_str(), second_ability.as_str()].contains(&"Simple"))
            {
                return Err(AppError::new("manual_rule","These two ability channels have no printed precedence; record a GM resolution instead of silently ordering them"));
            }
            for (p, ability) in [(first, first_ability), (partner, second_ability)] {
                let s = &c.sheets[&p.sheet_id];
                if !p.has_ability(ability)
                    && !s
                        .workflow
                        .as_ref()
                        .is_some_and(|b| b.available_abilities.contains(ability))
                {
                    return Err(error("Choose one available ability from each component"));
                }
                if cat.find("abilities", "Name", ability).is_none() {
                    return Err(error("Unresolved component ability"));
                }
            }
            if [first_ability.as_str(), second_ability.as_str()].contains(&"Imposter") {
                return Err(AppError::new("manual_rule","As One/Imposter nesting has no printed identity precedence; adjudicate before joining"));
            }
            original(&mut next, partner_id)?;
            set_form(
                &mut next,
                combatant_id,
                species_id,
                "As One",
                (false, 0, false),
                cat,
            )?;
            let mut moves = rules::base_sheet(c, first)?.moves;
            for name in rules::base_sheet(c, partner)?.moves {
                if !moves.contains(&name) {
                    moves.push(name);
                }
            }
            let v = state_mut(&mut next).current.get_mut(combatant_id).unwrap();
            v.sheet.moves = moves;
            v.ability = first_ability.clone();
            v.secondary = Some(second_ability.clone());
            v.partner = Some(partner_id.clone());
            let p = fighter_mut(&mut next, combatant_id)?;
            p.selected_ability = first_ability.clone();
            p.runtime
                .ability
                .flags
                .insert(format!("as-one:{second_ability}"));
            fighter_mut(&mut next, partner_id)?.deployed = false;
            fighter_mut(&mut next, partner_id)?.field_slot = None;
            flow_mut(&mut next).extra.active.remove(combatant_id);
            flow_mut(&mut next)
                .battlefield
                .known_active
                .remove(combatant_id);
            record(&mut next,combatant_id,"GM selected a reviewed joint-form build and one ability from each ally; component movepools combined");
        }
    }
    if battle(c)?.active {
        ability_changes::lifecycle(c, &mut next, cat)?;
        field::lifecycle(c, &mut next, cat)?;
        abilities::lifecycle(c, &mut next)?;
        extra::lifecycle(c, &mut next, cat)?;
    }
    if !matches!(command, FormCommand::Missile { .. }) {
        check_count(dice, 0)?;
    }
    if matches!(command, FormCommand::Missile { .. }) {
        redirect_losses(c, &mut next)?;
    }
    refresh_initiative(c, &mut next)?;
    let log = format!("Reviewed form decision: {why}");
    information::lifecycle(&mut next, cat)?;
    standalone::lifecycle(&mut next)?;
    checkpoint(c, &mut next, &log);
    Ok((next, log))
}
fn set_type(c: &mut Campaign, id: &str, type_name: &str) -> Result<()> {
    if ![
        "Normal", "Fire", "Water", "Electric", "Grass", "Ice", "Fight", "Poison", "Ground",
        "Flying", "Psychic", "Bug", "Rock", "Ghost", "Dragon", "Dark", "Steel", "Fairy",
    ]
    .contains(&type_name)
    {
        return Err(error("Choose one of the 18 reviewed types"));
    }
    original(c, id)?;
    if !state(c).current.contains_key(id) {
        let p = fighter(c, id)?.clone();
        let s = c.sheets[&p.sheet_id].clone();
        state_mut(c).current.insert(
            id.into(),
            FormView {
                sheet: s,
                types: vec![type_name.into()],
                ability: p.selected_ability,
                label: type_name.into(),
                source: "typing".into(),
                mega: false,
                secondary: None,
                partner: None,
            },
        );
    } else {
        state_mut(c).current.get_mut(id).unwrap().types = vec![type_name.into()];
    }
    Ok(())
}

pub fn grant_temporary_move(c: &mut Campaign, id: &str, name: &str) {
    if enabled(c) {
        if let Some(v) = state_mut(c).current.get_mut(id) {
            if !v.sheet.moves.iter().any(|m| m == name) {
                v.sheet.moves.push(name.into());
            }
        }
    }
}
