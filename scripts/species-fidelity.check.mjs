import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { DatabaseSync } from "node:sqlite";
const read = (p) => JSON.parse(readFileSync(p, "utf8"));
const review = read("rules/species-fidelity-review.json");

test("M4-S01 evolution audit inventories every JSON identity and separates all form changes", () => {
  const report = read("rules/species-evolution-json-review.json");
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    assert.equal(
      report.catalog_hash,
      JSON.parse(
        db.prepare("select value from metadata where key='hash'").get().value,
      ),
    );
    assert.equal(report.species.length, 1200);
    assert.equal(report.ordinary_branches.length, 522);
    assert.equal(report.form_change_branches.length, 112);
    assert.equal(report.pending_cost_decisions.length, 11);
    for (const key of [
      "unexpected_runtime_differences",
      "structural_issues",
      "ordinary_cycles",
      "prerequisite_issues",
    ])
      assert.deepEqual(report[key], []);
    assert.equal(report.retained_guidance_discrepancies.length, 1);
    const entries = new Map(
      db
        .prepare("select id,data from entries where kind='species'")
        .all()
        .map((r) => [r.id, JSON.parse(r.data)]),
    );
    for (const s of report.species) {
      assert.equal(
        createHash("sha256")
          .update(readFileSync(`../${s.source}`))
          .digest("hex"),
        s.sha256,
      );
      assert.equal(
        entries.get(s.id).Evolutions.length,
        s.runtime_relation_count,
      );
    }
    for (const branch of [
      ...report.ordinary_branches,
      ...report.form_change_branches,
    ]) {
      const s = entries.get(branch.source_id),
        t = entries.get(branch.target_id);
      const relation = s.Evolutions.find((r) => r.To === t.Name);
      assert.deepEqual(
        Object.fromEntries(
          Object.entries(relation).filter(([k]) => k !== "To"),
        ),
        branch.conditions,
      );
      const ordinary =
        !["Mega", "Form"].includes(relation.Kind) && s.Number !== t.Number;
      assert.equal(ordinary, Object.hasOwn(branch, "tp_cost"));
    }
    for (const fix of review.json_evolution_normalizations) {
      const s = entries.get(fix.source_id),
        t = entries.get(fix.target_id);
      assert.deepEqual(
        s.Evolutions.find((r) => r.To === t.Name),
        { To: t.Name, ...fix.resolved_conditions },
      );
      assert.deepEqual(
        t.Evolutions.find((r) => r.From === s.Name),
        { From: s.Name, ...fix.resolved_conditions },
      );
      assert.equal(s.Review.fields.Evolutions.status, "unverified");
      assert.ok(
        s.FidelityReview.some(
          (r) => r.source === "supplied-json-normalization",
        ),
      );
    }
  } finally {
    db.close();
  }
});

test("M4-S01 keeps all 34 exact forms in the 14 repaired families on their own JSON stats", () => {
  const scope = review.json_form_stat_review;
  assert.equal(scope.family_numbers.length, 14);
  assert.equal(scope.entries.length, 34);
  const corrections = read("rules/catalog-corrections.json");
  const forms = read("rules/battle-forms.json");
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    const entries = db
      .prepare("select data from entries where kind='species'")
      .all()
      .map((r) => JSON.parse(r.data));
    assert.deepEqual(
      entries
        .filter((s) => scope.family_numbers.includes(s.Number))
        .map((s) => s.AppId)
        .sort(),
      scope.entries.map((s) => s.id).sort(),
    );
    for (const form of scope.entries) {
      const bytes = readFileSync(`../${form.source}`);
      assert.equal(
        createHash("sha256").update(bytes).digest("hex"),
        form.sha256,
      );
      const source = JSON.parse(bytes);
      const entry = entries.find((s) => s.AppId === form.id);
      assert.equal(entry.Name, source.Name);
      assert.equal(entry.Creation.stat_source, form.source);
      for (const field of Object.keys(form.stats)) {
        assert.equal(entry[field], source[field], `${form.id} ${field}`);
        assert.equal(form.stats[field], source[field]);
        assert.ok(
          !Object.hasOwn(corrections.species[form.id]?.fields ?? {}, field),
        );
        assert.equal(entry.Review.fields[field].status, "unverified");
        if (field === "BaseHP") {
          if (forms.species[form.id])
            assert.equal(forms.species[form.id].base_hp, source[field]);
          continue;
        }
        const max = field.startsWith("Max");
        const a = max ? field.slice(3) : field;
        assert.equal(
          entry.Creation[max ? "limits" : "baselines"][a],
          source[field],
        );
        const row = db
          .prepare(
            "select baseline, species_limit from species_stats where species_id=? and attribute=?",
          )
          .get(form.id, a);
        assert.equal(row[max ? "species_limit" : "baseline"], source[field]);
        if (forms.species[form.id])
          assert.equal(
            forms.species[form.id][max ? "limits" : "stats"][a],
            source[field],
          );
      }
    }
    assert.equal(scope.entries.filter((s) => s.number === 681).length, 2);
    assert.equal(scope.entries.filter((s) => s.number === 898).length, 3);
    assert.equal(scope.entries.filter((s) => s.number === 386).length, 4);
    assert.equal(scope.entries.filter((s) => s.number === 555).length, 4);
  } finally {
    db.close();
  }
});

test("M4-S01 applies all 23 explicit stat rulings without printed certification", () => {
  const expected = {
    Binacle: { MaxDexterity: 4 },
    "Brute Bonnet": { MaxStrength: 7 },
    Cranidos: { MaxDexterity: 4 },
    Dusknoir: { MaxVitality: 7, MaxInsight: 7 },
    Entei: { Insight: 5, MaxInsight: 5 },
    Flareon: { MaxSpecial: 6 },
    Florges: { Insight: 4, MaxInsight: 8 },
    Gumshoos: { BaseHP: 5 },
    Hydrapple: { Strength: 2 },
    Kricketune: { MaxDexterity: 4 },
    Lampent: { MaxDexterity: 4 },
    Liepard: { Dexterity: 3 },
    Lillipup: { MaxDexterity: 4 },
    Lopunny: { MaxDexterity: 6 },
    Marill: { Insight: 2, MaxInsight: 4 },
    Stoutland: { MaxDexterity: 5 },
    Swoobat: { MaxDexterity: 6 },
    Togekiss: { MaxDexterity: 5 },
    Trubbish: { MaxDexterity: 4 },
  };
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    assert.equal(review.user_stat_rulings.length, 19);
    for (const [name, fields] of Object.entries(expected)) {
      const entry = JSON.parse(
        db
          .prepare("select data from entries where kind='species' and name=?")
          .get(name).data,
      );
      for (const [field, value] of Object.entries(fields)) {
        assert.equal(entry[field], value, `${name} ${field}`);
        assert.equal(entry.StatRuling.fields[field].resolved, value);
        assert.equal(entry.Review.fields[field].source, "user-ruling");
        assert.equal(entry.Review.fields[field].status, "unverified");
        assert.equal(entry.Review.fields[field].pdf_page, null);
        if (field !== "BaseHP") {
          const maximum = field.startsWith("Max");
          const attribute = maximum ? field.slice(3) : field;
          assert.equal(
            entry.Creation[maximum ? "limits" : "baselines"][attribute],
            value,
          );
          const row = db
            .prepare(
              "select baseline, species_limit from species_stats where species_id=? and attribute=?",
            )
            .get(entry.AppId, attribute);
          assert.equal(row[maximum ? "species_limit" : "baseline"], value);
        }
      }
      const ruling = review.user_stat_rulings.find((r) => r.name === name);
      const source = read(`../Pokerole-Data/v3.0/Pokedex/${name}.json`);
      for (const [field, values] of Object.entries(ruling.fields))
        assert.equal(source[field], values.source_json);
    }
  } finally {
    db.close();
  }
});

test("M4-S01 Kanto continuation repairs Metapod's limit and preserves the explicit Mega Protect ruling", () => {
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    const get = (name) =>
      JSON.parse(
        db
          .prepare("select data from entries where kind='species' and name=?")
          .get(name).data,
      );
    assert.equal(get("Metapod").MaxVitality, 4);
    assert.equal(
      read("../Pokerole-Data/v3.0/Pokedex/Metapod.json").MaxVitality,
      4,
    );
    const mega = get("Blastoise (Mega Form)");
    assert.equal(
      mega.Moves.find((m) => m.Name === "Protect").Learned,
      "Standard",
    );
    assert.ok(
      mega.FidelityReview.some(
        (r) =>
          r.source === "user-ruling" && r.decision.includes("printed Advanced"),
      ),
    );
    assert.equal(
      get("Blastoise").Moves.find((m) => m.Name === "Protect").Learned,
      "Standard",
    );
    assert.equal(mega.Creation.stage, null);
    assert.equal(mega.Review.fields.Evolutions.status, "unverified");
    for (const name of [
      "Caterpie",
      "Metapod",
      "Butterfree",
      "Weedle",
      "Kakuna",
      "Pidgey",
      "Pidgeotto",
    ])
      assert.equal(get(name).Review.fields.Evolutions.status, "reviewed");
  } finally {
    db.close();
  }
});

test("M4-S01 targeted printed evolution corrections preserve identities and partial review", () => {
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    const get = (id) =>
      JSON.parse(
        db
          .prepare("select data from entries where kind='species' and id=?")
          .get(id).data,
      );
    for (const fix of review.source_evolution_corrections) {
      const source = get(fix.source_id),
        target = get(fix.target_id);
      assert.deepEqual(
        source.Evolutions.find((r) => r.To === target.Name),
        { To: target.Name, ...fix.resolved_conditions },
      );
      assert.deepEqual(
        target.Evolutions.find((r) => r.From === source.Name),
        { From: source.Name, ...fix.resolved_conditions },
      );
      for (const entry of [source, target]) {
        assert.equal(entry.Review.fields.Evolutions.status, "unverified");
        assert.ok(
          entry.FidelityReview.some(
            (r) => r.source === "printed source" && r.decision === fix.basis,
          ),
        );
      }
      for (const ref of fix.references)
        assert.equal(ref.pdf_page + 133, ref.printed_page);
    }
    assert.deepEqual(
      get("0744:rockruff").Evolutions.map((r) => r.Special),
      ["Day", "Dusk", "Night"],
    );
    assert.equal(get("0772:type-null").Creation.stage, 1);
    assert.equal(get("0884:duraludon").Creation.stage, 1);
    const standalone = get("0901F1:ursaluna-kitakami-form");
    assert.equal(standalone.Creation.stage, 3);
    assert.deepEqual(standalone.Evolutions, []);
    assert.ok(
      standalone.FidelityReview.some(
        (r) => r.source === "user-ruling" && r.decision.includes("standalone"),
      ),
    );
    const ursaring = get("0217:ursaring");
    assert.ok(!ursaring.Evolutions.some((r) => r.To === standalone.Name));
    assert.ok(ursaring.Evolutions.some((r) => r.To === "Ursaluna"));
  } finally {
    db.close();
  }
});

test("M4-S01 preserves exact source identities and bounds manual certification", () => {
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    assert.equal(review.status, "in-progress");
    for (const source of Object.values(review.sources))
      assert.equal(
        createHash("sha256")
          .update(readFileSync(`../${source.path}`))
          .digest("hex"),
        source.sha256,
      );
    for (const source of review.manual_reviews) {
      const bytes = readFileSync(`../${source.source}`);
      assert.equal(
        createHash("sha256").update(bytes).digest("hex"),
        source.sha256,
      );
      assert.equal(source.printed_page - 133, source.pdf_page);
      const e = JSON.parse(
        db
          .prepare("select data from entries where kind='species' and id=?")
          .get(source.id).data,
      );
      assert.equal(e.Name, source.name);
      for (const field of source.fields) {
        assert.equal(e.Review.fields[field].status, "reviewed");
        assert.equal(e.Review.fields[field].pdf_page, source.pdf_page);
      }
    }
    const e = JSON.parse(
      db
        .prepare(
          "select data from entries where kind='species' and name='Charizard'",
        )
        .get().data,
    );
    assert.equal(e.Review.fields.Moves.status, "reviewed");
    assert.equal(e.Review.fields.Evolutions.status, "unverified");
    const mega = JSON.parse(
      db
        .prepare(
          "select data from entries where kind='species' and name='Charizard (Mega X Form)'",
        )
        .get().data,
    );
    assert.equal(mega.Creation.stage, null);
    assert.equal(mega.Review.fields.Moves.status, "reviewed");
  } finally {
    db.close();
  }
});

test("M4-S01 user stage rulings remain distinct from printed verification", () => {
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    for (const name of [
      "Articuno",
      "Rotom",
      "Deoxys",
      "Groudon (Primal Form)",
      "Necrozma (Ultra Burst Form)",
      "Terapagos (Stellar Form)",
      "Aegislash (Blade Form)",
    ]) {
      const e = JSON.parse(
        db
          .prepare("select data from entries where kind='species' and name=?")
          .get(name).data,
      );
      assert.equal(e.Creation.stage, 3, name);
      assert.equal(e.Creation.stage_ruling.source, "user-ruling", name);
      assert.equal(e.Review.fields.Evolutions.status, "unverified", name);
      assert.match(e.Creation.review_note, /table ruling/);
    }
    const egg = JSON.parse(
      db
        .prepare(
          "select data from entries where kind='species' and id='0000:egg'",
        )
        .get().data,
    );
    assert.equal(egg.Creation.stage_ruling, undefined);
  } finally {
    db.close();
  }
});

test("M4-S01 repairs confirmed identity errors but retains disputed conditions", () => {
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    const get = (id) =>
      JSON.parse(
        db
          .prepare("select data from entries where kind='species' and id=?")
          .get(id).data,
      );
    const floette = get("0670:floette");
    assert.equal(floette.Evolutions[0].From, "Flab\u00e9b\u00e9");
    const pawmot = get("0923:pawmot");
    assert.equal(pawmot.Evolutions[0].From, "Pawmo");
    assert.equal(pawmot.Evolutions[0].Speed, "Slow");
    assert.equal(
      read("../Pokerole-Data/v3.0/Pokedex/Pawmot.json").Evolutions[0].From,
      "Pawmot",
    );
    const audit = JSON.parse(
      db
        .prepare("select value from metadata where key='species_fidelity'")
        .get().value,
    );
    assert.equal(audit.species_total, 1200);
    assert.equal(audit.unresolved_evolution_issues, 0);
    assert.equal(audit.evolution_issues.length, 1);
    assert.equal(audit.evolution_issues[0].source_name, "Scyther");
    assert.equal(audit.evolution_issues[0].review.status, "retained-guidance");
    assert.ok(
      audit.evolution_issues.every((e) =>
        [
          "directional-condition-conflict",
          "missing-reciprocal-relation",
        ].includes(e.issue),
      ),
    );
    assert.ok(!audit.evolution_issues.some((e) => e.source_name === "Dipplin"));
    assert.ok(!audit.evolution_issues.some((e) => e.source_name === "Aipom"));
    assert.equal(audit.unresolved_forward_rules, "preserve");
    assert.deepEqual(
      audit.special_evolution_requirements,
      review.user_policy.special_evolution_requirements,
    );
    assert.deepEqual(audit.special_evolution_requirements.enforced, [
      "gender",
      "known-move",
    ]);
    assert.equal(
      audit.special_evolution_requirements.consume_evolution_items,
      false,
    );
    assert.equal(
      audit.special_evolution_requirements.completion_requires_gm,
      true,
    );
  } finally {
    db.close();
  }
});

test("M4-S01 applies only the named evolution rulings and preserves unresolved forward rules", () => {
  const db = new DatabaseSync("src-tauri/resources/catalog.db", {
    readOnly: true,
  });
  try {
    const get = (id) =>
      JSON.parse(
        db
          .prepare("select data from entries where kind='species' and id=?")
          .get(id).data,
      );
    for (const ruling of review.user_evolution_rulings) {
      const source = get(ruling.source_id),
        target = get(ruling.target_id);
      assert.deepEqual(
        source.Evolutions.find((r) => r.To === target.Name),
        { To: target.Name, ...ruling.resolved_conditions },
      );
      assert.deepEqual(
        target.Evolutions.find((r) => r.From === source.Name),
        { From: source.Name, ...ruling.resolved_conditions },
      );
      assert.equal(source.Review.fields.Evolutions.status, "unverified");
      assert.ok(source.FidelityReview.some((r) => r.source === "user-ruling"));
    }
    assert.equal(get("1011:dipplin").Creation.stage, 2);
    const scyther = get("0123:scyther");
    assert.equal(
      scyther.Evolutions.find((r) => r.To === "Kleavor").Kind,
      "Trade",
    );
    const rayquaza = get("0384:rayquaza");
    assert.equal(
      rayquaza.Evolutions.find((r) => r.To.includes("Mega")).Kind,
      "Mega",
    );
  } finally {
    db.close();
  }
});
