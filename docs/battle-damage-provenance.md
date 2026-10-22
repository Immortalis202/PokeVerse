# Damage-provenance abilities

Source evidence is the supplied Pokerole 3.0 ability cards in `Pokerole-Data/v3.0/Abilities/`. Engine 0.16.0-core enables `TurnFlow.provenance_rules` for new battles. The saved flag defaults to false for historical encounters; the 0.15 sequencing engine remains supported. Application 0.23.0 uses protocol 24 and schema 4.

## Tinted Lens

Ignore one individual type resistance to the user's move against a foe. Two resistances become one; a resistance paired with a weakness becomes a weakness. Type immunity is preserved. The shared type calculation also serves reviewed fixed/varied formulas and Clash.

## Unaware

Ignore increases and reductions to the foe's Attributes when dealing or receiving damage. Damage uses permanent sheet attributes plus explicit GM overrides, before battle stat changes. The holder's own attribute changes remain effective. Explicit Defense/Special Defense channels are retained: these are distinct from the Vitality/Insight attribute contributions. Rank defense bonuses, weather defenses, Wonder Room's selected attribute, and damage reductions remain effective. This ability does not alter Accuracy or rewrite either combatant's sheet.

Shared damage freezes both effective attacker attributes and the baseline attributes needed by an Unaware defender. Field snapshots retain both ordinary defenses and defenses derived from neutral attributes, avoiding subtraction from an already clamped defense score. Existing saves default the new snapshot fields to empty collections.

## Stakeout

A deployed holder witnesses a foe replacing an occupied active slot. Record that incoming foe as eligible for one extra damage on the holder's first successful attack against it. Each holder and incoming foe pair is independent. The marker is consumed by a nonimmune successful damage resolution, including a hit intercepted by a decoy; misses preserve it. Withdrawal, fainting, battle end and loss of the ability remove unneeded eligibility. Frozen attack ability identity remains authoritative while shared damage is pending.

Normal damage, reviewed fixed/varied formulas and immediate/deferred Clash apply the bonus. Deferred Clash comparisons include the bonus before choosing the strongest counter. An attack on an ice block is not an attack on the incoming foe and preserves the marker. A frozen shared attack retains its unconsumed target markers if a hit reaction faints or changes the source.

## Sheer Force

The owner or GM can choose between retaining added effects and trading them for two extra damage dice before declaring an action. The choice spends no battle action and is logged. Changing it while an attack or ability trigger is pending is rejected. The choice is saved on the queued attack, including a stored charge, and damage previews show its effect. The UI exposes this choice only to the holder's owner and GM.

Eligibility currently uses the supplied card's nonempty `AddedEffects` object and Physical/Special category. Move attributes such as recoil, double action, accuracy and charge are not erased. Choosing the tradeoff skips the move's added-effect resolution after damage; opponent abilities and hit-history recording remain active. Moves without added effects receive no bonus. Held-item added effects are a dependency of the following held-item milestone and must not be silently assumed available now.

The focused tests cover ownership, no action cost, optionality, two extra ordinary damage dice, suppression of Ember's burn, rejection of mid-action changes, no bonus on Tackle, shared targets after source identity changes, successive Twineedle hits, both charge-choice directions through Bounce release, and retaining Volt Tackle recoil while suppressing its paralysis chance. The room fixture covers separate host/GM/owner roles, duplicate and stale submissions, reconnect, SQLite persistence and rollback of the choice.

The staged held-item engine additionally recognizes matching King's Rock (Special) and Razor Fang (Physical) chance effects when tagging Sheer Force eligibility. Item-only and combined move/item added effects award the same two extra dice once; traded Flinch effects do not roll. Historical item-disabled battles retain card-only eligibility. The item engine also preserves Life Orb/native Recoil separately from traded move added effects. These held-item rules remain disabled in ordinary new encounters until their milestone is released; see [held-item integration](battle-held-items.md).

## Remaining integration

Held-item added effects belong to the following held-item milestone. Remaining event/accuracy definitions, including target-specific Tangled Feet penalties, remain in the roadmap before that milestone. Existing contradictory/unspecified printed rules remain explicit GM decisions.
