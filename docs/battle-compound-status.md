# Compound and conditional move effects (M2-C)

Delivered in app **0.39.0**, engine **0.32.0-core**, protocol **40**. The 28 source cards and their fingerprints are preserved in [the registry](../rules/compound-status-moves.json). This implements their battle clauses; it does not certify every narrative or character-progression clause.

## Behavior

- Dire Claw and Tri Attack have three independent pools. The Fang moves have separate Flinch and ailment pools. Pool order follows printed prose, including Burn → Freeze → Paralysis for Tri Attack. Several ailments can trigger together. Serene Grace, Wonder Skin, Loaded Dice, weather, protection and immunity rules apply; a held-item Flinch pool merges without consuming the other ailment pool.
- Flatter, Swagger, Toxic Thread and Glaciate apply their guaranteed stat changes independently of condition immunity. Burning Jealousy requires an increased attribute. Infernal Parade adds two damage dice against a structured active condition.
- Fake Out is restricted to the first deployed round, including later field entries. The four evolution-restricted moves reject final-stage identities; Mega/Form links do not count as future evolutions. Upper Hand only adds Flinch when its recorded incoming action was itself a reaction.
- A failed Axe Kick accuracy roll, including a failed Clash, queues a mandatory four-die typeless crash roll against its user, ignoring defense. The owner or GM rolls it; no extra action is spent. It resolves before ordinary continuation and can still resolve when a switch is pending. State survives serialization and rollback.
- Venom Drench reduces Strength, Special and Dexterity when already poisoned, or watches for Poison applied later in the same round. Each recorded application fires once; an untriggered watch expires at the round boundary. GM structured status commands use the same trigger.
- Hydro Steam gains its printed two damage dice in Sunny weather and avoids the ordinary Sunny Water penalty. Shell Side Arm selects Physical/Strength/Brawl only when current Strength exceeds Special; otherwise it uses Special/Channel. Accuracy and damage both read this category choice.
- Mortal Spin poisons eligible targets and independently clears applicable hazards/terrains from the user's side and move-based ongoing effects/Leech Seed from the user. Poison resolves for every target before the cleanup, including under Misty Terrain; target immunity/protection does not suppress the separate cleanup. Opposing hazards and ordinary Burn/Poison ailments remain.
- Hypnosis can put Dark types to Sleep. Lovely Kiss uses missing Beauty points against the existing rank social cap (5; Champion 7), including a zero-die miss at the cap. No permanent Beauty or learning ledger is modified.
- Twister reuses the existing Fly/Bounce/Sky Drop vulnerability path and adds its printed two-die Flinch chance. Stomp adds its chance effect and reads recorded Minimize vulnerability, reusing the existing vulnerability bonus and reaction restrictions.

## Decisions and retained boundaries

The **Special moves** panel now includes conditional-move controls. The GM confirms all physical surroundings present. Secret Power automatically includes active terrain sources; the trainer chooses between concurrently available sources, as MASTER 530 explicitly permits. Missing surroundings or an unresolved choice produces a visible input requirement. Its eleven outcomes include the Water environment's Strength −2, not an invented Water ailment. Ordinary protection and Misty Terrain condition immunity remain authoritative, including when the table selects In Love.

After an unblocked successful Relic Song hit, the owner can keep the current form or switch Meloetta between its exact Normal/Psychic and Normal/Fight forms, after all targets finish. This treats “If successful” as successful move resolution, independently of the Sleep Chance result. Other users can keep their form; the automatic change is restricted to the two reviewed Meloetta identities. Existing form preparation, resource preservation and end-of-battle restoration apply. The choice is not automatic healing and never overwrites the library sheet.

Minimize's remaining move mechanics are still GM-adjudicated. Adjudicating its effect records its state for Stomp; the GM can explicitly confirm/clear that prerequisite in the new panel. This does not mark the whole Minimize card implemented. Lovely Kiss's optional permanent Beauty forfeiture and restoration on forgetting are character-progression work, outside this battle batch.

## Source review and compatibility

JSON supplies the ordinary effect definitions. Printed exceptions were compared at MASTER 459, 463, 491, 495, 499, 500, 508, 509 and 525. MASTER 530's environment table and MASTER 327's two Meloetta rows were rendered and visually checked.

The prior catalog vector extraction had copied Pirouette's 7/7/5/5/5 into base Meloetta. An application-owned correction now preserves base Meloetta's printed 5/5/5/7/7 and keeps the Pirouette identity **0648F1:meloetta-pirouette-form** distinct from **0648:meloetta**. Source JSON and existing permanent character sheets are unchanged; an old character with incompatible allocations still needs the existing form-preparation/GM correction workflow.

New current-engine encounters enable the defaulted `compound` state. Historical encounters retain their previous routes, flags and inputs. Source registry and form mappings are fingerprinted into coverage/catalog compatibility. The status queue now has ten cards remaining: six M2-D and four M2-E.

## Verification

Sixteen new Rust test groups exercise all 28 cards, independent pool slices, modifiers and item merging, permissions, crash/Clash/rollback and pending-switch ordering, evolution/entry guards, delayed poisoning, category/weather hooks, source choices, dark-type Sleep, cleanup and form/library preservation. Three frontend tests check control permissions and historical visibility. All 740 Rust tests passed: 623 runtime regressions (including existing localhost multiplayer checks) and 117 serial binding exports. The 892-card diagnostic also passed both profiles (1,784 isolated effect-phase scenarios, not full battle simulations). TypeScript, 68 frontend tests, all 29 script/catalog checks, Rust formatting and diff checks passed. Native UI, physical networking, Clippy and app packaging are not part of this run.
