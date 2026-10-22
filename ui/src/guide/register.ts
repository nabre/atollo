// The app as the Guide's renderer: its language and its widgets. Imported
// once, by the Guide's entry (index.ts) and by the tests that draw topics.
import { lang } from "../i18n";
import { bindGuideLang } from "./core/lang";
import { registerGuideWidgets, type GuideWidgetImpl } from "./core/widgets";
import { Badge, FilterList, KitInfo, SourcesTable } from "./live";
import * as W from "./widgets";

/** The app's renderer of the contract. */
export const APP_WIDGETS: GuideWidgetImpl = {
  P: W.P,
  Steps: W.Steps,
  Bullets: W.Bullets,
  Section: W.Section,
  InBrief: W.InBrief,
  Callout: W.Callout,
  Under: W.Under,
  QA: W.QA,
  Flow: W.Flow,
  Table: W.Table,
  Key: W.Key,
  Ui: W.Ui,
  Code: W.Code,
  Cards: W.Cards,
  GuideCode: W.GuideCode,
  GuideShot: W.GuideShot,
  GuideCard: W.GuideCard,
  See: W.See,
  Related: W.Related,
  Go: W.Go,
  KitFoldersBar: W.KitFoldersBar,
  Terms: W.Terms,
  WindowSketch: W.WindowSketch,
  Only: W.Only,
  Badge,
  KitInfo,
  SourcesTable,
  FilterList,
};

bindGuideLang(lang);
registerGuideWidgets(APP_WIDGETS);
