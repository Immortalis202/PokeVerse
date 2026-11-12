# Typing, accuracy and targeting moves (MOVE-A49)

App 0.92.0 / engine 0.84.0-core / protocol 93. Older encounters keep their saved blocks and these cards stay manual there.

**Typing.** Conversion rolls two host dice; the 36 face pairs map evenly onto the 18 types, and the result replaces the user's main type while any different secondary type is kept. Reflect Type copies all current types of a foe; with one active foe it copies that foe, otherwise the caster's owner or the GM saves the foe with the "Save choice" control (ChooseMoveOption) first. Both changes last for the scene or until the user leaves battle.

**Accuracy.** Lock-On adds two automatic successes to the user's next move roll, including reaction moves, and is then spent. Mind Reader marks one target: the user's next Accuracy roll or Evasion roll against that target gains two successes added after every penalty, then the mark is spent; if both apply they add. Miracle Eye lasts for the scene: the user's Psychic moves treat Dark targets as neutral, foes cannot Evade the user's moves, and move, item and ability Accuracy reductions on the user are ignored. Printed Low Accuracy and field effects still apply.

**Targeting and damage.** Snipe Shot ignores Lightning Rod/Storm Drain/lure redirection and cover. Moongeist Beam and Sunsteel Strike ignore type and ability immunities, shields, weather that would prevent the move and force-field reductions, and cannot be Clashed. Defenses, cover, decoys and survival effects still apply, as do damage-reducing (not preventing) abilities. Sunsteel Strike's source metadata says "Foe's Battlefield"; its printed "Single Target." is applied instead.

Marks end when their source or target leaves battle. Verification references: completion_a49 public-command tests.
