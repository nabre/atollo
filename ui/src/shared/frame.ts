// Work done once per frame the window draws, however many events asked for
// it before: a download's progress, an install's percent, the lines of an
// action's output come many to a frame, and one render of them is enough.

/** Calls `flush` at the next frame, once whatever the number of `ask`s
 * before it; `cancel` drops a flush asked and not done. While the window is
 * hidden no frame comes: the flush waits for the window to show again. */
export function frameBatch(flush: () => void): { ask: () => void; cancel: () => void } {
  let id: number | null = null;
  const run = () => {
    id = null;
    flush();
  };
  return {
    ask: () => {
      id ??= requestAnimationFrame(run);
    },
    cancel: () => {
      if (id !== null) cancelAnimationFrame(id);
      id = null;
    },
  };
}
