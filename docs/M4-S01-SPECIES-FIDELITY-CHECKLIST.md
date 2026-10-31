# M4-S01: species creation and evolution fidelity

Started 30 October 2026 after M4-R01 (`4d37d09`). **In progress: not a completed review of all 1,200 species/forms.** Full source certification must not be inferred from extraction or graph checks.

## Verified work

- [x] Inventory all 1,200 exact catalog identities and scan evolution relations in both directions, including reverse-only links.
- [x] Preserve a fingerprint of the complete supplied species snapshot. Source JSON is unchanged.
- [x] Visually review complete MASTER 134-136 / PDF 1-3. Record BaseHP, five baseline/limit dot rows, both ordinary ability slots and every move-rank icon for 12 identities: Bulbasaur, Ivysaur, Venusaur, Mega Venusaur, Charmander, Charmeleon, Charizard, Mega Charizard Y, Mega Charizard X, Squirtle, Wartortle and Blastoise. These fields agree with the supplied values. Mega forms remain distinct identities.
- [x] Verify ordinary level links and Medium evolution time for Bulbasaur, Ivysaur, Charmander, Charmeleon, Squirtle and Wartortle. Do not certify whole evolution arrays containing temporary-form prerequisites from a species picture alone.
- [x] Correct Floette's pre-evolution spelling to exact `Flabébé` (MASTER 333 / PDF 200), preserving its ID and existing conditions.
- [x] Correct Pawmot's erroneous self-pre-evolution to Pawmo using the printed #921 → #922 → #923 diagram (MASTER 405 / PDF 272). Preserve the unreviewed speed; do not silently certify the whole evolution record.
- [x] Retain already delivered M4-R01 references and Mew's user rulings when adding manual species fields. Species review fields are applied once, before subsequent targeted corrections.

The initial 12 identities added 174 field certifications. Four later cards were visually reviewed on 31 October: Type: Null (MASTER 362 / PDF 229), Silvally (363 / 230), Duraludon (392 / 259), and Archaludon (430 / 297), adding 56 certifications. Their whole evolution arrays remain unverified; the specific repairs below carry separate evidence.

The page-by-page review then continued through MASTER 137–139 / PDF 4–6: Mega Blastoise, Caterpie, Metapod, Butterfree, Weedle, Kakuna, Beedrill, Mega Beedrill, Pidgey, Pidgeotto, Pidgeot and Mega Pidgeot. Base HP, all five baseline/limit pairs, ordinary ability slots and every move-rank icon were inspected. This adds **149 previously unverified fields**, including complete ordinary evolution arrays for Caterpie, Metapod, Butterfree, Weedle, Kakuna, Pidgey and Pidgeotto. Fast speeds for both insect lines and Medium speeds for the Pidgey line were checked on the relevant First/Second cards. Mega prerequisites remain unverified.

Two findings need explicit provenance:

- Metapod's earlier application-owned manual review incorrectly set MaxVitality to 5. The enlarged printed dot row and unchanged supplied file both show **4**. Correct the manual override to 4; retain Vitality baseline 2. The regression checks that an Expert Metapod can train to 4 and cannot train to 5 (Champion's ordinary +2 cap bonus remains separate).
- Mega Blastoise prints an Advanced Protect icon; ordinary Blastoise prints Standard. The user explicitly chose **Standard** for Mega Blastoise. Record this as a retained move-rank discrepancy and user ruling, not agreement with the printed Mega icon. Other inspected values on these three pages match the supplied data.

The S01 registry now records **28 identities / 379 new field certifications**; combined with earlier reviews, 32 identities have some manual evidence. **17,558 species-field records remain unverified.** The contiguous first six MASTER PDF pages have been visually inspected, but temporary-form prerequisites and all uninspected remaining pages are not blanket-certified.

## User stage-cost policy

User decisions, 30 October 2026:

1. Standalone Pokémon use final-stage learning costs, including entries printed as Unknown.
2. Alternate non-Mega/Gigantamax forms share their base final-stage costs.
3. Primal Groudon/Kyogre, Ultra Necrozma and Terapagos battle forms also share those costs.

Determine standalone status from both incoming and outgoing ordinary relations. Same-species form changes are not ordinary evolution. For evolved form families, a known final-stage base and no outgoing ordinary evolution permit final-stage inheritance. Do not guess a final stage for an intermediate family. Mega/Gigantamax rows and the Egg placeholder are excluded from this assignment.

The policy changes stage assignment on 192 rows in the current snapshot. Record `Creation.stage_ruling` as a user decision and retain printed Unknown/other source notes. It does not mark unknown stats, abilities, moves or evolution conditions reviewed, waive source adjudication, or add battle support.

## Confirmed evolution and gender rulings

- [x] Knowing Double Hit (Aipom), Twin Beam (Girafarig), Hyper Drill (Dunsparce), Rage Fist (Primeape), or Psyshield Bash (Stantler) satisfies the named move prerequisite without separate mastery adjudication.
- [x] Applin → Dipplin lists Syrupy Apple as guidance. Dipplin → Hydrapple requires knowing Dragon Cheer, without consuming an apple. Dragon Cheer remains available through the existing TM/tutor path; no ordinary learnset rank is added. Dipplin uses second-stage learning costs (4 TP before any below-rank discount).
- [x] Kirlia → Gallade enforces Male and lists Dawn Stone as guidance. Kirlia → Gardevoir uses Medium evolution (30 TP), for Male or Female, without a stone. These are user rulings resolving ambiguous printed/source data, not new printed certifications (MASTER 221 / PDF 88 and MASTER 276 / PDF 143).
- [x] Pokémon sheets expose Male, Female, Genderless and Unspecified in normal identity editing. Existing sheets default to Unspecified; trainers have no Pokémon gender field. Concrete gender and known-move prerequisites are enforced at preparation and completion, including after a normal identity edit. Evolution scene confirmation remains unchanged.

The latest Applin/Dipplin ruling supersedes the earlier Syrupy Apple answer for Dipplin → Hydrapple. Application-owned overrides validate both original directions before replacing them. A later general ruling supersedes item ownership/consumption: gender and known moves stay enforced, while items, stones, trades, Happiness/Loyalty, region and narrative conditions are guidance for the player and GM. Items are never consumed by evolution. Normal level TP costs and GM confirmation of completion remain enforced. Supplied source files remain unchanged. Schema compatibility uses release 0.43.0 / protocol 44; omitted Unspecified preserves legacy serialized sheets and progression fingerprints.

## Evolution graph review checkpoint

- [x] Account for all nine previously open graph discrepancies. Six targeted printed corrections, one user cost ruling, one user exclusion and one retained guidance discrepancy close this diagnostic queue. This is not certification of every evolution in the catalog.

| Source → target | Disposition and evidence |
| --- | --- |
| Type: Null → Silvally | Add missing forward Level / Slow link. Printed First / Slow and both #772 → #773 diagrams agree with existing reverse relation. MASTER 362–363 / PDF 229–230. Cost: 50 TP. |
| Duraludon → Archaludon | Add missing forward Level / Slow link. Printed First / Slow and both #884 → #1018 diagrams agree with existing reverse relation. MASTER 392 and 430 / PDF 259 and 297. Cost: 50 TP. |
| Goomy → Hisuian Sliggoo | Restore omitted reverse Medium speed; preserve existing Hisui guidance. Printed Goomy is Medium and the regional line is shown. MASTER 342 / PDF 209. |
| Rockruff → Dusk Lycanroc | Correct forward Dawn to Dusk using the explicit dusk narrative and training condition. Preserve Slow TP and the Day/Night branches. MASTER 355 / PDF 222. |
| Mantyke → Mantine | Correct reverse Hit to Bit by Remoraid using the explicit Mantyke condition. Guidance only. MASTER 272 / PDF 139; Mantine diagram at MASTER 205 / PDF 72. |
| Galarian Slowpoke → Galarian Slowking | Use existing reverse Wear Shellder as a Hat guidance in both directions. The printed regional Slowking says this; King's Rock is printed for ordinary Slowking. Preserve the other Galarian branch. MASTER 160 and 197 / PDF 27 and 64. |
| Sneasel → Weavile | User confirmed item evolution at 0 TP. Razor Claw at night is printed guidance, without consumption or a missing-speed cost approval. MASTER 202 / PDF 69; Weavile diagram at MASTER 273 / PDF 140. |
| Ursaring → Kitakami Ursaluna | User confirmed Bloodmoon/Kitakami is standalone in the Ursaluna family. Remove the erroneous reverse relation; keep its exact identity and final-stage costs. Do not offer this branch from Ursaring. Printed family diagram remains source evidence, not an ordinary evolution assignment. MASTER 398 / PDF 265. |
| Scyther → Kleavor | Retain the supplied Trade/Item difference as a reviewed guidance discrepancy. General Scyther text and specific Kleavor text differ. The user assigns eligibility to player and GM, with no enforced trade or item consumption. MASTER 174 and 398 / PDF 41 and 265. |

Generated metadata retains the Scyther discrepancy with `review.status = retained-guidance`; **zero unreviewed graph issues remain in this snapshot**. Exact original conditions are guarded before each targeted repair, exclusion or retained-discrepancy disposition. Supplied source files remain unchanged. Unreviewed source fields stay visibly unverified, and normal evolution completion still requires GM confirmation.

## Evidence and remaining work

### JSON stat comparison, 31 October 2026

At the user's request, further HP/baseline/limit review switches from page-by-page PDF inspection to comparison with the updated upstream JSON repository. `git pull --ff-only` confirmed Pokerole-Data was already current at `376358078498717863c311d7a8fe323fd4fe7375`, with no tracked source changes. This is a JSON consistency review; it does not extend printed-source certification or reduce the unverified printed-field count above.

Run `npm run catalog` then `node scripts/species-stat-json-review.mjs` to refresh the [exact per-field report and 1,200 source-file fingerprints](../rules/species-stat-json-review.json). The report compares effective runtime values after all overrides, creation baselines/limits and database stat rows against BaseHP and all five baseline/maximum pairs in upstream JSON.

- 1,200 exact species/form identities and 13,200 fields compared; 13,068 fields agree.
- 132 differing fields across 33 identities remain pending reconciliation. Of these, 130 originate in extraction-based corrections, while Marill's Insight (JSON 2, runtime 1) and MaxInsight (JSON 4, runtime 3) originate in an existing manual review.
- No missing/extra or mismatched identities, invalid numeric ranges, baseline-above-maximum values, creation metadata mismatches or database stat-row mismatches were found.
- Differences are recorded without changing runtime stats or replacing existing manual evidence. JSON agreement alone does not prove printed fidelity. The Egg placeholder is included in the exact 1,200-row comparison.

Affected identities: Aegislash, Binacle, Braviary, Brute Bonnet, Calyrex, Cranidos, Darmanitan, Deoxys, Dialga, Dusknoir, Eiscue, Entei, Eternatus, Flareon, Florges, Gumshoos, Hydrapple, Keldeo, Kricketune, Kyurem, Lampent, Liepard, Lillipup, Lopunny, Marill, Palkia, Shaymin, Stoutland, Swoobat, Togekiss, Trubbish, Wishiwashi and Zacian. The report records each exact application ID and both values.

The earlier PDF checkpoint remains historical evidence. Next stat work should reconcile this 132-field JSON queue rather than resume sequential PDF stat review.

### Exact alternate-form reconciliation

The comparison now tests every affected identity against every other JSON form with the same National Dex number. A form explanation requires all eleven fields to match, not just the differing field. Exact candidate IDs/files and matching/differing fields are retained in `form_reconciliation` in the report.

**109 differences across 14 identities match a complete alternate-form stat vector:**

| Current identity | Exact matching JSON form |
| --- | --- |
| Aegislash (base/Shield) | Aegislash (Blade Form) |
| Braviary | Braviary (Hisuian Form) |
| Calyrex | Calyrex (Ice Rider Form) |
| Darmanitan | Darmanitan (Zen Form) |
| Deoxys | Deoxys (Speed Form) |
| Dialga | Dialga (Origin Form) |
| Eiscue | Eiscue (No Ice Form) |
| Eternatus | Eternatus (Eternamax Form) |
| Keldeo | Keldeo (Resolute Form) |
| Kyurem | Kyurem (White Form) |
| Palkia | Palkia (Origin Form) |
| Shaymin | Shaymin (Sky Form) |
| Wishiwashi | Wishiwashi (Swarm Form) |
| Zacian | Zacian (Crown Form) |

These values are valid matches for the named alternate forms; their application to the base identity remains a form-assignment conflict. User confirmation explicitly establishes Aegislash Shield Strength 2 / maximum 4 and Sword (Blade) Strength 4 / maximum 8. No runtime form assignment or stats have been changed by this audit.

**23 differences across 19 identities remain unexplained by alternate-form JSON:** Binacle, Brute Bonnet, Cranidos, Dusknoir, Entei, Flareon, Florges, Gumshoos, Hydrapple, Kricketune, Lampent, Liepard, Lillipup, Lopunny, Marill, Stoutland, Swoobat, Togekiss and Trubbish. Lopunny has a Mega candidate, but its complete stat vector does not match; the other 18 have no alternate-form JSON entry with the same National Dex number. These are unresolved differences, not proven source errors.

Validation: catalog regeneration and the JSON comparison completed successfully; `npm test` passed 71 frontend tests and 45 script/catalog checks. The form-reconciliation rerun also passed these checks after setting process-local TEMP/TMP to `tmp/vitest` to avoid missing sandbox temporary transform files on the initial attempt. Prettier and diff checks passed. No runtime rules were changed; Rust tests and native builds were not run for this audit-only batch.

[Source snapshot, user policy and per-identity manual references](../rules/species-fidelity-review.json), [application-owned manual fields](../rules/catalog-reviewed.json), and generated catalog metadata `species_fidelity` preserve the evidence. The ordinary field queue remains in parent `outputs/rule-coverage/catalog-review.json`.

- [x] Implement the answered move, apple, Kirlia and gender choices.
- [x] Review and dispose of the nine diagnosed graph discrepancies, retaining the documented Scyther guidance conflict.
- [ ] Continue exact-identity visual review of the remaining species/forms, recording each reviewed field and its page.
- [ ] Review evolution branches, items, gender, region and move/narrative prerequisites before certifying their arrays.
- [ ] Reconcile all source and runtime evidence before marking M4-S01 complete.

## Validation of this checkpoint

Prior gender checkpoint, release 0.43.0 / protocol 44: **791 Rust tests passed, 1 diagnostic ignored**; that diagnostic was separately rerun and passed for all 892 source cards. The final gender/progression reconciliation adjustment passed its focused regression afterward. **71 frontend tests, 40 script/catalog checks, TypeScript, Rust formatting and diff checks passed.** No native packages, Clippy, native UI or physical-network acceptance are included.

Prior special-eligibility guidance batch: **792 Rust tests passed, 1 diagnostic ignored; 71 frontend tests and 40 script/catalog checks passed; TypeScript passed.** Final move-forgetting and readable-guidance adjustments passed the focused six-test M4-S01 suite. Formatting and diff checks passed. No native packages or native/physical-network acceptance were run.

31 October graph/source batch: **793 Rust tests passed, 1 diagnostic ignored; 71 frontend tests and 41 script/catalog checks passed; TypeScript, Rust formatting and diff checks passed.** No native package, Clippy, native UI or physical-network acceptance was run.

31 October Kanto pages 137–139 batch: **794 Rust tests passed, 1 diagnostic ignored; 71 frontend tests and 42 script/catalog checks passed; TypeScript, Rust formatting, relevant Prettier checks and diff checks passed.** No native package, Clippy, native UI or physical-network acceptance was run.
