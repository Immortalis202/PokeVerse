//! Guaranteed conditions from the 22 M2-A cards; printed references are in
//! rules/move-status-print-review.json. Accuracy and per-target expansion remain
//! in the ordinary action pipeline; these effects never roll Chance dice.
use super::*;

pub fn kind(name: &str) -> Option<&'static str> {
    Some(match name {
        "Alluring Voice" | "Confuse Ray" | "Dynamic Punch" | "Supersonic" | "Sweet Kiss"
        | "Teeter Dance" => "Confusion",
        "Attract" => "In Love",
        "Dark Void" | "Grass Whistle" | "Sing" | "Sleep Powder" | "Spore" => "Sleep",
        "Glare" | "Nuzzle" | "Stun Spore" | "Thunder Wave" | "Zap Cannon" => "Paralysis",
        "Poison Gas" | "Poison Powder" => "Poison",
        "Toxic" => "Badly Poisoned",
        "Inferno" => "Burn 3",
        "Will-O-Wisp" => "Burn",
        _ => return None,
    })
}

pub fn effect(c: &mut Campaign, q: &QueuedAction, cat: &Catalog) -> Result<bool> {
    let Some(kind) = kind(&q.move_name) else {
        return Ok(false);
    };
    if !flow(c)?.direct_status {
        return Ok(false);
    }
    if completion::sport_effect_blocked(c, q, cat)? {
        return Ok(true);
    }
    let m = move_entry(cat, &q.move_name)?;
    if special::target_effects_blocked(c, q, &m)
        || (!reviewed::extreme(c, &reviewed::source(c, q)?)
            && abilities::move_immune(c, fighter(c, &q.target_id)?, &m))
    {
        return Ok(true);
    }
    let detail = if kind == "In Love" { &q.actor_id } else { "" };
    abilities::inflict(c, &q.actor_id, &q.target_id, kind, detail, None, cat)?;
    Ok(true)
}
