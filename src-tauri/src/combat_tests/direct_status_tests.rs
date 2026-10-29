use super::targeting_tests::{attack, damage_command, pass, roll};
use super::*;

const CARDS: [(&str, &str); 22] = [
    ("Alluring Voice", "Confusion"),
    ("Attract", "In Love"),
    ("Confuse Ray", "Confusion"),
    ("Dark Void", "Sleep"),
    ("Dynamic Punch", "Confusion"),
    ("Glare", "Paralysis"),
    ("Grass Whistle", "Sleep"),
    ("Inferno", "Burn 3"),
    ("Nuzzle", "Paralysis"),
    ("Poison Gas", "Poison"),
    ("Poison Powder", "Poison"),
    ("Sing", "Sleep"),
    ("Sleep Powder", "Sleep"),
    ("Spore", "Sleep"),
    ("Stun Spore", "Paralysis"),
    ("Supersonic", "Confusion"),
    ("Sweet Kiss", "Confusion"),
    ("Teeter Dance", "Confusion"),
    ("Thunder Wave", "Paralysis"),
    ("Toxic", "Badly Poisoned"),
    ("Will-O-Wisp", "Burn"),
    ("Zap Cannon", "Paralysis"),
];

fn setup(name: &str) -> (Campaign, String, String) {
    let (mut c, a, t) = special_tests::setup(name);
    let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
    f.direct_status = true;
    f.status_escalation = true;
    (c, a, t)
}
fn queued(c: &Campaign) -> crate::combat::QueuedAction {
    c.battle.as_ref().unwrap().flow.as_ref().unwrap().queue[0].clone()
}
fn effect(c: &Campaign) -> Action {
    Action::ResolveEffect {
        sequence: queued(c).sequence,
        reason: "Printed direct status".into(),
    }
}
fn pending(c: &Campaign, a: &str, t: &str, name: &str) -> Campaign {
    let mut c = hit(c, a, t, name);
    if c.battle.as_ref().unwrap().flow.as_ref().unwrap().window {
        c = pass(&c);
    }
    if catalog().find("moves", "Name", name).unwrap()["Category"] != "Support" {
        c = roll(&c, "gm", &damage_command(&c), 1);
    }
    c
}
fn status<'a>(c: &'a Campaign, id: &str, kind: &str) -> Option<&'a crate::combat::Status> {
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
        .find(|s| s.kind == kind)
}

#[test]
fn direct_status_all_22_resolve_without_chance_dice_and_attract_records_source() {
    for (name, kind) in CARDS {
        let (c, a, t) = setup(name);
        let c = pending(&c, &a, &t, name);
        let command = effect(&c);
        assert_eq!(
            rules::dice_count(&c, &command, &catalog()).unwrap(),
            0,
            "{name}"
        );
        assert!(battle::transition(&c, "other", &command, vec![], &catalog()).is_err());
        let c = decision(&c, "gm", command);
        let s = status(&c, &t, kind).unwrap_or_else(|| panic!("{name}: {kind}"));
        if name == "Attract" {
            assert_eq!(s.detail, a);
        }
        assert!(status(&c, &a, kind).is_none(), "{name}: source unaffected");
    }
}

#[test]
fn direct_status_burn_repeats_escalate_but_inferno_stays_third_degree() {
    let (c, a, t) = setup("Will-O-Wisp");
    let mut c = pending(&c, &a, &t, "Will-O-Wisp");
    let q = queued(&c);
    for kind in ["Burn", "Burn 2", "Burn 3", "Burn 3"] {
        crate::combat::audit_effect_route(&mut c, &q, &catalog()).unwrap();
        assert!(status(&c, &t, kind).is_some());
    }
    let mut q = q;
    q.move_name = "Inferno".into();
    crate::combat::audit_effect_route(&mut c, &q, &catalog()).unwrap();
    assert!(status(&c, &t, "Burn 3").is_some());
}

#[test]
fn direct_status_old_flow_keeps_manual_fallback() {
    let (c, a, t) = special_tests::setup("Confuse Ray");
    let c = pending(&c, &a, &t, "Confuse Ray");
    let e = battle::transition(&c, "gm", &effect(&c), vec![], &catalog()).unwrap_err();
    assert_eq!(e.code, "manual_rule");
    assert!(status(&c, &t, "Confusion").is_none());
}

#[test]
fn direct_status_immunities_reflection_and_guaranteed_damage_effects() {
    for (name, ability, blocked) in [
        ("Spore", "Overcoat", true),
        ("Sing", "Soundproof", true),
        ("Confuse Ray", "Own Tempo", true),
        ("Attract", "Oblivious", true),
        ("Attract", "Aroma Veil", true),
        ("Spore", "Insomnia", true),
        ("Thunder Wave", "Limber", true),
        ("Toxic", "Immunity", false), // Printed Immunity prevents poison damage, not the ailment.
        ("Will-O-Wisp", "Water Veil", true),
        ("Nuzzle", "Shield Dust", false),
        ("Dynamic Punch", "Shield Dust", false),
        ("Inferno", "Shield Dust", false),
    ] {
        let (mut c, a, t) = setup(name);
        c.battle.as_mut().unwrap().combatants[1].selected_ability = ability.into();
        let c = pending(&c, &a, &t, name);
        let c = decision(&c, "gm", effect(&c));
        assert_eq!(
            status(&c, &t, crate::combat::direct_status::kind(name).unwrap()).is_none(),
            blocked,
            "{name}: {ability}"
        );
    }
    for (name, species) in [
        ("Thunder Wave", "Pikachu"),
        ("Toxic", "Magnemite"),
        ("Inferno", "Charmander"),
    ] {
        let (mut c, a, t) = setup(name);
        let sid = c.battle.as_ref().unwrap().combatants[1].sheet_id.clone();
        c.sheets.get_mut(&sid).unwrap().species_id = Some(
            catalog().find("species", "Name", species).unwrap()["AppId"]
                .as_str()
                .unwrap()
                .into(),
        );
        let c = pending(&c, &a, &t, name);
        let c = decision(&c, "gm", effect(&c));
        assert!(
            c.battle.as_ref().unwrap().combatants[1]
                .runtime
                .statuses
                .is_empty(),
            "{name}: {species}"
        );
    }
    let (mut c, a, t) = setup("Attract");
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .tactics
        .enabled = true;
    c.battle.as_mut().unwrap().combatants[1].selected_ability = "Magic Bounce".into();
    let c = pending(&c, &a, &t, "Attract");
    let c = decision(&c, "gm", effect(&c));
    assert_eq!(status(&c, &a, "In Love").unwrap().detail, t);
    assert!(status(&c, &t, "In Love").is_none());
}

#[test]
fn direct_status_decoy_blocks_but_sound_bypasses_and_protect_blocks() {
    for name in ["Confuse Ray", "Alluring Voice"] {
        let (c, a, t) = setup(name);
        let mut c = pending(&c, &a, &t, name);
        let f = c.battle.as_mut().unwrap().flow.as_mut().unwrap();
        f.decoys.enabled = true;
        f.decoys.active.insert(
            t.clone(),
            crate::combat::decoys::Decoy {
                hp: 5,
                defenses: [2, 2],
                source: t.clone(),
                move_name: "Substitute".into(),
            },
        );
        let c = decision(&c, "gm", effect(&c));
        assert_eq!(
            status(&c, &t, "Confusion").is_some(),
            name == "Alluring Voice"
        );
    }
    let (c, a, t) = setup("Thunder Wave");
    let mut c = pending(&c, &a, &t, "Thunder Wave");
    let round = c.battle.as_ref().unwrap().round;
    c.battle
        .as_mut()
        .unwrap()
        .flow
        .as_mut()
        .unwrap()
        .special
        .shields
        .insert(
            t.clone(),
            crate::combat::special::Shield {
                name: "Protect".into(),
                source: t.clone(),
                round,
                armed_turn: 0,
                available: true,
            },
        );
    let c = decision(&c, "gm", effect(&c));
    assert!(status(&c, &t, "Paralysis").is_none());
}

#[test]
fn direct_status_area_and_all_foes_resolve_each_target() {
    for (name, kind) in [
        ("Dark Void", "Sleep"),
        ("Poison Gas", "Poison"),
        ("Teeter Dance", "Confusion"),
    ] {
        let (mut c, a, t, third) = targeting_tests::multi(name);
        let b = c.battle.as_mut().unwrap();
        let f = b.flow.as_mut().unwrap();
        f.direct_status = true;
        f.ability_rules = true;
        f.tactics.enabled = true;
        if name == "Teeter Dance" {
            b.combatants[2].side = "Players".into();
        }
        let targets = if name == "Teeter Dance" {
            vec![]
        } else {
            vec![t.clone(), third.clone()]
        };
        let mut command = attack(&a, targets, name);
        if let Action::AccuracyTargets { pool_modifier, .. } = &mut command {
            *pool_modifier = 10;
        }
        let mut c = roll(&c, "gm", &command, 6);
        assert_eq!(queued(&c).targets.len(), 2, "{name}");
        for _ in 0..2 {
            c = pass(&c);
        }
        for _ in 0..2 {
            c = decision(&c, "gm", effect(&c));
        }
        assert!(status(&c, &t, kind).is_some(), "{name}");
        assert!(status(&c, &third, kind).is_some(), "{name}");
        assert!(status(&c, &a, kind).is_none());
    }
}
