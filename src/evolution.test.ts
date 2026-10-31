import { expect, it } from "vitest";
import { isOrdinaryEvolution } from "./catalog";

it("offers ordinary evolution while excluding explicit and same-species form changes", () => {
  expect(
    isOrdinaryEvolution(
      { DexID: "0281" },
      { DexID: "0475" },
      { Kind: "Stone" },
    ),
  ).toBe(true);
  expect(
    isOrdinaryEvolution(
      { DexID: "0550H" },
      { DexID: "0902F1" },
      { Kind: "Level" },
    ),
  ).toBe(true);
  for (const [source, target, kind] of [
    ["0006", "0006M1", "Mega"],
    ["0681", "0681F1", "Form"],
    ["0898", "0898F2", "Special"],
    ["0492", "0492F1", "Special"],
    ["0646", "0646F1", "Stone"],
  ])
    expect(
      isOrdinaryEvolution({ DexID: source }, { DexID: target }, { Kind: kind }),
    ).toBe(false);
});
