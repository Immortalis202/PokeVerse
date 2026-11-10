# Remaining move completion blocks

User authorization (4 November 2026): implement all remaining moves in coherent blocks, validate each block and commit it separately, then continue to the next block. Original exclusions (Max/Z/Dynamax, Sappy Seed, lethal wounds, automatic Will recovery and narrative GM decisions) remain.

The live source of remaining route work is `rules/move-effect-audit.json`. Its fallback count is an effect-dispatch diagnostic, not a count of missing mechanics. Existing executable scopes also retain their documented clause boundaries.

| Block | Scope | State |
| --- | --- | --- |
| MOVE-A01 | 66 damaging attribute clauses | Committed 9ec5fde |
| MOVE-A02 | Spikes, Stealth Rock, Sticky Web | Committed 4642278 |
| MOVE-A03 | Seven scene Blocking moves; shared saved block adoption | Complete |
| MOVE-A04 | Ten standard four-round trapping/ongoing moves and first-source priority | Complete |
| MOVE-A05 | Magma Storm and Salt Cure damage exceptions | Complete |
| MOVE-A06 | 68 ordinary attacks whose printed attributes already run in accuracy/damage | Complete |
| MOVE-A07 | Ten remaining unconditional damaging stat changes | Complete |
| MOVE-A08 | Eight Support stat cards and source-defined type/weather predicates | Complete |
| MOVE-A09 | Ten defense/critical/weather attack completion routes | Complete |
| MOVE-A10 | Eleven Accuracy-success penalties | Complete |
| MOVE-A11 | Four explicit three-stack stat cards | Complete |
| MOVE-A12 | Rapid Spin, Ice Spinner and Steel Roller cleanup | Complete |
| MOVE-A13 | Seven source-type and conditional damage attacks | Complete |
| MOVE-A14 | Five self-damage/stat/Blocking support cards | Complete |
| MOVE-A15 | Taunt, Throat Chop and Imprison restrictions | Complete |
| MOVE-A16 | Four history/round restrictions and saved deployment clocks | Complete |
| MOVE-A17 | Four Shield/Force Field bypass attacks | Complete |
| MOVE-A18 | Nine range, Water/Ground immunity and grounding attacks | Complete |
| MOVE-A19 | Existing item-move completion and two ordinary/narrative cards | Complete |
| MOVE-A20 | Four partner attacks and exact final-evolution guards | Complete |
| MOVE-A21 | Tailwind and Fairy Lock battlefield effects | Complete |
| MOVE-A22 | Spider Web entry Blocking and hazard cleanup | Complete |
| MOVE-A23 | Three ordinary attack completions and conditional Grassy Glide Reaction | Complete |
| MOVE-A24 | First Impression uses the first eligible action round (user ruling) | Complete |
| MOVE-A25 | Aura Wheel Dexterity and existing Hunger Switch typing | Complete |
| MOVE-A26 | Mud Sport and Water Sport damage fields | Complete |
| MOVE-A27 | Tar Shot independent stat and Fire vulnerability clauses | Complete |
| MOVE-A28 | Victory Dance and Autotomize stats and scene auxiliaries | Complete |
| MOVE-A29 | Charge next-damaging-Electric cast bonus and Clash consistency | Complete |
| MOVE-A30 | Laser Focus five-extra-successes defense condition | Complete |
| MOVE-A31 | Dragon Cheer High Critical and conditional Dragon stats | Complete |
| MOVE-A32 | Shelter defenses and per-target Accuracy protection | Complete |
| MOVE-A33 | Clear Smog, Haze and Freezy Frost resets using the user passive-bonus ruling | Complete |
| MOVE-A34 | Foresight, Odor Sleuth and Sweet Scent detection effects | Complete |
| MOVE-A35 | Telekinesis two-round hovering, immunity and no-Evade | Complete |
| MOVE-A36 | Fickle Beam host-rolled Power | Complete |
| MOVE-A37 | False Swipe non-fainting damage | Complete |
| MOVE-A38 | Explosion, Self Destruct and Misty Explosion | Complete |
| MOVE-A39 | Magic Powder type replacement | Complete |
| MOVE-A40 | Magnet Rise scene Ground immunity | Complete |
| MOVE-A41 | Ion Deluge and Plasma Fists type conversion | Complete |
| MOVE-A42 | Electrify next damaging cast | Complete |
| MOVE-A43 | Jump Kick, High Jump Kick and Supercell Slam crash damage | Complete |
| MOVE-A44 | Order Up covered Tatsugiri form bonus | Implemented; verification recorded below |
| Stat reset ruling | Preserve active passive bonuses; clear applied move and triggered-ability changes, confirmed 6 November 2026 | Adopted by MOVE-A33 |
| First Impression ruling | First eligible action round, confirmed by the user on 5 November 2026 | Implemented in MOVE-A24 |
| MOVE-A45 | Saved Hold Hands resistance and GM battle-ending workflow | Complete |
| MOVE-A46 | Metronome saved draw, matching template and shared copied cast | Delivered workflow; template closure open |
| Later blocks | Ordinary attacks with existing attributes; remaining stat changes, resets and swaps; type/damage conversions; field changes; conditional damage; timing/action restrictions; items; copy/selection effects | Open |

New block adoption is persisted in `TurnFlow.completion.blocks`. Every release records its engine in `rules/move-completion.json`; historical saves adopt no additional blocks automatically. The shared special-effect state is boxed while preserving its JSON shape to keep transition stack usage bounded.

No native packages, Clippy, native UI or physical LAN/VPN testing is scheduled automatically.


## Final whole-card completion project (10 November 2026)

The authorized final project covers all 892 source cards, preserving the existing exclusions. `rules/move-card-completion.json` is the provisional ledger; its source paths distinguish colliding catalog IDs. Existing route scopes are review inputs, never semantic certification. All in-scope clauses remain pending until reviewed and evidenced. The reproducible ledger assigns all pending review units to implementation batches 1–11. Sentence boundaries are review aids; compound sentences still require complete semantic review.

Ordered batches: (1) resolution inputs/GM choices; (2) stats/traits; (3) types/category/damage; (4) history/conditional attacks; (5) initiative/reactions/maneuvers; (6) allied contributions; (7) delayed/triggers; (8) fields/protection; (9) items/berries; (10) copying/selection; (11) compound effects/rewards; (12) whole-catalog closure. Each batch requires verification and a separate commit.

`npm run test:moves:closure` deliberately fails until semantic review is complete and every pending clause is resolved. The ordinary ledger integrity check does not assert completion.

MOVE-A44 baseline verification on 10 November: 978 Rust tests passed, one inventory ignored by the normal suite; 77 frontend and 54 script tests passed, as did TypeScript. Native/physical networking acceptance remains separate.


Inventory implementation: `npm run moves:inventory` verifies actual source SHA256 values and regenerates all 892 cards, preserving reviewed decisions only for unchanged source and matching clauses. The checklist contains 11,833 sentence/metadata review units, including Power, typing, damage/accuracy stats and structured effects. `npm run test:moves:inventory` checks regeneration, exact prose reconstruction, metadata preservation and assignments. These units are not counts of distinct mechanics or completed clauses. Whole-card semantic certification remains pending.


### MOVE-A45: saved Hold Hands decisions

See [the move-decision workflow](battle-move-decisions.md). Target resistance, reflected recipients, GM rulings, persistence and atomic battle-end rollback are implemented. Whole-card semantic review of the catalog remains open; the supported effect clauses have source-linked test evidence in the ledger.


### MOVE-A46: committed Metronome and copied casts

[Saved move decisions](battle-move-decisions.md) describes the explicit category table, fixed draw, template choice, original Accuracy, normal template execution, successive/charge support, wrapper identity, permissions and rollback. Targeting and committed-draw clauses have ledger evidence; complete template/copy semantic closure remains open. Validation passed 996 Rust tests plus the separate inventory, 86 frontend tests, 59 script checks, TypeScript and formatting.
