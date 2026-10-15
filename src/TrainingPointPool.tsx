import { useEffect, useRef, useState } from "react";
import { api, errorOf } from "./api";
import type { Sheet } from "./bindings/Sheet";
import type { PointTransferRequest } from "./bindings/PointTransferRequest";
import type { PointTransferResult } from "./bindings/PointTransferResult";
export default function TrainingPointPool({
  trainer,
  pokemon,
  catalogHash,
  onChanged,
  disabled,
}: {
  trainer: Sheet;
  pokemon: Sheet[];
  catalogHash: string;
  onChanged: () => Promise<void>;
  disabled: boolean;
}) {
  const [recipient, setRecipient] = useState(""),
    [amount, setAmount] = useState(1),
    [reason, setReason] = useState("Player distribution");
  const [preview, setPreview] = useState<{
    request: PointTransferRequest;
    result: PointTransferResult;
  } | null>(null);
  const [busy, setBusy] = useState(false),
    [error, setError] = useState(""),
    [notice, setNotice] = useState("");
  const requestEpoch = useRef(0);
  useEffect(() => {
    requestEpoch.current += 1;
    setPreview(null);
  }, [
    trainer.revision,
    catalogHash,
    pokemon.map((p) => `${p.id}:${p.revision}`).join("|"),
  ]);
  const balance = trainer.workflow?.training_points ?? 0;
  async function review() {
    const target = pokemon.find((p) => p.id === recipient);
    if (!target) return;
    setBusy(true);
    setError("");
    setNotice("");
    const epoch = requestEpoch.current;
    try {
      const request: PointTransferRequest = {
        kind: "point_transfer",
        operation_id: crypto.randomUUID(),
        trainer_id: trainer.id,
        pokemon_id: target.id,
        trainer_revision: trainer.revision,
        pokemon_revision: target.revision,
        catalog_hash: catalogHash,
        amount,
        reason,
      };
      const result = await api.previewPointTransfer(request);
      if (epoch === requestEpoch.current) setPreview({ request, result });
    } catch (e) {
      setError(errorOf(e).message);
    } finally {
      setBusy(false);
    }
  }
  async function confirm() {
    if (!preview) return;
    setBusy(true);
    setError("");
    try {
      await api.commitPointTransfer(preview.request);
      setPreview(null);
      await onChanged();
      setNotice(
        "Points distributed. Synchronize the linked trainer and Pokémon when you reconnect.",
      );
    } catch (e) {
      setError(errorOf(e).message);
    } finally {
      setBusy(false);
    }
  }
  return (
    <section
      className="panel trainer-point-pool"
      aria-label="Trainer Training Point pool"
    >
      <div className="inline between">
        <h3>Training Point pool</h3>
        <strong>{balance} TP available</strong>
      </div>
      <p>
        The GM awards your trainer. Distribute accepted points to your Pokémon
        offline; no further GM approval is needed. Transfers are one-way and
        one-for-one.
      </p>
      {error && (
        <p className="error" role="alert">
          {error}
        </p>
      )}
      {notice && (
        <p className="notice" role="status">
          {notice}
        </p>
      )}
      {!preview ? (
        <>
          <div className="form-grid">
            <label>
              Recipient Pokémon
              <select
                aria-label="Training Point recipient"
                value={recipient}
                disabled={busy}
                onChange={(e) => {
                  setRecipient(e.target.value);
                  setError("");
                }}
              >
                <option value="">Choose Pokémon</option>
                {pokemon.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name} · {p.workflow?.training_points ?? 0} TP
                  </option>
                ))}
              </select>
            </label>
            <label>
              Points to distribute
              <input
                type="number"
                aria-label="Points to distribute"
                min={1}
                max={Math.min(balance, 100000)}
                step={1}
                value={amount}
                disabled={busy}
                onChange={(e) => setAmount(Number(e.target.value))}
              />
            </label>
            <label>
              Reason
              <input
                value={reason}
                maxLength={1000}
                disabled={busy}
                onChange={(e) => setReason(e.target.value)}
              />
            </label>
          </div>
          <button
            disabled={
              disabled ||
              busy ||
              trainer.archived ||
              !recipient ||
              !Number.isInteger(amount) ||
              amount <= 0 ||
              amount > balance ||
              amount > 100000 ||
              !reason.trim()
            }
            onClick={review}
          >
            Review distribution
          </button>
        </>
      ) : (
        <>
          <h4>Review distribution</h4>
          <p>
            {amount} TP from {trainer.name} to {preview.result.pokemon.name}.
          </p>
          <p>
            Trainer pool: {balance} →{" "}
            {preview.result.trainer.workflow?.training_points} TP. Pokémon
            balance:{" "}
            {pokemon.find((p) => p.id === recipient)?.workflow
              ?.training_points ?? 0}{" "}
            → {preview.result.pokemon.workflow?.training_points} TP.
          </p>
          <p>
            No points are consumed by transfer and no character stats change.
          </p>
          <div className="toolbar">
            <button disabled={busy} onClick={() => setPreview(null)}>
              Back
            </button>
            <button
              className="primary"
              disabled={disabled || busy}
              onClick={confirm}
            >
              Confirm distribution
            </button>
          </div>
        </>
      )}
    </section>
  );
}
