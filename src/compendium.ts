import { ranks } from "./catalog";
import type { CompendiumReference } from "./bindings/CompendiumReference";
export function learnsetGroups(references: CompendiumReference[]) {
  const moves = references.filter((r) => r.kind === "moves");
  const methods = [
    ...ranks,
    ...new Set(moves.map((r) => r.role).filter((r) => !ranks.includes(r))),
  ];
  return methods
    .map((method) => ({
      method,
      moves: moves.filter((r) => r.role === method),
    }))
    .filter((g) => g.moves.length);
}
