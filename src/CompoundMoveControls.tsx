import { useState } from "react";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";
import type { CompoundCommand } from "./bindings/CompoundCommand";

const surroundings = [
  "City/Building",
  "Plains/Grasslands",
  "Forest/Jungle",
  "Lake/River/Sea",
  "Cave/Mountain",
  "Desert/Volcano",
  "Ice/Snow",
];
const terrains = [
  "Electric Terrain",
  "Grassy Terrain",
  "Misty Terrain",
  "Psychic Terrain",
];

export default function CompoundMoveControls({
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
    s = f?.compound;
  if (!b?.active || !s?.enabled || !f) return null;
  const gm = campaign.gm_id === playerId;
  const owned = (id: string) =>
    gm || b.combatants.find((p) => p.id === id)?.owner_id === playerId;
  const name = (id: string) =>
    b.combatants.find((p) => p.id === id)?.name ?? id;
  const send = (command: CompoundCommand) =>
    execute({
      type: "compound_move",
      command,
      reason: reason.trim() || "Recorded compound move choice",
    });
  return (
    <div aria-label="Compound move choices">
      <h4>Conditional move choices</h4>
      <label>
        Decision reason{" "}
        <input value={reason} onChange={(e) => setReason(e.target.value)} />
      </label>
      {s.crashes.map((id) => (
        <p key={id}>
          {name(id)}: Axe Kick missed.{" "}
          <button
            disabled={disabled || !owned(id)}
            onClick={() => send({ kind: "Crash", combatant_id: id })}
          >
            Roll 4 crash damage dice
          </button>
        </p>
      ))}
      {gm && (
        <fieldset disabled={disabled}>
          <legend>Confirm Secret Power surroundings</legend>
          <p>
            Select every physical environment present. Active terrains are added
            automatically; the trainer chooses between available sources.
          </p>
          {surroundings.map((env) => (
            <label key={env}>
              <input
                type="checkbox"
                checked={s.environments.includes(env)}
                onChange={(e) =>
                  send({
                    kind: "Environment",
                    sources: e.target.checked
                      ? [...s.environments, env]
                      : s.environments.filter((x) => x !== env),
                  })
                }
              />
              {env}
            </label>
          ))}
        </fieldset>
      )}
      {b.combatants
        .filter((p) => p.deployed && p.hp > 0 && owned(p.id))
        .map((p) => {
          const choices = [
            ...s.environments,
            ...terrains.filter((t) =>
              f.battlefield.rules.some(
                (r) =>
                  r.kind === t &&
                  (r.scope === "all" ||
                    r.scope === `side:${p.side}` ||
                    r.scope === `combatant:${p.id}` ||
                    (r.scope.startsWith("opponents-of:") &&
                      r.scope !== `opponents-of:${p.side}`)),
              ),
            ),
          ];
          return (
            <div key={p.id}>
              <label>
                {p.name}: Secret Power alignment{" "}
                <select
                  disabled={disabled}
                  value={
                    s.alignments[p.id] ??
                    (choices.length === 1 ? choices[0] : "")
                  }
                  onChange={(e) =>
                    send({
                      kind: "Align",
                      combatant_id: p.id,
                      source: e.target.value,
                    })
                  }
                >
                  <option value="" disabled>
                    Choose available energy
                  </option>
                  {[...new Set(choices)].map((x) => (
                    <option key={x}>{x}</option>
                  ))}
                </select>
              </label>
              {gm && (
                <label>
                  <input
                    type="checkbox"
                    disabled={disabled}
                    checked={s.minimized.includes(p.id)}
                    onChange={(e) =>
                      send({
                        kind: "Minimized",
                        combatant_id: p.id,
                        active: e.target.checked,
                      })
                    }
                  />
                  GM confirms Minimize is active for Stomp vulnerability
                </label>
              )}
            </div>
          );
        })}
      {s.relic.map((id) => (
        <p key={id}>
          {name(id)}: Relic Song form choice.
          <button
            disabled={disabled || !owned(id) || f.queue.length > 0}
            onClick={() =>
              send({ kind: "Relic", combatant_id: id, change: true })
            }
          >
            Change Meloetta form
          </button>
          <button
            disabled={disabled || !owned(id) || f.queue.length > 0}
            onClick={() =>
              send({ kind: "Relic", combatant_id: id, change: false })
            }
          >
            Keep current form
          </button>
        </p>
      ))}
    </div>
  );
}
