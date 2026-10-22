import { battleSheet } from "./sheets";
import { useState } from "react";
import type { Action } from "./bindings/Action";
import type { Campaign } from "./bindings/Campaign";
import type { TacticsCommand } from "./bindings/TacticsCommand";
import { api, errorOf } from "./api";
export default function TacticsControls({
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
  const [choices, setChoices] = useState<Record<string, string>>({});
  const [source, setSource] = useState(""),
    [target, setTarget] = useState(""),
    [move, setMove] = useState(""),
    [mode, setMode] = useState("printed"),
    [reason, setReason] = useState(""),
    [notice, setNotice] = useState("");
  const [coat, setCoat] = useState("next_effect");
  const b = campaign.battle,
    f = b?.flow,
    s = f?.tactics;
  if (!b?.active || !f || !s?.enabled) return null;
  const gm = campaign.gm_id === playerId;
  const name = (id: string) =>
    b.combatants.find((p) => p.id === id)?.name ?? id;
  const why = reason.trim() || "Owner resolves the reviewed replacement";
  const action = (command: TacticsCommand): Action => ({
    type: "tactics",
    command,
    reason: why,
  });
  async function send(command: TacticsCommand, preview = false) {
    setNotice("");
    try {
      if (preview) {
        const v = await api.previewBattleAction(campaign, action(command));
        setNotice(`${v.dice} host dice. ${v.warnings.join(" ")}`);
      } else await execute(action(command));
    } catch (e) {
      setNotice(errorOf(e).message);
    }
  }
  return (
    <section className="panel" aria-label="Targeting and replacement decisions">
      <h3>Targeting and replacements</h3>
      <p>
        Redirection and priority restrictions are automatic. The outgoing owner
        chooses its registered reserve; the host validates the choice. No
        reserve never silently ends the encounter.
      </p>
      {s.switches.map((req) => {
        const out = b.combatants.find((p) => p.id === req.outgoing);
        const owned = gm || out?.owner_id === playerId;
        const trainer = out && campaign.sheets[out.sheet_id]?.trainer_id;
        const reserves = b.combatants.filter(
          (p) =>
            !!trainer &&
            !p.deployed &&
            p.hp > 0 &&
            p.owner_id === out?.owner_id &&
            p.side === out?.side &&
            campaign.sheets[p.sheet_id]?.trainer_id === trainer,
        );
        const ready = req.immediate || f.queue.length === 0;
        const selected = choices[req.id] ?? "";
        return (
          <div className="notice" key={req.id}>
            <strong>
              {req.name}: replace {name(req.outgoing)}
            </strong>
            <p>
              {ready
                ? "Choose a conscious reserve from the same trainer."
                : "Replacement is queued until this attack chain finishes."}{" "}
              The incoming Pokémon inherits the slot, initiative and remaining
              actions.
            </p>
            {owned && (
              <>
                <label>
                  Incoming reserve
                  <select
                    value={selected}
                    onChange={(e) =>
                      setChoices((v) => ({ ...v, [req.id]: e.target.value }))
                    }
                  >
                    <option value="">Choose a reserve</option>
                    {reserves.map((p) => (
                      <option key={p.id} value={p.id}>
                        {p.name}
                      </option>
                    ))}
                  </select>
                </label>
                <div className="toolbar">
                  <button
                    disabled={disabled || !ready || !selected}
                    onClick={() =>
                      send({
                        kind: "Switch",
                        request_id: req.id,
                        incoming_id: selected,
                        ruling: "replace",
                      })
                    }
                  >
                    Confirm replacement
                  </button>
                  {gm && (
                    <>
                      <button
                        disabled={
                          disabled ||
                          !ready ||
                          reserves.length > 0 ||
                          !reason.trim()
                        }
                        onClick={() =>
                          send({
                            kind: "Switch",
                            request_id: req.id,
                            incoming_id: null,
                            ruling: "withdraw",
                          })
                        }
                      >
                        Withdraw without reserve
                      </button>
                      <button
                        disabled={disabled || !ready || !reason.trim()}
                        onClick={() =>
                          send({
                            kind: "Switch",
                            request_id: req.id,
                            incoming_id: null,
                            ruling: "waive",
                          })
                        }
                      >
                        Record GM waiver
                      </button>
                    </>
                  )}
                </div>
              </>
            )}
            {!reserves.length && (
              <p>
                No conscious registered reserve is available. The GM must record
                withdrawal or a waiver, or end the battle.
              </p>
            )}
          </div>
        );
      })}
      {!!s.lures.length && (
        <ul>
          {s.lures.map((l, i) => (
            <li key={i}>
              {l.name}: {name(l.target)} · round {l.round}
            </li>
          ))}
        </ul>
      )}
      <label>
        Decision / exception reason
        <input
          value={reason}
          onChange={(e) => setReason(e.target.value)}
          maxLength={1000}
        />
      </label>
      {gm && (
        <details>
          <summary>GM redirection and priority rulings</summary>
          <p>
            Choose a redirector only when multiple effects compete. For an
            ability Reaction 1 that conflicts with printed Late Reaction, record
            the timing to use.
          </p>
          <div className="form-grid">
            <label>
              Acting Pokémon
              <select
                value={source}
                onChange={(e) => {
                  setSource(e.target.value);
                  setMove("");
                }}
              >
                <option value="">Choose</option>
                {b.combatants
                  .filter((p) => p.deployed && p.hp > 0)
                  .map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name}
                    </option>
                  ))}
              </select>
            </label>
            <label>
              Chosen redirector
              <select
                value={target}
                onChange={(e) => setTarget(e.target.value)}
              >
                <option value="">Choose</option>
                {b.combatants
                  .filter((p) => p.deployed && p.hp > 0)
                  .map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name}
                    </option>
                  ))}
              </select>
            </label>
            <label>
              Move with conflicting timing
              <select value={move} onChange={(e) => setMove(e.target.value)}>
                <option value="">Choose</option>
                {(battleSheet(campaign, source)?.moves ?? []).map((n) => (
                  <option key={n}>{n}</option>
                ))}
              </select>
            </label>
            <label>
              Timing
              <select value={mode} onChange={(e) => setMode(e.target.value)}>
                <option value="printed">Printed timing</option>
                <option value="ability">Ability Reaction 1</option>
              </select>
            </label>
          </div>
          <div className="toolbar">
            <button
              disabled={
                disabled ||
                f.queue.some((q) => !!q.damage_context || q.damage_done) ||
                !source ||
                !target ||
                !reason.trim()
              }
              onClick={() =>
                send({
                  kind: "Redirect",
                  combatant_id: source,
                  recipient_id: target,
                })
              }
            >
              Record competing redirector
            </button>
            <button
              disabled={
                disabled ||
                f.queue.some((q) => !!q.damage_context || q.damage_done) ||
                !source ||
                !move ||
                !reason.trim()
              }
              onClick={() =>
                send({
                  kind: "Priority",
                  combatant_id: source,
                  move_name: move,
                  mode,
                })
              }
            >
              Record priority ruling
            </button>
            <label>
              Magic Coat duration
              <select value={coat} onChange={(e) => setCoat(e.target.value)}>
                <option value="next_effect">Next reflected effect</option>
                <option value="round">Current round</option>
                <option value="scene">Encounter until recall</option>
              </select>
            </label>
            <button
              disabled={disabled || !source || !reason.trim()}
              onClick={() =>
                send({
                  kind: "CoatDuration",
                  combatant_id: source,
                  duration: coat,
                })
              }
            >
              Record Magic Coat duration
            </button>
          </div>
        </details>
      )}
      {!!s.history.length && (
        <details>
          <summary>Targeting and switch history</summary>
          <ul>
            {s.history.slice(-20).map((v, i) => (
              <li key={i}>{v}</li>
            ))}
          </ul>
        </details>
      )}
      {notice && <p role="status">{notice}</p>}
    </section>
  );
}
