//! Training encounters are resolved without combat. All accepted purchases use creation::apply.
use crate::{catalog::Catalog, creation, model::*};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use ts_rs::TS;

pub const GENERATOR: u32 = 1;
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct WildIndividual {
    pub id: String,
    pub species_id: String,
    pub rank: String,
    pub ability: String,
    pub seed: String,
    pub conditions: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct TrainingEncounter {
    pub id: String,
    pub campaign_id: String,
    pub trainer_id: String,
    pub habitat_id: String,
    pub catalog_hash: String,
    pub generator_version: u32,
    pub seed: String,
    pub individuals: Vec<WildIndividual>,
    pub status: String,
    pub revision: u32,
    pub captured: BTreeMap<String, String>,
    pub rewards: BTreeMap<String, i32>,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct CaptureChoice {
    pub individual_id: String,
    pub request: WorkflowRequest,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct EncounterResolution {
    pub encounter_id: String,
    pub encounter_revision: u32,
    pub captures: Vec<CaptureChoice>,
    pub rewards: Vec<WorkflowRequest>,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct CapturePreview {
    pub request: WorkflowRequest,
    pub preview: WorkflowPreview,
}

fn hash(s: &str) -> String {
    format!("{:x}", Sha256::digest(s.as_bytes()))
}
fn draw(seed: &str, n: usize) -> usize {
    let bytes = Sha256::digest(seed.as_bytes());
    let mut a = [0; 8];
    a.copy_from_slice(&bytes[..8]);
    (u64::from_le_bytes(a) % (n as u64)) as usize
}
pub fn abilities(sp: &serde_json::Value, cat: &Catalog) -> Vec<String> {
    let mut out = vec![];
    for key in ["Ability1", "Ability2"] {
        if let Some(a) = sp[key].as_str().filter(|a| !a.is_empty()) {
            if cat.find("abilities", "Name", a).is_some() && !out.contains(&a.to_string()) {
                out.push(a.into());
            }
        }
    }
    out
}
pub fn generate(
    campaign_id: &str,
    trainer_id: &str,
    habitat_id: &str,
    rank: &str,
    size: u32,
    seed: &str,
    cat: &Catalog,
) -> Result<TrainingEncounter> {
    if !(1..=50).contains(&size)
        || creation::rank(rank).is_none()
        || seed.is_empty()
        || seed.len() > 100
        || campaign_id.len() > 100
        || trainer_id.len() > 100
    {
        return Err(AppError::new(
            "validation",
            "Choose 1–50 Pokémon, a valid rank and a seed of 1–100 characters",
        ));
    }
    let members = cat.habitat_members(habitat_id)?;
    if members.is_empty() {
        return Err(AppError::new(
            "empty_pool",
            "No accepted species in this habitat",
        ));
    }
    let key = hash(&format!(
        "{campaign_id}:{trainer_id}:{habitat_id}:{seed}:{rank}:{size}:{GENERATOR}"
    ));
    let mut individuals = vec![];
    for i in 0..size {
        let individual_seed = format!("{seed}:{i}");
        let species_id = members[draw(&individual_seed, members.len())].clone();
        let sp = cat.lookup("species", "AppId", &species_id)?.unwrap();
        let ordinary = abilities(&sp, cat);
        let ability = if ordinary.is_empty() {
            String::new()
        } else {
            ordinary[draw(&format!("{individual_seed}:ability"), ordinary.len())].clone()
        };
        individuals.push(WildIndividual {
            id: format!("wild-{}", &hash(&format!("{key}:{i}"))[..32]),
            conditions: cat.habitat_conditions(habitat_id, &species_id)?,
            species_id,
            rank: rank.into(),
            ability,
            seed: individual_seed,
        });
    }
    Ok(TrainingEncounter {
        id: format!("training-{}", &key[..32]),
        campaign_id: campaign_id.into(),
        trainer_id: trainer_id.into(),
        habitat_id: habitat_id.into(),
        catalog_hash: cat.hash.clone(),
        generator_version: GENERATOR,
        seed: seed.into(),
        individuals,
        status: "draft".into(),
        revision: 0,
        captured: BTreeMap::new(),
        rewards: BTreeMap::new(),
        reason: String::new(),
    })
}
pub fn validate(e: &TrainingEncounter, cat: &Catalog) -> Result<()> {
    if e.generator_version != GENERATOR || e.catalog_hash != cat.hash {
        return Err(AppError::new(
            "version",
            "Encounter catalog/generator changed; regenerate the draft",
        ));
    }
    if e.id.len() > 100
        || e.id.is_empty()
        || e.seed.len() > 100
        || e.individuals.is_empty()
        || e.individuals.len() > 50
        || e.reason.len() > 1000
    {
        return Err(AppError::new(
            "validation",
            "Invalid encounter size or fields",
        ));
    }
    let members: BTreeSet<_> = cat.habitat_members(&e.habitat_id)?.into_iter().collect();
    let mut seen = BTreeSet::new();
    for p in &e.individuals {
        if p.id.is_empty()
            || p.id.len() > 100
            || p.seed.len() > 120
            || p.conditions.len() > 4000
            || !seen.insert(&p.id)
            || !members.contains(&p.species_id)
            || creation::rank(&p.rank).is_none()
        {
            return Err(AppError::new(
                "validation",
                "Invalid individual or species outside the accepted habitat pool",
            ));
        }
        let sp = cat.lookup("species", "AppId", &p.species_id)?.unwrap();
        let ordinary = abilities(&sp, cat);
        if !p.ability.is_empty() && !ordinary.contains(&p.ability) {
            return Err(AppError::new(
                "validation",
                "Choose an ordinary species ability",
            ));
        }
    }
    Ok(())
}
pub fn request(
    sheet: Sheet,
    kind: &str,
    trainer: Option<&Sheet>,
    cat: &Catalog,
    reason: &str,
) -> WorkflowRequest {
    WorkflowRequest {
        operation_id: id(),
        expected_revision: sheet.revision,
        trainer_revision: trainer.map(|t| t.revision),
        rules_version: creation::VERSION,
        catalog_hash: cat.hash.clone(),
        kind: kind.into(),
        sheet,
        move_name: String::new(),
        replacement: String::new(),
        method: String::new(),
        target_species: String::new(),
        amount: 0,
        reason: reason.into(),
    }
}
fn allocate(
    values: &mut BTreeMap<String, i32>,
    keys: &[&str],
    caps: &BTreeMap<String, i32>,
    mut budget: i32,
    seed: &str,
) {
    let mut order = keys.to_vec();
    order.sort_by_key(|k| hash(&format!("{seed}:{k}")));
    while budget > 0 {
        let next = order
            .iter()
            .filter(|k| values.get(**k).copied().unwrap_or(0) < caps.get(**k).copied().unwrap_or(0))
            .min_by_key(|k| values.get(**k).copied().unwrap_or(0));
        let Some(k) = next else { break };
        *values.entry((*k).to_string()).or_default() += 1;
        budget -= 1;
    }
}
pub fn capture_preview(
    e: &TrainingEncounter,
    p: &WildIndividual,
    trainer: &Sheet,
    c: &Campaign,
    cat: &Catalog,
    edited: Option<WorkflowRequest>,
) -> Result<CapturePreview> {
    validate(e, cat)?;
    if p.ability.is_empty() {
        return Err(AppError::new(
            "adjudication",
            "Choose a reviewed ordinary ability before capture",
        ));
    }
    if e.campaign_id != c.id
        || trainer.kind != "trainer"
        || trainer.archived
        || !e.individuals.iter().any(|x| x.id == p.id)
    {
        return Err(AppError::new(
            "validation",
            "Select an active campaign trainer and encounter individual",
        ));
    }
    let sp = cat
        .lookup("species", "AppId", &p.species_id)?
        .ok_or_else(|| AppError::new("missing", "Unknown species"))?;
    let mut sheet = Sheet {
        medical: None,
        id: format!("captured-{}", &hash(&format!("{}:{}", e.id, p.id))[..32]),
        owner_id: trainer.owner_id.clone(),
        revision: 0,
        kind: "pokemon".into(),
        name: sp["Name"].as_str().unwrap_or("Pokémon").into(),
        trainer_id: Some(trainer.id.clone()),
        species_id: Some(p.species_id.clone()),
        rank: p.rank.clone(),
        nature: cat.search("natures", "", "", 0)?["entries"][0]["Name"]
            .as_str()
            .unwrap_or("")
            .into(),
        ability: p.ability.clone(),
        stats: BTreeMap::new(),
        skills: BTreeMap::new(),
        moves: vec![],
        hp: 1,
        max_hp: 1,
        xp: 0,
        inventory: BTreeMap::new(),
        held_items: None,
        stored_items: None,
        notes: format!(
            "Training encounter {} in {}. Capture confirmed by GM; victory assumed.",
            e.id, e.habitat_id
        ),
        archived: false,
        squad_order: 0,
        conditions: vec![],
        workflow: None,
    };
    let mut build = creation::build(&sheet, cat);
    build.origin = "captured".into();
    build.profile = c.rules_profile.clone();
    build.happiness = 2;
    build.loyalty = 2;
    build.training_points = 0;
    build.allocations.clear();
    build.socials.clear();
    sheet.workflow = Some(build);
    let mut req = request(
        sheet,
        "create",
        Some(trainer),
        cat,
        "Training encounter capture",
    );
    let initial = creation::preview(None, Some(trainer), &req, cat);
    let ri = creation::rank(&p.rank).unwrap();
    let caps = creation::ATTRS
        .into_iter()
        .map(|k| {
            (
                k.to_string(),
                (sp[format!("Max{k}")].as_i64().unwrap_or(0) - sp[k].as_i64().unwrap_or(0)) as i32
                    + if ri == 7 { 2 } else { 0 },
            )
        })
        .collect();
    allocate(
        &mut req.sheet.workflow.as_mut().unwrap().allocations,
        &creation::ATTRS,
        &caps,
        initial.remaining.get("Attributes").copied().unwrap_or(0),
        &p.seed,
    );
    let caps = creation::SOCIALS
        .into_iter()
        .map(|k| (k.to_string(), if ri == 7 { 6 } else { 4 }))
        .collect();
    allocate(
        &mut req.sheet.workflow.as_mut().unwrap().socials,
        &creation::SOCIALS,
        &caps,
        initial.remaining.get("Socials").copied().unwrap_or(0),
        &p.seed,
    );
    let skills = [
        "Brawl", "Channel", "Clash", "Evasion", "Alert", "Athletic", "Nature", "Stealth",
    ];
    let cap = [1, 2, 3, 4, 5, 5, 5, 5][ri];
    let caps = skills.into_iter().map(|k| (k.to_string(), cap)).collect();
    allocate(
        &mut req.sheet.skills,
        &skills,
        &caps,
        initial.remaining.get("Skills").copied().unwrap_or(0),
        &p.seed,
    );
    let preview = creation::preview(None, Some(trainer), &req, cat);
    let capacity = preview
        .derived
        .get("MoveCapacity")
        .copied()
        .unwrap_or(3)
        .max(0) as usize;
    let mut moves = sp["Moves"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|m| {
            let name = m["Name"].as_str()?;
            let rank = m["Learned"].as_str().or_else(|| m["Rank"].as_str())?;
            if creation::rank(rank).is_some_and(|r| r <= ri)
                && cat.find("moves", "Name", name).is_some()
            {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    moves.sort();
    moves.dedup();
    moves.sort_by_key(|m| hash(&format!("{}:{m}", p.seed)));
    req.sheet.moves = moves.into_iter().take(capacity).collect();
    if let Some(edited) = edited {
        req = edited;
    }
    if req.kind != "create"
        || req.sheet.kind != "pokemon"
        || req.sheet.id != format!("captured-{}", &hash(&format!("{}:{}", e.id, p.id))[..32])
        || req.sheet.owner_id != trainer.owner_id
        || req.sheet.trainer_id.as_deref() != Some(&trainer.id)
        || req.sheet.species_id.as_deref() != Some(&p.species_id)
        || req.sheet.rank != p.rank
        || req.sheet.ability != p.ability
        || req.sheet.workflow.as_ref().is_none_or(|b| {
            b.origin != "captured" || b.profile != c.rules_profile || b.training_points != 0
        })
        || !req.sheet.inventory.is_empty()
    {
        return Err(AppError::new("validation","Capture identity, origin, species, rank, ability and resources must match the encounter"));
    }
    let mut preview = creation::preview(None, Some(trainer), &req, cat);
    for m in sp["Moves"].as_array().into_iter().flatten() {
        let learned = m["Learned"].as_str().or_else(|| m["Rank"].as_str());
        if learned.is_none_or(|r| creation::rank(r).is_none())
            || cat
                .find("moves", "Name", m["Name"].as_str().unwrap_or(""))
                .is_none()
        {
            preview.warnings.push(format!("Not automatically selected: {} has missing or non-rank learning prerequisites; review before granting it",m["Name"].as_str().unwrap_or("unknown move")));
        }
    }
    Ok(CapturePreview {
        request: req,
        preview,
    })
}
fn reason(s: &str) -> Result<()> {
    if s.trim().is_empty() || s.len() > 1000 {
        Err(AppError::new(
            "validation",
            "Record a GM reason of 1–1000 characters",
        ))
    } else {
        Ok(())
    }
}
pub fn transition(
    current: &Campaign,
    actor: &str,
    action: &Action,
    cat: &Catalog,
) -> Result<(Campaign, String)> {
    if actor != current.gm_id {
        return Err(AppError::new(
            "permission",
            "Only the appointed GM may manage training encounters",
        ));
    }
    if current.battle.as_ref().is_some_and(|b| b.active) {
        return Err(AppError::new(
            "battle_locked",
            "Finish the active battle first",
        ));
    }
    let mut c = current.clone();
    let description = match action {
        Action::AdoptCatalog => {
            if c.training_encounters
                .iter()
                .any(|e| e.status == "published")
                || !c.proposals.is_empty()
            {
                return Err(AppError::new(
                    "locked",
                    "Resolve published encounters and proposals before adopting a catalog",
                ));
            }
            c.catalog_hash = cat.hash.clone();
            "GM adopted the installed catalog; prior records preserved".into()
        }
        Action::PublishTraining { encounter: e } => {
            validate(e, cat)?;
            if c.catalog_hash != cat.hash || e.campaign_id != c.id {
                return Err(AppError::new(
                    "version",
                    "Adopt the installed catalog before publishing",
                ));
            }
            let trainer = c
                .sheets
                .get(&e.trainer_id)
                .filter(|t| t.kind == "trainer" && !t.archived)
                .ok_or_else(|| AppError::new("missing", "Select an active campaign trainer"))?;
            let _ = trainer;
            let mut next = e.clone();
            next.status = "published".into();
            next.captured.clear();
            next.rewards.clear();
            next.reason.clear();
            for p in &mut next.individuals {
                p.conditions = cat.habitat_conditions(&next.habitat_id, &p.species_id)?;
            }
            if let Some(index) = c.training_encounters.iter().position(|x| x.id == e.id) {
                let old = &c.training_encounters[index];
                if old.status != "published" || old.revision != e.revision {
                    return Err(AppError::new(
                        "stale",
                        "Encounter already resolved or changed",
                    ));
                }
                next.revision = old.revision + 1;
                c.training_encounters[index] = next;
            } else {
                if e.revision != 0 || c.training_encounters.len() >= 200 {
                    return Err(AppError::new(
                        "limit",
                        "Invalid encounter revision or campaign encounter limit reached",
                    ));
                }
                next.revision = 1;
                c.training_encounters.push(next);
            }
            "GM revealed/updated a training encounter".into()
        }
        Action::CancelTraining {
            encounter_id,
            reason: r,
        } => {
            reason(r)?;
            let e = c
                .training_encounters
                .iter_mut()
                .find(|e| e.id == *encounter_id && e.status == "published")
                .ok_or_else(|| AppError::new("missing", "No open encounter"))?;
            e.status = "cancelled".into();
            e.reason = r.clone();
            e.revision += 1;
            "GM cancelled a training encounter".into()
        }
        Action::ResolveTraining { resolution: r } => {
            reason(&r.reason)?;
            let index = c
                .training_encounters
                .iter()
                .position(|e| e.id == r.encounter_id && e.status == "published")
                .ok_or_else(|| {
                    AppError::new("closed", "Encounter is already resolved or missing")
                })?;
            let e = c.training_encounters[index].clone();
            validate(&e, cat)?;
            if e.revision != r.encounter_revision
                || c.catalog_hash != cat.hash
                || r.captures.len() > 50
                || r.rewards.len() > 100
            {
                return Err(AppError::new(
                    "stale",
                    "Encounter/catalog changed or outcome is too large",
                ));
            }
            let mut seen = BTreeSet::new();
            let mut captured = BTreeMap::new();
            for choice in &r.captures {
                if !seen.insert(choice.individual_id.clone()) {
                    return Err(AppError::new("duplicate", "Duplicate capture"));
                }
                let p = e
                    .individuals
                    .iter()
                    .find(|p| p.id == choice.individual_id)
                    .ok_or_else(|| AppError::new("missing", "Unknown encounter individual"))?;
                let t = c
                    .sheets
                    .get(choice.request.sheet.trainer_id.as_deref().unwrap_or(""))
                    .ok_or_else(|| AppError::new("missing", "Capture trainer missing"))?;
                let v = capture_preview(&e, p, t, &c, cat, Some(choice.request.clone()))?;
                if c.sheets.contains_key(&v.request.sheet.id) {
                    return Err(AppError::new("duplicate", "Capture already exists"));
                }
                let mut sheet =
                    creation::apply(None, Some(t), &v.request, cat, Some((&c.id, actor)))?;
                let b = sheet.workflow.as_mut().unwrap();
                b.will = b.max_will;
                b.ledger.last_mut().unwrap().approval_reason = Some(r.reason.clone());
                sheet.hp = sheet.max_hp;
                sheet.conditions.clear();
                sheet.squad_order = c
                    .sheets
                    .values()
                    .filter(|s| s.trainer_id == sheet.trainer_id)
                    .map(|s| s.squad_order)
                    .max()
                    .unwrap_or(-1)
                    + 1;
                cat.validate(&sheet)?;
                captured.insert(p.id.clone(), sheet.id.clone());
                c.sheets.insert(sheet.id.clone(), sheet);
            }
            let mut rewards = BTreeMap::new();
            for req in &r.rewards {
                if req.kind != "award"
                    || req.amount <= 0
                    || rewards.contains_key(&req.sheet.id)
                    || captured.values().any(|id| id == &req.sheet.id)
                {
                    return Err(AppError::new(
                        "validation",
                        "Choose existing participating Pokémon and nonnegative rewards, once each",
                    ));
                }
                let old = c
                    .sheets
                    .get(&req.sheet.id)
                    .filter(|s| s.kind == "trainer" && !s.archived)
                    .ok_or_else(|| AppError::new("missing", "Trainer reward recipient missing"))?;
                let t = old.trainer_id.as_ref().and_then(|id| c.sheets.get(id));
                let mut award = req.clone();
                award.reason = format!("Training encounter {}: {}", e.id, r.reason);
                let mut next = creation::apply(Some(old), t, &award, cat, Some((&c.id, actor)))?;
                next.workflow
                    .as_mut()
                    .unwrap()
                    .ledger
                    .last_mut()
                    .unwrap()
                    .approval_reason = Some(r.reason.clone());
                let earned = next.workflow.as_ref().unwrap().training_points
                    - old.workflow.as_ref().map_or(0, |b| b.training_points);
                cat.validate(&next)?;
                rewards.insert(next.id.clone(), earned);
                c.sheets.insert(next.id.clone(), next);
            }
            let target = &mut c.training_encounters[index];
            target.status = "resolved".into();
            target.captured = captured;
            target.rewards = rewards;
            target.reason = r.reason.clone();
            target.revision += 1;
            c.battle_undo.clear();
            format!(
                "Training encounter {}: victory assumed; {} captures, {} rewards. {}",
                e.id,
                r.captures.len(),
                r.rewards.len(),
                r.reason
            )
        }
        _ => return Err(AppError::new("validation", "Not a training command")),
    };
    Ok((c, description))
}
