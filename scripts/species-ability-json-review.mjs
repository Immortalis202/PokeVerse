import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { DatabaseSync } from "node:sqlite";
import path from "node:path";

const app = path.resolve(import.meta.dirname, "..");
const source = path.resolve(app, "../Pokerole-Data/v3.0/Pokedex");
const manual = JSON.parse(
  readFileSync(path.join(app, "rules/catalog-reviewed.json")),
);
const db = new DatabaseSync(path.join(app, "src-tauri/resources/catalog.db"), {
  readOnly: true,
});
const runtime = new Map(
  db
    .prepare("select id,data from entries where kind='species'")
    .all()
    .map((r) => [r.id, JSON.parse(r.data)]),
);
const report = {
  batch: "M4-S01",
  scope:
    "Exact JSON ordinary Ability1 and Ability2 only; no hidden/event slots, battle automation or printed certification.",
  catalog_hash: JSON.parse(
    db.prepare("select value from metadata where key='hash'").get().value,
  ),
  species_compared: 0,
  slots_compared: 0,
  differences: [],
  identity_issues: [],
  relational_mismatches: [],
  entries: [],
};
const seen = new Set();
for (const file of readdirSync(source)
  .filter((f) => f.endsWith(".json"))
  .sort()) {
  const bytes = readFileSync(path.join(source, file)),
    original = JSON.parse(bytes);
  const id = `${original.DexID}:${original._id}`,
    effective = runtime.get(id);
  if (seen.has(id) || !effective || effective.Name !== original.Name) {
    report.identity_issues.push({ id, file });
    continue;
  }
  seen.add(id);
  report.species_compared++;
  const entry = {
    id,
    name: original.Name,
    file,
    sha256: createHash("sha256").update(bytes).digest("hex"),
    slots: {},
  };
  for (const slot of ["Ability1", "Ability2"]) {
    report.slots_compared++;
    const reviewed = manual.species[id]?.fields ?? {};
    const expected = Object.hasOwn(reviewed, slot)
      ? reviewed[slot]
      : original[slot];
    entry.slots[slot] = {
      source: original[slot],
      runtime: effective[slot],
      expected,
      basis: Object.hasOwn(reviewed, slot)
        ? "retained-manual-review"
        : "exact-form-json",
    };
    if (effective[slot] !== expected)
      report.differences.push({ id, slot, ...entry.slots[slot] });
    const row = db
      .prepare(
        "select ability_name from species_abilities where species_id=? and slot=?",
      )
      .get(id, slot);
    if ((row?.ability_name ?? "") !== (effective[slot] ?? ""))
      report.relational_mismatches.push({ id, slot });
  }
  report.entries.push(entry);
}
for (const id of runtime.keys())
  if (!seen.has(id))
    report.identity_issues.push({ id, issue: "runtime-only identity" });
db.close();
writeFileSync(
  path.join(app, "rules/species-ability-json-review.json"),
  `${JSON.stringify(report, null, 2)}\n`,
);
console.log(JSON.stringify({ ...report, entries: undefined }, null, 2));
if (
  report.differences.length ||
  report.identity_issues.length ||
  report.relational_mismatches.length
)
  process.exitCode = 1;
