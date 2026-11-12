# Delayed and triggered moves (MOVE-A54)

App 0.97.0 / engine 0.89.0-core / protocol 98. Older encounters keep their saved blocks and these cards stay manual there.

**Future Sight and Doom Desire.** Accuracy and the damage roll happen as usual (their shield, force-field, Cover and decoy bypass is unchanged), but the damage is saved and dealt when turns start in the next round. If the original target has left battle, the combatant now in its field slot takes it; otherwise an active ally on that side does. Doom Desire's "extra effects at Storyteller's discretion" stay with the GM.

**Rage.** For the rest of the round in which Rage hits, each Physical or Special hit from a foe raises the user's Strength by 1, stacking to +3.

**Destiny Bond.** If the user faints to Physical or Special damage in the round Destiny Bond was used, the attacker's HP drops to 0. Using it in two consecutive rounds is refused.

**Pursuit.** A target Pursuit damages is marked. The first time it is recalled, switched out (including by a Switcher move) or uses Run Away, it takes 2 typeless damage. The outside-battle tracking clause is narrative.

**Glaive Rush.** Its user becomes Vulnerable (the existing +2 damage and barrier/Cover bypass) until the next Physical or Special move hits it, or it leaves battle.

Verification references: completion_a54 public-command tests.

# Trigger moves (MOVE-A55)

App 0.98.0 / engine 0.90.0-core / protocol 99.

**Shell Trap.** It cannot be declared as an ordinary action; it is a Late Reaction to a non-ranged Physical attack aimed at its user. Its damage can be rolled only after that attack has damaged the user this round; if it missed, the damage step refuses and the GM cancels it.

**Powder.** The target stays Powdered until it leaves battle. When it next declares a Fire-type move, it owes typeless damage dice equal to that move's Power (defenses ignored), rolled with the "Roll Powder damage" control (ResolvePowder). The Fire move itself still proceeds.

**Curse.** The user rolls half its current HP as typeless self-damage dice. The target loses one success on its Accuracy rolls (through the shared strongest-loss channel) and on its damage rolls, for the rest of the battle.

**Flame Burst.** After it deals damage, the caster's owner may save up to two other foes ("none" by default). Each rolls one Fire-type die: a success deals 1 damage, adjusted by the type matchup; defenses are ignored.

Verification references: completion_a55 public-command tests and MoveOptionControls.test.tsx.
