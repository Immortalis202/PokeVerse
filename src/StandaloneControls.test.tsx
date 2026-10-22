import { expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import StandaloneControls from "./StandaloneControls";
import AbilityControls from "./AbilityControls";
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
        standalone: {
          enabled: false,
          profiles: {},
          rivalry_basis: null,
          torrent_bonus: null,
          perish: {},
          ended_round: 0,
          history: [],
        },
        information: { enabled: false, active: [], offers: {}, reveals: [] },
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

function current() {
  const c = campaign("Moody");
  c.engine = "0.21.0-core";
  c.battle!.round = 1;
  c.battle!.flow!.standalone.enabled = true;
  return c;
}
const render = (c: Campaign, id: string) =>
  renderToStaticMarkup(
    <StandaloneControls
      campaign={c}
      playerId={id}
      execute={async () => true}
      disabled={false}
    />,
  );
it("offers explicit metal weights, Rivalry wording and Torrent alternatives to the GM", () => {
  const html = render(current(), "gm");
  expect(html).toContain("GM Heavy/Light Metal weight");
  expect(html).toContain("Use supplied-card gender wording");
  expect(html).toContain("Use printed preference wording");
  expect(html).toContain("Record Torrent: 1 die");
  expect(html).toContain("Record Torrent: 2 dice");
  expect(html).toContain("Unknown");
  const owner = render(current(), "owner");
  expect(owner).not.toContain("Record profile for");
  expect(owner).not.toContain("Record Torrent:");
});
it("shows the actual Perish deadline and keeps historical encounters gated", () => {
  const c = current();
  c.battle!.flow!.standalone.perish.unit = {
    source_id: "source",
    due_round: 4,
  };
  const html = render(c, "owner");
  expect(html).toContain("end of round 4");
  expect(html).toContain("3");
  c.battle!.flow!.standalone.enabled = false;
  expect(render(c, "gm")).toBe("");
});
it("explains Moody rejections and action-free rerolls at the pending owner decision", () => {
  const c = current();
  const f = c.battle!.flow!;
  f.ability_rules = true;
  f.items.berry_hits = {};
  f.ability_events = [
    {
      id: "moody",
      combatant_id: "unit",
      target_id: "unit",
      ability: "Moody",
      kind: "standalone-moody",
      move_name: "round end",
      choices: ["roll"],
    },
  ];
  const html = renderToStaticMarkup(
    <AbilityControls
      campaign={c}
      playerId="owner"
      execute={async () => true}
      disabled={false}
    />,
  );
  expect(html).toContain("twenty distinct decrease/increase pairs");
  expect(html).toContain("leave this decision pending");
  expect(html).toContain("Roll ability dice");
});
