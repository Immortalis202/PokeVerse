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
  const b = campaign.battle,
    f = b?.flow;
  if (!b?.active || !f?.ability_rules) return null;
  const gm = campaign.gm_id === playerId;
  const event = f.ability_events[0];
  const owner = event && b.combatants.find((p) => p.id === event.combatant_id);
  const target = event && b.combatants.find((p) => p.id === event.target_id);
  const owned = owner && (gm || owner.owner_id === playerId);
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
                            : "Decline healing"}
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
