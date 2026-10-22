# The Guide

The text of Atollo's Guide, in Italian, English, French and German, drawn by two renderers: the app (`ui/src/guide`, the Guide page, interactive) and the website (`site/guide.mjs`, one static page per topic and language, https://nabre.github.io/atollo/guide/). See D-164 in `docs/app/DECISIONS.md`.

This folder imports nothing from the app or the site, only React and its own files: the site's loader (`render/load.cjs`) refuses any other import, and `test/ui/guide/contract.test.ts` checks it.

- `core/lang.ts`: `tr()` with the four texts, `rich()`, and the language, which each renderer binds (`bindGuideLang`). Nothing may keep a `tr()` result from the time a module loads: one process draws the four languages one after another.
- `core/model.ts`: chapters and topics (title, subtitle, codicon name, keywords, the page of the app they describe), `pageTopic`, `sectionCode`.
- `core/widgets.ts`: **the contract**, the widgets' names and props. The chapters draw with its stand-ins; a renderer registers a function for every name (`registerGuideWidgets`), or the stand-in throws.
- `core/labels.ts`: the words the widgets write themselves; `core/search.ts`: the search rule; `core/types.ts`: the plain shapes a topic receives.
- `content/`: one file per chapter, `sources.ts` (the table of sources), `folders.ts` (the kit's folders), `index.ts` (every topic's text).
- `render/load.cjs`: the site's loader of these TypeScript files.

Rules for the text: no `className` or `style` (a widget instead), `info` only inside `KitInfo`, links as `See`, `Go` or a card's `to`, never a function; text true only in the app or only on the site inside `Only`.
