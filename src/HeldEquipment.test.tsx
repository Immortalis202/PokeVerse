import { expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import SheetEditor from "./SheetEditor";
import { newSheet } from "./sheets";
import type { Catalog } from "./catalog";
import type { WorkflowRequest } from "./bindings/WorkflowRequest";

const catalog: Catalog = {
  edition: "3.0",
  hash: "catalog",
  species: [],
  moves: [],
  abilities: [],
  natures: [],
  items: [],
  warnings: [],
};
function render(kind: string, trainer = false) {
  const sheet = newSheet("player", trainer ? "trainer" : "pokemon");
  sheet.revision = 1;
  sheet.workflow!.status = "strict";
  sheet.inventory = { "Oran Berry": 2, "Pecha Berry": 0 };
  sheet.held_items = [
    {
      id: "held",
      item_id: "oran",
      name: "Oran Berry",
      origin_sheet_id: sheet.id,
      uses: 0,
    },
  ];
  sheet.stored_items = [
    {
      id: "stored",
      item_id: "leftovers",
      name: "Leftovers",
      origin_sheet_id: sheet.id,
      uses: 2,
    },
  ];
  const request: WorkflowRequest = {
    operation_id: "equip-test",
    expected_revision: 1,
    trainer_revision: null,
    rules_version: 1,
    catalog_hash: catalog.hash,
    kind,
    sheet,
    move_name: "",
    replacement: "",
    method: "ordinary",
    target_species: "",
    amount: 0,
    reason: "Prepare equipment",
  };
  return renderToStaticMarkup(
    <SheetEditor
      initial={sheet}
      catalog={catalog}
      trainers={[]}
      close={() => {}}
      saved={async () => {}}
      draftRequest={request}
    />,
  );
}
it("offers owned inventory and stored units for equipment without claiming battle activation", () => {
  const html = render("equip");
  expect(html).toContain("Owned item to equip");
  expect(html).toContain("Oran Berry (2 available)");
  expect(html).toContain("Leftovers — stored, recorded usage 2");
  expect(html).not.toContain("Pecha Berry");
  expect(html).toContain("Battle item effects are not active in this build");
});
it("selects a particular held unit when putting it away", () => {
  const html = render("unequip");
  expect(html).toContain("Held item to put away");
  expect(html).toContain('value="held"');
  expect(html).not.toContain('value="stored"');
});
it("keeps Pokemon equipment operations out of trainer progression", () => {
  expect(render("describe", true)).not.toContain("Equip an owned item");
});
