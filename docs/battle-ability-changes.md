# Temporary abilities — 0.18.0

Engine `0.11.0-core`, protocol 19, user schema 4. Old engines remain readable and unchanged; the GM adopts the new rules and catalog between encounters. Matching client builds are required. No user database migration or permanent sheet change is introduced.

Reviewed sources are POKEDEX MASTER printed pp.480 (Worry Seed), 500 (Simple Beam), 507 (Gastro Acid), 515 (Role Play and Skill Swap), 534 (general ability rules), 557 (Neutralizing Gas) and 575 (Trace). Unique status uses the exact catalog definition's explicit Unique Ability marker; unresolved references require a ruling.

| Effect | Scope and lifetime |
| --- | --- |
| Worry Seed | Target becomes Insomnia for the scene, including through switching. |
| Simple Beam | Target becomes Simple for the scene, including through switching. |
| Gastro Acid | Negates the target's ability for the scene. |
| Role Play | Copies the target's ability for the scene; unique abilities cannot be copied or replaced. |
| Skill Swap | Exchanges user/target abilities for the scene, explicitly surviving switching. Unique abilities cannot be switched. |
| Trace | An entry event uses host dice to select an eligible random foe. Unique abilities are excluded; selection is unbiased using base-six rejection sampling. The owner or GM resolves the event, without spending an action. The copy ends on departure or fainting, restoring any underlying scene-long assignment before the original ability. |
| Neutralizing Gas | Negates foes in the GM-confirmed abstract range, with the rank target cap and stable target ordering. Allies are unaffected; unique abilities prevail. Gas ends when its source leaves, faints or loses the ability, and affected abilities resume. |

Opposing Gas sources create an explicit GM-only stalemate decision. The supported choice keeps both emitters active and negates their applicable foe effects. The decision and reason are recorded; no video-game precedence is silently chosen. Other resolutions use existing GM adjudication and table controls.

Negation disables effects without erasing ability identity: copying an affected foe copies its assigned ability, not its negation.

The encounter stores original abilities, scene assignments, persistent negation, current suppression, Trace copies and provenance. `selected_ability` is the materialized effective battle ability (empty when negated), so existing damage, immunity, reaction, status, weather and stat hooks all read the same value. Original sheet abilities and available abilities are unchanged. Huge/Pure Power subtract the permanent sheet bonus before applying the current battle ability's bonus, preventing duplicate or retained boosts.

Gaining a supported entry ability activates its existing hooks. Losing it removes its ability-generated Comatose sleep, ongoing aura/weather and obsolete pending ability events; actual ability-caused stat changes retain the existing channel rules. Recalculating an ability never repeats an arena entry hazard. Confirmed initiative turns remain preserved. Attack damage contexts retain frozen ability inputs for their shared calculation sequence.

The Abilities panel displays current abilities and temporary history; action previews explain the active scope. Normal move accuracy, target reactions, action costs, decoys, reflection and GM adjudication still apply. Reconnect and rollback serialize the complete state. Command receipts remain host-authoritative and idempotent.

This milestone does not implement Mummy, Lingering Aroma, Receiver, Power of Alchemy, Wandering Spirit, Entrainment, Doodle, Core Enforcer, Mold Breaker bypass, held-item interactions or transformations. Their existing registry boundaries remain visible. Dynamax, Max Moves (including Max Guard), Z-Moves, trainer-target damage, lethal wounds and automatic Will recovery remain out of scope.

Run focused tests with `cargo test --manifest-path src-tauri/Cargo.toml --lib ability_changes_`. `npm run test:3v3` includes both the full twelve-Pokémon gym replay and the strict Ralts/Trace four-installation variant with owner permissions, duplicate receipts, rollback, reconnect and database reopening. No GUI executable is launched by these tests. Physical LAN and interactive native use require separate verification.

Validation: 324 Rust tests, 25 frontend tests and 12 catalog checks passed; Clippy all targets passed with warnings denied. The full suite includes the complete gym replay and the Trace four-installation variant. Native interactive execution and physical LAN have not been exercised for this release.
