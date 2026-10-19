// docs/app/GUIDE.md and GUIDE.it.md come from the in-app Guide
// (scripts/guide-docs.cjs): every topic is there with its title and
// subtitle. When this fails, run `node scripts/guide-docs.cjs`.
import { afterAll, describe, expect, it } from "vitest";
import { setLang } from "../../../ui/src/i18n";
import { GUIDE_TOPICS, sectionCode } from "../../../ui/src/guide/model";

const docs = import.meta.glob<string>("../../../docs/app/GUIDE*.md", {
  query: "?raw",
  import: "default",
  eager: true,
});

function doc(name: string): string {
  const entry = Object.entries(docs).find(([path]) => path.endsWith(`/${name}`));
  if (!entry) throw new Error(`${name} not found`);
  return entry[1];
}

describe("guide documents", () => {
  afterAll(() => {
    setLang("it");
  });

  it.each([
    ["en", "GUIDE.md"],
    ["it", "GUIDE.it.md"],
  ] as const)("%s: %s has every topic of the in-app guide", (lang, name) => {
    setLang(lang);
    const text = doc(name);
    for (const topic of GUIDE_TOPICS) {
      expect(text, topic.id).toContain(`### ${sectionCode(topic)} ${topic.title()}`);
      expect(text, topic.id).toContain(topic.subtitle().slice(0, 40));
    }
  });
});
