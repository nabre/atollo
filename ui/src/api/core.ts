// Typed wrappers of the Rust commands (tauri/src/commands/). The
// interface calls nothing else: no generic "run this" command exists.
//
// Outside Tauri (`npm run dev` in ui\ opened in a browser) the commands do
// not exist: the wrappers answer with sample data, so the pages can be
// designed in the browser, and the interface shows a "preview" banner.
import { invoke, isTauri, type InvokeArgs } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/** True when the page runs in the browser instead of the Dev-Kit window. */
export const browserPreview = !isTauri();

/** A command that answers with data; `preview` is the answer in the browser. */
export function query<T>(cmd: string, preview: T, args?: InvokeArgs): Promise<T> {
  return browserPreview ? Promise.resolve(preview) : invoke<T>(cmd, args);
}

/** A command that answers with nothing (it acts); nothing happens in the browser. */
export function act(cmd: string, args?: InvokeArgs): Promise<void> {
  return browserPreview ? Promise.resolve() : invoke<undefined>(cmd, args);
}

/** Every `event` from Rust to `handler`; resolves to the function that stops
 * listening. Nothing arrives in the browser. `T` is what Rust sends with
 * that event: the caller names it, as with `invoke<T>`. */
// eslint-disable-next-line @typescript-eslint/no-unnecessary-type-parameters
export function subscribe<T>(event: string, handler: (payload: T) => void): Promise<() => void> {
  if (browserPreview) return Promise.resolve(() => undefined);
  return listen<T>(event, (e) => {
    handler(e.payload);
  });
}
