import BattlefieldControls from "./BattlefieldControls";
import type { Catalog } from "./catalog";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";
import { useState } from "react";
import { api, errorOf } from "./api";
export default function CoreBattleFlow({
  campaign,
  playerId,
  execute,
  disabled,
  catalog,
}: {
  catalog: Catalog;
  campaign: Campaign;
  playerId: string;
  execute: (a: Action) => Promise<boolean>;
  disabled: boolean;
}) {
  const [poolModifier, setPoolModifier] = useState(0),
    [successModifier, setSuccessModifier] = useState(0),
    [damageModifier, setDamageModifier] = useState(0);
  const [reactionMove, setReactionMove] = useState("");
  const [correctedWill, setCorrectedWill] = useState(0);
  const [fieldDuration, setFieldDuration] = useState(0);
  const [severeDamage, setSevereDamage] = useState(0),
    [duration, setDuration] = useState(0);
  const [iceActor, setIceActor] = useState(""),
    [iceTarget, setIceTarget] = useState(""),
    [iceMove, setIceMove] = useState("");
  const [statusSource, setStatusSource] = useState("");
  const [kind, setKind] = useState("Burn"),
    [detail, setDetail] = useState(""),
    [statusTarget, setStatusTarget] = useState("");
  const [reason, setReason] = useState(""),
    [notice, setNotice] = useState("");
  const b = campaign.battle,
    f = b?.flow;
  if (!b?.active || !f) return null;
  const gm = campaign.gm_id === playerId;
  const owned = (id: string) =>
    gm || b.combatants.find((p) => p.id === id)?.owner_id === playerId;
  const current = b.combatants.find((p) => p.id === f.order[f.cursor]);
  const incoming = f.tactics?.enabled ? f.queue[0] : f.queue.at(-1),
    pending = f.queue[0];
  const damaging =
    pending &&
    !(
      (f.special?.enabled && pending.move_name === "Present") ||
      (f.special?.enabled &&
        pending.move_name === "Bide" &&
        !f.special.bides[pending.actor_id])
    ) &&
    catalog.moves.find((m) => m.Name === pending.move_name)?.Category !==
      "Support";
  const defender = b.combatants.find((p) => p.id === incoming?.target_id);
  async function send(a: Action, preview = false) {
    setNotice("");
    try {
      if (preview) {
        const p = await api.previewBattleAction(campaign, a);
        setNotice(`${p.dice} host dice. ${p.warnings.join(" ")}`);
        return;
      }
      await execute(a);
    } catch (e) {
      setNotice(errorOf(e).message);
    }
  }
  const why = reason.trim() || "Player battle decision";
  return (
    <section className="panel" aria-label="Core battle turn flow">
      <BattlefieldControls
        campaign={campaign}
        playerId={playerId}
        execute={execute}
        disabled={disabled}
      />
      <h3>Turn and reaction flow</h3>
      <p>
        {!f.started
          ? "Roll initiative for every active Pokémon, then confirm the order."
          : current
            ? `Turn ${f.turn}: ${current.name}`
            : "Round actions complete or held. The GM can advance the round."}
      </p>
      {notice && (
        <p role="status" className="notice">
          {notice}
        </p>
      )}
      <label>
        Decision reason
        <input
          value={reason}
          maxLength={1000}
          onChange={(e) => setReason(e.target.value)}
        />
      </label>
      {!f.started && gm && (
        <button
          disabled={disabled}
          onClick={() => send({ type: "start_turns" })}
        >
          Confirm initiative order
        </button>
      )}
      {current && f.queue.length === 0 && owned(current.id) && (
        <div className="toolbar">
          <button
            disabled={disabled}
            onClick={() =>
              send({ type: "pass_turn", combatant_id: current.id, reason: why })
            }
          >
            Hold / resolve skipped turn
          </button>
          {(["sleep", "burn", "love"] as const).map((kind) => (
            <button
              key={kind}
              disabled={
                disabled ||
                !current.runtime.statuses.some((s) =>
                  kind === "burn"
                    ? s.kind.startsWith("Burn")
                    : s.kind === (kind === "sleep" ? "Sleep" : "In Love"),
                )
              }
              onClick={() =>
                send({
                  type: "status_check",
                  combatant_id: current.id,
                  kind,
                  reason: why,
                })
              }
            >
              {kind} recovery check
            </button>
          ))}
        </div>
      )}
      {b.combatants
        .filter((p) => p.deployed && owned(p.id))
        .map((p) => (
          <div className="list-row" key={p.id}>
            <div>
              <strong>{p.name}</strong>
              <p>
                Will {p.runtime.will} ·{" "}
                {p.runtime.statuses
                  .map(
                    (s) =>
                      `${s.kind}${s.rounds ? ` (${s.rounds} rounds)` : ""}`,
                  )
                  .join(", ") || "No structured conditions"}
              </p>
            </div>
            <div className="toolbar">
              {f.held.includes(p.id) && f.order.length > 0 && (
                <button
                  disabled={
                    disabled || f.queue.length > 0 || f.deferred !== null
                  }
                  onClick={() =>
                    send({
                      type: "take_held_turn",
                      combatant_id: p.id,
                      reason: why,
                    })
                  }
                >
                  Take held turn
                </button>
              )}
              {p.runtime.statuses.some((s) => s.kind === "Confusion") && (
                <button
                  disabled={disabled}
                  onClick={() =>
                    send({
                      type: "status_check",
                      combatant_id: p.id,
                      kind: "confusion",
                      reason: why,
                    })
                  }
                >
                  Check Confusion
                </button>
              )}
              {gm && !p.runtime.will_enabled && (
                <button
                  disabled={disabled}
                  onClick={() =>
                    send({
                      type: "authorize_will",
                      combatant_id: p.id,
                      enabled: true,
                      reason: why,
                    })
                  }
                >
                  Authorize Will prerequisite
                </button>
              )}
              {(["pain", "chance", "fate"] as const).map((mode) => (
                <button
                  key={mode}
                  disabled={
                    disabled ||
                    p.runtime.will < 1 ||
                    (!gm && !p.runtime.will_enabled)
                  }
                  onClick={() =>
                    send({
                      type: "spend_will",
                      combatant_id: p.id,
                      mode,
                      reason: why,
                    })
                  }
                >
                  Will: {mode}
                </button>
              ))}
            </div>
          </div>
        ))}
      {f.window && defender && incoming && (
        <div className="panel">
          <h4>
            {defender.name}: reaction to {incoming.move_name}
          </h4>
          <p>
            Evasion is limited to once per turn and shares the reaction slot
            with reaction moves. Clash can respond to each eligible incoming
            attack, independently of that slot, using an unused damaging move.
            Each attempt spends one of five actions per round, even on failure.
            Passing closes this response chain.
          </p>
          {owned(defender.id) && (
            <>
              <label>
                Reaction / Clash move
                <select
                  value={reactionMove}
                  onChange={(e) => setReactionMove(e.target.value)}
                >
                  <option value="">Choose a move</option>
                  {campaign.sheets[defender.sheet_id]?.moves.map((m) => (
                    <option key={m}>{m}</option>
                  ))}
                </select>
              </label>
              <div className="toolbar">
                {["evade", "clash", "move"].map((kind) => (
                  <button
                    key={kind}
                    disabled={disabled || (kind !== "evade" && !reactionMove)}
                    onClick={() =>
                      send({
                        type: "react",
                        combatant_id: defender.id,
                        kind,
                        move_name: reactionMove,
                        reason: why,
                      })
                    }
                  >
                    {kind === "move" ? "Use reaction move" : kind}
                  </button>
                ))}
                <button
                  disabled={disabled}
                  onClick={() =>
                    send({
                      type: "pass_reaction",
                      combatant_id: defender.id,
                      reason: why,
                    })
                  }
                >
                  Pass reaction
                </button>
              </div>
            </>
          )}
          {[
            "0.4.0-core",
            "0.5.0-core",
            "0.6.0-core",
            "0.7.0-core",
            "0.8.0-core",
            "0.9.0-core",
            "0.10.0-core",
            "0.11.0-core",
            "0.12.0-core",
          ].includes(campaign.engine) &&
            b.combatants
              .filter(
                (p) =>
                  p.deployed &&
                  p.hp > 0 &&
                  p.side === defender.side &&
                  p.id !== defender.id &&
                  owned(p.id),
              )
              .map((p) => (
                <button
                  key={p.id}
                  disabled={disabled}
                  onClick={() =>
                    send({
                      type: "cover_ally",
                      combatant_id: p.id,
                      ally_id: defender.id,
                      reason: why,
                    })
                  }
                >
                  {p.name}: Cover {defender.name} (Will, Loyalty above 2)
                </button>
              ))}
        </div>
      )}
      {!f.window && pending && (
        <div className="panel">
          <h4>Resolve {pending.move_name}</h4>
          <p>
            {b.combatants.find((p) => p.id === pending.actor_id)?.name} →{" "}
            {b.combatants.find((p) => p.id === pending.target_id)?.name} ·{" "}
            {pending.timing} reaction / main action
          </p>
          <div className="toolbar">
            {gm && (
              <div className="form-grid">
                {[
                  ["Dice adjustment", poolModifier, setPoolModifier],
                  ["Success adjustment", successModifier, setSuccessModifier],
                  [
                    "Final damage adjustment",
                    damageModifier,
                    setDamageModifier,
                  ],
                ].map(([label, value, set]) => (
                  <label key={String(label)}>
                    {String(label)}
                    <input
                      type="number"
                      min={-20}
                      max={20}
                      value={Number(value)}
                      onChange={(e) =>
                        (set as (n: number) => void)(Number(e.target.value))
                      }
                    />
                  </label>
                ))}
              </div>
            )}
            {pending.targets.length > 0 && (
              <p>
                Targets:{" "}
                {pending.targets
                  .map(
                    (t) =>
                      `${b.combatants.find((p) => p.id === t.target_id)?.name || t.target_id}${t.avoided ? " (avoided)" : t.resolved ? " (resolved)" : ""}`,
                  )
                  .join(", ")}
                . Shared damage dice retained: {pending.shared_dice.length}.
              </p>
            )}
            {pending.field_scope && (
              <div>
                <p>
                  Field scope: {pending.field_scope}. Resolve reviewed mechanics
                  automatically, or record a GM ruling for unsupported effects.
                </p>
                {owned(pending.actor_id) && f.battlefield?.enabled && (
                  <button
                    disabled={disabled}
                    onClick={() =>
                      send({
                        type: "resolve_effect",
                        sequence: pending.sequence,
                        reason: why,
                      })
                    }
                  >
                    Apply reviewed battlefield effect
                  </button>
                )}
                {gm && (
                  <>
                    <label>
                      Field duration (0 = encounter)
                      <input
                        type="number"
                        min={0}
                        max={1000}
                        value={fieldDuration}
                        onChange={(e) =>
                          setFieldDuration(Number(e.target.value))
                        }
                      />
                    </label>
                    <button
                      disabled={disabled || !reason.trim()}
                      onClick={() =>
                        send({
                          type: "adjudicate_field",
                          sequence: pending.sequence,
                          rounds: fieldDuration || null,
                          reason,
                        })
                      }
                    >
                      Record field effect and finish action
                    </button>
                  </>
                )}
              </div>
            )}
            {owned(pending.actor_id) && !pending.field_scope && (
              <>
                {damaging && (
                  <>
                    <button
                      disabled={disabled}
                      onClick={() =>
                        send(
                          {
                            type: "damage",
                            combatant_id: pending.actor_id,
                            sequence: pending.sequence,
                            pool_modifier: gm ? poolModifier : 0,
                            success_modifier: gm ? successModifier : 0,
                            damage_modifier: gm ? damageModifier : 0,
                            reason: why,
                          },
                          true,
                        )
                      }
                    >
                      Preview damage dice
                    </button>
                    <button
                      disabled={disabled || pending.damage_done}
                      onClick={() =>
                        send({
                          type: "damage",
                          combatant_id: pending.actor_id,
                          sequence: pending.sequence,
                          pool_modifier: gm ? poolModifier : 0,
                          success_modifier: gm ? successModifier : 0,
                          damage_modifier: gm ? damageModifier : 0,
                          reason: why,
                        })
                      }
                    >
                      Resolve automatic damage
                    </button>
                  </>
                )}
                <button
                  disabled={disabled}
                  onClick={() =>
                    send({
                      type: "resolve_effect",
                      sequence: pending.sequence,
                      reason: why,
                    })
                  }
                >
                  Resolve reviewed effects
                </button>
              </>
            )}
            {gm && !pending.field_scope && (
              <button
                disabled={disabled || !reason.trim()}
                onClick={() =>
                  send({
                    type: "adjudicate_effect",
                    sequence: pending.sequence,
                    reason,
                  })
                }
              >
                Record GM resolution
              </button>
            )}
          </div>
        </div>
      )}
      {f.field_effects.length > 0 && (
        <div className="panel">
          <h4>Active field effects · GM adjudicated mechanics</h4>
          {f.field_effects.map((effect) => (
            <div key={effect.sequence}>
              <p>
                {effect.move_name} · {effect.scope} ·{" "}
                {effect.rounds ?? "encounter"} rounds · {effect.ruling}
              </p>
              {gm && (
                <button
                  disabled={disabled || !reason.trim()}
                  onClick={() =>
                    send({
                      type: "remove_field_effect",
                      sequence: effect.sequence,
                      reason,
                    })
                  }
                >
                  Remove {effect.move_name}
                </button>
              )}
            </div>
          ))}
        </div>
      )}
      {Object.entries(f.covers).length > 0 && (
        <p>
          Cover:{" "}
          {Object.entries(f.covers)
            .map(
              ([ally, coverer]) =>
                `${b.combatants.find((p) => p.id === coverer)?.name} covers ${b.combatants.find((p) => p.id === ally)?.name}`,
            )
            .join("; ")}
          . Holding preserves cover; acting forfeits it.
        </p>
      )}
      {f.started &&
        f.order.length === 0 &&
        f.queue.length === 0 &&
        b.combatants.some(
          (p) =>
            p.deployed &&
            p.runtime.statuses.some((s) => s.kind === "Frozen Solid"),
        ) && (
          <div className="panel">
            <h4>Break ice before the next round</h4>
            <p>
              Frozen subjects and allies with actions remaining can attack the 5
              HP ice block. Its defenses are 2; a successful super-effective
              move breaks it immediately.
            </p>
            <div className="form-grid">
              <label>
                Acting Pokémon
                <select
                  value={iceActor}
                  onChange={(e) => {
                    setIceActor(e.target.value);
                    setIceMove("");
                  }}
                >
                  <option value="">Choose</option>
                  {b.combatants
                    .filter(
                      (p) =>
                        p.deployed && owned(p.id) && p.hp > 0 && p.actions < 5,
                    )
                    .map((p) => (
                      <option key={p.id} value={p.id}>
                        {p.name}
                      </option>
                    ))}
                </select>
              </label>
              <label>
                Frozen Pokémon
                <select
                  value={iceTarget}
                  onChange={(e) => setIceTarget(e.target.value)}
                >
                  <option value="">Choose</option>
                  {b.combatants
                    .filter(
                      (p) =>
                        p.deployed &&
                        p.runtime.statuses.some(
                          (s) => s.kind === "Frozen Solid",
                        ),
                    )
                    .map((p) => (
                      <option key={p.id} value={p.id}>
                        {p.name} · ice{" "}
                        {5 -
                          (p.runtime.statuses.find(
                            (s) => s.kind === "Frozen Solid",
                          )?.progress ?? 0)}
                        /5 HP
                      </option>
                    ))}
                </select>
              </label>
              <label>
                Move
                <select
                  value={iceMove}
                  onChange={(e) => setIceMove(e.target.value)}
                >
                  <option value="">Choose</option>
                  {campaign.sheets[
                    b.combatants.find((p) => p.id === iceActor)?.sheet_id ?? ""
                  ]?.moves.map((m) => (
                    <option key={m}>{m}</option>
                  ))}
                </select>
              </label>
            </div>
            <button
              disabled={disabled || !iceActor || !iceTarget || !iceMove}
              onClick={() =>
                send({
                  type: "break_ice",
                  combatant_id: iceActor,
                  target_id: iceTarget,
                  move_name: iceMove,
                  reason: why,
                })
              }
            >
              Attack ice
            </button>
          </div>
        )}
      {gm && (
        <details>
          <summary>Structured status decision</summary>
          <div className="form-grid">
            <label>
              Combatant
              <select
                value={statusTarget}
                onChange={(e) => setStatusTarget(e.target.value)}
              >
                <option value="">Choose</option>
                {b.combatants
                  .filter((p) => p.deployed)
                  .map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name}
                    </option>
                  ))}
              </select>
            </label>
            <label>
              Condition
              <select value={kind} onChange={(e) => setKind(e.target.value)}>
                {[
                  "Burn",
                  "Burn 2",
                  "Burn 3",
                  "Poison",
                  "Badly Poisoned",
                  "Paralysis",
                  "Sleep",
                  "Confusion",
                  "Flinch",
                  "Disable",
                  "Frozen Solid",
                  "In Love",
                ].map((k) => (
                  <option key={k}>{k}</option>
                ))}
              </select>
            </label>
            <label>
              Duration in rounds (0 = no automatic expiry)
              <input
                type="number"
                min={0}
                max={100}
                value={duration}
                onChange={(e) => setDuration(Number(e.target.value))}
              />
            </label>
            {f.ability_rules && (
              <label>
                Status inflictor (optional; enables Synchronize)
                <select
                  value={statusSource}
                  onChange={(e) => setStatusSource(e.target.value)}
                >
                  <option value="">No inferred source / GM correction</option>
                  {b.combatants
                    .filter((p) => p.deployed)
                    .map((p) => (
                      <option key={p.id} value={p.id}>
                        {p.name}
                      </option>
                    ))}
                </select>
              </label>
            )}
            <label>
              Disabled move / beloved ID
              <input
                value={detail}
                onChange={(e) => setDetail(e.target.value)}
              />
            </label>
          </div>
          <div className="toolbar">
            {[false, true].map((remove) => (
              <button
                key={String(remove)}
                disabled={disabled || !statusTarget || !reason.trim()}
                onClick={() =>
                  send(
                    !remove && statusSource
                      ? {
                          type: "inflict_status",
                          combatant_id: statusTarget,
                          source_id: statusSource,
                          kind,
                          detail,
                          rounds: duration || null,
                          reason,
                        }
                      : {
                          type: "set_status",
                          combatant_id: statusTarget,
                          kind,
                          detail,
                          rounds: duration || null,
                          remove,
                          reason,
                        },
                  )
                }
              >
                {remove ? "Remove" : "Apply"} condition
              </button>
            ))}
          </div>
          <label>
            GM Will correction
            <input
              type="number"
              min={0}
              max={100}
              value={correctedWill}
              onChange={(e) => setCorrectedWill(Number(e.target.value))}
            />
          </label>
          <button
            disabled={disabled || !statusTarget || !reason.trim()}
            onClick={() =>
              send({
                type: "adjust_will",
                combatant_id: statusTarget,
                will: correctedWill,
                reason,
              })
            }
          >
            Correct encounter Will
          </button>
          <label>
            GM severe status damage this round
            <input
              type="number"
              min={0}
              max={999}
              value={severeDamage}
              onChange={(e) => setSevereDamage(Number(e.target.value))}
            />
          </label>
          <button
            disabled={disabled || !statusTarget || !reason.trim()}
            onClick={() =>
              send({
                type: "adjudicate_status",
                combatant_id: statusTarget,
                damage: severeDamage,
                reason,
              })
            }
          >
            Record severe status outcome
          </button>
          <p className="muted">
            Severe status escalation has contradictory PDF values and requires
            explicit GM adjudication. Unknown text conditions remain manual.
          </p>
        </details>
      )}
    </section>
  );
}
