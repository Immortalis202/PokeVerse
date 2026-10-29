//! M2-E: source-reviewed field protection, Sleep cure and hazard installation.
use super::*;

pub fn enabled(c: &Campaign) -> bool {
    flow(c).is_ok_and(|f| f.cure_protection)
}
pub fn handles(c: &Campaign, name: &str) -> bool {
    enabled(c) && ["Safeguard", "Sparkling Aria", "Toxic Spikes", "Uproar"].contains(&name)
}
pub fn prevents_status(
    c: &Campaign,
    target: &Combatant,
    source: Option<&str>,
    cat: &Catalog,
) -> bool {
    if !enabled(c) || !field::has(c, "Safeguard", target) {
        return false;
    }
    let bypass = source.and_then(|id| fighter(c, id).ok()).is_some_and(|p| {
        // Keep the existing force-field bypass rules, including Infiltrator and sound.
        let m = flow(c)
            .ok()
            .and_then(|f| f.queue.first())
            .filter(|q| q.actor_id == p.id && q.target_id == target.id)
            .and_then(|q| move_entry(cat, &q.move_name).ok())
            .unwrap_or_else(|| serde_json::json!({}));
        p.side != target.side && field::force_field_bypass(c, p, target, &m)
    });
    !bypass
}
pub fn effect(c: &mut Campaign, q: &QueuedAction, cat: &Catalog) -> Result<bool> {
    if !handles(c, &q.move_name) || q.move_name == "Sparkling Aria" {
        return Ok(false);
    }
    let p = fighter(c, &q.actor_id)?.clone();
    let (scope, rounds) = match q.move_name.as_str() {
        "Safeguard" => (
            q.field_scope
                .clone()
                .unwrap_or_else(|| format!("side:{}", p.side)),
            Some(4),
        ),
        "Toxic Spikes" => (
            q.field_scope
                .clone()
                .unwrap_or_else(|| format!("opponents-of:{}", p.side)),
            None,
        ),
        "Uproar" => ("all".into(), Some(1)),
        _ => unreachable!(),
    };
    // Recasting refreshes the field clock; hazards do not gain video-game layers.
    flow_mut(c)
        .battlefield
        .rules
        .retain(|r| r.kind != q.move_name || r.scope != scope);
    field::insert(
        c,
        &q.move_name,
        &format!("move:{}", q.sequence),
        &scope,
        rounds,
        0,
    );
    cure_sleep(c, cat)?;
    Ok(true)
}
pub fn cure_sleep(c: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let m = move_entry(cat, "Uproar")?;
    let ids: Vec<_> = battle(c)?
        .combatants
        .iter()
        .filter(|p| {
            p.deployed
                && p.hp > 0
                && status(p, "Sleep")
                && field::has(c, "Uproar", p)
                && !p.has_ability("Comatose")
                && !abilities::move_immune(c, p, &m)
        })
        .map(|p| p.id.clone())
        .collect();
    for id in ids {
        fighter_mut(c, &id)?
            .runtime
            .statuses
            .retain(|s| s.kind != "Sleep");
    }
    Ok(())
}
