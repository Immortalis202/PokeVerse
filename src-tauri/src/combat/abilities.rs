//! Reviewed Pokerole abilities. Scene flags and temporary stat channels are separate.
use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct AbilityState {
    pub flags: BTreeSet<String>,
    pub counters: BTreeMap<String, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct AbilityEvent {
    pub id: String,
    pub combatant_id: String,
    pub target_id: String,
    pub ability: String,
    pub kind: String,
    pub move_name: String,
    pub choices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct DamageContext {
    #[serde(default)]
    pub special_bonus: BTreeMap<String, i32>,
    #[serde(default)]
    pub ability_sources: Vec<Combatant>,
    #[serde(default)]
    pub field_snapshot: Option<field::FieldSnapshot>,
    #[serde(default)]
    pub types: Vec<String>,
    #[serde(default)]
    pub field_bonus: i32,
    pub stats: BTreeMap<String, i32>,
    pub hp: i32,
    pub pain_ignored: i32,
    pub counters: BTreeMap<String, i32>,
    pub statuses: Vec<Status>,
    pub love_resisted_turn: u32,
}
pub fn context<'a>(c: &'a Campaign, actor: &str, sequence: u32) -> Option<&'a DamageContext> {
    c.battle
        .as_ref()?
        .flow
        .as_ref()?
        .queue
        .iter()
        .find(|q| q.actor_id == actor && q.sequence == sequence)?
        .damage_context
        .as_ref()
}

pub fn enabled(c: &Campaign) -> bool {
    c.battle
        .as_ref()
        .and_then(|b| b.flow.as_ref())
        .is_some_and(|f| f.ability_rules)
}
pub fn pending(c: &Campaign) -> bool {
    enabled(c)
        && c.battle
            .as_ref()
            .and_then(|b| b.flow.as_ref())
            .is_some_and(|f| !f.ability_events.is_empty())
}
pub fn guard(c: &Campaign, source: &Combatant) -> Result<()> {
    if enabled(c)
        && c.battle.as_ref().is_some_and(|b| b.automatic_validation)
        && (["Mold Breaker", "Teravolt", "Turboblaze"].contains(&source.selected_ability.as_str())
            || c.battle
                .as_ref()
                .unwrap()
                .combatants
                .iter()
                .any(|p| p.deployed && p.hp > 0 && p.selected_ability == "Neutralizing Gas"))
    {
        return Err(AppError::new("manual_rule","Ability suppression/bypass requires a GM ruling; automatic rolls cannot assume the ordinary ability interaction"));
    }
    Ok(())
}
pub fn move_guard(c: &Campaign, name: &str) -> Result<()> {
    if enabled(c)
        && [
            "Self-Destruct",
            "Explosion",
            "Misty Explosion",
            "Mind Blown",
            "Burn Up",
        ]
        .contains(&name)
        && battle(c)?
            .combatants
            .iter()
            .any(|p| p.deployed && p.hp > 0 && p.selected_ability == "Damp")
    {
        return Err(error(
            "Damp prevents this move while its holder is on the field",
        ));
    }
    Ok(())
}
pub fn status_damage_immune(on: bool, p: &Combatant, kind: &str) -> bool {
    on && (p.selected_ability == "Magic Guard"
        || (p.selected_ability == "Immunity" && kind.contains("Poison")))
}
pub fn phase(c: &Campaign, action: &Action) -> Result<()> {
    if pending(c)
        && matches!(
            action,
            Action::PassTurn { .. }
                | Action::TakeHeldTurn { .. }
                | Action::PassReaction { .. }
                | Action::StatusCheck { .. }
                | Action::StartTurns
                | Action::NextRound
                | Action::Accuracy { .. }
                | Action::AccuracyTargets { .. }
                | Action::React { .. }
                | Action::CoverAlly { .. }
                | Action::BreakIce { .. }
                | Action::Damage { .. }
                | Action::ResolveEffect { .. }
                | Action::AdjudicateEffect { .. }
                | Action::SwitchPokemon { .. }
                | Action::EndBattle
                | Action::Initiative { .. }
                | Action::Roll { .. }
                | Action::Resolve { .. }
        )
    {
        return Err(error(
            "Resolve the pending ability trigger before continuing the battle",
        ));
    }
    Ok(())
}
pub fn coverage(c: &Campaign, name: &str) -> Option<&'static str> {
    static REGISTRY: std::sync::LazyLock<serde_json::Value> = std::sync::LazyLock::new(|| {
        serde_json::from_str(include_str!("../../../rules/ability-coverage.json"))
            .expect("valid owned ability registry")
    });
    let entry = &REGISTRY["entries"][name];
    if (entry["introduced_engine"] == "0.7.0-core" && !extra::enabled(c))
        || (entry["introduced_engine"] == "0.8.0-core" && !special::enabled(c))
        || (entry["introduced_engine"] == "0.6.0-core" && !field::enabled(c))
    {
        return Some("This ability requires a newer encounter engine; upgrade between battles or record a GM ruling");
    }
    entry["scope"].as_str()
}
pub fn status_immune(c: &Campaign, p: &Combatant, kind: &str) -> bool {
    enabled(c)
        && match p.selected_ability.as_str() {
            "Limber" => kind == "Paralysis",
            "Insomnia" | "Vital Spirit" => kind == "Sleep",
            "Water Veil" => kind.starts_with("Burn"),
            "Magma Armor" => kind == "Frozen Solid",
            "Inner Focus" => kind == "Flinch",
            _ => false,
        }
}
pub fn stats(c: &Campaign, p: &Combatant, s: &mut Sheet) {
    if !enabled(c) {
        return;
    }
    let ailment = p.runtime.statuses.iter().any(|v| {
        v.kind.starts_with("Burn")
            || v.kind.contains("Poison")
            || ["Paralysis", "Sleep", "Frozen Solid"].contains(&v.kind.as_str())
    });
    let bonus = match p.selected_ability.as_str() {
        "Guts" if ailment => Some(("Strength", 2)),
        "Moxie" => Some((
            "Strength",
            *p.runtime.ability.counters.get("Moxie").unwrap_or(&0),
        )),
        "Sap Sipper" => Some((
            "Strength",
            *p.runtime.ability.counters.get("Sap Sipper").unwrap_or(&0),
        )),
        "Speed Boost" => Some((
            "Dexterity",
            *p.runtime.ability.counters.get("Speed Boost").unwrap_or(&0),
        )),
        _ => None,
    };
    let intimidated = p.selected_ability != "Inner Focus"
        && flow(c).is_ok_and(|f| f.intimidate.values().any(|ids| ids.contains(&p.id)));
    if let Some((key, n)) = bonus {
        if let Some(v) = s.stats.get_mut(key) {
            *v = (*v + n - i32::from(intimidated && key == "Strength")).clamp(1, 10);
        }
    }
    if intimidated && bonus.is_none_or(|(key, _)| key != "Strength") {
        if let Some(v) = s.stats.get_mut("Strength") {
            *v = (*v - 1).max(1);
        }
    }
}
pub fn defense(c: &Campaign, p: &Combatant, physical: bool) -> i32 {
    if extra::enabled(c) {
        return extra::defense(c, p, physical);
    }
    i32::from(
        enabled(c)
            && physical
            && p.selected_ability == "Marvel Scale"
            && p.runtime.statuses.iter().any(|s| {
                s.kind.starts_with("Burn")
                    || s.kind.contains("Poison")
                    || ["Paralysis", "Sleep", "Frozen Solid"].contains(&s.kind.as_str())
            }),
    ) * 2
}
pub fn critical(c: &Campaign, attacker: &Combatant, target: &Combatant) -> i32 {
    if !enabled(c) {
        return 2;
    }
    if attacker.side != target.side
        && ["Battle Armor", "Shell Armor"].contains(&target.selected_ability.as_str())
    {
        0
    } else if attacker.selected_ability == "Sniper" {
        3
    } else {
        2
    }
}
pub fn low_accuracy(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> i32 {
    i32::from(enabled(c) && p.selected_ability == "Hustle" && m["Category"] == "Physical")
}
pub fn damage_bonus(
    c: &Campaign,
    p: &Combatant,
    m: &serde_json::Value,
    stab: bool,
    counters: &BTreeMap<String, i32>,
) -> i32 {
    let _ = counters;
    if !enabled(c) {
        return 0;
    }
    match p.selected_ability.as_str() {
        "Adaptability" if stab => 1,
        "Technician" if matches!(m["Power"].as_i64(), Some(1 | 2)) => 1,
        "Iron Fist" if m["Attributes"]["FistMove"] == true => 1,
        "Hustle" if m["Category"] == "Physical" => 2,
        "Flash Fire" if m["Type"] == "Fire" && p.runtime.ability.flags.contains("Flash Fire") => 1,
        _ => 0,
    }
}
pub fn move_immune(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> bool {
    enabled(c)
        && ((p.selected_ability == "Levitate"
            && m["Type"] == "Ground"
            && !field::has(c, "Gravity", p)
            && !special::roost(c, &p.id))
            || (p.selected_ability == "Soundproof" && m["Attributes"]["SoundMove"] == true))
}
pub fn damage_immune(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> bool {
    extra::immune(c, p, m)
        || move_immune(c, p, m)
        || (enabled(c)
            && match p.selected_ability.as_str() {
                "Flash Fire" => m["Type"] == "Fire",
                "Sap Sipper" => m["Type"] == "Grass",
                "Water Absorb" => m["Type"] == "Water",
                "Dry Skin" if field::enabled(c) => m["Type"] == "Water",
                "Volt Absorb" => m["Type"] == "Electric",
                _ => false,
            })
}
pub fn reduction(
    c: &Campaign,
    p: &Combatant,
    m: &serde_json::Value,
    effective: i32,
    source: &str,
) -> i32 {
    if !enabled(c) {
        return 0;
    }
    extra::reduction(c, p, m, effective)
        - i32::from(
            extra::enabled(c)
                && effective > 0
                && fighter(c, source).is_ok_and(|p| p.selected_ability == "Neuroforce"),
        )
        + match p.selected_ability.as_str() {
            "Dry Skin" if field::enabled(c) && m["Type"] == "Fire" => -1,
            "Thick Fat" if m["Type"] == "Fire" || m["Type"] == "Ice" => 1,
            "Filter" if effective > 0 && fighter(c, source).is_ok_and(|a| a.side != p.side) => 1,
            "Solid Rock" if effective > 0 => 1,
            "Multiscale" if p.hp == p.max_hp => 1,
            "Fur Coat" if m["Category"] == "Physical" => 2,
            _ => 0,
        }
}
pub fn contact(m: &serde_json::Value) -> bool {
    m["Category"] == "Physical"
        && m["Attributes"]["Ranged"] != true
        && m["Attributes"]["PhysicalRanged"] != true
}
fn damage(c: &mut Campaign, id: &str, amount: i32) -> Result<()> {
    if enabled(c)
        && (fighter(c, id)?.selected_ability == "Magic Guard"
            || (extra::enabled(c) && fighter(c, id)?.selected_ability == "Wonder Guard"))
    {
        return Ok(());
    }
    let p = fighter_mut(c, id)?;
    p.hp = (p.hp - amount).max(0);
    Ok(())
}
fn enqueue(c: &mut Campaign, p: &Combatant, target: &str, kind: &str, m: &str, choices: &[&str]) {
    let id = format!(
        "{}:{}:{}",
        c.revision + 1,
        flow(c).unwrap().ability_events.len(),
        p.id
    );
    flow_mut(c).ability_events.push(AbilityEvent {
        id,
        combatant_id: p.id.clone(),
        target_id: target.into(),
        ability: p.selected_ability.clone(),
        kind: kind.into(),
        move_name: m.into(),
        choices: choices.iter().map(|v| (*v).into()).collect(),
    });
}
pub fn sturdy(c: &mut Campaign, target: &str, amount: i32, before_hp: i32) -> Result<i32> {
    let amount = extra::disguise(c, target, amount)?;
    let p = fighter(c, target)?;
    if enabled(c)
        && p.selected_ability == "Sturdy"
        && before_hp > 0
        && amount >= before_hp
        && !p.runtime.ability.flags.contains("Sturdy")
    {
        let result = before_hp - 1;
        fighter_mut(c, target)?
            .runtime
            .ability
            .flags
            .insert("Sturdy".into());
        Ok(result)
    } else {
        Ok(amount)
    }
}
pub fn clash_amount(
    c: &mut Campaign,
    id: &str,
    m: &serde_json::Value,
    raw: i32,
    source: &str,
) -> Result<i32> {
    let p = fighter(c, id)?;
    if damage_immune(c, p, m) {
        return Ok(0);
    }
    let reduced = (raw
        - reduction(c, p, m, raw - 1, source)
        - field::reduction(c, fighter(c, source)?, p, m))
    .max(0);
    let before_hp = p.hp;
    sturdy(c, id, reduced, before_hp)
}
pub fn hit(
    c: &mut Campaign,
    q: &QueuedAction,
    before_hp: i32,
    m: &serde_json::Value,
) -> Result<()> {
    if !enabled(c) || q.actor_id == q.target_id {
        return Ok(());
    }
    let target = fighter(c, &q.target_id)?.clone();
    let attacker = fighter(c, &q.actor_id)?.clone();
    let resolved = field::move_data(c, &attacker, m);
    let m = &resolved;
    if target.hp == 0
        && before_hp > 0
        && attacker.hp > 0
        && attacker.side != target.side
        && attacker.selected_ability == "Moxie"
    {
        let v = fighter_mut(c, &q.actor_id)?
            .runtime
            .ability
            .counters
            .entry("Moxie".into())
            .or_default();
        *v = (*v + 1).min(3);
    }
    match target.selected_ability.as_str() {
        "Flash Fire" if m["Type"] == "Fire" => {
            fighter_mut(c, &q.target_id)?
                .runtime
                .ability
                .flags
                .insert("Flash Fire".into());
        }
        "Sap Sipper" if m["Type"] == "Grass" => {
            let state = &mut fighter_mut(c, &q.target_id)?.runtime.ability;
            if state.flags.insert("Sap Sipper".into()) {
                state.counters.insert("Sap Sipper".into(), 1);
            }
        }
        "Water Absorb" | "Dry Skin"
            if m["Type"] == "Water"
                && (target.selected_ability != "Dry Skin" || field::enabled(c)) =>
        {
            enqueue(
                c,
                &target,
                &target.id,
                "heal",
                &q.move_name,
                &["heal", "decline"],
            )
        }
        "Volt Absorb" if m["Type"] == "Electric" => enqueue(
            c,
            &target,
            &target.id,
            "heal",
            &q.move_name,
            &["heal", "decline"],
        ),
        _ => {}
    }
    extra::hit(c, q, before_hp, m)?;
    if move_immune(c, &target, m)
        || !contact(m)
        || attacker.hp == 0
        || attacker.selected_ability == "Long Reach"
    {
        return Ok(());
    }
    match target.selected_ability.as_str() {
        "Static" | "Poison Point" | "Flame Body" | "Cute Charm" | "Effect Spore" => {
            enqueue(c, &target, &q.actor_id, "contact", &q.move_name, &["roll"])
        }
        "Rough Skin" | "Iron Barbs" => enqueue(
            c,
            &target,
            &q.actor_id,
            "retaliate",
            &q.move_name,
            &["neutral", "roll"],
        ),
        "Aftermath"
            if target.hp == 0
                && before_hp > 0
                && !battle(c)?
                    .combatants
                    .iter()
                    .any(|p| p.deployed && p.hp > 0 && p.selected_ability == "Damp") =>
        {
            damage(c, &q.actor_id, 2)?
        }
        _ => {}
    }
    Ok(())
}

pub fn inflict(
    c: &mut Campaign,
    source: &str,
    target: &str,
    kind: &str,
    detail: &str,
    rounds: Option<u32>,
    cat: &Catalog,
) -> Result<()> {
    fighter(c, source)?;
    let before = fighter(c, target)?.runtime.statuses.clone();
    match set_status_from(c, target, kind, detail, rounds, cat, Some(source)) {
        Err(e)
            if [
                "Species type is immune to this condition",
                "Selected ability is immune to this condition",
                "Only one Flinch per subject per round",
                "Battlefield prevents this condition",
            ]
            .contains(&e.message.as_str()) =>
        {
            return Ok(())
        }
        other => other?,
    }
    if extra::enabled(c)
        && kind.contains("Poison")
        && fighter(c, source)?.selected_ability == "Poison Puppeteer"
        && fighter(c, target)?.runtime.statuses != before
    {
        let _ = set_status_from(c, target, "Confusion", "", None, cat, Some(source));
    }
    let t = fighter(c, target)?;
    if !enabled(c)
        || t.selected_ability != "Synchronize"
        || source == target
        || t.runtime.statuses == before
        || fighter(c, source)?.side == t.side
        || fighter(c, source)?.hp == 0
    {
        return Ok(());
    }
    let reflected_detail = if kind == "In Love" { target } else { detail };
    match set_status_from(c, source, kind, reflected_detail, rounds, cat, Some(target)) {
        Err(e)
            if [
                "Species type is immune to this condition",
                "Selected ability is immune to this condition",
                "Only one Flinch per subject per round",
                "Battlefield prevents this condition",
            ]
            .contains(&e.message.as_str()) =>
        {
            Ok(())
        }
        other => other,
    }
}

pub fn dice(c: &Campaign, event_id: &str, choice: &str) -> Result<u32> {
    let e = flow(c)?
        .ability_events
        .first()
        .filter(|e| e.id == event_id)
        .ok_or_else(|| error("Resolve the first pending ability trigger"))?;
    if choice == "manual" {
        return Ok(0);
    }
    if !e.choices.iter().any(|v| v == choice) {
        return Err(error("Choose one of the recorded ability options"));
    }
    if let Some(n) = extra::dice(c, e, choice) {
        return Ok(n);
    }
    let wonder = extra::enabled(c)
        && fighter(c, &e.target_id).is_ok_and(|p| p.selected_ability == "Wonder Skin");
    Ok(match e.kind.as_str() {
        "contact" => {
            if e.ability == "Effect Spore" {
                if wonder {
                    2
                } else {
                    4
                }
            } else {
                if wonder {
                    1
                } else {
                    3
                }
            }
        }
        "retaliate" if choice == "roll" => 2,
        _ => 0,
    })
}
pub fn resolve(
    c: &Campaign,
    next: &mut Campaign,
    actor: &str,
    command: &Action,
    results: &[u8],
    cat: &Catalog,
) -> Result<String> {
    let Action::ResolveAbility {
        event_id,
        choice,
        reason: r,
    } = command
    else {
        return Err(error("Expected an ability decision"));
    };
    reason(r)?;
    let choice = choice.as_str();
    let count = dice(c, event_id, choice)?;
    check_count(results, count)?;
    let e = flow(c)?.ability_events[0].clone();
    owner(c, actor, &e.combatant_id)?;
    if choice == "manual" {
        gm(c, actor)?;
    } else if e.kind.starts_with("extra-") {
        extra::resolve(next, actor, &e, choice, results, cat)?;
    } else if e.kind == "heal" && choice == "heal" {
        let allowed = !special::healing_blocked(c, &e.target_id);
        let p = fighter_mut(next, &e.target_id)?;
        if p.hp > 0 && allowed {
            p.hp = (p.hp + 1).min(p.max_hp);
        }
    } else if e.kind == "retaliate" {
        damage(
            next,
            &e.target_id,
            if choice == "neutral" {
                1
            } else {
                rules::successes(results)
            },
        )?;
    } else if e.kind == "contact"
        && results[..results.len() - usize::from(e.ability == "Effect Spore")].contains(&6)
        && fighter(c, &e.target_id)?.hp > 0
    {
        let kind = match e.ability.as_str() {
            "Static" => "Paralysis",
            "Poison Point" => "Poison",
            "Flame Body" => "Burn 2",
            "Cute Charm" => "In Love",
            "Effect Spore" => match *results.last().unwrap() {
                1 | 2 => "Poison",
                3 | 4 => "Paralysis",
                _ => "Sleep",
            },
            _ => return Err(error("Unsupported contact trigger")),
        };
        inflict(
            next,
            &e.combatant_id,
            &e.target_id,
            kind,
            if kind == "In Love" {
                &e.combatant_id
            } else {
                ""
            },
            None,
            cat,
        )?;
    }
    flow_mut(next).ability_events.remove(0);
    sync_labels(next);
    Ok(format!(
        "{}: {} chose {choice}; host dice {results:?}. {r}",
        e.ability, e.combatant_id
    ))
}

pub fn lifecycle(before: &Campaign, next: &mut Campaign) -> Result<()> {
    if !enabled(next) || next.battle.as_ref().is_none_or(|b| !b.active) {
        return Ok(());
    }
    let old: BTreeSet<_> = before
        .battle
        .as_ref()
        .filter(|b| b.active)
        .map(|b| {
            b.combatants
                .iter()
                .filter(|p| p.deployed && p.hp > 0)
                .map(|p| p.id.clone())
                .collect()
        })
        .unwrap_or_default();
    for p in &mut next.battle.as_mut().unwrap().combatants {
        if !p.deployed {
            p.runtime.ability.counters.clear();
        }
    }
    let active: BTreeSet<_> = battle(next)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    flow_mut(next).intimidate.retain(|source, targets| {
        targets.retain(|id| active.contains(id));
        active.contains(source)
    });
    let entrants: Vec<_> = battle(next)?
        .combatants
        .iter()
        .filter(|p| {
            active.contains(&p.id)
                && p.selected_ability == "Intimidate"
                && (!old.contains(&p.id)
                    || before
                        .battle
                        .as_ref()
                        .and_then(|b| b.flow.as_ref())
                        .and_then(|f| f.ranges.get(&p.id))
                        != flow(next).ok().and_then(|f| f.ranges.get(&p.id)))
        })
        .cloned()
        .collect();
    for p in entrants {
        let cap = [1, 2, 3, 4, 5, 6, 8, 10][crate::creation::rank(&next.sheets[&p.sheet_id].rank)
            .ok_or_else(|| error("Unknown ability range rank"))?];
        let mut targets: Vec<_> = battle(next)?
            .combatants
            .iter()
            .filter(|t| {
                active.contains(&t.id)
                    && t.side != p.side
                    && flow(next)
                        .unwrap()
                        .ranges
                        .get(&p.id)
                        .is_none_or(|ids| ids.contains(&t.id))
            })
            .map(|t| t.id.clone())
            .collect();
        targets.sort();
        targets.truncate(cap);
        flow_mut(next).intimidate.insert(p.id, targets);
    }
    Ok(())
}
pub fn end_round(c: &mut Campaign) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let b = c.battle.as_mut().unwrap();
    for p in &mut b.combatants {
        if p.deployed
            && p.hp > 0
            && p.available_round <= b.round
            && p.selected_ability == "Speed Boost"
        {
            let v = p
                .runtime
                .ability
                .counters
                .entry("Speed Boost".into())
                .or_default();
            *v = (*v + 1).min(3);
        }
    }
    Ok(())
}
