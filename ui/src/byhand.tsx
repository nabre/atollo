// A download that failed, done in the browser instead (src-tauri/src/downloads.rs,
// DECISIONS.md D-82): the link to the file, and a box where the downloaded
// file is dropped or picked. The app keeps it in data\downloads and tries
// again; the file is used only with the checksum the source publishes.
import { useCallback, useEffect, useRef, useState } from "react";
import { VscCloudDownload } from "react-icons/vsc";
import { downloadsAdd, downloadsPick, onFileDrag, setupByHandOpen, type ByHand } from "./api";
import { t } from "./i18n";
import { Link } from "./workbench";

export function ByHandBox({
  id,
  name,
  byHand,
  disabled,
  onRetry,
}: {
  /** The step that failed: component and step name. */
  id: string;
  name: string;
  byHand: ByHand;
  /** While another installation runs: the file is not taken. */
  disabled: boolean;
  /** Installs again, once the file is kept. */
  onRetry: () => Promise<void>;
}) {
  const box = useRef<HTMLDivElement>(null);
  const [over, setOver] = useState(false);
  const [added, setAdded] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  // The latest props, for the drop listener registered once.
  const latest = useRef({ disabled, onRetry });
  useEffect(() => {
    latest.current = { disabled, onRetry };
  }, [disabled, onRetry]);

  const take = useCallback(async (get: () => Promise<string[]>) => {
    setError(null);
    try {
      const names = await get();
      if (names.length === 0) return;
      setAdded(names);
      await latest.current.onRetry();
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    let stop: (() => void) | undefined;
    let alive = true;
    const inside = (x: number, y: number) => {
      const r = box.current?.getBoundingClientRect();
      return r !== undefined && x >= r.left && x <= r.right && y >= r.top && y <= r.bottom;
    };
    onFileDrag((d) => {
      if (d.kind === "leave") {
        setOver(false);
        return;
      }
      const hit = inside(d.x, d.y) && !latest.current.disabled;
      if (d.kind === "over") {
        setOver(hit);
        return;
      }
      setOver(false);
      if (hit) void take(() => downloadsAdd(d.paths));
    }).then(
      (unlisten) => {
        if (alive) stop = unlisten;
        else unlisten();
      },
      () => undefined,
    );
    return () => {
      alive = false;
      stop?.();
    };
  }, [take]);

  return (
    <div
      ref={box}
      className={`mt-1.5 rounded border border-dashed px-3 py-2 text-[12px] ${
        over ? "border-vsc-focus bg-vsc-focus/15" : "border-vsc-border"
      }`}
    >
      <p>
        <VscCloudDownload className="mr-1 inline" size={14} aria-hidden="true" />
        {t("Scaricalo dal browser:")}{" "}
        <Link
          onClick={() => {
            setupByHandOpen(id, name).catch((e: unknown) => {
              setError(String(e));
            });
          }}
        >
          {byHand.fileName}
        </Link>
      </p>
      <p className="mt-0.5 text-vsc-desc">
        {t("Poi trascina qui il file scaricato o")}{" "}
        {disabled ? t("sfoglia…") : <Link onClick={() => void take(downloadsPick)}>{t("sfoglia…")}</Link>}{" "}
        {t("Il kit lo conserva e lo usa solo se il suo SHA-256 è quello pubblicato.")}
      </p>
      {added.length > 0 && (
        <p className="mt-0.5 text-vsc-success">{t("Aggiunto {0}: nuovo tentativo.", added.join(", "))}</p>
      )}
      {error && <p className="mt-0.5 text-vsc-error select-text">{error}</p>}
    </div>
  );
}
