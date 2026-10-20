// VSCodium's own controls, @vscode-elements/elements, registered once with
// their ES modules (never dist/bundled.js: the React wrappers of
// @vscode-elements/react-elements import the same modules, so each tag is
// defined once). They read the --vscode-* variables of theme-vscode.css.
import { useCallback, useRef, type DetailedHTMLProps, type HTMLAttributes } from "react";
import "@vscode-elements/elements";
import codiconHref from "@vscode/codicons/dist/codicon.css?url";

// vscode-icon looks this link up by id and copies its href (and nonce) into
// each icon's shadow root, where the Codicon font must be loaded again. The
// file is served by the app itself, as the CSP wants (style-src 'self').
// Disabled in the page, which has the same rules in its base layer
// (theme.css): unlayered, its 16px would win over every size utility.
if (!document.getElementById("vscode-codicon-stylesheet")) {
  const link = document.createElement("link");
  link.id = "vscode-codicon-stylesheet";
  link.rel = "stylesheet";
  link.disabled = true;
  link.href = codiconHref;
  document.head.append(link);
}

/** A component class as Lit keeps its styles: the shadow root of each new
 * element takes `elementStyles`, which `finalizeStyles` fills from a sheet
 * (as itself where the browser adopts sheets, as text where it doesn't). */
interface Styled {
  elementStyles: unknown[];
  finalizeStyles(styles: CSSStyleSheet): unknown[];
}

/** Adds rules to a component's own, inside its shadow root, where no
 * ::part reaches: they come after its own, for every element drawn from
 * now on. */
function restyle(tag: VscTag, rules: string) {
  const ctor = customElements.get(tag) as unknown as Styled | undefined;
  if (!ctor) return;
  const sheet = new CSSStyleSheet();
  sheet.replaceSync(rules);
  ctor.elementStyles.push(...ctor.finalizeStyles(sheet));
}

// VSCodium 1.135's input box and select box (.monaco-inputbox: 4px 6px
// around the text, 26px high with its border; .monaco-select-box in the
// Settings editor, 26px; its dropdown, the large corners and the lg
// shadow). The radius of both stays the components' 4px, cornerRadius-small.
const FIELD = `padding: var(--vscode-spacing-size40) var(--vscode-spacing-size60);
  line-height: var(--vscode-spacing-size160);`;
restyle("vscode-textfield", `input:not([type="file"]) { ${FIELD} }`);
restyle("vscode-textarea", `textarea { padding: var(--vscode-spacing-size40) var(--vscode-spacing-size60); }`);
restyle(
  "vscode-single-select",
  `.select-face { ${FIELD} }
  .select-face .text { height: var(--vscode-spacing-size160); }
  .dropdown { overflow: hidden; border-radius: var(--vscode-cornerRadius-large); box-shadow: var(--vscode-shadow-lg); }`,
);

// JSX for the raw <vscode-*> tags (as vscode-elements/examples, react-vite):
// their properties, plus `class` and `for`, which a custom element takes
// instead of className and htmlFor. Fields and selects dispatch a "change"
// that doesn't bubble, so React's onChange never sees it on a raw tag: use
// the wrappers of @vscode-elements/react-elements for them.
type VscTag = Extract<keyof HTMLElementTagNameMap, `vscode-${string}`>;
type VscProps<E extends HTMLElement> = DetailedHTMLProps<HTMLAttributes<E>, E> &
  Partial<Omit<E, keyof HTMLElement>> & { class?: string; for?: string };
type VscIntrinsicElements = { [T in VscTag]: VscProps<HTMLElementTagNameMap[T]> };

/** A ref for a component that keeps its role (and tab stop) as a property:
 * Lit writes the attribute on its first update, a microtask after React's
 * commit. Written here at once, it is there for what runs in the same
 * commit (a focus() in an effect, a query in a test); the component then
 * writes the same value, or its own (tabindex -1 when disabled). Gives the
 * callback for `ref` and the element. */
export function useHost<E extends HTMLElement>(role: string, tabStop = false) {
  const element = useRef<E | null>(null);
  const ref = useCallback(
    (el: E | null) => {
      if (el) {
        if (!el.hasAttribute("role")) el.setAttribute("role", role);
        if (tabStop && !el.hasAttribute("tabindex")) el.setAttribute("tabindex", "0");
      }
      element.current = el;
    },
    [role, tabStop],
  );
  return [ref, element] as const;
}

declare module "react" {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace JSX {
    // eslint-disable-next-line @typescript-eslint/no-empty-object-type
    interface IntrinsicElements extends VscIntrinsicElements {}
  }
}
