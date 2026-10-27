// The app's Guide page and the site's index find the same topics: both
// take each topic's haystack and the matching rule from guide/core/search.ts.
import { describe, expect, it } from "vitest";
import { setLang, type Lang } from "@/i18n";
import "@/guide/register";
import { GUIDE_TOPICS } from "@guide/core/model";
import { matches, searchWords, topicHaystack } from "@guide/core/search";
import { siteGuide } from "../../../site/guide.mjs";

const QUERIES = ["porta", "port", "mysql password", "agent", "1.3", "git", "zzz-nothing"];

describe("guide search, app and site", () => {
  it.each(["it", "en", "fr", "de"] as Lang[])("finds the same topics in %s", async (lang) => {
    await setLang(lang);
    const site = siteGuide();
    site.setLang(lang);
    const index = site.guide.core.model.GUIDE_TOPICS.map((t) => ({
      id: t.id,
      hay: site.guide.core.search.topicHaystack(t),
    }));
    for (const q of QUERIES) {
      const words = searchWords(q);
      const app = GUIDE_TOPICS.filter((t) => matches(words, topicHaystack(t))).map((t) => t.id);
      const web = index.filter((t) => site.guide.core.search.matches(words, t.hay)).map((t) => t.id);
      expect(web, `${lang}: ${q}`).toEqual(app);
    }
    await setLang("it");
  });
});
