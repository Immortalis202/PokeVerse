import { useState } from "react";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";

export default function PrivateInformation({
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
  const f = campaign.battle?.flow;
  if (!f?.information?.enabled) return null;
  const gm = campaign.gm_id === playerId;
  const event = f.ability_events[0];
  const pending = event?.kind.startsWith("info-") ? event : undefined;
  const offer = gm && pending ? f.information.offers[pending.id] : undefined;
  const reveals = f.information.reveals.filter(
    (r) => gm || r.recipient === playerId,
  );
  const name = (id: string) =>
    campaign.battle?.combatants.find((p) => p.id === id)?.name ?? "Pokémon";
  const resolve = (choice: string) =>
    void execute({
      type: "resolve_ability",
      event_id: pending!.id,
      choice,
      reason,
    });
  const highest = Math.max(
    0,
    ...(offer?.candidates.map((m) => m.power ?? 0) ?? []),
  );
  return (
    <section className="panel" aria-label="Private trainer information">
      <h3>Private trainer information</h3>
      {pending && (
        <p>
          {name(pending.combatant_id)}: {pending.ability} awaits the GM’s
          private decision.
        </p>
      )}
      {offer && (
        <>
          <p>
            {pending!.ability === "Anticipation"
              ? "The trainer receives an alert, without foe or move names. Lethal and super-effective checks use current deployed foes and supported types/immunities. Review unresolved definitions."
              : "Choose the strongest move in secret. Printed Power is a comparison aid; variable/fixed damage, added effects and ties need your ruling. Confirm the trainer’s Insight check before revealing the message; record a failed check with No message."}
          </p>
          <ul>
            {offer.candidates.map((m, i) => (
              <li key={`${m.foe_id}:${m.move_name}`}>
                <strong>
                  {name(m.foe_id)}: {m.move_name}
                </strong>{" "}
                — {m.category}; Power{" "}
                {m.power ?? "variable / fixed / unresolved"}
                {m.power === highest && highest > 0
                  ? " (highest printed Power)"
                  : ""}
                {m.lethal ? "; lethal threat" : ""}
                {m.super_effective ? "; super-effective threat" : ""}
                {m.uncertain ? "; GM classification required" : ""}
                <details>
                  <summary>Card and comparison</summary>
                  {m.detail}
                </details>
                {pending!.ability === "Forewarn" && (
                  <button
                    disabled={disabled || !reason.trim()}
                    onClick={() => resolve(`move:${i}`)}
                  >
                    Reveal {m.move_name} to the trainer
                  </button>
                )}
              </li>
            ))}
          </ul>
          <label>
            Private ruling / Insight result
            <input
              maxLength={1000}
              value={reason}
              onChange={(e) => setReason(e.target.value)}
            />
          </label>
          {pending!.choices
            .filter((c) => !c.startsWith("move:"))
            .map((c) => (
              <button
                key={c}
                disabled={disabled || !reason.trim()}
                onClick={() => resolve(c)}
              >
                {c === "alert"
                  ? "Send threat alert"
                  : c === "clear"
                    ? "Record no classified threat"
                    : "No message / failed Insight"}
              </button>
            ))}
          <button
            disabled={disabled || !reason.trim()}
            onClick={() => resolve("manual")}
          >
            Close with private GM ruling
          </button>
        </>
      )}
      {reveals.map((r, i) => (
        <p key={i}>
          <strong>
            {name(r.source_id)} — {r.ability}, round {r.round}:
          </strong>{" "}
          {r.decision === "manual"
            ? "GM ruling recorded."
            : r.move_name
              ? `Strongest move: ${r.move_name}.`
              : r.alert
                ? "A foe knows a lethal or super-effective move."
                : r.ability === "Anticipation"
                  ? "No classified threat at this warning window."
                  : "No message delivered."}{" "}
          {r.reason}
        </p>
      ))}
      {!pending && reveals.length === 0 && <p>No private messages for you.</p>}
    </section>
  );
}
