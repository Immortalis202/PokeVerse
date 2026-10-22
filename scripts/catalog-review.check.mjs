import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { DatabaseSync } from "node:sqlite";
import { compendiumDetail } from "./compendium-preview.mjs";
test("review evidence corrects owned catalog data and keeps source JSON unchanged", () => {
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    const move = (name) =>
      JSON.parse(
        db
          .prepare("select data from entries where kind='moves' and name=?")
          .get(name).data,
      );
    assert.equal(move("Submission").Type, "Fight");
    assert.equal(move("Magic Coat").Target, "Self");
    assert.equal(
      JSON.parse(
        readFileSync("../Pokerole-Data/v3.0/Moves/Magic Coat.json", "utf8"),
      ).Target,
      "Foe's Battlefield",
    );
    assert.match(move("Follow Me").Effect, /Reaction 5/);
    assert.match(
      JSON.parse(
        readFileSync("../Pokerole-Data/v3.0/Moves/Follow Me.json", "utf8"),
      ).Effect,
      /Reaction 4/,
    );

    assert.equal(
      JSON.parse(
        readFileSync("../Pokerole-Data/v3.0/Moves/Submission.json", "utf8"),
      ).Type,
      "Fighting",
    );
    assert.equal(move("Protect").Review.fields.Effect.status, "reviewed");
    assert.equal(move("Protect").Review.fields.Effect.page, 498);
    const pikachu = JSON.parse(
      db
        .prepare(
          "select data from entries where kind='species' and name='Pikachu'",
        )
        .get().data,
    );
    assert.equal(pikachu.Review.fields.BaseHP.page, 142);
    assert.equal(pikachu.Review.fields.BaseHP.pdf_page, 9);
    const torrent = JSON.parse(
      db
        .prepare(
          "select data from entries where kind='abilities' and name='Torrent'",
        )
        .get().data,
    );
    assert.match(torrent.Review.notes, /icon grants 1/);
    assert.equal(torrent.Review.fields.Effect.status, "unverified");
    const species = JSON.parse(
      db
        .prepare(
          "select data from entries where kind='species' and name='Eevee'",
        )
        .get().data,
    );
    assert.equal(species.Review.fields.Moves.status, "unverified");
    assert.ok(
      compendiumDetail(db, "species", species.AppId).warnings.some((w) =>
        w.includes("Moves"),
      ),
    );
  } finally {
    db.close();
  }
});
test("coverage and PDF fingerprints are preserved with a complete field-level queue", () => {
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    const meta = Object.fromEntries(
      db
        .prepare("select * from metadata")
        .all()
        .map((r) => [r.key, JSON.parse(r.value)]),
    );
    for (const [name, hash] of Object.entries(meta.review_sources))
      assert.equal(
        createHash("sha256")
          .update(readFileSync(`../Pokerole-Data/${name}`))
          .digest("hex"),
        hash,
      );
    assert.equal(
      createHash("sha256")
        .update(
          readFileSync("rules/coverage.json", "utf8").replace(/\r\n/g, "\n"),
        )
        .digest("hex"),
      meta.rules_coverage_hash,
    );
    const queue = JSON.parse(
      readFileSync("../outputs/rule-coverage/catalog-review.json", "utf8"),
    );
    assert.equal(queue.total_entries, 2658);
    assert.equal(
      queue.issues.filter((i) => i.status === "missing_reference").length,
      35,
    );
    assert.equal(
      queue.issues.filter((i) => i.status === "missing_asset").length,
      168,
    );
    assert.ok(
      queue.issues.some(
        (i) =>
          i.name === "Eevee" &&
          i.field === "Moves" &&
          i.status === "unverified",
      ),
    );
    const tests =
      readFileSync("src-tauri/src/combat_tests.rs", "utf8") +
      readFileSync("src-tauri/src/combat_tests/targeting_tests.rs", "utf8") +
      readFileSync("src-tauri/src/combat_tests/ability_tests.rs", "utf8") +
      readFileSync(
        "src-tauri/src/combat_tests/ability_change_tests.rs",
        "utf8",
      ) +
      readFileSync("src-tauri/src/combat_tests/transfer_tests.rs", "utf8") +
      readFileSync("src-tauri/src/combat_tests/reviewed_tests.rs", "utf8") +
      readFileSync("src-tauri/src/combat_tests/stat_channel_tests.rs", "utf8") +
      readFileSync("src-tauri/src/combat_tests/sequencing_tests.rs", "utf8") +
      readFileSync("src-tauri/src/combat_tests/event_tests.rs", "utf8") +
      readFileSync("src-tauri/src/combat_tests/field_tests.rs", "utf8") +
      readFileSync("src-tauri/src/combat_tests/extra_tests.rs", "utf8") +
      readFileSync("src-tauri/src/combat_tests/special_tests.rs", "utf8") +
      readFileSync("src-tauri/src/combat_tests/tactics_tests.rs", "utf8") +
      readFileSync("src-tauri/src/tests.rs", "utf8");
    const coverage = JSON.parse(readFileSync("rules/coverage.json", "utf8"));
    for (const entry of coverage.entries)
      for (const fixture of entry.fixtures)
        assert.ok(
          tests.includes(`fn ${fixture}(`),
          `Missing coverage fixture ${fixture}`,
        );
  } finally {
    db.close();
  }
});
test("ability inventory exposes reviewed scopes and preserves manual boundaries", () => {
  const registry = JSON.parse(
    readFileSync("rules/ability-coverage.json", "utf8"),
  );
  const coverage = JSON.parse(readFileSync("rules/coverage.json", "utf8"));
  assert.equal(
    coverage.ability_registry_hash,
    createHash("sha256")
      .update(
        readFileSync("rules/ability-coverage.json", "utf8").replace(
          /\r\n/g,
          "\n",
        ),
      )
      .digest("hex"),
  );
  const inventory = JSON.parse(
    readFileSync("../outputs/rule-coverage/ability-coverage.json", "utf8"),
  );
  assert.equal(inventory.entries.length, 305);
  assert.equal(
    inventory.entries.filter((e) => e.status !== "manual").length,
    275,
  );
  assert.equal(
    Object.values(registry.entries).filter((e) => e.status !== "manual").length,
    275,
  );
  assert.equal(
    inventory.entries.find((e) => e.name === "Static").status,
    "automated",
  );
  assert.equal(
    inventory.entries.find((e) => e.name === "Torrent").status,
    "manual",
  );
  assert.equal(
    inventory.entries.find((e) => e.name === "Mold Breaker").status,
    "automated",
  );
  for (const name of ["Tinted Lens", "Unaware", "Stakeout"]) {
    assert.equal(registry.entries[name].status, "automated");
    assert.equal(registry.entries[name].introduced_engine, "0.16.0-core");
  }
  assert.equal(registry.entries["Sheer Force"].status, "automated");
  for (const name of [
    "Stench",
    "Pressure",
    "Tangled Feet",
    "Wind Power",
    "Electromorphosis",
  ]) {
    assert.equal(registry.entries[name].status, "automated");
    assert.equal(registry.entries[name].introduced_engine, "0.17.0-core");
  }
  assert.deepEqual(registry.entries["Sheer Force"].dependencies, []);
  for (const name of [
    "Ball Fetch",
    "Frisk",
    "Cud Chew",
    "Ripen",
    "Symbiosis",
  ]) {
    assert.equal(registry.entries[name].status, "automated");
    assert.equal(registry.entries[name].introduced_engine, "0.18.0-core");
  }
  for (const entry of inventory.entries) {
    if (entry.status === "manual")
      assert.ok(entry.dependencies.length, entry.name);
    else assert.ok(entry.page && entry.scope, entry.name);
  }
  assert.equal(
    inventory.entries.find((e) => e.name === "Color Change").status,
    "automated",
  );
  assert.equal(
    inventory.entries.find((e) => e.name === "Infiltrator").status,
    "automated",
  );
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    const raw = db
      .prepare(
        "select data from entries where kind='abilities' and name='Static'",
      )
      .get();
    const ability = JSON.parse(raw.data);
    assert.equal(ability.Review.fields.Effect.status, "reviewed");
    assert.ok(ability.Review.coverage.includes("three chance dice"));
  } finally {
    db.close();
  }
});

test("special move inventory has stable IDs, manual boundaries and a compatibility fingerprint", () => {
  const registry = JSON.parse(readFileSync("rules/special-moves.json", "utf8"));
  const coverage = JSON.parse(readFileSync("rules/coverage.json", "utf8"));
  assert.equal(
    coverage.special_registry_hash,
    createHash("sha256")
      .update(
        readFileSync("rules/special-moves.json", "utf8").replace(/\r\n/g, "\n"),
      )
      .digest("hex"),
  );
  const inventory = JSON.parse(
    readFileSync("../outputs/rule-coverage/special-move-coverage.json", "utf8"),
  );
  assert.equal(inventory.entries.length, 221);
  assert.equal(registry.entries["Max Guard"].status, "manual");
  assert.ok(
    registry.entries["Bide"].dependencies.includes("manual-wording-decision"),
  );
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    for (const e of inventory.entries) {
      const row = db
        .prepare("select data from entries where kind='moves' and id=?")
        .get(e.id);
      assert.ok(row, e.name);
      const move = JSON.parse(row.data);
      assert.equal(move.Name, e.name);
      assert.equal(move.SpecialRules.status, e.status);
      assert.ok(e.id && e.scope && e.families.length, e.name);
      if (e.status === "automated") assert.ok(e.page, e.name);
      else assert.ok(e.dependencies.length, e.name);
    }
  } finally {
    db.close();
  }
});

test("targeting registry is fingerprinted and catalog definitions preserve exact IDs and boundaries", () => {
  const text = readFileSync("rules/tactics-reviewed.json", "utf8").replace(
    /\r\n/g,
    "\n",
  );
  const registry = JSON.parse(text),
    coverage = JSON.parse(readFileSync("rules/coverage.json", "utf8"));
  assert.equal(
    coverage.tactics_registry_hash,
    createHash("sha256").update(text).digest("hex"),
  );
  const inventory = JSON.parse(
    readFileSync("../outputs/rule-coverage/tactics-coverage.json", "utf8"),
  );
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    for (const e of inventory.entries) {
      const row = db
        .prepare("select data from entries where kind=? and id=?")
        .get(e.kind, e.id);
      assert.ok(row, e.name);
      const value = JSON.parse(row.data);
      assert.equal(value.Name, e.name);
      assert.equal(value.TacticsRules.scope, e.scope);
      assert.ok(e.page, e.name);
      if (e.status === "partial") assert.ok(e.dependencies.length, e.name);
    }
    assert.equal(registry.moves["Shed Tail"].status, "automated");
  } finally {
    db.close();
  }
});

test("decoy corrections preserve source and excluded transformation moves remain reference only", () => {
  const original = JSON.parse(
    readFileSync("../Pokerole-Data/v3.0/Moves/Substitute.json", "utf8"),
  );
  assert.equal(original.Attributes.SwitcherMove, true);
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    const entry = (name) =>
      JSON.parse(
        db
          .prepare("select data from entries where kind='moves' and name=?")
          .get(name).data,
      );
    assert.deepEqual(entry("Substitute").Attributes, {});
    assert.equal(entry("Substitute").SpecialRules.status, "automated");
    for (const name of ["Max Guard", "Breakneck Blitz"])
      assert.match(
        entry(name).Review.coverage,
        /Out of scope.*Dynamax.*Z-Moves/,
      );
    assert.equal(entry("Shed Tail").TacticsRules.status, "automated");
  } finally {
    db.close();
  }
});
