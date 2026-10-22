// The site's renderer of the Guide (site/guide.mjs): every topic in every
// language draws with SITE_WIDGETS, has text, leaves no class of the app
// and no {0}, links only to topics that exist and shows only screenshots
// that exist (the renderer throws on a missing ui/public/guide/<lang>/
// file); the site draws every widget of the contract and no other.
import { describe, expect, it } from "vitest";
import { siteGuide } from "../../../site/guide.mjs";

const LANGS = ["en", "it", "fr", "de"];
const site = siteGuide();
const ids = site.guide.core.model.GUIDE_TOPICS.map((t) => t.id);

describe("guide on the site", () => {
  it("has a site widget for every name of the contract, and no other", () => {
    expect(Object.keys(site.widgets).sort()).toEqual([...site.guide.core.widgets.WIDGET_NAMES].sort());
  });

  it("finds the topics", () => {
    expect(ids.length).toBeGreaterThan(30);
  });

  it.each(LANGS)("draws every topic in %s", (lang) => {
    let images = 0;
    for (const id of ids) {
      const html = site.renderTopic(lang, id, { href: (t) => `@topic/${t}/`, shotBase: "@shot/" });
      expect(html.replace(/<[^>]+>/g, "").length, id).toBeGreaterThan(200);
      expect(html, id).not.toMatch(/\bvsc-|className|\{\d+\}/);
      for (const m of html.matchAll(/href="@topic\/([^/"]+)\/"/g)) expect(ids, `${id} → ${m[1] ?? ""}`).toContain(m[1]);
      for (const m of html.matchAll(/<img [^>]*src="([^"]+)"/g)) {
        expect(m[1], id).toMatch(/^@shot\/[\w-]+\.webp$/);
        images += 1;
      }
    }
    expect(images).toBeGreaterThan(10);
  });
});
