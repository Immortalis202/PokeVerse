# Milestone 7: Windows and LAN acceptance

Prepared 23 October 2026 for app **0.30.0**, engine **0.23.0-core**, protocol **31**, schema **4**. Milestones 1–6 are implemented. **Milestone 7 remains pending manual acceptance.** Automated localhost tests, a portable native run, installed-app tests, direct physical LAN and private VPN are separate results.

The user requested preparation of the acceptance checks. No installer, uninstaller, firewall change or native test session is run by these preparation commands. Use disposable Windows users/VMs for installation tests. The portable profiles never select the normal personal app-data directory. Do not use personal characters, room tokens or credentials as fixtures.

## Prepare the isolated lab

From the repository, with the existing v0.30.0 release build:

```powershell
node scripts/prepare-acceptance.mjs
```

The command creates a new `tmp/acceptance/0.30.0-<unique-id>` directory with five independent copies: `host`, `gm`, `owner1`, `owner2`, `owner3`. Each contains `pokeverse.exe`, `resources/catalog.db` and an empty `portable-data` directory. It verifies both copied files by SHA256, records artifact paths/hashes, executable product version, source commit/worktree state and installer signature in `manifest.json`, and copies the pending case ledger to `results.csv`. Repeating preparation creates a separate lab; it does not overwrite a profile. The manifest identifies prepared files, not successful acceptance.

Open each profile's `pokeverse.exe` manually when ready. The pre-existing `portable-data` directory is essential: do not move the executable away from it. Keep one host and a separately appointed GM; the other three profiles own trainers. All clients must use the matching build. For the five-instance local rehearsal, use `ws://127.0.0.1:39231/room`. This establishes only same-computer connectivity.

Use `passed`, `failed`, `blocked` or `pending` in `results.csv`. Record the exact case, machine/profile, app version, date, expected/actual behavior and an evidence path. Keep screenshots/log excerpts local and redact tokens, addresses and character notes before sharing. Do not check personal test profiles or database files into Git.

## Manual cases and pass criteria

| Case | Action and required result |
| --- | --- |
| A01 | Check the executable and installer versions and hashes against the manifest. Record `Get-AuthenticodeSignature` for the installer. v0.30.0 is currently unsigned; record signing as pending, not trusted distribution. Open a portable profile; verify the workspace loads and the About version is 0.30.0. Close/reopen; identity and saved test data remain. |
| A02 | In a disposable Windows user/VM, install the previous installer, create test trainer/Pokémon, progression history, draft and campaign, then close the app and take data evidence/backups. Upgrade using the new installer without clearing app data. Verify all records, identity, memberships, inventory, partial medicine doses, lockouts, history, drafts and campaign progress survive. Test a fresh install separately. Record installer versions, install directories and actual app-data path. A portable executable replacement does not establish installer upgrade acceptance. |
| A03 | Exercise creation, named progression, move learning/replacement, evolution previews, unused budgets and trainer-point distribution at **960×650**, **1280×850** and **1920×1080**. Record each size. Navigate with Tab/Shift+Tab/Enter/Escape; verify focus, labels, dialogs, error text and long names/content remain reachable without clipped action buttons. |
| A04 | Disconnect external internet while retaining the local connection. Search Compendium species/forms/moves/items, open exact form and evolution links, navigate Back and verify named sprites/placeholders. Restart and repeat. Record offline operation separately from missing-asset/catalog review boundaries. |
| A05 | Connect the five local profiles. Host appoints `gm`; host remains transport authority. Synchronize each trainer before their Pokémon. Test valid owner actions, rejected cross-owner actions, GM-only controls hidden/rejected for owners and host, token errors and matching-build requirements. Do not store the access token in the ledger. |
| A06 | Native gym rehearsal: three owner trainers with two conscious Pokémon each; GM trainer with six. Register **12**, deploy three per side, keep six reserves. Roll initiative and resolve ordinary accuracy/damage. Successfully Clash twice on separate incoming hits in the same turn; reject a second Evasion in that turn; reset on the next turn. Switch a reserve, enforce owner/trainer limits and next-round availability, replace a fainted starter, finish/end battle, reopen history. Follow the scenario assertions in headless-3v3.md; a headless replay does not pass this native case. |
| A07 | In new engine battles, exercise milestone 1 held ownership/suppression/consumption; milestone 2 exact form/Mega/Transform and restoration; milestone 4 GM metal weights/Rivalry/Torrent decisions and Moody/Perish timing; milestone 5 sourced stat/chance and shield/forced-switch interactions. Use the linked batch documents for exact printed boundaries. Verify previews, rejection messages, shared damage inputs, rollback and permanent-sheet protection. Record each exercised definition; do not claim all definitions from a single example. |
| A08 | Milestone 3: test Anticipation/Forewarn warnings with different owner, GM and observer windows. Private comparisons, move choices and GM rulings must reach only eligible recipients, including after reconnect/rollback/history. Milestone 6: Choice selection and GM timing, Ring Target policy, balloon popping, Focus Sash once, medicine partial doses/shared three-HP cap, fainted reserve revival, herbal ruling/Gluttony, Full Restore next-day lockout and Recycle. Undo restores the participating supplies and preserves unrelated inventories. |
| A09 | GM creates a private training-encounter draft; owners/observer cannot see it. Publish, assign mixed capture recipients, manually confirm capture and trainer rewards. Owners import results to their library and spend accepted points offline. Reconnect and confirm rewards/imports do not duplicate. Capture roll/ball automation remains excluded from this case. |
| A10 | Through the native UI, generate a stale preview, duplicate a pending submission and disconnect/reconnect while an operation is outstanding. Verify explicit stale rejection, one receipt/effect per ID and retained retry after restart. Change GM and verify current permissions. Keep a baseline before each destructive/rollback step. |
| A11 | Create conflicting offline progression branches; synchronize and resolve explicitly without losing unrelated accepted operations. Make a backup, alter only disposable test data, restore through the UI while not in a room, restart and confirm identity/history/data. Record expected restore contents; restoring test data intentionally discards later test changes. |
| A12 | **Direct LAN on at least two physical Windows machines**: host on A; a fresh independently created owner profile on B. Use A's private LAN IPv4 and displayed port, not loopback. Join, synchronize, perform an owner command, observe matching GM/owner state, disconnect/reconnect, retry without duplication and reopen persisted state. Record machine labels, Windows versions, transport, roles, port, results and redacted error evidence. Do not copy the host's initialized identity to B. Five local processes or two VMs do not count as this case. |
| A13 | Record wrong token/host/port and unavailable-host errors. Restore valid settings and verify reconnection. Record firewall-denied behavior only in a controlled test environment operated by the user; do not weaken firewall rules or expose the unencrypted v1 transport publicly. Private-VPN joining/reconnect is an additional separately labelled run; it cannot substitute for A12. |
| A14 | In the disposable installed-app environment, close the app, uninstall and verify the chosen data-preservation behavior. Reinstall and verify retained data/identity when preservation was selected. Record paths and before/after evidence. Confirm the uninstaller affects only the intended installation and leaves the source checkout/other profiles alone. Do not perform this on the personal installation as an unattended check. |
| A15 | Distribution gate: record certificate subject/issuer/validity, a valid Authenticode verification, timestamp and trusted Windows installation on a clean machine. Verify the exact delivered installer hash. An unsigned build or passing build command does not pass trusted signing. Windows prompts/reputation must be observed, never bypassed by automation. |

## Read-only preservation evidence

Close the test apps before taking a cross-database snapshot. Substitute the actual lab directory printed by preparation:

```powershell
node scripts/acceptance-data.mjs snapshot "tmp/acceptance/0.30.0-LAB-ID" "tmp/before-upgrade.json"
# Perform the intended upgrade/restart on disposable test profiles; close the apps.
node scripts/acceptance-data.mjs snapshot "tmp/acceptance/0.30.0-LAB-ID" "tmp/after-upgrade.json"
node scripts/acceptance-data.mjs compare "tmp/before-upgrade.json" "tmp/after-upgrade.json"
```

Initialize all five profiles before snapshotting. The tool opens databases read-only and records integrity, schema, table row counts and canonical content hashes; it never exports raw characters, credentials, tokens or player IDs. Missing databases fail without creating files. Evidence outputs cannot be overwritten. Comparison fails for changed/missing profile data; review intentional migration changes individually. Identical hashes do not prove the native workflow or an installer was exercised. These commands cover portable data; installed-profile evidence must be taken from the actual installed app-data directories in the disposable environment, not substituted with portable evidence. Keep app-produced backups of actual installed data for A02/A14.

For the actual installed profile in the disposable environment, use the directory you recorded in A02:

```powershell
node scripts/acceptance-data.mjs snapshot-profile "ACTUAL-INSTALLED-APP-DATA" "tmp/installed-before.json"
# Upgrade/restart and close the installed app.
node scripts/acceptance-data.mjs snapshot-profile "ACTUAL-INSTALLED-APP-DATA" "tmp/installed-after.json"
node scripts/acceptance-data.mjs compare "tmp/installed-before.json" "tmp/installed-after.json"
```

The profile and five-profile lab report kinds cannot be compared interchangeably. Do not point this command at a personal profile when testing disposable installations.


## Automated baseline

Run catalog generation before Rust tests, with all profile apps closed. Catalog generation must not run while tests hold SQLite open.

```powershell
npm run catalog
npm test
cargo test --manifest-path src-tauri/Cargo.toml --lib -- --test-threads=4
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
npx tsc --noEmit
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
```

Rust binding tests regenerate TypeScript files; format changed bindings with Prettier before reviewing the final diff. Rebuild with `npm run package` only when application/catalog changes require a new artifact. The acceptance preparation itself does not alter the application version, protocol, schema or existing installer. Keep automation evidence distinct from the manual case ledger.

## Private-group scope decision — 23 October 2026

The user declined public release/signing work for the private friend group. Case A15 is outside the planned scope, not passed. The installer remains unsigned. Installation, UI and networking results retain their actual pending/passed status; do not infer successful tests from this decision. Future gameplay work follows [the GM-choice roadmap](GM-CHOICE-ROADMAP.md).

## Completion rule

Milestone 7 can be marked complete only when applicable cases have recorded results and evidence, all failures are fixed/retested, and native installation/upgrade/uninstall, interactive workflows, physical LAN and signing are explicitly resolved. A user-approved deferral must stay labelled as a deferral. Until then the kit and automated checks are delivered, and acceptance remains pending.
