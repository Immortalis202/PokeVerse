import { useState } from "react";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";
import { api, errorOf } from "./api";

export default function AbilityControls({
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
  const [reason, setReason] = useState(""),
    [notice, setNotice] = useState("");
  const [statSource, setStatSource] = useState(""),
    [statTarget, setStatTarget] = useState(""),
    [stat, setStat] = useState("Strength"),
    [change, setChange] = useState(-1);
  const b = campaign.battle,
    f = b?.flow;
  if (!b?.active || !f?.ability_rules) return null;
  const gm = campaign.gm_id === playerId;
  const event = f.ability_events[0];
  const owner = event && b.combatants.find((p) => p.id === event.combatant_id);
  const target = event && b.combatants.find((p) => p.id === event.target_id);
  const owned =
    owner &&
    (gm || (event?.kind !== "extra-download" && owner.owner_id === playerId));
  const command = (choice: string): Action => ({
    type: "resolve_ability",
    event_id: event!.id,
    choice,
    reason: reason.trim() || "Resolve the recorded ability trigger",
  });
  async function preview(choice: string) {
    try {
      const p = await api.previewBattleAction(campaign, command(choice));
      setNotice(
        `${p.dice} host dice; no battle action spent. ${p.warnings.join(" ")}`,
      );
    } catch (e) {
      setNotice(errorOf(e).message);
    }
  }
  return (
    <section className="panel" aria-label="Battle ability automation">
      <h3>Abilities</h3>
      <p>
        Selected battle abilities are fixed for this encounter. Supported
        passive effects apply automatically. Unsupported interactions require a
        GM ruling.
      </p>
      <div className="toolbar">
        {b.combatants
          .filter((p) => p.deployed)
          .map((p) => (
            <div key={p.id}>
              <strong>
                {p.name}: {p.selected_ability || "No ability"}
              </strong>
              <p>
                {Object.entries(p.runtime.ability.counters)
                  .map(([name, value]) => `${name}: ${value}`)
                  .join(" · ")}
                {p.runtime.ability.flags.length === 0
                  ? ""
                  : p.runtime.ability.flags.join(" · ")}
              </p>
            </div>
          ))}
      </div>
      {gm && f.extra?.enabled && (
        <details>
          <summary>Sourced battle stat ruling</summary>
          <p>
            Record the source of a manual stat effect so protections and
            reactive abilities can resolve. Permanent sheets are unchanged.
          </p>
          <div className="form-grid">
            <label>
              Source Pokémon
              <select
                value={statSource}
                onChange={(e) => setStatSource(e.target.value)}
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
              Affected Pokémon
              <select
                value={statTarget}
                onChange={(e) => setStatTarget(e.target.value)}
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
              Attribute / defense
              <select value={stat} onChange={(e) => setStat(e.target.value)}>
                {[
                  "Strength",
                  "Dexterity",
                  "Vitality",
                  "Special",
                  "Insight",
                  "Defense",
                  "SpecialDefense",
                ].map((k) => (
                  <option key={k}>{k}</option>
                ))}
              </select>
            </label>
            <label>
              Change
              <input
                type="number"
                min={-3}
                max={3}
                value={change}
                onChange={(e) => setChange(Number(e.target.value))}
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
          <button
            disabled={
              disabled ||
              !!event ||
              !statSource ||
              !statTarget ||
              !reason.trim() ||
              change === 0
            }
            onClick={() =>
              execute({
                type: "battle_stat",
                source_id: statSource,
                combatant_id: statTarget,
                stat,
                change,
                reason,
              })
            }
          >
            Apply sourced stat effect
          </button>
        </details>
      )}
      {event && (
        <div className="notice" role="status">
          <h4>Pending: {event.ability}</h4>
          <p>
            {owner?.name} → {target?.name} after {event.move_name}. Resolve this
            trigger before continuing the battle. It does not spend another
            action.
          </p>
          {owned && (
            <>
              <label>
                Ability decision / GM ruling
                <input
                  value={reason}
                  onChange={(e) => setReason(e.target.value)}
                  maxLength={1000}
                />
              </label>
              <div className="toolbar">
                {event.choices.map((choice) => (
                  <span key={choice}>
                    <button disabled={disabled} onClick={() => preview(choice)}>
                      Preview {choice}
                    </button>
                    <button
                      disabled={disabled}
                      onClick={() => execute(command(choice))}
                    >
                      {choice === "neutral"
                        ? "Deal 1 neutral damage"
                        : choice === "roll"
                          ? "Roll ability dice"
                          : choice === "heal"
                            ? "Restore 1 HP"
                            : choice === "decline"
                              ? "Decline"
                              : choice === "activate"
                                ? "Activate No Guard"
                                : (b.combatants.find((p) => p.id === choice)
                                    ?.name ?? `Select ${choice}`)}
                    </button>
                  </span>
                ))}
                {gm && (
                  <button
                    disabled={disabled || !reason.trim()}
                    onClick={() => execute(command("manual"))}
                  >
                    Record manual ability resolution
                  </button>
                )}
              </div>
            </>
          )}
          {notice && <p>{notice}</p>}
        </div>
      )}
    </section>
  );
}
