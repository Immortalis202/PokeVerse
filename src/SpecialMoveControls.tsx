import { useState } from "react";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";
import type { SpecialCommand } from "./bindings/SpecialCommand";
import { api, errorOf } from "./api";

export default function SpecialMoveControls({
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
  const [actor, setActor] = useState(""),
    [target, setTarget] = useState(""),
    [move, setMove] = useState(""),
    [attempts, setAttempts] = useState(2);
  const [recipient, setRecipient] = useState(""),
    [secondary, setSecondary] = useState(""),
    [reason, setReason] = useState(""),
    [notice, setNotice] = useState("");
  const [asReaction, setAsReaction] = useState(false);
  const [lift, setLift] = useState("");
  const [weight, setWeight] = useState(""),
    [night, setNight] = useState(false);
  const b = campaign.battle,
    f = b?.flow,
    s = f?.special;
  if (!b?.active || !f || !s?.enabled) return null;
  const gm = campaign.gm_id === playerId,
    owned = (id: string) =>
      gm || b.combatants.find((p) => p.id === id)?.owner_id === playerId;
  const active = b.combatants.filter((p) => p.deployed && p.hp > 0),
    source = active.find((p) => p.id === actor),
    pending = f.queue[0];
  const name = (id: string) =>
    b.combatants.find((p) => p.id === id)?.name ?? id;
  const action = (command: SpecialCommand): Action => ({
    type: "special_move",
    command,
    reason: reason.trim() || "Owner special-move decision",
  });
  async function send(command: SpecialCommand, preview = false) {
    setNotice("");
    try {
      if (preview) {
        const p = await api.previewBattleAction(campaign, action(command));
        setNotice(`${p.dice} host dice. ${p.warnings.join(" ")}`);
      } else await execute(action(command));
    } catch (e) {
      setNotice(errorOf(e).message);
    }
  }
  const successive: SpecialCommand = {
    kind: "Successive",
    reaction: asReaction,
    combatant_id: actor,
    target_id: target,
    move_name: move,
    attempts,
  };
  return (
    <section className="panel" aria-label="Special move automation">
      <h3>Special moves</h3>
      <p>
        Commit successive accuracy rolls together, then resolve each hit and its
        responses on the same turn. Stored charges and rampages remain available
        after reconnect.
      </p>
      {f.decoys?.enabled && (
        <>
          <p className="muted">
            Dynamax, Max Moves (including Max Guard) and Z-Moves are out of
            scope.
          </p>
          {Object.entries(f.decoys.active).map(
            ([id, decoy]) =>
              decoy && (
                <div className="notice" key={id}>
                  <strong>
                    {name(id)}: {decoy.move_name} decoy · {decoy.hp}/2 HP
                  </strong>
                  <p>
                    Frozen Defense {decoy.defenses[0]} · Special Defense{" "}
                    {decoy.defenses[1]}. Incoming move effects are blocked;
                    environment and existing statuses still affect the Pokémon.
                  </p>
                </div>
              ),
          )}
          {f.decoys.history.length > 0 && (
            <details>
              <summary>Decoy history</summary>
              <ul>
                {f.decoys.history.map((entry, i) => (
                  <li key={i}>{entry}</li>
                ))}
              </ul>
            </details>
          )}
        </>
      )}
      {Object.entries(s.charges).map(
        ([id, c]) =>
          c && (
            <div className="notice" key={id}>
              <strong>
                {name(id)}: charging {c.action.move_name}
              </strong>
              <p>
                {c.remaining > 0
                  ? `${c.remaining} more preparation action before release`
                  : `Ready to release on the next turn`}
              </p>
              {owned(id) && (
                <div className="toolbar">
                  <button
                    disabled={disabled}
                    onClick={() => send({ kind: "Release", combatant_id: id })}
                  >
                    {c.remaining > 0
                      ? "Continue charging"
                      : "Release charged move"}
                  </button>
                  <button
                    disabled={disabled}
                    onClick={() =>
                      send({ kind: "CancelCharge", combatant_id: id })
                    }
                  >
                    Cancel charge
                  </button>
                </div>
              )}
            </div>
          ),
      )}
      {Object.entries(s.rampages).map(
        ([id, r]) =>
          r && (
            <div className="notice" key={id}>
              {name(id)}: {r.name}, {r.uses}/3 uses this round.{" "}
              {owned(id) && (
                <button
                  disabled={disabled}
                  onClick={() => send({ kind: "EndRampage", combatant_id: id })}
                >
                  End rampage and apply Confusion
                </button>
              )}
            </div>
          ),
      )}
      {s.series && (
        <p role="status">
          {name(s.series.actor_id)}: {s.series.name}; {s.series.attempts}{" "}
          actions committed, {s.series.pending.length} queued hits remain.
        </p>
      )}
      <details>
        <summary>Successive attacks and encounter parameters</summary>
        <div className="form-grid">
          <label>
            Acting Pokémon
            <select value={actor} onChange={(e) => setActor(e.target.value)}>
              <option value="">Choose</option>
              {active
                .filter((p) => owned(p.id))
                .map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name}
                  </option>
                ))}
            </select>
          </label>
          <label>
            Move
            <select value={move} onChange={(e) => setMove(e.target.value)}>
              <option value="">Choose</option>
              {(source ? campaign.sheets[source.sheet_id]?.moves : [])?.map(
                (n) => (
                  <option key={n}>{n}</option>
                ),
              )}
            </select>
          </label>
          <label>
            One target
            <select value={target} onChange={(e) => setTarget(e.target.value)}>
              <option value="">Choose</option>
              {active
                .filter((p) => p.id !== actor)
                .map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name}
                  </option>
                ))}
            </select>
          </label>
          <label>
            Committed attempts
            <input
              type="number"
              min={1}
              max={5}
              value={attempts}
              onChange={(e) => setAttempts(Number(e.target.value))}
            />
          </label>
          <label>
            Recorded reason
            <input
              value={reason}
              maxLength={1000}
              onChange={(e) => setReason(e.target.value)}
            />
          </label>
        </div>
        <label>
          <input
            type="checkbox"
            checked={asReaction}
            onChange={(e) => setAsReaction(e.target.checked)}
          />
          Use as a reaction to the incoming attacker
        </label>
        <div className="toolbar">
          <button
            disabled={disabled || !actor || !target || !move}
            onClick={() => send(successive, true)}
          >
            Preview successive accuracy
          </button>
          <button
            disabled={disabled || !actor || !target || !move}
            onClick={() => send(successive)}
          >
            Commit successive actions
          </button>
        </div>
        {gm && (
          <>
            <p>
              Species weight is used when available. Record a corrected
              encounter weight or nighttime for conditional moves.
            </p>
            <div className="form-grid">
              <label>
                Weight override (kg)
                <input
                  type="number"
                  min="0.001"
                  step="0.1"
                  value={weight}
                  onChange={(e) => setWeight(e.target.value)}
                />
              </label>
              <label>
                GM lifting capacity (kg)
                <input
                  type="number"
                  min="0.001"
                  step="0.1"
                  value={lift}
                  onChange={(e) => setLift(e.target.value)}
                />
              </label>
              <label>
                <input
                  type="checkbox"
                  checked={night}
                  onChange={(e) => setNight(e.target.checked)}
                />
                Nighttime
              </label>
            </div>
            <button
              disabled={disabled || !actor || !reason.trim()}
              onClick={() =>
                send({
                  kind: "Parameters",
                  combatant_id: actor,
                  weight_kg: weight ? Number(weight) : null,
                  lift_kg: lift ? Number(lift) : null,
                  night,
                })
              }
            >
              Record encounter parameters
            </button>
          </>
        )}
      </details>
      {pending && owned(pending.actor_id) && (
        <div className="notice">
          <strong>Pending move effect: {pending.move_name}</strong>
          {["Magnitude", "Present"].includes(pending.move_name) && (
            <button
              disabled={disabled || s.random[pending.sequence] != null}
              onClick={() =>
                send({ kind: "Randomize", sequence: pending.sequence })
              }
            >
              Host-roll {pending.move_name}
            </button>
          )}
          {[
            "Healing Wish",
            "Lunar Dance",
            "Lunar Blessing",
            "Parabolic Charge",
            "Purify",
            "Bide",
          ].includes(pending.move_name) && (
            <>
              <div className="form-grid">
                <label>
                  {pending.move_name === "Bide"
                    ? "GM damage basis (user or target)"
                    : "Allied healing recipient"}
                  <select
                    value={recipient}
                    onChange={(e) => setRecipient(e.target.value)}
                  >
                    <option value="">None / decline optional healing</option>
                    {b.combatants
                      .filter((p) =>
                        pending.move_name === "Bide"
                          ? [pending.actor_id, pending.target_id].includes(p.id)
                          : p.side ===
                            b.combatants.find((v) => v.id === pending.actor_id)
                              ?.side,
                      )
                      .map((p) => (
                        <option key={p.id} value={p.id}>
                          {p.name}
                          {!p.deployed ? " (reserve)" : ""}
                          {p.hp === 0 ? " (fainted)" : ""}
                        </option>
                      ))}
                  </select>
                </label>
                <label>
                  Additional affected target
                  <select
                    value={secondary}
                    onChange={(e) => setSecondary(e.target.value)}
                  >
                    <option value="">Choose if needed</option>
                    {b.combatants.map((p) => (
                      <option key={p.id} value={p.id}>
                        {p.name}
                      </option>
                    ))}
                  </select>
                </label>
              </div>
              <button
                disabled={disabled || (pending.move_name === "Bide" && !gm)}
                onClick={() =>
                  send({
                    kind: "Select",
                    sequence: pending.sequence,
                    recipient_id: recipient,
                    secondary_id: secondary,
                  })
                }
              >
                Record move choices
              </button>
            </>
          )}
          <button
            disabled={disabled || f.window}
            onClick={() =>
              execute({
                type: "resolve_effect",
                sequence: pending.sequence,
                reason: reason.trim() || "Resolve reviewed special effect",
              })
            }
          >
            Resolve move effect
          </button>
        </div>
      )}
      {f.battlefield.end_round === b.round &&
        s.effects.map(
          (e, index) =>
            e.name === "Leech Seed" &&
            e.started < b.round &&
            owned(e.source) && (
              <div className="notice" key={index}>
                {name(e.target)}: Leech Seed round-end damage{" "}
                <button
                  disabled={disabled}
                  onClick={() => send({ kind: "Tick", index })}
                >
                  Roll ongoing damage and drain
                </button>
              </div>
            ),
        )}
      {!!s.effects.length && (
        <details>
          <summary>Active move effects</summary>
          <ul>
            {s.effects
              .filter((e) => !e.name.includes(":"))
              .map((e, i) => (
                <li key={i}>
                  {e.name}: {name(e.target)}
                  {e.rounds != null
                    ? ` (${e.rounds} rounds remaining)`
                    : ` (scene)`}
                </li>
              ))}
          </ul>
        </details>
      )}
      {notice && <p role="status">{notice}</p>}
    </section>
  );
}
