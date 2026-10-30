# M3: battlefield and ice review checklist

Completed review: **30 October 2026**, app **0.42.0**, engine **0.35.0-core**. All 16 checks are closed through implementation, regression verification, or an explicit retained GM ruling/parameter. Completion does not mean every underspecified rule is automated.

## Completion gate

- [x] Reconcile the named checks with printed sources and exact catalog cards.
- [x] Resolve M3-I03/I04/I05/T01 through the decisions recorded below.
- [x] Correct Magic Guard / Perish Song and preserve historical encounters.
- [x] Verify setup, Sunny Day, Drought, live night changes, suppression and expiry using the user's Sunny-at-night ruling.
- [x] Add focused ice, weather, terrain and Perish Song regressions.
- [x] Retain source fingerprints and update coverage/backlog records.

## Sources and interpretation limits

The [machine-readable review](../rules/battlefield-ice-review.json) preserves original source hashes, printed/PDF references, baseline findings, code symbols and real fixture names. RULING pages 85, 89 and 91 and MASTER printed 554 / PDF 421 were visually inspected in the original audit. Source files remain unchanged. Healing Pits page 91 explicitly shows Heal 1.

Sunny at night follows the user's explicit 30 October ruling. Snow Special Defense follows the narrower printed wording. Live ice weather timing and terrain tie handling are recorded local policies; unspecified object formulas, terrain precedence and arena choices remain visible GM decisions.

## Named checks

<a id="m3-i01"></a>

### M3-I01: Ordinary ice-breaking baseline

- [x] **retained-existing** — Ordinary Physical and Special attacks retain 5 HP, Defense 2 and Special Defense 2, successful super-effective breaking, action costs, and Pokemon HP isolation.

**Code:** `src-tauri/src/combat.rs::ice_attack`, `src-tauri/src/combat.rs::ice_damage_pool`.

**Regression references:** `frozen_subject_can_break_ice_at_round_end_without_losing_its_hp`, `special_charge_defense_is_temporary_and_matcha_can_break_its_own_ice`.

<a id="m3-i02"></a>

### M3-I02: Snow ice HP and physical Defense

- [x] **verified-existing** — Focused tests verify Snow 7 HP and physical Defense 3, compared with ordinary 5 HP and Defense 2.

**Code:** `src-tauri/src/combat.rs::ice_damage_pool`, `src-tauri/src/combat.rs::Ice block damage`.

**Regression references:** `field_all_weather_bonuses_and_move_failures`, `m3_snow_ice_category_pools_and_live_suppression`, `m3_snow_block_hp_uses_live_weather_without_resetting_damage_or_pokemon_hp`.

<a id="m3-i03"></a>

### M3-I03: Snow ice Special Defense wording

- [x] **implemented** — Read the printed attributes narrowly: Snow raises Defense to 3; Special Defense remains 2. This is a recorded interpretation of RULING 85/89, not a claim that page 89 names Special Defense. Historical encounters retain the shared 3 subtraction.

**Code:** `src-tauri/src/combat.rs::ice_damage_pool`.

**Regression references:** `m3_snow_ice_category_pools_and_live_suppression`.

<a id="m3-i04"></a>

### M3-I04: Fixed and variable damage against ice: five named cards

- [x] **retained-gm-ruling** — Each of Sonic Boom (fixed 1), Dragon Rage (fixed 2), Seismic Toss (rank dice, super-effective Fight), Night Shade (rank dice), and Super Fang (remaining-HP dice) remains an explicit atomic manual_rule for ice. The cards specify Pokemon targets, not object formulas. GM must record object damage/removal, including the Seismic Toss instant-break decision, and any action/cost adjudication; unsupported attempts spend nothing. No object HP conversion is invented.

**Code:** `src-tauri/src/combat.rs::ice_attack`, `src-tauri/src/combat/special.rs::resolve_formula`.

**Regression references:** `m3_named_nonordinary_ice_formulas_remain_atomic_gm_rulings`.

<a id="m3-i05"></a>

### M3-I05: Snow changes or suppression while ice is damaged

- [x] **retained-reviewed-policy** — Retain live target-effective weather for each damage resolution, with cumulative damage unchanged: Snow 7 HP, otherwise 5; no automatic removal solely on a weather change. Air Lock and target Cloud Nine remove Snow benefits without deleting weather. Snow starting/ending changes the next pool/threshold. This timing policy is local and explicit because source does not settle snapshot semantics.

**Code:** `src-tauri/src/combat.rs::ice_damage_pool`, `src-tauri/src/combat.rs::Ice block damage`, `src-tauri/src/combat/field.rs::weather`.

**Regression references:** `m3_snow_ice_category_pools_and_live_suppression`, `m3_snow_block_hp_uses_live_weather_without_resetting_damage_or_pokemon_hp`.

<a id="m3-w01"></a>

### M3-W01: Sunny Weather at night

- [x] **implemented-user-ruling** — User ruling on 30 October 2026: move/ability Sunny can activate at night and persists for its effective duration/source lifetime. Night suppresses natural Sunny only; current-engine natural Sunny setup/environment at night is rejected. Sunny Day remains four rounds, Drought lasts with its source; reachable-sunlight/indoor restrictions and historical encounters remain unchanged.

**Code:** `src-tauri/src/combat/field.rs::validate_setup`, `src-tauri/src/combat/field.rs::guard_move`, `src-tauri/src/combat/field.rs::set_weather`, `src-tauri/src/combat/field.rs::FieldCommand::Night`.

**Regression references:** `environment_rejects_invalid_setup_unauthorized_starts_and_historical_engines_atomically`, `environment_moonlight_uses_live_night_with_explicit_overrides_and_historical_fallback`, `m3_sunny_move_at_night_retains_duration_and_natural_weather_stays_dark`, `m3_drought_at_night_survives_night_change_and_suppression`, `m3_sunny_natural_setup_and_environment_reject_night_atomically`.

<a id="m3-w02"></a>

### M3-W02: Weather priority, ability Will contests and expiry

- [x] **verified-existing** — Retain move replacement, equal-weather ability handling, Will contests/tied rerolls, source departure, natural fallback, duration expiry and extreme-weather guards; existing regressions rerun.

**Code:** `src-tauri/src/combat/field.rs::set_weather`, `src-tauri/src/combat/field.rs::weather-contest`, `src-tauri/src/combat/field.rs::lifecycle`, `src-tauri/src/combat/field.rs::expire`.

**Regression references:** `field_weather_ability_contest_suppression_departure_and_reconnect`, `field_shared_attack_freezes_weather_when_its_source_faints`, `field_round_end_events_complete_before_expiry_and_ticks_are_once`.

<a id="m3-w03"></a>

### M3-W03: Air Lock and Cloud Nine suppression

- [x] **verified-existing** — Retain Air Lock global suppression and Cloud Nine holder-only exemption, preserving source records; focused night/Snow checks supplement existing entry/exit regressions.

**Code:** `src-tauri/src/combat/field.rs::weather`, `src-tauri/src/combat/field.rs::set_weather`.

**Regression references:** `field_weather_ability_contest_suppression_departure_and_reconnect`, `m3_drought_at_night_survives_night_change_and_suppression`, `m3_snow_ice_category_pools_and_live_suppression`.

<a id="m3-t01"></a>

### M3-T01: Mixed terrain precedence and scope

- [x] **implemented-with-gm-ruling** — Preserve disjoint scoped terrains when applying entry-ability winners; consider overlap when collecting contenders and Seed Sower conflicts. Overlapping different move terrains and move/Surge/Seed Sower precedence retain explicit GM rulings. Ability Will contests retain tied rerolls as the existing local policy. An accepted replacement removes whole overlapping records; GM must split/remove broad records explicitly for partial coexistence. No universal last-cast rule is adopted.

**Code:** `src-tauri/src/combat/field.rs::move_effect`, `src-tauri/src/combat/field.rs::terrain-contest`, `src-tauri/src/combat/field.rs::terrain-ruling`, `src-tauri/src/combat/field.rs::apply_terrain`, `src-tauri/src/combat/field.rs::seed_sower`.

**Regression references:** `environment_entry_abilities_resolve_against_preexisting_conditions`, `field_terrain_healing_is_capped_and_expiry_does_not_repeat`, `m3_disjoint_terrain_entry_preserves_other_side_and_overlap_requests_ruling`.

<a id="m3-f01"></a>

### M3-F01: Force fields, bypass and cleanup

- [x] **verified-existing** — Retain nonstacking screens, bypass, cleanup and Safeguard integrations; existing combination and cleanup regressions rerun.

**Code:** `src-tauri/src/combat/field.rs::force_field_bypass`, `src-tauri/src/combat/field.rs::screen_cleaner`.

**Regression references:** `field_barriers_do_not_stack_and_require_snow_for_veil`, `cure_protection_defog_removes_safeguard_and_toxic_spikes`.

<a id="m3-f02"></a>

### M3-F02: Magic Guard explicit Perish Song exception

- [x] **implemented** — Perish Song bypasses Magic Guard in newly started reviewed encounters, as explicitly required by MASTER 554. Historical encounters retain immunity; other indirect damage and Wonder Guard paths are unchanged.

**Code:** `src-tauri/src/combat/recurring.rs::pub fn end_round`, `src-tauri/src/combat/special.rs::pub(super) fn indirect`.

**Regression references:** `recurring_perish_indirect_immunity_and_nonstacking_are_preserved`, `m3_perish_song_bypasses_magic_guard_only_in_reviewed_encounters`.

<a id="m3-a01"></a>

### M3-A01: Healing Pits printed Heal 1 and configurable variants

- [x] **retained-gm-parameter** — Printed Healing Pits reference is Heal 1; UI defaults to 1. Existing positive configurable amount, one-action cost and depletion remain. Other amounts are GM variants.

**Code:** `src-tauri/src/combat/field.rs::FieldCommand::HealingPit`, `src/BattlefieldControls.tsx::[parameter, setParameter]`.

**Verification:** source/code review; no new runtime behavior for this retained GM parameter.

<a id="m3-a02"></a>

### M3-A02: High Poles difficulty

- [x] **retained-gm-parameter** — High Poles uses Dexterity + Athletic at round end, 2 damage on failure, and an explicit positive GM difficulty because the source omits difficulty.

**Code:** `src-tauri/src/combat/field.rs::high-poles`, `src-tauri/src/combat/field.rs::FieldCommand::Preset`.

**Regression references:** `field_arena_permissions_typing_and_reaction_restrictions`.

<a id="m3-a03"></a>

### M3-A03: Torn World target distribution

- [x] **retained-gm-ruling** — Torn World targeting remains explicit GM adjudication; source gives no sampling distribution/legality policy. Ordinary targeting stays blocked until adjudicated.

**Code:** `src-tauri/src/combat/field.rs::guard_move`.

**Verification:** source/code review; no new runtime behavior for this retained GM parameter.

<a id="m3-a04"></a>

### M3-A04: Cemetery / Run Away narrative escape

- [x] **retained-gm-ruling** — Cemetery encounter hooks remain implemented; Run Away narrative escape bonus remains storyteller discretion.

**Code:** `src-tauri/src/combat/field.rs::cemetery`, `src-tauri/src/combat/extra.rs::Run Away`.

**Verification:** source/code review; no new runtime behavior for this retained GM parameter.

<a id="m3-a05"></a>

### M3-A05: Arena exposure clocks and round-end ordering

- [x] **verified-existing** — Retain exposure-relative underwater clocks, pending Chance/falling checks, events-before-expiry ordering and once-per-round ticks. Existing scoped checks rerun; trainer damage/narrative simulation remain excluded.

**Code:** `src-tauri/src/combat/field.rs::underwater`, `src-tauri/src/combat/field.rs::end`, `src-tauri/src/combat/field.rs::expire`.

**Regression references:** `field_round_end_events_complete_before_expiry_and_ticks_are_once`, `field_round_checks_are_host_rolled_owned_choices_and_idempotent`.

## Compatibility and acceptance

The serialized `battlefield_review` gate defaults to false for saved encounters. New 0.35.0-core encounters enable it; 0.34.0-core retains M2-E features and previous behavior. Protocol 43 carries the added state. Existing permission checks, host dice, receipts, rollback and library-sheet isolation remain in place.

Validation: **782 Rust tests passed, 1 diagnostic ignored**; the diagnostic was separately executed and passed, regenerating all 892 move-effect probes. **Eight new M3 regressions** are included. **71 frontend tests and 33 script/catalog checks passed**, along with TypeScript, Rust formatting and diff checks. Localhost tests required a rerun outside the socket-restricted sandbox. Frontend tests used workspace-local temporary files. Native UI, Windows/Linux packages and physical-network acceptance are left to the user; no Clippy or packaging is required for this batch.
