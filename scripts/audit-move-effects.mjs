import { createHash } from "node:crypto";
import { readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import { isDeepStrictEqual } from "node:util";

const app = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const source = path.resolve(app, "../Pokerole-Data/v3.0/Moves");
const read = async (name) =>
  JSON.parse(await readFile(path.join(app, name), "utf8"));
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const directStatus = await read("rules/direct-status-moves.json");
const deliveredDirect = new Set(directStatus.moves.map((m) => m.name));
const chanceStatus = await read("rules/chance-status-moves.json");
const deliveredChance = new Set(Object.keys(chanceStatus.moves));
const cureProtection = await read("rules/cure-protection-moves.json");
const deliveredCureProtection = new Set(Object.keys(cureProtection.moves));
const recurringStatus = await read("rules/recurring-status-moves.json");
const deliveredRecurring = new Set(Object.keys(recurringStatus.moves));
const compoundStatus = await read("rules/compound-status-moves.json");
const deliveredCompound = new Set(Object.keys(compoundStatus.moves));
const probes = await read("rules/move-effect-probes.json");
const damagingStats = await read("rules/damaging-stat-moves.json");
const interaction = await read("rules/battle-interactions.json");
const special = await read("rules/special-moves.json");
const tactics = await read("rules/tactics-reviewed.json");
const identity = await read("rules/identity-moves.json");
const will = await read("rules/will-moves-review.json");
const printed = await read("rules/move-status-print-review.json");
const version = (await read("package.json")).version;
assert.equal(
  probes.release,
  version,
  "Regenerate runtime probes for this release",
);
const byName = new Map(probes.moves.map((p) => [p.name, p]));
assert.equal(byName.size, 892);
const missing = (p) =>
  p.result === "route_error" &&
  p.code === "manual_rule" &&
  p.message.includes("no reviewed executable definition");
const statuses =
  /\b(burn\w*|poison\w*|paraly\w*|sleep|asleep|freez\w*|frozen|confus\w*|flinch\w*|status|ailment\w*)\b|in love/i;
const direct = new Set([
  "Alluring Voice",
  "Attract",
  "Confuse Ray",
  "Dark Void",
  "Dynamic Punch",
  "Glare",
  "Grass Whistle",
  "Hypnosis",
  "Inferno",
  "Nuzzle",
  "Poison Gas",
  "Poison Powder",
  "Sing",
  "Sleep Powder",
  "Spore",
  "Stun Spore",
  "Supersonic",
  "Sweet Kiss",
  "Teeter Dance",
  "Thunder Wave",
  "Toxic",
  "Will-O-Wisp",
  "Zap Cannon",
]);
const recurring = new Set([
  "Yawn",
  "Nightmare",
  "Malignant Chain",
  "Perish Song",
  "Sleep Talk",
  "Snore",
]);
const compound = new Set([
  "Axe Kick",
  "Burning Jealousy",
  "Buzzy Buzz",
  "Dire Claw",
  "Fake Out",
  "Fire Fang",
  "Flatter",
  "Floaty Fall",
  "Glaciate",
  "Hydro Steam",
  "Hypnosis",
  "Ice Fang",
  "Infernal Parade",
  "Lovely Kiss",
  "Mortal Spin",
  "Relic Song",
  "Secret Power",
  "Shell Side Arm",
  "Sizzly Slide",
  "Splishy Splash",
  "Stomp",
  "Swagger",
  "Thunder Fang",
  "Toxic Thread",
  "Tri Attack",
  "Twister",
  "Upper Hand",
  "Venom Drench",
]);
const custom = {
  Uproar: {
    status: "missing-effect",
    batch: "M2-E",
    reason:
      "Printed battlefield-wide Sleep cure and one-round lifecycle have no executable move effect; random-foe targeting is a separate requirement.",
    evidence: ["src-tauri/src/combat.rs:effects_inner_base"],
  },
  "Wild Charge": {
    status: "missing-clause",
    batch: "M2-B",
    reason:
      "Recoil route resolves, but special::chance has no Wild Charge entry: the printed one-die paralysis clause is not applied.",
    evidence: [
      "src-tauri/src/combat/special.rs:effects",
      "src-tauri/src/combat/special.rs:chance",
    ],
  },
  "Sparkling Aria": {
    status: "partial-clause",
    batch: "M2-E",
    reason:
      "Burned-target cure and Support conversion exist; the unburned-target effect phase falls through to manual_rule. Repair the normal damage/no-cure branch; do not reimplement the existing cure.",
    evidence: [
      "src-tauri/src/combat/special.rs:effects",
      "src-tauri/src/combat/special.rs:move_data",
    ],
  },
  "Toxic Spikes": {
    status: "missing-move-entry",
    batch: "M2-E",
    reason:
      "Poison hazard entry resolution exists for Toxic Debris/GM field state, but the Toxic Spikes move has no install branch in field::move_effect.",
    evidence: [
      "src-tauri/src/combat/field.rs:move_effect",
      "src-tauri/src/combat/field.rs:toxic_debris",
    ],
  },
  Safeguard: {
    status: "missing-effect",
    batch: "M2-E",
    reason:
      "Safeguard is named only by Screen Cleaner cleanup. Neither its move installation nor its condition-immunity predicate is implemented; the effect resolver falls through.",
    evidence: [
      "src-tauri/src/combat/field.rs:move_effect",
      "src-tauri/src/combat/field.rs:screen_cleaner",
    ],
  },
  "Healing Wish": {
    status: "source-discrepancy",
    batch: "source-correction",
    reason:
      "Cure/sacrifice and chosen recipient are implemented. JSON Heal.WillPointCost says one Will, while printed MASTER 512 has no Will charge. Preserve current no-cost behavior; owned metadata correction is separate from new mechanics.",
    evidence: [
      "src-tauri/src/combat/special.rs:effects",
      "rules/will-moves-review.json",
    ],
  },
};
const supportedStatus = new Set([
  ...Object.keys(interaction.chance_moves),
  "Thunder Shock",
  "Thunderbolt",
  "Thunder Punch",
  "Spark",
  "Confusion",
  "Ice Beam",
  "Ice Punch",
  "Blizzard",
  "Powder Snow",
  "Thunder",
  "Hurricane",
  "Hyper Fang",
  "Flare Blitz",
  "Volt Tackle",
  "Matcha Gotcha",
  "Bounce",
  "Freeze Shock",
  "Ice Burn",
  "Sky Attack",
  "Double Iron Bash",
  "Twineedle",
  "Barb Barrage",
  "Bitter Malice",
  "Baneful Bunker",
  "Burning Bulwark",
  "Beak Blast",
  "Sky Drop",
  "Focus Punch",
  "Rest",
  "Aromatherapy",
  "Electric Terrain",
  "Heal Bell",
  "Jungle Healing",
  "Lunar Blessing",
  "Lunar Dance",
  "Misty Terrain",
  "Psycho Shift",
  "Purify",
  "Refresh",
  "Smelling Salts",
  "Sparkly Swirl",
  "Take Heart",
  "Wake-Up Slap",
]);
const nonProducers = new Set([
  "Dive",
  "Synchronoise",
  "Dream Eater",
  "Facade",
  "Hex",
  "Venoshock",
]);
const registryEvidence = (name) =>
  [
    damagingStats.moves[name] && {
      registry: "rules/damaging-stat-moves.json",
      status: "automated",
      scope:
        "MOVE-A01: exact damaging attribute clause; single Chance pool or guaranteed change, cast-level user changes; other clauses remain separate",
    },
    cureProtection.moves[name] && {
      registry: "rules/cure-protection-moves.json",
      status: "automated",
      scope: cureProtection.decisions[name],
    },
    recurringStatus.moves[name] && {
      registry: "rules/recurring-status-moves.json",
      status: "automated",
      scope: recurringStatus.decisions[name],
    },
    compoundStatus.moves[name] && {
      registry: "rules/compound-status-moves.json",
      status: "automated",
      scope:
        "Reviewed compound battle clauses, independent pools and explicit source choices; permanent Beauty changes and unrelated Minimize clauses remain outside scope",
    },
    chanceStatus.moves[name] && {
      registry: "rules/chance-status-moves.json",
      status: "automated",
      scope: `${chanceStatus.moves[name].dice} Chance dice for ${chanceStatus.moves[name].status}; new encounter gate; other card clauses remain separate`,
    },
    special.entries[name] && {
      registry: "rules/special-moves.json",
      ...special.entries[name],
    },
    tactics.moves[name] && {
      registry: "rules/tactics-reviewed.json",
      ...tactics.moves[name],
    },
    interaction.moves[name] && {
      registry: "rules/battle-interactions.json",
      scope: "Reviewed stat/trait changes",
      ...interaction.moves[name],
    },
    interaction.chance_moves[name] && {
      registry: "rules/battle-interactions.json",
      scope: "Reviewed chance status",
      ...interaction.chance_moves[name],
    },
    identity.entries[name] && {
      registry: "rules/identity-moves.json",
      ...identity.entries[name],
    },
    will.moves.find((m) => m.name === name) && {
      registry: "rules/will-moves-review.json",
      scope: will.moves.find((m) => m.name === name).will_scope,
    },
  ]
    .filter(Boolean)
    .map((e) => ({
      registry: e.registry,
      status: e.status ?? "scoped",
      scope: e.scope,
      page: e.page,
      dependencies: e.dependencies ?? [],
    }));

const files = (await readdir(source)).filter((f) => f.endsWith(".json")).sort();
const rows = [];
for (const filename of files) {
  const bytes = await readFile(path.join(source, filename));
  const card = JSON.parse(bytes);
  const name = card.Name;
  const probe = byName.get(name);
  assert.ok(probe, `Missing probe for ${name}`);
  const scopes = registryEvidence(name);
  const allMissing = Object.values(probe.scenarios).every(missing);
  const anyResolved = Object.values(probe.scenarios).some(
    (p) => p.result === "route_resolved",
  );
  const excluded =
    name === "Sappy Seed"
      ? "User excluded Sappy Seed on 2026-10-28"
      : card.Attributes?.MaxMove ||
          card.Attributes?.ZMove ||
          name === "Max Guard"
        ? "Max/Dynamax/Z moves excluded"
        : null;
  const gm =
    name === "Hold Hands"
      ? "The storyteller decides battle ending; resisted Will is narrative, not a drain."
      : null;
  const route = excluded
    ? "excluded"
    : gm
      ? "gm-choice"
      : allMissing
        ? "manual-effect-route"
        : anyResolved
          ? "executable-scope"
          : "input-required";
  let statusAudit = null;
  if (
    (statuses.test(card.Effect) && !nonProducers.has(name)) ||
    recurring.has(name)
  ) {
    statusAudit = excluded
      ? { status: "excluded", reason: excluded }
      : deliveredDirect.has(name)
        ? {
            status: "implemented-clause",
            batch: "M2-A",
            completed: true,
            reason:
              "Guaranteed printed condition wired through direct_status; see rules/direct-status-moves.json and direct_status_tests.",
          }
        : deliveredCureProtection.has(name)
          ? {
              status: "implemented-clause",
              batch: "M2-E",
              completed: true,
              reason:
                "Reviewed cure/protection and hazard entry path; see rules/cure-protection-moves.json.",
            }
          : deliveredRecurring.has(name)
            ? {
                status: "implemented-clause",
                batch: "M2-D",
                completed: true,
                reason:
                  "Reviewed recurring clocks and host sleep actions; see rules/recurring-status-moves.json for retained choices.",
              }
            : deliveredCompound.has(name)
              ? {
                  status: "implemented-clause",
                  batch: "M2-C",
                  completed: true,
                  reason:
                    "Reviewed compound battle clause; source choices and retained boundaries in rules/compound-status-moves.json.",
                }
              : deliveredChance.has(name)
                ? {
                    status: "implemented-clause",
                    batch: "M2-B",
                    completed: true,
                    reason:
                      "Reviewed Chance status clause; see rules/chance-status-moves.json and chance_status_tests. Other clauses remain separately scoped.",
                  }
                : (custom[name] ??
                  (supportedStatus.has(name)
                    ? {
                        status: "implemented-clause",
                        reason:
                          "Named chance/cure/reaction handler exists; this is scope evidence, not certification of every other card clause.",
                      }
                    : allMissing
                      ? {
                          status: "missing-effect",
                          batch: recurring.has(name)
                            ? "M2-D"
                            : compound.has(name)
                              ? "M2-C"
                              : direct.has(name)
                                ? "M2-A"
                                : /Chance/i.test(card.Effect)
                                  ? "M2-B"
                                  : "M2-C",
                          reason:
                            "Both unprotected effect probes reach the explicit manual_rule fallback; no executable named definition was found for this card's status clause.",
                        }
                      : {
                          status: "review-required",
                          reason:
                            "A route exists but the status clause has no reviewed named handler in this audit; inspect before claiming automation.",
                        }));
  }
  rows.push({
    name,
    id: card._id,
    source: `Pokerole-Data/v3.0/Moves/${filename}`,
    printed_review: printed.checks.find((p) => p.name === name) ?? null,
    sha256: sha(bytes),
    category: card.Category,
    target: card.Target,
    effect: card.Effect,
    attributes: card.Attributes,
    added_effects: card.AddedEffects,
    catalog_differences: Object.fromEntries(
      [
        ["effect", card.Effect ?? null, probe.catalog_effect],
        ["target", card.Target ?? null, probe.catalog_target],
        ["attributes", card.Attributes ?? null, probe.catalog_attributes],
      ]
        .filter(
          ([, original, effective]) => !isDeepStrictEqual(original, effective),
        )
        .map(([field, , effective]) => [field, effective]),
    ),
    route,
    exclusion: excluded,
    gm_choice: gm,
    excluded_clauses: [
      ...(/\bLethal\b/i.test(card.Effect)
        ? [
            "Optional lethal wounds are excluded; ordinary damage remains separate",
          ]
        : []),
      ...(/\bDynamax\b/i.test(card.Effect)
        ? [
            "Dynamax-specific bonus is excluded; ordinary move scope remains separate",
          ]
        : []),
      ...(/out of battle/i.test(card.Effect)
        ? ["Outside-battle narrative clauses are outside this battle audit"]
        : []),
    ],
    declared_scopes: scopes,
    status_audit: statusAudit,
    probe: probe.scenarios,
  });
}
assert.equal(rows.length, 892);
const counts = (values) =>
  Object.fromEntries(
    [...new Set(values)]
      .sort()
      .map((v) => [v, values.filter((x) => x === v).length]),
  );
const output = {
  version: 1,
  date: "2026-11-01",
  release: version,
  engine: probes.engine,
  scope:
    "892-card effect-route inventory; focused semantic review of status infliction, cure and protection clauses. M2-A through M2-E battle status clauses and MOVE-A01 damaging attribute clauses delivered; see scoped registries and retained choices.",
  limits: [
    "An executable route is not full card coverage.",
    "Probes bypass accuracy, damage, timing and public command permissions; no native or network acceptance is implied.",
    "Synthetic baseline/ailment profiles have no selections, previous hits, weather or evolution prerequisites; legitimate input errors are not missing mechanics.",
    "Source JSON prose and metadata are preserved verbatim; printed PDF discrepancies require explicit review, not inferred video-game rules.",
    "Manual-effect-route counts are dispatcher outcomes, not a count of distinct missing mechanics. Ordinary damage/attributes can already work.",
  ],
  printed_source: {
    source: printed.source,
    sha256: printed.sha256,
    reviewed_cards: printed.checks.length,
  },
  counts: {
    routes: counts(rows.map((m) => m.route)),
    status_clauses: counts(
      rows.filter((m) => m.status_audit).map((m) => m.status_audit.status),
    ),
  },
  moves: rows,
};
const json = JSON.stringify(output, null, 2) + "\n";
const target = path.join(app, "rules/move-effect-audit.json");
if (process.argv.includes("--check"))
  assert.equal(
    await readFile(target, "utf8"),
    json,
    "Move audit is stale; regenerate probes and inventory",
  );
else await writeFile(target, json);
console.log(JSON.stringify(output.counts));
