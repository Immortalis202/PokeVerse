import { test } from "node:test";
import assert from "node:assert/strict";
import { DatabaseSync } from "node:sqlite";
import { createHash } from "node:crypto";
import catalogPreview from "./catalog-preview.mjs";

test("browser preview serves summaries, queried pages and selected details", () => {
  let handler, close;
  catalogPreview().configureServer({
    httpServer: {
      once: (_, fn) => {
        close = fn;
      },
    },
    middlewares: {
      use: (_, fn) => {
        handler = fn;
      },
    },
  });
  const request = (url) => {
    const response = {
      statusCode: 200,
      setHeader() {},
      end(text) {
        this.value = text ? JSON.parse(text) : null;
      },
    };
    handler({ url }, response);
    return response;
  };
  try {
    const bootstrap = request("/get_catalog").value;
    assert.equal(bootstrap.species.length, 1200);
    assert.ok(!("Moves" in bootstrap.species[0]));
    const first = request("/search_catalog?kind=species&offset=0").value;
    const next = request("/search_catalog?kind=species&offset=150").value;
    assert.equal(first.total, 1200);
    assert.equal(first.entries.length, 150);
    assert.notEqual(first.entries[0].AppId, next.entries[0].AppId);
    const species = bootstrap.species.find((s) => s.Name === "Pikachu");
    const detail = request(
      `/catalog_entry?kind=species&id=${encodeURIComponent(species.AppId)}`,
    ).value;
    assert.ok(detail.Moves.length);
    assert.ok(detail.Creation);
    const compendium = request(
      `/compendium_detail?kind=species&id=${encodeURIComponent(species.AppId)}`,
    ).value;
    assert.equal(compendium.entry.AppId, species.AppId);
    assert.ok(
      compendium.references.some(
        (r) => r.name === "Raichu (Alolan Form)" && r.id,
      ),
    );
    assert.equal(typeof species.RecommendedRank, "string");
    assert.equal(typeof species.Legendary, "boolean");

    const ability = bootstrap.abilities[0];
    assert.ok(
      request(
        `/catalog_entry?kind=abilities&id=${encodeURIComponent(ability.AppId)}`,
      ).value.Description !== undefined,
    );
    assert.equal(
      request("/search_catalog?kind=species&query=Pikachu&typeFilter=Electric")
        .value.entries[0].Type1,
      "Electric",
    );
    assert.equal(request("/catalog_entry?kind=species&id=missing").value, null);
    assert.equal(request("/search_catalog?kind=bad").statusCode, 400);
    assert.equal(
      request("/search_catalog?kind=species&offset=NaN").statusCode,
      400,
    );
    assert.equal(
      request("/search_catalog?kind=species&query=%25").value.entries.every(
        (e) => e.Name.includes("%"),
      ),
      true,
    );
  } finally {
    close();
  }
});

test("SQLite full records preserve the corrected catalog fingerprint", () => {
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    const meta = Object.fromEntries(
      db
        .prepare("SELECT key,value FROM metadata")
        .all()
        .map((r) => [r.key, JSON.parse(r.value)]),
    );
    const catalog = {
      edition: meta.edition,
      hash: "",
      species: [],
      moves: [],
      abilities: [],
      natures: [],
      items: [],
      warnings: meta.warnings,
    };
    for (const kind of ["species", "moves", "abilities", "natures", "items"])
      catalog[kind] = db
        .prepare("SELECT data FROM entries WHERE kind=? ORDER BY position")
        .all(kind)
        .map((r) => JSON.parse(r.data));
    catalog.review_sources = meta.review_sources;
    catalog.rules_coverage_hash = meta.rules_coverage_hash;
    catalog.habitat_data = {
      habitats: db
        .prepare("SELECT data FROM habitats ORDER BY rowid")
        .all()
        .map((r) => JSON.parse(r.data)),
      assignments: db
        .prepare("SELECT * FROM species_habitats ORDER BY rowid")
        .all()
        .map((r) => ({
          species_id: r.species_id,
          habitat_id: r.habitat_id,
          review_decision: r.review_decision,
          provenance: JSON.parse(r.provenance),
        })),
    };
    catalog.source_hash = meta.source_hash;
    assert.equal(
      createHash("sha256").update(JSON.stringify(catalog)).digest("hex"),
      meta.hash,
    );
    assert.equal(
      db.prepare("PRAGMA integrity_check").get().integrity_check,
      "ok",
    );
    assert.throws(() => db.exec("DELETE FROM entries"));
  } finally {
    db.close();
  }
});
