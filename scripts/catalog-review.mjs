import { createHash } from "node:crypto";
import { readFile, mkdir, writeFile, access } from "node:fs/promises";
import path from "node:path";
export async function auditCatalog(catalog, { root, app, reviewed }) {
  const sources = {};
  for (const name of ["RULING", "POKEDEX MASTER"]) {
    const filename = `POKEROLE COREBOOK 3.0 ${name}.pdf`;
    sources[filename] = createHash("sha256")
      .update(await readFile(path.join(root, "Pokerole-Data", filename)))
      .digest("hex");
  }
  const overrides = JSON.parse(
    await readFile(path.join(app, "rules/battle-reviewed.json"), "utf8"),
  );
  const issues = [];
  const abilityRegistry = JSON.parse(
    await readFile(path.join(app, "rules/ability-coverage.json"), "utf8"),
  );
  const abilityHash = createHash("sha256")
    .update(
      (
        await readFile(path.join(app, "rules/ability-coverage.json"), "utf8")
      ).replace(/\r\n/g, "\n"),
    )
    .digest("hex");
  const specialText = (
    await readFile(path.join(app, "rules/special-moves.json"), "utf8")
  ).replace(/\r\n/g, "\n");
  const specialRegistry = JSON.parse(specialText);
  const specialHash = createHash("sha256").update(specialText).digest("hex");
  const tacticsText = (
    await readFile(path.join(app, "rules/tactics-reviewed.json"), "utf8")
  ).replace(/\r\n/g, "\n");
  const tacticsRegistry = JSON.parse(tacticsText);
  const tacticsHash = createHash("sha256").update(tacticsText).digest("hex");
  const coverageRegistry = JSON.parse(
    await readFile(path.join(app, "rules/coverage.json"), "utf8"),
  );
  const itemText = (
    await readFile(path.join(app, "rules/item-completion.json"), "utf8")
  ).replace(/\r\n/g, "\n");
  const itemRegistry = JSON.parse(itemText);
  if (
    coverageRegistry.item_completion_registry_hash !==
    createHash("sha256").update(itemText).digest("hex")
  )
    throw new Error("Item registry fingerprint mismatch");
  for (const definition of Object.values(itemRegistry.entries)) {
    const bytes = await readFile(path.join(root, definition.source));
    if (
      createHash("sha256").update(bytes).digest("hex") !== definition.sha256 ||
      JSON.parse(bytes.toString("utf8")).Description !== definition.description
    )
      throw new Error("Item source fingerprint mismatch");
  }
  const interactionText = (
    await readFile(path.join(app, "rules/battle-interactions.json"), "utf8")
  ).replace(/\r\n/g, "\n");
  const interactions = JSON.parse(interactionText);
  if (
    coverageRegistry.interaction_registry_hash !==
    createHash("sha256").update(interactionText).digest("hex")
  )
    throw new Error(
      "Interaction definitions changed without updating the coverage fingerprint",
    );
  const formText = (
    await readFile(path.join(app, "rules/battle-forms.json"), "utf8")
  ).replace(/\r\n/g, "\n");
  if (
    coverageRegistry.form_registry_hash !==
    createHash("sha256").update(formText).digest("hex")
  )
    throw new Error(
      "Form registry changed without updating the rules coverage fingerprint",
    );
  if (coverageRegistry.ability_registry_hash !== abilityHash)
    throw new Error(
      "Ability registry changed without updating the rules coverage fingerprint",
    );
  if (coverageRegistry.special_registry_hash !== specialHash)
    throw new Error(
      "Special move registry changed without updating the coverage fingerprint",
    );
  if (coverageRegistry.tactics_registry_hash !== tacticsHash)
    throw new Error(
      "Targeting registry changed without updating the rules coverage fingerprint",
    );
  for (const kind of ["species", "moves", "abilities", "items", "natures"]) {
    for (const entry of catalog[kind]) {
      const fields =
        kind === "species"
          ? [
              "BaseHP",
              "Strength",
              "MaxStrength",
              "Dexterity",
              "MaxDexterity",
              "Vitality",
              "MaxVitality",
              "Special",
              "MaxSpecial",
              "Insight",
              "MaxInsight",
              "Ability1",
              "Ability2",
              "Moves",
              "Evolutions",
            ]
          : kind === "moves"
            ? [
                "Type",
                "Category",
                "Power",
                "Accuracy1",
                "Accuracy2",
                "Damage1",
                "Damage2",
                "Target",
                "Effect",
                "Attributes",
                "AddedEffects",
              ]
            : ["Effect", "Description"];
      const evidence =
        kind === "species"
          ? reviewed.species[entry.AppId]
          : overrides[kind]?.[entry.AppId];
      if (evidence?.fields) Object.assign(entry, evidence.fields);
      const fieldReview = {};
      for (const field of fields) {
        const verified =
          evidence?.fields && Object.hasOwn(evidence.fields, field);
        fieldReview[field] = {
          status: verified ? "reviewed" : "unverified",
          source: verified
            ? (evidence.source ?? "POKEROLE COREBOOK 3.0 POKEDEX MASTER.pdf")
            : "supplied JSON / extracted PDF metadata",
          page: verified
            ? (evidence.page ??
              (evidence.pdf_page == null ? null : evidence.pdf_page + 133))
            : null,
          pdf_page: verified
            ? (evidence.pdf_page ??
              (evidence.page == null
                ? null
                : evidence.source?.includes("POKEDEX MASTER")
                  ? evidence.page - 133
                  : evidence.page))
            : null,
        };
        if (!verified)
          issues.push({
            kind,
            id: entry.AppId,
            name: entry.Name,
            field,
            status: "unverified",
            reason:
              field === "Moves"
                ? "Rank icons and learning prerequisites need field-level review"
                : field === "Evolutions"
                  ? "Requirements, branches and form identity need field-level review"
                  : "No recorded visual/manual verification",
            pdf_page: entry.Creation?.pdf_page ?? null,
          });
      }
      if (kind === "moves" && specialRegistry.entries[entry.Name])
        entry.SpecialRules = specialRegistry.entries[entry.Name];
      if (
        kind === "moves" &&
        (interactions.moves[entry.Name] ||
          interactions.chance_moves[entry.Name])
      ) {
        const definition =
          interactions.moves[entry.Name] ??
          interactions.chance_moves[entry.Name];
        entry.InteractionRules = {
          engine: interactions.engine,
          status: "automated",
          scope: `Reviewed in engine ${interactions.engine}: ${definition.effect} Uses sourced stat/trait or chance/status interactions; older encounters retain their saved rules.`,
          source: definition.source,
          sha256: definition.sha256,
        };
      }
      if (tacticsRegistry[kind]?.[entry.Name])
        entry.TacticsRules = tacticsRegistry[kind][entry.Name];
      if (kind === "items" && itemRegistry.entries[entry.Name])
        entry.ItemRules = {
          engine: itemRegistry.engine,
          ...itemRegistry.entries[entry.Name],
        };
      entry.Review = {
        version: 1,
        fields: fieldReview,
        coverage:
          (kind === "abilities"
            ? abilityRegistry.entries[entry.Name]?.scope
            : null) ??
          entry.ItemRules?.scope ??
          entry.InteractionRules?.scope ??
          entry.TacticsRules?.scope ??
          entry.SpecialRules?.scope ??
          evidence?.coverage ??
          "GM adjudication",
        notes: evidence?.notes ?? null,
      };
      if (
        kind === "moves" &&
        (entry.Attributes?.MaxMove || entry.Attributes?.ZMove)
      ) {
        entry.Review.coverage =
          "Out of scope: Dynamax, Max Moves (including Max Guard) and Z-Moves. Available in the Compendium for reference only.";
      }
      if (
        evidence?.notes &&
        evidence.coverage?.includes("GM adjudication required")
      )
        issues.push({
          kind,
          id: entry.AppId,
          name: entry.Name,
          field: "rule interpretation",
          status: "adjudication_required",
          reason: evidence.notes,
          pdf_page:
            evidence.page == null
              ? null
              : evidence.source?.includes("POKEDEX MASTER")
                ? evidence.page - 133
                : evidence.page,
        });
      if (kind === "species" && entry.Image) {
        try {
          await access(path.join(app, "public/sprites", entry.Image));
        } catch {
          issues.push({
            kind,
            id: entry.AppId,
            name: entry.Name,
            field: "Image",
            status: "missing_asset",
            reason: "Exact local sprite unavailable; retain named placeholder",
          });
        }
      }
      if (kind === "species") {
        for (const m of entry.Moves ?? [])
          if (!catalog.moves.some((v) => v.Name === m.Name))
            issues.push({
              kind,
              id: entry.AppId,
              name: entry.Name,
              field: `Moves:${m.Name}`,
              status: "missing_reference",
              reason: "No exact move reference; do not choose a fuzzy match",
            });
        for (const field of ["Ability1", "Ability2"])
          if (
            entry[field] &&
            !catalog.abilities.some((v) => v.Name === entry[field])
          )
            issues.push({
              kind,
              id: entry.AppId,
              name: entry.Name,
              field,
              status: "missing_reference",
              reason: `No exact ability reference: ${entry[field]}`,
            });
      }
    }
  }
  catalog.review_sources = sources;
  catalog.rules_coverage_hash = createHash("sha256")
    .update(
      (await readFile(path.join(app, "rules/coverage.json"), "utf8")).replace(
        /\r\n/g,
        "\n",
      ),
    )
    .digest("hex");
  const output = path.join(root, "outputs/rule-coverage");
  await mkdir(output, { recursive: true });
  await writeFile(
    path.join(output, "tactics-coverage.json"),
    JSON.stringify(
      {
        version: 1,
        engine: tacticsRegistry.engine,
        entries: ["moves", "abilities"].flatMap((kind) =>
          catalog[kind]
            .filter((e) => e.TacticsRules)
            .map((e) => ({
              ...e.TacticsRules,
              kind,
              id: e.AppId,
              name: e.Name,
            })),
        ),
      },
      null,
      2,
    ),
  );
  await writeFile(
    path.join(output, "special-move-coverage.json"),
    JSON.stringify(
      {
        version: 1,
        engine: specialRegistry.engine,
        entries: catalog.moves
          .filter((e) => e.SpecialRules)
          .map((e) => ({ id: e.AppId, name: e.Name, ...e.SpecialRules })),
      },
      null,
      2,
    ),
  );
  await writeFile(
    path.join(output, "ability-coverage.json"),
    JSON.stringify(
      {
        version: 1,
        engine: abilityRegistry.engine,
        entries: catalog.abilities.map((entry) => ({
          id: entry.AppId,
          name: entry.Name,
          status:
            abilityRegistry.entries[entry.Name]?.status ??
            abilityRegistry.default_status,
          scope: entry.Review.coverage,
          page: abilityRegistry.entries[entry.Name]?.page ?? null,
          dependencies: abilityRegistry.entries[entry.Name]?.dependencies ?? [],
        })),
      },
      null,
      2,
    ),
  );
  await writeFile(
    path.join(output, "catalog-review.json"),
    JSON.stringify(
      {
        version: 1,
        sources,
        total_entries: [
          "species",
          "moves",
          "abilities",
          "items",
          "natures",
        ].reduce((n, k) => n + catalog[k].length, 0),
        issues,
      },
      null,
      2,
    ),
  );
  const quote = (v) => `"${String(v ?? "").replaceAll('"', '""')}"`;
  await writeFile(
    path.join(output, "catalog-review.csv"),
    [
      "kind,id,name,field,status,reason,pdf_page",
      ...issues.map((i) =>
        [i.kind, i.id, i.name, i.field, i.status, i.reason, i.pdf_page]
          .map(quote)
          .join(","),
      ),
    ].join("\n"),
  );
}
