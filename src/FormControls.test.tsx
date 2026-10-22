import { expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import FormControls from "./FormControls";
import type { Campaign } from "./bindings/Campaign";
import { newSheet } from "./sheets";

function campaign(ability = "Multitype", enabled = true): Campaign {
  const sheet = newSheet("owner", "pokemon");
  sheet.id = "pokemon";
  sheet.name = "My Pokémon";
  sheet.species_id = "0493:arceus";
  sheet.ability = ability;
  sheet.moves = ["Transform"];
  const trainer = newSheet("owner", "trainer");
  trainer.id = "trainer";
  trainer.name = "Trainer";
  return {
    form_memory: {},
    id: "forms",
    name: "Forms",
    gm_id: "gm",
    host_id: "host",
    revision: 1,
    catalog_hash: "catalog",
    engine: "0.19.0-core",
    sheets: { pokemon: sheet, trainer },
    battle: {
      active: true,
      combatants: [
        {
          id: "unit",
          sheet_id: "pokemon",
          owner_id: "owner",
          name: "My Pokémon",
          side: "Players",
          deployed: true,
          hp: 10,
          max_hp: 10,
          selected_ability: ability,
          runtime: { ability: { flags: [], counters: {} } },
        },
      ],
      flow: {
        forms: {
          enabled,
          originals: {},
          current: {},
          plans: {},
          choices: {},
          seen: [],
          mega_trainers: [],
          key_stones: [],
          equipment: {},
          history: [],
        },
        items: { units: {} },
        ability_events: [],
      },
    } as unknown as NonNullable<Campaign["battle"]>,
    finished_battles: [],
    events: [],
    battle_undo: [],
    proposals: [],
    training_encounters: [],
    rules_profile: { version: 1, defenses: "split", hp: "vitality" },
    ability_inheritance: { minutes: 0, copies: {}, selected: {}, history: [] },
  };
}
const render = (c: Campaign, playerId: string) =>
  renderToStaticMarkup(
    <FormControls
      campaign={c}
      playerId={playerId}
      execute={async () => true}
      disabled={false}
    />,
  );
it("exposes owned Multitype and Transform choices while reserving equipment rulings for the GM", () => {
  const owner = render(campaign(), "owner");
  expect(owner).toContain("Change type");
  expect(owner).toContain("Choose Transform model");
  expect(owner).not.toContain("Record Key Stone");
  expect(owner).not.toContain("GM equipment, arrival and joint-form rulings");
  const gm = render(campaign(), "gm");
  expect(gm).toContain("Record Key Stone: Trainer");
  expect(gm).toContain("These decisions create no inventory items");
  expect(render(campaign(), "observer")).toBe("");
});
it("keeps historical encounters gated and shows explicit partial-data equipment boundaries", () => {
  expect(render(campaign("Multitype", false), "owner")).toBe("");
  expect(render(campaign("RKS System"), "gm")).toContain(
    "Record Disc equipment",
  );
  expect(render(campaign("Embody Aspect"), "gm")).toContain(
    "Record mask equipment",
  );
  expect(render(campaign("As One"), "gm")).toContain(
    "Join reviewed form with As One",
  );
});
it("displays current form identity and both As One ability channels", () => {
  const c = campaign("Adaptability");
  c.battle!.flow!.forms.current.unit = {
    sheet: c.sheets.pokemon!,
    types: ["Psychic", "Ice"],
    ability: "Adaptability",
    secondary: "Technician",
    partner: "ally",
    label: "Reviewed joint form",
    source: "As One",
    mega: false,
  };
  const html = render(c, "owner");
  expect(html).toContain("Reviewed joint form");
  expect(html).toContain("Adaptability + Technician");
  expect(html).toContain("Psychic / Ice");
});
