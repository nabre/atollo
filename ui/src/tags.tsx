// Small labels shared by every page, so a tag looks the same everywhere:
// Tag (bordered chip), Badge (vscode-badge), Mark (an icon named on hover).
import type { ReactNode } from "react";

const TONES = {
  default: "border-vsc-border text-vsc-desc",
  focus: "border-vsc-focus text-vsc-fg",
  warning: "border-vsc-orange font-medium text-vsc-orange",
} as const;

/** A bordered chip: languages, dependencies, the projects that need a tool. */
export function Tag({
  icon,
  tone = "default",
  dashed = false,
  title,
  className = "",
  children,
}: {
  icon?: ReactNode;
  tone?: keyof typeof TONES;
  dashed?: boolean;
  title?: string | undefined;
  className?: string;
  children: ReactNode;
}) {
  return (
    <span
      title={title}
      className={`inline-flex items-center gap-1 rounded-sm border px-1.5 align-middle text-[11px] leading-4 ${TONES[tone]} ${
        dashed ? "border-dashed" : ""
      } ${className}`}
    >
      {icon}
      {children}
    </span>
  );
}

/** A filled label: a state ("in uso") or a plan (vscode-badge, default variant). */
export function Badge({ className = "", children }: { className?: string; children: ReactNode }) {
  return <vscode-badge className={className}>{children}</vscode-badge>;
}

/** An icon that stands for a word: named on hover and for screen readers. */
export function Mark({ label, className = "", children }: { label: string; className?: string; children: ReactNode }) {
  return (
    <span role="img" aria-label={label} title={label} className={`inline-flex shrink-0 ${className}`}>
      {children}
    </span>
  );
}
