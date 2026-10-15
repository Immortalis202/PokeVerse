import { expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import SpeciesStatCard, { AttributePoints } from "./SpeciesStatCard";
it("shows species baseline dots and remaining capacity without treating them as character allocations", () => {
  const html = renderToStaticMarkup(
    <AttributePoints name="Strength" minimum={2} maximum={4} />,
  );
  expect(html.match(/species-point minimum/g)).toHaveLength(2);
  expect(html.match(/species-point available/g)).toHaveLength(2);
  expect(html).toContain("Strength: minimum 2, maximum 4");
  expect(html).toContain("2 / 4");
});
it("keeps unknown scores unresolved instead of inventing minimums or maximums", () => {
  const html = renderToStaticMarkup(
    <AttributePoints name="Insight" minimum={undefined} maximum={5} />,
  );
  expect(html).not.toContain("species-point minimum");
  expect(html).not.toContain("species-point available");
  expect(html).toContain("unresolved points");
});
it("uses corrected baselines and limits in the reference card and keeps imperial measurements", () => {
  const html = renderToStaticMarkup(
    <SpeciesStatCard
      entry={{
        Name: "Charmander",
        DexID: "0004",
        Strength: 1,
        MaxStrength: 3,
        BaseHP: 3,
        Creation: {
          baselines: { Strength: 2 },
          limits: { Strength: 4 },
          stage: 1,
        },
        Height: { Meters: 0.6, Feet: 2 },
        Weight: { Kilograms: 8, Pounds: 18 },
      }}
      references={[]}
      onSelect={() => {}}
    />,
  );
  expect(html).toContain("Strength: minimum 2, maximum 4");
  expect(html).toContain("0.6 m / 2 ft");
  expect(html).toContain("8 kg / 18 lb");
  expect(html).toContain("First");
});
