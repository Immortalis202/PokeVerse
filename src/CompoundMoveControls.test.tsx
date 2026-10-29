import { describe, expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import CompoundMoveControls from "./CompoundMoveControls";
import type { Campaign } from "./bindings/Campaign";

function view(who: string, enabled = true, pending = false) {
  const campaign = {
    gm_id: "gm",
    battle: {
      active: true,
      combatants: [
        {
          id: "a",
          name: "Singer",
          owner_id: "owner",
          deployed: true,
          hp: 10,
          side: "Players",
        },
      ],
      flow: {
        compound: {
          enabled,
          crashes: ["a"],
          environments: ["Forest/Jungle"],
          alignments: {},
          minimized: [],
          relic: ["a"],
        },
        queue: pending ? [{}] : [],
        battlefield: { rules: [] },
      },
    },
  } as unknown as Campaign;
  return renderToStaticMarkup(
    <CompoundMoveControls
      campaign={campaign}
      playerId={who}
      execute={async () => true}
      disabled={false}
    />,
  );
}
describe("compound move choices", () => {
  it("exposes scene and Minimize confirmation only to the GM", () => {
    expect(view("gm")).toContain("Confirm Secret Power surroundings");
    expect(view("gm")).toContain("GM confirms Minimize");
    expect(view("owner")).not.toContain("Confirm Secret Power surroundings");
    expect(view("owner")).not.toContain("GM confirms Minimize");
    expect(view("owner")).toContain("Forest/Jungle");
  });
  it("restricts crash and optional form decisions to owners or GM and waits for song resolution", () => {
    expect(view("owner")).toContain(
      "<button>Roll 4 crash damage dice</button>",
    );
    expect(view("other")).toContain(
      '<button disabled="">Roll 4 crash damage dice</button>',
    );
    expect(view("owner", true, true)).toContain(
      '<button disabled="">Change Meloetta form</button>',
    );
    expect(view("owner")).toContain("<button>Keep current form</button>");
  });
  it("keeps compound controls absent from historical encounters", () => {
    expect(view("owner", false)).toBe("");
  });
});
