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
      {f.provenance_rules &&
        b.combatants
          .filter(
            (p) =>
              p.deployed &&
              p.hp > 0 &&
              p.selected_ability === "Sheer Force" &&
              (gm || p.owner_id === playerId),
          )
          .map((p) => {
            const damage =
              p.runtime.ability.flags.includes("sheer-force:damage");
            return (
              <p key={p.id}>
                {p.name}: Sheer Force —{" "}
                {damage
                  ? "trade added effects for two damage dice"
                  : "keep added effects"}
                <button
                  disabled={
                    disabled ||
                    f.queue.length > 0 ||
                    f.ability_events.length > 0
                  }
                  onClick={() =>
                    void execute({
                      type: "choose_sheer_force",
                      combatant_id: p.id,
                      ignore_added_effects: !damage,
                      reason:
                        reason.trim() ||
                        "Choose Sheer Force for subsequent attacks",
                    })
                  }
                >
                  {damage
                    ? "Keep added effects"
                    : "Trade added effects for damage"}
                </button>
              </p>
            );
          })}
      {f.provenance_rules && (
        <details>
          <summary>Damage provenance</summary>
          <p>
            Tinted Lens ignores one foe resistance. Unaware ignores foe
            attribute changes when dealing or receiving damage.
          </p>
          {b.combatants.flatMap((p) =>
            p.runtime.ability.flags
              .filter((flag) => flag.startsWith("stakeout:"))
              .map((flag) => (
                <p key={`${p.id}:${flag}`}>
                  {p.name}: Stakeout awaits the first successful attack on{" "}
                  {b.combatants.find((t) => t.id === flag.slice(9))?.name ??
                    "the incoming foe"}
                  .
                </p>
              )),
          )}
        </details>
      )}
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
      {f.stat_channels?.enabled && (
        <details>
          <summary>Stat copying and transfer history</summary>
          {f.stat_channels.history.map((entry, i) => (
            <p key={i}>{entry}</p>
          ))}
        </details>
      )}
      {f.sequencing?.enabled && (
        <details>
          <summary>Action sequencing</summary>
          <p>
            Loyalty checks are included with the declared action. Parental Bond
            keeps the higher of two damage rolls and deals damage once. Dancer
            copies Accuracy successes and spends no action.
          </p>
          {b.combatants
            .filter((p) => p.deployed)
            .map((p) => (
              <p key={p.id}>
                {p.name}
                {f.sequencing.gorilla[p.id]
                  ? `: Gorilla Tactics - ${f.sequencing.gorilla[p.id]}`
                  : ""}
                {p.selected_ability === "Dancer"
                  ? f.sequencing.dancer_round[p.id] === b.round
                    ? ": free dance used this round"
                    : ": free dance available"
                  : ""}
                {p.selected_ability === "Truant"
                  ? `: personal turn ${f.sequencing.visits[p.id] ?? 1}; even turns require Loyalty`
                  : ""}
              </p>
            ))}
          {f.sequencing.history.map((entry, i) => (
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
          {event.kind === "sequence-dancer-random" && (
            <p>
              The GM selects the random foe; the owner may decline the free
              dance.
            </p>
          )}
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
                    <button
                      disabled={
                        disabled ||
                        (event.kind === "sequence-dancer-random" &&
                          !gm &&
                          choice !== "decline")
                      }
                      onClick={() => preview(choice)}
                    >
                      Preview {choice}
                    </button>
                    <button
                      disabled={
                        disabled ||
                        (event.kind === "sequence-dancer-random" &&
                          !gm &&
                          choice !== "decline")
                      }
                      onClick={() => execute(command(choice))}
                    >
                      {choice.startsWith("activate:")
                        ? `Copy ${f.transfers?.offers[event.id]?.donors[Number(choice.slice(9))]?.ability ?? "ability"} from ${f.transfers?.offers[event.id]?.donors[Number(choice.slice(9))]?.name ?? "ally"}`
                        : choice === "store"
                          ? "Store all copies; keep Power of Alchemy active"
                          : choice === "neutral"
                            ? "Deal 1 neutral damage"
                            : choice === "copy"
                              ? "Copy this dance"
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
                    {event.kind === "stat-costar" && (
                      <small>
                        {(f.stat_channels?.choices[event.id]?.[choice] ?? [])
                          .map((g) => `${g.key} +${g.positive} (${g.channel})`)
                          .join(" � ")}
                      </small>
                    )}
                  </span>
                ))}
                {gm &&
                  !["ability-gas", "sequence-gorilla"].includes(event.kind) && (
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
