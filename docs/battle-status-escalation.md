# Burn and Poison escalation

Application **0.34.0**, engine **0.27.0-core**, protocol **35**, schema **4**.

## Source and selected precedence

Reviewed supplied POKEROLE COREBOOK 3.0 RULING, printed/PDF pages 84-86. SHA256: `f50014b330cc00db83906cf232ba432855f4f624684a84c3c494b47390284d74`.

The detailed ailment entries on pages 84-85 increase third-degree Burn and Badly Poisoned by **2 per round**. The illustrated guide on page 86 says **1 per round**. On 27 October 2026 the user explicitly selected the detailed entries. This is a recorded precedence decision, not a claim that the book is consistent.

| Condition | End-of-round damage | Burn treatment |
| --- | --- | --- |
| Burn | 1 each round | 4 cumulative Dexterity + Athletic successes |
| Burn 2 | 2 each round | 6 cumulative successes |
| Burn 3 | 3, 5, 7, 9, ... | 8 cumulative successes |
| Poison | 2 each round | Existing supported cures |
| Badly Poisoned | 2, 4, 6, 8, ... | Existing supported cures |

Only severe conditions accrue an escalation age. The first end-of-round tick after reaching the severe degree uses its starting value. The counter advances once per round while the condition persists, including rounds where an ability prevents its damage. Removing that ability exposes the damage for the elapsed round; it does not rewind time. Treatment successes and escalation age are independent.

Repeat infliction promotes Burn through its degrees and Poison to Badly Poisoned. Reapplying an already severe condition preserves its age. Cumulative Burn treatment remains accumulated when its degree changes. A directly applied third-degree Burn cannot be downgraded by an existing lower-degree Burn. A cure, finite GM duration expiry or faint clears the condition and its counter; a later application starts again. Switching alone does not clear it. Existing type/ability immunities, Poison Heal, Ring Target exceptions, source bypasses and supported cures retain their reviewed paths. There is no new arbitrary damage cap; arithmetic saturates safely at the serialized integer limit.

New encounters snapshot `TurnFlow.status_escalation = true`; historical/omitted flags stay false and retain the previous explicit GM severe-status command. New battles reject that command to prevent a second status debit; a deliberate GM exception uses the existing explicit HP correction. The battle display shows the upcoming severe damage before immunity. Host validation applies damage once during Next Round, together with other end-of-round effects. Serialization, receipts and rollback include each condition's `elapsed_rounds`, whose omitted historical default is zero. Upgrade campaigns between battles and use matching clients.

Scope remains the battle engine. Wild/training encounters and outside-battle clocks are unchanged. Optional lethal wounds, death, fainted hourly damage and new recovery workflows are excluded; HP stops at zero under the existing engine boundary. Shadow Force cross-battle recovery remains intentionally skipped.

## Verification

Focused regressions cover damage sequences, separate condition ages, fixed ailments, reinfliction, cumulative 4/6/8 treatment, expiry/faint/cure, immunity changes, Poison Heal, integer saturation, historical saves/gates, ownership, double-debit rejection and rollback. A localhost-room regression checks a remote GM command, duplicate receipts, reconnect, SQLite persistence and rollback. Windows validation passed **693 Rust tests**, **64 frontend tests** and **22 catalog/script checks**, plus TypeScript, warnings-denied Clippy, Rust formatting and diff checks. The Windows installer and Linux Debian/AppImage packages were built before the user changed the standing build preference; both extracted Linux catalogs match Windows and Linux shared libraries resolve. Future app builds and WSL packaging are opt-in, as recorded in `AGENTS.md`. Native GUI and physical Tailscale play remain manual acceptance checks.
