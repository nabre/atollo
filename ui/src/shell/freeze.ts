// Prototype pollution guard for the app window only. Tauri's
// `freezePrototype` would inject it into every webview, editor windows
// included, and VSCodium does not start with a frozen Object.prototype.
// Imported first by main.tsx, before any other module runs. Its methods
// stay overridable on other objects (freezeproto.ts): xterm.js assigns
// `toString` to its own.
import { freezeOverridable } from "./freezeproto";

freezeOverridable(Object.prototype);
