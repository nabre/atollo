// Types of site/guide.mjs, for the tests of test/ui/guide.
export function esc(s: unknown): string;
export function webpSize(file: string): { width: number; height: number };
export function topicPath(pathOf: (l: string) => string, l: string, id: string): string;

export interface SiteGuide {
  guide: {
    core: {
      lang: { bindGuideLang: (g: () => string) => void };
      model: {
        GUIDE_TOPICS: { id: string; group: string; title: () => string; keywords: string }[];
        sectionCode: (t: unknown) => string;
      };
      search: {
        topicHaystack: (t: unknown) => string;
        matches: (words: string[], hay: string) => boolean;
        searchWords: (q: string) => string[];
      };
      widgets: { WIDGET_NAMES: readonly string[] };
    };
  };
  widgets: Record<string, unknown>;
  setLang: (l: string) => void;
  renderTopic: (l: string, id: string, options?: { href?: (id: string) => string; shotBase?: string }) => string;
}
export function siteGuide(): SiteGuide;
export function siteWidgets(guide: unknown): Record<string, unknown>;
export function guidePages(options: unknown): { pages: unknown[]; topics: string[] };
