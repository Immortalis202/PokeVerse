import { describe, it, expect } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import TrainingPointPool from "./TrainingPointPool";
import { newSheet } from "./sheets";
describe("trainer point distribution", () => {
  it("shows a separate pool and linked recipients with a review step", () => {
    const t = newSheet("player", "trainer");
    t.name = "Maya";
    t.xp = 77;
    t.workflow!.training_points = 100;
    const p = newSheet("player", "pokemon", t.id);
    p.name = "Pikachu";
    p.workflow!.training_points = 12;
    const html = renderToStaticMarkup(
      <TrainingPointPool
        trainer={t}
        pokemon={[p]}
        catalogHash="catalog"
        onChanged={async () => {}}
        disabled={false}
      />,
    );
    expect(html).toContain("100 TP available");
    expect(html).toContain("Pikachu · 12 TP");
    expect(html).toContain("Review distribution");
    expect(html).toContain("one-for-one");
    expect(html).not.toContain("77 TP");
    expect(html).toContain('aria-label="Points to distribute"');
  });
});
