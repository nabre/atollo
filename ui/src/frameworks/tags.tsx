// A tool of the kit and a framework as small pictures: the tag of a tool,
// the logo of a base or addition.
import type { ComponentInfo, FrameworkPiece } from "../api";
import { t } from "../i18n";
import { VscClose, VscExtensions, VscPackage } from "react-icons/vsc";
import { ComponentLogo, FrameworkLogo } from "../logos";
import { Tag } from "../tags";

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
        <button
          type="button"
          aria-label={t("Togli {0}", name)}
          title={t("Togli {0}", name)}
          onClick={onRemove}
          className="-mr-1 flex size-4 items-center justify-center rounded hover:bg-vsc-toolbar-hover"
        >
          <VscClose size={12} aria-hidden="true" />
        </button>
      )}
    </Tag>
  );
}

/** The logo of a base or addition; a generic icon for those without one. */
export function PieceLogo({ piece, size = 16 }: { piece: FrameworkPiece; size?: number }) {
  const Fallback = piece.kind === "base" ? VscPackage : VscExtensions;
  return <FrameworkLogo id={piece.id} size={size} fallback={<Fallback size={size} aria-hidden="true" />} />;
}
