import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { isDeepStrictEqual } from "node:util";
import { DatabaseSync } from "node:sqlite";
import path from "node:path";

const app = path.resolve(import.meta.dirname, "..");
const read = (p) => JSON.parse(readFileSync(path.join(app, p), "utf8"));
const review = read("rules/species-fidelity-review.json");
const manual = read("rules/catalog-reviewed.json");
const sources = readdirSync(path.join(app, "../Pokerole-Data/v3.0/Pokedex"))
  .filter((f) => f.endsWith(".json"))
  .sort()
  .map((file) => {
    const bytes = readFileSync(
      path.join(app, "../Pokerole-Data/v3.0/Pokedex", file),
    );
    const entry = JSON.parse(bytes);
    return {
      id: `${entry.DexID}:${entry._id}`,
      file,
      sha256: createHash("sha256").update(bytes).digest("hex"),
      entry,
    };
  });
const expected = new Map(
  sources.map((s) => [
    s.id,
    {
      ...s.entry,
      Evolutions: structuredClone(
        manual.species[s.id]?.fields?.Evolutions ?? s.entry.Evolutions ?? [],
      ),
    },
  ]),
);
// Replay only recorded evolution dispositions, so any unrecorded runtime
// alteration is reported independently of the runtime graph scanner.
for (const fix of review.clear_corrections) {
  const row = expected.get(fix.id).Evolutions.find((r) => r.From === fix.from);
  row.From = fix.to;
}
for (const fix of [
  ...(review.source_evolution_corrections ?? []),
  ...(review.json_evolution_normalizations ?? []),
  ...(review.user_evolution_rulings ?? []),
]) {
  const source = expected.get(fix.source_id),
    target = expected.get(fix.target_id);
  for (const [entry, key, name] of [
    [source, "To", target.Name],
    [target, "From", source.Name],
  ]) {
    const index = entry.Evolutions.findIndex((r) => r[key] === name);
    const relation = { [key]: name, ...fix.resolved_conditions };
    if (index < 0) entry.Evolutions.push(relation);
    else entry.Evolutions[index] = relation;
  }
}
for (const fix of review.user_evolution_exclusions ?? []) {
  const source = expected.get(fix.source_id),
    target = expected.get(fix.target_id);
  target.Evolutions = target.Evolutions.filter((r) => r.From !== source.Name);
}
const db = new DatabaseSync(path.join(app, "src-tauri/resources/catalog.db"), {
  readOnly: true,
});
const runtime = new Map(
  db
    .prepare("select id,data from entries where kind='species'")
    .all()
    .map((r) => [r.id, JSON.parse(r.data)]),
);
const byName = new Map([...runtime.values()].map((s) => [s.Name, s]));
const moves = new Set(
  db
    .prepare("select name from entries where kind='moves'")
    .all()
    .map((r) => r.name),
);
const conditions = (r) =>
  Object.fromEntries(
    Object.entries(r).filter(([k]) => !["To", "From"].includes(k)),
  );
const report = {
  batch: "M4-S01",
  scope:
    "Full exact-identity evolution JSON comparison, recorded rulings, ordinary graph and form-change separation; no new printed-source certification",
  catalog_hash: JSON.parse(
    db.prepare("select value from metadata where key='hash'").get().value,
  ),
  species: [],
  recorded_source_differences: [],
  unexpected_runtime_differences: [],
  structural_issues: [],
  retained_guidance_discrepancies: [],
  ordinary_cycles: [],
  ordinary_branches: [],
  form_change_branches: [],
  pending_cost_decisions: [],
  prerequisite_issues: [],
};
const graph = new Map();
for (const source of sources) {
  const entry = runtime.get(source.id);
  report.species.push({
    id: source.id,
    name: source.entry.Name,
    source: `Pokerole-Data/v3.0/Pokedex/${source.file}`,
    sha256: source.sha256,
    source_relation_count: source.entry.Evolutions?.length ?? 0,
    runtime_relation_count: entry?.Evolutions?.length ?? 0,
  });
  if (
    !entry ||
    entry.Name !== source.entry.Name ||
    !isDeepStrictEqual(
      entry.Evolutions ?? [],
      expected.get(source.id).Evolutions,
    )
  )
    report.unexpected_runtime_differences.push({
      id: source.id,
      expected: expected.get(source.id).Evolutions,
      runtime: entry?.Evolutions ?? null,
    });
  if (!entry) continue;
  if (!isDeepStrictEqual(source.entry.Evolutions ?? [], entry.Evolutions ?? []))
    report.recorded_source_differences.push({
      id: source.id,
      name: entry.Name,
      original: source.entry.Evolutions ?? [],
      runtime: entry.Evolutions ?? [],
      evidence: entry.FidelityReview ?? [],
    });
  const relationKeys = new Set();
  for (const relation of entry.Evolutions ?? []) {
    const name = relation.To ?? relation.From;
    const other = byName.get(name);
    const key = `${relation.To ? "To" : "From"}:${name}`;
    if (relationKeys.has(key))
      report.structural_issues.push({
        id: source.id,
        issue: "duplicate-relation",
        relation,
      });
    relationKeys.add(key);
    if (
      !other ||
      other.AppId === entry.AppId ||
      !!relation.To === !!relation.From
    ) {
      report.structural_issues.push({
        id: source.id,
        issue: !other
          ? "missing-reference"
          : other.AppId === entry.AppId
            ? "self-reference"
            : "invalid-direction",
        relation,
      });
      continue;
    }
    if (!relation.To) {
      if (
        !["Mega", "Form"].includes(relation.Kind) &&
        entry.Number !== other.Number &&
        !(other.Evolutions ?? []).some((r) => r.To === entry.Name)
      )
        report.structural_issues.push({
          id: source.id,
          issue: "reverse-only-ordinary-relation",
          relation,
        });
      continue;
    }
    const branch = {
      source_id: source.id,
      source_name: entry.Name,
      target_id: other.AppId,
      target_name: other.Name,
      conditions: conditions(relation),
    };
    if (
      ["Mega", "Form"].includes(relation.Kind) ||
      entry.Number === other.Number
    ) {
      report.form_change_branches.push({
        ...branch,
        progression: "excluded-from-ordinary-evolution",
      });
      continue;
    }
    const reverse = (other.Evolutions ?? []).filter(
      (r) => r.From === entry.Name,
    );
    if (
      reverse.length !== 1 ||
      !isDeepStrictEqual(conditions(relation), conditions(reverse[0]))
    ) {
      const retained = (review.retained_evolution_discrepancies ?? []).find(
        (r) => r.source_id === source.id && r.target_id === other.AppId,
      );
      (retained
        ? report.retained_guidance_discrepancies
        : report.structural_issues
      ).push({
        ...branch,
        issue:
          reverse.length !== 1
            ? "missing-reciprocal"
            : "directional-condition-conflict",
        reverse,
        ...(retained ? { decision: retained.decision } : {}),
      });
    }
    graph.set(source.id, [...(graph.get(source.id) ?? []), other.AppId]);
    let cost = 0;
    if (relation.Kind === "Level") {
      cost = { Fast: 10, Medium: 30, Slow: 50 }[relation.Speed] ?? null;
      if (cost === null)
        report.pending_cost_decisions.push({
          ...branch,
          decision: "GM-set cost pending explicit ruling",
        });
    }
    if (relation.Move && !moves.has(relation.Move))
      report.prerequisite_issues.push({
        ...branch,
        issue: "unknown-required-move",
      });
    if (relation.Kind === "Move" && !relation.Move)
      report.prerequisite_issues.push({
        ...branch,
        issue: "missing-required-move",
      });
    if (["Male", "Female", "Genderless"].includes(relation.Special))
      report.prerequisite_issues.push({
        ...branch,
        issue: "gender-left-in-guidance",
      });
    report.ordinary_branches.push({
      ...branch,
      tp_cost: cost,
      enforced: {
        ...(relation.Gender ? { Gender: relation.Gender } : {}),
        ...(relation.AllowedGenders
          ? { AllowedGenders: relation.AllowedGenders }
          : {}),
        ...(relation.Move ? { Move: relation.Move } : {}),
      },
      guidance: Object.fromEntries(
        Object.entries(conditions(relation)).filter(
          ([k]) =>
            !["Kind", "Speed", "Gender", "AllowedGenders", "Move"].includes(k),
        ),
      ),
      items_consumed: false,
      completion: "GM confirmation required",
    });
  }
}
const finished = new Set(),
  active = new Set();
function visit(id, trail) {
  if (active.has(id)) {
    report.ordinary_cycles.push([...trail, id]);
    return;
  }
  if (finished.has(id)) return;
  active.add(id);
  for (const next of graph.get(id) ?? []) visit(next, [...trail, id]);
  active.delete(id);
  finished.add(id);
}
for (const id of graph.keys()) visit(id, []);
db.close();
writeFileSync(
  path.join(app, "rules/species-evolution-json-review.json"),
  `${JSON.stringify(report, null, 2)}\n`,
);
console.log(
  JSON.stringify(
    Object.fromEntries(
      Object.entries(report)
        .filter(([, v]) => Array.isArray(v))
        .map(([k, v]) => [k, v.length]),
    ),
    null,
    2,
  ),
);
if (
  report.unexpected_runtime_differences.length ||
  report.structural_issues.length ||
  report.ordinary_cycles.length ||
  report.prerequisite_issues.length
)
  process.exitCode = 1;
