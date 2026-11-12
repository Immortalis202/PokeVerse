# Allied contribution moves (MOVE-A53)

App 0.96.0 / engine 0.88.0-core / protocol 97. Older encounters keep their saved blocks and these cards stay manual there.

**Helping Hand.** The ally gains 2 extra dice on its next Accuracy roll and 2 extra dice on its next Physical or Special damage roll. A second Helping Hand before those rolls refreshes the bonus instead of stacking it.

**Help Another.** This maneuver is rolled with the "Roll help" control (HelpAnother), not cast as an ordinary move. The helper names the ally and the move the ally is about to attempt, spends one action and rolls that move's Accuracy pool using the helper's own Attributes and Skills, as Chance dice. Each 6 adds one die to the ally's next Accuracy roll; at most 3 helpers and 6 dice per action.

**Beat Up.** While a Beat Up's damage is pending, up to 3 different active allies may each spend an action with "join Beat Up" (JoinMove), adding 2 damage dice each.

**Round.** While a Round's damage is pending, an active ally that knows Round may join ("join Round"). The caster's Round and the ally's own Round this round each gain 3 damage dice.

Skill checks other than Accuracy are not changed. Verification references: completion_a53 public-command tests and MoveOptionControls.test.tsx.
