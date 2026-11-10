import { readFileSync } from "node:fs";
import assert from "node:assert/strict";
import { test } from "node:test";

const read = (name) => JSON.parse(readFileSync(name, "utf8"));
const ledger = read("rules/move-card-completion.json");
const audit = read("rules/move-effect-audit.json");

test("whole-card ledger accounts for every source identity without route-based certification", () => {
  assert.equal(ledger.moves.length, audit.moves.length);
  const names = new Set();
  for (const card of ledger.moves) {
    assert.ok(!names.has(card.source), card.id);
    names.add(card.source);
    const source = audit.moves.find((move) => move.source === card.source);
    assert.ok(source, card.id);
    assert.equal(card.sha256, source.sha256, card.name);
    assert.equal(card.source, source.source, card.name);
    assert.ok(card.clauses.length, card.name);
    for (const clause of card.clauses) {
      assert.ok(
        ["pending", "automated", "gm-decision-supported", "excluded"].includes(
          clause.status,
        ),
      );
      if (clause.status === "pending")
        assert.ok(
          Number.isInteger(clause.batch) &&
            clause.batch >= 1 &&
            clause.batch <= 12,
        );
      if (["automated", "gm-decision-supported"].includes(clause.status)) {
        assert.ok(
          clause.evidence.length > 0,
          `${card.name}: missing semantic evidence`,
        );
      }
    }
  }
});

if (process.argv.includes("--closure")) {
  assert.equal(
    ledger.state,
    "complete",
    "Whole-card semantic review remains incomplete",
  );
  const pending = ledger.moves.flatMap((card) =>
    card.clauses
      .filter((clause) => clause.status === "pending")
      .map((clause) => `${card.name}:${clause.id}`),
  );
  assert.deepEqual(pending, [], "Unfinished clauses prevent move completion");
}
