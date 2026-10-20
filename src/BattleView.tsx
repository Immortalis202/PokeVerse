import { useEffect, useState } from "react";
import { Dice5, Flag, Heart, Play, SkipForward, Shield } from "lucide-react";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";
import type { Combatant } from "./bindings/Combatant";
import type { Catalog, Move } from "./catalog";
import type { AppError } from "./bindings/AppError";
import { api, errorOf } from "./api";
import { sheetValues } from "./sheets";
import AbilityInheritanceControls from "./AbilityInheritanceControls";
import AbilityControls from "./AbilityControls";
import SpecialMoveControls from "./SpecialMoveControls";
import TacticsControls from "./TacticsControls";
import AdvancedTargeting from "./AdvancedTargeting";
import CoreBattleFlow from "./CoreBattleFlow";
import BattleTools from "./BattleTools";

export default function BattleView({
  campaign,
  playerId,
  catalog,
  execute,
  disabled,
  error,
}: {
  campaign: Campaign | null;
  playerId: string;
  catalog: Catalog;
  execute: (action: Action) => Promise<boolean>;
  disabled: boolean;
  error: AppError | null;
}) {
  const [selected, setSelected] = useState<string[]>([]);
  const [teams, setTeams] = useState(true);
  const [slots, setSlots] = useState(3);
  const [starters, setStarters] = useState<string[]>([]);
  const [switchOut, setSwitchOut] = useState("");
  const [switchIn, setSwitchIn] = useState("");
  const [switchReason, setSwitchReason] = useState("");
  const [switchOverride, setSwitchOverride] = useState(false);
  const [sides, setSides] = useState<Record<string, string>>({});
  const [actor, setActor] = useState("");
  const [target, setTarget] = useState("");
  const [move, setMove] = useState("");
  const [dice, setDice] = useState(0);
  const [delta, setDelta] = useState(0);
  const [condition, setCondition] = useState("");
  const [reason, setReason] = useState("");
  const [useAction, setUseAction] = useState(true);
  const [override, setOverride] = useState<Combatant | null>(null);
  const [overrideReason, setOverrideReason] = useState("");
  const [phase, setPhase] = useState("");
  const [poolModifier, setPoolModifier] = useState(0);
  const [successModifier, setSuccessModifier] = useState(0);
  const [damageModifier, setDamageModifier] = useState(0);
  const [previewNotice, setPreviewNotice] = useState("");
  useEffect(() => {
    setActor("");
    setTarget("");
    setMove("");
    setCondition("");
    setReason("");
    setDelta(0);
    setSwitchOut("");
    setSwitchIn("");
    setSwitchReason("");
    setSwitchOverride(false);
  }, [campaign?.battle?.id]);
  const [moveDetail, setMoveDetail] = useState<Move | null>(null);
  useEffect(() => {
    const b = campaign?.battle;
    if (
      actor &&
      !b?.combatants.some(
        (p) => p.id === actor && p.deployed && p.available_round <= b.round,
      )
    )
      setActor("");
    if (target && !b?.combatants.some((p) => p.id === target && p.deployed))
      setTarget("");
  }, [campaign?.battle, actor, target]);
  const [catalogError, setCatalogError] = useState("");
  useEffect(() => {
    let cancelled = false;
    setMoveDetail(null);
    setCatalogError("");
    const entry = catalog.moves.find((m) => m.Name === move);
    if (entry)
      api
        .catalogEntry<Move>("moves", entry.AppId)
        .then((value) => {
          if (!cancelled) setMoveDetail(value);
        })
        .catch((e) => {
          if (!cancelled) setCatalogError(errorOf(e).message);
        });
    return () => {
      cancelled = true;
    };
  }, [move, catalog]);
  if (!campaign)
    return (
      <div className="empty large">
        <Shield size={40} />
        <h2>A shared battlefield</h2>
        <p>Host or join a room, then synchronize your trainer and squad.</p>
      </div>
    );
  const gm = campaign.gm_id === playerId;
  const core = [
    "0.2.0-core",
    "0.3.0-core",
    "0.4.0-core",
    "0.5.0-core",
    "0.6.0-core",
    "0.7.0-core",
    "0.8.0-core",
    "0.9.0-core",
    "0.10.0-core",
    "0.11.0-core",
    "0.15.0-core",
    "0.14.0-core",
    "0.13.0-core",
    "0.12.0-core",
  ].includes(campaign.engine);
  const battle = campaign.battle;
  const actorSheet =
    campaign.sheets[
      battle?.combatants.find((c) => c.id === actor)?.sheet_id ?? ""
    ];
  const chosenMove = moveDetail?.Name === move ? moveDetail : null;
  const pending = battle?.combatants.find(
    (p) => p.id === actor,
  )?.pending_damage;
  const busy = disabled;
  const set = (id: string) =>
    setSelected((s) =>
      s.includes(id) ? s.filter((x) => x !== id) : [...s, id],
    );
  return (
    <>
      {catalogError && (
        <p role="alert" className="notice">
          {catalogError}
        </p>
      )}
      <div className="page-head">
        <div>
          <span className="eyebrow">
            {campaign.name} · {gm ? "GAME MASTER" : "PLAYER"}
          </span>
          <h1>{battle?.active ? `Round ${battle.round}` : "Battle control"}</h1>
          <p>
            {battle?.active
              ? battle.phase
              : "Choose the participants and bring your encounter to life."}
          </p>
        </div>
        {battle?.active && gm && (
          <div className="inline">
            <button
              disabled={busy}
              onClick={() => execute({ type: "next_round" })}
            >
              <SkipForward size={16} /> Next round
            </button>
            <button
              disabled={busy}
              onClick={() => execute({ type: "end_battle" })}
            >
              <Flag size={16} /> End battle
            </button>
          </div>
        )}
      </div>
      <div className="notice">
        <strong>
          {core ? "Pokerole 3.0 core rolls." : "Legacy manual campaign."}
        </strong>{" "}
        {core
          ? `Server checks are automatic and need no host-player approval. ${battle?.automatic_validation === false ? "Battle rule enforcement is OFF: the GM adjudicates exceptions." : "Battle rule enforcement is ON for the supported core rules."} Select a target and roll accuracy, resolve reactions, then roll automatic damage. The host calculates ordinary Pokémon damage pools, defenses, STAB, critical bonuses, pain and type effectiveness, and updates HP. Reviewed effects and turn/reaction controls are available below in the new engine. Unsupported abilities and move effects remain explicitly adjudicated.`
          : "Upgrade between battles to enable verified core rolls. Previous records are preserved."}
        {campaign.engine !== "0.15.0-core" && gm && !battle?.active && (
          <button
            disabled={busy}
            onClick={() => execute({ type: "upgrade_rules" })}
          >
            Enable core rolls for this campaign
          </button>
        )}
      </div>
      <AbilityInheritanceControls
        campaign={campaign}
        playerId={playerId}
        execute={execute}
        disabled={busy}
      />
      <AbilityControls
        campaign={campaign}
        playerId={playerId}
        execute={execute}
        disabled={busy}
      />
      <SpecialMoveControls
        campaign={campaign}
        playerId={playerId}
        execute={execute}
        disabled={busy}
      />
      <TacticsControls
        campaign={campaign}
        playerId={playerId}
        execute={execute}
        disabled={busy}
      />
      <AdvancedTargeting
        campaign={campaign}
        catalog={catalog}
        playerId={playerId}
        execute={execute}
        disabled={busy}
      />
      <CoreBattleFlow
        catalog={catalog}
        campaign={campaign}
        playerId={playerId}
        execute={execute}
        disabled={busy}
      />
      {battle && gm && (
        <BattleTools campaign={campaign} disabled={busy} execute={execute} />
      )}
      {!battle?.active ? (
        <>
          <div className="panel">
            <h3>{teams ? "Registered teams" : "Combatants"}</h3>
            {gm && (
              <>
                <label className="checkbox">
                  <input
                    type="checkbox"
                    checked={teams}
                    onChange={(e) => setTeams(e.target.checked)}
                  />
                  Active slots and reserves
                </label>
                {teams && (
                  <>
                    <label>
                      Active slots per side
                      <input
                        type="number"
                        min={1}
                        max={6}
                        value={slots}
                        onChange={(e) => setSlots(Number(e.target.value))}
                      />
                    </label>
                    <p>
                      Register the full roster, then mark the starters. For 3v3,
                      register all twelve Pokémon and choose three starters on
                      each side. The GM records the players' team choices.
                    </p>
                  </>
                )}
              </>
            )}
            {sheetValues(campaign.sheets)
              .filter((s) => !s.archived && (!teams || s.kind === "pokemon"))
              .map((s) => (
                <div className="list-row" key={s.id}>
                  <label className="checkbox">
                    <input
                      type="checkbox"
                      checked={selected.includes(s.id)}
                      onChange={() => set(s.id)}
                      disabled={!gm}
                    />
                    {s.name}
                    <span className="muted">
                      {s.kind} · HP {s.hp}/{s.max_hp}
                    </span>
                  </label>
                  {teams && (
                    <label className="checkbox">
                      <input
                        type="checkbox"
                        aria-label={`Start ${s.name} active`}
                        checked={
                          selected.includes(s.id) && starters.includes(s.id)
                        }
                        disabled={!gm || !selected.includes(s.id)}
                        onChange={() =>
                          setStarters((v) =>
                            v.includes(s.id)
                              ? v.filter((id) => id !== s.id)
                              : [...v, s.id],
                          )
                        }
                      />
                      Active at start
                    </label>
                  )}
                  <select
                    aria-label={`Side for ${s.name}`}
                    value={
                      sides[s.id] ??
                      (s.owner_id === campaign.gm_id ? "Opponents" : "Players")
                    }
                    onChange={(e) =>
                      setSides({ ...sides, [s.id]: e.target.value })
                    }
                    disabled={!gm}
                  >
                    <option>Players</option>
                    <option>Opponents</option>
                    {!teams && <option>Neutral</option>}
                  </select>
                </div>
              ))}
            {!Object.keys(campaign.sheets).length && (
              <p>Synchronize sheets in the room lobby first.</p>
            )}
            <button
              className="primary"
              disabled={busy || !gm || !selected.length}
              onClick={() =>
                execute(
                  teams
                    ? {
                        type: "start_team_battle",
                        sheet_ids: selected,
                        active_sheet_ids: starters.filter((id) =>
                          selected.includes(id),
                        ),
                        slots_per_side: slots,
                        sides: Object.fromEntries(
                          selected.map((id) => [
                            id,
                            sides[id] ??
                              (campaign.sheets[id]!.owner_id === campaign.gm_id
                                ? "Opponents"
                                : "Players"),
                          ]),
                        ),
                      }
                    : {
                        type: "start_battle",
                        sheet_ids: selected,
                        sides: Object.fromEntries(
                          selected.map((id) => [
                            id,
                            sides[id] ??
                              (campaign.sheets[id]!.owner_id === campaign.gm_id
                                ? "Opponents"
                                : "Players"),
                          ]),
                        ),
                      },
                )
              }
            >
              <Play size={16} /> Start encounter
            </button>
          </div>
        </>
      ) : (
        <>
          {gm && (
            <div className="inline phase-control">
              <input
                aria-label="Battle phase"
                placeholder="Set phase or declaration step…"
                value={phase}
                onChange={(e) => setPhase(e.target.value)}
              />
              <button
                disabled={busy || !phase.trim()}
                onClick={() => execute({ type: "set_phase", phase })}
              >
                Set phase
              </button>
            </div>
          )}
          <div className="combat-grid">
            {[...battle.combatants]
              .filter((p) => p.deployed)
              .sort((a, b) => {
                const base = (p: Combatant) => {
                  const s = campaign.sheets[p.sheet_id];
                  return (
                    (p.stat_overrides.Dexterity ?? s?.stats.Dexterity ?? 0) +
                    (p.skill_overrides.Alert ?? s?.skills.Alert ?? 0) +
                    (["Master", "Champion"].includes(s?.rank ?? "") ? 3 : 0)
                  );
                };
                return b.initiative - a.initiative || base(b) - base(a);
              })
              .map((p) => {
                const s = campaign.sheets[p.sheet_id];
                const species = catalog.species.find(
                  (x) => x.AppId === s?.species_id,
                );
                return (
                  <div
                    className={`combat-card ${p.hp === 0 ? "fainted" : ""}`}
                    key={p.id}
                  >
                    <div className="inline between">
                      <span className="pill">
                        {p.side}
                        {p.field_slot !== null
                          ? ` · Slot ${p.field_slot + 1}`
                          : ""}
                      </span>
                      <span className="muted">Initiative {p.initiative}</span>
                    </div>
                    <div className="combat-title">
                      {species && (
                        <img src={`/sprites/${species.Image}`} alt="" />
                      )}
                      <div>
                        <h3>{p.name}</h3>
                        <small>
                          {p.owner_id === playerId ? "Your combatant" : p.side}
                        </small>
                      </div>
                    </div>
                    <div className="hp-label">
                      <Heart size={15} />
                      <strong>
                        {p.hp} / {p.max_hp}
                      </strong>
                      {p.hp === 0 && <span>At 0 HP</span>}
                    </div>
                    <div className="hp-bar">
                      <span style={{ width: `${(100 * p.hp) / p.max_hp}%` }} />
                    </div>
                    <div className="counter-row">
                      <span>
                        Actions{" "}
                        <b>
                          {p.actions}
                          {core ? " / 5" : ""}
                        </b>
                      </span>
                      <span>
                        Reactions <b>{p.reactions}</b>
                      </span>
                    </div>
                    {(Object.keys(p.stat_overrides).length > 0 ||
                      Object.keys(p.skill_overrides).length > 0 ||
                      p.max_hp !== s?.max_hp) && (
                      <small>
                        Battle-only corrections active:{" "}
                        {[
                          ...Object.entries(p.stat_overrides),
                          ...Object.entries(p.skill_overrides),
                        ]
                          .map(([key, value]) => `${key} ${value}`)
                          .join(" · ")}
                        {p.max_hp !== s?.max_hp ? ` · Max HP ${p.max_hp}` : ""}
                      </small>
                    )}
                    {p.available_round > battle.round && (
                      <p className="notice">
                        Can act from round {p.available_round}; can still be
                        targeted now.
                      </p>
                    )}
                    {core && (gm || p.owner_id === playerId) && (
                      <button
                        disabled={
                          busy ||
                          p.available_round > battle.round ||
                          (p.initiative_rolled && !gm)
                        }
                        onClick={() =>
                          execute({ type: "initiative", combatant_id: p.id })
                        }
                      >
                        {p.initiative_rolled
                          ? "GM initiative tie reroll"
                          : "Roll initiative"}
                      </button>
                    )}
                    <div className="type-row">
                      <span className="type">
                        {p.selected_ability || "No ability"}
                      </span>
                      {p.conditions.map((c) => (
                        <span className="type" key={c}>
                          {c}
                        </span>
                      ))}
                    </div>
                    {gm && (
                      <button
                        disabled={busy}
                        onClick={() => {
                          setOverride(structuredClone(p));
                          setOverrideReason("");
                        }}
                      >
                        GM adjustment
                      </button>
                    )}
                  </div>
                );
              })}
          </div>
          {battle.slots_per_side > 0 && (
            <div className="panel">
              <h3>Reserves and switching</h3>
              <p>
                The first two recall/send-out operations per trainer each round
                are free. A switch uses two operations; further operations cost
                trainer actions (maximum five). Incoming Pokémon cannot act
                until the next round. Timing, trapping and switch-triggered
                effects require GM adjudication.
              </p>
              <div className="combat-grid">
                {battle.combatants
                  .filter((p) => !p.deployed)
                  .map((p) => (
                    <div className="list-row" key={p.id}>
                      <span>
                        {p.name} · {p.side} ·{" "}
                        {
                          campaign.sheets[
                            campaign.sheets[p.sheet_id]?.trainer_id ?? ""
                          ]?.name
                        }{" "}
                        · HP {p.hp}/{p.max_hp}
                      </span>
                    </div>
                  ))}
              </div>
              {Object.entries(battle.trainer_switches).map(([id, count]) => (
                <p key={id}>
                  {campaign.sheets[id]?.name}: {count} recall/send-out
                  operations · {battle.trainer_actions[id] ?? 0}/5 switching
                  actions this round
                </p>
              ))}
              <div className="form-grid">
                <label>
                  Recall / replace
                  <select
                    value={switchOut}
                    onChange={(e) => {
                      setSwitchOut(e.target.value);
                      setSwitchIn("");
                    }}
                  >
                    <option value="">Choose an active Pokémon</option>
                    {battle.combatants
                      .filter(
                        (p) => p.deployed && (gm || p.owner_id === playerId),
                      )
                      .map((p) => (
                        <option key={p.id} value={p.id}>
                          {p.name} · {p.side}
                          {p.hp === 0 ? " · fainted" : ""}
                        </option>
                      ))}
                  </select>
                </label>
                <label>
                  Send out
                  <select
                    value={switchIn}
                    onChange={(e) => setSwitchIn(e.target.value)}
                  >
                    <option value="">Choose a conscious reserve</option>
                    {battle.combatants
                      .filter((p) => {
                        const out = battle.combatants.find(
                          (o) => o.id === switchOut,
                        );
                        return (
                          out &&
                          !p.deployed &&
                          p.hp > 0 &&
                          p.side === out.side &&
                          p.owner_id === out.owner_id &&
                          campaign.sheets[p.sheet_id]?.trainer_id ===
                            campaign.sheets[out.sheet_id]?.trainer_id
                        );
                      })
                      .map((p) => (
                        <option key={p.id} value={p.id}>
                          {p.name} · HP {p.hp}/{p.max_hp}
                        </option>
                      ))}
                  </select>
                </label>
                <label>
                  Switching reason
                  <input
                    maxLength={1000}
                    value={switchReason}
                    onChange={(e) => setSwitchReason(e.target.value)}
                  />
                </label>
              </div>
              {gm && (
                <label className="checkbox">
                  <input
                    type="checkbox"
                    checked={switchOverride}
                    onChange={(e) => setSwitchOverride(e.target.checked)}
                  />
                  GM exception: waive switching cost and allow acting this round
                </label>
              )}
              <button
                disabled={
                  busy || !switchOut || !switchIn || !switchReason.trim()
                }
                onClick={async () => {
                  if (
                    await execute({
                      type: "switch_pokemon",
                      outgoing_id: switchOut,
                      incoming_id: switchIn,
                      gm_override: gm && switchOverride,
                      reason: switchReason,
                    })
                  ) {
                    setSwitchOut("");
                    setSwitchIn("");
                    setSwitchReason("");
                    setSwitchOverride(false);
                    setActor("");
                    setTarget("");
                  }
                }}
              >
                Confirm switch / replacement
              </button>
            </div>
          )}
          <div className="panel">
            <h3>{gm ? "Resolve an action" : "Request a roll"}</h3>
            <div className="form-grid">
              <label>
                Acting combatant
                <select
                  value={actor}
                  onChange={(e) => {
                    setActor(e.target.value);
                    setMove("");
                  }}
                >
                  <option value="">Select combatant</option>
                  {battle.combatants
                    .filter(
                      (p) =>
                        p.deployed &&
                        p.available_round <= battle.round &&
                        (gm || p.owner_id === playerId),
                    )
                    .map((p) => (
                      <option key={p.id} value={p.id}>
                        {p.name}
                      </option>
                    ))}
                </select>
              </label>
              {
                <>
                  <label>
                    Target
                    <select
                      value={target}
                      onChange={(e) => setTarget(e.target.value)}
                    >
                      <option value="">Select target</option>
                      {battle.combatants
                        .filter((p) => p.deployed)
                        .map((p) => (
                          <option key={p.id} value={p.id}>
                            {p.name}
                          </option>
                        ))}
                    </select>
                  </label>
                </>
              }
              <label>
                Move
                <select value={move} onChange={(e) => setMove(e.target.value)}>
                  <option value="">Other / manual action</option>
                  {(gm && !battle.automatic_validation
                    ? catalog.moves.map((m) => m.Name)
                    : (actorSheet?.moves ?? [])
                  ).map((m) => (
                    <option key={m}>{m}</option>
                  ))}
                </select>
              </label>
              {core && gm && (
                <>
                  <label>
                    GM dice modifier
                    <input
                      type="number"
                      min={-20}
                      max={20}
                      value={poolModifier}
                      onChange={(e) => setPoolModifier(Number(e.target.value))}
                    />
                  </label>
                  <label>
                    GM success modifier
                    <input
                      type="number"
                      min={-20}
                      max={20}
                      value={successModifier}
                      onChange={(e) =>
                        setSuccessModifier(Number(e.target.value))
                      }
                    />
                  </label>
                </>
              )}
              <label>
                Manual dice pool
                <input
                  type="number"
                  min={0}
                  max={100}
                  value={dice}
                  onChange={(e) => setDice(Number(e.target.value))}
                />
              </label>
              {gm && (
                <>
                  <label>
                    Target HP change (negative = damage)
                    <input
                      type="number"
                      value={delta}
                      onChange={(e) => setDelta(Number(e.target.value))}
                    />
                  </label>
                  <label>
                    Add condition (optional)
                    <input
                      value={condition}
                      onChange={(e) => setCondition(e.target.value)}
                      maxLength={100}
                    />
                  </label>
                </>
              )}
            </div>
            {chosenMove && (
              <div className="move-reference">
                <strong>{chosenMove.Name}</strong>
                <p>{chosenMove.Effect}</p>
                <small>
                  Accuracy sources:{" "}
                  {[chosenMove.Accuracy1, chosenMove.Accuracy2]
                    .filter(Boolean)
                    .join(" + ")}{" "}
                  · Damage source: {chosenMove.Damage1 || "—"} · Power:{" "}
                  {chosenMove.Power}
                </small>
              </div>
            )}
            {pending && (
              <div className="notice">
                <strong>
                  {pending.move_name}: damage pending
                  {pending.critical ? " · critical hit" : ""}
                </strong>
                <p>
                  Target:{" "}
                  {
                    battle.combatants.find((p) => p.id === pending.target_id)
                      ?.name
                  }
                  . Resolve reactions before continuing. GM pool and success
                  modifiers also apply to damage. Use the final damage
                  adjustment for shields and other reductions. Fixed or varied
                  damage requires a manual ruling.
                </p>
                {gm && (
                  <label>
                    GM final damage adjustment (negative reduces damage)
                    <input
                      type="number"
                      min={-20}
                      max={20}
                      value={damageModifier}
                      onChange={(e) =>
                        setDamageModifier(Number(e.target.value))
                      }
                    />
                  </label>
                )}
                <button
                  className="primary"
                  disabled={busy}
                  onClick={() =>
                    execute({
                      type: "damage",
                      combatant_id: actor,
                      sequence: pending.sequence,
                      pool_modifier: gm ? poolModifier : 0,
                      success_modifier: gm ? successModifier : 0,
                      damage_modifier: gm ? damageModifier : 0,
                      reason:
                        reason.trim() ||
                        `${pending.move_name} damage after reactions`,
                    })
                  }
                >
                  Roll damage and apply HP
                </button>
                {gm && (
                  <button
                    disabled={busy || !reason.trim()}
                    onClick={() =>
                      execute({
                        type: "cancel_damage",
                        combatant_id: actor,
                        sequence: pending.sequence,
                        reason,
                      })
                    }
                  >
                    Cancel pending damage
                  </button>
                )}
              </div>
            )}
            <label>
              Resolution / roll reason
              <input
                required
                maxLength={1000}
                value={reason}
                onChange={(e) => setReason(e.target.value)}
                placeholder="Explain pool modifiers or the manual ruling…"
              />
            </label>
            {previewNotice && (
              <p className="notice" role="status">
                {previewNotice}
              </p>
            )}
            <div className="inline between roll-controls">
              <label className="checkbox">
                <input
                  type="checkbox"
                  checked={useAction}
                  onChange={(e) => setUseAction(e.target.checked)}
                />{" "}
                Count one action for manual roll
              </label>
              <div className="inline">
                {[
                  "0.3.0-core",
                  "0.4.0-core",
                  "0.5.0-core",
                  "0.6.0-core",
                  "0.7.0-core",
                  "0.8.0-core",
                  "0.9.0-core",
                  "0.10.0-core",
                  "0.11.0-core",
                  "0.15.0-core",
                  "0.14.0-core",
                  "0.13.0-core",
                  "0.12.0-core",
                ].includes(campaign.engine) && (
                  <button
                    disabled={busy || !actor || !move}
                    onClick={async () => {
                      try {
                        const p = await api.previewBattleAction(campaign, {
                          type: "accuracy",
                          combatant_id: actor,
                          target_id:
                            target ||
                            (chosenMove?.Target === "Self" ? actor : null),
                          move_name: move,
                          pool_modifier: gm ? poolModifier : 0,
                          success_modifier: gm ? successModifier : 0,
                          reason: reason.trim() || `${move} preview`,
                        });
                        setPreviewNotice(
                          `${p.dice} host dice. ${p.warnings.join(" ")}`,
                        );
                      } catch (e) {
                        setPreviewNotice(errorOf(e).message);
                      }
                    }}
                  >
                    Preview accuracy dice
                  </button>
                )}
                {core && (
                  <button
                    className="primary"
                    disabled={
                      busy ||
                      !actor ||
                      !move ||
                      !!pending ||
                      ([
                        "0.3.0-core",
                        "0.4.0-core",
                        "0.5.0-core",
                        "0.6.0-core",
                        "0.7.0-core",
                        "0.8.0-core",
                        "0.9.0-core",
                        "0.10.0-core",
                        "0.11.0-core",
                        "0.15.0-core",
                        "0.14.0-core",
                        "0.13.0-core",
                        "0.12.0-core",
                      ].includes(campaign.engine)
                        ? !target && chosenMove?.Target !== "Self"
                        : chosenMove?.Category !== "Support" && !target)
                    }
                    onClick={() =>
                      execute({
                        type: "accuracy",
                        combatant_id: actor,
                        target_id:
                          target ||
                          (chosenMove?.Target === "Self" ? actor : null),
                        move_name: move,
                        pool_modifier: gm ? poolModifier : 0,
                        success_modifier: gm ? successModifier : 0,
                        reason: reason.trim() || `${move} accuracy`,
                      })
                    }
                  >
                    Roll accuracy
                  </button>
                )}
                <button
                  disabled={
                    busy ||
                    !actor ||
                    !reason.trim() ||
                    ([
                      "0.3.0-core",
                      "0.4.0-core",
                      "0.5.0-core",
                      "0.6.0-core",
                      "0.7.0-core",
                      "0.8.0-core",
                      "0.9.0-core",
                      "0.10.0-core",
                      "0.11.0-core",
                      "0.15.0-core",
                      "0.14.0-core",
                      "0.13.0-core",
                      "0.12.0-core",
                    ].includes(campaign.engine) &&
                      battle.automatic_validation)
                  }
                  onClick={() =>
                    execute({
                      type: "roll",
                      combatant_id: actor,
                      dice,
                      use_action: useAction,
                      reason,
                    })
                  }
                >
                  <Dice5 size={16} /> Manual roll
                </button>
                {gm && (
                  <button
                    className="primary"
                    disabled={
                      busy ||
                      !actor ||
                      !target ||
                      !reason.trim() ||
                      ([
                        "0.3.0-core",
                        "0.4.0-core",
                        "0.5.0-core",
                        "0.6.0-core",
                        "0.7.0-core",
                        "0.8.0-core",
                        "0.9.0-core",
                        "0.10.0-core",
                        "0.11.0-core",
                        "0.15.0-core",
                        "0.14.0-core",
                        "0.13.0-core",
                        "0.12.0-core",
                      ].includes(campaign.engine) &&
                        battle.automatic_validation)
                    }
                    onClick={() =>
                      execute({
                        type: "resolve",
                        combatant_id: actor,
                        target_id: target,
                        move_name: move,
                        dice: 0,
                        hp_delta: delta,
                        condition: condition.trim() || null,
                        use_action: false,
                        reason,
                      })
                    }
                  >
                    Apply ruling
                  </button>
                )}
              </div>
            </div>
          </div>
        </>
      )}
      <div className="panel">
        <h3>Battle & campaign log</h3>
        <div className="event-log">
          {[...campaign.events]
            .reverse()
            .slice(0, 100)
            .map((e) => (
              <div className="event" key={e.sequence}>
                <span className="event-number">
                  {String(e.sequence).padStart(3, "0")}
                </span>
                <div>
                  <p>{e.description}</p>
                  {e.dice.length > 0 && (
                    <div className="dice-row">
                      {e.dice.map((d, i) => (
                        <span key={i}>{d}</span>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            ))}
          {!campaign.events.length && (
            <p className="muted">
              Events will appear here as your campaign progresses.
            </p>
          )}
        </div>
      </div>
      {campaign.finished_battles.length > 0 && (
        <div className="panel">
          <h3>Completed encounters</h3>
          {[...campaign.finished_battles].reverse().map((b) => (
            <details key={b.id}>
              <summary>
                {b.combatants.map((c) => c.name).join(" · ")} — {b.round} rounds
              </summary>
              {b.combatants.map((c) => (
                <div className="list-row" key={c.id}>
                  <strong>{c.name}</strong>
                  <span>
                    Final HP {c.hp}/{c.max_hp} ·{" "}
                    {c.conditions.join(", ") || "No conditions"}
                  </span>
                </div>
              ))}
            </details>
          ))}
        </div>
      )}
      {override && (
        <div className="overlay">
          <div className="modal">
            <div className="modal-head">
              <h2>Adjust {override.name}</h2>
              <button onClick={() => setOverride(null)}>Close</button>
            </div>
            <div className="editor-body">
              {error && (
                <div className="error" role="alert">
                  {error.message}
                </div>
              )}
              <div className="form-grid">
                {(["hp", "actions", "reactions", "initiative"] as const).map(
                  (k) => (
                    <label key={k}>
                      {k}
                      <input
                        type="number"
                        value={override[k]}
                        min={k === "initiative" ? undefined : 0}
                        max={k === "hp" ? override.max_hp : undefined}
                        onChange={(e) =>
                          setOverride({
                            ...override,
                            [k]: Number(e.target.value),
                          })
                        }
                      />
                    </label>
                  ),
                )}
              </div>
              <label>
                Conditions (comma separated)
                <input
                  value={override.conditions.join(", ")}
                  onChange={(e) =>
                    setOverride({
                      ...override,
                      conditions: e.target.value
                        .split(",")
                        .map((s) => s.trim())
                        .filter(Boolean),
                    })
                  }
                />
              </label>
              <label>
                Reason
                <input
                  required
                  value={overrideReason}
                  onChange={(e) => setOverrideReason(e.target.value)}
                />
              </label>
            </div>
            <div className="modal-actions">
              <button
                className="primary"
                disabled={busy || !overrideReason.trim()}
                onClick={async () => {
                  const accepted = await execute({
                    type: "override",
                    combatant_id: override.id,
                    hp: override.hp,
                    conditions: override.conditions,
                    actions: override.actions,
                    reactions: override.reactions,
                    initiative: override.initiative,
                    reason: overrideReason,
                  });
                  if (accepted) setOverride(null);
                }}
              >
                Apply adjustment
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
