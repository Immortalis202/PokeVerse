import { describe, expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import MetronomeControls from "./MetronomeControls";
import type { Campaign } from "./bindings/Campaign";
function view(playerId: string, stage = "empty", enabled = true) {
  const campaign = {
    gm_id: "gm",
    sheets: { sa: { moves: ["Metronome"] }, st: { moves: [] } },
    battle: {
      active: true,
      combatants: [
        {
          id: "a",
          name: "Caster",
          sheet_id: "sa",
          owner_id: "owner",
          deployed: true,
          hp: 10,
          side: "Players",
        },
        {
          id: "t",
          name: "Target",
          sheet_id: "st",
          owner_id: "target",
          deployed: true,
          hp: 10,
          side: "Opponents",
        },
      ],
      flow: {
        order: ["a"],
        cursor: 0,
        queue: [],
        completion: {
          blocks: enabled ? ["MOVE-A46"] : [],
          metronome_table:
            stage === "empty"
              ? null
              : { physical_faces: [1, 2, 3], reason: "Saved table" },
          metronome_draw: ["drawn", "selected", "successive"].includes(stage)
            ? {
                actor_id: "a",
                token: 9,
                faces: [1, 2],
                category: "Physical",
                power: 2,
                candidates: ["Flame Charge", "Tackle", "Double Kick"],
                selected:
                  stage === "selected"
                    ? "Flame Charge"
                    : stage === "successive"
                      ? "Double Kick"
                      : null,
                attempt_limit: stage === "successive" ? 2 : 1,
              }
            : null,
          copied_cast:
            stage === "casting"
              ? { actor_id: "a", template: "Flame Charge", origin: "Metronome" }
              : null,
        },
      },
    },
  } as unknown as Campaign;
  return renderToStaticMarkup(
    <MetronomeControls
      campaign={campaign}
      playerId={playerId}
      execute={async () => true}
      disabled={false}
    />,
  );
}
describe("Metronome committed choices", () => {
  it("requires block adoption and an explicit GM table with no selected default", () => {
    expect(view("gm", "empty", false)).toBe("");
    expect(view("gm")).toContain("the card supplies no table");
    expect(view("gm")).toContain("Record category table");
    expect(view("gm")).not.toContain('checked=""');
    expect(view("owner")).not.toContain("Record category table");
  });
  it("shows the persisted dice and limits template selection to the GM", () => {
    expect(view("owner", "drawn")).toContain(
      "Saved dice: 1, 2. Result: Physical, Power 2.",
    );
    expect(view("owner", "drawn")).toContain("Flame Charge");
    expect(view("owner", "drawn")).not.toContain("Save template choice");
    expect(view("gm", "drawn")).toContain("Save template choice");
    expect(view("gm", "drawn")).not.toContain(
      "Roll category and Power</button>",
    );
  });
  it("exposes the selected cast and preview while reserving discard to GM", () => {
    expect(view("owner", "selected")).toContain("Selected: Flame Charge");
    expect(view("owner", "selected")).toContain("Cast selected move");
    expect(view("owner", "selected")).toContain("Preview copied Accuracy");
    expect(view("owner", "selected")).not.toContain(
      "Record failed attempt and spend action",
    );
    expect(view("gm", "selected")).toContain(
      "Record failed attempt and spend action",
    );
  });
  it("keeps pending copied damage in normal battle controls", () => {
    expect(view("owner", "casting")).toContain(
      "Resolve copied Flame Charge through the normal battle controls.",
    );
    expect(view("owner", "casting")).not.toContain(
      "Roll category and Power</button>",
    );
  });
  it("offers normal successive-action counts for copied multi-hit moves", () => {
    expect(view("owner", "successive")).toContain("Copied successive actions");
    expect(view("owner", "successive")).toContain(
      '<option value="2">2</option>',
    );
  });
});
