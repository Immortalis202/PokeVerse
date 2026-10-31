# M4-S01: species creation and evolution fidelity

Started 30 October 2026 after M4-R01 (`4d37d09`). **In progress: not a completed review of all 1,200 species/forms.** Full source certification must not be inferred from extraction or graph checks.

## Verified work

- [x] Inventory all 1,200 exact catalog identities and scan evolution relations in both directions, including reverse-only links.
- [x] Preserve a fingerprint of the complete supplied species snapshot. Source JSON is unchanged.
- [x] Visually review complete MASTER 134-136 / PDF 1-3. Record BaseHP, five baseline/limit dot rows, both ordinary ability slots and every move-rank icon for 12 identities: Bulbasaur, Ivysaur, Venusaur, Mega Venusaur, Charmander, Charmeleon, Charizard, Mega Charizard Y, Mega Charizard X, Squirtle, Wartortle and Blastoise. These fields agree with the supplied values. Mega forms remain distinct identities.
- [x] Verify ordinary level links and Medium evolution time for Bulbasaur, Ivysaur, Charmander, Charmeleon, Squirtle and Wartortle. Do not certify whole evolution arrays containing temporary-form prerequisites from a species picture alone.
- [x] Correct Floette's pre-evolution spelling to exact `FlabÃƒÂ©bÃƒÂ©` (MASTER 333 / PDF 200), preserving its ID and existing conditions.
- [x] Correct Pawmot's erroneous self-pre-evolution to Pawmo using the printed #921 Ã¢â€ â€™ #922 Ã¢â€ â€™ #923 diagram (MASTER 405 / PDF 272). Preserve the unreviewed speed; do not silently certify the whole evolution record.
- [x] Retain already delivered M4-R01 references and Mew's user rulings when adding manual species fields. Species review fields are applied once, before subsequent targeted corrections.

The 12 new field reviews add 174 field certifications (14 fields per identity plus six ordinary Evolution arrays). Combined with the six previously reviewed identities, the registry now contains 18 identities with some manual evidence. **17,763 species-field records remain unverified.** Six newly reviewed identities have all 15 tracked fields reviewed; the others remain partial. None of this certifies uninspected remaining pages.

## User stage-cost policy

User decisions, 30 October 2026:

1. Standalone PokÃƒÂ©mon use final-stage learning costs, including entries printed as Unknown.
2. Alternate non-Mega/Gigantamax forms share their base final-stage costs.
3. Primal Groudon/Kyogre, Ultra Necrozma and Terapagos battle forms also share those costs.

Determine standalone status from both incoming and outgoing ordinary relations. Same-species form changes are not ordinary evolution. For evolved form families, a known final-stage base and no outgoing ordinary evolution permit final-stage inheritance. Do not guess a final stage for an intermediate family. Mega/Gigantamax rows and the Egg placeholder are excluded from this assignment.

The policy changes stage assignment on 192 rows in the current snapshot. Record `Creation.stage_ruling` as a user decision and retain printed Unknown/other source notes. It does not mark unknown stats, abilities, moves or evolution conditions reviewed, waive source adjudication, or add battle support.

## Confirmed evolution and gender rulings

- [x] Knowing Double Hit (Aipom), Twin Beam (Girafarig), Hyper Drill (Dunsparce), Rage Fist (Primeape), or Psyshield Bash (Stantler) satisfies the named move prerequisite without separate mastery adjudication.
- [x] Applin â†’ Dipplin consumes one owned Syrupy Apple. Dipplin â†’ Hydrapple requires knowing Dragon Cheer, without consuming an apple. Dragon Cheer remains available through the existing TM/tutor path; no ordinary learnset rank is added. Dipplin uses second-stage learning costs (4 TP before any below-rank discount).
- [x] Kirlia â†’ Gallade requires Male and consumes one owned Dawn Stone. Kirlia â†’ Gardevoir uses Medium evolution (30 TP), for Male or Female, without a stone. These are user rulings resolving ambiguous printed/source data, not new printed certifications (MASTER 221 / PDF 88 and MASTER 276 / PDF 143).
- [x] PokÃ©mon sheets expose Male, Female, Genderless and Unspecified in normal identity editing. Existing sheets default to Unspecified; trainers have no PokÃ©mon gender field. Concrete gender prerequisites are enforced at preparation and completion, including after a normal identity edit. Evolution scene confirmation remains unchanged.

The latest Applin/Dipplin ruling supersedes the earlier Syrupy Apple answer for Dipplin â†’ Hydrapple. Application-owned overrides validate both original directions before replacing them. Supplied source files remain unchanged. Schema compatibility uses release 0.43.0 / protocol 44; omitted Unspecified preserves legacy serialized sheets and progression fingerprints.

## Evolution discrepancies requiring further review

The complete bidirectional pass retains **9** discrepancies after the two confirmed identity repairs:

| Source Ã¢â€ â€™ target | Conflicting or incomplete evidence | Current disposition |
| --- | --- | --- |
| Aipom Ã¢â€ â€™ Ambipom | Mastering/learning Double Hit versus Medium level evolution | Asked user how mastery is satisfied; pending |
| Dipplin Ã¢â€ â€™ Hydrapple | Syrupy Apple versus Syrupy Seed; printed Dipplin Final versus later branch | Asked user for prerequisite and stage-cost choice; pending |
| Girafarig Ã¢â€ â€™ Farigiraf | Learning versus mastering Twin Beam | Further source review/ruling required |
| Kirlia Ã¢â€ â€™ Gallade | Reverse relation adds Male | Further source review/ruling required |
| Goomy Ã¢â€ â€™ Hisuian Sliggoo | Medium speed absent in reverse relation | Further source review required |
| Scyther Ã¢â€ â€™ Kleavor | Trade versus Item, both Black Augurite | Further source review/ruling required |
| Rockruff Ã¢â€ â€™ Dusk Lycanroc | Dawn versus Dusk | Further source review required |
| Mantyke Ã¢â€ â€™ Mantine | Bit versus hit by Remoraid | Further source review/ruling required |
| Galarian Slowpoke Ã¢â€ â€™ Galarian Slowking | King's Rock trade versus Shellder hat | Further source review/ruling required |
| Sneasel Ã¢â€ â€™ Weavile | Moonlight condition absent in reverse relation | Further source review/ruling required |
| Duraludon Ã¢â€ â€™ Archaludon | Reverse relation exists without forward relation | Further source review required |
| Type: Null Ã¢â€ â€™ Silvally | Reverse relation exists without forward relation | Further source review required |
| Ursaring Ã¢â€ â€™ Kitakami Ursaluna | Reverse relation exists without matching forward branch | Further source review/ruling required |

User explicitly chose to keep current forward rules for unresolved preparations. No blanket GM gate is added. Existing unresolved source conditions are retained. The diagnostic report does not claim these are all printed-source errors or all possible semantic issues.

## Evidence and remaining work

[Source snapshot, user policy and per-identity manual references](../rules/species-fidelity-review.json), [application-owned manual fields](../rules/catalog-reviewed.json), and generated catalog metadata `species_fidelity` preserve the evidence. The ordinary field queue remains in parent `outputs/rule-coverage/catalog-review.json`.

- [x] Implement the answered move, apple, Kirlia and gender choices.
- [ ] Verify and resolve the nine remaining conflicting evolution conditions.
- [ ] Continue exact-identity visual review of the remaining species/forms, recording each reviewed field and its page.
- [ ] Review evolution branches, items, gender, region and move/narrative prerequisites before certifying their arrays.
- [ ] Reconcile all source and runtime evidence before marking M4-S01 complete.

## Validation of this checkpoint

Release 0.43.0 / protocol 44: **791 Rust tests passed, 1 diagnostic ignored**; that diagnostic was separately rerun and passed for all 892 source cards. The final gender/progression reconciliation adjustment passed its focused regression afterward. **71 frontend tests, 40 script/catalog checks, TypeScript, Rust formatting and diff checks passed.** No native packages, Clippy, native UI or physical-network acceptance are included.
