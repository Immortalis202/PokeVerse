# Will spending and draining audit

Application **0.36.0**, engine **0.29.0-core**, protocol **37**, schema **4**.

## Sources and scope

Compared the exact supplied v3.0 move JSON cards with executable move handlers. The 34 relevant cards, their unchanged prose and AddedEffects, source paths and SHA256 values are recorded in `rules/will-moves-review.json`. Will in an accuracy pool does not itself spend Will. Quash's grammatical “Target's will go last” is not a resource effect. Hold Hands uses a resisted Will score and remains a narrative decision, outside this spending audit. Automatic Will recovery remains excluded.

Also checked the MASTER printed cards: Spite p.474 (PDF 341), Grudge p.472 (PDF 339), Eerie Spell p.511 (PDF 378), and Sappy Seed p.479 (PDF 346). Spite's printed Support card specifies Will + Intimidate, one foe, no damage, reducing Will to 1 and resetting spending effects. Eerie Spell specifies three Will without benefit and says it does not stack. Grudge additionally faints the user, drains to zero and requires escalating end-of-round Loyalty checks. These are distinct effects.

## Engine comparison

| Cards | Existing Will handling |
| --- | --- |
| Recover, Slack Off, Soft Boiled | One Will, Basic Heal in `combat.rs` |
| Heal Pulse | One Will; Basic Heal or Mega Launcher Complete Heal in `extra.rs` |
| Floral Healing, Heal Order, Milk Drink, Moonlight, Morning Sun, Rest, Roost, Shore Up, Swallow, Synthesis, Lunar Blessing | One Will, move-specific healing path in `special.rs`; Swallow requires Stockpile |
| Aqua Ring, Ingrain, Life Dew, Wish | One Will and persistent/delayed healing in `special.rs` |
| Grassy Terrain | One Will, side effect in `field.rs` |
| Jungle Healing | One Will per action, not per recipient, in `special.rs` |
| Leech Seed | One Will and recurring damage/healing in `special.rs` |
| Pain Split, Strength Sap | One Will and their explicit HP/stat effects in `special.rs` |
| Pollen Puff | One Will only for allied healing in `special.rs` |
| Purify | Optional one Will for self-healing after a successful cure in `special.rs` |
| Revival Blessing | Requires positive Will, spends all remaining Will in `special.rs` |
| Roar Of Time | Target pays one Will per attempted action through the existing special/sequencing paths |
| Trump Card | Damage depends on remaining Will; no activation cost |
| Healing Wish | Existing sacrifice/full-HP path. JSON healing metadata says one Will, but prose specifies sacrifice and no Will cost. Preserve this discrepancy; no new charge added here. |
| Spite | First unsupported drain implemented in this batch |
| Eerie Spell | Implemented in 0.36.0: remove up to three current Will per successful unblocked hit; no benefit or persistent effect. User interpretation recorded 28 October 2026. |
| Grudge | Implemented in 0.36.0: user faints, target drains to zero and loses core Will benefits; recurring Loyalty checks. |
| Sappy Seed | Excluded from planned implementation by the user on 28 October 2026. Printed p.479 says the user **heals** per damage; JSON erroneously says **deals**. Source discrepancy remains evidence only. |

This is an audit of Will clauses, not a certification of every other clause or icon on these moves. Existing exact selections, costs, limits and historical gates remain authoritative.

## Spite

Successful, unblocked effect resolution reduces current Will to `min(current, 1)`: zero stays zero. It clears accumulated pain relief, active Chance benefits and pending Fate. It preserves GM narrative authorization, original Will capacity, prior spending markers, completed rolls, and permanent sheets. It does not reverse healing or remove ongoing moves that had their own activation cost; the reset concerns the core Will-spending benefits. No Will is refunded.

Chance cancellation is stored separately from its round marker so Spite cannot reopen the round's Chance/Fate spending restriction. All ordinary, reaction, status, successive-action and per-target accuracy calculations honor cancellation. A later valid Chance expenditure clears cancellation.

Normal accuracy, reactions, reflection, ownership, decoys, shields and ability protection remain in the common resolution pipeline. Aroma Veil blocks Spite. Preview states the drain and cleared benefits; ordinary battle state and the effect-resolution event record the result. Effects resolve once per queued action and are covered by host receipts and rollback.

New current-engine encounters snapshot `TurnFlow.spite = true`. Historical saves omit it as false and retain explicit GM adjudication. The preceding status engine remains recognized and retains status escalation and all earlier gates. Upgrade between battles and use matching protocol clients. The additive Chance cancellation field defaults to false in older saves.

## Grudge and Eerie Spell follow-up

The user requested the printed Grudge behavior on 28 October 2026. On a successful, unblocked Grudge effect, the user faints, target Will becomes zero and core spending benefits reset exactly as for Spite. The target must have a recorded Loyalty of 0-5 before effect resolution; missing Loyalty produces an explicit manual-rule error before either participant changes. No default score is invented.

At the end of the application round and every subsequent round, the target rolls its Loyalty score, requiring 1 success, then 2, 3, 4 and so on. This is a success threshold, not a changed die-face threshold. A successful roll increases the next requirement by one; failure directly faints the target, without damage rolls or defense reductions. Zero Loyalty rolls zero dice and fails the first check. The source's faint does not end the curse. Switching does not remove it; benched registered targets still check. Target faint or encounter end removes it. Reapplying Grudge does not restart an existing curse's difficulty.

Next Round first records ordinary round-end effects and exposes pending Grudge checks. Target owner or GM then triggers **Roll Grudge Loyalty** in the battle controls; the host supplies all dice. The target and current required successes remain visible. Round advancement and new turn actions wait for the check. The phase and checked-round marker prevent double rolls or duplicate round-end damage. Check outcome, exact dice and difficulty are logged. Serialization, receipts, reconnect and battle rollback include the curse and its progression.

The user selected a simple flat Eerie Spell drain: remove three current Will from the enemy, clamped at zero. Each later successful hit can remove another three; no persistent modifier or stacked status is created. The move retains ordinary Psychic Special damage and sound protections. Unlike Spite/Grudge it does not clear existing core Will benefits. Shields, decoys and ability/type immunities use the shared resolution paths; sound moves retain their existing decoy bypass.

New encounters snapshot `TurnFlow.will_drains = true`; omitted historical flags remain false. New `SpecialState.grudges` defaults to empty. The Spite engine remains supported with Spite enabled and the two newer drains manual. Upgrade between encounters and use matching protocol clients. No automatic Will recovery is added.

Within the audited Will clauses, no other unimplemented spending/draining move remains in scope after the Sappy Seed exclusion. Healing Wish's metadata/prose disagreement remains explicitly unresolved; this batch does not add its metadata-only Will charge. Other status/move families in the general roadmap are separate work.

## Verification from 0.35.0

Regressions cover zero/one/multiple Will, benefit reset, spending history, cancellation dice, ownership, historical saves, Aroma Veil, decoys, repeat-resolution rejection, unchanged sheets, serialization and rollback. A localhost-room test covers remote GM resolution, duplicate receipts, reconnect, SQLite persistence and rollback. Validation: all 699 Rust cases passed across the final 697/699 parallel run and the successful 5/5 serial multiplayer rerun, which included both cases that timed out under parallel load. The six Spite regressions passed, including its localhost-room case. Also passed: 64 frontend tests, 23 catalog/script checks, TypeScript, warnings-denied Clippy, Rust formatting and diff checks. The first sandbox runs could not open test sockets or share the frontend temporary cache; localhost permission and a workspace-local frontend cache resolved those environment restrictions. Native GUI and physical LAN acceptance are not claimed; no application packages were built.

## Verification for 0.36.0

Passed 707 Rust tests (four test threads, localhost sockets enabled), including 13 Will-effect tests; 65 frontend tests; 23 catalog/script checks; TypeScript; warnings-denied Clippy; Rust formatting and diff checks. New regressions cover Grudge sacrifice/reset, 1/2/3/4-success progression, round blocking, zero Loyalty, missing Loyalty, source faint, switching, ownership, historical gates, save/reconnect/receipt/rollback, and Eerie Spell repeat drains, clamping and Soundproof/Psychic immunity. The battle UI test checks difficulty display and owner/GM-only roll buttons. No app packages were built; native GUI and physical-network acceptance remain unrun.
