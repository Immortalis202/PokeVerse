# Cure, protection and hazard move paths

Delivered in app 0.41.0, engine 0.34.0-core, protocol 42. The [source registry](../rules/cure-protection-moves.json) preserves all four exact JSON effects, source hashes and printed references: MASTER 499, 505, 509 and 526 (PDF pages 366, 372, 376 and 393). Source data is unchanged.

| Move | Behavior |
| --- | --- |
| Safeguard | Cast on the user's side, including later entrants. Prevents all new structured conditions for four rounds including the cast round; previously inflicted conditions remain. Uses existing hostile force-field bypass rules, including Infiltrator and sound. Screen Cleaner and Defog remove it. Recasting refreshes its clock. |
| Sparkling Aria | Normal rank-limited Area targeting and per-target resolution. A burned target uses the existing Support conversion and has all Burn levels cured without damage. An unburned target takes ordinary Special damage and finishes its zero-dice effect successfully. Shields, decoys and ability immunities retain the existing pipeline. |
| Toxic Spikes | Casts a persistent hazard on opposing sides. Does not poison present participants immediately and does not stack video-game layers. Entrants use the existing one-die host event: Poison normally, Badly Poisoned on a six. Ground immunity, Heavy-Duty Boots, Poison immunity, Safeguard and existing hazard cleanup are reused. |
| Uproar | Normal Random Foe attack; the GM records its randomly selected foe through existing targeting controls. The successful effect establishes battlefield-wide Sleep curing for the rest of the cast round, including later Sleep inflictions or entrants. Soundproof and Comatose retain their immunity/never-wake behavior. Other conditions remain. No video-game rampage or multi-round attack lock is introduced. |

Safeguard, Toxic Spikes and Uproar appear in the existing battlefield rule display with scope and duration; field removal, authoritative commands, receipts and rollback use the shared infrastructure. No new manual choice screen is required. Uproar's field cure is separate from the selected foe's damage defenses. Its duration expires through ordinary round-end field expiry; a later round can inflict Sleep normally.

New current-engine encounters enable the saved `TurnFlow.cure_protection` flag. Missing flags default false; historical 0.33.0-core and older encounters keep their previous behavior. The earlier recurring, compound and Chance flags remain enabled in their appropriate historical engines. No temporary field state is written to library sheets.

The four cards complete the M2 audited status queue. This certifies the reviewed battle clauses, not every effect on all 892 catalog cards. Healing Wish's metadata cost discrepancy, Bleakwind Storm's accuracy sign, excluded narrative/lethal/Max/Z clauses and the documented Sleep Talk GM boundary remain separately scoped. M3 battlefield/ice review is next.

Validation: 12 new Rust groups; full suite 652 runtime tests plus 122 binding exports, TypeScript, 71 frontend tests, 31 script/catalog checks, formatting and diff checks. The 892-card diagnostic was regenerated for both profiles. No Clippy, app builds, native UI or physical networking acceptance was run.
