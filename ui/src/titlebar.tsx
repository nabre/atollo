// Title bar of the app window, drawn by the interface: the window has no
// Windows frame (src-tauri/src/main.rs, `decorations(false)`). The empty part
// drags the window and a double click maximizes it (`data-tauri-drag-region`);
// the three buttons are those of Windows. Only the main window has the
// permissions they use (capabilities/main.json).
import { useEffect, useState } from "react";
import { VscChromeClose, VscChromeMaximize, VscChromeMinimize, VscChromeRestore } from "react-icons/vsc";
import { getCurrentWindow, type Window } from "@tauri-apps/api/window";
import { browserPreview } from "./api";
import { t } from "./i18n";
import logo from "./assets/logo-32.png";

/** The window, or null in the browser preview and in the tests. */
function current(): Window | null {
  if (browserPreview) return null;
  try {
    return getCurrentWindow();
  } catch {
    return null;
  }
}

function act(f: (w: Window) => Promise<void>) {
  const w = current();
  if (w) f(w).catch(() => undefined);
}

export function TitleBar({ title }: { title: string | undefined }) {
  const [maximized, setMaximized] = useState(false);
  useEffect(() => {
    const w = current();
    if (!w) return;
    let stop: (() => void) | null = null;
    let gone = false;
    const read = () => {
      w.isMaximized().then(
        (m) => {
          if (!gone) setMaximized(m);
        },
        () => undefined,
      );
    };
    read();
    w.onResized(read).then(
      (f) => {
        if (gone) f();
        else stop = f;
      },
      () => undefined,
    );
    return () => {
      gone = true;
      stop?.();
    };
  }, []);
  const button =
    "flex h-full w-11 items-center justify-center text-vsc-fg hover:bg-vsc-list-hover focus-visible:bg-vsc-list-hover";
  return (
    <div
      data-tauri-drag-region
      className="flex h-[30px] shrink-0 items-center border-b border-vsc-border bg-vsc-activitybar select-none"
    >
      <span data-tauri-drag-region className="flex items-center gap-2 pl-3 text-[12px]">
        <img data-tauri-drag-region src={logo} alt="" width={16} height={16} draggable={false} />
        <span data-tauri-drag-region className="font-semibold">
          Dev-Kit
        </span>
        {title && (
          <span data-tauri-drag-region className="text-vsc-desc">
            — {title}
          </span>
        )}
      </span>
      <span data-tauri-drag-region className="h-full flex-1" />
      <button
        type="button"
        aria-label={t("Riduci a icona")}
        title={t("Riduci a icona")}
        className={button}
        onClick={() => {
          act((w) => w.minimize());
        }}
      >
        <VscChromeMinimize size={16} aria-hidden="true" />
      </button>
      <button
        type="button"
        aria-label={maximized ? t("Ripristina") : t("Ingrandisci")}
        title={maximized ? t("Ripristina") : t("Ingrandisci")}
        className={button}
        onClick={() => {
          act((w) => w.toggleMaximize());
        }}
      >
        {maximized ? (
          <VscChromeRestore size={16} aria-hidden="true" />
        ) : (
          <VscChromeMaximize size={16} aria-hidden="true" />
        )}
      </button>
      <button
        type="button"
        aria-label={t("Chiudi")}
        title={t("Chiudi")}
        className="flex h-full w-11 items-center justify-center text-vsc-fg hover:bg-[#e81123] hover:text-white focus-visible:bg-[#e81123] focus-visible:text-white"
        onClick={() => {
          // Same path as the Windows button: the app asks first if something is open.
          act((w) => w.close());
        }}
      >
        <VscChromeClose size={16} aria-hidden="true" />
      </button>
    </div>
  );
}
