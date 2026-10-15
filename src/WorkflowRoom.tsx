import { useEffect, useState } from "react";
import type { Campaign } from "./bindings/Campaign";
import type { WorkflowRequest } from "./bindings/WorkflowRequest";
import type { WorkflowPreview } from "./bindings/WorkflowPreview";
import type { Action } from "./bindings/Action";
import { api, errorOf } from "./api";
export default function WorkflowRoom({
  campaign,
  drafts,
  playerId,
  execute,
  pull,
  disabled,
}: {
  campaign: Campaign;
  drafts: WorkflowRequest[];
  playerId: string;
  execute: (a: Action) => Promise<boolean>;
  pull: (ids: string[]) => Promise<void>;
  disabled: boolean;
}) {
  const [reason, setReason] = useState("");
  const [profile, setProfile] = useState(campaign.rules_profile);
  const [previews, setPreviews] = useState<Record<string, WorkflowPreview>>({});
  const [error, setError] = useState("");
  const gm = campaign.gm_id === playerId;
  useEffect(() => {
    setProfile(campaign.rules_profile);
    let cancelled = false;
    Promise.all(
      campaign.proposals.map(
        async (p) =>
          [
            p.id,
            await api.previewCampaignWorkflow(campaign, p.request),
          ] as const,
      ),
    )
      .then((values) => {
        if (!cancelled) setPreviews(Object.fromEntries(values));
      })
      .catch((e) => {
        if (!cancelled) setError(errorOf(e).message);
      });
    return () => {
      cancelled = true;
    };
  }, [campaign]);
  return (
    <div className="panel">
      <h2>Creation, progression & GM review</h2>
      <p>
        Verified purchases work offline. Submit narrative drafts here; accepted
        campaign sheets change only after GM approval. Synchronize the trainer
        first for a new Pokémon.
      </p>
      {error && <div className="error">{error}</div>}
      {drafts.map((d) => (
        <div className="list-row" key={d.operation_id}>
          <div>
            <strong>{d.sheet.name || "Unnamed character"}</strong>
            <small>
              {d.kind} · {d.reason}
            </small>
          </div>
          <button
            disabled={
              disabled ||
              campaign.proposals.some((p) => p.id === d.operation_id)
            }
            onClick={() => execute({ type: "propose_workflow", request: d })}
          >
            Submit draft
          </button>
        </div>
      ))}
      {gm && (
        <label>
          GM decision / profile-change reason
          <input
            maxLength={1000}
            value={reason}
            onChange={(e) => setReason(e.target.value)}
          />
        </label>
      )}
      {campaign.proposals.map((p) => {
        const preview = previews[p.id];
        return (
          <details key={p.id} open>
            <summary>
              {p.request.sheet.name} · {p.request.kind}
            </summary>
            <p>{p.request.reason}</p>
            <p>
              Sheet revision {p.request.expected_revision} · Trainer revision{" "}
              {p.request.trainer_revision ?? "—"}
            </p>
            {preview && (
              <>
                <p>
                  Cost {preview.cost} TP · Result {preview.sheet.rank} · HP{" "}
                  {preview.sheet.hp}/{preview.sheet.max_hp} · TP{" "}
                  {preview.sheet.workflow?.training_points}
                </p>
                <p>
                  Attributes:{" "}
                  {Object.entries(preview.sheet.stats)
                    .map(([k, v]) => `${k} ${v}`)
                    .join(" · ")}
                </p>
                <p>
                  Skills:{" "}
                  {Object.entries(preview.sheet.skills)
                    .map(([k, v]) => `${k} ${v}`)
                    .join(" · ")}
                </p>
                <p>
                  Moves: {preview.sheet.moves.join(", ") || "none"} · Ability:{" "}
                  {preview.sheet.ability}
                </p>
                <p>
                  Inventory:{" "}
                  {Object.entries(preview.sheet.inventory)
                    .map(([k, v]) => `${k} ${v}`)
                    .join(" · ") || "none"}
                </p>
                {preview.errors.map((e, i) => (
                  <p className="error" key={i}>
                    {e}
                  </p>
                ))}
                {preview.approvals.map((e, i) => (
                  <p key={i}>{e}</p>
                ))}
              </>
            )}
            {gm && (
              <div className="inline">
                <button
                  disabled={
                    disabled ||
                    !reason.trim() ||
                    !preview ||
                    !!preview.errors.length
                  }
                  onClick={() =>
                    execute({
                      type: "approve_workflow",
                      proposal_id: p.id,
                      accept: true,
                      reason,
                    })
                  }
                >
                  Accept reviewed result
                </button>
                <button
                  disabled={disabled || !reason.trim()}
                  onClick={() =>
                    execute({
                      type: "approve_workflow",
                      proposal_id: p.id,
                      accept: false,
                      reason,
                    })
                  }
                >
                  Reject
                </button>
              </div>
            )}
          </details>
        );
      })}
      <button
        disabled={disabled}
        onClick={() =>
          pull(
            Object.values(campaign.sheets)
              .filter(
                (s): s is NonNullable<typeof s> =>
                  !!s && s.owner_id === playerId,
              )
              .map((s) => s.id),
          )
        }
      >
        Pull accepted results into library
      </button>
      {gm && (
        <details>
          <summary>Campaign rules profile</summary>
          <p>
            Change between encounters. Characters with another profile require a
            reviewed rebuild.
          </p>
          <div className="form-grid">
            <label>
              Defenses
              <select
                value={profile.defenses}
                onChange={(e) =>
                  setProfile({ ...profile, defenses: e.target.value })
                }
              >
                <option value="split">Vitality DEF / Insight S.DEF</option>
                <option value="vitality">Vitality for both defenses</option>
              </select>
            </label>
            <label>
              HP rule
              <select
                value={profile.hp}
                onChange={(e) => setProfile({ ...profile, hp: e.target.value })}
              >
                <option value="vitality">Base HP + Vitality</option>
                <option value="highest_limit">
                  Attribute with higher species limit
                </option>
              </select>
            </label>
          </div>
          <button
            disabled={disabled || !!campaign.battle?.active || !reason.trim()}
            onClick={() =>
              execute({ type: "set_rules_profile", profile, reason })
            }
          >
            Set campaign profile
          </button>
        </details>
      )}
    </div>
  );
}
