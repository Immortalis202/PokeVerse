import { expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import RecurringMoveControls from "./RecurringMoveControls";
import type { Campaign } from "./bindings/Campaign";
function view(who: string, enabled = true, pending = false) {
  const campaign = {
    gm_id: "gm",
    sheets: { s: { moves: ["Sleep Talk"] } },
    battle: {
      active: true,
      round: 1,
      combatants: [
        {
          id: "a",
          sheet_id: "s",
          name: "Sleeper",
          owner_id: "owner",
          deployed: true,
          hp: 10,
          actions: 0,
          runtime: { statuses: [{ kind: "Sleep" }] },
        },
      ],
      flow: {
        queue: [],
        battlefield: { end_round: 1 },
        recurring: {
          enabled,
          yawn: { a: { due_round: 2 } },
          perish: {},
          nightmares: {},
          malignant: {},
          executing: null,
          pending_talk: pending
            ? { actor: "a", move_name: "Tackle", foe: "foe" }
            : null,
        },
      },
    },
  } as unknown as Campaign;
  return renderToStaticMarkup(
    <RecurringMoveControls
      campaign={campaign}
      playerId={who}
      execute={async () => true}
      disabled={false}
    />,
  );
}
it("shows clocks and offers sleep draw only to owner or GM", () => {
  expect(view("owner")).toContain("Yawn applies Sleep");
  expect(view("owner")).toContain("Sleeper: draw Sleep Talk");
  expect(view("other")).not.toContain("Sleeper: draw Sleep Talk");
});
it("committed draw replaces reroll and GM alone adjudicates failure", () => {
  expect(view("owner", true, true)).toContain("drew Tackle");
  expect(view("owner", true, true)).not.toContain("Sleeper: draw Sleep Talk");
  expect(view("owner", true, true)).not.toContain("Record failed attempt");
  expect(view("gm", true, true)).toContain("Record failed attempt");
});
it("preserves historical encounters", () => {
  expect(view("gm", false)).toBe("");
});
