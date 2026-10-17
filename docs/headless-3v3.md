# Headless 3v3 gym scenario

The current Rust engine is exercised without React, WebView or PokeVerse.exe. Four identities participate: a host who also owns a trainer, two other trainer owners, and a separate appointed GM who controls the gym trainer.

## Run

From the application repository (`Y:\PokeVerse\PokeVerse`):

```powershell
npm run test:3v3
```

This runs both the deterministic engine test and the real localhost WebSocket replay. On a fresh checkout, first run `npm ci` and `npm run catalog`, and install the project's normal Rust/Windows build prerequisites. The existing compiled catalog is reused during routine test runs.

Individual tests can also be selected directly:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml scenario_3v3_engine --lib -- --nocapture
cargo test --manifest-path src-tauri/Cargo.toml scenario_3v3_websocket --lib -- --nocapture
```

The test runner prints accepted command count, round count, deployments and gym defeats. The pure engine run normally takes about one second after compilation; the network replay took about 40 seconds on the development machine. Individual network requests/reconnects have five-second deadlines; the complete engine/network scenarios have 60/120-second deadlines and explicit command/loop limits. There is no desktop executable launch or interactive Windows startup check. Codex's sandbox may still require authorization for local socket tests; that is separate from a PokeVerse Windows execution dialog.

## Scenario and assertions

- Twelve validated Standard-rank Eevee: two per player trainer, six for the gym. One player starter per trainer and three gym starters occupy three active slots per side. Run Away avoids unrelated reactive ability effects. Tackle, Quick Attack, Swift and Bite are eligible learned moves; unused allocation points remain unused.
- The GM starts the battle; owners roll initiative and the GM confirms turns. All subsequent state changes use production commands, with automatic validation enabled. HP, actions, queues and deployment are never patched directly after setup.
- Swift opens three target-specific responses. Three gym Pokémon use Quick Attack reactions; the same player successfully Clashes twice in that turn with distinct eligible moves, then fails an Evasion. A second Evasion in that turn is rejected without mutation. Parent accuracy/action accounting and shared damage dice survive the reaction chains.
- A later turn in the same round allows Evasion again, spending the player's fifth action; a sixth action is rejected. Round advancement resets action/reaction budgets. A recorded GM authorization spends real Will on pain relief, which remains spent when the battle is persisted.
- Ordinary owner-selected switches preserve trainer ownership and next-round availability. A fainted player starter is replaced by its owner. Every player and gym reserve enters the encounter, and all six gym Pokémon are defeated through ordinary moves. The GM explicitly ends the battle; victory does not silently close it.
- Unsupported ownership, reserve actions/targets, wrong-trainer replacements, premature damage, pending-chain switching/round advancement and host use of GM powers are rejected. The fixture retains the separate host/GM distinction throughout.
- Completed damage and a replacement are rolled back. Retrying their original command IDs after rollback cannot apply either operation again. New command IDs explicitly recommit the restored action.
- Final history contains one completed battle. Pokémon and trainer links, combatant identities, permanent attributes/skills, species, rank, nature, ability, learned moves and Training Point balances are preserved; HP, spent Will and resulting conditions persist correctly.

## WebSocket replay and isolation

The second test starts one actual host and three authenticated clients, bound exclusively to `127.0.0.1` on an ephemeral port. All four installations use temporary databases. After every accepted command, it compares the full authoritative campaign state with a direct Rust transition using the same dice. Fresh battle/combatant UUIDs are normalized only for initial registration; subsequent identity changes are not normalized away. Typed sets are canonicalized after that initial remapping, while ordered queues and history retain their order.

Scripted dice are a Rust `cfg(test)` facility on the host, associated with a specific command ID and validated face count/range. There is no production flag, frontend control or network field for choosing dice. Non-test builds retain the existing random-roll path and LAN listener behavior.

The network test additionally checks duplicate receipts, stale revisions, another identity reusing a receipt, reconnect during a pending response, reconnect after replacement, membership and final persisted state. Successful teardown closes clients/server and waits for database handles before reopening the host store and verifying its saved campaign and identity. Personal profiles are never opened.

## Verification and boundaries

Verified on 17 October 2026: **306 Rust tests passed**, including both scenario tests, and Clippy passed for all targets with warnings denied. Windows emitted its existing incremental-cache hard-link fallback warning; checks succeeded. No battle-rule defect requiring a production behavior change was found during this milestone.

This adds tests and test infrastructure; application 0.17.0, engine `0.10.0-core`, protocol 18 and user schema 4 are unchanged. No installer rebuild or native UI launch is needed. It is one controlled gym encounter, not exhaustive species/move/ability coverage or physical multi-machine LAN acceptance. Dynamax, Max Moves and Z-Moves remain excluded.
