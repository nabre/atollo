// One process draws the Guide in four languages one after the other (the
// site's build): nothing may keep a tr() from the time a module loaded.
// Drawing Italian then English gives the English of a fresh process.
import { describe, expect, it } from "vitest";
import { siteGuide } from "../../../site/guide.mjs";

// Drawing the whole Guide takes seconds, more under load (the whole suite
// in parallel): not vitest's 5 s.
describe("guide languages in one process", { timeout: 60_000 }, () => {
  it("draws English after Italian as English alone", () => {
    const fresh = siteGuide();
    const ids = fresh.guide.core.model.GUIDE_TOPICS.map((t) => t.id);
    const alone = ids.map((id) => fresh.renderTopic("en", id));

    const after = siteGuide();
    for (const id of ids) after.renderTopic("it", id);
    ids.forEach((id, i) => {
      expect(after.renderTopic("en", id), id).toBe(alone[i]);
    });
  });

  it("draws each language differently", () => {
    const site = siteGuide();
    const texts = ["it", "en", "fr", "de"].map((l) => site.renderTopic(l, "welcome"));
    expect(new Set(texts).size).toBe(4);
  });
});
