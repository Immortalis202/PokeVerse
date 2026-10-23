// Read-only, content-hashed evidence. Never copy credentials or character payloads into reports.
import { DatabaseSync } from "node:sqlite";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync, realpathSync, existsSync } from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

const hash = (value) => createHash("sha256").update(value).digest("hex");
const json = (file) =>
  JSON.parse(readFileSync(file, "utf8").replace(/^\uFEFF/, ""));
function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === "object")
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map((k) => [k, canonical(value[k])]),
    );
  // SQLite JSON is compared semantically, avoiding harmless object-key ordering changes.
  if (typeof value === "string" && /^[\[{]/.test(value)) {
    try {
      return canonical(JSON.parse(value));
    } catch {
      /* ordinary text */
    }
  }
  return value;
}
export function snapshotDatabase(file) {
  if (!existsSync(file))
    throw new Error(`Profile has not initialized this database: ${file}`);
  const db = new DatabaseSync(file, { readOnly: true });
  try {
    db.exec("BEGIN");
    const integrity = db
      .prepare("PRAGMA integrity_check")
      .get().integrity_check;
    if (integrity !== "ok")
      throw new Error(`Database integrity failed: ${file}`);
    const schema = db.prepare("PRAGMA user_version").get().user_version;
    const tables = {};
    for (const { name } of db
      .prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
      )
      .all()) {
      const rows = db
        .prepare(`SELECT * FROM "${name.replaceAll('"', '""')}"`)
        .all()
        .map((r) => JSON.stringify(canonical(r)))
        .sort();
      tables[name] = { rows: rows.length, sha256: hash(JSON.stringify(rows)) };
    }
    db.exec("COMMIT");
    return { schema, integrity, tables };
  } finally {
    db.close();
  }
}
export function snapshotLab(lab) {
  const root = realpathSync(lab);
  const manifest = json(path.join(root, "manifest.json"));
  const roles = ["host", "gm", "owner1", "owner2", "owner3"];
  if (manifest.format !== 1 || manifest.profiles?.length !== roles.length)
    throw new Error("Unsupported acceptance manifest");
  const profiles = {};
  for (const role of roles) {
    const entry = manifest.profiles.find(
      (p) => p.role === role && p.directory === role,
    );
    if (!entry) throw new Error(`Missing expected profile: ${role}`);
    const directory = realpathSync(path.join(root, role, "portable-data"));
    if (!directory.startsWith(root + path.sep))
      throw new Error("Profile resolves outside the acceptance lab");
    profiles[role] = Object.fromEntries(
      ["library.db", "campaigns.db"].map((file) => [
        file,
        snapshotDatabase(path.join(directory, file)),
      ]),
    );
  }
  return {
    format: 1,
    kind: "lab",
    captured_utc: new Date().toISOString(),
    version: manifest.version,
    profiles,
  };
}
export function snapshotProfile(directory) {
  const root = realpathSync(directory);
  return {
    format: 1,
    kind: "profile",
    captured_utc: new Date().toISOString(),
    profiles: {
      installed: Object.fromEntries(
        ["library.db", "campaigns.db"].map((file) => [
          file,
          snapshotDatabase(path.join(root, file)),
        ]),
      ),
    },
  };
}
export function compareSnapshots(before, after) {
  if (
    before.format !== 1 ||
    after.format !== 1 ||
    !before.profiles ||
    !after.profiles
  )
    throw new Error("Unsupported data snapshot");
  const kind = before.kind ?? "lab";
  if (!["lab", "profile"].includes(kind) || kind !== (after.kind ?? "lab"))
    throw new Error("Incompatible evidence kinds");
  const roles =
    kind === "profile"
      ? ["installed"]
      : ["host", "gm", "owner1", "owner2", "owner3"];
  for (const snapshot of [before, after]) {
    if (
      Object.keys(snapshot.profiles).length !== roles.length ||
      roles.some((role) =>
        ["library.db", "campaigns.db"].some((file) => {
          const db = snapshot.profiles[role]?.[file];
          return (
            !db ||
            !Number.isInteger(db.schema) ||
            db.schema < 1 ||
            db.integrity !== "ok" ||
            !db.tables ||
            Object.keys(db.tables).length === 0
          );
        }),
      )
    )
      throw new Error("Incomplete acceptance data evidence");
  }
  const differences = [];
  for (const role of new Set([
    ...Object.keys(before.profiles),
    ...Object.keys(after.profiles),
  ])) {
    if (
      JSON.stringify(canonical(before.profiles[role])) !==
      JSON.stringify(canonical(after.profiles[role]))
    )
      differences.push(role);
  }
  return {
    equivalent: differences.length === 0,
    changed_profiles: differences,
    note: "Data equality is a gate for the tested profiles; it does not prove installer, native UI or physical LAN acceptance.",
  };
}
function main() {
  const [mode, first, second] = process.argv.slice(2);
  if (["snapshot", "snapshot-profile"].includes(mode) && first && second) {
    if (existsSync(second))
      throw new Error(
        "Choose a new output file; evidence is never overwritten",
      );
    const report =
      mode === "snapshot" ? snapshotLab(first) : snapshotProfile(first);
    writeFileSync(second, JSON.stringify(report, null, 2) + "\n", {
      flag: "wx",
    });
    console.log(`Saved hashed data evidence: ${second}`);
  } else if (mode === "compare" && first && second) {
    const result = compareSnapshots(json(first), json(second));
    console.log(JSON.stringify(result, null, 2));
    if (!result.equivalent) process.exitCode = 1;
  } else
    throw new Error(
      "Usage: node scripts/acceptance-data.mjs snapshot LAB NEW-REPORT.json | snapshot-profile APP-DATA NEW-REPORT.json | compare BEFORE.json AFTER.json. Close all test apps first.",
    );
}
if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href
) {
  try {
    main();
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
