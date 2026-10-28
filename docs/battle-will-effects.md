# Will spending and draining audit

Application **0.35.0**, engine **0.28.0-core**, protocol **36**, schema **4**.

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
| Eerie Spell, Grudge | Remain explicit GM effects; review non-stacking scope and Loyalty lifecycle separately |
| Sappy Seed | Recurring effect remains unsupported. Printed p.479 says the user **heals** per damage; JSON erroneously says **deals**. Record correction and evolution eligibility before implementation. |

This is an audit of Will clauses, not a certification of every other clause or icon on these moves. Existing exact selections, costs, limits and historical gates remain authoritative.

## Spite

Successful, unblocked effect resolution reduces current Will to `min(current, 1)`: zero stays zero. It clears accumulated pain relief, active Chance benefits and pending Fate. It preserves GM narrative authorization, original Will capacity, prior spending markers, completed rolls, and permanent sheets. It does not reverse healing or remove ongoing moves that had their own activation cost; the reset concerns the core Will-spending benefits. No Will is refunded.

Chance cancellation is stored separately from its round marker so Spite cannot reopen the round's Chance/Fate spending restriction. All ordinary, reaction, status, successive-action and per-target accuracy calculations honor cancellation. A later valid Chance expenditure clears cancellation.

Normal accuracy, reactions, reflection, ownership, decoys, shields and ability protection remain in the common resolution pipeline. Aroma Veil blocks Spite. Preview states the drain and cleared benefits; ordinary battle state and the effect-resolution event record the result. Effects resolve once per queued action and are covered by host receipts and rollback.

New current-engine encounters snapshot `TurnFlow.spite = true`. Historical saves omit it as false and retain explicit GM adjudication. The preceding status engine remains recognized and retains status escalation and all earlier gates. Upgrade between battles and use matching protocol clients. The additive Chance cancellation field defaults to false in older saves.

## Verification

Regressions cover zero/one/multiple Will, benefit reset, spending history, cancellation dice, ownership, historical saves, Aroma Veil, decoys, repeat-resolution rejection, unchanged sheets, serialization and rollback. A localhost-room test covers remote GM resolution, duplicate receipts, reconnect, SQLite persistence and rollback. Validation: all 699 Rust cases passed across the final 697/699 parallel run and the successful 5/5 serial multiplayer rerun, which included both cases that timed out under parallel load. The six Spite regressions passed, including its localhost-room case. Also passed: 64 frontend tests, 23 catalog/script checks, TypeScript, warnings-denied Clippy, Rust formatting and diff checks. The first sandbox runs could not open test sockets or share the frontend temporary cache; localhost permission and a workspace-local frontend cache resolved those environment restrictions. Native GUI and physical LAN acceptance are not claimed; no application packages were built.
