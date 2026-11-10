# Saved move decisions

MOVE-A45 (app 0.88.0, engine 0.80.0-core, protocol 89) implements Hold Hands through the normal battle controls. Earlier encounters keep their saved completion blocks and do not silently adopt this rule.

After successful Accuracy and completed reactions, the source owner or GM prepares the target decision. Preparation resolves ordinary Support protection and reflection, then saves the actual recipient, Accuracy threshold and Will-score pool. A protected move resolves through the ordinary effect command without offering a battle-ending ruling.

The recipient owner or GM chooses whether to resist. Resistance rolls the Will Score, not remaining spendable Will Points, and spends no Will. Guided characters use their recorded maximum Will; legacy sheets derive Insight + 3. Equal or greater successes resist. The result and exact dice are saved and cannot be rerolled. Declining resistance is an explicit saved response, not an inferred timeout.

Only the GM can then continue or end the battle, with a recorded reason. Successful resistance prevents this move from ending the battle. Failed or declined resistance does not automatically end it. Ending uses the existing EndBattle transition for HP, conditions, temporary transformations, equipment settlement and finished-battle history. One rollback restores the unresolved ruling and its prior resources. Normal explicit GM overrides remain available separately.

The typed SpecialCommand variants are PrepareHoldHands, ResistHoldHands and RuleHoldHands. Each is bound to the current cast sequence; wrong owners, stale sequences, duplicate resistance and premature rulings are rejected. Save/load preserves all stages in TurnFlow.completion.hold_hands. No database schema change is needed.

Outside-battle reduction of hostile intent remains narrative. Automatic Will recovery, lethal wounds, Max/Z/Dynamax and Sappy Seed remain excluded.

Verification references: completion_a45 public-command tests and MoveDecisionControls.test.tsx. The source checklist records the implemented effect clauses separately from the remaining whole-catalog semantic review; this delivery does not certify all moves.
