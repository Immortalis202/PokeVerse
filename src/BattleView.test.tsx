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
  form_memory: {},
  ability_inheritance: { minutes: 0, copies: {}, selected: {}, history: [] },
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
    provenance = false,
    events = false,
    heldItems = false,
    symbiosis = "",
  ) {
    const combatants = ["active", "opponent"].map(fighter);
    if (provenance) combatants[0].selected_ability = "Sheer Force";
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
          standalone: {
            enabled: false,
            profiles: {},
            rivalry_basis: null,
            torrent_bonus: null,
            perish: {},
            ended_round: 0,
            history: [],
          },
          information: { enabled: false, active: [], offers: {}, reveals: [] },
          forms: {
            enabled: false,
            originals: {},
            current: {},
            plans: {},
            choices: {},
            seen: [],
            mega_trainers: [],
            key_stones: [],
            equipment: {},
            history: [],
          },
          items: {
            enabled: false,
            initialized: false,
            finalized: false,
            return_policy: null,
            units: {},
            history: [],
            unburdened: [],
            symbiosis_chains: {},
            thefts: {},
            theft_seen: [],
            end_round: 0,
            redacted: false,
            frisk_active: [],
            reveals: [],
            natural_gift: {},
            berry_moves: {},
            fling: {},
            fling_moves: {},
            fling_hits: [],
            berry_hits: {},
            recycle: {},
            recycle_uses: {},
            recycle_attempts: [],
            recycle_moves: {},
            chew_charges: {},
            chew_serial: 0,
            stages: {},
            life_orb_moves: {},
            chance_moves: {},
            fetches: {},
          },
          events: {
            enabled: events,
            pressure_sources: [],
            pressure_targets: [],
            history: events
              ? [
                  "Pressure: opponent entered; active spent 3 Will without gaining an effect",
                ]
              : [],
          },
          provenance_rules: provenance,
          sequencing: {
            enabled: engine === "0.15.0-core",
            identities: {},
            gorilla: {},
            visits: {},
            visited_turn: null,
            truant: {},
            penalties: [],
            preparing: [],
            dancer_round: {},
            pending: [],
            dances: {},
            resume: false,
            copies: [],
            parental: {},
            history: [],
          },
          stat_channels: {
            enabled: engine === "0.14.0-core",
            active: [],
            grants: [],
            removed: {},
            removed_abilities: {},
            choices: {},
            history: [],
          },
          reviewed_rules: engine === "0.13.0-core",
          transfers: {
            enabled: false,
            contact_overrides: {},
            seen: [],
            offers: {},
            history: [],
          },
          ability_changes: {
            enabled: false,
            originals: {},
            assignments: {},
            negated: [],
            suppressed: [],
            trace: {},
            active: [],
            gas_rulings: [],
            history: [],
          },
          decoys: {
            enabled: engine === "0.10.0-core",
            active:
              engine === "0.10.0-core"
                ? {
                    active: {
                      hp: 1,
                      defenses: [4, 5],
                      source: "active",
                      move_name: "Substitute",
                    },
                  }
                : {},
            intercepted: [],
            history: [],
          },
          tactics: {
            enabled: ["0.9.0-core", "0.10.0-core"].includes(engine),
            lures: [],
            switches: [],
            switchers: {},
            redirect_choices: {},
            priority_choices: {},
            coat_rules: {},
            clash_damage: {},
            clash_pending: {},
            baton: {},
            history: [],
            payer: null,
          },
          special: {
            enabled: ["0.8.0-core", "0.9.0-core", "0.10.0-core"].includes(
              engine,
            ),
            bides: {},
            charges: {},
            rampages: {},
            shields: {},
            effects: [],
            history: [],
            parameters: {},
            selections: {},
            random: {},
            series: null,
          },
          extra: {
            enabled: [
              "0.7.0-core",
              "0.8.0-core",
              "0.9.0-core",
              "0.10.0-core",
              "0.13.0-core",
            ].includes(engine),
            active: [],
            fainted: [],
            round_started: 0,
          },
          battlefield: {
            enabled: [
              "0.6.0-core",
              "0.7.0-core",
              "0.8.0-core",
              "0.9.0-core",
              "0.10.0-core",
              "0.13.0-core",
            ].includes(engine),
            natural: null,
            weather: null,
            rules: [],
            events: [],
            sheltered: [],
            known_active: [],
            start_round: 0,
            end_round: 0,
            serial: 0,
            indoors: false,
            sunlight: true,
            night: false,
            corrections: [],
            species_types: {},
          },
          started: true,
          order: ["active", "opponent"],
          cursor: 0,
          turn: 1,
          held: [],
          reacted: [],
          window: queued && !field,
          deferred: null,
          ability_rules: [
            "0.5.0-core",
            "0.6.0-core",
            "0.7.0-core",
            "0.8.0-core",
            "0.9.0-core",
          ].includes(engine),
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
                  sequencing_penalty: false,
                  sheer_force: false,
                  special_index: 0,
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
                  targets: events
                    ? [
                        {
                          target_id: "active",
                          successes: 2,
                          critical: true,
                          responded: false,
                          avoided: false,
                          resolved: false,
                        },
                      ]
                    : [],
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
    if (heldItems) {
      const items = c.battle!.flow!.items;
      items.enabled = true;
      items.initialized = true;
      for (const [holder, berry] of [
        ["active", "Oran Berry"],
        ["opponent", "Sitrus Berry"],
      ]) {
        items.units[holder] = {
          unit: {
            id: holder,
            item_id: berry,
            name: berry,
            origin_sheet_id: holder,
            uses: 0,
          },
          initial_holder: holder,
          initial_slot: 0,
          holder,
          slot: 0,
          state: "held",
          harvest_pending: false,
        };
      }
    }
    if (symbiosis) {
      c.battle!.flow!.ability_rules = true;
      c.battle!.flow!.ability_events = [
        {
          id: "symbiosis-event",
          combatant_id: "active",
          target_id: "opponent",
          ability: "Symbiosis",
          kind: symbiosis,
          move_name: "Held-item loss",
          choices: ["give:active"],
        },
      ];
    }
    if (symbiosis === "item-frisk") {
      c.battle!.flow!.ability_events[0].ability = "Frisk";
      c.battle!.flow!.ability_events[0].choices = ["opponent"];
      c.battle!.flow!.items.reveals = [
        {
          owner_id: "player",
          source_id: "active",
          target_id: "opponent",
          items: ["Revealed Berry"],
          round: 1,
        },
      ];
    }
    if (symbiosis === "item-theft") {
      c.battle!.flow!.ability_events[0].ability = "Magician";
      c.battle!.flow!.ability_events[0].choices = ["swap:0", "decline"];
      c.battle!.flow!.items.thefts["symbiosis-event"] = {
        "swap:0": {
          thief_id: "active",
          target_id: "opponent",
          stolen_id: "opponent",
          outgoing_id: "active",
        },
      };
    }
    if (symbiosis === "natural-gift") {
      c.sheets = structuredClone(c.sheets);
      c.sheets.active!.moves = ["Natural Gift"];
      c.battle!.combatants[0].selected_ability = "Ripen";
      c.battle!.flow!.ability_events = [];
      c.battle!.flow!.items.units.active!.unit.flavor = "Spicy";
    }
    if (symbiosis === "fling") {
      c.sheets = structuredClone(c.sheets);
      c.sheets.active!.moves = ["Fling"];
      c.battle!.flow!.ability_events = [];
      c.battle!.flow!.items.units.opponent!.state = "ground";
    }
    if (symbiosis === "item-cud-chew") {
      c.battle!.flow!.ability_events[0].ability = "Cud Chew";
      c.battle!.flow!.ability_events[0].choices = ["activate"];
      c.battle!.flow!.items.berry_hits["symbiosis-event"] = "opponent";
      c.battle!.flow!.items.units.opponent!.state = "berry_hit_pending";
    }
    if (symbiosis === "recycle") {
      c.sheets = structuredClone(c.sheets);
      c.sheets.active!.moves = ["Recycle"];
      c.battle!.combatants[0].selected_ability = "Cud Chew";
      c.battle!.flow!.ability_events = [];
      const items = c.battle!.flow!.items;
      items.units.opponent!.state = "consumed";
      items.recycle_uses.active = 1;
      items.chew_charges.saved = {
        combatant_id: "active",
        unit: items.units.opponent!.unit,
        source: "recycle",
      };
    }
    if (symbiosis === "fetch") {
      c.sheets = structuredClone(c.sheets);
      c.sheets.trainer!.inventory = { Pokeball: 1 };
      c.sheets.opponent!.trainer_id = null;
      c.battle!.combatants[0].selected_ability = "Ball Fetch";
      c.battle!.flow!.ability_events = [];
      c.battle!.flow!.items.fetches["capture-roll-1"] = {
        trainer_id: "trainer",
        fetcher_id: "active",
        target_id: "opponent",
        unit: {
          id: "protected-ball",
          item_id: "pokeball",
          name: "Pokeball",
          origin_sheet_id: "trainer",
          uses: 0,
        },
        round: 1,
        returned: false,
      };
    }
    if (symbiosis === "review-gluttony" || symbiosis === "review-klutz") {
      c.battle!.flow!.ability_events = [];
      c.battle!.combatants[0].selected_ability =
        symbiosis === "review-gluttony" ? "Gluttony" : "Klutz";
      c.battle!.flow!.items.units.active!.unit.name = "Choice Band";
      c.battle!.flow!.items.units.active!.review = {
        scope: "manual",
        note: "Manual-review marker: this effect requires GM adjudication.",
      };
    }
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
  it("shows private item effect reviews and printed Gluttony/Klutz boundaries", () => {
    const html = (player: string, kind: string) =>
      core(player, false, "0.17.0-core", false, true, false, false, true, kind);
    expect(html("player", "review-gluttony")).toContain(
      "no negative ingestion effects",
    );
    expect(html("player", "review-gluttony")).toContain("Manual-review marker");
    expect(html("gm", "review-klutz")).toContain(
      "printed form/type-changing exceptions",
    );
    expect(html("host", "review-klutz")).not.toContain("Manual-review marker");
  });
  it("shows Ball Fetch recording only to the GM and protected balls only to their trainer", () => {
    const html = (player: string) =>
      core(
        player,
        false,
        "0.17.0-core",
        false,
        true,
        false,
        false,
        true,
        "fetch",
      );
    expect(html("gm")).toContain("Record failed capture and protect this ball");
    expect(html("player")).not.toContain(
      "Record failed capture and protect this ball",
    );
    expect(html("player")).toContain("capture-roll-1");
    expect(html("host")).not.toContain("capture-roll-1");
    expect(html("gm")).toContain("Adjudicate its action cost first");
  });
  it("offers spent-item Recycle choices and saved Cud Chew effects only for owned participants", () => {
    const html = (player: string) =>
      core(
        player,
        false,
        "0.17.0-core",
        false,
        true,
        false,
        false,
        true,
        "recycle",
      );
    expect(html("player")).toContain("Select Sitrus Berry for Recycle");
    expect(html("player")).toContain("Recycle attempts this battle: 1/3");
    expect(html("player")).toContain("Chew Sitrus Berry again");
    expect(html("host")).not.toContain("Select Sitrus Berry for Recycle");
    expect(html("host")).not.toContain("Chew Sitrus Berry again");
    expect(html("gm")).toContain("Record item spent after adjudication");
    expect(html("player")).not.toContain(
      "Record item spent after adjudication",
    );
  });
  it("offers Fling selection and paid ground retrieval only through owned combatants", () => {
    const html = (player: string) =>
      core(
        player,
        false,
        "0.17.0-core",
        false,
        true,
        false,
        false,
        true,
        "fling",
      );
    expect(html("player")).toContain("Use Oran Berry for Fling");
    expect(html("player")).toContain(
      "Retrieve Sitrus Berry for active · spend one action",
    );
    expect(html("host")).not.toContain("Use Oran Berry for Fling");
    expect(html("host")).not.toContain("Retrieve Sitrus Berry for active");
  });
  it("names Cud Chew's thrown-berry decision for its owner and GM", () => {
    const html = (player: string) =>
      core(
        player,
        false,
        "0.17.0-core",
        false,
        true,
        false,
        false,
        true,
        "item-cud-chew",
      );
    expect(html("player")).toContain("Activate the berry; save another chew");
    expect(html("player")).toContain("Sitrus Berry");
    expect(html("host")).not.toContain("Sitrus Berry");
    expect(html("gm")).toContain("Activate the berry; save another chew");
    expect(html("host")).not.toContain("Activate the berry; save another chew");
  });
  it("offers Natural Gift selection to owners and leaves flavor and Ripen healing rulings with the GM", () => {
    const html = (player: string) =>
      core(
        player,
        false,
        "0.17.0-core",
        false,
        true,
        false,
        false,
        true,
        "natural-gift",
      );
    expect(html("player")).toContain("Use Oran Berry for Natural Gift");
    expect(html("player")).toContain("flavor Spicy");
    expect(html("player")).not.toContain("GM Ripen extra healing");
    expect(html("gm")).toContain("GM Ripen extra healing");
    expect(html("gm")).toContain("Record the berry&#x27;s actual flavor");
    expect(html("host")).not.toContain("Use Oran Berry for Natural Gift");
  });
  it("keeps Frisk findings with the trainer and reserves the target choice for the GM", () => {
    const html = (player: string) =>
      core(
        player,
        false,
        "0.17.0-core",
        false,
        true,
        false,
        false,
        true,
        "item-frisk",
      );
    expect(html("player")).toContain("Revealed Berry");
    expect(html("player")).not.toContain("Reveal held items of opponent");
    expect(html("gm")).toContain("Reveal held items of opponent");
    expect(html("gm")).toContain("Revealed Berry");
    expect(html("host")).not.toContain("Revealed Berry");
    expect(html("host")).not.toContain("Reveal held items of opponent");
  });
  it("labels Magician exchanges and the choice to keep an item for its owner", () => {
    const html = (player: string) =>
      core(
        player,
        false,
        "0.17.0-core",
        false,
        true,
        false,
        false,
        true,
        "item-theft",
      );
    expect(html("player")).toContain("Exchange Oran Berry for Sitrus Berry");
    expect(html("player")).toContain("Keep my held item");
    expect(html("gm")).toContain("Exchange Oran Berry for Sitrus Berry");
    expect(html("host")).not.toContain("Exchange Oran Berry for Sitrus Berry");
  });
  it("shows named Symbiosis handoffs to the donor owner and simultaneous ordering only to the GM", () => {
    const html = (player: string, kind: string) =>
      core(player, false, "0.17.0-core", false, true, false, false, true, kind);
    expect(html("player", "item-symbiosis")).toContain(
      "Give Oran Berry from active to opponent",
    );
    expect(html("host", "item-symbiosis")).not.toContain("Give Oran Berry");
    expect(html("player", "item-symbiosis-order")).not.toContain(
      "Give Oran Berry",
    );
    expect(html("gm", "item-symbiosis-order")).toContain(
      "Give Oran Berry from active to opponent",
    );
    expect(html("gm", "item-symbiosis-cycle")).toContain(
      "Repeated Symbiosis handoffs would form a cycle",
    );
  });
  it("exposes held berry use only to owners and reserves item rulings for the GM", () => {
    const html = (player: string) =>
      core(player, false, "0.17.0-core", false, false, false, false, true);
    const player = html("player");
    expect(player).toContain("Consume Oran Berry for active");
    expect(player).not.toContain("Sitrus Berry");
    expect(player).not.toContain("Record transfer");
    const gm = html("gm");
    expect(gm).toContain("Consume Sitrus Berry for opponent");
    expect(gm).toContain("Record atomic swap");
    expect(gm).toContain("Explicitly override Sticky Hold");
    expect(gm).toContain("Return surviving items");
    const host = html("host");
    expect(host).not.toContain("Oran Berry");
    expect(host).not.toContain("Sitrus Berry");
    expect(host).not.toContain("Record item loss");
    expect(core("gm")).not.toContain("Held battle items");
  });
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
  it("shows saved Pressure history and per-target accuracy results", () => {
    const html = core("player", true, "0.9.0-core", true, false, false, true);
    expect(html).toContain("Battle ability events");
    expect(html).toContain("spent 3 Will without gaining an effect");
    expect(html).toContain("2 accuracy successes, critical");
    expect(core("player", true, "0.9.0-core")).not.toContain(
      "Battle ability events",
    );
  });
  it("shows the Sheer Force tradeoff to its owner and GM", () => {
    for (const player of ["player", "gm"]) {
      expect(core(player, false, "0.9.0-core", false, false, true)).toContain(
        "Trade added effects for damage",
      );
    }
    expect(core("host", false, "0.9.0-core", false, false, true)).not.toContain(
      "Trade added effects for damage",
    );
    expect(core("player", false, "0.9.0-core")).not.toContain(
      "Trade added effects for damage",
    );
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
  it("exposes battlefield environment, typing and arena controls only to the appointed GM", () => {
    const gm = core("gm", false, "0.6.0-core");
    expect(gm).toContain("Apply environment");
    expect(gm).toContain("Apply temporary typing");
    expect(gm).toContain("Add arena condition");
    expect(core("host", false, "0.6.0-core")).not.toContain(
      "Apply environment",
    );
    expect(core("player", false, "0.6.0-core")).toContain("Battlefield");
    expect(core("player", false, "0.6.0-core")).not.toContain(
      "Apply temporary typing",
    );
  });
  it("reserves sourced stat effects for the GM in the extended ability engine", () => {
    expect(core("gm", false, "0.7.0-core")).toContain(
      "Apply sourced stat effect",
    );
    expect(core("player", false, "0.7.0-core")).not.toContain(
      "Apply sourced stat effect",
    );
    expect(core("host", false, "0.7.0-core")).not.toContain(
      "Apply sourced stat effect",
    );
    expect(core("gm", false, "0.6.0-core")).not.toContain(
      "Apply sourced stat effect",
    );
  });
  it("exposes successive sequences and keeps physical parameters exclusive to the GM", () => {
    const gm = core("gm", false, "0.8.0-core");
    expect(gm).toContain("Special move automation");
    expect(gm).toContain("Commit successive");
    expect(gm).toContain("Use as a reaction");
    expect(gm).toContain("Weight override");
    expect(core("host", false, "0.8.0-core")).not.toContain("Weight override");
    expect(core("player", false, "0.7.0-core")).not.toContain(
      "Special move automation",
    );
  });
  it("exposes targeted reactions and reserves GM timing/redirector rulings for the GM", () => {
    const gm = core("gm", false, "0.9.0-core");
    expect(gm).toContain("Use selected targets for a reaction move");
    expect(gm).toContain("Targeting and replacements");
    expect(gm).toContain("Record competing redirector");
    expect(gm).toContain("Record Magic Coat duration");
    const player = core("player", false, "0.9.0-core");
    expect(player).toContain("Targeting and replacements");
    expect(player).not.toContain("Record priority ruling");
    expect(core("host", false, "0.9.0-core")).not.toContain(
      "Record competing redirector",
    );
    expect(core("gm", false, "0.8.0-core")).not.toContain(
      "Targeting and replacements",
    );
  });
  it("shows decoy resources and excluded transformations for owners, GM and host", () => {
    for (const player of ["gm", "player", "host"]) {
      const html = core(player, false, "0.10.0-core");
      expect(html).toContain("Substitute decoy");
      expect(html).toContain("1/2 HP");
      expect(html).toContain("Frozen Defense 4");
      expect(html).toContain("Dynamax");
      expect(html).toContain("Z-Moves are out of scope");
      expect(html).toContain("Targeting and replacements");
    }
    expect(core("gm", false, "0.9.0-core")).not.toContain("Substitute decoy");
  });
  it("shows explicit night state to players and reserves its control for the GM", () => {
    expect(core("gm", false, "0.13.0-core")).toContain(
      "Night time (Mind’s Eye)",
    );
    expect(core("player", false, "0.13.0-core")).toContain("Day time.");
    expect(core("player", false, "0.13.0-core")).not.toContain(
      "Night time (Mind’s Eye)",
    );
    expect(core("host", false, "0.13.0-core")).not.toContain(
      "Night time (Mind’s Eye)",
    );
  });
});
