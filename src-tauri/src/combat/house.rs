//! Private-group interpretations selected explicitly on 26 October 2026.
use super::*;

pub fn enabled(c: &Campaign) -> bool {
    c.battle
        .as_ref()
        .and_then(|b| b.flow.as_ref())
        .is_some_and(|f| f.house_rules)
}
pub fn ring(c: &Campaign, p: &Combatant) -> bool {
    enabled(c) && items::completion::live(c, &p.id, "Ring Target")
}

/// Expire an existing Coat after its user's next paid action or new move attempt.
/// A round/turn change alone is not an action. Casting a replacement Coat renews it.
pub fn lifecycle(before: &Campaign, next: &mut Campaign) {
    if !enabled(next) {
        return;
    }
    let (Some(old), Some(new)) = (&before.battle, &next.battle) else {
        return;
    };
    let (Some(old_flow), Some(new_flow)) = (&old.flow, &new.flow) else {
        return;
    };
    let expired: BTreeSet<_> = old_flow
        .tactics
        .lures
        .iter()
        .filter(|l| l.name == "Magic Coat")
        .filter_map(|l| {
            let a = old.combatants.iter().find(|p| p.id == l.target)?;
            let b = new.combatants.iter().find(|p| p.id == l.target)?;
            let new_move = new_flow.queue.iter().any(|q| {
                q.actor_id == l.target
                    && !old_flow
                        .queue
                        .iter()
                        .any(|old| old.sequence == q.sequence && old.actor_id == q.actor_id)
            });
            let renewing = new_flow
                .tactics
                .lures
                .iter()
                .any(|n| n.name == "Magic Coat" && n.target == l.target)
                && new_flow.queue.iter().any(|q| {
                    q.actor_id == l.target
                        && q.move_name == "Magic Coat"
                        && q.damage_done
                        && !old_flow.queue.iter().any(|old| {
                            old.sequence == q.sequence
                                && old.actor_id == q.actor_id
                                && old.damage_done
                        })
                });
            ((b.actions > a.actions || new_move || !b.deployed || b.hp == 0) && !renewing)
                .then(|| l.target.clone())
        })
        .collect();
    flow_mut(next)
        .tactics
        .lures
        .retain(|l| l.name != "Magic Coat" || !expired.contains(&l.target));
}

pub fn attacked(c: &mut Campaign, source: &str, targets: &[String]) {
    if !enabled(c) {
        return;
    }
    for id in targets {
        if id != source {
            if let Some(b) = flow_mut(c).special.bides.get_mut(id) {
                b.target_id = source.into();
            }
        }
    }
}
