// Writes docs/app/GUIDE.md (English) and GUIDE.it.md (Italian) from the
// in-app Guide (ui/src/guide), so the documents never drift from the app.
// It runs the topic files with React and the widgets replaced by stand-ins
// and turns each widget into Markdown.
//
//   node scripts/guide-docs.cjs           (both languages)
//   node scripts/guide-docs.cjs it        (one of them)
//   node scripts/guide-docs.cjs --check   (fails if a document is behind)
//
// Needs the repository's node_modules (TypeScript). Run it after any change
// to the Guide's text; the CI runs --check. The screenshots are
// ui/public/guide/<lang>/.
const fs = require("fs");
const path = require("path");

const ROOT = path.resolve(__dirname, "..");
const ts = require(path.join(ROOT, "node_modules", "typescript"));
const GUIDE = path.join(ROOT, "ui", "src", "guide").replace(/\\/g, "/");

/** The words of the documents that the Guide's widgets write themselves. */
const WORDS = {
  en: {
    file: "GUIDE.md",
    title: "# Atollo Guide",
    bar: "English · [Italiano](GUIDE.it.md)",
    contents: "## Contents",
    inShort: "In short",
    callout: { tip: "Tip", info: "Good to know", warn: "Watch out" },
    under: "Under the hood",
    related: "Related topics: ",
    example: "Practical example in screenshot",
    quote: ["“", "”"],
    folders: [
      ["Projects folder", "projects\\"],
      ["Shared agents", "data\\agents\\"],
      ["Database data", "data\\db\\"],
      ["Database backups", "data\\projects\\<project>\\private\\backups\\"],
    ],
    sources: ["Component", "Versions from", "Files from", "Check", "In data\\apps\\<system>\\"],
    end: "The same guide is in the app, on the Guide page, in four languages.",
  },
  it: {
    file: "GUIDE.it.md",
    title: "# Guida di Atollo",
    bar: "[English](GUIDE.md) · Italiano",
    contents: "## Indice",
    inShort: "In breve",
    callout: { tip: "Consiglio", info: "Da sapere", warn: "Attenzione" },
    under: "Sotto il cofano",
    related: "Argomenti collegati: ",
    example: "Esempio pratico nello screenshot",
    quote: ["«", "»"],
    folders: [
      ["Cartella progetti", "projects\\"],
      ["Agenti condivisi", "data\\agents\\"],
      ["Dati database", "data\\db\\"],
      ["Backup database", "data\\projects\\<progetto>\\private\\backups\\"],
    ],
    sources: ["Componente", "Versioni da", "File da", "Verifica", "In data\\apps\\<sistema>\\"],
    end: "La stessa guida è nell'app, pagina Guida, nelle quattro lingue.",
  },
};

// ---------- the Guide's modules, with stand-ins for React and the widgets
const FRAG = Symbol("frag");
function h(type, props, ...children) {
  return { type, props: { ...(props || {}), children: children.length <= 1 ? children[0] : children } };
}
function marker(name) {
  const f = function () {};
  f.__marker = name;
  return f;
}
const markerProxy = new Proxy({}, { get: (_, k) => (k === "__esModule" ? false : marker(String(k))) });
const react = {
  useState: (v) => [typeof v === "function" ? v() : v, () => {}],
  useMemo: (f) => f(),
  useContext: (c) => c._default,
  createContext: (d) => ({ _default: d }),
  useId: () => "id",
  useRef: () => ({ current: null }),
  Fragment: FRAG,
  createElement: h,
};
function richFn(text, ...nodes) {
  const parts = text.split(/\{(\d+)\}/).map((part, i) => (i % 2 ? nodes[Number(part)] : part));
  return { type: FRAG, props: { children: parts } };
}

function loadGuide(lang) {
  const cache = {};
  function load(file) {
    if (cache[file]) return cache[file].exports;
    const out = ts.transpileModule(fs.readFileSync(file, "utf8"), {
      compilerOptions: {
        module: ts.ModuleKind.CommonJS,
        target: ts.ScriptTarget.ES2022,
        jsx: ts.JsxEmit.React,
        jsxFactory: "__h",
        jsxFragmentFactory: "__F",
      },
    }).outputText;
    const mod = { exports: {} };
    cache[file] = mod;
    const req = (spec) => {
      if (spec === "react") return react;
      if (spec === "react-icons/vsc") return markerProxy;
      const abs = path.resolve(path.dirname(file), spec).replace(/\\/g, "/");
      if (abs.endsWith("/src/i18n")) return { rich: richFn, lang: () => lang, t: (s) => s };
      if (abs === GUIDE + "/widgets") {
        return new Proxy(
          { GuideNavContext: { _default: { select() {}, open() {} } } },
          { get: (o, k) => (k in o ? o[k] : k === "__esModule" ? false : marker(String(k))) },
        );
      }
      if (abs === GUIDE + "/model" || abs === GUIDE + "/sources" || abs.startsWith(GUIDE + "/topics/")) {
        for (const ext of [".ts", ".tsx"]) if (fs.existsSync(abs + ext)) return load(abs + ext);
      }
      return markerProxy; // tags, workbench, logos, api
    };
    // The repository's own sources, transpiled: a build step, no outside input.
    new Function("require", "module", "exports", "__h", "__F", out)(req, mod, mod.exports, h, FRAG);
    return mod.exports;
  }
  const model = load(GUIDE + "/model.ts");
  const sources = load(GUIDE + "/sources.ts");
  const texts = {};
  for (const f of fs.readdirSync(GUIDE + "/topics")) {
    Object.assign(texts, ...Object.values(load(GUIDE + "/topics/" + f)));
  }
  return { model, sources, texts };
}

// ---------- Markdown
function slug(s) {
  return s
    .toLowerCase()
    .replace(/[^\p{L}\p{M}\p{N}\p{Pc}\- ]/gu, "")
    .replace(/ /g, "-");
}
function codeSpan(s) {
  s = String(s);
  return s.includes("`") ? "`` " + s + " ``" : "`" + s + "`";
}
/** Plain text: words with a backslash become code, Markdown characters escaped. */
function esc(s) {
  return (String(s).match(/(?:<[^<>]*>|\S)+|\s+/g) || [])
    .map((w) => {
      if (/^\s+$/.test(w)) return w;
      if (w.includes("\\")) {
        const m = w.match(/^([(“"«]*)(.*?)([)”"»,.;:]*)$/);
        return m[1] + codeSpan(m[2]) + m[3];
      }
      return w.replace(/([*<>[\]`])/g, "\\$1");
    })
    .join("");
}
const isEl = (n) => n && typeof n === "object" && !Array.isArray(n) && "type" in n;
function kids(n) {
  const c = n.props.children;
  return c === undefined ? [] : Array.isArray(c) ? c : [c];
}
function textOf(n) {
  if (n == null || typeof n === "boolean") return "";
  if (typeof n === "string" || typeof n === "number") return String(n);
  if (Array.isArray(n)) return n.map(textOf).join("");
  return isEl(n) ? kids(n).map(textOf).join("") : "";
}
function name(n) {
  if (n.type === FRAG) return "Fragment";
  if (typeof n.type === "string") return n.type;
  return n.type.__marker || null;
}
const oneLine = (s) => s.replace(/\s*\n\s*/g, " ").trim();
const quote = (s) =>
  s
    .split("\n")
    .map((l) => (l ? "> " + l : ">"))
    .join("\n");
function table(cols, rows) {
  return [
    "| " + cols.join(" | ") + " |",
    "| " + cols.map(() => "---").join(" | ") + " |",
    ...rows.map((r) => "| " + r.join(" | ") + " |"),
  ].join("\n");
}
const BLOCKY = new Set([
  "Fragment", "Cards", "InBrief", "Section", "P", "Steps", "Bullets", "Flow", "Table", "GuideCode", "Callout",
  "Under", "QA", "Related", "GuideShot", "GuideCard", "KitFoldersBar", "dl", "div", "Go", "SearchBox", "Button",
]);

function render(lang) {
  const W = WORDS[lang];
  const { model, sources, texts } = loadGuide(lang);
  const { GUIDE_GROUPS, GUIDE_TOPICS, sectionCode, topicOf } = model;
  const heading = (t) => `${sectionCode(t)} ${t.title()}`;
  const link = (id, text) => `[${text ?? heading(topicOf(id))}](#${slug(heading(topicOf(id))) })`;

  function inline(n) {
    if (n == null || typeof n === "boolean") return "";
    if (typeof n === "string" || typeof n === "number") return esc(n);
    if (Array.isArray(n)) return n.map(inline).join("");
    if (!isEl(n)) return "";
    const nm = name(n);
    if (nm === null) return inline(n.type(n.props));
    switch (nm) {
      case "Code":
      case "code": {
        const pad = n.props.style && parseInt(n.props.style.paddingLeft, 10) / 14;
        return (pad ? "&emsp;".repeat(pad) : "") + codeSpan(textOf(n));
      }
      case "Key":
        return "<kbd>" + textOf(n) + "</kbd>";
      case "Ui":
        return W.quote[0] + inline(kids(n)) + W.quote[1];
      case "See":
        return link(n.props.id, n.props.children !== undefined ? inline(n.props.children) : undefined);
      case "strong":
      case "b":
        return "**" + inline(kids(n)) + "**";
      case "em":
        return "*" + inline(kids(n)) + "*";
      case "Go":
      case "Button":
      case "SearchBox":
      case "ComponentLogo":
        return "";
      default:
        return inline(kids(n));
    }
  }
  const cell = (n) => oneLine(inline(n)).replace(/\|/g, "\\|");
  const isBlock = (n) => Array.isArray(n) || (isEl(n) && (name(n) === null || BLOCKY.has(name(n))));
  function mergeInline(list) {
    const out = [];
    let buf = [];
    const flush = () => {
      const s = oneLine(buf.map(inline).join(""));
      if (s) out.push(s);
      buf = [];
    };
    for (const c of list.flat(Infinity)) {
      if (isBlock(c)) {
        flush();
        out.push(...blocks(c));
      } else buf.push(c);
    }
    flush();
    return out.filter((b) => b !== "");
  }
  // Local components of the topic files, by function name.
  const LOCAL = {
    ChapterMap: () => [
      GUIDE_GROUPS.map((g, i) => {
        const tops = GUIDE_TOPICS.filter((x) => x.group === g.id);
        return `${i + 1}. **[${esc(g.title())}](#${slug(`${i + 1}. ${g.title()}`)})** — ${tops.map((x) => esc(x.title())).join(" · ")}`;
      }).join("\n"),
    ],
    SourcesTable: () => [
      table(
        W.sources.map(esc),
        sources.TOOL_SOURCES.map((r) =>
          [
            "[" + esc(r.name) + "](" + r.home + ")",
            esc(r.metaSource),
            esc(r.binarySource) + (r.unix ? "<br>" + esc(r.unix) : ""),
            esc(r.verifyKind) + " — " + esc(r.verifyDetail),
            codeSpan(r.targetPath),
          ].map((c) => c.replace(/\|/g, "\\|")),
        ),
      ),
    ],
  };
  function windowSketch(n) {
    const t = [];
    const walk = (x) => {
      if (typeof x === "string") t.push(x);
      else if (Array.isArray(x)) x.forEach(walk);
      else if (isEl(x)) kids(x).forEach(walk);
    };
    walk(n);
    const [title, activity, sidebar, tabs, panel, status] = t;
    const A = 10, B = 15, C = 28;
    const width = A + B + C + 10;
    const line = (s) => "| " + s.padEnd(width - 4) + " |";
    const row = (a, b, c) => "| " + a.padEnd(A) + " | " + b.padEnd(B) + " | " + c.padEnd(C) + " |";
    const rowSplit = (a, b) => "| " + a.padEnd(A) + " | " + b.padEnd(B) + " +" + "-".repeat(C + 2) + "+";
    const sep = "+" + "-".repeat(width - 2) + "+";
    const mid = "+" + "-".repeat(A + 2) + "+" + "-".repeat(B + 2) + "+" + "-".repeat(C + 2) + "+";
    const [first, ...rest] = activity.split(" · ");
    const act = [first + " ·", ...rest.join(" ").split(" ")];
    return (
      "```\n" +
      [
        sep,
        line(title),
        mid,
        row(act[0] ?? "", sidebar, tabs),
        row(act[1] ?? "", "", ""),
        rowSplit(act[2] ?? "", ""),
        row(act[3] ?? "", "", panel),
        mid,
        line(status),
        sep,
      ].join("\n") +
      "\n```"
    );
  }
  function blocks(n) {
    if (n == null || typeof n === "boolean" || n === "") return [];
    if (typeof n === "string" || typeof n === "number") return [esc(n)];
    if (Array.isArray(n)) return mergeInline(n);
    if (!isEl(n)) return [];
    const nm = name(n);
    const p = n.props;
    if (nm === null) return LOCAL[n.type.name] ? LOCAL[n.type.name](p) : blocks(n.type(p));
    switch (nm) {
      case "Fragment":
      case "Cards":
        return mergeInline(kids(n));
      case "InBrief":
        return [quote(`**${W.inShort}**\n\n` + p.points.map((x) => "- " + oneLine(inline(x))).join("\n"))];
      case "Section":
        return ["#### " + p.title, ...mergeInline(kids(n))];
      case "P":
        return [oneLine(inline(kids(n)))];
      case "Steps":
        return [p.steps.map((x, i) => `${i + 1}. ` + oneLine(inline(x))).join("\n")];
      case "Bullets":
        return [p.items.map((x) => "- " + oneLine(inline(x))).join("\n")];
      case "Flow":
        return [p.steps.map((x, i) => `${i + 1}. **${esc(x.title)}** — ${esc(x.text)}`).join("\n")];
      case "Table":
        return [table(p.columns.map(esc), p.rows.map((r) => r.map(cell)))];
      case "GuideCode":
        return ["**" + esc(p.title) + "**" + (p.path ? " " + codeSpan(p.path) : ""), "```\n" + p.code + "\n```"];
      case "Callout": {
        const kind = { tip: "TIP", info: "NOTE", warn: "WARNING" }[p.tone];
        const body = mergeInline(kids(n)).join("\n\n");
        return [quote(`[!${kind}]\n**${esc(p.title ?? W.callout[p.tone])}**\n\n` + body)];
      }
      case "Under":
        return ["<details>\n<summary>" + (p.title ?? W.under) + "</summary>", ...mergeInline(kids(n)), "</details>"];
      case "QA":
        return ["**" + esc(p.q) + "**", ...mergeInline(kids(n))];
      case "Related":
        return [W.related + p.ids.map((id) => link(id)).join(", ")];
      case "GuideShot": {
        const out = [`![${esc(p.title)}](../../ui/public/guide/${lang}/${p.src})`, "*" + esc(p.caption) + "*"];
        if (p.example) out.push(quote(`**${W.example}**\n\n` + oneLine(inline(p.example))));
        return out;
      }
      case "GuideCard":
        return [(p.badge ? p.badge + ". " : "") + "**" + esc(p.title) + "** — " + oneLine(inline(kids(n)))];
      case "KitFoldersBar":
        return [W.folders.map(([l, x]) => `- **${l}** — ${codeSpan(x)}`).join("\n")];
      case "dl":
        return [
          kids(n)
            .flat()
            .map((d) => {
              const [dt, dd] = kids(d);
              return `- **${oneLine(inline(dt))}** — ${oneLine(inline(dd))}`;
            })
            .join("\n"),
        ];
      case "div":
        if (String(p.className || "").includes("grid-cols-[48px_1fr]")) return [windowSketch(n)];
        return mergeInline(kids(n));
      case "Go":
      case "SearchBox":
      case "Button":
        return [];
      default:
        return [inline(n)];
    }
  }

  const md = [W.title, W.bar];
  const welcome = texts.welcome({ info: null, components: null });
  const brief = kids(welcome).find((x) => isEl(x) && name(x) === "InBrief");
  md.push(brief.props.points.slice(0, 2).map((x) => oneLine(inline(x))).join(" "));
  md.push(W.contents);
  const toc = [];
  GUIDE_GROUPS.forEach((g, i) => {
    toc.push(`${i + 1}. [${esc(g.title())}](#${slug(`${i + 1}. ${g.title()}`)})`);
    for (const t of GUIDE_TOPICS.filter((x) => x.group === g.id)) toc.push(`   - [${esc(heading(t))}](#${slug(heading(t))})`);
  });
  md.push(toc.join("\n"));
  GUIDE_GROUPS.forEach((g, i) => {
    md.push(`## ${i + 1}. ${g.title()}`);
    for (const t of GUIDE_TOPICS.filter((x) => x.group === g.id)) {
      const fn = texts[t.id];
      if (!fn) throw new Error("no text for " + t.id);
      md.push(`### ${heading(t)}`, "*" + esc(t.subtitle()) + "*", ...blocks(fn({ info: null, components: null })));
    }
  });
  md.push(W.end);
  // Two lists of the same kind in a row would merge: a comment splits them.
  const kind = (b) => (/^1\. /.test(b) ? "ol" : /^- /.test(b) ? "ul" : null);
  const out = [];
  for (const b of md) {
    const prev = out[out.length - 1];
    if (prev !== undefined && kind(b) && kind(prev) === kind(b)) out.push("<!-- -->");
    out.push(b);
  }
  return out.join("\n\n") + "\n";
}

// --check compares instead of writing: it fails when a document is not what
// the Guide gives today (the CI runs it).
const check = process.argv.includes("--check");
const asked = process.argv.slice(2).filter((a) => a !== "--check");
let stale = 0;
for (const lang of asked.length ? asked : Object.keys(WORDS)) {
  if (!WORDS[lang]) throw new Error(`unknown language: ${lang} (en, it)`);
  const file = path.join(ROOT, "docs", "app", WORDS[lang].file);
  const text = render(lang);
  if (!check) {
    fs.writeFileSync(file, text);
    console.log(`${WORDS[lang].file}: written`);
    continue;
  }
  const now = fs.existsSync(file) ? fs.readFileSync(file, "utf8").replace(/\r\n/g, "\n") : "";
  if (now === text) {
    console.log(`${WORDS[lang].file}: up to date`);
    continue;
  }
  stale += 1;
  const a = now.split("\n");
  const b = text.split("\n");
  let i = 0;
  while (i < a.length && i < b.length && a[i] === b[i]) i += 1;
  console.error(`${WORDS[lang].file}: not the Guide's text from line ${String(i + 1)}`);
  console.error(`  in the file:  ${a[i] ?? "(end)"}`);
  console.error(`  from the app: ${b[i] ?? "(end)"}`);
}
if (stale) {
  console.error("Run `node scripts/guide-docs.cjs` and commit the documents.");
  process.exit(1);
}
