import { test } from "node:test";
import assert from "node:assert/strict";
import { DatabaseSync } from "node:sqlite";
import { compendiumDetail } from "./compendium-preview.mjs";
test("compendium exposes direct branches, exact forms, learnset roles and metadata warnings", () => {
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    const detail = (name) =>
      compendiumDetail(
        db,
        "species",
        db
          .prepare("SELECT id FROM entries WHERE kind='species' AND name=?")
          .get(name).id,
      );
    assert.equal(
      detail("Eevee").references.filter((r) => r.role === "Evolution").length,
      8,
    );
    const pika = detail("Pikachu");
    const regional = pika.references.find(
      (r) => r.name === "Raichu (Alolan Form)",
    );
    assert.equal(regional.requirements.Region, "Alola");
    assert.equal(
      compendiumDetail(db, "species", regional.id).entry.Name,
      "Raichu (Alolan Form)",
    );
    assert.ok(
      pika.references.some(
        (r) => r.name === "Pichu" && r.role === "Pre-evolution",
      ),
    );
    assert.equal(
      detail("Mew").references.filter((r) => r.kind === "species").length,
      0,
    );
    assert.ok(detail("Abomasnow (Mega Form)").warnings.length);
    for (const r of pika.references.filter((r) => r.kind === "moves" && r.id)) {
      const move = compendiumDetail(db, "moves", r.id);
      assert.equal(move.entry.Name, r.name);
      assert.equal(move.entry.Type, r.summary.Type);
      assert.equal(move.entry.Category, r.summary.Category);
    }
    assert.ok(
      detail("Rockruff").references.some(
        (r) => r.role === "Event" && r.name === "Own Tempo",
      ),
    );
    assert.equal(compendiumDetail(db, "species", "absent"), null);
  } finally {
    db.close();
  }
});
test("missing and duplicate names never select arbitrary references; original metadata survives", () => {
  const db = new DatabaseSync(":memory:");
  db.exec(
    "CREATE TABLE entries(kind TEXT,id TEXT,name TEXT,summary TEXT,data TEXT)",
  );
  const insert = (kind, id, name, data) =>
    db
      .prepare("INSERT INTO entries VALUES(?,?,?,?,?)")
      .run(
        kind,
        id,
        name,
        JSON.stringify({ AppId: id, Name: name }),
        JSON.stringify(data),
      );
  insert("species", "fixture", "Fixture", {
    Name: "Fixture",
    Ability1: "Duplicate",
    HiddenAbility: "Absent",
    Moves: [{ Name: "Tutor Move", Learned: "Tutor" }],
    Evolutions: [
      { To: "Missing Form", Kind: "Special", Special: "Recorded condition" },
    ],
  });
  insert("abilities", "a", "Duplicate", {});
  insert("abilities", "b", "Duplicate", {});
  try {
    const result = compendiumDetail(db, "species", "fixture");
    assert.equal(result.references[0].issue, "Ambiguous catalog reference");
    assert.ok(result.references.every((r) => r.id === null));
    assert.ok(
      result.references.some(
        (r) => r.role === "Tutor" && r.name === "Tutor Move",
      ),
    );
    assert.equal(
      result.references.at(-1).requirements.Special,
      "Recorded condition",
    );
  } finally {
    db.close();
  }
});
