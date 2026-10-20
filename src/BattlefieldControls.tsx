import { useState } from "react";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";
import type { FieldCommand } from "./bindings/FieldCommand";
import type { Weather } from "./bindings/Weather";
import { api, errorOf } from "./api";

const weatherNames: Weather[] = [
  "Sunny",
  "Rain",
  "Sandstorm",
  "Snow",
  "Hail",
  "Desolate",
  "Typhoon",
  "StrongWinds",
];
const presets = [
  "fog",
  "mud",
  "underwater",
  "fire",
  "electric-poles",
  "flowers",
  "sewage",
  "jungle",
  "high-poles",
  "sprinklers",
  "cemetery",
  "minefield",
  "healing-pits",
  "torn-world",
  "final-destination",
];
const types = [
  "Normal",
  "Fire",
  "Water",
  "Electric",
  "Grass",
  "Ice",
  "Fight",
  "Poison",
  "Ground",
  "Flying",
  "Psychic",
  "Bug",
  "Rock",
  "Ghost",
  "Dragon",
  "Dark",
  "Steel",
  "Fairy",
];
export default function BattlefieldControls({
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
  const field = campaign.battle?.flow?.battlefield;
  const [weather, setWeather] = useState<Weather | "">(field?.natural ?? "");
  const [indoors, setIndoors] = useState(field?.indoors ?? false);
  const [sunlight, setSunlight] = useState(field?.sunlight ?? true);
  const [preset, setPreset] = useState("fog"),
    [scope, setScope] = useState("all");
  const [rounds, setRounds] = useState(0),
    [parameter, setParameter] = useState(1);
  const [target, setTarget] = useState(""),
    [mode, setMode] = useState("replace"),
    [type, setType] = useState("Water");
  const [reason, setReason] = useState(""),
    [notice, setNotice] = useState("");
  if (!field?.enabled || !campaign.battle?.active) return null;
  const gm = playerId === campaign.gm_id;
  const active = campaign.battle.combatants.filter((p) => p.deployed);
  const owned = (id: string) =>
    gm || active.some((p) => p.id === id && p.owner_id === playerId);
  const event = field.events[0];
  async function send(command: FieldCommand, preview = false) {
    try {
      const action: Action = {
        type: "battlefield",
        command,
        reason: reason.trim() || "Battlefield decision",
      };
      if (preview) {
        const p = await api.previewBattleAction(campaign, action);
        setNotice(`${p.dice} host dice. ${p.warnings.join(" ")}`);
      } else {
        if (await execute(action)) setNotice("");
      }
    } catch (e) {
      setNotice(errorOf(e).message);
    }
  }
  return (
    <details className="panel" open={!!event}>
      <summary>
        Battlefield · {field.weather?.weather ?? field.natural ?? "Clear"} ·{" "}
        {field.rules.length} active effects
      </summary>
      <p>
        Natural weather: {field.natural ?? "Clear"}. Temporary weather:{" "}
        {field.weather
          ? `${field.weather.weather} · ${field.weather.rounds ?? "source remains"} · ${field.weather.source}`
          : "none"}
        . Air Lock suppresses weather globally; Cloud Nine protects its holder.{" "}
        {field.night
          ? "Night time: Mind’s Eye grants Never Miss."
          : "Day time."}
      </p>
      {event && (
        <div className="notice" role="status">
          <strong>Pending: {event.kind}</strong> ·{" "}
          {active.find((p) => p.id === event.target)?.name ??
            event.contenders.map((c) => c.source).join(", ")}
          <p>
            {event.dice} host dice · difficulty {event.difficulty} · amount{" "}
            {event.amount} {event.damage_type}
          </p>
          {(["heal", "weather-choice"].includes(event.kind)
            ? owned(event.target)
            : gm) && (
            <div className="toolbar">
              {event.kind === "weather-choice" ? (
                <>
                  {(["Snow", "Hail"] as Weather[]).map((weather) => (
                    <button
                      key={weather}
                      disabled={disabled}
                      onClick={() =>
                        send({
                          kind: "SelectWeather",
                          event_id: event.id,
                          weather,
                        })
                      }
                    >
                      Choose {weather}
                    </button>
                  ))}
                </>
              ) : (
                <>
                  <button
                    disabled={disabled}
                    onClick={() =>
                      send(
                        { kind: "Resolve", event_id: event.id, accept: true },
                        true,
                      )
                    }
                  >
                    Preview check
                  </button>
                  <button
                    disabled={disabled}
                    onClick={() =>
                      send({
                        kind: "Resolve",
                        event_id: event.id,
                        accept: true,
                      })
                    }
                  >
                    {event.kind === "heal"
                      ? "Accept healing"
                      : event.kind === "terrain-ruling"
                        ? "Replace existing terrain"
                        : "Resolve host check"}
                  </button>
                  {["heal", "terrain-ruling"].includes(event.kind) && (
                    <button
                      disabled={disabled}
                      onClick={() =>
                        send({
                          kind: "Resolve",
                          event_id: event.id,
                          accept: false,
                        })
                      }
                    >
                      Decline / retain existing terrain
                    </button>
                  )}
                </>
              )}
            </div>
          )}
        </div>
      )}
      {gm && event && (
        <button
          disabled={disabled || !reason.trim()}
          onClick={() => send({ kind: "Adjudicate", event_id: event.id })}
        >
          Record GM resolution of pending event
        </button>
      )}
      <p>
        {active
          .map(
            (p) =>
              `${p.name}: ${(field.species_types[p.id] ?? []).join(" / ") || "Typeless / unresolved"}`,
          )
          .join(" � ")}
      </p>
      {field.rules.map((rule) => (
        <div className="list-row" key={rule.id}>
          <span>
            {rule.kind} · {rule.scope} · {rule.rounds ?? "encounter"} rounds ·{" "}
            {rule.source}
            {rule.parameter !== 0 ? ` · value ${rule.parameter}` : ""}
          </span>
          {gm && (
            <button
              disabled={disabled || !!event || !reason.trim()}
              onClick={() => send({ kind: "Remove", id: rule.id })}
            >
              Remove {rule.kind}
            </button>
          )}
          {rule.kind === "healing-pits" &&
            active
              .filter(
                (p) =>
                  owned(p.id) &&
                  (rule.scope === "all" || rule.scope === `side:${p.side}`),
              )
              .map((p) => (
                <button
                  key={p.id}
                  disabled={disabled || !!event}
                  onClick={() =>
                    send({
                      kind: "HealingPit",
                      combatant_id: p.id,
                      effect_id: rule.id,
                    })
                  }
                >
                  Use pit: {p.name}
                </button>
              ))}
        </div>
      ))}
      {field.corrections.length > 0 && (
        <p className="notice">
          Adjudication history: {field.corrections.join("; ")}
        </p>
      )}
      {gm && (
        <>
          <label>
            Recorded reason
            <input
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              maxLength={1000}
            />
          </label>
          <fieldset disabled={disabled || !!event || !reason.trim()}>
            <legend>Environment and arena</legend>
            <div className="form-grid">
              <label>
                Natural weather
                <select
                  value={weather}
                  onChange={(e) => setWeather(e.target.value as Weather | "")}
                >
                  <option value="">Clear</option>
                  {weatherNames.map((w) => (
                    <option key={w}>{w}</option>
                  ))}
                </select>
              </label>
              <label>
                <input
                  type="checkbox"
                  checked={indoors}
                  onChange={(e) => setIndoors(e.target.checked)}
                />{" "}
                Indoors / weather cannot reach
              </label>
              <label>
                <input
                  type="checkbox"
                  checked={sunlight}
                  onChange={(e) => setSunlight(e.target.checked)}
                />{" "}
                Sunlight reaches the arena
              </label>
            </div>
            {campaign.engine === "0.13.0-core" && (
              <label>
                <input
                  type="checkbox"
                  checked={field.night ?? false}
                  disabled={disabled}
                  onChange={(e) =>
                    send({ kind: "Night", night: e.target.checked })
                  }
                />{" "}
                Night time (Mind’s Eye)
              </label>
            )}
            <div className="toolbar">
              <button
                onClick={() =>
                  send({
                    kind: "Environment",
                    weather: weather || null,
                    indoors,
                    sunlight,
                  })
                }
              >
                Apply environment
              </button>
              {field.weather && (
                <button onClick={() => send({ kind: "Remove", id: "weather" })}>
                  Clear temporary weather
                </button>
              )}
            </div>
            <div className="form-grid">
              <label>
                Arena preset
                <select
                  value={preset}
                  onChange={(e) => setPreset(e.target.value)}
                >
                  {presets.map((p) => (
                    <option key={p}>{p}</option>
                  ))}
                </select>
              </label>
              <label>
                Affected area
                <select
                  value={scope}
                  onChange={(e) => setScope(e.target.value)}
                >
                  <option value="all">Whole battlefield</option>
                  {[...new Set(active.map((p) => p.side))].map((s) => (
                    <option key={s} value={`side:${s}`}>
                      {s}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                Rounds (0 = encounter)
                <input
                  type="number"
                  min={0}
                  max={1000}
                  value={rounds}
                  onChange={(e) => setRounds(Number(e.target.value))}
                />
              </label>
              <label>
                Check difficulty / healing amount
                <input
                  type="number"
                  min={1}
                  max={100}
                  value={parameter}
                  onChange={(e) => setParameter(Number(e.target.value))}
                />
              </label>
            </div>
            <button
              onClick={() =>
                send({
                  kind: "Preset",
                  name: preset,
                  scope,
                  rounds: rounds || null,
                  parameter,
                })
              }
            >
              Add arena condition
            </button>
            <p>
              High poles requires an agreed difficulty. Healing pits consume an
              action and are depleted after use. Torn World requires recorded
              random-target adjudication. Cemetery grants narrative Run Away
              access; held-item effects remain unsupported.
            </p>
            <div className="form-grid">
              <label>
                Combatant
                <select
                  value={target}
                  onChange={(e) => setTarget(e.target.value)}
                >
                  <option value="">Choose Pokémon</option>
                  {active.map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                Typing operation
                <select value={mode} onChange={(e) => setMode(e.target.value)}>
                  {["replace", "add", "remove", "restore"].map((m) => (
                    <option key={m}>{m}</option>
                  ))}
                </select>
              </label>
              <label>
                Type
                <select value={type} onChange={(e) => setType(e.target.value)}>
                  {types.map((t) => (
                    <option key={t}>{t}</option>
                  ))}
                </select>
              </label>
            </div>
            <div className="toolbar">
              <button
                disabled={!target}
                onClick={() =>
                  send({
                    kind: "Typing",
                    combatant_id: target,
                    mode,
                    types: mode === "restore" ? [] : [type],
                  })
                }
              >
                Apply temporary typing
              </button>
              <button
                disabled={!target}
                onClick={() =>
                  send({
                    kind: "Shelter",
                    combatant_id: target,
                    sheltered: !field.sheltered.includes(target),
                  })
                }
              >
                {field.sheltered.includes(target) ? "Remove" : "Grant"}{" "}
                full-body weather cover
              </button>
            </div>
          </fieldset>
        </>
      )}
      {notice && <p role="status">{notice}</p>}
    </details>
  );
}
