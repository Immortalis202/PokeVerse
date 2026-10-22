# Standalone battle abilities

Application **0.28.0**, engine **0.21.0-core**, protocol **29** delivers ordered milestone 4. Schema remains 4. New encounters enable `TurnFlow.standalone`; old saved encounters default this state to disabled.

## Evidence and explicit rulings

The supplied v3.0 ability cards and MASTER printed pages 549 (Heavy Metal), 553 (Light Metal), 556 (Moody), 559 (Perish Body), 564 (Rivalry) and 574 (Torrent) were reviewed. Supplied JSON/PDF files are unchanged.

Heavy/Light Metal explicitly leave the weight increase/reduction to the Storyteller. Descriptions give examples, not a compulsory double/half multiplier. The GM records ordinary and modified kg for the exact current form, with finite positive values at most 100000 kg. Heavy must increase and Light must decrease that form's ordinary weight. Suppression uses ordinary weight; another form needs its own reviewed metal weight. Without an explicit ordinary override, existing recorded special-move weight parameters or the current species/form catalog kg are used. Missing values require a ruling. These inputs never rewrite library sheets or the species catalog.

The four reviewed weight moves retain their literal supplied formulas: Low Kick and Grass Knot add floor(target kg / 50), clamped to 0–4 dice; Heavy Slam and Heat Crash add floor((user kg - target kg) / 50), clamped to 0–4. They use kg differences, not video-game weight ratios. Shared damage retains its saved special-move bonus after later GM input changes. Other unreviewed weight-changing effects, including the weight portion of Autotomize, remain GM-managed through encounter inputs; implementing these two abilities does not complete the move catalog audit.

Rivalry's supplied JSON says same/opposite **gender**; printed page 564 instead says **gender preference**. The GM records which wording governs the encounter. Individual profiles record gender and a separate pair of comparison groups for the printed preference interpretation. No species gender ratio, name, appearance or owner identity assigns a gender. Unknown, genderless and neutral cases grant no automatic change. The GM can revise a profile with a reason, without changing the permanent sheet.

Torrent's printed text and supplied JSON grant **two** Water damage dice at half HP or less, while the printed icon grants **one**. The GM must choose one of those values in the encounter. No default silently settles the conflict. Water accuracy/damage pain immunity applies at every HP level; Water Support moves do not need a damage-bonus choice. The extra damage-pool dice apply only at half HP or less. Existing literal fixed/formula and neutral Clash damage rules are retained; extra dice are not converted into invented flat damage.

These four entries retain partial status for their printed choices and broader unreviewed interactions. Their reviewed battle behavior and controls are released. Moody and Perish Body are automated within the scopes below.

## Rivalry and saved attacks

Against a foe in the same group, Strength increases by one; against the opposite group it decreases by one. Ally/self targeting is neutral. The normal strongest-positive/strongest-negative ability channels, attribute caps, Contrary/Simple behavior and supported Unaware damage handling are respected. A slash attribute choice is recalculated after changing Strength, rather than automatically adding a die when another attribute remains higher.

For an attack with mixed comparisons, the host rolls the largest needed accuracy pool once. Each target uses the appropriate prefix, including a shared Chance die after the largest pool, its own success/critical result and the normal shared action expenditure. Supported Strength-based reaction and Clash pools use the opposing participant. Ordinary damage changes Strength before the minimum damage-pool calculation. At the first shared damage roll, effective target-specific Rivalry deltas, the effective Torrent bonus (including zero above half HP), source ability/stat/resource snapshots and existing weight bonuses are saved. Later profile/ability changes do not change already saved inputs. Permanent sheets remain unchanged.

## Moody

At round end, reset only that source's own prior Moody grants. Existing unrelated grants and copies with other provenance remain. The host samples an ordered pair from five attributes: Strength, Dexterity, Vitality, Special and Insight. One decreases by one and another distinct attribute increases by one.

Two uniform d6 yield values 0–35. Values 0–19 map one-to-one to the 20 distinct ordered pairs; values 20–35 are rejected. A rejection is an accepted, saved command with recorded dice and a pending reroll, rather than a silent reroll or an error that discards evidence. Retrying the same command ID retains its receipt; a fresh roll needs a new command. No battle action is spent. All 20 pairs have equal probability, and the increase is uniform over the four attributes other than the decrease.

The result uses normal ability channels and caps, supports Contrary/Simple, and is visible to existing copying/provenance rules. Source suppression, loss, fainting or recall clears its own Moody grants and obsolete pending decisions; copies with distinct provenance remain. Round-end processing is idempotent, so resolving other pending checks does not reset/reroll Moody twice in the same round. A generic manual close is GM-only.

## Perish Body

A landed hostile non-ranged Physical hit curses the attacker, including a hit that faints the holder. Long Reach/ranged/special hits, protected decoy subjects, supported damage immunity and attack-scoped bypass do not arm the curse. Repeat hits preserve the earliest existing deadline rather than extending survival.

The reviewed timing is three full rounds following the hit: a hit in round R causes remaining-HP ability damage at the end of round R+3. The panel shows the exact deadline. Existing round-end condition, field and item work runs before this deadline check; due Perish damage resolves before new Moody choices. Departure or fainting of the cursed attacker clears the countdown. The emitting holder's later departure/suppression does not remove an already armed curse. Magic Guard/Wonder Guard follow the existing ability-damage immunity policy. The remaining-HP hit bypasses ordinary move-only survival/ice/disguise pools, and can trigger normal witnessed fainting/lifecycle processing. No lethal wounds are inflicted.

## Authority, persistence and compatibility

Profile, Rivalry wording and Torrent choices are GM-only, require a reason, cost no action and create normal battle checkpoints. Owners resolve their Moody rolls. The panel exposes rulings, exact kg, pending Moody sampling, Perish deadlines and recorded history. Room commands use host dice, normal stable IDs, revisions, duplicate receipts, persistence, reconnect and rollback. New state and new DamageContext fields have safe serde defaults. Private information/held-item projections continue to apply.

Engine 0.20 retains private battle information and all previous gates; engine 0.19 retains transformations, 0.18 held items and earlier mechanics. No active historical battle is upgraded silently. Current rules can be adopted between encounters. All room clients need matching protocol/build/catalog fingerprints.

## Verification

Release checks for application 0.28.0:

- **614 Rust checks passed:** 108 binding exports and 506 behavioral tests. The final full suite includes 13 standalone fixtures: exact weight thresholds/direction, Rivalry wording/caps/shared prefixes and frozen deltas, explicit Torrent threshold/pain/zero snapshots, all 20 unbiased Moody pairs and recorded rejections, reset/suppression provenance, Perish timing/killing hits/ranged guards/departure and ability-damage immunity, historical defaults and real localhost permissions/retry/reconnect/persistence/rollback.
- **57 frontend tests** and **15 catalog checks** passed, along with TypeScript, Rust formatting, warnings-denied Clippy and diff checks.
- Windows production build and NSIS packaging passed: `src-tauri/target/release/bundle/nsis/PokeVerse_0.28.0_x64-setup.exe` (**214.35 MiB**). The final installer includes the verified effective Torrent snapshot correction.

Native interaction, installed upgrade/data preservation, trusted signing and physical LAN remain separate milestone 7 acceptance. Neither executable nor installer is launched during packaging.
