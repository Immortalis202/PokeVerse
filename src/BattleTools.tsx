import { useEffect, useState } from "react";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";
import { attributes } from "./catalog";

export default function BattleTools({
  campaign,
  disabled,
  execute,
}: {
  campaign: Campaign;
  disabled: boolean;
  execute: (action: Action) => Promise<boolean>;
}) {
  const [reason, setReason] = useState("");
  const [selected, setSelected] = useState("");
  const [stats, setStats] = useState<Record<string, number>>({});
  const [skills, setSkills] = useState<Record<string, number>>({});
  const [maxHp, setMaxHp] = useState(1);
  const [statReason, setStatReason] = useState("");
  useEffect(() => {
    setSelected("");
    setReason("");
    setStatReason("");
  }, [campaign.battle?.id]);
  const b = campaign.battle;
  if (!b) return null;
  const p = b.combatants.find((p) => p.id === selected);
  const sheet = p ? campaign.sheets[p.sheet_id] : null;
  const undo = campaign.battle_undo.at(-1);
  const skillNames = [
    ...new Set([
      "Alert",
      "Athletic",
      "Brawl",
      "Channel",
      "Clash",
      "Evasion",
      "Nature",
      ...Object.keys(sheet?.skills ?? {}),
      ...Object.keys(skills),
    ]),
  ];
  return (
    <div className="panel">
      <h3>Battle configuration & recovery</h3>
      <p>
        Automatic checks run on the server without asking the host player for
        approval. This covers core accuracy and ordinary damage; reactions and
        special effects still need GM adjudication.
      </p>
      <label>
        Reason for configuration change or rollback
        <input
          value={reason}
          maxLength={1000}
          onChange={(e) => setReason(e.target.value)}
          placeholder="Explain the rule exception or correction…"
        />
      </label>
      <label className="checkbox">
        <input
          type="checkbox"
          checked={b.automatic_validation}
          disabled={disabled || !b.active || !reason.trim()}
          onChange={(e) =>
            execute({
              type: "battle_config",
              automatic_validation: e.target.checked,
              reason,
            })
          }
        />
        Automatic battle rule checks
      </label>
      <small>
        On: checks action limits, fainted actors, learned moves and repeated
        moves. Off: allows manual exceptions. Permissions and numeric bounds
        always apply. A reason is required to change this setting.
      </small>
      <div className="inline roll-controls">
        <button
          disabled={disabled || !undo || !reason.trim()}
          onClick={async () => {
            if (
              undo &&
              (await execute({
                type: "rollback_battle",
                event_sequence: undo.event_sequence,
                reason,
              }))
            )
              setSelected("");
          }}
        >
          Roll back last battle step
        </button>
        <span>{campaign.battle_undo.length} steps available (up to 20)</span>
      </div>
      {undo && (
        <p className="muted">
          Next rollback: #{undo.event_sequence} · {undo.description}. Logs and
          dice are retained. End battle can be undone until another campaign
          operation clears recovery history.
        </p>
      )}
      {b.active && (
        <details>
          <summary>
            Correct battle-only attributes, skills or maximum HP
          </summary>
          <label>
            Combatant
            <select
              value={selected}
              disabled={disabled}
              onChange={(e) => {
                const combatant = b.combatants.find(
                  (p) => p.id === e.target.value,
                );
                setSelected(e.target.value);
                setStats(
                  Object.fromEntries(
                    Object.entries(combatant?.stat_overrides ?? {}).filter(
                      (entry): entry is [string, number] =>
                        entry[1] !== undefined,
                    ),
                  ),
                );
                setSkills(
                  Object.fromEntries(
                    Object.entries(combatant?.skill_overrides ?? {}).filter(
                      (entry): entry is [string, number] =>
                        entry[1] !== undefined,
                    ),
                  ),
                );
                setMaxHp(combatant?.max_hp ?? 1);
                setStatReason("");
              }}
            >
              <option value="">Select combatant</option>
              {b.combatants
                .filter((p) => p.deployed)
                .map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name}
                  </option>
                ))}
            </select>
          </label>
          {p && sheet && (
            <>
              <p>
                Corrections affect this battle's roll calculations. Permanent
                attributes, skills and maximum HP are preserved. Lowering
                maximum HP caps current battle HP.
              </p>
              <div className="form-grid">
                {attributes.map((key) => (
                  <label key={key}>
                    {key}
                    <input
                      type="number"
                      min={0}
                      max={999}
                      value={stats[key] ?? sheet.stats[key] ?? 0}
                      disabled={disabled}
                      onChange={(e) =>
                        setStats({ ...stats, [key]: Number(e.target.value) })
                      }
                    />
                  </label>
                ))}
                <label>
                  Battle maximum HP
                  <input
                    type="number"
                    min={1}
                    max={999}
                    value={maxHp}
                    disabled={disabled}
                    onChange={(e) => setMaxHp(Number(e.target.value))}
                  />
                </label>
                {skillNames.map((key) => (
                  <label key={key}>
                    {key} skill
                    <input
                      type="number"
                      min={0}
                      max={999}
                      value={skills[key] ?? sheet.skills[key] ?? 0}
                      disabled={disabled}
                      onChange={(e) =>
                        setSkills({ ...skills, [key]: Number(e.target.value) })
                      }
                    />
                  </label>
                ))}
              </div>
              <label>
                Correction reason
                <input
                  value={statReason}
                  maxLength={1000}
                  onChange={(e) => setStatReason(e.target.value)}
                />
              </label>
              <div className="inline roll-controls">
                <button
                  disabled={disabled}
                  onClick={() => {
                    setStats({});
                    setSkills({});
                    setMaxHp(sheet.max_hp);
                  }}
                >
                  Reset draft to character sheet
                </button>
                <button
                  disabled={disabled || !statReason.trim()}
                  onClick={async () => {
                    if (
                      await execute({
                        type: "battle_stats",
                        combatant_id: p.id,
                        stats,
                        skills,
                        max_hp: maxHp,
                        reason: statReason,
                      })
                    ) {
                      setSelected("");
                      setStatReason("");
                    }
                  }}
                >
                  Apply battle-only correction
                </button>
              </div>
            </>
          )}
        </details>
      )}
    </div>
  );
}
