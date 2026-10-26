# Reduce recurring GM work

Direction agreed 23 October 2026: PokeVerse is for a private friend group with technical users. Public release, trusted Windows signing and distribution polish are not planned work. Native and networking test results must still describe checks actually performed; this scope decision does not mark pending acceptance tests as passed.

This roadmap follows the implemented ability/item batches. It distinguishes missing automation from intentional rules choices. The initial audit did not change runtime behavior. The user supplied the eight interpretations on 26 October; they are now implemented in app 0.31.0 / engine 0.24.0-core. See [selected group rules](battle-group-rules.md).

## 1. Campaign house rules

Implemented first batch: the eight interpretations selected by this private group are fixed defaults in new current-engine encounters, so no repeated policy selection is required. Preserve historical encounter-local decisions, explanations, privacy, host validation, receipts and rollback. Generic GM adjudication remains available for explicit exceptions. A campaign default must not silently rewrite an active/saved battle. Unselected policies must remain explicit; recommendations must not become invented official rules.

These recurring choices are present in current runtime code and documented batch reviews:

| Choice | Why a choice exists | Current location |
| --- | --- | --- |
| Torrent damage bonus | Printed text and icon disagree | `combat/standalone.rs` |
| Rivalry comparison | Supplied gender wording differs from printed preference wording | `combat/standalone.rs` |
| Full-HP revival versus healing cap | Full revival wording conflicts with the listed cap exceptions | `combat/items/completion.rs` |
| Ring Target immunity scope | Broad printed wording needs an explicit supported scope | `combat/items/completion.rs` |
| Choice Scarf timing | Granted Reaction conflicts with a move's Late Reaction | `combat/items/completion.rs` |
| Mycelium Might / ability timing | Granted timing conflicts with a printed reaction | `combat/tactics.rs` |
| Magic Coat duration | Printed card does not specify duration | `combat/tactics.rs` |
| Bide recipient interpretation | Contradictory User/Target wording | See `battle-special-moves.md` and current special-move rules |

Keep actor-specific choices separate from campaign policies. A learned move for a Choice item, the actual weight of a form, a private Forewarn comparison, accessible supplies, individual herbal acceptance and a replacement Pokémon are not global rules defaults.

Completion criterion: the selected interpretation is visible at campaign setup and encounter preparation, applied only through authoritative commands, snapshotted into new battles, protected from conflicting policy commands, and covered by permissions/retry/reconnect/rollback/historical tests. The selected interpretations are recorded in battle-group-rules.md and rules/group-rules.json. This audit table retains the original reasons choices existed.

## 2. Automate defined effects currently delegated to the GM

After recurring policies, implement source-reviewed mechanics in separate coherent batches:

- Completed in 0.32.0: [starting multiplayer day/night, weather, terrain and arena effects](battle-starting-environment.md), including Moonlight using the live night setting with deliberate overrides. Wild/training encounters retain separate preparation.
- Persist Shadow Force's defined recovery restriction beyond the encounter; encounter-local debt already exists.
- Review remaining identity effects individually: Entrainment, Doodle and Core Enforcer.
- Extend executable Support/stat/status/move definitions so reflection, copied traits and dependent abilities can resolve more effects automatically. Review the exact cards; ordinary damage support does not imply all 892 move effects are implemented.
- Review status escalation and remaining Will spending/draining cards. Preserve explicit choices when printed values conflict. Automatic Will recovery remains excluded.

Each batch needs source evidence, clear supported boundaries, host validation, preview/output feedback, ownership/privacy, persistence, retry/rollback and relevant regressions. Version changes are warranted for changed authoritative behavior; public release/signing work is not a prerequisite.

## 3. Keep encounter facts and narrative decisions with the GM

Abstract range, story circumstances, unknown weights, private information comparisons, missing arena parameters, herbal acceptance and genuine unique-ability/terrain precedence conflicts still need table input. Improve reuse and visibility where useful; do not replace these decisions with video-game rules or silent approximations.

The older backlog contains historical tables whose labels predate implemented batches. Current code and per-definition registries take precedence. An ability marked partial because its outside-battle narrative effect is excluded is not an unfinished battle feature by itself.
