import { isDeepStrictEqual } from "node:util";

const baseNumber = (e) => e.DexID.match(/^\d+/)?.[0];
const excluded = (e) =>
  /\(Mega\b|Gigantamax/i.test(e.Name) ||
  e.Evolutions?.some((r) => r.From && r.Kind === "Mega");
const ordinary = (a, b, r) =>
  !["Mega", "Form"].includes(r.Kind) && baseNumber(a) !== baseNumber(b);
const conditions = (r) =>
  Object.fromEntries(
    Object.entries(r).filter(([k]) => !["To", "From"].includes(k)),
  );

export function applySpeciesFidelity(catalog, review, sourceHash) {
  if (review.source_species_hash !== sourceHash)
    throw new Error(
      "Species source snapshot changed; review M4-S01 before applying its rulings",
    );
  if (
    review.user_policy.standalone_stage !== 3 ||
    !review.user_policy.alternate_forms_share_final_stage ||
    !review.user_policy.primal_ultra_terapagos_share_final_stage ||
    !review.user_policy.exclude_mega_gigantamax
  )
    throw new Error(
      "Unsupported species-stage ruling; implement and verify the changed policy first",
    );
  const byId = new Map(catalog.species.map((s) => [s.AppId, s]));
  const byName = new Map(catalog.species.map((s) => [s.Name, s]));
  for (const fix of review.clear_corrections) {
    const s = byId.get(fix.id);
    const matches = s?.Evolutions?.filter((r) => r.From === fix.from) ?? [];
    if (matches.length !== 1 || !byName.has(fix.to))
      throw new Error(`Evolution correction identity mismatch: ${fix.id}`);
    matches[0].From = fix.to;
    s.FidelityReview ??= [];
    s.FidelityReview.push({
      source: "printed source",
      decision: fix.basis,
      printed_page: fix.printed_page,
      pdf_page: fix.pdf_page,
    });
  }
  if (review.user_policy.unresolved_forward_rules !== "preserve")
    throw new Error("Unsupported unresolved-evolution policy");
  for (const ruling of review.user_evolution_rulings ?? []) {
    const source = byId.get(ruling.source_id),
      target = byId.get(ruling.target_id);
    if (!source || !target)
      throw new Error(`Unknown ruled evolution identity: ${ruling.id}`);
    const forward = source.Evolutions.filter((r) => r.To === target.Name);
    const reverse = target.Evolutions.filter((r) => r.From === source.Name);
    if (
      forward.length !== 1 ||
      reverse.length !== 1 ||
      !isDeepStrictEqual(conditions(forward[0]), ruling.expected_forward) ||
      !isDeepStrictEqual(conditions(reverse[0]), ruling.expected_reverse)
    )
      throw new Error(`Ruled evolution changed: ${ruling.id}`);
    const fi = source.Evolutions.indexOf(forward[0]),
      ri = target.Evolutions.indexOf(reverse[0]);
    source.Evolutions[fi] = { To: target.Name, ...ruling.resolved_conditions };
    target.Evolutions[ri] = {
      From: source.Name,
      ...ruling.resolved_conditions,
    };
    for (const entry of [source, target]) {
      entry.FidelityReview ??= [];
      entry.FidelityReview.push({
        source: "user-ruling",
        decision: ruling.decision,
      });
    }
    if (ruling.source_stage !== undefined) {
      source.Creation.stage = ruling.source_stage;
      source.Creation.stage_ruling = {
        source: "user-ruling",
        date: review.date,
        stage: ruling.source_stage,
        decision: ruling.decision,
      };
      source.Creation.review_note = [
        source.Creation.review_note,
        ruling.decision,
      ]
        .filter(Boolean)
        .join(" ");
    }
  }
  const formLinks = new Map(catalog.species.map((s) => [s.AppId, new Set()]));
  const hasOrdinary = new Set();
  const outgoingOrdinary = new Set();
  const issues = new Map();
  for (const s of catalog.species)
    for (const r of s.Evolutions ?? []) {
      const name = r.To ?? r.From;
      const other = byName.get(name);
      if (!other) {
        issues.set(`${s.AppId}:${name}`, {
          id: s.AppId,
          name: s.Name,
          issue: "missing-reference",
          relation: r,
        });
        continue;
      }
      if (other.AppId === s.AppId) {
        issues.set(s.AppId, {
          id: s.AppId,
          name: s.Name,
          issue: "self-reference",
          relation: r,
        });
        continue;
      }
      if (ordinary(s, other, r)) {
        hasOrdinary.add(s.AppId);
        hasOrdinary.add(other.AppId);
        const source = r.To ? s : other,
          target = r.To ? other : s;
        outgoingOrdinary.add(source.AppId);
        const forward = source.Evolutions.filter((e) => e.To === target.Name);
        const reverse = target.Evolutions.filter((e) => e.From === source.Name);
        if (
          forward.length !== 1 ||
          reverse.length !== 1 ||
          !isDeepStrictEqual(conditions(forward[0]), conditions(reverse[0]))
        )
          issues.set(`${source.AppId}->${target.AppId}`, {
            source_id: source.AppId,
            target_id: target.AppId,
            source_name: source.Name,
            target_name: target.Name,
            issue:
              forward.length !== 1 || reverse.length !== 1
                ? "missing-reciprocal-relation"
                : "directional-condition-conflict",
            forward,
            reverse,
          });
      } else if (!excluded(s) && !excluded(other)) {
        formLinks.get(s.AppId).add(other.AppId);
        formLinks.get(other.AppId).add(s.AppId);
      }
    }
  const applied = [];
  const seen = new Set();
  for (const s of catalog.species) {
    if (seen.has(s.AppId) || excluded(s) || s.DexID === "0000") continue;
    const pending = [s.AppId],
      component = [];
    while (pending.length) {
      const id = pending.pop();
      if (seen.has(id)) continue;
      seen.add(id);
      component.push(id);
      pending.push(...formLinks.get(id));
    }
    const standalone = !component.some((id) => hasOrdinary.has(id));
    const finalFormFamily =
      component.some((id) => byId.get(id).Creation.stage === 3) &&
      !component.some((id) => outgoingOrdinary.has(id));
    if (!standalone && !finalFormFamily) continue;
    for (const id of component) {
      const entry = byId.get(id);
      if (entry.Creation.stage === 3) continue;
      entry.Creation.stage = 3;
      const decision =
        "User ruling (30 October 2026): standalone species and non-Mega/Gigantamax forms of final-stage species use final-stage learning costs, including Primal, Ultra and Terapagos forms. This is a table ruling, not a printed-stage certification.";
      entry.Creation.stage_ruling = {
        source: "user-ruling",
        date: "2026-10-30",
        stage: 3,
        decision,
      };
      entry.Creation.review_note = [entry.Creation.review_note, decision]
        .filter(Boolean)
        .join(" ");
      entry.FidelityReview ??= [];
      entry.FidelityReview.push({ source: "user-ruling", decision });
      applied.push(id);
    }
  }
  return {
    batch: "M4-S01",
    status: "in-progress",
    species_total: catalog.species.length,
    stage_ruling_applied: applied.sort(),
    evolution_issues: [...issues.values()],
    manual_reviews_added: review.manual_reviews.length,
    resolved_evolution_rulings: (review.user_evolution_rulings ?? []).map(
      (r) => r.id,
    ),
    unresolved_forward_rules: review.user_policy.unresolved_forward_rules,
  };
}
