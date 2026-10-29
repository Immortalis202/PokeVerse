//! M2-B: reviewed single-pool status chances. Source prose, metadata and the
//! printed exceptions are preserved in rules/chance-status-moves.json.
use super::*;

#[derive(Deserialize)]
pub struct Chance {
    pub status: String,
    pub dice: u32,
}
pub fn definition(c: &Campaign, name: &str) -> Option<&'static Chance> {
    if !flow(c).is_ok_and(|f| f.chance_status) {
        return None;
    }
    static DATA: std::sync::OnceLock<BTreeMap<String, Chance>> = std::sync::OnceLock::new();
    DATA.get_or_init(|| {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../../../rules/chance-status-moves.json")).unwrap();
        serde_json::from_value(data["moves"].clone()).unwrap()
    })
    .get(name)
}
pub fn native_flinch(c: &Campaign, name: &str) -> Option<(&'static str, u32)> {
    definition(c, name)
        .filter(|e| e.status == "Flinch")
        .map(|e| ("Flinch", e.dice))
}
fn recoil_dice(c: &Campaign, q: &QueuedAction, cat: &Catalog) -> Result<u32> {
    // Wild Charge's original special handler owns recoil, including Rock Head
    // and Magic Guard. It has no historical native status Chance pool.
    if q.move_name == "Wild Charge" {
        Ok(special::effect_dice(c, q, cat)?.unwrap_or(0))
    } else {
        Ok(0)
    }
}
pub fn effect_dice(c: &Campaign, q: &QueuedAction, cat: &Catalog) -> Result<Option<u32>> {
    let Some(effect) = definition(c, &q.move_name) else {
        return Ok(None);
    };
    let recoil = recoil_dice(c, q, cat)?;
    if q.ice || (provenance::enabled(c) && q.sheer_force) {
        return Ok(Some(recoil));
    }
    let source = reviewed::source(c, q)?;
    let target = fighter(c, &q.target_id)?;
    let bypass = source.side != target.side && reviewed::extreme(c, &source);
    let n = effect.dice
        + items::completion::loaded(c, &q.actor_id)
        + 2 * u32::from(source.has_ability("Serene Grace"))
        + u32::from(
            effect.status == "Frozen Solid"
                && matches!(
                    field::weather(c, None),
                    Some(field::Weather::Snow | field::Weather::Hail)
                ),
        );
    Ok(Some(
        recoil + n.saturating_sub(2 * u32::from(!bypass && target.has_ability("Wonder Skin"))),
    ))
}
pub fn effect(c: &mut Campaign, q: &QueuedAction, dice: &[u8], cat: &Catalog) -> Result<bool> {
    let Some(effect) = definition(c, &q.move_name) else {
        return Ok(false);
    };
    let recoil = recoil_dice(c, q, cat)? as usize;
    if q.move_name == "Wild Charge" {
        special::effects(c, q, &dice[..recoil], cat)?;
    }
    if !q.ice && !(provenance::enabled(c) && q.sheer_force) {
        interactions::apply_status_chance(c, q, &dice[recoil..], &effect.status, cat)?;
    }
    Ok(true)
}
