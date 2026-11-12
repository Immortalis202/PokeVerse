import { useState } from "react";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";

/** Saved owner choices: the pending cast's open option and Conversion 2 types. */
export default function MoveOptionControls({
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
  const [choice, setChoice] = useState("");
  const [types, setTypes] = useState<Record<string, string>>({});
  const b = campaign.battle,
    f = b?.flow,
    q = f?.queue[0];
  if (!b?.active || !f) return null;
  const options = q ? f.completion?.open_options?.[q.sequence] : undefined;
  const conversions = Object.entries(f.completion?.conversion_choices ?? {});
  const adopted = f.completion?.blocks?.includes("MOVE-A52");
  const grapplers = adopted
    ? [
        ...new Set(
          f.special.effects
            .filter((e) => e.slot === "grapple")
            .map((e) => e.source),
        ),
      ]
    : [];
  const timed = adopted
    ? b.combatants.flatMap((p) =>
        p.deployed && p.hp > 0
          ? (campaign.sheets[p.sheet_id]?.moves ?? [])
              .filter((m) => m === "Sucker Punch" || m === "Thunderclap")
              .map((m) => ({ id: p.id, move: m }))
          : [],
      )
    : [];
  if (
    !options?.length &&
    !conversions.length &&
    !grapplers.length &&
    !timed.length
  )
    return null;
  const owns = (id: string) =>
    campaign.gm_id === playerId ||
    b.combatants.some((p) => p.id === id && p.owner_id === playerId);
  const label = (o: string) => b.combatants.find((p) => p.id === o)?.name ?? o;
  const value = options?.includes(choice) ? choice : options?.[0];
  return (
    <>
      {q && options?.length && value ? (
        <section aria-label="Move choice">
          <h4>{q.move_name}: choose</h4>
          {f.completion.move_options?.[q.sequence] && (
            <p>Saved choice: {label(f.completion.move_options[q.sequence]!)}</p>
          )}
          <select
            aria-label={`${q.move_name} option`}
            value={value}
            disabled={disabled || !owns(q.actor_id)}
            onChange={(e) => setChoice(e.target.value)}
          >
            {options.map((o) => (
              <option key={o} value={o}>
                {label(o)}
              </option>
            ))}
          </select>
          <button
            disabled={disabled || !owns(q.actor_id)}
            onClick={() =>
              execute({
                type: "special_move",
                command: {
                  kind: "ChooseMoveOption",
                  sequence: q.sequence,
                  option: value,
                },
                reason: `Owner chooses ${value} for ${q.move_name}`,
              })
            }
          >
            Save choice
          </button>
        </section>
      ) : null}
      {grapplers.map((id) => (
        <section key={`grapple-${id}`} aria-label="Grapple">
          <p>{label(id)} is Grappling and cannot act.</p>
          <button
            disabled={disabled || !owns(id)}
            onClick={() =>
              execute({
                type: "special_move",
                command: { kind: "ReleaseGrapple", combatant_id: id },
                reason: "Release the Grapple to act",
              })
            }
          >
            Release Grapple
          </button>
        </section>
      ))}
      {timed.map(({ id, move }) => {
        const late = f.completion.reaction_timing?.[`${id}:${move}`] === "late";
        return (
          <section key={`timing-${id}-${move}`} aria-label={`${move} timing`}>
            <p>
              {label(id)}: {move} reacts{" "}
              {late ? "late (Late Reaction 1)" : "early (Reaction 6)"}.
            </p>
            <button
              disabled={disabled || !owns(id)}
              onClick={() =>
                execute({
                  type: "special_move",
                  command: {
                    kind: "ReactionTiming",
                    combatant_id: id,
                    move_name: move,
                    late: !late,
                  },
                  reason: `${move} reacts ${late ? "early" : "late"}`,
                })
              }
            >
              {late ? "React early" : "React late"}
            </button>
          </section>
        );
      })}
      {conversions.map(([id, list]) => {
        const picked =
          types[id] && list?.includes(types[id]) ? types[id] : list?.[0];
        if (!list?.length || !picked) return null;
        return (
          <section key={id} aria-label="Conversion 2 choice">
            <h4>Conversion 2: {label(id)}</h4>
            <select
              aria-label={`Conversion 2 type for ${label(id)}`}
              value={picked}
              disabled={disabled || !owns(id)}
              onChange={(e) => setTypes({ ...types, [id]: e.target.value })}
            >
              {list.map((t) => (
                <option key={t} value={t}>
                  {t}
                </option>
              ))}
            </select>
            <button
              disabled={disabled || !owns(id)}
              onClick={() =>
                execute({
                  type: "special_move",
                  command: {
                    kind: "ConvertType",
                    combatant_id: id,
                    type_name: picked,
                  },
                  reason: `Conversion 2 changes the main type to ${picked}`,
                })
              }
            >
              Change type
            </button>
          </section>
        );
      })}
    </>
  );
}
