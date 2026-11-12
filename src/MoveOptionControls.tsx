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
  const [help, setHelp] = useState({ helper: "", ally: "", move: "" });
  const [copy, setCopy] = useState({
    user: "",
    origin: "",
    source: "",
    template: "",
    target: "",
  });
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
  const allied = f.completion?.blocks?.includes("MOVE-A53");
  const powder = Object.entries(f.completion?.powder_due ?? {});
  const copying = f.completion?.blocks?.includes("MOVE-A59");
  const COPIES = ["Copycat", "Mirror Move", "Instruct", "Assist", "Me First"];
  const copyUsers = copying
    ? b.combatants.filter(
        (p) =>
          p.deployed &&
          p.hp > 0 &&
          (campaign.sheets[p.sheet_id]?.moves ?? []).some((m) =>
            COPIES.includes(m),
          ),
      )
    : [];
  const copyUser = copyUsers.find((p) => p.id === copy.user) ?? copyUsers[0];
  const origins = copyUser
    ? (campaign.sheets[copyUser.sheet_id]?.moves ?? []).filter((m) =>
        COPIES.includes(m),
      )
    : [];
  const origin = origins.includes(copy.origin) ? copy.origin : origins[0];
  const others = copyUser
    ? b.combatants.filter((p) => p.deployed && p.hp > 0 && p.id !== copyUser.id)
    : [];
  const source = others.find((p) => p.id === copy.source) ?? others[0];
  const lastMove = source
    ? [...(f.completion?.uses ?? [])]
        .reverse()
        .find((u) => u.actor === source.id)?.name
    : undefined;
  const templates =
    origin === "Assist" && source
      ? (campaign.sheets[source.sheet_id]?.moves ?? [])
      : origin === "Me First"
        ? q && q.actor_id === source?.id
          ? [q.move_name]
          : []
        : lastMove
          ? [lastMove]
          : [];
  const template = templates.includes(copy.template)
    ? copy.template
    : templates[0];
  const copyTarget = others.find((p) => p.id === copy.target) ?? source;
  const active = b.combatants.filter((p) => p.deployed && p.hp > 0);
  const caster = q ? active.find((p) => p.id === q.actor_id) : undefined;
  const joiners =
    allied &&
    q &&
    caster &&
    !q.damage_done &&
    !f.window &&
    ["Beat Up", "Round"].includes(q.move_name)
      ? active.filter((p) => p.side === caster.side && p.id !== caster.id)
      : [];
  const helpers = allied
    ? active.filter((p) =>
        (campaign.sheets[p.sheet_id]?.moves ?? []).includes("Help Another"),
      )
    : [];
  const helper = helpers.find((p) => p.id === help.helper) ?? helpers[0];
  const allies = helper
    ? active.filter((p) => p.side === helper.side && p.id !== helper.id)
    : [];
  const ally = allies.find((p) => p.id === help.ally) ?? allies[0];
  const allyMoves = ally ? (campaign.sheets[ally.sheet_id]?.moves ?? []) : [];
  const move = allyMoves.includes(help.move) ? help.move : allyMoves[0];
  if (
    !options?.length &&
    !conversions.length &&
    !grapplers.length &&
    !timed.length &&
    !joiners.length &&
    !powder.length &&
    !copyUser &&
    !(helper && ally && move)
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
      {q &&
        joiners.map((p) => (
          <section key={`join-${p.id}`} aria-label={`Join ${q.move_name}`}>
            <button
              disabled={disabled || !owns(p.id)}
              onClick={() =>
                execute({
                  type: "special_move",
                  command: {
                    kind: "JoinMove",
                    combatant_id: p.id,
                    sequence: q.sequence,
                  },
                  reason: `${p.name} joins ${q.move_name}`,
                })
              }
            >
              {p.name}: join {q.move_name}
            </button>
          </section>
        ))}
      {helper && ally && move && (
        <section aria-label="Help Another">
          <h4>Help Another</h4>
          <select
            aria-label="Helper"
            value={helper.id}
            onChange={(e) => setHelp({ ...help, helper: e.target.value })}
          >
            {helpers.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name}
              </option>
            ))}
          </select>
          <select
            aria-label="Ally to help"
            value={ally.id}
            onChange={(e) => setHelp({ ...help, ally: e.target.value })}
          >
            {allies.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name}
              </option>
            ))}
          </select>
          <select
            aria-label="Ally's intended move"
            value={move}
            onChange={(e) => setHelp({ ...help, move: e.target.value })}
          >
            {allyMoves.map((m) => (
              <option key={m} value={m}>
                {m}
              </option>
            ))}
          </select>
          <button
            disabled={disabled || !owns(helper.id)}
            onClick={() =>
              execute({
                type: "special_move",
                command: {
                  kind: "HelpAnother",
                  combatant_id: helper.id,
                  ally_id: ally.id,
                  move_name: move,
                },
                reason: `${helper.name} helps ${ally.name} with ${move}`,
              })
            }
          >
            Roll help
          </button>
        </section>
      )}
      {copyUser && origin && (
        <section aria-label="Copy move">
          <h4>Copy move</h4>
          <select
            aria-label="Copy user"
            value={copyUser.id}
            onChange={(e) => setCopy({ ...copy, user: e.target.value })}
          >
            {copyUsers.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name}
              </option>
            ))}
          </select>
          <select
            aria-label="Copy move"
            value={origin}
            onChange={(e) => setCopy({ ...copy, origin: e.target.value })}
          >
            {origins.map((m) => (
              <option key={m} value={m}>
                {m}
              </option>
            ))}
          </select>
          <select
            aria-label="Copy from"
            value={source?.id ?? ""}
            onChange={(e) => setCopy({ ...copy, source: e.target.value })}
          >
            {others.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name}
              </option>
            ))}
          </select>
          <select
            aria-label="Copied move"
            value={template ?? ""}
            onChange={(e) => setCopy({ ...copy, template: e.target.value })}
          >
            {templates.map((m) => (
              <option key={m} value={m}>
                {m}
              </option>
            ))}
          </select>
          <select
            aria-label="Copy target"
            value={copyTarget?.id ?? ""}
            onChange={(e) => setCopy({ ...copy, target: e.target.value })}
          >
            {others.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name}
              </option>
            ))}
          </select>
          <button
            disabled={
              disabled ||
              !owns(copyUser.id) ||
              !source ||
              !template ||
              !copyTarget
            }
            onClick={() =>
              source &&
              template &&
              copyTarget &&
              execute({
                type: "special_move",
                command: {
                  kind: "CastCopy",
                  combatant_id: copyUser.id,
                  origin,
                  source_id: source.id,
                  template,
                  target_ids: [copyTarget.id],
                },
                reason: `${origin} copies ${template}`,
              })
            }
          >
            Cast copy
          </button>
        </section>
      )}
      {powder.map(([id, dice]) => (
        <section key={`powder-${id}`} aria-label="Powder">
          <p>
            {label(id)} owes {dice} Powder damage dice (typeless, no defenses).
          </p>
          <button
            disabled={disabled || !owns(id)}
            onClick={() =>
              execute({
                type: "special_move",
                command: { kind: "ResolvePowder", combatant_id: id },
                reason: "Powder explodes on the Fire move",
              })
            }
          >
            Roll Powder damage
          </button>
        </section>
      ))}
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
