# Move-effect audit: milestone 1

Snapshot refreshed: 29 October 2026, app **0.41.0**, engine **0.34.0-core**. **Milestone 1 is complete at the scope below; M2-A through M2-E deliver all 107 queued battle-status cards (22 direct, 47 Chance-based, 28 compound/conditional, 6 recurring/sleep, 4 cure/protection).**

## Deliverables and limits

- [All 892 move cards and classifications](../rules/move-effect-audit.json): exact source prose/metadata, SHA256, application-catalog differences, scoped registry evidence, exclusions, two runtime effect-phase outcomes and the focused status assessment.
- [Frozen runtime route probes](../rules/move-effect-probes.json): 1,784 isolated effect-phase scenarios, baseline and synthetic Burn 2/Poison/Paralysis/Sleep profiles, with current feature gates enabled.
- [M2-B source and printed exceptions](../rules/chance-status-moves.json): all 47 JSON card fingerprints and exact Chance definitions, with nine additional printed-card checks; Fiery Wrath was also rendered and visually checked.
- [Printed-card references](../rules/move-status-print-review.json): 28 cards, including all 22 first-batch cards and the principal exceptions. Their queued status clauses were compared with extracted MASTER card text; printed page 455 was also rendered and visually checked for Wild Charge. This is not a visual review of all 892 cards.
- [Reproducible generator](../scripts/audit-move-effects.mjs) and [consistency checks](../scripts/move-effect-audit.check.mjs).

The full catalog is inventoried at the effect-dispatch level. The focused semantic review covers **180 status-related cards**, including infliction, cure, protection, sleep-related actions and recurring effects. Other moves have named, source-backed route records, but are not certified clause by clause. Existing runtime registries describe supported scopes, not complete coverage of all prose.

The diagnostic deliberately bypasses accuracy, damage, targeting expansion, action timing and command permissions. Participants are wounded and have no abilities, shields or decoys. All supplied chance dice are six. There are no prior hits, player selections, active weather or evolution prerequisites. A resolved route is not proof of full card support. An input/precondition error is not proof of missing mechanics. Mutations from an error are discarded, as with authoritative command rejection. Native UI and networking acceptance are not part of this audit.

## Inventory results

| Effect-route result | Cards | Meaning |
| --- | ---: | --- |
| executable-scope | 458 | At least one probe resolves; only recorded scopes are asserted. |
| input-required | 12 | Existing handler requests a selection, resource, weather or other legitimate prerequisite. |
| manual-effect-route | 383 | Both unprotected probes reach the explicit no-reviewed-effect fallback. Ordinary damage and some attributes may still work. |
| gm-choice | 1 | Hold Hands: narrative battle-ending decision remains with the storyteller. |
| excluded | 38 | Supplied Max/Z cards and the user-excluded Sappy Seed. |

These totals sum to 892. **383 manual-effect-route results are not 383 new mechanics or an instruction to implement every card.** The per-card source must still be divided into battle effects, excluded clauses, narrative clauses and already-supported components. Lethal wounds, outside-battle narrative effects and Dynamax-specific bonuses are separately marked and must not become automatic tasks.

| Focused status result | Cards |
| --- | ---: |
| excluded | 6 |
| implemented-clause | 173 |
| source-discrepancy | 1 |

The M2 implementation queue is complete at its reviewed battle scope: **0 queued cards remain**. All five batches are delivered. Of the 180 focused classifications, 173 have implemented clauses, six are excluded, and Healing Wish retains its source-metadata discrepancy. Source choices, narrative exclusions and unrelated card clauses remain documented; an implemented cure does not certify every targeting, range or damage clause on that move.

## Milestone 2: implementation queue

Complete one coherent sub-batch, run relevant regressions, update this inventory and commit before proceeding. M2-A through M2-E are delivered. The next milestone is M3 battlefield/ice edge-case review.

### M2-A. Direct status application (22 cards) — completed in 0.37.0

Delivered via [direct-status registry](../rules/direct-status-moves.json) and `combat/direct_status.rs`: guaranteed conditions without Chance dice, source identity for Attract, explicit Burn 3 for Inferno, repeated Will-O-Wisp escalation, and per-target area effects. Ordinary accuracy, damage, range/target selection, reflection, shields, decoys and condition immunities are reused. New current-engine encounters enable `TurnFlow.direct_status`; saved flows default false and historical engines retain their prior rules. Lethal wounds remain excluded.

Alluring Voice, Attract, Confuse Ray, Dark Void, Dynamic Punch, Glare, Grass Whistle, Inferno, Nuzzle, Poison Gas, Poison Powder, Sing, Sleep Powder, Spore, Stun Spore, Supersonic, Sweet Kiss, Teeter Dance, Thunder Wave, Toxic, Will-O-Wisp, Zap Cannon.

### M2-B. Chance-based status effects (47 cards) — completed in 0.38.0

Delivered via [Chance-status registry](../rules/chance-status-moves.json) and `combat/chance_status.rs`: exact dice pools and ailment degrees for all 47 cards. A six triggers the effect; ordinary successes alone do not. Serene Grace, Wonder Skin, Loaded Dice, native/item Flinch merging, Shield Dust, immunities, shields, decoys, sound bypass, Snow/Hail Freeze bonus and per-target resolution are retained. Wild Charge keeps its existing recoil pool and adds a separate one-die Paralysis pool. Sheer Force recognizes the new definitions even when source AddedEffects is empty, suppressing the chance while retaining recoil. New current-engine encounters enable `TurnFlow.chance_status`; saved flows default false.

Source policy: JSON prose and structured ailments agree for most cards. Nine printed exceptions were checked: Fiery Wrath, the five Torque moves, Mountain Gale, Smog and Bleakwind Storm. Fiery Wrath uses **2 dice**, confirmed by printed text and icon (MASTER 445 / PDF 312), despite structured metadata saying 3. The other missing fields follow printed prose. Original JSON is unchanged. Bleakwind Storm's separate accuracy-sign discrepancy remains queued below. Outside-battle extras on Headbutt/Rock Climb and lethal wounds remain excluded.

Aqua Tail, Astonish, Blaze Kick, Blazing Torque, Bleakwind Storm, Blue Flare, Bolt Strike, Bone Club, Chatter, Combat Torque, Cross Poison, Dizzy Punch, Dragon Breath, Dragon Rush, Fiery Wrath, Fire Punch, Flame Wheel, Force Palm, Freezing Glare, Headbutt, Heart Stamp, Heat Wave, Icicle Crash, Lick, Magical Torque, Mirror Shot, Mountain Gale, Needle Arm, Noxious Torque, Poison Fang, Poison Jab, Poison Tail, Pyro Ball, Rock Climb, Rolling Kick, Sacred Fire, Sandsear Storm, Scorching Sands, Searing Shot, Smog, Steam Eruption, Steamroller, Strange Steam, Wicked Torque, Wild Charge, Wildbolt Storm, Zing Zap.

### M2-C. Compound and conditional status effects (28 cards) — completed in 0.39.0

Delivered via [compound battle rules](battle-compound-status.md) and [source registry](../rules/compound-status-moves.json). Separate Chance pools, stat-plus-condition effects, entry/reaction predicates, evolution restrictions, delayed Venom Drench, Axe Kick crash checks, weather/category changes and Mortal Spin cleanup are executable. Hypnosis affects Dark types. Secret Power uses the printed environment table and trainer alignment; Relic Song offers the optional reviewed Meloetta form change. Permanent Beauty learning/forgetting changes and the rest of Minimize remain outside this batch; Stomp consumes recorded Minimize state.

Axe Kick, Burning Jealousy, Buzzy Buzz, Dire Claw, Fake Out, Fire Fang, Flatter, Floaty Fall, Glaciate, Hydro Steam, Hypnosis, Ice Fang, Infernal Parade, Lovely Kiss, Mortal Spin, Relic Song, Secret Power, Shell Side Arm, Sizzly Slide, Splishy Splash, Stomp, Swagger, Thunder Fang, Toxic Thread, Tri Attack, Twister, Upper Hand, Venom Drench.

### M2-D. Recurring and sleep-action effects (6 cards) — completed in 0.40.0

Delivered via [recurring battle rules](battle-recurring-status.md) and [source registry](../rules/recurring-status-moves.json): Yawn starts next round; Nightmare damages failed wake attempts; Perish Song expires after three rounds; Malignant Chain rolls raw Loyalty at round end and changes battle allegiance from next round until cured. Sleep Talk commits a host-random known move/foe and uses the normal pipeline, up to five actions; Snore requires Sleep. Recursive/unsupported Sleep Talk draws remain explicit GM adjudication. Historical encounters retain their earlier behavior.

Malignant Chain, Nightmare, Perish Song, Sleep Talk, Snore, Yawn.

### M2-E. Cure/protection and missing entry paths (4 cards) - completed in 0.41.0

Delivered via [cure/protection rules](battle-cure-protection.md) and [source registry](../rules/cure-protection-moves.json): Uproar cures Sleep across the field for the rest of its cast round, retaining Soundproof/Comatose and existing GM random-foe selection. Sparkling Aria resolves unburned targets and keeps per-target Burn curing/Support conversion. Safeguard protects the user's side for four rounds without curing existing conditions, sharing force-field bypass and removal behavior. Toxic Spikes casts into the existing persistent hazard system and reuses its entry Chance roll, immunities and cleanup.

Safeguard, Sparkling Aria, Toxic Spikes, Uproar.

## Already implemented: do not schedule these again

The following cards have a reviewed named status clause or cure/protection handler in the current engine. Keep their existing scope and source-specific prerequisites. The audit does not mark the whole card complete or remove historical feature gates.

Air Slash, Aromatherapy, Baneful Bunker, Barb Barrage, Beak Blast, Bite, Bitter Malice, Blizzard, Body Slam, Bounce, Burning Bulwark, Confusion, Dark Pulse, Discharge, Double Iron Bash, Electric Terrain, Ember, Extrasensory, Fire Blast, Flamethrower, Flare Blitz, Focus Punch, Freeze Shock, Gunk Shot, Heal Bell, Hurricane, Hyper Fang, Ice Beam, Ice Burn, Ice Punch, Iron Head, Jungle Healing, Lava Plume, Lunar Blessing, Lunar Dance, Matcha Gotcha, Misty Terrain, Poison Sting, Powder Snow, Psybeam, Psycho Shift, Purify, Refresh, Rest, Rock Slide, Scald, Signal Beam, Sky Attack, Sky Drop, Sludge Bomb, Sludge Wave, Sludge, Smelling Salts, Spark, Sparkly Swirl, Take Heart, Thunder Punch, Thunder Shock, Thunder, Thunderbolt, Twineedle, Volt Tackle, Wake-Up Slap, Water Pulse, Waterfall, Zen Headbutt.

Costar, Opportunist, Grudge, Eerie Spell and Spite remain delivered at their existing documented scopes; none is reclassified as missing by this audit. Status-dependent damage moves such as Facade, Hex, Venoshock and Dream Eater are not counted as new status-infliction mechanics just because their prose mentions an ailment. Dive's underwater narrative and Synchronoise's example type are likewise excluded from the status candidate set.

## Source discrepancies, partial routes and genuine decisions

- **Healing Wish, MASTER 512 / PDF 379:** the printed card specifies sacrifice and full recovery, with no Will charge. The supplied JSON healing metadata alone adds one Will. The current no-charge implementation matches the printed prose. Track an application-owned metadata correction separately; there is no reason to invent an additional spend or ask the GM to resolve the same mismatch every use. Source JSON remains unchanged.
- **Wild Charge, MASTER 455 / PDF 322:** printed prose and the paralysis icon both specify one Chance die. Delivered in M2-B: the new gated Chance handler adds that Paralysis pool independently of recoil.
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

## M2-A verification (29 October 2026)

All 713 Rust tests passed across the regression run and a serial binding-export retry: the four-thread run passed 682 tests and hit Windows mapped-file locks in 31 export tests; all 114 binding-export tests then passed serially. The six new test groups cover every direct-status card through authoritative accuracy/damage/effect commands, ownership, Burn escalation, Inferno degree, Attract source and Magic Bounce identity, immunity rules, Shield Dust on guaranteed effects, decoys/sound bypass, shields, area/all-foes targets and historical fallback. The existing engine-gate regression now also checks the prior Will engine and the new encounter flag.

TypeScript, 65 frontend tests, all 27 script/catalog checks, Rust formatting and diff checks passed. The 892-card two-profile diagnostic was refreshed for this engine; its 1,784 effect-phase scenarios remain an effect-route inventory, not full battle coverage. No Clippy, app packaging or native/physical-network acceptance was run. M2-B followed this batch.

## M2-B verification (29 October 2026)

Eight new Rust test groups cover all 47 cards with triggering and non-triggering dice, exact pool counts, permissions and invalid dice rejection, modifiers and zero pools, native/item Flinch merging including Teravolt/Wonder Skin bypass, metadata-empty Sheer Force, type/condition immunity, sound/decoy behavior, shields, once-per-round Flinch, weather, independent area-target rolls, recoil separation, rollback and historical fallback. The 892-card two-profile diagnostic was refreshed (1,784 effect-phase scenarios, not full battle simulations). All 721 Rust tests passed: 607 runtime regressions including localhost multiplayer checks, plus 114 binding exports run serially to avoid Windows file locks. After the final scoped native/item Flinch adjustment, all eight Chance-status groups and all 136 item regressions passed again. TypeScript, 65 frontend tests, all 28 script/catalog checks, Rust formatting and diff checks passed. No Clippy, app packages, native UI or physical-network acceptance was run.

Remaining source-metadata correction: **Bleakwind Storm, MASTER 469 / PDF 336**, has `AccuracyReduction: +2` in supplied JSON but printed/JSON prose says Low Accuracy 2. This Chance-effect batch preserves existing accuracy behavior; an application-owned correction and accuracy regression remain required.

## M2-C verification (29 October 2026)

See [the delivered clauses, source discrepancies, controls and retained boundaries](battle-compound-status.md). The diagnostic inventory is refreshed for this engine; Secret Power correctly requires scene/terrain input rather than being mislabeled missing. All 740 Rust tests passed (623 runtime and 117 serial binding exports), plus TypeScript, 68 frontend tests, 29 script/catalog checks, Rust formatting and diff checks. All 892 diagnostic routes were refreshed in both profiles; these remain effect-phase probes rather than end-to-end battle certification. No Clippy, app packaging, native UI or physical-network acceptance was run.


## M2-D verification (29 October 2026)

See [recurring battle effects and retained choices](battle-recurring-status.md). All 762 Rust tests passed (640 runtime and 122 serial binding exports), with 17 recurring test groups. The final benched-owner permission adjustment was also checked by rerunning those 17 groups. TypeScript, 71 frontend tests, 30 script/catalog checks, Rust formatting and diff checks passed. All 892 effect routes were refreshed in both profiles. The new recurring state is boxed to keep ordinary battle transition stack usage bounded while preserving its serialized shape. No Clippy, app packaging, native UI or physical-network acceptance was run. At that checkpoint, M2-E had four remaining cards; they are delivered below.


## M2-E verification (29 October 2026)

All 774 Rust tests passed (652 runtime and 122 serial binding exports), including 12 new cure/protection test groups. The groups cover four-round expiry, every structured condition, retained existing conditions, hostile Infiltrator/sound bypass, field cleanup, mixed-area Burn cure versus damage, one-round Sleep cure and later inflictions, Soundproof/Comatose, hazard entry Chance dice, Ground immunity, recast non-stacking, permissions, serialization, rollback and historical gates. TypeScript, 71 frontend tests, 31 script/catalog checks, Rust formatting and diff checks passed. All 892 effect routes were refreshed in both profiles, with the four M2-E baseline routes asserted executable. No Clippy, app packaging, native UI or physical-network acceptance was run. The M2 queue is complete at its reviewed scope; M3 is next.
