import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { DatabaseSync } from "node:sqlite";
import path from "node:path";

const app = path.resolve(import.meta.dirname, "..");
const read = (name) =>
  JSON.parse(readFileSync(path.join(app, `rules/${name}.json`)));
const registry = read("species-fidelity-review");
const stats = read("species-stat-json-review"),
  evolution = read("species-evolution-json-review"),
  stages = read("species-stage-review"),
  moves = read("species-move-json-review"),
  abilities = read("species-ability-json-review");
const db = new DatabaseSync(path.join(app, "src-tauri/resources/catalog.db"), {
  readOnly: true,
});
const hash = JSON.parse(
  db.prepare("select value from metadata where key='hash'").get().value,
);
const metadata = JSON.parse(
  db.prepare("select value from metadata where key='species_fidelity'").get()
    .value,
);
db.close();
for (const report of [stats, evolution, stages, moves, abilities])
  assert.equal(
    report.catalog_hash,
    hash,
    "Stale audit: refresh all five JSON reports",
  );
for (const [report, fields] of [
  [
    stats,
    [
      "identity_issues",
      "invalid_values",
      "creation_mismatches",
      "relational_mismatches",
    ],
  ],
  [
    evolution,
    [
      "unexpected_runtime_differences",
      "structural_issues",
      "ordinary_cycles",
      "prerequisite_issues",
    ],
  ],
  [stages, ["issues"]],
  [
    moves,
    [
      "identity_issues",
      "differences",
      "invalid_values",
      "relational_mismatches",
    ],
  ],
  [abilities, ["identity_issues", "differences", "relational_mismatches"]],
])
  for (const field of fields)
    assert.deepEqual(report[field], [], `Unresolved ${field}`);
for (const count of [
  stats.species_compared,
  evolution.species.length,
  stages.species.length,
  moves.species_compared,
  abilities.species_compared,
])
  assert.equal(count, 1200);
assert.equal(stats.fields_compared, 13200);
assert.equal(moves.matching_arrays, 1200);
assert.equal(abilities.slots_compared, 2400);
for (const difference of stats.differences) {
  const ruling = registry.user_stat_rulings.find((r) => r.id === difference.id)
    ?.fields[difference.field];
  assert.ok(ruling, "Unclassified stat difference");
  assert.equal(difference.runtime, ruling.resolved);
  assert.equal(difference.override_origin, "user-ruling");
}
const costs = evolution.pending_cost_decisions
  .map((r) => `${r.source_id}->${r.target_id}`)
  .sort();
assert.deepEqual(costs, [...registry.completion.retained_gm_costs].sort());
assert.equal(
  evolution.retained_guidance_discrepancies.length,
  registry.retained_evolution_discrepancies.length,
);
assert.equal(metadata.unresolved_evolution_issues, 0);
assert.equal(registry.status, "complete");
assert.equal(metadata.status, registry.status);
assert.equal(metadata.completion.printed_certification, "partial-optional");
const report = {
  batch: "M4-S01",
  status: "complete",
  date: registry.completion.date,
  scope: registry.completion.scope,
  catalog_hash: hash,
  identities: 1200,
  stat_fields: 13200,
  matching_stat_fields: stats.matching_fields,
  retained_stat_deviations: stats.differences,
  ordinary_evolution_branches: evolution.ordinary_branches.length,
  form_change_branches: evolution.form_change_branches.length,
  retained_evolution_guidance: evolution.retained_guidance_discrepancies,
  retained_gm_cost_branches: evolution.pending_cost_decisions,
  learning_stage_summary: stages.summary,
  matching_learnsets: moves.matching_arrays,
  ordinary_ability_slots: abilities.slots_compared,
  unresolved_json_issues: [],
  printed_certification: "partial-optional",
  scope_limit: registry.completion.scope_limit,
  reports: [
    "species-stat-json-review.json",
    "species-evolution-json-review.json",
    "species-stage-review.json",
    "species-move-json-review.json",
    "species-ability-json-review.json",
  ],
};
writeFileSync(
  path.join(app, "rules/species-s01-closure.json"),
  `${JSON.stringify(report, null, 2)}\n`,
);
console.log(
  JSON.stringify(
    {
      status: report.status,
      identities: report.identities,
      unresolved_json_issues: 0,
      retained_stat_deviations: stats.differences.length,
      retained_gm_cost_branches: costs.length,
      printed_certification: report.printed_certification,
    },
    null,
    2,
  ),
);
