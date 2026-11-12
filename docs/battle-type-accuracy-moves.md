# Typing, accuracy and targeting moves (MOVE-A49)

App 0.92.0 / engine 0.84.0-core / protocol 93. Older encounters keep their saved blocks and these cards stay manual there.

**Typing.** Conversion rolls two host dice; the 36 face pairs map evenly onto the 18 types, and the result replaces the user's main type while any different secondary type is kept. Reflect Type copies all current types of a foe; with one active foe it copies that foe, otherwise the caster's owner or the GM saves the foe with the "Save choice" control (ChooseMoveOption) first. Both changes last for the scene or until the user leaves battle.

**Accuracy.** Lock-On adds two automatic successes to the user's next move roll, including reaction moves, and is then spent. Mind Reader marks one target: the user's next Accuracy roll or Evasion roll against that target gains two successes added after every penalty, then the mark is spent; if both apply they add. Miracle Eye lasts for the scene: the user's Psychic moves treat Dark targets as neutral, foes cannot Evade the user's moves, and move, item and ability Accuracy reductions on the user are ignored. Printed Low Accuracy and field effects still apply.

**Targeting and damage.** Snipe Shot ignores Lightning Rod/Storm Drain/lure redirection and cover. Moongeist Beam and Sunsteel Strike ignore type and ability immunities, shields, weather that would prevent the move and force-field reductions, and cannot be Clashed. Defenses, cover, decoys and survival effects still apply, as do damage-reducing (not preventing) abilities. Sunsteel Strike's source metadata says "Foe's Battlefield"; its printed "Single Target." is applied instead.

Marks end when their source or target leaves battle. Verification references: completion_a49 public-command tests.

# Damage-calculation moves (MOVE-A50)

App 0.93.0 / engine 0.85.0-core / protocol 94.

**Saved choices before damage.** Hidden Power's printed Type is "Varies": after Accuracy, the caster's owner (or the GM) saves one of the 18 types with "Save choice". Photon Geyser, Tera Blast and Tera Starstorm print a "Physical/Special" category: the owner saves Physical (Strength against Defense) or Special (Special against Sp. Defense). Damage is refused with a clear message until the choice is saved.

**Damage rules.** Hidden Power is always super-effective, even against a type that would be immune; Unown may repeat it in the same round (corebook p.530). Flying Press adds the Flying matchup to its Fighting one. Foul Play's pool uses the target's higher Strength/Special and the user's STAB. Techno Blast becomes Fire, Ice, Water or Electric when a Genesect holds a Burn, Chill, Douse or Shock Drive; the current catalog has no Drive items, so this never triggers yet.

**Typing.** Terastallization and Tera Types are not modelled, so Tera Blast and Tera Starstorm never change type. Tera Starstorm still rolls two host dice per damaged target and replaces that target's typing with one of the 18 types (evenly). Conversion 2 marks the user for the scene; whenever Physical or Special damage reaches it, its owner may pick a type that resists (or is immune to) that move's type with the "Change type" control (ConvertType). The main type changes and the secondary is kept; one choice per hit.

Verification references: completion_a50 public-command tests and MoveOptionControls.test.tsx.
