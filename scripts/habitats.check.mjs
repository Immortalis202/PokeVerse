import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile, mkdtemp, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { loadHabitats, parseCsv } from "./habitats.mjs";

test("imported habitat pools exactly preserve current accepted CSV entries, forms and decisions", async () => {
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    const directory = path.resolve("../outputs/habitat-review");
    const species = new Set(
      db
        .prepare("SELECT id FROM entries WHERE kind='species'")
        .all()
        .map((r) => r.id),
    );
    const expected = await loadHabitats(directory, species);
    const actual = db
      .prepare("SELECT * FROM species_habitats ORDER BY rowid")
      .all()
      .map((r) => ({
        species_id: r.species_id,
        habitat_id: r.habitat_id,
        review_decision: r.review_decision,
        provenance: JSON.parse(r.provenance),
      }));
    assert.deepEqual(actual, expected.assignments);
    assert.deepEqual(
      db
        .prepare("SELECT data FROM habitats ORDER BY rowid")
        .all()
        .map((r) => JSON.parse(r.data)),
      expected.habitats,
    );
    const roster = parseCsv(
      await readFile(
        path.join(directory, "pokemon-habitats-review.csv"),
        "utf8",
      ),
    );
    for (const row of roster) {
      const accepted = row.habitat_ids
        .split(";")
        .map((s) => s.trim())
        .filter(Boolean)
        .sort();
      assert.deepEqual(
        actual
          .filter(
            (a) =>
              a.species_id === row.species_id && a.provenance.included_in_pool,
          )
          .map((a) => a.habitat_id)
          .sort(),
        accepted,
      );
    }
    assert.ok(actual.some((a) => a.provenance.conditions));
    assert.ok(actual.some((a) => !a.provenance.included_in_pool));
    assert.ok(
      actual.every(
        (a) =>
          !a.provenance.included_in_pool ||
          !["excluded", "pending"].includes(a.review_decision),
      ),
    );
  } finally {
    db.close();
  }
});

test("CSV parser preserves quotes, BOM, commas and multiline notes; contradictory acceptance fails", async () => {
  assert.deepEqual(
    parseCsv('\uFEFFid,note\r\nx,"tree, shore\n""seasonal"""\r\n'),
    [{ id: "x", note: 'tree, shore\n"seasonal"' }],
  );
  assert.throws(() => parseCsv('id,note\nx,"unfinished'), /Unclosed/);
  assert.throws(() => parseCsv("id,note\nx"), /mismatch/);
  const directory = await mkdtemp(path.join(tmpdir(), "pokeverse-habitats-"));
  try {
    await writeFile(
      path.join(directory, "habitat-list.csv"),
      "habitat_id,habitat\nforest,Forest\n",
    );
    await writeFile(
      path.join(directory, "pokemon-habitats-review.csv"),
      "species_id,pokemon,habitat_ids,pending_habitat_ids\npikachu,Pikachu,forest,forest\n",
    );
    await assert.rejects(
      loadHabitats(directory, new Set(["pikachu"])),
      /Contradictory/,
    );
    const review = JSON.stringify({
      forest: { decision: "removed" },
    }).replaceAll('"', '""');
    await writeFile(
      path.join(directory, "pokemon-habitats-review.csv"),
      `species_id,pokemon,habitat_ids,biome_reviews_json\npikachu,Pikachu,forest,"${review}"\n`,
    );
    await assert.rejects(
      loadHabitats(directory, new Set(["pikachu"])),
      /Contradictory/,
    );
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
