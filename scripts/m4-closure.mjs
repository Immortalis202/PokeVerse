import assert from "node:assert/strict";
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { createHash } from "node:crypto";
import { DatabaseSync } from "node:sqlite";
import path from "node:path";
import { compendiumDetail } from "./compendium-preview.mjs";

const app = path.resolve(import.meta.dirname, "..");
const read = (name) =>
  JSON.parse(readFileSync(path.join(app, `rules/${name}.json`)));
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const registry = read("catalog-reliability-review");
assert.deepEqual(
  registry.completed_checks
    .filter((c) => c.status === "complete")
    .map((c) => c.id)
    .sort(),
  ["M4-A01", "M4-C01", "M4-R01", "M4-S01"],
);
const s01 = read("species-s01-closure");
const assets = read("species-asset-review");
const dbPath = path.join(app, "src-tauri/resources/catalog.db");
const db = new DatabaseSync(dbPath, { readOnly: true });
const metadata = Object.fromEntries(
  db
    .prepare("select key,value from metadata")
    .all()
    .map((r) => [r.key, JSON.parse(r.value)]),
);
assert.equal(db.prepare("pragma integrity_check").get().integrity_check, "ok");
assert.equal(s01.catalog_hash, metadata.hash);
assert.equal(s01.status, "complete");
for (const file of s01.reports)
  assert.equal(read(file.replace(/\.json$/, "")).catalog_hash, metadata.hash);
assert.equal(read("species-stat-json-review").source_tracked_changes, "");
assert.deepEqual(s01.unresolved_json_issues, []);
assert.equal(metadata.species_fidelity.status, "complete");
assert.deepEqual(metadata.warnings, []);
assert.equal(assets.status, "complete");
assert.equal(registry.reference_resolution.status, "complete");
assert.equal(registry.species_fidelity_progress.status, "complete");
assert.equal(registry.asset_resolution.status, "complete");
const fingerprints = {};
for (const [file, expected] of Object.entries(registry.sources)) {
  const actual = sha(readFileSync(path.resolve(app, "../Pokerole-Data", file)));
  assert.equal(actual, expected);
  fingerprints[file] = actual;
}
for (const source of read("species-stat-json-review").source_files)
  assert.equal(
    sha(
      readFileSync(
        path.resolve(app, "../Pokerole-Data/v3.0/Pokedex", source.file),
      ),
    ),
    source.sha256,
  );
for (const asset of assets.entries) {
  assert.equal(sha(readFileSync(path.join(app, asset.asset))), asset.sha256);
  assert.equal(
    sha(readFileSync(path.join(app, "public/sprites", asset.image))),
    asset.sha256,
  );
}
const counts = Object.fromEntries(
  db
    .prepare("select kind,count(*) as n from entries group by kind")
    .all()
    .map((r) => [r.kind, r.n]),
);
assert.equal(counts.species, 1200);
let references = 0;
const outsideScope = [];
for (const row of db
  .prepare("select id,data from entries where kind='species' order by id")
  .all()) {
  const entry = JSON.parse(row.data),
    detail = compendiumDetail(db, "species", row.id);
  assert.equal(detail.entry.AppId, row.id);
  assert.ok(
    existsSync(path.join(app, "public/sprites", entry.Image)),
    `Missing sprite: ${row.id}`,
  );
  assert.equal(
    detail.references.filter((r) => r.kind === "moves").length,
    entry.Moves.length,
  );
  for (const ref of detail.references) {
    references++;
    if (ref.issue) {
      assert.ok(
        ["Hidden", "Event"].includes(ref.role),
        `Unresolved Compendium reference: ${row.id} ${ref.name}`,
      );
      outsideScope.push({ id: row.id, ...ref });
    } else {
      const target = compendiumDetail(db, ref.kind, ref.id);
      assert.ok(target);
      assert.equal(target.entry.Name, ref.name);
    }
  }
}
const queue = JSON.parse(
  readFileSync(
    path.resolve(app, "../outputs/rule-coverage/catalog-review.json"),
  ),
);
for (const status of ["missing_reference", "missing_asset"])
  assert.equal(queue.issues.filter((i) => i.status === status).length, 0);
db.close();
assert.equal(registry.status, "complete");
const report = {
  batch: "M4",
  status: "complete",
  date: "2026-10-31",
  scope:
    "Named catalog reliability batches C01, R01, S01 JSON reconciliation and A01 assets; final generated-catalog and automated Compendium checks.",
  catalog_hash: metadata.hash,
  source_hash: metadata.source_hash,
  database_sha256: sha(readFileSync(dbPath)),
  source_fingerprints: fingerprints,
  counts,
  missing_references: 0,
  missing_assets: 0,
  species_source_files_verified: 1200,
  reviewed_asset_hashes_verified: assets.entries.length,
  compendium_species_checked: 1200,
  compendium_references_checked: references,
  outside_scope_references: outsideScope,
  retained_stat_deviations: s01.retained_stat_deviations.length,
  retained_evolution_guidance: s01.retained_evolution_guidance,
  retained_gm_cost_branches: s01.retained_gm_cost_branches,
  printed_certification: "partial-optional",
  native_acceptance: "Not performed; M5 remains open.",
  scope_limit:
    "No blanket printed certification, new battle support, native UI, installer or physical-network acceptance. Saved sheets and source data are unchanged.",
};
writeFileSync(
  path.join(app, "rules/catalog-reliability-closure.json"),
  `${JSON.stringify(report, null, 2)}\n`,
);
console.log(
  JSON.stringify(
    {
      ...report,
      source_fingerprints: undefined,
      retained_evolution_guidance: undefined,
      retained_gm_cost_branches: undefined,
      outside_scope_references: outsideScope.length,
    },
    null,
    2,
  ),
);
