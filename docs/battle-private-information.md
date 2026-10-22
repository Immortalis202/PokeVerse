# Private battle information

Application **0.27.0**, engine **0.20.0-core**, protocol **28** delivers milestone 3: Anticipation and Forewarn. Frisk remains the reviewed held-item reveal from milestone 1. Database schema stays 4.

## Evidence and review decisions

The supplied v3.0 ability cards and MASTER printed pages 537 and 546 are authoritative. Anticipation alerts the trainer when a foe knows a move that could deal lethal or super-effective damage. It does not reveal a move or identify a foe. Forewarn tells the trainer the strongest move one of its foes has, in secret. Its description calls for trainer Insight to understand the message, but gives no difficulty. The printed illustration describes high Power moves; it supplies no universal formula for comparing fixed/variable damage, attributes, conditional effects or ties.

The GM sees the exact current foe cards and their printed/effective Power, damage attributes and effects. Highest numeric Power is a comparison aid. The GM selects the strongest move and records comparison/tie reasoning and the Insight outcome. No video-game power substitutions, random tie policy, Insight difficulty or automatic damage rolls are invented. No message / failed Insight and a private GM ruling are explicit alternatives. Both registry entries retain partial status for these unprinted timing/comparison and unsupported classification boundaries; the private transport is implemented.

## Warning windows and classification

New encounters enable `TurnFlow.information`. A conscious deployed source opens a warning window on entry, acquiring the ability or regaining it after suppression. It waits if there are no conscious deployed foes. A change to foe rosters, known movepools, ability/type identities, the source's type/ability or current weather refreshes the window. Ordinary rounds and actions do not repeatedly disclose a movepool. Recalled, fainted or suppressed sources lose pending windows. Reserves and allies are not foes. Timing outside these reviewed windows remains a recorded GM decision.

Anticipation checks the current virtual form movepool, not just the permanent sheet. Structured `Attributes.Lethal` and supported type effectiveness, conversions, type/ability immunities, Tera Shell and Strong Winds feed the classification. Support and self/ally-only cards do not qualify merely because their type matches a weakness. Copy-call cards (including Ally-targeting Assist), environment-dependent Nature Power, and unresolved definitions/types remain visible to the GM, who records their classification instead of silently treating them as safe. A known classified threat offers an alert; a known safe set offers no classified threat. A generic manual ruling can close an expired or unsupported window. This only classifies a warning: it never inflicts lethal wounds or automates excluded lethal damage.

Forewarn lists distinct known damaging move names per deployed foe, including zero/variable/fixed-power cards, copy callers and unresolved definitions. Conditional damaging cards such as Bide/Present remain candidates even when the current action projection labels them Support. It does not silently invent their strength or choose the first tied entry. The GM confirms Insight and comparison in a private reason before selecting a message. A reveal names only the selected move, without the full candidate list or foe identity. A conditional move's actual future success is not predicted.

## Recipients and secrecy

A linked trainer sheet's owner receives the warning. An unlinked Pokémon falls back to its own owner; this fallback is explicit rather than dropping the message. A differently owned linked trainer receives the warning instead of the Pokémon's owner. The GM can inspect candidates and decisions. The authoritative host retains complete state for validation and persistence.

Player projections remove every pending information offer and window signature, including from archived battles and undo checkpoints. Only the intended trainer receives the resulting alert/move. GM comparison/Insight notes are removed even from the trainer's message; public event descriptions contain no selected move, candidate index, result or private reason. Authorization runs before dice counting so guessed candidate indices cannot probe the private choice count.

For information-enabled encounters, foreign learned moves and workflow learned/forgotten/ledger copies are hidden in permanent and virtual/original form sheets. Foreign Gorilla Tactics entry options and prepared form builds are also hidden. Already announced attacks, public battle history and user-authored free text remain normal shared campaign information; this is not a full fog-of-war rewrite or a way to erase knowledge a player already received. Owners still see their own movepool and use the normal battle previews. The host always validates commits against its authoritative campaign.

## Persistence, rollback and compatibility

Warnings use normal stable room commands, revisions, duplicate receipts and checkpoints. Accepted commands, broadcasts, initial joins, reconnects, local host views, persisted replicas, archives and undo use the same privacy projection. Rollback restores the pending warning and removes the delivered in-app message; it cannot erase a person's memory of a revealed message.

Missing saved information state defaults to disabled. Engine 0.19 retains transformations and all earlier gates; 0.18 retains held items and preceding mechanics. No active historical encounter is silently upgraded. The new engine can be adopted between battles, and all room clients need matching builds/protocol/catalog fingerprints. Supplied source data stays untouched.

## Verification

Release checks for application 0.27.0:

- **597 Rust checks:** 104 binding exports and 493 behavioral tests passed. The final full suite includes ten private-information fixtures, including a five-installation localhost test with separate host, GM, Pokémon owner, linked-trainer owner and observer, plus existing gym/held-item/transformation regressions.
- **54 frontend tests** and **14 catalog checks** passed. TypeScript, formatting, warnings-denied Clippy and staged diff checks passed.
- Windows production build and NSIS packaging passed: `src-tauri/target/release/bundle/nsis/PokeVerse_0.27.0_x64-setup.exe` (**214.31 MiB**). The final installer was rebuilt after the classification correction.
- Source review covers the supplied ability cards, MASTER 537/546, structured lethal/type rules, copied/conditional move classification, secret candidate/GM-note projections, trainer recipients, stale-input guards, receipts, reconnect, persistence and rollback.

Native interaction, installed upgrade/data preservation, signing and physical LAN acceptance remain milestone 7 work. Neither executable nor installer is launched during packaging.
