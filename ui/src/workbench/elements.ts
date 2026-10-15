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
if (!document.getElementById("vscode-codicon-stylesheet")) {
  const link = document.createElement("link");
  link.id = "vscode-codicon-stylesheet";
  link.rel = "stylesheet";
  link.href = codiconHref;
  document.head.append(link);
}

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
