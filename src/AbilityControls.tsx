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
    (gm ||
      (!["extra-download", "ability-gas", "transfer-review"].includes(
        event?.kind ?? "",
      ) &&
        owner.owner_id === playerId));
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
        Permanent abilities stay on the sheet. Temporary copying, swapping and
        suppression are recorded below. Supported passive effects apply
        automatically. Unsupported interactions require a GM ruling.
      </p>
      {f.transfers?.enabled && (
        <details>
          <summary>Contact transfer history</summary>
          {f.transfers.history.map((entry, i) => (
            <p key={i}>{entry}</p>
          ))}
        </details>
      )}
      {f.ability_changes?.enabled && (
        <details>
          <summary>Temporary ability history</summary>
          {f.ability_changes.history.map((entry, i) => (
            <p key={i}>{entry}</p>
          ))}
        </details>
      )}
      <div className="toolbar">
        {b.combatants
          .filter((p) => p.deployed)
          .map((p) => (
            <div key={p.id}>
              <strong>
                {p.name}:{" "}
                {p.selected_ability ||
                  (f.ability_changes?.suppressed.includes(p.id)
                    ? "Ability negated"
                    : "No ability")}
              </strong>
              {f.ability_changes?.enabled && (
                <p>
                  Original: {f.ability_changes.originals[p.id] || "None"}
                  {f.ability_changes.assignments[p.id]
                    ? ` · Assigned: ${f.ability_changes.assignments[p.id]}`
                    : ""}
                </p>
              )}
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
            {owner?.name} →{" "}
            {event.kind === "ability-trace"
              ? "random eligible foe"
              : event.kind === "ability-gas"
                ? "opposing Gas sources"
                : target?.name}{" "}
            after {event.move_name}. Resolve this trigger before continuing the
            battle. It does not spend another action.
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
                      {choice.startsWith("activate:")
                        ? `Copy ${f.transfers?.offers[event.id]?.donors[Number(choice.slice(9))]?.ability ?? "ability"} from ${f.transfers?.offers[event.id]?.donors[Number(choice.slice(9))]?.name ?? "ally"}`
                        : choice === "store"
                          ? "Store all copies; keep Power of Alchemy active"
                          : choice === "neutral"
                            ? "Deal 1 neutral damage"
                            : choice === "roll"
                              ? "Roll ability dice"
                              : choice === "heal"
                                ? "Restore 1 HP"
                                : choice === "decline"
                                  ? "Decline"
                                  : choice === "simultaneous"
                                    ? "Confirm both Gas emitters remain active"
                                    : choice === "activate"
                                      ? "Activate No Guard"
                                      : (b.combatants.find(
                                          (p) => p.id === choice,
                                        )?.name ?? `Select ${choice}`)}
                    </button>
                  </span>
                ))}
                {gm && event.kind !== "ability-gas" && (
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
