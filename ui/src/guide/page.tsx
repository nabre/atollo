// Main GuidePage view built on the VSCodium Workbench template:
// - Left SideBar with instant search and 5 collapsible chapter ViewPanes
// - Top EditorTabs for the 5 chapters + EditorHeader with direct "Vai alla pagina" button
// - Scrollable EditorBody with GuideTopicContent and Prev/Next topic navigation.
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
  PaneStack,
  SearchBox,
  SideBarTitle,
  ViewContainer,
  ViewPane,
} from "../workbench";
import {
  DEFAULT_GUIDE_TOPIC,
  GUIDE_GROUPS,
  GUIDE_TOPICS,
  tr,
  type AppPageTarget,
  type GuideGroupId,
  type GuideTopicId,
} from "./model";
import { GuideTopicContent } from "./sections";

export function GuidePage({
  info,
  initialTopic,
  onNavigate,
}: {
  info: AppInfo | null;
  initialTopic?: GuideTopicId | null;
  onNavigate: (page: AppPageTarget) => void;
}) {
  const [topicId, setTopicId] = useState<GuideTopicId>(initialTopic ?? "overview");
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
        // Ignore in test environments or if backend is busy
      });
    return () => {
      alive = false;
    };
  }, []);

  // Scroll editor body back to top when switching topics.
  useEffect(() => {
    const scroller = bodyWrapRef.current?.querySelector(".overflow-y-auto");
    if (scroller instanceof HTMLElement) scroller.scrollTop = 0;
  }, [topicId]);

  const currentTopic = useMemo(() => GUIDE_TOPICS.find((x) => x.id === topicId) ?? DEFAULT_GUIDE_TOPIC, [topicId]);

  const currentIndex = useMemo(() => GUIDE_TOPICS.findIndex((x) => x.id === currentTopic.id), [currentTopic]);
  const prevTopic = currentIndex > 0 ? GUIDE_TOPICS[currentIndex - 1] : null;
  const nextTopic = currentIndex >= 0 && currentIndex < GUIDE_TOPICS.length - 1 ? GUIDE_TOPICS[currentIndex + 1] : null;

  const filteredTopics = useMemo(() => {
    const q = search.trim().toLowerCase();
    if (!q) return GUIDE_TOPICS;
    return GUIDE_TOPICS.filter(
      (item) =>
        item.title().toLowerCase().includes(q) ||
        item.subtitle().toLowerCase().includes(q) ||
        item.sectionCode.toLowerCase().includes(q) ||
        item.keywords.toLowerCase().includes(q),
    );
  }, [search]);

  const toggleGroup = (id: GuideGroupId) => {
    setClosed((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const selectTopic = (id: GuideTopicId) => {
    setTopicId(id);
  };

  const selectGroup = (groupId: string) => {
    const first = GUIDE_TOPICS.find((x) => x.group === groupId);
    if (first) setTopicId(first.id);
  };

  const TopicIcon = currentTopic.icon;

  return (
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
          <PaneStack>
            {GUIDE_GROUPS.map((grp, idx) => {
              const items = filteredTopics.filter((x) => x.group === grp.id);
              if (search.trim() && items.length === 0) return null;
              return (
                <ViewPane
                  key={grp.id}
                  first={idx === 0}
                  fill
                  title={grp.title()}
                  badge={String(items.length)}
                  expanded={!closed.has(grp.id) || search.trim() !== ""}
                  onToggle={() => {
                    toggleGroup(grp.id);
                  }}
                >
                  <List label={grp.title()}>
                    {items.map((item) => {
                      const Icon = item.icon;
                      return (
                        <ListRow
                          key={item.id}
                          selected={item.id === currentTopic.id}
                          onSelect={() => {
                            selectTopic(item.id);
                          }}
                          icon={<Icon size={15} aria-hidden="true" />}
                          label={item.title()}
                          decorations={<span className="font-mono text-[11px] opacity-70">{item.sectionCode}</span>}
                        />
                      );
                    })}
                  </List>
                </ViewPane>
              );
            })}
          </PaneStack>
        </>
      }
    >
      <EditorTabs
        active={currentTopic.group}
        onSelect={selectGroup}
        tabs={GUIDE_GROUPS.map((g) => ({
          id: g.id,
          label: g.title(),
          pinned: true,
        }))}
        actions={<Tag tone="focus">{`${String(currentIndex + 1)} / ${String(GUIDE_TOPICS.length)}`}</Tag>}
      />
      <EditorHeader
        icon={<TopicIcon size={44} aria-hidden="true" className="text-vsc-focus" />}
        title={currentTopic.title()}
        meta={[
          <Tag key="sec" tone="focus">
            {currentTopic.sectionCode}
          </Tag>,
          <span key="grp" className="inline-flex items-center gap-1 text-vsc-desc">
            <VscBook size={13} aria-hidden="true" />
            {GUIDE_GROUPS.find((g) => g.id === currentTopic.group)?.title()}
          </span>,
        ]}
        description={currentTopic.subtitle()}
        actions={
          currentTopic.targetPage ? (
            <Button
              small
              variant="primary"
              onClick={() => {
                if (currentTopic.targetPage) onNavigate(currentTopic.targetPage);
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
          <GuideTopicContent
            topicId={currentTopic.id}
            info={info}
            components={components}
            onNavigate={onNavigate}
            onSelectTopic={selectTopic}
          />

          <div className="flex flex-wrap items-center justify-between gap-2 border-t border-vsc-border pt-4">
            <div>
              {prevTopic && (
                <Button
                  variant="secondary"
                  onClick={() => {
                    selectTopic(prevTopic.id);
                  }}
                >
                  <span className="inline-flex items-center gap-1.5">
                    <VscArrowLeft size={13} aria-hidden="true" />
                    <span>{prevTopic.title()}</span>
                  </span>
                </Button>
              )}
            </div>
            <div>
              {nextTopic && (
                <Button
                  variant="primary"
                  onClick={() => {
                    selectTopic(nextTopic.id);
                  }}
                >
                  <span className="inline-flex items-center gap-1.5">
                    <span>{nextTopic.title()}</span>
                    <VscArrowRight size={13} aria-hidden="true" />
                  </span>
                </Button>
              )}
            </div>
          </div>
        </EditorBody>
      </div>
    </ViewContainer>
  );
}
