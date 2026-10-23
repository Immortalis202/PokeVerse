# Acceptance preparation evidence — 23 October 2026

**Status: kit delivered; milestone 7 manual acceptance pending.** The user selected preparation of the acceptance checks. See [the runbook and 15 manual cases](windows-lan-acceptance.md). No manual case in the generated ledger is marked passed by automation.

## Release under test

App 0.30.0, engine 0.23.0-core, protocol 31, user schema 4. The existing release is from milestone 6 commit `5ebed2489dc6509dd867e55c6782791a6099f612`. This delivery changes acceptance tools, tests and documentation; it does not change application behavior, versions, catalog or installer. No replacement installer was built.

| Artifact | Bytes | SHA256 |
| --- | ---: | --- |
| `src-tauri/target/release/pokeverse.exe` | 24334848 | `586de8e3f38193dbce3b73e0449cfcc1484f6a8edb671dbf105be0df7c9c8f84` |
| `src-tauri/target/release/resources/catalog.db` | 19382272 | `284344d2c3b6c40e658e6d3d9387cedf068ff464d9da19e866f2513af3c5f7f4` |
| `src-tauri/target/release/bundle/nsis/PokeVerse_0.30.0_x64-setup.exe` | 225542128 | `476e56397a502a9b7721c65e9cd0a0506149d9995553385404017517f516ad20` |

Windows product version read from the executable: **0.30.0**. Installer Authenticode result: **NotSigned**. Trusted signing/distribution is not accepted.

## Checks actually run

- Full Rust library suite: **651 passed**, including separate host/GM/owner/observer, current-engine gym, retries, reconnects, rollback and database reopening. These are headless/localhost results.
- Frontend: **58 passed**. Catalog/script checks: **20 passed**, including three new acceptance-evidence tests. They cover read-only snapshots, canonical JSON comparison, private payload hashing, missing-file rejection without creation, incomplete evidence rejection, changed-profile detection, all five profiles, installed-profile report separation and refusal to overwrite evidence.
- TypeScript, all-target Clippy with warnings denied, Rust formatting and Git whitespace checks passed. Windows emitted its existing incremental-cache hard-link fallback warnings; compilation/checks succeeded.
- `node scripts/prepare-acceptance.mjs` successfully created five isolated profiles and verified the copied executable/catalog hashes. The first prepared lab is `tmp/acceptance/0.30.0-6c6a22a4-64b5-4168-b7d7-8e165f879dc9`; its manifest records the existing release commit and the dirty worktree containing this acceptance kit. Its `results.csv` contains 15 pending manual cases. Profiles are prepared, not initialized by an app session.

The initial PowerShell preparation approach was blocked by Windows policy. The delivered Node command uses read-only Windows metadata queries, preferring available PowerShell 7 with the legacy shell as a fallback. It does not change execution policy. If signature verification is unavailable in that shell, it records `Unavailable`; that result must never be treated as a valid signature.

## Native and external results

A computer-use launch attempt timed out awaiting app approval. No PokeVerse window/process or initialized profile databases were observed after that attempt. The user then selected preparation of the acceptance checks. No successful native startup, interactive workflow, installation, upgrade, uninstall, physical LAN, private VPN or trusted signing acceptance is claimed.

The kit never launches an application/installer, configures the firewall, modifies the personal app-data directory or initializes test identities automatically. Native/manual, installation/data preservation, direct physical LAN, optional VPN and distribution gates remain separately recorded as pending in the runbook and backlog.
