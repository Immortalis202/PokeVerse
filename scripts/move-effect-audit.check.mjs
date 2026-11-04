import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";

const read = (p) => JSON.parse(readFileSync(p, "utf8"));
const audit = read("rules/move-effect-audit.json");
const byName = new Map(audit.moves.map((m) => [m.name, m]));

test("MOVE-A02 preserves exact hazard sources and current executable routes", () => {
  const registry = read("rules/entry-hazard-moves.json");
  assert.deepEqual(Object.keys(registry.moves).sort(), [
    "Spikes",
    "Stealth Rock",
    "Sticky Web",
  ]);
  assert.equal(
    createHash("sha256")
      .update(
        readFileSync("rules/entry-hazard-moves.json", "utf8").replace(
          /\r\n/g,
          "\n",
        ),
      )
      .digest("hex"),
    read("rules/coverage.json").entry_hazards_registry_hash,
  );
  for (const [name, entry] of Object.entries(registry.moves)) {
    const m = byName.get(name);
    assert.equal(entry.sha256, m.sha256);
    assert.equal(entry.effect, m.effect);
    assert.equal(m.route, "executable-scope");
    assert.ok(
      m.declared_scopes.some(
        (s) => s.registry === "rules/entry-hazard-moves.json",
      ),
    );
    for (const profile of Object.values(m.probe))
      assert.equal(profile.result, "route_resolved");
  }
});

test("MOVE-A01 damaging attribute definitions retain exact source and executable scope", () => {
  const registry = read("rules/damaging-stat-moves.json");
  assert.equal(Object.keys(registry.moves).length, 66);
  assert.equal(
    createHash("sha256")
      .update(
        readFileSync("rules/damaging-stat-moves.json", "utf8").replace(
          /\r\n/g,
          "\n",
        ),
      )
      .digest("hex"),
    read("rules/coverage.json").damaging_stats_registry_hash,
  );
  for (const [name, entry] of Object.entries(registry.moves)) {
    const card = byName.get(name);
    assert.equal(entry.sha256, card.sha256, name);
    assert.equal(entry.effect, card.effect, name);
    assert.equal(card.route, "executable-scope", name);
    assert.ok(
      card.declared_scopes.some(
        (s) => s.registry === "rules/damaging-stat-moves.json",
      ),
    );
    assert.ok([0, 1, 2, 3, 4, 5].includes(entry.dice));
    assert.ok(
      entry.keys.every((k) =>
        [
          "Strength",
          "Dexterity",
          "Special",
          "Defense",
          "SpecialDefense",
        ].includes(k),
      ),
    );
    assert.ok([-2, -1, 1, 2].includes(entry.amount));
  }
});

test("move audit covers every supplied card once and preserves exact source evidence", () => {
  assert.equal(byName.size, 892);
  assert.equal(audit.moves.length, 892);
  for (const move of audit.moves) {
    const bytes = readFileSync(`../${move.source}`);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), move.sha256);
    const source = JSON.parse(bytes);
    assert.equal(source.Name, move.name);
    assert.equal(source.Effect, move.effect);
    assert.deepEqual(source.AddedEffects, move.added_effects);
    assert.deepEqual(source.Attributes, move.attributes);
    assert.deepEqual(Object.keys(move.probe).sort(), ["ailments", "baseline"]);
  }
});
test("status queue distinguishes missing clauses from existing scopes and input requirements", () => {
  for (const name of [
    "Confuse Ray",
    "Spore",
    "Thunder Wave",
    "Toxic",
    "Will-O-Wisp",
  ])
    assert.equal(byName.get(name).status_audit.status, "implemented-clause");
  assert.equal(
    byName.get("Air Slash").status_audit.status,
    "implemented-clause",
  );
  assert.equal(
    byName.get("Wild Charge").status_audit.status,
    "implemented-clause",
  );
  assert.equal(
    byName.get("Sparkling Aria").status_audit.status,
    "implemented-clause",
  );
  assert.equal(
    byName.get("Healing Wish").status_audit.status,
    "source-discrepancy",
  );
  assert.equal(byName.get("Lunar Dance").route, "input-required");
  assert.equal(byName.get("Hold Hands").route, "gm-choice");
  assert.equal(byName.get("Sappy Seed").route, "excluded");
  assert.equal(byName.get("Max Guard").route, "excluded");
  const missing = audit.moves.filter(
    (m) => m.status_audit?.status === "missing-effect",
  );
  for (const move of missing)
    for (const probe of Object.values(move.probe))
      assert.match(probe.message, /no reviewed executable definition/);
  const planned = audit.moves.filter(
    (m) =>
      m.status_audit?.batch?.startsWith("M2-") && !m.status_audit?.completed,
  );
  assert.equal(planned.length, 0);
  assert.ok(planned.every((m) => m.route !== "excluded"));
  const delivered = read("rules/direct-status-moves.json");
  assert.equal(delivered.moves.length, 22);
  assert.equal(new Set(delivered.moves.map((m) => m.name)).size, 22);
  for (const move of delivered.moves) {
    const audited = byName.get(move.name);
    assert.equal(move.source_sha256, audited.sha256);
    assert.equal(move.effect, audited.effect);
    assert.equal(audited.status_audit.completed, true);
    assert.equal(audited.status_audit.batch, "M2-A");
    assert.equal(audited.status_audit.status, "implemented-clause");
    assert.equal(audited.probe.baseline.result, "route_resolved");
    assert.ok(audited.probe.baseline.target_statuses.includes(move.kind));
  }
});
test("printed review has traceable PDF pages and protects the priority cards", () => {
  const printed = read("rules/move-status-print-review.json");
  assert.equal(
    createHash("sha256")
      .update(readFileSync(`../${printed.source}`))
      .digest("hex"),
    printed.sha256,
  );
  assert.equal(printed.checks.length, 28);
  for (const move of audit.moves.filter(
    (m) => m.status_audit?.batch === "M2-A",
  )) {
    assert.ok(move.printed_review, `Missing printed reference: ${move.name}`);
    assert.deepEqual(
      move.printed_review.printed_pages.map((p) => p - 133),
      move.printed_review.pdf_pages,
    );
  }
  assert.deepEqual(
    byName.get("Wild Charge").printed_review.printed_pages,
    [455],
  );
});

test("M2-B registry matches source prose, structured ailments and reviewed printed exceptions", () => {
  const registry = read("rules/chance-status-moves.json");
  const coverage = read("rules/coverage.json");
  assert.equal(
    createHash("sha256")
      .update(
        readFileSync("rules/chance-status-moves.json", "utf8").replace(
          /\r\n/g,
          "\n",
        ),
      )
      .digest("hex"),
    coverage.chance_status_registry_hash,
  );
  assert.equal(Object.keys(registry.moves).length, 47);
  assert.equal(
    createHash("sha256")
      .update(readFileSync(`../${registry.printed_pdf.source}`))
      .digest("hex"),
    registry.printed_pdf.sha256,
  );
  const kinds = {
    Burn: "Burn",
    Burn1: "Burn",
    Burn2: "Burn 2",
    Burn3: "Burn 3",
    Poison: "Poison",
    BadlyPoison: "Badly Poisoned",
    Paralyze: "Paralysis",
    Confuse: "Confusion",
    Flinch: "Flinch",
    Freeze: "Frozen Solid",
    Sleep: "Sleep",
  };
  let printedExceptions = 0;
  for (const [name, entry] of Object.entries(registry.moves)) {
    const bytes = readFileSync(`../${entry.source}`);
    const source = JSON.parse(bytes);
    assert.equal(
      createHash("sha256").update(bytes).digest("hex"),
      entry.sha256,
    );
    assert.equal(source.Name, name);
    assert.equal(source.Effect, entry.effect);
    assert.deepEqual(source.AddedEffects, entry.added_effects);
    assert.equal(
      entry.dice,
      Number(/Roll (\d+) Chance/i.exec(source.Effect)[1]),
      name,
    );
    const ailment = source.AddedEffects.Ailments?.[0];
    if (ailment) {
      assert.equal(entry.status, kinds[ailment.Type], name);
      if (ailment.ChanceDice != null && name !== "Fiery Wrath")
        assert.equal(entry.dice, ailment.ChanceDice, name);
    }
    if (!ailment || ailment.ChanceDice == null || name === "Fiery Wrath") {
      assert.ok(
        entry.printed_review,
        `Printed exception review missing for ${name}`,
      );
    }
    if (entry.printed_review) {
      printedExceptions++;
      assert.equal(
        entry.printed_review.printed_page - 133,
        entry.printed_review.pdf_page,
      );
    }
    const audited = byName.get(name);
    assert.equal(audited.status_audit.completed, true);
    assert.equal(audited.status_audit.batch, "M2-B");
    assert.equal(audited.probe.baseline.result, "route_resolved");
  }
  assert.equal(printedExceptions, 9);
  assert.equal(registry.moves["Fiery Wrath"].dice, 2);
  assert.equal(registry.moves["Smog"].dice, 4);
  assert.equal(registry.moves["Blazing Torque"].status, "Burn 2");
  assert.equal(registry.moves["Wicked Torque"].status, "Sleep");
  assert.equal(registry.moves["Wild Charge"].dice, 1);
});
test("audit artifacts reproduce deterministically from cards, registries and frozen route probes", () => {
  execFileSync(
    process.execPath,
    ["scripts/audit-move-effects.mjs", "--check"],
    { stdio: "pipe" },
  );
});

test("M2-C preserves all 28 source identities, independent pool order and printed choice tables", () => {
  const r = read("rules/compound-status-moves.json"),
    coverage = read("rules/coverage.json");
  assert.equal(Object.keys(r.moves).length, 28);
  assert.equal(
    createHash("sha256")
      .update(
        readFileSync("rules/compound-status-moves.json", "utf8").replace(
          /\r\n/g,
          "\n",
        ),
      )
      .digest("hex"),
    coverage.compound_status_registry_hash,
  );
  assert.equal(
    createHash("sha256")
      .update(readFileSync(`../${r.printed_source.source}`))
      .digest("hex"),
    r.printed_source.sha256,
  );
  for (const [name, d] of Object.entries(r.moves)) {
    const bytes = readFileSync(`../${d.source}`),
      source = JSON.parse(bytes);
    assert.equal(source.Name, name);
    assert.equal(source.Effect, d.effect);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), d.sha256);
    assert.equal(byName.get(name).status_audit.batch, "M2-C");
    assert.equal(byName.get(name).status_audit.completed, true);
    assert.ok(
      d.effects.every(
        (p) => Number.isInteger(p.dice) && p.dice >= 0 && p.dice <= 3,
      ),
    );
  }
  assert.deepEqual(
    r.moves["Dire Claw"].effects.map((p) => p.kind),
    ["Poison", "Paralysis", "Sleep"],
  );
  assert.deepEqual(
    r.moves["Tri Attack"].effects.map((p) => p.kind),
    ["Burn", "Frozen Solid", "Paralysis"],
  );
  assert.equal(Object.keys(r.secret_power).length, 11);
  assert.equal(r.secret_power["Lake/River/Sea"], "Strength -2");
  assert.equal(r.secret_power["Misty Terrain"], "In Love");
  assert.equal(r.printed_reviews["Secret Power table"].printed_page, 530);
  const forms = read("rules/battle-forms.json");
  assert.equal(forms.species["0648:meloetta"].stats.Special, 7);
  assert.equal(
    forms.species["0648F1:meloetta-pirouette-form"].stats.Strength,
    7,
  );
  assert.equal(forms.species["0648:meloetta-pirouette-form"], undefined);
  assert.equal(byName.get("Secret Power").route, "input-required");
});

test("recurring registry preserves all six sources and printed references", () => {
  const r = read("rules/recurring-status-moves.json");
  assert.equal(Object.keys(r.moves).length, 6);
  for (const [name, d] of Object.entries(r.moves)) {
    assert.equal(d.effect, byName.get(name).effect);
    assert.equal(d.sha256, byName.get(name).sha256);
    assert.equal(byName.get(name).status_audit.batch, "M2-D");
    assert.equal(
      r.printed_review[name].printed_page - 133,
      r.printed_review[name].pdf_page,
    );
  }
});

test("M2-E preserves four source cards and completes the audited status queue", () => {
  const r = read("rules/cure-protection-moves.json");
  assert.equal(Object.keys(r.moves).length, 4);
  for (const [name, d] of Object.entries(r.moves)) {
    assert.equal(d.effect, byName.get(name).effect);
    assert.equal(d.sha256, byName.get(name).sha256);
    assert.equal(byName.get(name).status_audit.batch, "M2-E");
    assert.equal(byName.get(name).status_audit.completed, true);
    assert.equal(
      r.printed_review[name].printed_page - 133,
      r.printed_review[name].pdf_page,
    );
  }
  assert.ok(
    !audit.moves.some(
      (m) =>
        m.status_audit?.batch?.startsWith("M2-") && !m.status_audit.completed,
    ),
  );
});
