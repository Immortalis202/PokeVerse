import type { Campaign } from "./bindings/Campaign";
import type { Sheet } from "./bindings/Sheet";
import type { Species } from "./catalog";
import { attributes } from "./catalog";
import type { CharacterBuild } from "./bindings/CharacterBuild";
export function newBuild(): CharacterBuild {
  return {
    version: 1,
    status: "draft",
    origin: "starter",
    age: "Teen",
    allocations: {},
    socials: {},
    happiness: 2,
    loyalty: 2,
    training_points: 0,
    legacy_xp: 0,
    will: 0,
    max_will: 0,
    profile: { version: 1, defenses: "split", hp: "vitality" },
    hp_attribute: "Vitality",
    available_abilities: [],
    learned: {},
    forgotten: {},
    overranked: null,
    evolution_ready: null,
    ledger: [],
  };
}
export function sheetValues(map: {
  [key: string]: Sheet | undefined;
}): Sheet[] {
  return Object.values(map).filter((s): s is Sheet => !!s);
}
export function newSheet(
  owner: string,
  kind: string,
  trainerId: string | null = null,
): Sheet {
  return {
    id: crypto.randomUUID(),
    owner_id: owner,
    revision: 0,
    kind,
    name: "",
    trainer_id: trainerId,
    species_id: null,
    rank: "Rookie",
    nature: "",
    ability: "",
    stats: Object.fromEntries(attributes.map((k) => [k, 0])),
    skills: {},
    moves: [],
    hp: kind === "trainer" ? 4 : 1,
    max_hp: kind === "trainer" ? 4 : 1,
    xp: 0,
    inventory: {},
    notes: "",
    archived: false,
    squad_order: 0,
    conditions: [],
    workflow: newBuild(),
  };
}
export function applySpecies(sheet: Sheet, species: Species): Sheet {
  return {
    ...sheet,
    species_id: species.AppId,
    name: sheet.name || species.Name,
    ability: species.Ability1,
    stats: Object.fromEntries(
      attributes.map((k) => [k, Number(species[k] ?? 0)]),
    ),
    max_hp: species.BaseHP + Number(species.Vitality ?? 0),
    hp: species.BaseHP + Number(species.Vitality ?? 0),
    moves: [],
    workflow: {
      ...(sheet.workflow ?? newBuild()),
      available_abilities: [species.Ability1, species.Ability2].filter(Boolean),
    },
  };
}
export function cloneTrainer(sheets: Sheet[], trainer: Sheet): Sheet[] {
  const trainerId = crypto.randomUUID();
  return [
    {
      ...structuredClone(trainer),
      id: trainerId,
      revision: 0,
      name: `${trainer.name} (copy)`,
      archived: false,
      workflow: trainer.workflow
        ? {
            ...structuredClone(trainer.workflow),
            status: "review",
            origin: "legacy",
            ledger: [],
          }
        : null,
    },
    ...sheets
      .filter((s) => s.trainer_id === trainer.id)
      .map((s) => ({
        ...structuredClone(s),
        id: crypto.randomUUID(),
        trainer_id: trainerId,
        revision: 0,
        archived: false,
        workflow: s.workflow
          ? {
              ...structuredClone(s.workflow),
              status: "review",
              origin: "legacy",
              ledger: [],
            }
          : null,
      })),
  ];
}

export function battleSheet(
  campaign: Campaign,
  id: string | undefined,
): Sheet | undefined {
  if (!id) return undefined;
  const battle = campaign.battle;
  if (battle?.active && battle.flow?.forms?.enabled) {
    const form = battle.flow.forms.current[id];
    if (form) return form.sheet;
  }
  const participant = battle?.combatants.find((p) => p.id === id);
  return participant ? campaign.sheets[participant.sheet_id] : undefined;
}
