# Reviewed ability batch (0.20.0)

Release 0.20.0 uses engine `0.13.0-core`, protocol 21 and database schema 4. Matching builds and explicit campaign rules/catalog adoption between battles remain required. Old encounters retain their recorded behavior through the serialized `reviewed_rules` flag (default false). No permanent sheet or source-data mutation is introduced.

## Reviewed definitions

The supplied MASTER cards, not video-game mechanics, determine these rules. RULING p.80 says Never Miss prevents Evasion; accuracy still needs successes and Clash remains legal. The three selected groups contain **16** definitions.

| Ability | MASTER page | Implemented scope |
| --- | --- | --- |
| Mold Breaker | 556 | Foe damage immunities, including type immunity. Target identity and damage reductions remain intact. Unique Wonder Guard prevails. |
| Teravolt / Turboblaze | 574 / 576 | Foe damage/type/status immunity, supported shields/decoys, move redirection and charging-target access. Unique-effect stalemates require GM adjudication. |
| Mycelium Might | 557 | Support moves get Never Miss and Late Reaction 1. The card grants no ability bypass. Printed Reaction conflicts remain adjudicated. |
| Keen Eye | 552 | Prevent success-removal penalties from move, ability and environment accuracy hooks. Action difficulty, pain and deliberate GM modifiers remain. |
| Mind's Eye | 555 | Never Miss at GM-recorded night; Normal/Fight hit Ghost for regular damage at all times. |
| Tangled Feet | 573 | Confused holder imposes Reduced Accuracy 2 on foe moves. Mixed penalties in one shared multi-target accuracy roll require a GM ruling. |
| Victory Star | 577 | User and living allies in confirmed range gain Never Miss on Physical/Special moves. |
| Early Bird | 544 | Halve structured Sleep duration, rounding up; wake at two accumulated Insight successes. Rest excluded. Narrative sleep remains manual. |
| Justified | 552 | Dark hits raise Strength in the ability channel, capped at three. Narrative injustice remains manual. |
| Motor Drive | 556 | Electric damage immunity and a first-hit Dexterity increase. A bypassed immunity does not grant absorption or its boost. |
| Screen Cleaner | 566 | Entry/effective activation removes supported force fields on both sides. Weather, terrain and rooms remain. |
| Seed Sower | 566 | Physical/Special hits create four-round Grassy Terrain on the holder's side, without refreshing existing Grassy Terrain. Other terrain uses the existing GM conflict queue. |
| Cotton Down | 541 | Non-ranged Physical hits reduce nearby Dexterity in the ability channel, with abstract range, rank caps, foes before allies and attribute protection. |
| Supersweet Syrup | 571 | Foes in confirmed range cannot Evade the holder's moves. Clash remains legal; narrative encounter rates remain manual. |
| Toxic Debris | 575 | Non-ranged Physical hits scatter Toxic Spikes on the attacker's side. Eligible new entrants roll one host Chance die: six causes Badly Poisoned, otherwise Poison. Ground immunity and status protection apply. |

Toxic Spikes follows its printed move card: no invented video-game layer stacking or Poison-type hazard absorption. The hazard persists until removed, and temporary ability refresh does not trigger a new entry. Hail/Snow and existing once-per-turn Evasion rules remain unchanged.

## Authority and state

All calculations and checks remain Rust-owned. Bypass is evaluated from the attacking identity and frozen shared-damage context; it does not negate or replace the target's ability. Neutralizing Gas and temporary copying therefore affect whether a bypass source is active, while committed shared damage retains its snapshot. Reactive contact abilities continue unless the printed rule actually grants immunity to that hit.

Night is an explicit GM battlefield command, independently recorded from sunlight and indoor permissions, visible to players and preserved in checkpoints/reconnects. Ability-channel changes and field effects are inspectable in battlefield history; previews state the relevant scope. Existing receipts, revision checks and transactions handle retries, atomic rejection and rollback.

## Explicit boundaries

Teravolt/Turboblaze are partial: held items, unsupported Support effects, unscripted blockers and conflicting unique effects remain GM-adjudicated. Bypass does not invent a move definition or waive ownership, fainting, action budgets, rank caps, move prerequisites or GM-confirmed range. Mixed Tangled Feet accuracy and Mycelium Might printed-priority collisions are rejected visibly rather than silently approximated.

Held items, transformations, outside-battle activities and the other unreviewed ability families remain open. Dynamax, Max Moves, Z-Moves, trainer-target damage, lethal wounds and automatic Will recovery remain out of scope.

The current registry contains 305 definitions: **188 automated, 60 partial and 57 manual**. A partial entry describes its exact remaining dependency.

## Validation

The focused Rust fixtures cover all sixteen definitions, historical defaults, Gas/copy identity, frozen shared damage, resource preservation, required entry checks and permission failures. The new four-installation headless 3v3 case verifies a strict Cranidos/Gastly encounter, separate host/GM/owner authority, GM-only night configuration, a Normal hit through Ghost immunity, retries, rollback, reconnect and host database reopen.

Native interactive use, install/upgrade/uninstall and physical multi-machine LAN verification are not automated by this milestone. The executable and installer are not launched during packaging.


### Final checks (20 October 2026)

- 377 Rust tests passed, including five headless 3v3 scenarios and real localhost host/GM/owner synchronization.
- 29 frontend tests and 12 catalog/browser-preview checks passed.
- TypeScript, production Vite build, warnings-denied all-target Clippy, formatting and Git whitespace checks passed.
- Cotton Down is checked against effective Dexterity, and Tidy Up/Defog remove the new Toxic Spikes records. Damage bypass preserves unrelated reactive boosts such as Stamina. Ability ranges retain rank caps.
- The Windows NSIS installer is packaged without launching either the app or installer. Interactive native, install/upgrade/uninstall and physical LAN acceptance remain unverified.

The complete current [remaining-work inventory](REMAINING-WORK.md) includes every manual and partial ability, named move boundaries, catalog/assets, native acceptance and excluded scope.
