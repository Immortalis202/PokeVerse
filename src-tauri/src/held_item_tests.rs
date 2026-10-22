use super::*;
use crate::combat::items;

fn stocked() -> (Sheet, Sheet) {
    let t = trainer("Rookie");
    let p = pokemon(&t, "Pikachu");
    let mut req = request(&p, "inventory", Some(&t));
    req.sheet.inventory = BTreeMap::from([
        ("Oran Berry".into(), 2),
        ("Pecha Berry".into(), 1),
        ("Chesto Berry".into(), 1),
        ("Leftovers".into(), 1),
    ]);
    let p = rules::apply(
        Some(&p),
        Some(&t),
        &req,
        &catalog(),
        Some(("Approved test inventory", "gm")),
    )
    .unwrap();
    (t, p)
}

#[test]
fn items_workflow_equipment_conserves_inventory_and_records_unit_identity() {
    let (t, p) = stocked();
    let mut req = request(&p, "equip", Some(&t));
    req.move_name = "Oran Berry".into();
    let equipped = rules::apply(Some(&p), Some(&t), &req, &catalog(), None).unwrap();
    let unit = &equipped.held_items.as_ref().unwrap()[0];
    assert_eq!(unit.id, req.operation_id);
    assert_eq!(unit.origin_sheet_id, p.id);
    assert_eq!(equipped.inventory["Oran Berry"], 1);
    assert_eq!(
        equipped.workflow.as_ref().unwrap().ledger.len(),
        p.workflow.as_ref().unwrap().ledger.len() + 1
    );
    let mut unequip = request(&equipped, "unequip", Some(&t));
    unequip.move_name = unit.id.clone();
    let stored = rules::apply(Some(&equipped), Some(&t), &unequip, &catalog(), None).unwrap();
    assert!(stored.held_items.is_none());
    assert_eq!(stored.stored_items.as_ref().unwrap()[0], *unit);
    assert_eq!(stored.inventory["Oran Berry"], 1);
    let mut again = request(&stored, "equip", Some(&t));
    again.move_name = unit.id.clone();
    let again = rules::apply(Some(&stored), Some(&t), &again, &catalog(), None).unwrap();
    assert_eq!(again.held_items.as_ref().unwrap()[0], *unit);
    assert!(again.stored_items.is_none());
    assert_eq!(again.inventory["Oran Berry"], 1);
}

#[test]
fn items_equipment_rejects_unowned_forged_and_over_capacity_items_atomically() {
    let (t, p) = stocked();
    let mut req = request(&p, "equip", Some(&t));
    req.move_name = "Sitrus Berry".into();
    req.sheet.inventory.insert("Sitrus Berry".into(), 999);
    assert!(rules::apply(Some(&p), Some(&t), &req, &catalog(), None).is_err());
    let equipped = items::equip(&p, "Oran Berry", "one", &catalog()).unwrap();
    assert!(items::equip(&equipped, "Pecha Berry", "two", &catalog()).is_err());
    assert_eq!(p.inventory["Oran Berry"], 2);
    let mut forged = equipped.clone();
    forged.stored_items = equipped.held_items.clone();
    assert!(items::validate_equipment(&forged, &catalog()).is_err());
    forged.stored_items = None;
    forged.held_items.as_mut().unwrap()[0].item_id = "not-a-catalog-item".into();
    assert!(items::validate_equipment(&forged, &catalog()).is_err());
}

#[test]
fn items_unequip_and_reequip_preserve_partially_used_units() {
    let (_, p) = stocked();
    let mut equipped = items::equip(&p, "Leftovers", "used-leftovers", &catalog()).unwrap();
    equipped.held_items.as_mut().unwrap()[0].uses = 2;
    let stored = items::unequip(&equipped, "used-leftovers", &catalog()).unwrap();
    let again = items::equip(
        &stored,
        "used-leftovers",
        "does-not-create-a-new-unit",
        &catalog(),
    )
    .unwrap();
    assert_eq!(again.held_items, equipped.held_items);
    assert_eq!(again.held_items.unwrap()[0].uses, 2);
    assert!(!again.inventory.contains_key("Leftovers"));
}

#[test]
fn items_gluttony_allows_three_distinct_berries_and_rejects_duplicate_effects() {
    let (_, mut p) = stocked();
    p.ability = "Gluttony".into();
    let first = items::equip(&p, "Oran Berry", "heal", &catalog()).unwrap();
    assert!(items::equip(&first, "Oran Berry", "same-effect", &catalog()).is_err());
    let second = items::equip(&first, "Pecha Berry", "poison", &catalog()).unwrap();
    let third = items::equip(&second, "Chesto Berry", "sleep", &catalog()).unwrap();
    assert_eq!(third.held_items.as_ref().unwrap().len(), 3);
    assert!(items::equip(&third, "Leftovers", "fourth", &catalog()).is_err());
    assert!(items::equip(&first, "Leftovers", "not-a-berry", &catalog()).is_err());
}

#[test]
fn items_old_sheets_keep_their_legacy_serialization_and_fingerprint() {
    let (_, p) = stocked();
    let value = serde_json::to_value(&p).unwrap();
    assert!(!value.as_object().unwrap().contains_key("held_items"));
    assert!(!value.as_object().unwrap().contains_key("stored_items"));
    let loaded: Sheet = serde_json::from_value(value).unwrap();
    assert_eq!(
        rules::fingerprint(&loaded).unwrap(),
        rules::fingerprint(&p).unwrap()
    );
    assert!(loaded.held_items.is_none() && loaded.stored_items.is_none());
}

#[test]
fn items_equipment_and_inventory_are_one_reconciliation_group() {
    let (_, p) = stocked();
    let local = items::equip(&p, "Oran Berry", "local-unit", &catalog()).unwrap();
    let remote = items::equip(&p, "Pecha Berry", "remote-unit", &catalog()).unwrap();
    let error = sync::merge(&p, &local, &remote, &BTreeMap::new()).unwrap_err();
    assert!(error.conflicts.contains(&"/progression".into()));
    let selected = sync::merge(
        &p,
        &local,
        &remote,
        &BTreeMap::from([("/progression".into(), "local".into())]),
    )
    .unwrap();
    assert_eq!(selected.held_items, local.held_items);
    assert_eq!(selected.inventory, local.inventory);
    assert_eq!(selected.inventory["Pecha Berry"], 1);
}
