import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";

const read = (p) => JSON.parse(readFileSync(p, "utf8"));
const review = read("rules/battlefield-ice-review.json");
test("M3 review preserves source fingerprints and real evidence references", () => {
  const ids = new Set();
  const doc = readFileSync("docs/M3-BATTLEFIELD-ICE-CHECKLIST.md", "utf8");
  for (const source of Object.values(review.sources)) {
    const bytes = readFileSync(`../${source.path}`);
    assert.equal(
      createHash("sha256").update(bytes).digest("hex"),
      source.sha256,
    );
    if (source.effect) assert.equal(JSON.parse(bytes).Effect, source.effect);
    for (const page of source.reviewed_pages ?? []) {
      assert.ok(page.printed > 0);
      assert.equal(page.printed + (source.pdf_offset ?? 0), page.pdf);
    }
  }
  for (const entry of review.entries) {
    assert.ok(!ids.has(entry.id));
    ids.add(entry.id);
    assert.ok(doc.includes(entry.id));
    assert.ok(entry.sources.length > 0);
    for (const ref of entry.sources) {
      assert.ok(review.sources[ref.source]);
      for (const page of ref.pages ?? [])
        assert.ok(
          review.sources[ref.source].reviewed_pages.some(
            (p) => p.printed === page,
          ),
        );
    }
    for (const ref of entry.code)
      assert.ok(
        readFileSync(ref.path, "utf8").includes(ref.symbol),
        `${entry.id}: ${ref.symbol}`,
      );
    for (const ref of entry.existing_fixtures)
      assert.ok(
        readFileSync(ref.path, "utf8").includes(`fn ${ref.name}(`),
        `${entry.id}: ${ref.name}`,
      );
  }
});

test("M3 completion keeps explicit decisions and release compatibility review", () => {
  assert.equal(review.entries.length, 16);
  const version = (v) => v.split(".").map(Number);
  const reviewed = version(review.release),
    current = version(read("package.json").version);
  assert.ok(
    reviewed.every(Number.isInteger) && current.every(Number.isInteger),
  );
  const firstDifference = reviewed.findIndex((part, i) => part !== current[i]);
  assert.ok(
    firstDifference === -1 ||
      reviewed[firstDifference] < current[firstDifference],
  );
  // This is the M3 delivery snapshot; later move batches keep that engine valid.
  assert.ok(
    readFileSync("src-tauri/src/model.rs", "utf8").includes(
      `= "${review.engine}";`,
    ),
  );
  for (const entry of review.entries) {
    assert.ok(
      [
        "retained-existing",
        "verified-existing",
        "implemented",
        "implemented-user-ruling",
        "retained-gm-ruling",
        "retained-gm-parameter",
        "retained-reviewed-policy",
        "implemented-with-gm-ruling",
      ].includes(entry.status),
      entry.id,
    );
    assert.ok(
      entry.baseline_observed &&
        entry.observed &&
        entry.next_step.startsWith("Closed for M3"),
    );
  }
  for (const id of ["M3-I04", "M3-A03", "M3-A04"])
    assert.equal(
      review.entries.find((e) => e.id === id).status,
      "retained-gm-ruling",
    );
  assert.equal(
    review.entries.find((e) => e.id === "M3-W01").status,
    "implemented-user-ruling",
  );
});
