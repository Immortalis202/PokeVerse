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

test("M4 records exact remaining references and sprites without claiming field verification", () => {
  assert.equal(review.status, "in-progress");
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
