import { useState } from "react";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";
import type { ItemCommand } from "./bindings/ItemCommand";
import { errorOf } from "./api";

const captureBalls = new Set([
  "Dusk Ball",
  "Fast Ball",
  "Greatball",
  "Heavy Ball",
  "Luxury Ball",
  "Masterball",
  "Old Pokeball",
  "Pokeball",
  "Quick Ball",
  "Ultraball",
]);
const berries = new Set([
  "Oran Berry",
  "Sitrus Berry",
  "Pecha Berry",
  "Rawst Berry",
  "Aspear Berry",
  "Cheri Berry",
  "Chesto Berry",
  "Persim Berry",
  "Lum Berry",
]);
const recyclable = new Set([
  ...berries,
  "Potion",
  "Super Potion",
  "Hyper Potion",
  "Antidote",
  "Awakening",
  "Burn Heal",
  "Ice Heal",
  "Paralyze Heal",
  "Full Heal",
]);
const flavors = [
  "Bitter",
  "Dry",
  "Effervescent",
  "Fresh",
  "Frozen",
  "Half-eaten",
  "Juicy",
  "Numbing",
  "Oily",
  "Rotten",
  "Salty",
  "Spicy",
  "Sour",
  "Sugary",
  "Tough",
  "Uneatable",
  "Withered",
];
export default function HeldItemControls({
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
  const [fetcherId, setFetcherId] = useState("");
  const [wildId, setWildId] = useState("");
  const [ball, setBall] = useState("");
  const [attemptId, setAttemptId] = useState("");
  const [notice, setNotice] = useState("");
  const [unitId, setUnitId] = useState("");
  const [recipient, setRecipient] = useState("");
  const [swapId, setSwapId] = useState("");
  const [override, setOverride] = useState(false);
  const [flavor, setFlavor] = useState("Spicy");
  const [extraHealing, setExtraHealing] = useState(1);
  const battle = campaign.battle;
  const flow = battle?.flow;
  const items = flow?.items;
  if (!battle?.active || !items?.enabled || !items.initialized) return null;
  const gm = campaign.gm_id === playerId;
  const units = Object.values(items.units).filter((unit) => unit !== undefined);
  const selected = items.units[unitId];
  const swap = items.units[swapId];
  const name = (id: string) =>
    battle.combatants.find((p) => p.id === id)?.name ?? id;
  const busy = disabled || !!flow?.ability_events.length;
  const rulingDisabled = busy || !reason.trim();
  const fetchers = battle.combatants.filter(
    (p) =>
      p.deployed &&
      p.hp > 0 &&
      p.selected_ability === "Ball Fetch" &&
      campaign.sheets[p.sheet_id]?.trainer_id,
  );
  const fetcher = fetchers.find((p) => p.id === fetcherId);
  const trainer = fetcher
    ? campaign.sheets[campaign.sheets[fetcher.sheet_id]?.trainer_id ?? ""]
    : undefined;
  const wildTargets = fetcher
    ? battle.combatants.filter(
        (p) =>
          p.deployed &&
          p.side !== fetcher.side &&
          campaign.sheets[p.sheet_id]?.kind === "pokemon" &&
          !campaign.sheets[p.sheet_id]?.trainer_id,
      )
    : [];

  const hasBall =
    !!trainer &&
    ((captureBalls.has(ball) && (trainer.inventory[ball] ?? 0) > 0) ||
      (trainer.stored_items ?? []).some(
        (unit) => unit.id === ball && captureBalls.has(unit.name),
      ));
  async function send(command: ItemCommand) {
    setNotice("");
    try {
      await execute({
        type: "held_item",
        command,
        reason: reason.trim() || "Owner resolves a held-item choice",
      });
    } catch (error) {
      setNotice(errorOf(error).message);
    }
  }
  return (
    <section className="panel" aria-label="Held battle items">
      <h3>Held items</h3>
      <p>
        Consuming a held berry spends no action. The host validates ownership,
        effects and ability restrictions.
      </p>
      <p>
        Ordinary healing items and moves share a limit of 3 HP per round. A
        saved Cud Chew activation can wait for a later round.
      </p>
      {battle.combatants
        .filter((p) => gm || p.owner_id === playerId)
        .map((p) => {
          const held = units
            .filter((u) => u.holder === p.id && u.state === "held")
            .sort(
              (a, b) => a.slot - b.slot || a.unit.id.localeCompare(b.unit.id),
            );
          return (
            <div key={p.id}>
              <strong>{p.name}</strong>{" "}
              <small>{p.runtime.healed} HP restored this round</small>
              {p.selected_ability === "Gluttony" && <p>Gluttony: eating food, medicine or herbal medicine causes no negative ingestion effects. Up to three held berries must have different effects; unreviewed medicine use still needs GM adjudication.</p>}
              {p.selected_ability === "Klutz" && <p>Klutz negates ordinary held effects. Its printed form/type-changing exceptions still apply; unsupported transformations are not enabled by this exception.</p>}
              {!held.length && <p>No held item.</p>}
              {held.map((u) => (
                <p key={u.unit.id}>
                  {u.unit.name} · recorded usage {u.unit.uses}{" "}
                  <small>{u.review ? u.review.note : "Effect review pending; ask the GM before applying this item's effects."}</small>{" "}
                  {campaign.sheets[p.sheet_id]?.moves.includes("Fling") && (
                    <button
                      disabled={
                        busy ||
                        p.hp <= 0 ||
                        !!flow?.queue.some((q) => q.actor_id === p.id)
                      }
                      onClick={() =>
                        send({
                          kind: "fling",
                          combatant_id: p.id,
                          unit_id: u.unit.id,
                        })
                      }
                    >
                      {items.fling[p.id] === u.unit.id
                        ? "Selected for Fling"
                        : `Use ${u.unit.name} for Fling`}
                    </button>
                  )}
                  {u.unit.flavor && <span> · flavor {u.unit.flavor} </span>}
                  {berries.has(u.unit.name) &&
                    campaign.sheets[p.sheet_id]?.moves.includes(
                      "Natural Gift",
                    ) && (
                      <button
                        disabled={
                          busy ||
                          p.hp <= 0 ||
                          !u.unit.flavor ||
                          !!flow?.queue.some((q) => q.actor_id === p.id)
                        }
                        onClick={() =>
                          send({
                            kind: "natural_gift",
                            combatant_id: p.id,
                            unit_id: u.unit.id,
                          })
                        }
                      >
                        {items.natural_gift[p.id] === u.unit.id
                          ? "Selected for Natural Gift"
                          : `Use ${u.unit.name} for Natural Gift`}
                      </button>
                    )}
                  {gm &&
                    p.selected_ability === "Ripen" &&
                    ["Oran Berry", "Sitrus Berry"].includes(u.unit.name) && (
                      <button
                        disabled={
                          rulingDisabled ||
                          p.hp <= 0 ||
                          extraHealing < 0 ||
                          extraHealing > 100
                        }
                        onClick={() =>
                          send({
                            kind: "consume_ripened_berry",
                            combatant_id: p.id,
                            unit_id: u.unit.id,
                            extra_healing: extraHealing,
                          })
                        }
                      >
                        Consume {u.unit.name} with GM Ripen healing increase
                      </button>
                    )}
                  {berries.has(u.unit.name) && (
                    <button
                      disabled={busy || p.hp <= 0}
                      onClick={() =>
                        send({
                          kind: "consume_berry",
                          combatant_id: p.id,
                          unit_id: u.unit.id,
                        })
                      }
                    >
                      Consume {u.unit.name} for {p.name}
                    </button>
                  )}
                </p>
              ))}
              {campaign.sheets[p.sheet_id]?.moves.includes("Recycle") && (
                <div>
                  <p>
                    Recycle attempts this battle:{" "}
                    {items.recycle_uses[p.id] ?? 0}/3
                  </p>
                  {units
                    .filter(
                      (unit) =>
                        unit.state === "consumed" &&
                        recyclable.has(unit.unit.name),
                    )
                    .map((unit) => (
                      <button
                        key={`recycle-${unit.unit.id}`}
                        disabled={
                          busy ||
                          p.hp <= 0 ||
                          (items.recycle_uses[p.id] ?? 0) >= 3 ||
                          !!flow?.queue.some((q) => q.actor_id === p.id)
                        }
                        onClick={() =>
                          send({
                            kind: "recycle",
                            combatant_id: p.id,
                            unit_id: unit.unit.id,
                          })
                        }
                      >
                        {items.recycle[p.id] === unit.unit.id
                          ? "Selected for Recycle"
                          : `Select ${unit.unit.name} for Recycle`}
                      </button>
                    ))}
                </div>
              )}
            </div>
          );
        })}
      {Object.entries(items.chew_charges).map(([chargeId, charge]) => {
        if (!charge) return null;
        const p = battle.combatants.find((p) => p.id === charge.combatant_id);
        if (!gm && p?.owner_id !== playerId) return null;
        return (
          <p key={chargeId}>
            {name(charge.combatant_id)} · saved second berry activation{" "}
            <button
              disabled={
                busy || !p || p.hp <= 0 || p.selected_ability !== "Cud Chew"
              }
              onClick={() => send({ kind: "chew_again", charge_id: chargeId })}
            >
              Chew {charge.unit.name} again
            </button>
          </p>
        );
      })}
      {units.some((unit) => unit.state === "ground") && (
        <details>
          <summary>Retrieve a ground item · one action</summary>
          {battle.combatants
            .filter((p) => p.deployed && (gm || p.owner_id === playerId))
            .map((p) => (
              <div key={p.id}>
                <strong>{p.name}</strong>
                {units
                  .filter((unit) => unit.state === "ground")
                  .map((unit) => (
                    <button
                      key={unit.unit.id}
                      disabled={
                        busy ||
                        p.hp <= 0 ||
                        !!flow?.queue.length ||
                        flow?.order[flow.cursor] !== p.id
                      }
                      onClick={() =>
                        send({
                          kind: "retrieve",
                          combatant_id: p.id,
                          unit_id: unit.unit.id,
                        })
                      }
                    >
                      Retrieve {unit.unit.name} for {p.name} · spend one action
                    </button>
                  ))}
              </div>
            ))}
        </details>
      )}
      {items.reveals
        .filter((reveal) => gm || reveal.owner_id === playerId)
        .map((reveal, index) => (
          <p key={`frisk-${index}`}>
            <strong>
              Frisk: {name(reveal.source_id)} → {name(reveal.target_id)}
            </strong>{" "}
            {reveal.items.length ? reveal.items.join(", ") : "No held item"} ·
            round {reveal.round}
          </p>
        ))}
      {Object.entries(items.fetches ?? {})
        .filter(
          ([, f]) =>
            f && (gm || campaign.sheets[f.trainer_id]?.owner_id === playerId),
        )
        .map(
          ([id, f]) =>
            f && (
              <p key={id}>
                Ball Fetch · {f.unit.name} · {id}:{" "}
                {f.returned
                  ? "returned to the trainer's stored equipment"
                  : `protected until the end of round ${f.round}`}
              </p>
            ),
        )}
      {gm && (
        <details>
          <summary>GM item rulings and recovery</summary>
          <label>
            Item ruling reason
            <input
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              maxLength={1000}
            />
          </label>
          <fieldset>
            <legend>Ball Fetch: record a failed wild capture</legend>
            <p>
              Confirm that the capture roll failed against a wild foe.
              Adjudicate its action cost first; this record protects an owned
              ball and returns it at round end.
            </p>
            <label>
              Capture attempt ID
              <input
                value={attemptId}
                maxLength={100}
                placeholder="capture-roll-1"
                onChange={(e) => setAttemptId(e.target.value)}
              />
            </label>
            <label>
              Ball Fetch Pokemon
              <select
                value={fetcherId}
                onChange={(e) => {
                  setFetcherId(e.target.value);
                  setBall("");
                  setWildId("");
                }}
              >
                <option value="">Choose the trainer's Pokemon</option>
                {fetchers.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name}
                  </option>
                ))}
              </select>
            </label>
            <label>
              Wild foe confirmed by GM
              <select
                value={wildId}
                onChange={(e) => setWildId(e.target.value)}
              >
                <option value="">Choose an untrained wild foe</option>
                {wildTargets.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name}
                  </option>
                ))}
              </select>
            </label>
            <label>
              Owned capture ball
              <select value={ball} onChange={(e) => setBall(e.target.value)}>
                <option value="">
                  Choose a ball from {trainer?.name ?? "the trainer"}
                </option>
                {Object.entries(trainer?.inventory ?? {})
                  .filter(
                    ([name, count]) =>
                      (count ?? 0) > 0 && captureBalls.has(name),
                  )
                  .map(([name, count]) => (
                    <option key={name} value={name}>
                      {name} · stock {count}
                    </option>
                  ))}
                {(trainer?.stored_items ?? [])
                  .filter((unit) => captureBalls.has(unit.name))
                  .map((unit) => (
                    <option key={unit.id} value={unit.id}>
                      {unit.name} · stored unit {unit.id}
                    </option>
                  ))}
              </select>
            </label>
            <button
              disabled={
                rulingDisabled ||
                !trainer ||
                !wildId ||
                !hasBall ||
                !attemptId.trim() ||
                !!items.fetches?.[attemptId.trim()] ||
                items.end_round === battle.round
              }
              onClick={() =>
                trainer &&
                send({
                  kind: "failed_capture",
                  attempt_id: attemptId.trim(),
                  trainer_id: trainer.id,
                  fetcher_id: fetcherId,
                  target_id: wildId,
                  ball,
                })
              }
            >
              Record failed capture and protect this ball
            </button>
          </fieldset>
          <label>
            Recorded item unit
            <select value={unitId} onChange={(e) => setUnitId(e.target.value)}>
              <option value="">Choose a unit</option>
              {units.map((u) => (
                <option key={u.unit.id} value={u.unit.id}>
                  {u.unit.name} · {name(u.holder)} · {u.state} · {u.unit.id}
                </option>
              ))}
            </select>
          </label>
          <button
            disabled={rulingDisabled || selected?.state !== "held"}
            onClick={() =>
              selected &&
              send({
                kind: "spend",
                combatant_id: selected.holder,
                unit_id: unitId,
              })
            }
          >
            Record item spent after adjudication
          </button>
          <p>
            Record a spent item after resolving its use and action cost. This
            record does not apply those effects.
          </p>
          <label>
            Natural Gift berry flavor
            <select value={flavor} onChange={(e) => setFlavor(e.target.value)}>
              {flavors.map((value) => (
                <option key={value} value={value}>
                  {value}
                </option>
              ))}
            </select>
          </label>
          <button
            disabled={
              rulingDisabled ||
              selected?.state !== "held" ||
              !berries.has(selected?.unit.name ?? "")
            }
            onClick={() =>
              send({ kind: "berry_flavor", unit_id: unitId, flavor })
            }
          >
            Record the berry's actual flavor
          </button>
          <label>
            GM Ripen extra healing
            <input
              type="number"
              min={0}
              max={100}
              value={extraHealing}
              onChange={(e) => setExtraHealing(Number(e.target.value))}
            />
          </label>
          <p>
            Ripen healing is a storyteller ruling. Enter a reason, then use the
            berry's Ripen consumption button above. Ordinary berry use keeps its
            printed healing.
          </p>
          <label>
            Recipient
            <select
              value={recipient}
              onChange={(e) => setRecipient(e.target.value)}
            >
              <option value="">Choose a recipient</option>
              {battle.combatants.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
          </label>
          <label>
            <input
              type="checkbox"
              checked={override}
              onChange={(e) => setOverride(e.target.checked)}
            />
            Explicitly override Sticky Hold
          </label>
          <div className="toolbar">
            <button
              disabled={
                rulingDisabled ||
                selected?.state !== "held" ||
                !recipient ||
                recipient === selected?.holder
              }
              onClick={() =>
                selected &&
                send({
                  kind: "transfer",
                  from_id: selected.holder,
                  to_id: recipient,
                  unit_id: unitId,
                  override_sticky_hold: override,
                })
              }
            >
              Record transfer
            </button>
            <button
              disabled={rulingDisabled || selected?.state !== "held"}
              onClick={() =>
                selected &&
                send({
                  kind: "lose",
                  combatant_id: selected.holder,
                  unit_id: unitId,
                  override_sticky_hold: override,
                })
              }
            >
              Record item loss
            </button>
            <button
              disabled={
                rulingDisabled || selected?.state !== "ground" || !recipient
              }
              onClick={() =>
                send({
                  kind: "recover",
                  combatant_id: recipient,
                  unit_id: unitId,
                })
              }
            >
              Recover ground item
            </button>
          </div>
          <label>
            Other held unit for swap
            <select value={swapId} onChange={(e) => setSwapId(e.target.value)}>
              <option value="">Choose another held unit</option>
              {units
                .filter(
                  (u) => u.state === "held" && u.holder !== selected?.holder,
                )
                .map((u) => (
                  <option key={u.unit.id} value={u.unit.id}>
                    {u.unit.name} · {name(u.holder)} · {u.unit.id}
                  </option>
                ))}
            </select>
          </label>
          <button
            disabled={
              rulingDisabled ||
              selected?.state !== "held" ||
              swap?.state !== "held" ||
              selected.holder === swap?.holder
            }
            onClick={() =>
              selected &&
              swap &&
              send({
                kind: "swap",
                first_id: selected.holder,
                second_id: swap.holder,
                first_unit_id: unitId,
                second_unit_id: swapId,
                override_sticky_hold: override,
              })
            }
          >
            Record atomic swap
          </button>
          <p>
            Recovery policy:{" "}
            {items.return_policy === "return_surviving"
              ? "Return surviving items to original holders"
              : items.return_policy === "keep_transfers"
                ? "Keep transfers and losses"
                : "Choose before ending a battle with surviving transferred or lost items"}
            .
          </p>
          <div className="toolbar">
            <button
              disabled={rulingDisabled}
              onClick={() =>
                send({ kind: "return_policy", policy: "return_surviving" })
              }
            >
              Return surviving items
            </button>
            <button
              disabled={rulingDisabled}
              onClick={() =>
                send({ kind: "return_policy", policy: "keep_transfers" })
              }
            >
              Keep transfers and losses
            </button>
          </div>
          <p>
            Consumed berries return only through a supported effect such as
            Harvest. Ground recovery cannot recreate a consumed item.
          </p>
          <ul>
            {items.history.map((e, i) => (
              <li key={i}>
                {e.kind}: {items.units[e.unit_id]?.unit.name ?? e.unit_id} ·{" "}
                {name(e.from)}
                {e.to && ` → ${name(e.to)}`} · round {e.round} · {e.reason}
              </li>
            ))}
          </ul>
        </details>
      )}
      {notice && <p role="alert">{notice}</p>}
    </section>
  );
}
