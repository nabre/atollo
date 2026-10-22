// Loads the Guide's TypeScript for Node, for the site only (site/guide.mjs;
// the app has Vite). Each file goes through typescript.transpileModule
// (CommonJS, React's automatic JSX), with nothing installed beyond what
// `npm ci` gives the site's workflow. It is also the check of the Guide's
// neutrality at build time: a file of guide/ may require React and other
// files of guide/, nothing else ("guide/ must not import <spec>").
//
//   const { loadGuide } = require("guide/render/load.cjs");
//   const guide = loadGuide(); // guide.core.lang, guide.core.widgets, guide.content...
//
// Every call loads afresh (its own modules, its own widgets' registry).
const fs = require("fs");
const path = require("path");
const { createRequire } = require("module");

const GUIDE = path.resolve(__dirname, "..");
const ROOT = path.dirname(GUIDE);
const requireFromRoot = createRequire(path.join(ROOT, "package.json"));
const ts = requireFromRoot("typescript");

/** The bare modules a file of the Guide may use. */
const PACKAGES = new Set(["react", "react/jsx-runtime"]);

function loadGuide() {
  const cache = new Map();

  function load(file) {
    const cached = cache.get(file);
    if (cached) return cached.exports;
    const out = ts.transpileModule(fs.readFileSync(file, "utf8"), {
      fileName: file,
      compilerOptions: {
        module: ts.ModuleKind.CommonJS,
        target: ts.ScriptTarget.ES2022,
        jsx: ts.JsxEmit.ReactJSX,
        esModuleInterop: true,
        verbatimModuleSyntax: false,
      },
    }).outputText;
    const mod = { exports: {} };
    cache.set(file, mod);
    const req = (spec) => {
      if (PACKAGES.has(spec)) return requireFromRoot(spec);
      if (!spec.startsWith(".")) throw new Error(`guide/ must not import ${spec} (${path.relative(ROOT, file)})`);
      const abs = path.resolve(path.dirname(file), spec);
      if (!(abs + path.sep).startsWith(GUIDE + path.sep) && abs !== GUIDE)
        throw new Error(`guide/ must not import ${spec} (${path.relative(ROOT, file)})`);
      for (const candidate of [abs + ".ts", abs + ".tsx", path.join(abs, "index.ts")]) {
        if (fs.existsSync(candidate)) return load(candidate);
      }
      throw new Error(`guide/: no file for ${spec} (${path.relative(ROOT, file)})`);
    };
    // The repository's own sources, transpiled: a build step, no outside input.
    new Function("require", "module", "exports", out)(req, mod, mod.exports);
    return mod.exports;
  }

  const core = {};
  for (const name of ["lang", "labels", "model", "search", "types", "widgets"]) {
    core[name] = load(path.join(GUIDE, "core", `${name}.ts`));
  }
  return {
    core,
    content: load(path.join(GUIDE, "content", "index.ts")),
    sources: load(path.join(GUIDE, "content", "sources.ts")),
    folders: load(path.join(GUIDE, "content", "folders.ts")),
  };
}

module.exports = { loadGuide, GUIDE };
