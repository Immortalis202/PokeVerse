# History and condition moves (MOVE-A51)

App 0.94.0 / engine 0.86.0-core / protocol 95. Older encounters keep their saved blocks and these cards stay manual there.

**Use guards (checked before Accuracy).** Belch needs the user to have eaten a berry this round (recorded berry consumption or a berry hit). Last Resort needs every other known move to have succeeded an Accuracy roll earlier in this battle; the refusal lists what is missing. Poltergeist needs the target to hold an item, or an item to have been consumed, used as medicine, recycled or harvested on the target's side. Ruination is refused against a target it already damaged this battle.

**Damage.** Ruination rolls dice equal to the target's total HP, ignoring defenses (type effectiveness still applies). Lash Out adds 3 dice if one of the user's Attributes or Traits was lowered this round; Revenge adds 2 dice if the user lost HP earlier this round; Temper Flare and Stomping Tantrum add 2 dice when the user's previous move failed its Accuracy roll. Stomping Tantrum already hits a target using Dig. Fell Stinger raises the user's Strength by 2 when its damage faints the target; the engine never lets a move target an already fainted Pokemon, so that half of the card cannot occur.

**Missing social points.** Belch, Lovely Kiss and Chilly Reception print Accuracy pools such as "MissingBeautyPoints": the pool counts the social points given up, which is the social cap (5, or 7 for Champions) minus the current score. The permanent forfeit/restore clauses happen outside battle. Chilly Reception could not be cast before this block; it now also activates Snowy Weather for 4 rounds before its Switcher effect (previously the weather was silently skipped).

The engine tracks, per battle: moves that succeeded their Accuracy roll, the last Accuracy result, the round of the last reduction and the last HP loss, and Ruination's targets. Verification references: completion_a51 public-command tests.
