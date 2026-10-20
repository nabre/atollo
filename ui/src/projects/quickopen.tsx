// Quick Open of Projects (Ctrl+P), as the editor's: a box at the top of the
// window, a project found by its name, platform or languages, Enter opens its
// page, Shift+Enter starts it; under the projects, the ways to make a new one.
import { useRef, useState, type KeyboardEvent, type ReactNode } from "react";
import type { ProjectInfo, RunInfo } from "../api";
import { t } from "../i18n";
import { codicon, TextField, useModalFocus } from "../workbench";
import { mainAction } from "./home";
import { KIND_ICON, ProjectState } from "./kind";
import { wayLabel, type NewWay } from "./new";

interface Item {
  key: string;
  icon: ReactNode;
  label: string;
  detail: string;
  decoration?: ReactNode;
  open: () => void;
  start?: (() => void) | undefined;
}

export function QuickOpen({
  projects,
  runs,
  onOpen,
  onStart,
  onNew,
  onClose,
}: {
  projects: ProjectInfo[];
  runs: Record<string, RunInfo>;
  onOpen: (id: string) => void;
  onStart: (p: ProjectInfo) => void;
  onNew: (way: NewWay) => void;
  onClose: () => void;
}) {
  const box = useRef<HTMLDivElement>(null);
  useModalFocus(box, onClose);
  const [query, setQuery] = useState("");
  const [at, setAt] = useState(0);
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  const done = (f: () => void) => () => {
    onClose();
    f();
  };
  const items: Item[] = [
    ...[...projects]
      .filter((p) => {
        const text = `${p.name} ${p.platform} ${p.languages.join(" ")}`.toLowerCase();
        return words.every((w) => text.includes(w));
      })
      .sort(
        (a, b) => Math.max(b.modified, b.settings.lastOpened ?? 0) - Math.max(a.modified, a.settings.lastOpened ?? 0),
      )
      .map((p): Item => {
        const Icon = KIND_ICON[p.type];
        return {
          key: p.id,
          icon: <Icon size={16} aria-hidden="true" />,
          label: p.name,
          detail: p.platform,
          decoration: <ProjectState p={p} runs={runs} />,
          open: done(() => {
            onOpen(p.id);
          }),
          start: mainAction(p)
            ? done(() => {
                onStart(p);
              })
            : undefined,
        };
      }),
    ...(
      [
        ["empty", codicon("new-folder")],
        ["git", codicon("repo-clone")],
        ["framework", codicon("wand")],
      ] as const
    )
      .filter(([way]) => words.every((w) => `${t("Nuovo progetto")} ${wayLabel(way)}`.toLowerCase().includes(w)))
      .map(([way, Icon]): Item => ({
        key: `new:${way}`,
        icon: <Icon size={16} aria-hidden="true" />,
        label: t("Nuovo progetto"),
        detail: wayLabel(way),
        open: done(() => {
          onNew(way);
        }),
      })),
  ];
  const current = Math.min(at, items.length - 1);
  const keys = (e: KeyboardEvent<HTMLElement>) => {
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const step = e.key === "ArrowDown" ? 1 : -1;
      setAt((current + step + items.length) % Math.max(items.length, 1));
    } else if (e.key === "Enter") {
      const item = items[current];
      if (!item) return;
      e.preventDefault();
      if (e.shiftKey && item.start) item.start();
      else item.open();
    }
  };
  return (
    <div
      className="fixed inset-0 z-50 flex justify-center pt-12"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        ref={box}
        role="dialog"
        aria-modal="true"
        aria-label={t("Apri un progetto")}
        tabIndex={-1}
        onKeyDown={keys}
        className="flex max-h-[60vh] w-[min(36rem,90vw)] flex-col gap-1 self-start rounded-md border border-vsc-widget-border bg-vsc-widget p-1.5 text-(--vscode-editorWidget-foreground) shadow-[0_0_8px_2px_var(--vsc-shadow)]"
      >
        <TextField
          label={t("Apri un progetto")}
          placeholder={t("Nome del progetto, piattaforma o linguaggio")}
          autoFocus
          value={query}
          onChange={(v) => {
            setQuery(v);
            setAt(0);
          }}
          className="w-full"
        />
        <ul role="listbox" aria-label={t("Progetti")} className="m-0 min-h-0 list-none overflow-y-auto p-0">
          {items.map((item, i) => (
            <li
              key={item.key}
              role="option"
              aria-selected={i === current}
              onMouseMove={() => {
                setAt(i);
              }}
              onClick={item.open}
              className={`flex h-[22px] cursor-pointer items-center gap-1.5 rounded px-1.5 ${
                i === current ? "bg-vsc-list-active text-vsc-list-active-fg" : ""
              }`}
            >
              <span className="flex w-4 shrink-0 justify-center">{item.icon}</span>
              <span className="truncate">{item.label}</span>
              <span className="truncate text-[12px] opacity-70">{item.detail}</span>
              <span className="flex-1" />
              {item.decoration}
            </li>
          ))}
          {items.length === 0 && (
            <li className="px-1.5 text-vsc-desc">{t("Nessun progetto corrisponde alla ricerca.")}</li>
          )}
        </ul>
        <p className="px-1.5 text-[11px] text-vsc-desc">{t("Invio apre la pagina · Maiusc+Invio avvia")}</p>
      </div>
    </div>
  );
}
