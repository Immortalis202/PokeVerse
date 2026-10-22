import { useState } from "react";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";
import type { CompletionCommand } from "./bindings/CompletionCommand";
const medicines = new Set([
  "Potion",
  "Super Potion",
  "Hyper Potion",
  "Max Potion",
  "Full Restore",
  "Antidote",
  "Awakening",
  "Burn Heal",
  "Ice Heal",
  "Paralyze Heal",
  "Full Heal",
  "Energy Root",
  "Remedy",
  "Heal Powder",
  "Revive",
  "Max Revive",
  "Revival Herb",
]);
export default function ItemCompletionControls({
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
  const [source, setSource] = useState("");
  const [unit, setUnit] = useState("");
  const [value, setValue] = useState("");
  const [recipient, setRecipient] = useState("");
  const [supply, setSupply] = useState("");
  const [medicine, setMedicine] = useState("");
  const [dose, setDose] = useState(1);
  const [trainer, setTrainer] = useState(false);
  const [reason, setReason] = useState("");
  const [notice, setNotice] = useState("");
  const [rule, setRule] = useState("revive");
  const [ruling, setRuling] = useState("full");
  const b = campaign.battle,
    f = b?.flow,
    items = f?.items,
    state = items?.completion;
  const gm = campaign.gm_id === playerId;
  if (!b?.active || !state?.enabled || !items) return null;
  const actors = b.combatants.filter(
    (p) => p.deployed && p.hp > 0 && (gm || p.owner_id === playerId),
  );
  const actor = actors.find((p) => p.id === source);
  const sheet = actor ? campaign.sheets[actor.sheet_id] : undefined;
  const supplies = [
    sheet,
    sheet?.trainer_id ? campaign.sheets[sheet.trainer_id] : undefined,
  ].filter((s) => s !== undefined);
  const supplySheet = supplies.find((s) => s.id === supply);
  const held = Object.values(items.units).filter(
    (u) => u && u.holder === source && u.state === "held",
  );
  const selected = items.units[unit];
  const targets = b.combatants.filter(
    (p) => actor && p.side === actor.side && (gm || p.owner_id === playerId),
  );
  const inventory = Object.keys(supplySheet?.inventory ?? {}).filter(
    (n) => medicines.has(n) && (supplySheet?.inventory[n] ?? 0) > 0,
  );
  const stored = (supplySheet?.stored_items ?? []).filter((u) =>
    medicines.has(u.name),
  );
  const partial = stored.find((u) => u.id === medicine);
  const busy = disabled || !reason.trim();
  async function send(command: CompletionCommand) {
    setNotice("");
    try {
      await execute({
        type: "held_item",
        command: { kind: "completion", command },
        reason,
      });
    } catch (e) {
      setNotice(String(e));
    }
  }
  return (
    <details className="panel">
      <summary>Reviewed item choices and medicine</summary>
      <label>
        Reason
        <input value={reason} onChange={(e) => setReason(e.target.value)} />
      </label>
      <label>
        Acting Pokémon
        <select
          value={source}
          onChange={(e) => {
            setSource(e.target.value);
            setUnit("");
            setSupply("");
            setMedicine("");
            setValue("");
          }}
        >
          <option value="">Choose</option>
          {actors.map((p) => (
            <option key={p.id} value={p.id}>
              {p.name}
            </option>
          ))}
        </select>
      </label>
      <label>
        Held unit
        <select
          value={unit}
          onChange={(e) => {
            setUnit(e.target.value);
            setValue("");
          }}
        >
          <option value="">Choose</option>
          {held.map(
            (u) =>
              u && (
                <option key={u.unit.id} value={u.unit.id}>
                  {u.unit.name}
                </option>
              ),
          )}
        </select>
      </label>
      {selected &&
        [
          "Choice Band",
          "Choice Scarf",
          "Choice Specs",
          "Power Increasers",
        ].includes(selected.unit.name) && (
          <>
            <label>
              Move or trait
              <select value={value} onChange={(e) => setValue(e.target.value)}>
                <option value="">Choose</option>
                {(selected.unit.name === "Power Increasers"
                  ? [
                      "Strength",
                      "Vitality",
                      "Special",
                      "Insight",
                      "Defense",
                      "SpecialDefense",
                      "AccuracyPool",
                      "EvasionPool",
                      "Initiative",
                    ]
                  : (sheet?.moves ?? [])
                ).map((n) => (
                  <option key={n}>{n}</option>
                ))}
              </select>
            </label>
            <p>
              Recorded choice: {state.choices[unit] ?? "none"}. Changes after
              selection require a GM timing ruling.
            </p>
            <button
              disabled={
                busy ||
                !source ||
                !unit ||
                !value ||
                !!f?.queue.length ||
                (!gm && !!state.choices[unit])
              }
              onClick={() =>
                send({
                  operation: "select",
                  combatant_id: source,
                  unit_id: unit,
                  value,
                })
              }
            >
              Record held-item choice
            </button>
          </>
        )}
      <h4>Medicine</h4>
      <label>
        Supplies
        <select
          value={supply}
          onChange={(e) => {
            setSupply(e.target.value);
            setMedicine("");
          }}
        >
          <option value="">Choose</option>
          {supplies.map((s) => (
            <option key={s.id} value={s.id}>
              {s.name}
            </option>
          ))}
        </select>
      </label>
      <label>
        Owned medicine
        <select value={medicine} onChange={(e) => setMedicine(e.target.value)}>
          <option value="">Choose</option>
          {inventory.map((n) => (
            <option key={n}>{n}</option>
          ))}
          {stored.map((u) => (
            <option key={u.id} value={u.id}>
              {u.name} · {u.uses} units already spent
            </option>
          ))}
        </select>
      </label>
      <label>
        Recipient
        <select
          value={recipient}
          onChange={(e) => setRecipient(e.target.value)}
        >
          <option value="">Choose</option>
          {targets.map((p) => (
            <option key={p.id} value={p.id}>
              {p.name} · {p.hp}/{p.max_hp} HP{!p.deployed ? " · reserve" : ""}
            </option>
          ))}
        </select>
      </label>
      <label>
        Dose
        <input
          type="number"
          min={1}
          max={99}
          value={dose}
          onChange={(e) => setDose(Number(e.target.value))}
        />
      </label>
      {gm && (
        <label>
          <input
            type="checkbox"
            checked={trainer}
            onChange={(e) => setTrainer(e.target.checked)}
          />
          Trainer is on the battlefield and pays their own action
        </label>
      )}
      <p>
        Use during the acting Pokémon's turn. Normally it pays one action.
        GM-confirmed supplies must be accessible in the small battle pocket.
        Ordinary healing shares the three-HP round limit; partial doses remain
        in supplies. Potions cannot revive.
      </p>
      <button
        disabled={
          busy ||
          !actor ||
          !recipient ||
          !supply ||
          !medicine ||
          !Number.isInteger(dose) ||
          dose < 1 ||
          !!f?.queue.length
        }
        onClick={() =>
          send({
            operation: "medicine",
            combatant_id: source,
            target_id: recipient,
            supply_sheet_id: supply,
            unit_id: partial?.id ?? null,
            name: partial?.name ?? medicine,
            dose,
            trainer_on_field: gm && trainer,
          })
        }
      >
        Use owned medicine
      </button>
      <button
        disabled={busy || !actor || !recipient || !!f?.queue.length}
        onClick={() =>
          send({
            operation: "recycle_recipient",
            combatant_id: source,
            target_id: recipient,
          })
        }
      >
        Use this recipient for Recycle
      </button>
      {gm && (
        <>
          <h4>Printed-rule decisions</h4>
          <label>
            Decision
            <select
              value={rule}
              onChange={(e) => {
                setRule(e.target.value);
                setRuling(
                  e.target.value === "revive"
                    ? "full"
                    : e.target.value === "ring"
                      ? "types"
                      : e.target.value === "herbal"
                        ? "accepted"
                        : e.target.value === "scarf"
                          ? "printed"
                          : e.target.value === "recycle-balloon"
                            ? "until-damage"
                            : "accessible",
                );
              }}
            >
              <option value="revive">Full-HP revive versus round cap</option>
              <option value="ring">Ring Target immunity scope</option>
              <option value="herbal">
                Herbal acceptance and happiness ruling
              </option>
              <option value="scarf">Choice Scarf conflicting timing</option>
              <option value="pocket">Accessible battle-pocket supplies</option>
              <option value="recycle-balloon">Recycled balloon duration</option>
            </select>
          </label>
          <label>
            Interpretation
            <select value={ruling} onChange={(e) => setRuling(e.target.value)}>
              {(rule === "revive"
                ? ["full", "capped"]
                : rule === "ring"
                  ? ["types", "reviewed"]
                  : rule === "herbal"
                    ? ["accepted"]
                    : rule === "scarf"
                      ? ["printed", "item"]
                      : rule === "recycle-balloon"
                        ? ["until-damage"]
                        : ["accessible"]
              ).map((n) => (
                <option key={n}>{n}</option>
              ))}
            </select>
          </label>
          <p>
            Ring Target: “types” removes type-chart immunities; “reviewed” also
            removes supported non-unique ability and field immunities.
            Conflicting unique rules and undefined effects still require
            adjudication. Record herbal acceptance and any narrative happiness
            consequence in the reason; no penalty amount is invented.
          </p>
          <button
            disabled={
              busy ||
              !!f?.queue.length ||
              (rule === "ring" && !source) ||
              (rule === "herbal" && !recipient) ||
              (rule === "scarf" && !unit) ||
              (rule === "pocket" && !supply)
            }
            onClick={() =>
              send({
                operation: "rule",
                key:
                  rule === "revive" || rule === "recycle-balloon"
                    ? rule
                    : rule === "ring"
                      ? `ring:${source}`
                      : rule === "herbal"
                        ? `herbal:${recipient}`
                        : rule === "scarf"
                          ? `scarf:${unit}`
                          : `pocket:${supply}`,
                value: ruling,
              })
            }
          >
            Record GM item ruling
          </button>
        </>
      )}
      {notice && <p role="alert">{notice}</p>}
    </details>
  );
}
