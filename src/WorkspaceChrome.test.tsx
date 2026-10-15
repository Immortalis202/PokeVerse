import { describe, it, expect } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { WorkspaceRail, WorkspaceToolbar } from "./WorkspaceChrome";
import type { Campaign } from "./bindings/Campaign";

const campaign = {
  name: "A very long campaign name",
  host_id: "host",
  gm_id: "gm",
} as Campaign;
describe("Nocturne workspace navigation", () => {
  it("exposes every page by accessible name, selected state and a tooltip", () => {
    const html = renderToStaticMarkup(
      <WorkspaceRail page="Battle" navigate={() => {}} connected desktop />,
    );
    for (const page of [
      "Library",
      "Compendium",
      "Room",
      "Battle",
      "Training encounters",
      "History",
      "Settings",
    ]) {
      expect(html).toContain(`aria-label="${page}"`);
      expect(html).toContain(`aria-hidden="true">${page}</span>`);
    }
    expect(html).toContain('aria-label="Battle" aria-current="page"');
    expect(html.indexOf('aria-label="Preferences"')).toBeGreaterThan(
      html.indexOf('aria-label="Workspace"'),
    );
    expect(html).toContain('aria-label="Local library available offline"');
  });
  it("shows host and GM as independent roles", () => {
    const render = (playerId: string) =>
      renderToStaticMarkup(
        <WorkspaceToolbar
          page="Battle"
          campaign={campaign}
          playerId={playerId}
          connected
          avatar="P"
        />,
      );
    expect(render("host")).toContain('class="role-badge">Host');
    expect(render("host")).not.toContain('class="role-badge">GM');
    expect(render("gm")).toContain('class="role-badge">GM');
    expect(render("gm")).not.toContain('class="role-badge">Host');
    expect(render("player")).toContain('class="role-badge">Player');
  });
  it("preserves campaign identity and paused status when disconnected", () => {
    const html = renderToStaticMarkup(
      <WorkspaceToolbar
        page="Training encounters"
        campaign={campaign}
        playerId="gm"
        connected={false}
        avatar="G"
      />,
    );
    expect(html).toContain('title="A very long campaign name"');
    expect(html).toContain("Host unavailable");
    expect(html).toContain("Training encounters");
  });
});
