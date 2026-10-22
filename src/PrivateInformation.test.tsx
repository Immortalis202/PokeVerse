import { expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import PrivateInformation from "./PrivateInformation";
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

const render = (c: Campaign, playerId: string) =>
  renderToStaticMarkup(
    <PrivateInformation
      campaign={c}
      playerId={playerId}
      execute={async () => true}
      disabled={false}
    />,
  );
function pending() {
  const c = campaign();
  const f = c.battle!.flow!;
  f.information = {
    enabled: true,
    active: ["unit:Forewarn"],
    offers: {
      secret: {
        recipient: "trainer-owner",
        candidates: [
          {
            foe_id: "foe",
            move_name: "Solar Beam",
            power: 5,
            category: "Special",
            lethal: false,
            super_effective: true,
            uncertain: false,
            detail: "Source card: variable sunlight timing",
          },
        ],
      },
    },
    reveals: [],
  };
  f.ability_events = [
    {
      id: "secret",
      combatant_id: "unit",
      target_id: "unit",
      ability: "Forewarn",
      kind: "info-entry",
      move_name: "private trainer warning window",
      choices: ["move:0", "none"],
    },
  ];
  return c;
}
it("shows comparison and Insight rulings only to the GM", () => {
  const c = pending();
  const gm = render(c, "gm");
  expect(gm).toContain("Solar Beam");
  expect(gm).toContain("Insight");
  expect(gm).toContain("highest printed Power");
  for (const id of ["owner", "trainer-owner", "observer"]) {
    const html = render(c, id);
    expect(html).not.toContain("Solar Beam");
    expect(html).not.toContain("Source card");
    expect(html).not.toContain("Close with private GM ruling");
  }
});
it("shows a secret message only to its trainer recipient and GM", () => {
  const c = pending();
  c.battle!.flow!.information.reveals = [
    {
      recipient: "trainer-owner",
      source_id: "unit",
      ability: "Forewarn",
      round: 1,
      alert: true,
      decision: "move:0",
      move_name: "Solar Beam",
      reason: "Private Insight result",
    },
  ];
  expect(render(c, "trainer-owner")).toContain("Strongest move: Solar Beam");
  expect(render(c, "owner")).not.toContain("Private Insight result");
  expect(render(c, "observer")).not.toContain("Solar Beam");
});
it("does not turn Anticipation into a move reveal and preserves old engine gates", () => {
  const c = pending();
  const f = c.battle!.flow!;
  f.ability_events = [];
  f.information.offers = {};
  f.information.reveals = [
    {
      recipient: "owner",
      source_id: "unit",
      ability: "Anticipation",
      round: 1,
      alert: true,
      decision: "alert",
      move_name: null,
      reason: "Threat reviewed",
    },
  ];
  const html = render(c, "owner");
  expect(html).toContain("A foe knows a lethal or super-effective move");
  expect(html).not.toContain("Solar Beam");
  f.information.enabled = false;
  expect(render(c, "owner")).toBe("");
});
