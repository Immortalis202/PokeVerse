import { useState } from "react";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";
import type { SpecialCommand } from "./bindings/SpecialCommand";
import { battleSheet } from "./sheets";
import { api, errorOf } from "./api";

const types = ["Normal", "Bug", "Dark", "Dragon", "Electric", "Fairy", "Fight", "Fire", "Flying", "Ghost", "Grass", "Ground", "Ice", "Poison", "Psychic", "Rock", "Steel", "Water"];
export default function JudgmentControls({ campaign, playerId, execute, disabled }: {
  campaign: Campaign; playerId: string; execute: (action: Action) => Promise<boolean>; disabled: boolean;
}) {
  const [actor, setActor] = useState("");
  const [target, setTarget] = useState("");
  const [moveType, setMoveType] = useState("");
  const [power, setPower] = useState("");
  const [unique, setUnique] = useState("");
  const [reason, setReason] = useState("");
  const [notice, setNotice] = useState("");
  const b = campaign.battle, f = b?.flow, state = f?.completion;
  if (!b?.active || !f || !state?.blocks.includes("MOVE-A47")) return null;
  const gm = campaign.gm_id === playerId;
  const eligible = b.combatants.filter((p) => p.deployed && p.hp > 0 && battleSheet(campaign, p.id)?.moves.includes("Judgment"));
  const prepared = state.judgment_prepared?.[actor];
  const profile = prepared?.turn === f.turn ? prepared : undefined;
  const q = f.queue[0];
  const cast = q?.move_name === "Judgment" ? state.judgment_casts?.[q.sequence] : undefined;
  const send = (command: SpecialCommand, why: string) => execute({ type: "special_move", command, reason: why });
  return <section aria-label="Judgment rulings">
    <h4>Judgment</h4>
    <p>The GM chooses Type, Power and extra effects. Damage ignores defenses, ordinary immunities, shields and decoys; Accuracy and normal action eligibility still apply. Judgment cannot be Clashed.</p>
    <label>Judgment user<select value={actor} onChange={(e) => setActor(e.target.value)}><option value="">Choose user</option>{eligible.map((p) => <option key={p.id} value={p.id}>{p.name}</option>)}</select></label>
    {gm && <>
      <label>Judgment Type<select value={moveType} onChange={(e) => setMoveType(e.target.value)}><option value="">Choose Type</option>{types.map((name) => <option key={name}>{name}</option>)}</select></label>
      <label>Judgment Power<input type="number" min={0} max={100} value={power} onChange={(e) => setPower(e.target.value)} /></label>
      <label>Unique protection ruling<select value={unique} onChange={(e) => setUnique(e.target.value)}><option value="">Record GM interpretation</option><option value="preserve">Preserve Unique target-ability protections</option><option value="bypass">Allow Judgment to bypass Unique target abilities</option></select></label>
      <label>Judgment ruling reason<input value={reason} onChange={(e) => setReason(e.target.value)} /></label>
      <button disabled={disabled || !actor || !f.started || !moveType || power.trim() === "" || !Number.isInteger(Number(power)) || Number(power) < 0 || Number(power) > 100 || !unique || !reason.trim() || f.queue.some((q) => q.actor_id === actor && q.move_name === "Judgment")} onClick={() => send({ kind: "PrepareJudgment", combatant_id: actor, move_type: moveType, power: Number(power), bypass_unique: unique === "bypass" }, reason.trim())}>Save Judgment parameters</button>
    </>}
    {profile && <>
      <p>Prepared: {profile.move_type}, Power {profile.power}. Unique target abilities will be {profile.bypass_unique ? "bypassed" : "preserved"} by the GM ruling. This preparation is valid for the current turn or reaction.</p>
      <label>Judgment preview target<select value={target} onChange={(e) => setTarget(e.target.value)}><option value="">Choose foe</option>{b.combatants.filter((p) => p.deployed && p.hp > 0 && p.side !== b.combatants.find((p) => p.id === actor)?.side).map((p) => <option key={p.id} value={p.id}>{p.name}</option>)}</select></label>
      <button disabled={disabled || !target || !actor} onClick={async () => { try {
        const preview = await api.previewBattleAction(campaign, { type: "accuracy", combatant_id: actor, target_id: target, move_name: "Judgment", pool_modifier: 0, success_modifier: 0, reason: "Preview Judgment using the saved GM parameters" });
        setNotice(`${preview.dice} Accuracy dice. ${preview.warnings.join(" ")}`);
      } catch (error) { setNotice(errorOf(error).message); } }}>Preview Judgment Accuracy</button>
      <p>Cast Judgment with the normal battle controls after checking the saved ruling.</p>
    </>}
    {prepared && !profile && <p>The previous preparation expired; the GM must renew it for this turn.</p>}
    {cast && <>
      <p>Saved cast: {cast.profile.move_type}, Power {cast.profile.power}.</p>
      {!q.damage_done || f.window ? <p>Resolve reactions and damage before the GM decides extra effects.</p> : <>
        {cast.recording ? <p>Use the normal GM stat, condition, battlefield, equipment or combatant controls to apply extra effects. Actual mechanical changes are recorded; a note alone does not complete an extra effect.</p> : <p>The GM can choose no extra mechanical effects or start recording applied consequences.</p>}
        {cast.consequences.length > 0 && <ul>{cast.consequences.map((effect, i) => <li key={i}>{effect.description}</li>)}</ul>}
        {gm && <>
          {!cast.recording && <button disabled={disabled || !reason.trim()} onClick={() => send({ kind: "BeginJudgmentEffects", sequence: q.sequence }, reason.trim())}>Start recording extra effects</button>}
          <button disabled={disabled || !reason.trim() || cast.consequences.length > 0} onClick={() => send({ kind: "RuleJudgment", sequence: q.sequence, no_extra_effects: true }, reason.trim())}>Finish: no extra mechanical effects</button>
          <button disabled={disabled || !reason.trim() || !cast.recording || cast.consequences.length === 0} onClick={() => send({ kind: "RuleJudgment", sequence: q.sequence, no_extra_effects: false }, reason.trim())}>Finish with applied extra effects</button>
        </>}
      </>}
    </>}
    {notice && <p role="status">{notice}</p>}
  </section>;
}
