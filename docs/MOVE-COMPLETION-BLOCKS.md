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
| MOVE-A27 | Tar Shot independent stat and Fire vulnerability clauses | Next |
| First Impression ruling | First eligible action round, confirmed by the user on 5 November 2026 | Implemented in MOVE-A24 |
| Later blocks | Ordinary attacks with existing attributes; remaining stat changes, resets and swaps; type/damage conversions; field changes; conditional damage; timing/action restrictions; items; copy/selection effects | Open |

New block adoption is persisted in `TurnFlow.completion.blocks`. Every release records its engine in `rules/move-completion.json`; historical saves adopt no additional blocks automatically. The shared special-effect state is boxed while preserving its JSON shape to keep transition stack usage bounded.

No native packages, Clippy, native UI or physical LAN/VPN testing is scheduled automatically.
