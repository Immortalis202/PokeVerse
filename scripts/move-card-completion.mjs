import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { isDeepStrictEqual } from "node:util";

export const batches = {
  1: "Resolution inputs and GM choices",
  2: "Stat and trait operations",
  3: "Type, category and damage calculation",
  4: "History and conditional attacks",
  5: "Initiative, reactions and maneuvers",
  6: "Allied contributions and linked actions",
  7: "Delayed and triggered effects",
  8: "Battlefield and protection effects",
  9: "Held items and berries",
  10: "Copying and move selection",
  11: "Compound effects and consequences",
};

const families = [
  [1, "Judgment|Metronome|Hold Hands|Secret Power|Nature Power|Camouflage"],
  [
    2,
    "Acupressure|Mystical Power|Guard Split|Power Split|Guard Swap|Power Swap|Speed Swap|Power Trick|Power Shift|Psych Up|Heart Swap|Topsy-Turvy|Spectral Thief|Baton Pass|Snatch",
  ],
  [
    3,
    "Conversion|Conversion 2|Reflect Type|Hidden Power|Flying Press|Foul Play|Photon Geyser|Techno Blast|Tera Blast|Tera Starstorm|Chip Away|Miracle Eye|Snipe Shot|Moongeist Beam|Sunsteel Strike",
  ],
  [
    4,
    "Belch|Last Resort|Lash Out|Revenge|Temper Flare|Stomping Tantrum|Poltergeist|Fell Stinger|Ruination",
  ],
  [
    5,
    "After You|Quash|Ambush|Encore|Sucker Punch|Thunderclap|Grapple|Run Away|Evasion|Clash|Double Team|Minimize|Cover An Ally",
  ],
  [6, "Helping Hand|Help Another|Beat Up|Round"],
  [
    7,
    "Future Sight|Doom Desire|Shell Trap|Powder|Pursuit|Rage|Destiny Bond|Curse|Glaive Rush|Flame Burst",
  ],
  [8, "Court Change|Fire Pledge|Grass Pledge|Water Pledge|Mist|Lucky Chant"],
  [
    9,
    "Bestow|Covet|Thief|Trick|Switcheroo|Knock Off|Corrosive Gas|Incinerate|Embargo|Bug Bite|Pluck|Stuff Cheeks|Teatime|Fling|Natural Gift|Recycle",
  ],
  [
    10,
    "Assist|Copycat|Instruct|Me First|Mimic|Mirror Move|Sketch|Sleep Talk|Transform",
  ],
  [
    11,
    "Memento|Springtide Storm|Synchronoise|Healing Wish|Lunar Dance|Pay Day|Make It Rain",
  ],
].map(([batch, names]) => [batch, new Set(names.split("|"))]);

// Segmentation is a review aid, not an assertion that each sentence is atomic.
// Preserve exact offsets and whitespace so no source prose can disappear.
export function effectSegments(text) {
  const starts = [0];
  for (const match of text.matchAll(/[.!?]\s+(?=[A-Z*])/g)) {
    if (/(?:Sp|ie|i\.e)\.$/.test(text.slice(0, match.index + 1))) continue;
    starts.push(match.index + match[0].length);
  }
  return starts.map((start, i) => ({
    start,
    end: starts[i + 1] ?? text.length,
    text: text.slice(start, starts[i + 1] ?? text.length),
  }));
}

export function batchFor(name, text, field) {
  const named = families.find(([, names]) => names.has(name));
  if (named) return named[0];
  if (/item|berr(?:y|ies)/i.test(text)) return 9;
  if (/copy|copied|replace.*move/i.test(text)) return 10;
  if (/storyteller|discretion|environment/i.test(text)) return 1;
  if (/initiative|evad|evasion|clash|reaction|grappl/i.test(text)) return 5;
  if (/next round|end of the round|ongoing|next time/i.test(text)) return 7;
  if (/terrain|weather|entry hazard|force field|shield|battlefield/i.test(text))
    return 8;
  if (/increase|reduce|attribute|trait/i.test(text)) return 2;
  if (
    /previous|last move|this round|already|successfully performed/i.test(text)
  )
    return 4;
  if (
    /type|damage|defen[cs]e|power|accuracy|target|category/i.test(
      `${field} ${text}`,
    )
  )
    return 3;
  return 11;
}

export function buildLedger(audit, previous, sourceRoot) {
  const prior = new Map(previous?.moves?.map((m) => [m.source, m]) ?? []);
  const moves = audit.moves.map((move) => {
    const bytes = readFileSync(resolve(sourceRoot, move.source));
    const hash = createHash("sha256").update(bytes).digest("hex");
    if (hash !== move.sha256) throw new Error(`Source drift: ${move.source}`);
    const source = JSON.parse(bytes);
    const old = prior.get(move.source);
    const sameSource = old?.sha256 === hash;
    const excluded = move.route === "excluded";
    if (excluded && !move.exclusion)
      throw new Error(`Missing exclusion reason: ${move.name}`);
    const definitions = effectSegments(source.Effect ?? "").map((part) => ({
      id: `effect:${part.start}-${part.end}`,
      field: "Effect",
      value: part.text,
      start: part.start,
      end: part.end,
    }));
    for (const field of [
      "Type",
      "Category",
      "Power",
      "Damage1",
      "Damage2",
      "Accuracy1",
      "Accuracy2",
      "Target",
      "Attributes",
      "AddedEffects",
    ]) {
      if (Object.hasOwn(source, field))
        definitions.push({
          id: `metadata:${field}`,
          field,
          value: source[field],
        });
    }
    const clauses = definitions.map((definition) => {
      const saved =
        sameSource &&
        old.clauses.find(
          (c) =>
            c.id === definition.id &&
            isDeepStrictEqual(c.value, definition.value),
        );
      return {
        ...definition,
        status: excluded ? "excluded" : (saved?.status ?? "pending"),
        batch: excluded
          ? null
          : (saved?.batch ??
            batchFor(
              move.name,
              typeof definition.value === "string"
                ? definition.value
                : JSON.stringify(definition.value),
              definition.field,
            )),
        evidence: saved?.evidence ?? [],
        ...(excluded
          ? { exclusion: move.exclusion }
          : saved?.exclusion
            ? { exclusion: saved.exclusion }
            : {}),
        ...(saved?.interpretation
          ? { interpretation: saved.interpretation }
          : {}),
      };
    });
    return {
      id: move.id,
      name: move.name,
      source: move.source,
      sha256: hash,
      semantic_review_complete:
        sameSource && old.semantic_review_complete === true,
      route_diagnostic: move.route,
      exclusion: move.exclusion,
      excluded_clauses: move.excluded_clauses,
      existing_scopes_to_verify: move.declared_scopes,
      printed_review: move.printed_review,
      catalog_differences: move.catalog_differences,
      clauses,
    };
  });
  return {
    version: 2,
    state: moves.every(
      (m) =>
        m.exclusion ||
        (m.semantic_review_complete &&
          m.clauses.every((c) => c.status !== "pending")),
    )
      ? "complete"
      : "semantic-review-in-progress",
    note: "Sentence segments and metadata are review units, not semantic certification. Compound sentences must be reviewed in full. Source fingerprints, assignments and existing scopes do not prove implementation. Every in-scope card requires explicit semantic review and executable evidence before closure.",
    batches,
    moves,
  };
}

if (
  process.argv[1] &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
  const path = resolve(root, "rules/move-card-completion.json");
  const audit = JSON.parse(
    readFileSync(resolve(root, "rules/move-effect-audit.json"), "utf8"),
  );
  const previous = JSON.parse(readFileSync(path, "utf8"));
  const result = buildLedger(audit, previous, resolve(root, ".."));
  if (process.argv.includes("--check")) {
    if (!isDeepStrictEqual(previous, result))
      throw new Error("Move ledger is stale; run npm run moves:inventory");
  } else writeFileSync(path, `${JSON.stringify(result, null, 2)}\n`);
  console.log(
    `${result.moves.length} cards; ${result.moves.reduce((n, m) => n + m.clauses.length, 0)} review units; ${result.state}`,
  );
}
