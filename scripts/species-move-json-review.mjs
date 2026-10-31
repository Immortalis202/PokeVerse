import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { DatabaseSync } from "node:sqlite";
import { isDeepStrictEqual } from "node:util";
import path from "node:path";

// Exact JSON reconciliation, not printed-page certification or TM/tutor access.
const app = path.resolve(import.meta.dirname, "..");
const folder = path.resolve(app, "../Pokerole-Data/v3.0/Pokedex");
const review = JSON.parse(
  readFileSync(path.join(app, "rules/species-fidelity-review.json")),
);
const manual = JSON.parse(
  readFileSync(path.join(app, "rules/catalog-reviewed.json")),
);
const corrections = JSON.parse(
  readFileSync(path.join(app, "rules/catalog-corrections.json")),
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
const sources = readdirSync(folder)
  .filter((f) => f.endsWith(".json"))
  .sort()
  .map((file) => {
    const bytes = readFileSync(path.join(folder, file));
    const entry = JSON.parse(bytes);
    return {
      id: `${entry.DexID}:${entry._id}`,
      file,
      sha256: createHash("sha256").update(bytes).digest("hex"),
      entry,
    };
  });
const pairs = (moves) =>
  moves
    .map((m) => [m.Name, m.Learned])
    .sort((a, b) => JSON.stringify(a).localeCompare(JSON.stringify(b)));
const report = {
  batch: "M4-S01",
  scope:
    "Every exact species/form Moves array: Name and Learned; JSON comparison only. No printed certification or ordinary ability review.",
  catalog_hash: JSON.parse(
    db.prepare("select value from metadata where key='hash'").get().value,
  ),
  species_compared: 0,
  move_entries_compared: 0,
  matching_arrays: 0,
  identity_issues: [],
  differences: [],
  invalid_values: [],
  relational_mismatches: [],
  user_move_rank_rulings: review.user_move_rank_rulings ?? [],
  entries: [],
};
const seen = new Set();
for (const s of sources) {
  const effective = runtime.get(s.id);
  if (seen.has(s.id) || !effective || effective.Name !== s.entry.Name) {
    report.identity_issues.push({
      id: s.id,
      file: s.file,
      issue: "duplicate or missing/mismatched exact identity",
    });
    continue;
  }
  seen.add(s.id);
  report.species_compared++;
  report.move_entries_compared += s.entry.Moves.length;
  let expected = structuredClone(
    manual.species[s.id]?.fields?.Moves ?? s.entry.Moves,
  );
  const references = JSON.parse(
    readFileSync(path.join(app, "rules/catalog-reference-review.json")),
  ).entries.filter((r) => r.kind === "species" && r.app_id === s.id);
  for (const reference of references) {
    if (reference.id === "gouging-fire-blast") {
      const move = expected.find((m) => m.Name === "FIre Blast");
      if (!move || move.Learned !== "Expert")
        throw new Error("Gouging Fire source changed");
      move.Name = "Fire Blast";
    } else if (reference.id === "mew-any-move") {
      const wildcard = expected.filter((m) => m.Name === "Any Move");
      if (wildcard.length !== 1 || wildcard[0].Learned !== "Ace")
        throw new Error("Mew source changed");
      expected = expected.filter((m) => m.Name !== "Any Move");
      const names = new Set(expected.map((m) => m.Name));
      for (const row of db
        .prepare("select name from entries where kind='moves'")
        .all())
        if (!names.has(row.name))
          expected.push({ Name: row.name, Learned: "Ace" });
    } else throw new Error(`Unhandled reference: ${reference.id}`);
  }
  const rulings = report.user_move_rank_rulings.filter((r) => r.id === s.id);
  for (const r of rulings) {
    const moves = expected.filter((m) => m.Name === r.move);
    if (moves.length !== 1 || moves[0].Learned !== r.expected_rank)
      throw new Error(`Move ruling source changed: ${s.id} ${r.move}`);
    moves[0].Learned = r.resolved_rank;
  }
  for (const [layer, moves] of [
    ["source", s.entry.Moves],
    ["runtime", effective.Moves],
  ]) {
    for (const m of moves) {
      if (
        typeof m.Name !== "string" ||
        !m.Name.trim() ||
        ![
          "Starter",
          "Rookie",
          "Ace",
          "Standard",
          "Advanced",
          "Expert",
          "Master",
          "Champion",
        ].includes(m.Learned)
      )
        report.invalid_values.push({ id: s.id, layer, move: m });
    }
  }
  const matches = isDeepStrictEqual(pairs(expected), pairs(effective.Moves));
  if (matches) report.matching_arrays++;
  const sourceMatches = isDeepStrictEqual(
    pairs(s.entry.Moves),
    pairs(effective.Moves),
  );
  const entry = {
    override_origin: manual.species[s.id]?.fields?.Moves
      ? "manual-review"
      : corrections.species[s.id]?.fields?.Moves
        ? "extracted-correction"
        : "source-or-other",
    id: s.id,
    name: s.entry.Name,
    file: s.file,
    sha256: s.sha256,
    source_moves: s.entry.Moves,
    runtime_moves: effective.Moves,
    source_matches: sourceMatches,
    matches_after_rulings: matches,
    rulings,
    references,
  };
  report.entries.push(entry);
  if (!matches)
    report.differences.push({
      ...entry,
      alternate_form_matches: sources
        .filter(
          (other) =>
            other.id !== s.id &&
            other.entry.Number === s.entry.Number &&
            isDeepStrictEqual(pairs(other.entry.Moves), pairs(effective.Moves)),
        )
        .map((other) => ({
          id: other.id,
          name: other.entry.Name,
          file: other.file,
        })),
      disposition:
        "Pending reconciliation; an alternate-form match is diagnostic evidence, not permission to overwrite a reviewed value.",
    });
  const rows = db
    .prepare("select move_name, rank from learnsets where species_id=?")
    .all(s.id)
    .map((r) => ({ Name: r.move_name, Learned: r.rank }));
  if (
    !isDeepStrictEqual(pairs(rows), [
      ...new Map(
        pairs(effective.Moves).map((p) => [JSON.stringify(p), p]),
      ).values(),
    ])
  )
    report.relational_mismatches.push({
      id: s.id,
      runtime_moves: effective.Moves,
      relational_moves: rows,
      issue:
        "Relational table cannot retain multiple ranks for one move name; inspect duplicate names.",
    });
}
for (const id of runtime.keys())
  if (!seen.has(id))
    report.identity_issues.push({ id, issue: "runtime identity not compared" });
for (const r of report.user_move_rank_rulings)
  if (!seen.has(r.id)) throw new Error(`Uncompared move ruling: ${r.id}`);
db.close();
writeFileSync(
  path.join(app, "rules/species-move-json-review.json"),
  `${JSON.stringify(report, null, 2)}\n`,
);
console.log(
  JSON.stringify(
    {
      ...report,
      invalid_values: report.invalid_values.length,
      relational_mismatches: report.relational_mismatches.length,
      entries: undefined,
      differences: report.differences.map((d) => ({
        id: d.id,
        name: d.name,
        alternate_form_matches: d.alternate_form_matches,
      })),
      user_move_rank_rulings: undefined,
    },
    null,
    2,
  ),
);
if (
  report.identity_issues.length ||
  report.invalid_values.length ||
  report.relational_mismatches.length ||
  report.differences.length
)
  process.exitCode = 1;
