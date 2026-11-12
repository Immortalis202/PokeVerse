# Initiative, reaction and maneuver moves (MOVE-A52)

App 0.95.0 / engine 0.87.0-core / protocol 96. Older encounters keep their saved blocks and these cards stay manual there.

**Initiative.** After You moves its ally to the front and Quash moves its target to the back of the initiative order, from the next time turns start, for the rest of the scene. Within each group the ordinary order (initiative, Trick Room, ability groups) is kept. Each user has one mark of each kind; recasting moves it to the new target. Quash persists across switching; After You ends when its target leaves battle.

**Ambush.** Ambush can be cast only before the first round's turns start. The target resists by rolling Insight + Alert and needs at least the user's Accuracy successes. If it does not resist, the user declares one surprise action before turns start; the Ambushed target cannot Evade or Clash it, but other foes can. When turns start, those pre-initiative actions are refunded.

**Encore.** The target must have acted in the previous round. For the rest of the round Encore is cast, the target's moves must repeat that round's moves in order and on the same targets (while they remain in battle); other moves, and anything beyond the recorded sequence, are refused. Counting includes moves it already made this round (corebook p.529). Repeated Evasions and Clashes are not forced. Encore cannot be used by the same user in consecutive rounds.

**Sucker Punch and Thunderclap.** They cannot be declared as ordinary actions and only react against a Physical or Special move. By default they react early (Reaction 6); the owner can switch a combatant to late (Late Reaction 1) with the "React late" control (ReactionTiming).

**Grapple.** The target resists with its better of Strength and Dexterity, needing at least the user's successes. Otherwise the target is Blocked and the user cannot act or react until it presses "Release Grapple" (ReleaseGrapple, no action cost), or either combatant leaves.

**Evasion maneuvers.** Double Team and Minimize let the user Evade beyond the usual once-per-turn reaction slot, up to five Evasions per round, for the scene. Minimize also halves the user's weight and makes it vulnerable to Stomp. Chip Away removes those evasion effects from a target it damages; Sand Veil has no evasion effect in the engine.

Verification references: completion_a52 public-command tests and MoveOptionControls.test.tsx.
