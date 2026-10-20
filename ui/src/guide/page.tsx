// The Guide page, laid out as a VSCodium workbench:
// - side bar with the search and one collapsible pane per chapter;
// - one tab per chapter, a header with the topic and a button that opens the
//   page of the app the topic is about;
// - the topic's text, then the previous and next topic.
import { useEffect, useMemo, useRef, useState } from "react";
import { VscArrowLeft, VscArrowRight, VscBook, VscFolderOpened } from "react-icons/vsc";
import { componentsList, folderOpen, type AppInfo, type ComponentInfo } from "../api";
import { t } from "../i18n";
import { Tag } from "../tags";
import {
  Button,
  EditorBody,
  EditorHeader,
  EditorTabs,
  IconButton,
  List,
  ListRow,
  SearchBox,
  SideBarTitle,
  ViewContainer,
  ViewPane,
} from "../workbench";
import {
  GUIDE_GROUPS,
  GUIDE_TOPICS,
  sectionCode,
  topicOf,
  tr,
  type AppPageTarget,
  type GuideGroupId,
  type GuideTopicId,
} from "./model";
import { GuideTopicContent } from "./sections";
import { GuideNavContext, type GuideNav } from "./widgets";

/** A chapter's title with its number: "3. Database, siti e terminali". */
function chapterTitle(id: GuideGroupId): string {
  const i = GUIDE_GROUPS.findIndex((g) => g.id === id);
  return `${String(i + 1)}. ${GUIDE_GROUPS[i]?.title() ?? ""}`;
}

export function GuidePage({
  info,
  initialTopic,
  onNavigate,
}: {
  info: AppInfo | null;
  initialTopic?: GuideTopicId | null;
  onNavigate: (page: AppPageTarget) => void;
}) {
  const [topicId, setTopicId] = useState<GuideTopicId>(initialTopic ?? "welcome");
  const [search, setSearch] = useState("");
  const [closed, setClosed] = useState<Set<GuideGroupId>>(new Set());
  const [components, setComponents] = useState<ComponentInfo[] | null>(null);
  const bodyWrapRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    let alive = true;
    componentsList()
      .then((rows) => {
        if (alive) setComponents(rows);
      })
      .catch(() => {
        // Without the list the sources table shows no state: nothing else needs it.
      });
    return () => {
      alive = false;
    };
  }, []);

  // Back to the top of the text when the topic changes.
  useEffect(() => {
    const scroller = bodyWrapRef.current?.querySelector(".overflow-y-auto");
    if (scroller instanceof HTMLElement) scroller.scrollTop = 0;
  }, [topicId]);

  const topic = topicOf(topicId);
  const index = GUIDE_TOPICS.indexOf(topic);
  const prev = index > 0 ? GUIDE_TOPICS[index - 1] : undefined;
  const next = index < GUIDE_TOPICS.length - 1 ? GUIDE_TOPICS[index + 1] : undefined;

  const found = useMemo(() => {
    const words = search.trim().toLowerCase().split(/\s+/).filter(Boolean);
    if (words.length === 0) return GUIDE_TOPICS;
    return GUIDE_TOPICS.filter((x) => {
      const text = `${x.title()} ${x.subtitle()} ${sectionCode(x)} ${x.keywords}`.toLowerCase();
      return words.every((w) => text.includes(w));
    });
  }, [search]);

  const nav = useMemo<GuideNav>(() => ({ select: setTopicId, open: onNavigate }), [onNavigate]);

  const TopicIcon = topic.icon;
  const searching = search.trim() !== "";

  return (
    <GuideNavContext.Provider value={nav}>
      <ViewContainer
        id="guide"
        side={
          <>
            <SideBarTitle
              title={t("Guida")}
              actions={
                <IconButton
                  icon={VscFolderOpened}
                  label={tr(
                    "Apri cartella progetti",
                    "Open projects folder",
                    "Ouvrir le dossier des projets",
                    "Projektordner öffnen",
                  )}
                  onClick={() => {
                    void folderOpen("projects");
                  }}
                />
              }
            />
            <div className="shrink-0 px-3 pt-1 pb-1.5">
              <SearchBox value={search} onChange={setSearch} placeholder={t("Cerca nella guida")} />
            </div>
            <div className="flex min-h-0 flex-1 flex-col overflow-y-auto pb-vsc-size40">
              {GUIDE_GROUPS.map((g, i) => {
                const items = found.filter((x) => x.group === g.id);
                if (searching && items.length === 0) return null;
                return (
                  <ViewPane
                    key={g.id}
                    first={i === 0}
                    title={chapterTitle(g.id)}
                    badge={String(items.length)}
                    expanded={!closed.has(g.id) || searching}
                    onToggle={() => {
                      setClosed((prevSet) => {
                        const s = new Set(prevSet);
                        if (s.has(g.id)) s.delete(g.id);
                        else s.add(g.id);
                        return s;
                      });
                    }}
                  >
                    <List label={g.title()}>
                      {items.map((x) => {
                        const Icon = x.icon;
                        return (
                          <ListRow
                            key={x.id}
                            selected={x.id === topic.id}
                            onSelect={() => {
                              setTopicId(x.id);
                            }}
                            icon={<Icon size={15} aria-hidden="true" />}
                            label={x.title()}
                            decorations={<span className="font-mono text-[11px] opacity-70">{sectionCode(x)}</span>}
                          />
                        );
                      })}
                    </List>
                  </ViewPane>
                );
              })}
              {searching && found.length === 0 && (
                <p className="px-5 py-2 text-[12px] text-vsc-desc">
                  {tr(
                    "Nessun argomento corrisponde alla ricerca.",
                    "No topic matches the search.",
                    "Aucun sujet ne correspond à la recherche.",
                    "Kein Thema entspricht der Suche.",
                  )}
                </p>
              )}
            </div>
          </>
        }
      >
        <EditorTabs
          active={topic.group}
          onSelect={(groupId) => {
            const first = GUIDE_TOPICS.find((x) => x.group === groupId);
            if (first) setTopicId(first.id);
          }}
          tabs={GUIDE_GROUPS.map((g) => ({ id: g.id, label: chapterTitle(g.id), pinned: true }))}
          actions={<Tag tone="focus">{`${String(index + 1)} / ${String(GUIDE_TOPICS.length)}`}</Tag>}
        />
        <EditorHeader
          icon={<TopicIcon size={44} aria-hidden="true" className="text-vsc-focus" />}
          title={topic.title()}
          meta={[
            <Tag key="sec" tone="focus">
              {sectionCode(topic)}
            </Tag>,
            <span key="grp" className="inline-flex items-center gap-1 text-vsc-desc">
              <VscBook size={13} aria-hidden="true" />
              {chapterTitle(topic.group)}
            </span>,
          ]}
          description={topic.subtitle()}
          actions={
            topic.targetPage ? (
              <Button
                small
                variant="primary"
                onClick={() => {
                  if (topic.targetPage) onNavigate(topic.targetPage);
                }}
              >
                <span className="inline-flex items-center gap-1">
                  {tr(
                    "Apri questa vista nell'app",
                    "Open this view in app",
                    "Ouvrir cette vue dans l'application",
                    "Diese Ansicht in der App öffnen",
                  )}
                  <VscArrowRight size={12} aria-hidden="true" />
                </span>
              </Button>
            ) : undefined
          }
        />

        <div ref={bodyWrapRef} className="flex min-h-0 flex-1 flex-col">
          <EditorBody>
            <div className="max-w-245 space-y-5">
              <GuideTopicContent topicId={topic.id} info={info} components={components} />
            </div>

            <div className="flex flex-wrap items-center justify-between gap-2 border-t border-vsc-border pt-4">
              <div>
                {prev && (
                  <Button
                    variant="secondary"
                    onClick={() => {
                      setTopicId(prev.id);
                    }}
                  >
                    <span className="inline-flex items-center gap-1.5">
                      <VscArrowLeft size={13} aria-hidden="true" />
                      <span>{prev.title()}</span>
                    </span>
                  </Button>
                )}
              </div>
              <div>
                {next && (
                  <Button
                    variant="primary"
                    onClick={() => {
                      setTopicId(next.id);
                    }}
                  >
                    <span className="inline-flex items-center gap-1.5">
                      <span>{next.title()}</span>
                      <VscArrowRight size={13} aria-hidden="true" />
                    </span>
                  </Button>
                )}
              </div>
            </div>
          </EditorBody>
        </div>
      </ViewContainer>
    </GuideNavContext.Provider>
  );
}
