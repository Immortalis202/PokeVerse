import { useState } from "react";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";

/** Saved owner choice a pending cast needs before its effect resolves. */
export default function MoveOptionControls({
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
  const [choice, setChoice] = useState("");
  const b = campaign.battle,
    f = b?.flow,
    q = f?.queue[0];
  const options = q ? f?.completion?.open_options?.[q.sequence] : undefined;
  if (!b?.active || !f || !q || !options?.length) return null;
  const saved = f.completion.move_options?.[q.sequence];
  const owns =
    campaign.gm_id === playerId ||
    b.combatants.some((p) => p.id === q.actor_id && p.owner_id === playerId);
  const value = options.includes(choice) ? choice : options[0];
  return (
    <section aria-label="Move choice">
      <h4>{q.move_name}: choose</h4>
      {saved && <p>Saved choice: {saved}</p>}
      <select
        aria-label={`${q.move_name} option`}
        value={value}
        disabled={disabled || !owns}
        onChange={(e) => setChoice(e.target.value)}
      >
        {options.map((o) => (
          <option key={o} value={o}>
            {o}
          </option>
        ))}
      </select>
      <button
        disabled={disabled || !owns}
        onClick={() =>
          execute({
            type: "special_move",
            command: {
              kind: "ChooseMoveOption",
              sequence: q.sequence,
              option: value,
            },
            reason: `Owner chooses ${value} for ${q.move_name}`,
          })
        }
      >
        Save choice
      </button>
    </section>
  );
}
