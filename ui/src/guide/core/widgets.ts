// THE CONTRACT of the Guide's widgets: their names and props. The chapters
// draw with the stand-ins below; each renderer registers, once per
// process, a function for every name (registerGuideWidgets): the app its
// interactive widgets, the site static HTML. A renderer that misses a
// widget is a type error; one that forgets to register gets a clear error,
// never an empty page.
import { createElement, type ReactNode } from "react";
import type { AppPageTarget, GuideTopicId } from "../model";

/** Where a link of the Guide leads: never a function, so every renderer
 * can draw it (the app opens the topic or the page, the site links). */
export type GuideTarget = { topic: GuideTopicId } | { page: AppPageTarget };

export interface GuideWidgetProps {
  /** A paragraph of the running text. */
  P: { children: ReactNode };
  /** Numbered steps: what to click, in order. */
  Steps: { steps: ReactNode[] };
  /** A bulleted list of the running text. */
  Bullets: { items: ReactNode[] };
  /** A titled part of a topic. */
  Section: { title: string; children: ReactNode };
  /** The opening box of a topic. */
  InBrief: { points: ReactNode[] };
  /** A tip, a note or a warning beside the text. */
  Callout: { tone: "tip" | "info" | "warn"; title?: string; children: ReactNode };
  /** The technical side of a topic, folded. */
  Under: { title?: string; children: ReactNode };
  /** A question and its answer. */
  QA: { q: string; children: ReactNode };
  /** A chain of steps drawn as boxes and arrows. */
  Flow: { steps: { title: string; text: string }[] };
  /** A plain table of the running text. */
  Table: { columns: string[]; rows: ReactNode[][] };
  /** A key of the keyboard. */
  Key: { children: ReactNode };
  /** A label of the interface as the user sees it. */
  Ui: { children: ReactNode };
  /** A path, a file name, a command or a value to type. */
  Code: { children: ReactNode };
  /** A group of cards. */
  Cards: { children: ReactNode };
  /** A code or configuration snippet to copy. */
  GuideCode: { title: string; path?: string; code: string };
  /** A screenshot of the app (a file name of the screenshots' folder). */
  GuideShot: { src: string; title: string; caption: string; example?: ReactNode; scenario?: string };
  /** A card with an icon, a title and where its action leads: another
   * topic, or a page of the app (on the site, the topic of that page). */
  GuideCard: {
    /** A codicon's name. */
    icon: string;
    badge?: string;
    title: string;
    children: ReactNode;
    to?: GuideTarget;
    actionLabel?: string;
  };
  /** A link to another topic, named by its title unless told otherwise. */
  See: { id: GuideTopicId; children?: ReactNode };
  /** The topics close to this one. */
  Related: { ids: GuideTopicId[] };
  /** A page of the app. */
  Go: { page: AppPageTarget; children: ReactNode };
  /** The kit's key folders. */
  KitFoldersBar: Record<string, never>;
}

export type WidgetName = keyof GuideWidgetProps;

/** Every widget's name, for the renderers' tests. */
export const WIDGET_NAMES = [
  "P",
  "Steps",
  "Bullets",
  "Section",
  "InBrief",
  "Callout",
  "Under",
  "QA",
  "Flow",
  "Table",
  "Key",
  "Ui",
  "Code",
  "Cards",
  "GuideCode",
  "GuideShot",
  "GuideCard",
  "See",
  "Related",
  "Go",
  "KitFoldersBar",
] as const satisfies readonly WidgetName[];

/** A renderer: a function for every widget of the contract. */
export type GuideWidgetImpl = { [K in WidgetName]: (props: GuideWidgetProps[K]) => ReactNode };

let current: GuideWidgetImpl | null = null;

/** Sets the renderer's widgets, for the whole process. */
export function registerGuideWidgets(impl: GuideWidgetImpl): void {
  current = impl;
}

function widget<K extends WidgetName>(name: K) {
  const C = (props: GuideWidgetProps[K]): ReactNode => {
    if (!current) throw new Error(`guide widgets not registered: ${name}`);
    return createElement(current[name] as (p: GuideWidgetProps[K]) => ReactNode, props);
  };
  C.displayName = name;
  C.widget = name;
  return C;
}

export const P = widget("P");
export const Steps = widget("Steps");
export const Bullets = widget("Bullets");
export const Section = widget("Section");
export const InBrief = widget("InBrief");
export const Callout = widget("Callout");
export const Under = widget("Under");
export const QA = widget("QA");
export const Flow = widget("Flow");
export const Table = widget("Table");
export const Key = widget("Key");
export const Ui = widget("Ui");
export const Code = widget("Code");
export const Cards = widget("Cards");
export const GuideCode = widget("GuideCode");
export const GuideShot = widget("GuideShot");
export const GuideCard = widget("GuideCard");
export const See = widget("See");
export const Related = widget("Related");
export const Go = widget("Go");
export const KitFoldersBar = widget("KitFoldersBar");
