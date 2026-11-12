import { describe, expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import MoveOptionControls from "./MoveOptionControls";
import type { Campaign } from "./bindings/Campaign";

function view(
  playerId: string,
  options: string[] = ["Dexterity", "Vitality"],
  conversion: Record<string, string[]> = {},
) {
  const campaign = {
    gm_id: "gm",
    battle: {
      active: true,
      combatants: [{ id: "a", name: "Caster", owner_id: "owner" }],
      flow: {
        window: false,
        special: { effects: [] },
        queue: [{ sequence: 4, move_name: "Mystical Power", actor_id: "a" }],
        completion: {
          open_options: { 4: options },
          move_options: {},
          conversion_choices: conversion,
        },
      },
    },
  } as unknown as Campaign;
  return renderToStaticMarkup(
    <MoveOptionControls
      campaign={campaign}
      playerId={playerId}
      execute={async () => true}
      disabled={false}
    />,
  );
}
describe("saved move choices", () => {
  it("shows only when the current cast has open options", () => {
    expect(view("owner", [])).toBe("");
    expect(view("owner")).toContain("Mystical Power: choose");
    expect(view("owner")).toContain(
      '<option value="Vitality">Vitality</option>',
    );
  });
  it("limits the choice to the caster's owner or the GM", () => {
    expect(view("owner")).toContain("<button>Save choice</button>");
    expect(view("gm")).toContain("<button>Save choice</button>");
    expect(view("other")).toContain('<button disabled="">Save choice</button>');
  });
  it("labels combatant options by name", () => {
    expect(view("owner", ["a", "Fire"])).toContain(">Caster</option>");
    expect(view("owner", ["a", "Fire"])).toContain(
      '<option value="Fire">Fire</option>',
    );
  });
  it("offers Conversion 2 types to the hit combatant's owner", () => {
    const html = view("owner", [], { a: ["Rock", "Steel"] });
    expect(html).toContain("Conversion 2: Caster");
    expect(html).toContain("<button>Change type</button>");
    expect(view("other", [], { a: ["Rock"] })).toContain(
      '<button disabled="">Change type</button>',
    );
  });
  it("offers Grapple release and reaction timing after MOVE-A52", () => {
    const campaign = {
      gm_id: "gm",
      sheets: { s: { moves: ["Sucker Punch"] } },
      battle: {
        active: true,
        combatants: [
          {
            id: "a",
            name: "Caster",
            owner_id: "owner",
            sheet_id: "s",
            deployed: true,
            hp: 5,
          },
        ],
        flow: {
          window: false,
          queue: [],
          special: { effects: [{ source: "a", target: "b", slot: "grapple" }] },
          completion: {
            blocks: ["MOVE-A52"],
            reaction_timing: { "a:Sucker Punch": "late" },
          },
        },
      },
    } as unknown as Campaign;
    const html = renderToStaticMarkup(
      <MoveOptionControls
        campaign={campaign}
        playerId="owner"
        execute={async () => true}
        disabled={false}
      />,
    );
    expect(html).toContain("<button>Release Grapple</button>");
    expect(html).toContain("Sucker Punch reacts late");
    expect(html).toContain("<button>React early</button>");
  });
  it("offers Beat Up joins and Help Another after MOVE-A53", () => {
    const campaign = {
      gm_id: "gm",
      sheets: {
        s: { moves: ["Beat Up", "Help Another"] },
        t: { moves: ["Tackle"] },
      },
      battle: {
        active: true,
        combatants: [
          {
            id: "a",
            name: "Caster",
            owner_id: "owner",
            sheet_id: "s",
            side: "x",
            deployed: true,
            hp: 5,
          },
          {
            id: "b",
            name: "Friend",
            owner_id: "owner",
            sheet_id: "t",
            side: "x",
            deployed: true,
            hp: 5,
          },
        ],
        flow: {
          window: false,
          queue: [
            {
              sequence: 9,
              move_name: "Beat Up",
              actor_id: "a",
              damage_done: false,
            },
          ],
          special: { effects: [] },
          completion: { blocks: ["MOVE-A53"] },
        },
      },
    } as unknown as Campaign;
    const html = renderToStaticMarkup(
      <MoveOptionControls
        campaign={campaign}
        playerId="owner"
        execute={async () => true}
        disabled={false}
      />,
    );
    expect(html).toContain("Friend: join Beat Up");
    expect(html).toContain("<button>Roll help</button>");
  });
  it("offers the owed Powder roll", () => {
    expect(view("owner", [], {})).toBe("");
    const campaign = {
      gm_id: "gm",
      sheets: {},
      battle: {
        active: true,
        combatants: [{ id: "a", name: "Caster", owner_id: "owner" }],
        flow: {
          window: false,
          queue: [],
          special: { effects: [] },
          completion: { powder_due: { a: 2 } },
        },
      },
    } as unknown as Campaign;
    const html = renderToStaticMarkup(
      <MoveOptionControls
        campaign={campaign}
        playerId="owner"
        execute={async () => true}
        disabled={false}
      />,
    );
    expect(html).toContain("owes 2 Powder damage dice");
    expect(html).toContain("<button>Roll Powder damage</button>");
  });
  it("offers a copied cast of the source's last move after MOVE-A59", () => {
    const campaign = {
      gm_id: "gm",
      sheets: { s: { moves: ["Copycat"] }, t: { moves: ["Tackle"] } },
      battle: {
        active: true,
        combatants: [
          {
            id: "a",
            name: "Copier",
            owner_id: "owner",
            sheet_id: "s",
            side: "x",
            deployed: true,
            hp: 5,
          },
          {
            id: "b",
            name: "Foe",
            owner_id: "other",
            sheet_id: "t",
            side: "y",
            deployed: true,
            hp: 5,
          },
        ],
        flow: {
          window: false,
          queue: [],
          special: { effects: [] },
          completion: {
            blocks: ["MOVE-A59"],
            uses: [{ actor: "b", name: "Tackle", round: 1 }],
          },
        },
      },
    } as unknown as Campaign;
    const html = renderToStaticMarkup(
      <MoveOptionControls
        campaign={campaign}
        playerId="owner"
        execute={async () => true}
        disabled={false}
      />,
    );
    expect(html).toContain(
      '<option value="Tackle" selected="">Tackle</option>',
    );
    expect(html).toContain("<button>Cast copy</button>");
  });
});
