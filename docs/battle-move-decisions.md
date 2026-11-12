# Saved move decisions

MOVE-A45 (app 0.88.0, engine 0.80.0-core, protocol 89) implements Hold Hands through the normal battle controls. Earlier encounters keep their saved completion blocks and do not silently adopt this rule.

After successful Accuracy and completed reactions, the source owner or GM prepares the target decision. Preparation resolves ordinary Support protection and reflection, then saves the actual recipient, Accuracy threshold and Will-score pool. A protected move resolves through the ordinary effect command without offering a battle-ending ruling.

The recipient owner or GM chooses whether to resist. Resistance rolls the Will Score, not remaining spendable Will Points, and spends no Will. Guided characters use their recorded maximum Will; legacy sheets derive Insight + 3. Equal or greater successes resist. The result and exact dice are saved and cannot be rerolled. Declining resistance is an explicit saved response, not an inferred timeout.

Only the GM can then continue or end the battle, with a recorded reason. Successful resistance prevents this move from ending the battle. Failed or declined resistance does not automatically end it. Ending uses the existing EndBattle transition for HP, conditions, temporary transformations, equipment settlement and finished-battle history. One rollback restores the unresolved ruling and its prior resources. Normal explicit GM overrides remain available separately.

The typed SpecialCommand variants are PrepareHoldHands, ResistHoldHands and RuleHoldHands. Each is bound to the current cast sequence; wrong owners, stale sequences, duplicate resistance and premature rulings are rejected. Save/load preserves all stages in TurnFlow.completion.hold_hands. No database schema change is needed.

Outside-battle reduction of hostile intent remains narrative. Automatic Will recovery, lethal wounds, Max/Z/Dynamax and Sappy Seed remain excluded.

Verification references: completion_a45 public-command tests and MoveDecisionControls.test.tsx. The source checklist records the implemented effect clauses separately from the remaining whole-catalog semantic review; this delivery does not certify all moves.

## Metronome (MOVE-A46)

App 0.89.0 / engine 0.81.0-core / protocol 90 adds a committed copied-cast workflow. The Metronome card requires category and Power dice but prints no category lookup table. The GM therefore records which die faces mean Physical before rolling; the other faces mean Special. No table is silently assumed. The saved table includes a reason and cannot change during a committed move.

The owner rolls two host dice before Accuracy. The category follows the saved table, and the second die face is base Power. The saved draw includes both faces and the exact matching catalog templates. The GM chooses a matching Physical/Special card with a reason; Unique, Max/Z and Sappy Seed templates are excluded from copying. A template whose mechanics are still pending elsewhere in the ledger remains visible: the engine never silently replaces or resamples it.

The source owner casts the selected template against one foe using Metronome's Insight + Perform. Normal template damage, added effects, protections, immunities, costs and prerequisites are dispatched through their existing handlers. Double/successive-action templates expose the normal attempt count and action budget. Charge casts and saved pending damage retain their copied context. The temporary template grant never modifies the permanent move list. Used-move bookkeeping, Gorilla Tactics and Choice-item comparisons retain the Metronome origin.

A pending draw prevents unrelated actions. Required environment, alignment, weight/night and Natural Gift berry-profile inputs, plus pending ability/field resolutions, remain available with their usual permissions. Invalid targets or missing prerequisites preserve the draw. The GM may record the attempt as failed, spending one action; the owner cannot discard it to reroll. Normal GM rollback restores the previous saved choice and resources.

The Metronome controls provide category-table setup, draw, exact template choice, target, successive count, Accuracy preview and failure recording. New SpecialCommand variants are MetronomeTable, DrawMetronome, SelectMetronome, CastMetronome and DiscardMetronome. Older encounters retain their existing completion blocks.

This delivers the saved resolution workflow and shared copied-cast foundation. Whole-card certification of Metronome and every eligible template remains subject to the exhaustive ledger and final copy-interaction audit; unsupported template clauses are not converted into discretionary GM rules.

## Judgment (MOVE-A47)

App 0.90.0 / engine 0.82.0-core / protocol 91. Before the cast, the GM prepares Judgment's Type, Power (0–100) and a saved ruling on Unique protection, with a reason, for the current turn. The prepared profile is frozen onto the cast at Accuracy, so an in-flight Judgment cannot be re-ruled and keeps its saved Type and Power even if the prepared ruling is later cleared.

Judgment ignores defenses and anything that would prevent its damage: damage immunities, shields, force fields, decoys, weather reductions, cover/redirection and ordinary damage-reducing abilities. It cannot be Clashed; ordinary Accuracy, Evasion and action eligibility still apply. Unique abilities, and abilities whose source text says they cannot be ignored (Shadow Shield), follow the saved GM ruling: preserved or bypassed.

After damage the GM either begins recording consequences, applies them through the normal battle controls (stat, condition, field, equipment, Will or combatant-state changes, each saved as a receipt and undoable with the normal rollback) and then finishes, or explicitly declares that there are no extra effects. Rejected or no-op commands do not count as consequences. The typed SpecialCommand variants are PrepareJudgment, BeginJudgmentEffects and RuleJudgment.

Verification references: completion_a47 public-command tests. The card's Unique clause and metadata remain under the whole-catalog review.

## Copy moves (MOVE-A59)

App 0.102.0 / engine 0.94.0-core / protocol 103. Copycat, Mirror Move, Instruct, Assist and Me First are cast with the "Cast copy" control (CastCopy), never as ordinary moves. The user names the source and the copied move and may choose new targets; the copied move then runs through the same copied-cast pipeline as Metronome, using its own Accuracy, damage and effects, spending one action and recording the copy move (not the template) as used.

- **Copycat, Mirror Move, Instruct:** copy the last Pokemon move the source performed this battle. Copycat and Mirror Move copy a foe; Instruct may also name an ally.
- **Assist:** copies any move known by an ally on the field or by a member of the user's party, active or not.
- **Me First:** during a reaction window, copies the incoming move as a Reaction 1. Support moves cannot be copied.
- Unique, Max/Z, Sappy Seed and other copy moves are never copied.
- **Mimic:** replaces itself with its target's last move for the scene (until the user leaves battle); the sheet is unchanged.
- **Sketch:** replaces itself on the user's sheet permanently, as printed.
- A move the user already knows is not copied again by Mimic or Sketch.

Verification references: completion_a59 public-command tests and MoveOptionControls.test.tsx.
