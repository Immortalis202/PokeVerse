//! Remaining battle event definitions. Outside-battle effects are excluded.
use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct EventState {
    pub enabled: bool,
    pub pressure_sources: BTreeSet<String>,
    pub pressure_targets: BTreeSet<String>,
    pub history: Vec<String>,
}

pub fn enabled(c: &Campaign) -> bool {
    c.battle
        .as_ref()
        .and_then(|b| b.flow.as_ref())
        .is_some_and(|f| f.events.enabled)
}

pub fn lifecycle(c: &mut Campaign) -> Result<()> {
    if !enabled(c) {
        return Ok(());
    }
    if !c.battle.as_ref().is_some_and(|b| b.active) {
        flow_mut(c).events.pressure_sources.clear();
        flow_mut(c).events.pressure_targets.clear();
        return Ok(());
    }
    let living: Vec<_> = battle(c)?
        .combatants
        .iter()
        .filter(|p| p.deployed && p.hp > 0)
        .cloned()
        .collect();
    let sources: Vec<_> = living
        .iter()
        .filter(|p| p.has_ability("Pressure"))
        .collect();
    let previous = flow(c)?.events.pressure_sources.clone();
    let covered: BTreeSet<_> = living
        .iter()
        .filter(|target| {
            sources
                .iter()
                .any(|source| source.side != target.side && reviewed::in_range(c, source, target))
        })
        .map(|p| p.id.clone())
        .collect();
    flow_mut(c)
        .events
        .pressure_targets
        .retain(|id| covered.contains(id));
    for source in sources.iter().filter(|p| !previous.contains(&p.id)) {
        let targets: Vec<_> = living
            .iter()
            .filter(|p| p.side != source.side && reviewed::in_range(c, source, p))
            .collect();
        for target in targets {
            if flow(c)?.events.pressure_targets.contains(&target.id) {
                continue;
            }
            let subject = fighter(c, &target.id)?;
            if !subject.runtime.will_enabled {
                return Err(AppError::new(
                    "manual_rule",
                    "Pressure requires reviewed remaining Will for every affected foe",
                ));
            }
            let spent = (subject.runtime.will + 1) / 2;
            fighter_mut(c, &target.id)?.runtime.will -= spent;
            flow_mut(c)
                .events
                .pressure_targets
                .insert(target.id.clone());
            flow_mut(c).events.history.push(format!(
                "Pressure: {} entered; {} spent {spent} Will without gaining an effect",
                source.name, target.name
            ));
        }
    }
    flow_mut(c).events.pressure_sources = sources.iter().map(|p| p.id.clone()).collect();
    Ok(())
}
