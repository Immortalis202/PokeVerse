import { useState } from "react";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";
import type { SpecialCommand } from "./bindings/SpecialCommand";
import { battleSheet } from "./sheets";
import { api, errorOf } from "./api";

export default function MetronomeControls({
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
  const [actor, setActor] = useState("");
  const [target, setTarget] = useState("");
  const [template, setTemplate] = useState("");
  const [physicalFaces, setPhysicalFaces] = useState<number[]>([]);
  const [reason, setReason] = useState("");
  const [notice, setNotice] = useState("");
  const [attempts, setAttempts] = useState(1);
  const b = campaign.battle,
    f = b?.flow,
    state = f?.completion;
  if (!b?.active || !state?.blocks.includes("MOVE-A46")) return null;
  const gm = campaign.gm_id === playerId;
  const owns = (id: string) =>
    gm || b.combatants.some((p) => p.id === id && p.owner_id === playerId);
  const active = b.combatants.filter((p) => p.deployed && p.hp > 0);
  const draw = state.metronome_draw;
  const count = Math.max(1, Math.min(attempts, draw?.attempt_limit ?? 1));
  const source = active.find((p) => p.id === draw?.actor_id);
  const send = (command: SpecialCommand, why: string) =>
    execute({ type: "special_move", command, reason: why });
  const eligible = active.filter((p) =>
    battleSheet(campaign, p.id)?.moves.includes("Metronome"),
  );
  return (
    <section aria-label="Metronome choices">
      <h4>Metronome</h4>
      <p>
        Roll category and Power before Accuracy. A committed draw stays until
        used or recorded as a failed attempt. The copied attack uses Insight +
        Perform against one foe.
      </p>
      <p>
        {state.metronome_table
          ? `Physical die faces: ${state.metronome_table.physical_faces.join(", ")}; all others mean Special.`
          : "The GM must record a category die table before rolling; the card supplies no table."}
      </p>
      {gm && (
        <details>
          <summary>Category die table</summary>
          <p>
            Select one to five die faces that mean Physical. Remaining faces
            mean Special.
          </p>
          {[1, 2, 3, 4, 5, 6].map((face) => (
            <label key={face}>
              <input
                type="checkbox"
                checked={physicalFaces.includes(face)}
                onChange={(e) =>
                  setPhysicalFaces(
                    e.target.checked
                      ? [...physicalFaces, face].sort()
                      : physicalFaces.filter((v) => v !== face),
                  )
                }
              />
              {face}
            </label>
          ))}
          <button
            disabled={
              disabled ||
              !!draw ||
              !!state.copied_cast ||
              !!f?.queue.length ||
              physicalFaces.length < 1 ||
              physicalFaces.length > 5 ||
              !reason.trim()
            }
            onClick={() =>
              send(
                { kind: "MetronomeTable", physical_faces: physicalFaces },
                reason.trim(),
              )
            }
          >
            Record category table
          </button>
        </details>
      )}
      {gm && (
        <label>
          Metronome ruling reason
          <input value={reason} onChange={(e) => setReason(e.target.value)} />
        </label>
      )}
      {draw ? (
        <>
          <p>
            Saved dice: {draw.faces.join(", ")}. Result: {draw.category}, Power{" "}
            {draw.power}.
          </p>
          {!draw.selected && (
            <>
              <label>
                Matching move
                <select
                  value={template}
                  disabled={!gm || disabled}
                  onChange={(e) => setTemplate(e.target.value)}
                >
                  <option value="">Choose matching template</option>
                  {draw.candidates.map((name) => (
                    <option key={name}>{name}</option>
                  ))}
                </select>
              </label>
              {gm && (
                <button
                  disabled={
                    disabled ||
                    !template ||
                    !draw.candidates.includes(template) ||
                    !reason.trim()
                  }
                  onClick={() =>
                    send(
                      {
                        kind: "SelectMetronome",
                        token: draw.token,
                        move_name: template,
                      },
                      reason.trim(),
                    )
                  }
                >
                  Save template choice
                </button>
              )}
              {!draw.candidates.length && (
                <p>
                  No in-scope, non-Unique template matches this draw. The GM can
                  record the attempt as failed.
                </p>
              )}
            </>
          )}
          {draw.selected && (
            <>
              {(draw.attempt_limit ?? 1) > 1 && (
                <label>
                  Copied successive actions
                  <select
                    value={count}
                    onChange={(e) => setAttempts(Number(e.target.value))}
                  >
                    {Array.from(
                      { length: draw.attempt_limit },
                      (_, i) => i + 1,
                    ).map((n) => (
                      <option key={n} value={n}>
                        {n}
                      </option>
                    ))}
                  </select>
                </label>
              )}
              <p>
                Selected: {draw.selected}. Normal damage, added effects, costs
                and defenses apply.
              </p>
              <label>
                Metronome target
                <select
                  value={target}
                  onChange={(e) => setTarget(e.target.value)}
                >
                  <option value="">Choose foe</option>
                  {active
                    .filter((p) => source && p.side !== source.side)
                    .map((p) => (
                      <option key={p.id} value={p.id}>
                        {p.name}
                      </option>
                    ))}
                </select>
              </label>
              <button
                disabled={disabled || !target || !owns(draw.actor_id)}
                onClick={() =>
                  send(
                    {
                      kind: "CastMetronome",
                      attempts: count,
                      token: draw.token,
                      target_id: target,
                    },
                    "Use committed Metronome draw with original Accuracy",
                  )
                }
              >
                Cast selected move
              </button>
              <button
                disabled={disabled || !target || !owns(draw.actor_id)}
                onClick={async () => {
                  try {
                    const preview = await api.previewBattleAction(campaign, {
                      type: "special_move",
                      command: {
                        kind: "CastMetronome",
                        attempts: count,
                        token: draw.token,
                        target_id: target,
                      },
                      reason: "Preview committed copied Accuracy",
                    });
                    setNotice(
                      `${preview.dice} host Accuracy dice. ${preview.warnings.join(" ")}`,
                    );
                  } catch (error) {
                    setNotice(errorOf(error).message);
                  }
                }}
              >
                Preview copied Accuracy
              </button>
            </>
          )}
          {gm && (
            <button
              disabled={disabled || !reason.trim()}
              onClick={() =>
                send(
                  { kind: "DiscardMetronome", token: draw.token },
                  reason.trim(),
                )
              }
            >
              Record failed attempt and spend action
            </button>
          )}
        </>
      ) : state.copied_cast ? (
        <p>
          Resolve copied {state.copied_cast.template} through the normal battle
          controls.
        </p>
      ) : (
        <>
          <label>
            Metronome user
            <select value={actor} onChange={(e) => setActor(e.target.value)}>
              <option value="">Choose user</option>
              {eligible.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
          </label>
          <button
            disabled={
              disabled ||
              !actor ||
              !owns(actor) ||
              !state.metronome_table ||
              !!f?.queue.length ||
              f?.order[f.cursor] !== actor
            }
            onClick={() =>
              send(
                { kind: "DrawMetronome", combatant_id: actor },
                "Commit Metronome category and Power dice",
              )
            }
          >
            Roll category and Power
          </button>
        </>
      )}
      {notice && <p role="status">{notice}</p>}
    </section>
  );
}
