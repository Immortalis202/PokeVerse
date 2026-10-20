import { describe, expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import AbilityControls from "./AbilityControls";
import type { Campaign } from "./bindings/Campaign";
function fixture(): Campaign {
  return {
    gm_id: "gm",
    battle: {
      active: true,
      combatants: [
        {
          id: "holder",
          owner_id: "owner",
          name: "Flamigo",
          deployed: true,
          selected_ability: "Costar",
          runtime: { ability: { counters: {}, flags: [] } },
        },
        { id: "ally", name: "Lucario", deployed: false },
      ],
      flow: {
        ability_rules: true,
        ability_events: [
          {
            id: "entry",
            combatant_id: "holder",
            target_id: "holder",
            ability: "Costar",
            kind: "stat-costar",
            move_name: "entry",
            choices: ["ally"],
          },
        ],
        stat_channels: {
          enabled: true,
          history: ["Costar: Lucario -> Flamigo"],
          choices: {
            entry: {
              ally: [
                { key: "Strength", positive: 2, channel: "move" },
                { key: "Strength", positive: 1, channel: "ability" },
              ],
            },
          },
        },
      },
    },
  } as unknown as Campaign;
}
function render(playerId: string) {
  return renderToStaticMarkup(
    <AbilityControls
      campaign={fixture()}
      playerId={playerId}
      execute={async () => true}
      disabled={false}
    />,
  );
}
describe("Costar entry choices", () => {
  it("shows frozen ally amounts separately and lets the owner preview", () => {
    const html = render("owner");
    expect(html).toContain("Lucario");
    expect(html).toContain("Strength +2 (move)");
    expect(html).toContain("Strength +1 (ability)");
    expect(html).toContain("Preview ally");
  });
  it("keeps resolving controls restricted to the owner and appointed GM", () => {
    expect(render("gm")).toContain("Preview ally");
    expect(render("host")).not.toContain("Preview ally");
    expect(render("host")).toContain("Stat copying and transfer history");
  });
});
