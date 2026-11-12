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
});
