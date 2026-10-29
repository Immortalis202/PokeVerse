import { battleSheet } from "./sheets";
import { useState } from "react";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";
import type { RecurringCommand } from "./bindings/RecurringCommand";
export default function RecurringMoveControls({
  campaign,
  playerId,
  execute,
  disabled,
}: {
  campaign: Campaign;
  playerId: string;
  execute: (a: Action) => Promise<boolean>;
  disabled: boolean;
}) {
  const [reason, setReason] = useState("");
  const b = campaign.battle,
    f = b?.flow,
    s = f?.recurring;
  if (!b?.active || !s?.enabled || !f) return null;
  const gm = campaign.gm_id === playerId,
    owned = (id: string) =>
      gm || b.combatants.find((p) => p.id === id)?.owner_id === playerId;
  const name = (id: string) =>
    b.combatants.find((p) => p.id === id)?.name ?? id;
  const send = (command: RecurringCommand) =>
    execute({
      type: "recurring_move",
      command,
      reason: reason.trim() || "Recorded recurring move decision",
    });
  return (
    <section aria-label="Recurring move effects">
      <h4>Recurring and sleep actions</h4>
      <label>
        Decision reason{" "}
        <input value={reason} onChange={(e) => setReason(e.target.value)} />
      </label>
      {Object.entries(s.yawn).map(([id, e]) => (
        <p key={`y${id}`}>
          {name(id)}: Yawn applies Sleep at the beginning of round{" "}
          {e?.due_round}; leaving cancels it.
        </p>
      ))}
      {Object.entries(s.perish).map(([id, e]) => (
        <p key={`p${id}`}>
          {name(id)}: Perish Song resolves at the end of round {e?.due_round};
          leaving cancels it.
        </p>
      ))}
      {Object.keys(s.nightmares).map((id) => (
        <p key={`n${id}`}>
          {name(id)}: Nightmare deals 1 damage after each failed wake attempt.
        </p>
      ))}
      {Object.entries(s.malignant).map(([id, e]) => (
        <p key={`m${id}`}>
          {name(id)}:{" "}
          {e?.active
            ? "Allied until Poison is cured"
            : e?.activates !== null
              ? `Allegiance changes in round ${e?.activates}`
              : "Malignant Chain Loyalty check pending"}
          .{" "}
          {e?.activates === null && (
            <button
              disabled={
                disabled ||
                !owned(id) ||
                f.battlefield.end_round !== b.round ||
                f.queue.length > 0
              }
              onClick={() => send({ kind: "Loyalty", combatant_id: id })}
            >
              Roll Loyalty (3 successes)
            </button>
          )}
        </p>
      ))}
      {s.pending_talk ? (
        <p>
          {name(s.pending_talk.actor)} drew {s.pending_talk.move_name}; random
          foe: {name(s.pending_talk.foe)}.{" "}
          <button
            disabled={disabled || !owned(s.pending_talk.actor)}
            onClick={() =>
              send({ kind: "UseCopy", combatant_id: s.pending_talk!.actor })
            }
          >
            Roll copied move accuracy
          </button>
          {gm && (
            <button
              disabled={disabled || !reason.trim()}
              onClick={() =>
                send({ kind: "Discard", combatant_id: s.pending_talk!.actor })
              }
            >
              Record failed attempt (spend 1 action)
            </button>
          )}
        </p>
      ) : (
        b.combatants
          .filter(
            (p) =>
              p.deployed &&
              p.hp > 0 &&
              owned(p.id) &&
              p.runtime.statuses.some((x) => x.kind === "Sleep") &&
              battleSheet(campaign, p.id)?.moves.includes("Sleep Talk"),
          )
          .map((p) => (
            <button
              key={p.id}
              disabled={
                disabled ||
                s.executing !== null ||
                f.queue.length > 0 ||
                p.actions >= 5
              }
              onClick={() => send({ kind: "Draw", combatant_id: p.id })}
            >
              {p.name}: draw Sleep Talk move and foe
            </button>
          ))
      )}
      <p>
        Sleep Talk uses a host draw and the normal move resolution. A committed
        unsupported draw requires GM adjudication; it cannot be rerolled freely.
        Snore uses the normal move controls while asleep.
      </p>
    </section>
  );
}
