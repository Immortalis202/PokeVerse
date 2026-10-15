export function compendiumDetail(db, kind, id) {
  const row = db
    .prepare("SELECT data FROM entries WHERE kind=? AND id=?")
    .get(kind, id);
  if (!row) return null;
  const entry = JSON.parse(row.data),
    references = [],
    warnings = [];
  const resolve = (kind, name, role, requirements = {}) => {
    if (!name) return;
    const matches = db
      .prepare(
        "SELECT id,summary FROM entries WHERE kind=? AND name=? ORDER BY id",
      )
      .all(kind, name);
    references.push({
      kind,
      name,
      role,
      requirements,
      id: matches.length === 1 ? matches[0].id : null,
      summary: matches.length === 1 ? JSON.parse(matches[0].summary) : null,
      issue:
        matches.length === 1
          ? null
          : matches.length
            ? "Ambiguous catalog reference"
            : "Missing catalog reference",
    });
  };
  if (kind === "species") {
    for (const [slot, role] of [
      ["Ability1", "Ordinary"],
      ["Ability2", "Ordinary"],
      ["HiddenAbility", "Hidden"],
      ["EventAbilities", "Event"],
    ]) {
      for (const name of Array.isArray(entry[slot])
        ? entry[slot]
        : [entry[slot]])
        if (typeof name === "string") resolve("abilities", name, role);
    }
    for (const m of entry.Moves ?? [])
      resolve("moves", m.Name, m.Learned ?? "Unspecified method");
    for (const e of entry.Evolutions ?? [])
      for (const [field, role] of [
        ["From", "Pre-evolution"],
        ["To", "Evolution"],
      ])
        resolve("species", e[field], role, e);
  }
  for (const field of ["unresolved_fields", "unresolved_moves"])
    if (entry.Creation?.[field]?.length)
      warnings.push(
        `Metadata review required: ${field.replaceAll("_", " ")}: ${entry.Creation[field].join(", ")}`,
      );
  if (entry.Review) {
    const fields = Object.entries(entry.Review.fields)
      .filter(([, v]) => v.status !== "reviewed")
      .map(([k]) => k);
    if (fields.length)
      warnings.push(
        `Unverified metadata: ${fields.join(", ")}. Resolved references are not proof of reviewed rules.`,
      );
    if (entry.Review.notes) warnings.push(entry.Review.notes);
    warnings.push(`Coverage: ${entry.Review.coverage}`);
  }
  if (entry.Creation?.review_note) warnings.push(entry.Creation.review_note);
  return { kind, entry, references, warnings };
}
