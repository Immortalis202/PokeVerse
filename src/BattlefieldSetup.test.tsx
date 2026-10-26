import { describe, it, expect } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import BattlefieldSetup, { newEncounterField } from "./BattlefieldSetup";

describe("starting multiplayer battlefield", () => {
  it("offers explicit day/night, every supported weather, terrains and room conditions", () => {
    const html = renderToStaticMarkup(
      <BattlefieldSetup
        value={newEncounterField()}
        onChange={() => {}}
        disabled={false}
      />,
    );
    for (const text of [
      "Time of day",
      "Night",
      "Clear",
      "Sun",
      "Rain",
      "Snow",
      "Hail",
      "Sandstorm",
      "Harsh sun",
      "Typhoon",
      "Strong winds",
      "Grassy Terrain",
      "Psychic Terrain",
      "Electric Terrain",
      "Misty Terrain",
      "Trick Room",
      "Gravity",
      "Magic Room",
      "Wonder Room",
      "fog",
    ])
      expect(html).toContain(text);
    expect(html).toContain("Wild Pokémon encounters use their own preparation");
    expect(newEncounterField()).toEqual({
      night: false,
      weather: null,
      indoors: false,
      sunlight: true,
      terrain: null,
      effects: [],
    });
  });
  it("keeps terrain and room remaining durations separate and supports disabled submission", () => {
    const value = newEncounterField();
    value.night = true;
    value.terrain = { kind: "Grassy Terrain", rounds: null, parameter: 1 };
    value.effects = [
      { kind: "Trick Room", rounds: 2, parameter: 1 },
      { kind: "Gravity", rounds: 4, parameter: 1 },
    ];
    const html = renderToStaticMarkup(
      <BattlefieldSetup value={value} onChange={() => {}} disabled />,
    );
    expect(html).toContain('disabled=""');
    for (const name of ["Grassy Terrain", "Trick Room", "Gravity"])
      expect(html).toContain(`Remaining rounds for ${name}`);
    expect(html).toContain('value="night" selected=""');
  });
});
