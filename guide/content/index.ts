// The text of every topic of the Guide, gathered from the chapter files
// (one per chapter, in the order of GUIDE_GROUPS).
import type { TopicTexts } from "../core/types";
import { AGENTS } from "./agents";
import { HELP } from "./help";
import { INSIDE } from "./inside";
import { PORTABLE } from "./portable";
import { PROJECTS } from "./projects";
import { SERVICES } from "./services";
import { SETTINGS } from "./settings";
import { START } from "./start";
import { TOOLS } from "./tools";

export const TOPIC_TEXTS: TopicTexts = {
  ...START,
  ...PROJECTS,
  ...SERVICES,
  ...AGENTS,
  ...TOOLS,
  ...SETTINGS,
  ...PORTABLE,
  ...INSIDE,
  ...HELP,
};
