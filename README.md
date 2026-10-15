# PokeVerse v0.11.0

Windows desktop workspace for Pokerole 3.0. Built with Tauri 2, React/TypeScript, and Rust. Each installation keeps its library locally and can host a persistent campaign over LAN or a private VPN.

The default Nocturne workspace uses RodeoPokemon's dark purple palette, a compact navigation rail and a toolbar with campaign, connection and separate host/GM status. Creation, progression, training encounters and battles retain their existing workflows. Version 0.11.0 uses protocol 12 and retains database schema 4; all room clients must use matching builds.

## Run

Prerequisites: Windows 10/11 x64, Node.js 24+, Rust with the MSVC target, Visual Studio C++ Build Tools, and WebView2.

```powershell
cd Y:\PokeVerse\PokeVerse
npm.cmd ci
npm.cmd run catalog
npm.cmd run desktop
```

The existing `../Pokerole-Data` directory must remain available when generating the catalog. The built application bundles its catalog and sprites and does not need that directory or the internet at runtime.

```powershell
npm.cmd run build
npm.cmd test
npm.cmd run test:rust
npm.cmd run package
```

The Windows NSIS installer is emitted under `src-tauri/target/release/bundle/nsis/`. It is configured to include the offline WebView2 installer; the packaging machine must download that installer on its first build. The installer is unsigned unless you configure signing for distribution.

## First session

1. Create a trainer in Library, then create its Pokémon through the guided steps. Defaults are a Rookie teen trainer, matching starter rank, Happiness/Loyalty 2, and zero Training Points. Preview shows baselines, allocations, modifiers, unused points and derived resources.
2. Progress offline through named operations. Rust calculates budgets, prices and resulting sheets. Save narrative permissions or unsupported catalog details as drafts; submit them in Room for a recorded GM decision. Accepted Training Points can be spent offline without a server. The GM awards the trainer pool; use Library�s **Training Point pool** to review and distribute points one-for-one to that trainer�s Pok�mon offline. Linked transfers and dependent purchases synchronize together. See [Training Points](docs/training-points.md).
3. In Room, create a campaign and start hosting. Share the displayed port, your LAN/private-VPN IPv4 address, and access token. Other players join using `ws://HOST-IP:39231/room`.
4. Allow the app through Windows Firewall on the **private** network if Windows requests it. The app does not change firewall settings. Do not expose this unencrypted v1 transport to the public internet.
5. The host can use Room → Choose the game master → Refresh members to appoint a joined player, or retain/reclaim the GM role. Hosting and validation always stay on the host computer. Synchronize the trainer before proposing a captured Pokémon or opponent. The host replays legal offline operations; conflicting progression branches must be reconciled as a whole.
6. In Battle, the GM registers the full Pokémon roster, assigns sides, sets active slots (default three per side), and marks the starters. Reserves remain registered without acting or being targeted. Roll initiative, select a learned move, and use automatic accuracy. After resolving reactions, roll automatic damage; the host computes dice and updates HP. Owners or the GM can switch with their trainer's conscious reserves, including replacements for fainted Pokémon. See [registered battle teams](docs/battle-teams.md).
7. End the battle, issue rewards, and reconcile sheets or pull campaign outcomes. Registered Pokémon, including reserves, cannot import sheet edits mid-battle.

GM opponents are ordinary sheets in the GM's personal library. Create a trainer for them, add their Pokémon, and synchronize them before battle. A character belongs to one campaign; copy the trainer and squad to use separate progression in another campaign.

## Training encounters

Open **Training encounters** after selecting a campaign in Room. The appointed GM privately generates and edits a habitat-based horde, reveals it, then confirms captures and manual Training Point awards. Victory is assumed; this does not start a battle. Captured sheets reuse the creation rules and import into their owners' offline libraries. Room also offers offline preparation from a saved campaign snapshot. Existing campaigns require explicit catalog adoption. See [the workflow and persistence details](docs/training-encounters.md).

## Implemented

- Guided trainer/Pokémon creation and legacy review; eight ranks, trainer age bonuses, species baselines/limits, rank allocation totals, skills, socials, Happiness/Loyalty and derived HP/Will/defenses/initiative/move capacity.
- Offline rank-up, free unused allocations, priced retraining, move learning/replacement/forgetting, pre-evolution recall, over-ranking, TM/TR consumption, evolution preparation and completion. Narrative permissions and awards require campaign-scoped GM approval.
- Ordinary available abilities are distinct from the selected battle ability. Huge Power/Pure Power use permanent modifiers without consuming points; conditional abilities remain manual. Encounter ability and rules profile are frozen.
- All 1,200 species/forms and 892 moves in the supplied v3.0 data, plus abilities, items, natures, and local sprites.
- Catalog validation, persistent progression history, bounded fields, species attribute maxima, move/ability references, and trainer ownership relationships.
- Local SQLite databases, migrations, consistent backups, same-identity backup restoration, and portable JSON exports/imports as new character copies with copied progression history.
- Host-authoritative persistent campaigns, authenticated reconnect credentials, up to eight remote players, ordered revisions, atomic command receipts, and automatic reconnect with snapshots.
- Three-way reconciliation of offline and campaign edits; explicit conflicts for competing changes; duplicate-command protection for mutations and rewards.
- Multiplayer manual battles with multiple combatants, targeting, learned-move checks, host d6 rolls, resource changes, conditions, action/reaction tracking, initiative adjustment, GM explanations, and completed battle records.
- Registered Pokémon teams with configurable 1–6 active slots per side, reserves, owner/GM switching, fainted replacements, Trainer Area switching costs, next-round availability, persistent deployments and rollback.
- Unacknowledged room commands retained in the frontend until retried with their original IDs, including after app restart.

## Verified core rules and remaining coverage

The supplied `POKEROLE COREBOOK 3.0 RULING.pdf` and `POKEROLE COREBOOK 3.0 POKEDEX MASTER.pdf` are now the rule references. See [the rule coverage matrix](docs/RULES.md) for page references, implemented formulas, and remaining work. The PDFs are not bundled into the installer.

New campaigns use engine `0.4.0-core`: explicit turns, multi-target attacks with shared damage dice, field-effect records, area Clash, Cover an Ally, Evasion/Clash, Will spending, structured major statuses and reviewed effects. Shared Rust calculations retain ordinary damage, defenses, STAB, critical bonuses and flat type adjustments. Reviewed Champion sheets inherit Master battle bonuses. Legal creation/progression recalculates permanent derived resources while preserving wounds and spent Will.

Existing campaigns retain `0.3.0-core`, `0.2.0-core` or `0.1.0-manual` until the appointed GM upgrades between battles. Previous battle records remain readable. Matching 0.11.0 builds, protocol 12 and catalog hashes are required. This update adds defaulted JSON fields and leaves user database schema 4 unchanged.

**Coverage is explicitly partial.** Protect, Recover, Agility, Thunder Shock/Confusion chance effects, Sonic Boom set damage, Submission recoil and Hyper Beam recharge have reviewed definitions. Selected conditional abilities are supported. Other special effects, multi-target chains, varied damage, rampage/successive actions, weather, temporary typing, trainer damage and lethal wounds remain GM-adjudicated. See [rule coverage](docs/RULES.md) and [machine-readable coverage](rules/coverage.json). Full PDF verification remains open; field-level review reports are generated under the workspace's `outputs/rule-coverage/`.

## Battle configuration and recovery

Server validation runs automatically; the host player never needs to approve individual commands. Players can roll accuracy for their own Pokémon directly. New core battles enable automatic rule enforcement. The GM can toggle it in Battle → Battle configuration & recovery with an explanation. On enforces the supported fainted/learned-move/repetition/five-action checks, including turn/reaction sequencing. Structured actions are required while automatic validation is enabled. Off permits rule exceptions while keeping ownership, GM permissions, command receipts and numeric bounds. Turning it off does not implement unsupported move effects or formulas.

The GM can roll back the last battle step, repeatedly up to 20 stored steps. Recovery includes rolls, rounds, configuration, combatant adjustments and battle ending. Original dice, events, command receipts, host and GM identities are retained; rollback is recorded as a new event with a new campaign revision. Retrying an undone command does not reapply it. Undoing battle ending restores sheet HP/conditions/Will while keeping sheet revisions monotonic. Starting a new battle, synchronizing sheets, awarding rewards, or upgrading rules clears rollback history to prevent overwriting later campaign changes. Recovery history survives host restarts.

Battle-only corrections can change attributes, skills and maximum HP, and affect subsequent accuracy and initiative calculations. An empty override map restores sheet scores. Existing GM adjustments cover current HP, conditions, initiative and action/reaction counters. Overrides are reasoned, logged and rollbackable. Ending the battle persists HP (capped to permanent maximum) and conditions, without changing permanent attributes, skills or maximum HP.

Pokémon XP migrates one-to-one to Training Points exactly once, preserving original balances and original sheets in migration history. Trainer XP remains legacy data; trainer rank advancement requires GM-confirmed achievements. Migrated/imported/copied characters require guided review and cannot enter strict battles until accepted. Automated training-session earnings remain outside this milestone.

Creation and progression use `src-tauri/src/creation.rs` for previews, commits and host replay. Prices come from reviewed manual fixtures in `rules/manual-fixtures.json`. The correction layer in `rules/catalog-corrections.json` compares PDF vector dots and rank icons; `rules/catalog-reviewed.json` records visual reviews. Ambiguous forms, overprinted PDF text, missing references and unresolved prerequisites remain visible and require adjudication. Source files are never rewritten.

## Storage and recovery

Tauri uses the Windows application-data directory for `com.pokeverse.desktop` (normally `%APPDATA%/com.pokeverse.desktop`). It contains `library.db`, `campaigns.db`, `backups/`, and a bounded `logs/app.log` with diagnostic codes. Tokens and credentials are not logged.

Use Settings → Create backup before experimenting with real campaign data. Restoration requires leaving the room, an explicit replacement checkbox, and a generated backup from the same player identity. A safety backup is created first. JSON import preserves the original library and creates new character IDs; use SQLite backup restoration when exact identities and campaign memberships matter.

The host's campaign database owns the accepted battle state. Campaign host ownership is independent from the appointed GM and persists across role changes and restarts. Only the host can appoint an authenticated campaign member as GM, including a member who joined in a prior session, or reclaim the role. Changes take effect immediately, including during a battle. The host is an ordinary player when someone else is GM; battle management and rewards require the current GM. If the host disappears, shared play pauses while personal editing remains available. Rehost the same campaign at the same address and port to reconnect. Room tokens persist across host restarts. There is no automatic host migration. Older campaigns infer their original host from the old GM field until the first appointment pins the host explicitly.

## Architecture

React calls typed Tauri commands. Rust owns database writes, the rules boundary, the room server, and the remote WebSocket connection. Axum runs the host server on Tokio; a per-store mutex serializes campaign mutations. Each accepted command atomically persists the campaign and receipt before broadcasting a snapshot. Stale revisions are rejected; retries use the original command ID.

Rust/Serde models generate TypeScript contracts through `ts-rs` during `npm run bindings` or Rust tests. Library and campaign databases are separate. Synchronization uses the last accepted baseline; descriptive changes merge independently. Mechanical progression, point balances and inventory purchases form one conflict group, with host replay of operation provenance. Protocol 11 requires the same application version, corrected catalog and rules engine.

Normal installations use a single-instance guard. For isolated portable installations or QA, place a `portable-data` directory beside the executable: that installation stores its databases there. Separate portable directories can run simultaneously; an exclusive Windows file lock prevents two processes opening the same profile. Portable mode needs WebView2 already installed.

Source `DexID` and `_id` values collide in a few entries. The catalog adds stable application IDs combining species `DexID` with `_id`, or other entries' `_id` with name, preserving source fields and every form. Settings exposes the 35 missing learnset/ability references detected in the current source data.

Browser `npm run dev` is a visual preview only; SQLite and room operations require the desktop app. V1 has no cloud accounts, relay, tactical map, automatic updates, or public matchmaking.

Pokerole and Pokémon content remains attributed to the source dataset and its original creators; see `../Pokerole-Data/readme.md` for credits.

Automatic damage currently covers Pokémon targets. Trainer combat damage requires GM adjudication; the engine does not assume humans have a Pokémon type.

Reference catalog storage uses a bundled read-only SQLite database, indexed lookups and on-demand detail loading. See [catalog storage](docs/catalog-storage.md) for schema and compatibility details.

## Offline compendium

Open **Compendium** to browse all 1,200 species/forms, moves, abilities, items and natures. Species have Stats, Abilities, Evolutions and Learnset tabs. Related entries open in the detail pane; Back restores the preceding tab and scroll position while preserving the list, filters and page. Missing or ambiguous exact references remain visible. Recorded requirements and metadata warnings are reference information, not character-specific eligibility. Details are queried from the bundled read-only SQLite catalog; this release keeps protocol 12 and user schema 4.

The Stats tab uses a violet corebook-style species card. Filled dots show the corrected species baseline; outlined dots extend to its species limit. Numeric minimum/maximum labels and accessible score descriptions accompany the dots. These are species reference scores, not allocated character points.
