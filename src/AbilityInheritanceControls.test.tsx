import { describe, expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import AbilityInheritanceControls from "./AbilityInheritanceControls";
import type { Campaign } from "./bindings/Campaign";
import { newSheet } from "./sheets";

function fixture(): Campaign {
  const sheet = {
    ...newSheet("player", "pokemon"),
    id: "receiver",
    name: "Rook",
    ability: "Receiver",
  };
  return {
    id: "campaign",
    name: "Campaign",
    gm_id: "gm",
    host_id: "host",
    revision: 3,
    catalog_hash: "catalog",
    engine: "0.12.0-core",
    sheets: { receiver: sheet },
    battle: null,
    finished_battles: [],
    events: [],
    battle_undo: [],
    proposals: [],
    training_encounters: [],
    rules_profile: { version: 1, defenses: "split", hp: "vitality" },
    form_memory: {},
    ability_inheritance: {
      minutes: 60,
      selected: { receiver: "copy" },
      history: ["Receiver copied Levitate"],
      copies: {
        copy: {
          id: "copy",
          sheet_id: "receiver",
          donor_sheet_id: "ally",
          donor_name: "Mist",
          ability: "Levitate",
          source: "Receiver",
          battle_id: "battle",
          acquired: 0,
          expires: 1440,
        },
      },
    },
  };
}
function render(campaign: Campaign, playerId: string) {
  return renderToStaticMarkup(
    <AbilityInheritanceControls
      campaign={campaign}
      playerId={playerId}
      execute={async () => true}
      disabled={false}
    />,
  );
}
describe("inherited abilities", () => {
  it("shows provenance and remaining game time with GM-only clock controls", () => {
    const c = fixture();
    const gm = render(c, "gm"),
      player = render(c, "player"),
      host = render(c, "host");
    expect(gm).toContain("Advance ability clock");
    expect(player).not.toContain("Advance ability clock");
    expect(host).not.toContain("Advance ability clock");
    expect(player).toContain("23h 0m remaining");
    expect(player).toContain("Levitate from Mist (Receiver)");
  });
  it("allows owner selection but keeps another player's selector disabled", () => {
    const c = fixture();
    expect(render(c, "host")).toContain(
      'aria-label="Active inheritance for Rook" disabled=""',
    );
    expect(render(c, "player")).not.toContain(
      'aria-label="Active inheritance for Rook" disabled=""',
    );
  });
  it("keeps expired copies visible and excludes historical engines", () => {
    const c = fixture();
    c.ability_inheritance.minutes = 1440;
    c.ability_inheritance.selected = {};
    const html = render(c, "player");
    expect(html).toContain("(expired)");
    expect(html).toContain('value="copy" disabled=""');
    c.engine = "0.11.0-core";
    expect(render(c, "gm")).toBe("");
  });
});
