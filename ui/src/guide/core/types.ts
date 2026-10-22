// What a topic receives from whoever draws it, as plain shapes: the app
// passes its own objects (they have these fields and more), the site null.
import type { ReactNode } from "react";
import type { GuideTopicId } from "../model";

/** The kit the Guide describes, when the app knows it. */
export interface GuideKitInfo {
  version: string;
  root: string;
  projects: string;
  diskFs: string | null;
}

/** A component of the catalog as the app sees it on this PC. */
export interface GuideComponentState {
  id: string;
  installed: boolean;
  version: string | null;
  available: boolean;
  noArm64: boolean;
  unavailableReason: string | null;
}

/** What a topic receives: the kit it describes, when known. */
export interface TopicProps {
  info: GuideKitInfo | null;
  components: GuideComponentState[] | null;
}

/** The text of the topics of a chapter, by topic. */
export type TopicTexts = Partial<Record<GuideTopicId, (p: TopicProps) => ReactNode>>;
