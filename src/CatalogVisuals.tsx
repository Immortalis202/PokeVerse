import { useEffect, useState } from "react";
export function Sprite({
  entry,
  trim = false,
}: {
  entry: Record<string, unknown>;
  trim?: boolean;
}) {
  const [failed, setFailed] = useState(false);
  const [frame, setFrame] = useState<{
    source: string;
    box: string;
    width: number;
    height: number;
  } | null>(null);
  useEffect(() => {
    setFailed(false);
  }, [entry.Image]);
  const source = `/sprites/${entry.Image}`;
  if (!entry.Image || failed)
    return (
      <span
        className="sprite-placeholder"
        role="img"
        aria-label={`Sprite unavailable for ${entry.Name}`}
      >
        ?
      </span>
    );
  if (trim && frame?.source === source)
    return (
      <svg
        className="compendium-sprite"
        role="img"
        aria-label={String(entry.Name)}
        viewBox={frame.box}
      >
        <image
          href={source}
          width={frame.width}
          height={frame.height}
          onError={() => setFailed(true)}
        />
      </svg>
    );
  return (
    <img
      className="compendium-sprite"
      src={source}
      alt={String(entry.Name)}
      onError={() => setFailed(true)}
      onLoad={(event) => {
        if (!trim) return;
        // Frame the visible sprite in the portrait; the bundled artwork remains untouched.
        const image = event.currentTarget;
        const canvas = document.createElement("canvas");
        canvas.width = image.naturalWidth;
        canvas.height = image.naturalHeight;
        const context = canvas.getContext("2d");
        if (!context) return;
        try {
          context.drawImage(image, 0, 0);
          const pixels = context.getImageData(
            0,
            0,
            canvas.width,
            canvas.height,
          ).data;
          let left = canvas.width,
            top = canvas.height,
            right = -1,
            bottom = -1;
          for (let y = 0; y < canvas.height; y++)
            for (let x = 0; x < canvas.width; x++)
              if (pixels[(y * canvas.width + x) * 4 + 3] > 0) {
                left = Math.min(left, x);
                right = Math.max(right, x);
                top = Math.min(top, y);
                bottom = Math.max(bottom, y);
              }
          if (right < 0) return;
          left = Math.max(0, left - 2);
          top = Math.max(0, top - 2);
          right = Math.min(canvas.width - 1, right + 2);
          bottom = Math.min(canvas.height - 1, bottom + 2);
          setFrame({
            source,
            box: `${left} ${top} ${right - left + 1} ${bottom - top + 1}`,
            width: canvas.width,
            height: canvas.height,
          });
        } catch {
          /* Keep the ordinary image if framing is unavailable. */
        }
      }}
    />
  );
}
export function Types({ entry }: { entry: Record<string, unknown> }) {
  return (
    <span className="compendium-types">
      {[entry.Type1 ?? entry.Type, entry.Type2]
        .filter(Boolean)
        .map((type, i) => (
          <span className="type-pill" key={i}>
            {String(type)}
          </span>
        ))}
    </span>
  );
}
