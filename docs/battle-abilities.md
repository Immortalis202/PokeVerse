# Battle abilities — 0.14.0

Engine `0.7.0-core`, protocol 15, user schema 4. This follows the separately committed 0.13.0 battlefield milestone. Source PDFs/JSON remain unchanged; definitions and corrections belong to the application.

## Coverage

Of 305 catalog abilities, **207 have automated or partial scope** and **98 remain manual**. The authoritative [registry](../rules/ability-coverage.json) gives each definition its printed PDF page, exact implemented scope, dependencies and introduction engine. The generated `outputs/rule-coverage/ability-coverage.json` inventories all 305. Supporting one part of an ability does not imply every interaction is implemented.

| Family | Examples and implemented behavior |
| --- | --- |
| Conditional attributes | Guts, Flare Boost, Toxic Boost, Quick Feet, Berserk, Anger Shell, Slow Start; separate move and ability channels |
| Sourced stat effects | Clear Body, Flower Veil, Mirror Armor, Contrary, Simple, Defiant, Competitive and Guard Dog; GM source/target/stat/reason controls |
| Damage and immunities | Typed/tagged bonuses, Bulletproof, Fluffy, Ice Scales, Prism Armor, Shadow Shield, Wonder Guard, Corrosion and Neuroforce |
| Encounter typing | Aerilate, Pixilate, Refrigerate, Galvanize, Normalize, Liquid Voice, Libero, Protean, Color Change and Mimicry |
| Ally/range effects | Battery, Power Spot, Steely Spirit, protective Veils, Dark/Fairy Aura and Ruin abilities; duplicate contributions do not stack |
| Hit and knockout | Anger Point, Stamina, Weak Armor, Gooey, Cursed Body, Poison Touch/Toxic Chain, Innards Out, Beast Boost and Neigh/Soul Heart boosts |
| Entry and order | Download, Hospitality, Curious Medicine, Supreme Overlord, Dauntless Shield, Intrepid Sword, Quick Draw and optional No Guard |
| Round/status | Natural Cure, Shed Skin, Healer, Poison Heal, Bad Dreams, Comatose, Serene Grace and Wonder Skin |
| Field-dependent abilities | Weather entry/contests/suppression, Surge abilities, Forecast, weather attributes/healing and protections; see [battlefield](battlefield.md) |

Reviewed Growl, Tail Whip, Leer, Charm, Swords Dance, Nasty Plot, Calm Mind, Iron Defense, Amnesia and Agility use sourced stat changes. Other stat effects can use the GM's recorded sourced operation. Ordinary damage remains automatic.

## Host validation and choices

The host validates and rolls automatically. The appointed GM may be a separate client. Owners resolve their own optional/chance ability events; Download's attribute ruling requires the GM. A separate host has no GM decision rights. Pending events stop action and round advancement until resolved.

Natural Cure uses the manual's three chance dice at round end. Shed Skin first rolls for activation and then independently selects one condition with unbiased dice. Healer chooses an ally before its chance roll. No Guard is an owner declaration each round; accepting removes Low Accuracy and prevents that user's Shields, Evasion and Clash for that round. The table's ordinary once-per-turn Evasion and repeatable Clash rules remain intact.

At round end, status/ability/field ticks are recorded once before event choices and duration expiry. Resolving an event and retrying round advancement cannot repeat poison damage or healing. Healing never revives a fainted participant. Poison Heal cures after three healing rounds. Mega Launcher's reviewed Heal Pulse uses Complete Healing (up to five HP) and one Will.

## State, snapshots and recovery

Ability selection is frozen for the encounter. Temporary attributes, typing, counters, source identities, choices and fainted-ally provenance are serialized with battle state. Permanent sheets and species baselines are unchanged. Color Change removes its own typing layer on recall, preserving underlying scene effects.

Move modifiers and ability modifiers use separate channels. Duplicate ability bonuses do not compound; positive and negative ability channels combine according to the reviewed rules. Shared multi-target damage freezes attack stats, counters, field state, types, defenses and ability sources before its first damage roll. A source fainting mid-sequence cannot change already committed math. Reconnect and rollback retain these snapshots, pending events and existing receipt/revision safeguards.

## Remaining mechanics

The inventory classifies every manual definition. Some require new held-item, form transformation, ability transfer/suppression, redirection, repeated-action, daily activity or hidden-information systems. Others still need an individual reviewed event/reaction definition and fixtures even where an underlying hook exists; they are deliberately listed as unfinished rather than labeled automated.

Partial boundaries are explicit: Infiltrator does not model Substitute; Triage does not add multi-target reaction sequencing; Hospitality does not model lethal wounds; Dark/Fairy Aura's narrative cooperation restrictions need a ruling; unreviewed recoil/healing/chance moves remain adjudicated. Torrent retains the text/icon discrepancy. Strict rolls with known suppression/bypass abilities retain the existing explicit GM adjudication path. Regenerator is an out-of-combat daily recovery ability in this manual, not automatic switch healing.

## Compatibility and verification

New campaigns use engine 0.7.0-core. Existing campaigns explicitly adopt rules/catalog between battles; historical engines retain their recorded feature flags. Matching 0.14.0 builds, protocol 15 and catalog hashes are required. Schema remains 4 with backward-compatible serialized defaults and existing database backups.

Fixtures cover damage, typing, modifiers, immunities, chance selectors, owner/GM authority, round staging, entry idempotency, serialization and preservation of permanent sheets across all supported definitions. Existing room receipts, reconnect, rollback and battle regressions remain required. Native startup checks use isolated profiles. Interactive native battle controls, installer install/uninstall and physical multi-machine LAN verification remain separate acceptance work.

Release checks: 216 Rust tests, 22 frontend tests and 9 catalog checks passed. TypeScript, Clippy with code warnings denied and the production build passed. The Windows NSIS installer was built. Two isolated native instances started, and their library/campaign databases passed integrity checks at schema 4. These checks do not establish interactive battle or physical LAN acceptance.
