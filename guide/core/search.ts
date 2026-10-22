// The Guide's search, one rule for the app and the site: a topic is found
// when every word typed is in its title, subtitle, code or keywords, in
// any order and case.
import { sectionCode, type GuideTopic } from "./model";

/** The words of a query, lowercase. */
export function searchWords(query: string): string[] {
  return query.trim().toLowerCase().split(/\s+/).filter(Boolean);
}

/** What a topic is found by, lowercase. */
export function topicHaystack(topic: GuideTopic): string {
  return `${topic.title()} ${topic.subtitle()} ${sectionCode(topic)} ${topic.keywords}`.toLowerCase();
}

/** Whether every word is in the haystack. */
export function matches(words: string[], haystack: string): boolean {
  return words.every((w) => haystack.includes(w));
}
