import SpeciesStatCard from "./SpeciesStatCard";
import { Sprite, Types } from "./CatalogVisuals";
import { learnsetGroups } from "./compendium";
import { useEffect, useRef, useState } from "react";
import { ArrowLeft, Search } from "lucide-react";
import { api, errorOf } from "./api";
import {
  ranks,
  type Catalog,
  type CatalogKind,
  type CatalogSummary,
} from "./catalog";
import type { CompendiumDetail } from "./bindings/CompendiumDetail";
import type { CompendiumReference } from "./bindings/CompendiumReference";
const categories: [CatalogKind, string][] = [
  ["species", "Pokémon"],
  ["moves", "Moves"],
  ["abilities", "Abilities"],
  ["items", "Items"],
  ["natures", "Natures"],
];
const tabs = ["Stats", "Abilities", "Evolutions", "Learnset"] as const;
type Tab = (typeof tabs)[number];
type Location = { kind: CatalogKind; id: string; tab: Tab; scroll: number };
function text(value: unknown): string {
  return value === undefined || value === null || value === ""
    ? "—"
    : typeof value === "object"
      ? Object.entries(value)
          .map(([key, item]) => `${key}: ${text(item)}`)
          .join("\n")
      : String(value);
}
export default function CatalogView({ catalog }: { catalog: Catalog }) {
  const [category, setCategory] = useState<CatalogKind>("species");
  const [query, setQuery] = useState("");
  const [type, setType] = useState("");
  const [offset, setOffset] = useState(0);
  const [list, setList] = useState<CatalogSummary[]>([]);
  const [total, setTotal] = useState(0);
  const [listLoading, setListLoading] = useState(true);
  const [listError, setListError] = useState("");
  const [location, setLocation] = useState<Location | null>(null);
  const [history, setHistory] = useState<Location[]>([]);
  const [detail, setDetail] = useState<CompendiumDetail | null>(null);
  const [detailLoading, setDetailLoading] = useState(false);
  const [detailError, setDetailError] = useState("");
  const listPane = useRef<HTMLDivElement>(null);
  const detailPane = useRef<HTMLDivElement>(null);
  const restoreScroll = useRef(0);
  function fresh(kind: CatalogKind, id: string) {
    setHistory([]);
    if (detailPane.current) detailPane.current.scrollTop = 0;
    restoreScroll.current = 0;
    setLocation({ kind, id, tab: "Stats", scroll: 0 });
  }
  function filters(change: () => void) {
    change();
    setOffset(0);
    setHistory([]);
    setLocation(null);
    setDetail(null);
  }
  useEffect(() => {
    let cancelled = false;
    setListLoading(true);
    setListError("");
    setList([]);
    setLocation(null);
    setHistory([]);
    const timer = setTimeout(() => {
      api
        .searchCatalog(category, query, type, offset)
        .then((page) => {
          if (cancelled) return;
          setList(page.entries);
          setTotal(page.total);
          if (listPane.current) listPane.current.scrollTop = 0;
          if (page.entries[0]) fresh(category, page.entries[0].AppId);
        })
        .catch((e) => {
          if (!cancelled) setListError(errorOf(e).message);
        })
        .finally(() => {
          if (!cancelled) setListLoading(false);
        });
    }, 150);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [category, query, type, offset]);
  useEffect(() => {
    let cancelled = false;
    setDetail(null);
    setDetailError("");
    if (!location) {
      setDetailLoading(false);
      return;
    }
    setDetailLoading(true);
    api
      .compendiumDetail(location.kind, location.id)
      .then((value) => {
        if (!cancelled) {
          setDetail(value);
          if (!value) setDetailError("This catalog entry is unavailable.");
        }
      })
      .catch((e) => {
        if (!cancelled) setDetailError(errorOf(e).message);
      })
      .finally(() => {
        if (!cancelled) setDetailLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [location?.kind, location?.id]);
  useEffect(() => {
    if (detail && detailPane.current)
      detailPane.current.scrollTop = restoreScroll.current;
  }, [detail]);
  function related(ref: CompendiumReference) {
    if (!ref.id || !location) return;
    setHistory((h) => [
      ...h,
      { ...location, scroll: detailPane.current?.scrollTop ?? 0 },
    ]);
    restoreScroll.current = 0;
    setLocation({
      kind: ref.kind as CatalogKind,
      id: ref.id,
      tab: "Stats",
      scroll: 0,
    });
  }
  function back() {
    const previous = history.at(-1);
    if (!previous) return;
    restoreScroll.current = previous.scroll;
    setHistory((h) => h.slice(0, -1));
    setLocation(previous);
  }
  function reference(ref: CompendiumReference, index: number) {
    return (
      <div className="reference-row" key={`${ref.role}-${ref.name}-${index}`}>
        {ref.kind === "species" && (
          <Sprite entry={ref.summary ?? { Name: ref.name }} />
        )}
        <div>
          {ref.id ? (
            <button className="reference-link" onClick={() => related(ref)}>
              {ref.name}
            </button>
          ) : (
            <strong>{ref.name}</strong>
          )}
          {ref.kind === "moves" && (
            <small>
              {text(ref.summary?.Type)} · {text(ref.summary?.Category)}
            </small>
          )}
          {ref.kind === "abilities" && <small>{ref.role} ability</small>}
          {ref.kind === "species" && (
            <>
              <small>{ref.role}</small>
              <dl className="requirement-list">
                {Object.entries(ref.requirements)
                  .filter(([k]) => k !== "From" && k !== "To")
                  .map(([k, v]) => (
                    <div key={k}>
                      <dt>{k}</dt>
                      <dd>{text(v)}</dd>
                    </div>
                  ))}
              </dl>
            </>
          )}
          {ref.issue && (
            <small className="warning-text">
              {ref.issue}: {ref.name}
            </small>
          )}
        </div>
      </div>
    );
  }
  const listSelection = history[0] ?? location;
  const entry = detail?.entry;
  const activeTab = location?.tab ?? "Stats";
  return (
    <section className="compendium">
      <header className="compendium-heading">
        <div>
          <h1>Compendium</h1>
          <p>
            Offline reference · {catalog.species.length.toLocaleString()}{" "}
            species and forms · Pokerole {catalog.edition}
          </p>
        </div>
      </header>
      <nav className="compendium-categories" aria-label="Compendium categories">
        {categories.map(([kind, label]) => (
          <button
            key={kind}
            aria-pressed={category === kind}
            className={category === kind ? "selected" : ""}
            onClick={() =>
              category !== kind &&
              filters(() => {
                setCategory(kind);
                setType("");
              })
            }
          >
            {label}
          </button>
        ))}
      </nav>
      <div className="compendium-workspace">
        <section className="compendium-list-panel" aria-label="Catalog list">
          <div className="compendium-filters">
            <label className="search">
              <Search size={16} />
              <input
                aria-label="Search compendium"
                value={query}
                placeholder="Search by name…"
                onChange={(e) => filters(() => setQuery(e.target.value))}
              />
            </label>
            {(category === "species" || category === "moves") && (
              <select
                aria-label="Filter by type"
                value={type}
                onChange={(e) => filters(() => setType(e.target.value))}
              >
                <option value="">All types</option>
                {[
                  ...new Set(
                    [
                      ...catalog.species.flatMap((s) => [s.Type1, s.Type2]),
                      ...catalog.moves.map((m) => m.Type),
                    ].filter(Boolean),
                  ),
                ]
                  .sort()
                  .map((t) => (
                    <option key={t}>{t}</option>
                  ))}
              </select>
            )}
          </div>
          <div
            className="compendium-list"
            ref={listPane}
            aria-busy={listLoading}
          >
            {listLoading ? (
              <p role="status">Loading catalog…</p>
            ) : listError ? (
              <p role="alert">{listError}</p>
            ) : !list.length ? (
              <p role="status">No entries match these filters.</p>
            ) : (
              list.map((item) => {
                const e = item as unknown as Record<string, unknown>;
                return (
                  <button
                    key={item.AppId}
                    className={`compendium-card ${listSelection?.kind === category && listSelection.id === item.AppId ? "selected" : ""}`}
                    onClick={() => fresh(category, item.AppId)}
                    aria-pressed={
                      listSelection?.kind === category &&
                      listSelection.id === item.AppId
                    }
                  >
                    {category === "species" && <Sprite entry={e} />}
                    <span>
                      <strong>{item.Name}</strong>
                      {category === "species" && (
                        <small>
                          #{text(e.DexID)} · {text(e.RecommendedRank)}
                          {e.Legendary ? " · Legendary" : ""}
                          {e.GoodStarter ? " · Starter" : ""}
                        </small>
                      )}
                      <Types entry={e} />
                      {category === "moves" && (
                        <small>{text(e.Category)}</small>
                      )}
                    </span>
                  </button>
                );
              })
            )}
          </div>
          <div className="compendium-pagination">
            <button
              disabled={listLoading || offset === 0}
              onClick={() => setOffset((o) => Math.max(0, o - 150))}
            >
              Previous
            </button>
            <span>
              {total
                ? `${offset + 1}–${Math.min(offset + 150, total)} / ${total}`
                : "0 entries"}
            </span>
            <button
              disabled={listLoading || offset + 150 >= total}
              onClick={() => setOffset((o) => o + 150)}
            >
              Next
            </button>
          </div>
        </section>
        <section
          className="compendium-detail-panel"
          aria-label="Catalog details"
        >
          <div className="compendium-detail-toolbar">
            <button disabled={!history.length} onClick={back}>
              <ArrowLeft size={16} /> Back
            </button>
            <small>Reference only</small>
          </div>
          <div
            className="compendium-detail"
            ref={detailPane}
            aria-busy={detailLoading}
          >
            {detailLoading ? (
              <p role="status">Loading details…</p>
            ) : detailError ? (
              <p role="alert">{detailError}</p>
            ) : !entry ? (
              <p>Select an entry to view its details.</p>
            ) : (
              <>
                {(detail.kind !== "species" || activeTab !== "Stats") && (
                  <header className="compendium-entry-header">
                    {detail.kind === "species" && <Sprite entry={entry} />}
                    <div>
                      <h2>{text(entry.Name)}</h2>
                      {detail.kind === "species" && (
                        <p>
                          #{text(entry.DexID)} · {text(entry.DexCategory)} ·{" "}
                          {text(entry.RecommendedRank)}
                        </p>
                      )}
                      <Types entry={entry} />
                    </div>
                  </header>
                )}
                {detail.kind === "species" ? (
                  <>
                    <div
                      className="compendium-detail-tabs"
                      role="tablist"
                      aria-label="Pokémon details"
                    >
                      {tabs.map((tab, i) => (
                        <button
                          role="tab"
                          id={`species-tab-${i}`}
                          aria-controls="species-tabpanel"
                          aria-selected={activeTab === tab}
                          tabIndex={activeTab === tab ? 0 : -1}
                          key={tab}
                          onClick={() => {
                            if (location) setLocation({ ...location, tab });
                            if (detailPane.current)
                              detailPane.current.scrollTop = 0;
                          }}
                          onKeyDown={(event) => {
                            const index =
                              event.key === "ArrowRight"
                                ? (i + 1) % 4
                                : event.key === "ArrowLeft"
                                  ? (i + 3) % 4
                                  : event.key === "Home"
                                    ? 0
                                    : event.key === "End"
                                      ? 3
                                      : -1;
                            if (index >= 0 && location) {
                              event.preventDefault();
                              setLocation({ ...location, tab: tabs[index] });
                              if (detailPane.current)
                                detailPane.current.scrollTop = 0;
                              document
                                .getElementById(`species-tab-${index}`)
                                ?.focus();
                            }
                          }}
                        >
                          {tab}
                        </button>
                      ))}
                    </div>
                    <div
                      role="tabpanel"
                      id="species-tabpanel"
                      aria-labelledby={`species-tab-${tabs.indexOf(activeTab)}`}
                      tabIndex={0}
                    >
                      {activeTab === "Stats" && (
                        <SpeciesStatCard
                          entry={entry}
                          references={detail.references}
                          onSelect={related}
                        />
                      )}
                      {activeTab === "Abilities" && (
                        <>
                          {detail.references
                            .filter((r) => r.kind === "abilities")
                            .map(reference)}
                          {!detail.references.some(
                            (r) => r.kind === "abilities",
                          ) && <p>No abilities recorded.</p>}
                          <p className="muted">
                            Hidden and event abilities require the appropriate
                            authorization in character workflows.
                          </p>
                        </>
                      )}
                      {activeTab === "Evolutions" && (
                        <>
                          {detail.references
                            .filter((r) => r.kind === "species")
                            .map(reference)}
                          {!detail.references.some(
                            (r) => r.kind === "species",
                          ) && (
                            <p>
                              No direct evolutions or pre-evolutions recorded.
                            </p>
                          )}
                          <p className="muted">
                            Recorded requirements are reference data; character
                            eligibility is evaluated in progression.
                          </p>
                        </>
                      )}
                      {activeTab === "Learnset" && (
                        <>
                          {learnsetGroups(detail.references).map((group) => (
                            <section
                              className="learnset-group"
                              key={group.method}
                            >
                              <h3>
                                {ranks.includes(group.method)
                                  ? group.method
                                  : `Learning method: ${group.method}`}
                              </h3>
                              {group.moves.map(reference)}
                            </section>
                          ))}
                          {!detail.references.some(
                            (r) => r.kind === "moves",
                          ) && <p>No moves recorded.</p>}
                        </>
                      )}
                    </div>
                  </>
                ) : (
                  <>
                    <p className="compendium-description">
                      {text(entry.Description)}
                    </p>
                    {detail.kind === "moves" && (
                      <dl className="compendium-facts">
                        {[
                          ["Category", entry.Category],
                          ["Power", entry.Power],
                          [
                            "Accuracy sources",
                            [entry.Accuracy1, entry.Accuracy2]
                              .filter(Boolean)
                              .join(" + "),
                          ],
                          [
                            "Damage sources",
                            [entry.Damage1, entry.Damage2]
                              .filter(Boolean)
                              .join(" + "),
                          ],
                          ["Target", entry.Target],
                        ].map(([label, value]) => (
                          <div key={String(label)}>
                            <dt>{text(label)}</dt>
                            <dd>{text(value)}</dd>
                          </div>
                        ))}
                      </dl>
                    )}
                    {entry.Effect !== undefined && (
                      <>
                        <h3>Effect</h3>
                        <p>{text(entry.Effect)}</p>
                      </>
                    )}
                    {Object.entries(entry)
                      .filter(
                        ([key, value]) =>
                          ![
                            "Name",
                            "AppId",
                            "_id",
                            "Description",
                            "Effect",
                            "Type",
                            "Power",
                            "Category",
                            "Accuracy1",
                            "Accuracy2",
                            "Damage1",
                            "Damage2",
                            "Target",
                            "Image",
                            "PMD",
                            "Review",
                          ].includes(key) &&
                          value !== "" &&
                          value !== null &&
                          !(
                            typeof value === "object" &&
                            Object.keys(value as object).length === 0
                          ),
                      )
                      .map(([key, value]) => (
                        <div className="compendium-extra" key={key}>
                          <h3>{key.replace(/([a-z])([A-Z])/g, "$1 $2")}</h3>
                          <p>{text(value)}</p>
                        </div>
                      ))}
                    {detail.kind === "moves" && (
                      <p className="notice">
                        Supported battle damage is calculated automatically.
                        Conditional, reactive, status and other unsupported
                        special effects require manual adjudication; consult the
                        effect text and battle preview.
                      </p>
                    )}
                  </>
                )}
                {!!detail.warnings.length && (
                  <aside
                    className="notice"
                    aria-label="Metadata review warnings"
                  >
                    <h3>Metadata review</h3>
                    {detail.warnings.map((warning, i) => (
                      <p key={i}>{warning}</p>
                    ))}
                  </aside>
                )}
              </>
            )}
          </div>
        </section>
      </div>
    </section>
  );
}
