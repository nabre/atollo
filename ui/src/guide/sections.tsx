// The text of every topic of the in-app Guide, gathered from the chapter
// files in topics/ (one per chapter, in the order of GUIDE_GROUPS).
import type { ReactNode } from "react";
import type { AppInfo, ComponentInfo } from "../api";
import type { GuideTopicId } from "./model";
import { AGENTS } from "./topics/agents";
import { HELP } from "./topics/help";
import { INSIDE } from "./topics/inside";
import { PORTABLE } from "./topics/portable";
import { PROJECTS } from "./topics/projects";
import { SERVICES } from "./topics/services";
import { SETTINGS } from "./topics/settings";
import { START } from "./topics/start";
import { TOOLS } from "./topics/tools";
import type { TopicTexts } from "./core/types";

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

export function GuideTopicContent({
  topicId,
  info,
  components,
}: {
  topicId: GuideTopicId;
  info: AppInfo | null;
  components: ComponentInfo[] | null;
}): ReactNode {
  const text = TOPIC_TEXTS[topicId];
  return text ? text({ info, components }) : null;
}
