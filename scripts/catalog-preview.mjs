import { compendiumDetail } from "./compendium-preview.mjs";
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
export default function catalogPreview() {
  return {
    name: "catalog-preview",
    configureServer(server) {
      const { DatabaseSync } = require("node:sqlite");
      const db = new DatabaseSync("src-tauri/resources/catalog.db", {
        readOnly: true,
      });
      server.httpServer?.once("close", () => db.close());
      server.middlewares.use("/__catalog", (req, res) => {
        try {
          const url = new URL(req.url, "http://localhost");
          const p = url.searchParams;
          const kind = p.get("kind") ?? "";
          let value;
          if (url.pathname === "/get_catalog") {
            const result = {};
            for (const row of db
              .prepare(
                "SELECT key,value FROM metadata WHERE key IN ('edition','hash','warnings')",
              )
              .all())
              result[row.key] = JSON.parse(row.value);
            for (const k of [
              "species",
              "moves",
              "abilities",
              "items",
              "natures",
            ])
              result[k] = db
                .prepare(
                  "SELECT summary FROM entries WHERE kind=? ORDER BY position",
                )
                .all(k)
                .map((r) => JSON.parse(r.summary));
            value = result;
          } else if (
            !["species", "moves", "abilities", "items", "natures"].includes(
              kind,
            )
          ) {
            res.statusCode = 400;
            res.end();
            return;
          } else if (url.pathname === "/compendium_detail") {
            value = compendiumDetail(db, kind, p.get("id") ?? "");
          } else if (url.pathname === "/catalog_entry") {
            const row = db
              .prepare("SELECT data FROM entries WHERE kind=? AND id=?")
              .get(kind, p.get("id") ?? "");
            value = row ? JSON.parse(row.data) : null;
          } else if (url.pathname === "/search_catalog") {
            const query = p.get("query") ?? "";
            const type = p.get("typeFilter") ?? "";
            const offset = Number(p.get("offset") ?? 0);
            if (
              !Number.isInteger(offset) ||
              offset < 0 ||
              offset > 4294967295 ||
              query.length > 200 ||
              type.length > 40
            ) {
              res.statusCode = 400;
              res.end();
              return;
            }
            const where =
              "kind=? AND instr(lower(name),lower(?))>0 AND (?='' OR type1=? OR type2=?)";
            const args = [kind, query, type, type, type];
            value = {
              total: db
                .prepare(`SELECT count(*) AS n FROM entries WHERE ${where}`)
                .get(...args).n,
              entries: db
                .prepare(
                  `SELECT summary FROM entries WHERE ${where} ORDER BY position LIMIT 150 OFFSET ?`,
                )
                .all(...args, offset)
                .map((r) => JSON.parse(r.summary)),
            };
          } else {
            res.statusCode = 404;
            res.end();
            return;
          }
          res.setHeader("Content-Type", "application/json");
          res.end(JSON.stringify(value));
        } catch {
          res.statusCode = 500;
          res.end();
        }
      });
    },
  };
}
