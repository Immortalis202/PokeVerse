//! Reviewed ability families using existing dice, damage, status and entry hooks.
//! These hooks are opt-in per encounter; historical engines keep their snapshots.
use super::*;
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct ExtraState {
    pub enabled: bool,
    pub active: BTreeSet<String>,
    pub fainted: BTreeSet<String>,
    pub round_started: u32,
}
pub fn enabled(c: &Campaign) -> bool {
    c.battle
        .as_ref()
        .and_then(|b| b.flow.as_ref())
        .is_some_and(|f| f.extra.enabled)
}
fn ailment(p: &Combatant) -> bool {
    p.runtime.statuses.iter().any(|s| {
        s.kind.starts_with("Burn")
            || s.kind.contains("Poison")
            || ["Sleep", "Paralysis", "Frozen Solid"].contains(&s.kind.as_str())
    })
}
pub fn participants(c: &Campaign) -> &[Combatant] {
    if enabled(c) {
        if let Some(ctx) = c
            .battle
            .as_ref()
            .and_then(|b| b.flow.as_ref())
            .and_then(|f| f.queue.first())
            .and_then(|q| q.damage_context.as_ref())
            .filter(|ctx| !ctx.ability_sources.is_empty())
        {
            return &ctx.ability_sources;
        }
    }
    c.battle.as_ref().map_or(&[], |b| b.combatants.as_slice())
}
fn living(c: &Campaign) -> impl Iterator<Item = &Combatant> {
    participants(c).iter().filter(|p| p.deployed && p.hp > 0)
}
fn count(p: &Combatant, k: &str) -> i32 {
    *p.runtime.ability.counters.get(k).unwrap_or(&0)
}
fn own_stat(c: &Campaign, p: &Combatant, key: &str) -> i32 {
    let low = p.hp <= p.max_hp / 2;
    let w = field::weather(c, Some(p));
    let n = match (p.selected_ability.as_str(), key) {
        ("Slow Start", "Strength" | "Dexterity")
            if battle(c).map_or(0, |b| b.round as i32) - count(p, "entry round") >= 3 =>
        {
            2
        }
        ("Guts", "Strength") if ailment(p) => 2,
        ("Toxic Boost", "Strength")
            if p.runtime.statuses.iter().any(|s| s.kind.contains("Poison")) =>
        {
            2
        }
        ("Flare Boost", "Special")
            if p.runtime
                .statuses
                .iter()
                .any(|s| s.kind.starts_with("Burn")) =>
        {
            2
        }
        ("Berserk", "Special") if low => 2,
        ("Anger Shell", "Strength" | "Special" | "Dexterity") if low => 1,
        ("Quick Feet", "Dexterity") if ailment(p) => 2,
        ("Moxie", "Strength") => count(p, "Moxie"),
        ("Sap Sipper", "Strength") => count(p, "Sap Sipper"),
        ("Lightning Rod" | "Storm Drain", "Special") if tactics::enabled(c) => {
            count(p, &p.selected_ability)
        }
        ("Speed Boost", "Dexterity") => count(p, "Speed Boost"),
        ("Huge Power" | "Pure Power", "Strength") => 2,
        ("Chlorophyll", "Dexterity")
            if matches!(w, Some(field::Weather::Sunny | field::Weather::Desolate)) =>
        {
            2
        }
        ("Swift Swim", "Dexterity")
            if matches!(w, Some(field::Weather::Rain | field::Weather::Typhoon)) =>
        {
            2
        }
        ("Slush Rush", "Dexterity")
            if matches!(w, Some(field::Weather::Snow | field::Weather::Hail)) =>
        {
            1
        }
        ("Sand Rush", "Dexterity") if w == Some(field::Weather::Sandstorm) => 1,
        ("Solar Power", "Special")
            if matches!(w, Some(field::Weather::Sunny | field::Weather::Desolate)) =>
        {
            2
        }
        ("Surge Surfer", "Dexterity") if field::has(c, "Electric Terrain", p) => 2,
        ("Hadron Engine", "Special") if field::has(c, "Electric Terrain", p) => 2,
        ("Orichalcum Pulse", "Strength")
            if matches!(w, Some(field::Weather::Sunny | field::Weather::Desolate)) =>
        {
            2
        }
        ("Plus", "Special")
            if living(c)
                .any(|q| q.id != p.id && q.side == p.side && q.selected_ability == "Minus") =>
        {
            2
        }
        ("Minus", "Special")
            if living(c)
                .any(|q| q.id != p.id && q.side == p.side && q.selected_ability == "Plus") =>
        {
            2
        }
        _ => 0,
    };
    let mut n = n.max(count(p, &format!("boost:{key}")));
    if highest_active(c, p) {
        let options = highest(c, p);
        let selected = if options.len() == 1 {
            options.first()
        } else {
            options
                .iter()
                .find(|k| count(p, &format!("highest:{k}")) > 0)
        };
        if selected.is_some_and(|k| k == key) {
            n = n.max(if key == "Dexterity" { 2 } else { 1 });
        }
    }
    n
}
fn protected(c: &Campaign, p: &Combatant, key: &str) -> bool {
    ["Clear Body", "Full Metal Body", "White Smoke"].contains(&p.selected_ability.as_str())
        || (key == "Strength"
            && ["Hyper Cutter", "Guard Dog"].contains(&p.selected_ability.as_str()))
        || (key == "Defense" && p.selected_ability == "Big Pecks")
        || field::aura(c, p, "Flower Veil", true)
}
pub fn apply_stats(c: &Campaign, p: &Combatant, s: &mut Sheet) {
    for (key, amount) in &p.runtime.modifiers {
        let n = special::modifier(c, p, key).unwrap_or_else(|| match p.selected_ability.as_str() {
            "Contrary" => -*amount,
            "Simple" => amount + amount.signum(),
            _ => *amount,
        });
        if let Some(v) = s.stats.get_mut(key) {
            *v = (*v + n).clamp(1, 10);
        }
    }
    if status(p, "Paralysis") && p.selected_ability != "Quick Feet" {
        if let Some(v) = s.stats.get_mut("Dexterity") {
            *v = (*v - 2).max(0);
        }
    }
    let intimidated = flow(c).is_ok_and(|f| f.intimidate.values().any(|ids| ids.contains(&p.id)));
    for key in ["Strength", "Dexterity", "Vitality", "Special", "Insight"] {
        let mut positive = own_stat(c, p, key);
        let mut negative = 0;
        if p.selected_ability != "Clear Body" {
            if key == "Strength"
                && matches!(
                    field::weather(c, Some(p)),
                    Some(field::Weather::Sunny | field::Weather::Desolate)
                )
                && field::aura(c, p, "Flower Gift", true)
            {
                positive = positive.max(2);
            }
            if key == "Special" && field::aura(c, p, "Battery", false) {
                positive = positive.max(1);
            }
        }
        if key == "Strength"
            && intimidated
            && p.selected_ability != "Inner Focus"
            && !protected(c, p, key)
        {
            match p.selected_ability.as_str() {
                "Contrary" => positive = positive.max(1),
                "Simple" => negative = -2,
                _ => negative = -1,
            }
        }
        if ![
            "Beads of Ruin",
            "Sword of Ruin",
            "Tablets of Ruin",
            "Vessel of Ruin",
        ]
        .contains(&p.selected_ability.as_str())
            && !protected(c, p, key)
        {
            let name = match key {
                "Strength" => "Tablets of Ruin",
                "Special" => "Vessel of Ruin",
                _ => "",
            };
            if !name.is_empty() && global_aura(c, p, name, false) {
                negative = negative.min(-2);
            }
        }
        let permanent = if key == "Strength"
            && ["Huge Power", "Pure Power"].contains(&if ability_changes::enabled(c) {
                c.sheets[&p.sheet_id].ability.as_str()
            } else {
                p.selected_ability.as_str()
            }) {
            2
        } else {
            0
        };
        if let Some(v) = s.stats.get_mut(key) {
            let minimum = if key == "Dexterity"
                && status(p, "Paralysis")
                && p.selected_ability != "Quick Feet"
            {
                0
            } else {
                1
            };
            *v = (*v + positive - permanent + negative).clamp(minimum, 10);
        }
    }
}
pub fn defense(c: &Campaign, p: &Combatant, physical: bool) -> i32 {
    let key = if physical {
        "Defense"
    } else {
        "SpecialDefense"
    };
    let mut positive = count(p, &format!("boost:{key}"));
    let mut negative = count(p, &format!("debuff:{key}"));
    if physical
        && ((p.selected_ability == "Marvel Scale" && ailment(p))
            || (p.selected_ability == "Grass Pelt" && field::has(c, "Grassy Terrain", p)))
    {
        positive = positive.max(2);
    }
    if !physical
        && p.selected_ability != "Clear Body"
        && matches!(
            field::weather(c, Some(p)),
            Some(field::Weather::Sunny | field::Weather::Desolate)
        )
        && field::aura(c, p, "Flower Gift", true)
    {
        positive = positive.max(2);
    }
    if p.selected_ability == "Anger Shell" && p.hp <= p.max_hp / 2 {
        negative = negative.min(-1);
    }
    if ![
        "Beads of Ruin",
        "Sword of Ruin",
        "Tablets of Ruin",
        "Vessel of Ruin",
    ]
    .contains(&p.selected_ability.as_str())
        && !protected(c, p, key)
        && global_aura(
            c,
            p,
            if physical {
                "Sword of Ruin"
            } else {
                "Beads of Ruin"
            },
            false,
        )
    {
        negative = negative.min(-2);
    }
    let move_mod = *p.runtime.modifiers.get(key).unwrap_or(&0);
    let move_mod =
        special::modifier(c, p, key).unwrap_or_else(|| match p.selected_ability.as_str() {
            "Contrary" => -move_mod,
            "Simple" => move_mod + move_mod.signum(),
            _ => move_mod,
        });
    positive + negative + move_mod
}
pub fn move_data(c: &Campaign, p: &Combatant, m: &mut serde_json::Value) {
    if !enabled(c) {
        return;
    }
    if p.selected_ability == "Long Reach" {
        m["Attributes"]["Ranged"] = serde_json::json!(true);
    }
    let converted = match p.selected_ability.as_str() {
        "Aerilate" if m["Type"] == "Normal" => Some("Flying"),
        "Pixilate" if m["Type"] == "Normal" => Some("Fairy"),
        "Galvanize" if m["Type"] == "Normal" => Some("Electric"),
        "Refrigerate" if m["Type"] == "Normal" => Some("Ice"),
        "Normalize" => Some("Normal"),
        "Liquid Voice" if m["Attributes"]["SoundMove"] == true => Some("Water"),
        _ => None,
    };
    if let Some(t) = converted {
        m["Type"] = serde_json::json!(t);
    }
    if p.selected_ability == "No Guard"
        && count(p, "No Guard round") == battle(c).map_or(0, |b| b.round as i32)
    {
        m["Attributes"]["AccuracyReduction"] = serde_json::json!(0);
    }
    if p.selected_ability == "Triage"
        && m["Category"] == "Support"
        && [
            "Recover",
            "Slack Off",
            "Soft Boiled",
            "Synthesis",
            "Morning Sun",
            "Moonlight",
            "Heal Bell",
            "Aromatherapy",
            "Refresh",
        ]
        .contains(&m["Name"].as_str().unwrap_or(""))
    {
        m["Attributes"]["Reaction"] = serde_json::json!("1");
    }
}
pub fn bonus(c: &Campaign, p: &Combatant, target: &Combatant, m: &serde_json::Value) -> i32 {
    if !enabled(c) {
        return 0;
    }
    let t = m["Type"].as_str().unwrap_or("");
    let name = m["Name"].as_str().unwrap_or("");
    let attrs = &m["Attributes"];
    let mut own = match p.selected_ability.as_str() {
        "Aerilate" if t == "Flying" => 1,
        "Pixilate" if t == "Fairy" => 1,
        "Galvanize" if t == "Electric" => 1,
        "Refrigerate" if t == "Ice" => 1,
        "Normalize" if t == "Normal" => 1,
        "Dragon Maw" if t == "Dragon" => 2,
        "Transistor" if t == "Electric" => 2,
        "Water Bubble" if t == "Water" => 2,
        "Rocky Payload" if t == "Rock" => 1,
        "Steelworker" if t == "Steel" => 1,
        "Strong Jaw" if attrs["BiteMove"] == true => 1,
        "Sharpness" if attrs["CutterMove"] == true => 1,
        "Wind Rider" if attrs["WindMove"] == true => 1,
        "Tough Claws" if abilities::contact(m) => 1,
        "Punk Rock" if attrs["SoundMove"] == true => 1,
        "Reckless"
            if attrs["Recoil"].is_object()
                || attrs["Recoil"].is_number()
                || attrs["Recoil"] == true
                || name == "Submission" =>
        {
            2
        }
        "Mega Launcher" if name.contains("Pulse") || name.contains("Aura") => 2,
        "Sand Force"
            if field::weather(c, Some(p)) == Some(field::Weather::Sandstorm)
                && ["Ground", "Steel", "Rock"].contains(&t) =>
        {
            1
        }
        "Analytic" if p.initiative < target.initiative => 1,
        _ => 0,
    };
    if t == "Steel" && field::aura(c, p, "Steely Spirit", true) {
        own = own.max(1);
    }
    if field::aura(c, p, "Power Spot", false)
        || (m["Category"] == "Special" && field::aura(c, p, "Battery", false))
    {
        own = own.max(1);
    }
    let aura = match t {
        "Dark" => "Dark Aura",
        "Fairy" => "Fairy Aura",
        _ => "",
    };
    if !aura.is_empty() && global_aura(c, p, aura, true) {
        if global_aura(c, p, "Aura Break", true) {
            if p.selected_ability == aura {
                own = own.min(-2);
            } else {
                own = own.max(2);
            }
        } else {
            own = own.max(2);
        }
    }
    if t == "Electric"
        && abilities::context(
            c,
            &p.id,
            flow(c)
                .ok()
                .and_then(|f| f.queue.first())
                .map_or(0, |q| q.sequence),
        )
        .map_or(count(p, "charged-electric"), |ctx| {
            *ctx.counters.get("charged-electric").unwrap_or(&0)
        }) > 0
    {
        own = own.max(2);
    }
    own
}
pub fn reduction(c: &Campaign, p: &Combatant, m: &serde_json::Value, effective: i32) -> i32 {
    if !enabled(c) {
        return 0;
    }
    let t = m["Type"].as_str().unwrap_or("");
    let physical = m["Category"] == "Physical";
    let name = m["Name"].as_str().unwrap_or("");
    match p.selected_ability.as_str() {
        "Fluffy" => (if physical { 2 } else { 0 }) - (if t == "Fire" { 2 } else { 0 }),
        "Heatproof" if t == "Fire" => 2,
        "Water Bubble" if t == "Fire" => 1,
        "Ice Scales" if !physical => 2,
        "Prism Armor" if effective > 0 => 2,
        "Shadow Shield" if p.hp == p.max_hp => 2,
        "Punk Rock" if m["Attributes"]["SoundMove"] == true => 2,
        "Purifying Salt" if t == "Ghost" => 3,
        "Bulletproof"
            if (physical && !abilities::contact(m))
                || (!physical
                    && [
                        "Ball",
                        "Blast",
                        "Bomb",
                        "Cannon",
                        "Octazooka",
                        "Puff",
                        "Shot",
                        "Sphere",
                        "Wrecker",
                    ]
                    .iter()
                    .any(|k| name.contains(k))) =>
        {
            1
        }
        _ => 0,
    }
}
pub fn immune(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> bool {
    enabled(c)
        && match p.selected_ability.as_str() {
            "Earth Eater" => m["Type"] == "Ground" && !field::has(c, "Gravity", p),
            "Water Compaction" => m["Type"] == "Water",
            "Well-Baked Body" => m["Type"] == "Fire",
            _ => false,
        }
}
pub fn source_immune(c: &Campaign, source: &Combatant, target: &Combatant, effective: i32) -> bool {
    enabled(c)
        && ((target.id != source.id
            && target.side == source.side
            && ["Telepathy", "Friend Guard"].contains(&target.selected_ability.as_str()))
            || (target.selected_ability == "Wonder Guard" && effective <= 0))
}
pub fn status_immune(c: &Campaign, p: &Combatant, kind: &str, source: Option<&str>) -> bool {
    if !enabled(c) {
        return false;
    }
    (p.selected_ability == "Purifying Salt" && ailment_kind(kind))
        || (p.selected_ability == "Comatose" && kind != "Sleep")
        || (p.selected_ability == "Thermal Exchange" && kind.starts_with("Burn"))
        || (p.selected_ability == "Overcoat" && kind == "powder")
        || (p.selected_ability == "Leaf Guard"
            && matches!(
                field::weather(c, Some(p)),
                Some(field::Weather::Sunny | field::Weather::Desolate)
            ))
        || (kind == "Sleep" && field::aura(c, p, "Sweet Veil", true))
        || (kind.contains("Poison") && field::aura(c, p, "Pastel Veil", true))
        || (kind == "In Love" && field::aura(c, p, "Aroma Veil", true))
        || (source != Some(p.id.as_str())
            && ailment_kind(kind)
            && field::aura(c, p, "Flower Veil", true))
}
pub fn after_status(c: &mut Campaign, id: &str, kind: &str) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let p = fighter(c, id)?.clone();
    if kind.contains("Poison") && p.selected_ability == "Poison Heal" {
        fighter_mut(c, id)?
            .runtime
            .ability
            .counters
            .insert("poison-heal-rounds".into(), 0);
    }
    if kind == "Flinch" && p.selected_ability == "Steadfast" {
        once(c, id, "Steadfast", &[("Dexterity", 1)])?;
    }
    Ok(())
}
fn boost(c: &mut Campaign, id: &str, key: &str, n: i32) -> Result<()> {
    let k = format!("{}:{key}", if n < 0 { "debuff" } else { "boost" });
    let v = fighter_mut(c, id)?
        .runtime
        .ability
        .counters
        .entry(k)
        .or_default();
    if n < 0 {
        *v = (*v).min(n);
    } else {
        *v = (*v).max(n);
    }
    Ok(())
}
fn once(c: &mut Campaign, id: &str, flag: &str, values: &[(&str, i32)]) -> Result<()> {
    if fighter(c, id)?.runtime.ability.flags.contains(flag) {
        return Ok(());
    }
    fighter_mut(c, id)?
        .runtime
        .ability
        .flags
        .insert(flag.into());
    for (k, n) in values {
        boost(c, id, k, *n)?;
    }
    Ok(())
}
fn enqueue(
    c: &mut Campaign,
    p: &Combatant,
    target: &str,
    kind: &str,
    m: &str,
    choices: Vec<String>,
) {
    let id = format!(
        "{}:{}:{}",
        c.revision + 1,
        flow(c).unwrap().ability_events.len(),
        p.id
    );
    flow_mut(c).ability_events.push(abilities::AbilityEvent {
        id,
        combatant_id: p.id.clone(),
        target_id: target.into(),
        ability: p.selected_ability.clone(),
        kind: kind.into(),
        move_name: m.into(),
        choices,
    });
}
fn hurt(c: &mut Campaign, id: &str, n: i32) -> Result<()> {
    if !["Magic Guard", "Wonder Guard"].contains(&fighter(c, id)?.selected_ability.as_str()) {
        let p = fighter_mut(c, id)?;
        p.hp = (p.hp - n).max(0);
    }
    Ok(())
}
pub fn disguise(c: &mut Campaign, id: &str, n: i32) -> Result<i32> {
    if enabled(c)
        && n > 0
        && fighter(c, id)?.selected_ability == "Disguise"
        && !fighter(c, id)?.runtime.ability.flags.contains("Disguise")
    {
        fighter_mut(c, id)?
            .runtime
            .ability
            .flags
            .insert("Disguise".into());
        Ok(0)
    } else {
        Ok(n)
    }
}
pub fn hit(c: &mut Campaign, q: &QueuedAction, before: i32, m: &serde_json::Value) -> Result<()> {
    if !enabled(c) || q.actor_id == q.target_id {
        return Ok(());
    }
    let source = fighter(c, &q.actor_id)?.clone();
    let target = fighter(c, &q.target_id)?.clone();
    let dealt = before > target.hp;
    let t = m["Type"].as_str().unwrap_or("");
    if target.hp == 0 && before > 0 && source.side != target.side && source.hp > 0 {
        if source.selected_ability == "Beast Boost" && count(&source, "Beast Boost total") < 3 {
            let sheet = rules::effective_sheet(c, &source)?;
            let max = ["Strength", "Dexterity", "Vitality", "Special", "Insight"]
                .iter()
                .filter_map(|k| sheet.stats.get(*k))
                .max()
                .copied()
                .unwrap_or(0);
            let options: Vec<String> = ["Strength", "Dexterity", "Vitality", "Special", "Insight"]
                .iter()
                .filter(|k| sheet.stats.get(**k) == Some(&max))
                .map(|k| (*k).into())
                .collect();
            enqueue(
                c,
                &source,
                &source.id,
                "extra-beast",
                "fainted foe",
                options,
            );
        }
        let key = match source.selected_ability.as_str() {
            "Chilling Neigh" => Some("Strength"),
            "Grim Neigh" | "Soul Heart" => Some("Special"),
            _ => None,
        };
        if let Some(key) = key {
            let n = (count(&source, &format!("boost:{key}")) + 1).min(3);
            boost(c, &source.id, key, n)?;
        }
    }
    if target.hp == 0 && before > 0 && target.selected_ability == "Innards Out" {
        hurt(c, &source.id, before)?;
    }
    if target.selected_ability == "Sand Spit" && abilities::contact(m) {
        field::sand_spit(c, &target.id)?;
    }
    if target.hp > 0 {
        match target.selected_ability.as_str() {
            "Anger Point" if q.critical => {
                boost(c, &target.id, "Strength", 3)?;
            }
            "Stamina" if dealt => once(
                c,
                &target.id,
                "Stamina",
                &[("Defense", 1), ("SpecialDefense", 1)],
            )?,
            "Water Compaction" if t == "Water" => {
                once(c, &target.id, "Water Compaction", &[("Defense", 2)])?
            }
            "Well-Baked Body" if t == "Fire" => {
                once(c, &target.id, "Well-Baked Body", &[("Defense", 2)])?
            }
            "Thermal Exchange" if t == "Fire" => {
                once(c, &target.id, "Thermal Exchange", &[("Strength", 1)])?
            }
            "Steam Engine" if dealt && ["Fire", "Water"].contains(&t) => {
                once(c, &target.id, "Steam Engine", &[("Dexterity", 3)])?
            }
            "Weak Armor" if m["Category"] == "Physical" => once(
                c,
                &target.id,
                "Weak Armor",
                &[("Dexterity", 1), ("Defense", -1)],
            )?,
            "Rattled" if ["Bug", "Dark", "Ghost"].contains(&t) => {
                once(c, &target.id, "Rattled", &[("Dexterity", 1)])?
            }
            "Earth Eater" if t == "Ground" && !field::has(c, "Gravity", &target) => enqueue(
                c,
                &target,
                &target.id,
                "heal",
                &q.move_name,
                vec!["heal".into(), "decline".into()],
            ),
            "Cursed Body"
                if dealt
                    && source.hp > 0
                    && source
                        .runtime
                        .statuses
                        .iter()
                        .filter(|s| s.kind == "Disable")
                        .count()
                        < 3 =>
            {
                enqueue(
                    c,
                    &target,
                    &source.id,
                    "extra-disable",
                    &q.move_name,
                    vec!["roll".into()],
                )
            }
            _ => {}
        }
        if dealt
            && ((target.selected_ability == "Electromorphosis")
                || (target.selected_ability == "Wind Power" && m["Attributes"]["WindMove"] == true))
        {
            fighter_mut(c, &target.id)?
                .runtime
                .ability
                .counters
                .insert("charged-electric".into(), 2);
        }
        if target.selected_ability == "Color Change" && dealt {
            field::color_change(c, &target.id, t)?;
        }
    }
    if abilities::contact(m) && source.side != target.side && source.hp > 0 {
        if ["Gooey", "Tangling Hair"].contains(&target.selected_ability.as_str())
            && source.selected_ability != "Long Reach"
            && !target
                .runtime
                .ability
                .flags
                .contains(&target.selected_ability)
        {
            fighter_mut(c, &target.id)?
                .runtime
                .ability
                .flags
                .insert(target.selected_ability.clone());
            change_stat(c, &target.id, &source.id, "Dexterity", -1)?;
        }
        if ["Poison Touch", "Toxic Chain"].contains(&source.selected_ability.as_str())
            && target.hp > 0
        {
            enqueue(
                c,
                &source,
                &target.id,
                "extra-poison",
                &q.move_name,
                vec!["roll".into()],
            );
        }
    }
    Ok(())
}
fn apply_stat(
    c: &mut Campaign,
    source: &str,
    id: &str,
    key: &str,
    n: i32,
    reflected: bool,
) -> Result<()> {
    if tactics::baton_blocked(c, source, id, key, n) {
        return Err(error("Baton Pass prevents this user from raising its own attributes until the recipient faints or the encounter ends"));
    }
    if ![
        "Strength",
        "Dexterity",
        "Vitality",
        "Special",
        "Insight",
        "Defense",
        "SpecialDefense",
    ]
    .contains(&key)
        || !(-3..=3).contains(&n)
        || n == 0
    {
        return Err(error(
            "Stat change needs a supported attribute/defense and a nonzero change from -3 to 3",
        ));
    }
    let p = fighter(c, id)?.clone();
    let hostile = source != id && fighter(c, source)?.side != p.side;
    if n < 0 && hostile && p.selected_ability == "Mirror Armor" && !reflected {
        return apply_stat(c, id, source, key, n, true);
    }
    if source != id && ((n < 0 && protected(c, &p, key)) || p.selected_ability == "Clear Body") {
        if n < 0 && key == "Strength" && p.selected_ability == "Guard Dog" {
            boost(c, id, key, 2)?;
        }
        return Ok(());
    }
    let reviewed = if special::enabled(c) {
        Some(special::record_modifier(c, source, id, key, n)?)
    } else {
        None
    };
    let rt = &mut fighter_mut(c, id)?.runtime;
    let old = *rt.modifiers.get(key).unwrap_or(&0);
    rt.modifiers.insert(
        key.into(),
        reviewed.unwrap_or_else(|| (old + n).clamp(-3, 3)),
    );
    if n < 0 && p.selected_ability == "Defiant" {
        once(c, id, "Defiant", &[("Strength", 2)])?;
    }
    if n < 0 && hostile && p.selected_ability == "Competitive" {
        once(c, id, "Competitive", &[("Special", 2)])?;
    }
    Ok(())
}
pub fn accuracy_attempt(
    c: &mut Campaign,
    id: &str,
    m: &serde_json::Value,
    cat: &Catalog,
) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let p = fighter(c, id)?.clone();
    if p.selected_ability == "Libero"
        || (p.selected_ability == "Protean" && m["Category"] != "Support")
    {
        let m = field::move_data(c, &p, m);
        field::change_type(
            c,
            id,
            "replace",
            &[m["Type"].as_str().unwrap_or("Normal").into()],
        )?;
    }
    let _ = cat;
    Ok(())
}
pub fn consume_charge(c: &mut Campaign, id: &str, m: &serde_json::Value) {
    if enabled(c) && m["Type"] == "Electric" {
        if let Ok(p) = fighter_mut(c, id) {
            p.runtime.ability.counters.remove("charged-electric");
        }
    }
}
pub fn round_start(c: &mut Campaign) -> Result<()> {
    if !enabled(c) || flow(c)?.extra.round_started == battle(c)?.round {
        return Ok(());
    }
    flow_mut(c).extra.round_started = battle(c)?.round;
    let ps: Vec<_> = living(c).cloned().collect();
    for p in ps {
        if p.selected_ability == "Quick Draw" {
            enqueue(
                c,
                &p,
                &p.id,
                "extra-quickdraw",
                "round start",
                vec!["roll".into()],
            );
        }
        if p.selected_ability == "No Guard" {
            enqueue(
                c,
                &p,
                &p.id,
                "extra-no-guard",
                "round start",
                vec!["activate".into(), "decline".into()],
            );
        }
    }
    Ok(())
}
pub fn end_round(c: &mut Campaign) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    let ps: Vec<_> = living(c).cloned().collect();
    for p in &ps {
        if ["Natural Cure", "Shed Skin"].contains(&p.selected_ability.as_str())
            && !p.runtime.statuses.is_empty()
        {
            enqueue(
                c,
                p,
                &p.id,
                if p.selected_ability == "Shed Skin" {
                    "extra-shed"
                } else {
                    "extra-cure"
                },
                "round end",
                vec!["roll".into()],
            );
        }
        if p.selected_ability == "Healer" {
            let choices: Vec<_> = ps
                .iter()
                .filter(|t| t.id != p.id && t.side == p.side && !t.runtime.statuses.is_empty())
                .map(|t| t.id.clone())
                .collect();
            if !choices.is_empty() {
                enqueue(c, p, &p.id, "extra-healer", "choose an ally", choices);
            }
        }
        if p.selected_ability == "Bad Dreams" {
            for t in ps.iter().filter(|t| status(t, "Sleep")) {
                hurt(c, &t.id, 1)?;
            }
        }
        if p.selected_ability == "Poison Heal"
            && p.runtime.statuses.iter().any(|s| s.kind.contains("Poison"))
        {
            if special::enabled(c) {
                special::heal(c, &p.id, 1, false)?;
            }
            let modern = special::enabled(c);
            let p = fighter_mut(c, &p.id)?;
            if !modern && p.hp > 0 {
                p.hp = (p.hp + 1).min(p.max_hp);
            }
            let n = p
                .runtime
                .ability
                .counters
                .entry("poison-heal-rounds".into())
                .or_default();
            *n += 1;
            if *n >= 3 {
                p.runtime.statuses.retain(|s| !s.kind.contains("Poison"));
            }
        }
    }
    Ok(())
}
pub fn lifecycle(before: &Campaign, c: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !enabled(c) || !c.battle.as_ref().is_some_and(|b| b.active) {
        return Ok(());
    }
    let ps = battle(c)?.combatants.clone();
    let old = flow(c)?.extra.active.clone();
    for p in &ps {
        if p.hp == 0
            && before
                .battle
                .as_ref()
                .is_some_and(|b| b.combatants.iter().any(|q| q.id == p.id && q.hp > 0))
        {
            flow_mut(c).extra.fainted.insert(p.id.clone());
        }
        if !p.deployed && p.selected_ability == "Color Change" {
            field::change_type_inactive(c, &p.id);
        }
        if !p.deployed || p.hp == 0 {
            continue;
        }
        if p.selected_ability == "Comatose" && !status(p, "Sleep") {
            fighter_mut(c, &p.id)?.runtime.statuses.push(Status {
                kind: "Sleep".into(),
                detail: "Comatose".into(),
                rounds: None,
                progress: 0,
            });
        }
        let intimidated = flow(c)?.intimidate.values().any(|ids| ids.contains(&p.id));
        let was_intimidated = before
            .battle
            .as_ref()
            .and_then(|b| b.flow.as_ref())
            .is_some_and(|f| f.intimidate.values().any(|ids| ids.contains(&p.id)));
        if intimidated && !was_intimidated {
            match p.selected_ability.as_str() {
                "Defiant" => once(c, &p.id, "Defiant", &[("Strength", 2)])?,
                "Competitive" => once(c, &p.id, "Competitive", &[("Special", 2)])?,
                "Guard Dog" => boost(c, &p.id, "Strength", 2)?,
                "Rattled" => once(c, &p.id, "Rattled", &[("Dexterity", 1)])?,
                _ => {}
            }
        }
        if highest_active(c, p) {
            let options = highest(c, p);
            if options.len() > 1
                && !options
                    .iter()
                    .any(|k| count(p, &format!("highest:{k}")) > 0)
                && !flow(c)?
                    .ability_events
                    .iter()
                    .any(|e| e.combatant_id == p.id && e.kind == "extra-highest")
            {
                enqueue(
                    c,
                    p,
                    &p.id,
                    "extra-highest",
                    "choose tied highest attribute",
                    options,
                );
            }
        }
        if !old.contains(&p.id) {
            let round = battle(c)?.round;
            fighter_mut(c, &p.id)?
                .runtime
                .ability
                .counters
                .insert("entry round".into(), round as i32);
            match p.selected_ability.as_str() {
                "Dauntless Shield" => boost(c, &p.id, "Defense", 2)?,
                "Intrepid Sword" => boost(c, &p.id, "Strength", 2)?,
                "Download" => enqueue(
                    c,
                    p,
                    &p.id,
                    "extra-download",
                    "entry scan",
                    vec!["Strength".into(), "Special".into()],
                ),
                "Hospitality" if p.runtime.will > 0 => {
                    let mut choices: Vec<_> = ps
                        .iter()
                        .filter(|t| {
                            t.deployed
                                && t.hp > 0
                                && t.hp < t.max_hp
                                && t.id != p.id
                                && t.side == p.side
                        })
                        .map(|t| t.id.clone())
                        .collect();
                    if !choices.is_empty() {
                        choices.push("decline".into());
                        enqueue(c, p, &p.id, "extra-hospitality", "entry heal", choices);
                    }
                }
                "Curious Medicine" => {
                    for t in ps
                        .iter()
                        .filter(|t| {
                            t.deployed
                                && t.hp > 0
                                && t.side == p.side
                                && t.id != p.id
                                && in_range(c, p, &t.id)
                        })
                        .cloned()
                        .collect::<Vec<_>>()
                    {
                        special::clear_modifiers(c, &t.id);
                        fighter_mut(c, &t.id)?.runtime.modifiers.clear();
                        fighter_mut(c, &t.id)?
                            .runtime
                            .ability
                            .counters
                            .retain(|k, _| {
                                !k.starts_with("boost:")
                                    && !k.starts_with("debuff:")
                                    && !["Moxie", "Sap Sipper", "Speed Boost"].contains(&k.as_str())
                            });
                    }
                }
                "Supreme Overlord" => {
                    let n = flow(c)?
                        .extra
                        .fainted
                        .iter()
                        .filter(|id| {
                            ps.iter()
                                .any(|q| q.id == **id && q.side == p.side && q.id != p.id)
                        })
                        .count()
                        .min(3) as i32;
                    boost(c, &p.id, "Strength", n)?;
                    boost(c, &p.id, "Special", n)?;
                }
                _ => {}
            }
        }
    }
    flow_mut(c).extra.active = ps
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .map(|p| p.id.clone())
        .collect();
    let _ = cat;
    Ok(())
}
fn in_range(c: &Campaign, p: &Combatant, id: &str) -> bool {
    field::aura(
        c,
        &battle(c)
            .unwrap()
            .combatants
            .iter()
            .find(|v| v.id == id)
            .unwrap()
            .clone(),
        &p.selected_ability,
        false,
    )
}
pub fn dice(c: &Campaign, e: &abilities::AbilityEvent, choice: &str) -> Option<u32> {
    if !e.kind.starts_with("extra-") {
        return None;
    }
    let wonder = fighter(c, &e.target_id).is_ok_and(|p| p.selected_ability == "Wonder Skin");
    Some(match e.kind.as_str() {
        "extra-disable" | "extra-poison" | "extra-cure" | "extra-heal-chance" | "extra-shed" => {
            if choice == "decline" {
                0
            } else if e.ability == "Poison Touch" {
                if wonder {
                    0
                } else {
                    2
                }
            } else {
                if wonder && ["extra-disable", "extra-poison"].contains(&e.kind.as_str()) {
                    1
                } else {
                    3
                }
            }
        }
        "extra-quickdraw" => 3,
        "extra-shed-select" => {
            let n = fighter(c, &e.target_id).map_or(1, |p| p.runtime.statuses.len());
            selection_dice(n)
        }
        _ => 0,
    })
}
pub fn resolve(
    c: &mut Campaign,
    actor: &str,
    e: &abilities::AbilityEvent,
    choice: &str,
    faces: &[u8],
    cat: &Catalog,
) -> Result<()> {
    if !e.kind.starts_with("extra-") {
        return Ok(());
    }
    if choice == "decline" {
        return Ok(());
    }
    match e.kind.as_str() {
        "extra-quickdraw" if faces.contains(&6) => {
            let round = battle(c)?.round;
            fighter_mut(c, &e.target_id)?
                .runtime
                .ability
                .counters
                .insert("Quick Draw round".into(), round as i32);
            reorder_start(c)?;
        }
        "extra-highest" => {
            let p = fighter_mut(c, &e.target_id)?;
            p.runtime
                .ability
                .counters
                .retain(|k, _| !k.starts_with("highest:"));
            p.runtime
                .ability
                .counters
                .insert(format!("highest:{choice}"), 1);
        }
        "extra-beast" => {
            let p = fighter(c, &e.target_id)?.clone();
            if count(&p, "Beast Boost total") >= 3 {
                return Err(error("Beast Boost already reached its three-point limit"));
            }
            boost(
                c,
                &e.target_id,
                choice,
                count(&p, &format!("boost:{choice}")) + 1,
            )?;
            *fighter_mut(c, &e.target_id)?
                .runtime
                .ability
                .counters
                .entry("Beast Boost total".into())
                .or_default() += 1;
        }
        "extra-download" => {
            gm(c, actor)?;
            boost(c, &e.target_id, choice, 1)?;
        }
        "extra-no-guard" => {
            if choice == "activate" {
                let round = battle(c)?.round;
                fighter_mut(c, &e.target_id)?
                    .runtime
                    .ability
                    .counters
                    .insert("No Guard round".into(), round as i32);
            }
        }
        "extra-hospitality" => {
            let p = fighter_mut(c, &e.combatant_id)?;
            if p.runtime.will < 1 {
                return Err(error("Hospitality needs one Will"));
            }
            p.runtime.will -= 1;
            if special::enabled(c) {
                special::heal(c, choice, 3, false)?;
            } else {
                let p = fighter_mut(c, choice)?;
                if p.hp > 0 {
                    p.hp = (p.hp + 3).min(p.max_hp);
                }
            }
        }
        "extra-disable" if faces.contains(&6) => {
            abilities::inflict(
                c,
                &e.combatant_id,
                &e.target_id,
                "Disable",
                &e.move_name,
                None,
                cat,
            )?;
        }
        "extra-poison" if faces.contains(&6) => {
            abilities::inflict(
                c,
                &e.combatant_id,
                &e.target_id,
                if e.ability == "Toxic Chain" {
                    "Badly Poisoned"
                } else {
                    "Poison"
                },
                "",
                None,
                cat,
            )?;
        }
        "extra-cure" | "extra-heal-chance" if faces.contains(&6) => {
            fighter_mut(c, &e.target_id)?.runtime.statuses.clear();
        }
        "extra-healer" => {
            let p = fighter(c, &e.combatant_id)?.clone();
            enqueue(
                c,
                &p,
                choice,
                "extra-heal-chance",
                "round end",
                vec!["roll".into()],
            );
        }
        "extra-shed" if faces.contains(&6) => {
            let p = fighter(c, &e.combatant_id)?.clone();
            enqueue(
                c,
                &p,
                &e.target_id,
                "extra-shed-select",
                "random condition",
                vec!["roll".into()],
            );
        }
        "extra-shed-select" => {
            let n = fighter(c, &e.target_id)?.runtime.statuses.len();
            if n > 0 {
                let value = faces
                    .iter()
                    .fold(0usize, |acc, d| acc * 6 + (*d as usize - 1));
                let space = 6usize
                    .checked_pow(faces.len() as u32)
                    .ok_or_else(|| error("Condition selector is too large"))?;
                if value < space.checked_div(n).unwrap_or(0) * n {
                    fighter_mut(c, &e.target_id)?
                        .runtime
                        .statuses
                        .remove(value % n);
                } else {
                    let p = fighter(c, &e.combatant_id)?.clone();
                    enqueue(
                        c,
                        &p,
                        &e.target_id,
                        "extra-shed-select",
                        "reroll unbiased selection",
                        vec!["roll".into()],
                    );
                }
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn critical_margin(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> i32 {
    if special::enabled(c) && m["Attributes"]["AlwaysCrit"] == true {
        return -1000;
    }
    let base = if m["Attributes"]["HighCritical"] == true {
        2
    } else {
        3
    };
    base - i32::from(enabled(c) && p.selected_ability == "Super Luck")
}
pub fn status_damage_immune(p: &Combatant, kind: &str) -> bool {
    (p.selected_ability == "Poison Heal" && kind.contains("Poison"))
        || (["Heatproof", "Water Bubble"].contains(&p.selected_ability.as_str())
            && ["Burn", "Burn 2"].contains(&kind))
}
pub fn ignores_ghost(c: &Campaign, p: &Combatant, move_type: &str, target_type: &str) -> bool {
    enabled(c)
        && p.selected_ability == "Scrappy"
        && target_type == "Ghost"
        && ["Normal", "Fight"].contains(&move_type)
}
pub fn accuracy_bonus(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> i32 {
    if enabled(c)
        && p.selected_ability == "Compound Eyes"
        && m["Attributes"]["AccuracyReduction"]
            .as_i64()
            .is_some_and(|n| n < 0)
    {
        2
    } else {
        0
    }
}
pub fn no_guard(c: &Campaign, p: &Combatant) -> bool {
    enabled(c)
        && p.selected_ability == "No Guard"
        && count(p, "No Guard round") == battle(c).map_or(0, |b| b.round as i32)
}
pub fn reaction(
    c: &Campaign,
    p: &Combatant,
    kind: &str,
    m: Option<&serde_json::Value>,
) -> Result<()> {
    if no_guard(c, p)
        && (["evade", "clash"].contains(&kind)
            || m.is_some_and(|m| m["Attributes"]["ShieldMove"] == true))
    {
        return Err(error(
            "No Guard declaration prevents Shields, Evasion and Clash this round",
        ));
    }
    Ok(())
}

pub fn move_effect(c: &mut Campaign, q: &QueuedAction) -> Result<bool> {
    if !enabled(c) {
        return Ok(false);
    }
    let changes: Vec<(&str, i32)> = match q.move_name.as_str() {
        "Growl" => vec![("Strength", -1)],
        "Tail Whip" | "Leer" => vec![("Defense", -1)],
        "Charm" => vec![("Strength", -2)],
        "Swords Dance" => vec![("Strength", 2)],
        "Nasty Plot" => vec![("Special", 2)],
        "Calm Mind" => vec![("Special", 1), ("SpecialDefense", 1)],
        "Iron Defense" => vec![("Defense", 2)],
        "Amnesia" => vec![("SpecialDefense", 2)],
        "Agility" => vec![("Dexterity", 2)],
        _ => vec![],
    };
    if !changes.is_empty() {
        for (key, n) in changes {
            change_stat(c, &q.actor_id, &q.target_id, key, n)?;
        }
        return Ok(true);
    }
    if q.move_name == "Heal Pulse" {
        let p = fighter_mut(c, &q.actor_id)?;
        if p.runtime.will < 1 {
            return Err(error("Heal Pulse needs one Will"));
        }
        p.runtime.will -= 1;
        let n = if p.selected_ability == "Mega Launcher" {
            5
        } else {
            3
        };
        field::move_heal(c, &q.target_id, n)?;
        return Ok(true);
    }
    Ok(false)
}

fn ailment_kind(kind: &str) -> bool {
    kind.starts_with("Burn")
        || kind.contains("Poison")
        || ["Sleep", "Paralysis", "Frozen Solid"].contains(&kind)
}
pub fn effects_immune(c: &Campaign, p: &Combatant, m: &serde_json::Value) -> bool {
    enabled(c)
        && ((p.selected_ability == "Overcoat" && m["Attributes"]["PowderMove"] == true)
            || ([
                "Taunt",
                "Torment",
                "Spite",
                "Attract",
                "Disable",
                "Encore",
                "Heal Block",
            ]
            .contains(&m["Name"].as_str().unwrap_or(""))
                && field::aura(c, p, "Aroma Veil", true)))
}
pub fn blocked(c: &Campaign, p: &Combatant, cat: &Catalog) -> bool {
    let ts = field::types(c, p, cat).unwrap_or_default();
    if p.selected_ability == "Run Away"
        || (field::has(c, "cemetery", p) && !ts.contains(&"Ghost".into()))
    {
        return false;
    }
    if field::has(c, "mud", p) && field::grounded(c, p, cat) {
        return true;
    }
    living(c).any(|q| {
        q.id != p.id && {
            let mut ids: Vec<_> = living(c)
                .filter(|t| {
                    t.id != q.id
                        && match q.selected_ability.as_str() {
                            "Shadow Tag" => {
                                !field::types(c, t, cat)
                                    .unwrap_or_default()
                                    .contains(&"Ghost".into())
                                    && t.selected_ability != "Shadow Tag"
                            }
                            "Arena Trap" => q.side != t.side && field::grounded(c, t, cat),
                            "Magnet Pull" => field::types(c, t, cat)
                                .unwrap_or_default()
                                .contains(&"Steel".into()),
                            _ => false,
                        }
                        && flow(c).is_ok_and(|f| {
                            f.ranges.get(&q.id).is_none_or(|ids| ids.contains(&t.id))
                        })
                })
                .map(|t| t.id.as_str())
                .collect();
            ids.sort();
            ids.truncate(
                [1, 2, 3, 4, 5, 6, 8, 10]
                    [crate::creation::rank(&c.sheets[&q.sheet_id].rank).unwrap_or(0)],
            );
            ids.contains(&p.id.as_str())
                && match q.selected_ability.as_str() {
                    "Shadow Tag" => {
                        !ts.contains(&"Ghost".into()) && p.selected_ability != "Shadow Tag"
                    }
                    "Arena Trap" => q.side != p.side && field::grounded(c, p, cat),
                    "Magnet Pull" => ts.contains(&"Steel".into()),
                    _ => false,
                }
        }
    })
}

pub fn change_stat(c: &mut Campaign, source: &str, id: &str, key: &str, n: i32) -> Result<()> {
    apply_stat(c, source, id, key, n, false)
}
fn highest(c: &Campaign, p: &Combatant) -> Vec<String> {
    let mut scores: Vec<_> = ["Strength", "Dexterity", "Vitality", "Special", "Insight"]
        .into_iter()
        .map(|key| {
            let base = *p
                .stat_overrides
                .get(key)
                .or(c.sheets[&p.sheet_id].stats.get(key))
                .unwrap_or(&0);
            (
                key.to_string(),
                base + p.runtime.modifiers.get(key).copied().unwrap_or(0),
            )
        })
        .collect();
    let max = scores.iter().map(|(_, n)| *n).max().unwrap_or(0);
    scores.retain(|(_, n)| *n == max);
    scores.into_iter().map(|(k, _)| k).collect()
}
fn highest_active(c: &Campaign, p: &Combatant) -> bool {
    (p.selected_ability == "Protosynthesis"
        && matches!(
            field::weather(c, Some(p)),
            Some(field::Weather::Sunny | field::Weather::Desolate)
        ))
        || (p.selected_ability == "Quark Drive" && field::has(c, "Electric Terrain", p))
}
pub fn order_group(c: &Campaign, p: &Combatant) -> i32 {
    if !enabled(c) {
        return 0;
    }
    if tactics::enabled(c) && p.selected_ability == "Stall" {
        return 2;
    }
    if count(p, "Quick Draw round") == battle(c).map_or(0, |b| b.round as i32) {
        return -1;
    }
    if p.selected_ability == "Slow Start" {
        if battle(c).map_or(0, |b| b.round as i32) - count(p, "entry round") >= 3 {
            -1
        } else {
            1
        }
    } else {
        0
    }
}
pub fn order(c: &Campaign, order: &mut Vec<&Combatant>) {
    if enabled(c) {
        order.sort_by_key(|p| order_group(c, p));
    }
}
fn reorder_start(c: &mut Campaign) -> Result<()> {
    let mut ids = flow(c)?.order.clone();
    ids.sort_by_key(|id| {
        let p = fighter(c, id).unwrap();
        (
            order_group(c, p),
            -p.initiative,
            -crate::creation::profile_stats(&rules::effective_sheet(c, p).unwrap())["Initiative"],
            id.clone(),
        )
    });
    let f = flow_mut(c);
    f.order = ids;
    f.cursor = 0;
    Ok(())
}

fn selection_dice(n: usize) -> u32 {
    let (mut size, mut dice) = (6usize, 1u32);
    while size < n {
        size = size.saturating_mul(6);
        dice += 1;
    }
    dice
}

fn global_aura(c: &Campaign, target: &Combatant, name: &str, include_self: bool) -> bool {
    living(c)
        .filter(|p| p.selected_ability == name)
        .any(|source| reaches(c, source, target, include_self))
}
fn reaches(c: &Campaign, source: &Combatant, target: &Combatant, include_self: bool) -> bool {
    let mut ids: Vec<_> = living(c)
        .filter(|p| {
            (include_self || p.id != source.id)
                && flow(c).is_ok_and(|f| {
                    f.ranges
                        .get(&source.id)
                        .is_none_or(|ids| p.id == source.id || ids.contains(&p.id))
                })
        })
        .map(|p| p.id.as_str())
        .collect();
    ids.sort_by_key(|id| (*id != source.id.as_str(), *id));
    ids.truncate(
        [1, 2, 3, 4, 5, 6, 8, 10]
            [crate::creation::rank(&c.sheets[&source.sheet_id].rank).unwrap_or(0)],
    );
    ids.contains(&target.id.as_str())
}
