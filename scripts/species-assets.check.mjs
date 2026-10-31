import { test } from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { DatabaseSync } from "node:sqlite";
import { readFile, mkdir, mkdtemp, copyFile, rm } from "node:fs/promises";
import path from "node:path";
import { prepareSpeciesAssets } from "./species-assets.mjs";

const app = path.resolve(import.meta.dirname, "..");
const read = async (file) =>
  JSON.parse(await readFile(path.join(app, file), "utf8"));
const review = await read("rules/species-asset-review.json");
const baseline = await read("rules/catalog-reliability-review.json");
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");

test("M4-A01 covers all 168 exact baseline identities with verified offline image bytes", async () => {
  assert.deepEqual(
    review.entries.map((a) => a.id).sort(),
    baseline.missing_assets.map((a) => a.id).sort(),
  );
  assert.equal(new Set(review.entries.map((a) => a.image)).size, 168);
  assert.equal(review.visual_review.count, 168);
  const db = new DatabaseSync(
    path.join(app, "src-tauri/resources/catalog.db"),
    { readOnly: true },
  );
  try {
    for (const asset of review.entries) {
      const entry = JSON.parse(
        db
          .prepare("select data from entries where kind='species' and id=?")
          .get(asset.id).data,
      );
      const summary = JSON.parse(
        db
          .prepare("select summary from entries where kind='species' and id=?")
          .get(asset.id).summary,
      );
      assert.equal(entry.Name, asset.name);
      assert.equal(entry.Image, asset.image);
      assert.equal(summary.Image, asset.image);
      assert.equal(entry.AssetReview.sha256, asset.sha256);
      for (const filename of [asset.asset, `public/sprites/${asset.image}`]) {
        const bytes = await readFile(path.join(app, filename));
        assert.equal(sha(bytes), asset.sha256, filename);
        assert.equal(bytes.readUInt32BE(16), asset.width);
        assert.equal(bytes.readUInt32BE(20), asset.height);
      }
      assert.equal(new URL(asset.url).hostname, "www.serebii.net");
      assert.ok(asset.source_page_sha256.match(/^[a-f0-9]{64}$/));
    }
    const audit = JSON.parse(
      await readFile(
        path.resolve(app, "../outputs/rule-coverage/catalog-review.json"),
        "utf8",
      ),
    );
    assert.equal(
      audit.issues.filter((i) => i.status === "missing_asset").length,
      0,
    );
  } finally {
    db.close();
  }
});

test("M4-A01 distinguishes alternate forms and records approved default appearances", () => {
  const get = (name) => review.entries.find((a) => a.name === name);
  for (const names of [
    [
      "Terapagos (Normal Form)",
      "Terapagos (Terastal Form)",
      "Terapagos (Stellar Form)",
    ],
    [
      "Tatsugiri (Curly Form)",
      "Tatsugiri (Droopy Form)",
      "Tatsugiri (Stretchy Form)",
    ],
    [
      "Ogerpon (Teal Mask Form)",
      "Ogerpon (Wellspring Mask Form)",
      "Ogerpon (Hearthflame Mask Form)",
      "Ogerpon (Cornerstone Mask Form)",
    ],
    ["Basculegion (Male Form)", "Basculegion (Female Form)"],
    ["Indeedee (Male Form)", "Indeedee (Female Form)"],
    ["Cramorant (Gulping Form)", "Cramorant (Gorging Form)"],
    ["Poltchageist (Artisan Form)", "Poltchageist (Counterfeit Form)"],
    ["Sinistcha (Masterpiece Form)", "Sinistcha (Unremarkable Form)"],
    ["Palafin (Zero Form)", "Palafin (Hero Form)"],
  ])
    assert.equal(new Set(names.map((n) => get(n).sha256)).size, names.length);
  assert.match(get("Greninja (Battle Bond Form)").source_label, /Ash-Greninja/);
  assert.match(get("Maushold").identity_note, /Family of Four/);
  assert.match(get("Dudunsparce").identity_note, /Two-Segment/);
  assert.match(get("Squawkabilly").identity_note, /Green Plumage/);
  assert.equal(get("Rotom (Dex Form)").media_kind, "official-artwork");
});

test("M4-A01 populates a fresh output and refuses corrupted files, changed identities and unsafe paths", async () => {
  await mkdir(path.join(app, "tmp"), { recursive: true });
  const folder = await mkdtemp(path.join(app, "tmp/a01-check-"));
  assert.equal(path.dirname(folder), path.join(app, "tmp"));
  const asset = review.entries[0];
  const fixtureReview = {
    ...review,
    entries: [asset],
    baseline_missing_assets: 1,
  };
  const catalog = () => ({
    species: [
      { AppId: asset.id, Name: asset.name, Image: asset.original_image },
      { AppId: "fixture", Name: "Unchanged source", Image: "base.png" },
    ],
  });
  try {
    await mkdir(path.join(folder, "assets/species"), { recursive: true });
    await mkdir(path.join(folder, "source/images/BoxSprites"), {
      recursive: true,
    });
    await copyFile(path.join(app, asset.asset), path.join(folder, asset.asset));
    await copyFile(
      path.join(app, asset.asset),
      path.join(folder, "source/images/BoxSprites/base.png"),
    );
    const options = { app: folder, source: path.join(folder, "source") };
    const built = catalog();
    await prepareSpeciesAssets(built, fixtureReview, options);
    assert.equal(built.species[0].Image, asset.image);
    assert.equal(built.species[1].Image, "base.png");
    assert.equal(
      sha(await readFile(path.join(folder, "public/sprites/base.png"))),
      asset.sha256,
    );
    assert.equal(
      sha(await readFile(path.join(folder, "public/sprites", asset.image))),
      asset.sha256,
    );
    for (const change of [
      { sha256: "0".repeat(64) },
      { name: "Another form" },
      { asset: "../outside.png" },
    ])
      await assert.rejects(
        prepareSpeciesAssets(
          catalog(),
          { ...fixtureReview, entries: [{ ...asset, ...change }] },
          options,
        ),
      );
  } finally {
    await rm(folder, { recursive: true, force: true });
  }
});
