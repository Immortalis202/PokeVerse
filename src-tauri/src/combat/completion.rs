//! Versioned, saved block adoption for the remaining exact-name move definitions.
use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct MoveCompletionState {
    pub blocks: BTreeSet<String>,
}
fn registry() -> &'static serde_json::Value {
    static DATA: std::sync::LazyLock<serde_json::Value> = std::sync::LazyLock::new(|| {
        serde_json::from_str(include_str!("../../../rules/move-completion.json")).unwrap()
    });
    &DATA
}
pub fn initial(engine: &str) -> MoveCompletionState {
    let blocks = registry()["blocks"].as_object().unwrap();
    let Some(last) = blocks
        .iter()
        .find(|(_, b)| b["engine"] == engine)
        .map(|(id, _)| id)
    else {
        return MoveCompletionState::default();
    };
    MoveCompletionState {
        blocks: blocks.keys().filter(|id| *id <= last).cloned().collect(),
    }
}
pub fn definition(c: &Campaign, name: &str) -> Option<(&'static str, &'static serde_json::Value)> {
    let f = flow(c).ok()?;
    registry()["blocks"]
        .as_object()?
        .iter()
        .filter(|(id, _)| f.completion.blocks.contains(*id))
        .find_map(|(_, b)| {
            b["moves"]
                .get(name)
                .map(|d| (b["kind"].as_str().unwrap(), d))
        })
}
pub fn effect_dice(c: &Campaign, q: &QueuedAction) -> Result<Option<u32>> {
    Ok(definition(c, &q.move_name).map(|_| 0))
}
fn immune_target(c: &Campaign, q: &QueuedAction, cat: &Catalog) -> Result<bool> {
    if q.ice {
        return Ok(true);
    }
    let source = reviewed::source(c, q)?;
    let target = fighter(c, &q.target_id)?;
    let mut m = move_entry(cat, &q.move_name)?;
    special::move_data(c, &source, &mut m);
    let m = field::move_data(c, &source, &m);
    if target.hp == 0
        || special::target_effects_blocked(c, q, &m)
        || (!reviewed::extreme(c, &source)
            && (extra::effects_immune(c, target, &m) || abilities::move_immune(c, target, &m)))
    {
        return Ok(true);
    }
    if m["Category"] != "Support" {
        let (n, immune) =
            rules::type_matchup(c, &source, target, m["Type"].as_str().unwrap_or(""), cat)?;
        return Ok(immune
            || abilities::damage_immune_from(c, &source, target, &m)
            || extra::source_immune(c, &source, target, n));
    }
    Ok(false)
}
fn install(c: &mut Campaign, q: &QueuedAction, source: &str, target: &str) -> Result<()> {
    let p = fighter(c, target)?;
    if p.hp == 0 || (p.has_ability("Run Away") && !house::ring(c, p)) {
        return Ok(());
    }
    let name = format!("move-block:{}", q.move_name);
    if flow(c)?
        .special
        .effects
        .iter()
        .any(|e| e.name == name && e.target == target)
    {
        return Ok(());
    }
    let started = battle(c)?.round;
    flow_mut(c).special.effects.push(special::TimedEffect {
        name,
        source: source.into(),
        target: target.into(),
        rounds: None,
        amount: 0,
        started,
        slot: String::new(),
    });
    Ok(())
}
pub fn effect(c: &mut Campaign, q: &QueuedAction, _dice: &[u8], cat: &Catalog) -> Result<bool> {
    let Some((kind, d)) = definition(c, &q.move_name) else {
        return Ok(false);
    };
    match kind {
        "trap" => {
            if !immune_target(c, q, cat)? {
                install(c, q, &q.actor_id, &q.target_id)?;
                if d["both"] == true {
                    install(c, q, &q.target_id, &q.actor_id)?;
                }
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}
pub fn blocked(c: &Campaign, id: &str) -> bool {
    if !flow(c).is_ok_and(|f| f.completion.blocks.contains("MOVE-A03")) {
        return false;
    }
    let Ok(p) = special::registered(c, id) else {
        return false;
    };
    if !p.deployed || p.hp == 0 {
        return false;
    }
    if p.has_ability("Run Away") && !house::ring(c, p) {
        return false;
    }
    flow(c).unwrap().special.effects.iter().any(|e| {
        e.target == id
            && e.name.starts_with("move-block:")
            && (e.name != "move-block:Jaw Lock"
                || special::registered(c, &e.source).is_ok_and(|p| p.deployed && p.hp > 0))
    })
}
