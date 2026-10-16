# PokeVerse — remaining work

Snapshot: 16 October 2026, application 0.15.0, protocol 16, user database schema 4.

The Training Point milestone below is implemented. Remaining battle, catalog and native acceptance work stays open.

## Existing foundation

- Offline trainer/Pokémon creation and named progression operations, with shared Rust validation, derived resources, budgets and reviewed costs.

- Campaign synchronization, campaign-scoped GM approvals for exceptions, and automatic host validation. The host and GM can be different players.

- Habitat-based training encounters, private preparation, published rosters, assumed victory, GM-confirmed captures, trainer assignment and manual rewards.

- Registered battle teams, active slots and reserves, switching/replacement, supported initiative/accuracy/damage automation, action counting, overrides and rollback.

- Offline Compendium for all 1,200 species/forms, linked references and violet minimum/maximum species cards.

These foundations do not imply complete rule coverage. Legal offline progression already works; GM approval is needed for exceptions and narrative permissions, rather than every ordinary edit or purchase.

## 1. Completed: trainer Training Point pool and player distribution

### Required behavior

The GM awards a general Training Point balance to a campaign trainer. The player decides how to distribute accepted points among that trainer's Pokémon. Awards must no longer require the GM to choose individual Pokémon, including awards from training encounters.

- [x] Add a clear trainer pool display and a player distribution workflow with amount, recipient and final review.

- [x] Replace Pokémon-recipient reward controls in encounters and general campaign awards with trainer recipients.

- [x] Keep campaign awards/corrections GM-authorized and record the source, reason and approving identity. Offline proposed awards remain pending.

- [x] Allow distribution of already accepted points offline, without another GM approval. Restrict recipients to Pokémon owned by the player and attached to the selected trainer.

- [x] Treat distribution as a single operation: atomically debit the trainer and credit the Pokémon, with provenance, stable operation IDs and expected revisions for both sheets.

- [x] Revalidate distribution on the host. Retries, reconnects and repeated imports must not duplicate points; concurrent offline distributions must not independently merge into an overspent pool.

- [x] Support conflict reconciliation for the complete transfer chain, including dependent purchases using transferred points. Rejected transfers must not leave a debit or credit behind.

- [x] Preserve existing Pokémon balances, awards and history. Do not silently move previously awarded points back to trainers. Keep legacy trainer XP separate from the new pool.

- [x] Update typed commands/bindings, encounter outcomes, synchronization, history, version/compatibility checks and migration backups where required by the final design.

### Adopted decisions

- Distribution is one-for-one. Disobedience never silently reduces an accepted pool transfer; specific earning formulas remain manual.

- Transfers are one-way; GM corrections are explicit and logged.

- Matching 0.9.0 clients use trainer awards immediately. Historical Pokémon awards and balances remain unchanged; legacy trainer XP stays separate. No database migration is needed.

Validation details: [Training Points](training-points.md). Full interactive Windows acceptance remains pending in section 4.

### Acceptance

- [x] GM awards 100 points to a trainer; the player distributes 40 and 30 to two Pokémon, leaving 30 in the trainer pool.

- [x] Accepted points can be distributed and spent offline, then synchronize once on reconnect.

- [x] Negative amounts, insufficient balance, another player's recipient and mismatched trainer links are rejected.

- [x] Stale revisions, concurrent branches, duplicate submissions and database failure cannot duplicate or lose points.

- [x] Historical awards and current Pokémon balances survive upgrade, backup restoration and repeated imports.

## 2. Battle rule coverage

A reviewed coverage matrix and field-level audit queue are implemented in 0.10.0. Explicit single-target turns/reactions, held-turn recovery, Evasion/Clash, Will, major statuses and ice breaking have fixtures. Selected healing/shield/chance/recoil/recharge/set-damage effects and conditional abilities are implemented. 0.11.0 adds multi-target ordinary attacks, rank-limited area selection, shared damage dice, field-effect records, area Clash and Cover an Ally. 0.13.0 adds typed weather, terrain, force fields, rooms, temporary typing and arena conditions. 0.14.0 adds conditional/reactive ability families, bringing automated or partial ability scopes to 209 of 305. 0.15.0 adds reviewed charge, rampage, successive-action and healing/shield families. Multi-target reaction moves and individually listed remaining ability definitions remain open. See [advanced targeting](battle-targeting.md). See [RULES.md](RULES.md) and [coverage.json](../rules/coverage.json).

- [x] Verify Champion battle roll/rank inheritance and implement the supported bonuses. Champion creation/progression is already supported; reviewed Champion sheets inherit Master battle bonuses.

- [ ] Complete turn and reaction sequencing, including evasion/clash handling and enforcement of applicable move restrictions.

- [ ] Implement structured status modifiers, durations, expiry and resource changes, including the remaining Will-related battle mechanics.

- [x] Add the first reviewed ability milestone: 51 supported definitions, entry/round/hit hooks, owner decisions, chance dice, status provenance, receipts and rollback.

- [x] Add weather/terrain-dependent and extended conditional/reactive ability families.

- [ ] Complete the remaining 96 ability definitions, including held items, form changes, suppression, redirection and individually listed unimplemented event/reaction definitions. See [battle abilities](battle-abilities.md).

- [x] Add reviewed special move families in 0.15.0: healing, shields, fixed/variable pools, charge, rampage, successive actions (including reaction sequences), recharge and recoil. See [special moves](battle-special-moves.md).

- [ ] Resolve the remaining special-move boundaries: Bide wording requires a recorded GM choice; Snatch decoys and ability modifiers need new systems; Shadow Force recovery restrictions outside the encounter need persistent tracking; Max Guard requires transformations. Other unreviewed move interactions remain explicit adjudication.

- [x] Enforce Never Miss's evasion prohibition; supported accuracy still needs its success check.

- [x] Review and implement supported temporary typing, battlefield effects and arena conditions.

- [x] Retain manual adjudication, logged overrides, idempotency and rollback as coverage expands.

Acceptance: each automated mechanic has reviewed fixtures and end-to-end tests; unsupported mechanics remain clearly identified. Existing ordinary damage automation must continue to work.

## 3. Catalog fidelity and missing assets

- [ ] Complete manual review of unresolved species baselines/limits, forms, move ranks, evolution stages and prerequisites. Preserve the application-owned correction layer and original source files.

- [ ] Resolve the current 35 missing catalog references through reviewed corrections; keep ambiguous/missing names visible rather than choosing arbitrary matches.

- [ ] Supply or review the 168 missing local species/form sprites. Placeholders already preserve names and navigation.

- [x] Distinguish a resolved reference from metadata that still needs manual verification. A clickable move is not proof that its learning prerequisite has been reviewed.

- [x] Keep the manually accepted habitat CSV authoritative; preserve exclusions, pending decisions and conditional notes during rebuilds.

Counts are this release's snapshot and should be rechecked after catalog updates. Unresolved metadata must not be silently accepted into strict character/battle workflows.

## 4. Windows and multiplayer acceptance verification

Automated Rust/frontend/catalog regressions, browser layouts and isolated Windows startup have passed. Full interactive native acceptance and actual installer installation/uninstallation remain unverified.

- [ ] Install/upgrade/uninstall in isolated Windows installations and verify offline catalog/sprite availability and personal data preservation.

- [ ] Exercise native creation, progression, Compendium links/Back navigation and modal controls at the minimum window size.

- [ ] Run separate host and GM installations with three player trainers: two registered Pokémon per player versus six gym Pokémon, three active slots per side, reserves and controlled replacement.

- [ ] Exercise encounter preparation/privacy, publication, captures with mixed recipients, rewards, library imports and offline progression after reconnect.

- [ ] Verify disconnect/reconnect, duplicate commands, stale previews, replacement GM, conflicting offline branches and backup restoration through the native interface.

- [ ] Verify direct LAN hosting/joining and clear connection errors on separate machines. Record any virtual-LAN setup separately from application networking.

## 5. Deliberately deferred features

These are future scope, not missing parts of the current assumed-victory encounter flow.

- Automated training-session rolls, earning formulas and attempt limits.

- Capture dice, ball consumption and automated capture success.

- Combat simulation or automatic victory resolution inside training encounters.

- Full evolution-tree diagrams; the Compendium currently supports direct relationships and navigation through them.

- Habitat editing and character-creation shortcuts inside the Compendium.

## Excluded from project scope

Trainer-target damage, lethal wounds and automatic Will recovery are excluded from the implementation roadmap. Manual GM resource corrections remain available.

## Documentation follow-up

- [x] Reconcile older wording in [RULES.md](RULES.md) and the README with the completed creation/progression service, particularly HP recalculation and evolution/move-learning statements. Do not present old manual-only descriptions as current behavior.

- [ ] Update the coverage matrix and validation reports whenever a backlog item becomes implemented and verified.

References: [rule coverage](RULES.md), [training encounters](training-encounters.md), [battle teams](battle-teams.md), [catalog storage](catalog-storage.md). Release validation evidence is in `outputs/compendium/` at the workspace root.

## Battlefield milestone (0.13.0)

Implemented typed weather, terrain, screens, rooms, temporary typing, arena hooks and 29 dependent abilities. See [battlefield automation](battlefield.md). Torn World target adjudication, narrative escape, items, transformations and unreviewed move sequencing remain open.
