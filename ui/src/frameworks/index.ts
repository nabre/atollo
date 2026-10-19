// The instructions of the new-project wizard in Settings (projects/frameworks/,
// D-83): a tree in the side bar, one tab per base (Laravel, Vite,
// Electron…) and per addition (React, Tailwind CSS, NativePHP…), with the
// commands that install it, one per line, the kit's tools as tags with their
// logo, the variables of .env and the database. The official ones come with
// the app; a changed one shows the bar of a modified setting and goes back
// with «Ripristina». Also the plan of a combination, shown here to try the
// instructions and in the wizard.
export {
  addPiece,
  bases,
  basesByCategory,
  categoryLabel,
  CATEGORIES,
  integrationTab,
  parseIntegrationTab,
  pieceTab,
  textOf,
  TRY_TAB,
  useCombination,
  useFrameworks,
  usePlan,
} from "./model";
export type { Frameworks } from "./model";
export { PieceLogo } from "./tags";
export { IntegrationGroup, PieceGroup } from "./editor";
export { FrameworksTree } from "./tree";
export { AddonChoice, PlanSummary, TryGroup } from "./plan";
