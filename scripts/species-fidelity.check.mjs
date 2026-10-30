import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { DatabaseSync } from "node:sqlite";
const read = (p) => JSON.parse(readFileSync(p, "utf8"));
const review = read("rules/species-fidelity-review.json");

test("M4-S01 preserves exact source identities and bounds manual certification", () => {
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    assert.equal(review.status, "in-progress");
    for (const source of Object.values(review.sources))
      assert.equal(
        createHash("sha256")
          .update(readFileSync(`../${source.path}`))
          .digest("hex"),
        source.sha256,
      );
    for (const source of review.manual_reviews) {
      const bytes = readFileSync(`../${source.source}`);
      assert.equal(
        createHash("sha256").update(bytes).digest("hex"),
        source.sha256,
      );
      assert.equal(source.printed_page - 133, source.pdf_page);
      const e = JSON.parse(
        db
          .prepare("select data from entries where kind='species' and id=?")
          .get(source.id).data,
      );
      assert.equal(e.Name, source.name);
      for (const field of source.fields) {
        assert.equal(e.Review.fields[field].status, "reviewed");
        assert.equal(e.Review.fields[field].pdf_page, source.pdf_page);
      }
    }
    const e = JSON.parse(
      db
        .prepare(
          "select data from entries where kind='species' and name='Charizard'",
        )
        .get().data,
    );
    assert.equal(e.Review.fields.Moves.status, "reviewed");
    assert.equal(e.Review.fields.Evolutions.status, "unverified");
    const mega = JSON.parse(
      db
        .prepare(
          "select data from entries where kind='species' and name='Charizard (Mega X Form)'",
        )
        .get().data,
    );
    assert.equal(mega.Creation.stage, null);
    assert.equal(mega.Review.fields.Moves.status, "reviewed");
  } finally {
    db.close();
  }
});

test("M4-S01 user stage rulings remain distinct from printed verification", () => {
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    for (const name of [
      "Articuno",
      "Rotom",
      "Deoxys",
      "Groudon (Primal Form)",
      "Necrozma (Ultra Burst Form)",
      "Terapagos (Stellar Form)",
      "Aegislash (Blade Form)",
    ]) {
      const e = JSON.parse(
        db
          .prepare("select data from entries where kind='species' and name=?")
          .get(name).data,
      );
      assert.equal(e.Creation.stage, 3, name);
      assert.equal(e.Creation.stage_ruling.source, "user-ruling", name);
      assert.equal(e.Review.fields.Evolutions.status, "unverified", name);
      assert.match(e.Creation.review_note, /table ruling/);
    }
    const egg = JSON.parse(
      db
        .prepare(
          "select data from entries where kind='species' and id='0000:egg'",
        )
        .get().data,
    );
    assert.equal(egg.Creation.stage_ruling, undefined);
  } finally {
    db.close();
  }
});

test("M4-S01 repairs confirmed identity errors but retains disputed conditions", () => {
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    const get = (id) =>
      JSON.parse(
        db
          .prepare("select data from entries where kind='species' and id=?")
          .get(id).data,
      );
    const floette = get("0670:floette");
    assert.equal(floette.Evolutions[0].From, "Flab\u00e9b\u00e9");
    const pawmot = get("0923:pawmot");
    assert.equal(pawmot.Evolutions[0].From, "Pawmo");
    assert.equal(pawmot.Evolutions[0].Speed, "Slow");
    assert.equal(
      read("../Pokerole-Data/v3.0/Pokedex/Pawmot.json").Evolutions[0].From,
      "Pawmot",
    );
    const audit = JSON.parse(
      db
        .prepare("select value from metadata where key='species_fidelity'")
        .get().value,
    );
    assert.equal(audit.species_total, 1200);
    assert.ok(
      audit.evolution_issues.every((e) =>
        [
          "directional-condition-conflict",
          "missing-reciprocal-relation",
        ].includes(e.issue),
      ),
    );
    assert.ok(
      audit.evolution_issues.some(
        (e) => e.source_name === "Dipplin" && e.target_name === "Hydrapple",
      ),
    );
    assert.ok(
      audit.evolution_issues.some(
        (e) => e.source_name === "Aipom" && e.target_name === "Ambipom",
      ),
    );
  } finally {
    db.close();
  }
});
