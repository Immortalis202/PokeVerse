//! MASTER 537,546: private trainer warnings. Candidates never enter public logs.
use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct InformationState {
    pub enabled: bool,
    pub active: BTreeSet<String>,
    pub offers: BTreeMap<String, InformationOffer>,
    pub reveals: Vec<InformationReveal>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct InformationMove {
    pub foe_id: String,
    pub move_name: String,
    pub power: Option<i32>,
    pub category: String,
    pub lethal: bool,
    pub super_effective: bool,
    pub uncertain: bool,
    pub detail: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct InformationOffer {
    pub recipient: String,
    pub candidates: Vec<InformationMove>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct InformationReveal {
    pub recipient: String,
    pub source_id: String,
    pub ability: String,
    pub round: u32,
    pub alert: bool,
    pub decision: String,
    pub move_name: Option<String>,
    pub reason: String,
}
pub fn enabled(c: &Campaign) -> bool {
    c.battle
        .as_ref()
        .and_then(|b| b.flow.as_ref())
        .is_some_and(|f| f.information.enabled)
}
fn recipient(c: &Campaign, source: &Combatant) -> String {
    c.sheets
        .get(&source.sheet_id)
        .and_then(|s| s.trainer_id.as_ref())
        .and_then(|id| c.sheets.get(id))
        .filter(|s| s.kind == "trainer")
        .map_or_else(|| source.owner_id.clone(), |s| s.owner_id.clone())
}
fn candidates(c: &Campaign, source: &Combatant, cat: &Catalog) -> Result<Vec<InformationMove>> {
    let mut result = Vec::new();
    for foe in &battle(c)?.combatants {
        if !foe.deployed || foe.hp == 0 || foe.side == source.side {
            continue;
        }
        let sheet = rules::base_sheet(c, foe)?;
        for name in sheet.moves.iter().collect::<BTreeSet<_>>() {
            let Some(raw) = cat.find("moves", "Name", name) else {
                result.push(InformationMove {
                    foe_id: foe.id.clone(),
                    move_name: name.clone(),
                    power: None,
                    category: "Unresolved".into(),
                    lethal: false,
                    super_effective: false,
                    uncertain: true,
                    detail: "Unresolved move definition; GM classification required".into(),
                });
                continue;
            };
            let m = field::move_data(c, foe, &raw);
            // Knowing a conditional damaging move is distinct from executing it now.
            // In particular Bide/Present can be temporarily projected as Support.
            let category = raw["Category"].as_str().unwrap_or("");
            let copy_move = [
                "Metronome",
                "Assist",
                "Copycat",
                "Mirror Move",
                "Me First",
                "Sleep Talk",
                "Instruct",
            ]
            .contains(&name.as_str());
            let unresolved_type = copy_move || name == "Nature Power";
            if category == "Support" && !copy_move {
                continue;
            }
            let damaging = ["Physical", "Special"].contains(&category);
            let target = m["Target"].as_str().unwrap_or("");
            if !copy_move
                && [
                    "Self",
                    "User",
                    "Ally",
                    "All Allies",
                    "User and Allies",
                    "Ally's Battlefield",
                ]
                .contains(&target)
            {
                continue;
            }
            let mut uncertain = !damaging || unresolved_type;
            let (mut effectiveness, mut immune) = match m["Type"]
                .as_str()
                .map(|t| rules::type_matchup(c, foe, source, t, cat))
            {
                Some(Ok(v)) => v,
                _ => {
                    uncertain = true;
                    (0, false)
                }
            };
            if forms::tera_shell(c, source, &m) {
                effectiveness = -1;
                immune = false;
            }
            if field::weather(c, Some(source)) == Some(field::Weather::StrongWinds)
                && field::types(c, source, cat)?.iter().any(|t| t == "Flying")
            {
                effectiveness = effectiveness.min(0);
            }
            immune |= abilities::damage_immune_from(c, foe, source, &m)
                || extra::source_immune(c, foe, source, effectiveness);
            let lethal = !unresolved_type && m["Attributes"]["Lethal"] == true && !immune;
            let super_effective = !unresolved_type && damaging && effectiveness > 0 && !immune;
            // Printed Power is a comparison aid, never an invented fixed value for a formula.
            let power = m["Power"]
                .as_i64()
                .filter(|p| (1..=100).contains(p))
                .map(|p| p as i32)
                .filter(|_| !copy_move);
            let detail = format!(
                "{}; {} / {}; {}",
                m["Type"].as_str().unwrap_or("unknown type"),
                m["Damage1"].as_str().unwrap_or(""),
                m["Damage2"].as_str().unwrap_or(""),
                m["Effect"]
                    .as_str()
                    .unwrap_or("GM reviews the supplied card")
            );
            result.push(InformationMove {
                foe_id: foe.id.clone(),
                move_name: name.clone(),
                power,
                category: category.into(),
                lethal,
                super_effective,
                uncertain,
                detail,
            });
        }
    }
    Ok(result)
}
// Entry/acquisition windows also refresh when foe rosters/movepools, types or weather change.
// Ordinary HP/action changes do not repeatedly reveal a movepool.
pub fn lifecycle(c: &mut Campaign, cat: &Catalog) -> Result<()> {
    if !enabled(c) || c.battle.as_ref().is_none_or(|b| !b.active) {
        return Ok(());
    }
    let mut live = BTreeSet::new();
    let mut live_sources = BTreeSet::new();
    let sources: Vec<_> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .cloned()
        .collect();
    for source in &sources {
        if !sources.iter().any(|p| p.side != source.side) {
            continue;
        }
        for ability in ["Anticipation", "Forewarn"] {
            if !source.has_ability(ability) {
                continue;
            }
            let key = format!("{}:{ability}", source.id);
            live_sources.insert(key.clone());
            let foes: Vec<_> = sources
                .iter()
                .filter(|p| p.side != source.side)
                .map(|p| {
                    Ok((
                        p.id.clone(),
                        rules::base_sheet(c, p)?.moves,
                        p.ability_names(),
                        field::types(c, p, cat).unwrap_or_default(),
                    ))
                })
                .collect::<Result<_>>()?;
            let window = format!(
                "{key}:{:?}:{:?}:{:?}:{:?}:{}",
                field::types(c, source, cat).unwrap_or_default(),
                source.ability_names(),
                foes,
                field::weather(c, Some(source)),
                recipient(c, source)
            );
            live.insert(window.clone());
            if flow(c)?.information.active.contains(&window) {
                continue;
            }
            let expired: BTreeSet<_> = flow(c)?
                .ability_events
                .iter()
                .filter(|e| {
                    e.kind.starts_with("info-")
                        && e.combatant_id == source.id
                        && e.ability == ability
                })
                .map(|e| e.id.clone())
                .collect();
            flow_mut(c)
                .ability_events
                .retain(|e| !expired.contains(&e.id));
            flow_mut(c)
                .information
                .offers
                .retain(|id, _| !expired.contains(id));
            let candidates = candidates(c, source, cat)?;
            let choices = if ability == "Anticipation" {
                let threat = candidates.iter().any(|m| m.lethal || m.super_effective);
                let uncertain = candidates.iter().any(|m| m.uncertain);
                if threat {
                    vec!["alert".into()]
                } else if uncertain {
                    vec!["alert".into(), "clear".into()]
                } else {
                    vec!["clear".into()]
                }
            } else {
                (0..candidates.len())
                    .map(|i| format!("move:{i}"))
                    .chain(std::iter::once("none".into()))
                    .collect()
            };
            let id = format!(
                "info:{}:{}:{key}",
                c.revision + 1,
                flow(c)?.ability_events.len()
            );
            let offer = InformationOffer {
                recipient: recipient(c, source),
                candidates,
            };
            flow_mut(c).information.offers.insert(id.clone(), offer);
            flow_mut(c).ability_events.push(abilities::AbilityEvent {
                id,
                combatant_id: source.id.clone(),
                target_id: source.id.clone(),
                ability: ability.into(),
                kind: "info-entry".into(),
                move_name: "private trainer warning window".into(),
                choices,
            });
        }
    }
    // Never leave an obsolete decision blocking the encounter after recall/fainting/suppression.
    let stale: BTreeSet<_> = flow(c)?
        .ability_events
        .iter()
        .filter(|e| {
            e.kind.starts_with("info-")
                && !live_sources.contains(&format!("{}:{}", e.combatant_id, e.ability))
        })
        .map(|e| e.id.clone())
        .collect();
    flow_mut(c)
        .ability_events
        .retain(|e| !stale.contains(&e.id));
    flow_mut(c)
        .information
        .offers
        .retain(|id, _| !stale.contains(id));
    flow_mut(c).information.active = live;
    Ok(())
}
pub fn resolve(
    c: &mut Campaign,
    e: &abilities::AbilityEvent,
    choice: &str,
    why: &str,
    cat: &Catalog,
) -> Result<()> {
    if !enabled(c) {
        return Err(error("Private warnings require the new encounter engine"));
    }
    let offer = flow(c)?
        .information
        .offers
        .get(&e.id)
        .cloned()
        .ok_or_else(|| error("Missing private warning"))?;
    if choice != "manual" {
        let source = fighter(c, &e.combatant_id)?;
        if source.hp == 0
            || !source.deployed
            || !source.has_ability(&e.ability)
            || recipient(c, source) != offer.recipient
            || candidates(c, source, cat)? != offer.candidates
        {
            return Err(error("Private warning inputs changed; GM must close the expired window with a recorded ruling"));
        }
    }
    let move_name = if e.ability == "Forewarn" && choice != "manual" && choice != "none" {
        let index = choice
            .strip_prefix("move:")
            .and_then(|s| s.parse::<usize>().ok())
            .ok_or_else(|| error("Choose a recorded private move"))?;
        Some(
            offer
                .candidates
                .get(index)
                .ok_or_else(|| error("Choose a recorded private move"))?
                .move_name
                .clone(),
        )
    } else {
        None
    };
    // Manual/none are private rulings, including failed Insight or no meaningful comparison.
    let round = battle(c)?.round;
    flow_mut(c).information.reveals.push(InformationReveal {
        recipient: offer.recipient,
        source_id: e.combatant_id.clone(),
        ability: e.ability.clone(),
        round,
        decision: choice.into(),
        alert: choice == "alert" || move_name.is_some(),
        move_name,
        reason: why.into(),
    });
    flow_mut(c).information.offers.remove(&e.id);
    Ok(())
}

/// Run before dice counting so guessed private choices cannot be used as an oracle.
pub fn authorize(c: &Campaign, actor: &str, action: &Action) -> Result<()> {
    if matches!(action, Action::ResolveAbility { .. })
        && c.battle
            .as_ref()
            .and_then(|b| b.flow.as_ref())
            .and_then(|f| f.ability_events.first())
            .is_some_and(|e| e.kind.starts_with("info-"))
    {
        gm(c, actor)?;
    }
    Ok(())
}
