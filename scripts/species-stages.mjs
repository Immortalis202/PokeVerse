export const temporaryStageForm = (s) =>
  /\(Mega\b|Gigantamax/i.test(s.Name) ||
  s.Evolutions?.some((r) => r.From && r.Kind === "Mega");

// This resolves learning-cost stages from the audited graph. It never certifies
// a printed label or infers evolution between unlinked regional forms.
export function resolveLearningStages(species) {
  const byName = new Map(species.map((s) => [s.Name, s]));
  const byId = new Map(species.map((s) => [s.AppId, s]));
  const incoming = new Map(species.map((s) => [s.AppId, new Set()]));
  const outgoing = new Map(species.map((s) => [s.AppId, new Set()]));
  const links = new Map(species.map((s) => [s.AppId, new Set()]));
  const excluded = (s) => temporaryStageForm(s) || s.DexID === "0000";
  for (const s of species) {
    for (const r of s.Evolutions ?? []) {
      const other = byName.get(r.To ?? r.From);
      if (!other || excluded(s) || excluded(other)) continue;
      const [source, target] = r.To ? [s, other] : [other, s];
      if (
        !["Mega", "Form"].includes(r.Kind) &&
        source.Number !== target.Number
      ) {
        outgoing.get(source.AppId).add(target.AppId);
        incoming.get(target.AppId).add(source.AppId);
      } else {
        links.get(s.AppId).add(other.AppId);
        links.get(other.AppId).add(s.AppId);
      }
    }
  }
  const direct = new Map();
  for (const s of species) {
    const before = incoming.get(s.AppId).size > 0,
      after = outgoing.get(s.AppId).size > 0;
    if (before || after) direct.set(s.AppId, after ? (before ? 2 : 1) : 3);
  }
  const visited = new Set();
  const changes = [],
    unresolved = [];
  for (const s of species) {
    if (excluded(s)) {
      s.Creation.stage_resolution = {
        kind:
          s.DexID === "0000"
            ? "excluded-placeholder"
            : "excluded-temporary-form",
        previous_stage: s.Creation.stage,
      };
      if (s.Creation.stage !== null)
        changes.push({
          id: s.AppId,
          name: s.Name,
          previous_stage: s.Creation.stage,
          stage: null,
        });
      s.Creation.stage = null;
      s.Creation.inferred_stage = null;
      continue;
    }
    if (visited.has(s.AppId)) continue;
    const pending = [s.AppId],
      component = [];
    while (pending.length) {
      const id = pending.pop();
      if (visited.has(id)) continue;
      visited.add(id);
      component.push(id);
      pending.push(...links.get(id));
    }
    const seeds = component.filter((id) => direct.has(id));
    const seedStages = new Set(seeds.map((id) => direct.get(id)));
    for (const id of component) {
      const entry = byId.get(id);
      const previous = entry.Creation.stage;
      const stage =
        direct.get(id) ??
        (seeds.length === 0
          ? 3
          : seedStages.size === 1
            ? direct.get(seeds[0])
            : null);
      const kind = direct.has(id)
        ? "ordinary-evolution-graph"
        : seeds.length === 0
          ? "user-standalone-final"
          : stage !== null
            ? "linked-form-inheritance"
            : "unresolved-form-family";
      entry.Creation.stage_resolution = {
        kind,
        previous_stage: previous,
        incoming: [...incoming.get(id)].sort(),
        outgoing: [...outgoing.get(id)].sort(),
        ...(kind === "linked-form-inheritance"
          ? { inherited_from: seeds.sort() }
          : {}),
      };
      entry.Creation.stage = stage;
      entry.Creation.inferred_stage = stage;
      if (stage !== previous)
        changes.push({ id, name: entry.Name, previous_stage: previous, stage });
      if (stage === null) unresolved.push(id);
    }
  }
  return { changes, unresolved };
}
