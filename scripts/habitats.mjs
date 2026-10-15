import { readFile } from "node:fs/promises";
import path from "node:path";
export function parseCsv(text) {
  const rows = [];
  let row = [],
    cell = "",
    quoted = false;
  text = text.replace(/^\uFEFF/, "");
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (c === '"') {
      if (quoted && text[i + 1] === '"') {
        cell += '"';
        i++;
      } else quoted = !quoted;
    } else if (!quoted && (c === "," || c === "\n")) {
      row.push(cell.replace(/\r$/, ""));
      cell = "";
      if (c === "\n") {
        if (row.some(Boolean)) rows.push(row);
        row = [];
      }
    } else cell += c;
  }
  if (quoted) throw Error("Unclosed CSV field");
  if (cell || row.length) {
    row.push(cell.replace(/\r$/, ""));
    rows.push(row);
  }
  const headers = rows.shift();
  return rows.map((r) => {
    if (r.length !== headers.length) throw Error("CSV column mismatch");
    return Object.fromEntries(headers.map((h, i) => [h, r[i]]));
  });
}
export async function loadHabitats(directory, speciesIds) {
  const habitats = parseCsv(
    await readFile(path.join(directory, "habitat-list.csv"), "utf8"),
  );
  const roster = parseCsv(
    await readFile(path.join(directory, "pokemon-habitats-review.csv"), "utf8"),
  );
  const ids = new Set(habitats.map((h) => h.habitat_id));
  const seen = new Set();
  const assignments = [];
  for (const r of roster) {
    if (!speciesIds.has(r.species_id) || seen.has(r.species_id))
      throw Error(`Unknown/duplicate habitat species ${r.species_id}`);
    seen.add(r.species_id);
    const accepted = new Set(
      r.habitat_ids
        .split(";")
        .map((h) => h.trim())
        .filter(Boolean),
    );
    const reviews = JSON.parse(r.biome_reviews_json || "{}");
    const pending = new Set(
      (r.pending_habitat_ids || "")
        .split(";")
        .map((h) => h.trim())
        .filter(Boolean),
    );
    for (const id of new Set([
      ...accepted,
      ...Object.keys(reviews),
      ...pending,
    ])) {
      if (!ids.has(id)) throw Error(`Unknown habitat ${id}`);
      const review = reviews[id] ?? {};
      if (
        accepted.has(id) &&
        (pending.has(id) ||
          ["excluded", "removed"].includes(review.decision) ||
          review.decision === "pending" ||
          review.included_in_pool === false)
      )
        throw Error(`Contradictory accepted habitat ${r.pokemon}: ${id}`);
      assignments.push({
        species_id: r.species_id,
        habitat_id: id,
        review_decision: accepted.has(id)
          ? review.decision || "accepted"
          : pending.has(id)
            ? "pending"
            : review.decision || "excluded",
        provenance: {
          included_in_pool: accepted.has(id),
          conditions: review.conditions || "",
          reason: review.reason || "",
          source_urls: r.source_urls,
          assignment_basis: r.assignment_basis,
        },
      });
    }
  }
  for (const h of habitats)
    h.pokemon_count = assignments.filter(
      (a) => a.habitat_id === h.habitat_id && a.provenance.included_in_pool,
    ).length;
  return { habitats, assignments };
}
