# Move-effect audit: milestone 1

Snapshot refreshed: 5 November 2026, app **0.70.0**, engine **0.62.0-core**. **Milestone 1 is complete at the scope below; M2-A through M2-E deliver all 107 queued battle-status cards (22 direct, 47 Chance-based, 28 compound/conditional, 6 recurring/sleep, 4 cure/protection).**

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
| executable-scope | 708 | At least one probe resolves; only recorded scopes are asserted. |
| input-required | 16 | Existing handler requests a selection, resource, weather or other legitimate prerequisite. |
| manual-effect-route | 129 | Both unprotected probes reach the explicit no-reviewed-effect fallback. Ordinary damage and some attributes may still work. |
| gm-choice | 1 | Hold Hands: narrative battle-ending decision remains with the storyteller. |
| excluded | 38 | Supplied Max/Z cards and the user-excluded Sappy Seed. |

These totals sum to 892. **129 manual-effect-route results remain after MOVE-A27. They are not a count of distinct missing mechanics.** The per-card source must still be divided into battle effects, excluded clauses, narrative clauses and already-supported components. Lethal wounds, outside-battle narrative effects and Dynamax-specific bonuses are separately marked and must not become automatic tasks.

| Focused status result | Cards |
| --- | ---: |
| excluded | 6 |
| implemented-clause | 173 |
| source-discrepancy | 1 |

The M2 implementation queue is complete at its reviewed battle scope: **0 queued cards remain**. All five batches are delivered. Of the 180 focused classifications, 173 have implemented clauses, six are excluded, and Healing Wish retains its source-metadata discrepancy. Source choices, narrative exclusions and unrelated card clauses remain documented; an implemented cure does not certify every targeting, range or damage clause on that move.

## Milestone 2: implementation queue

Complete one coherent sub-batch, run relevant regressions, update this inventory and commit before proceeding. M2-A through M2-E are delivered. M3 battlefield/ice review and M4 catalog reliability are also delivered. The move-completion follow-up continues below; roadmap M5 remains native acceptance.

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

Resolved by M4-C01 (30 October 2026): **Bleakwind Storm, MASTER 469 / PDF 336**, has `AccuracyReduction: +2` in supplied JSON but printed/JSON prose says Low Accuracy 2. This Chance-effect batch preserves existing accuracy behavior; the application-owned correction and accuracy regression are delivered (see rules/catalog-reliability-review.json).

## M2-C verification (29 October 2026)

See [the delivered clauses, source discrepancies, controls and retained boundaries](battle-compound-status.md). The diagnostic inventory is refreshed for this engine; Secret Power correctly requires scene/terrain input rather than being mislabeled missing. All 740 Rust tests passed (623 runtime and 117 serial binding exports), plus TypeScript, 68 frontend tests, 29 script/catalog checks, Rust formatting and diff checks. All 892 diagnostic routes were refreshed in both profiles; these remain effect-phase probes rather than end-to-end battle certification. No Clippy, app packaging, native UI or physical-network acceptance was run.


## M2-D verification (29 October 2026)

See [recurring battle effects and retained choices](battle-recurring-status.md). All 762 Rust tests passed (640 runtime and 122 serial binding exports), with 17 recurring test groups. The final benched-owner permission adjustment was also checked by rerunning those 17 groups. TypeScript, 71 frontend tests, 30 script/catalog checks, Rust formatting and diff checks passed. All 892 effect routes were refreshed in both profiles. The new recurring state is boxed to keep ordinary battle transition stack usage bounded while preserving its serialized shape. No Clippy, app packaging, native UI or physical-network acceptance was run. At that checkpoint, M2-E had four remaining cards; they are delivered below.


## M2-E verification (29 October 2026)

All 774 Rust tests passed (652 runtime and 122 serial binding exports), including 12 new cure/protection test groups. The groups cover four-round expiry, every structured condition, retained existing conditions, hostile Infiltrator/sound bypass, field cleanup, mixed-area Burn cure versus damage, one-round Sleep cure and later inflictions, Soundproof/Comatose, hazard entry Chance dice, Ground immunity, recast non-stacking, permissions, serialization, rollback and historical gates. TypeScript, 71 frontend tests, 31 script/catalog checks, Rust formatting and diff checks passed. All 892 effect routes were refreshed in both profiles, with the four M2-E baseline routes asserted executable. No Clippy, app packaging, native UI or physical-network acceptance was run. The M2 queue is complete at its reviewed scope; M3 was subsequently delivered.


M3 reconciliation note (29 October 2026): [M3-F02](M3-BATTLEFIELD-ICE-CHECKLIST.md#m3-f02) identifies a source mismatch in the delivered Perish Song interaction: printed Magic Guard explicitly permits its damage, while the current shared helper and regression block it. This baseline mismatch is corrected in reviewed 0.35.0-core encounters by M3 (30 October 2026); historical encounters retain the old expectation.


## Remaining move implementation

MOVE-A01 delivers **66 damaging attribute clauses** in [the exact-name source registry](../rules/damaging-stat-moves.json) and `combat/damaging_stats.rs`. Definitions follow supplied JSON prose with exact SHA256 provenance; upstream source files are unchanged. This is a JSON-backed implementation review, not a new PDF visual review.

Guaranteed changes (38):

Acid Spray, Apple Acid, Aqua Step, Armor Cannon, Breaking Swipe, Bulldoze, Chilling Water, Clanging Scales, Close Combat, Draco Meteor, Dragon Ascent, Drum Beating, Electroweb, Esper Wing, Fire Lash, Flame Charge, Fleur Cannon, Grav Apple, Hammer Arm, Headlong Rush, Ice Hammer, Icy Wind, Overheat, Pounce, Power-Up Punch, Psycho Boost, Rock Tomb, Skitter Smack, Snarl, Spin Out, Spirit Break, Struggle Bug, Superpower, Thunderous Kick, Torch Song, Trailblaze, Trop Kick, V-Create.

Chance changes (28):

Acid, Ancient Power, Aurora Beam, Bubble, Bubble Beam, Bug Buzz, Charge Beam, Constrict, Crunch, Crush Claw, Diamond Storm, Earth Power, Energy Ball, Fiery Dance, Flash Cannon, Focus Blast, Iron Tail, Ominous Wind, Play Rough, Psychic, Psyshield Bash, Razor Shell, Rock Smash, Seed Flare, Shadow Ball, Shadow Bone, Silver Wind, Steel Wing.

A six triggers the single printed Chance pool. Serene Grace, Wonder Skin, Loaded Dice, Sheer Force, Shield Dust on hostile target chances, Clear Amulet, Clear Body, Mirror Armor, Contrary/Simple and the existing separate non-stacking move channels are reused. Shields, decoys, sound bypass and damage/type immunity are retained. Target changes resolve independently; user changes and pools occur once per cast across area recipients. Once-per-cast state survives save/reconnect and rollback and is removed when the cast completes.

New 0.36.0-core encounters enable the boxed `damaging_stats` state. Missing saved state defaults disabled; 0.35.0-core remains accepted with its previous M3 behavior. Protocol 45 carries the new state. App version is 0.44.0.

**The whole move catalog is not complete.** The 129 remaining fallback routes still require clause review, including plain attacks with existing attributes, conditional damage, item/type/stat exchange, trapping/ongoing damage, action restrictions, copy moves and field effects. Existing routes can also have unsupported clauses. Continue in coherent batches, run regressions, refresh the probes and commit each batch. Narrative rulings, lethal wounds, automatic Will recovery, Sappy Seed and Max/Z exclusions remain in force.


### MOVE-A01 verification (1 November 2026)

All **810 Rust tests passed**, including the explicitly enabled diagnostic (892 cards / 1,784 effect-phase probes) and localhost multiplayer regressions. After the final rollback assertion and registry label update, all six damaging-stat regression groups plus their binding export passed again. The area regression verifies independent target Chance pools and one user pool per cast. The new state is boxed to avoid growing ordinary transition stack frames; the full 3v3 networking regression passes with the default test stack.

**74 frontend tests, 52 script/catalog checks, TypeScript, Rust formatting and diff checks passed.** The catalog was rebuilt and the existing five species comparison reports and two closure records refreshed against its new compatibility hash. No Clippy, native UI, app packages or physical LAN/VPN acceptance was run. This completes MOVE-A01; the 317 remaining fallback routes are not claimed complete.


### MOVE-A02: entry hazards (4 November 2026)

Spikes, Stealth Rock and Sticky Web are delivered through [exact source records and decisions](../rules/entry-hazard-moves.json) and `combat/entry_hazards.rs`. This review uses supplied JSON prose and SHA256, without altering upstream files or claiming a new PDF visual review.

All three install persistent opposing-side rules without hitting current occupants. Recasting or overlapping scopes do not add layers or multiply entry effects. For overlapping scopes of the same hazard, the first installed applicable rule supplies caster provenance. Spikes deals 1 typeless damage, Stealth Rock deals 1 Rock-type damage under the existing additive type/indirect-damage rules, and Sticky Web applies Dexterity -1 through the existing separate non-stacking move channels. Effects apply only to a genuine Pokemon entry, once even when entry/lifecycle processing runs multiple times or a Pokemon has multiple abilities. They survive the caster's departure and save/reconnect.

Heavy-Duty Boots prevents all three; Ground immunity prevents Spikes and Sticky Web. Live type, Levitate, Earth Eater, Air Balloon, Gravity, indirect-damage immunity, stat protections, Clear Amulet, Contrary/Simple and stat-reaction abilities retain their reviewed rules. Persistent hazards do not inherit their caster's live attack ability bypass. Magic Bounce/Coat retains the existing side reflection. Sticky Web with Mirror Armor redirects the stat effect to its recorded conscious active caster; if that caster has left or fainted, the absent-source redirected outcome remains explicit GM adjudication and rejects the command atomically.

Defog and Tidy Up remove all three hazards across the battlefield; Mortal Spin removes them from its user's side through its existing cleanup. Cleanup removes future entry effects and does not undo an already-applied stat reduction. Fainted Pokemon do not take additional entry damage or revive. Trainer entrants are excluded.

New 0.37.0-core encounters enable `TurnFlow.entry_hazards`; saved missing state defaults false. The prior 0.36.0-core engine retains MOVE-A01 and its historical missing-hazard routes. App 0.45.0 uses protocol 46. The refreshed inventory has 527 executable scopes and 314 manual-effect routes; a resolved route still does not certify every clause on other cards.


#### MOVE-A02 verification

All **818 Rust tests passed**, including localhost multiplayer regressions and the explicitly enabled 892-card / 1,784-scenario route diagnostic. After the final distinction between a fresh deployment and recovery in an occupied slot, all **eight entry-hazard regression groups** and the diagnostic passed again. These groups cover casts, permissions and invalid dice; recasts and overlapping scopes; repeat lifecycle calls; genuine switches and re-entry; fixed and typed damage; Boots, Ground and indirect-damage immunity; live Gravity; sourced stat protections and reactions; caster departure; reflection and the atomic absent-source GM boundary; Defog/Tidy Up/Mortal Spin; save roundtrips, rollback and missing historical flags.

**74 frontend tests, 53 script/catalog checks, TypeScript, Rust formatting and diff checks passed.** The final regenerated audit also passed all ten move-audit checks. The catalog and five existing species comparison reports plus both closure records were refreshed for the new rules fingerprint. No Clippy, native UI, app packaging or physical LAN/VPN acceptance was run. MOVE-A02 is complete at this documented scope; 314 other fallback routes remain.


### MOVE-A03: basic Blocking (4 November 2026)

Anchor Shot, Block, Jaw Lock, Mean Look, Snap Trap, Spirit Shackle and Thousand Waves use the exact source records in [move-completion.json](../rules/move-completion.json). RULING 76 explicitly prevents escape, recall, switching and Switcher effects, including forced replacement. Run Away is immune. Physical damage immunity, shields, decoys and source reflection reuse the ordinary pipeline; Support moves do not gain an invented blanket Ghost-type escape exemption.

Ordinary scene-long target blocks survive source departure; target departure/fainting ends the target's effect through the existing lifecycle. Jaw Lock records both participants and releases the pair when either departs or faints, following its printed description and the necessary absence of a maintained grip. Each target of Thousand Waves resolves independently. The GM retains an explicit switch override; pending Switcher requests are revalidated.

Snap Trap: supplied JSON `OngoingDamage: true` conflicts with MASTER printed 479 / PDF 346. The full printed card was rendered and visually checked: prose and icons show Blocking plus scene duration, with no ongoing damage. An application-owned Attributes correction removes that unsupported flag; upstream JSON is unchanged.

New 0.38.0-core encounters adopt MOVE-A03; prior 0.37.0-core retains MOVE-A02. Serialized missing completion state adopts no blocks. App 0.46.0 / protocol 47. Further block work is tracked in [the completion ledger](MOVE-COMPLETION-BLOCKS.md).


MOVE-A03 verification: all **824 Rust tests passed**, including the diagnostic and localhost/3v3 regressions with the boxed special-effect state. **74 frontend tests, 54 script/catalog checks, TypeScript, Rust formatting and diff checks passed.** After resolving the generated-name collision with item completion, both move-state and TurnFlow binding exports passed and TypeScript was checked again. Inventory: **534 executable scopes, 307 manual-effect routes**. No Clippy, app packages, native UI or physical-network acceptance was run.


### MOVE-A04: standard ongoing damage (4 November 2026)

Bind, Clamp, Fire Spin, Infestation, Sand Tomb, Thunder Cage, Whirlpool, Wrap, Ceaseless Edge and Stone Axe now install the source-defined four-round effect. RULING 80 supplies two typeless damage dice at round end, ignoring defenses; the first active ongoing source takes priority and later sources do not stack or wait in a hidden queue. Leech Seed is its own named damage/drain effect, not a source of the Ongoing Damage icon.

Each round's roll is host-generated, source-owner/GM controlled, end-phase only and once per round. Both the ongoing clock and any separate Blocking clock expire after four ticks. Casts do not replay attack damage; ongoing rolls do not gain Chance-die modifiers or drain HP. Indirect-damage immunity remains live. The timed ongoing effect survives caster departure; Clamp's maintained paired grip releases on either participant's departure/fainting. Mortal Spin clears ongoing traps and their paired counterpart while retaining unrelated scene-long blocks. Source-defined increases to Evasion difficulty remain an explicit storyteller choice through existing GM roll modifiers.

The move-effect controls now show readable Blocking/ongoing records and offer a due damage roll only to the source owner or GM, including a retired caster. Save/reconnect and rollback retain the pending roll clock. Old MOVE-A03 encounters do not adopt this block. App 0.47.0, engine 0.39.0-core, protocol 48. Magma Storm and Salt Cure's nonstandard damage clauses are the next separate block.


MOVE-A04 verification: all nine MOVE-A03/A04 regression groups passed, plus the legacy Leech Seed, Mortal Spin cleanup, engine-adoption and two field round-clock regressions. All 125 binding exports passed serially. The 892-card / 1,784-scenario diagnostic was regenerated. **76 frontend tests, 54 script/catalog checks, TypeScript, Rust formatting and diff checks passed.** Inventory: **544 executable scopes, 297 manual-effect routes**. No packages, Clippy, native UI or physical-network acceptance was run.


### MOVE-A05: ongoing damage exceptions (4 November 2026)

Magma Storm installs four-round Blocking and a three-die Fire-type ongoing roll. Salt Cure installs four-round two-die typeless ongoing damage, makes initial damage super-effective against Steel/Water targets, and adds one fixed ongoing damage against either type (not one die, and not twice for Steel/Water together). The fixed extra still applies if both ongoing dice fail. Both reuse first-source priority, cleanup, end-phase permission checks and saved clocks from MOVE-A04.

Magma Storm's typed roll retains live Fire immunity and indirect-damage protection. Salt Cure's extra uses current typing at the ongoing resolution; its initial super-effectiveness enters the existing type pipeline before the retained weather/Tera Shell precedence. Earlier MOVE-A04 encounters retain ordinary Salt Cure typing and their missing effect route. App 0.48.0, engine 0.40.0-core, protocol 49.


MOVE-A05 verification: all 12 move-completion regression groups passed, including the new typed-pool, fixed-extra, live-type and historical-typing checks. Legacy Leech Seed, Mortal Spin, engine adoption, field clocks and all 125 serial binding exports passed. The 892-card diagnostic was refreshed. **76 frontend tests, 54 script/catalog checks, TypeScript, formatting and diff checks passed.** Inventory: **546 executable scopes, 295 manual-effect routes**. No packages, Clippy or native/physical-network acceptance was run.


### MOVE-A06: ordinary attack completion (4 November 2026)

68 exact-name Physical/Special cards now finish automatically after damage when they have no native effect beyond the existing accuracy, damage, reaction, range, targeting and printed-attribute machinery. Held-item Chance effects retain their resolution stage. This is a source-backed exact-name list, not a generic prose heuristic at runtime. No upstream move cards changed. Historical encounters retain their earlier completion routes. App 0.49.0, engine 0.41.0-core, protocol 50.

Validation: all 68 attacks exercised through public accuracy/damage commands; historical routing and Razor Fang Chance resolution checked separately. All completion regressions, adjacent Leech Seed/Mortal Spin/field clocks, engine adoption and 125 binding exports passed. The 892-card/1,784-scenario route diagnostic, frontend/script suites, TypeScript, formatting and diff checks are refreshed for this block.


### MOVE-A07: remaining ordinary damaging stat clauses (4 November 2026)

Leaf Storm, Liquidation, Low Sweep, Lumina Crash, Lunge, Metal Claw, Meteor Mash, Moonblast, Mud Shot and Mystical Fire now reuse the MOVE-A01 exact-pool/stat-channel implementation through saved block adoption. This preserves source-owned self changes, six-only Chance success, Serene Grace/Loaded Dice/Wonder Skin, Shield Dust, Contrary/Simple, reflection and stat protection. Earlier encounters gain none of these definitions automatically. App 0.50.0, engine 0.42.0-core, protocol 51.

Validation covers all ten cards through public commands, pool/authority rejection, recipients, failed Chance rolls, unchanged HP/sheets at effect resolution, protection and historical behavior. Completion regressions, engine gates, adjacent round/status checks, binding exports, full route diagnostic, frontend/script suites, TypeScript and formatting are refreshed.


### MOVE-A08: Support stat cards (4 November 2026)

Meditate, Growth, Gear Up, Magnetic Flux, Flower Shield, Rototiller, Curse (Non-Ghost) and Coaching now apply their printed stat changes. Growth uses the active sunny-weather context; type restrictions use current battle typing. The two battlefield moves affect qualifying active Grass Pokémon on both sides. Coaching requires allies and excludes its user, with matching target-selection guidance. Non-Ghost Curse has no effect for a Ghost user.

Meditate's exact source prose says Special and Sp. Defense; its structured StatChanges incorrectly names Strength. The application-owned definition follows the prose and retains the exact source/hash discrepancy in this registry scope. Permanent sheets and upstream cards are unchanged. App 0.51.0, engine 0.43.0-core, protocol 52.


MOVE-A08 validation: public self/ally/battlefield commands cover every card, type exclusions, sunny Growth, Ghost-user rejection, Coaching target legality and multiple selected recipients; permanent sheets remain unchanged. All completion/stat regressions, adjacent status/field clocks, historical gates, binding exports, the 892-card route inventory, frontend/script tests, TypeScript, formatting and diff checks are refreshed.


### MOVE-A09: defense, critical and weather attacks (4 November 2026)

Darkest Lariat and Sacred Sword ignore defenses. Psyshock, Psystrike and Secret Sword use Defense throughout damage calculation and target-defense ordering while retaining Special category and the printed attacking attribute. Frost Breath, Storm Throw, Wicked Blow and Flower Trick retain the existing AlwaysCrit machinery. Weather Ball retains the existing six-weather type mapping and two bonus dice. All ten finish after damage while retaining optional item effects; earlier encounter gates preserve prior routes and defense calculations. App 0.52.0, engine 0.44.0-core, protocol 53.

Validation exercises all ten through accuracy and damage, guarantees criticals at one net success, compares different physical/special defenses and historical states, and checks all ordinary weather mappings and exactly two additional Weather Ball dice. Completion/stat regressions, adjacent status/field checks, engine gates, binding exports, route inventory, frontend/script tests, TypeScript and formatting are refreshed.


### MOVE-A10: Accuracy-success reductions (4 November 2026)

Flash, Kinesis, Sand Attack, Smokescreen, Mud Slap, Spacial Rend, Leaf Tornado, Mud Bomb, Muddy Water, Night Daze and Octazooka remove Accuracy successes without reducing pool dice. The strongest scene-local reduction applies; repeated or different losses do not add together. Spacial Rend removes five. The five Chance cards retain their exact pools, six-only success, Serene Grace/Loaded Dice/Wonder Skin and Shield Dust. Keen Eye remains a live success-removal immunity; Clear Amulet blocks incoming reductions and Mirror Armor reflects them. Shields, decoys, type/added-effect immunity and Sheer Force remain in the effect pipeline.

Saved effects use the shared lifecycle and show readable move/penalty labels. Normal Accuracy, move reactions and tactics reactions use the penalty; action difficulty and printed low-accuracy penalties remain distinct. Narrative tracking/obscuring clauses remain with the GM. App 0.53.0, engine 0.45.0-core, protocol 54.

Validation covers all eleven public command paths, exact pools and six-only success, unchanged dice counts with fewer successes, strongest-only behavior, live Keen Eye, save/departure cleanup, Mirror Armor, Clear Amulet, Sheer Force and independent Muddy Water target pools. Completion/stat regressions, adjacent status/field checks, gates, exports, route inventory, frontend/script tests, TypeScript and formatting are refreshed.


### MOVE-A11: explicitly stacking stat effects (5 November 2026)

Sharpen, Syrup Bomb, Luster Purge and Mist Ball now maintain named scene-local counters capped at three applications. The two Chance cards use five dice and require a six; failed Chance rolls do not advance the counter. Counter updates happen after stat protection and reflection. Ordinary changes from different moves still use the existing strongest-increase/strongest-reduction channels. Simple applies its additional point to each one-point stack; Contrary reverses the result, and copied stat snapshots retain the effective stacked value.

Counters survive saving, expire on target departure, and clear with temporary modifiers. Permanent sheets and source cards remain unchanged; older encounters retain the fallback route. App 0.54.0, engine 0.46.0-core, protocol 55.

Validation covers every card through four casts, exact Chance pools/failure, the three-stack cap, Clear Body, Mirror Armor, Contrary, Simple per increment, serialization, clearing and historical adoption. Completion/stat and stat-transfer regressions, adjacent status/field clocks, gates, exports, the full route inventory, frontend/script suites, TypeScript and formatting are refreshed.

The 19 stat-channel regressions, including the localhost room/reconnect/persistence/rollback case, also passed with localhost sockets enabled after the sandbox-only Windows 10013 failure.


### MOVE-A12: spinning field cleanup (5 November 2026)

Rapid Spin raises the user's Dexterity by one and removes its ongoing damage and Leech Seed, plus entry hazards and terrains affecting its side. Pure scene Blocking remains. Shared/global field scopes are narrowed to preserve effects on opposing sides; the same corrected cleanup applies to Mortal Spin only in encounters adopting this block. Ice Spinner and Steel Roller remove all active terrain scopes; Steel Roller requires an active terrain at declaration. Cleanup still resolves after the target faints. Immunity/protection and Sheer Force remain in the effect pipeline.

Validation covers all three public attack paths, Steel Roller's declaration prerequisite, ongoing/Leech Seed removal, retained scene blocks/screens/opposing hazards, narrowed global terrain/hazard scopes, target fainting, immunity, Sheer Force and historical fallback. Completion/stat/transfer regressions, adjacent Mortal Spin/Leech Seed/field clocks, gates, exports, the full route inventory, frontend/script tests, TypeScript and formatting passed. App 0.55.0, engine 0.47.0-core, protocol 56.


### MOVE-A13: source-type and context damage (5 November 2026)

Ivy Cudgel and Raging Bull use the user's current secondary type, falling back to the main type. Multi-Attack and Revelation Dance use the current main type. Their resolved move type enters the existing STAB, resistance, immunity and ability-conversion pipeline using the encounter's current/frozen typing cache. Terrain Pulse adopts the applicable active terrain's type and gains two dice. Psyblade gains two dice in applicable Electric Terrain. Acrobatics loses two pool dice while actually holding an item, regardless of item-effect suppression. Existing damage snapshots retain their behavior.

Validation exercises all seven attacks through public commands, dual/single typing, live GM-confirmed typing changes, STAB and effectiveness, all four terrain mappings, side scope, exact bonus/penalty dice, held-item presence and historical behavior. Completion/stat/transfer regressions, adjacent status/field clocks, gates, exports, route inventory, frontend/script tests, TypeScript and formatting passed. App 0.56.0, engine 0.48.0-core, protocol 57.


### MOVE-A14: self-damage, stats and Blocking (5 November 2026)

Belly Drum rolls half the user's current HP as typeless self-damage dice. Clangorous Soul rolls half total HP, rounded down. Fillet Away deals two fixed self damage. These are damage pools, not Chance pools; Serene Grace and Wonder Skin do not alter them. Their printed stat changes use the existing channels, and indirect-damage protection remains active. No Retreat raises its five printed stats and Blocks its user; Octolock reduces Defense/Sp. Defense and Blocks its target. Run Away blocks only the trapping clause, and stat protection blocks only the stat clause.

Validation covers every card, odd/current/total HP, zero-die Belly Drum, exact pool rejection, ordinary damage successes, failed damage dice, Magic Guard, every stat recipient, Run Away, Clear Body, sheets and historical fallback. Completion/stat/transfer regressions, adjacent status/field clocks, gates, exports, route inventory, frontend/script tests, TypeScript and formatting passed. App 0.57.0, engine 0.49.0-core, protocol 58.


### MOVE-A15: Support, Sound and shared-move restrictions (5 November 2026)

Taunt prevents Support Pokémon moves for four rounds while preserving Typeless Support Maneuvers. Throat Chop prevents Sound moves for the scene and persists through recall/switching, ending when its target faints. Imprison prevents foes from using moves currently known by its active caster; it ends on caster departure or fainting. Restrictions enter the ordinary accuracy/reaction validation path and retain existing immunity, reflection and protection. The GM can explicitly clear them through the existing authorized status-removal action; the UI shows readable restrictions, clocks and GM-only clear controls.

Validation covers all three public effect paths, legal/illegal categories, four-round expiry, dynamic known moves, public switching, serialization/faint cleanup, Aroma Veil, GM/intruder authority and historical fallback. Completion/stat/transfer regressions, adjacent status/field clocks, gates, exports, route inventory, frontend/script tests, TypeScript and formatting passed. App 0.58.0, engine 0.50.0-core, protocol 59.


### MOVE-A16: saved move history and round restrictions (5 November 2026)

Disable selects the target's last recorded Pokémon move, replaces any prior disabled move on that target, and rejects declaration when no prior move exists. Torment dynamically prevents the target's previous-round moves for four rounds. Blood Moon and Gigaton Hammer reject use after any previous-round attempt, including a failed Clash, and become usable after a gap. Gigaton Hammer's optional Steel-target bonus remains an explicit GM damage-pool modifier with a reason.

Accepted Pokémon move attempts and actual deployment rounds are saved in the completion state and restored by undo. Typeless Maneuvers are separate. Old states default empty and historical encounters adopt none of these rules. Disable is correctly classified as input-required in the deliberately history-free diagnostic, rather than pretending its prerequisite exists. First Impression was deferred at this block and is delivered in MOVE-A24 using the user-confirmed first eligible action round. Normal switches deploy now but prohibit acting until the following round.

Validation covers all four moves through public commands, declaration prerequisites, replacement, moving previous-round sets, failed Clash history, gaps, serialization, undo, missing historical state and actual switch deployment/availability clocks. Completion/stat/transfer regressions, adjacent status/field clocks, gates, 126 exports, route inventory, frontend/script tests, TypeScript and formatting passed. App 0.59.0, engine 0.51.0-core, protocol 60.


### MOVE-A17: Shield and Force Field bypass attacks (5 November 2026)

Hyper Drill, Hyperspace Hole, Mighty Cleave and Hyperspace Fury retain their printed Shield bypass. Mighty Cleave now also bypasses Force Fields; Hyperspace Hole/Fury retain their existing Force Field behavior, and Hyper Drill still takes Force Field reduction. Hyperspace Hole ignores defenses. Hyperspace Fury reduces its user's Defense by one after damage, including a target knockout. Other environment/weather reductions remain independent, and optional held-item effects retain their resolution stage.

Validation compares all four public attacks with and without Protect plus both defense screens, verifies Fury's self change, Hole's zero defense, retained environment reduction, knockout resolution and historical Mighty Cleave behavior. Completion/stat/transfer regressions, adjacent status/field clocks, gates, exports, route inventory, frontend/script tests, TypeScript and formatting passed. App 0.60.0, engine 0.52.0-core, protocol 61.


### MOVE-A18: range, typed immunity and grounding (5 November 2026)

Earthquake, Gust, Sky Uppercut and Surf complete with their existing specific charge vulnerabilities. Freeze Dry now resolves Dive's source spelling Freeze-Dry to the canonical card, retains the printed Dive vulnerability, and deals super-effective damage against Water targets. Both source fingerprints are retained. Origin Pulse and Precipice Blades can hit targets hidden by charging moves and bypass immunity to their printed Water/Ground damage type; weather activation guards, Shields and Force Fields remain independent.

Smack Down and Thousand Arrows remove the target's Ground-move immunity for the scene, including Flying/Levitate and other Ground-move immunity. Thousand Arrows bypasses Ground immunity on its initial hit. Marked targets satisfy the existing grounded-field checks; recall/faint clears the mark. The effect does not invent a charge cancellation. Shields still prevent the grounding clause, and the UI shows the live Ground-immunity removal.

Validation exercises all nine public attack paths, specific/all hidden charges, unrelated hidden-target rejection, three Water dual-type cases, typed/ability immunity, follow-up Ground attacks, public recall, shields and historical behavior. Focused completion/stat/transfer/status/field/gate checks, 126 bindings, full route inventory, 76 frontend tests, 54 script checks, TypeScript and formatting passed. The full Rust suite also passed with localhost networking enabled: 878 passed, one ignored inventory test; the ignored inventory passed separately. Generated bindings were normalized and TypeScript/format/diff checks passed afterward. App 0.61.0, engine 0.53.0-core, protocol 62.


### MOVE-A19: completion of existing item and ordinary mechanics (5 November 2026)

Cut and Strength now finish their ordinary battle damage. Outside-battle obstacle/lifting narration and lethal wounds remain excluded. Strength keeps the M4-reviewed canonical name plus the exact trailing-space legacy identity, with an explicit fingerprint-backed runtime alias.

Fling and Natural Gift retain their existing authoritative item preparation, physical-unit spending, rarity/flavor/Ripen calculations, hit effects, protections, ownership, privacy and rollback. A valid committed item move now finishes after damage; the phase-only diagnostic, which bypasses item/accuracy commitment, correctly requests that missing context. Unarmed Fling remains valid. This block reconciles existing mechanics and completion routes rather than claiming the item subsystem is new.

Validation covers canonical/legacy Strength and Cut, item preparation rejection, selected/consumed physical units, valid unarmed Fling, automatic completion and prior encounter stages. Completion/stat/item-move/transfer regressions, adjacent status/field clocks, gates, exports, route inventory, frontend/script tests, TypeScript and formatting passed. App 0.62.0, engine 0.54.0-core, protocol 63.


### MOVE-A20: partner attacks and final evolution (5 November 2026)

Baddy Bad and Glitzy Glow create four-round own-side Physical/Special Force Fields after actually dealing damage. Pika Papow adds zero to five Happiness dice; Zippy Zap retains its guaranteed Critical and Reaction 2. The final-evolution guard follows forward ordinary evolution edges, excluding From, Mega, Form and same-Dex edges. This also corrects the guard on Bouncy Bubble, Veevee Volley and Sparkly Swirl in newly adopting encounters. Unknown evolution identities require explicit review.

Baddy Bad's raw JSON Target says Ally's Battlefield, while its Effect explicitly says Single Target and describes damage. The application-owned block resolves it as a Foe attack; source data is preserved, and advanced targeting plus accuracy previews use the corrected target. Historical encounters retain their prior targeting and evolution behavior.

Validation covers all four public attacks, field clocks/category/side, zero damage, Sheer Force, Happiness bounds, exact one-success Critical, Reaction priority, eight species evolution cases and historical guards. All 66 completion tests, adjacent stat/item/targeting/field regressions, bindings, route inventory, 76 frontend tests, 54 script checks, TypeScript and formatting passed. The localhost targeting regression passed with socket access after sandbox Windows 10013. App 0.63.0, engine 0.55.0-core, protocol 64.


### MOVE-A21: timed side and battlefield Support (5 November 2026)

Tailwind gives two Dexterity to active allies on the user's side for four rounds, including later entrants. It shares the strongest-positive/strongest-negative move stat channel with Agility and copied boosts. Simple, Contrary and external Clear Body rules apply. The field persists without its caster, and effective stat transfer captures its value once. Fairy Lock Blocks both sides and new entrants for one round; Run Away and explicit GM switch overrides retain their behavior. RULING p.77 prevents an active timed field from restarting its counter on recast. Historical encounters keep manual resolution.

Validation covers both public Support paths, exact side/global scopes, effective sheets, non-additive buffs, ability channels, transfer, serialization, public switching, new entrants, caster departure, recast and expiry. All 70 completion tests and adjacent regression groups, bindings, route inventory, 76 frontend tests, 54 script checks, TypeScript and formatting passed. The final recast correction then passed all four block tests again. App 0.64.0, engine 0.56.0-core, protocol 65.


### MOVE-A22: Spider Web entry Blocking (5 November 2026)

Spider Web installs a non-stacking scene-long hazard on the foe's side. RULING p.77 limits entry hazards to newly entering Pokémon; current occupants are unaffected. An entrant receives ordinary scene Blocking. The card has no Ground-immunity predicate, so Flying/Levitate do not bypass it. Heavy-Duty Boots prevent entry effects, and Run Away prevents Blocking. The field and applied blocks persist after caster departure. Recall/faint ends a target's applied block.

Rapid Spin, Mortal Spin, Defog and Tidy Up now remove this adopted hazard, preserving existing side/scope rules. Hazard removal does not retroactively cure a previously applied scene block. Historical encounters retain manual Spider Web and existing cleanup semantics.

Validation covers public casting and switching, both sides, current occupants/new entrants, repeated casting, scene persistence, caster departure, serialization, recall, GM override, Boots, Run Away, Levitate, all four cleanup moves and historical adoption. All 73 completion tests, eight existing entry-hazard tests, adjacent regressions, bindings, route inventory, 76 frontend tests, 54 script checks, TypeScript and formatting passed. App 0.65.0, engine 0.57.0-core, protocol 66.


### MOVE-A23: ordinary attacks and terrain Reaction (5 November 2026)

Behemoth Bash, Behemoth Blade and Dynamax Cannon now finish ordinary battle damage, retaining their existing printed attributes. Dynamax/Gigantamax bonuses and lethal wounds remain explicitly excluded. Grassy Glide now has Reaction 1 only while Grassy Terrain applies to its user; raw metadata previously granted it unconditionally. The same resolved metadata enters public reaction validation. Historical encounters retain prior behavior.

Validation covers all four public damage paths, exact base Power, completion and historical fallback; live Grassy Terrain side scope, expiry, public Reaction acceptance/rejection and priority. All 75 completion tests, entry-hazard/targeting/stat/item/field regressions, bindings, route inventory, 76 frontend tests, 54 script checks, TypeScript and formatting passed. A final audit-document newline issue was normalized before the final diff check. App 0.66.0, engine 0.58.0-core, protocol 67.


### MOVE-A24: First Impression eligibility (5 November 2026)

The user confirmed that First Impression works during the first round the deployed user can act. Its guard compares the current round with max(saved deployment round, available_round). Normal switch-ins therefore get their opportunity next round. Waiting beyond that eligible round does not postpone the opportunity; re-entry starts a new deployment window. The restriction applies equally to ordinary attacks and Reaction 2, and does not invent a once-per-deployment use limit.

Initial flow creation now records starting participants as well as subsequent deployments. Entry history is saved and restored with battle state; a missing entry requires review rather than silently guessing. Historical encounters retain their prior behavior. The history-free route diagnostic requests the missing context explicitly.

Validation covers starting participants, normal switching, first eligible versus deployment rounds, missed windows, re-entry, Reaction 2, serialization, missing history, initial flow observation and historical fallback. All 78 completion tests, adjacent regressions, bindings, route inventory, 76 frontend tests, 54 script checks, TypeScript and formatting passed. The full Rust suite passed with localhost networking: 897 passed, one ignored route inventory test; that inventory passed separately. Generated bindings were normalized and TypeScript/format/diff checks passed afterward. App 0.67.0, engine 0.59.0-core, protocol 68.


### MOVE-A25: Aura Wheel completion (5 November 2026)

Aura Wheel now raises its user's Dexterity by one through the existing damaging-stat channel, including after a target knockout. Hunger Switch retains its existing Full-Belly Electric and Hangry Dark move typing; no alternate catalog form is invented. Simple, Contrary and the explicit Sheer Force choice retain their behavior. Historical encounters retain manual effect completion.

Validation covers both forms through public attacks and actual damage pools, target knockout, exact user stat change, Simple, Contrary, Sheer Force and historical fallback. All 80 completion tests and adjacent stat/item/entry-hazard/targeting/field regressions, bindings, route inventory, 76 frontend tests, 54 script checks, TypeScript and formatting passed. App 0.68.0, engine 0.60.0-core, protocol 69.


### MOVE-A26: Mud Sport and Water Sport (5 November 2026)

Both cards create non-refreshable four-round global fields. Damaging Electric moves under Mud Sport and Fire moves under Water Sport have Power zero and no positive effectiveness. This preserves resistance, immunity, STAB and independent extra dice; zero Power does not imply zero total damage. The final resolved move type is used after ability/item/form conversions. Electro Drift's super-effective bonus also respects the suppressed effectiveness. Analytical type-chart queries remain unchanged.

Damage-linked status, stat, item and ability checks now share the sport-adjusted matchup. The legacy Thunderbolt/Thunder Punch Chance path and guaranteed Nuzzle/Zap Cannon status path also respect it, preventing status application when Wonder Guard blocks the sport-adjusted attack. Historical encounters retain prior behavior.

Validation covers public field/attack paths, Physical/Special pools, exact zero Power, retained other dice, positive/negative/immune matchups, Galvanize conversion, Electro Drift, Wonder Guard versus Chance/guaranteed status and stat effects, Support separation, recast, expiry and historical scope. All 84 completion tests and adjacent regressions, bindings, route inventory, 76 frontend tests, 54 script checks, TypeScript and formatting passed. The full Rust suite passed with localhost networking: 903 passed, one ignored inventory; the inventory passed separately. Generated bindings were normalized and TypeScript/format/diff checks passed afterward. App 0.69.0, engine 0.61.0-core, protocol 70.


### MOVE-A27: Tar Shot and frozen Fire vulnerability (5 November 2026)

Tar Shot reduces Dexterity by one through ordinary stat protection/reflection and independently makes damaging Fire moves super-effective against its target for the scene. Clear Body blocks the stat reduction, not the Fire vulnerability; Contrary, Simple and Mirror Armor retain their stat behavior. Ability immunity remains, and Water Sport still prevents positive effectiveness. Support immunity prevents both clauses. Recall/faint clears the mark.

The damage snapshot now optionally preserves Fire vulnerability, with a default for older snapshots. Marked Fire attacks using legacy single-target commands also capture this context. A knockout therefore cannot clear the live mark and retroactively block an attack's user boost through Wonder Guard. New snapshots use the resolved move type for field bonuses and restrict conditional bonus calculation to actual recipients. Historical encounters retain prior behavior.

Validation covers public casting/damage, independent stat/Fire clauses, four stat abilities, Support immunity, Flash Fire, Water Sport, recall, serialization, knockout plus Flame Charge's user boost, and converted Ivy Cudgel/Weather Ball damage previews in sunlight. All 89 completion tests, adjacent regressions, bindings, route inventory, 76 frontend tests, 54 script checks, TypeScript and formatting passed. App 0.70.0, engine 0.62.0-core, protocol 71.
