import { describe, expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import MoveDecisionControls from "./MoveDecisionControls";
import type { Campaign } from "./bindings/Campaign";

function view(
  playerId: string,
  state: "prepare" | "target" | "gm" | "resisted" = "prepare",
  block = true,
  window = false,
) {
  const campaign = {
    gm_id: "gm",
    battle: {
      active: true,
      combatants: [
        { id: "a", name: "Offerer", owner_id: "source" },
        { id: "t", name: "Recipient", owner_id: "target" },
      ],
      flow: {
        window,
        queue: [{ sequence: 7, move_name: "Hold Hands", actor_id: "a" }],
        completion: {
          blocks: block ? ["MOVE-A45"] : [],
          hold_hands:
            state === "prepare"
              ? {}
              : {
                  7: {
                    target_id: "t",
                    accuracy: 3,
                    will_dice: 7,
                    resisted: state === "target" ? null : state === "resisted",
                    resistance: state === "resisted" ? 3 : null,
                  },
                },
        },
      },
    },
  } as unknown as Campaign;
  return renderToStaticMarkup(
    <MoveDecisionControls
      campaign={campaign}
      playerId={playerId}
      execute={async () => true}
      disabled={false}
    />,
  );
}
describe("saved Hold Hands choices", () => {
  it("requires block adoption and waits for reactions", () => {
    expect(view("gm", "prepare", false)).toBe("");
    expect(view("gm", "prepare", true, true)).toContain("Resolve reactions");
    expect(view("gm", "prepare", true, true)).toContain(
      '<button disabled="">Prepare target decision</button>',
    );
  });
  it("limits preparation to source owner or GM", () => {
    expect(view("source")).toContain(
      "<button>Prepare target decision</button>",
    );
    expect(view("target")).toContain(
      '<button disabled="">Prepare target decision</button>',
    );
  });
  it("offers the optional resistance to the actual target owner", () => {
    expect(view("target", "target")).toContain(
      "<button>Roll Will resistance</button>",
    );
    expect(view("target", "target")).toContain(
      "<button>Do not resist</button>",
    );
    expect(view("source", "target")).toContain(
      '<button disabled="">Roll Will resistance</button>',
    );
    expect(view("target", "target")).toContain(
      "7 Will dice; 3 successes required",
    );
    expect(view("target", "target")).toContain("spends no Will");
  });
  it("shows GM ruling controls only after resistance and only to the GM", () => {
    expect(view("gm", "target")).not.toContain("End battle by agreement");
    expect(view("target", "gm")).not.toContain("End battle by agreement");
    expect(view("gm", "gm")).toContain("Hold Hands ruling reason");
    expect(view("gm", "resisted")).toContain(
      "Target resisted; this move cannot end the battle.",
    );
    expect(view("gm", "resisted")).toContain(
      '<button disabled="">End battle by agreement</button>',
    );
  });
});
