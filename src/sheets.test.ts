import { describe, expect, it } from "vitest";
import { newSheet, cloneTrainer, applySpecies } from "./sheets";
import type { Species } from "./catalog";
describe("personal library identities", () => {
  it("copies a trainer and squad without retaining campaign identities", () => {
    const trainer = {
      ...newSheet("owner", "trainer"),
      name: "Maya",
      revision: 3,
    };
    const pokemon = {
      ...newSheet("owner", "pokemon", trainer.id),
      revision: 4,
    };
    const copies = cloneTrainer([trainer, pokemon], trainer);
    expect(copies[0].id).not.toBe(trainer.id);
    expect(copies[1].id).not.toBe(pokemon.id);
    expect(copies[1].trainer_id).toBe(copies[0].id);
    expect(copies.every((s) => s.revision === 0)).toBe(true);
    copies[1].stats.Strength = 5;
    expect(pokemon.stats.Strength).toBe(0);
  });
  it("starts species HP at base HP plus Vitality, as in corebook p.26", () => {
    const species: Species = {
      AppId: "0025:pikachu",
      DexID: "0025",
      Name: "Pikachu",
      Type1: "Electric",
      Type2: "",
      Ability2: "",
      HiddenAbility: "",
      Image: "pikachu.png",
      DexDescription: "",
      Moves: [],
      Evolutions: [],
      BaseHP: 4,
      Strength: 2,
      Dexterity: 2,
      Vitality: 1,
      Special: 2,
      Insight: 2,
      Ability1: "Static",
      RecommendedRank: "Rookie",
    };
    const p = applySpecies(newSheet("owner", "pokemon", "trainer"), species);
    expect(p.hp).toBe(5);
    expect(p.name).toBe("Pikachu");
    expect(p.stats.Strength).toBe(2);
    expect(p.moves).toEqual([]);
  });
});
