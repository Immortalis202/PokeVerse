use super::*;

#[test]
#[ignore = "explicit diagnostic inventory; writes only when POKEVERSE_MOVE_AUDIT_OUTPUT is set"]
fn move_effect_route_inventory() {
    let cat = catalog();
    let (base, a, t) = special_tests::setup("Tackle");
    let queued = targeting_tests::pass(&hit(&base, &a, &t, "Tackle"));
    let template = queued.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].clone();
    let folder =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Pokerole-Data/v3.0/Moves");
    let mut files: Vec<_> = std::fs::read_dir(folder)
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    let mut rows = vec![];
    for path in files {
        let source: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let name = source["Name"].as_str().unwrap();
        let m = cat.find("moves", "Name", name).unwrap();
        let mut scenarios = serde_json::Map::new();
        for ailments in [false, true] {
            let mut c = base.clone();
            let b = c.battle.as_mut().unwrap();
            let f = b.flow.as_mut().unwrap();
            f.spite = true;
            f.status_escalation = true;
            f.will_drains = true;
            f.identity_moves = true;
            f.house_rules = true;
            f.interaction_rules = true;
            f.reviewed_rules = true;
            f.provenance_rules = true;
            f.tactics.enabled = true;
            f.decoys.enabled = true;
            f.items.enabled = true;
            f.items.completion.enabled = true;
            f.events.enabled = true;
            f.transfers.enabled = true;
            f.standalone.enabled = true;
            f.information.enabled = true;
            f.forms.enabled = true;
            f.ability_changes.enabled = true;
            f.stat_channels.enabled = true;
            f.sequencing.enabled = true;
            f.queue.clear();
            f.window = false;
            for p in &mut b.combatants {
                p.hp = 10;
                p.max_hp = 20;
                p.selected_ability.clear();
                if ailments {
                    p.runtime.statuses = ["Burn 2", "Poison", "Paralysis", "Sleep"]
                        .map(|kind| crate::combat::Status {
                            kind: kind.into(),
                            detail: "Audit fixture".into(),
                            rounds: None,
                            progress: 0,
                            elapsed_rounds: 0,
                        })
                        .to_vec();
                }
            }
            if m["Target"].as_str().is_some_and(|s| s.contains("Ally")) {
                b.combatants[1].side = b.combatants[0].side.clone();
            }
            sequencing_tests::loyal(&mut c, &t, 3);
            let mut q = template.clone();
            q.move_name = name.into();
            q.target_id = if m["Target"] == "Self" {
                a.clone()
            } else {
                t.clone()
            };
            q.targets.clear();
            q.damage_done = true;
            q.damage = 2;
            q.raw_damage_successes = 2;
            c.battle
                .as_mut()
                .unwrap()
                .flow
                .as_mut()
                .unwrap()
                .queue
                .push(q.clone());
            let result = crate::combat::audit_effect_route(&mut c, &q, &cat);
            let outcome = match result {
                Ok(()) => {
                    let statuses = |id: &str| {
                        c.battle
                            .as_ref()
                            .unwrap()
                            .combatants
                            .iter()
                            .find(|p| p.id == id)
                            .unwrap()
                            .runtime
                            .statuses
                            .iter()
                            .map(|s| s.kind.clone())
                            .collect::<Vec<_>>()
                    };
                    serde_json::json!({"result":"route_resolved", "user_statuses":statuses(&a), "target_statuses":statuses(&q.target_id)})
                }
                Err(e) => {
                    serde_json::json!({"result":"route_error","code":e.code,"message":e.message})
                }
            };
            scenarios.insert(
                if ailments { "ailments" } else { "baseline" }.into(),
                outcome,
            );
        }
        rows.push(serde_json::json!({"name":name,"scenarios":scenarios,
            "catalog_effect":m["Effect"], "catalog_target":m["Target"],
            "catalog_attributes":m["Attributes"]}));
    }
    assert_eq!(rows.len(), 892);
    let missing = |name: &str| {
        rows.iter().find(|r| r["name"] == name).unwrap()["scenarios"]["baseline"]["message"]
            .as_str()
            .is_some_and(|m| m.contains("no reviewed executable definition"))
    };
    assert!(missing("Confuse Ray"));
    assert!(missing("Thunder Wave"));
    assert!(!missing("Air Slash"));
    let row = |name: &str| rows.iter().find(|r| r["name"] == name).unwrap();
    assert_eq!(
        row("Wild Charge")["scenarios"]["baseline"]["result"],
        "route_resolved"
    );
    assert_eq!(
        row("Wild Charge")["scenarios"]["baseline"]["target_statuses"],
        serde_json::json!([])
    );
    assert!(missing("Sparkling Aria"));
    assert_eq!(
        row("Sparkling Aria")["scenarios"]["ailments"]["result"],
        "route_resolved"
    );
    assert!(
        !row("Sparkling Aria")["scenarios"]["ailments"]["target_statuses"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s == "Burn 2")
    );
    if let Ok(path) = std::env::var("POKEVERSE_MOVE_AUDIT_OUTPUT") {
        let output = serde_json::json!({"engine":ENGINE,"release":env!("CARGO_PKG_VERSION"),
            "scenario":"baseline and synthetic Burn2/Poison/Paralysis/Sleep profiles; effect phase only; wounded participants; no abilities, shields or decoys; all chance dice six; no GM selections/history; current gates enabled",
            "warning":"A resolved route is not proof that every card clause is implemented. Errors may be legitimate input requirements.","moves":rows});
        std::fs::write(path, serde_json::to_string_pretty(&output).unwrap() + "\n").unwrap();
    }
}
