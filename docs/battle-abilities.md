# Battle abilities — 0.12.0

Engine `0.5.0-core`, protocol 13, user schema 4. The source PDF and JSON files remain unchanged. Reviewed effect text and icons are recorded through application-owned evidence and `rules/ability-coverage.json`.

## Supported scope

The registry covers 51 automated or partially automated abilities from the 305-entry ability catalog. The generated `outputs/rule-coverage/ability-coverage.json` lists every ability, including the 254 that remain manual. It does not claim complete ability coverage or that every interaction of a supported ability is automated.

| Group | Abilities |
| --- | --- |
| Existing shared effects | Huge Power, Pure Power, Blaze, Overgrow, Swarm, Thick Fat, Own Tempo, Oblivious |
| Accuracy/damage pools | Adaptability, Technician, Iron Fist, Hustle, Sniper, Battle Armor, Shell Armor |
| Defensive modifiers | Filter, Solid Rock, Multiscale, Fur Coat, Marvel Scale |
| Move immunity/absorption | Levitate, Soundproof, Flash Fire, Sap Sipper, Water Absorb, Volt Absorb |
| Conditions | Limber, Insomnia, Vital Spirit, Water Veil, Magma Armor, Inner Focus, Immunity, Shield Dust, Guts |
| Contact/faint reactions | Static, Poison Point, Flame Body, Cute Charm, Effect Spore, Rough Skin, Iron Barbs, Aftermath, Sturdy |
| Entry/round/knockout | Intimidate, Speed Boost, Moxie |
| Status propagation | Synchronize |
| Cross-ability interactions | Damp, Long Reach, Magic Guard |

Exact costs, triggers and limits are in the registry and Compendium coverage notices. For example, Immunity prevents poison **damage**, while allowing the poisoned condition. Marvel Scale changes Defense, leaving Vitality, HP and Special Defense untouched. Hustle removes one accuracy success from Physical moves and adds two damage dice; it does not add an accuracy die.

## Owner decisions and automatic host validation

Passive effects apply through the shared Rust calculations. Contact chance effects and optional choices appear in **Abilities** in the battle workspace. The owner or GM resolves the first pending trigger; the host calculates and rolls its dice automatically. The separate host player has no GM decision rights.

- Static, Poison Point, Flame Body and Cute Charm use three chance dice, succeeding on a six. Effect Spore uses those three dice plus an independent d6 selector with two faces per possible status.
- Rough Skin and Iron Barbs offer one neutral damage or two neutral damage dice. They do not activate Sturdy, since the damage comes from an ability.
- Water Absorb and Volt Absorb offer an explicit one-HP heal or a decline. These ability heals spend no action, do not revive, and are outside the three-HP cap specifically stated for healing moves/items.
- Pending triggers block turn changes, further damage/effects and round advancement. GM corrections and logged manual ability resolution remain available. Ability rolls do not consume actions, Will or the ordinary reaction slot and are not modified by Taking Chances/Pushing Fate.

Failed contact rolls, optional declines and manual rulings remain recorded. Stable command receipts prevent retries or reconnects from applying an effect twice. Rollback restores pending choices, HP, conditions, flags and counters without rewinding campaign revisions or deleting receipts.

## Lifetime, provenance and stacking

Temporary ability stat bonuses have their own channel, separate from move/item modifiers and stored allocations. Duplicate Intimidate sources do not stack. Opposing positive/negative ability changes cancel, and move/item bonuses still combine with the ability channel. Temporary stat counters clear on recall; scene-use flags such as Sturdy and Flash Fire remain recorded. Flash Fire's bonus lasts to scene end as its card states. Sap Sipper's temporary Strength clears on recall while its first-hit scene flag remains used.

Intimidate is applied on entry, using rank caps and confirmed range. With no range ruling, all deployed foes are eligible and stable IDs break excess-target ties. A GM range correction updates those targets. Its reduction disappears when the source faints or leaves. Inner Focus ignores it. Speed Boost and Moxie cap at three temporary points. Ability selection remains frozen for the encounter.

Multi-target attacks snapshot the attacker's damage stats, HP/pain and relevant state before their shared damage rolls. A knockout boost or contact retaliation after the first target therefore cannot alter an attack already underway. Shared damage dice and the snapshot survive reconnect and rollback.

Synchronize requires an inflictor. Reviewed move/contact effects supply it automatically. GM status controls offer an optional source selector. A correction with no source does not invent an attacker. Reflection respects species/ability immunities, applies once and does not recursively bounce between Synchronize users.

## Manual boundaries

Weather/terrain, entry hazards, held-item abilities, form transformations, ability copying/swapping/suppression, complex redirection and unreviewed move effects still need GM adjudication. Levitate covers Ground moves here; its field immunity awaits field mechanics. Oblivious covers In Love; other emotional/narrative effects remain manual. Torrent's text/icon bonus conflict remains unresolved.

Known bypass/suppression interactions such as Mold Breaker, Teravolt, Turboblaze and an active Neutralizing Gas prevent strict ordinary rolls from silently assuming the standard ability interaction. The GM must adjudicate them explicitly. Full suppression semantics are not implemented. GM manual mode, resource/stat overrides and recorded resolutions remain available.

Sturdy applies once per scene to supported ordinary move/Clash damage. Indirect damage does not activate it. Reviewed set damage retains its existing defense/ability bypass. Contact triggers require a successful non-ranged Physical hit, including eligible successful Clash interactions; misses and Evasion create no contact trigger.

## Upgrade and verification

Existing campaigns retain engines `0.4.0-core`, `0.3.0-core` or older until the GM upgrades between battles. Historical ability state is defaulted when absent. All clients need matching 0.12.0 builds, protocol 13 and catalog fingerprints; existing campaigns explicitly adopt the new catalog. No SQL migration is required, and existing backup/restore behavior is retained.

Regression fixtures cover modifiers, immunities, chance success/failure, optional healing, source tracking, stacking, first-hit flags, action budgets, shared damage snapshots, persistence, receipts, rollback and a separate host/GM/owner reconnect. Interactive native battle workflows and physical multi-machine LAN remain separate acceptance work.
