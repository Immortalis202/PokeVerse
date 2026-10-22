import { useState } from "react";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";
import type { BattleProfile } from "./bindings/BattleProfile";
import type { StandaloneCommand } from "./bindings/StandaloneCommand";
function ProfileEditor({
  id,
  name,
  profile,
  send,
  disabled,
}: {
  id: string;
  name: string;
  profile?: BattleProfile;
  send: (c: StandaloneCommand) => void;
  disabled: boolean;
}) {
  const [gender, setGender] = useState(profile?.gender ?? "unknown"),
    [preference, setPreference] = useState(profile?.preference ?? "unknown"),
    [weight, setWeight] = useState(profile?.weight_kg?.toString() ?? ""),
    [metal, setMetal] = useState(profile?.metal_weight_kg?.toString() ?? "");
  return (
    <details>
      <summary>{name}: weight and Rivalry inputs</summary>
      <label>
        Gender (supplied JSON)
        <select value={gender} onChange={(e) => setGender(e.target.value)}>
          {["unknown", "genderless", "male", "female"].map((v) => (
            <option key={v}>{v}</option>
          ))}
        </select>
      </label>
      <label>
        Preference comparison (printed rule)
        <select
          value={preference}
          onChange={(e) => setPreference(e.target.value)}
        >
          <option value="unknown">Unknown</option>
          <option value="neutral">Neutral</option>
          <option value="male">Group A</option>
          <option value="female">Group B</option>
        </select>
      </label>
      <label>
        Ordinary encounter weight, kg (blank uses catalog / recorded move
        parameters)
        <input
          type="number"
          min="0.001"
          max="100000"
          step="any"
          value={weight}
          onChange={(e) => setWeight(e.target.value)}
        />
      </label>
      <label>
        GM Heavy/Light Metal weight, kg (exact current form)
        <input
          type="number"
          min="0.001"
          max="100000"
          step="any"
          value={metal}
          onChange={(e) => setMetal(e.target.value)}
        />
      </label>
      <button
        disabled={disabled}
        onClick={() =>
          send({
            operation: "profile",
            combatant_id: id,
            gender,
            preference,
            weight_kg: weight ? Number(weight) : null,
            metal_weight_kg: metal ? Number(metal) : null,
          })
        }
      >
        Record profile for {name}
      </button>
    </details>
  );
}
export default function StandaloneControls({
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
  const [reason, setReason] = useState("");
  const b = campaign.battle,
    f = b?.flow,
    state = f?.standalone;
  if (!b?.active || !state?.enabled) return null;
  const gm = campaign.gm_id === playerId;
  const send = (command: StandaloneCommand) =>
    void execute({ type: "standalone", command, reason });
  return (
    <section className="panel" aria-label="Standalone battle abilities">
      <h3>Standalone battle abilities</h3>
      <p>
        Heavy/Light Metal use a recorded weight, with no fixed multiplier.
        Rivalry uses a recorded wording choice and individual comparisons. Moody
        rolls two host dice with rejection sampling for distinct, unbiased
        attributes. Perish Body expires after three full rounds unless the
        cursed Pokémon leaves battle.
      </p>
      <p>
        Torrent:{" "}
        {state.torrent_bonus === null
          ? "GM must choose the printed one- or two-die bonus"
          : `${state.torrent_bonus} extra Water damage dice at half HP or less`}
        . Rivalry basis: {state.rivalry_basis ?? "GM ruling required"}.
      </p>
      {Object.entries(state.perish).map(([id, curse]) => (
        <p key={id}>
          <strong>
            {b.combatants.find((p) => p.id === id)?.name ?? id}: Perish Body
          </strong>{" "}
          — remaining-HP damage at end of round {curse!.due_round} (
          {Math.max(0, curse!.due_round - b.round)} rounds remaining). Departure
          cancels this countdown.
        </p>
      ))}
      {gm && (
        <details>
          <summary>GM weight, comparison and Torrent rulings</summary>
          <label>
            Rule evidence / decision
            <input
              maxLength={1000}
              value={reason}
              onChange={(e) => setReason(e.target.value)}
            />
          </label>
          <p>
            Rivalry's supplied JSON says gender; printed page 564 says
            preference. Unknown, genderless or neutral cases receive no
            automatic change. No species ratio assigns an individual gender.
          </p>
          <button
            disabled={disabled || !reason.trim()}
            onClick={() =>
              send({ operation: "rivalry_basis", basis: "gender" })
            }
          >
            Use supplied-card gender wording
          </button>
          <button
            disabled={disabled || !reason.trim()}
            onClick={() =>
              send({ operation: "rivalry_basis", basis: "preference" })
            }
          >
            Use printed preference wording
          </button>
          <p>
            Torrent's text grants two dice; its icon grants one. Water pain
            immunity applies at every HP level.
          </p>
          {[1, 2].map((bonus) => (
            <button
              key={bonus}
              disabled={disabled || !reason.trim()}
              onClick={() => send({ operation: "torrent", bonus })}
            >
              Record Torrent: {bonus} {bonus === 1 ? "die" : "dice"}
            </button>
          ))}
          {b.combatants.map((p) => (
            <ProfileEditor
              key={`${p.id}:${JSON.stringify(state.profiles[p.id])}`}
              id={p.id}
              name={p.name}
              profile={state.profiles[p.id]}
              send={send}
              disabled={disabled || !reason.trim()}
            />
          ))}
        </details>
      )}
      <details>
        <summary>Recorded ability history</summary>
        {state.history.slice(-15).map((entry, i) => (
          <p key={i}>{entry}</p>
        ))}
      </details>
    </section>
  );
}
