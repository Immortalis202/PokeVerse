import { readFileSync } from "node:fs";
import assert from "node:assert/strict";
import { test } from "node:test";
import { resolve } from "node:path";
import {
  buildLedger,
  effectSegments,
  batchFor,
} from "./move-card-completion.mjs";

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
            clause.batch <= 11,
        );
      if (["automated", "gm-decision-supported"].includes(clause.status)) {
        assert.ok(
          clause.evidence.length > 0,
          `${card.name}: missing semantic evidence`,
        );
        for (const evidence of clause.evidence) {
          assert.equal(typeof evidence.command, "string");
          assert.equal(typeof evidence.file, "string");
          assert.equal(typeof evidence.test, "string");
          assert.ok(
            readFileSync(evidence.file, "utf8").includes(evidence.test),
            `${card.name}: stale test evidence`,
          );
        }
      }
      if (clause.status === "excluded")
        assert.ok(
          clause.exclusion,
          `${card.name}: missing clause exclusion reason`,
        );
    }
  }
});

test("ledger regenerates from actual source bytes and preserves reviewed decisions", () => {
  assert.deepEqual(buildLedger(audit, ledger, resolve("..")), ledger);
});

test("source drift rejects stale audits and invalidates prior semantic evidence", () => {
  const stale = structuredClone(audit);
  stale.moves[0].sha256 = "outdated-source";
  assert.throws(
    () => buildLedger(stale, ledger, resolve("..")),
    /Source drift/,
  );
  const old = structuredClone(ledger);
  const card = old.moves.find((m) => m.name === "Hold Hands");
  card.sha256 = "outdated-source";
  card.semantic_review_complete = true;
  const refreshed = buildLedger(audit, old, resolve(".."));
  const result = refreshed.moves.find((m) => m.name === "Hold Hands");
  assert.equal(result.semantic_review_complete, false);
  assert.ok(
    result.clauses.every(
      (c) => c.status === "pending" && c.evidence.length === 0,
    ),
  );
});

test("review units retain every effect character and all combat metadata", () => {
  for (const card of ledger.moves) {
    const source = read(resolve("..", card.source));
    const effects = card.clauses.filter((c) => c.field === "Effect");
    assert.equal(
      effects.map((c) => c.value).join(""),
      source.Effect ?? "",
      card.name,
    );
    let offset = 0;
    for (const clause of effects) {
      assert.equal(clause.start, offset);
      assert.equal(clause.value, source.Effect.slice(clause.start, clause.end));
      offset = clause.end;
    }
    for (const field of [
      "Type",
      "Category",
      "Power",
      "Damage1",
      "Damage2",
      "Accuracy1",
      "Accuracy2",
      "Target",
      "Attributes",
      "AddedEffects",
    ]) {
      if (Object.hasOwn(source, field))
        assert.deepEqual(
          card.clauses.find((c) => c.field === field)?.value,
          source[field],
          `${card.name}: ${field}`,
        );
    }
    if (card.exclusion) {
      assert.ok(
        card.clauses.every(
          (c) => c.status === "excluded" && c.exclusion === card.exclusion,
        ),
      );
    }
  }
});

test("segmentation preserves abbreviations, decimal values and compound source prose", () => {
  const text =
    "Increase Sp. Defense by 1. Add 0.5 damage. Whole Scene Duration.";
  assert.deepEqual(
    effectSegments(text).map((c) => c.text),
    [
      "Increase Sp. Defense by 1. ",
      "Add 0.5 damage. ",
      "Whole Scene Duration.",
    ],
  );
  assert.equal(batchFor("Judgment", "Ignore Defenses.", "Effect"), 1);
  assert.equal(batchFor("Spectral Thief", "Single Target.", "Effect"), 2);
  assert.equal(batchFor("Sketch", "Permanent Duration.", "Effect"), 10);
});

if (process.argv.includes("--closure")) {
  assert.deepEqual(
    buildLedger(audit, ledger, resolve("..")),
    ledger,
    "Source drift or stale review ledger",
  );
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
  assert.ok(
    ledger.moves.every(
      (card) => card.exclusion || card.semantic_review_complete === true,
    ),
    "Every in-scope card needs a full semantic review",
  );
}
