import { useEffect, useState } from "react";
import { call, errorOf } from "./api";
import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";
import { sheetValues } from "./sheets";

export default function GmAssignment({
  campaign,
  disabled,
  execute,
}: {
  campaign: Campaign;
  disabled: boolean;
  execute: (action: Action) => Promise<boolean>;
}) {
  const host = campaign.host_id || campaign.gm_id;
  const [members, setMembers] = useState<string[]>([host]);
  const [selected, setSelected] = useState(campaign.gm_id);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  useEffect(() => {
    setSelected(campaign.gm_id);
  }, [campaign.gm_id]);
  useEffect(() => {
    let live = true;
    call<string[]>("room_members")
      .then((ids) => {
        if (live) setMembers(ids);
      })
      .catch((e) => {
        if (live) setError(errorOf(e).message);
      });
    return () => {
      live = false;
    };
  }, [campaign.id]);
  return (
    <div className="panel">
      <h3>Choose the game master</h3>
      <p>
        The host runs validation, dice rolls and campaign storage. The appointed
        GM manages battles and rewards. You can change the GM during a session
        or take the role back.
      </p>
      <label>
        Game master
        <select
          value={selected}
          disabled={disabled || loading}
          onChange={(e) => setSelected(e.target.value)}
        >
          {[...new Set([...members, campaign.gm_id])].map((id) => {
            const names = sheetValues(campaign.sheets)
              .filter((s) => s.owner_id === id && s.kind === "trainer")
              .map((s) => s.name)
              .join(", ");
            return (
              <option key={id} value={id}>
                {id === host
                  ? "You (host)"
                  : `${names || "Room member"} · ${id}`}
              </option>
            );
          })}
        </select>
      </label>
      <div className="inline">
        <button
          disabled={disabled || loading}
          onClick={async () => {
            setLoading(true);
            setError("");
            try {
              setMembers(await call<string[]>("room_members"));
            } catch (e) {
              setError(errorOf(e).message);
            } finally {
              setLoading(false);
            }
          }}
        >
          Refresh members
        </button>
        <button
          disabled={
            disabled ||
            loading ||
            selected === campaign.gm_id ||
            !members.includes(selected)
          }
          onClick={() => execute({ type: "assign_gm", player_id: selected })}
        >
          Appoint GM
        </button>
      </div>
      <small>
        Players must join once before they can be appointed. Refresh after a
        player joins; previously authenticated members remain selectable for
        later sessions.
      </small>
      {error && <p role="alert">{error}</p>}
    </div>
  );
}
