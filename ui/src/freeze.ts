// Prototype pollution guard for the app window only. Tauri's
// `freezePrototype` would inject it into every webview, editor windows
// included, and VSCodium does not start with a frozen Object.prototype.
// Imported first by main.tsx, before any other module runs.
Object.freeze(Object.prototype);
