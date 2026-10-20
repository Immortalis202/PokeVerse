# Contact and fainted-ally ability transfers — 0.19.0

Engine `0.12.0-core`, protocol 20, user database schema 4. Adopt rules/catalog between battles; matching client builds are required. Earlier engines retain their behavior. New campaign, encounter and rollback fields have serialized defaults; there is no SQL migration or permanent sheet-ability change.

Reviewed POKEDEX MASTER printed sources: general ability rules p.534, Lingering Aroma p.553, Mummy p.557, Power of Alchemy p.560, Receiver p.563 and Wandering Spirit p.577.

| Ability | Supported trigger |
| --- | --- |
| Mummy | The holder hits or is hit by a foe with a non-ranged Physical move; the foe becomes Mummy. |
| Lingering Aroma | The same two contact directions; the foe becomes Lingering Aroma. |
| Wandering Spirit | The holder hits a foe with a non-ranged Physical move and exchanges ability identities. Receiving an attack alone does not trigger it. |
| Receiver | A deployed ally newly faints while Receiver is active. Its owner chooses one eligible donor if several faint simultaneously. Only one unexpired copy may be held. |
| Power of Alchemy | Deployed allies newly faint while Power of Alchemy is active. Eligible donor snapshots can be stored together; the owner may activate one or keep Power of Alchemy active. |

Contact changes respect type/ability immunities, Long Reach, decoys, suppression and unique-ability restrictions. Damage and contact reuse the same type-immunity calculation. Successful hit callbacks cover ordinary, fixed/formula and Clash damage; a newly gained ability does not recursively trigger on that same hit. Independent incoming/outgoing contamination uses the pre-contact identities. Static, Rough Skin and supported contact-poison reactions already triggered by that hit remain resolvable after the contact replacement; newly acquired abilities do not gain a retroactive trigger. An outgoing Wandering Spirit hitting Mummy/Aroma produces the same exchange without applying a second contradictory chain.

Contact assignments use a separate encounter layer. Following the general battle removal rule, recall, fainting or scene end removes that layer and restores any underlying scene assignment or inherited copy. Later named replacement moves supersede the contact layer. Permanent abilities, available species abilities and stored allocation points remain unchanged. Narrative curses beyond the encounter remain GM-adjudicated.

Inheritance snapshots the ally's ability identity at fainting, including a prior contact or scene assignment, before cleanup removes that temporary effect. Foes, reserves already fainted before entry and unique donor abilities are excluded. Missing donor metadata creates an explicit GM review event without cancelling the damage. Pending inheritance cannot be resolved by another owner, retriggered by repeated lifecycle calls, or accepted after the recipient leaves/faints.

Copies live in a campaign ledger, identified by recipient/donor sheet IDs, source ability, battle, acquisition minute and expiry minute. They last exactly **1440 game minutes**. This is game time, not computer wall time. The GM advances elapsed minutes between battles; application closure and reconnect do not consume or reset time. Clock corrections require a logged GM reason; expired copies can be reselected if a correction restores their validity. Expired records remain as provenance, while their active selection is removed. A selected unexpired copy is reapplied when the Pokémon enters a later encounter.

Only one copy is active. Owners select existing copies between encounters. During combat the GM must explicitly confirm selection timing, between action chains and unresolved choices. This is a logged timing ruling, not an invented automatic action cost. Acquisition requires Receiver/Power of Alchemy to be the currently active ability; keeping Power of Alchemy active allows further collection. Selecting another copy does not leave a second ability mechanically active.

The Battle page includes inherited-ability selection, donor provenance, remaining game time, a GM clock control and contact/inheritance history. Original sheets remain usable offline; these temporary effects stay campaign-scoped. Creation/progression validation, Evasion once per turn, repeated eligible Clash and automatic host validation are unchanged.

Rollback restores the campaign inheritance ledger together with the battle and resources. Host receipts and expected revisions prevent duplicate acquisition or duplicate clock advancement. Subsequent ability edits outside battle invalidate the prior battle undo stack so a late battle rollback cannot erase newer campaign time or selection changes. The new strict four-installation 3v3 scenario uses ordinary contact and gym damage, Receiver acquisition, reconnect, rollback, a second battle with the same copy, expiry and reopening the host database. It runs without UI or library data from the user's installation.

Receiver and Power of Alchemy remain marked partial for narrative acquisitions outside witnessed battle fainting; mid-combat selection timing also requires a GM ruling. Copied abilities retain their own coverage limits. Entrainment, Doodle, Core Enforcer, Mold Breaker/Teravolt/Turboblaze bypass, held-item systems and transformations remain separate work. Dynamax, Max Moves, Z-Moves, trainer-target damage, lethal wounds and automatic Will recovery remain out of scope.

Validation: 351 Rust tests, 28 frontend tests and 12 catalog checks passed, including the two-battle four-installation scenario. TypeScript, production build and Clippy all targets with warnings denied passed. Native interactive and physical LAN use remain unverified; no app or installer is automatically launched.

The 0.20.0 [reviewed ability batch](battle-reviewed-ability-batch.md) implements the next bypass scopes; the remaining-work list above describes the historical 0.19.0 milestone. Use [the current backlog](REMAINING-WORK.md) for unfinished work.
