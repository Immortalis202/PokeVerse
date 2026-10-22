//! Pokerole 3.0 RULING pp.78,88-91 and reviewed move/ability entries.
//! Encounter state only: never write temporary effects back to a library sheet.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub enum Weather {
    Sunny,
    Rain,
    Sandstorm,
    Snow,
    Hail,
    Desolate,
    Typhoon,
    StrongWinds,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct FieldSnapshot {
    #[serde(default)]
    pub base_attribute_defenses: BTreeMap<String, [i32; 2]>,
    pub weather: Option<Weather>,
    #[serde(default)]
    pub defenses: BTreeMap<String, [i32; 2]>,
    pub types: BTreeMap<String, Vec<String>>,
}
fn frozen(c: &Campaign) -> Option<&FieldSnapshot> {
    c.battle
        .as_ref()?
        .flow
        .as_ref()?
        .queue
        .first()?
        .damage_context
        .as_ref()?
        .field_snapshot
        .as_ref()
}
pub fn snapshot(c: &Campaign, cat: &Catalog) -> Result<FieldSnapshot> {
    let mut defenses = BTreeMap::new();
    let mut base_attribute_defenses = BTreeMap::new();
    for p in battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && c.sheets[&p.sheet_id].kind == "pokemon")
    {
        let stats = crate::creation::profile_stats(&rules::effective_sheet(c, p)?);
        if provenance::enabled(c) {
            let base = crate::creation::profile_stats(&rules::base_sheet(c, p)?);
            base_attribute_defenses.insert(
                p.id.clone(),
                [
                    damage_defense(
                        c,
                        p,
                        true,
                        cat,
                        base["Defense"] + abilities::defense(c, p, true),
                        true,
                    )?
                    .max(1),
                    damage_defense(
                        c,
                        p,
                        false,
                        cat,
                        base["SpecialDefense"] + abilities::defense(c, p, false),
                        true,
                    )?
                    .max(1),
                ],
            );
        }
        defenses.insert(
            p.id.clone(),
            [
                defense(
                    c,
                    p,
                    true,
                    cat,
                    stats["Defense"] + abilities::defense(c, p, true),
                )?
                .max(1),
                defense(
                    c,
                    p,
                    false,
                    cat,
                    stats["SpecialDefense"] + abilities::defense(c, p, false),
                )?
                .max(1),
            ],
        );
    }
    Ok(FieldSnapshot {
        base_attribute_defenses,
        defenses,
        weather: weather(c, None),
        types: battle(c)?
            .combatants
            .iter()
            .filter(|p| p.deployed && c.sheets[&p.sheet_id].kind == "pokemon")
            .map(|p| Ok((p.id.clone(), types(c, p, cat)?)))
            .collect::<Result<_>>()?,
    })
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct FieldRule {
    pub id: String,
    pub kind: String,
    pub source: String,
    pub scope: String,
    pub rounds: Option<u32>,
    pub parameter: i32,
    pub started_round: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct WeatherSource {
    pub weather: Weather,
    pub source: String,
    pub rounds: Option<u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct FieldEvent {
    pub id: String,
    pub target: String,
    pub kind: String,
    pub dice: u32,
    pub difficulty: i32,
    pub amount: i32,
    pub damage_type: String,
    pub contenders: Vec<WeatherSource>,
    pub terrains: Vec<TerrainSource>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct TerrainSource {
    pub name: String,
    pub source: String,
    pub scope: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct Battlefield {
    pub enabled: bool,
    pub natural: Option<Weather>,
    pub weather: Option<WeatherSource>,
    pub rules: Vec<FieldRule>,
    pub events: Vec<FieldEvent>,
    pub sheltered: BTreeSet<String>,
    pub known_active: BTreeSet<String>,
    pub start_round: u32,
    pub end_round: u32,
    pub serial: u32,
    pub indoors: bool,
    pub sunlight: bool,
    #[serde(default)]
    pub night: bool,
    pub corrections: Vec<String>,
    #[serde(default)]
    pub species_types: BTreeMap<String, Vec<String>>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export, export_to = "../../src/bindings/")]
pub enum FieldCommand {
    Night {
        night: bool,
    },
    Environment {
        weather: Option<Weather>,
        indoors: bool,
        sunlight: bool,
    },
    Preset {
        name: String,
        scope: String,
        rounds: Option<u32>,
        parameter: i32,
    },
    Remove {
        id: String,
    },
    Adjudicate {
        event_id: String,
    },
    SelectWeather {
        event_id: String,
        weather: Weather,
    },
    Shelter {
        combatant_id: String,
        sheltered: bool,
    },
    Typing {
        combatant_id: String,
        mode: String,
        types: Vec<String>,
    },
    Resolve {
        event_id: String,
        accept: bool,
    },
    HealingPit {
        combatant_id: String,
        effect_id: String,
    },
}
pub fn enabled(c: &Campaign) -> bool {
    state(c).is_some_and(|f| f.enabled)
}
fn state(c: &Campaign) -> Option<&Battlefield> {
    c.battle.as_ref()?.flow.as_ref().map(|f| &f.battlefield)
}
fn state_mut(c: &mut Campaign) -> &mut Battlefield {
    &mut flow_mut(c).battlefield
}
pub fn pending(c: &Campaign) -> bool {
    enabled(c) && state(c).is_some_and(|f| !f.events.is_empty())
}
pub fn phase(c: &Campaign, a: &Action) -> Result<()> {
    if pending(c)
        && !matches!(
            a,
            Action::Battlefield { .. }
                | Action::RollbackBattle { .. }
                | Action::Override { .. }
                | Action::SetStatus { .. }
        )
    {
        return Err(error(
            "Resolve the pending battlefield check or choice first",
        ));
    }
    Ok(())
}
fn active(c: &Campaign, id: &str) -> bool {
    fighter(c, id).is_ok_and(|p| p.hp > 0)
}
pub fn weather(c: &Campaign, p: Option<&Combatant>) -> Option<Weather> {
    if !enabled(c) || p.is_some_and(|p| p.has_ability("Cloud Nine")) {
        return None;
    }
    if let Some(snapshot) = frozen(c) {
        return snapshot.weather;
    }
    if forms::zero_active(c) {
        return None;
    }
    if battle(c).is_ok_and(|b| {
        b.combatants
            .iter()
            .any(|p| p.deployed && p.hp > 0 && p.has_ability("Air Lock"))
    }) {
        return None;
    }
    let f = state(c)?;
    f.weather.as_ref().map(|w| w.weather).or(f.natural)
}
fn applies(scope: &str, p: &Combatant) -> bool {
    scope == "all"
        || scope == format!("side:{}", p.side)
        || scope == format!("combatant:{}", p.id)
        || scope
            .strip_prefix("opponents-of:")
            .is_some_and(|s| s != p.side)
}
pub fn has(c: &Campaign, name: &str, p: &Combatant) -> bool {
    if forms::zero_active(c) {
        return false;
    }
    enabled(c)
        && state(c)
            .unwrap()
            .rules
            .iter()
            .any(|r| r.kind == name && applies(&r.scope, p))
}
fn insert(
    c: &mut Campaign,
    kind: &str,
    source: &str,
    scope: &str,
    rounds: Option<u32>,
    parameter: i32,
) {
    let started_round = battle(c).unwrap().round;
    let f = state_mut(c);
    f.serial += 1;
    f.rules.push(FieldRule {
        id: format!("field:{}", f.serial),
        kind: kind.into(),
        source: source.into(),
        scope: scope.into(),
        rounds,
        parameter,
        started_round,
    });
}
fn enqueue(
    c: &mut Campaign,
    target: &str,
    kind: &str,
    dice: u32,
    difficulty: i32,
    amount: i32,
    damage_type: &str,
) {
    let f = state_mut(c);
    f.serial += 1;
    f.events.push(FieldEvent {
        id: format!("field-event:{}", f.serial),
        target: target.into(),
        kind: kind.into(),
        dice,
        difficulty,
        amount,
        damage_type: damage_type.into(),
        contenders: vec![],
        terrains: vec![],
    });
}
fn type_rule(c: &mut Campaign, id: &str, mode: &str, types: &[String]) -> Result<()> {
    fighter(c, id)?;
    if !["replace", "add", "remove", "restore"].contains(&mode)
        || types.len() > 2
        || (mode != "restore" && types.is_empty())
    {
        return Err(error(
            "Choose replace/add/remove/restore and one or two types",
        ));
    }
    for t in types {
        rules::type_effect("Normal", t)?;
    }
    if mode == "add" && types.len() != 1 {
        return Err(error("Only one additional type is supported"));
    }
    let scope = format!("combatant:{id}");
    if mode == "restore" {
        state_mut(c)
            .rules
            .retain(|r| r.scope != scope || !r.kind.starts_with("typing:"));
    } else {
        if mode == "replace" {
            state_mut(c)
                .rules
                .retain(|r| r.scope != scope || !r.kind.starts_with("typing:"));
        }
        if mode == "add" {
            state_mut(c)
                .rules
                .retain(|r| r.scope != scope || !r.kind.starts_with("typing:add:"));
        }
        insert(
            c,
            &format!("typing:{mode}:{}", types.join(",")),
            "typing",
            &scope,
            None,
            0,
        );
    }
    Ok(())
}
pub fn types(c: &Campaign, p: &Combatant, cat: &Catalog) -> Result<Vec<String>> {
    if let Some(ts) = frozen(c).and_then(|f| f.types.get(&p.id)) {
        return Ok(ts.clone());
    }
    live_types(c, p, cat)
}
pub fn live_types(c: &Campaign, p: &Combatant, cat: &Catalog) -> Result<Vec<String>> {
    let s = &c.sheets[&p.sheet_id];
    if s.kind != "pokemon" {
        return Err(AppError::new(
            "manual_rule",
            "Trainer typing requires GM adjudication",
        ));
    }
    let species = cat
        .find("species", "AppId", s.species_id.as_deref().unwrap_or(""))
        .ok_or_else(|| error("Missing species typing"))?;
    let mut ts: Vec<String> = ["Type1", "Type2"]
        .iter()
        .filter_map(|k| species[*k].as_str())
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
        .collect();
    if special::enabled(c) {
        for t in &mut ts {
            if t == "Fighting" {
                *t = "Fight".into();
            }
        }
    }
    if let Some(form_types) = forms::types(c, &p.id) {
        ts = form_types;
    }
    if !enabled(c) {
        return Ok(ts);
    }
    if extra::enabled(c) && p.has_ability("Mimicry") {
        let terrain = state(c)
            .unwrap()
            .rules
            .iter()
            .rev()
            .find(|r| r.kind.ends_with("Terrain") && applies(&r.scope, p));
        if let Some(t) = terrain {
            let ty = match t.kind.as_str() {
                "Electric Terrain" => "Electric",
                "Grassy Terrain" => "Grass",
                "Misty Terrain" => "Fairy",
                "Psychic Terrain" => "Psychic",
                _ => "",
            };
            if !ty.is_empty() {
                if ts.is_empty() {
                    ts.push(ty.into());
                } else {
                    ts[0] = ty.into();
                }
            }
        }
    }
    if p.has_ability("Forecast") {
        if let Some(t) = match weather(c, Some(p)) {
            Some(Weather::Sunny | Weather::Desolate) => Some("Fire"),
            Some(Weather::Rain | Weather::Typhoon) => Some("Water"),
            Some(Weather::Snow | Weather::Hail) => Some("Ice"),
            Some(Weather::Sandstorm) => Some("Rock"),
            _ => None,
        } {
            ts = vec![t.into()];
        }
    }
    for r in &state(c).unwrap().rules {
        if !applies(&r.scope, p) {
            continue;
        }
        if r.kind == "sprinklers" {
            ts = vec!["Water".into()];
        }
        if let Some(rest) = r.kind.strip_prefix("typing:") {
            let (mode, values) = rest.split_once(':').unwrap_or(("", ""));
            let values: Vec<String> = values.split(',').map(str::to_owned).collect();
            match mode {
                "replace" => ts = values,
                "add" => {
                    if ts.len() > 2 {
                        ts.pop();
                    }
                    for t in values {
                        if !ts.contains(&t) {
                            ts.push(t);
                        }
                    }
                }
                "remove" => ts.retain(|t| !values.contains(t)),
                _ => {}
            }
        }
    }
    ts.dedup();
    Ok(ts)
}
pub fn grounded(c: &Campaign, p: &Combatant, cat: &Catalog) -> bool {
    has(c, "Gravity", p)
        || items::iron_ball(c, &p.id)
        || (!types(c, p, cat)
            .unwrap_or_default()
            .iter()
            .any(|t| t == "Flying")
            && !p.has_ability("Levitate"))
}
pub fn stats(c: &Campaign, p: &Combatant, s: &mut Sheet) {
    if !enabled(c) {
        return;
    }
    if !extra::enabled(c) {
        let w = weather(c, Some(p));
        let flower = matches!(w, Some(Weather::Sunny | Weather::Desolate))
            && aura(c, p, "Flower Gift", true);
        if flower {
            let own = match p.ability_for(&["Guts", "Moxie", "Sap Sipper"]) {
                "Guts"
                    if p.runtime.statuses.iter().any(|s| {
                        s.kind.starts_with("Burn")
                            || s.kind.contains("Poison")
                            || ["Sleep", "Paralysis", "Frozen Solid"].contains(&s.kind.as_str())
                    }) =>
                {
                    2
                }
                "Moxie" | "Sap Sipper" => *p
                    .runtime
                    .ability
                    .counters
                    .get(&p.selected_ability)
                    .unwrap_or(&0),
                _ => 0,
            };
            let v = s.stats.entry("Strength".into()).or_default();
            *v = (*v + (2 - own).max(0)).clamp(1, forms::stat_cap(c, p));
        }
        let bonus = match p.ability_for(&[
            "Chlorophyll",
            "Swift Swim",
            "Sand Rush",
            "Slush Rush",
            "Solar Power",
            "Surge Surfer",
        ]) {
            "Chlorophyll" if matches!(w, Some(Weather::Sunny | Weather::Desolate)) => {
                Some(("Dexterity", 2))
            }
            "Swift Swim" if matches!(w, Some(Weather::Rain | Weather::Typhoon)) => {
                Some(("Dexterity", 2))
            }
            "Sand Rush" if w == Some(Weather::Sandstorm) => Some(("Dexterity", 1)),
            "Slush Rush" if matches!(w, Some(Weather::Snow | Weather::Hail)) => {
                Some(("Dexterity", 1))
            }
            "Solar Power" if matches!(w, Some(Weather::Sunny | Weather::Desolate)) => {
                Some(("Special", 2))
            }
            "Surge Surfer" if has(c, "Electric Terrain", p) => Some(("Dexterity", 2)),
            _ => None,
        };
        if let Some((key, n)) = bonus {
            let v = s.stats.entry(key.into()).or_default();
            *v = (*v + n).clamp(1, forms::stat_cap(c, p));
        }
    }
    if has(c, "mud", p)
        && (items::iron_ball(c, &p.id)
            || (!p.has_ability("Levitate")
                && !state(c)
                    .unwrap()
                    .species_types
                    .get(&p.id)
                    .is_some_and(|ts| ts.iter().any(|t| t == "Flying"))))
    {
        let v = s.stats.entry("Dexterity".into()).or_default();
        *v = (*v - 1).max(1);
    }
    for (preset, skills) in [
        ("jungle", vec!["Stealth", "Nature"]),
        ("cemetery", vec!["Intimidate"]),
    ] {
        if has(c, preset, p) {
            for key in skills {
                *s.skills.entry(key.into()).or_default() += 2;
            }
        }
    }
}
pub fn defense(
    c: &Campaign,
    p: &Combatant,
    physical: bool,
    cat: &Catalog,
    base: i32,
) -> Result<i32> {
    damage_defense(c, p, physical, cat, base, false)
}
pub fn damage_defense(
    c: &Campaign,
    p: &Combatant,
    physical: bool,
    cat: &Catalog,
    base: i32,
    ignore_attributes: bool,
) -> Result<i32> {
    if !enabled(c) {
        return Ok(base);
    }
    let ts = types(c, p, cat).unwrap_or_default();
    defense_for_types(c, p, physical, &ts, base, ignore_attributes)
}
pub fn order_defense(c: &Campaign, p: &Combatant, physical: bool, base: i32) -> Result<i32> {
    if !enabled(c) {
        return Ok(base);
    }
    let ts = state(c)
        .unwrap()
        .species_types
        .get(&p.id)
        .cloned()
        .unwrap_or_default();
    defense_for_types(c, p, physical, &ts, base, false)
}
fn defense_for_types(
    c: &Campaign,
    p: &Combatant,
    physical: bool,
    ts: &[String],
    base: i32,
    ignore_attributes: bool,
) -> Result<i32> {
    let w = weather(c, Some(p));
    let mut n = base;
    if !extra::enabled(c)
        && !physical
        && matches!(w, Some(Weather::Sunny | Weather::Desolate))
        && aura(c, p, "Flower Gift", true)
    {
        n += 2;
    }
    if has(c, "Wonder Room", p) {
        let s = if ignore_attributes {
            rules::base_sheet(c, p)?
        } else {
            rules::effective_sheet(c, p)?
        };
        let split = p.rules_profile.defenses == "split";
        // profile_stats retains reviewed rank bonuses; replace only its attribute contribution.
        let original = if physical || !split {
            "Vitality"
        } else {
            "Insight"
        };
        let replacement = if split && !physical {
            "Vitality"
        } else {
            "Insight"
        };
        n += rules::score(&s, replacement)? - rules::score(&s, original)?;
    }
    n += i32::from(!physical && w == Some(Weather::Sandstorm) && ts.iter().any(|t| t == "Rock"));
    n += i32::from(physical && w == Some(Weather::Snow) && ts.iter().any(|t| t == "Ice"));
    if !extra::enabled(c) && physical && p.has_ability("Grass Pelt") && has(c, "Grassy Terrain", p)
    {
        n += 2;
    }
    Ok(n)
}
pub fn low_accuracy(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> i32 {
    if !enabled(c) || extra::no_guard(c, p) {
        return 0;
    }
    i32::from(
        has(c, "fog", p)
            && !(extra::enabled(c) && aura(c, p, "Illuminate", true))
            && ["Foe", "All Foes", "Random Foe", "Area"]
                .contains(&m["Target"].as_str().unwrap_or("")),
    ) + if has(c, "Gravity", p)
        && [
            "Fly",
            "Bounce",
            "Sky Drop",
            "Jump Kick",
            "High Jump Kick",
            "Splash",
            "Flying Press",
        ]
        .contains(&m["Name"].as_str().unwrap_or(""))
    {
        2
    } else {
        0
    }
}
pub fn guard_move(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> Result<()> {
    extra::reaction(c, p, "move", Some(m))?;
    if !enabled(c) {
        return Ok(());
    }
    let t = m["Type"].as_str().unwrap_or("");
    if matches!(
        (weather(c, Some(p)), t),
        (Some(Weather::Desolate), "Water") | (Some(Weather::Typhoon), "Fire")
    ) {
        return Err(error("The active weather prevents this move"));
    }
    let n = m["Name"].as_str().unwrap_or("");
    if n == "Sunny Day" && (!state(c).unwrap().sunlight || state(c).unwrap().indoors) {
        return Err(error("Sunny weather needs reachable sunlight"));
    }
    if ["Rain Dance", "Sandstorm", "Hail"].contains(&n) && state(c).unwrap().indoors {
        return Err(error("This weather cannot reach an indoor battlefield"));
    }
    if n == "Aurora Veil" && !matches!(weather(c, None), Some(Weather::Snow | Weather::Hail)) {
        return Err(error("Aurora Veil requires Hail or Snow"));
    }
    if has(c, "torn-world", p) {
        return Err(AppError::new("manual_rule","Torn World changes targeting: GM must record the random target using an explicit ruling"));
    }
    Ok(())
}
pub fn reaction(c: &Campaign, p: &Combatant, kind: &str, cat: &Catalog) -> Result<()> {
    if has(c, "final-destination", p) && ["evade", "clash"].contains(&kind) {
        return Err(error("This arena prevents Evasion and Clash"));
    }
    if kind == "evade" && has(c, "flowers", p) {
        return Err(error("Lovely Flowers prevents Evasion"));
    }
    if !["evade", "clash"].contains(&kind) && has(c, "Psychic Terrain", p) && grounded(c, p, cat) {
        return Err(error("Psychic Terrain prevents grounded reaction moves"));
    }
    Ok(())
}
pub fn bonus(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> i32 {
    let t = m["Type"].as_str().unwrap_or("");
    let mut n = match (weather(c, Some(p)), t) {
        (Some(Weather::Sunny), "Fire")
        | (Some(Weather::Rain), "Water")
        | (Some(Weather::Snow), "Ice") => 1,
        (Some(Weather::Desolate), "Fire")
        | (Some(Weather::Typhoon), "Water")
        | (Some(Weather::StrongWinds), "Flying") => 2,
        _ => 0,
    };
    for (kind, tpe) in [
        ("Electric Terrain", "Electric"),
        ("Grassy Terrain", "Grass"),
        ("Psychic Terrain", "Psychic"),
        ("electric-poles", "Electric"),
    ] {
        if t == tpe && has(c, kind, p) {
            n += 1;
        }
    }
    if !extra::enabled(c)
        && p.has_ability("Sand Force")
        && weather(c, Some(p)) == Some(Weather::Sandstorm)
        && ["Ground", "Steel", "Rock"].contains(&t)
    {
        n += 1;
    }
    if enabled(c) && m["Name"] == "Weather Ball" && weather(c, Some(p)).is_some() {
        n += 2;
    }
    n
}
pub fn reduction(
    c: &Campaign,
    source: &Combatant,
    target: &Combatant,
    m: &serde_json::Value,
) -> i32 {
    if !enabled(c) {
        return 0;
    }
    let t = m["Type"].as_str().unwrap_or("");
    let physical = m["Category"] == "Physical";
    let mut n = i32::from(matches!(
        (weather(c, Some(source)), t),
        (Some(Weather::Sunny), "Water") | (Some(Weather::Rain), "Fire")
    ));
    if has(c, "flowers", source) {
        n += 2;
    }
    if special::vulnerable(c, &target.id, m["Name"].as_str().unwrap_or("")) {
        n -= 2;
    }
    if !interactions::cover_bypass(c, &source.id, m)
        && !source.has_ability("Infiltrator")
        && !(special::enabled(c)
            && (m["Attributes"]["SoundMove"] == true
                || [
                    "Phantom Force",
                    "Shadow Force",
                    "Hyperspace Hole",
                    "Hyperspace Fury",
                    "Doom Desire",
                    "Future Sight",
                ]
                .contains(&m["Name"].as_str().unwrap_or(""))
                || special::vulnerable(c, &target.id, m["Name"].as_str().unwrap_or(""))))
        && !["Brick Break", "Psychic Fangs"].contains(&m["Name"].as_str().unwrap_or(""))
    {
        n += state(c)
            .unwrap()
            .rules
            .iter()
            .filter(|r| {
                applies(&r.scope, target)
                    && (r.kind == "Aurora Veil"
                        || (physical && r.kind == "Reflect")
                        || (!physical && r.kind == "Light Screen"))
            })
            .map(|r| r.parameter)
            .max()
            .unwrap_or(0);
    }
    n
}
pub fn prevents_status(c: &Campaign, p: &Combatant, kind: &str) -> bool {
    has(c, "Misty Terrain", p)
        || match weather(c, Some(p)) {
            Some(Weather::Sunny | Weather::Desolate) => kind == "Frozen Solid",
            Some(Weather::Rain) => ["Burn 2", "Burn 3"].contains(&kind),
            Some(Weather::Typhoon) => kind.starts_with("Burn"),
            _ => false,
        }
}
fn heal(c: &mut Campaign, id: &str, n: i32, limited: bool) -> Result<()> {
    if special::healing_blocked(c, id) {
        return Ok(());
    }
    let p = fighter_mut(c, id)?;
    if p.hp == 0 {
        return Ok(());
    }
    let n = n
        .min(p.max_hp - p.hp)
        .min(if limited {
            ((if n >= 5 { 5 } else { 3 }) - p.runtime.healed).max(0)
        } else {
            n
        })
        .max(0);
    p.hp += n;
    if limited {
        p.runtime.healed += n;
    }
    Ok(())
}
fn hurt(c: &mut Campaign, id: &str, n: i32, t: &str, cat: &Catalog) -> Result<()> {
    let p = fighter(c, id)?;
    if p.hp == 0
        || p.has_ability("Magic Guard")
        || (extra::enabled(c) && p.has_ability("Wonder Guard"))
    {
        return Ok(());
    }
    let mut amount = n;
    if t != "Neutral" {
        for ty in types(c, p, cat)? {
            let e = rules::type_effect(t, &ty)?;
            if e == -100 {
                return Ok(());
            }
            amount += e;
        }
    }
    let p = fighter_mut(c, id)?;
    p.hp = (p.hp - amount.max(0)).max(0);
    Ok(())
}
fn set_weather(c: &mut Campaign, w: WeatherSource) -> Result<()> {
    if battle(c)?
        .combatants
        .iter()
        .any(|p| p.deployed && p.hp > 0 && p.has_ability("Air Lock"))
    {
        return Err(error("Air Lock prevents weather activation"));
    }
    if state(c).unwrap().indoors
        && [
            Weather::Sunny,
            Weather::Rain,
            Weather::Sandstorm,
            Weather::Hail,
        ]
        .contains(&w.weather)
    {
        return Err(error("This weather cannot reach the indoor battlefield"));
    }
    if w.weather == Weather::Sunny && !state(c).unwrap().sunlight {
        return Err(error("Sunny weather requires reachable sunlight"));
    }
    if let Some(old) = state(c)
        .unwrap()
        .weather
        .as_ref()
        .map(|w| w.weather)
        .or(state(c).unwrap().natural)
    {
        if old == Weather::StrongWinds && old != w.weather {
            return Err(error("Strong Winds cannot be overridden"));
        }
        if [Weather::Desolate, Weather::Typhoon].contains(&old)
            && ![Weather::Desolate, Weather::Typhoon, Weather::StrongWinds].contains(&w.weather)
        {
            return Err(error(
                "Extreme weather prevents ordinary weather activation",
            ));
        }
    }
    state_mut(c).weather = Some(w);
    Ok(())
}
pub fn move_effect(c: &mut Campaign, q: &QueuedAction, cat: &Catalog) -> Result<bool> {
    if !enabled(c) {
        return Ok(false);
    }
    let name = q.move_name.as_str();
    let w = match name {
        "Sunny Day" => Some(Weather::Sunny),
        "Rain Dance" => Some(Weather::Rain),
        "Sandstorm" => Some(Weather::Sandstorm),
        "Hail" => Some(Weather::Hail),
        "Snowscape" => Some(Weather::Snow),
        _ => None,
    };
    if let Some(weather) = w {
        set_weather(
            c,
            WeatherSource {
                weather,
                source: format!("move:{}", q.sequence),
                rounds: Some(4),
            },
        )?;
        return Ok(true);
    }
    let p = fighter(c, &q.actor_id)?.clone();
    if [
        "Electric Terrain",
        "Grassy Terrain",
        "Misty Terrain",
        "Psychic Terrain",
        "Reflect",
        "Light Screen",
        "Aurora Veil",
        "Trick Room",
        "Wonder Room",
        "Magic Room",
        "Gravity",
    ]
    .contains(&name)
    {
        guard_move(c, &p, &move_entry(cat, name)?)?;
        let scope = q.field_scope.clone().unwrap_or_else(|| "all".into());
        if name.ends_with("Terrain")
            && state(c).unwrap().rules.iter().any(|r| {
                r.kind.ends_with("Terrain")
                    && r.kind != name
                    && (r.scope == scope || r.scope == "all" || scope == "all")
            })
        {
            return Err(error("Overlapping different terrains need a GM ruling: remove the conflicting terrain before resolution"));
        }
        if name == "Grassy Terrain" {
            let p = fighter_mut(c, &tactics::payer(c, &q.actor_id))?;
            if p.runtime.will < 1 {
                return Err(error("Grassy Terrain requires one Will"));
            }
            p.runtime.will -= 1;
        }
        state_mut(c)
            .rules
            .retain(|r| r.kind != name || r.scope != scope);
        insert(c, name, &format!("move:{}", q.sequence), &scope, Some(4), 1);
        cure_terrain(c, cat)?;
        if name == "Trick Room" {
            reverse_remaining(c);
        }
        return Ok(true);
    }
    match name {
        "Soak" => type_rule(c, &q.target_id, "replace", &["Water".into()])?,
        "Forest's Curse" => type_rule(c, &q.target_id, "add", &["Grass".into()])?,
        "Trick-or-Treat" | "Trick-Or-Treat" => {
            type_rule(c, &q.target_id, "add", &["Ghost".into()])?
        }
        "Burn Up" => type_rule(c, &q.actor_id, "remove", &["Fire".into()])?,
        "Defog" => {
            state_mut(c).rules.retain(|r| {
                !(r.kind.ends_with("Terrain")
                    || [
                        "Reflect",
                        "Light Screen",
                        "Aurora Veil",
                        "minefield",
                        "Toxic Spikes",
                    ]
                    .contains(&r.kind.as_str()))
            });
        }
        "Double Shock" => type_rule(c, &q.actor_id, "remove", &["Electric".into()])?,
        "Synthesis" | "Morning Sun" | "Moonlight" => {
            let amount = match weather(c, Some(&p)) {
                Some(Weather::Sunny | Weather::Desolate) => 5,
                Some(Weather::Rain | Weather::Sandstorm | Weather::Hail) => 1,
                _ => 3,
            };
            let p = fighter_mut(c, &tactics::payer(c, &q.actor_id))?;
            if p.runtime.will < 1 {
                return Err(error("Healing requires one Will"));
            }
            p.runtime.will -= 1;
            heal(c, &q.actor_id, amount, true)?;
        }
        "Brick Break" | "Psychic Fangs" => {
            let side = fighter(c, &q.target_id)?.side.clone();
            state_mut(c).rules.retain(|r| {
                !(applies(
                    &r.scope,
                    &Combatant {
                        side: side.clone(),
                        ..p.clone()
                    },
                ) && ["Reflect", "Light Screen", "Aurora Veil"].contains(&r.kind.as_str()))
            });
        }
        _ => return Ok(false),
    }
    Ok(true)
}
fn reverse_remaining(c: &mut Campaign) {
    let modern = tactics::enabled(c);
    let stalled = if modern {
        battle(c)
            .unwrap()
            .combatants
            .iter()
            .filter(|p| p.has_ability("Stall"))
            .map(|p| p.id.clone())
            .collect::<BTreeSet<_>>()
    } else {
        BTreeSet::new()
    };
    let f = flow_mut(c);
    if f.started && !f.order.is_empty() {
        let start = (f.cursor + 1).min(f.order.len());
        f.order[start..].reverse();
        if modern {
            f.order[start..].sort_by_key(|id| stalled.contains(id));
        }
    }
}
fn cure_terrain(c: &mut Campaign, cat: &Catalog) -> Result<()> {
    let ps = battle(c)?.combatants.clone();
    for p in ps.iter().filter(|p| p.deployed && p.hp > 0) {
        if has(c, "Misty Terrain", p) {
            fighter_mut(c, &p.id)?.runtime.statuses.clear();
        } else if has(c, "Electric Terrain", p) && grounded(c, p, cat) {
            fighter_mut(c, &p.id)?
                .runtime
                .statuses
                .retain(|s| s.kind != "Sleep");
        }
    }
    Ok(())
}
pub fn start(c: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !enabled(c) || state(c).unwrap().start_round == battle(c)?.round {
        return Ok(());
    }
    state_mut(c).start_round = battle(c)?.round;
    for p in battle(c)?
        .combatants
        .clone()
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
    {
        let ts = types(c, p, cat).unwrap_or_default();
        let w = weather(c, Some(p));
        let s = rules::effective_sheet(c, p)?;
        let check = match w {
            Some(Weather::Desolate) if !ts.contains(&"Fire".into()) => Some(("Vitality", "Fire")),
            Some(Weather::Typhoon) if !ts.contains(&"Water".into()) => Some(("Vitality", "Water")),
            Some(Weather::StrongWinds)
                if items::iron_ball(c, &p.id)
                    || (!ts.contains(&"Flying".into()) && !p.has_ability("Levitate")) =>
            {
                Some(("Dexterity", "Neutral"))
            }
            _ => None,
        };
        if let Some((stat, t)) = check {
            let n = rules::score(&s, stat)?;
            if n <= 4 {
                enqueue(c, &p.id, "check", n.max(0) as u32, 2, 2, t);
            }
        }
    }
    Ok(())
}
pub fn end(c: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !enabled(c) || state(c).unwrap().end_round == battle(c)?.round {
        return Ok(());
    }
    state_mut(c).end_round = battle(c)?.round;
    for p in battle(c)?
        .combatants
        .clone()
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
    {
        let w = weather(c, Some(p));
        let ts = types(c, p, cat).unwrap_or_default();
        let sand = ts
            .iter()
            .any(|t| ["Rock", "Ground", "Steel"].contains(&t.as_str()))
            || p.has_any_ability(&["Sand Rush", "Sand Veil", "Sand Force"])
            || (super::extra::enabled(c) && p.has_ability("Overcoat"));
        let hail = ts.contains(&"Ice".into())
            || p.has_any_ability(&["Ice Body", "Snow Cloak", "Slush Rush"])
            || (super::extra::enabled(c) && p.has_ability("Overcoat"));
        if !state(c).unwrap().sheltered.contains(&p.id)
            && ((w == Some(Weather::Sandstorm) && !sand) || (w == Some(Weather::Hail) && !hail))
        {
            hurt(c, &p.id, 1, "Neutral", cat)?;
        }
        if matches!(w, Some(Weather::Sunny | Weather::Desolate))
            && p.has_any_ability(&["Solar Power", "Dry Skin"])
        {
            hurt(c, &p.id, 1, "Neutral", cat)?;
        }
        if has(c, "Grassy Terrain", p) {
            heal(c, &p.id, 1, true)?;
        }
        if (p.has_ability("Rain Dish") && matches!(w, Some(Weather::Rain | Weather::Typhoon)))
            || (p.has_ability("Ice Body") && matches!(w, Some(Weather::Snow | Weather::Hail)))
        {
            enqueue(c, &p.id, "heal", 0, 0, 1, "Neutral");
        }
        if p.has_ability("Hydration") && matches!(w, Some(Weather::Rain | Weather::Typhoon)) {
            fighter_mut(c, &p.id)?.runtime.statuses.clear();
        }
        if has(c, "underwater", p) && !ts.contains(&"Water".into()) {
            enqueue(
                c,
                &p.id,
                "drown",
                rules::score(&rules::effective_sheet(c, p)?, "Vitality")?.max(0) as u32,
                ((battle(c)?.round
                    - state(c)
                        .unwrap()
                        .rules
                        .iter()
                        .find(|r| r.kind == "underwater" && applies(&r.scope, p))
                        .unwrap()
                        .started_round
                    + 1)
                    * 2) as i32,
                0,
                "Neutral",
            );
        }
        for (preset, kind) in [("fire", "Burn 2"), ("sewage", "Poison")] {
            if has(c, preset, p) {
                enqueue(c, &p.id, kind, 3, 0, 0, "Neutral");
            }
        }
        if has(c, "high-poles", p) && !ts.contains(&"Flying".into()) {
            let n = rules::score(&rules::effective_sheet(c, p)?, "Dexterity")?
                + rules::score(&rules::effective_sheet(c, p)?, "Athletic")?;
            let difficulty = state(c)
                .unwrap()
                .rules
                .iter()
                .find(|r| r.kind == "high-poles" && applies(&r.scope, p))
                .unwrap()
                .parameter;
            enqueue(c, &p.id, "check", n.max(0) as u32, difficulty, 2, "Neutral");
        }
    }
    Ok(())
}
pub fn expire(c: &mut Campaign) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let f = state_mut(c);
    if let Some(w) = f.weather.as_mut() {
        if let Some(n) = w.rounds.as_mut() {
            *n = n.saturating_sub(1);
        }
    }
    if f.weather.as_ref().is_some_and(|w| w.rounds == Some(0)) {
        f.weather = None;
    }
    for r in &mut f.rules {
        if let Some(n) = r.rounds.as_mut() {
            *n = n.saturating_sub(1);
        }
    }
    f.rules.retain(|r| r.rounds != Some(0));
    Ok(())
}
pub fn lifecycle(before: &Campaign, c: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !enabled(c) || !c.battle.as_ref().is_some_and(|b| b.active) {
        return Ok(());
    }
    let entries = battle(c)?.combatants.clone();
    for p in &entries {
        let ts = live_types(c, p, cat).unwrap_or_default();
        state_mut(c).species_types.insert(p.id.clone(), ts);
    }
    let active_ids: BTreeSet<String> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    let old = state(c).unwrap().known_active.clone();
    if state(c).unwrap().weather.as_ref().is_some_and(|w| {
        w.source.starts_with("ability:")
            && w.rounds.is_none()
            && !active_ids.contains(w.source.trim_start_matches("ability:"))
    }) {
        state_mut(c).weather = None;
    }
    let mut candidates = vec![];
    for p in battle(c)?
        .combatants
        .clone()
        .iter()
        .filter(|p| active_ids.contains(&p.id) && !old.contains(&p.id))
    {
        for name in p.ability_names() {
            let mut variant = p.clone();
            variant.selected_ability = name;
            let p = &variant;
            if reviewed::enabled(c)
                && !before.battle.as_ref().is_some_and(|b| {
                    b.combatants
                        .iter()
                        .any(|q| q.id == p.id && q.deployed && q.hp > 0)
                })
                && has(c, "Toxic Spikes", p)
                && grounded(c, p, cat)
                && !abilities::damage_immune(
                    c,
                    p,
                    &serde_json::json!({"Type":"Ground","Category":"Physical"}),
                )
            {
                enqueue(c, &p.id, "toxic-spikes", 1, 0, 0, "Poison");
            }
            if has(c, "minefield", p)
                && (!ability_changes::enabled(c)
                    || !before.battle.as_ref().is_some_and(|b| {
                        b.combatants
                            .iter()
                            .any(|q| q.id == p.id && q.deployed && q.hp > 0)
                    }))
            {
                hurt(c, &p.id, 1, "Neutral", cat)?;
            }
            let weather = match p.ability_for(&[
                "Drought",
                "Orichalcum Pulse",
                "Drizzle",
                "Sand Stream",
                "Desolate Land",
                "Primordial Sea",
                "Delta Stream",
            ]) {
                "Drought" => Some(Weather::Sunny),
                "Orichalcum Pulse" if extra::enabled(c) => Some(Weather::Sunny),
                "Drizzle" => Some(Weather::Rain),
                "Sand Stream" => Some(Weather::Sandstorm),
                "Desolate Land" => Some(Weather::Desolate),
                "Primordial Sea" => Some(Weather::Typhoon),
                "Delta Stream" => Some(Weather::StrongWinds),
                _ => None,
            };
            if let Some(weather) = weather {
                candidates.push(WeatherSource {
                    weather,
                    source: format!("ability:{}", p.id),
                    rounds: None,
                });
            }
            if p.has_ability("Snow Warning") {
                enqueue(c, &p.id, "weather-choice", 0, 0, 0, "Neutral");
            }
            let terrain = match p.ability_for(&[
                "Grassy Surge",
                "Electric Surge",
                "Hadron Engine",
                "Misty Surge",
                "Psychic Surge",
            ]) {
                "Grassy Surge" => Some("Grassy Terrain"),
                "Electric Surge" => Some("Electric Terrain"),
                "Hadron Engine" if extra::enabled(c) => Some("Electric Terrain"),
                "Misty Surge" => Some("Misty Terrain"),
                "Psychic Surge" => Some("Psychic Terrain"),
                _ => None,
            };
            if let Some(name) = terrain {
                let scope = if name == "Grassy Terrain" {
                    format!("side:{}", p.side)
                } else {
                    "all".into()
                };
                let mut contenders: Vec<TerrainSource> = state(c)
                    .unwrap()
                    .rules
                    .iter()
                    .filter(|r| {
                        r.kind.ends_with("Terrain")
                            && r.kind != name
                            && r.source.starts_with("ability:")
                            && active_ids.contains(r.source.trim_start_matches("ability:"))
                    })
                    .map(|r| TerrainSource {
                        name: r.kind.clone(),
                        source: r.source.clone(),
                        scope: r.scope.clone(),
                    })
                    .collect();
                contenders.push(TerrainSource {
                    name: name.into(),
                    source: format!("ability:{}", p.id),
                    scope,
                });
                if contenders.len() == 1
                    && state(c).unwrap().rules.iter().any(|r| {
                        r.kind.ends_with("Terrain")
                            && r.kind != name
                            && !r.source.starts_with("ability:")
                    })
                {
                    enqueue(c, &p.id, "terrain-ruling", 0, 0, 0, "Neutral");
                    state_mut(c).events.last_mut().unwrap().terrains = contenders;
                } else if contenders.len() > 1 {
                    let n = contenders.iter().map(|t| will(c, &t.source)).sum();
                    enqueue(c, "", "terrain-contest", n, 0, 0, "Neutral");
                    state_mut(c).events.last_mut().unwrap().terrains = contenders;
                } else {
                    apply_terrain(c, &contenders[0]);
                }
            }
        }
    }
    if !candidates.is_empty() {
        if let Some(w) = state(c).unwrap().weather.clone().filter(|w| {
            w.source.starts_with("ability:")
                && active_ids.contains(w.source.trim_start_matches("ability:"))
        }) {
            candidates.insert(0, w);
        }
        if candidates.len() > 1
            && candidates
                .iter()
                .any(|w| w.weather != candidates[0].weather)
        {
            let n = candidates
                .iter()
                .map(|w| {
                    fighter(c, w.source.trim_start_matches("ability:")).map(|p| {
                        c.sheets[&p.sheet_id]
                            .workflow
                            .as_ref()
                            .map_or(p.runtime.will, |b| b.max_will)
                            .max(0) as u32
                    })
                })
                .collect::<Result<Vec<_>>>()?
                .iter()
                .sum();
            enqueue(c, "", "weather-contest", n, 0, 0, "Neutral");
            state_mut(c).events.last_mut().unwrap().contenders = candidates;
        } else if let Some(w) = candidates.pop() {
            if let Err(e) = set_weather(c, w) {
                state_mut(c).corrections.push(e.message);
            }
        }
    }
    state_mut(c).known_active = active_ids;
    if before.battle.as_ref().is_some_and(|b| !b.active) {
        state_mut(c).start_round = 0;
    }
    cure_terrain(c, cat)?;
    Ok(())
}
pub fn dice(c: &Campaign, command: &FieldCommand) -> Result<u32> {
    if let FieldCommand::Resolve { event_id, accept } = command {
        let e = state(c)
            .and_then(|f| f.events.first())
            .filter(|e| e.id == *event_id)
            .ok_or_else(|| error("Pending battlefield event changed"))?;
        if !accept && e.kind != "heal" && e.kind != "terrain-ruling" {
            return Err(error("Required battlefield checks cannot be skipped"));
        }
        return Ok(if *accept { e.dice } else { 0 });
    }
    Ok(0)
}
pub fn command(
    c: &mut Campaign,
    actor: &str,
    cmd: &FieldCommand,
    dice: &[u8],
    r: &str,
    cat: &Catalog,
) -> Result<String> {
    if !enabled(c) {
        return Err(error("Upgrade battlefield rules between battles"));
    }
    reason(r)?;
    check_count(dice, self::dice(c, cmd)?)?;
    if pending(c)
        && !matches!(
            cmd,
            FieldCommand::Resolve { .. }
                | FieldCommand::SelectWeather { .. }
                | FieldCommand::Adjudicate { .. }
        )
    {
        return Err(error(
            "Resolve the pending battlefield event before changing conditions",
        ));
    }
    if !matches!(
        cmd,
        FieldCommand::Resolve { .. }
            | FieldCommand::SelectWeather { .. }
            | FieldCommand::HealingPit { .. }
    ) {
        gm(c, actor)?;
        if !flow(c)?.queue.is_empty() {
            return Err(error(
                "Finish the pending attack before changing battlefield rules",
            ));
        }
    }
    match cmd {
        FieldCommand::Night { night } => {
            if !reviewed::enabled(c) {
                return Err(error(
                    "Night-dependent abilities require the current engine",
                ));
            }
            state_mut(c).night = *night;
        }
        FieldCommand::Adjudicate { event_id } => {
            gm(c, actor)?;
            if state(c)
                .unwrap()
                .events
                .first()
                .is_none_or(|e| e.id != *event_id)
            {
                return Err(error("Pending battlefield event changed"));
            }
            state_mut(c).events.remove(0);
            state_mut(c)
                .corrections
                .push(format!("GM adjudicated {event_id}: {r}"));
        }
        FieldCommand::SelectWeather { event_id, weather } => {
            let e = state(c)
                .unwrap()
                .events
                .first()
                .filter(|e| e.id == *event_id && e.kind == "weather-choice")
                .ok_or_else(|| error("Pending weather choice changed"))?
                .clone();
            owner(c, actor, &e.target)?;
            if ![Weather::Snow, Weather::Hail].contains(weather) {
                return Err(error("Snow Warning chooses Snow or Hail"));
            }
            state_mut(c).events.remove(0);
            let w = WeatherSource {
                weather: *weather,
                source: format!("ability:{}", e.target),
                rounds: None,
            };
            if let Some(old) = state(c)
                .unwrap()
                .weather
                .clone()
                .filter(|old| old.source.starts_with("ability:") && old.weather != *weather)
            {
                let contenders = vec![old, w];
                let n = contenders.iter().map(|v| will(c, &v.source)).sum();
                enqueue(c, "", "weather-contest", n, 0, 0, "Neutral");
                state_mut(c).events.last_mut().unwrap().contenders = contenders;
            } else if let Err(e) = set_weather(c, w) {
                state_mut(c).corrections.push(e.message);
            }
        }
        FieldCommand::Environment {
            weather,
            indoors,
            sunlight,
        } => {
            let f = state_mut(c);
            f.natural = *weather;
            f.indoors = *indoors;
            f.sunlight = *sunlight;
        }
        FieldCommand::Preset {
            name,
            scope,
            rounds,
            parameter,
        } => {
            if ![
                "fog",
                "mud",
                "underwater",
                "fire",
                "electric-poles",
                "flowers",
                "sewage",
                "jungle",
                "high-poles",
                "sprinklers",
                "cemetery",
                "minefield",
                "healing-pits",
                "torn-world",
                "final-destination",
            ]
            .contains(&name.as_str())
                || rounds.is_some_and(|n| n == 0 || n > 1000)
                || !(-20..=100).contains(parameter)
            {
                return Err(error("Invalid arena preset or duration"));
            }
            if scope != "all"
                && !battle(c)?
                    .combatants
                    .iter()
                    .any(|p| scope == &format!("side:{}", p.side))
            {
                return Err(error("Choose the whole battlefield or an existing side"));
            }
            if ["high-poles", "healing-pits"].contains(&name.as_str()) && *parameter < 1 {
                return Err(error("Specify the check difficulty or healing amount"));
            }
            insert(c, name, "gm", scope, *rounds, *parameter);
        }
        FieldCommand::Remove { id } => {
            if id == "weather" {
                state_mut(c).weather = None;
            } else {
                let f = state_mut(c);
                if !f.rules.iter().any(|v| v.id == *id) {
                    return Err(error("Unknown battlefield effect"));
                }
                let trick = f
                    .rules
                    .iter()
                    .any(|v| v.id == *id && v.kind == "Trick Room");
                f.rules.retain(|v| v.id != *id);
                if trick {
                    reverse_remaining(c);
                }
            }
        }
        FieldCommand::Shelter {
            combatant_id,
            sheltered,
        } => {
            fighter(c, combatant_id)?;
            if *sheltered {
                state_mut(c).sheltered.insert(combatant_id.clone());
            } else {
                state_mut(c).sheltered.remove(combatant_id);
            }
        }
        FieldCommand::Typing {
            combatant_id,
            mode,
            types,
        } => type_rule(c, combatant_id, mode, types)?,
        FieldCommand::HealingPit {
            combatant_id,
            effect_id,
        } => {
            owner(c, actor, combatant_id)?;
            turn(c, combatant_id)?;
            ready(c, combatant_id)?;
            let p = fighter(c, combatant_id)?;
            let e = state(c)
                .unwrap()
                .rules
                .iter()
                .find(|e| e.id == *effect_id && e.kind == "healing-pits" && applies(&e.scope, p))
                .ok_or_else(|| error("Healing pit unavailable"))?
                .clone();
            heal(c, combatant_id, e.parameter, true)?;
            fighter_mut(c, combatant_id)?.actions += 1;
            state_mut(c).rules.retain(|e| e.id != *effect_id);
            advance(c);
        }
        FieldCommand::Resolve {
            event_id: _,
            accept,
        } => {
            let e = state(c).unwrap().events.first().unwrap().clone();
            if e.kind == "weather-choice" {
                return Err(error("Choose Hail or Snow for Snow Warning"));
            }
            if e.kind == "heal" {
                owner(c, actor, &e.target)?;
            } else {
                gm(c, actor)?;
            }
            state_mut(c).events.remove(0);
            if e.kind == "terrain-ruling" {
                if *accept {
                    apply_terrain(c, &e.terrains[0]);
                    cure_terrain(c, cat)?;
                }
            } else if e.kind == "terrain-contest" {
                let mut offset = 0;
                let mut scored = vec![];
                for t in &e.terrains {
                    let n = will(c, &t.source) as usize;
                    scored.push((t.clone(), rules::successes(&dice[offset..offset + n])));
                    offset += n;
                }
                let best = scored.iter().map(|(_, n)| *n).max().unwrap_or(0);
                let tied: Vec<_> = scored
                    .into_iter()
                    .filter(|(_, n)| *n == best)
                    .map(|(t, _)| t)
                    .collect();
                if tied.len() == 1 {
                    apply_terrain(c, &tied[0]);
                    cure_terrain(c, cat)?;
                } else {
                    let n = tied.iter().map(|t| will(c, &t.source)).sum();
                    enqueue(c, "", "terrain-contest", n, 0, 0, "Neutral");
                    state_mut(c).events.last_mut().unwrap().terrains = tied;
                }
            } else if e.kind == "weather-contest" {
                let mut offset = 0;
                let mut scores = vec![];
                for w in &e.contenders {
                    let p = fighter(c, w.source.trim_start_matches("ability:"))?;
                    let n = c.sheets[&p.sheet_id]
                        .workflow
                        .as_ref()
                        .map_or(p.runtime.will, |b| b.max_will)
                        .max(0) as usize;
                    scores.push(rules::successes(&dice[offset..offset + n]));
                    offset += n;
                }
                let max = scores.iter().max().copied().unwrap_or(0);
                let tied: Vec<_> = e
                    .contenders
                    .iter()
                    .zip(scores)
                    .filter(|(_, s)| *s == max)
                    .map(|(w, _)| w.clone())
                    .collect();
                if tied.len() == 1 {
                    if let Err(err) = set_weather(c, tied[0].clone()) {
                        state_mut(c).corrections.push(err.message);
                    }
                } else {
                    let n = tied
                        .iter()
                        .map(|w| {
                            let p = fighter(c, w.source.trim_start_matches("ability:")).unwrap();
                            c.sheets[&p.sheet_id]
                                .workflow
                                .as_ref()
                                .map_or(p.runtime.will, |b| b.max_will)
                                .max(0) as u32
                        })
                        .sum();
                    enqueue(c, "", "weather-contest", n, 0, 0, "Neutral");
                    state_mut(c).events.last_mut().unwrap().contenders = tied;
                }
            } else if *accept && active(c, &e.target) {
                match e.kind.as_str() {
                    "heal" => heal(c, &e.target, e.amount, false)?,
                    "toxic-spikes" => {
                        let kind = if dice.contains(&6) {
                            "Badly Poisoned"
                        } else {
                            "Poison"
                        };
                        match set_status(c, &e.target, kind, "Toxic Spikes", None, cat) {
                            Err(err)
                                if [
                                    "Species type is immune to this condition",
                                    "Selected ability is immune to this condition",
                                    "Battlefield prevents this condition",
                                ]
                                .contains(&err.message.as_str()) => {}
                            result => result?,
                        }
                    }
                    "check" if rules::successes(dice) < e.difficulty => {
                        hurt(c, &e.target, e.amount, &e.damage_type, cat)?
                    }
                    "drown" if rules::successes(dice) < e.difficulty => {
                        fighter_mut(c, &e.target)?.hp = 0
                    }
                    "Burn 2" | "Poison" if dice.contains(&6) => {
                        let _ = set_status(c, &e.target, &e.kind, "", None, cat);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(format!("Battlefield {cmd:?}; host dice {dice:?}. {r}"))
}

fn will(c: &Campaign, source: &str) -> u32 {
    fighter(c, source.trim_start_matches("ability:"))
        .map(|p| {
            c.sheets[&p.sheet_id]
                .workflow
                .as_ref()
                .map_or(p.runtime.will, |b| b.max_will)
                .max(0) as u32
        })
        .unwrap_or(0)
}
fn apply_terrain(c: &mut Campaign, t: &TerrainSource) {
    state_mut(c).rules.retain(|r| !r.kind.ends_with("Terrain"));
    insert(c, &t.name, &t.source, &t.scope, Some(4), 1);
}

/// Resolve context-sensitive catalog fields before every accuracy/damage calculation.
pub fn move_data(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> serde_json::Value {
    let mut m = m.clone();
    items::move_data(c, p, &mut m);
    forms::move_data(c, p, &mut m);
    if !enabled(c) {
        return m;
    }
    let name = m["Name"].as_str().unwrap_or("").to_owned();
    let w = weather(c, Some(p));
    if ["Thunder", "Hurricane", "Blizzard"].contains(&name.as_str()) {
        let n = if matches!(w, Some(Weather::Sunny | Weather::Desolate)) {
            -4
        } else if (name == "Blizzard" && matches!(w, Some(Weather::Snow | Weather::Hail)))
            || (name != "Blizzard" && matches!(w, Some(Weather::Rain | Weather::Typhoon)))
        {
            0
        } else {
            -2
        };
        m["Attributes"]["AccuracyReduction"] = serde_json::json!(n);
        if name == "Blizzard" {
            m["Attributes"]["NeverMiss"] =
                serde_json::json!(matches!(w, Some(Weather::Snow | Weather::Hail)));
        }
    }
    if name == "Weather Ball" {
        let t = match w {
            Some(Weather::Sunny | Weather::Desolate) => Some("Fire"),
            Some(Weather::Rain | Weather::Typhoon) => Some("Water"),
            Some(Weather::Snow | Weather::Hail) => Some("Ice"),
            Some(Weather::Sandstorm) => Some("Rock"),
            Some(Weather::StrongWinds) => Some("Flying"),
            None => None,
        };
        if let Some(t) = t {
            m["Type"] = serde_json::json!(t);
        }
    }
    extra::move_data(c, p, &mut m);
    special::move_data(c, p, &mut m);
    tactics::move_data(c, p, &mut m);
    reviewed::move_data(c, p, &mut m);
    m
}
pub fn target_bonus(c: &Campaign, target: &Combatant, m: &serde_json::Value) -> i32 {
    if enabled(c)
        && m["Name"] == "Brick Break"
        && state(c).unwrap().rules.iter().any(|r| {
            applies(&r.scope, target)
                && ["Reflect", "Light Screen", "Aurora Veil"].contains(&r.kind.as_str())
        })
    {
        2
    } else {
        0
    }
}

/// Stable rank-limited ally selection shares the GM's abstract range controls.
pub fn aura(c: &Campaign, target: &Combatant, ability: &str, include_self: bool) -> bool {
    if !enabled(c) {
        return false;
    }
    battle(c).is_ok_and(|_| {
        extra::participants(c)
            .iter()
            .filter(|q| q.deployed && q.hp > 0 && q.side == target.side && q.has_ability(ability))
            .any(|q| {
                let cap = [1, 2, 3, 4, 5, 6, 8, 10]
                    [crate::creation::rank(&c.sheets[&q.sheet_id].rank).unwrap_or(0)];
                let mut ids: Vec<_> = extra::participants(c)
                    .iter()
                    .filter(|p| {
                        p.deployed
                            && p.hp > 0
                            && p.side == q.side
                            && (include_self || p.id != q.id)
                            && flow(c).is_ok_and(|f| {
                                f.ranges
                                    .get(&q.id)
                                    .is_none_or(|ids| p.id == q.id || ids.contains(&p.id))
                            })
                    })
                    .map(|p| p.id.as_str())
                    .collect();
                ids.sort_by_key(|id| (*id != q.id.as_str(), *id));
                ids.truncate(cap);
                ids.contains(&target.id.as_str())
            })
    })
}

pub fn change_type(c: &mut Campaign, id: &str, mode: &str, types: &[String]) -> Result<()> {
    type_rule(c, id, mode, types)
}
pub fn change_type_inactive(c: &mut Campaign, id: &str) {
    let scope = format!("combatant:{id}");
    state_mut(c)
        .rules
        .retain(|r| !(r.scope == scope && r.source == "color-change"));
}

pub fn move_heal(c: &mut Campaign, id: &str, n: i32) -> Result<()> {
    heal(c, id, n, true)
}
pub fn color_change(c: &mut Campaign, id: &str, t: &str) -> Result<()> {
    fighter(c, id)?;
    let scope = format!("combatant:{id}");
    state_mut(c)
        .rules
        .retain(|r| !(r.scope == scope && r.source == "color-change"));
    insert(
        c,
        &format!("typing:replace:{t}"),
        "color-change",
        &scope,
        None,
        0,
    );
    Ok(())
}

pub fn sand_spit(c: &mut Campaign, id: &str) -> Result<()> {
    let raw = state(c)
        .unwrap()
        .weather
        .as_ref()
        .map(|w| w.weather)
        .or(state(c).unwrap().natural);
    if raw == Some(Weather::Sandstorm) {
        return Ok(());
    }
    let mut candidates = vec![WeatherSource {
        weather: Weather::Sandstorm,
        source: format!("ability:{id}"),
        rounds: Some(4),
    }];
    if let Some(old) = state(c).unwrap().weather.clone().filter(|w| {
        w.source.starts_with("ability:") && active(c, w.source.trim_start_matches("ability:"))
    }) {
        candidates.insert(0, old);
    }
    if candidates.len() > 1 {
        let n = candidates.iter().map(|w| will(c, &w.source)).sum();
        enqueue(c, "", "weather-contest", n, 0, 0, "Neutral");
        state_mut(c).events.last_mut().unwrap().contenders = candidates;
    } else if let Err(e) = set_weather(c, candidates.remove(0)) {
        state_mut(c).corrections.push(e.message);
    }
    Ok(())
}

pub fn screen_cleaner(c: &mut Campaign) {
    state_mut(c).rules.retain(|r| {
        ![
            "Reflect",
            "Light Screen",
            "Aurora Veil",
            "Safeguard",
            "Mist",
        ]
        .contains(&r.kind.as_str())
    });
    state_mut(c)
        .corrections
        .push("Screen Cleaner removed active force fields on both sides".into());
}
pub fn seed_sower(c: &mut Campaign, p: &Combatant) -> Result<()> {
    if state(c)
        .unwrap()
        .rules
        .iter()
        .any(|r| r.kind == "Grassy Terrain")
    {
        return Ok(());
    }
    let t = TerrainSource {
        name: "Grassy Terrain".into(),
        source: format!("ability:{}", p.id),
        scope: format!("side:{}", p.side),
    };
    if state(c)
        .unwrap()
        .rules
        .iter()
        .any(|r| r.kind.ends_with("Terrain"))
    {
        if !state(c)
            .unwrap()
            .events
            .iter()
            .any(|e| e.kind == "terrain-ruling" && e.target == p.id)
        {
            enqueue(c, &p.id, "terrain-ruling", 0, 0, 0, "Neutral");
            state_mut(c).events.last_mut().unwrap().terrains = vec![t];
        }
    } else {
        insert(c, &t.name, &t.source, &t.scope, Some(4), 1);
    }
    state_mut(c).corrections.push(format!(
        "Seed Sower: four-round Grassy Terrain for {} (overlap requires GM ruling)",
        p.side
    ));
    Ok(())
}
pub fn toxic_debris(c: &mut Campaign, p: &Combatant, a: &Combatant) {
    if p.side == a.side {
        return;
    }
    let scope = format!("side:{}", a.side);
    if state(c)
        .unwrap()
        .rules
        .iter()
        .any(|r| r.kind == "Toxic Spikes" && r.scope == scope)
    {
        return;
    }
    insert(
        c,
        "Toxic Spikes",
        &format!("ability:{}", p.id),
        &scope,
        None,
        0,
    );
    state_mut(c).corrections.push(format!(
        "Toxic Debris: Toxic Spikes on {}; entry uses one host Chance die",
        a.side
    ));
}

pub fn clear_ability_hazards(c: &mut Campaign) {
    if reviewed::enabled(c) {
        state_mut(c).rules.retain(|r| r.kind != "Toxic Spikes");
    }
}
