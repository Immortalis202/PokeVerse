import { DatabaseSync } from "node:sqlite";
import { unlink } from "node:fs/promises";
import {
  readdir,
  readFile,
  mkdir,
  writeFile,
  copyFile,
} from "node:fs/promises";
import path from "node:path";
import { createHash } from "node:crypto";
const root = path.resolve(import.meta.dirname, "../..");
const app = path.join(root, "PokeVerse");
const source = path.join(root, "Pokerole-Data");
const catalog = {
  edition: "3.0",
  hash: "",
  species: [],
  moves: [],
  abilities: [],
  natures: [],
  items: [],
  warnings: [],
};
for (const [folder, key] of [
  ["Pokedex", "species"],
  ["Moves", "moves"],
  ["Abilities", "abilities"],
  ["Natures", "natures"],
  ["Items", "items"],
]) {
  const ids = new Set();
  for (const file of (await readdir(path.join(source, "v3.0", folder)))
    .filter((f) => f.endsWith(".json"))
    .sort()) {
    const entry = JSON.parse(
      await readFile(path.join(source, "v3.0", folder, file), "utf8"),
    );
    entry.AppId =
      key === "species"
        ? `${entry.DexID}:${entry._id}`
        : `${entry._id}:${entry.Name}`;
    const id = entry.AppId;
    if (!id || ids.has(id))
      throw new Error(`Missing/duplicate ${key} ID: ${file}`);
    ids.add(id);
    catalog[key].push(entry);
  }
}
const moves = new Set(catalog.moves.map((m) => m.Name)),
  abilities = new Set(catalog.abilities.map((a) => a.Name));
const original = structuredClone(catalog);
for (const s of original.species) {
  for (const m of s.Moves ?? [])
    if (!moves.has(m.Name))
      original.warnings.push(`${s.Name}: unknown move ${m.Name}`);
  for (const a of [s.Ability1, s.Ability2])
    if (a && !abilities.has(a))
      original.warnings.push(`${s.Name}: unknown ability ${a}`);
}
const sourceHash = createHash("sha256")
  .update(JSON.stringify(original))
  .digest("hex");
const audited = JSON.parse(
  await readFile(path.join(app, "rules/catalog-corrections.json"), "utf8"),
);
const reviewed = JSON.parse(
  await readFile(path.join(app, "rules/catalog-reviewed.json"), "utf8"),
);
// Application metadata; source entries remain unchanged on disk.
for (const s of catalog.species) {
  const audit = audited.species[s.AppId];
  const review = reviewed.species[s.AppId];
  if (audit)
    Object.assign(
      s,
      Object.fromEntries(
        Object.entries(audit.fields).filter(
          ([k, v]) =>
            !["Ability1", "Ability2"].includes(k) || !v || abilities.has(v),
        ),
      ),
    );
  if (review) Object.assign(s, review.fields);
  const before = (s.Evolutions ?? []).filter((e) => e.From);
  const after = (s.Evolutions ?? []).filter(
    (e) => e.To && !["Mega", "Form"].includes(e.Kind),
  );
  s.Creation = {
    stage: review?.stage ?? audit?.stage ?? null,
    inferred_stage: before.length && after.length ? 2 : after.length ? 1 : 3,
    baselines: Object.fromEntries(
      ["Strength", "Dexterity", "Vitality", "Special", "Insight"].map((k) => [
        k,
        s[k],
      ]),
    ),
    limits: Object.fromEntries(
      ["Strength", "Dexterity", "Vitality", "Special", "Insight"].map((k) => [
        k,
        s[`Max${k}`],
      ]),
    ),
    source:
      audit || review
        ? "POKEROLE COREBOOK 3.0 POKEDEX MASTER.pdf"
        : "supplied-v3-json",
    audit: audit?.audit ?? "unverified",
    pdf_page: review?.pdf_page ?? audit?.pdf_page ?? null,
    unresolved_moves: (
      audit?.unresolved_moves ?? (s.Moves ?? []).map((m) => m.Name)
    ).filter((m) => !review?.moves_reviewed?.includes(m)),
    unresolved_fields:
      audit || review
        ? ["BaseHP", "Ability1", "Ability2"].filter((k) => {
            if (k in (review?.fields ?? {})) return false;
            if (!(k in (audit?.fields ?? {}))) return true;
            return (
              k.startsWith("Ability") &&
              audit.fields[k] &&
              !abilities.has(audit.fields[k])
            );
          })
        : ["baselines", "limits", "BaseHP", "ordinary abilities"],
    evolution_review: review?.evolution_speed ?? null,
    review_note: review?.review ?? null,
  };
  if (review?.evolution_speed) {
    for (const e of s.Evolutions ?? [])
      if (e.To && e.Kind === "Level") e.Speed = review.evolution_speed;
  }
  if (
    /mega|gigantamax/i.test(s.Name) ||
    ((s.Evolutions ?? []).length &&
      !before.length &&
      !after.length &&
      (s.Evolutions ?? []).every((e) => ["Mega", "Form"].includes(e.Kind)))
  ) {
    s.Creation.stage = null; // Temporary forms are not ordinary evolution stages.
  }
}
for (const s of catalog.species) {
  for (const m of s.Moves ?? [])
    if (!moves.has(m.Name))
      catalog.warnings.push(`${s.Name}: unknown move ${m.Name}`);
  for (const a of [s.Ability1, s.Ability2])
    if (a && !abilities.has(a))
      catalog.warnings.push(`${s.Name}: unknown ability ${a}`);
}
await (
  await import("./catalog-review.mjs")
).auditCatalog(catalog, { root, app, reviewed });
const habitatData = await (
  await import("./habitats.mjs")
).loadHabitats(
  path.join(root, "outputs/habitat-review"),
  new Set(catalog.species.map((s) => s.AppId)),
);
catalog.habitat_data = habitatData;
// Preserve the old source hash only to recognize v0.4 campaigns during migration.
// Operation eligibility uses the corrected catalog hash, including audit status.
catalog.source_hash = sourceHash;
catalog.hash = createHash("sha256")
  .update(JSON.stringify(catalog))
  .digest("hex");
await mkdir(path.join(app, "public/sprites"), { recursive: true });
await mkdir(path.join(app, "src-tauri/resources"), { recursive: true });
// JSON is a build input only; the shipped reference catalog is read-only SQLite.
const dbPath = path.join(app, "src-tauri/resources/catalog.db");
const temporary = `${dbPath}.tmp`;
await unlink(temporary).catch((e) => {
  if (e.code !== "ENOENT") throw e;
});
const db = new DatabaseSync(temporary);
db.exec(`
 PRAGMA foreign_keys=ON;
 PRAGMA user_version=1;
 CREATE TABLE metadata(key TEXT PRIMARY KEY, value TEXT NOT NULL);
 CREATE TABLE entries(kind TEXT NOT NULL, id TEXT NOT NULL, name TEXT NOT NULL,
   dex TEXT, type1 TEXT, type2 TEXT, position INTEGER NOT NULL,
   summary TEXT NOT NULL CHECK(json_valid(summary)), data TEXT NOT NULL CHECK(json_valid(data)),
   PRIMARY KEY(kind,id));
 CREATE INDEX entry_name ON entries(kind,name);
 CREATE INDEX entry_type ON entries(kind,type1);
 CREATE INDEX entry_type2 ON entries(kind,type2);
 CREATE INDEX entry_position ON entries(kind,position);
 CREATE TABLE species_stats(species_id TEXT NOT NULL, attribute TEXT NOT NULL,
   baseline INTEGER NOT NULL, species_limit INTEGER NOT NULL, PRIMARY KEY(species_id,attribute));
 CREATE TABLE learnsets(species_id TEXT NOT NULL, move_name TEXT NOT NULL, rank TEXT NOT NULL,
   PRIMARY KEY(species_id,move_name));
 CREATE INDEX learnset_move ON learnsets(move_name);
 CREATE TABLE species_abilities(species_id TEXT NOT NULL, ability_name TEXT NOT NULL, slot TEXT NOT NULL,
   PRIMARY KEY(species_id,slot));
 CREATE TABLE evolutions(species_id TEXT NOT NULL, position INTEGER NOT NULL, target_name TEXT, parent_name TEXT,
   data TEXT NOT NULL, PRIMARY KEY(species_id,position));
 CREATE INDEX evolution_target ON evolutions(target_name);
 CREATE TABLE habitats(id TEXT PRIMARY KEY, data TEXT NOT NULL);
 CREATE TABLE species_habitats(species_id TEXT NOT NULL, habitat_id TEXT NOT NULL REFERENCES habitats(id),
   review_decision TEXT NOT NULL, provenance TEXT NOT NULL, PRIMARY KEY(species_id,habitat_id));
 CREATE INDEX habitat_species ON species_habitats(habitat_id,species_id);
 BEGIN;
`);
for (const h of habitatData.habitats)
  db.prepare("INSERT INTO habitats VALUES (?,?)").run(
    h.habitat_id,
    JSON.stringify(h),
  );
for (const a of habitatData.assignments)
  db.prepare("INSERT INTO species_habitats VALUES (?,?,?,?)").run(
    a.species_id,
    a.habitat_id,
    a.review_decision,
    JSON.stringify(a.provenance),
  );
const metadata = db.prepare("INSERT INTO metadata VALUES (?,?)");
for (const key of [
  "edition",
  "hash",
  "source_hash",
  "warnings",
  "review_sources",
  "rules_coverage_hash",
])
  metadata.run(key, JSON.stringify(catalog[key]));
const insert = db.prepare("INSERT INTO entries VALUES (?,?,?,?,?,?,?,?,?)");
const stats = db.prepare("INSERT INTO species_stats VALUES (?,?,?,?)");
const learn = db.prepare("INSERT OR IGNORE INTO learnsets VALUES (?,?,?)");
const ability = db.prepare("INSERT INTO species_abilities VALUES (?,?,?)");
const evolution = db.prepare("INSERT INTO evolutions VALUES (?,?,?,?,?)");
for (const kind of ["species", "moves", "abilities", "items", "natures"]) {
  for (const [position, entry] of catalog[kind].entries()) {
    const keys =
      kind === "species"
        ? [
            "AppId",
            "DexID",
            "Name",
            "Type1",
            "Type2",
            "Image",
            "Ability1",
            "Ability2",
            "RecommendedRank",
            "Legendary",
            "GoodStarter",
          ]
        : kind === "moves"
          ? ["AppId", "Name", "Type", "Category"]
          : ["AppId", "Name"];
    const summary = Object.fromEntries(keys.map((k) => [k, entry[k] ?? ""]));
    insert.run(
      kind,
      entry.AppId,
      entry.Name,
      entry.DexID ?? null,
      entry.Type1 ?? entry.Type ?? "",
      entry.Type2 ?? "",
      position,
      JSON.stringify(summary),
      JSON.stringify(entry),
    );
    if (kind === "species") {
      for (const attribute of [
        "Strength",
        "Dexterity",
        "Vitality",
        "Special",
        "Insight",
      ])
        stats.run(
          entry.AppId,
          attribute,
          entry[attribute],
          entry[`Max${attribute}`],
        );
      for (const m of entry.Moves ?? [])
        learn.run(entry.AppId, m.Name, m.Learned ?? "");
      for (const slot of [
        "Ability1",
        "Ability2",
        "HiddenAbility",
        "EventAbilities",
      ])
        if (entry[slot]) ability.run(entry.AppId, entry[slot], slot);
      for (const [i, e] of (entry.Evolutions ?? []).entries())
        evolution.run(
          entry.AppId,
          i,
          e.To ?? null,
          e.From ?? null,
          JSON.stringify(e),
        );
    }
  }
}
db.exec("COMMIT; PRAGMA optimize;");
if (db.prepare("PRAGMA integrity_check").get().integrity_check !== "ok")
  throw new Error("Catalog integrity check failed");
db.close();
// Replace only after a complete, verified build. No user databases are touched.
await unlink(dbPath).catch((e) => {
  if (e.code !== "ENOENT") throw e;
});
await (await import("node:fs/promises")).rename(temporary, dbPath);
for (const obsolete of [
  "public/catalog.json",
  "src-tauri/resources/catalog.json",
])
  await unlink(path.join(app, obsolete)).catch((e) => {
    if (e.code !== "ENOENT") throw e;
  });
for (const s of catalog.species)
  if (s.Image) {
    try {
      await copyFile(
        path.join(source, "images/BoxSprites", s.Image),
        path.join(app, "public/sprites", s.Image),
      );
    } catch {
      /* UI supplies a placeholder for missing source sprites. */
    }
  }
console.log(
  `Catalog 3.0: ${catalog.species.length} species, ${catalog.moves.length} moves, ${catalog.warnings.length} reference warnings. ${catalog.hash.slice(0, 12)}`,
);
