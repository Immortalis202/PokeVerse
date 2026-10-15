import { attributes } from "./catalog";
import { Sprite, Types } from "./CatalogVisuals";
import type { CompendiumReference } from "./bindings/CompendiumReference";
type Entry = Record<string, unknown>;
const label = (value: unknown) =>
  value === null || value === undefined || value === "" ? "—" : String(value);
function measurement(
  value: unknown,
  metric: string,
  imperial: string,
  unit: string,
  imperialUnit: string,
) {
  const data = value as Record<string, unknown> | undefined;
  return data
    ? [
        data[metric] !== undefined ? `${data[metric]} ${unit}` : null,
        data[imperial] !== undefined
          ? `${data[imperial]} ${imperialUnit}`
          : null,
      ]
        .filter(Boolean)
        .join(" / ") || "—"
    : "—";
}
export function AttributePoints({
  name,
  minimum,
  maximum,
}: {
  name: string;
  minimum: unknown;
  maximum: unknown;
}) {
  const valid =
    typeof minimum === "number" &&
    typeof maximum === "number" &&
    Number.isInteger(minimum) &&
    Number.isInteger(maximum) &&
    minimum >= 0 &&
    maximum >= minimum;
  return (
    <div
      className="species-attribute"
      role="group"
      aria-label={
        valid
          ? `${name}: minimum ${minimum}, maximum ${maximum}`
          : `${name}: minimum ${label(minimum)}, maximum ${label(maximum)}, unresolved points`
      }
    >
      <strong>{name}</strong>
      <span className="species-points" aria-hidden="true">
        {valid ? (
          Array.from({ length: maximum }, (_, i) => (
            <span
              key={i}
              className={`species-point ${i < minimum ? "minimum" : "available"}`}
            />
          ))
        ) : (
          <span className="warning-text">?</span>
        )}
      </span>
      <span className="species-range" aria-hidden="true">
        {label(minimum)} / {label(maximum)}
      </span>
    </div>
  );
}
export default function SpeciesStatCard({
  entry,
  references,
  onSelect,
}: {
  entry: Entry;
  references: CompendiumReference[];
  onSelect: (ref: CompendiumReference) => void;
}) {
  const creation = entry.Creation as
    | {
        baselines?: Record<string, number>;
        limits?: Record<string, number>;
        stage?: number | null;
      }
    | undefined;
  const abilities = references.filter((r) => r.kind === "abilities");
  const evolutions = references.filter((r) => r.kind === "species");
  const speeds = [
    ...new Set(
      evolutions
        .filter((r) => r.role === "Evolution")
        .map((r) => r.requirements.Speed)
        .filter(Boolean),
    ),
  ];
  const stage = creation?.stage;
  const stageLabel =
    stage === 1
      ? "First"
      : stage === 2
        ? "Second"
        : stage === 3
          ? "Third"
          : typeof stage === "number"
            ? String(stage)
            : "Unreviewed";
  function link(ref: CompendiumReference) {
    return ref.id ? (
      <button className="reference-link" onClick={() => onSelect(ref)}>
        {ref.name}
      </button>
    ) : (
      <span>
        {ref.name}
        <small className="warning-text">
          {ref.issue ?? "Unresolved reference"}
        </small>
      </span>
    );
  }
  return (
    <article
      className="species-sheet-card"
      aria-label={`${entry.Name} species reference`}
    >
      <header className="species-sheet-heading">
        <div>
          <span className="species-number">#{label(entry.DexID)}</span>
          <h2>{label(entry.Name)}</h2>
        </div>
        <div className="species-sheet-types">
          <small>Type</small>
          <Types entry={entry} />
        </div>
      </header>
      <div className="species-sheet-body">
        <section className="species-profile" aria-label="Species description">
          <div className="species-portrait">
            <Sprite entry={entry} trim />
          </div>
          <dl className="species-measurements">
            <div>
              <dt>Height</dt>
              <dd>{measurement(entry.Height, "Meters", "Feet", "m", "ft")}</dd>
            </div>
            <div>
              <dt>Weight</dt>
              <dd>
                {measurement(entry.Weight, "Kilograms", "Pounds", "kg", "lb")}
              </dd>
            </div>
          </dl>
          <div className="species-ecology">
            <h3>{label(entry.DexCategory)}</h3>
            <p>{label(entry.DexDescription)}</p>
            {!!entry.GenderType && (
              <small>Gender: {label(entry.GenderType)}</small>
            )}
          </div>
        </section>
        <section
          className="species-mechanics"
          aria-label="Species scores and evolution"
        >
          <div className="species-attributes">
            {attributes.map((name) => (
              <AttributePoints
                key={name}
                name={name}
                minimum={creation?.baselines?.[name] ?? entry[name]}
                maximum={creation?.limits?.[name] ?? entry[`Max${name}`]}
              />
            ))}
          </div>
          <p className="species-points-key">
            <span className="species-point minimum" /> Minimum{" "}
            <span className="species-point available" /> Up to maximum{" "}
            <span className="species-range-key">min / max</span>
          </p>
          <div className="species-resource-row">
            <div className="species-base-hp">
              <span>Base HP</span>
              <strong>{label(entry.BaseHP)}</strong>
            </div>
            <div className="species-rank">
              <span>Suggested rank</span>
              <strong>{label(entry.RecommendedRank)}</strong>
            </div>
          </div>
          <div className="species-sheet-abilities">
            <h3>Abilities</h3>
            {abilities.length ? (
              abilities.map((ref, i) => (
                <div
                  className="species-ability"
                  key={`${ref.role}-${ref.name}-${i}`}
                >
                  {link(ref)}
                  <small>{ref.role}</small>
                </div>
              ))
            ) : (
              <p>No abilities recorded.</p>
            )}
          </div>
          <div className="species-sheet-evolution">
            <dl>
              <div>
                <dt>Evolution stage</dt>
                <dd>{stageLabel}</dd>
              </div>
              <div>
                <dt>Evolution time</dt>
                <dd>{speeds.length ? speeds.join(" / ") : "—"}</dd>
              </div>
            </dl>
            <div className="species-evolution-strip">
              {evolutions.length ? (
                evolutions.map((ref, i) => (
                  <div
                    key={`${ref.role}-${ref.name}-${i}`}
                    className="species-evolution-mini"
                  >
                    <small>{ref.role}</small>
                    {ref.id ? (
                      <button
                        className="species-evolution-link"
                        onClick={() => onSelect(ref)}
                      >
                        <Sprite entry={ref.summary ?? { Name: ref.name }} />
                        <span>{ref.name}</span>
                        <small>#{label(ref.summary?.DexID)}</small>
                      </button>
                    ) : (
                      <>
                        <Sprite entry={{ Name: ref.name }} />
                        {link(ref)}
                      </>
                    )}
                  </div>
                ))
              ) : (
                <small>No direct evolutions recorded.</small>
              )}
            </div>
          </div>
        </section>
      </div>
    </article>
  );
}
