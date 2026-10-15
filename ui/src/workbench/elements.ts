// VSCodium's own controls, @vscode-elements/elements, registered once with
// their ES modules (never dist/bundled.js: the React wrappers of
// @vscode-elements/react-elements import the same modules, so each tag is
// defined once). They read the --vscode-* variables of theme-vscode.css.
import type { DetailedHTMLProps, HTMLAttributes } from "react";
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

declare module "react" {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace JSX {
    // eslint-disable-next-line @typescript-eslint/no-empty-object-type
    interface IntrinsicElements extends VscIntrinsicElements {}
  }
}
