# Battle events and per-target accuracy

Application 0.24.0 / engine 0.17.0-core / protocol 25 enables these rules for new battles. `TurnFlow.events.enabled` defaults to false for historical encounters. Engine 0.16.0-core remains supported with its previous behavior; database schema remains 4.

## Stench

Source: supplied Pokerole 3.0 Stench card. A non-ranged Physical hit queues one Chance Die to Flinch the attacker. The existing host-rolled ability event records the source and target, requires the holder's owner or GM, and preserves the ordinary Long Reach and Wonder Skin interactions. Narrative encounter-rate reduction is excluded.

## Pressure

Source: supplied Pokerole 3.0 Pressure card. Entry or effective activation spends half of each living in-range foe's remaining Will, rounded up, without granting an action or resource benefit. Unknown/unreviewed Will requires explicit GM resolution rather than assuming a number.

Saved source and target sets prevent repeated refreshes from spending Will again. An overlapping Pressure entry does not spend Will again on a currently covered target. When no opposing Pressure covers the target, that target's exposure ends; a later Pressure entry can apply again. Entering range after the source's entry does not itself trigger the ability. Departure and fainting remove sources, and battle end clears active exposure sets while retaining history. This records the non-stacking interpretation explicitly rather than importing video-game PP mechanics.

## Tangled Feet

One shared host accuracy roll produces separate success and critical results for each target. A confused opposing Tangled Feet holder receives the existing two-success penalty. Keen Eye prevents it. Targets missed by that roll are marked avoided without spending a reaction; successful targets retain their individual results through response windows and damage ordering. Later status changes do not rewrite those saved results. Old encounters retain the prior explicit mixed-target ruling, and old target records default the new fields to null.

Dancer retains each original target's success count when copying a mixed-target Dance, including when the source failed all of its action requirements. The free copy uses its own one-success action threshold and critical margin. A target absent from the original roll uses the highest recorded source count; the copy does not depend on damage-resolution ordering. Unsupported Dance added effects still require their existing explicit GM resolution.

## Electric charge consumption

Wind Power and Electromorphosis charge the next Electric move. A committed Electric Support attempt now consumes the charge without granting Support dice. Existing damaging, failed-attempt and shared-damage behavior remains intact. The version gate preserves historical behavior.

## Verification and remaining integration

Focused fixtures cover Stench permissions, its chance pool, Long Reach and Wonder Skin; Pressure's rounding, range, entry, no refresh, overlapping emitters and reactivation; mixed-target misses, target-specific criticals and Keen Eye; and both Electric charge abilities' Support consumption. Saved per-target results survive serialization and damage rollback. Old target records without the additional fields load normally, and engine 0.16 retains its previous mixed-target ruling.

The localhost room fixture uses separate host, GM and owner stores. It validates owner-controlled Pressure entry, one Will expenditure, unauthorized/stale/duplicate commands, reconnect, SQLite persistence, GM rollback of both deployment and spent Will, and a new entry after rollback. This does not establish native-app interaction or physical LAN acceptance.

Release validation: all 448 Rust checks (81 binding exports and 367 behavioral tests), 35 frontend tests and 12 catalog checks passed, along with TypeScript, formatting, warnings-denied Clippy and production build. The final Windows NSIS installer is 213.80 MiB and includes the Dancer correction. It was not launched.
