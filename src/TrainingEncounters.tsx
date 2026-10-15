import { useEffect, useState } from "react";
import { api, errorOf } from "./api";
import { ranks, type Catalog, type Species } from "./catalog";
import { sheetValues } from "./sheets";
import SheetEditor from "./SheetEditor";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";
import type { TrainingEncounter } from "./bindings/TrainingEncounter";
import type { CapturePreview } from "./bindings/CapturePreview";
import type { WorkflowRequest } from "./bindings/WorkflowRequest";
import type { WorkflowPreview } from "./bindings/WorkflowPreview";
type Habitat = {
  habitat_id: string;
  habitat: string;
  description: string;
  pokemon_count: number;
};
export default function TrainingEncounters({
  campaign,
  playerId,
  catalog,
  execute,
  connected,
  busy,
  pull,
}: {
  campaign: Campaign | null;
  playerId: string;
  catalog: Catalog;
  execute: (a: Action) => Promise<boolean>;
  connected: boolean;
  busy: boolean;
  pull: (ids: string[]) => Promise<void>;
}) {
  const [habitats, setHabitats] = useState<Habitat[]>([]),
    [habitat, setHabitat] = useState("temperate_forest");
  const [trainer, setTrainer] = useState(""),
    [rank, setRank] = useState("Rookie"),
    [size, setSize] = useState(5),
    [seed, setSeed] = useState<string>(crypto.randomUUID());
  const [draft, setDraft] = useState<TrainingEncounter | null>(null),
    [saved, setSaved] = useState<TrainingEncounter[]>([]),
    [error, setError] = useState(""),
    [working, setWorking] = useState(false);
  const [query, setQuery] = useState(""),
    [offset, setOffset] = useState(0),
    [pool, setPool] = useState<Awaited<ReturnType<typeof api.encounterPool>>>({
      entries: [],
      total: 0,
    });
  const [captures, setCaptures] = useState<Record<string, CapturePreview>>({}),
    [reward, setReward] = useState<Record<string, number>>({}),
    [reason, setReason] = useState(""),
    [review, setReview] = useState(false),
    [editor, setEditor] = useState<string | null>(null);
  const [awards, setAwards] = useState<
    { request: WorkflowRequest; preview: WorkflowPreview }[]
  >([]);
  const [details, setDetails] = useState<Record<string, Species>>({});
  const gm = campaign?.gm_id === playerId,
    trainers = campaign
      ? sheetValues(campaign.sheets).filter(
          (s) => s.kind === "trainer" && !s.archived,
        )
      : [];
  const published =
    campaign?.training_encounters.filter((e) => e.status === "published") ?? [];
  const encounter = draft ?? published[published.length - 1] ?? null;
  const pending = busy || working;
  async function run(f: () => Promise<void>) {
    setWorking(true);
    setError("");
    try {
      await f();
    } catch (e) {
      setError(errorOf(e).message);
    } finally {
      setWorking(false);
    }
  }
  useEffect(() => {
    let cancelled = false;
    api
      .encounterHabitats()
      .then((v) => {
        if (!cancelled) setHabitats(v);
      })
      .catch((e) => {
        if (!cancelled) setError(errorOf(e).message);
      });
    return () => {
      cancelled = true;
    };
  }, [catalog.hash]);
  useEffect(() => {
    let cancelled = false;
    if (gm)
      api
        .encounterDrafts()
        .then((v) => {
          if (!cancelled)
            setSaved(v.filter((e) => e.campaign_id === campaign?.id));
        })
        .catch((e) => {
          if (!cancelled) setError(errorOf(e).message);
        });
    return () => {
      cancelled = true;
    };
  }, [campaign?.id, gm]);
  useEffect(() => {
    setReview(false);
    setAwards([]);
    if (draft?.status === "published") {
      const latest = campaign?.training_encounters.find(
        (e) => e.id === draft.id,
      );
      if (
        !latest ||
        latest.status !== "published" ||
        latest.revision !== draft.revision
      ) {
        setCaptures({});
        setDraft(
          latest?.status === "published" ? structuredClone(latest) : null,
        );
      }
    }
  }, [campaign?.revision]);
  useEffect(() => {
    setDraft(null);
    setCaptures({});
    setReward({});
    setReview(false);
    setTrainer("");
  }, [campaign?.id]);
  useEffect(() => {
    if (trainers[0]) setRank(trainers[0].rank);
  }, [campaign?.id, trainers[0]?.id]);
  useEffect(() => {
    let cancelled = false;
    if (gm)
      api
        .encounterPool(encounter?.habitat_id ?? habitat, query, offset)
        .then((v) => {
          if (!cancelled) setPool(v);
        })
        .catch((e) => {
          if (!cancelled) setError(errorOf(e).message);
        });
    return () => {
      cancelled = true;
    };
  }, [habitat, encounter?.habitat_id, query, offset, gm]);
  useEffect(() => {
    let cancelled = false;
    const ids = [
      ...new Set(encounter?.individuals.map((p) => p.species_id) ?? []),
    ];
    Promise.all(
      ids.map(
        async (id) =>
          [id, await api.catalogEntry<Species>("species", id)] as const,
      ),
    )
      .then((v) => {
        if (!cancelled)
          setDetails(
            Object.fromEntries(
              v.filter((x): x is readonly [string, Species] => !!x[1]),
            ),
          );
      })
      .catch((e) => {
        if (!cancelled) setError(errorOf(e).message);
      });
    return () => {
      cancelled = true;
    };
  }, [encounter]);
  if (!campaign)
    return (
      <div className="empty large">
        <h2>Training encounters</h2>
        <p>
          Select a campaign in the Room page. Private preparation works offline;
          reveal and resolution require a room connection.
        </p>
      </div>
    );
  const c = campaign,
    activeTrainer = trainer || trainers[0]?.id || "",
    canResolve =
      !!encounter &&
      published.some(
        (e) => e.id === encounter.id && e.revision === encounter.revision,
      ) &&
      encounter.status === "published";
  const adoption = c.catalog_hash !== catalog.hash;
  function change(e: TrainingEncounter) {
    setDraft(e);
    setCaptures({});
    setReview(false);
  }
  async function generate(newSeed = seed) {
    const e = await api.generateEncounter(
      c.id,
      activeTrainer,
      habitat,
      rank,
      size,
      newSeed,
    );
    change(e);
    setSeed(newSeed);
  }
  async function save() {
    if (!encounter) return;
    await api.saveEncounterDraft(c, encounter);
    setSaved(await api.encounterDrafts());
  }
  function editIndividual(
    id: string,
    patch: Partial<TrainingEncounter["individuals"][number]>,
  ) {
    if (!encounter) return;
    change({
      ...encounter,
      status: "draft",
      individuals: encounter.individuals.map((p) =>
        p.id === id ? { ...p, ...patch } : p,
      ),
    });
  }
  async function replace(id: string, speciesId?: string) {
    if (!encounter) return;
    const old = encounter.individuals.find((p) => p.id === id)!;
    const fresh = await api.generateEncounter(
      c.id,
      encounter.trainer_id,
      encounter.habitat_id,
      old.rank,
      1,
      crypto.randomUUID(),
    );
    const p = fresh.individuals[0];
    if (speciesId) {
      const sp = await api.catalogEntry<Species>("species", speciesId);
      if (!sp) throw Error("Species unavailable");
      p.species_id = speciesId;
      p.ability =
        [sp.Ability1, sp.Ability2].find((a) =>
          catalog.abilities.some((x) => x.Name === a),
        ) ?? "";
      p.conditions =
        pool.entries.find((s) => s.AppId === speciesId)?.habitat_provenance
          .conditions ?? "";
    }
    editIndividual(id, { ...p, id: old.id });
  }
  async function selectCapture(id: string, trainerId: string) {
    setReview(false);
    if (!encounter) return;
    if (!trainerId) {
      setCaptures((v) => {
        const next = { ...v };
        delete next[id];
        return next;
      });
      return;
    }
    const preview = await api.previewCapture(c, encounter, id, trainerId);
    setCaptures((v) => ({ ...v, [id]: preview }));
  }
  async function prepareReview() {
    if (!encounter) return;
    const fresh: Record<string, CapturePreview> = {};
    for (const [individual, v] of Object.entries(captures)) {
      const p = await api.previewCapture(
        c,
        encounter,
        individual,
        v.request.sheet.trainer_id ?? "",
        v.request,
      );
      if (p.preview.errors.length) throw Error(p.preview.errors.join("; "));
      fresh[individual] = p;
    }
    const planned: typeof awards = [];
    for (const [id, amount] of Object.entries(reward)) {
      if (!Number.isInteger(amount) || amount < 0)
        throw Error("Rewards must be nonnegative whole numbers");
      if (!amount) continue;
      const sheet = c.sheets[id];
      if (!sheet) throw Error("Reward recipient changed");
      const request: WorkflowRequest = {
        operation_id: crypto.randomUUID(),
        expected_revision: sheet.revision,
        trainer_revision: sheet.trainer_id
          ? (c.sheets[sheet.trainer_id]?.revision ?? null)
          : null,
        rules_version: 1,
        catalog_hash: catalog.hash,
        kind: "award",
        sheet,
        move_name: "",
        replacement: "",
        method: "",
        target_species: "",
        amount,
        reason: `Training encounter ${encounter.id}: ${reason}`,
      };
      const preview = await api.previewCampaignWorkflow(c, request);
      if (preview.errors.length) throw Error(preview.errors.join("; "));
      planned.push({ request, preview });
    }
    setCaptures(fresh);
    setAwards(planned);
    setReview(true);
  }
  async function confirm() {
    if (!encounter) return;
    const ok = await execute({
      type: "resolve_training",
      resolution: {
        encounter_id: encounter.id,
        encounter_revision: encounter.revision,
        captures: Object.entries(captures).map(([individual_id, v]) => ({
          individual_id,
          request: v.request,
        })),
        rewards: awards.map((a) => a.request),
        reason,
      },
    });
    if (ok) {
      await api.deleteEncounterDraft(encounter.id);
      setSaved((v) => v.filter((e) => e.id !== encounter.id));
      setDraft(null);
      setCaptures({});
      setReward({});
      setReview(false);
      setReason("");
    }
  }
  const selected = Object.values(captures),
    errors = selected.flatMap((v) => v.preview.errors);
  const history = c.training_encounters.filter((e) => e.status !== "published");
  return (
    <>
      <div className="page-head">
        <div>
          <span className="eyebrow">
            {c.name} · {gm ? "GAME MASTER" : "PLAYER"}
          </span>
          <h1>Training encounters</h1>
          <p>
            Victory is assumed. The GM confirms captures and Training Point
            rewards.
          </p>
        </div>
      </div>
      {error && (
        <div className="error" role="alert">
          {error}
        </div>
      )}
      {adoption && (
        <div className="notice">
          This campaign uses an earlier catalog. Historical records are
          preserved.
          <button
            disabled={!gm || !connected || pending}
            onClick={() =>
              run(async () => {
                await execute({ type: "adopt_catalog" });
              })
            }
          >
            Adopt installed habitat catalog
          </button>
        </div>
      )}
      {gm && (
        <div className="panel">
          <h2>Prepare privately</h2>
          <div className="form-grid">
            <label>
              Primary trainer
              <select
                value={activeTrainer}
                onChange={(e) => {
                  setTrainer(e.target.value);
                  setRank(c.sheets[e.target.value]?.rank ?? "Rookie");
                }}
              >
                {trainers.map((t) => (
                  <option key={t.id} value={t.id}>
                    {t.name} · {t.rank}
                  </option>
                ))}
              </select>
            </label>
            <label>
              Habitat
              <select
                value={habitat}
                onChange={(e) => {
                  setHabitat(e.target.value);
                  setOffset(0);
                }}
              >
                {habitats.map((h) => (
                  <option key={h.habitat_id} value={h.habitat_id}>
                    {h.habitat} ({h.pokemon_count})
                  </option>
                ))}
              </select>
            </label>
            <label>
              Rank
              <select value={rank} onChange={(e) => setRank(e.target.value)}>
                {ranks.map((r) => (
                  <option key={r}>{r}</option>
                ))}
              </select>
            </label>
            <label>
              Horde size
              <input
                type="number"
                min={1}
                max={50}
                value={size}
                onChange={(e) => setSize(Number(e.target.value))}
              />
            </label>
            <label>
              Generation seed
              <input
                maxLength={100}
                value={seed}
                onChange={(e) => setSeed(e.target.value)}
              />
            </label>
          </div>
          <p>{habitats.find((h) => h.habitat_id === habitat)?.description}</p>
          <div className="inline">
            <button
              disabled={pending || !activeTrainer}
              onClick={() => run(() => generate())}
            >
              Generate horde
            </button>
            <button
              disabled={pending || !activeTrainer}
              onClick={() => run(() => generate(crypto.randomUUID()))}
            >
              Generate with new seed
            </button>
          </div>
          {saved
            .filter((e) => e.campaign_id === c.id)
            .map((e) => (
              <div className="list-row" key={e.id}>
                <span>
                  Private draft ·{" "}
                  {habitats.find((h) => h.habitat_id === e.habitat_id)
                    ?.habitat ?? e.habitat_id}{" "}
                  · {e.individuals.length} Pokémon
                </span>
                <button
                  onClick={() => {
                    change(structuredClone(e));
                    setHabitat(e.habitat_id);
                    setTrainer(e.trainer_id);
                  }}
                >
                  Open
                </button>
                <button
                  onClick={() =>
                    run(async () => {
                      await api.deleteEncounterDraft(e.id);
                      setSaved((v) => v.filter((x) => x.id !== e.id));
                    })
                  }
                >
                  Delete draft
                </button>
              </div>
            ))}
        </div>
      )}
      {published.length > 0 && (
        <div className="panel">
          <h3>Published encounters</h3>
          {published.map((e) => (
            <button
              key={e.id}
              onClick={() => {
                setDraft(structuredClone(e));
                setCaptures({});
                setReview(false);
              }}
            >
              {habitats.find((h) => h.habitat_id === e.habitat_id)?.habitat ??
                e.habitat_id}{" "}
              · {e.individuals.length} Pokémon
            </button>
          ))}
        </div>
      )}
      {encounter && (
        <div className="panel">
          <h2>
            {encounter.status === "draft"
              ? "Private preparation"
              : "Revealed horde"}
          </h2>
          <p>
            {habitats.find((h) => h.habitat_id === encounter.habitat_id)
              ?.habitat ?? encounter.habitat_id}{" "}
            · {encounter.individuals.length} individuals
          </p>
          {gm && (
            <div className="inline">
              <button disabled={pending} onClick={() => run(save)}>
                Save privately
              </button>
              <button
                disabled={pending || !connected || adoption}
                onClick={() =>
                  run(async () => {
                    if (
                      await execute({ type: "publish_training", encounter })
                    ) {
                      setDraft(null);
                      setCaptures({});
                      setReview(false);
                    }
                  })
                }
              >
                {encounter.revision
                  ? "Publish roster update"
                  : "Reveal to room"}
              </button>
              <button
                disabled={pending || encounter.individuals.length >= 50}
                onClick={() =>
                  run(async () => {
                    const e = await api.generateEncounter(
                      c.id,
                      encounter.trainer_id,
                      encounter.habitat_id,
                      rank,
                      1,
                      crypto.randomUUID(),
                    );
                    change({
                      ...encounter,
                      status: "draft",
                      individuals: [...encounter.individuals, e.individuals[0]],
                    });
                  })
                }
              >
                Add individual
              </button>
            </div>
          )}
          {gm && (
            <div className="inline">
              <input
                aria-label="Search habitat species"
                placeholder="Search replacements within this habitat…"
                value={query}
                onChange={(e) => {
                  setQuery(e.target.value);
                  setOffset(0);
                }}
              />
              <button
                disabled={!offset}
                onClick={() => setOffset(Math.max(0, offset - 150))}
              >
                Previous species
              </button>
              <span>
                {offset + 1}–{Math.min(offset + 150, pool.total)} of{" "}
                {pool.total}
              </span>
              <button
                disabled={offset + 150 >= pool.total}
                onClick={() => setOffset(offset + 150)}
              >
                Next species
              </button>
            </div>
          )}
          <div className="training-grid">
            {encounter.individuals.map((p, i) => {
              const sp =
                details[p.species_id] ??
                catalog.species.find((s) => s.AppId === p.species_id);
              const ordinary = details[p.species_id]
                ? [
                    details[p.species_id].Ability1,
                    details[p.species_id].Ability2,
                  ].filter(
                    (a, j, all) =>
                      a &&
                      all.indexOf(a) === j &&
                      catalog.abilities.some((x) => x.Name === a),
                  )
                : [];
              const capture = captures[p.id];
              return (
                <div className="panel" key={p.id}>
                  <div className="inline">
                    <img
                      className="training-sprite"
                      src={`/sprites/${sp?.Image}`}
                      alt=""
                    />
                    <h3>
                      {i + 1}. {sp?.Name ?? p.species_id}
                    </h3>
                  </div>
                  <p>
                    {p.rank}
                    {gm
                      ? ` · ${p.ability || "Ability requires catalog adjudication"}`
                      : ""}
                  </p>
                  {gm &&
                    (details[p.species_id]?.Creation?.unresolved_fields
                      ?.length ?? 0) > 0 && (
                      <p className="notice">
                        Species metadata needs GM adjudication before capture
                        confirmation.
                      </p>
                    )}
                  {p.conditions && (
                    <p className="notice">Habitat condition: {p.conditions}</p>
                  )}
                  {gm && (
                    <>
                      <div className="form-grid">
                        <label>
                          Rank
                          <select
                            value={p.rank}
                            onChange={(e) =>
                              editIndividual(p.id, { rank: e.target.value })
                            }
                          >
                            {ranks.map((r) => (
                              <option key={r}>{r}</option>
                            ))}
                          </select>
                        </label>
                        <label>
                          Ordinary ability
                          <select
                            value={p.ability}
                            onChange={(e) =>
                              editIndividual(p.id, { ability: e.target.value })
                            }
                          >
                            <option value="">Unresolved</option>
                            {ordinary.map((a) => (
                              <option key={a}>{a}</option>
                            ))}
                          </select>
                        </label>
                        <label>
                          Replace species
                          <select
                            value=""
                            onChange={(e) =>
                              run(() => replace(p.id, e.target.value))
                            }
                          >
                            <option value="">Choose from pool…</option>
                            {pool.entries.map((s) => (
                              <option key={s.AppId} value={s.AppId}>
                                {s.Name}
                              </option>
                            ))}
                          </select>
                        </label>
                      </div>
                      <div className="inline">
                        <button
                          disabled={pending}
                          onClick={() => run(() => replace(p.id))}
                        >
                          Reroll individual
                        </button>
                        <button
                          disabled={
                            pending || encounter.individuals.length === 1
                          }
                          onClick={() =>
                            change({
                              ...encounter,
                              status: "draft",
                              individuals: encounter.individuals.filter(
                                (x) => x.id !== p.id,
                              ),
                            })
                          }
                        >
                          Remove
                        </button>
                      </div>
                      {canResolve && (
                        <>
                          <label>
                            Capture for
                            <select
                              disabled={pending}
                              value={capture?.request.sheet.trainer_id ?? ""}
                              onChange={(e) =>
                                run(() => selectCapture(p.id, e.target.value))
                              }
                            >
                              <option value="">Not captured</option>
                              {trainers.map((t) => (
                                <option key={t.id} value={t.id}>
                                  {t.name}
                                </option>
                              ))}
                            </select>
                          </label>
                          {capture && (
                            <>
                              <p>
                                HP {capture.preview.sheet.max_hp} ·{" "}
                                {Object.entries(capture.preview.sheet.stats)
                                  .map(([k, v]) => `${k} ${v}`)
                                  .join(" · ")}
                              </p>
                              <p>
                                Moves:{" "}
                                {capture.preview.sheet.moves.join(", ") ||
                                  "None"}
                              </p>
                              {[
                                ...capture.preview.errors,
                                ...capture.preview.approvals,
                                ...capture.preview.warnings,
                              ].map((m, j) => (
                                <p
                                  key={j}
                                  className={
                                    capture.preview.errors.includes(m)
                                      ? "error"
                                      : "notice"
                                  }
                                >
                                  {m}
                                </p>
                              ))}
                              <button onClick={() => setEditor(p.id)}>
                                Adjust captured sheet
                              </button>
                              <button
                                disabled={pending}
                                onClick={() =>
                                  run(async () => {
                                    const edited = {
                                      ...capture.request,
                                      trainer_revision:
                                        c.sheets[
                                          capture.request.sheet.trainer_id!
                                        ]?.revision ?? null,
                                    };
                                    const v = await api.previewCapture(
                                      c,
                                      encounter,
                                      p.id,
                                      edited.sheet.trainer_id!,
                                      edited,
                                    );
                                    setCaptures((x) => ({ ...x, [p.id]: v }));
                                    setReview(false);
                                  })
                                }
                              >
                                Refresh preview
                              </button>
                            </>
                          )}
                        </>
                      )}
                    </>
                  )}
                </div>
              );
            })}
          </div>
          {gm && canResolve && (
            <>
              <h3>Training Point rewards</h3>
              <p>
                Choose existing participating Pokémon. Awards are manual and
                recorded against this encounter.
              </p>
              {sheetValues(c.sheets)
                .filter((s) => s.kind === "trainer" && !s.archived)
                .map((s) => (
                  <label className="list-row" key={s.id}>
                    {s.name} · {c.sheets[s.trainer_id ?? ""]?.name}
                    <input
                      aria-label={`Training Points for ${s.name}`}
                      type="number"
                      min={0}
                      max={100000}
                      value={reward[s.id] ?? 0}
                      onChange={(e) => {
                        setReward((v) => ({
                          ...v,
                          [s.id]: Number(e.target.value),
                        }));
                        setReview(false);
                      }}
                    />
                  </label>
                ))}
              <label>
                GM outcome / adjudication reason
                <input
                  maxLength={850}
                  value={reason}
                  onChange={(e) => {
                    setReason(e.target.value);
                    setReview(false);
                  }}
                  placeholder="Confirm captures, unresolved catalog details and rewards…"
                />
              </label>
              <div className="inline">
                <button
                  disabled={
                    pending || !connected || !reason.trim() || errors.length > 0
                  }
                  onClick={() => run(prepareReview)}
                >
                  Review outcome
                </button>
                <button
                  disabled={pending || !connected || !reason.trim()}
                  onClick={() =>
                    run(async () => {
                      if (
                        await execute({
                          type: "cancel_training",
                          encounter_id: encounter.id,
                          reason,
                        })
                      ) {
                        setDraft(null);
                        setCaptures({});
                      }
                    })
                  }
                >
                  Cancel encounter
                </button>
              </div>
              {review && (
                <div className="notice">
                  <h3>Confirm assumed victory</h3>
                  <p>
                    {selected.length} captures ·{" "}
                    {awards.reduce(
                      (sum, a) =>
                        sum +
                        (a.preview.sheet.workflow?.training_points ?? 0) -
                        (a.request.sheet.workflow?.training_points ?? 0),
                      0,
                    )}{" "}
                    Training Points
                  </p>
                  {selected.map((v) => (
                    <p key={v.request.sheet.id}>
                      {v.request.sheet.name} →{" "}
                      {c.sheets[v.request.sheet.trainer_id!]?.name} ·{" "}
                      {v.request.sheet.rank}
                    </p>
                  ))}
                  {selected
                    .flatMap((v) => v.preview.approvals)
                    .map((a, i) => (
                      <p key={i}>GM acceptance: {a}</p>
                    ))}
                  {awards.map((a) => (
                    <div key={a.request.sheet.id}>
                      <p>
                        {a.preview.sheet.name}: {a.request.amount} requested →{" "}
                        {(a.preview.sheet.workflow?.training_points ?? 0) -
                          (a.request.sheet.workflow?.training_points ?? 0)}{" "}
                        TP awarded
                      </p>
                      {[...a.preview.warnings, ...a.preview.approvals].map(
                        (message, i) => (
                          <p key={i}>{message}</p>
                        ),
                      )}
                    </div>
                  ))}
                  <p>{reason}</p>
                  <button
                    className="primary"
                    disabled={pending || !connected || errors.length > 0}
                    onClick={() => run(confirm)}
                  >
                    Confirm captures and rewards
                  </button>
                </div>
              )}
            </>
          )}
        </div>
      )}
      <div className="panel">
        <h2>Completed training encounters</h2>
        {!history.length && <p>No completed encounters yet.</p>}
        {history
          .slice()
          .reverse()
          .map((e) => {
            const owned = Object.values(e.captured).filter(
              (id): id is string => !!id && c.sheets[id]?.owner_id === playerId,
            );
            return (
              <div className="list-row" key={e.id}>
                <div>
                  <strong>
                    {habitats.find((h) => h.habitat_id === e.habitat_id)
                      ?.habitat ?? e.habitat_id}{" "}
                    · {e.status}
                  </strong>
                  <p>
                    Victory{" "}
                    {e.status === "resolved" ? "assumed" : "not resolved"} ·{" "}
                    {Object.keys(e.captured).length} captured ·{" "}
                    {e.individuals.length - Object.keys(e.captured).length} not
                    captured
                  </p>
                  <p>{e.reason}</p>
                  <p>
                    {Object.entries(e.rewards)
                      .map(([id, n]) => `${c.sheets[id]?.name ?? id}: ${n} TP`)
                      .join(" · ")}
                  </p>
                </div>
                {owned.length > 0 && (
                  <button
                    disabled={pending || !connected}
                    onClick={() =>
                      run(() =>
                        pull([
                          ...new Set([
                            ...owned,
                            ...owned
                              .map((id) => c.sheets[id]?.trainer_id)
                              .filter((x): x is string => !!x),
                          ]),
                        ]),
                      )
                    }
                  >
                    Import my captured Pokémon
                  </button>
                )}
              </div>
            );
          })}
      </div>
      {editor && encounter && captures[editor] && (
        <SheetEditor
          key={editor}
          catalog={catalog}
          initial={captures[editor].request.sheet}
          trainers={trainers}
          draftRequest={captures[editor].request}
          close={() => setEditor(null)}
          saved={async () => {}}
          previewRequest={async (req) =>
            (
              await api.previewCapture(
                c,
                encounter,
                editor,
                req.sheet.trainer_id ?? "",
                req,
              )
            ).preview
          }
          captureSave={async (req) => {
            const v = await api.previewCapture(
              c,
              encounter,
              editor,
              req.sheet.trainer_id ?? "",
              req,
            );
            if (v.preview.errors.length)
              throw Error(v.preview.errors.join("; "));
            setCaptures((x) => ({ ...x, [editor]: v }));
            setReview(false);
          }}
        />
      )}
    </>
  );
}
