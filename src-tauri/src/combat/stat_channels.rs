//! Scene-local stat snapshots. Ability identity and permanent sheets are never copied.
use super::*;
pub const KEYS: [&str; 7] = [
    "Strength",
    "Dexterity",
    "Vitality",
    "Special",
    "Insight",
    "Defense",
    "SpecialDefense",
];
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct StatChannels {
    pub enabled: bool,
    pub active: BTreeSet<String>,
    pub grants: Vec<StatGrant>,
    pub removed: BTreeMap<String, BTreeMap<String, (i32, i32)>>,
    pub removed_abilities: BTreeMap<String, String>,
    pub choices: BTreeMap<String, BTreeMap<String, Vec<StatGrant>>>,
    pub history: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct StatGrant {
    pub source: String,
    pub target: String,
    pub cause: String,
    pub key: String,
    pub channel: String,
    pub positive: i32,
    pub negative: i32,
}
pub fn enabled(c: &Campaign) -> bool {
    flow(c).is_ok_and(|f| f.stat_channels.enabled)
}
fn raw(c: &Campaign, p: &Combatant, key: &str) -> (i32, i32) {
    if key == "Defense" || key == "SpecialDefense" {
        extra::defense_channel(c, p, key == "Defense")
    } else {
        extra::attribute_channel(c, p, key)
    }
}
pub fn channel(c: &Campaign, p: &Combatant, key: &str) -> (i32, i32) {
    let (mut positive, mut negative) = raw(c, p, key);
    if !enabled(c) {
        return (positive, negative);
    }
    let state = &flow(c).unwrap().stat_channels;
    if let Some((up, down)) = state.removed.get(&p.id).and_then(|v| v.get(key)) {
        if positive <= *up {
            positive = permanent(c, p, key);
        }
        if negative >= *down {
            negative = 0;
        }
    }
    if tactics::baton_blocked(c, &p.id, &p.id, key, 1) {
        positive = permanent(c, p, key);
    }
    for g in state
        .grants
        .iter()
        .filter(|g| g.target == p.id && g.key == key && g.channel == "ability")
    {
        if !tactics::baton_blocked(c, &p.id, &p.id, key, 1) {
            positive = positive.max(g.positive);
        }
        negative = negative.min(g.negative);
    }
    (positive, negative)
}
fn snapshot(
    c: &Campaign,
    p: &Combatant,
    target: &str,
    cause: &str,
    reductions: bool,
    moves: bool,
) -> Vec<StatGrant> {
    let mut grants = vec![];
    for key in KEYS {
        let (positive, negative) = channel(c, p, key);
        let positive = (positive - permanent(c, p, key)).max(0);
        if positive > 0 || (reductions && negative < 0) {
            grants.push(StatGrant {
                source: p.id.clone(),
                target: target.into(),
                cause: cause.into(),
                key: key.into(),
                channel: "ability".into(),
                positive,
                negative: if reductions { negative } else { 0 },
            });
        }
        if moves {
            // Preserve the two non-stacking move channels, including Contrary/Simple.
            let mut values: Vec<i32> = flow(c)
                .unwrap()
                .special
                .effects
                .iter()
                .filter(|e| {
                    e.target == p.id
                        && (e.name == format!("buff:{key}") || e.name == format!("debuff:{key}"))
                })
                .map(|e| match p.ability_for(&["Contrary", "Simple"]) {
                    "Contrary" => -e.amount,
                    "Simple" => e.amount + e.amount.signum(),
                    _ => e.amount,
                })
                .collect();
            if values.is_empty() {
                let amount = p.runtime.modifiers.get(key).copied().unwrap_or(0);
                values.push(match p.ability_for(&["Contrary", "Simple"]) {
                    "Contrary" => -amount,
                    "Simple" => amount + amount.signum(),
                    _ => amount,
                });
                if key == "Defense"
                    && flow(c)
                        .unwrap()
                        .special
                        .charges
                        .get(&p.id)
                        .is_some_and(|q| q.action.move_name == "Skull Bash")
                {
                    values.push(match p.ability_for(&["Contrary", "Simple"]) {
                        "Contrary" => -1,
                        "Simple" => 2,
                        _ => 1,
                    });
                }
            }
            values.extend(move_values(c, p, key));
            let positive = values.iter().copied().max().unwrap_or(0).max(0);
            let negative = if reductions {
                values.iter().copied().min().unwrap_or(0).min(0)
            } else {
                0
            };
            if positive > 0 || negative < 0 {
                grants.push(StatGrant {
                    source: p.id.clone(),
                    target: target.into(),
                    cause: cause.into(),
                    key: key.into(),
                    channel: "move".into(),
                    positive,
                    negative,
                });
            }
        }
    }
    grants
}
fn apply(c: &mut Campaign, grants: Vec<StatGrant>) -> Result<()> {
    for g in grants {
        if g.cause == "Baton Pass"
            && special::registered(c, &g.target)?.has_ability("Clear Body")
            && g.source != g.target
        {
            continue;
        }
        if g.positive > 0 && tactics::baton_blocked(c, &g.target, &g.target, &g.key, 1) {
            continue;
        }
        if let Some(existing) = flow_mut(c).stat_channels.grants.iter_mut().find(|v| {
            v.source == g.source
                && v.target == g.target
                && v.key == g.key
                && v.channel == g.channel
                && v.cause == g.cause
        }) {
            if existing.positive >= g.positive && existing.negative <= g.negative {
                continue;
            }
            existing.positive = existing.positive.max(g.positive);
            existing.negative = existing.negative.min(g.negative);
        } else {
            flow_mut(c).stat_channels.grants.push(g.clone());
        }
        let source_name = special::registered(c, &g.source)?.name.clone();
        let target_name = special::registered(c, &g.target)?.name.clone();
        flow_mut(c).stat_channels.history.push(format!(
            "{}: {} -> {}, {} {} +{}/{}",
            g.cause, source_name, target_name, g.channel, g.key, g.positive, g.negative
        ));
    }
    Ok(())
}
pub fn transfer(c: &mut Campaign, source: &str, target: &str, cause: &str) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let p = special::registered(c, source)?.clone();
    let reductions = cause == "Snatch";
    let grants = snapshot(c, &p, target, cause, reductions, true)
        .into_iter()
        .filter(|g| {
            cause != "Baton Pass" || !["Defense", "SpecialDefense"].contains(&g.key.as_str())
        })
        .collect::<Vec<_>>();
    if reductions {
        flow_mut(c)
            .stat_channels
            .removed_abilities
            .insert(source.into(), p.selected_ability.clone());
        let captured: BTreeMap<_, _> = KEYS
            .into_iter()
            .map(|key| (key.to_string(), raw(c, &p, key)))
            .collect();
        flow_mut(c)
            .stat_channels
            .removed
            .insert(source.into(), captured);
        flow_mut(c)
            .stat_channels
            .grants
            .retain(|g| g.target != source || g.channel != "ability");
        fighter_mut(c, source)?
            .runtime
            .ability
            .counters
            .retain(|k, _| {
                !k.starts_with("boost:")
                    && !k.starts_with("debuff:")
                    && ![
                        "Moxie",
                        "Sap Sipper",
                        "Speed Boost",
                        "Lightning Rod",
                        "Storm Drain",
                    ]
                    .contains(&k.as_str())
            });
    }
    apply(c, grants)
}
pub fn resolve(c: &mut Campaign, e: &abilities::AbilityEvent, choice: &str) -> Result<()> {
    let p = fighter(c, &e.combatant_id)?;
    if !p.deployed || p.hp <= 0 || !p.has_ability("Costar") {
        return Err(error("Costar is no longer active"));
    }
    let grants = flow_mut(c)
        .stat_channels
        .choices
        .remove(&e.id)
        .and_then(|mut choices| choices.remove(choice))
        .ok_or_else(|| error("Choose one of the frozen Costar ally snapshots"))?;
    apply(c, grants)
}
pub fn lifecycle(before: &Campaign, c: &mut Campaign) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let active: BTreeSet<_> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0 && battle(c).unwrap().active)
        .map(|p| p.id.clone())
        .collect();
    let old = flow(c)?.stat_channels.active.clone();
    let costars: BTreeSet<_> = battle(c)?
        .combatants
        .iter()
        .filter(|p| active.contains(&p.id) && p.has_ability("Costar"))
        .map(|p| p.id.clone())
        .collect();
    flow_mut(c)
        .ability_events
        .retain(|e| e.kind != "stat-costar" || costars.contains(&e.combatant_id));
    flow_mut(c)
        .stat_channels
        .grants
        .retain(|g| active.contains(&g.target));
    flow_mut(c)
        .stat_channels
        .removed
        .retain(|id, _| active.contains(id));
    let identities = flow(c)?.stat_channels.removed_abilities.clone();
    for (id, identity) in identities {
        if !active.contains(&id) || fighter(c, &id)?.selected_ability != identity {
            flow_mut(c).stat_channels.removed.remove(&id);
            flow_mut(c).stat_channels.removed_abilities.remove(&id);
        }
    }
    // A stolen conditional channel can activate again only after its condition clears.
    let removed = flow(c)?.stat_channels.removed.clone();
    for (id, keys) in removed {
        for key in keys.keys() {
            let (up, down) = raw(c, fighter(c, &id)?, key);
            let pair = flow_mut(c)
                .stat_channels
                .removed
                .get_mut(&id)
                .unwrap()
                .get_mut(key)
                .unwrap();
            if up == 0 {
                pair.0 = 0;
            }
            if down == 0 {
                pair.1 = 0;
            }
        }
    }
    let ps = battle(c)?.combatants.clone();
    let entry_snapshots = c.clone();
    for p in ps
        .iter()
        .filter(|p| active.contains(&p.id) && !old.contains(&p.id) && p.has_ability("Costar"))
    {
        let choices: BTreeMap<_, _> = ps
            .iter()
            .filter(|q| active.contains(&q.id) && q.id != p.id && q.side == p.side)
            .filter_map(|q| {
                let grants = snapshot(&entry_snapshots, q, &p.id, "Costar", false, true);
                if grants.is_empty() {
                    None
                } else {
                    Some((q.id.clone(), grants))
                }
            })
            .collect();
        if choices.len() == 1 {
            apply(c, choices.into_values().next().unwrap())?;
        } else if !choices.is_empty() {
            let id = format!("costar:{}:{}:{}", p.id, c.revision + 1, battle(c)?.round);
            flow_mut(c).ability_events.push(abilities::AbilityEvent {
                id: id.clone(),
                combatant_id: p.id.clone(),
                target_id: p.id.clone(),
                ability: "Costar".into(),
                kind: "stat-costar".into(),
                move_name: "Choose an ally's entry-time increases".into(),
                choices: choices.keys().cloned().collect(),
            });
            flow_mut(c).stat_channels.choices.insert(id, choices);
        }
    }
    // Compare immutable before/after channels, then apply once. This prevents
    // reciprocal Opportunists from recursively generating increases.
    let observed = c.clone();
    for p in ps
        .iter()
        .filter(|p| active.contains(&p.id) && p.has_ability("Opportunist"))
    {
        for q in ps
            .iter()
            .filter(|q| active.contains(&q.id) && q.side != p.side)
        {
            let previous = before.battle.as_ref().and_then(|b| {
                b.combatants
                    .iter()
                    .find(|a| a.id == q.id && a.deployed && a.hp > 0)
            });
            let now = snapshot(&observed, q, &p.id, "Opportunist", false, true);
            let was = previous.map_or_else(Vec::new, |previous| {
                snapshot(before, previous, &p.id, "Opportunist", false, true)
            });
            let grants = now
                .into_iter()
                .filter(|g| {
                    g.positive
                        > was
                            .iter()
                            .find(|v| v.key == g.key && v.channel == g.channel)
                            .map_or(0, |v| v.positive)
                })
                .collect();
            apply(c, grants)?;
        }
    }
    let pending: BTreeSet<_> = flow(c)?
        .ability_events
        .iter()
        .filter(|e| e.kind == "stat-costar")
        .map(|e| e.id.clone())
        .collect();
    flow_mut(c)
        .stat_channels
        .choices
        .retain(|id, _| pending.contains(id));
    flow_mut(c).stat_channels.active = active;
    Ok(())
}

pub fn permanent(c: &Campaign, p: &Combatant, key: &str) -> i32 {
    if forms::types(c, &p.id).is_some() {
        return 0;
    }
    if key == "Strength"
        && ["Huge Power", "Pure Power"].contains(&if ability_changes::enabled(c) {
            c.sheets[&p.sheet_id].ability.as_str()
        } else {
            p.selected_ability.as_str()
        })
    {
        2
    } else {
        0
    }
}
pub fn move_values(c: &Campaign, p: &Combatant, key: &str) -> Vec<i32> {
    if !enabled(c) {
        return vec![];
    }
    flow(c)
        .unwrap()
        .stat_channels
        .grants
        .iter()
        .filter(|g| g.target == p.id && g.key == key && g.channel == "move")
        .flat_map(|g| [g.positive, g.negative])
        .filter(|n| *n != 0)
        .collect()
}

pub fn finish(c: &mut Campaign) {
    if let Some(f) = c
        .battle
        .as_mut()
        .and_then(|b| b.flow.as_mut())
        .filter(|f| f.stat_channels.enabled)
    {
        f.stat_channels.grants.clear();
        f.stat_channels.removed.clear();
        f.stat_channels.removed_abilities.clear();
        f.stat_channels.choices.clear();
        f.stat_channels.active.clear();
        f.ability_events.retain(|e| e.kind != "stat-costar");
    }
}

pub fn copy_transform(c: &mut Campaign, source: &Combatant, target: &str) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let grants = snapshot(c, source, target, "Transform", true, true);
    flow_mut(c)
        .stat_channels
        .grants
        .retain(|g| g.target != target);
    apply(c, grants)
}
