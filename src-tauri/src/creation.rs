//! Shared Pokerole 3.0 creation/progression rules: pp.23–31,41–45,107–112;
//! ability availability pp.534–535. All prices and effective scores are server derived.
use crate::{catalog::Catalog, model::*};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const VERSION: u32 = 1;
pub const RANKS: [&str; 8] = [
    "Starter", "Rookie", "Standard", "Advanced", "Expert", "Ace", "Master", "Champion",
];
pub const ATTRS: [&str; 5] = ["Strength", "Dexterity", "Vitality", "Special", "Insight"];
pub const SOCIALS: [&str; 5] = ["Tough", "Cool", "Beauty", "Cute", "Clever"];
const BUDGET: [i32; 8] = [0, 2, 4, 6, 8, 10, 10, 14];
const SKILLS: [i32; 8] = [5, 10, 14, 17, 19, 20, 22, 25];
const CAPS: [i32; 8] = [1, 2, 3, 4, 5, 5, 5, 5];
const UP: [i32; 7] = [5, 15, 25, 30, 35, 40, 50];
const RETRAIN: [i32; 8] = [1, 10, 20, 25, 30, 35, 40, 45];
pub fn rank(s: &str) -> Option<usize> {
    RANKS.iter().position(|r| *r == s)
}
pub fn valid_profile(p: &RulesProfile) -> bool {
    p.version == 1
        && ["split", "vitality"].contains(&p.defenses.as_str())
        && ["vitality", "highest_limit"].contains(&p.hp.as_str())
}
pub fn fingerprint(s: &Sheet) -> Result<String> {
    let mut value = serde_json::to_value(s)?;
    for key in [
        "revision",
        "name",
        "notes",
        "archived",
        "squad_order",
        "hp",
        "conditions",
    ] {
        value.as_object_mut().unwrap().remove(key);
    }
    if let Some(b) = value.get_mut("workflow").and_then(|v| v.as_object_mut()) {
        b.remove("ledger");
        b.remove("will");
    }
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(&value)?)))
}
pub fn build(s: &Sheet, catalog: &Catalog) -> CharacterBuild {
    let species = catalog.find(
        "species",
        "AppId",
        s.species_id.as_deref().unwrap_or_default(),
    );
    let allocations = ATTRS
        .into_iter()
        .filter(|a| s.kind == "pokemon" || *a != "Special")
        .map(|a| {
            let base = if s.kind == "trainer" {
                1
            } else {
                species.as_ref().and_then(|v| v[a].as_i64()).unwrap_or(0) as i32
            };
            (a.into(), s.stats.get(a).copied().unwrap_or(base) - base)
        })
        .collect();
    CharacterBuild {
        version: VERSION,
        status: "review".into(),
        origin: "legacy".into(),
        age: "Teen".into(),
        allocations,
        socials: SOCIALS.into_iter().map(|a| (a.into(), 0)).collect(),
        happiness: 2,
        loyalty: 2,
        training_points: if s.kind == "pokemon" { s.xp } else { 0 },
        legacy_xp: s.xp,
        will: s.stats.get("Insight").copied().unwrap_or(1) + 3,
        max_will: s.stats.get("Insight").copied().unwrap_or(1) + 3,
        profile: RulesProfile::default(),
        hp_attribute: "Vitality".into(),
        available_abilities: vec![],
        learned: s
            .moves
            .iter()
            .map(|m| (m.clone(), "legacy".into()))
            .collect(),
        forgotten: BTreeMap::new(),
        overranked: None,
        evolution_ready: None,
        ledger: vec![],
    }
}
pub fn profile_stats(s: &Sheet) -> BTreeMap<String, i32> {
    let bonus = if rank(&s.rank).is_some_and(|r| r >= 6) {
        3
    } else {
        0
    };
    let vitality = *s.stats.get("Vitality").unwrap_or(&0);
    let insight = *s.stats.get("Insight").unwrap_or(&0);
    let split = s
        .workflow
        .as_ref()
        .is_none_or(|b| b.profile.defenses == "split");
    BTreeMap::from([
        ("Defense".into(), vitality + bonus),
        (
            "SpecialDefense".into(),
            if split {
                insight + bonus
            } else {
                vitality + bonus
            },
        ),
        (
            "Initiative".into(),
            s.stats.get("Dexterity").unwrap_or(&0) + s.skills.get("Alert").unwrap_or(&0) + bonus,
        ),
        ("MoveCapacity".into(), insight + 3),
        ("MaxWill".into(), insight + 3 + bonus),
        ("MaxHP".into(), s.max_hp),
    ])
}
fn species(s: &Sheet, c: &Catalog) -> Option<serde_json::Value> {
    c.find(
        "species",
        "AppId",
        s.species_id.as_deref().unwrap_or_default(),
    )
}
fn stage(s: &Sheet, c: &Catalog) -> Option<usize> {
    let v = species(s, c)?;
    v["Creation"]["stage"].as_u64().map(|v| v as usize)
}
fn move_rank(s: &Sheet, name: &str, c: &Catalog) -> Option<usize> {
    species(s, c)?["Moves"]
        .as_array()?
        .iter()
        .find(|m| m["Name"].as_str() == Some(name))
        .and_then(|m| rank(m["Learned"].as_str()?))
}

pub fn preview(
    old: Option<&Sheet>,
    trainer: Option<&Sheet>,
    req: &WorkflowRequest,
    catalog: &Catalog,
) -> WorkflowPreview {
    let mut s = old.cloned().unwrap_or_else(|| req.sheet.clone());
    let mut errors = vec![];
    let mut approvals = vec![];
    let mut warnings = vec![];
    if req.sheet.skills.len() > 50
        || req.sheet.moves.len() > 100
        || req.sheet.workflow.as_ref().is_some_and(|b| {
            b.allocations.len() > 10 || b.socials.len() > 10 || b.available_abilities.len() > 20
        })
    {
        return WorkflowPreview {
            sheet: req.sheet.clone(),
            errors: vec!["Character field counts exceed hard limits".into()],
            approvals: vec![],
            warnings: vec![],
            cost: 0,
            remaining: BTreeMap::new(),
            baselines: BTreeMap::new(),
            modifiers: BTreeMap::new(),
            derived: BTreeMap::new(),
        };
    }
    let mut cost = 0;
    if req.rules_version != VERSION || req.catalog_hash != catalog.hash {
        errors.push("Rules/catalog version changed; refresh the preview".into());
    }
    if req.operation_id.trim().is_empty() || req.reason.trim().is_empty() || req.reason.len() > 1000
    {
        errors.push("An operation ID and a reason of 1–1000 characters are required".into());
    }
    if let Some(old) = old {
        if old.id != req.sheet.id
            || old.owner_id != req.sheet.owner_id
            || old.kind != req.sheet.kind
        {
            errors.push("Character identity and kind are immutable".into());
        }
        if old.revision != req.expected_revision {
            errors.push("Character revision changed; reopen the operation".into());
        }
    } else if req.expected_revision != 0 || req.kind != "create" {
        errors.push("Create a new character before progressing it".into());
    }
    let mut b = old
        .and_then(|s| s.workflow.clone())
        .unwrap_or_else(|| build(&s, catalog));
    let draft = req
        .sheet
        .workflow
        .clone()
        .unwrap_or_else(|| build(&req.sheet, catalog));
    let old_rank = rank(&s.rank).unwrap_or(0);
    let mut resource_base = old.cloned();
    let mut preserve_moves = false;
    match req.kind.as_str() {
        "create" | "review" | "exception" => {
            if req.kind == "create" && old.is_some() {
                errors.push("Character already exists".into());
            }
            if req.kind == "review"
                && old.is_some_and(|s| s.workflow.as_ref().is_some_and(|b| b.status == "strict"))
            {
                errors.push("Use progression operations for an already reviewed character".into());
            }
            let ledger = b.ledger.clone();
            let tp = b.training_points;
            let xp = b.legacy_xp;
            s = req.sheet.clone();
            b = draft.clone();
            b.ledger = ledger;
            b.training_points = if old.is_some() { tp } else { 0 };
            b.legacy_xp = if old.is_some() { xp } else { 0 };
            b.learned = s
                .moves
                .iter()
                .map(|m| (m.clone(), "initial".into()))
                .collect();
            b.forgotten.clear();
            b.overranked = None;
            b.evolution_ready = None;
            if req.kind == "exception" {
                approvals.push("GM-reviewed rebuild or rule exception".into());
            }
            if old.is_none() {
                resource_base = None;
            }
            if s.kind == "pokemon" {
                if b.origin == "starter" && trainer.is_some_and(|t| t.rank != s.rank) {
                    errors.push("A starting Pokemon must share its trainer's rank".into());
                }
                if ["captured", "narrative", "opponent", "legacy"].contains(&b.origin.as_str()) {
                    approvals.push(format!("Confirm {} origin and initial resources", b.origin));
                }
            } else if s.rank != "Rookie" {
                approvals.push("Confirm the trainer's starting rank".into());
            }
            if old.is_some() {
                approvals
                    .push("Confirm legacy conversion/rebuild and retained Training Points".into());
            }
        }
        "describe" => {
            s.name = req.sheet.name.clone();
            s.notes = req.sheet.notes.clone();
            s.squad_order = req.sheet.squad_order;
            s.archived = req.sheet.archived;
        }
        "allocate" | "retrain" => {
            if req.kind == "allocate"
                && (b
                    .allocations
                    .iter()
                    .any(|(k, v)| *draft.allocations.get(k).unwrap_or(&0) < *v)
                    || b.socials
                        .iter()
                        .any(|(k, v)| *draft.socials.get(k).unwrap_or(&0) < *v)
                    || s.skills
                        .iter()
                        .any(|(k, v)| *req.sheet.skills.get(k).unwrap_or(&0) < *v))
            {
                errors.push("Redistribution requires retraining".into());
            }
            if req.kind == "retrain" {
                if s.kind == "trainer" {
                    approvals.push(
                        "Trainer retraining requires confirmed achievements and a mentor".into(),
                    );
                } else {
                    cost = RETRAIN[old_rank];
                }
            }
            b.allocations = draft.allocations.clone();
            b.socials = draft.socials.clone();
            s.skills = req.sheet.skills.clone();
            preserve_moves = true;
        }
        "rank_up" => {
            if old_rank == 7 {
                errors.push("Champion is the highest rank".into());
            } else {
                s.rank = RANKS[old_rank + 1].into();
                if s.kind == "trainer" {
                    approvals.push("GM must confirm completed rank achievements".into());
                } else {
                    cost = UP[old_rank];
                    if trainer
                        .and_then(|t| rank(&t.rank))
                        .is_none_or(|r| r < old_rank + 1)
                    {
                        errors.push("Cannot train a Pokemon above its trainer's rank".into());
                    }
                }
            }
            preserve_moves = true;
        }
        "award" => {
            if !["pokemon", "trainer"].contains(&s.kind.as_str())
                || req.amount == 0
                || !(-100_000..=100_000).contains(&req.amount)
            {
                errors.push(
                    "Training Point award/correction requires a trainer or Pokemon and a nonzero bounded amount"
                        .into(),
                );
            } else {
                b.training_points = b.training_points.saturating_add(req.amount);
            }
            approvals.push("GM must confirm the Training Point award/correction and source".into());
            preserve_moves = true;
        }
        "bond" => {
            b.happiness = draft.happiness;
            b.loyalty = draft.loyalty;
            approvals.push("GM must confirm narrative Happiness/Loyalty changes".into());
            preserve_moves = true;
        }
        "resources" => {
            s.hp = req.sheet.hp;
            b.will = draft.will;
            s.conditions = req.sheet.conditions.clone();
            approvals.push("GM must confirm healing, Will recovery or conditions".into());
            preserve_moves = true;
        }
        "ability" => {
            s.ability = req.sheet.ability.clone();
            if !b.available_abilities.contains(&s.ability) {
                b.available_abilities.push(s.ability.clone());
                approvals.push("GM must grant this exceptional ability".into());
            }
            preserve_moves = true;
        }
        "learn" => {
            let name = &req.move_name;
            let method = req.method.as_str();
            if s.kind != "pokemon"
                || catalog.find("moves", "Name", name).is_none()
                || s.moves.contains(name)
            {
                errors.push("Select an unlearned catalog move for a Pokemon".into());
            }
            let mr = move_rank(&s, name, catalog);
            let capacity = *s.stats.get("Insight").unwrap_or(&0) + 3;
            let st = stage(&s, catalog);
            if st.is_none()
                && (method == "overrank"
                    || method == "pre_evolution"
                    || method == "ordinary"
                        && (s.moves.len() as i32 >= capacity
                            || !req.replacement.is_empty()
                            || b.forgotten.contains_key(name)))
            {
                approvals.push("Evolution stage needs a source-backed decision".into());
            }
            if req.amount < 0 || req.amount > 100_000 {
                errors.push("Proposed learning cost must be between 0 and 100000".into());
            }
            match method {
                "ordinary" => {
                    if mr.is_none_or(|r| r > old_rank) {
                        errors.push("Ordinary learning requires a move from the current species at the current rank or lower".into());
                    }
                    if s.moves.len() as i32 >= capacity
                        || !req.replacement.is_empty()
                        || b.forgotten.contains_key(name)
                    {
                        cost = st.map_or(req.amount, |st| {
                            [2, 4, 6][st - 1]
                                / if mr.is_some_and(|r| r < old_rank) {
                                    2
                                } else {
                                    1
                                }
                        });
                    }
                }
                "pre_evolution" => {
                    let ancestors = ancestors(&s, catalog);
                    if !ancestors.iter().any(|v| {
                        v["Moves"].as_array().is_some_and(|ms| {
                            ms.iter().any(|m| {
                                m["Name"].as_str() == Some(name)
                                    && m["Learned"]
                                        .as_str()
                                        .and_then(rank)
                                        .is_some_and(|r| r <= old_rank)
                            })
                        })
                    }) {
                        errors.push("Move is not an eligible pre-evolution move".into());
                    }
                    if b.forgotten
                        .get(name)
                        .is_some_and(|m| ["tm", "tr"].contains(&m.as_str()))
                    {
                        errors.push("Forgotten TM/TR moves require TM/TR training again".into());
                    }
                    cost = if st == Some(2) { 5 } else { 10 };
                }
                "tm" | "tr" => {
                    cost = 5;
                    approvals.push(
                        "GM must confirm TM/TR availability and species compatibility".into(),
                    );
                    let item = &req.target_species;
                    if item.trim().is_empty() || s.inventory.get(item).copied().unwrap_or(0) < 1 {
                        errors.push("Select an owned TM/TR inventory item".into());
                    } else if method == "tr" {
                        *s.inventory.get_mut(item).unwrap() -= 1;
                    }
                }
                "overrank" => {
                    if mr.is_none_or(|r| r <= old_rank) {
                        errors.push(
                            "Over-ranking requires a higher-rank move from this species".into(),
                        );
                    }
                    if b.happiness + b.loyalty < 7 {
                        errors
                            .push("Over-ranking requires Happiness + Loyalty of at least 7".into());
                    }
                    cost = st.map_or(req.amount, |st| [5, 15, 20][st - 1])
                        * mr.map_or(0, |r| r.saturating_sub(old_rank) as i32);
                    if let Some(previous) = b.overranked.clone() {
                        forget(&mut s, &mut b, &previous);
                    }
                    b.overranked = Some(name.clone());
                }
                "tutor" => {
                    approvals.push(
                        "GM must confirm tutor compatibility, narrative prerequisites and cost"
                            .into(),
                    );
                    cost = req.amount.max(0);
                }
                _ => errors.push("Choose a supported learning method".into()),
            }
            if s.moves.len() as i32 >= capacity && req.replacement.is_empty() {
                errors.push("At move capacity: select a move to forget".into());
            }
            if !req.replacement.is_empty() {
                if !s.moves.contains(&req.replacement) {
                    errors.push("Replacement move is not currently learned".into());
                } else {
                    forget(&mut s, &mut b, &req.replacement);
                }
            }
            s.moves.push(name.clone());
            b.learned.insert(name.clone(), method.into());
            preserve_moves = true;
        }
        "forget" => {
            if s.kind != "pokemon" || !s.moves.contains(&req.move_name) {
                errors.push("Select a currently learned Pokemon move to forget".into());
            } else {
                forget(&mut s, &mut b, &req.move_name);
            }
            preserve_moves = true;
        }
        "prepare_evolution" | "evolve" => {
            let current_species = species(&s, catalog);
            let edge = current_species
                .as_ref()
                .and_then(|v| v["Evolutions"].as_array())
                .and_then(|es| {
                    es.iter().find(|e| {
                        catalog
                            .find("species", "AppId", &req.target_species)
                            .is_some_and(|target| e["To"] == target["Name"])
                    })
                })
                .cloned();
            if let Some(e) = edge {
                if req.kind == "prepare_evolution" {
                    if !(0..=100_000).contains(&req.amount) {
                        errors.push("Proposed evolution cost must be between 0 and 100000".into());
                    }
                    if b.evolution_ready.is_some() {
                        errors.push("An evolution is already prepared".into());
                    }
                    match e["Kind"].as_str().unwrap_or("") {
                        "Level" => match e["Speed"].as_str() {
                            Some("Fast") => cost = 10,
                            Some("Medium") => cost = 30,
                            Some("Slow") => cost = 50,
                            _ => approvals.push("Missing evolution speed: GM must set cost".into()),
                        },
                        "Stat" => {
                            let key = e["Stat"].as_str().unwrap_or("");
                            let value = match key {
                                "Happiness" => b.happiness,
                                "Loyalty" => b.loyalty,
                                _ => *s.stats.get(key).unwrap_or(&0),
                            };
                            if e["Value"].as_i64().is_none_or(|v| i64::from(value) < v) {
                                errors.push(format!("Evolution requires {key} {}", e["Value"]));
                            }
                        }
                        "Stone" | "Item" => {
                            let item = e["Item"].as_str().or(e["Stone"].as_str()).unwrap_or("");
                            if s.inventory.get(item).copied().unwrap_or(0) < 1 {
                                errors.push(format!("Evolution requires an owned {item}"));
                            } else {
                                *s.inventory.get_mut(item).unwrap() -= 1;
                            }
                        }
                        _ => approvals
                            .push("Narrative/trade/form evolution requires GM confirmation".into()),
                    }
                    for key in ["Region", "Gender", "Special", "Game"] {
                        if e.get(key).is_some() {
                            approvals
                                .push(format!("Confirm evolution condition {key}: {}", e[key]));
                        }
                    }
                    if let Some(m) = e["Move"].as_str() {
                        if !s.moves.iter().any(|n| n == m) {
                            errors.push(format!("Evolution requires learned move {m}"));
                        }
                    }
                    if cost == 0 && e["Kind"].as_str() == Some("Level") {
                        cost = req.amount.max(0);
                    }
                    b.evolution_ready = Some(req.target_species.clone());
                } else {
                    if b.evolution_ready.as_deref() != Some(&req.target_species) {
                        errors.push("Prepare this evolution before completing it".into());
                    }
                    s.species_id = Some(req.target_species.clone());
                    b.evolution_ready = None;
                    b.available_abilities.clear();
                    s.ability.clear();
                    approvals.push(
                        "GM must confirm the evolution scene and remaining narrative conditions"
                            .into(),
                    );
                }
            } else {
                errors.push("Select a direct evolution from this species".into());
            }
            preserve_moves = true;
        }
        "inventory" => {
            s.inventory = req.sheet.inventory.clone();
            approvals.push("GM must confirm inventory additions/removals".into());
            preserve_moves = true;
        }
        _ => errors.push("Unknown progression operation".into()),
    }
    if old.is_some_and(|s| s.workflow.as_ref().is_none_or(|b| b.status == "review"))
        && !["review", "exception", "describe"].contains(&req.kind.as_str())
    {
        errors.push("Complete legacy review before progressing this character".into());
    }
    if s.kind == "pokemon" {
        match trainer {
            Some(t) if t.kind == "trainer" && !t.archived && t.owner_id == s.owner_id => {
                if req.trainer_revision != Some(t.revision) {
                    errors.push("Trainer revision changed; refresh this operation".into());
                }
                let gap = rank(&s.rank)
                    .unwrap_or(0)
                    .saturating_sub(rank(&t.rank).unwrap_or(0));
                if gap > 0 {
                    warnings.push(
                        if gap == 1 {
                            "Low disobedience: training rewards are halved"
                        } else {
                            "High disobedience: refuses training"
                        }
                        .into(),
                    );
                }
            }
            _ => errors.push("Select an active trainer owned by the same player".into()),
        }
    }
    if b.version != VERSION || !valid_profile(&b.profile) {
        errors.push("Unsupported rules profile".into());
    }
    if old.is_some()
        && !["review", "exception"].contains(&req.kind.as_str())
        && (draft.profile != b.profile
            || draft.hp_attribute != b.hp_attribute
            || draft.age != b.age
            || draft.origin != b.origin)
    {
        errors.push("Changing origin, age or rules profile requires a reviewed rebuild".into());
    }
    let ri = rank(&s.rank).unwrap_or_else(|| {
        errors.push("Unknown rank".into());
        0
    });
    if b.overranked
        .as_ref()
        .is_some_and(|m| move_rank(&s, m, catalog).is_some_and(|r| r <= ri))
    {
        b.overranked = None;
    }
    if !["starter", "captured", "narrative", "opponent", "legacy"].contains(&b.origin.as_str()) {
        errors.push("Unknown origin".into());
    }
    for value in b.allocations.values_mut().chain(b.socials.values_mut()) {
        if !(-999..=999).contains(value) {
            *value = 0;
            errors.push("Allocation outside hard numeric bounds".into());
        }
    }
    for value in s.skills.values_mut() {
        if !(-999..=999).contains(value) {
            *value = 0;
            errors.push("Skill outside hard numeric bounds".into());
        }
    }
    let age = if s.kind == "trainer" {
        match b.age.as_str() {
            "Child" => (0, 0),
            "Teen" => (2, 2),
            "Adult" => (4, 4),
            "Senior" => (3, 6),
            _ => {
                errors.push("Select a valid age group".into());
                (0, 0)
            }
        }
    } else {
        (0, 0)
    };
    let mut baselines = BTreeMap::new();
    let mut modifiers = BTreeMap::new();
    let sp = species(&s, catalog);
    if s.kind == "pokemon" && sp.is_none() {
        errors.push("Select a catalog species/form".into());
    }
    let mut used = 0;
    if s.ability.is_empty() && s.kind == "pokemon" {
        s.ability = sp
            .as_ref()
            .and_then(|v| v["Ability1"].as_str())
            .unwrap_or_default()
            .into();
    }
    s.stats.clear();
    for a in ATTRS {
        if s.kind == "trainer" && a == "Special" {
            continue;
        }
        let base = if s.kind == "trainer" {
            1
        } else {
            sp.as_ref().and_then(|v| v[a].as_i64()).unwrap_or(0) as i32
        };
        let cap = if s.kind == "trainer" {
            5
        } else {
            sp.as_ref()
                .and_then(|v| v[format!("Max{a}")].as_i64())
                .unwrap_or(base as i64) as i32
        } + if ri == 7 { 2 } else { 0 };
        let alloc = *b.allocations.get(a).unwrap_or(&0);
        used += alloc;
        if alloc < 0 || base + alloc > cap {
            errors.push(format!(
                "{a} allocation must keep the trained score between {base} and {cap}"
            ));
        }
        let modifier =
            if a == "Strength" && ["Huge Power", "Pure Power"].contains(&s.ability.as_str()) {
                if ri >= 4 {
                    2
                } else {
                    1
                }
            } else {
                0
            };
        baselines.insert(a.into(), base);
        modifiers.insert(a.into(), modifier);
        s.stats.insert(a.into(), base + alloc + modifier);
    }
    if b.allocations
        .keys()
        .any(|a| !ATTRS.contains(&a.as_str()) || s.kind == "trainer" && a == "Special")
    {
        errors.push("Unknown attribute allocation".into());
    }
    let mut social_used = 0;
    for a in SOCIALS {
        let alloc = *b.socials.get(a).unwrap_or(&0);
        social_used += alloc;
        if alloc < 0 || 1 + alloc > if ri == 7 { 7 } else { 5 } {
            errors.push(format!("Invalid {a} allocation"));
        }
        s.stats.insert(a.into(), 1 + alloc);
    }
    if b.socials.keys().any(|a| !SOCIALS.contains(&a.as_str())) {
        errors.push("Unknown social attribute".into());
    }
    let skill_used = s.skills.values().sum::<i32>();
    for (k, v) in &s.skills {
        if *v > 0
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
            && old.is_none_or(|s| s.skills.get(k).copied().unwrap_or(0) == 0)
        {
            approvals.push(format!("Authorize extra skill {k} and instruction"));
        }
        if k.trim().is_empty() || k.len() > 50 || *v < 0 || *v > CAPS[ri] {
            errors.push(format!("Skill {k} must be between 0 and {}", CAPS[ri]));
        }
    }
    let remaining = BTreeMap::from([
        ("Attributes".into(), BUDGET[ri] + age.0 - used),
        ("Socials".into(), BUDGET[ri] + age.1 - social_used),
        ("Skills".into(), SKILLS[ri] - skill_used),
    ]);
    for (k, v) in &remaining {
        if *v < 0 {
            errors.push(format!("{k} budget exceeded by {} points", -v));
        } else if *v > 0 {
            warnings.push(format!("{v} unallocated {k} points"));
        }
    }
    if !(0..=5).contains(&b.happiness) || !(0..=5).contains(&b.loyalty) {
        errors.push("Happiness and Loyalty must be between 0 and 5".into());
    }
    if s.kind == "pokemon" {
        if let Some(sp) = &sp {
            let metadata = &sp["Creation"];
            if ["create", "review", "exception", "evolve"].contains(&req.kind.as_str()) {
                for field in metadata["unresolved_fields"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|v| v.as_str())
                {
                    approvals.push(format!(
                        "Catalog adjudication: confirm {} {field} against the PDF",
                        sp["Name"].as_str().unwrap_or("species")
                    ));
                }
            }
            for name in metadata["unresolved_moves"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str())
            {
                if req.kind == "learn" && req.move_name == name
                    || ["create", "review", "exception"].contains(&req.kind.as_str())
                        && s.moves.iter().any(|m| m == name)
                {
                    approvals.push(format!(
                        "Catalog adjudication: confirm {name} learn rank against the PDF"
                    ));
                }
            }
            if metadata["audit"].as_str() != Some("verified-vector-fields") {
                warnings.push(format!(
                    "Catalog audit: {}. Unresolved fields/moves require GM adjudication when used.",
                    metadata["audit"].as_str().unwrap_or("unverified")
                ));
            }
        }
        let ordinary: Vec<String> = sp
            .as_ref()
            .map(|v| {
                ["Ability1", "Ability2"]
                    .into_iter()
                    .filter_map(|k| v[k].as_str().filter(|a| !a.is_empty()).map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        if req.kind == "create" || req.kind == "review" || req.kind == "evolve" {
            for a in &ordinary {
                if !b.available_abilities.contains(a) {
                    b.available_abilities.push(a.clone());
                }
            }
        }
        if s.ability.is_empty() {
            s.ability = ordinary.first().cloned().unwrap_or_default();
        }
        for a in &b.available_abilities {
            if catalog.find("abilities", "Name", a).is_none() {
                errors.push(format!(
                    "Unknown ability {a}; catalog reference needs correction"
                ));
            }
            if !ordinary.contains(a)
                && !old
                    .and_then(|s| s.workflow.as_ref())
                    .is_some_and(|b| b.available_abilities.contains(a))
            {
                approvals.push(format!("Authorize exceptional ability {a}"));
            }
        }
        if !b.available_abilities.contains(&s.ability) {
            errors.push("Select an available ability".into());
        }
        if !["Huge Power", "Pure Power"].contains(&s.ability.as_str()) && !s.ability.is_empty() {
            warnings.push(format!(
                "{}: conditional/reactive effects apply only in battle when supported; unreviewed interactions require GM adjudication",
                s.ability
            ));
        }
        let capacity = s.stats.get("Insight").copied().unwrap_or(0) + 3;
        if s.moves.len() as i32 > capacity {
            errors.push(format!(
                "Move capacity is {capacity}; resolve excess moves explicitly"
            ));
        }
        if s.moves.is_empty() {
            warnings.push("No moves selected".into());
        }
        for m in &s.moves {
            if catalog.find("moves", "Name", m).is_none() {
                errors.push(format!("Unknown move {m}"));
            }
            if !preserve_moves
                && req.kind != "exception"
                && move_rank(&s, m, catalog).is_none_or(|r| r > ri)
            {
                errors.push(format!("{m} is unavailable at this rank"));
            }
        }
        if stage(&s, catalog).is_none() {
            warnings.push("Evolution stage requires catalog review before priced training".into());
        }
    }
    if b.profile.hp == "highest_limit" && s.kind == "pokemon" {
        if old.is_none() || ["review", "exception"].contains(&req.kind.as_str()) {
            b.hp_attribute = if sp.as_ref().is_some_and(|v| {
                v["MaxInsight"].as_i64().unwrap_or(0) > v["MaxVitality"].as_i64().unwrap_or(0)
            }) {
                "Insight"
            } else {
                "Vitality"
            }
            .into();
        }
    } else {
        b.hp_attribute = "Vitality".into();
    }
    let bonus = if ri >= 6 { 3 } else { 0 };
    let maxhp = if s.kind == "trainer" {
        4
    } else {
        sp.as_ref().and_then(|v| v["BaseHP"].as_i64()).unwrap_or(1) as i32
    } + s.stats.get(&b.hp_attribute).copied().unwrap_or(0)
        + bonus;
    let maxwill = s.stats.get("Insight").copied().unwrap_or(0) + 3 + bonus;
    if let Some(old) = &resource_base {
        if req.kind != "resources" {
            s.hp = if old.hp == 0 {
                0
            } else {
                (maxhp - (old.max_hp - old.hp)).clamp(0, maxhp.max(0))
            };
            b.will = (maxwill - old.workflow.as_ref().map_or(0, |b| b.max_will - b.will))
                .clamp(0, maxwill.max(0));
        }
    } else {
        s.hp = maxhp;
        b.will = maxwill;
    }
    s.max_hp = maxhp;
    b.max_will = maxwill;
    if s.hp < 0 || s.hp > maxhp || b.will < 0 || b.will > maxwill {
        errors.push("Current HP/Will must fit their derived maximums".into());
    }
    if b.training_points > 1_000_000 {
        errors.push("Training Point balance exceeds hard limits".into());
    }
    if b.training_points < cost || b.training_points < 0 {
        errors.push(format!(
            "Need {cost} Training Points; available {}",
            b.training_points
        ));
    }
    b.training_points = b.training_points.saturating_sub(cost);
    if s.kind == "pokemon" {
        s.xp = b.training_points;
    }
    b.status = if req.kind == "exception"
        || old
            .and_then(|s| s.workflow.as_ref())
            .is_some_and(|b| b.status == "exception")
    {
        "exception"
    } else {
        "strict"
    }
    .into();
    s.workflow = Some(b);
    if s.name.trim().is_empty() || s.name.len() > 200 || s.notes.len() > 20_000 {
        errors.push("Enter a name and keep text within limits".into());
    }
    if catalog.find("natures", "Name", &s.nature).is_none() {
        errors.push("Choose a catalog nature".into());
    }
    if s.inventory.len() > 100 || s.inventory.values().any(|v| *v < 0 || *v > 100_000) {
        errors.push("Invalid inventory quantities".into());
    }
    if old.is_none() && !s.inventory.is_empty() {
        approvals.push("Authorize starting inventory".into());
    }
    if s.moves.len()
        != s.moves
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    {
        errors.push("Duplicate learned moves".into());
    }
    approvals.sort();
    approvals.dedup();
    if req.kind == "exception" {
        errors.retain(|e| {
            if e.contains("allocation must")
                || e.contains("budget exceeded")
                || e.starts_with("Skill ")
                || e.starts_with("Move capacity")
                || e.contains("unavailable at this rank")
                || e.starts_with("A starting Pokemon must")
            {
                approvals.push(format!("Rule exception: {e}"));
                false
            } else {
                true
            }
        });
    }
    // Existing, campaign-approved exceptions survive operations that leave the
    // exact affected scores/rank/species unchanged. They cannot authorize a new
    // allocation branch, another species or a newly oversized move set.
    if req.kind != "exception" {
        if let Some(old) = old.filter(|old| {
            old.workflow
                .as_ref()
                .is_some_and(|b| b.status == "exception")
        }) {
            let same_scores = old.stats == s.stats
                && old.skills == s.skills
                && old.rank == s.rank
                && old.species_id == s.species_id;
            if same_scores {
                errors.retain(|e| {
                    if e.starts_with("Move capacity") && old.moves != s.moves {
                        return true;
                    }
                    !old.workflow.as_ref().unwrap().ledger.iter().any(|r| {
                        r.approved_by.is_some()
                            && r.adjudications.contains(&format!("Rule exception: {e}"))
                    })
                });
            }
        }
    }
    let derived = profile_stats(&s);
    WorkflowPreview {
        sheet: s,
        errors,
        approvals,
        warnings,
        cost,
        remaining,
        baselines,
        modifiers,
        derived,
    }
}

fn forget(s: &mut Sheet, b: &mut CharacterBuild, name: &str) {
    s.moves.retain(|m| m != name);
    b.forgotten.insert(
        name.into(),
        b.learned.remove(name).unwrap_or_else(|| "legacy".into()),
    );
    if b.overranked.as_deref() == Some(name) {
        b.overranked = None;
    }
}
fn ancestors(s: &Sheet, c: &Catalog) -> Vec<serde_json::Value> {
    let mut out = vec![];
    let mut pending = species(s, c).into_iter().collect::<Vec<_>>();
    while let Some(v) = pending.pop() {
        if out.len() > 20 {
            break;
        }
        for e in v["Evolutions"].as_array().into_iter().flatten() {
            if let Some(parent) = e["From"]
                .as_str()
                .and_then(|n| c.find("species", "Name", n))
            {
                if !out
                    .iter()
                    .any(|p: &serde_json::Value| p["AppId"] == parent["AppId"])
                {
                    out.push(parent.clone());
                    pending.push(parent.clone());
                }
            }
        }
    }
    out
}
pub fn apply(
    old: Option<&Sheet>,
    trainer: Option<&Sheet>,
    req: &WorkflowRequest,
    catalog: &Catalog,
    approval: Option<(&str, &str)>,
) -> Result<Sheet> {
    let mut result = preview(old, trainer, req, catalog);
    if !result.errors.is_empty() {
        return Err(AppError::new("validation", result.errors.join("; ")));
    }
    if !result.approvals.is_empty() && approval.is_none() {
        return Err(AppError::new(
            "approval_required",
            result.approvals.join("; "),
        ));
    }
    let mut record_req = req.clone();
    if let Some(b) = record_req.sheet.workflow.as_mut() {
        b.ledger.clear();
    }
    let before = old.map(fingerprint).transpose()?.unwrap_or_default();
    let b = result.sheet.workflow.as_mut().unwrap();
    if b.ledger.len() >= 200 {
        return Err(AppError::new(
            "limit",
            "Reconcile progression before exceeding 200 recorded operations",
        ));
    }
    b.ledger.push(WorkflowRecord {
        id: req.operation_id.clone(),
        before_fingerprint: before,
        input_json: serde_json::to_string(&record_req)?,
        cost: result.cost,
        trainer_fingerprint: trainer.map(fingerprint).transpose()?,
        campaign_id: approval.map(|a| a.0.into()),
        approved_by: approval.map(|a| a.1.into()),
        adjudications: result.approvals,
        approval_reason: None,
    });
    result.sheet.revision = req
        .expected_revision
        .checked_add(1)
        .ok_or_else(|| AppError::new("limit", "Revision exhausted"))?;
    Ok(result.sheet)
}

/// Replay the unaccepted suffix; client-supplied approval claims never authorize it.
pub fn verify_sync(
    old: Option<&Sheet>,
    submitted: &Sheet,
    trainer: Option<&Sheet>,
    catalog: &Catalog,
) -> Result<()> {
    let Some(build) = submitted.workflow.as_ref() else {
        return Ok(());
    };
    if !["strict", "exception"].contains(&build.status.as_str()) {
        return Err(AppError::new(
            "review_required",
            "This character needs GM-reviewed creation/legacy repair",
        ));
    }
    let prior = old
        .and_then(|s| s.workflow.as_ref())
        .map(|b| b.ledger.as_slice())
        .unwrap_or(&[]);
    if build.ledger.len() < prior.len() || &build.ledger[..prior.len()] != prior {
        return Err(AppError::new(
            "conflict",
            "Progression branches conflict; keep the campaign branch or propose a reviewed rebuild",
        ));
    }
    let mut current = old.cloned();
    for record in &build.ledger[prior.len()..] {
        if record.approved_by.is_some() || record.campaign_id.is_some() {
            return Err(AppError::new(
                "approval_required",
                "Approval must be recorded by this campaign's GM",
            ));
        }
        if record.before_fingerprint
            != current
                .as_ref()
                .map(fingerprint)
                .transpose()?
                .unwrap_or_default()
        {
            return Err(AppError::new("stale","Progression prerequisites changed; preview the operation against the accepted campaign sheet"));
        }
        if record.trainer_fingerprint != trainer.map(fingerprint).transpose()? {
            return Err(AppError::new(
                "stale",
                "Trainer prerequisites changed during offline progression",
            ));
        }
        let mut req: WorkflowRequest = serde_json::from_str(&record.input_json)?;
        if req.operation_id != record.id
            || req.sheet.id != submitted.id
            || req.sheet.owner_id != submitted.owner_id
        {
            return Err(AppError::new("ownership", "Invalid operation provenance"));
        }
        req.expected_revision = current.as_ref().map_or(0, |s| s.revision);
        req.trainer_revision = trainer.map(|s| s.revision);
        let next = apply(current.as_ref(), trainer, &req, catalog, None)?;
        if next.workflow.as_ref().unwrap().ledger.last().unwrap().cost != record.cost {
            return Err(AppError::new("stale", "Progression cost changed"));
        }
        current = Some(next);
    }
    let current =
        current.ok_or_else(|| AppError::new("provenance", "Creation provenance is missing"))?;
    if fingerprint(&current)? != fingerprint(submitted)?
        || current.hp != submitted.hp
        || current.conditions != submitted.conditions
        || current.workflow.as_ref().map(|b| b.will) != submitted.workflow.as_ref().map(|b| b.will)
    {
        return Err(AppError::new(
            "provenance",
            "Mechanical fields do not match the accepted progression operations",
        ));
    }
    Ok(())
}
