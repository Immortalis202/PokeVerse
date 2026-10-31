import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { DatabaseSync } from "node:sqlite";

const read = (p) => JSON.parse(readFileSync(p, "utf8"));
const review = read("rules/catalog-reliability-review.json");
test("M4 Bleakwind correction preserves source identity and non-accuracy clauses", () => {
  const correction = review.completed_checks.find((e) => e.id === "M4-C01");
  const bytes = readFileSync(`../${correction.source}`);
  assert.equal(
    createHash("sha256").update(bytes).digest("hex"),
    correction.sha256,
  );
  const source = JSON.parse(bytes);
  assert.equal(source.Attributes.AccuracyReduction, 2);
  assert.equal(correction.printed_page - 133, correction.pdf_page);
  assert.equal(correction.visual_review, true);
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    const card = JSON.parse(
      db
        .prepare("select data from entries where kind='moves' and name=?")
        .get("Bleakwind Storm").data,
    );
    assert.deepEqual(card.Attributes, {
      ...source.Attributes,
      AccuracyReduction: -2,
    });
    assert.deepEqual(card.AddedEffects, source.AddedEffects);
    assert.equal(card.Effect, source.Effect);
    assert.equal(card.Review.fields.Attributes.status, "reviewed");
    assert.equal(card.Review.fields.Attributes.page, 469);
    assert.equal(card.Review.fields.Attributes.pdf_page, 336);
  } finally {
    db.close();
  }
});

test("M4 closes named batches while preserving historical gaps and partial field certification", () => {
  assert.equal(review.status, "complete");
  const closure = read("rules/catalog-reliability-closure.json");
  const s01 = read("rules/species-s01-closure.json");
  assert.equal(closure.status, review.status);
  assert.equal(closure.catalog_hash, s01.catalog_hash);
  assert.equal(closure.printed_certification, "partial-optional");
  assert.equal(closure.missing_references, 0);
  assert.equal(closure.missing_assets, 0);
  assert.equal(closure.compendium_species_checked, 1200);
  assert.equal(closure.species_source_files_verified, 1200);
  assert.equal(closure.reviewed_asset_hashes_verified, 168);
  assert.equal(closure.retained_gm_cost_branches.length, 4);
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    assert.equal(
      closure.catalog_hash,
      JSON.parse(
        db.prepare("select value from metadata where key='hash'").get().value,
      ),
    );
    assert.equal(
      closure.source_hash,
      JSON.parse(
        db.prepare("select value from metadata where key='source_hash'").get()
          .value,
      ),
    );
  } finally {
    db.close();
  }
  assert.equal(
    review.missing_references.length,
    review.baseline.missing_references,
  );
  assert.equal(review.missing_assets.length, review.baseline.missing_assets);
  assert.equal(
    new Set(review.missing_assets.map((e) => e.id)).size,
    review.missing_assets.length,
  );
  for (const issue of [
    ...review.missing_references,
    ...review.missing_assets,
  ]) {
    assert.ok(issue.id && issue.name && issue.field && issue.reason);
    assert.ok(["missing_reference", "missing_asset"].includes(issue.status));
  }
  for (const [file, hash] of Object.entries(review.sources))
    assert.equal(
      createHash("sha256")
        .update(readFileSync(`../Pokerole-Data/${file}`))
        .digest("hex"),
      hash,
    );
});

test("M4-R01 resolves all original references with stable exact identities", () => {
  const registry = read("rules/catalog-reference-review.json");
  for (const correction of registry.entries) {
    const bytes = readFileSync(`../${correction.source}`);
    assert.equal(
      createHash("sha256").update(bytes).digest("hex"),
      correction.sha256,
    );
    assert.equal(correction.printed_page - 133, correction.pdf_page);
  }
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  const get = (kind, id) =>
    JSON.parse(
      db.prepare("select data from entries where kind=? and id=?").get(kind, id)
        .data,
    );
  try {
    const strength = get("moves", "strength_:Strength ");
    assert.equal(strength.Name, "Strength");
    assert.deepEqual(strength.LegacyNames, ["Strength "]);
    assert.deepEqual(
      JSON.parse(
        db
          .prepare("select summary from entries where kind='moves' and id=?")
          .get(strength.AppId).summary,
      ).LegacyNames,
      ["Strength "],
    );
    assert.equal(
      read("../Pokerole-Data/v3.0/Moves/Strength.json").Name,
      "Strength ",
    );
    for (const ref of review.missing_references.filter(
      (r) => r.field === "Moves:Strength",
    ))
      assert.ok(
        get("species", ref.id).Moves.some((m) => m.Name === "Strength"),
        ref.id,
      );
    const gouging = get("species", "1020:gouging-fire");
    assert.equal(gouging.DexID, "1020");
    assert.ok(
      gouging.Moves.some(
        (m) => m.Name === "Fire Blast" && m.Learned === "Expert",
      ),
    );
    assert.ok(!gouging.Moves.some((m) => m.Name === "FIre Blast"));
    const mew = get("species", "0151:mew");
    const names = db
      .prepare("select name from entries where kind='moves' order by name")
      .all()
      .map((r) => r.name);
    assert.deepEqual(mew.Moves.map((m) => m.Name).sort(), names);
    assert.ok(mew.Moves.every((m) => m.Learned === "Ace"));
    assert.equal(mew.Creation.stage, 3);
    assert.match(mew.Creation.review_note, /User ruling/);
    assert.equal(mew.LearnsetRule.kind, "any-catalog-move");
    assert.equal(
      db
        .prepare(
          "select count(*) as n from entries where kind='moves' and name='Any Move'",
        )
        .get().n,
      0,
    );
    const warnings = JSON.parse(
      db.prepare("select value from metadata where key='warnings'").get().value,
    );
    assert.deepEqual(warnings, []);
  } finally {
    db.close();
  }
});
