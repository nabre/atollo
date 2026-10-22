// The site's renderer of the Guide (D-164): the content of guide/, loaded
// by guide/render/load.cjs, drawn with SITE_WIDGETS into static HTML
// (renderToStaticMarkup; the page runs no React), one page per topic and
// language: /guide/ (the welcome topic) and /guide/<topic>/ in English,
// /<lang>/guide/... for the others. Beside the pages: each language's
// search index (guide/search.json: titles, subtitles, codes and keywords,
// the app's rule, core/search.ts), the Guide's screenshots copied from
// ui/public/guide/<lang>/ with their size, and the codicons as inline SVG.
// site/build.mjs calls guidePages() and writes them with its own model.
import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createElement as h, Fragment } from "react";
import { renderToStaticMarkup } from "react-dom/server";

const repo = dirname(dirname(fileURLToPath(import.meta.url)));
const require = createRequire(import.meta.url);
const { loadGuide } = require("../guide/render/load.cjs");

/** Text for HTML, in elements and attributes. */
export const esc = (s) =>
  String(s).replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll('"', "&quot;");

// ---------- codicons, as the app draws them, here from their SVG
const codicons = join(repo, "node_modules", "@vscode", "codicons", "src", "icons");
const svgs = new Map();
function svg(name) {
  if (!svgs.has(name)) {
    const file = join(codicons, `${name}.svg`);
    if (!existsSync(file)) throw new Error(`guide: no codicon ${name}`);
    svgs.set(name, readFileSync(file, "utf8").trim().replace("<svg ", '<svg aria-hidden="true" focusable="false" '));
  }
  return svgs.get(name);
}
const Icon = ({ name }) => h("span", { className: "g-icon", dangerouslySetInnerHTML: { __html: svg(name) } });

// ---------- the Guide's screenshots: ui/public/guide/<lang>/*.webp
const shotsDir = (lang) => join(repo, "ui", "public", "guide", lang);

/** A webp's width and height, from its header (VP8, VP8L or VP8X). */
export function webpSize(file) {
  const b = readFileSync(file);
  if (b.toString("ascii", 0, 4) !== "RIFF" || b.toString("ascii", 8, 12) !== "WEBP") throw new Error(`${file}: not a webp`);
  const kind = b.toString("ascii", 12, 16);
  if (kind === "VP8 ") return { width: b.readUInt16LE(26) & 0x3fff, height: b.readUInt16LE(28) & 0x3fff };
  if (kind === "VP8L") {
    const bits = b.readUInt32LE(21);
    return { width: (bits & 0x3fff) + 1, height: ((bits >> 14) & 0x3fff) + 1 };
  }
  if (kind === "VP8X") return { width: b.readUIntLE(24, 3) + 1, height: b.readUIntLE(27, 3) + 1 };
  throw new Error(`${file}: unknown webp ${kind}`);
}

// ---------- the drawing of one page: what the widgets need to know
let ctx = null;

/** A section's anchor: its title as a slug, unique in the page. */
function anchor(title) {
  const base =
    title
      .toLowerCase()
      .normalize("NFKD")
      .replace(/[̀-ͯ]/g, "")
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-|-$/g, "") || "section";
  let id = base;
  for (let n = 2; ctx.ids.has(id); n += 1) id = `${base}-${String(n)}`;
  ctx.ids.add(id);
  return id;
}

const list = (tag, className, items) =>
  h(
    tag,
    { className },
    items.map((x, i) => h("li", { key: i }, x)),
  );

/** The site's renderer of the contract (guide/core/widgets.ts). */
export function siteWidgets(guide) {
  const { guideLabels } = guide.core.labels;
  const { topicOf, sectionCode, pageTopic } = guide.core.model;
  const L = () => guideLabels();
  const topicLink = (id, children) => h("a", { href: ctx.href(id) }, children);
  return {
    P: ({ children }) => h("p", null, children),
    Steps: ({ steps }) => list("ol", "g-steps", steps),
    Bullets: ({ items }) => list("ul", "g-bullets", items),
    Section: ({ title, children }) => {
      const id = anchor(title);
      return h(
        "section",
        { className: "g-section" },
        h("h2", { id }, h("a", { href: `#${id}`, className: "g-anchor" }, title)),
        children,
      );
    },
    InBrief: ({ points }) =>
      h(
        "div",
        { className: "g-brief" },
        h("p", { className: "g-label" }, h(Icon, { name: "lightbulb" }), L().inShort),
        list("ul", null, points),
      ),
    Callout: ({ tone, title, children }) =>
      h(
        "aside",
        { className: `g-callout g-${tone}` },
        h(
          "p",
          { className: "g-label" },
          h(Icon, { name: { tip: "lightbulb", info: "info", warn: "warning" }[tone] }),
          title ?? L()[tone],
        ),
        h("div", null, children),
      ),
    Under: ({ title, children }) =>
      h(
        "details",
        { className: "g-under" },
        h("summary", null, h(Icon, { name: "tools" }), title ?? L().under),
        h("div", null, children),
      ),
    QA: ({ q, children }) => h("div", { className: "g-qa" }, h("p", { className: "g-q" }, q), h("div", null, children)),
    Flow: ({ steps }) =>
      h(
        "ol",
        { className: "g-flow" },
        steps.map((s) => h("li", { key: s.title }, h("b", null, s.title), h("span", null, s.text))),
      ),
    Table: ({ columns, rows }) =>
      h(
        "div",
        { className: "g-table" },
        h(
          "table",
          null,
          h("thead", null, h("tr", null, columns.map((c, i) => h("th", { key: i, scope: "col" }, c)))),
          h(
            "tbody",
            null,
            rows.map((r, i) => h("tr", { key: i }, r.map((c, j) => h("td", { key: j }, c)))),
          ),
        ),
      ),
    Key: ({ children }) => h("kbd", null, children),
    Ui: ({ children }) => h("strong", { className: "g-ui" }, "«", children, "»"),
    Code: ({ children, indent }) =>
      indent === undefined
        ? h("code", null, children)
        : h("code", { className: "g-tree", style: { marginLeft: `${String(indent * 1.25)}em` } }, children),
    Cards: ({ children }) => h("div", { className: "g-cards" }, children),
    GuideCode: ({ title, path, code }) =>
      h(
        "figure",
        { className: "g-codeblock" },
        h(
          "figcaption",
          null,
          h(Icon, { name: "file-code" }),
          h("b", null, title),
          path ? h("code", null, path) : null,
          h("button", { type: "button", className: "g-copy", "data-copied": L().copied }, L().copy),
        ),
        h("pre", null, h("code", null, code)),
      ),
    GuideShot: ({ src, title, caption, example, scenario }) => {
      const shot = ctx.shot(src);
      return h(
        "figure",
        { className: "g-shot" },
        h(
          "figcaption",
          { className: "g-shot-head" },
          h("b", null, title),
          h("span", { className: "g-badge" }, scenario ?? L().sample),
        ),
        h(
          "a",
          { className: "g-zoom", href: shot.href, title: L().zoomHint },
          h("img", { src: shot.href, alt: title, loading: "lazy", width: shot.width, height: shot.height }),
        ),
        h(
          "div",
          { className: "g-caption" },
          h("p", null, caption),
          example
            ? h(
                "div",
                { className: "g-example" },
                h("p", { className: "g-label" }, h(Icon, { name: "info" }), L().example),
                h("div", null, example),
              )
            : null,
        ),
      );
    },
    GuideCard: ({ icon, badge, title, children, to, actionLabel }) => {
      let action = null;
      if (to) {
        const id = "topic" in to ? to.topic : pageTopic(to.page);
        const label = "topic" in to ? (actionLabel ?? L().read) : L().read;
        if (id !== ctx.topic) action = h("a", { className: "g-card-action", href: ctx.href(id) }, label, " →");
      }
      return h(
        "div",
        { className: "g-card" },
        h(
          "div",
          { className: "g-card-head" },
          h(Icon, { name: icon }),
          h("h3", null, title),
          badge ? h("span", { className: "g-badge g-focus" }, badge) : null,
        ),
        h("div", null, children),
        action,
      );
    },
    See: ({ id, children }) => topicLink(id, children ?? `${topicOf(id).title()} (${sectionCode(topicOf(id))})`),
    Related: ({ ids }) =>
      h(
        "nav",
        { className: "g-related" },
        h("p", { className: "g-label" }, L().related),
        h(
          "ul",
          null,
          ids.map((id) => h("li", { key: id }, topicLink(id, `${topicOf(id).title()} (${sectionCode(topicOf(id))})`))),
        ),
      ),
    // A page of the app: here, the topic that describes it; nothing when
    // that is this very topic.
    Go: ({ page, children }) => {
      const id = pageTopic(page);
      return id === ctx.topic ? null : h("a", { className: "g-go", href: ctx.href(id) }, children, " →");
    },
    KitFoldersBar: () =>
      h(
        "dl",
        { className: "g-folders" },
        guide.folders.kitFolders().map((f) => h(Fragment, { key: f.id }, h("dt", null, f.label), h("dd", null, h("code", null, f.path)))),
      ),
    Badge: ({ tone, children }) => h("span", { className: tone ? "g-badge g-focus" : "g-badge" }, children),
    // The site knows no kit.
    KitInfo: () => null,
    SourcesTable: ({ rows, categories }) => {
      const l = L().sources;
      return h(
        "div",
        { className: "g-sources" },
        h(
          "div",
          { className: "g-filters" },
          h("button", { type: "button", "data-group": "", "aria-pressed": "true" }, `${l.all} (${String(rows.length)})`),
          categories.map((c) => h("button", { type: "button", key: c.id, "data-group": c.id, "aria-pressed": "false" }, c.label)),
          h("input", { type: "search", className: "g-filter", placeholder: l.filter, "aria-label": l.filter }),
        ),
        h(
          "div",
          { className: "g-table" },
          h(
            "table",
            null,
            h("thead", null, h("tr", null, l.columns.map((c, i) => h("th", { key: i, scope: "col" }, c)))),
            h(
              "tbody",
              null,
              rows.map((r) =>
                h(
                  "tr",
                  {
                    key: r.id,
                    "data-group": r.group,
                    "data-text": [r.id, r.name, r.home, r.metaSource, r.binarySource, r.verifyKind, r.targetPath].join(" ").toLowerCase(),
                  },
                  h("td", null, h("a", { href: r.home, rel: "noopener" }, h("b", null, r.name))),
                  h("td", null, h("code", null, r.metaSource)),
                  h("td", null, h("code", null, r.binarySource), r.unix ? h("p", null, r.unix) : null),
                  h("td", null, h("span", { className: "g-badge g-focus" }, r.verifyKind), h("p", null, r.verifyDetail)),
                  h("td", null, h("code", null, r.targetPath)),
                ),
              ),
            ),
          ),
        ),
      );
    },
    FilterList: ({ groups, placeholder }) =>
      h(
        "div",
        { className: "g-filterlist" },
        h("input", { type: "search", className: "g-filter", placeholder, "aria-label": placeholder }),
        groups.map((g) =>
          h(
            "section",
            { className: "g-section", key: g.title },
            h("h2", { id: anchor(g.title) }, g.title),
            g.items.map((p) =>
              h(
                "div",
                { className: "g-qa", key: p.q, "data-text": `${p.q} ${p.a}`.toLowerCase() },
                h("p", { className: "g-q" }, p.q),
                h(
                  "div",
                  null,
                  h("p", null, p.a),
                  p.see ? h("p", null, L().learnMore, " ", topicLink(p.see, `${topicOf(p.see).title()} (${sectionCode(topicOf(p.see))})`)) : null,
                ),
              ),
            ),
          ),
        ),
      ),
    Terms: ({ items }) =>
      h(
        "dl",
        { className: "g-terms" },
        items.map(([term, text]) => h(Fragment, { key: term }, h("dt", null, term), h("dd", null, text))),
      ),
    WindowSketch: ({ title, activity, sidebar, tabs, panel, status }) =>
      h(
        "div",
        { className: "g-sketch", role: "img", "aria-label": [title, activity, sidebar, tabs, panel, status].join(", ") },
        h("div", { className: "g-sk-title" }, title),
        h("div", { className: "g-sk-activity" }, activity),
        h("div", { className: "g-sk-side" }, sidebar),
        h("div", { className: "g-sk-tabs" }, tabs),
        h("div", { className: "g-sk-panel" }, panel),
        h("div", { className: "g-sk-status" }, status),
      ),
    Only: ({ in: where, children }) => (where === "web" ? children : null),
  };
}

/**
 * Loads the Guide and binds it to the site: a language that the caller
 * sets before each drawing, the site's widgets. Returns what draws.
 */
export function siteGuide() {
  const guide = loadGuide();
  let lang = "en";
  guide.core.lang.bindGuideLang(() => lang);
  const widgets = siteWidgets(guide);
  guide.core.widgets.registerGuideWidgets(widgets);
  const { GUIDE_TOPICS } = guide.core.model;
  const sizes = new Map();
  return {
    guide,
    widgets,
    setLang: (l) => {
      lang = l;
    },
    /**
     * A topic's text as HTML in language `l`. `href(id)` gives a topic's
     * address from this page, `shotBase` the screenshots' folder from it.
     */
    renderTopic(l, id, { href = (t) => `../${t}/`, shotBase = "" } = {}) {
      lang = l;
      const text = guide.content.TOPIC_TEXTS[id];
      if (!text) throw new Error(`guide: no text for ${id}`);
      if (!GUIDE_TOPICS.some((t) => t.id === id)) throw new Error(`guide: no topic ${id}`);
      ctx = {
        topic: id,
        ids: new Set(),
        href,
        shot: (src) => {
          const file = join(shotsDir(l), src);
          if (!existsSync(file)) throw new Error(`guide ${l}/${id}: no screenshot ${src}`);
          if (!sizes.has(file)) sizes.set(file, webpSize(file));
          return { href: `${shotBase}${src}`, ...sizes.get(file) };
        },
      };
      try {
        return renderToStaticMarkup(h(Fragment, null, text({ info: null, components: null })));
      } finally {
        ctx = null;
      }
    },
  };
}

/** A topic's address from the site's root in language l. */
export const topicPath = (pathOf, l, id) => `${pathOf(l)}guide/${id === "welcome" ? "" : `${id}/`}`;

/**
 * The Guide's pages for site/build.mjs, in every language: page
 * descriptors for its renderPage, plus the search indexes and the
 * screenshots written into `out`.
 */
export function guidePages({ langs, strings, pathOf, out, model, base, jsonLdApp }) {
  const site = siteGuide();
  const { guide } = site;
  const { GUIDE_GROUPS, GUIDE_TOPICS, sectionCode, chapterTitle, pageName } = guide.core.model;
  const { topicHaystack } = guide.core.search;
  const pages = [];
  for (const lang of langs) {
    site.setLang(lang);
    const s = strings[lang];
    // The screenshots of this language, beside the site's own.
    const shots = join(out, "assets", "guide", lang);
    mkdirSync(shots, { recursive: true });
    for (const f of readdirSync(shotsDir(lang)).filter((f) => f.endsWith(".webp"))) cpSync(join(shotsDir(lang), f), join(shots, f));
    // The search index: the app's haystack of each topic (core/search.ts).
    const index = GUIDE_TOPICS.map((t) => ({
      id: t.id,
      code: sectionCode(t),
      title: t.title(),
      chapter: chapterTitle(t.group),
      href: t.id === "welcome" ? "" : `${t.id}/`,
      hay: topicHaystack(t),
    }));
    mkdirSync(join(out, pathOf(lang), "guide"), { recursive: true });
    writeFileSync(join(out, pathOf(lang), "guide", "search.json"), JSON.stringify(index));

    GUIDE_TOPICS.forEach((topic, i) => {
      const here = topicPath(pathOf, lang, topic.id);
      const root = "../".repeat(here.split("/").length - 1);
      const guideBase = `${root}${pathOf(lang)}guide/`;
      const href = (id) => `${guideBase}${id === "welcome" ? "" : `${id}/`}`;
      const body = site.renderTopic(lang, topic.id, { href, shotBase: `${root}assets/guide/${lang}/` });
      const nav = GUIDE_GROUPS.map((g) => {
        const items = GUIDE_TOPICS.filter((t) => t.group === g.id)
          .map((t) => {
            const current = t.id === topic.id ? ' aria-current="page"' : "";
            return `<li><a href="${href(t.id)}"${current}><span class="g-code">${sectionCode(t)}</span> ${esc(t.title())}</a></li>`;
          })
          .join("");
        return `<section class="g-group"><h2>${esc(chapterTitle(g.id))}</h2><ol>${items}</ol></section>`;
      }).join("");
      const prev = GUIDE_TOPICS[i - 1];
      const next = GUIDE_TOPICS[i + 1];
      const pager =
        (prev ? `<a class="g-prev" href="${href(prev.id)}" rel="prev"><span>${esc(s.guidePrev)}</span>${esc(prev.title())}</a>` : "<span></span>") +
        (next ? `<a class="g-next" href="${href(next.id)}" rel="next"><span>${esc(s.guideNext)}</span>${esc(next.title())}</a>` : "");
      const title = `${sectionCode(topic)} ${topic.title()} — ${s.guideTitle}`;
      const inApp = topic.targetPage
        ? `    <p class="g-inapp">${esc(s.guideInApp)}: <strong>«${esc(pageName(topic.targetPage))}»</strong></p>`
        : "";
      const jsonLd = JSON.stringify({
        "@context": "https://schema.org",
        "@type": "TechArticle",
        headline: topic.title(),
        description: topic.subtitle(),
        inLanguage: lang,
        url: `${base}${here}`,
        isPartOf: jsonLdApp(lang),
      }).replaceAll("<", "\\u003c");
      pages.push({
        lang,
        path: (l) => topicPath(pathOf, l, topic.id),
        model,
        name: "guide.html",
        values: {
          title: esc(title),
          description: esc(topic.subtitle()),
          ogType: "article",
          jsonLd,
          headExtra: `\n<link rel="stylesheet" href="${root}assets/guide.css">\n<script src="${root}assets/guide.js" defer></script>`,
          guideCurrent: topic.id === "welcome" ? ' aria-current="page"' : "",
          guideIndex: `${guideBase}search.json`,
          guideBase,
          guideNav: nav,
          guideChapter: esc(chapterTitle(topic.group)),
          guideCode: sectionCode(topic),
          guideHeading: esc(topic.title()),
          guideSubtitle: esc(topic.subtitle()),
          guideInAppLine: inApp,
          guideBody: body,
          guidePager: pager,
        },
      });
    });
  }
  return { pages, topics: GUIDE_TOPICS.map((t) => t.id) };
}
