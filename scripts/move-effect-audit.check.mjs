import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";

const read = (p) => JSON.parse(readFileSync(p, "utf8"));
const audit = read("rules/move-effect-audit.json");
const byName = new Map(audit.moves.map((m) => [m.name, m]));

test("move audit covers every supplied card once and preserves exact source evidence", () => {
  assert.equal(byName.size, 892);
  assert.equal(audit.moves.length, 892);
  for (const move of audit.moves) {
    const bytes = readFileSync(`../${move.source}`);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), move.sha256);
    const source = JSON.parse(bytes);
    assert.equal(source.Name, move.name);
    assert.equal(source.Effect, move.effect);
    assert.deepEqual(source.AddedEffects, move.added_effects);
    assert.deepEqual(source.Attributes, move.attributes);
    assert.deepEqual(Object.keys(move.probe).sort(), ["ailments", "baseline"]);
  }
});
test("status queue distinguishes missing clauses from existing scopes and input requirements", () => {
  for (const name of [
    "Confuse Ray",
    "Spore",
    "Thunder Wave",
    "Toxic",
    "Will-O-Wisp",
  ])
    assert.equal(byName.get(name).status_audit.status, "missing-effect");
  assert.equal(
    byName.get("Air Slash").status_audit.status,
    "implemented-clause",
  );
  assert.equal(byName.get("Wild Charge").status_audit.status, "missing-clause");
  assert.equal(
    byName.get("Sparkling Aria").status_audit.status,
    "partial-clause",
  );
  assert.equal(
    byName.get("Healing Wish").status_audit.status,
    "source-discrepancy",
  );
  assert.equal(byName.get("Lunar Dance").route, "input-required");
  assert.equal(byName.get("Hold Hands").route, "gm-choice");
  assert.equal(byName.get("Sappy Seed").route, "excluded");
  assert.equal(byName.get("Max Guard").route, "excluded");
  const missing = audit.moves.filter(
    (m) => m.status_audit?.status === "missing-effect",
  );
  for (const move of missing)
    for (const probe of Object.values(move.probe))
      assert.match(probe.message, /no reviewed executable definition/);
  const planned = audit.moves.filter((m) =>
    m.status_audit?.batch?.startsWith("M2-"),
  );
  assert.equal(planned.length, 107);
  assert.ok(planned.every((m) => m.route !== "excluded"));
});
test("printed review has traceable PDF pages and protects the priority cards", () => {
  const printed = read("rules/move-status-print-review.json");
  assert.equal(
    createHash("sha256")
      .update(readFileSync(`../${printed.source}`))
      .digest("hex"),
    printed.sha256,
  );
  assert.equal(printed.checks.length, 28);
  for (const move of audit.moves.filter(
    (m) => m.status_audit?.batch === "M2-A",
  )) {
    assert.ok(move.printed_review, `Missing printed reference: ${move.name}`);
    assert.deepEqual(
      move.printed_review.printed_pages.map((p) => p - 133),
      move.printed_review.pdf_pages,
    );
  }
  assert.deepEqual(
    byName.get("Wild Charge").printed_review.printed_pages,
    [455],
  );
});
test("audit artifacts reproduce deterministically from cards, registries and frozen route probes", () => {
  execFileSync(
    process.execPath,
    ["scripts/audit-move-effects.mjs", "--check"],
    { stdio: "pipe" },
  );
});
