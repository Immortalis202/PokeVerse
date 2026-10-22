import { useEffect, useState } from "react";
import type { Sheet } from "./bindings/Sheet";
import type { WorkflowRequest } from "./bindings/WorkflowRequest";
import type { WorkflowPreview } from "./bindings/WorkflowPreview";
import type { Catalog, Species, CatalogEntry } from "./catalog";
import { attributes, ranks } from "./catalog";
import { api, errorOf } from "./api";
import { newBuild } from "./sheets";
const steps = [
  "Identity",
  "Attributes",
  "Socials",
  "Skills",
  "Abilities",
  "Moves",
  "Review",
];
const operations = [
  "describe",
  "allocate",
  "rank_up",
  "learn",
  "forget",
  "retrain",
  "prepare_evolution",
  "evolve",
  "ability",
  "award",
  "bond",
  "resources",
  "inventory",
  "equip",
  "unequip",
  "exception",
];
const labels: Record<string, string> = {
  describe: "Name and notes",
  allocate: "Spend unallocated points",
  rank_up: "Rank up",
  learn: "Learn or replace a move",
  forget: "Forget a move / resolve evolution capacity",
  retrain: "Retrain allocations",
  prepare_evolution: "Prepare evolution",
  evolve: "Complete evolution",
  ability: "Select or request an ability",
  award: "Training Point award / correction",
  bond: "Happiness / Loyalty",
  resources: "Healing / Will / conditions",
  inventory: "Inventory grant / correction",
  equip: "Equip an owned item",
  unequip: "Put a held item away",
  exception: "GM-reviewed rebuild / rule exception",
};
const skills = [
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
];
export default function SheetEditor({
  initial,
  catalog,
  trainers,
  close,
  saved,
  draftRequest,
  previewRequest,
  captureSave,
}: {
  initial: Sheet;
  catalog: Catalog;
  trainers: Sheet[];
  close: () => void;
  saved: () => Promise<void>;
  draftRequest?: WorkflowRequest | null;
  previewRequest?: (request: WorkflowRequest) => Promise<WorkflowPreview>;
  captureSave?: (request: WorkflowRequest) => Promise<void>;
}) {
  const [request, setRequest] = useState<WorkflowRequest>(
    () =>
      draftRequest ?? {
        operation_id: crypto.randomUUID(),
        expected_revision: initial.revision,
        trainer_revision:
          trainers.find((t) => t.id === initial.trainer_id)?.revision ?? null,
        rules_version: 1,
        catalog_hash: catalog.hash,
        kind: initial.revision
          ? initial.workflow?.status === "strict"
            ? "describe"
            : "review"
          : "create",
        sheet: {
          ...structuredClone(initial),
          rank:
            !initial.revision && initial.kind === "pokemon"
              ? (trainers.find((t) => t.id === initial.trainer_id)?.rank ??
                initial.rank)
              : initial.rank,
          workflow: initial.workflow ?? newBuild(),
        },
        move_name: "",
        replacement: "",
        method: "ordinary",
        target_species: "",
        amount: 0,
        reason: initial.revision ? "Character update" : "Character created",
      },
  );
  const [step, setStep] = useState(0);
  const [preview, setPreview] = useState<WorkflowPreview | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [pending, setPending] = useState(false);
  const [extra, setExtra] = useState("");
  const [item, setItem] = useState("");
  const sheet = request.sheet;
  const build = sheet.workflow ?? newBuild();
  const [speciesDetail, setSpeciesDetail] = useState<Species | null>(null);
  const [abilityDetail, setAbilityDetail] = useState<CatalogEntry | null>(null);
  const species =
    speciesDetail?.AppId === sheet.species_id ? speciesDetail : null;
  useEffect(() => {
    let cancelled = false;
    setSpeciesDetail(null);
    if (sheet.species_id)
      api
        .catalogEntry<Species>("species", sheet.species_id)
        .then((value) => {
          if (!cancelled) setSpeciesDetail(value);
        })
        .catch((e) => {
          if (!cancelled) setError(errorOf(e).message);
        });
    return () => {
      cancelled = true;
    };
  }, [sheet.species_id]);
  useEffect(() => {
    let cancelled = false;
    setAbilityDetail(null);
    const entry = catalog.abilities.find((a) => a.Name === sheet.ability);
    if (entry)
      api
        .catalogEntry<CatalogEntry>("abilities", entry.AppId)
        .then((value) => {
          if (!cancelled) setAbilityDetail(value);
        })
        .catch((e) => {
          if (!cancelled) setError(errorOf(e).message);
        });
    return () => {
      cancelled = true;
    };
  }, [sheet.ability, catalog]);
  const creation = ["create", "review", "exception"].includes(request.kind);
  const allocation = creation || ["allocate", "retrain"].includes(request.kind);
  const change = (patch: Partial<WorkflowRequest>) =>
    setRequest((r) => ({ ...r, ...patch }));
  const edit = (patch: Partial<Sheet>) =>
    setRequest((r) => ({ ...r, sheet: { ...r.sheet, ...patch } }));
  const editBuild = (patch: Partial<typeof build>) =>
    edit({ workflow: { ...build, ...patch } });
  useEffect(() => {
    let cancelled = false;
    setPending(true);
    setPreview(null);
    const timer = setTimeout(
      () =>
        (previewRequest ?? api.previewWorkflow)(request)
          .then((p) => {
            if (!cancelled) {
              setPreview(p);
              setError("");
            }
          })
          .catch((e) => {
            if (!cancelled) setError(errorOf(e).message);
          })
          .finally(() => {
            if (!cancelled) setPending(false);
          }),
      250,
    );
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [request]);
  async function submit(draft: boolean) {
    setBusy(true);
    setError("");
    try {
      if (captureSave) await captureSave(request);
      else if (draft) await api.saveDraft(request);
      else await api.commitWorkflow(request);
      await saved();
      close();
    } catch (e) {
      setError(errorOf(e).message);
    } finally {
      setBusy(false);
    }
  }
  const number = (
    label: string,
    value: number,
    onChange: (n: number) => void,
    min = 0,
    max = 999,
  ) => (
    <label key={label}>
      {label}
      <input
        type="number"
        min={min}
        max={max}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
      />
    </label>
  );
  const visible = creation ? steps[step] : "Operation";
  return (
    <div className="overlay">
      <div
        className="modal editor"
        role="dialog"
        aria-label="Character workflow"
      >
        <div className="modal-head">
          <div>
            <span className="eyebrow">OFFLINE CREATION & PROGRESSION</span>
            <h2>{initial.revision ? sheet.name : `Create ${sheet.kind}`}</h2>
          </div>
          <button onClick={close}>Close</button>
        </div>
        {creation && (
          <div className="tabs">
            {steps.map((s, i) => (
              <button
                key={s}
                className={step === i ? "active" : ""}
                onClick={() => setStep(i)}
              >
                {i + 1}. {s}
              </button>
            ))}
          </div>
        )}
        <div className="editor-body">
          {error && (
            <div className="error" role="alert">
              {error}
            </div>
          )}
          {!!initial.revision && (
            <label>
              Operation
              <select
                value={request.kind}
                onChange={(e) => {
                  change({
                    kind: e.target.value,
                    operation_id: crypto.randomUUID(),
                    sheet: {
                      ...structuredClone(initial),
                      workflow: initial.workflow ?? newBuild(),
                    },
                    move_name: "",
                    replacement: "",
                    target_species: "",
                    amount: 0,
                  });
                  setStep(0);
                }}
              >
                {initial.workflow?.status !== "strict" && (
                  <option value="review">Review legacy sheet</option>
                )}
                {operations
                  .filter(
                    (k) =>
                      initial.kind === "pokemon" ||
                      !["equip", "unequip"].includes(k),
                  )
                  .map((k) => (
                    <option key={k} value={k}>
                      {k === "award" && initial.kind === "pokemon"
                        ? "Legacy Pok�mon balance correction (GM)"
                        : labels[k]}
                    </option>
                  ))}
              </select>
            </label>
          )}
          {(visible === "Identity" || request.kind === "describe") && (
            <div className="form-grid">
              <label>
                Name
                <input
                  value={sheet.name}
                  maxLength={200}
                  onChange={(e) => edit({ name: e.target.value })}
                />
              </label>
              {creation && (
                <>
                  {sheet.kind === "trainer" ? (
                    <label>
                      Age group
                      <select
                        value={build.age}
                        onChange={(e) => editBuild({ age: e.target.value })}
                      >
                        {["Child", "Teen", "Adult", "Senior"].map((a) => (
                          <option key={a}>{a}</option>
                        ))}
                      </select>
                    </label>
                  ) : (
                    <>
                      <label>
                        Trainer
                        <select
                          disabled={!!captureSave}
                          value={sheet.trainer_id ?? ""}
                          onChange={(e) => {
                            const t = trainers.find(
                              (t) => t.id === e.target.value,
                            );
                            change({
                              trainer_revision: t?.revision ?? null,
                              sheet: {
                                ...sheet,
                                trainer_id: t?.id ?? null,
                                rank:
                                  build.origin === "starter"
                                    ? (t?.rank ?? sheet.rank)
                                    : sheet.rank,
                              },
                            });
                          }}
                        >
                          <option value="">Select trainer</option>
                          {trainers.map((t) => (
                            <option key={t.id} value={t.id}>
                              {t.name} · {t.rank}
                            </option>
                          ))}
                        </select>
                      </label>
                      <label>
                        Origin
                        <select
                          disabled={!!captureSave}
                          value={build.origin}
                          onChange={(e) =>
                            editBuild({ origin: e.target.value })
                          }
                        >
                          {[
                            "starter",
                            "captured",
                            "narrative",
                            "opponent",
                            "legacy",
                          ].map((o) => (
                            <option key={o}>{o}</option>
                          ))}
                        </select>
                      </label>
                      <label>
                        Species / form
                        <select
                          disabled={!!captureSave}
                          value={sheet.species_id ?? ""}
                          onChange={(e) => {
                            const s = catalog.species.find(
                              (s) => s.AppId === e.target.value,
                            );
                            if (s)
                              edit({
                                species_id: s.AppId,
                                name: sheet.name || s.Name,
                                ability: s.Ability1,
                                workflow: {
                                  ...build,
                                  available_abilities: [
                                    s.Ability1,
                                    s.Ability2,
                                  ].filter(Boolean),
                                },
                              });
                          }}
                        >
                          <option value="">Select species</option>
                          {catalog.species.map((s) => (
                            <option key={s.AppId} value={s.AppId}>
                              {s.Name}
                            </option>
                          ))}
                        </select>
                      </label>
                    </>
                  )}
                  <label>
                    Rank
                    <select
                      value={sheet.rank}
                      disabled={
                        !!captureSave ||
                        (sheet.kind === "pokemon" && build.origin === "starter")
                      }
                      onChange={(e) => edit({ rank: e.target.value })}
                    >
                      {ranks.map((r) => (
                        <option key={r}>{r}</option>
                      ))}
                    </select>
                  </label>
                  <label>
                    Defense rules
                    <select
                      disabled={!!captureSave}
                      value={build.profile.defenses}
                      onChange={(e) =>
                        editBuild({
                          profile: {
                            ...build.profile,
                            defenses: e.target.value,
                          },
                        })
                      }
                    >
                      <option value="split">
                        Vitality DEF / Insight S.DEF
                      </option>
                      <option value="vitality">
                        Vitality for both defenses
                      </option>
                    </select>
                  </label>
                  <label>
                    HP rule
                    <select
                      disabled={!!captureSave}
                      value={build.profile.hp}
                      onChange={(e) =>
                        editBuild({
                          profile: { ...build.profile, hp: e.target.value },
                        })
                      }
                    >
                      <option value="vitality">Base HP + Vitality</option>
                      <option value="highest_limit">
                        Base HP + attribute with higher species limit
                      </option>
                    </select>
                  </label>
                </>
              )}
              <label>
                Nature
                <select
                  value={sheet.nature}
                  onChange={(e) => edit({ nature: e.target.value })}
                  disabled={!creation}
                >
                  <option value="">Select nature</option>
                  {catalog.natures.map((n) => (
                    <option key={n.Name}>{n.Name}</option>
                  ))}
                </select>
              </label>
              <label>
                Notes
                <textarea
                  value={sheet.notes}
                  maxLength={20000}
                  onChange={(e) => edit({ notes: e.target.value })}
                />
              </label>
            </div>
          )}
          {(visible === "Attributes" || (!creation && allocation)) && (
            <>
              <h3>Attribute allocations</h3>
              <p>
                Enter allocated points, not total scores. Species baselines and
                ability modifiers are calculated separately.
              </p>
              <div className="form-grid">
                {attributes
                  .filter((a) => sheet.kind === "pokemon" || a !== "Special")
                  .map((a) => (
                    <label key={a}>
                      {a}
                      <small>
                        Base{" "}
                        {preview?.baselines[a] ?? String(species?.[a] ?? 1)} ·
                        Modifier {preview?.modifiers[a] ?? 0} · Effective{" "}
                        {preview?.sheet.stats[a] ?? "—"}
                      </small>
                      <input
                        type="number"
                        min={0}
                        max={14}
                        value={build.allocations[a] ?? 0}
                        onChange={(e) =>
                          editBuild({
                            allocations: {
                              ...build.allocations,
                              [a]: Number(e.target.value),
                            },
                          })
                        }
                      />
                    </label>
                  ))}
              </div>
            </>
          )}
          {(visible === "Socials" || (!creation && allocation)) && (
            <>
              <h3>Social allocations</h3>
              <div className="form-grid">
                {["Tough", "Cool", "Beauty", "Cute", "Clever"].map((a) =>
                  number(
                    a,
                    build.socials[a] ?? 0,
                    (n) => editBuild({ socials: { ...build.socials, [a]: n } }),
                    0,
                    6,
                  ),
                )}
              </div>
            </>
          )}
          {(visible === "Skills" || (!creation && allocation)) && (
            <>
              <h3>Skills</h3>
              <div className="form-grid">
                {[...new Set([...skills, ...Object.keys(sheet.skills)])]
                  .filter(
                    (k) =>
                      sheet.kind === "trainer" ||
                      !["Empathy", "Throw", "Weapons"].includes(k),
                  )
                  .map((k) =>
                    number(
                      k,
                      sheet.skills[k] ?? 0,
                      (n) => edit({ skills: { ...sheet.skills, [k]: n } }),
                      0,
                      5,
                    ),
                  )}
              </div>
              <div className="inline">
                <input
                  placeholder="Extra skill name (GM authorization)"
                  value={extra}
                  onChange={(e) => setExtra(e.target.value)}
                />
                <button
                  onClick={() => {
                    if (extra.trim()) {
                      edit({ skills: { ...sheet.skills, [extra.trim()]: 0 } });
                      setExtra("");
                    }
                  }}
                >
                  Add extra skill
                </button>
              </div>
            </>
          )}
          {(visible === "Abilities" || request.kind === "ability") &&
            sheet.kind === "pokemon" && (
              <>
                <h3>Abilities</h3>
                <p>
                  Ordinary abilities are available outside battle. Choose one
                  for combat. Hidden and exceptional abilities require GM
                  acceptance.
                </p>
                <label>
                  Selected battle ability
                  <select
                    disabled={!!captureSave}
                    value={sheet.ability}
                    onChange={(e) => edit({ ability: e.target.value })}
                  >
                    <option value="">Select ability</option>
                    {[
                      ...new Set([
                        ...build.available_abilities,
                        species?.Ability1,
                        species?.Ability2,
                        species?.HiddenAbility,
                        ...catalog.abilities.map((a) => a.Name),
                      ]),
                    ]
                      .filter((a): a is string => !!a)
                      .map((a) => (
                        <option key={a}>{a}</option>
                      ))}
                  </select>
                </label>
                <p>{abilityDetail?.Effect}</p>
                {creation && !captureSave && (
                  <button
                    onClick={() =>
                      editBuild({
                        available_abilities: [
                          ...new Set([
                            ...build.available_abilities,
                            sheet.ability,
                          ]),
                        ].filter(Boolean),
                      })
                    }
                  >
                    Include selected exceptional ability in request
                  </button>
                )}
              </>
            )}
          {visible === "Moves" && sheet.kind === "pokemon" && (
            <>
              <h3>Starting moves</h3>
              <p>
                Eligibility and capacity are checked by Rust. Higher-rank moves
                remain visible with their requirements.
              </p>
              {(species?.Moves ?? []).map((m) => (
                <label className="checkbox" key={m.Name}>
                  <input
                    type="checkbox"
                    checked={sheet.moves.includes(m.Name)}
                    onChange={(e) =>
                      edit({
                        moves: e.target.checked
                          ? [...sheet.moves, m.Name]
                          : sheet.moves.filter((n) => n !== m.Name),
                      })
                    }
                  />
                  {m.Name} · {m.Learned}
                </label>
              ))}
            </>
          )}
          {request.kind === "learn" && (
            <div className="form-grid">
              <label>
                Learning method
                <select
                  value={request.method}
                  onChange={(e) => change({ method: e.target.value })}
                >
                  {[
                    "ordinary",
                    "pre_evolution",
                    "tm",
                    "tr",
                    "overrank",
                    "tutor",
                  ].map((m) => (
                    <option key={m}>{m}</option>
                  ))}
                </select>
              </label>
              <label>
                Move
                <select
                  value={request.move_name}
                  onChange={(e) => change({ move_name: e.target.value })}
                >
                  <option value="">Select move</option>
                  {catalog.moves.map((m) => (
                    <option key={m.AppId} value={m.Name}>
                      {m.Name} ·{" "}
                      {species?.Moves.find((n) => n.Name === m.Name)?.Learned ??
                        "special method required"}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                Forget / replace
                <select
                  value={request.replacement}
                  onChange={(e) => change({ replacement: e.target.value })}
                >
                  <option value="">No replacement</option>
                  {sheet.moves.map((m) => (
                    <option key={m}>{m}</option>
                  ))}
                </select>
              </label>
              {["tm", "tr"].includes(request.method) && (
                <label>
                  Owned TM/TR item
                  <select
                    value={request.target_species}
                    onChange={(e) => change({ target_species: e.target.value })}
                  >
                    <option value="">Select item</option>
                    {Object.entries(sheet.inventory)
                      .filter(([, v]) => (v ?? 0) > 0)
                      .map(([i]) => (
                        <option key={i}>{i}</option>
                      ))}
                  </select>
                </label>
              )}
              {(request.method === "tutor" || !species?.Creation?.stage) &&
                number("Proposed tutor cost", request.amount, (n) =>
                  change({ amount: n }),
                )}
            </div>
          )}
          {request.kind === "forget" && (
            <label>
              Move to forget
              <select
                value={request.move_name}
                onChange={(e) => change({ move_name: e.target.value })}
              >
                <option value="">Select learned move</option>
                {sheet.moves.map((m) => (
                  <option key={m}>{m}</option>
                ))}
              </select>
              <small>
                Recorded in forgotten moves. Learning it again uses the
                applicable cost.
              </small>
            </label>
          )}
          {["prepare_evolution", "evolve"].includes(request.kind) && (
            <>
              <label>
                Direct evolution
                <select
                  value={request.target_species}
                  onChange={(e) => change({ target_species: e.target.value })}
                >
                  <option value="">Select evolution</option>
                  {(species?.Evolutions ?? [])
                    .filter((e) => e.To)
                    .map((e) => catalog.species.find((s) => s.Name === e.To))
                    .filter((s): s is NonNullable<typeof s> => !!s)
                    .map((s) => (
                      <option key={s.AppId} value={s.AppId}>
                        {s.Name}
                      </option>
                    ))}
                </select>
              </label>
              <p>
                Prepared target: {build.evolution_ready ?? "none"}. Completion
                requires GM confirmation of the scene.
              </p>
              {number(
                "Proposed cost for missing narrative metadata",
                request.amount,
                (n) => change({ amount: n }),
              )}
            </>
          )}
          {request.kind === "award" &&
            number(
              "Training Point award / correction",
              request.amount,
              (n) => change({ amount: n }),
              -100000,
              100000,
            )}
          {request.kind === "bond" && (
            <div className="form-grid">
              {number(
                "Happiness",
                build.happiness,
                (n) => editBuild({ happiness: n }),
                0,
                5,
              )}
              {number(
                "Loyalty",
                build.loyalty,
                (n) => editBuild({ loyalty: n }),
                0,
                5,
              )}
            </div>
          )}
          {request.kind === "resources" && (
            <div className="form-grid">
              {number("Current HP", sheet.hp, (n) => edit({ hp: n }))}
              {number("Current Will", build.will, (n) =>
                editBuild({ will: n }),
              )}
              <label>
                Conditions
                <input
                  value={sheet.conditions.join(", ")}
                  onChange={(e) =>
                    edit({
                      conditions: e.target.value
                        .split(",")
                        .map((c) => c.trim())
                        .filter(Boolean),
                    })
                  }
                />
              </label>
            </div>
          )}
          {["equip", "unequip"].includes(request.kind) && (
            <>
              <p>
                Prepare equipment between battles. Battle item effects are not
                active in this build.
              </p>
              <label>
                {request.kind === "equip"
                  ? "Owned item to equip"
                  : "Held item to put away"}
                <select
                  value={request.move_name}
                  onChange={(e) => change({ move_name: e.target.value })}
                >
                  <option value="">Choose an item</option>
                  {request.kind === "equip" ? (
                    <>
                      {Object.entries(sheet.inventory)
                        .filter(([, quantity]) => (quantity ?? 0) > 0)
                        .map(([name, quantity]) => (
                          <option key={`bulk:${name}`} value={name}>
                            {name} ({quantity} available)
                          </option>
                        ))}
                      {(sheet.stored_items ?? []).map((unit) => (
                        <option key={unit.id} value={unit.id}>
                          {unit.name} — stored
                          {unit.uses > 0 ? `, recorded usage ${unit.uses}` : ""}
                        </option>
                      ))}
                    </>
                  ) : (
                    (sheet.held_items ?? []).map((unit) => (
                      <option key={unit.id} value={unit.id}>
                        {unit.name}
                        {unit.uses > 0 ? ` — recorded usage ${unit.uses}` : ""}
                      </option>
                    ))
                  )}
                </select>
              </label>
              <p>
                Currently held:{" "}
                {(sheet.held_items ?? []).map((unit) => unit.name).join(", ") ||
                  "None"}
                .
              </p>
            </>
          )}
          {request.kind === "inventory" && (
            <>
              <div className="inline">
                <select value={item} onChange={(e) => setItem(e.target.value)}>
                  <option value="">Select item</option>
                  {catalog.items.map((i) => (
                    <option key={i.Name}>{i.Name}</option>
                  ))}
                </select>
                <button
                  onClick={() => {
                    if (item)
                      edit({
                        inventory: {
                          ...sheet.inventory,
                          [item]: (sheet.inventory[item] ?? 0) + 1,
                        },
                      });
                  }}
                >
                  Add
                </button>
              </div>
              <div className="form-grid">
                {Object.entries(sheet.inventory).map(([i, v]) =>
                  number(
                    i,
                    v ?? 0,
                    (n) => edit({ inventory: { ...sheet.inventory, [i]: n } }),
                    0,
                    100000,
                  ),
                )}
              </div>
            </>
          )}
          <label>
            Reason / narrative source
            <input
              maxLength={1000}
              value={request.reason}
              onChange={(e) => change({ reason: e.target.value })}
            />
          </label>
          <div className="workflow-review">
            <h3>{pending ? "Calculating…" : "Rules preview"}</h3>
            {preview && (
              <>
                <div className="counter-row">
                  {Object.entries(preview.remaining).map(([k, v]) => (
                    <span key={k}>
                      {k} remaining <b>{v}</b>
                    </span>
                  ))}
                </div>
                <p>
                  Cost: <strong>{preview.cost} TP</strong> · Remaining TP:{" "}
                  <strong>
                    {preview.sheet.workflow?.training_points ?? 0}
                  </strong>{" "}
                  · HP {preview.sheet.hp}/{preview.sheet.max_hp} · Will{" "}
                  {preview.sheet.workflow?.will}/
                  {preview.sheet.workflow?.max_will}
                </p>
                <p>
                  {Object.entries(preview.derived)
                    .map(([k, v]) => `${k}: ${v}`)
                    .join(" · ")}
                </p>
                {!!preview.errors.length && (
                  <div className="error" role="alert">
                    {preview.errors.map((e, i) => (
                      <p key={i}>{e}</p>
                    ))}
                  </div>
                )}
                {!!preview.approvals.length && (
                  <div className="notice">
                    <strong>Draft requires GM acceptance</strong>
                    {preview.approvals.map((e, i) => (
                      <p key={i}>{e}</p>
                    ))}
                  </div>
                )}
                {preview.warnings.map((e, i) => (
                  <p className="muted" key={i}>
                    {e}
                  </p>
                ))}
              </>
            )}
          </div>
        </div>
        <div className="modal-foot inline between">
          {creation && (
            <div className="inline">
              <button disabled={step === 0} onClick={() => setStep(step - 1)}>
                Back
              </button>
              <button
                disabled={step === steps.length - 1}
                onClick={() => setStep(step + 1)}
              >
                Next
              </button>
            </div>
          )}
          <div className="inline">
            <button disabled={busy} onClick={() => submit(true)}>
              {captureSave ? "Keep capture edits" : "Save draft"}
            </button>
            <button
              className="primary"
              disabled={
                busy ||
                pending ||
                !preview ||
                !!preview.errors.length ||
                (!captureSave && !!preview.approvals.length) ||
                (creation && step !== 6)
              }
              onClick={() => submit(false)}
            >
              {captureSave
                ? "Keep validated capture"
                : "Commit verified operation"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
