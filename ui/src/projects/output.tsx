// An action's output, as the editor's Output view: the project's page and
// the Activity panel show it. It reads the lines from the logs (runs.ts)
// itself, so a line that arrives renders this view only, once a frame; the
// lines go in blocks, so a new line redraws the last block, not the log.
import { memo, useLayoutEffect, useRef } from "react";
import { useRunOutput, type RunLogs } from "./runs";

/** Lines in a block: the blocks before the last one stay as they are. */
const BLOCK = 100;

/** A block's text, rendered again only when it changes. */
const Block = memo(function Block({ text }: { text: string }) {
  return text;
});

/**
 * The output of the action `runKey` in a <pre>. `follow`: always shows the
 * last line (the page's «Segui»); without it, the view keeps to the last
 * line while the user is there, and stays where they scrolled to otherwise.
 * Where the user is comes from the scroll events, not measured at each line.
 */
export function RunOutputView({
  logs,
  runKey,
  label,
  className,
  follow,
}: {
  logs: RunLogs;
  runKey: string;
  label: string;
  className: string;
  follow?: boolean;
}) {
  const { lines, first } = useRunOutput(logs, runKey);
  const ref = useRef<HTMLPreElement>(null);
  // Whether the user is at the last line; another action shown starts there.
  const atEnd = useRef(true);
  const shown = useRef(runKey);
  useLayoutEffect(() => {
    if (shown.current !== runKey) {
      shown.current = runKey;
      atEnd.current = true;
    }
    const el = ref.current;
    if (el && (follow ?? atEnd.current)) el.scrollTop = el.scrollHeight;
  }, [lines, follow, runKey]);
  // Blocks by the lines' numbers, so a block keeps its text while lines
  // are added after it; every block but the last ends with its newline.
  const blocks: { key: number; text: string }[] = [];
  for (let start = Math.floor(first / BLOCK) * BLOCK; start < first + lines.length; start += BLOCK) {
    const end = start + BLOCK;
    const text = lines.slice(Math.max(0, start - first), end - first).join("\n");
    blocks.push({ key: start, text: end < first + lines.length ? `${text}\n` : text });
  }
  return (
    <pre
      ref={ref}
      tabIndex={0}
      aria-label={label}
      onScroll={(e) => {
        const el = e.currentTarget;
        atEnd.current = el.scrollHeight - el.scrollTop - el.clientHeight < 4;
      }}
      className={className}
    >
      {blocks.map((b) => (
        <Block key={b.key} text={b.text} />
      ))}
    </pre>
  );
}
