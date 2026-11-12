# Scene and consequence moves (MOVE-A58)

App 0.101.0 / engine 0.93.0-core / protocol 102. Older encounters keep their saved blocks and these cards stay manual there.

**Memento.** The user faints, even if its Accuracy roll fails. The target's Strength, Dexterity, Special, Defense and Sp. Defense drop by 2 through ordinary stat protection (Clear Body, Mist and similar still apply). The reductions that took effect persist for the scene, even after the target switches out and back.

**Springtide Storm.** The caster's owner saves "Raise the user" (+1 Strength, Dexterity, Special, Defense and Sp. Defense) or "Lower the targets" (Defense and Sp. Defense −2). Three Chance dice, plus Serene Grace and Water Pledge, apply the choice on any 6.

**Synchronoise.** Before damage the owner saves one type; targets without that type are unaffected.

**Nature Power and Camouflage.** The GM records the surroundings with the Secret Power environment control; active terrains are added automatically. Each source maps to the printed energy types (corebook p.530: City/Building Electric or Steel, Plains/Grasslands Grass, Forest/Jungle Bug or Poison, Lake/River/Sea Water, Cave/Mountain Fight or Rock, Desert/Volcano Fire or Ground, Ice/Snow Ice, Electric/Grassy/Misty/Psychic Terrain Electric/Grass/Fairy/Psychic). With one possible type it is used directly; otherwise the owner saves one. Nature Power takes that type; Camouflage replaces the user's typing with it for the scene.

**Pay Day and Make It Rain.** Money by trainer rank (Starter/Rookie 100, Standard 250, Advanced 500, Ace or more 1000): Pay Day earns it for the user's trainer on each hit; Make It Rain once for the user's trainer and for the trainer of each Pokemon it hits, and lowers the user's Special by 2. The app does not track money, so the totals appear in the battle-end log for the GM to record.

**Fix.** Ending a battle while any combatant had a stat change could crash, because the special-move state was read through the active-battle check after the battle ended. The state is now read from the saved flow.

Verification references: completion_a58 public-command tests.
