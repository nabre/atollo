// What a topic receives from whoever draws it, as plain shapes: the app
// passes its own objects (they have these fields and more), the site null.
import type { ReactNode } from "react";
import type { GuideTopicId } from "./model";

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
  unavailableReason: string | null;
}

/** The kinds of the Tools page (components/model.ts, D-139), plus the parts. */
export type SourceCategory =
  "system" | "agents" | "ai" | "languages" | "tools" | "web" | "databases" | "dbviewers" | "part";

/** A row of the table of sources (content/sources.ts). */
export interface ToolSourceRow {
  id: string;
  logoId: string;
  name: string;
  group: SourceCategory;
  home: string;
  metaSource: string;
  binarySource: string;
  verifyKind: "SHA-256" | "SHA-512" | "SHA3-256" | "npm sha512" | "Authenticode + SHA";
  verifyDetail: string;
  targetPath: string;
  /** Where macOS and Linux get it when not the same source as Windows
   * (the other columns describe Windows); null when only the file names
   * change. */
  unix: string | null;
}

/** A problem of the Guide's list: what the user sees, what to do. */
export interface GuideProblem {
  q: string;
  a: string;
  see?: GuideTopicId;
}

/** What a topic receives: the kit it describes, when known. */
export interface TopicProps {
  info: GuideKitInfo | null;
  components: GuideComponentState[] | null;
}

/** The text of the topics of a chapter, by topic. */
export type TopicTexts = Partial<Record<GuideTopicId, (p: TopicProps) => ReactNode>>;
