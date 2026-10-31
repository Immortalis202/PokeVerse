import { createHash } from "node:crypto";
import { copyFile, mkdir, readFile } from "node:fs/promises";
import path from "node:path";

// Build input only: reference images are bundled locally, never fetched by the app.
export async function prepareSpeciesAssets(catalog, review, { app, source }) {
  if (
    review.batch !== "M4-A01" ||
    review.entries.length !== review.baseline_missing_assets
  )
    throw new Error("Incomplete species asset review");
  const species = new Map(catalog.species.map((entry) => [entry.AppId, entry]));
  const ids = new Set(),
    images = new Set();
  await mkdir(path.join(app, "public/sprites"), { recursive: true });
  for (const asset of review.entries) {
    const entry = species.get(asset.id);
    if (
      !entry ||
      entry.Name !== asset.name ||
      entry.Image !== asset.original_image ||
      ids.has(asset.id) ||
      images.has(asset.image)
    )
      throw new Error(`Species asset identity changed: ${asset.id}`);
    if (
      !/^m4-a01-[a-z0-9-]+\.png$/.test(asset.image) ||
      asset.asset !== `assets/species/${asset.image}`
    )
      throw new Error(`Unsafe species asset path: ${asset.id}`);
    for (const reference of [asset.url, asset.source_page]) {
      const url = new URL(reference);
      if (url.protocol !== "https:" || url.hostname !== "www.serebii.net")
        throw new Error(`Unreviewed asset reference: ${asset.id}`);
    }
    const bytes = await readFile(path.join(app, asset.asset));
    if (
      bytes.length < 24 ||
      bytes.subarray(0, 8).toString("hex") !== "89504e470d0a1a0a" ||
      createHash("sha256").update(bytes).digest("hex") !== asset.sha256 ||
      bytes.length !== asset.bytes ||
      bytes.readUInt32BE(16) !== asset.width ||
      bytes.readUInt32BE(20) !== asset.height
    )
      throw new Error(`Species asset bytes changed: ${asset.id}`);
    ids.add(asset.id);
    images.add(asset.image);
    await copyFile(
      path.join(app, asset.asset),
      path.join(app, "public/sprites", asset.image),
    );
    entry.Image = asset.image;
    entry.AssetReview = {
      batch: review.batch,
      source: "Serebii.net",
      source_page: asset.source_page,
      url: asset.url,
      sha256: asset.sha256,
      media_kind: asset.media_kind,
      identity_note: asset.identity_note,
      copyright: review.copyright,
    };
  }
  // Copy supplied sprites before auditing: a clean checkout must not report
  // missing assets merely because the previous build has not populated public/.
  for (const entry of catalog.species) {
    if (!entry.Image || ids.has(entry.AppId)) continue;
    try {
      await copyFile(
        path.join(source, "images/BoxSprites", entry.Image),
        path.join(app, "public/sprites", entry.Image),
      );
    } catch (error) {
      if (error.code !== "ENOENT") throw error;
      // The UI retains a named placeholder for any genuinely unavailable source asset.
    }
  }
}
