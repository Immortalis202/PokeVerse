import type { Campaign } from "./bindings/Campaign";
import type { Action } from "./bindings/Action";

export default function MoveCompletionControls({
  campaign,
  playerId,
  execute,
  disabled,
}: {
  campaign: Campaign;
  playerId: string;
  execute: (a: Action) => Promise<boolean>;
  disabled: boolean;
}) {
  const b = campaign.battle,
    f = b?.flow;
  if (!b?.active || !f?.special.enabled) return null;
  const entries = f.special.effects
    .map((effect, index) => ({ effect, index }))
    .filter(
      ({ effect: e }) =>
        e.name.startsWith("move-block:") ||
        e.name.startsWith("ongoing:") ||
        e.name.startsWith("accuracy-loss:"),
    );
  if (!entries.length) return null;
  const owned = (id: string) =>
    campaign.gm_id === playerId ||
    b.combatants.find((p) => p.id === id)?.owner_id === playerId;
  return (
    <section aria-label="Move effects">
      <h4>Move effects</h4>
      {entries.map(({ effect: e, index }) => {
        const ongoing = e.name.startsWith("ongoing:"),
          accuracy = e.name.startsWith("accuracy-loss:"),
          move = e.name.replace(/^(move-block:|ongoing:|accuracy-loss:)/, "");
        const target = b.combatants.find((p) => p.id === e.target);
        const due =
          ongoing &&
          f.battlefield.end_round === b.round &&
          !f.queue.length &&
          !f.order.length &&
          e.started < b.round &&
          (target?.hp ?? 0) > 0;
        return (
          <div key={index} className="notice">
            {target?.name ?? "Registered participant"}:{" "}
            {accuracy
              ? `${move}: remove ${e.amount} Accuracy successes`
              : ongoing
                ? `${move} ongoing damage`
                : `blocked by ${move}`}{" "}
            {e.rounds == null ? "(scene)" : `(${e.rounds} rounds remaining)`}
            {due && owned(e.source) && (
              <button
                disabled={disabled}
                onClick={() =>
                  execute({
                    type: "special_move",
                    command: { kind: "Tick", index },
                    reason: `Resolve ${move} round-end damage`,
                  })
                }
              >
                Roll ongoing damage
              </button>
            )}
          </div>
        );
      })}
    </section>
  );
}
