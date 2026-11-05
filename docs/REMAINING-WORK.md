# PokeVerse - remaining work

Snapshot: **5 November 2026**, application **0.63.0**, engine **0.55.0-core**, protocol **64**, user database schema **4**.

This is the current project backlog. Implemented foundations are summarized first; unfinished features, explicit GM rulings, verification gaps and excluded scope are distinguished below. Counts come from the checked-in registries and should be refreshed after catalog changes.

## Complete item and ability inventory (22 October audit)

See the [complete item and ability roadmap](ITEM-ABILITY-ROADMAP.md) for the ordered deliverables and every catalog item. Its [ability inventory](ability-roadmap.csv) accounts for all 305 abilities; its [item inventory](item-roadmap.csv) accounts for all 236 items. This audit distinguishes released scope, disabled worktree implementations, genuinely missing mechanics, GM rulings and exclusions.

The registry records 242 automated, 60 partial and 3 manual abilities. Sections 8–9 below are refreshed from `rules/ability-coverage.json`. A partial entry describes implemented behavior plus a remaining boundary; it does not mean the whole ability is unimplemented. The reviewed held-item batch is released in 0.25.0; transformation batch 2 is delivered in 0.26.0; private-information batch 3 is delivered in 0.27.0; standalone batch 4 is delivered in 0.28.0; interaction review batch 5 is delivered in 0.29.0. Milestone 6 delivers the 30 battle/species item definitions and all 17 ordinary medicines with reviewed Recycle. The 47 Mega stones remain released. See [item rules and retained GM policies](battle-item-completion.md). All 35 Z-crystals remain excluded; 10 items explicitly forbid battle use. Other reference-only entries require scope review before becoming battle tasks.

## Current priority

Private friend-group use: public release/signing/distribution polish is not planned, by user decision on 23 October 2026. M1–M4 of the move-effect/catalog queue are complete within their recorded scopes. **M5 remains native multiplayer acceptance**, covering native UI, separate-machine LAN/private-VPN behavior, reconnect/recovery and packaged-app data preservation. Automated tests do not substitute for those outcomes. Existing [group rules](battle-group-rules.md), [GM choices](GM-CHOICE-ROADMAP.md) and unsupported-effect boundaries remain in force; Sappy Seed remains excluded. Packaging runs only when explicitly requested.

The user requested continued move completion on 1 November. Follow-up MOVE-A01 implements 66 damaging attribute clauses (38 guaranteed, 28 Chance-based); see [the updated move audit](MOVE-EFFECT-AUDIT.md#remaining-move-implementation). MOVE-A02 delivers Spikes, Stealth Rock and Sticky Web on 4 November. The remaining 141 fallback routes and unsupported clauses on existing routes still require review and implementation in separate batches. Native acceptance is still open.

## Move-effect milestone queue (updated 31 October 2026)

- [x] **Milestone 1: move-effect audit.** [Named source-backed queue and limits](MOVE-EFFECT-AUDIT.md), covering all 892 effect routes and a focused semantic review of 180 status-related cards. The initial 107 actionable status cards were divided into five implementation sub-batches; all 107 are now delivered within their reviewed battle scope. Route success is not whole-card completion, and manual-route totals are not counts of distinct missing mechanics.
- [x] **Milestone 2: implement the audited effects.** M2-A direct status application (22 cards) is delivered in 0.37.0; M2-B Chance-based status effects (47 cards) is delivered in 0.38.0. M2-C compound/conditional battle clauses (28 cards) are delivered in 0.39.0; see [scope and retained choices](battle-compound-status.md). M2-D recurring/sleep effects (6 cards) are delivered in 0.40.0; see [timing, controls and retained choices](battle-recurring-status.md). M2-E cure/protection entry paths (4 cards) are delivered in 0.41.0; see [scope and verification](battle-cure-protection.md). Preserve implemented scopes and exclusions. M3 review is complete; M4 catalog corrections remain separate.
- [x] **Milestone 3: battlefield/ice edge-case review.** [All 16 checks closed](M3-BATTLEFIELD-ICE-CHECKLIST.md), app 0.42.0: Sunny-at-night user ruling, Magic Guard / Perish Song correction, Snow category/threshold regressions and scoped terrain entry. Underspecified formulas and precedence retain explicit GM rulings; this is review completion, not universal automation.
- [x] **Milestone 4: catalog reliability.** [Named checklist and final evidence](M4-CATALOG-RELIABILITY-CHECKLIST.md). M4-C01 corrects Bleakwind Storm accuracy; M4-R01 resolves all 35 baseline references; M4-S01 reconciles all 1,200 exact species/forms against JSON plus recorded rulings; M4-A01 resolves all 168 baseline sprite gaps. Final catalog integrity, source/asset fingerprints and all species Compendium links pass automated checks. Zero missing references/assets remain. Optional PDF certification remains partial; nine stat deviations, one evolution guidance discrepancy and four GM-set costs remain explicit decisions. Native/UI/physical-network acceptance remains M5.
- [ ] **Milestone 5: native multiplayer acceptance.** Record actual native, physical LAN and private-VPN outcomes separately from automated tests.

## Implemented foundation

Offline trainer/Pokémon creation and named progression use shared Rust budgets, costs, prerequisites and derived resources. Campaign synchronization supports accepted offline purchases, indivisible progression chains and scoped GM exception approval. The host appoints the GM and automatically validates normal commands; these roles can be separate people.

Training encounters use the reviewed habitat pool, private preparation, publication, assumed victory, captures and trainer assignment. The GM awards Training Points to a trainer pool; owners distribute accepted points among that trainer's Pokémon offline. Existing Pokémon balances are preserved.

Battles register full teams, three active slots per side, reserves, controlled switching, accuracy/damage automation, multi-target responses, field effects, supported abilities, decoys, overrides and rollback. Evasion is once per turn; eligible Clash can answer each incoming hit within the action budget. The Compendium works offline for all 1,200 species/forms.

Dancer, Parental Bond, Gorilla Tactics, Truant and Defeatist now use the reviewed [action-sequencing rules](battle-action-sequencing.md), including free copies, independent damage pools, entry move selection and per-action Loyalty checks. Unreviewed move effects remain explicit GM resolutions.

## 1. Remaining battle systems

- [x] **Held-item ability batch:** released in 0.25.0 with owned equipment, reviewed effects, dependent abilities, private decisions, rollback and synchronization. Reviewed item/medicine definitions were delivered in batch 6 (0.30.0); native acceptance remains batch 7.
- [x] **Forms and transformations (batch 2):** reviewed all 20 entries, exact temporary builds/types/abilities/moves, 47 Mega stones and Transform, permanent-sheet protection, scene restoration, persistent Minior Core memory, owner/GM decisions, frozen damage inputs and rollback. Printed ambiguities and absent item stock retain the explicit boundaries in [transformation rules](battle-transformations.md).
- [x] **Private battle information (batch 3):** Anticipation/Forewarn warnings, private GM comparisons and Insight rulings, linked-trainer recipients, movepool projections, transport, persistence and rollback. See [private-information rules](battle-private-information.md).
- [x] **Standalone abilities (batch 4):** Heavy/Light Metal weight inputs/formulas, Rivalry comparisons, Moody host sampling, Perish Body and explicit Torrent wording choice. See [standalone ability rules](battle-standalone-abilities.md).
- [x] **Partial interaction review (batch 5):** reviewed all 55 assigned entries, released 67 exact move definitions, shield/field/Cover bypass, forced-switch protection, timing decisions and copied traits. [Reviewed rules and retained boundaries](battle-partial-interactions.md). Tinted Lens, Unaware, Stakeout and the move portion of Sheer Force are implemented. The full ability inventory is in sections 8-9.
- [x] **Supported stat/trait copying:** Costar copies a selected ally's supported positive move/ability channels on entry; Opportunist copies supported foe increases. Baton Pass and Snatch transfer their reviewed channels. Coil/Hone Claws accuracy pools and Focus Energy High Critical are also supported. See [stat copying and transfer](battle-stat-channels.md) and [the later trait review](battle-partial-interactions.md).
- [ ] **Additional unreviewed trait effects:** review specific source-defined traits beyond the implemented channels before extending copying. This remaining boundary is why Costar and Opportunist retain a `partial` registry label; their supported copying behavior is already implemented. Temporary ability identity copying is a separate implemented system.
- [x] **Scene identity effects:** Entrainment, Doodle and Core Enforcer are implemented in 0.33.0; see [exact rules and verification](battle-identity-moves.md). Role Play, Skill Swap, Gastro Acid, Trace, Gas, contact replacement and witnessed ally inheritance already exist.
- [x] **Burn and Poison escalation:** selected detailed rules automated in 0.34.0; see [rules and verification](battle-status-escalation.md).
- [x] **Audited status-effect queue:** all 107 M2 cards are delivered at their reviewed battle scope; see [the completed batches and retained boundaries](MOVE-EFFECT-AUDIT.md). Will drains remain delivered, Sappy Seed and automatic Will recovery excluded. Healing Wish's metadata discrepancy remains a catalog task.
- [x] **Ice edge review:** M3-I01-I05 are closed in the [M3 checklist](M3-BATTLEFIELD-ICE-CHECKLIST.md). Snow is 7 HP / physical Defense 3 / Special Defense 2; live target weather changes the next resolution without resetting damage. Five named nonordinary object formulas retain explicit GM adjudication.
- [ ] **Unreviewed move effects:** the [892-card route inventory](../rules/move-effect-audit.json) names every effect-route outcome; semantic review outside the focused status subset remains separate. Extend executable definitions beyond the selected special/targeting registries. Ordinary damage automation does not imply every added effect on all 892 moves is executable. Preserve explicit GM resolution for unsupported cards.

## 2. Remaining named move and battlefield boundaries

| Entry | Remaining work or intentional ruling |
| --- | --- |
| Baton Pass | Supported positive move and ability attribute channels and replacement rules are implemented. |
| Snatch | Review additional effects; supported move/ability modifiers, shields, decoys and typed effects already transfer. |
| Shadow Force | Cross-encounter recovery enforcement is intentionally not scheduled, by user decision on 27 October 2026. Existing encounter-local debt is unchanged. |
| Bide | Engine 0.24 uses the selected group rule: user damage basis, return to the last attacker. Historical encounters retain their recorded interpretation. |
| Magic Coat | Group rule: reflect supported effects until the user next acts. Extend unsupported Support definitions separately; historical durations remain recorded. |
| Mycelium Might | Engine 0.24 fixes all Support moves to Never Miss/Late Reaction 1; a selected Choice Scarf overrides Late Reaction. Historical encounters retain their recorded timing. No ability bypass. |
| Tangled Feet | Mixed target-specific success and critical results are implemented in engine 0.17; historical encounters retain their previous ruling. |
| Teravolt / Turboblaze | Batch 5 adds force-field, Cover and non-unique forced-switch prevention bypass. Extend individually unsupported move/item definitions; retain explicit conflicting unique-ability rulings. Current type/damage/status immunity, supported shields/decoys, redirection and charge-target access are implemented. |
| Neutralizing Gas | Opposing emitters require a recorded GM policy. This is a rules stalemate, not a missing host validation step. |
| Receiver / Power of Alchemy | Narrative acquisition outside witnessed campaign fainting; explicit timing for switching active copies during combat. Witnessed acquisition, one active copy, game-time expiry and persistence already work. |
| Terrain overlaps | Preserve recorded GM decisions where the manual does not define precedence. Do not substitute video-game terrain replacement rules. |
| Torn World | M3-A03: distribution/target legality are unspecified; retain recorded GM adjudication until a group policy is chosen. |
| Arena examples | M3-A01: Healing Pits has a printed Heal 1 icon and existing configurable mechanics. M3-A02: High Poles omits difficulty, so retain a GM parameter. Narrative escape remains discretionary; see the M3 checklist. |
| Magic Room | Held-item suppression is integrated since batch 1; reviewed item/medicine definitions were delivered in batch 6 (0.30.0). Unreviewed reference-only items need scope review, not reimplementation of the released batch. |
| Night parameters | Implemented in 0.32.0: Moonlight reads live battlefield day/night, with explicit individual day/night overrides and preserved legacy positive-night rulings. Mind's Eye uses the battlefield setting. See starting-environment rules. |

These are not all bugs to eliminate: contradictory or unspecified rules can remain explicit, logged GM decisions. New definitions must include reviewed evidence, previews, host validation, history, retry/rollback behavior and meaningful fixtures.

## 3. Catalog, creation fidelity and assets

- [ ] Finish review of unresolved/unverified species baselines and limits, forms, move ranks, evolution branches/prerequisites and ability availability. Fix the application-owned correction layer, never the supplied source data.
- [x] Resolve the **35 missing catalog references** (M4-R01): Strength name corrected with stable ID/legacy alias, Gouging Fire Fire Blast spelling corrected at Expert while keeping #1020, and Mew wildcard implemented by user ruling. Current missing references: **0**; see [decisions and limits](M4-CATALOG-RELIABILITY-CHECKLIST.md).
- [x] Supply and review all **168 missing species/form images** in M4-A01. Exact game renders and Rotom Dex official artwork are bundled locally; source/credit and integrity records live in [the A01 checklist](M4-A01-SPECIES-ASSETS-CHECKLIST.md). Current missing assets: **0**.
- [ ] Work through the field-level audit queue in `outputs/rule-coverage/` in the parent workspace. An unverified field is not necessarily wrong, and a resolved link is not proof that prerequisites were reviewed.
- [ ] Continue manual habitat review when desired. Keep accepted CSV assignments, explicit exclusions, conditional notes and persistent user overrides authoritative; do not regenerate type-affinity suggestions over them.
- [ ] Audit the older broad coverage-matrix descriptions as individual mechanics advance. Per-definition registries and executable fixtures define precise coverage.

Current catalog: 1,200 species/forms and 892 moves. The special-move registry covers 221 selected definitions (217 automated, three partial, one excluded/manual); the targeting registry covers 16 moves (15 automated, one partial). These are overlapping scopes, not a complete move catalog or a sum of distinct supported moves.

## 4. Windows, multiplayer and distribution acceptance

Headless localhost WebSocket tests establish separate host/GM/owner authority, receipts, reconnect, rollback and database persistence. They do not establish interactive Windows or physical LAN acceptance.

Milestone 7 preparation is delivered: [isolated lab, preservation evidence and 15 manual acceptance cases](windows-lan-acceptance.md). The user selected preparation of the checks; native installation/workflows and physical LAN remain pending. Trusted signing/public distribution is not planned for this private group. See [the 23 October automated baseline](acceptance-baseline-2026-10-23.md).

- [ ] Install, upgrade and uninstall the packaged app in isolated Windows installations; verify preservation of personal data and offline catalog/sprites.
- [ ] Exercise creation, progression, trainer-point distribution, Compendium links/Back navigation and dialogs at 960x650, 1280x850 and 1920x1080, including keyboard access and long content.
- [ ] Play the native three-owner versus GM gym scenario: twelve registered Pokémon, six active, reserves, repeated Clash, once-per-turn Evasion, supported field/ability effects and controlled replacement.
- [ ] Exercise training-encounter draft privacy, publication, mixed capture recipients, trainer rewards, owner library imports and subsequent offline progression.
- [ ] Exercise stale previews, duplicate submissions, disconnect/reconnect, replacement GM, conflicting offline branches and backup restoration through the native interface.
- [ ] Test direct LAN joining on separate physical Windows machines, firewall/connection errors and reconnection. Record private-VPN tests separately from direct LAN tests.
- **Not planned for private group use:** trusted Windows signing/public distribution. The current installer remains unsigned; packaging is separate from native acceptance. Do not automatically launch executables or installers during unattended checks.

Linux x86-64 packaging is now available for v0.30.0, with 651 Rust tests passing on Ubuntu 26.04 and verified Debian/AppImage catalog contents. See [Linux build and installation](linux-build.md). Native Ubuntu 26.10 and physical Windows/Linux Tailscale acceptance remain pending.

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
2. Held items and their dependent abilities: released in 0.25.0; reviewed item/medicine definitions delivered in batch 6 (0.30.0).
3. Supported form changes and transformations: delivered in 0.26.0, retaining exclusions and recorded GM boundaries.
4. Remaining reviewed move/status interactions and catalog audit work.
5. Native Windows and physical LAN/private-VPN acceptance. Trusted signing/public distribution is not planned.

Each milestone should preserve historical engines, bump authoritative versions when needed, update coverage and this document, run appropriate regressions and commit separately. App packaging and Clippy run only when explicitly requested, as recorded in `AGENTS.md`.

### Damage-provenance milestone

Application 0.23.0 / engine 0.16.0-core / protocol 24 enables Tinted Lens, Unaware, Stakeout and the battle-move portion of Sheer Force. Historical encounters retain their saved feature flags. See [damage-provenance rules](battle-damage-provenance.md) for exact behavior and fixtures. The event/accuracy batch followed in 0.24.0; reviewed held-item added effects completed Sheer Force in 0.25.0.

### Event/accuracy milestone

Application 0.24.0 / engine 0.17.0-core / protocol 25 enables Stench's battle reaction, Pressure's entry Will expenditure, mixed-target Tangled Feet success/critical results, and Electric Support charge consumption for Wind Power and Electromorphosis. Historical encounters retain their saved rule flags. See [battle events and per-target accuracy](battle-event-accuracy.md) for exact behavior and fixtures. Stench's out-of-battle encounter effects remain excluded. Held items and dependent abilities followed in 0.25.0.

### Partial-interaction milestone

Application 0.29.0 / engine 0.22.0-core / protocol 30 delivers ordered milestone **5 of 7**. See [reviewed interactions](battle-partial-interactions.md) for all 55 assigned abilities, 67 exact move definitions, retained GM boundaries, compatibility and verification. Milestone 6 subsequently delivers the remaining item and medicine definitions.

### Standalone ability milestone

Application 0.28.0 / engine 0.21.0-core / protocol 29 delivers ordered milestone **4 of 7**. See [standalone ability rules](battle-standalone-abilities.md) for six reviewed cards, source wording conflicts, exact weight formulas, unbiased Moody sampling, Perish timing, target-specific Rivalry, shared snapshots, compatibility and verification. Discretionary metal weights and conflicting Rivalry/Torrent wording remain explicit GM decisions. The interaction review was subsequently delivered in milestone 5.

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

### Historical held-item staging notes (before 0.25.0 release)

Historical status, superseded by the 0.25.0 release and 0.30.0 item-completion milestone: the worktree had individually owned item units, stored/held equipment, equip/unequip progression and editor controls, capacity validation, preserved partial usage, backward-compatible sheet serialization and atomic inventory/equipment reconciliation. The packaged 0.24.0 release does not contain this staged work. The staged battle layer now includes reviewed consumption, transfers, theft and dependent-ability hooks. Remaining item-definition/blocker review, battle activation, compatibility gates and release integration remain; see [held-item implementation and remaining work](battle-held-items.md).

## 8. Manual ability registry entries (3)

Refreshed from the current ability registry on 28 October 2026. These entries have no automated battle implementation in their recorded scope; excluded narrative effects are not new tasks.

| Ability | Recorded scope | Remaining boundary |
| --- | --- | --- |
| Honey Gather | Requires noncombat item gathering or daily recovery; no video-game battle substitute is applied. | outside-battle-activities |
| Pickup | Outside-battle found-item acquisition remains GM adjudicated; battle equipment ownership and transfer are implemented. | outside-battle-acquisition |
| Regenerator | Requires noncombat item gathering or daily recovery; no video-game battle substitute is applied. | outside-battle-activities |

## 9. Partially automated abilities (60)

Refreshed from the current ability registry on 28 October 2026. The implementation column is already delivered. Only the remaining boundary needs review; a narrative or intentionally excluded boundary is not an implementation task.

| Ability | Implemented scope | Remaining boundary |
| --- | --- | --- |
| Anticipation | Private trainer alert at the reviewed entry/acquisition window from current deployed conscious foes. Classify structured lethal flags and supported super-effective typing/conversions/immunities; Support/self/ally cards do not qualify. Unresolved cards and timing outside this window require a private GM ruling. No foe or move names are sent with the alert. | See recorded scope |
| Arena Trap | Block ordinary recall for eligible targets in confirmed abstract range, with rank target caps and reviewed Ground/Steel/Ghost exceptions. GM switching overrides remain explicit. User Switcher Moves respect trapping. Whether external forced replacement overrides printed blocking remains an explicit GM exception; no video-game precedence is assumed. | forced-switch-trapping-ruling |
| Aroma Veil | In-range allies and user ignore In Love and reviewed Taunt, Torment, Spite, Attract, Disable, Encore and Heal Block effects. | unreviewed-move-effects |
| As One | GM selects an exact reviewed joint-form build and one available ability from each conscious allied component. Movepools combine and both supported ability channels remain active; the partner cannot act separately. Conflicting type/stat precedence and nested joint-form changes remain explicit GM boundaries. | joint-form-precedence-ruling |
| Aura Break | Reviewed range-limited Dark/Fairy damage aura and Aura Break inversion. Duplicate sources do not stack; attack/cooperation narrative restrictions require a GM ruling. | narrative-relationship-rulings |
| Big Pecks | Protect the reviewed stat changes through sourced battle effects and supported moves; Clear Body also rejects external increases and Flower Veil protects in-range allies. | unreviewed-stat-moves |
| Clear Body | Protect the reviewed stat changes through sourced battle effects and supported moves; Clear Body also rejects external increases and Flower Veil protects in-range allies. | unreviewed-stat-moves |
| Competitive | Sourced stat changes trigger reviewed protection, once-only retaliation, single reflection, reversal or one additional point. Move and ability channels remain distinct. Reviewed ordinary stat cards and Parting Shot use the same sourced protection/reflection/reversal/retaliation path. | unreviewed-stat-moves |
| Contrary | Sourced stat changes trigger reviewed protection, once-only retaliation, single reflection, reversal or one additional point. Move and ability channels remain distinct. Reviewed ordinary stat cards and Parting Shot use the same sourced protection/reflection/reversal/retaliation path. | unreviewed-stat-moves |
| Corrosion | Source-tracked Poison bypasses type/status poison immunity; global battlefield status restrictions still apply. | ability-bypass-interactions |
| Costar | On field entry, copy one active ally's supported positive move and ability attribute/defense channels. One eligible ally is automatic; multiple allies use frozen entry snapshots and an owner choice. Highest modifiers do not stack within a channel; sheets and ability identities are unchanged. Reviewed Coil/Hone Claws accuracy-pool dice and Focus Energy High Critical copy independently with their own units; other trait definitions remain manual. | unreviewed-trait-channels |
| Dancer | Once per round, choose a free copy of a completed Dance move, including self and field targeting, inheriting Accuracy successes and rolling fresh damage. Owner target choice, GM random-foe selection, normal move/status/range guards and one-use rampage copies apply. Engine 0.17 preserves each original target accuracy result through the copied action, including missed sources; new targets use the highest recorded source count. Unreviewed dance added effects remain explicitly adjudicated. | unsupported-dance-move-definitions |
| Dark Aura | Reviewed range-limited Dark/Fairy damage aura and Aura Break inversion. Duplicate sources do not stack; attack/cooperation narrative restrictions require a GM ruling. | narrative-relationship-rulings |
| Dauntless Shield | On entry, add two temporary Defense or Strength respectively. Outside-battle narrative effects are GM adjudicated. | narrative-effects |
| Defiant | Sourced stat changes trigger reviewed protection, once-only retaliation, single reflection, reversal or one additional point. Move and ability channels remain distinct. Reviewed ordinary stat cards and Parting Shot use the same sourced protection/reflection/reversal/retaliation path. | unreviewed-stat-moves |
| Early Bird | Structured Sleep duration is halved (rounded up); waking requires two cumulative Insight successes. Rest is excluded. Narrative sleep remains manual. | outside-battle-activities |
| Fairy Aura | Reviewed range-limited Dark/Fairy damage aura and Aura Break inversion. Duplicate sources do not stack; attack/cooperation narrative restrictions require a GM ruling. | narrative-relationship-rulings |
| Flower Veil | Protect the reviewed stat changes through sourced battle effects and supported moves; Clear Body also rejects external increases and Flower Veil protects in-range allies. | unreviewed-stat-moves |
| Forewarn | Private strongest-move message to the linked trainer owner, falling back to Pokemon owner when unlinked. GM compares current foe cards using printed Power, handles variable/fixed damage and ties, and records the trainer Insight result without inventing a difficulty. Candidates/rulings remain private; host transport, rollback and persistence are supported. | See recorded scope |
| Full Metal Body | Protect the reviewed stat changes through sourced battle effects and supported moves; Clear Body also rejects external increases and Flower Veil protects in-range allies. | unreviewed-stat-moves |
| Gale Wings | Add Reaction 1 to Flying moves without imposing a video-game full-HP requirement. Retain higher printed early priority; conflicting Late Reaction needs a recorded timing ruling. Charge reactions still need adjudication. | conflicting-late-priority-ruling, charge-reaction-definition |
| Gluttony | May hold three berries with different reviewed effects. Ignores negative ingestion effects for the 17 reviewed ordinary medicines, including herbal acceptance/happiness penalties. Other narrative food ingestion remains GM-managed. | narrative-food-ingestion |
| Guard Dog | Sourced stat changes trigger reviewed protection, once-only retaliation, single reflection, reversal or one additional point. Move and ability channels remain distinct. Reviewed ordinary stat cards and Parting Shot use the same sourced protection/reflection/reversal/retaliation path. Foe Switcher Moves are prevented; Teravolt/Turboblaze bypass this non-unique prevention. | unreviewed-stat-moves |
| Heavy Metal | GM records an increased encounter weight for the exact current form. Reviewed Low Kick/Grass Knot and Heavy Slam/Heat Crash formulas use literal 50 kg steps capped at four dice; no fixed weight multiplier is invented. Suppression restores ordinary weight; saved shared damage freezes the resulting bonus. | See recorded scope |
| Hospitality | On entry the owner may spend one Will to heal one wounded active ally by three HP. No revival; lethal wounds remain manual. | lethal-resource-system |
| Hyper Cutter | Protect the reviewed stat changes through sourced battle effects and supported moves; Clear Body also rejects external increases and Flower Veil protects in-range allies. | unreviewed-stat-moves |
| Illuminate | In-range user/allies ignore the supported fog/visibility accuracy penalty; narrative encounter effects remain manual. | narrative-encounter-effects |
| Intimidate | Entry-only rank-limited Strength reduction, non-stacking, removed when source leaves or faints. GM range defaults to all deployed foes; stable IDs break excess-target ties. An explicit GM range correction updates affected targets. | See recorded scope |
| Intrepid Sword | On entry, add two temporary Defense or Strength respectively. Outside-battle narrative effects are GM adjudicated. | narrative-effects |
| Justified | Dark hits raise the separate Strength ability channel by one, capped at three; narrative injustice triggers remain GM adjudicated. | outside-battle-activities |
| Levitate | Ground immunity and supported grounded terrain/entry-hazard exclusions already use the battlefield service. Source/precedence edge review is in M3; this is not a missing hazard subsystem. | See recorded scope |
| Light Metal | GM records a reduced encounter weight for the exact current form. Reviewed weight formulas use the recorded kg and literal 50 kg steps capped at four dice. No half/quarter multiplier is silently chosen; suppression and shared damage snapshots preserve the selected inputs. | See recorded scope |
| Magic Bounce | Reflect executable foe Support effects aimed at the holder or its side, once, without extra actions/accuracy. Preserve original resource payer and reflected effect source. Unimplemented Support effects still require adjudication; manual field records retain reflected scope. | unsupported-support-effects |
| Magic Guard | Weather and supported item/hazard protection already exist. M3-F02 is corrected in reviewed encounters: Perish Song bypasses Magic Guard. Historical behavior is retained; other named exceptions need scoped review. | M3-F02 |
| Magnet Pull | Block ordinary recall for eligible targets in confirmed abstract range, with rank target caps and reviewed Ground/Steel/Ghost exceptions. GM switching overrides remain explicit. User Switcher Moves respect trapping. Whether external forced replacement overrides printed blocking remains an explicit GM exception; no video-game precedence is assumed. | forced-switch-trapping-ruling |
| Mirror Armor | Sourced stat changes trigger reviewed protection, once-only retaliation, single reflection, reversal or one additional point. Move and ability channels remain distinct. Reviewed ordinary stat cards and Parting Shot use the same sourced protection/reflection/reversal/retaliation path. | unreviewed-stat-moves |
| Mycelium Might | Support moves gain Never Miss and Late Reaction 1; no ability bypass is granted by the printed card. Printed Reaction collisions require explicit GM adjudication. For printed Reaction collisions the GM can persist printed or ability Late Reaction timing before damage; Never Miss remains in both choices. | conflicting-printed-reaction-ruling |
| Neutralizing Gas | Negates foes in abstract range subject to rank target cap; restores on source departure, fainting or range change. Unique abilities prevail. Opposing Gas stalemates require GM ruling. | GM ruling for opposing Gas stalemates |
| Oblivious | Immune to In Love; other feeling-related and narrative effects remain GM-adjudicated. | See recorded scope |
| Opportunist | Copy supported foe move/ability attribute and defense increases on authoritative state changes, retaining the highest within each channel. Immutable before/after observation prevents reciprocal loops. Decreases and unchanged modifiers do not trigger a copy. Reviewed Coil/Hone Claws accuracy-pool dice and Focus Energy High Critical copy independently with their own units; other trait definitions remain manual. | unreviewed-trait-channels |
| Power of Alchemy | On active allies fainting, store multiple eligible ability snapshots for 1440 game minutes, with one active copy. Acquisition requires Power of Alchemy to be active; selecting within combat requires a GM timing ruling. Campaign clock, provenance and rollback persist. | GM ruling for selection timing during combat, Narrative acquisition outside witnessed campaign fainting |
| Prankster | Add Reaction 1 to Support moves. Retain higher printed early reaction priority; conflicting Late Reaction requires a recorded printed/ability timing ruling. Unsupported move effects and charge reaction exceptions remain explicit. | conflicting-late-priority-ruling, charge-reaction-definition |
| Receiver | On an active ally fainting, the owner chooses one eligible ability snapshot. One unexpired copy, preserved in campaign state for 1440 game minutes; the GM advances game time between encounters. Unique donor abilities cannot be copied. | Narrative acquisition outside witnessed campaign fainting |
| Rivalry | GM chooses supplied JSON gender versus printed preference wording and records individual comparison groups. Same/opposite foe comparisons change Strength by +1/-1 through existing channel/cap rules, including target-specific accuracy prefixes, reactions and frozen shared damage. Unknown, genderless and neutral groups are neutral. | See recorded scope |
| Run Away | Ignore supported Blocked recall restrictions. Narrative escape and capture consequences require GM adjudication. | narrative-escape-effects |
| Sand Veil | Weather damage immunity; the table rule still limits Evasion to once per turn and five total actions per round. | See recorded scope |
| Serene Grace | Add two chance dice to supported move added effects; unreviewed chance effects still need GM resolution. Batch 5 adds 23 exact chance definitions with sourced status immunity, Shield Dust and Sheer Force behavior; unsupported multi-effect cards remain explicit. | unreviewed-chance-moves |
| Shadow Tag | Block ordinary recall for eligible targets in confirmed abstract range, with rank target caps and reviewed Ground/Steel/Ghost exceptions. GM switching overrides remain explicit. User Switcher Moves respect trapping. Whether external forced replacement overrides printed blocking remains an explicit GM exception; no video-game precedence is assumed. | forced-switch-trapping-ruling |
| Simple | Sourced stat changes trigger reviewed protection, once-only retaliation, single reflection, reversal or one additional point. Move and ability channels remain distinct. Reviewed ordinary stat cards and Parting Shot use the same sourced protection/reflection/reversal/retaliation path. | unreviewed-stat-moves |
| Snow Cloak | Weather damage immunity; the table rule still limits Evasion to once per turn and five total actions per round. | See recorded scope |
| Supersweet Syrup | Foes in confirmed range cannot Evade the holder moves; Clash remains available. Narrative encounter-rate changes remain GM adjudicated. | narrative-encounter-effects |
| Synchronize | Reflect source-tracked statuses once, respecting immunities. GM status edits without an inflictor have no inferred source. | See recorded scope |
| Teraform Zero | Active Terapagos removes supported battlefield effects, weather and terrain and suppresses ordinary abilities. Existing unique-ability protection remains; conflicting unique effects require a recorded GM ruling. | unique-ability-stalemate-ruling |
| Teravolt | Action-scoped foe type/damage/status immunity bypass, supported shields, decoys, redirection and charging-target access. Unique-effect stalemates and unsupported effect blockers require GM adjudication. Reviewed holder immunities are integrated with the item definitions in engine 0.23. Force fields, Cover and non-unique forced-switch prevention are also bypassed in engine 0.22. | unique-stalemates, unsupported-support-effects |
| Torrent | Water accuracy/damage ignore pain at every HP level. GM chooses the conflicting text two-dice versus icon one-die bonus; the chosen Water damage-pool bonus applies at half HP or less and is frozen for shared damage. Support moves require no damage-bonus ruling. | See recorded scope |
| Triage | Reviewed Basic Heal and status-cure Support moves gain Reaction 1, including selected allies and multi-target cures. Existing action, capacity, target, Will and phase validation apply. Unsupported healing/cure definitions remain manual. | unsupported-healing-cure-definitions |
| Turboblaze | Action-scoped foe type/damage/status immunity bypass, supported shields, decoys, redirection and charging-target access. Unique-effect stalemates and unsupported effect blockers require GM adjudication. Reviewed holder immunities are integrated with the item definitions in engine 0.23. Force fields, Cover and non-unique forced-switch prevention are also bypassed in engine 0.22. | unique-stalemates, unsupported-support-effects |
| White Smoke | Protect the reviewed stat changes through sourced battle effects and supported moves; Clear Body also rejects external increases and Flower Veil protects in-range allies. | unreviewed-stat-moves |
| Wonder Guard | Prevent non-super-effective move damage and supported indirect damage; condition damage still applies. | ability-bypass-interactions |
| Wonder Skin | Reduce chance dice for supported incoming added effects by two; independent selectors remain independent. Batch 5 adds 23 exact chance definitions with sourced status immunity, Shield Dust and Sheer Force behavior; unsupported multi-effect cards remain explicit. | unreviewed-chance-moves |

## References and completion discipline

[Ability registry](../rules/ability-coverage.json), [special move registry](../rules/special-moves.json), [targeting registry](../rules/tactics-reviewed.json), [reviewed ability batch](battle-reviewed-ability-batch.md), [rules](RULES.md), [headless 3v3](headless-3v3.md), [training points](training-points.md), [catalog storage](catalog-storage.md).

Before closing a task, record what changed and what was actually verified. Build success, native startup, interactive use and physical-machine LAN testing are distinct results. Keep this inventory synchronized with registry status changes.

### Item completion milestone

Application 0.30.0 / engine 0.23.0-core / protocol 31 delivers ordered milestone **6 of 7**. See [item and medicine rules](battle-item-completion.md). Next is **milestone 7: native and physical LAN acceptance**. Installer execution, upgrade/data preservation and physical multi-device acceptance have not been performed.
