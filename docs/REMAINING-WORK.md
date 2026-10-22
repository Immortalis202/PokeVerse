# PokeVerse - remaining work

Snapshot: **22 October 2026**, application **0.28.0**, engine **0.21.0-core**, protocol **29**, user database schema **4**.

This is the current project backlog. Implemented foundations are summarized first; unfinished features, explicit GM rulings, verification gaps and excluded scope are distinguished below. Counts come from the checked-in registries and should be refreshed after catalog changes.

## Complete item and ability inventory (22 October audit)

See the [complete item and ability roadmap](ITEM-ABILITY-ROADMAP.md) for the ordered deliverables and every catalog item. Its [ability inventory](ability-roadmap.csv) accounts for all 305 abilities; its [item inventory](item-roadmap.csv) accounts for all 236 items. This audit distinguishes released scope, disabled worktree implementations, genuinely missing mechanics, GM rulings and exclusions.

The registry records 234 automated, 68 partial and 3 manual abilities. The older tables in sections 8–9 reproduce the pre-release baseline; generic held-item and booster dependencies there are stale and must not override the newer audited worktree assessment. The reviewed held-item batch is released in 0.25.0; transformation batch 2 is delivered in 0.26.0; private-information batch 3 is delivered in 0.27.0; standalone batch 4 is delivered in 0.28.0. There are 30 unimplemented battle/species item definitions, 47 released Mega stones, eight missing medicine definitions and nine medicines with only reviewed Recycle effects. All 35 Z-crystals remain excluded; 10 items explicitly forbid battle use. Other reference-only entries require scope review before becoming battle tasks.

## Implemented foundation

Offline trainer/Pokémon creation and named progression use shared Rust budgets, costs, prerequisites and derived resources. Campaign synchronization supports accepted offline purchases, indivisible progression chains and scoped GM exception approval. The host appoints the GM and automatically validates normal commands; these roles can be separate people.

Training encounters use the reviewed habitat pool, private preparation, publication, assumed victory, captures and trainer assignment. The GM awards Training Points to a trainer pool; owners distribute accepted points among that trainer's Pokémon offline. Existing Pokémon balances are preserved.

Battles register full teams, three active slots per side, reserves, controlled switching, accuracy/damage automation, multi-target responses, field effects, supported abilities, decoys, overrides and rollback. Evasion is once per turn; eligible Clash can answer each incoming hit within the action budget. The Compendium works offline for all 1,200 species/forms.

Dancer, Parental Bond, Gorilla Tactics, Truant and Defeatist now use the reviewed [action-sequencing rules](battle-action-sequencing.md), including free copies, independent damage pools, entry move selection and per-action Loyalty checks. Unreviewed move effects remain explicit GM resolutions.

## 1. Remaining battle systems

- [x] **Held-item ability batch:** released in 0.25.0 with owned equipment, reviewed effects, dependent abilities, private decisions, rollback and synchronization. Remaining item definitions are batch 6; native acceptance remains batch 7.
- [x] **Forms and transformations (batch 2):** reviewed all 20 entries, exact temporary builds/types/abilities/moves, 47 Mega stones and Transform, permanent-sheet protection, scene restoration, persistent Minior Core memory, owner/GM decisions, frozen damage inputs and rollback. Printed ambiguities and absent item stock retain the explicit boundaries in [transformation rules](battle-transformations.md).
- [x] **Private battle information (batch 3):** Anticipation/Forewarn warnings, private GM comparisons and Insight rulings, linked-trainer recipients, movepool projections, transport, persistence and rollback. See [private-information rules](battle-private-information.md).
- [x] **Standalone abilities (batch 4):** Heavy/Light Metal weight inputs/formulas, Rivalry comparisons, Moody host sampling, Perish Body and explicit Torrent wording choice. See [standalone ability rules](battle-standalone-abilities.md).
- [ ] **Other ability families:** finish the remaining partial interaction scopes described in milestone 5. Tinted Lens, Unaware, Stakeout and the move portion of Sheer Force are implemented. The full ability inventory is in sections 8-9.
- [ ] **Unreviewed trait copying:** Costar and Opportunist now copy supported attribute/defense move and ability channels. Baton Pass transfers supported positive attribute channels, and Snatch steals supported ability modifiers. Review additional trait channels separately; temporary ability identity copying remains distinct. See [stat copying and transfer](battle-stat-channels.md).
- [ ] **Remaining identity effects:** review Entrainment, Doodle and Core Enforcer individually. Role Play, Skill Swap, Gastro Acid, Trace, Gas, contact replacement and witnessed ally inheritance already exist.
- [ ] **Status edge cases:** review severe Burn/Poison escalation with conflicting printed values, unsupported status-producing/cure effects and remaining Will spending/draining effects. Core statuses, durations and expiry already exist; automatic Will recovery remains excluded.
- [ ] **Ice edge cases:** review any still-adjudicated Snow/special-damage interactions against ice against current fixtures before adding mechanics. Ordinary ice attempts and supported special interactions already have tests.
- [ ] **Unreviewed move effects:** extend executable definitions beyond the selected special/targeting registries. Ordinary damage automation does not imply every added effect on all 892 moves is executable. Preserve explicit GM resolution for unsupported cards.

## 2. Remaining named move and battlefield boundaries

| Entry | Remaining work or intentional ruling |
| --- | --- |
| Baton Pass | Supported positive move and ability attribute channels and replacement rules are implemented. |
| Snatch | Review additional effects; supported move/ability modifiers, shields, decoys and typed effects already transfer. |
| Shadow Force | Persist its 24-hour recovery restriction after leaving the encounter; encounter-local debt already works. |
| Bide | Keep the recorded GM choice for contradictory printed User/Target wording. Do not silently select an interpretation. |
| Magic Coat | Extend supported reflected Support effects. Keep the explicit GM duration ruling because the card does not state one. |
| Mycelium Might | Resolve printed Reaction versus granted Late Reaction collisions explicitly. Pokerole grants Never Miss/Late Reaction, not video-game ability bypass. |
| Tangled Feet | Mixed target-specific success and critical results are implemented in engine 0.17; historical encounters retain their previous ruling. |
| Teravolt / Turboblaze | Extend held-item and unsupported effect-blocker interactions; retain explicit conflicting unique-ability rulings. Current type/damage/status immunity, supported shields/decoys, redirection and charge-target access are implemented. |
| Neutralizing Gas | Opposing emitters require a recorded GM policy. This is a rules stalemate, not a missing host validation step. |
| Receiver / Power of Alchemy | Narrative acquisition outside witnessed campaign fainting; explicit timing for switching active copies during combat. Witnessed acquisition, one active copy, game-time expiry and persistence already work. |
| Terrain overlaps | Preserve recorded GM decisions where the manual does not define precedence. Do not substitute video-game terrain replacement rules. |
| Torn World | Define or record the retargeting distribution before automating it. |
| Arena examples | Supply GM parameters where examples omit healing amounts or check difficulty; narrative escape and encounter-rate effects remain manual. |
| Magic Room | Integrate the future held-item engine; recording a field effect is already supported. |
| Night parameters | Reconcile the legacy per-Pokémon Moonlight night parameter with the new battlefield night setting, preserving explicitly logged GM overrides. Mind's Eye currently uses the battlefield setting. |

These are not all bugs to eliminate: contradictory or unspecified rules can remain explicit, logged GM decisions. New definitions must include reviewed evidence, previews, host validation, history, retry/rollback behavior and meaningful fixtures.

## 3. Catalog, creation fidelity and assets

- [ ] Finish review of unresolved/unverified species baselines and limits, forms, move ranks, evolution branches/prerequisites and ability availability. Fix the application-owned correction layer, never the supplied source data.
- [ ] Resolve **35 missing catalog references**, preserving exact species/form identities. Missing/ambiguous matches must remain visible until reviewed.
- [ ] Supply or review **168 missing species/form sprites**; current placeholders retain names and navigation.
- [ ] Work through the field-level audit queue in `outputs/rule-coverage/` in the parent workspace. An unverified field is not necessarily wrong, and a resolved link is not proof that prerequisites were reviewed.
- [ ] Continue manual habitat review when desired. Keep accepted CSV assignments, explicit exclusions, conditional notes and persistent user overrides authoritative; do not regenerate type-affinity suggestions over them.
- [ ] Audit the older broad coverage-matrix descriptions as individual mechanics advance. Per-definition registries and executable fixtures define precise coverage.

Current catalog: 1,200 species/forms and 892 moves. The special-move registry covers 221 selected definitions (217 automated, three partial, one excluded/manual); the targeting registry covers 16 moves (15 automated, one partial). These are overlapping scopes, not a complete move catalog or a sum of distinct supported moves.

## 4. Windows, multiplayer and distribution acceptance

Headless localhost WebSocket tests establish separate host/GM/owner authority, receipts, reconnect, rollback and database persistence. They do not establish interactive Windows or physical LAN acceptance.

- [ ] Install, upgrade and uninstall the packaged app in isolated Windows installations; verify preservation of personal data and offline catalog/sprites.
- [ ] Exercise creation, progression, trainer-point distribution, Compendium links/Back navigation and dialogs at 960x650, 1280x850 and 1920x1080, including keyboard access and long content.
- [ ] Play the native three-owner versus GM gym scenario: twelve registered Pokémon, six active, reserves, repeated Clash, once-per-turn Evasion, supported field/ability effects and controlled replacement.
- [ ] Exercise training-encounter draft privacy, publication, mixed capture recipients, trainer rewards, owner library imports and subsequent offline progression.
- [ ] Exercise stale previews, duplicate submissions, disconnect/reconnect, replacement GM, conflicting offline branches and backup restoration through the native interface.
- [ ] Test direct LAN joining on separate physical Windows machines, firewall/connection errors and reconnection. Record private-VPN tests separately from direct LAN tests.
- [ ] Finish and verify the trusted Windows signing/distribution process; building an installer does not establish a trusted signature or remove Windows execution prompts. Do not automatically launch executables or installers during unattended checks.

## 5. Deliberately deferred product features

These require a separate scope decision; they are not missing parts of the assumed-victory training-encounter flow.

- Automated training-session rolls, earning formulas and attempt limits.
- Capture dice, ball consumption and automatic capture success.
- Combat simulation within training encounters.
- Full evolution-tree diagrams rather than direct relationship navigation.
- Habitat editing and character-creation shortcuts inside the Compendium.
- Narrative/outside-battle activities listed in the ability inventory. Battle-triggered private information reveals remain in scope.

## 6. Excluded scope

**Trainer-target damage, lethal wounds, automatic Will recovery, Dynamax, all Max Moves including Max Guard, and Z-Moves are excluded.** Their references may remain visible in the Compendium. Do not turn these into roadmap tasks because an old registry names a related dependency. Manual GM resource corrections remain available.

## Stat copying/transfer milestone verification

Version 0.21.0 / engine 0.14.0-core / protocol 22 implements the supported stat-channel batch described in [stat copying and transfer](battle-stat-channels.md). The full Rust suite passed 396 tests, including localhost host/GM/owner, receipt, reconnect, persistence and rollback coverage. After the final event-counter removal adjustment, the focused stat-channel suite passed all 19 tests. Frontend validation passed 31 tests and 12 catalog checks; production build and warnings-denied Clippy passed. Windows NSIS packaging passed: `src-tauri/target/release/bundle/nsis/PokeVerse_0.21.0_x64-setup.exe` (213.67 MiB). The executable and installer were not launched. Native interaction, installed-app acceptance and physical LAN were not tested.

## Action-sequencing milestone verification

Version 0.22.0 / engine 0.15.0-core / protocol 23 adds all five reviewed [sequencing ability rules](battle-action-sequencing.md). The full Rust suite passed 417 tests, including the separate-host/GM/two-owner sequencing fixture and existing 3v3 receipt, reconnect, rollback and persistence scenarios. Frontend validation passed 33 tests and 12 catalog checks; production build, formatting and warnings-denied Clippy passed. Windows NSIS packaging passed: `src-tauri/target/release/bundle/nsis/PokeVerse_0.22.0_x64-setup.exe` (213.74 MiB). The executable and installer were not launched. Native interaction, installed-app acceptance and physical LAN were not tested.

## Damage-provenance milestone verification

Version 0.23.0 / engine 0.16.0-core / protocol 24 passed all 433 Rust tests, split into 80 serial binding-export checks and 353 behavioral tests to avoid a Windows mapped-file collision during parallel binding generation. The behavioral set includes separate host/GM/owner choice, duplicate/stale command, reconnect, SQLite persistence and rollback coverage. After the final Clash effectiveness correction, its focused regression passed again. Frontend validation passed 34 tests and 12 catalog checks; the final battle-control test run passed all 14 tests. Production build, TypeScript, formatting and warnings-denied Clippy passed. Windows NSIS packaging produced `src-tauri/target/release/bundle/nsis/PokeVerse_0.23.0_x64-setup.exe` (213.76 MiB). Neither executable nor installer was launched; native interaction, installed-app acceptance and physical LAN remain unverified.

## Event/accuracy milestone verification

Version 0.24.0 / engine 0.17.0-core / protocol 25 passed all 448 Rust checks: 81 serial binding-export tests and 367 behavioral tests. The behavioral suite includes separate host/GM/owner Pressure entry, permission, stale/duplicate command, reconnect, SQLite persistence and rollback; per-target accuracy serialization/rollback; historical engine gates; and Dancer copies after partially successful and entirely missed mixed-target source rolls. Frontend validation passed 35 tests and all 12 catalog checks. TypeScript, formatting, warnings-denied Clippy and production build passed. The final Windows NSIS build includes the Dancer correction: `src-tauri/target/release/bundle/nsis/PokeVerse_0.24.0_x64-setup.exe` (213.80 MiB). Neither executable nor installer was launched. Native interaction, installed-app acceptance and physical LAN remain unverified.

## 7. Suggested implementation order

User-approved scope: implement all remaining in-battle abilities in this order, including transformation abilities as core battle mechanics. Out-of-battle ability effects are excluded from implementation. Mixed abilities retain their in-battle effects in scope. The existing Dynamax, Max Move and Z-Move exclusions still apply.

1. Damage-provenance and event/accuracy abilities: implemented in 0.23.0 and 0.24.0; remaining card-specific boundaries stay visible in the inventory.
2. Held items and their dependent abilities: released in 0.25.0; remaining item definitions are batch 6.
3. Supported form changes and transformations: delivered in 0.26.0, retaining exclusions and recorded GM boundaries.
4. Remaining reviewed move/status interactions and catalog audit work.
5. Native Windows and physical LAN acceptance, followed by distribution/signing work as needed.

Each milestone should preserve historical engines, bump authoritative versions when needed, update coverage and this document, run appropriate regressions, package Windows and commit separately.

### Damage-provenance milestone

Application 0.23.0 / engine 0.16.0-core / protocol 24 enables Tinted Lens, Unaware, Stakeout and the battle-move portion of Sheer Force. Historical encounters retain their saved feature flags. See [damage-provenance rules](battle-damage-provenance.md) for exact behavior and fixtures. The event/accuracy batch followed in 0.24.0; reviewed held-item added effects completed Sheer Force in 0.25.0.

### Event/accuracy milestone

Application 0.24.0 / engine 0.17.0-core / protocol 25 enables Stench's battle reaction, Pressure's entry Will expenditure, mixed-target Tangled Feet success/critical results, and Electric Support charge consumption for Wind Power and Electromorphosis. Historical encounters retain their saved rule flags. See [battle events and per-target accuracy](battle-event-accuracy.md) for exact behavior and fixtures. Stench's out-of-battle encounter effects remain excluded. Held items and dependent abilities followed in 0.25.0.

### Standalone ability milestone

Application 0.28.0 / engine 0.21.0-core / protocol 29 delivers ordered milestone **4 of 7**. See [standalone ability rules](battle-standalone-abilities.md) for six reviewed cards, source wording conflicts, exact weight formulas, unbiased Moody sampling, Perish timing, target-specific Rivalry, shared snapshots, compatibility and verification. Discretionary metal weights and conflicting Rivalry/Torrent wording remain explicit GM decisions. Next is **milestone 5: finish partial battle interactions**.

### Private-information milestone

Application 0.27.0 / engine 0.20.0-core / protocol 28 delivers ordered milestone **3 of 7**. See [private-information rules](battle-private-information.md) for classification, comparison/ties, trainer Insight, recipient secrecy, historical gates and verification. Both entries retain explicit partial boundaries for unprinted timing/comparison and unsupported classification; their private reveal operation is released. Standalone abilities were subsequently delivered in milestone 4.

### Transformation milestone

Application 0.26.0 / engine 0.19.0-core / protocol 27 delivers ordered milestone **2 of 7** in the [complete roadmap](ITEM-ABILITY-ROADMAP.md). See [transformation rules](battle-transformations.md) for all 20 entries, 127 exact source-backed form records, all 47 Mega stones, Transform, preparation/ownership, restoration, Minior persistence, private projections, compatibility and verification. As One joint-build/precedence decisions, Schooling arrival, missing Disc/mask equipment and Gulp Missile's unspecified added effect remain recorded GM decisions. Teraform Zero preserves unique-ability conflicts as explicit rulings. Private battle information was subsequently delivered in milestone 3.

### Held-item milestone

Application 0.25.0 / engine 0.18.0-core / protocol 26 releases ordered deliverable **1 of 7** in the [complete roadmap](ITEM-ABILITY-ROADMAP.md). New battles enable reviewed item rules; saved encounters preserve historical flags. Private owner/GM decisions, equipment persistence and rollback are described in [held-item rules and verification](battle-held-items.md). Gluttony ingestion, discretionary Ripen healing and unreviewed items retain explicit boundaries. Transformations are next. Native Windows and physical LAN acceptance remains separate.

Verification: 555 Rust checks (94 binding exports and 461 behavioral), 48 frontend tests, 12 catalog checks, TypeScript, formatting, warnings-denied Clippy and production build passed. Windows NSIS package: `src-tauri/target/release/bundle/nsis/PokeVerse_0.25.0_x64-setup.exe` (214.08 MiB). Neither executable nor installer was launched. See the held-item release verification for environment restrictions and acceptance limits.

The following notes record implementation history before activation:

Held-item worktree progress: Ball Fetch owned failed-throw returns and trainer-resource rollback, King's Rock/Razor Fang chance pools and item-granted Sheer Force eligibility, Life Orb dice and saved recoil, Expert Belt across ordinary/formula/Clash damage, all 18 type boosters with saved shared damage, Eviolite reviewed-stage defenses, Iron Ball stat/grounding effects, Quick Claw initiative and Wide Lens/Zoom Lens/Bright Powder accuracy effects are staged with suppression and per-target results. Owned equipment preparation, encounter unit history, free reviewed berry consumption, GM transfer/swap/loss/recovery, end-of-battle persistence and equipment rollback are staged behind a disabled encounter flag. Reviewed hooks cover Sticky Hold, Unburden, Unnerve, Cheek Pouch, Cud Chew held-berry activation, Harvest end-of-battle regrowth, queued Symbiosis handoffs, hit-triggered Magician/Pickpocket theft and Klutz/Magic Room berry suppression. Symbiosis supports donor ownership, simultaneous-donor GM ordering and explicit cycle rulings. Leftovers round-end healing and lifetime consumption now use the same loss/persistence hooks. Frisk entry reveals and per-player equipment projections cover room welcome, accepted commands, broadcasts, reconnect, local host notifications, archives and undo views; the host retains full authoritative state. Natural Gift now uses persistent GM-recorded berry flavors, spends its ingredient on an attempt, and freezes Ripen's two-die damage bonus. Ripen healing increases require explicit GM rulings. Fling snapshots item/rarity and Ripen damage bonuses, tracks thrown ground units, supports paid retrieval through sequencing checks, and queues Cud Chew berry-hit effects after damage. Recycle reuses reviewed spent consumables from either side with a three-attempt battle limit and no unit duplication. Cud Chew stores owner-controlled second activations, and item healing now shares the printed three-HP round limit with moves. Magic Room forbids berry/item use according to its card. The staged battle panel exposes owner berry use and GM item rulings; remaining item effects and multiplayer acceptance still require integration. See [held-item implementation notes](battle-held-items.md).

Held-item scope review: the supplied Pickup card is scene-end narrative gathering and Honey Gather is daily production; both are excluded by the user's out-of-battle boundary. Ball Fetch differs: its supplied card describes returning a failed-capture ball at round end. That battle hook is now staged with owned-ball tracking and GM-recorded failed throws; automatic capture rolls remain a separate deferred system. The older registry dependency descriptions are not substitutes for these printed cards.

### Held-item milestone in progress

The worktree now has individually owned item units, stored/held equipment, equip/unequip progression and editor controls, capacity validation, preserved partial usage, backward-compatible sheet serialization and atomic inventory/equipment reconciliation. The packaged 0.24.0 release does not contain this staged work. The staged battle layer now includes reviewed consumption, transfers, theft and dependent-ability hooks. Remaining item-definition/blocker review, battle activation, compatibility gates and release integration remain; see [held-item implementation and remaining work](battle-held-items.md).

## 8. All manual abilities (44)

Every currently manual registry entry appears below. Dependencies describe remaining boundaries; excluded out-of-battle effects are not implementation tasks.

| Ability | Current boundary |
| --- | --- |
| Anticipation | Requires a private GM/owner information-reveal operation with reviewed move classification. |
| As One | Requires encounter form/state transformation and reviewed replacement metadata. |
| Ball Fetch | Staged GM-recorded failed wild throws protect an owned ball and return it at round end; activation/release gates remain. |
| Cheek Pouch | Requires held-item ownership, consumption, transfer or activation hooks. |
| Commander | Requires encounter form/state transformation and reviewed replacement metadata. |
| Cud Chew | Requires held-item ownership, consumption, transfer or activation hooks. |
| Embody Aspect | Requires encounter form/state transformation and reviewed replacement metadata. |
| Forewarn | Requires a private GM/owner information-reveal operation with reviewed move classification. |
| Frisk | Requires held-item ownership, consumption, transfer or activation hooks. |
| Gluttony | Requires held-item ownership, consumption, transfer or activation hooks. |
| Gulp Missile | Requires encounter form/state transformation and reviewed replacement metadata. |
| Harvest | Requires held-item ownership, consumption, transfer or activation hooks. |
| Heavy Metal | Requires reviewed weight/gender inputs and formulas in the shared damage calculation. |
| Honey Gather | Requires noncombat item gathering or daily recovery; no video-game battle substitute is applied. |
| Hunger Switch | Requires encounter form/state transformation and reviewed replacement metadata. |
| Ice Face | Requires encounter form/state transformation and reviewed replacement metadata. |
| Illusion | Requires encounter form/state transformation and reviewed replacement metadata. |
| Imposter | Requires encounter form/state transformation and reviewed replacement metadata. |
| Klutz | Requires held-item ownership, consumption, transfer or activation hooks. |
| Light Metal | Requires reviewed weight/gender inputs and formulas in the shared damage calculation. |
| Magician | Requires held-item ownership, consumption, transfer or activation hooks. |
| Moody | Requires independent unbiased attribute increase/reduction choices and their recorded provenance. |
| Multitype | Requires encounter form/state transformation and reviewed replacement metadata. |
| Perish Body | Existing hook families are available; this individual definition and its ordering still require implementation and fixtures. |
| Pickpocket | Requires held-item ownership, consumption, transfer or activation hooks. |
| Pickup | Requires held-item ownership, consumption, transfer or activation hooks. |
| Power Construct | Requires encounter form/state transformation and reviewed replacement metadata. |
| RKS System | Requires encounter form/state transformation and reviewed replacement metadata. |
| Regenerator | Requires noncombat item gathering or daily recovery; no video-game battle substitute is applied. |
| Ripen | Requires held-item ownership, consumption, transfer or activation hooks. |
| Rivalry | Requires reviewed weight/gender inputs and formulas in the shared damage calculation. |
| Schooling | Requires encounter form/state transformation and reviewed replacement metadata. |
| Shields Down | Requires encounter form/state transformation and reviewed replacement metadata. |
| Stance Change | Requires encounter form/state transformation and reviewed replacement metadata. |
| Sticky Hold | Requires held-item ownership, consumption, transfer or activation hooks. |
| Symbiosis | Requires held-item ownership, consumption, transfer or activation hooks. |
| Tera Shell | Requires encounter form/state transformation and reviewed replacement metadata. |
| Tera Shift | Requires encounter form/state transformation and reviewed replacement metadata. |
| Teraform Zero | Requires encounter form/state transformation and reviewed replacement metadata. |
| Torrent | Printed text grants 2 damage dice but icon grants 1. GM ruling required. |
| Unburden | Requires held-item ownership, consumption, transfer or activation hooks. |
| Unnerve | Requires held-item ownership, consumption, transfer or activation hooks. |
| Zen Mode | Requires encounter form/state transformation and reviewed replacement metadata. |
| Zero to Hero | Requires encounter form/state transformation and reviewed replacement metadata. |

## 9. All partial abilities (61)

Every currently partial registry entry appears below. Dependencies describe remaining boundaries; excluded out-of-battle effects are not implementation tasks.

| Ability | Current implementation and boundary | Dependencies |
| --- | --- | --- |
| Arena Trap | Block ordinary recall for eligible targets in confirmed abstract range, with rank target caps and reviewed Ground/Steel/Ghost exceptions. GM switching overrides remain explicit. | forced-switch-move-resolution |
| Aroma Veil | In-range allies and user ignore In Love and reviewed Taunt, Torment, Spite, Attract, Disable, Encore and Heal Block effects. | unreviewed-move-effects |
| Aura Break | Reviewed range-limited Dark/Fairy damage aura and Aura Break inversion. Duplicate sources do not stack; attack/cooperation narrative restrictions require a GM ruling. | narrative-relationship-rulings |
| Big Pecks | Protect the reviewed stat changes through sourced battle effects and supported moves; Clear Body also rejects external increases and Flower Veil protects in-range allies. | unreviewed-stat-moves |
| Clear Body | Protect the reviewed stat changes through sourced battle effects and supported moves; Clear Body also rejects external increases and Flower Veil protects in-range allies. | unreviewed-stat-moves |
| Competitive | Sourced stat changes trigger reviewed protection, once-only retaliation, single reflection, reversal or one additional point. Move and ability channels remain distinct. | unreviewed-stat-moves, forced-switch-move-resolution |
| Contrary | Sourced stat changes trigger reviewed protection, once-only retaliation, single reflection, reversal or one additional point. Move and ability channels remain distinct. | unreviewed-stat-moves, forced-switch-move-resolution |
| Corrosion | Source-tracked Poison bypasses type/status poison immunity; global battlefield status restrictions still apply. | ability-bypass-interactions |
| Costar | On field entry, copy one active ally's supported positive move and ability attribute/defense channels. One eligible ally is automatic; multiple allies use frozen entry snapshots and an owner choice. Highest modifiers do not stack within a channel; sheets and ability identities are unchanged. | unreviewed-trait-channels |
| Dancer | Once per round, choose a free copy of a completed Dance move, including self and field targeting, inheriting Accuracy successes and rolling fresh damage. Owner target choice, GM random-foe selection, normal move/status/range guards and one-use rampage copies apply. Engine 0.17 preserves each original target accuracy result through the copied action, including missed sources; new targets use the highest recorded source count. Unreviewed dance added effects remain explicitly adjudicated. | unsupported-dance-move-definitions |
| Dark Aura | Reviewed range-limited Dark/Fairy damage aura and Aura Break inversion. Duplicate sources do not stack; attack/cooperation narrative restrictions require a GM ruling. | narrative-relationship-rulings |
| Dauntless Shield | On entry, add two temporary Defense or Strength respectively. Outside-battle narrative effects are GM adjudicated. | narrative-effects |
| Defiant | Sourced stat changes trigger reviewed protection, once-only retaliation, single reflection, reversal or one additional point. Move and ability channels remain distinct. | unreviewed-stat-moves, forced-switch-move-resolution |
| Disguise | Negate the first damaging hit in a scene. Cosmetic disguise and species/form transformations remain manual. | form-transformation |
| Early Bird | Structured Sleep duration is halved (rounded up); waking requires two cumulative Insight successes. Rest is excluded. Narrative sleep remains manual. | outside-battle-activities |
| Fairy Aura | Reviewed range-limited Dark/Fairy damage aura and Aura Break inversion. Duplicate sources do not stack; attack/cooperation narrative restrictions require a GM ruling. | narrative-relationship-rulings |
| Flower Veil | Protect the reviewed stat changes through sourced battle effects and supported moves; Clear Body also rejects external increases and Flower Veil protects in-range allies. | unreviewed-stat-moves |
| Full Metal Body | Protect the reviewed stat changes through sourced battle effects and supported moves; Clear Body also rejects external increases and Flower Veil protects in-range allies. | unreviewed-stat-moves |
| Gale Wings | Add Reaction 1 to Flying moves without imposing a video-game full-HP requirement. Retain higher printed early priority; conflicting Late Reaction needs a recorded timing ruling. Charge reactions still need adjudication. | conflicting-late-priority-ruling, charge-reaction-definition |
| Guard Dog | Sourced stat changes trigger reviewed protection, once-only retaliation, single reflection, reversal or one additional point. Move and ability channels remain distinct. | unreviewed-stat-moves, forced-switch-move-resolution |
| Hadron Engine | Add two Special under Electric Terrain or two Strength under Sunny/Desolate respectively. Weather/terrain activation follows the reviewed field definition. | held-booster-item-system |
| Hospitality | On entry the owner may spend one Will to heal one wounded active ally by three HP. No revival; lethal wounds remain manual. | lethal-resource-system |
| Hyper Cutter | Protect the reviewed stat changes through sourced battle effects and supported moves; Clear Body also rejects external increases and Flower Veil protects in-range allies. | unreviewed-stat-moves |
| Illuminate | In-range user/allies ignore the supported fog/visibility accuracy penalty; narrative encounter effects remain manual. | narrative-encounter-effects |
| Intimidate | Entry-only rank-limited Strength reduction, non-stacking, removed when source leaves or faints. GM range defaults to all deployed foes; stable IDs break excess-target ties. An explicit GM range correction updates affected targets. |  |
| Intrepid Sword | On entry, add two temporary Defense or Strength respectively. Outside-battle narrative effects are GM adjudicated. | narrative-effects |
| Justified | Dark hits raise the separate Strength ability channel by one, capped at three; narrative injustice triggers remain GM adjudicated. | outside-battle-activities |
| Levitate | Ground move immunity automated; terrain and entry-hazard effects remain manual. |  |
| Magic Bounce | Reflect executable foe Support effects aimed at the holder or its side, once, without extra actions/accuracy. Preserve original resource payer and reflected effect source. Unimplemented Support effects still require adjudication; manual field records retain reflected scope. | unsupported-support-effects |
| Magic Guard | Prevents reviewed ability damage, burn/poison ticks and Submission recoil. Confusion and direct move damage remain; weather, held items and entry hazards await their mechanics. |  |
| Magnet Pull | Block ordinary recall for eligible targets in confirmed abstract range, with rank target caps and reviewed Ground/Steel/Ghost exceptions. GM switching overrides remain explicit. | forced-switch-move-resolution |
| Mega Launcher | Pulse/Aura damaging moves gain two damage dice. Heal Pulse uses five-HP Complete Healing and one Will. | other-pulse-healing-definitions |
| Mirror Armor | Sourced stat changes trigger reviewed protection, once-only retaliation, single reflection, reversal or one additional point. Move and ability channels remain distinct. | unreviewed-stat-moves, forced-switch-move-resolution |
| Mycelium Might | Support moves gain Never Miss and Late Reaction 1; no ability bypass is granted by the printed card. Printed Reaction collisions require explicit GM adjudication. | conflicting-printed-reaction |
| Neutralizing Gas | Negates foes in abstract range subject to rank target cap; restores on source departure, fainting or range change. Unique abilities prevail. Opposing Gas stalemates require GM ruling. | GM ruling for opposing Gas stalemates |
| Oblivious | Immune to In Love; other feeling-related and narrative effects remain GM-adjudicated. |  |
| Opportunist | Copy supported foe move/ability attribute and defense increases on authoritative state changes, retaining the highest within each channel. Immutable before/after observation prevents reciprocal loops. Decreases and unchanged modifiers do not trigger a copy. | unreviewed-trait-channels |
| Orichalcum Pulse | Add two Special under Electric Terrain or two Strength under Sunny/Desolate respectively. Weather/terrain activation follows the reviewed field definition. | held-booster-item-system |
| Power of Alchemy | On active allies fainting, store multiple eligible ability snapshots for 1440 game minutes, with one active copy. Acquisition requires Power of Alchemy to be active; selecting within combat requires a GM timing ruling. Campaign clock, provenance and rollback persist. | GM ruling for selection timing during combat, Narrative acquisition outside witnessed campaign fainting |
| Prankster | Add Reaction 1 to Support moves. Retain higher printed early reaction priority; conflicting Late Reaction requires a recorded printed/ability timing ruling. Unsupported move effects and charge reaction exceptions remain explicit. | conflicting-late-priority-ruling, charge-reaction-definition |
| Protosynthesis | Under qualifying weather/terrain boost the highest attribute by one, or Dexterity by two; tied highest attributes require an owner choice. | held-booster-item-system |
| Quark Drive | Under qualifying weather/terrain boost the highest attribute by one, or Dexterity by two; tied highest attributes require an owner choice. | held-booster-item-system |
| Receiver | On an active ally fainting, the owner chooses one eligible ability snapshot. One unexpired copy, preserved in campaign state for 1440 game minutes; the GM advances game time between encounters. Unique donor abilities cannot be copied. | Narrative acquisition outside witnessed campaign fainting |
| Reckless | Recoil-tagged moves and reviewed Submission gain two damage dice; other unreviewed recoil formulas still need GM resolution. | unreviewed-recoil-moves |
| Rock Head | Ignore the supported Submission recoil; other recoil moves require reviewed definitions. | unreviewed-recoil-moves |
| Run Away | Ignore supported Blocked recall restrictions. Narrative escape and capture consequences require GM adjudication. | narrative-escape-effects |
| Sand Veil | Weather damage immunity; the table rule still limits Evasion to once per turn and five total actions per round. |  |
| Serene Grace | Add two chance dice to supported move added effects; unreviewed chance effects still need GM resolution. | unreviewed-chance-moves |
| Shadow Tag | Block ordinary recall for eligible targets in confirmed abstract range, with rank target caps and reviewed Ground/Steel/Ghost exceptions. GM switching overrides remain explicit. | forced-switch-move-resolution |
| Sheer Force | Owner or GM chooses whether damaging moves trade their structured AddedEffects for two damage dice. Freeze the choice for shared targets, successive hits and charge release; preserve recoil and recharge attributes. Held-item added effects await the item engine. | held-item-system |
| Simple | Sourced stat changes trigger reviewed protection, once-only retaliation, single reflection, reversal or one additional point. Move and ability channels remain distinct. | unreviewed-stat-moves, forced-switch-move-resolution |
| Snow Cloak | Weather damage immunity; the table rule still limits Evasion to once per turn and five total actions per round. |  |
| Supersweet Syrup | Foes in confirmed range cannot Evade the holder moves; Clash remains available. Narrative encounter-rate changes remain GM adjudicated. | narrative-encounter-effects |
| Synchronize | Reflect source-tracked statuses once, respecting immunities. GM status edits without an inflictor have no inferred source. |  |
| Teravolt | Action-scoped foe type/damage/status immunity bypass, supported shields, decoys, redirection and charging-target access. Unique-effect stalemates, held items and unsupported effect blockers require GM adjudication. | unique-stalemates, held-item-system, unsupported-support-effects |
| Triage | Reviewed Basic Heal and status-cure Support moves gain Reaction 1, including selected allies and multi-target cures. Existing action, capacity, target, Will and phase validation apply. Unsupported healing/cure definitions remain manual. | unsupported-healing-cure-definitions |
| Turboblaze | Action-scoped foe type/damage/status immunity bypass, supported shields, decoys, redirection and charging-target access. Unique-effect stalemates, held items and unsupported effect blockers require GM adjudication. | unique-stalemates, held-item-system, unsupported-support-effects |
| Unseen Fist | Non-ranged Physical Moves bypass Substitute/Shed Tail decoys. Remaining shield, force-field and Cover bypass hooks require implementation. | shield-force-field-cover-bypass |
| White Smoke | Protect the reviewed stat changes through sourced battle effects and supported moves; Clear Body also rejects external increases and Flower Veil protects in-range allies. | unreviewed-stat-moves |
| Wonder Guard | Prevent non-super-effective move damage and supported indirect damage; condition damage still applies. | ability-bypass-interactions |
| Wonder Skin | Reduce chance dice for supported incoming added effects by two; independent selectors remain independent. | unreviewed-chance-moves |

## References and completion discipline

[Ability registry](../rules/ability-coverage.json), [special move registry](../rules/special-moves.json), [targeting registry](../rules/tactics-reviewed.json), [reviewed ability batch](battle-reviewed-ability-batch.md), [rules](RULES.md), [headless 3v3](headless-3v3.md), [training points](training-points.md), [catalog storage](catalog-storage.md).

Before closing a task, record what changed and what was actually verified. Build success, native startup, interactive use and physical-machine LAN testing are distinct results. Keep this inventory synchronized with registry status changes.
