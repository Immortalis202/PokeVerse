# V1 validation

Current build: 0.4.0, verified 2026-10-10. Protocol 5; new campaigns use engine `0.2.0-core`. Legacy campaigns can retain manual rules or upgrade between battles. Host ownership is independent from GM assignment. Battle validation is configurable; recovery history and battle-only stat corrections are persistent.

Verified on Windows x64 with Node 24 and Rust 1.97/MSVC.

| Check                                     | Result                                                                                              |
| ----------------------------------------- | --------------------------------------------------------------------------------------------------- |
| Catalog generation                        | 1,200 species/forms, 892 moves; all source entries preserved                                        |
| Source reference audit                    | 35 warnings surfaced in Settings; invalid references are not silently automated                     |
| TypeScript + Vite production build        | Passed                                                                                              |
| Frontend tests                            | 2 passed                                                                                            |
| Browser UI smoke check                    | Library layout, trainer editor, save-error visibility, species search, and reference details passed |
| Rust tests                                | 50 checks passed: 31 behavior/integration tests and 19 generated-contract checks                    |
| Rust Clippy, all targets, warnings denied | Passed                                                                                              |
| npm advisory audit                        | No vulnerabilities after updating Vitest                                                            |
| Windows release compilation               | Passed                                                                                              |
| NSIS offline installer generation         | Passed                                                                                              |

Behavior checks cover persistent and idempotent saves, invalid-write rollback, form identity preservation, three-way merge/conflicts, inventory deletion, ownership enforcement, battle locking and state changes, export/import links, backup restoration, reward reconciliation, real WebSocket joins, rejected incompatible handshakes, reconnect at a changed host address, stable room tokens, and duplicate-command reward protection.

Core fixtures also cover numeric initiative and GM-only rerolls, Attribute choices, failed attempts consuming actions, move repetition and round reset, five-action exhaustion, pain and Vitality exemptions, low accuracy, critical margins, Never Miss, unsupported formulas, GM modifiers, Master bonuses and explicit legacy upgrades preserving completed encounters. Page references are in `docs/RULES.md`.

GM assignment checks cover host-only appointment, rejection of unknown identities, legacy host inference, reclaiming the role, idempotent appointments, remote GM commands over a real WebSocket, loss of GM privileges for the host after delegation, and persisted independent host/GM identities after reopening the campaign. The host performs validation and stores the accepted remote GM result.

Recovery tests cover validation toggling with retained ownership/bounds, temporary stats feeding pools, permanent-stat preservation, end-of-battle HP capping, undoing rolls/rounds/battle ending, monotonic sheet revisions, bounded persistent checkpoints and clearing recovery after rewards. A real WebSocket test verifies player accuracy is accepted automatically without GM/host approval, and rollback preserves original dice and command receipts so retrying an undone command does not reapply it. Rollback identifies the displayed step explicitly and rejects an outdated selection even if the client has already received a newer campaign revision.

## Verification limits

- The Windows UI launch tool timed out waiting for app approval. Browser UI verification succeeded after excluding Rust build artifacts from Vite's file watcher. Native desktop interaction and clean-machine installer smoke tests remain unverified.
- The supplied manuals support source-backed core-roll tests. Initiative, accuracy, action attempts, pain, low accuracy and critical thresholds are covered. Special attack effects, turn/reaction sequencing, training and evolution remain manual; see `docs/RULES.md`. Move examples were cross-checked against the PDF, but the full JSON catalog has not yet been audited.
- Network integration tests use separate databases and real loopback WebSockets; testing two physical machines over LAN/private VPN remains a release smoke-test step.
- The installer is unsigned. Its offline WebView2 payload makes it roughly 211 MiB; the application executable itself is much smaller.

Automatic damage checks cover defenses, STAB, critical dice, Master defense, pain, the minimum pool, minimum damage, weakness bonuses only on success, dual weaknesses, resistance and immunity. Fixtures cover target binding, failed accuracy, stale/single-use damage, ownership, GM adjustments including shields negating minimum damage, cancellation, pending-round guards, unsupported fixed damage and rollback. A real WebSocket test verifies host-generated damage dice, automatic HP changes, persistence and duplicate-command protection after rollback. Native damage UI remains unverified.

Automatic damage currently covers Pokémon targets. Trainer combat damage requires GM adjudication; the engine does not assume humans have a Pokémon type.


## 0.5.1 SQLite reference catalog

- Runtime reference catalog moved from embedded JSON to bundled read-only `resources/catalog.db`.
- All 76 Rust tests passed, including five new catalog regressions and existing loopback host/client, progression, migration and battle tests. Loopback tests required execution outside the sandbox because sandbox networking rejected sockets with Windows error 10013.
- Frontend: two Vitest tests and two Node preview/catalog integrity checks passed. TypeScript/Vite production build and Clippy with warnings denied passed.
- Picker summaries total 305,243 characters versus 3,360,230 characters of full entry payloads; these are serialized data sizes, not measured process memory.
- Database schema version 1; application 0.5.1; multiplayer protocol 7. Corrected catalog hash preserved, confirmed by reconstructing full records from SQLite and hashing the original catalog structure.
- Native interactive two-installation acceptance was not rerun for this storage change.
- Windows NSIS package completed: PokeVerse_0.5.1_x64-setup.exe (212.10 MiB). Generated installer script includes only resources/catalog.db; no catalog JSON is bundled.
