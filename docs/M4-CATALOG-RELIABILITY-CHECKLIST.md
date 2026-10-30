# M4: catalog reliability checklist

Started **30 October 2026**, baseline commit `2da1635`, app 0.42.0. **M4 is in progress.** M3's 16 checks are closed; its retained GM rulings do not become automatic catalog corrections.

## Named batches

- [x] **M4-C01: Bleakwind Storm accuracy sign.** Complete MASTER printed 469 / PDF 336 rendered and visually reviewed. The Low Accuracy 2 prose and downward accuracy-2 icon agree: application `AccuracyReduction` is -2. Preserve Lethal, WindMove, Area targeting and three Freeze Chance dice. Supplied JSON stays +2 and unchanged. Correction lives in `rules/battle-reviewed.json`; the chance registry links its resolution.
- [ ] **M4-R01: missing move references.** Baseline: 35 species/form references: 33 to Strength, Gouging Fire's `FIre Blast`, and Mew's `Any Move`. These require distinct decisions: recover the actual Strength definition; verify the Fire Blast spelling with exact species/rank; represent Mew's wildcard without inventing a move card. Preserve exact form identities and learnset ranks. Resolving a link does not certify its learning prerequisites.
- [ ] **M4-S01: creation/evolution fidelity.** Review exact species/form baselines, limits, ordinary abilities, move ranks and evolution branches/prerequisites against the printed pages. There are 17,937 unverified species-field records, including 1,200 Moves and 1,200 Evolutions records. These counts describe missing review provenance, not proven errors. Automated extraction is not visual certification. Keep ambiguous fields visibly unverified.
- [ ] **M4-A01: species/form sprites.** Baseline: 168 missing exact assets, individually named in the registry. Review exact available assets before adding application-owned mappings. Keep named placeholders where an exact asset is unavailable; do not borrow another form silently.

## Source-backed inventory

[Machine-readable baseline and correction evidence](../rules/catalog-reliability-review.json) includes exact IDs/names/fields for all 35 references and 168 asset gaps, original PDF fingerprints and Bleakwind source fingerprint. The field-level queue remains in parent `outputs/rule-coverage/catalog-review.json` and `.csv`.

Baseline totals: 28,359 unverified field records (17,937 species; 9,501 moves; 399 abilities; 472 items; 50 natures), 35 missing references, 168 missing assets, and one adjudication-required record. Bleakwind's Attributes review reduces the unverified count by one; it does not certify other move fields. Habitat assignments and user overrides remain authoritative and are outside this batch.

## Completion gate

- [ ] Resolve or explicitly classify every M4-R01 reference with exact source/form evidence.
- [ ] Finish the named M4-S01 species review, preserving visible unresolved/ambiguous fields.
- [ ] Resolve or individually review all M4-A01 asset gaps.
- [ ] Reconcile generated catalog, fingerprints, Compendium behavior and remaining-work records, then close M4 with actual validation evidence.

## M4-C01 validation

Nine affected Rust regressions passed, including the new check that Bleakwind removes two accuracy successes without removing dice. All 71 frontend tests and 35 script/catalog checks passed; TypeScript, Rust formatting and diff checks passed. The ignored move-route diagnostic was separately rerun and passed, refreshing the 892-card probe inventory. The full Rust suite was not rerun for this catalog-only batch. No native packages, Clippy, physical-network acceptance or source-data edits are included.
