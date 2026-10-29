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
