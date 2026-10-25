// Title bar of the app window, drawn by the interface: the window has no
// Windows frame (tauri/src/main.rs, `decorations(false)`). The empty part
// drags the window and a double click maximizes it (`data-tauri-drag-region`);
// the three buttons are those of Windows, native buttons as VSCodium's
// window controls (no vscode-elements component draws them). Only the main window has the
// permissions they use (capabilities/main.json). Its measures are VSCodium's
// without the command center: 30px high, window controls 46px wide and as
// high as the bar. No border under it: with the modern UI VSCodium sets no
// titleBar.border (TitlebarPart.updateStyles), the bar is one with the
// workbench's color.
import { useEffect, useState } from "react";
import { browserPreview, watchMaximized, windowClose, windowMinimize, windowToggleMaximize } from "../ipc";
import { t } from "../i18n";
import { BrandWordmark } from "../shared/brand";
import { Badge } from "../shared/tags";
import { Codicon } from "../workbench";

export function TitleBar({ title }: { title: string | undefined }) {
  const [maximized, setMaximized] = useState(false);
  useEffect(() => watchMaximized(setMaximized), []);
  const button =
    "flex h-full w-vsc-window-control items-center justify-center text-vsc-titlebar-fg hover:bg-vsc-toolbar-hover focus-visible:bg-vsc-toolbar-hover";
  return (
    <div
      data-tauri-drag-region
      className="flex h-vsc-titlebar shrink-0 items-center bg-vsc-titlebar text-vsc-titlebar-fg select-none"
    >
      <span data-tauri-drag-region className="flex items-center gap-vsc-size80 pl-vsc-size120 text-vsc-body-small">
        <BrandWordmark height={14} />
        {title && (
          <span data-tauri-drag-region className="text-vsc-desc">
            — {title}
          </span>
        )}
      </span>
      <span data-tauri-drag-region className="h-full flex-1" />
      {browserPreview && <Badge className="mr-vsc-size80">{t("Anteprima nel browser: dati di esempio")}</Badge>}
      <button
        type="button"
        aria-label={t("Riduci a icona")}
        title={t("Riduci a icona")}
        className={button}
        onClick={() => {
          windowMinimize();
        }}
      >
        <Codicon name="chrome-minimize" size={16} />
      </button>
      <button
        type="button"
        aria-label={maximized ? t("Ripristina") : t("Ingrandisci")}
        title={maximized ? t("Ripristina") : t("Ingrandisci")}
        className={button}
        onClick={() => {
          windowToggleMaximize();
        }}
      >
        {maximized ? <Codicon name="chrome-restore" size={16} /> : <Codicon name="chrome-maximize" size={16} />}
      </button>
      <button
        type="button"
        aria-label={t("Chiudi")}
        title={t("Chiudi")}
        // VSCodium's close hover, a fixed color in its CSS: not a theme color.
        className="flex h-full w-vsc-window-control items-center justify-center text-vsc-titlebar-fg hover:bg-[rgba(232,17,35,0.9)] hover:text-white focus-visible:bg-[rgba(232,17,35,0.9)] focus-visible:text-white"
        onClick={() => {
          windowClose();
        }}
      >
        <Codicon name="chrome-close" size={16} />
      </button>
    </div>
  );
}
