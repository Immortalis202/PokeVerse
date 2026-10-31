import { readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { DatabaseSync } from "node:sqlite";
import path from "node:path";

const app = path.resolve(import.meta.dirname, "..");
const read = (p) => JSON.parse(readFileSync(path.join(app, p), "utf8"));
const evolution = read("rules/species-evolution-json-review.json");
const prices = read("rules/manual-fixtures.json");
const db = new DatabaseSync(path.join(app, "src-tauri/resources/catalog.db"), {
  readOnly: true,
});
const hash = JSON.parse(
  db.prepare("select value from metadata where key='hash'").get().value,
);
if (evolution.catalog_hash !== hash)
  throw new Error(
    "Regenerate species-evolution-json-review.mjs before the stage audit",
  );
const entries = new Map(
  db
    .prepare("select id,data from entries where kind='species'")
    .all()
    .map((r) => [r.id, JSON.parse(r.data)]),
);
const incoming = new Map([...entries.keys()].map((id) => [id, []]));
const outgoing = new Map([...entries.keys()].map((id) => [id, []]));
const links = new Map([...entries.keys()].map((id) => [id, []]));
const excluded = (s) =>
  s.DexID === "0000" ||
  /\(Mega\b|Gigantamax/i.test(s.Name) ||
  s.Evolutions.some((r) => r.From && r.Kind === "Mega");
for (const b of evolution.ordinary_branches) {
  incoming.get(b.target_id).push(b.source_id);
  outgoing.get(b.source_id).push(b.target_id);
}
for (const b of evolution.form_change_branches) {
  if (excluded(entries.get(b.source_id)) || excluded(entries.get(b.target_id)))
    continue;
  links.get(b.source_id).push(b.target_id);
  links.get(b.target_id).push(b.source_id);
}
const direct = (id) =>
  outgoing.get(id).length
    ? incoming.get(id).length
      ? 2
      : 1
    : incoming.get(id).length
      ? 3
      : null;
const report = {
  batch: "M4-S01",
  scope:
    "All exact identities' learning-cost stages and price schedules; JSON graph and user policies, no new printed certification",
  catalog_hash: hash,
  species: [],
  issues: [],
  summary: {},
  pricing_policy: {
    free_ordinary_learning:
      "An eligible new move with capacity available is free; stage prices apply to replacement, full capacity or relearning a forgotten move",
    same_rank: prices.move_stage_costs,
    below_rank: prices.move_stage_costs.map((n) => n / 2),
    overrank_per_rank: prices.overrank_per_rank_stage_costs,
    pre_evolution: prices.pre_evolution_recall_costs,
    tm_tr: prices.tm_tr_cost,
    tutor: "GM-set; separate from stage prices",
  },
  name_match_repairs: [],
  implementation_stage_repairs: JSON.parse(
    db.prepare("select value from metadata where key='species_fidelity'").get()
      .value,
  ).learning_stage_review.changes,
};
for (const source of evolution.species) {
  const entry = entries.get(source.id);
  const original = read(`../${source.source}`);
  const sha256 = createHash("sha256")
    .update(readFileSync(path.join(app, "..", source.source)))
    .digest("hex");
  if (sha256 !== source.sha256)
    throw new Error(`Stage source changed: ${source.id}`);
  let expected = null,
    basis = "excluded";
  if (!excluded(entry)) {
    expected = direct(source.id);
    basis = "ordinary-evolution-graph";
    if (expected === null) {
      const pending = [source.id],
        seen = new Set();
      while (pending.length) {
        const id = pending.pop();
        if (seen.has(id)) continue;
        seen.add(id);
        pending.push(...links.get(id));
      }
      const seedStages = new Set(
        [...seen].map(direct).filter((s) => s !== null),
      );
      expected =
        seedStages.size === 0
          ? 3
          : seedStages.size === 1
            ? [...seedStages][0]
            : null;
      basis =
        seedStages.size === 0
          ? "user-standalone-final"
          : seedStages.size === 1
            ? "linked-form-inheritance"
            : "unresolved-form-family";
    }
  }
  if (
    entry.Creation.stage !== expected ||
    (!excluded(entry) && expected === null)
  )
    report.issues.push({
      id: source.id,
      expected,
      runtime: entry.Creation.stage,
      basis,
    });
  if (entry.Creation.inferred_stage !== expected)
    report.issues.push({ id: source.id, issue: "stale-inferred-stage" });
  // Supplied JSON has no stage field. Missing extraction is not evidence that
  // an exact printed card says Unknown; retain that distinction explicitly.
  const sourceStage = entry.Creation.source_stage;
  const stage = entry.Creation.stage;
  const row = {
    id: source.id,
    name: entry.Name,
    source: source.source,
    sha256,
    supplied_json_stage: original.Stage ?? null,
    source_stage: sourceStage,
    effective_stage: stage,
    expected_stage: expected,
    basis,
    incoming: incoming.get(source.id).sort(),
    outgoing: outgoing.get(source.id).sort(),
    resolution: entry.Creation.stage_resolution,
    pricing:
      stage === null
        ? null
        : {
            ordinary_same_rank: prices.move_stage_costs[stage - 1],
            ordinary_below_rank: prices.move_stage_costs[stage - 1] / 2,
            overrank_per_rank: prices.overrank_per_rank_stage_costs[stage - 1],
            pre_evolution:
              stage === 2
                ? prices.pre_evolution_recall_costs.Second
                : prices.pre_evolution_recall_costs.Final,
          },
  };
  if (!excluded(entry) && /mega/i.test(entry.Name))
    report.name_match_repairs.push({ id: source.id, name: entry.Name, stage });
  report.species.push(row);
}
for (const row of report.species) {
  const key =
    row.effective_stage === null
      ? "excluded"
      : ["", "first", "second", "final"][row.effective_stage];
  report.summary[key] = (report.summary[key] ?? 0) + 1;
}
db.close();
writeFileSync(
  path.join(app, "rules/species-stage-review.json"),
  `${JSON.stringify(report, null, 2)}\n`,
);
console.log(
  JSON.stringify(
    {
      identities: report.species.length,
      ...report.summary,
      issues: report.issues.length,
      stage_repairs: report.implementation_stage_repairs.length,
      name_match_repairs: report.name_match_repairs,
    },
    null,
    2,
  ),
);
if (report.issues.length) process.exitCode = 1;
