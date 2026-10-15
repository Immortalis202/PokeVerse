import { describe, it, expect } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import BattleView from "./BattleView";
import { newSheet } from "./sheets";
import type { Campaign } from "./bindings/Campaign";
import type { Combatant } from "./bindings/Combatant";
import type { Catalog } from "./catalog";

const catalog: Catalog = {
  edition: "3.0",
  hash: "catalog",
  species: [],
  moves: [],
  abilities: [],
  natures: [],
  items: [],
  warnings: [],
};
const trainer = {
  ...newSheet("player", "trainer"),
  id: "trainer",
  name: "Player trainer",
};
const sheets = Object.fromEntries(
  [
    trainer,
    ...["active", "reserve", "waiting", "opponent"].map((id) => ({
      ...newSheet(id === "opponent" ? "gm" : "player", "pokemon"),
      id,
      trainer_id: trainer.id,
      name: id,
    })),
  ].map((s) => [s.id, s]),
);
const profile = { version: 1, defenses: "split", hp: "vitality" };
const campaign: Campaign = {
  id: "campaign",
  name: "Gym",
  host_id: "host",
  gm_id: "gm",
  revision: 1,
  engine: "0.2.0-core",
  catalog_hash: "catalog",
  sheets,
  battle: null,
  finished_battles: [],
  events: [],
  battle_undo: [],
  rules_profile: profile,
  proposals: [],
  training_encounters: [],
};
function fighter(id: string): Combatant {
  return {
    runtime: {
      ability: { flags: [], counters: {} },
      will: 0,
      pain_ignored: 0,
      chance_round: 0,
      fate_round: 0,
      fate_pending: false,
      statuses: [],
      confusion_checked: 0,
      confusion_resisted: 0,
      love_resisted_turn: 0,
      flinch_round: 0,
      shield: 0,
      shield_round: 0,
      shield_streak: 0,
      healed: 0,
      recharge_actions: 0,
      recharge_round: 0,
      love_checked_turn: 0,
      will_enabled: false,
      adjudicated_round: 0,
      modifiers: {},
    },
    id,
    sheet_id: id,
    owner_id: id === "opponent" ? "gm" : "player",
    name: id,
    side: id === "opponent" ? "Opponents" : "Players",
    deployed: id !== "reserve",
    field_slot: id === "reserve" ? null : 0,
    available_round: id === "waiting" ? 2 : 0,
    hp: 10,
    max_hp: 10,
    conditions: [],
    actions: 0,
    reactions: 0,
    initiative: 0,
    initiative_rolled: false,
    used_moves: [],
    pending_damage: null,
    stat_overrides: {},
    skill_overrides: {},
    rules_profile: profile,
    selected_ability: "Static",
  };
}
function render(player: string, active = false) {
  const c = {
    ...campaign,
    battle: active
      ? {
          flow: null,
          id: "battle",
          active: true,
          round: 1,
          phase: "",
          slots_per_side: 3,
          trainer_switches: {},
          trainer_actions: {},
          automatic_validation: true,
          combatants: ["active", "reserve", "waiting", "opponent"].map(fighter),
        }
      : null,
  };
  return renderToStaticMarkup(
    <BattleView
      campaign={c}
      playerId={player}
      catalog={catalog}
      execute={async () => true}
      disabled={false}
      error={null}
    />,
  );
}
function select(html: string, label: string) {
  return (
    html.match(new RegExp(`${label}<select[^>]*>(.*?)</select>`))?.[1] ?? ""
  );
}
describe("registered battle teams", () => {
  it("GM preparation exposes starters and three active slots", () => {
    const html = render("gm");
    expect(html).toContain("Active slots per side");
    expect(html).toContain('min="1" max="6" value="3"');
    expect(html).toContain("Start reserve active");
    expect(html).not.toContain("Start Player trainer active");
  });
  it("players can act only with ready active Pokémon and target incoming opponents", () => {
    const html = render("player", true);
    expect(select(html, "Acting combatant")).toContain('value="active"');
    for (const id of ["reserve", "waiting", "opponent"])
      expect(select(html, "Acting combatant")).not.toContain(`value="${id}"`);
    expect(html).toContain("Can act from round 2");
    expect(select(html, "Target")).toContain('value="waiting"');
    expect(select(html, "Target")).not.toContain('value="reserve"');
    expect(select(html, "Recall / replace")).not.toContain('value="opponent"');
    expect(html).not.toContain("GM exception: waive switching cost");
  });
  it("appointed GM has switching overrides while a separate host does not", () => {
    expect(render("gm", true)).toContain("GM exception: waive switching cost");
    expect(select(render("gm", true), "Recall / replace")).toContain(
      'value="opponent"',
    );
    expect(render("host", true)).not.toContain(
      "GM exception: waive switching cost",
    );
  });
});

describe("structured core battle controls", () => {
  function core(
    player: string,
    queued = false,
    engine = "0.3.0-core",
    field = false,
    ability = false,
  ) {
    const combatants = ["active", "opponent"].map(fighter);
    combatants[0].runtime.will = 6;
    combatants[0].runtime.will_enabled = true;
    const c: Campaign = {
      ...campaign,
      engine,
      battle: {
        id: "core",
        active: true,
        round: 1,
        phase: "",
        slots_per_side: 1,
        trainer_switches: {},
        trainer_actions: {},
        automatic_validation: true,
        combatants,
        flow: {
          started: true,
          order: ["active", "opponent"],
          cursor: 0,
          turn: 1,
          held: [],
          reacted: [],
          window: queued && !field,
          deferred: null,
          ability_rules: engine === "0.5.0-core",
          ability_events: ability
            ? [
                {
                  id: "trigger",
                  combatant_id: "active",
                  target_id: "opponent",
                  ability: "Rough Skin",
                  kind: "retaliate",
                  move_name: "Tackle",
                  choices: ["neutral", "roll"],
                },
              ]
            : [],
          intimidate: {},
          ranges: {},
          covers: {},
          field_effects: [],
          queue: queued
            ? [
                {
                  actor_id: "opponent",
                  target_id: "active",
                  move_name: "Tackle",
                  sequence: 1,
                  successes: 2,
                  critical: false,
                  timing: "main",
                  priority: 0,
                  cancelled: false,
                  damage_done: false,
                  damage: 0,
                  raw_damage_successes: 0,
                  ice: false,
                  targets: [],
                  field_scope: field ? "side:Opponents" : null,
                  shared_dice: [],
                  damage_modifiers: null,
                  defense_key: "",
                  damage_context: null,
                },
              ]
            : [],
        },
      },
    };
    return renderToStaticMarkup(
      <BattleView
        campaign={c}
        playerId={player}
        catalog={catalog}
        execute={async () => true}
        disabled={false}
        error={null}
      />,
    );
  }
  it("shows owned Will/reaction decisions and reserves GM adjudication for the appointed GM", () => {
    const player = core("player", true);
    expect(player).toContain("Will: fate");
    expect(player).toContain("Pass reaction");
    expect(player).not.toContain("Record GM resolution");
    expect(player).not.toContain("Structured status decision");
    const gm = core("gm", true);
    expect(gm).toContain("Structured status decision");
    expect(gm).toContain("GM severe status damage this round");
    expect(core("host", true)).not.toContain("Structured status decision");
  });
  it("shows current turn and exposes no combatant controls to a separate host", () => {
    expect(core("player")).toContain("Turn 1: active");
    expect(core("player")).toContain("Hold / resolve skipped turn");
    expect(core("host")).not.toContain("Hold / resolve skipped turn");
  });
  it("exposes advanced targeting while keeping range and field rulings GM-only", () => {
    const player = core("player", false, "0.4.0-core");
    expect(player).toContain("Multiple targets and battlefield moves");
    expect(player).toContain("Preview target legality and accuracy");
    expect(player).not.toContain("Confirm selected Pokémon in range");
    expect(core("gm", false, "0.4.0-core")).toContain(
      "Confirm selected Pokémon in range",
    );
    const gmField = core("gm", true, "0.4.0-core", true);
    expect(gmField).toContain("Record field effect and finish action");
    expect(gmField).not.toContain("Resolve automatic damage");
    expect(core("host", true, "0.4.0-core", true)).not.toContain(
      "Record field effect and finish action",
    );
  });
  it("lets the ability owner choose retaliation while the separate host only observes", () => {
    const player = core("player", false, "0.5.0-core", false, true);
    expect(player).toContain("Pending: Rough Skin");
    expect(player).toContain("Deal 1 neutral damage");
    expect(player).not.toContain("Record manual ability resolution");
    expect(core("gm", false, "0.5.0-core", false, true)).toContain(
      "Record manual ability resolution",
    );
    const host = core("host", false, "0.5.0-core", false, true);
    expect(host).toContain("Pending: Rough Skin");
    expect(host).not.toContain("Deal 1 neutral damage");
  });
});
