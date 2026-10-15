// VSCodium's Settings editor (preferences/browser/settingsEditor2): the
// search on top, the table of contents on the left, the groups on the right
// with one item per setting: "Category: Name" in bold, the description, the
// control under it, a bar on the left when changed. The search hides what
// does not match; a group with nothing left hides itself.
import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from "react";
import { SearchBox } from "./controls";

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
      <div className="flex shrink-0 items-center gap-2 px-5 pt-3 pb-2.5">
        <SearchBox
          value={search}
          onChange={onSearch}
          placeholder={placeholder}
          large
          className="max-w-[880px] flex-1"
        />
        {actions}
      </div>
      <div className="flex min-h-0 flex-1 border-t border-vsc-border">
        <nav aria-label={placeholder} className="hidden w-[180px] shrink-0 overflow-y-auto py-2 md:block">
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
                    className={`flex h-[22px] w-full items-center pr-2 pl-5 text-left ${
                      on ? "font-semibold text-vsc-settings-header" : "text-vsc-desc hover:text-vsc-fg"
                    }`}
                  >
                    <span className="truncate">{entry.label}</span>
                  </button>
                </li>
              );
            })}
          </ul>
        </nav>
        <div ref={body} className="min-h-0 flex-1 overflow-y-auto pb-10">
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

/** A group of settings with its title; hidden when the search leaves it empty. */
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
    <section id={id} aria-labelledby={`${id}-title`} className="scroll-mt-0 pt-2 [&:not(:has([data-setting]))]:hidden">
      <h2
        id={`${id}-title`}
        className="px-[14px] pt-2 pb-1 text-[20px] leading-7 font-semibold text-vsc-settings-header"
      >
        {title}
      </h2>
      {description && <p className="max-w-[880px] px-[14px] pb-1 text-vsc-desc">{description}</p>}
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
    <div
      data-setting
      className={`relative max-w-[880px] rounded-md border-l-2 hover:bg-(--vscode-settings-rowHoverBackground) ${
        modified ? "border-l-vsc-settings-modified" : "border-l-transparent"
      }`}
    >
      {/* The bar takes 2px of the group's 14px on the left. */}
      <vscode-form-group variant="settings-group" className="max-w-none pl-3">
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
