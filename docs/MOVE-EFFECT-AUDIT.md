# Move-effect audit: milestone 1

Snapshot: 28 October 2026, app **0.36.0**, engine **0.29.0-core**. **Milestone 1 is complete at the scope below; no move mechanics changed.**

## Deliverables and limits

- [All 892 move cards and classifications](../rules/move-effect-audit.json): exact source prose/metadata, SHA256, application-catalog differences, scoped registry evidence, exclusions, two runtime effect-phase outcomes and the focused status assessment.
- [Frozen runtime route probes](../rules/move-effect-probes.json): 1,784 isolated effect-phase scenarios, baseline and synthetic Burn 2/Poison/Paralysis/Sleep profiles, with current feature gates enabled.
- [Printed-card references](../rules/move-status-print-review.json): 28 cards, including all 22 first-batch cards and the principal exceptions. Their queued status clauses were compared with extracted MASTER card text; printed page 455 was also rendered and visually checked for Wild Charge. This is not a visual review of all 892 cards.
- [Reproducible generator](../scripts/audit-move-effects.mjs) and [consistency checks](../scripts/move-effect-audit.check.mjs).

The full catalog is inventoried at the effect-dispatch level. The focused semantic review covers **180 status-related cards**, including infliction, cure, protection, sleep-related actions and recurring effects. Other moves have named, source-backed route records, but are not certified clause by clause. Existing runtime registries describe supported scopes, not complete coverage of all prose.

The diagnostic deliberately bypasses accuracy, damage, targeting expansion, action timing and command permissions. Participants are wounded and have no abilities, shields or decoys. All supplied chance dice are six. There are no prior hits, player selections, active weather or evolution prerequisites. A resolved route is not proof of full card support. An input/precondition error is not proof of missing mechanics. Mutations from an error are discarded, as with authoritative command rejection. Native UI and networking acceptance are not part of this audit.

## Inventory results

| Effect-route result | Cards | Meaning |
| --- | ---: | --- |
| executable-scope | 355 | At least one probe resolves; only recorded scopes are asserted. |
| input-required | 10 | Existing handler requests a selection, resource, weather or other legitimate prerequisite. |
| manual-effect-route | 488 | Both unprotected probes reach the explicit no-reviewed-effect fallback. Ordinary damage and some attributes may still work. |
| gm-choice | 1 | Hold Hands: narrative battle-ending decision remains with the storyteller. |
| excluded | 38 | Supplied Max/Z cards and the user-excluded Sappy Seed. |

These totals sum to 892. **488 manual-effect-route results are not 488 new mechanics or an instruction to implement every card.** The per-card source must still be divided into battle effects, excluded clauses, narrative clauses and already-supported components. Lethal wounds, outside-battle narrative effects and Dynamax-specific bonuses are separately marked and must not become automatic tasks.

| Focused status result | Cards |
| --- | ---: |
| excluded | 6 |
| implemented-clause | 66 |
| missing-clause | 1 |
| missing-effect | 104 |
| missing-move-entry | 1 |
| partial-clause | 1 |
| source-discrepancy | 1 |

The actionable implementation queue contains **107 cards**: 104 missing effects, Wild Charge's missing clause, Toxic Spikes' missing move installation and Sparkling Aria's incomplete branch. These cards form the five sub-batches below. A further 66 cards already have a named implemented status clause, six are excluded, and Healing Wish has a source-metadata discrepancy. These 180 classifications are about the audited clauses; an implemented cure does not certify every targeting, range or damage clause on that move.

## Milestone 2: implementation queue

Complete one coherent sub-batch, run relevant regressions, update this inventory and commit before proceeding. These are planned tasks, not behavior added by milestone 1.

### M2-A. Direct status application (22 cards)

Reuse existing authoritative condition rules, immunity checks, reflection, range/target expansion and severe-status escalation. No new generic status system is needed. Start with Confuse Ray, Thunder Wave, Toxic, Will-O-Wisp, Spore and Sing, then the other cards below. Attract needs the beloved source identity; area effects need per-target handling.

Alluring Voice, Attract, Confuse Ray, Dark Void, Dynamic Punch, Glare, Grass Whistle, Inferno, Nuzzle, Poison Gas, Poison Powder, Sing, Sleep Powder, Spore, Stun Spore, Supersonic, Sweet Kiss, Teeter Dance, Thunder Wave, Toxic, Will-O-Wisp, Zap Cannon.

### M2-B. Chance-based status effects (47 cards)

Extend exact chance definitions with printed dice counts and Burn degrees. Retain Serene Grace, Wonder Skin, Shield Dust, Loaded Dice, shields, decoys and per-target immunity rules. Wild Charge already has recoil; add only its printed one-die paralysis clause. Outside-battle extras on Headbutt/Rock Climb remain out of scope.

Aqua Tail, Astonish, Blaze Kick, Blazing Torque, Bleakwind Storm, Blue Flare, Bolt Strike, Bone Club, Chatter, Combat Torque, Cross Poison, Dizzy Punch, Dragon Breath, Dragon Rush, Fiery Wrath, Fire Punch, Flame Wheel, Force Palm, Freezing Glare, Headbutt, Heart Stamp, Heat Wave, Icicle Crash, Lick, Magical Torque, Mirror Shot, Mountain Gale, Needle Arm, Noxious Torque, Poison Fang, Poison Jab, Poison Tail, Pyro Ball, Rock Climb, Rolling Kick, Sacred Fire, Sandsear Storm, Scorching Sands, Searing Shot, Smog, Steam Eruption, Steamroller, Strange Steam, Wicked Torque, Wild Charge, Wildbolt Storm, Zing Zap.

### M2-C. Compound and conditional status effects (28 cards)

Handle multiple independent chance pools, stat-plus-condition effects, first-entry/reaction predicates, environment dependence, evolution restrictions and special target rules. Hypnosis explicitly affects Dark types. Do not substitute video-game one-status selection rules for Dire Claw/Tri Attack or invent Secret Power environment outcomes. Relic Song requires a separately reviewed form choice.

Axe Kick, Burning Jealousy, Buzzy Buzz, Dire Claw, Fake Out, Fire Fang, Flatter, Floaty Fall, Glaciate, Hydro Steam, Hypnosis, Ice Fang, Infernal Parade, Lovely Kiss, Mortal Spin, Relic Song, Secret Power, Shell Side Arm, Sizzly Slide, Splishy Splash, Stomp, Swagger, Thunder Fang, Toxic Thread, Tri Attack, Twister, Upper Hand, Venom Drench.

### M2-D. Recurring and sleep-action effects (6 cards)

Review the exact clocks and cancellation conditions for Yawn, Nightmare, Perish Song and Malignant Chain. Sleep Talk/Snore need their sleep/action prerequisites and permitted selection/repetition behavior. Reuse existing recurring-check infrastructure where appropriate; do not infer behavior from similarly named video-game moves.

Malignant Chain, Nightmare, Perish Song, Sleep Talk, Snore, Yawn.

### M2-E. Cure/protection and missing entry paths (4 cards)

Uproar needs its printed battlefield Sleep cure and one-round scope. Sparkling Aria already has Burn curing and Support conversion for burned targets; repair its unburned-target effect branch. Safeguard has a cleanup reference but no cast or immunity implementation. Toxic Spikes already has hazard entry behavior through Toxic Debris/field state; add the move installation path, not a duplicate hazard system.

Safeguard, Sparkling Aria, Toxic Spikes, Uproar.

## Already implemented: do not schedule these again

The following cards have a reviewed named status clause or cure/protection handler in the current engine. Keep their existing scope and source-specific prerequisites. The audit does not mark the whole card complete or remove historical feature gates.

Air Slash, Aromatherapy, Baneful Bunker, Barb Barrage, Beak Blast, Bite, Bitter Malice, Blizzard, Body Slam, Bounce, Burning Bulwark, Confusion, Dark Pulse, Discharge, Double Iron Bash, Electric Terrain, Ember, Extrasensory, Fire Blast, Flamethrower, Flare Blitz, Focus Punch, Freeze Shock, Gunk Shot, Heal Bell, Hurricane, Hyper Fang, Ice Beam, Ice Burn, Ice Punch, Iron Head, Jungle Healing, Lava Plume, Lunar Blessing, Lunar Dance, Matcha Gotcha, Misty Terrain, Poison Sting, Powder Snow, Psybeam, Psycho Shift, Purify, Refresh, Rest, Rock Slide, Scald, Signal Beam, Sky Attack, Sky Drop, Sludge Bomb, Sludge Wave, Sludge, Smelling Salts, Spark, Sparkly Swirl, Take Heart, Thunder Punch, Thunder Shock, Thunder, Thunderbolt, Twineedle, Volt Tackle, Wake-Up Slap, Water Pulse, Waterfall, Zen Headbutt.

Costar, Opportunist, Grudge, Eerie Spell and Spite remain delivered at their existing documented scopes; none is reclassified as missing by this audit. Status-dependent damage moves such as Facade, Hex, Venoshock and Dream Eater are not counted as new status-infliction mechanics just because their prose mentions an ailment. Dive's underwater narrative and Synchronoise's example type are likewise excluded from the status candidate set.

## Source discrepancies, partial routes and genuine decisions

- **Healing Wish, MASTER 512 / PDF 379:** the printed card specifies sacrifice and full recovery, with no Will charge. The supplied JSON healing metadata alone adds one Will. The current no-charge implementation matches the printed prose. Track an application-owned metadata correction separately; there is no reason to invent an additional spend or ask the GM to resolve the same mismatch every use. Source JSON remains unchanged.
- **Wild Charge, MASTER 455 / PDF 322:** printed prose and the paralysis icon both specify one Chance die. The recoil route succeeds, but `special::chance` has no Wild Charge entry, so that secondary clause is missing.
- **Sparkling Aria, MASTER 526 / PDF 393:** curing Burn instead of damage is already handled for burned targets. The baseline non-burned effect route falls through. Treat it as a partial branch repair, not an absent cure.
- **Safeguard, MASTER 499 / PDF 366:** a name in Screen Cleaner cleanup is not an executable Safeguard definition. The move needs both installation and its protection predicate.
- **Toxic Spikes, MASTER 509 / PDF 376:** Toxic Debris/field entry poisoning already exists. The move's installation branch does not.
- **Hold Hands:** the storyteller's battle-ending decision remains narrative. GM choices are not missing automatic outcomes.
- **Sappy Seed:** excluded by the user on 28 October. Max/Dynamax and Z-Move exclusions remain in force. Automatic Will recovery and lethal wounds remain excluded.

The first direct-status cards have these printed references:

| Card | MASTER page | PDF page |
| --- | ---: | ---: |
| Alluring Voice | 455 | 322 |
| Attract | 487 | 354 |
| Confuse Ray | 472 | 339 |
| Dark Void | 444 | 311 |
| Dynamic Punch | 460 | 327 |
| Glare | 493 | 360 |
| Grass Whistle | 476 | 343 |
| Inferno | 467 | 334 |
| Nuzzle | 453 | 320 |
| Poison Gas | 508 | 375 |
| Poison Powder | 508 | 375 |
| Sing | 500 | 367 |
| Sleep Powder | 479 | 346 |
| Spore | 479 | 346 |
| Stun Spore | 480 | 347 |
| Supersonic | 503 | 370 |
| Sweet Kiss | 458 | 325 |
| Teeter Dance | 504 | 371 |
| Thunder Wave | 454 | 321 |
| Toxic | 509 | 376 |
| Will-O-Wisp | 468 | 335 |
| Zap Cannon | 455 | 322 |

## Existing input requirements

These probe failures are expected prerequisites rather than missing effects. Normal UI/host command validation remains authoritative.

| Move | Probe feedback |
| --- | --- |
| Aurora Veil | Aurora Veil requires Hail or Snow |
| Healing Wish | Choose the healing recipient before resolving |
| Lunar Blessing | Choose the foe affected by Lunar Blessing |
| Lunar Dance | Choose the healing recipient before resolving |
| Parabolic Charge | Select one Parabolic Charge damage recipient for draining |
| Present | Host-roll Present before resolving |
| Recycle | Recycle's reserved spent item is unavailable |
| Revival Blessing | Choose a fainted registered ally |
| Swallow | Swallow requires Stockpile |
| Transform | Choose the visible Transform model before attempting the move |

## Reproduce and verify

From the repository directory in PowerShell:

```powershell
$env:POKEVERSE_MOVE_AUDIT_OUTPUT = Join-Path (Get-Location) 'rules/move-effect-probes.json'
cargo test --manifest-path src-tauri/Cargo.toml --lib move_effect_route_inventory -- --ignored --nocapture
node scripts/audit-move-effects.mjs
node --test scripts/move-effect-audit.check.mjs
```

The diagnostic test is explicitly ignored by ordinary Rust test runs and writes only when the output environment variable is present. It uses isolated fixtures and a read-only catalog, not saved campaigns. Frozen probes are snapshot evidence: rerun after changing effect handlers, then regenerate the inventory and review classification overrides. `--check` verifies deterministic generation from the frozen probes and sources; it does not rerun Rust or certify a changed engine. The Markdown queue should be updated alongside regenerated classifications.

Verification for this batch: the diagnostic passed all 892 cards in both profiles (1,784 effect-phase scenarios), including explicit assertions for missing direct statuses, existing Air Slash, Wild Charge's unresolved secondary clause and Sparkling Aria's conditional branch. The four audit checks validate complete source coverage, classifications, printed-source fingerprints and deterministic generation. The full frontend suite passed 65 tests and all 27 catalog/script checks passed. Rust formatting and diff checks passed; the existing 707-case Rust suite was not rerun because runtime code is unchanged outside a test-only diagnostic helper. No runtime behavior changed; no app packages or Clippy runs were requested.
