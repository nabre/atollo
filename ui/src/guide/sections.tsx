// The text of a topic of the Guide (guide/content), with what the app
// knows of the kit.
import type { ReactNode } from "react";
import { TOPIC_TEXTS } from "@guide/content";
import type { GuideTopicId } from "@guide/core/model";
import type { AppInfo, ComponentInfo } from "../api";

export { TOPIC_TEXTS };

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
