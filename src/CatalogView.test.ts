import { describe, it, expect } from "vitest";
import { learnsetGroups } from "./compendium";
import type { CompendiumReference } from "./bindings/CompendiumReference";
const move = (name: string, role: string): CompendiumReference => ({
  kind: "moves",
  name,
  role,
  id: null,
  summary: null,
  issue: "Missing catalog reference",
  requirements: {},
});
describe("Compendium learnsets", () => {
  it("orders every rank before recorded non-rank methods, preserving unresolved names and within-rank order", () => {
    const groups = learnsetGroups([
      move("Last", "Champion"),
      move("Tutor move", "Tutor"),
      move("First", "Starter"),
      move("Second", "Starter"),
      move("Machine move", "TM/TR"),
      move("Master move", "Master"),
    ]);
    expect(groups.map((g) => g.method)).toEqual([
      "Starter",
      "Master",
      "Champion",
      "Tutor",
      "TM/TR",
    ]);
    expect(groups[0].moves.map((m) => m.name)).toEqual(["First", "Second"]);
    expect(groups[0].moves[0].issue).toBe("Missing catalog reference");
  });
  it("does not invent moves for an empty learnset", () =>
    expect(learnsetGroups([])).toEqual([]));
});
