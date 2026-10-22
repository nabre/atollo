// VSCodium's Settings editor (preferences/browser/settingsEditor2): the
// search on top, the table of contents on the left, the groups on the right
// with one item per setting: "Category: Name" in bold, the description, the
// control under it, a bar on the left when changed. The search hides what
// does not match; a group with nothing left hides itself.
import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from "react";
import { SearchBox } from "./inputs";

const Query = createContext<string[]>([]);

function matches(words: string[], text: string): boolean {
  const lower = text.toLowerCase();
  return words.every((w) => lower.includes(w));
}

export interface TocEntry {
  id: string;
  label: string;
}

export function SettingsEditor({
  toc,
  search,
  onSearch,
  placeholder,
  actions,
  children,
}: {
  toc: TocEntry[];
  search: string;
  onSearch: (v: string) => void;
  placeholder: string;
  /** On the right of the search (Save, filters). */
  actions?: ReactNode;
  children: ReactNode;
}) {
  const body = useRef<HTMLDivElement>(null);
  const [current, setCurrent] = useState<string | null>(toc[0]?.id ?? null);
  const words = search.toLowerCase().split(/\s+/).filter(Boolean);

  // The entry of the group at the top of the scroll is the current one.
  useEffect(() => {
    const el = body.current;
    if (!el) return;
    const onScroll = () => {
      const top = el.getBoundingClientRect().top;
      let at: string | null = toc[0]?.id ?? null;
      for (const entry of toc) {
        const g = document.getElementById(entry.id);
        if (g && g.offsetParent !== null && g.getBoundingClientRect().top - top <= 24) at = entry.id;
      }
      setCurrent(at);
    };
    el.addEventListener("scroll", onScroll, { passive: true });
    return () => {
      el.removeEventListener("scroll", onScroll);
    };
  }, [toc]);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {/* .settings-header: 11px + 3px from the top, 24px from the sides. */}
      <div className="mt-vsc-size100 flex shrink-0 items-center gap-vsc-size80 px-vsc-size240 pt-vsc-size40">
        <SearchBox value={search} onChange={onSearch} placeholder={placeholder} large className="flex-1" />
        {actions}
      </div>
      {/* .settings-body, 14px under the header, no line between (modern UI). */}
      <div className="mt-vsc-size100 flex min-h-0 flex-1 pt-vsc-size40">
        {/* .settings-toc-wrapper: 200px with 24px in; rows of 22px, the
            others at 90% of the foreground, the current one bold in the
            header color, the hovered one in the header's at 70%. */}
        <nav
          aria-label={placeholder}
          className="hidden w-vsc-settings-toc shrink-0 overflow-y-auto pl-vsc-size240 md:block"
        >
          <ul className="m-0 list-none p-0">
            {toc.map((entry) => {
              const on = entry.id === current;
              return (
                <li key={entry.id}>
                  <button
                    type="button"
                    aria-current={on ? "location" : undefined}
                    onClick={() => {
                      document.getElementById(entry.id)?.scrollIntoView({ block: "start" });
                      setCurrent(entry.id);
                    }}
                    title={entry.label}
                    className={`flex h-vsc-row w-full items-center pr-vsc-size80 text-left outline-none focus-visible:outline-1 focus-visible:-outline-offset-1 focus-visible:outline-vsc-focus ${
                      on ? "font-bold text-vsc-settings-header" : "text-vsc-fg/90 hover:text-vsc-settings-header/70"
                    }`}
                  >
                    <span className="truncate">{entry.label}</span>
                  </button>
                </li>
              );
            })}
          </ul>
        </nav>
        {/* The tree: its rows 24px in from the sides. */}
        <div ref={body} className="min-h-0 flex-1 overflow-y-auto px-vsc-size240 pb-vsc-size400">
          <Query.Provider value={words}>{children}</Query.Provider>
        </div>
      </div>
    </div>
  );
}

/** The search of a settings view kept elsewhere (a side bar): the items
 * inside hide what does not match `search`, as in SettingsEditor. */
export function SettingsSearch({ search, children }: { search: string; children: ReactNode }) {
  const words = search.toLowerCase().split(/\s+/).filter(Boolean);
  return <Query.Provider value={words}>{children}</Query.Provider>;
}

/** A group of settings with its title; hidden when the search leaves it
 * empty. The title as a first level's .settings-group-title-label: 26px
 * semibold, 10px around it and 15px on the left. */
export function SettingsGroup({
  id,
  title,
  description,
  children,
}: {
  id: string;
  title: string;
  description?: ReactNode;
  children: ReactNode;
}) {
  return (
    <section id={id} aria-labelledby={`${id}-title`} className="scroll-mt-0 [&:not(:has([data-setting]))]:hidden">
      <h2
        id={`${id}-title`}
        className="truncate p-vsc-size100 pl-vsc-settings-title text-vsc-heading1 font-semibold text-vsc-settings-header"
      >
        {title}
      </h2>
      {description && <p className="px-vsc-settings-title pb-vsc-size40 text-vsc-desc">{description}</p>}
      {children}
    </section>
  );
}

/** One setting, as vscode-form-group's "settings-group": the label
 * "Category: Name", the description (vscode-form-helper) and the control. A
 * boolean puts its checkbox before the description (`inline`). */
export function SettingItem({
  category,
  title,
  description,
  modified = false,
  keywords = "",
  inline = false,
  children,
}: {
  category?: string;
  title: string;
  description?: ReactNode;
  /** Different from the default: a bar on the left. */
  modified?: boolean;
  /** More words the search finds it by. */
  keywords?: string;
  /** The control on the description's line (checkbox). */
  inline?: boolean;
  children?: ReactNode;
}) {
  const words = useContext(Query);
  const text = `${category ?? ""} ${title} ${typeof description === "string" ? description : ""} ${keywords}`;
  if (words.length > 0 && !matches(words, text)) return null;
  return (
    <div data-setting className="relative rounded-vsc-medium hover:bg-(--vscode-settings-rowHoverBackground)">
      {/* .setting-item-modified-indicator: 2px, 5px in, from 15px under
          the top to 18px over the bottom, in the item's 14px on the left. */}
      {modified && (
        <span
          aria-hidden="true"
          className="absolute top-vsc-modified-top bottom-vsc-modified-bottom left-vsc-modified-left w-vsc-size20 bg-vsc-settings-modified"
        />
      )}
      <vscode-form-group variant="settings-group" className="max-w-none">
        {/* The title in the header color, "Category:" lightened. */}
        <vscode-label className="[--vscode-foreground:var(--vscode-settings-headerForeground)]">
          {category && <span className="normal lightened">{category}: </span>}
          {title}
        </vscode-label>
        {inline ? (
          <div className="mt-[9px]">{children}</div>
        ) : (
          <>
            {description && <vscode-form-helper>{description}</vscode-form-helper>}
            {children && <div className="mt-[9px] max-w-[500px]">{children}</div>}
          </>
        )}
      </vscode-form-group>
    </div>
  );
}
