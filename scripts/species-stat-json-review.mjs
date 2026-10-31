import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { DatabaseSync } from "node:sqlite";
import path from "node:path";

// Compare the effective runtime catalog, including application overrides, with
// upstream JSON. Agreement is not a printed-source field certification.
const app = path.resolve(import.meta.dirname, "..");
const source = path.resolve(app, "../Pokerole-Data");
const attributes = ["Strength", "Dexterity", "Vitality", "Special", "Insight"];
const fields = ["BaseHP", ...attributes.flatMap((a) => [a, `Max${a}`])];
const corrections = JSON.parse(
  readFileSync(path.join(app, "rules/catalog-corrections.json"), "utf8"),
);
const reviewed = JSON.parse(
  readFileSync(path.join(app, "rules/catalog-reviewed.json"), "utf8"),
);
const fidelity = JSON.parse(
  readFileSync(path.join(app, "rules/species-fidelity-review.json"), "utf8"),
);
const battleForms = JSON.parse(
  readFileSync(path.join(app, "rules/battle-forms.json"), "utf8"),
);
const db = new DatabaseSync(path.join(app, "src-tauri/resources/catalog.db"), {
  readOnly: true,
});
const runtime = new Map(
  db
    .prepare("select id, data from entries where kind='species'")
    .all()
    .map((r) => [r.id, JSON.parse(r.data)]),
);
const report = {
  batch: "M4-S01",
  scope: "Base HP and five baseline/maximum pairs; JSON comparison only",
  source_remote: execFileSync(
    "git",
    ["-C", source, "remote", "get-url", "origin"],
    { encoding: "utf8" },
  ).trim(),
  source_commit: execFileSync("git", ["-C", source, "rev-parse", "HEAD"], {
    encoding: "utf8",
  }).trim(),
  source_tracked_changes: execFileSync(
    "git",
    ["-C", source, "status", "--porcelain", "--untracked-files=no"],
    { encoding: "utf8" },
  ).trim(),
  catalog_hash: JSON.parse(
    db.prepare("select value from metadata where key='hash'").get().value,
  ),
  fields,
  species_compared: 0,
  fields_compared: 0,
  matching_fields: 0,
  user_stat_rulings: [...runtime.values()]
    .filter((s) => s.StatRuling)
    .map((s) => s.StatRuling),
  differences: [],
  identity_issues: [],
  invalid_values: [],
  creation_mismatches: [],
  relational_mismatches: [],
  source_files: [],
};
const seen = new Set();
const sourceEntries = [];
for (const file of readdirSync(path.join(source, "v3.0/Pokedex"))
  .filter((f) => f.endsWith(".json"))
  .sort()) {
  const bytes = readFileSync(path.join(source, "v3.0/Pokedex", file));
  const original = JSON.parse(bytes);
  const id = `${original.DexID}:${original._id}`;
  sourceEntries.push({ id, file, entry: original });
  report.source_files.push({
    id,
    file,
    sha256: createHash("sha256").update(bytes).digest("hex"),
  });
  if (seen.has(id))
    report.identity_issues.push({ id, issue: "duplicate source identity" });
  seen.add(id);
  const effective = runtime.get(id);
  if (!effective || effective.Name !== original.Name) {
    report.identity_issues.push({
      id,
      source_name: original.Name,
      runtime_name: effective?.Name ?? null,
    });
    continue;
  }
  report.species_compared++;
  for (const field of fields) {
    report.fields_compared++;
    if (original[field] === effective[field]) report.matching_fields++;
    else
      report.differences.push({
        id,
        name: original.Name,
        field,
        source: original[field] ?? null,
        runtime: effective[field] ?? null,
        override_origin: Object.hasOwn(
          effective.StatRuling?.fields ?? {},
          field,
        )
          ? "user-ruling"
          : Object.hasOwn(reviewed.species[id]?.fields ?? {}, field)
            ? "manual-review"
            : Object.hasOwn(corrections.species[id]?.fields ?? {}, field)
              ? "extracted-correction"
              : "other-runtime-change",
        disposition:
          "pending reconciliation; JSON difference alone does not establish which value is correct",
      });
    for (const [layer, entry] of [
      ["source", original],
      ["runtime", effective],
    ]) {
      if (
        !Number.isInteger(entry[field]) ||
        entry[field] < 0 ||
        (field === "BaseHP" && entry[field] === 0)
      )
        report.invalid_values.push({
          id,
          layer,
          field,
          value: entry[field] ?? null,
        });
    }
  }
  for (const a of attributes) {
    for (const [layer, entry] of [
      ["source", original],
      ["runtime", effective],
    ])
      if (entry[a] > entry[`Max${a}`])
        report.invalid_values.push({
          id,
          layer,
          field: a,
          issue: "baseline exceeds maximum",
        });
    if (
      effective.Creation.baselines[a] !== effective[a] ||
      effective.Creation.limits[a] !== effective[`Max${a}`]
    )
      report.creation_mismatches.push({ id, attribute: a });
    const row = db
      .prepare(
        "select baseline, species_limit from species_stats where species_id=? and attribute=?",
      )
      .get(id, a);
    if (
      !row ||
      row.baseline !== effective[a] ||
      row.species_limit !== effective[`Max${a}`]
    )
      report.relational_mismatches.push({ id, attribute: a, row: row ?? null });
  }
}
for (const id of runtime.keys())
  if (!seen.has(id))
    report.identity_issues.push({ id, issue: "runtime-only identity" });

report.repaired_form_stats = (
  fidelity.json_form_stat_review?.entries ?? []
).map((form) => {
  const original = sourceEntries.find((s) => s.id === form.id)?.entry;
  const effective = runtime.get(form.id);
  return {
    id: form.id,
    name: form.name,
    source: form.source,
    fields_compared: fields.length,
    differing_fields: fields.filter(
      (field) =>
        !original || !effective || original[field] !== effective[field],
    ),
    battle_mapping: battleForms.species[form.id]
      ? "existing-reviewed-mapping"
      : "manual-rule-guard-retained",
  };
});

// Use exact National Dex identity to bound form candidates. A single shared
// value is not evidence of form contamination: require all eleven fields.
report.form_reconciliation = [];
for (const id of new Set(report.differences.map((d) => d.id))) {
  const original = sourceEntries.find((s) => s.id === id);
  const effective = runtime.get(id);
  const family = sourceEntries.filter(
    (s) => s.id !== id && s.entry.Number === original.entry.Number,
  );
  const candidates = family.map((s) => ({
    id: s.id,
    name: s.entry.Name,
    file: s.file,
    matching_fields: fields.filter((f) => s.entry[f] === effective[f]),
    differing_fields: fields.filter((f) => s.entry[f] !== effective[f]),
  }));
  const matches = candidates.filter((s) => s.differing_fields.length === 0);
  const status = matches.length
    ? "exact-alternate-form-stat-vector"
    : effective.StatRuling
      ? "resolved-user-ruling"
      : "unexplained-by-alternate-form-json";
  report.form_reconciliation.push({
    id,
    name: original.entry.Name,
    status,
    alternate_form_candidates: candidates,
    exact_matches: matches.map((s) => s.id),
    decision: effective.StatRuling
      ? effective.StatRuling.decision
      : matches.length
        ? "Runtime stats match another exact form across all eleven fields; retain this as a form-assignment conflict, not an incorrect alternate-form stat value."
        : "No exact alternate-form stat vector explains this discrepancy; retain for reconciliation without declaring either source correct.",
    ...(id === "0681:aegislash"
      ? {
          user_confirmation:
            "User confirmed Shield Strength 2 / maximum 4 and Sword (Blade) Strength 4 / maximum 8. Base JSON is Shield; Blade is a separate exact identity.",
        }
      : {}),
  });
  for (const difference of report.differences.filter((d) => d.id === id)) {
    difference.form_status = status;
    difference.exact_form_matches = matches.map((s) => s.id);
    difference.disposition = effective.StatRuling
      ? effective.StatRuling.decision
      : matches.length
        ? "form-assignment conflict; alternate-form stats are valid for the matched form, not certification of the current identity"
        : "pending reconciliation; no alternate-form JSON vector explains the difference";
  }
}
db.close();
const output = path.join(app, "rules/species-stat-json-review.json");
writeFileSync(output, `${JSON.stringify(report, null, 2)}\n`);
console.log(
  JSON.stringify(
    {
      source_commit: report.source_commit,
      species_compared: report.species_compared,
      fields_compared: report.fields_compared,
      matching_fields: report.matching_fields,
      differences: report.differences.length,
      affected_species: new Set(report.differences.map((d) => d.id)).size,
      exact_form_matches: report.form_reconciliation.filter(
        (r) => r.exact_matches.length,
      ).length,
      repaired_forms_checked: report.repaired_form_stats.length,
      repaired_form_mismatches: report.repaired_form_stats.filter(
        (s) => s.differing_fields.length,
      ).length,
      unexplained_species: report.form_reconciliation.filter(
        (r) => r.status === "unexplained-by-alternate-form-json",
      ).length,
      identity_issues: report.identity_issues.length,
      invalid_values: report.invalid_values.length,
      creation_mismatches: report.creation_mismatches.length,
      relational_mismatches: report.relational_mismatches.length,
    },
    null,
    2,
  ),
);
if (
  report.source_tracked_changes ||
  report.identity_issues.length ||
  report.invalid_values.length ||
  report.creation_mismatches.length ||
  report.relational_mismatches.length ||
  report.repaired_form_stats.some((s) => s.differing_fields.length)
)
  process.exitCode = 1;
