//! Accepted trainer pools, one-way conserved transfers and atomic chain replay.
use crate::{catalog::Catalog, creation, model::*, store::Store};
use rusqlite::{params, OptionalExtension};
use std::collections::{BTreeMap, BTreeSet};

pub fn transfer_record(record: &WorkflowRecord) -> Option<PointTransferRequest> {
    serde_json::from_str::<PointTransferRequest>(&record.input_json)
        .ok()
        .filter(|r| r.kind == "point_transfer")
}
pub fn has_transfers<'a>(mut sheets: impl Iterator<Item = &'a Sheet>) -> bool {
    sheets.any(|s| {
        s.workflow
            .as_ref()
            .is_some_and(|b| b.ledger.iter().any(|r| transfer_record(r).is_some()))
    })
}
pub fn apply(
    trainer: &Sheet,
    pokemon: &Sheet,
    req: &PointTransferRequest,
    catalog: &Catalog,
) -> Result<PointTransferResult> {
    if req.kind != "point_transfer"
        || req.operation_id.trim().is_empty()
        || req.operation_id.len() > 200
        || req.reason.trim().is_empty()
        || req.reason.len() > 1000
        || req.catalog_hash != catalog.hash
    {
        return Err(AppError::new(
            "validation",
            "Transfer needs an operation ID, reason and current catalog",
        ));
    }
    if trainer.id != req.trainer_id
        || pokemon.id != req.pokemon_id
        || trainer.kind != "trainer"
        || pokemon.kind != "pokemon"
        || pokemon.trainer_id.as_ref() != Some(&trainer.id)
        || trainer.owner_id != pokemon.owner_id
        || trainer.archived
        || pokemon.archived
    {
        return Err(AppError::new(
            "ownership",
            "Select an active trainer and its own Pokemon",
        ));
    }
    if trainer.revision != req.trainer_revision || pokemon.revision != req.pokemon_revision {
        return Err(AppError::new(
            "stale",
            "Trainer or Pokemon revision changed; preview again",
        ));
    }
    if req.amount <= 0 || req.amount > 100_000 {
        return Err(AppError::new(
            "validation",
            "Transfer 1–100000 whole Training Points",
        ));
    }
    for s in [trainer, pokemon] {
        if s.workflow.as_ref().is_none_or(|b| {
            !["strict", "exception"].contains(&b.status.as_str())
                || b.version != creation::VERSION
                || b.ledger.len() >= 200
        }) {
            return Err(AppError::new(
                "review_required",
                "Review both sheets before distributing points; reconcile full ledgers",
            ));
        }
    }
    let mut next = PointTransferResult {
        trainer: trainer.clone(),
        pokemon: pokemon.clone(),
    };
    let balance = trainer.workflow.as_ref().unwrap().training_points;
    if balance < req.amount {
        return Err(AppError::new(
            "insufficient_points",
            "Trainer pool has insufficient Training Points",
        ));
    }
    let received = pokemon
        .workflow
        .as_ref()
        .unwrap()
        .training_points
        .checked_add(req.amount)
        .filter(|n| *n <= 1_000_000)
        .ok_or_else(|| AppError::new("limit", "Pokemon Training Point balance exceeds limit"))?;
    next.trainer.workflow.as_mut().unwrap().training_points = balance - req.amount;
    next.pokemon.workflow.as_mut().unwrap().training_points = received;
    next.pokemon.xp = received; // Trainer XP remains legacy data.
    for (before, after) in [(trainer, &mut next.trainer), (pokemon, &mut next.pokemon)] {
        if before
            .workflow
            .as_ref()
            .unwrap()
            .ledger
            .iter()
            .any(|r| r.id == req.operation_id)
        {
            return Err(AppError::new(
                "duplicate",
                "Transfer operation already exists",
            ));
        }
        after.revision = before
            .revision
            .checked_add(1)
            .ok_or_else(|| AppError::new("limit", "Revision exhausted"))?;
        after
            .workflow
            .as_mut()
            .unwrap()
            .ledger
            .push(WorkflowRecord {
                id: req.operation_id.clone(),
                before_fingerprint: creation::fingerprint(before)?,
                input_json: serde_json::to_string(req)?,
                cost: 0,
                trainer_fingerprint: if before.kind == "pokemon" {
                    Some(creation::fingerprint(trainer)?)
                } else {
                    None
                },
                campaign_id: None,
                approved_by: None,
                adjudications: vec![],
                approval_reason: None,
            });
        catalog.validate(after)?;
    }
    Ok(next)
}
#[derive(serde::Serialize, serde::Deserialize)]
struct Receipt {
    request: PointTransferRequest,
    result: PointTransferResult,
}
impl Store {
    pub(crate) fn linked_point_ids(
        &self,
        ids: &[String],
        campaign_id: &str,
        campaign: Option<&Campaign>,
    ) -> Result<Vec<String>> {
        let mut linked = ids.iter().cloned().collect::<BTreeSet<_>>();
        loop {
            let before = linked.len();
            for id in linked.clone() {
                let baseline: Option<String> = self
                    .library
                    .query_row(
                        "SELECT baseline FROM membership WHERE sheet_id=?1 AND campaign_id=?2",
                        params![id, campaign_id],
                        |r| r.get(0),
                    )
                    .optional()?;
                let baseline: Option<Sheet> =
                    baseline.map(|t| serde_json::from_str(&t)).transpose()?;
                let accepted = baseline
                    .as_ref()
                    .and_then(|s| s.workflow.as_ref())
                    .map(|b| {
                        b.ledger
                            .iter()
                            .map(|r| r.id.clone())
                            .collect::<BTreeSet<_>>()
                    })
                    .unwrap_or_default();
                let local = self.sheet(&id)?;
                for sheet in local
                    .as_ref()
                    .into_iter()
                    .chain(campaign.and_then(|c| c.sheets.get(&id)))
                {
                    if sheet.owner_id != self.player_id {
                        return Err(AppError::new(
                            "ownership",
                            "Linked transfer sheets must belong to this player",
                        ));
                    }
                    if let Some(build) = &sheet.workflow {
                        for record in &build.ledger {
                            if !accepted.contains(&record.id) {
                                if let Some(req) = transfer_record(record) {
                                    linked.insert(req.trainer_id);
                                    linked.insert(req.pokemon_id);
                                }
                            }
                        }
                    }
                }
            }
            if linked.len() > 100 {
                return Err(AppError::new(
                    "limit",
                    "At most 100 linked sheets can synchronize together",
                ));
            }
            if linked.len() == before {
                break;
            }
        }
        Ok(linked.into_iter().collect())
    }

    pub fn preview_point_transfer(
        &self,
        request: &PointTransferRequest,
        catalog: &Catalog,
    ) -> Result<PointTransferResult> {
        let trainer = self
            .sheet(&request.trainer_id)?
            .ok_or_else(|| AppError::new("missing", "Trainer missing"))?;
        let pokemon = self
            .sheet(&request.pokemon_id)?
            .ok_or_else(|| AppError::new("missing", "Pokemon missing"))?;
        if trainer.owner_id != self.player_id || pokemon.owner_id != self.player_id {
            return Err(AppError::new(
                "ownership",
                "Cannot distribute another player's points",
            ));
        }
        // Do not let an offline command bypass battle-frozen campaign sheets.
        let mut q = self.library.prepare("SELECT data FROM campaign_cache")?;
        for text in q.query_map([], |r| r.get::<_, String>(0))? {
            let campaign: Campaign = serde_json::from_str(&text?)?;
            if campaign.battle.as_ref().is_some_and(|b| {
                b.active
                    && b.combatants
                        .iter()
                        .any(|p| p.sheet_id == trainer.id || p.sheet_id == pokemon.id)
            }) {
                return Err(AppError::new(
                    "battle_locked",
                    "Distribute points after the battle ends",
                ));
            }
        }
        apply(&trainer, &pokemon, request, catalog)
    }
    pub fn commit_point_transfer(
        &mut self,
        request: &PointTransferRequest,
        catalog: &Catalog,
    ) -> Result<PointTransferResult> {
        if let Some(text) = self
            .library
            .query_row(
                "SELECT data FROM receipts WHERE id=?1",
                [&request.operation_id],
                |r| r.get::<_, String>(0),
            )
            .optional()?
        {
            let receipt: Receipt = serde_json::from_str(&text)
                .map_err(|_| AppError::new("duplicate", "Operation ID already accepted"))?;
            if receipt.request != *request {
                return Err(AppError::new(
                    "duplicate",
                    "Operation ID reused with different transfer",
                ));
            }
            return Ok(receipt.result);
        }
        let result = self.preview_point_transfer(request, catalog)?;
        let trainer = self.sheet(&request.trainer_id)?.unwrap();
        let pokemon = self.sheet(&request.pokemon_id)?.unwrap();
        let tx = self.library.transaction()?;
        for (before, after, role) in [
            (trainer, &result.trainer, "debit"),
            (pokemon, &result.pokemon, "credit"),
        ] {
            tx.execute(
                "UPDATE sheets SET data=?1 WHERE id=?2",
                params![serde_json::to_string(after)?, after.id],
            )?;
            let history = Progression {
                id: format!("{}:{role}", request.operation_id),
                sheet_id: after.id.clone(),
                revision: after.revision,
                description: format!("Training Point transfer {role}: {}", request.reason),
                xp_cost: if role == "debit" {
                    request.amount
                } else {
                    -request.amount
                },
                before,
                after: after.clone(),
            };
            tx.execute(
                "INSERT INTO history VALUES (?1,?2)",
                params![history.id, serde_json::to_string(&history)?],
            )?;
        }
        tx.execute(
            "INSERT INTO receipts VALUES (?1,?2)",
            params![
                request.operation_id,
                serde_json::to_string(&Receipt {
                    request: request.clone(),
                    result: result.clone()
                })?
            ],
        )?;
        tx.commit()?;
        Ok(result)
    }
}

/// Replay a coupled transfer and all dependent purchases against intermediate sheets.
/// This keeps three-way descriptive merging but rejects unmatched financial branches.
pub fn verify_batch(
    old: &BTreeMap<String, Sheet>,
    submitted: &BTreeMap<String, Sheet>,
    ids: &[String],
    catalog: &Catalog,
) -> Result<()> {
    let mut current = old.clone();
    let mut pending: BTreeMap<String, Vec<WorkflowRecord>> = BTreeMap::new();
    let mut seen: BTreeSet<String> = old
        .values()
        .filter_map(|s| s.workflow.as_ref())
        .flat_map(|b| b.ledger.iter().map(|r| r.id.clone()))
        .collect();
    for id in ids {
        let sheet = &submitted[id];
        let Some(build) = &sheet.workflow else {
            continue;
        };
        if !["strict", "exception"].contains(&build.status.as_str()) {
            return Err(AppError::new(
                "review_required",
                "Review character before synchronization",
            ));
        }
        let prior = old
            .get(id)
            .and_then(|s| s.workflow.as_ref())
            .map_or(&[][..], |b| b.ledger.as_slice());
        if build.ledger.len() < prior.len() || &build.ledger[..prior.len()] != prior {
            return Err(AppError::new(
                "conflict",
                "Reconcile the entire transfer/progression branch",
            ));
        }
        pending.insert(id.clone(), build.ledger[prior.len()..].to_vec());
    }
    while pending.values().any(|records| !records.is_empty()) {
        let mut progressed = false;
        let keys = pending.keys().cloned().collect::<Vec<_>>();
        for id in keys {
            let Some(record) = pending[&id].first().cloned() else {
                continue;
            };
            if record.approved_by.is_some() || record.campaign_id.is_some() {
                return Err(AppError::new(
                    "approval_required",
                    "Campaign GM must approve new awards and exceptions",
                ));
            }
            let before = current.get(&id);
            if record.before_fingerprint
                != before
                    .map(creation::fingerprint)
                    .transpose()?
                    .unwrap_or_default()
            {
                continue;
            }
            // A purchase can depend on the trainer between two transfers. Replay
            // those readers before advancing the trainer to the next balance.
            if before.is_some_and(|s| s.kind == "trainer")
                && pending.iter().any(|(other_id, records)| {
                    other_id != &id
                        && records.iter().any(|reader| {
                            transfer_record(reader).is_none()
                                && reader.trainer_fingerprint.as_ref()
                                    == Some(&record.before_fingerprint)
                        })
                })
            {
                continue;
            }
            if let Some(mut req) = transfer_record(&record) {
                if id != req.trainer_id {
                    continue;
                }
                let partner=pending.get(&req.pokemon_id).and_then(|v|v.first()).ok_or_else(||AppError::new("transfer_conflict","Synchronize and reconcile trainer and Pokemon transfer chains together"))?;
                if partner.id != record.id {
                    continue; // Its earlier purchases must replay before this transfer.
                }
                if transfer_record(partner).as_ref() != Some(&req)
                    || partner.id != record.id
                    || record.id != req.operation_id
                    || record.cost != 0
                    || partner.cost != 0
                    || partner.approved_by.is_some()
                    || partner.campaign_id.is_some()
                {
                    return Err(AppError::new("provenance", "Transfer pair does not match"));
                }
                let t = current.get(&req.trainer_id).ok_or_else(|| {
                    AppError::new("missing", "Trainer missing during transfer replay")
                })?;
                let p = current.get(&req.pokemon_id).ok_or_else(|| {
                    AppError::new("missing", "Pokemon missing during transfer replay")
                })?;
                if partner.before_fingerprint != creation::fingerprint(p)?
                    || partner.trainer_fingerprint != Some(creation::fingerprint(t)?)
                {
                    continue;
                }
                if !seen.insert(req.operation_id.clone()) {
                    return Err(AppError::new(
                        "duplicate",
                        "Transfer operation ID already accepted",
                    ));
                }
                req.trainer_revision = t.revision;
                req.pokemon_revision = p.revision;
                let result = apply(t, p, &req, catalog)?;
                pending.get_mut(&req.trainer_id).unwrap().remove(0);
                pending.get_mut(&req.pokemon_id).unwrap().remove(0);
                current.insert(req.trainer_id, result.trainer);
                current.insert(req.pokemon_id, result.pokemon);
                progressed = true;
            } else {
                let mut req: WorkflowRequest = serde_json::from_str(&record.input_json)?;
                let trainer = req.sheet.trainer_id.as_ref().and_then(|id| current.get(id));
                if record.trainer_fingerprint != trainer.map(creation::fingerprint).transpose()? {
                    continue;
                }
                if req.operation_id != record.id
                    || req.sheet.id != id
                    || req.sheet.owner_id != submitted[&id].owner_id
                    || !seen.insert(record.id.clone())
                {
                    return Err(AppError::new("provenance", "Invalid operation identity"));
                }
                req.expected_revision = before.map_or(0, |s| s.revision);
                req.trainer_revision = trainer.map(|s| s.revision);
                let next = creation::apply(before, trainer, &req, catalog, None)?;
                if next.workflow.as_ref().unwrap().ledger.last().unwrap().cost != record.cost {
                    return Err(AppError::new("stale", "Progression cost changed"));
                }
                current.insert(id.clone(), next);
                pending.get_mut(&id).unwrap().remove(0);
                progressed = true;
            }
        }
        if !progressed {
            return Err(AppError::new(
                "transfer_conflict",
                "Transfer/progression prerequisites changed; reconcile the complete linked chain",
            ));
        }
    }
    for id in ids {
        if submitted[id].workflow.is_none() {
            continue;
        }
        let actual = current
            .get(id)
            .ok_or_else(|| AppError::new("provenance", "Missing creation provenance"))?;
        let proposed = &submitted[id];
        if creation::fingerprint(actual)? != creation::fingerprint(proposed)?
            || actual.hp != proposed.hp
            || actual.conditions != proposed.conditions
            || actual.workflow.as_ref().map(|b| b.will)
                != proposed.workflow.as_ref().map(|b| b.will)
        {
            return Err(AppError::new(
                "provenance",
                "Transfer or dependent purchases do not match recorded operations",
            ));
        }
    }
    Ok(())
}
