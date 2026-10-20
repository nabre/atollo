// VSCodium's icons: the Codicon font of @vscode/codicons (codicon.css,
// imported by theme/theme.css), drawn as the workbench draws them, a
// <span class="codicon codicon-NAME"> with the glyph in its ::before. 16px
// is VSCodium's codiconFontSize, 12px its compact one; the line is the
// glyph's height (16px/1), so it centers in a flex row as the SVGs did.
import type { CSSProperties } from "react";
import type { IconBaseProps, IconType } from "react-icons";

/** The size's token class, when VSCodium has one for it. */
const TOKEN: Partial<Record<number, string>> = {
  16: "text-vsc-codicon",
  12: "text-vsc-codicon-compact",
};

/** A Codicon by name (codicon.css: `gear`, `chevron-down`...). Hidden from
 * assistive technology, unless it has a `label`: then it is an image with
 * that name. */
export function Codicon({
  name,
  size = 16,
  className = "",
  style,
  title,
  label,
}: {
  name: string;
  /** In px: 16 and 12 are the token classes, the others its font size. */
  size?: number | string | undefined;
  className?: string | undefined;
  style?: CSSProperties | undefined;
  title?: string | undefined;
  label?: string | undefined;
}) {
  const token = typeof size === "number" ? TOKEN[size] : undefined;
  return (
    <span
      role={label ? "img" : undefined}
      aria-label={label}
      aria-hidden={label ? undefined : "true"}
      title={title}
      className={["codicon", `codicon-${name}`, token ?? "", className].filter(Boolean).join(" ")}
      style={token ? style : { fontSize: size, ...style }}
    />
  );
}

const components = new Map<string, IconType>();

/** A Codicon as an icon component, where a control takes one (IconButton,
 * ActionButton, a list's or a page's icon) beside the brand logos of
 * react-icons: the same props (size, className, style, title, aria-label).
 * One component per name, so a page keeps its icons from one render to the
 * next. */
export function codicon(name: string): IconType {
  let icon = components.get(name);
  if (!icon) {
    icon = function CodiconIcon({ size, className, style, title, "aria-label": label }: IconBaseProps) {
      return <Codicon name={name} size={size} className={className} style={style} title={title} label={label} />;
    };
    components.set(name, icon);
  }
  return icon;
}
