import { readFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import path from "node:path";

// Exact, reviewed corrections only. Never normalize arbitrary names or borrow forms.
export async function applyReferenceReview(catalog, review, root) {
  for (const correction of review.entries) {
    const bytes = await readFile(path.join(root, correction.source));
    if (createHash("sha256").update(bytes).digest("hex") !== correction.sha256)
      throw new Error(`Reference source changed: ${correction.id}`);
    const entry = catalog[correction.kind].find(
      (e) => e.AppId === correction.app_id,
    );
    if (!entry)
      throw new Error(`Missing reviewed identity: ${correction.app_id}`);
    if (correction.id === "strength-name") {
      if (entry.Name !== "Strength ")
        throw new Error("Strength source-name mismatch");
      if (catalog.moves.some((m) => m.Name === "Strength"))
        throw new Error("Strength name collision");
      entry.Name = "Strength";
      entry.LegacyNames = ["Strength "];
    } else if (correction.id === "gouging-fire-blast") {
      const matching = entry.Moves.filter((m) => m.Name === "FIre Blast");
      if (
        entry.DexID !== "1020" ||
        matching.length !== 1 ||
        matching[0].Learned !== "Expert"
      )
        throw new Error("Gouging Fire identity/rank mismatch");
      matching[0].Name = "Fire Blast";
      entry.Creation.unresolved_moves = entry.Creation.unresolved_moves.filter(
        (n) => n !== "FIre Blast",
      );
    } else if (correction.id === "mew-any-move") {
      const wildcard = entry.Moves.filter((m) => m.Name === "Any Move");
      if (
        wildcard.length !== 1 ||
        wildcard[0].Learned !== "Ace" ||
        entry.Evolutions?.length
      )
        throw new Error("Mew wildcard/evolution source mismatch");
      entry.Moves = entry.Moves.filter((m) => m.Name !== "Any Move");
      const existing = new Set(entry.Moves.map((m) => m.Name));
      for (const m of catalog.moves)
        if (!existing.has(m.Name))
          entry.Moves.push({ Name: m.Name, Learned: "Ace" });
      entry.Creation.stage = 3;
      entry.Creation.review_note =
        "User ruling (30 October 2026): Mew has no evolution and is final stage. MASTER 184 prints Unknown. Any catalog move is learnable at Ace with normal final-stage costs; unsupported battle effects remain unsupported.";
      entry.Creation.unresolved_moves = entry.Creation.unresolved_moves.filter(
        (n) => n !== "Any Move",
      );
      entry.LearnsetRule = {
        kind: "any-catalog-move",
        rank: "Ace",
        source_name: "Any Move",
        printed_page: 184,
        pdf_page: 51,
        user_ruling:
          "Mew is final stage; use normal final-stage learning costs. Learning eligibility does not automate unsupported battle effects.",
      };
    } else throw new Error(`Unknown reference correction: ${correction.id}`);
    entry.ReferenceReview ??= [];
    entry.ReferenceReview.push(correction);
  }
}
