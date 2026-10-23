import { test } from "node:test";
import assert from "node:assert/strict";
import { DatabaseSync } from "node:sqlite";
import {
  mkdtempSync,
  rmSync,
  existsSync,
  mkdirSync,
  writeFileSync,
  readFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import {
  snapshotDatabase,
  snapshotLab,
  snapshotProfile,
  compareSnapshots,
} from "./acceptance-data.mjs";

test("acceptance snapshots are read-only, redact payloads and detect data loss", () => {
  const root = mkdtempSync(path.join(tmpdir(), "pokeverse-acceptance-"));
  try {
    const file = path.join(root, "library.db");
    const db = new DatabaseSync(file);
    db.exec(
      "PRAGMA user_version=4; CREATE TABLE sheets(id TEXT PRIMARY KEY,data TEXT); CREATE TABLE metadata(key TEXT PRIMARY KEY,value TEXT)",
    );
    db.prepare("INSERT INTO sheets VALUES (?,?)").run(
      "private-id",
      '{"name":"private-name","hp":4}',
    );
    db.prepare("INSERT INTO metadata VALUES (?,?)").run(
      "player_id",
      "private-owner",
    );
    const before = snapshotDatabase(file);
    assert.equal(before.integrity, "ok");
    assert.equal(before.schema, 4);
    assert.equal(before.tables.sheets.rows, 1);
    assert.ok(!JSON.stringify(before).includes("private-"));
    db.prepare("UPDATE sheets SET data=?").run(
      '{"hp":4,"name":"private-name"}',
    );
    assert.deepEqual(snapshotDatabase(file), before);
    assert.equal(db.prepare("SELECT count(*) AS n FROM sheets").get().n, 1);
    db.exec("DELETE FROM sheets");
    assert.notDeepEqual(snapshotDatabase(file), before);
    db.close();
    const absent = path.join(root, "absent.db");
    assert.throws(() => snapshotDatabase(absent), /not initialized/);
    assert.equal(existsSync(absent), false);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("acceptance comparison rejects incomplete evidence and identifies changed profiles", () => {
  assert.throws(
    () =>
      compareSnapshots(
        { format: 1, profiles: {} },
        { format: 1, profiles: {} },
      ),
    /Incomplete/,
  );
  const profiles = Object.fromEntries(
    ["host", "gm", "owner1", "owner2", "owner3"].map((role) => [
      role,
      {
        "library.db": {
          schema: 4,
          integrity: "ok",
          tables: { sheets: { rows: 1, sha256: "first" } },
        },
        "campaigns.db": {
          schema: 4,
          integrity: "ok",
          tables: { campaigns: { rows: 1, sha256: "first" } },
        },
      },
    ]),
  );
  const before = { format: 1, profiles };
  const after = structuredClone(before);
  assert.equal(compareSnapshots(before, after).equivalent, true);
  after.profiles.owner2["library.db"].tables.sheets.sha256 = "changed";
  assert.deepEqual(compareSnapshots(before, after).changed_profiles, [
    "owner2",
  ]);
});

test("acceptance lab snapshots cover all five profiles and CLI preserves existing evidence", async () => {
  const { spawnSync } = await import("node:child_process");
  const root = mkdtempSync(path.join(tmpdir(), "pokeverse-lab-"));
  try {
    const roles = ["host", "gm", "owner1", "owner2", "owner3"];
    writeFileSync(
      path.join(root, "manifest.json"),
      JSON.stringify({
        format: 1,
        version: "0.30.0",
        profiles: roles.map((role) => ({ role, directory: role })),
      }),
    );
    for (const role of roles) {
      const dir = path.join(root, role, "portable-data");
      mkdirSync(dir, { recursive: true });
      for (const name of ["library.db", "campaigns.db"]) {
        const db = new DatabaseSync(path.join(dir, name));
        db.exec(
          "PRAGMA user_version=4; CREATE TABLE metadata(key TEXT PRIMARY KEY,value TEXT); INSERT INTO metadata VALUES ('test','preserved')",
        );
        db.close();
      }
    }
    assert.deepEqual(Object.keys(snapshotLab(root).profiles), roles);
    const installed = snapshotProfile(path.join(root, "host", "portable-data"));
    assert.equal(
      compareSnapshots(installed, structuredClone(installed)).equivalent,
      true,
    );
    assert.throws(
      () => compareSnapshots(installed, snapshotLab(root)),
      /Incompatible/,
    );

    const output = path.join(root, "evidence.json");
    const cli = new URL("./acceptance-data.mjs", import.meta.url);
    const { fileURLToPath } = await import("node:url");
    const args = [fileURLToPath(cli), "snapshot", root, output];
    assert.equal(spawnSync(process.execPath, args).status, 0);
    const before = readFileSync(output, "utf8");
    assert.equal(spawnSync(process.execPath, args).status, 1);
    assert.equal(readFileSync(output, "utf8"), before);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
