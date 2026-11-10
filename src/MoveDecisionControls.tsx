import { useState } from "react";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";
import type { SpecialCommand } from "./bindings/SpecialCommand";

export default function MoveDecisionControls({
  campaign,
  playerId,
  execute,
  disabled,
}: {
  campaign: Campaign;
  playerId: string;
  execute: (action: Action) => Promise<boolean>;
  disabled: boolean;
}) {
  const [reason, setReason] = useState("");
  const b = campaign.battle,
    f = b?.flow,
    q = f?.queue[0];
  if (
    !b?.active ||
    !f ||
    q?.move_name !== "Hold Hands" ||
    !f.completion?.blocks.includes("MOVE-A45")
  )
    return null;
  const d = f.completion.hold_hands?.[q.sequence];
  const gm = campaign.gm_id === playerId;
  const owns = (id: string) =>
    gm || b.combatants.some((p) => p.id === id && p.owner_id === playerId);
  const send = (command: SpecialCommand, why: string) =>
    execute({ type: "special_move", command, reason: why });
  const locked = disabled || f.window;
  return (
    <section aria-label="Hold Hands decision">
      <h4>Hold Hands</h4>
      <p>
        The target may resist by rolling their Will Score against this move’s
        Accuracy. The GM decides whether an unresisted offer ends the battle.
      </p>
      {f.window && <p>Resolve reactions before making this decision.</p>}
      {!d ? (
        <button
          disabled={locked || !owns(q.actor_id)}
          onClick={() =>
            send(
              { kind: "PrepareHoldHands", sequence: q.sequence },
              "Prepare Hold Hands target resistance",
            )
          }
        >
          Prepare target decision
        </button>
      ) : (
        <>
          <p>
            {b.combatants.find((p) => p.id === d.target_id)?.name ?? "Target"}:{" "}
            {d.will_dice} Will dice; {d.accuracy} successes required. This roll
            spends no Will.
          </p>
          {d.resisted == null ? (
            <>
              <button
                disabled={locked || !owns(d.target_id)}
                onClick={() =>
                  send(
                    {
                      kind: "ResistHoldHands",
                      sequence: q.sequence,
                      resist: true,
                    },
                    "Target chooses to resist Hold Hands",
                  )
                }
              >
                Roll Will resistance
              </button>
              <button
                disabled={locked || !owns(d.target_id)}
                onClick={() =>
                  send(
                    {
                      kind: "ResistHoldHands",
                      sequence: q.sequence,
                      resist: false,
                    },
                    "Target declines Hold Hands resistance",
                  )
                }
              >
                Do not resist
              </button>
            </>
          ) : (
            <>
              <p>
                {d.resisted
                  ? "Target resisted; this move cannot end the battle."
                  : "Target did not resist successfully; awaiting the GM’s decision."}
                {d.resistance != null && ` Will successes: ${d.resistance}.`}
              </p>
              {gm && (
                <>
                  <p>
                    Ending the battle saves HP, conditions and held-item results
                    through normal battle completion and restores temporary
                    forms.
                  </p>
                  <label>
                    Hold Hands ruling reason
                    <input
                      value={reason}
                      onChange={(e) => setReason(e.target.value)}
                    />
                  </label>
                  <button
                    disabled={locked || !reason.trim()}
                    onClick={() =>
                      send(
                        {
                          kind: "RuleHoldHands",
                          sequence: q.sequence,
                          end_battle: false,
                        },
                        reason.trim(),
                      )
                    }
                  >
                    Continue battle
                  </button>
                  <button
                    disabled={locked || !reason.trim() || d.resisted}
                    onClick={() =>
                      send(
                        {
                          kind: "RuleHoldHands",
                          sequence: q.sequence,
                          end_battle: true,
                        },
                        reason.trim(),
                      )
                    }
                  >
                    End battle by agreement
                  </button>
                </>
              )}
            </>
          )}
        </>
      )}
    </section>
  );
}
