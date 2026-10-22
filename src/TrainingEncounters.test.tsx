import { describe, it, expect } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import TrainingEncounters from "./TrainingEncounters";
import { newSheet } from "./sheets";
import type { Campaign } from "./bindings/Campaign";
import type { Catalog } from "./catalog";

const trainer = {
  ...newSheet("player", "trainer"),
  id: "trainer",
  name: "Maya",
};
const catalog: Catalog = {
  edition: "3.0",
  hash: "catalog",
  species: [
    {
      AppId: "0025:pikachu",
      DexID: "0025",
      Name: "Pikachu",
      Type1: "Electric",
      Type2: "",
      Ability1: "Static",
      Ability2: "",
      Image: "pikachu.png",
    },
  ],
  moves: [],
  abilities: [],
  natures: [],
  items: [],
  warnings: [],
};
const campaign: Campaign = {
  form_memory: {},
  ability_inheritance: { minutes: 0, copies: {}, selected: {}, history: [] },
  id: "campaign",
  name: "Training",
  gm_id: "gm",
  host_id: "host",
  revision: 4,
  catalog_hash: "catalog",
  engine: "0.2.0-core",
  sheets: { trainer },
  battle: null,
  finished_battles: [],
  events: [],
  battle_undo: [],
  rules_profile: { version: 1, defenses: "split", hp: "vitality" },
  proposals: [],
  training_encounters: [
    {
      id: "encounter",
      campaign_id: "campaign",
      trainer_id: "trainer",
      habitat_id: "forest",
      catalog_hash: "catalog",
      generator_version: 1,
      seed: "seed",
      individuals: [
        {
          id: "wild",
          species_id: "0025:pikachu",
          rank: "Rookie",
          ability: "Static",
          seed: "seed:0",
          conditions: "near streams",
        },
      ],
      status: "published",
      revision: 1,
      captured: {},
      rewards: {},
      reason: "",
    },
  ],
};
function render(playerId: string, c: Campaign | null = campaign) {
  return renderToStaticMarkup(
    <TrainingEncounters
      campaign={c}
      playerId={playerId}
      catalog={catalog}
      execute={async () => true}
      connected={false}
      busy={false}
      pull={async () => {}}
    />,
  );
}
describe("training encounter visibility", () => {
  it("players see revealed species and conditions without preparation or capture controls", () => {
    const html = render("player");
    expect(html).toContain("Pikachu");
    expect(html).toContain("/sprites/pikachu.png");
    expect(html).toContain("Rookie");
    expect(html).toContain("near streams");
    expect(html).not.toContain("Generate horde");
    expect(html).not.toContain("Capture for");
    expect(html).not.toContain("Training Point rewards</h3>");
    expect(html).not.toContain("Static");
  });
  it("the appointed GM has offline preparation and manual resolution, while the host has player visibility", () => {
    expect(render("gm")).toContain("Generate horde");
    expect(render("gm")).toContain("Capture for");
    expect(render("gm")).toContain("Review outcome");
    expect(render("host")).not.toContain("Generate horde");
    expect(render("gm", null)).toContain("Select a campaign");
  });
});
