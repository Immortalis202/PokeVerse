# M4: catalog reliability checklist

Started **30 October 2026**, baseline commit `2da1635`, app 0.42.0. **M4 is complete as of 31 October 2026 for its named catalog-reliability scope.** M3's 16 checks are closed; its retained GM rulings do not become automatic catalog corrections.

## Named batches

- [x] **M4-C01: Bleakwind Storm accuracy sign.** Complete MASTER printed 469 / PDF 336 rendered and visually reviewed. The Low Accuracy 2 prose and downward accuracy-2 icon agree: application `AccuracyReduction` is -2. Preserve Lethal, WindMove, Area targeting and three Freeze Chance dice. Supplied JSON stays +2 and unchanged. Correction lives in `rules/battle-reviewed.json`; the chance registry links its resolution.
- [x] **M4-R01: missing move references.** All 35 baseline references resolved; current missing-reference warnings: **0**. The 33 Strength references were caused by a trailing space in the move name; correct only the application name and preserve `strength_:Strength ` (including its trailing space). Keep the original spelling as an explicit lookup alias for saved sheets, and prevent duplicate learning through the alias. Gouging Fire remains #1020 by user confirmation; correct only `FIre Blast` to `Fire Blast` at Expert (MASTER 430 / PDF 297 prints a conflicting #1021). Mew's `Any Move` at Ace (MASTER 184 / PDF 51) expands into actual catalog choices. User rulings: any catalog move at Ace or higher, Mew has no evolution and is final stage, normal final-stage learning costs, searchable starting-move selection. Printed Mew stage Unknown remains in provenance. No fake move card or blanket learning-prerequisite certification. [Exact correction/source registry](../rules/catalog-reference-review.json).
- [x] **M4-S01: creation/evolution JSON fidelity.** Complete for all 1,200 exact identities against species/form JSON plus retained manual evidence and explicit user rulings. All 13,200 stat fields are accounted for (nine user deviations), 522 ordinary evolution branches and 112 form links are classified, all stages resolve, all learnsets reconcile, and all 2,400 ordinary ability slots reconcile. Zero unexplained JSON issues remain. One evolution guidance discrepancy and four GM-set costs are retained decisions. [S01 closure, evidence and scope](M4-S01-SPECIES-FIDELITY-CHECKLIST.md), [machine-readable closure](../rules/species-s01-closure.json). Optional PDF certification remains partial; its 17,560 unverified species-field records do not block the JSON scope. This does not certify hidden/event abilities or new battle support.
- [x] **M4-A01: species/form sprites.** All 168 baseline gaps resolved with exact reference-linked images from Serebii.net: 167 official game renders and one official Rotom Dex artwork. Images are bundled locally with exact IDs, original image names, source-page/asset URLs, source-page and PNG hashes, dimensions and individual identity notes. All 168 decoded PNGs were visually inspected. User-approved defaults: Family of Four Maushold, Two-Segment Dudunsparce, Green Plumage Squawkabilly and Ash-Greninja for Battle Bond. [Asset review and validation](M4-A01-SPECIES-ASSETS-CHECKLIST.md).

## Source-backed inventory

[Machine-readable baseline and correction evidence](../rules/catalog-reliability-review.json) includes exact baseline IDs/names/fields for all 35 resolved references and 168 asset gaps, plus the M4-R01 resolution decisions, original PDF fingerprints and Bleakwind source fingerprint. The field-level queue remains in parent `outputs/rule-coverage/catalog-review.json` and `.csv`.

Baseline totals: 28,359 unverified field records (17,937 species; 9,501 moves; 399 abilities; 472 items; 50 natures), 35 missing references, 168 missing assets, and one adjudication-required record. Bleakwind's Attributes review reduces the unverified count by one; it does not certify other move fields. Habitat assignments and user overrides remain authoritative and are outside this batch.

## Completion gate

- [x] Resolve or explicitly classify every M4-R01 reference with exact source/form evidence.
- [x] Finish the named M4-S01 JSON reconciliation, preserving explicit rulings, GM-set costs and optional printed-certification metadata.
- [x] Resolve and individually review all 168 M4-A01 asset gaps; generated missing-asset count is zero.
- [x] Reconcile generated catalog, fingerprints, automated Compendium behavior and remaining-work records. Native acceptance remains M5.

## M4-C01 validation

Nine affected Rust regressions passed, including the new check that Bleakwind removes two accuracy successes without removing dice. All 71 frontend tests and 35 script/catalog checks passed; TypeScript, Rust formatting and diff checks passed. The ignored move-route diagnostic was separately rerun and passed, refreshing the 892-card probe inventory. The full Rust suite was not rerun for this catalog-only batch. No native packages, Clippy, physical-network acceptance or source-data edits are included.


## M4-R01 validation

Validation: **785 Rust tests passed, 1 diagnostic ignored**. The ignored diagnostic was separately rerun and passed for all 892 source cards. **71 frontend tests, 36 script/catalog checks and TypeScript passed**, along with Rust formatting and diff checks. Coverage includes Ace/Master pricing, below-Ace rejection, capacity, replay verification, stable IDs, old Strength spelling and duplicate-name rejection. Native UI and physical-network acceptance were not run. Source JSON files remain unchanged. Mew's wildcard changes learning availability, not battle-engine support; Max/Z and other unsupported move effects keep their existing guards. Saved Strength spellings remain readable, and new/legacy names cannot be learned as two separate moves. Other move names are not trimmed or fuzzy-matched.

## Final closure, 31 October 2026

[Machine-readable final audit](../rules/catalog-reliability-closure.json) records the current catalog/source/SQLite fingerprints, source-PDF hashes, all five entry counts and retained rulings. The catalog contains 1,200 species/forms, 892 moves, 305 abilities, 236 items and 25 natures. Integrity checks pass; missing references and assets are both zero. All 1,200 species JSON hashes and both original PDF fingerprints match. All 168 application-owned asset hashes match their source files and shipped sprite copies; all 1,200 species have offline sprite files.

The automated Compendium sweep opens every exact species identity, retains every move/rank reference and resolves all 30,064 references to the exact destination identity/name. Existing regression checks cover direct evolution branches, regional forms, Mew's wildcard expansion, historical Strength naming, unresolved/ambiguous-reference behavior, rank grouping and retained metadata warnings. These are automated reference/navigation checks, not native UI acceptance.

All four named batches are complete. S01's species review is exact JSON reconciliation with recorded manual/user decisions; optional printed certification stays partial. Nine stat deviations, one Trade/Item guidance discrepancy and four GM-set evolution costs remain classified, not unexplained errors. No source JSON, saved sheets or historical balances are rewritten by this closure. M5 remains open for actual native, packaged-app, physical LAN and private-VPN acceptance.

Refresh: generate the catalog, refresh the five S01 reports and `node scripts/species-s01-closure.mjs`, then run `node scripts/m4-closure.mjs` and the relevant checks. Close catalog readers before regenerating SQLite on Windows.

Final validation: **802 Rust tests passed, zero failed, one diagnostic ignored**, with localhost access; **74 frontend tests and 51 script/catalog checks passed**. The updated closure assertions and focused M4/Compendium/asset checks also passed. TypeScript, relevant Prettier and Rust formatting passed. Regenerating after catalog readers closed produced identical catalog hash `25d8ac5c0b6fc59c96fc77f33bbbe082fdd81d8bc1fe043d515e1a9f18b4595d` and SQLite SHA-256 `d1a9c47e1a76a3814fe8a1fbcd1cbe01ecb06abfe32d935dd276ecc051ef9b69`. The first concurrent repeat was blocked by Windows file locking; the sequential retry passed. Source repository tracked files remain unchanged. No native packages, Clippy, native UI, installer or physical-network acceptance were run.
