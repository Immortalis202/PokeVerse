import { useEffect, useState } from "react";
import type { Action } from "./bindings/Action";
import type { Campaign } from "./bindings/Campaign";
import type { Catalog, Move } from "./catalog";
import { api, errorOf } from "./api";

export default function AdvancedTargeting({
  campaign,
  catalog,
  playerId,
  execute,
  disabled,
}: {
  campaign: Campaign;
  catalog: Catalog;
  playerId: string;
  execute: (a: Action) => Promise<boolean>;
  disabled: boolean;
}) {
  const [actor, setActor] = useState(""),
    [move, setMove] = useState("");
  const [targets, setTargets] = useState<string[]>([]),
    [reason, setReason] = useState("");
  const [notice, setNotice] = useState("");
  const [m, setMoveDetail] = useState<Move | null>(null);
  useEffect(() => {
    let stale = false;
    setMoveDetail(null);
    const entry = catalog.moves.find((v) => v.Name === move);
    if (entry)
      api
        .catalogEntry<Move>("moves", entry.AppId)
        .then((v) => {
          if (!stale) setMoveDetail(v);
        })
        .catch((e) => {
          if (!stale) setNotice(errorOf(e).message);
        });
    return () => {
      stale = true;
    };
  }, [move, catalog]);
  const b = campaign.battle;
  if (
    !["0.4.0-core", "0.5.0-core", "0.6.0-core"].includes(campaign.engine) ||
    !b?.active
  )
    return null;
  const gm = campaign.gm_id === playerId;
  const active = b.combatants.filter(
    (p) => p.deployed && p.hp > 0 && p.available_round <= b.round,
  );
  const p = active.find((p) => p.id === actor);
  const sheet = p && campaign.sheets[p.sheet_id];
  const automatic = m?.Target === "Area" || m?.Target?.includes("Battlefield");
  const toggle = (id: string) =>
    setTargets((v) =>
      v.includes(id) ? v.filter((k) => k !== id) : [...v, id],
    );
  const action: Action = {
    type: "accuracy_targets",
    combatant_id: actor,
    target_ids: automatic ? [] : targets,
    move_name: move,
    pool_modifier: 0,
    success_modifier: 0,
    reason: reason.trim() || `${move} targeting`,
  };
  async function preview() {
    try {
      const v = await api.previewBattleAction(campaign, action);
      setNotice(`${v.dice} host accuracy dice. ${v.warnings.join(" ")}`);
    } catch (e) {
      setNotice(errorOf(e).message);
    }
  }
  return (
    <section className="panel" aria-label="Advanced battle targeting">
      <h3>Multiple targets and battlefield moves</h3>
      <p>
        One accuracy roll and one action for the whole move. The host enforces
        rank limits. Damage shares the same dice, progressing from highest
        defense to lowest. Area moves automatically include foes first, then
        allies in range.
      </p>
      <div className="form-grid">
        <label>
          Acting Pokémon
          <select
            value={actor}
            onChange={(e) => {
              setActor(e.target.value);
              setMove("");
              setTargets([]);
              setNotice("");
            }}
          >
            <option value="">Choose a Pokémon</option>
            {active
              .filter((p) => gm || p.owner_id === playerId)
              .map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
          </select>
        </label>
        <label>
          Move
          <select
            value={move}
            onChange={(e) => {
              setMove(e.target.value);
              setTargets([]);
              setNotice("");
            }}
          >
            <option value="">Choose a move</option>
            {sheet?.moves.map((name) => (
              <option key={name}>{name}</option>
            ))}
          </select>
        </label>
        <label>
          Decision reason
          <input
            value={reason}
            onChange={(e) => setReason(e.target.value)}
            maxLength={1000}
          />
        </label>
      </div>
      {m && (
        <p>
          Target pattern: {m.Target}.{" "}
          {automatic
            ? "Targets are determined by field scope or the GM-confirmed range."
            : "Select the affected Pokémon below; user-and-allies moves must include the user."}
        </p>
      )}
      <fieldset disabled={disabled || !actor}>
        <legend>{gm ? "Targets / range selection" : "Targets"}</legend>
        <div className="toolbar">
          {active.map((t) => (
            <label key={t.id} className="checkbox">
              <input
                type="checkbox"
                checked={targets.includes(t.id)}
                onChange={() => toggle(t.id)}
                disabled={automatic && !gm}
              />
              {t.name} · {t.side}
            </label>
          ))}
        </div>
      </fieldset>
      {p && (
        <p>
          Current range:{" "}
          {b.flow?.ranges[actor]
            ?.map((id) => active.find((v) => v.id === id)?.name || id)
            .join(", ") ?? "All active Pokémon"}
          . Range is a GM ruling; no map distance is inferred.
        </p>
      )}
      {notice && (
        <p role="status" className="notice">
          {notice}
        </p>
      )}
      <div className="toolbar">
        <button disabled={disabled || !actor || !m} onClick={preview}>
          Preview target legality and accuracy
        </button>
        <button
          className="primary"
          disabled={disabled || !actor || !m || !!b.flow?.queue.length}
          onClick={() => execute(action)}
        >
          Roll targeted move
        </button>
        {gm && (
          <button
            disabled={
              disabled || !actor || !!b.flow?.queue.length || !reason.trim()
            }
            onClick={() =>
              execute({
                type: "set_battle_range",
                combatant_id: actor,
                in_range_ids: targets,
                reason,
              })
            }
          >
            Confirm selected Pokémon in range
          </button>
        )}
      </div>
    </section>
  );
}
