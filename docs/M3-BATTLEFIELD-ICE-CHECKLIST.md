# M3: battlefield and ice review checklist

Source/code reconciliation: **29 October 2026**, app **0.41.0**, engine **0.34.0-core**, baseline commit `bf148ef`. This deliverable changes audit records and documentation only. It does not implement new rules, adopt a house policy, bump the engine or schedule an implementation batch.

## Deliverable and scheduling gate

- [x] Name the checks, inspect current paths and identify existing regression evidence.
- [x] Compare the relevant printed rules and exact move/ability JSON; record source hashes.
- [x] Reconcile stale coverage/backlog claims without claiming unrun edge cases pass.
- [ ] Resolve M3-I03, M3-I04, M3-I05 and M3-T01 before selecting implementations that depend on those interpretations.
- [ ] Reproduce M3-F02 and replace its conflicting regression expectation before implementing the explicit Magic Guard exception.
- [ ] Reproduce M3-W01 across setup, casting, ability entry and live night changes; then select a narrowly defined fix.
- [ ] Add focused regressions for reviewed gaps and close each checklist item with implementation or an explicit retained ruling.

M3 is **not complete**. There are **16 named checks**: six existing paths (one explicitly lacking focused Snow regressions), four source/formula reviews, two source/code gaps, two parameterized examples and two intentional GM decisions. Existing mechanics are not automatically new development tasks.

## Sources and evidence limits

- [RULING PDF](<../../Pokerole-Data/POKEROLE COREBOOK 3.0 RULING.pdf>): printed and PDF pages agree here. Text checked on 78, 85-86 and 88-91; pages **85, 89 and 91 rendered and visually inspected**. Page 91 icons matter: Healing Pits explicitly shows **Heal 1**.
- Exact move/ability JSON effects and SHA256 values, code symbols and existing fixture names are preserved in [the machine-readable review](../rules/battlefield-ice-review.json). Original source files are unchanged.
- This is a bounded source/code review, not an exhaustive verification of every battlefield combination or every catalog move. A named existing fixture demonstrates only its assertions. There is no dedicated Snow ice threshold/category fixture in the baseline inspected here.

- [MASTER 554 / PDF 421](<../../Pokerole-Data/POKEROLE COREBOOK 3.0 POKEDEX MASTER.pdf#page=421>) visually checked for the explicit Magic Guard / Perish Song exception (M3-F02).

## Named checklist

| ID | Check | Classification | Source |
| --- | --- | --- | --- |
| [M3-I01](#m3-i01) | Ordinary ice-breaking baseline | Existing automation | RULING 85,86 |
| [M3-I02](#m3-i02) | Snow ice HP and physical Defense | Existing; focused tests needed | RULING 89 |
| [M3-I03](#m3-i03) | Snow ice Special Defense wording | Source/interpretation review | RULING 85,89 |
| [M3-I04](#m3-i04) | Fixed and variable damage against ice: five named cards | Source/interpretation review | RULING 85,86; Sonic Boom JSON; Dragon Rage JSON; Seismic Toss JSON; Night Shade JSON; Super Fang JSON |
| [M3-I05](#m3-i05) | Snow changes or suppression while ice is damaged | Source/interpretation review | RULING 85,89; Air Lock JSON; Cloud Nine JSON |
| [M3-W01](#m3-w01) | Sunny Weather at night | Source/code gap | RULING 88 |
| [M3-W02](#m3-w02) | Weather priority, ability Will contests and expiry | Existing automation | RULING 88,89,90; Drought JSON; Drizzle JSON |
| [M3-W03](#m3-w03) | Air Lock and Cloud Nine suppression | Existing automation | Air Lock JSON; Cloud Nine JSON; RULING 89,90 |
| [M3-T01](#m3-t01) | Mixed terrain precedence and scope | Source/interpretation review | Grassy Terrain JSON; Electric Terrain JSON; Misty Terrain JSON; Psychic Terrain JSON; Grassy Surge JSON; Electric Surge JSON; Misty Surge JSON; Psychic Surge JSON; Seed Sower JSON; Levitate JSON |
| [M3-F01](#m3-f01) | Force fields, bypass and cleanup | Existing automation | RULING 78 |
| [M3-F02](#m3-f02) | Magic Guard explicit Perish Song exception | Source/code gap | MASTER 554; Magic Guard JSON |
| [M3-A01](#m3-a01) | Healing Pits printed Heal 1 and configurable variants | Existing, configurable | RULING 91 |
| [M3-A02](#m3-a02) | High Poles difficulty | Existing, configurable | RULING 91 |
| [M3-A03](#m3-a03) | Torn World target distribution | Intentional GM decision | RULING 91 |
| [M3-A04](#m3-a04) | Cemetery / Run Away narrative escape | Intentional GM decision | RULING 91; Run Away JSON |
| [M3-A05](#m3-a05) | Arena exposure clocks and round-end ordering | Existing automation | RULING 90,91 |

## Findings and required next checks

<a id="m3-i01"></a>

### M3-I01: Ordinary ice-breaking baseline

**Existing automation.** End-of-round self/ally attempts, remaining actions, 5 HP, both defenses 2, successful super-effective break and no Pokemon HP loss are implemented. Physical and ordinary Special categories are accepted.

**Next:** Retain current behavior. Extend dedicated tests only for an identified uncovered branch; do not reimplement ordinary Special attacks.

**Code:** `src-tauri/src/combat.rs::ice_attack`, `src-tauri/src/combat.rs::ice_damage_pool`.

**Existing evidence:** `frozen_subject_can_break_ice_at_round_end_without_losing_its_hp`, `special_charge_defense_is_temporary_and_matcha_can_break_its_own_ice`. These names are references, not claims that every edge listed above is already tested.

<a id="m3-i02"></a>

### M3-I02: Snow ice HP and physical Defense

**Existing; focused tests needed.** Engine already uses 7 HP and subtracts defense 3 in Snow. The registry claiming all Snow-versus-ice behavior is manual was stale.

**Next:** Reconcile scope now; later add focused Snow/no-Snow pool and HP-threshold regressions without scheduling a replacement implementation.

**Code:** `src-tauri/src/combat.rs::ice_damage_pool`, `src-tauri/src/combat.rs::Ice block damage`.

**Existing evidence:** `field_all_weather_bonuses_and_move_failures`. These names are references, not claims that every edge listed above is already tested.

<a id="m3-i03"></a>

### M3-I03: Snow ice Special Defense wording

**Source/interpretation review.** Page 85 names Defense and Special Defense separately, both 2. Page 89 raises HP and Defense to 7 and 3 but does not explicitly name Special Defense. Current shared ice pool subtracts 3 for either attack category in Snow.

**Next:** Resolve the narrower printed wording before approving any Special Defense correction; add physical-versus-Special acceptance cases once the interpretation is recorded. Current code is not evidence that Snow Special Defense 3 is official.

**Code:** `src-tauri/src/combat.rs::ice_damage_pool`.

<a id="m3-i04"></a>

### M3-I04: Fixed and variable damage against ice: five named cards

**Source/interpretation review.** Ice guard rejects Power 0, SetDamage and Varies, including these otherwise supported damaging moves. Seismic Toss is Fight-type and the printed ice rule grants immediate breaking to super-effective moves. Ordinary Special-category attacks are not this gap.

**Next:** Review each named formula against the ice object. Separate super-effective instant breaking from neutral fixed/rank/remaining-HP damage; establish object HP, effect and cost handling before selecting an implementation batch. Do not assume every conditional move works on ice.

**Code:** `src-tauri/src/combat.rs::ice_attack`, `src-tauri/src/combat/special.rs::resolve_formula`.

<a id="m3-i05"></a>

### M3-I05: Snow changes or suppression while ice is damaged

**Source/interpretation review.** Ice damage progress is stored on the status, while HP threshold and defense are read from live target-effective weather. Source does not explicitly state whether an existing block snapshots Snow or changes with it.

**Next:** Retain explicit uncertainty. Record timing semantics for Snow starting/ending and suppression; then test partially damaged blocks. Do not silently import video-game rules.

**Code:** `src-tauri/src/combat.rs::ice_damage_pool`, `src-tauri/src/combat.rs::Ice block damage`, `src-tauri/src/combat/field.rs::weather`.

<a id="m3-w01"></a>

### M3-W01: Sunny Weather at night

**Source/code gap.** Printed Sunny Weather cannot be active at night. Setup validation, move guard and weather activation test sunlight reach/indoors but do not read the independent night flag; Night command only changes that flag.

**Next:** Before a future fix, reproduce night + reachable sunlight across setup, Sunny Day and Drought. Define handling for changing to night with existing Sunny and preserve historical encounter rules. This audit does not implement the missing night validation.

**Code:** `src-tauri/src/combat/field.rs::validate_setup`, `src-tauri/src/combat/field.rs::guard_move`, `src-tauri/src/combat/field.rs::set_weather`, `src-tauri/src/combat/field.rs::FieldCommand::Night`.

**Existing evidence:** `environment_rejects_invalid_setup_unauthorized_starts_and_historical_engines_atomically`, `environment_moonlight_uses_live_night_with_explicit_overrides_and_historical_fallback`. These names are references, not claims that every edge listed above is already tested.

<a id="m3-w02"></a>

### M3-W02: Weather priority, ability Will contests and expiry

**Existing automation.** Move replacement, ability Will contests with tied rerolls, departure, natural fallback and extreme-weather guards exist. These are not a wholly unimplemented precedence system.

**Next:** Review move-versus-ability replacement/re-entry, equal weather, ties and extreme-weather blockers against existing tests; open a named bug only on a reproduced discrepancy.

**Code:** `src-tauri/src/combat/field.rs::set_weather`, `src-tauri/src/combat/field.rs::weather-contest`, `src-tauri/src/combat/field.rs::lifecycle`, `src-tauri/src/combat/field.rs::expire`.

**Existing evidence:** `field_weather_ability_contest_suppression_departure_and_reconnect`, `field_shared_attack_freezes_weather_when_its_source_faints`, `field_round_end_events_complete_before_expiry_and_ticks_are_once`. These names are references, not claims that every edge listed above is already tested.

<a id="m3-w03"></a>

### M3-W03: Air Lock and Cloud Nine suppression

**Existing automation.** Global Air Lock suppression and holder-only Cloud Nine exemption exist, with weather records retained. Ice reads target-effective weather; interactions with old blocks remain M3-I05.

**Next:** Retain suppression infrastructure; verify entry/exit, retained source and effective-weather edge cases without treating suppression as weather deletion.

**Code:** `src-tauri/src/combat/field.rs::weather`, `src-tauri/src/combat/field.rs::set_weather`.

**Existing evidence:** `field_weather_ability_contest_suppression_departure_and_reconnect`. These names are references, not claims that every edge listed above is already tested.

<a id="m3-t01"></a>

### M3-T01: Mixed terrain precedence and scope

**Source/interpretation review.** Move-cast overlapping different terrains request a GM ruling. Surge-versus-surge has Will contests; the ability cards mention higher Will as a stalemate hint. Surge-versus-move and Seed Sower conflicts are explicit rulings. Terrain application currently removes other terrain records globally.

**Next:** Review partially overlapping side/all scopes and the differing source cases. Keep genuinely unspecified move/ability precedence and tied-ability policy visible; do not invent a universal last-cast rule from the weather paragraph.

**Code:** `src-tauri/src/combat/field.rs::move_effect`, `src-tauri/src/combat/field.rs::terrain-contest`, `src-tauri/src/combat/field.rs::terrain-ruling`, `src-tauri/src/combat/field.rs::apply_terrain`, `src-tauri/src/combat/field.rs::seed_sower`.

**Existing evidence:** `environment_entry_abilities_resolve_against_preexisting_conditions`, `field_terrain_healing_is_capped_and_expiry_does_not_repeat`. These names are references, not claims that every edge listed above is already tested.

<a id="m3-f01"></a>

### M3-F01: Force fields, bypass and cleanup

**Existing automation.** Same-category damage reduction does not stack. Existing screen code and M2-E Safeguard have shared bypass/removal behavior; this audit does not reopen their delivered scope.

**Next:** Retain delivered mechanics. Check combination/expiry regressions only where a concrete edge is identified.

**Code:** `src-tauri/src/combat/field.rs::force_field_bypass`, `src-tauri/src/combat/field.rs::screen_cleaner`.

**Existing evidence:** `field_barriers_do_not_stack_and_require_snow_for_veil`, `cure_protection_defog_removes_safeguard_and_toxic_spikes`. These names are references, not claims that every edge listed above is already tested.

<a id="m3-f02"></a>

### M3-F02: Magic Guard explicit Perish Song exception

**Source/code gap.** Printed MASTER 554 / PDF 421 and the exact Magic Guard JSON explicitly allow Perish Song damage. Perish Song currently calls special::indirect, which blocks Magic Guard; an existing M2-D regression asserts that survival. The passing test confirms current behavior, not agreement with the printed rule.

**Next:** Record this source-backed correction candidate separately from M2 delivery scope. Before a future fix, reproduce the deadline hit with and without Magic Guard, replace the conflicting expectation, and preserve its other indirect-damage immunities and historical encounters. No mechanic changes in this audit.

**Code:** `src-tauri/src/combat/recurring.rs::pub fn end_round`, `src-tauri/src/combat/special.rs::pub(super) fn indirect`.

**Existing evidence:** `recurring_perish_indirect_immunity_and_nonstacking_are_preserved`. These names are references, not claims that every edge listed above is already tested.

<a id="m3-a01"></a>

### M3-A01: Healing Pits printed Heal 1 and configurable variants

**Existing, configurable.** Visual review confirms a Heal 1 icon omitted by text extraction. Existing UI parameter defaults to 1; engine takes a configured positive amount, spends one action and depletes the pit. Prior documentation incorrectly called the printed amount unspecified.

**Next:** Correct the documentation now. Keep 1 as the printed reference and distinguish GM-designed variants. No missing healing subsystem or mandatory new amount choice is established by this audit.

**Code:** `src-tauri/src/combat/field.rs::FieldCommand::HealingPit`, `src/BattlefieldControls.tsx::[parameter, setParameter]`.

<a id="m3-a02"></a>

### M3-A02: High Poles difficulty

**Existing, configurable.** Source defines Dexterity + Athletic at round end for non-Flying Pokemon, with 2 damage on failure, but gives no success difficulty. Existing engine validates and uses a positive GM parameter.

**Next:** Keep GM-defined difficulty with provenance. Do not invent an official default or classify the supported check/damage as missing.

**Code:** `src-tauri/src/combat/field.rs::high-poles`, `src-tauri/src/combat/field.rs::FieldCommand::Preset`.

**Existing evidence:** `field_arena_permissions_typing_and_reaction_restrictions`. These names are references, not claims that every edge listed above is already tested.

<a id="m3-a03"></a>

### M3-A03: Torn World target distribution

**Intentional GM decision.** Source says targets change randomly without a sampling distribution or legality rules. Engine blocks ordinary targeting until GM adjudication.

**Next:** Retain explicit target ruling. Automation requires a separately recorded group policy covering candidates, Self/Area/Field moves and invalid targets.

**Code:** `src-tauri/src/combat/field.rs::guard_move`.

<a id="m3-a04"></a>

### M3-A04: Cemetery / Run Away narrative escape

**Intentional GM decision.** Cemetery supplies an Intimidate bonus and Run Away to non-Ghost Pokemon. Narrative escape bonus dice are explicitly at storyteller discretion in the ability card; existing encounter hooks are separate.

**Next:** Retain storyteller decisions for narrative escape; do not turn this into a blanket missing battle mechanic.

**Code:** `src-tauri/src/combat/field.rs::cemetery`, `src-tauri/src/combat/extra.rs::Run Away`.

<a id="m3-a05"></a>

### M3-A05: Arena exposure clocks and round-end ordering

**Existing automation.** Underwater difficulty uses exposure rounds from effect start, increasing by two. Fire/sewage Chance events, falling checks, expiry ordering and once-per-round ticks already exist.

**Next:** Retain existing mechanics. Cross-check scoped entrants, suppression, pending events and expiry with named reproductions before adding tasks; trainer damage and narrative simulation stay outside this batch.

**Code:** `src-tauri/src/combat/field.rs::underwater`, `src-tauri/src/combat/field.rs::end`, `src-tauri/src/combat/field.rs::expire`.

**Existing evidence:** `field_round_end_events_complete_before_expiry_and_ticks_are_once`, `field_round_checks_are_host_rolled_owned_choices_and_idempotent`. These names are references, not claims that every edge listed above is already tested.

## Reconciled records

| Record | Correction |
| --- | --- |
| `rules/coverage.json` / `frozen` | Ordinary Special attacks and Snow 7 HP/shared defense 3 already execute. Retain `partial` because category wording, Snow transitions and named fixed/variable formulas remain review items. |
| `rules/ability-coverage.json` | Reconcile stale Levitate and Magic Guard scopes; keep partial labels and record the explicit Perish Song mismatch as M3-F02. |
| `rules/coverage.json` / `battlefield` | Remove the obsolete claim that held items and transformations have no system; those foundations were delivered in later releases. Point to the named M3 review. |
| `docs/REMAINING-WORK.md` | Keep M2 complete, replace the stale generic status/ice tasks with the reviewed scope, and correct Healing Pits and Levitate/hazard claims. |
| `docs/battlefield.md` | Label the 0.13.0 release report as historical so old unsupported-system statements and old test counts are not current claims. Correct the Healing Pits icon omission. |
| `docs/battle-recurring-status.md`, `docs/MOVE-EFFECT-AUDIT.md` | Annotate the existing Perish Song / Magic Guard claim with the newly verified printed-source mismatch. |
| `docs/GM-CHOICE-ROADMAP.md` | Link this deliverable; do not schedule a replacement terrain/weather system or silently choose a policy. |

## Completion criteria for later implementation

A future change must identify its checklist ID, quote or paraphrase its exact source and record any interpretation; preserve historical encounter behavior, host validation, permissions, receipts, rollback and sheet isolation; and run tests specific to the changed behavior. No Clippy, app packaging or native/physical-network acceptance is required for this documentation deliverable. M2 delivered clauses and M4 catalog corrections remain separate.


## Validation of this audit deliverable

- 23 existing battlefield/environment/ice/Freeze-chance Rust regressions passed, including the loopback environment reconnect case. The initial sandbox socket restriction was resolved by rerunning with localhost access.
- The existing Perish Song/Magic Guard regression was also rerun and passed its current survival expectation. That is baseline evidence for M3-F02, not approval of the expectation against the printed source. Together these are 24 distinct existing tests; the full Rust suite was not rerun for this audit-only batch.
- 71 frontend tests and 32 script/catalog checks passed, including the new source-fingerprint/code-reference checklist check. The catalog was regenerated to match the reconciled registry metadata; app and engine versions remain unchanged.
- Diff checks passed. No runtime source was edited, no new mechanic or house policy was adopted, and no Clippy, app package, native UI or physical-network acceptance was run.
