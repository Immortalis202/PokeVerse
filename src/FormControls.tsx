import { useState } from "react";
import type { Sheet } from "./bindings/Sheet";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";
import type { FormCommand } from "./bindings/FormCommand";
import forms from "../rules/battle-forms.json";

const types = [
  "Normal",
  "Fire",
  "Water",
  "Electric",
  "Grass",
  "Ice",
  "Fight",
  "Poison",
  "Ground",
  "Flying",
  "Psychic",
  "Bug",
  "Rock",
  "Ghost",
  "Dragon",
  "Dark",
  "Steel",
  "Fairy",
];
const attributes = ["Strength", "Dexterity", "Vitality", "Special", "Insight"];
const definition = forms.species as Record<
  string,
  {
    name: string;
    stats: Record<string, number>;
    limits: Record<string, number>;
  }
>;
export default function FormControls({
  campaign,
  playerId,
  execute,
  disabled,
}: {
  campaign: Campaign;
  playerId: string;
  execute: (action: Action) => Promise<boolean>;
  disabled: boolean;
}) {
  const [selected, setSelected] = useState("");
  const [form, setForm] = useState("");
  const [allocation, setAllocation] = useState<Record<string, number>>({});
  const [socials, setSocials] = useState<Record<string, number>>({});
  const [skills, setSkills] = useState<Record<string, number>>({});
  const [moves, setMoves] = useState("");
  const [target, setTarget] = useState("");
  const [type, setType] = useState("Normal");
  const [mask, setMask] = useState("Teal");
  const [firstAbility, setFirstAbility] = useState("");
  const [secondAbility, setSecondAbility] = useState("");
  const [reason, setReason] = useState("");
  const battle = campaign.battle,
    state = battle?.flow?.forms;
  if (!battle || !state?.enabled) return null;
  const gm = playerId === campaign.gm_id;
  if (!battle.active)
    return gm && Object.keys(campaign.form_memory).length > 0 ? (
      <section className="card">
        <h2>Persistent Core forms</h2>
        <p>
          A Core form remains until the GM records its release and later
          recapture.
        </p>
        <label>
          Restoration ruling
          <input value={reason} onChange={(e) => setReason(e.target.value)} />
        </label>
        {Object.keys(campaign.form_memory).map((id) => (
          <button
            key={id}
            disabled={disabled || !reason.trim()}
            onClick={() =>
              void execute({
                type: "form",
                command: { operation: "restore_core", sheet_id: id },
                reason,
              })
            }
          >
            Confirm release and later recapture:{" "}
            {campaign.sheets[id]?.name ?? "Minior"}
          </button>
        ))}
      </section>
    ) : null;
  const owned = battle.combatants.filter((p) => gm || p.owner_id === playerId);
  const p = owned.find((p) => p.id === selected) ?? owned[0];
  if (!p) return null;
  const sheet = campaign.sheets[p.sheet_id];
  if (!sheet) return null;
  const current = state.current[p.id];
  const species = current?.sheet.species_id ?? sheet.species_id ?? "";
  const held = Object.values(battle.flow!.items?.units ?? {}).filter(
    (u) => u && u.holder === p.id && u.state === "held",
  );
  const mega = held.flatMap((u) => {
    const m = (forms.mega as Record<string, { base: string; form: string }>)[
      u!.unit.name
    ];
    return m?.base === species ? [{ stone: u!.unit.name, form: m.form }] : [];
  });
  const linked =
    (forms.links as Record<string, Record<string, string[]>>)[
      p.selected_ability
    ]?.[species] ?? [];
  const options = [
    ...new Set([
      ...linked,
      ...mega.map((m) => m.form),
      ...(p.selected_ability === "As One" ? Object.keys(definition) : []),
    ]),
  ];
  const chosen = options.includes(form) ? form : (options[0] ?? "");
  const trainers = Object.values(campaign.sheets).filter(
    (s): s is Sheet => !!s && s.kind === "trainer",
  );
  const send = (command: FormCommand, fallback: string) =>
    execute({ type: "form", command, reason: reason.trim() || fallback });
  const disabledReason = disabled || (gm && reason.trim().length === 0);
  const formName = (id: string) => definition[id]?.name ?? id;
  return (
    <section className="card" aria-label="Forms and transformations">
      <h2>Forms and transformations</h2>
      <p>
        Temporary forms keep permanent sheets intact. A form uses its own rank
        point distribution. Unspecified arrival times, equipment and joint-form
        details require a recorded GM ruling.
      </p>
      <label>
        Pokémon
        <select
          value={p.id}
          onChange={(e) => {
            setSelected(e.target.value);
            setAllocation({});
            setSocials({});
            setSkills({});
          }}
        >
          {owned.map((p) => (
            <option key={p.id} value={p.id}>
              {p.name}
            </option>
          ))}
        </select>
      </label>
      <p>
        {p.name}: {current?.label ?? formName(species)} · {p.selected_ability}
        {current?.secondary ? ` + ${current.secondary}` : ""}
        {current?.types.length ? ` · ${current.types.join(" / ")}` : ""}
      </p>
      {state.choices[`illusion:${p.id}`] && (
        <p>
          Illusion appearance:{" "}
          {battle.combatants.find(
            (t) => t.id === state.choices[`illusion:${p.id}`],
          )?.name ?? "party Pokémon"}
          . It breaks on damage.
        </p>
      )}
      {p.runtime.ability.flags.includes("Disguise") && (
        <p>Disguise has absorbed its first damaging hit this scene.</p>
      )}
      {state.choices[`hunger:${p.id}`] && (
        <p>Hunger Switch: {state.choices[`hunger:${p.id}`]} form.</p>
      )}
      <label>
        Reason or GM ruling
        <input
          value={reason}
          onChange={(e) => setReason(e.target.value)}
          placeholder={
            gm
              ? "Required for GM equipment and timing rulings"
              : "Optional explanation"
          }
        />
      </label>
      {options.length > 0 && (
        <details>
          <summary>Prepare form attributes and moves</summary>
          <label>
            Form
            <select
              value={chosen}
              onChange={(e) => {
                setForm(e.target.value);
                setAllocation({});
              }}
            >
              {options.map((id) => (
                <option key={id} value={id}>
                  {formName(id)}
                </option>
              ))}
            </select>
          </label>
          <div className="form-grid">
            {attributes.map((key) => (
              <label key={key}>
                {key} points{" "}
                <small>
                  Base {definition[chosen]?.stats[key]} · limit{" "}
                  {definition[chosen]?.limits[key]}
                </small>
                <input
                  type="number"
                  min={0}
                  max={14}
                  value={allocation[key] ?? 0}
                  onChange={(e) =>
                    setAllocation({
                      ...allocation,
                      [key]: Number(e.target.value),
                    })
                  }
                />
              </label>
            ))}
          </div>
          <details>
            <summary>Redistribute social and skill points</summary>
            {["Tough", "Cool", "Beauty", "Cute", "Clever"].map((key) => (
              <label key={key}>
                {key} points
                <input
                  type="number"
                  min={0}
                  max={6}
                  value={socials[key] ?? 0}
                  onChange={(e) =>
                    setSocials({ ...socials, [key]: Number(e.target.value) })
                  }
                />
              </label>
            ))}
            {[
              "Brawl",
              "Channel",
              "Clash",
              "Evasion",
              "Alert",
              "Athletic",
              "Nature",
              "Stealth",
              "Charm",
              "Etiquette",
              "Intimidate",
              "Perform",
              "Empathy",
              "Throw",
              "Weapons",
              "Crafts",
              "Lore",
              "Medicine",
              "Science",
            ].map((key) => (
              <label key={key}>
                {key}
                <input
                  type="number"
                  min={0}
                  max={5}
                  value={skills[key] ?? sheet.skills[key] ?? 0}
                  onChange={(e) =>
                    setSkills({
                      ...(sheet.skills as Record<string, number>),
                      ...skills,
                      [key]: Number(e.target.value),
                    })
                  }
                />
              </label>
            ))}
          </details>
          <label>
            Different Mega moves (comma separated, optional)
            <input value={moves} onChange={(e) => setMoves(e.target.value)} />
          </label>
          <button
            disabled={disabled}
            onClick={() =>
              void send(
                {
                  operation: "prepare",
                  combatant_id: p.id,
                  species_id: chosen,
                  build: {
                    attributes: allocation,
                    socials,
                    skills,
                    moves: moves
                      .split(",")
                      .map((s) => s.trim())
                      .filter(Boolean),
                  },
                },
                "Prepare this form's rank point distribution",
              )
            }
          >
            Save form build
          </button>
        </details>
      )}
      {mega.map((m) => (
        <button
          key={m.stone}
          disabled={disabled || !p.deployed || p.hp <= 0}
          onClick={() =>
            void send(
              { operation: "mega", combatant_id: p.id, stone: m.stone },
              "Mega Evolve during this Pokémon's turn",
            )
          }
        >
          Mega Evolve with {m.stone}
        </button>
      ))}
      {(p.selected_ability === "Imposter" ||
        (current?.sheet.moves ?? sheet.moves).includes("Transform")) && (
        <div>
          <label>
            Transform model
            <select value={target} onChange={(e) => setTarget(e.target.value)}>
              <option value="">Choose a visible Pokémon</option>
              {battle.combatants
                .filter((t) => t.id !== p.id && t.deployed && t.hp > 0)
                .map((t) => (
                  <option key={t.id} value={t.id}>
                    {t.name}
                  </option>
                ))}
            </select>
          </label>
          <button
            disabled={disabled || !target}
            onClick={() =>
              void send(
                { operation: "choose", combatant_id: p.id, target_id: target },
                "Choose the model for the next Transform attempt",
              )
            }
          >
            Choose Transform model
          </button>
        </div>
      )}
      {p.selected_ability === "Multitype" && (
        <div>
          <label>
            Multitype
            <select value={type} onChange={(e) => setType(e.target.value)}>
              {types.map((t) => (
                <option key={t}>{t}</option>
              ))}
            </select>
          </label>
          <button
            disabled={disabled}
            onClick={() =>
              void send(
                { operation: "type", combatant_id: p.id, type_name: type },
                "Change Arceus's type with Multitype",
              )
            }
          >
            Change type
          </button>
        </div>
      )}
      {gm && (
        <details>
          <summary>GM equipment, arrival and joint-form rulings</summary>
          <p>
            Key Stones, Discs and masks are attested here because catalog stock
            is incomplete. These decisions create no inventory items.
          </p>
          {trainers.map((t) => (
            <button
              key={t.id}
              disabled={disabledReason}
              onClick={() =>
                void send(
                  {
                    operation: "key_stone",
                    trainer_id: t.id,
                    present: !state.key_stones.includes(t.id),
                  },
                  reason,
                )
              }
            >
              {state.key_stones.includes(t.id) ? "Remove" : "Record"} Key Stone:{" "}
              {t.name}
            </button>
          ))}
          {p.selected_ability === "Schooling" && (
            <button
              disabled={disabledReason || !state.choices[`school:${p.id}`]}
              onClick={() =>
                void send(
                  { operation: "school_arrives", combatant_id: p.id },
                  reason,
                )
              }
            >
              Confirm called school arrives
            </button>
          )}
          {p.selected_ability === "RKS System" && (
            <div>
              <label>
                Verified Disc type
                <select value={type} onChange={(e) => setType(e.target.value)}>
                  {types.map((t) => (
                    <option key={t}>{t}</option>
                  ))}
                </select>
              </label>
              <button
                disabled={disabledReason}
                onClick={() =>
                  void send(
                    {
                      operation: "equipment",
                      combatant_id: p.id,
                      kind: "disc",
                      value: type,
                    },
                    reason,
                  )
                }
              >
                Record Disc equipment
              </button>
            </div>
          )}
          {p.selected_ability === "Embody Aspect" && (
            <div>
              <label>
                Verified mask
                <select value={mask} onChange={(e) => setMask(e.target.value)}>
                  {["Teal", "Wellspring", "Hearthflame", "Cornerstone"].map(
                    (t) => (
                      <option key={t}>{t}</option>
                    ),
                  )}
                </select>
              </label>
              <button
                disabled={disabledReason}
                onClick={() =>
                  void send(
                    {
                      operation: "equipment",
                      combatant_id: p.id,
                      kind: "mask",
                      value: mask,
                    },
                    reason,
                  )
                }
              >
                Record mask equipment
              </button>
            </div>
          )}
          {p.selected_ability === "As One" && (
            <div>
              <label>
                Allied component
                <select
                  value={target}
                  onChange={(e) => setTarget(e.target.value)}
                >
                  <option value="">Choose an ally</option>
                  {battle.combatants
                    .filter(
                      (t) =>
                        t.id !== p.id &&
                        t.side === p.side &&
                        t.deployed &&
                        t.hp > 0,
                    )
                    .map((t) => (
                      <option key={t.id} value={t.id}>
                        {t.name}
                      </option>
                    ))}
                </select>
              </label>
              <label>
                First component ability
                <input
                  value={firstAbility}
                  onChange={(e) => setFirstAbility(e.target.value)}
                  placeholder={p.selected_ability}
                />
              </label>
              <label>
                Second component ability
                <input
                  value={secondAbility}
                  onChange={(e) => setSecondAbility(e.target.value)}
                />
              </label>
              <button
                disabled={
                  disabledReason || !chosen || !target || !secondAbility
                }
                onClick={() =>
                  void send(
                    {
                      operation: "union",
                      combatant_id: p.id,
                      partner_id: target,
                      species_id: chosen,
                      first_ability: firstAbility || p.selected_ability,
                      second_ability: secondAbility,
                    },
                    reason,
                  )
                }
              >
                Join reviewed form with As One
              </button>
            </div>
          )}
          {campaign.form_memory?.[sheet.id] && (
            <button
              disabled={disabledReason || p.deployed}
              onClick={() =>
                void send(
                  { operation: "restore_core", sheet_id: sheet.id },
                  reason,
                )
              }
            >
              Confirm Minior release and later recapture
            </button>
          )}
          {battle
            .flow!.ability_events.filter(
              (e) => e.kind === "form-missile" && e.combatant_id === p.id,
            )
            .map((e) => (
              <div key={e.id}>
                <p>
                  Gulp Missile: the GM records its unspecified added effect in
                  the ruling above; the host rolls two damage dice.
                </p>
                <button
                  disabled={disabledReason}
                  onClick={() =>
                    void send(
                      {
                        operation: "missile",
                        combatant_id: p.id,
                        target_id: e.choices[0],
                        status: null,
                      },
                      reason,
                    )
                  }
                >
                  Resolve Gulp Missile retaliation
                </button>
              </div>
            ))}
        </details>
      )}
      <details>
        <summary>Form history</summary>
        {state.history.map((entry, i) => (
          <p key={i}>{entry}</p>
        ))}
      </details>
    </section>
  );
}
