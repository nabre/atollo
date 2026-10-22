// A tool of the kit and a framework as small pictures: the tag of a tool,
// the logo of a base or addition.
import type { ComponentInfo, FrameworkPiece } from "../api";
import { t } from "../i18n";
import { ComponentLogo, FrameworkLogo } from "../shared/logos";
import { Tag } from "../shared/tags";
import { Codicon, codicon, IconButton } from "../workbench";

/** A tool of the kit as a tag: its logo and its name in the catalog. */
export function ToolTag({
  id,
  components,
  onRemove,
}: {
  id: string;
  components: ComponentInfo[];
  onRemove?: () => void;
}) {
  const name = components.find((c) => c.id === id)?.name ?? id;
  return (
    <Tag icon={<ComponentLogo id={id} size={12} />} className="text-vsc-fg">
      {name}
      {onRemove && (
        // The toolbar button's look, at the tag's height (16px, a 12px glyph).
        <IconButton
          icon={codicon("close")}
          label={t("Togli {0}", name)}
          onClick={onRemove}
          className="-mr-1 size-4! [&>.codicon]:text-vsc-codicon-compact"
        />
      )}
    </Tag>
  );
}

/** The logo of a base or addition; a generic icon for those without one. */
export function PieceLogo({ piece, size = 16 }: { piece: FrameworkPiece; size?: number }) {
  const fallback = piece.kind === "base" ? "package" : "extensions";
  return <FrameworkLogo id={piece.id} size={size} fallback={<Codicon name={fallback} size={size} />} />;
}
