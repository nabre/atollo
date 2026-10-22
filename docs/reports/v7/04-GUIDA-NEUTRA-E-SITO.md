# 04 — Guida Neutra e Guida sul Sito (`v7`)

> **Riferimento**: `HEAD 1f84370`. Letti per intero: `ui/src/guide/{widgets.tsx,page.tsx,sections.tsx,index.ts}`, la testa di `model.ts` e `sources.ts`, i componenti locali dei capitoli (`ChapterMap`, `PageMap`, `Properties` e lo schizzo della finestra in `start.tsx`, `SourcesTable` in `inside.tsx`, `Problems` e il glossario in `help.tsx`), `scripts/guide-docs.cjs`, `site/build.mjs`, `site/template.html`, `site/assets/site.js`, `test/ui/guide/*`, `ui/vite.config.ts`, `ui/tsconfig.json`, `eslint.config.mjs`, i `package.json`, `.github/workflows/site.yml` e il passo «Levels» di `app.yml`. Non letti: il testo di ogni capitolo, `site.css` nel dettaglio, `scripts/guide-shots/make.mjs` oltre il percorso d'uscita.
> **Dipendenze verificate**: `typescript` 6.0.3 e `vite` 8.3.1 sono dipendenze dirette; `esbuild` non è installato; `rolldown` 1.2.13 e `jiti` 2.7.0 sono solo transitive; `@vscode/codicons/src/icons/*.svg` e `react-icons/vsc` sono disponibili. Le schermate di `ui/public/guide` pesano 3,5 MB (13 × 4 lingue). La Guida ha 37 argomenti in 9 capitoli.

---

## 1. Obiettivo e decisione del proprietario

**Obiettivo.** Portare il contenuto della Guida fuori da `ui/src`, in una cartella `guide/` alla radice che non importa nulla dall'app né dal sito, e pubblicarlo sul sito come pagine vere (una per argomento e per lingua), così la Guida si legge, si cerca e si indicizza senza installare Atollo.

**Decisione (22/10/2026): niente più Markdown.** Il progetto iniziale prevedeva tre renderer (app, sito, Markdown per `docs/app/GUIDE*.md`). Il proprietario ha deciso di **togliere il renderer Markdown**: la Guida ha solo due renderer.

| Renderer | Dove | Che cosa produce |
|---|---|---|
| **App** | `ui/src/guide` | la pagina Guida dell'app, interattiva (pulsanti verso le pagine, stato reale dei componenti, cartelle del kit) |
| **Sito** | `site/guide.mjs` | pagine statiche `/guide/<argomento>/` per lingua: inglese alla radice, poi `it/`, `fr/`, `de/` |

**Conseguenze**:

- si tolgono `scripts/guide-docs.cjs`, `docs/app/GUIDE.md`, `docs/app/GUIDE.it.md` e `test/ui/guide/docs.test.ts`;
- sparisce il passo `node scripts/guide-docs.cjs --check` della CI ([`app.yml:178`](../../../.github/workflows/app.yml#L178)) e la riga del livello `ui` che sorveglia GUIDE*.md e lo script ([`app.yml:113-114`](../../../.github/workflows/app.yml#L113-L114));
- i link a `GUIDE*.md` nei README (×4), in ARCHITECTURE, TESTING e MARKETING puntano a **https://nabre.github.io/atollo/guide/** (e `…/it/guide/`, `…/fr/guide/`, `…/de/guide/`);
- `GUI-V7-04` e `GUI-V7-05` sono **superati: il renderer Markdown viene tolto**;
- `guide/render/markdown.cjs` non esiste; il caricatore del TSX (`guide/render/load.cjs`) serve **solo al sito**;
- nuova decisione **D-155** in `DECISIONS.md` (§10). Il numero D-153 indicato nella richiesta è già usato dalla diagnostica e D-154 dalle copie di sicurezza dei database, entrambe in `main`: la prima libera è D-155.

Che cosa si perde: una versione della Guida leggibile su GitHub senza rete verso il sito e un confronto testuale riga per riga a ogni modifica. Il primo lo copre il sito (e l'app stessa, che ha la Guida dentro); il secondo lo coprono le prove di §8 (rendering di ogni argomento in ogni lingua, nessun widget mancante, link e immagini verificati).

---

## 2. Rilievi

| ID | Grav. | Stato | Dove | Titolo |
|---|:---:|:---:|---|---|
| `GUI-V7-01` | MEDIA | aperto | [`ui/src/guide/model.ts:8`](../../../ui/src/guide/model.ts#L8) | Il modello della Guida è legato all'app: `tr()` legge l'i18n dell'app, le icone sono componenti codicon |
| `GUI-V7-02` | MEDIA | aperto | [`ui/src/guide/topics/inside.tsx:4`](../../../ui/src/guide/topics/inside.tsx#L4) | I capitoli importano direttamente componenti e API dell'app |
| `GUI-V7-03` | MEDIA | aperto | [`ui/src/guide/topics/start.tsx:30`](../../../ui/src/guide/topics/start.tsx#L30) | La navigazione nel contenuto passa da callback imperative di `GuideNavContext` |
| `GUI-V7-04` | MEDIA | **superato** | [`scripts/guide-docs.cjs:243`](../../../scripts/guide-docs.cjs#L243) | Il renderer Markdown trova il contenuto per nome di funzione e per classe Tailwind |
| `GUI-V7-05` | BASSA | **superato** | [`scripts/guide-docs.cjs:20`](../../../scripts/guide-docs.cjs#L20) | `guide-docs.cjs` ripete le etichette dei widget, e solo in en/it |
| `GUI-V7-06` | BASSA | aperto | [`ui/src/guide/topics/help.tsx:1118`](../../../ui/src/guide/topics/help.tsx#L1118) | Il contenuto porta classi Tailwind/vsc che solo il CSS dell'app conosce |
| `GUI-V7-07` | BASSA | aperto | [`ui/src/guide/topics/start.tsx:465`](../../../ui/src/guide/topics/start.tsx#L465) | Il testo descrive la cornice dell'app, che la pagina del sito non ha |
| `GUI-V7-08` | BASSA | aperto | [`ui/vite.config.ts:38`](../../../ui/vite.config.ts#L38) | I gruppi di chunk della build cercano i percorsi `ui/src/guide` |
| `GUI-V7-09` | BASSA | aperto | [`eslint.config.mjs:10`](../../../eslint.config.mjs#L10) | Lint, formattazione e tipi non coprono una cartella `guide/` alla radice |
| `GUI-V7-10` | MEDIA | aperto | [`test/ui/guide/text.test.tsx:10`](../../../test/ui/guide/text.test.tsx#L10) | Le prove del testo della Guida passerebbero senza file dopo lo spostamento |
| `GUI-V7-11` | MEDIA | aperto | [`.github/workflows/app.yml:122`](../../../.github/workflows/app.yml#L122) | La build del sito non gira mai sulle pull request né nella CI dell'app |
| `GUI-V7-12` | BASSA | aperto | [`site/template.html:52`](../../../site/template.html#L52) | Modello e builder del sito sanno fare una sola pagina per lingua |

### 2.1 Dettaglio

- **`GUI-V7-01`** — `import { lang } from "../i18n"`, `import { codicon } from "../workbench"`; `tr()` chiama `lang()` dell'app; `GuideTopic.icon` è un `IconType` (`codicon("home")`), un componente React che disegna il font dell'app, assente sul sito. Oggi `guide-docs.cjs` aggira il problema sostituendo `../i18n` e facendo da proxy a `../workbench`. **Correzione**: `tr`/`L`/`rich` in `guide/core/lang.ts` con una funzione di lingua iniettata (`bindGuideLang`, §5); icone come nomi di codicon (`"home"`, `"rocket"`): l'app le mappa con `codicon(name)`, il sito inserisce l'SVG di `@vscode/codicons/src/icons/<name>.svg` alla build.
- **`GUI-V7-02`** — `inside.tsx:4-8` importa `urlOpen`, `ComponentInfo`, `ComponentLogo`, `Tag`, `Button`, `Codicon`, `SearchBox`; `start.tsx:3-6` e `help.tsx:4-5` simili. `SourcesTable` (`inside.tsx:266`) usa lo stato reale dei componenti, un filtro interattivo e `urlOpen`; `Problems` (`help.tsx:375`) usa `SearchBox` e `useState`; `Properties` (`start.tsx:475`) mostra i dati del kit. È l'ostacolo principale a una cartella neutra. **Correzione**: widget del contratto `Badge` (al posto di `Tag`), `KitInfo`, `SourcesTable`, `FilterList`; i capitoli importano solo da `guide/core`.
- **`GUI-V7-03`** — `ChapterMap` e `PageMap` chiamano `nav.select()`/`nav.open()` da `GuideNavContext`; una funzione non ha un rendering statico e sul sito diventerebbe un pulsante morto. **Correzione**: navigazione dichiarativa: `See{id}`, `Go{page}`, `GuideCard{to: {topic} | {page}}`; ogni renderer traduce la destinazione (l'app chiama `select`/`open`, il sito scrive un `<a href>`). `GuideNavContext` resta privato di `ui/src/guide`.
- **`GUI-V7-04`, `GUI-V7-05`** — **superati: il renderer Markdown viene tolto.** I problemi descritti (oggetto `LOCAL` cercato per nome di funzione, sonda sulla classe `grid-cols-[48px_1fr]`, percorso fisso del modulo dei widget, etichette `WORDS` ripetute solo in en/it) spariscono con lo script. Resta valida l'idea preventiva di `GUI-V7-05`: le parole che scrivono i widget stessi («In breve», «Suggerimento», «Argomenti collegati», «Copia»…) stanno in `guide/core/labels.ts`, così il sito non ne crea una seconda copia.
- **`GUI-V7-06`** — `help.tsx:1118` (`<dl className="… md:grid-cols-[max-content_minmax(0,1fr)]">`), `start.tsx:973` (`bg-vsc-titlebar`), `inside.tsx:306-394`: 29 `className` nei capitoli. Il sito non ha Tailwind né i token vsc; una volta fuori da `ui/`, Tailwind v4 (`theme.css:17`, senza `@source`) non vedrebbe più queste classi neanche nell'app. **Correzione**: widget `Terms` e `WindowSketch` (e lo stile di `SourcesTable` nella sola implementazione dell'app); una prova che vieta `className=` e `style=` in `guide/content`.
- **`GUI-V7-07`** — «Il pulsante in alto a destra apre la pagina dell'app… La casella di ricerca a sinistra…»: sul sito non c'è. **Correzione**: widget `Only{in: "app" | "web"}`, o un'etichetta fornita dal renderer; controllare gli altri capitoli per riferimenti a posizioni dell'app.
- **`GUI-V7-08`** — `ui/vite.config.ts:36-38` cerca `src/guide/topics/(\w+).tsx` e `src/guide/(model|sources|widgets)`. Dopo lo spostamento nessuna delle due corrisponde: i capitoli finiscono nel bundle iniziale senza che nulla fallisca. **Correzione**: nello stesso commit dello spostamento, `[\\/]guide[\\/]content[\\/](\w+)\.tsx$` e `[\\/]guide[\\/](core|model)[\\/]`, e un controllo sull'uscita della build (numero di `dist/assets/guide-*.js` uguale ai capitoli) nel job `ui`.
- **`GUI-V7-09`** — `eslint.config.mjs` guarda solo `ui/src/**` e `test/ui/**`; gli script `lint` e `format:check` di `ui/package.json` e `ui/tsconfig.json` (`include: ["src", "vite.config.ts"]`) non vedono `guide/`; `projectService` fallisce su file fuori da ogni tsconfig. **Correzione**: `guide/tsconfig.json` (estende `../ui/tsconfig.json`, `include: ["."]`, senza i tipi di Vite), `"guide/**/*.{ts,tsx}"` in ESLint, `../guide` negli script di lint e formattazione.
- **`GUI-V7-10`** — `text.test.tsx:10` usa `import.meta.glob(["../../../ui/src/guide/**/*.ts", …])` e scorre i file senza controllare che ce ne siano; l'eccezione per la definizione di `tr`/`L` è legata a `model.ts`. Dopo lo spostamento le prove «nessuna bozza `L()`» e «ogni `tr()` ha quattro lingue» girerebbero su zero file e passerebbero. **Correzione**: glob su `../../../guide/**`, `expect(Object.keys(files).length).toBeGreaterThan(10)`, eccezione su `guide/core/lang.ts`.
- **`GUI-V7-11`** — nel passo «Levels» `site/*` è tra i percorsi senza livello e `site.yml` gira solo su `push` in `main`: un capitolo che rompe il renderer del sito (un widget mancante, un `See` con id sbagliato) fallirebbe solo nel deploy, dopo il merge. Oggi `guide/*` cadrebbe nel ramo «tutto». **Correzione**: §9.
- **`GUI-V7-12`** — la navigazione usa ancore nude (`#features`), il menu delle lingue torna sempre alla pagina principale, canonical/hreflang/sitemap conoscono solo le quattro radici, e `langDetect` su `/guide/` porterebbe a `/guide/it/` invece di `/it/guide/`. **Correzione**: §3 e §6.

---

## 3. Albero finale

```text
guide/                         neutra: nessun import da ui/ o da site/
  README.md                    riassunto del contratto, in inglese
  tsconfig.json                estende ../ui/tsconfig.json, include ["."], types: []
  core/
    lang.ts                    Lang, LANGS, bindGuideLang(getter), guideLang(), tr(), L(), rich()
    labels.ts                  guideLabels(): le parole che i widget scrivono da sé (In breve, Suggerimento,
                               Da sapere, Attenzione, Sotto il cofano, Argomenti collegati, Esempio pratico,
                               dati d'esempio, Ingrandisci, Copia, Inizia da qui, Apri la pagina, Leggi…) come tr()
    model.ts                   GUIDE_GROUPS, GUIDE_TOPICS (icon: nome di codicon), topicOf, sectionCode,
                               chapterTitle, AppPageTarget, GuideTopicId, pageTopic(page)
    search.ts                  topicHaystack(topic) e matches(words, haystack): una regola per app e sito
    widgets.ts                 IL CONTRATTO: tipi delle props, componenti segnaposto, registerGuideWidgets(impl), WidgetName
    types.ts                   TopicProps { info: KitInfo | null; components: ComponentState[] | null }, tipi strutturali
  content/                     gli ex ui/src/guide/topics, un file per capitolo
    start.tsx projects.tsx services.tsx agents.tsx tools.tsx settings.tsx portable.tsx inside.tsx help.tsx
    sources.ts                 TOOL_SOURCES e sourceCounts (dati puri + tr)
    folders.ts                 KIT_FOLDERS (id, etichetta tr, percorso)
    index.ts                   TOPIC_TEXTS (l'ex sections.tsx senza l'involucro React)
  render/
    load.cjs                   caricatore Node del TSX, usato SOLO dal sito (§4)
ui/src/guide/                  solo il renderer dell'app
  page.tsx                     come oggi, importa da @guide/core
  widgets.tsx                  implementazione del contratto per l'app (vscode-elements, codicon, Tag,
                               dialogo d'ingrandimento, useCopy; GuideNavContext privato qui)
  register.ts                  bindGuideLang(lang) + registerGuideWidgets(APP_WIDGETS), importato una volta
                               da main.tsx (e da test/ui/setup.ts)
  index.ts                     esportazioni pubbliche invariate (GuidePage, GUIDE_GROUPS, GUIDE_TOPICS, TOOL_SOURCES)
site/
  build.mjs                    pagina principale + pagine della Guida
  guide.mjs                    renderer del sito: SITE_WIDGETS (HTML statico con renderToStaticMarkup),
                               montaggio delle pagine, indice di ricerca, voci della sitemap
  partials/head.html header.html footer.html
  landing.html guide.html      (template.html diviso)
  assets/guide.css guide.js    stili dei widget sui token del sito; ricerca, copia, ingrandimento
  strings/<lang>.json          + navGuide, guideTitle, guideSearch, guideNoResult, guidePrev, guideNext, guideInApp
```

Tolti: `scripts/guide-docs.cjs`, `docs/app/GUIDE.md`, `docs/app/GUIDE.it.md`, `test/ui/guide/docs.test.ts`. Non creato: `guide/render/markdown.cjs`.

---

## 4. Contratto dei widget (`guide/core/widgets.ts`)

**Meccanismo.** Ogni widget è un componente segnaposto:

```ts
function widget<K extends WidgetName>(name: K) {
  const C = (props: PropsOf<K>) => current(name)(props); // errore chiaro se non registrato
  C.widget = name;
  return C;
}
export const P = widget("P");
```

`registerGuideWidgets(impl)` imposta l'implementazione a livello di modulo, una volta per processo: l'app all'avvio, il sito prima di ogni giro. In TypeScript `GuideWidgetImpl = { [K in WidgetName]: (p: PropsOf<K>) => ReactNode }`: un renderer a cui manca un widget è un errore di compilazione. Se un renderer dimentica di registrarsi, il segnaposto lancia «guide widgets not registered: P» invece di non disegnare nulla. I capitoli tengono i loro import (ora da `../core/widgets`); il contesto React si usa solo dentro l'implementazione dell'app.

| Widget | App | Sito |
|---|---|---|
| `P`, `Steps`, `Bullets`, `Section{title,id?}`, `InBrief`, `Callout{tone,title?}`, `Under`, `QA`, `Flow`, `Table`, `Key`, `Ui`, `Code`, `Cards` | come oggi | HTML semantico con classi `g-*` (`details/summary` per `Under`, `kbd`, `table`) |
| `Terms{items}` (nuovo, sostituisce il `<dl>` del glossario) | griglia come oggi | `<dl>` |
| `WindowSketch{title,activity,sidebar,tabs,panel,status}` (nuovo, sostituisce il `div` con le classi) | schizzo come oggi | schizzo con `g-*` |
| `GuideCode{title,path?,code}` | pulsante di copia | `<pre>` + pulsante collegato da `guide.js` (senza JS si seleziona il testo) |
| `GuideShot{src,title,caption,example?,scenario?}` | come oggi | `<figure><a href="{base}/{lang}/{src}" class="g-zoom"><img loading=lazy width height>`; `guide.js` apre il `<dialog>`, senza JS il link apre l'immagine. La base la dà il renderer, mai `lang()` nel widget |
| `GuideCard{icon,badge?,title,to?,actionLabel?}` | pulsante → `select`/`open` | `{topic}` → `<a href>` alla pagina; `{page}` → link alla pagina di `pageTopic(page)` con l'etichetta `readTopic` |
| `See{id}`, `Related{ids}` | link → `select` | `<a href="{guideBase}{id}/">` |
| `ChapterMap`, `PageMap` (spostati da `start.tsx` nel contratto, leggono il modello) | da `GuideCard` con `to` | da `GuideCard` con `to` |
| `Badge{tone?}` (sostituisce `Tag`) | `Tag` | `<span class="g-badge">` |
| **Solo app, con ripiego**: | | |
| `Go{page}` (10 usi) | pulsante → `open(page)` | link «→ …» all'argomento di `pageTopic(page)`, niente se è l'argomento corrente |
| `KitFoldersBar` | pulsanti → `folderOpen` | elenco di definizioni da `KIT_FOLDERS` (una fonte) |
| `KitInfo{info}` (il blocco `Properties` del benvenuto; anche gli usi di `info.root` in `inside.tsx:726` e `settings.tsx:606`) | mostrato se `info != null` | il sito passa sempre `info: null`: niente. **Regola**: il contenuto usa `info` solo dentro `KitInfo` |
| `SourcesTable{rows,components}` | come oggi (filtri, `SearchBox`, stato reale, `urlOpen`, `ComponentLogo`) | tabella statica di tutte le righe + campo filtro collegato da `guide.js` (`data-group`); link `<a href rel=noopener>` |
| `FilterList{groups}` (la ricerca dei problemi in `help.tsx`) | `SearchBox` + `useMemo` | tutto disegnato + filtro in `guide.js` |
| `Only{in: "app" \| "web"}` | mostra `app` | mostra `web` |

L'azione d'intestazione «Apri questa vista nell'app» (`page.tsx`) è solo dell'app; il sito mostra una riga «Nell'app: «<nome della pagina>»» (`guideInApp`).

---

## 5. Lingua e `tr`

`guide/core/lang.ts`:

```ts
let get: () => Lang = () => "it";
export function bindGuideLang(g: () => Lang) { get = g; }
export function guideLang() { return get(); }
export function tr(it: string, en: string, fr: string, de: string) { /* sceglie con get(), stesso contratto: quattro letterali, nessun ripiego */ }
```

`rich(text, ...nodes)` usa `createElement`/`Fragment` di React (quello vero, in app e sul sito). L'app chiama `bindGuideLang(lang)` di `ui/src/i18n` in `ui/src/guide/register.ts`: `setLang()` continua a funzionare e nulla si ridisegna diversamente. `ui/src/i18n` tiene il suo `rich()` per il resto dell'app (o riesporta quello della Guida). `Lang` e `LANGS` si spostano in `guide/core` e `ui/src/i18n` li importa: un solo elenco di lingue.

Il sito fa `let current; bindGuideLang(() => current);` e poi per ogni lingua `current = lang; render`. Tutto è pigro (i titoli sono funzioni, `sources.ts` usa getter), quindi un processo disegna le quattro lingue una dopo l'altra. **Regola**: niente può memorizzare il risultato di un `tr()` al caricamento del modulo; una prova la sorveglia (§8).

---

## 6. Il sito

### 6.1 Come `site/build.mjs` carica il TSX

`esbuild` non è installato. Opzioni considerate:

| Opzione | Esito |
|---|---|
| **a.** `typescript.transpileModule` + uno shim `require` CommonJS (quello che fa oggi `guide-docs.cjs`) | **scelta**: funziona con `npm ci --ignore-scripts` di `site.yml`, nessuna dipendenza nuova |
| b. `vite` 8 (diretta): `createServer({ server: { middlewareMode: true }, appType: "custom" }).ssrLoadModule` o `transformWithOxc` | TSX ed ESM completi, ma più pesante e legata al binding nativo di rolldown; ripiego se (a) non basta |
| c. `jiti`/`rolldown` | solo transitive: non contarci |

`guide/render/load.cjs` (usato **solo** dal sito, con `createRequire` da `build.mjs`): opzioni `module: CommonJS`, `jsx: React` (classico), fabbrica e `Fragment` di React; `"react"` è il modulo vero (`renderToStaticMarkup` regge `useState`/`useMemo`/`useId`/`useContext`, e gli hook girano comunque solo nelle implementazioni dell'app). Può richiedere solo `guide/**`: qualunque altro percorso relativo lancia «guide/ must not import <spec>». È il controllo di neutralità alla build.

### 6.2 URL, ancore, SEO

Una pagina per argomento e per lingua (37 × 4 = 148) più un indice per lingua:

| Lingua | Indice | Argomento |
|---|---|---|
| en | `https://nabre.github.io/atollo/guide/` | `/atollo/guide/<id>/` |
| it, fr, de | `https://nabre.github.io/atollo/<lang>/guide/` | `/atollo/<lang>/guide/<id>/` |

- L'indice è l'argomento di benvenuto più la `ChapterMap`. L'id dell'argomento è lo slug in ogni lingua, così le alternative hreflang si accoppiano senza tabelle.
- Ancore di sezione: sul sito `Section` riceve `id = slug(title)`, unico nella pagina (suffisso `-2` in caso di conflitto); l'app ignora gli id (`useId` resta).
- Ogni pagina ha canonical, le quattro alternative + `x-default` (la pagina inglese), `og:url`, JSON-LD `TechArticle` (`headline`, `inLanguage`, `isPartOf` la `SoftwareApplication`) e `<title>{n.m} {titolo} — Atollo Guide</title>`.
- Impaginazione: l'intestazione del sito (wordmark, navigazione con un link «Guida» → `{{root}}{{langPath}}guide/`, il ciclo del tema, il menu delle lingue che punta allo **stesso** argomento nell'altra lingua), una colonna a sinistra con i capitoli e i codici di sezione (un `<details>` su mobile), l'articolo, i link precedente/successivo e il piè di pagina. Font Geist e token chiaro/scuro vengono da `site.css`; `guide.css` aggiunge solo le regole `g-*`.
- Il modello diventa `partials/{head,header,footer}.html` + `landing.html` + `guide.html`; i link della navigazione diventano `{{home}}#features`, così l'intestazione comune funziona su ogni pagina. Il builder riceve descrittori di pagina `{lang, path, alternates(lang)}` da cui calcola menu, canonical, `og:url`, hreflang e sitemap (`GUI-V7-12`).
- La sitemap aggiunge 4 × 38 voci `<url>` con alternative `xhtml:link`. `langDetect` resta solo sulla radice inglese; facoltativo: le pagine della Guida leggono `atollo-lang` da `localStorage` e mostrano un suggerimento «questa pagina in Italiano» invece di reindirizzare.
- Search Console vedrà 148 URL nuovi: reinviare la sitemap.

### 6.3 Ricerca

Lato client, costruita alla build: per ogni lingua `_site/<lang path>guide/search.json = [{id, code, title, subtitle, chapter, hay}]`, dove `hay = topicHaystack(topic)` in minuscolo (titolo, sottotitolo, codice, parole chiave: la stessa regola del filtro di `page.tsx`, spostata in `guide/core/search.ts` e usata anche dall'app). Facoltativo: i primi ~400 caratteri del testo per la ricerca nel testo (circa 25 kB per lingua). `guide.js` carica il file la prima volta che la casella riceve il fuoco e filtra con `words.every(includes)`, la regola dell'app; i risultati prendono il posto dell'elenco dei capitoli, con `guideNoResult` se vuoto. Senza JS la casella è nascosta (classe `.js`) e resta l'elenco dei capitoli.

### 6.4 Schermate

- **Fase 1**: i file restano in `ui/public/guide/<lang>/*.webp`, prodotti dall'anteprima dell'app da `scripts/guide-shots/make.mjs` e serviti da Vite come `/guide/<lang>/`. La build del sito li copia in `_site/assets/guide/<lang>/` (come fa con `site/shots`) e scrive `width`/`height` (1600×1000, letti una volta dall'intestazione webp) per evitare spostamenti dell'impaginazione. `GuideShot` riceve solo il nome del file; la base la dà ogni renderer.
- **Fase 2 facoltativa** (decisione del proprietario, [05](./05-PIANO-ESECUTIVO-V7.md) §5): spostarli in `guide/shots/<lang>/`, con `publicDir` di Vite che resta `ui/public` più un piccolo plugin di copia (o un passo di copia in `dev-ui.mjs`, senza collegamenti simbolici), e aggiornare `make.mjs`, ARCHITECTURE*.md e TESTING*.md (citano il percorso) e la nota di D-136.

---

## 7. Interfaccia dell'app

Nessun cambiamento visibile: `ui/src/guide/page.tsx` disegna come oggi con l'implementazione `APP_WIDGETS`. Cambiano solo gli import (`@guide/core/…`, vedi [03](./03-STRUTTURA-DUPLICATI-ALIAS.md) §5) e la registrazione all'avvio. I nomi dei componenti in React DevTools cambiano (segnaposto).

---

## 8. Prove

**Si tengono, con percorsi nuovi** (in `test/ui/guide`, la scelta più semplice; in alternativa un progetto Vitest `test/guide`):

- `text.test.tsx`: glob su `guide/**`, controllo che i file non siano zero, eccezione della definizione su `guide/core/lang.ts` (`GUI-V7-10`). La prova di rendering gira con i widget dell'app registrati (`test/ui/setup.ts` importa `ui/src/guide/register`).
- `sources.test.ts`: nuovo percorso di `sources.ts`.

**Si toglie**: `docs.test.ts` (controllava che GUIDE.md avesse ogni argomento).

**Si aggiungono**:

- `contract.test`: le implementazioni APP e SITE hanno esattamente le chiavi di `WidgetName`; nessun import fuori da `guide/` in `guide/**`; nessun `className=` né `style=` in `guide/content` (`GUI-V7-06`).
- `site-render.test` (Vitest, ambiente node): per ogni lingua e argomento, `renderToStaticMarkup` con `SITE_WIDGETS`; nessuna eccezione, lunghezza > 200, nessuna classe `vsc-`, nessun segnaposto `{0}` rimasto, ogni `href` a `guide/<id>/` è un id vero, ogni `img` esiste in `ui/public/guide/<lang>/`.
- `lang.test`: disegnare `it` poi `en` nello stesso processo e confrontare con `en` da solo (nessun `tr()` memorizzato al caricamento).
- `search.test`: `page.tsx` e l'indice del sito danno gli stessi risultati per alcune ricerche (entrambi usano `core/search.ts`).
- Prova di fumo della build: `node site/build.mjs --check` disegna senza pubblicare, controlla `_site/guide/<id>/index.html` × 4 lingue e che `sitemap.xml` li elenchi.
- Controllo dei chunk dopo `vite build`: numero di `dist/assets/guide-*.js` uguale ai capitoli (`GUI-V7-08`).

---

## 9. CI

- **`site.yml`**: in `on.push.paths` aggiungere `guide/**` e `ui/public/guide/**` (`site/**` copre già `guide.mjs` e i partial). Il commento del passo d'installazione diventa «react, react-dom, react-icons, typescript, @vscode/codicons»: sono già nel lockfile, `npm ci` non scarica nulla di nuovo.
- **`app.yml`, passo «Levels»**: aggiungere `guide/*) ui=true ;;` prima del ramo «tutto»; togliere `site/*` dalla riga senza livello e farne un livello `site` che nel job `ui` esegue `node site/build.mjs --check` (anche per `guide/*` e `ui/public/guide/*`); togliere la riga `docs/app/GUIDE.md | docs/app/GUIDE.it.md | scripts/guide-docs.cjs` (righe 113-114) e il passo `node scripts/guide-docs.cjs --check` (riga 178); aggiungere `guide/render/*` al livello `ui`.
- Il controllo dei link relativi dei documenti non trova più `GUIDE*.md`: i README puntano al sito, URL assoluti.

---

## 10. Migrazione (ramo `feature/v7-guide-site`, un commit per passo)

Tutto avviene in un ramo, con merge in `main` solo alla fine: gli stati intermedi non arrivano agli utenti.

| # | Passo | Controllo | Rischio |
|---|---|---|---|
| 1 | **Contratto dentro `ui/src/guide`**, senza spostare: `core/lang.ts` con `bindGuideLang`, `labels.ts`, segnaposto e `registerGuideWidgets`; l'implementazione dell'app diventa il `widgets.tsx` di oggi | prove verdi; **ultimo uso di `guide-docs.cjs --check`**: l'uscita Markdown deve restare identica byte per byte, prova che il contratto non cambia nulla | basso |
| 2 | **Togliere il Markdown**: cancellare `scripts/guide-docs.cjs`, `docs/app/GUIDE.md`, `GUIDE.it.md`, `test/ui/guide/docs.test.ts`, il passo e la riga di livello di `app.yml`; **nello stesso commit** i link a `GUIDE*.md` di README (×4), ARCHITECTURE*.md, TESTING*.md e MARKETING.md passano a `https://nabre.github.io/atollo/guide/` (e `it/`, `fr/`, `de/`), così il controllo dei link relativi resta verde a ogni commit | prove verdi; controllo dei link; `text.test` con il rendering dell'app resta la rete | basso |
| 3 | **Pulire il contenuto**: icone come nomi, navigazione dichiarativa, `KitInfo`/`SourcesTable`/`FilterList`/`Terms`/`WindowSketch`/`Badge`/`Only`, nessun `className` nei capitoli, nessun import da `ui/src/api` (tipi strutturali) | `text.test`, `contract.test`; controllo a vista della Guida nell'app nelle 4 lingue | medio: regressioni di contenuto possibili |
| 4 | **Spostare in `guide/`** (`git mv`, la storia resta): `core`, `content`, `render/load.cjs`; aggiornare import, `@guide` in tsconfig e Vite, espressioni dei chunk, `@source` di `theme.css` se resta una classe, glob di ESLint, `guide/tsconfig.json`, script di lint e prettier, glob delle prove con il controllo «non vuoto», `app.yml` `guide/*` | prove verdi; controllo dei chunk; lint su `guide/` | medio: i guasti sono silenziosi (chunk, prove vuote, Tailwind), per questo ognuno ha un controllo esplicito |
| 5 | **Dividere `site/template.html`** in partial e `landing.html`; descrittori di pagina nel builder | uscita della pagina principale identica (confronto di `_site` prima e dopo) | basso-medio (tag SEO) |
| 6 | **Renderer del sito**: `site/guide.mjs`, `SITE_WIDGETS`, `guide.css`/`guide.js`, chiavi nuove nelle 4 `site/strings` (il builder impone chiavi uguali), SVG delle codicon, copia delle schermate, sitemap/hreflang/JSON-LD, link «Guida» | `site-render.test`, `build.mjs --check` | medio: peso delle pagine (immagini pigre) |
| 7 | **CI**: percorsi di `site.yml`, livello `site` in `app.yml` con `build --check` | una PR di prova che tocca `guide/` fa girare il livello giusto | basso |
| 8 | **Documenti tecnici**: ARCHITECTURE*.md (albero con `guide/`, sezione Guida, tabella degli script senza `guide-docs.cjs`), TESTING*.md (passo `--check` tolto, prove nuove, `build.mjs --check`) | controllo dei link relativi verde | basso |
| 9 | Facoltativo: schermate in `guide/shots` (§6.4) | | basso |

Rischi complessivi: (1) un registro a livello di modulo: chi dimentica di registrarsi riceve un errore chiaro, non una pagina vuota; (2) un processo disegna quattro lingue di fila, quindi nessun `tr()` memorizzato al caricamento; (3) il testo deve restare vero fuori dall'app (`Only`); (4) i ripieghi di `Go` non devono produrre link alla pagina corrente.

### 10.1 Testo proposto per D-155

```markdown
## D-155 Guida neutra, pubblicata sul sito; niente più GUIDE.md

- **Richiesta dell'utente** (22/10/2026): portare la Guida fuori dall'app e pubblicarla sul sito, una pagina per argomento e per lingua; togliere la versione Markdown.
- **Cosa.** Il contenuto sta in `guide/` alla radice (`core/` modello, lingua, etichette, ricerca e contratto dei widget; `content/` i capitoli), senza import da `ui/` né da `site/`. Due renderer: l'app (`ui/src/guide`, widget interattivi) e il sito (`site/guide.mjs`, HTML statico con `renderToStaticMarkup`), pagine `/guide/<argomento>/` in inglese e `/<lingua>/guide/<argomento>/` per it, fr, de, con ricerca lato client, hreflang e sitemap.
- **Niente Markdown.** Tolti `scripts/guide-docs.cjs`, `docs/app/GUIDE.md`, `GUIDE.it.md`, `test/ui/guide/docs.test.ts` e il passo `--check` della CI; i README rimandano a https://nabre.github.io/atollo/guide/. Una terza copia del testo costava manutenzione (riconoscimento dei widget per nome e per classe, etichette ripetute) e il sito la sostituisce.
- **Neutralità.** Il caricatore del sito (`guide/render/load.cjs`, `typescript.transpileModule`) rifiuta ogni import fuori da `guide/`; una prova vieta `className` e `style` nel contenuto; le implementazioni dell'app e del sito devono coprire tutti i widget del contratto (errore di compilazione e prova).
- **Prove.** `contract.test`, `site-render.test`, `lang.test`, `search.test`, `node site/build.mjs --check` nel job `ui`.
```
