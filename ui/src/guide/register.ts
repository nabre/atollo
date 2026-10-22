// The app as the Guide's renderer: its language and its widgets. Imported
// once, by the Guide's entry (index.ts) and by the tests that draw topics.
import { lang } from "../i18n";
import { bindGuideLang } from "./core/lang";
import { registerGuideWidgets } from "./core/widgets";
import { APP_WIDGETS } from "./widgets";

bindGuideLang(lang);
registerGuideWidgets(APP_WIDGETS);
