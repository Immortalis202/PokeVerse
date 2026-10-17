import { useState } from "react";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";

export default function AbilityInheritanceControls({
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
  const [minutes, setMinutes] = useState(60);
  const [clockMode, setClockMode] = useState<"advance" | "correct_clock">(
    "advance",
  );
  const [reason, setReason] = useState("");
  if (campaign.engine !== "0.12.0-core") return null;
  const state = campaign.ability_inheritance;
  const gm = campaign.gm_id === playerId;
  const active = !!campaign.battle?.active;
  const records = Object.values(state.copies).filter((r) => !!r);
  const sheets = [...new Set(records.map((r) => r.sheet_id))];
  return (
    <section className="panel" aria-label="Inherited abilities and game time">
      <details>
        <summary>Inherited abilities · game minute {state.minutes}</summary>
        <p>
          Receiver and Power of Alchemy copies last 24 game hours. The GM
          advances elapsed game time between battles. Each Pokémon has one
          active ability.
        </p>
        {records.length === 0 && <p>No inherited abilities recorded.</p>}
        {sheets.map((id) => {
          const sheet = campaign.sheets[id];
          const owned = gm || sheet?.owner_id === playerId;
          return (
            <div key={id}>
              <label>
                Active inheritance for {sheet?.name ?? id}
                <select
                  aria-label={`Active inheritance for ${sheet?.name ?? id}`}
                  disabled={
                    disabled || !owned || (active && (!gm || !reason.trim()))
                  }
                  value={state.selected[id] ?? ""}
                  onChange={(e) =>
                    execute({
                      type: "ability_transfer",
                      command: {
                        kind: "select",
                        sheet_id: id,
                        copy_id: e.target.value || null,
                      },
                      reason: reason.trim() || "Select one inherited ability",
                    })
                  }
                >
                  <option value="">
                    Original ability: {sheet?.ability ?? "unresolved"}
                  </option>
                  {records
                    .filter((r) => r.sheet_id === id)
                    .map((r) => (
                      <option
                        key={r.id}
                        value={r.id}
                        disabled={r.expires <= state.minutes}
                      >
                        {r.ability} — {r.donor_name}
                        {r.expires <= state.minutes ? " (expired)" : ""}
                      </option>
                    ))}
                </select>
              </label>
              {records
                .filter((r) => r.sheet_id === id)
                .map((r) => (
                  <p key={r.id}>
                    {r.ability} from {r.donor_name} ({r.source}):{" "}
                    {r.expires <= state.minutes
                      ? "expired"
                      : `${Math.floor((r.expires - state.minutes) / 60)}h ${(r.expires - state.minutes) % 60}m remaining`}
                  </p>
                ))}
            </div>
          );
        })}
        {active && (
          <p>
            During battle, selection timing requires a GM ruling. Resolve
            pending actions and choices first.
          </p>
        )}
        <label>
          Decision / elapsed-time reason
          <input
            value={reason}
            maxLength={1000}
            onChange={(e) => setReason(e.target.value)}
          />
        </label>
        {gm && !active && (
          <div className="toolbar">
            <label>
              Clock operation
              <select
                value={clockMode}
                onChange={(e) =>
                  setClockMode(e.target.value as "advance" | "correct_clock")
                }
              >
                <option value="advance">Advance elapsed time</option>
                <option value="correct_clock">
                  GM correction: set exact minute
                </option>
              </select>
            </label>
            <label>
              {clockMode === "advance"
                ? "Elapsed game minutes"
                : "Correct campaign minute"}
              <input
                type="number"
                min={clockMode === "advance" ? 1 : 0}
                max={clockMode === "advance" ? 525600 : 4294965855}
                value={minutes}
                onChange={(e) => setMinutes(Number(e.target.value))}
              />
            </label>
            <button
              disabled={
                disabled ||
                !reason.trim() ||
                !Number.isInteger(minutes) ||
                minutes < (clockMode === "advance" ? 1 : 0) ||
                minutes > (clockMode === "advance" ? 525600 : 4294965855)
              }
              onClick={() =>
                execute({
                  type: "ability_transfer",
                  command: { kind: clockMode, minutes },
                  reason: reason.trim(),
                })
              }
            >
              {clockMode === "advance"
                ? "Advance ability clock"
                : "Correct ability clock"}
            </button>
          </div>
        )}
        <details>
          <summary>Inheritance history</summary>
          {state.history.map((entry, index) => (
            <p key={index}>{entry}</p>
          ))}
        </details>
      </details>
    </section>
  );
}
