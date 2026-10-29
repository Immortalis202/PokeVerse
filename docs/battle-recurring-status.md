# Recurring and sleep-action moves

Delivered in app 0.40.0, engine 0.33.0-core, protocol 41. The six source cards and printed references are preserved in [the recurring registry](../rules/recurring-status-moves.json). New encounters enable `TurnFlow.recurring`; historical engines and saved flows default to disabled. State is serialized, host-validated and included in rollback.

| Move | Delivered behavior |
| --- | --- |
| Yawn | Applies Sleep at the beginning of cast round + 1, checking current condition immunities. Target departure cancels it even if the target returns; source departure does not. |
| Nightmare | Sleeping target required. Deals 1 indirect Typeless damage after each failed wake attempt; successful waking, cure or departure clears it. No round-end damage tick. |
| Perish Song | Normal Area targeting. At the end of cast round + 3, deals the target's remaining HP as indirect damage, currently blocked by Magic Guard/Wonder Guard. [M3-F02](M3-BATTLEFIELD-ICE-CHECKLIST.md#m3-f02) identifies Magic Guard blocking as a printed-source mismatch, since its card explicitly permits Perish Song damage. Individual departure cancels; source departure does not. Repeated application does not reset or stack the clock. |
| Malignant Chain | 5 Chance dice for Badly Poisoned; the printed Poison+ icon agrees with JSON despite abbreviated prose. One raw Loyalty check at the end of the infliction round needs 3 successes. Failure changes battle side next round until poison is cured. Ownership and library records stay unchanged; curing or ending battle restores allegiance. Repeated claims do not stack. |
| Snore | Asleep-only ordinary attack, 3 Chance dice for Flinch. Uses normal damage, chance modifiers and immunities. Does not make a wake attempt. |
| Sleep Talk | Asleep-only host draw over all known moves and available foes in range, followed by ordinary accuracy, reactions, damage and effects. May repeat up to five actions per round; does not advance wake progress or trigger Nightmare. |

The battle controls display pending clocks, Nightmare and Malignant Chain state. Owners or GM roll the round-end Loyalty check. Sleeping owners or GM can draw Sleep Talk, then roll the selected move's accuracy. Selection uses six host d6 with rejection sampling so every known move and eligible foe has equal probability. The chosen move and foe are committed before resolution; copies retain their intrinsic Self/Area targeting. The ordinary pipeline spends one action per attempt and applies normal action penalties.

Sleep Talk can draw itself or a move whose parameters/behavior need GM input. The card does not define recursive copying. Those selections stay committed; the GM records a reason and consumes the failed attempt. They are never silently excluded or freely rerolled. This remains an explicit retained boundary, not whole-catalog copied-move support. The random eligible foe is recorded even for intrinsic Self/Area moves.

Malignant Chain changes temporary battle allegiance, not player control. Its owner keeps issuing commands. The first unresolved poison claim is retained until cured; a passed Loyalty check clears its pending claim. Withdrawal does not itself cure Poison or allegiance. Historical encounters do not acquire these new mechanics.

Automated validation covers timing, departure, delayed immunity, failed and successful waking, poison/Loyalty branches, side restoration, sleep prerequisites, random distribution, ownership checks, committed selections, five-action limits, serialization and rollback. App packaging, native UI and physical network acceptance remain separate. Clippy was not run.
