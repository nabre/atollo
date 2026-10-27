// Work done once per frame the window draws, however many events asked for
// it before: a download's progress, an install's percent, the lines of an
// action's output come many to a frame, and one render of them is enough.

/** While the window is hidden no frame comes: what waits is done after
 * this many milliseconds instead, so it does not pile up. */
const HIDDEN_MS = 500;

/** Calls `flush` at the next frame, once whatever the number of `ask`s
 * before it (or after HIDDEN_MS, when no frame comes: a hidden window);
 * `cancel` drops a flush asked and not done. */
export function frameBatch(flush: () => void): { ask: () => void; cancel: () => void } {
  let frame: number | null = null;
  let timer: number | null = null;
  const cancel = () => {
    if (frame !== null) cancelAnimationFrame(frame);
    if (timer !== null) window.clearTimeout(timer);
    frame = null;
    timer = null;
  };
  const run = () => {
    cancel();
    flush();
  };
  return {
    ask: () => {
      if (frame !== null) return;
      frame = requestAnimationFrame(run);
      timer = window.setTimeout(run, HIDDEN_MS);
    },
    cancel,
  };
}
