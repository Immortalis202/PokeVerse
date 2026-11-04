//! MOVE-A01: exact damaging-move attribute clauses, never inferred from free text.
use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct StatMoveState {
    pub enabled: bool,
    pub completed: Vec<u32>,
}

#[derive(Deserialize)]
pub struct Definition {
    dice: u32,
    user: bool,
    keys: Vec<String>,
    amount: i32,
}
pub fn definition(c: &Campaign, name: &str) -> Option<&'static Definition> {
    if completion::definition(c, name).is_some_and(|(kind, _)| kind == "stat") {
        static BLOCKS: std::sync::LazyLock<BTreeMap<String, Definition>> =
            std::sync::LazyLock::new(|| {
                let data: serde_json::Value =
                    serde_json::from_str(include_str!("../../../rules/move-completion.json"))
                        .unwrap();
                data["blocks"]
                    .as_object()
                    .unwrap()
                    .values()
                    .filter(|b| b["kind"] == "stat")
                    .flat_map(|b| {
                        b["moves"]
                            .as_object()
                            .unwrap()
                            .iter()
                            .map(|(n, d)| (n.clone(), serde_json::from_value(d.clone()).unwrap()))
                    })
                    .collect()
            });
        return BLOCKS.get(name);
    }

    if !flow(c).is_ok_and(|f| f.damaging_stats.enabled) {
        return None;
    }
    static DATA: std::sync::LazyLock<BTreeMap<String, Definition>> =
        std::sync::LazyLock::new(|| {
            let data: serde_json::Value =
                serde_json::from_str(include_str!("../../../rules/damaging-stat-moves.json"))
                    .unwrap();
            serde_json::from_value(data["moves"].clone()).unwrap()
        });
    DATA.get(name)
}
fn consumed(c: &Campaign, q: &QueuedAction, d: &Definition) -> bool {
    d.user && flow(c).is_ok_and(|f| f.damaging_stats.completed.contains(&q.sequence))
}
pub fn effect_dice(c: &Campaign, q: &QueuedAction) -> Result<Option<u32>> {
    let Some(d) = definition(c, &q.move_name) else {
        return Ok(None);
    };
    if d.dice == 0 || q.ice || consumed(c, q, d) || (provenance::enabled(c) && q.sheer_force) {
        return Ok(Some(0));
    }
    let source = reviewed::source(c, q)?;
    let target = fighter(c, &q.target_id)?;
    let bypass = source.side != target.side && reviewed::extreme(c, &source);
    let n = d.dice
        + items::completion::loaded(c, &q.actor_id)
        + 2 * u32::from(source.has_ability("Serene Grace"));
    Ok(Some(n.saturating_sub(
        2 * u32::from(!bypass && target.has_ability("Wonder Skin")),
    )))
}
pub fn effect(c: &mut Campaign, q: &QueuedAction, dice: &[u8], cat: &Catalog) -> Result<bool> {
    let Some(d) = definition(c, &q.move_name) else {
        return Ok(false);
    };
    if q.ice || consumed(c, q, d) || (provenance::enabled(c) && q.sheer_force) {
        return Ok(true);
    }
    let source = reviewed::source(c, q)?;
    let target = fighter(c, &q.target_id)?;
    let mut m = move_entry(cat, &q.move_name)?;
    special::move_data(c, &source, &mut m);
    let m = field::move_data(c, &source, &m);
    let (effectiveness, immune) =
        rules::type_matchup(c, &source, target, m["Type"].as_str().unwrap_or(""), cat)?;
    if special::target_effects_blocked(c, q, &m)
        || immune
        || abilities::damage_immune_from(c, &source, target, &m)
        || extra::source_immune(c, &source, target, effectiveness)
        || (!reviewed::extreme(c, &source)
            && (extra::effects_immune(c, target, &m) || abilities::move_immune(c, target, &m)))
    {
        return Ok(true);
    }
    // A user's single pool/change belongs to the cast, not each area recipient.
    if d.user {
        flow_mut(c).damaging_stats.completed.push(q.sequence);
    }
    let target = fighter(c, &q.target_id)?;
    if d.dice > 0
        && (!dice.contains(&6)
            || (!d.user
                && source.side != target.side
                && target.has_ability("Shield Dust")
                && !reviewed::extreme(c, &source)))
    {
        return Ok(true);
    }
    let recipient = if d.user { &q.actor_id } else { &q.target_id };
    for key in &d.keys {
        extra::change_stat(c, &q.actor_id, recipient, key, d.amount)?;
    }
    Ok(true)
}
