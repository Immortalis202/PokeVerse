import { expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import MoveCompletionControls from "./MoveCompletionControls";
import type { Campaign } from "./bindings/Campaign";
function view(who: string, ended = true, done = false) {
  const campaign = {
    gm_id: "gm",
    battle: {
      active: true,
      round: 2,
      combatants: [
        { id: "a", name: "Caster", owner_id: "owner", deployed: false, hp: 10 },
        { id: "t", name: "Target", owner_id: "foe", deployed: true, hp: 10 },
      ],
      flow: {
        queue: [],
        order: [],
        battlefield: { end_round: ended ? 2 : 0 },
        special: {
          enabled: true,
          effects: [
            {
              name: "grounded:Smack Down",
              source: "a",
              target: "t",
              rounds: null,
              started: 0,
              slot: "ground-immunity",
            },
            {
              name: "Throat Chop",
              source: "a",
              target: "t",
              rounds: null,
              started: 0,
              slot: "persistent-restriction",
            },
            {
              name: "accuracy-loss:Mud Slap",
              source: "a",
              target: "t",
              rounds: null,
              started: 0,
              amount: 1,
            },
            {
              name: "move-block:Mean Look",
              source: "a",
              target: "t",
              rounds: null,
              started: 0,
            },
            {
              name: "ongoing:Bind",
              source: "a",
              target: "t",
              rounds: 4,
              started: done ? 2 : 0,
            },
          ],
        },
      },
    },
  } as unknown as Campaign;
  return renderToStaticMarkup(
    <MoveCompletionControls
      campaign={campaign}
      playerId={who}
      execute={async () => true}
      disabled={false}
    />,
  );
}
it("shows readable scene blocks and ongoing clocks without exposing internal labels", () => {
  const html = view("foe");
  expect(html).toContain("blocked by Mean Look");
  expect(html).toContain("Smack Down: Ground immunity removed");
  expect(html).not.toContain("grounded:");
  expect(html).toContain("Throat Chop: Sound moves disabled");
  expect(html).not.toContain("Clear Throat Chop");
  expect(view("gm")).toContain("Clear Throat Chop");
  expect(html).toContain("Bind ongoing damage");
  expect(html).toContain("4 rounds remaining");
  expect(html).not.toContain("move-block:");
  expect(html).not.toContain("ongoing:");
  expect(html).not.toContain("accuracy-loss:");
  expect(html).toContain("Mud Slap: remove 1 Accuracy successes");
});
it("permits only the source owner or GM to roll due damage, including a retired caster", () => {
  expect(view("owner")).toContain("Roll ongoing damage");
  expect(view("gm")).toContain("Roll ongoing damage");
  expect(view("foe")).not.toContain("Roll ongoing damage");
  expect(view("owner", false)).not.toContain("Roll ongoing damage");
  expect(view("owner", true, true)).not.toContain("Roll ongoing damage");
});
