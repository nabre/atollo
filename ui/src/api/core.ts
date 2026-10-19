// Typed wrappers of the Rust commands (tauri/src/commands/). The
// interface calls nothing else: no generic "run this" command exists.
//
// Outside Tauri (`npm run dev` in ui\ opened in a browser) the commands do
// not exist: the wrappers answer with sample data, so the pages can be
// designed in the browser, and the interface shows a "preview" banner.
import { invoke, isTauri, type InvokeArgs } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/** True when the page runs in the browser instead of the Atollo window. */
export const browserPreview = !isTauri();

/** A command that answers with data; `preview` is the answer in the browser. */
export function query<T>(cmd: string, preview: T, args?: InvokeArgs): Promise<T> {
  return browserPreview ? Promise.resolve(preview) : invoke<T>(cmd, args);
}

/** A command that answers with nothing (it acts); nothing happens in the browser. */
export function act(cmd: string, args?: InvokeArgs): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>(cmd, args);
}

/** The handlers of the browser preview's events, by event (`previewEmit`). */
const previewHandlers = new Map<string, Set<(payload: unknown) => void>>();

/** Every `event` from Rust to `handler`; resolves to the function that stops
 * listening. In the browser only what the preview sends (`previewEmit`)
 * arrives. `T` is what Rust sends with that event: the caller names it, as
 * with `invoke<T>`. */
// eslint-disable-next-line @typescript-eslint/no-unnecessary-type-parameters
export function subscribe<T>(event: string, handler: (payload: T) => void): Promise<() => void> {
  if (browserPreview) {
    const set = previewHandlers.get(event) ?? new Set();
    previewHandlers.set(event, set);
    const h = handler as (payload: unknown) => void;
    set.add(h);
    return Promise.resolve(() => {
      set.delete(h);
    });
  }
  return listen<T>(event, (e) => {
    handler(e.payload);
  });
}

/** The browser preview's stand-in for an event Rust would send. */
export function previewEmit(event: string, payload: unknown): void {
  for (const h of previewHandlers.get(event) ?? []) h(payload);
}
