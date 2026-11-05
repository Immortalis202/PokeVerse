//! M2-C battle clauses; exact source cards and pool order live in the registry.
use self::{CompoundCommand as Command, CompoundState as State};
use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct CompoundState {
    pub enabled: bool,
    pub entered: BTreeMap<String, u32>,
    pub venom: Vec<Venom>,
    pub crashes: Vec<String>,
    pub environments: Vec<String>,
    pub alignments: BTreeMap<String, String>,
    pub relic: Vec<String>,
    pub minimized: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Venom {
    pub source: String,
    pub target: String,
    pub round: u32,
    pub applied: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export, export_to = "../../src/bindings/")]
pub enum CompoundCommand {
    Crash {
        combatant_id: String,
    },
    Environment {
        sources: Vec<String>,
    },
    Align {
        combatant_id: String,
        source: String,
    },
    Relic {
        combatant_id: String,
        change: bool,
    },
    Minimized {
        combatant_id: String,
        active: bool,
    },
}
#[derive(Deserialize)]
pub struct Definition {
    pub effects: Vec<Pool>,
    changes: Vec<Change>,
}
#[derive(Deserialize)]
pub struct Pool {
    pub kind: String,
    pub dice: u32,
}
#[derive(Deserialize)]
struct Change {
    key: String,
    amount: i32,
}
pub fn enabled(c: &Campaign) -> bool {
    flow(c).is_ok_and(|f| f.compound.enabled)
}
fn state(c: &Campaign) -> &State {
    &flow(c).unwrap().compound
}
fn state_mut(c: &mut Campaign) -> &mut State {
    &mut flow_mut(c).compound
}
pub fn registry() -> &'static serde_json::Value {
    static R: std::sync::LazyLock<serde_json::Value> = std::sync::LazyLock::new(|| {
        serde_json::from_str(include_str!("../../../rules/compound-status-moves.json")).unwrap()
    });
    &R
}
pub fn definition(c: &Campaign, name: &str) -> Option<&'static Definition> {
    if !enabled(c) {
        return None;
    }
    static R: std::sync::LazyLock<BTreeMap<String, Definition>> =
        std::sync::LazyLock::new(|| serde_json::from_value(registry()["moves"].clone()).unwrap());
    R.get(name)
}
pub fn phase(c: &Campaign, a: &Action) -> Result<()> {
    if enabled(c)
        && !state(c).crashes.is_empty()
        && !matches!(
            a,
            Action::CompoundMove {
                command: Command::Crash { .. },
                ..
            } | Action::RollbackBattle { .. }
                | Action::Override { .. }
        )
    {
        return Err(error("Resolve the pending Axe Kick crash damage first"));
    }
    Ok(())
}
pub fn dice(c: &Campaign, command: &Command) -> Result<u32> {
    if !enabled(c) {
        return Err(error("Compound move rules require a new encounter"));
    }
    if let Command::Crash { combatant_id } = command {
        if state(c).crashes.first() != Some(combatant_id) {
            return Err(error("No pending Axe Kick crash for this combatant"));
        }
        return Ok(4);
    }
    Ok(0)
}
pub fn choices(c: &Campaign, id: &str) -> Vec<String> {
    let mut choices = state(c).environments.clone();
    if let Ok(p) = fighter(c, id) {
        for t in [
            "Electric Terrain",
            "Grassy Terrain",
            "Misty Terrain",
            "Psychic Terrain",
        ] {
            if field::has(c, t, p) {
                choices.push(t.into());
            }
        }
    }
    choices.sort();
    choices.dedup();
    choices
}
fn secret(c: &Campaign, id: &str) -> Result<String> {
    let choices = choices(c, id);
    let selected = state(c).alignments.get(id).filter(|s| choices.contains(s)).cloned()
        .or_else(|| (choices.len() == 1).then(|| choices[0].clone()))
        .ok_or_else(|| AppError::new("manual_rule", "Secret Power requires confirmed surroundings and the trainer's alignment when several sources are available"))?;
    Ok(registry()["secret_power"][selected]
        .as_str()
        .unwrap()
        .into())
}
pub fn transition(
    c: &Campaign,
    actor: &str,
    command: &Command,
    why: &str,
    dice: &[u8],
    cat: &Catalog,
) -> Result<(Campaign, String)> {
    reason(why)?;
    if !state(c).crashes.is_empty() && !matches!(command, Command::Crash { .. }) {
        return Err(error("Resolve pending Axe Kick crash damage first"));
    }
    check_count(dice, self::dice(c, command)?)?;
    let mut next = c.clone();
    match command {
        Command::Crash { combatant_id } => {
            owner(c, actor, combatant_id)?;
            let p = fighter_mut(&mut next, combatant_id)?;
            p.hp = (p.hp - rules::successes(dice)).max(0);
            state_mut(&mut next).crashes.remove(0);
        }
        Command::Environment { sources } => {
            gm(c, actor)?;
            if sources.len() > 7
                || sources
                    .iter()
                    .any(|s| registry()["secret_power"][s].is_null() || s.ends_with(" Terrain"))
            {
                return Err(error("Choose the printed physical environments"));
            }
            state_mut(&mut next).environments = sources.clone();
            state_mut(&mut next).alignments.clear();
        }
        Command::Align {
            combatant_id,
            source,
        } => {
            owner(c, actor, combatant_id)?;
            if !choices(c, combatant_id).contains(source) {
                return Err(error(
                    "Alignment must use an available environment or active terrain",
                ));
            }
            state_mut(&mut next)
                .alignments
                .insert(combatant_id.clone(), source.clone());
        }
        Command::Relic {
            combatant_id,
            change,
        } => {
            owner(c, actor, combatant_id)?;
            if !state(c).relic.contains(combatant_id) || !flow(c)?.queue.is_empty() {
                return Err(error(
                    "Finish a successful Relic Song before choosing its form change",
                ));
            }
            if *change {
                forms::relic_song(&mut next, combatant_id, cat)?;
            }
            state_mut(&mut next).relic.retain(|id| id != combatant_id);
        }
        Command::Minimized {
            combatant_id,
            active,
        } => {
            gm(c, actor)?;
            fighter(c, combatant_id)?;
            state_mut(&mut next)
                .minimized
                .retain(|id| id != combatant_id);
            if *active {
                state_mut(&mut next).minimized.push(combatant_id.clone());
            }
        }
    }
    ability_changes::lifecycle(c, &mut next, cat)?;
    field::lifecycle(c, &mut next, cat)?;
    abilities::lifecycle(c, &mut next)?;
    extra::lifecycle(c, &mut next, cat)?;
    special::lifecycle(c, &mut next, cat)?;
    decoys::lifecycle(&mut next);
    sync_labels(&mut next);
    let log = format!("Compound move decision: {why}");
    checkpoint(c, &mut next, &log);
    Ok((next, log))
}
pub fn guard(c: &Campaign, id: &str, name: &str, cat: &Catalog) -> Result<()> {
    if definition(c, name).is_none() {
        return Ok(());
    }
    if name == "Fake Out" && state(c).entered.get(id).copied().unwrap_or(1) != battle(c)?.round {
        return Err(error(
            "Fake Out fails after the user's first deployed round",
        ));
    }
    if [
        "Buzzy Buzz",
        "Sizzly Slide",
        "Floaty Fall",
        "Splishy Splash",
    ]
    .contains(&name)
    {
        let p = fighter(c, id)?;
        let s = rules::base_sheet(c, p)?;
        let species = cat
            .find("species", "AppId", s.species_id.as_deref().unwrap_or(""))
            .ok_or_else(|| error("Missing evolution identity"))?;
        if !species["Evolutions"].as_array().is_some_and(|es| {
            es.iter().any(|e| {
                e["To"].is_string()
                    && !["Form", "Mega", "Primal", "Gigantamax", "Dynamax"]
                        .contains(&e["Kind"].as_str().unwrap_or(""))
            })
        }) {
            return Err(error("This move fails for a final-stage user"));
        }
    }
    if name == "Secret Power" {
        secret(c, id)?;
    }
    Ok(())
}
fn adjusted(c: &Campaign, q: &QueuedAction, p: &Pool) -> Result<u32> {
    if p.dice == 0 || q.ice || (provenance::enabled(c) && q.sheer_force) {
        return Ok(0);
    }
    let source = reviewed::source(c, q)?;
    let t = fighter(c, &q.target_id)?;
    let n = p.dice
        + items::completion::loaded(c, &q.actor_id)
        + 2 * u32::from(source.has_ability("Serene Grace"))
        + u32::from(
            (p.kind == "Frozen Solid"
                || (p.kind == "environment" && secret(c, &q.actor_id)? == "Frozen Solid"))
                && matches!(
                    field::weather(c, None),
                    Some(field::Weather::Snow | field::Weather::Hail)
                ),
        );
    Ok(n.saturating_sub(
        2 * u32::from(
            t.has_ability("Wonder Skin")
                && !(source.side != t.side && reviewed::extreme(c, &source)),
        ),
    ))
}
pub fn native_flinch(c: &Campaign, name: &str) -> Option<(&'static str, u32)> {
    let d = definition(c, name)?;
    let environment_flinch = name == "Secret Power"
        && flow(c)
            .ok()?
            .queue
            .first()
            .filter(|q| q.move_name == name)
            .is_some_and(|q| secret(c, &q.actor_id).ok().as_deref() == Some("Flinch"));
    d.effects
        .iter()
        .find(|p| {
            p.dice > 0 && (p.kind == "Flinch" || (p.kind == "environment" && environment_flinch))
        })
        .map(|p| ("Flinch", p.dice))
}
pub fn flinch_dice(c: &Campaign, q: &QueuedAction) -> Result<u32> {
    if native_flinch(c, &q.move_name).is_none() {
        return Ok(0);
    }
    let p = definition(c, &q.move_name)
        .unwrap()
        .effects
        .iter()
        .find(|p| p.kind == "Flinch" || p.kind == "environment")
        .unwrap();
    adjusted(c, q, p)
}
pub fn effect_dice(c: &Campaign, q: &QueuedAction) -> Result<Option<u32>> {
    let Some(d) = definition(c, &q.move_name) else {
        return Ok(None);
    };
    if q.move_name == "Secret Power" {
        secret(c, &q.actor_id)?;
    }
    Ok(Some(d.effects.iter().try_fold(0, |sum, p| {
        Ok::<u32, AppError>(sum + adjusted(c, q, p)?)
    })?))
}
fn blocked(c: &Campaign, q: &QueuedAction, cat: &Catalog) -> Result<bool> {
    let source = reviewed::source(c, q)?;
    let t = fighter(c, &q.target_id)?;
    let mut m = move_entry(cat, &q.move_name)?;
    special::move_data(c, &source, &mut m);
    if special::target_effects_blocked(c, q, &m)
        || (!reviewed::extreme(c, &source)
            && (extra::effects_immune(c, t, &m) || abilities::move_immune(c, t, &m)))
    {
        return Ok(true);
    }
    if m["Category"] != "Support" {
        let (n, immune) =
            rules::type_matchup(c, &source, t, m["Type"].as_str().unwrap_or(""), cat)?;
        return Ok(immune
            || abilities::damage_immune_from(c, &source, t, &m)
            || extra::source_immune(c, &source, t, n));
    }
    Ok(false)
}
fn raised(c: &Campaign, p: &Combatant) -> Result<bool> {
    let base = rules::base_sheet(c, p)?;
    let live = rules::effective_sheet(c, p)?;
    Ok(crate::creation::ATTRS
        .iter()
        .any(|k| live.stats.get(*k).unwrap_or(&0) > base.stats.get(*k).unwrap_or(&0)))
}
pub fn effect(c: &mut Campaign, q: &QueuedAction, dice: &[u8], cat: &Catalog) -> Result<bool> {
    let Some(d) = definition(c, &q.move_name) else {
        return Ok(false);
    };
    // Resolve every target's Poison against the pre-cleanup field. The separate
    // user clause still runs when the last target is immune or protected.
    let cleanse_due = q.move_name == "Mortal Spin"
        && q.targets
            .iter()
            .all(|t| t.target_id == q.target_id || t.resolved || t.avoided);
    if q.ice || blocked(c, q, cat)? {
        if cleanse_due {
            cleanse(c, &q.actor_id)?;
        }
        return Ok(true);
    }
    let name = q.move_name.as_str();
    if name == "Burning Jealousy" && !raised(c, fighter(c, &q.target_id)?)? {
        return Ok(true);
    }
    if name == "Upper Hand" && q.response_to.is_none() {
        return Ok(true);
    }
    if name == "Venom Drench" {
        let round = battle(c)?.round;
        if !state(c)
            .venom
            .iter()
            .any(|v| v.source == q.actor_id && v.target == q.target_id && v.round == round)
        {
            state_mut(c).venom.push(Venom {
                source: q.actor_id.clone(),
                target: q.target_id.clone(),
                round,
                applied: false,
            });
        }
        poisoned(c, &q.target_id)?;
    }
    for change in &d.changes {
        extra::change_stat(c, &q.actor_id, &q.target_id, &change.key, change.amount)?;
    }
    let merged = items::native_item_flinch_dice(c, q, cat)? > 0;
    let mut offset = 0;
    for p in &d.effects {
        let kind = if p.kind == "environment" {
            secret(c, &q.actor_id)?
        } else {
            p.kind.clone()
        };
        if merged && kind == "Flinch" && p.dice > 0 {
            continue;
        }
        let n = adjusted(c, q, p)? as usize;
        let trigger = p.dice == 0 || dice[offset..offset + n].contains(&6);
        let faces = &dice[offset..offset + n];
        offset += n;
        if !trigger {
            continue;
        }
        if kind == "Strength -2" {
            if fighter(c, &q.actor_id)?.side == fighter(c, &q.target_id)?.side
                || !fighter(c, &q.target_id)?.has_ability("Shield Dust")
                || reviewed::extreme(c, &reviewed::source(c, q)?)
            {
                extra::change_stat(c, &q.actor_id, &q.target_id, "Strength", -2)?;
            }
        } else if p.dice > 0 {
            interactions::apply_status_chance(c, q, faces, &kind, cat)?;
        } else {
            let detail = if kind == "In Love" {
                q.actor_id.as_str()
            } else {
                ""
            };
            abilities::inflict(c, &q.actor_id, &q.target_id, &kind, detail, None, cat)?;
        }
    }
    if name == "Relic Song" && !state(c).relic.contains(&q.actor_id) {
        state_mut(c).relic.push(q.actor_id.clone());
    }
    if cleanse_due {
        cleanse(c, &q.actor_id)?;
    }
    Ok(true)
}
pub fn poisoned(c: &mut Campaign, id: &str) -> Result<()> {
    if !enabled(c)
        || !fighter(c, id)?
            .runtime
            .statuses
            .iter()
            .any(|s| s.kind.contains("Poison"))
    {
        return Ok(());
    }
    let round = battle(c)?.round;
    let pending: Vec<_> = state(c)
        .venom
        .iter()
        .enumerate()
        .filter(|(_, v)| v.target == id && v.round == round && !v.applied)
        .map(|(i, v)| (i, v.source.clone()))
        .collect();
    for (index, source) in pending {
        state_mut(c).venom[index].applied = true;
        for key in ["Strength", "Special", "Dexterity"] {
            extra::change_stat(c, &source, id, key, -1)?;
        }
    }
    Ok(())
}
pub(super) fn cleanse(c: &mut Campaign, id: &str) -> Result<()> {
    completion::cleanse(c, id);
    let side = fighter(c, id)?.side.clone();
    let affects = |scope: &str| {
        scope == "all"
            || scope == format!("side:{side}")
            || scope == format!("combatant:{id}")
            || scope
                .strip_prefix("opponents-of:")
                .is_some_and(|s| s != side)
    };

    let scoped = flow(c)?.completion.blocks.contains("MOVE-A12");
    let sides: BTreeSet<String> = battle(c)?
        .combatants
        .iter()
        .map(|p| p.side.clone())
        .collect();
    let remaining = |scope: &str| -> Vec<String> {
        if !affects(scope) {
            return vec![scope.into()];
        }
        if !scoped {
            return vec![];
        }
        if scope == "all" {
            return vec![format!("opponents-of:{side}")];
        }
        if let Some(excluded) = scope.strip_prefix("opponents-of:") {
            return sides
                .iter()
                .filter(|s| s.as_str() != excluded && **s != side)
                .map(|s| format!("side:{s}"))
                .collect();
        }
        vec![]
    };
    let spider = entry_hazards::spider_enabled(c);
    let kinds = [
        "Spikes",
        "Toxic Spikes",
        "Stealth Rock",
        "Sticky Web",
        "Electric Terrain",
        "Grassy Terrain",
        "Misty Terrain",
        "Psychic Terrain",
    ];
    let rules = flow(c)?
        .battlefield
        .rules
        .iter()
        .flat_map(|r| {
            let scopes = if kinds.contains(&r.kind.as_str()) || (spider && r.kind == "Spider Web") {
                remaining(&r.scope)
            } else {
                vec![r.scope.clone()]
            };
            scopes
                .into_iter()
                .map(|scope| {
                    let mut r = r.clone();
                    if r.scope != scope {
                        r.id = format!("{}:cleanse:{scope}", r.id);
                    }
                    r.scope = scope;
                    r
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let effects = flow(c)?
        .field_effects
        .iter()
        .flat_map(|r| {
            let scopes = if kinds.contains(&r.move_name.as_str())
                || (spider && r.move_name == "Spider Web")
            {
                remaining(&r.scope)
            } else {
                vec![r.scope.clone()]
            };
            scopes
                .into_iter()
                .map(|scope| {
                    let mut r = r.clone();
                    r.scope = scope;
                    r
                })
                .collect::<Vec<_>>()
        })
        .collect();
    flow_mut(c).battlefield.rules = rules;
    flow_mut(c).field_effects = effects;
    flow_mut(c).special.effects.retain(|e| {
        !(e.target == id
            && [
                "Leech Seed",
                "Bind",
                "Wrap",
                "Fire Spin",
                "Whirlpool",
                "Sand Tomb",
                "Infestation",
                "Magma Storm",
                "Clamp",
                "Thunder Cage",
                "Snap Trap",
            ]
            .contains(&e.name.as_str()))
    });
    Ok(())
}
pub fn lifecycle(before: &Campaign, c: &mut Campaign, a: &Action, _cat: &Catalog) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let round = battle(c)?.round;
    let living: Vec<_> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    state_mut(c).entered.retain(|id, _| living.contains(id));
    for id in living {
        state_mut(c).entered.entry(id).or_insert(round);
    }
    state_mut(c).venom.retain(|v| v.round == round);
    if let Action::AdjudicateEffect { .. } = a {
        if let Some(q) = before
            .battle
            .as_ref()
            .and_then(|b| b.flow.as_ref())
            .and_then(|f| f.queue.first())
            .filter(|q| q.move_name == "Minimize")
        {
            if !state(c).minimized.contains(&q.actor_id) {
                state_mut(c).minimized.push(q.actor_id.clone());
            }
        }
    }
    let active: Vec<_> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    state_mut(c).minimized.retain(|id| active.contains(id));
    state_mut(c).relic.retain(|id| active.contains(id));
    if let Action::Accuracy { combatant_id, .. } | Action::AccuracyTargets { combatant_id, .. } = a
    {
        state_mut(c).relic.retain(|id| id != combatant_id);
    }
    let ids: Vec<_> = state(c).venom.iter().map(|v| v.target.clone()).collect();
    for id in ids {
        poisoned(c, &id)?;
    }
    Ok(())
}
pub fn missed(c: &mut Campaign, id: &str, name: &str, success: bool) {
    if enabled(c) && name == "Axe Kick" && !success {
        state_mut(c).crashes.push(id.into());
    }
}
pub fn minimized(c: &Campaign, id: &str, name: &str) -> bool {
    enabled(c) && name == "Stomp" && state(c).minimized.iter().any(|s| s == id)
}
pub fn move_data(c: &Campaign, p: &Combatant, m: &mut serde_json::Value) {
    if !enabled(c) || m["Name"] != "Shell Side Arm" {
        return;
    }
    if let Ok(s) = rules::effective_sheet(c, p) {
        let physical = s.stats.get("Strength").unwrap_or(&0) > s.stats.get("Special").unwrap_or(&0);
        m["Category"] = if physical { "Physical" } else { "Special" }.into();
        m["Accuracy1"] = if physical { "Strength" } else { "Special" }.into();
        m["Accuracy2"] = if physical { "Brawl" } else { "Channel" }.into();
        m["Damage1"] = if physical { "Strength" } else { "Special" }.into();
        m["Damage2"] = "".into();
    }
}
