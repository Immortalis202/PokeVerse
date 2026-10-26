import type { EncounterField } from "./bindings/EncounterField";
import type { InitialFieldEffect } from "./bindings/InitialFieldEffect";
import type { Weather } from "./bindings/Weather";

export const newEncounterField = (): EncounterField => ({
  night: false,
  weather: null,
  indoors: false,
  sunlight: true,
  terrain: null,
  effects: [],
});
const weatherOptions: [Weather, string][] = [
  ["Sunny", "Sun"],
  ["Rain", "Rain"],
  ["Snow", "Snow"],
  ["Hail", "Hail"],
  ["Sandstorm", "Sandstorm"],
  ["Desolate", "Harsh sun / Desolate"],
  ["Typhoon", "Heavy rain / Typhoon"],
  ["StrongWinds", "Strong winds"],
];
const rooms = [
  "Trick Room",
  "Wonder Room",
  "Magic Room",
  "Gravity",
  "Reflect",
  "Light Screen",
  "Aurora Veil",
];
const arenas = [
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
export default function BattlefieldSetup({
  value,
  onChange,
  disabled,
}: {
  value: EncounterField;
  onChange: (value: EncounterField) => void;
  disabled: boolean;
}) {
  const patch = (v: Partial<EncounterField>) => onChange({ ...value, ...v });
  const update = (name: string, v: Partial<InitialFieldEffect>) =>
    patch({
      effects: value.effects.map((e) => (e.kind === name ? { ...e, ...v } : e)),
    });
  const duration = (
    effect: InitialFieldEffect,
    change: (rounds: number | null) => void,
  ) => (
    <label>
      Remaining rounds for {effect.kind}
      <input
        type="number"
        min={0}
        max={1000}
        value={effect.rounds ?? 0}
        onChange={(e) => change(Number(e.target.value) || null)}
      />
    </label>
  );
  return (
    <fieldset className="panel" disabled={disabled}>
      <legend>Starting battlefield · multiplayer battle</legend>
      <p>
        Choose the conditions before starting this battle. Wild Pokémon
        encounters use their own preparation. Zero remaining rounds means the
        condition lasts until removed; no Pokémon action or Will is spent.
      </p>
      <div className="form-grid">
        <label>
          Time of day
          <select
            value={value.night ? "night" : "day"}
            onChange={(e) => patch({ night: e.target.value === "night" })}
          >
            <option value="day">Day</option>
            <option value="night">Night</option>
          </select>
        </label>
        <label>
          Starting natural weather
          <select
            value={value.weather ?? ""}
            onChange={(e) =>
              patch({ weather: (e.target.value || null) as Weather | null })
            }
          >
            <option value="">Clear</option>
            {weatherOptions.map(([w, label]) => (
              <option key={w} value={w}>
                {label}
              </option>
            ))}
          </select>
        </label>
        <label className="checkbox">
          <input
            type="checkbox"
            checked={value.indoors}
            onChange={(e) => patch({ indoors: e.target.checked })}
          />
          Indoors / weather cannot reach
        </label>
        <label className="checkbox">
          <input
            type="checkbox"
            checked={value.sunlight}
            onChange={(e) => patch({ sunlight: e.target.checked })}
          />
          Sunlight can reach the arena
        </label>
        <label>
          Starting terrain
          <select
            value={value.terrain?.kind ?? ""}
            onChange={(e) =>
              patch({
                terrain: e.target.value
                  ? { kind: e.target.value, rounds: null, parameter: 1 }
                  : null,
              })
            }
          >
            <option value="">None</option>
            {[
              "Grassy Terrain",
              "Electric Terrain",
              "Misty Terrain",
              "Psychic Terrain",
            ].map((t) => (
              <option key={t}>{t}</option>
            ))}
          </select>
        </label>
        {value.terrain &&
          duration(value.terrain, (rounds) =>
            patch({ terrain: { ...value.terrain!, rounds } }),
          )}
      </div>
      <details>
        <summary>Rooms, Gravity and arena conditions</summary>
        {[...rooms, ...arenas].map((name) => {
          const effect = value.effects.find((e) => e.kind === name);
          return (
            <div className="form-grid" key={name}>
              <label className="checkbox">
                <input
                  type="checkbox"
                  checked={!!effect}
                  onChange={(e) =>
                    patch({
                      effects: e.target.checked
                        ? [
                            ...value.effects,
                            {
                              kind: name,
                              rounds: rooms.includes(name) ? 4 : null,
                              parameter: 1,
                            },
                          ]
                        : value.effects.filter((v) => v.kind !== name),
                    })
                  }
                />
                {name}
              </label>
              {effect && duration(effect, (rounds) => update(name, { rounds }))}
              {effect && ["high-poles", "healing-pits"].includes(name) && (
                <label>
                  {name === "high-poles"
                    ? "Check difficulty"
                    : "Healing amount"}
                  <input
                    type="number"
                    min={1}
                    max={100}
                    value={effect.parameter}
                    onChange={(e) =>
                      update(name, { parameter: Number(e.target.value) })
                    }
                  />
                </label>
              )}
            </div>
          );
        })}
        <p>
          One terrain at a time; room and arena effects can coexist. Entry
          abilities and later moves still follow their normal replacement and
          conflict rules. Torn World keeps its explicit GM targeting ruling.
        </p>
      </details>
    </fieldset>
  );
}
