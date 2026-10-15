"""VSCodium's "Dark Modern" and "Light Modern", applied to Atollo.

The theme files in vscodium-themes/ are copied unchanged from the VSCodium
server 1.140 (extensions/theme-defaults/themes, MIT license, Microsoft).
This script resolves their `include` chain, fills the colors the themes
leave to VS Code's built-in defaults, swaps the blue emphasis for the brand
colors (make.py, brand.png) and writes:

- ui/src/theme-colors.css: the app's color tokens, one per theme key;
- tauri/editor-theme/: the editor's "Atollo Dark" and "Atollo Light",
  the two themes whole (colors, token colors) with the same emphasis, as
  the extension `atollo.theme` that the kit installs (editor_theme.rs).

Run `python docs/brand/themes.py` after changing anything here.
"""

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
THEMES = Path(__file__).with_name("vscodium-themes")

# Colors the theme files don't set: VS Code's registered defaults (the
# workbench's colorRegistry), for the keys the app uses.
BUILTIN = {
    "dark": {
        "list.hoverBackground": "#2A2D2E",
        "list.activeSelectionBackground": "#04395E",
        "list.activeSelectionForeground": "#FFFFFF",
        "list.inactiveSelectionBackground": "#37373D",
        "list.focusOutline": "#0078D4",
        "list.focusAndSelectionOutline": "#0078D4",
        "list.highlightForeground": "#2AAAFF",
        "editorWarning.foreground": "#CCA700",
        "editorInfo.foreground": "#3794FF",
        "charts.orange": "#D18616",
        "gitDecoration.addedResourceForeground": "#81B88B",
        "gitDecoration.untrackedResourceForeground": "#73C991",
        "scrollbarSlider.background": "#79797966",
        "scrollbarSlider.hoverBackground": "#646464B3",
        "scrollbarSlider.activeBackground": "#BFBFBF66",
        "scrollbar.shadow": "#000000",
        "widget.shadow": "#0000005C",
        "editor.selectionBackground": "#264F78",
        "toolbar.hoverBackground": "#5A5D5E50",
        "checkbox.foreground": "#CCCCCC",
        "menu.selectionForeground": "#FFFFFF",
        "keybindingLabel.background": "#8080802B",
        "keybindingLabel.border": "#33333399",
        "keybindingLabel.bottomBorder": "#44444499",
        "statusBarItem.errorBackground": "#C72E0F",
        "statusBarItem.errorForeground": "#FFFFFF",
    },
    "light": {
        "list.activeSelectionBackground": "#0060C0",
        "list.inactiveSelectionBackground": "#E4E6F1",
        "list.focusOutline": "#005FB8",
        "list.highlightForeground": "#0066BF",
        "editorWarning.foreground": "#BF8803",
        "editorInfo.foreground": "#1A85FF",
        "charts.orange": "#D18616",
        "gitDecoration.addedResourceForeground": "#587C0C",
        "gitDecoration.untrackedResourceForeground": "#007100",
        "scrollbarSlider.background": "#64646466",
        "scrollbarSlider.hoverBackground": "#646464B3",
        "scrollbarSlider.activeBackground": "#00000099",
        "scrollbar.shadow": "#DDDDDD",
        "widget.shadow": "#00000029",
        "editor.selectionBackground": "#ADD6FF",
        "toolbar.hoverBackground": "#B8B8B850",
        "checkbox.foreground": "#3B3B3B",
        "menu.background": "#FFFFFF",
        "menu.foreground": "#3B3B3B",
        "menu.separatorBackground": "#D4D4D4",
        "keybindingLabel.background": "#DDDDDD66",
        "keybindingLabel.border": "#CCCCCC66",
        "keybindingLabel.bottomBorder": "#BBBBBB66",
        "statusBarItem.errorForeground": "#FFFFFF",
    },
}

# The brand emphasis: every blue of the themes, in emerald. On dark the
# emerald buttons carry near-black text (white on #10B981 is 2.5:1), on
# light the buttons are a deeper emerald with white text. Orange takes the
# place of charts.orange: what needs a choice. The status bar is the brand's
# band, as in themes like Meng Dark Pro: emerald, the remote item in Deep
# Forest, debugging in orange.
BRAND = {
    "dark": {
        "focusBorder": "#10B981",
        "activityBar.activeBorder": "#10B981",
        "activityBarBadge.background": "#10B981",
        "activityBarBadge.foreground": "#04140D",
        "button.background": "#10B981",
        "button.hoverBackground": "#0EA572",
        "button.foreground": "#04140D",
        "inputOption.activeBackground": "#10B98166",
        "inputOption.activeBorder": "#10B981",
        "menu.selectionBackground": "#10B981",
        "menu.selectionForeground": "#04140D",
        "list.activeSelectionBackground": "#0B3D2C",
        "list.focusOutline": "#10B981",
        "list.focusAndSelectionOutline": "#10B981",
        "list.highlightForeground": "#34D399",
        "panelTitle.activeBorder": "#10B981",
        "progressBar.background": "#10B981",
        "statusBar.background": "#10B981",
        "statusBar.foreground": "#04140D",
        "statusBar.border": "#10B981",
        "statusBar.noFolderBackground": "#10B981",
        "statusBar.noFolderForeground": "#04140D",
        "statusBar.debuggingBackground": "#F97316",
        "statusBar.debuggingForeground": "#04140D",
        "statusBar.debuggingBorder": "#F97316",
        "statusBar.focusBorder": "#04140D",
        "statusBarItem.focusBorder": "#04140D",
        "statusBarItem.hoverBackground": "#04140D26",
        "statusBarItem.hoverForeground": "#04140D",
        "statusBarItem.activeBackground": "#04140D40",
        "statusBarItem.prominentBackground": "#04140D33",
        "statusBarItem.prominentForeground": "#04140D",
        "statusBarItem.remoteBackground": "#0F3D2E",
        "statusBarItem.remoteForeground": "#D1F0E0",
        "statusBarItem.remoteHoverBackground": "#14503C",
        "statusBarItem.remoteHoverForeground": "#D1F0E0",
        "tab.activeBorderTop": "#10B981",
        "tab.selectedBorderTop": "#6EE7B7",
        "terminal.tab.activeBorder": "#10B981",
        "textLink.foreground": "#34D399",
        "textLink.activeForeground": "#34D399",
        "welcomePage.progress.foreground": "#10B981",
        "editorGutter.modifiedBackground": "#10B981",
        "editor.selectionBackground": "#10B9814D",
        "charts.orange": "#F97316",
        "modernActivityBarItem.activeBackground": "#10B98133",
    },
    "light": {
        "focusBorder": "#047857",
        "activityBar.activeBorder": "#047857",
        "activityBarBadge.background": "#047857",
        "activityBarBadge.foreground": "#FFFFFF",
        "button.background": "#047857",
        "button.hoverBackground": "#065F46",
        "button.foreground": "#FFFFFF",
        "inputOption.activeBackground": "#D1F0E0",
        "inputOption.activeBorder": "#047857",
        "menu.selectionBackground": "#047857",
        "menu.selectionForeground": "#FFFFFF",
        "list.focusOutline": "#047857",
        "list.focusAndSelectionOutline": "#047857",
        "list.highlightForeground": "#047857",
        "panelTitle.activeBorder": "#047857",
        "progressBar.background": "#047857",
        "statusBar.background": "#047857",
        "statusBar.foreground": "#FFFFFF",
        "statusBar.border": "#047857",
        "statusBar.noFolderBackground": "#047857",
        "statusBar.noFolderForeground": "#FFFFFF",
        "statusBar.debuggingBackground": "#C2410C",
        "statusBar.debuggingForeground": "#FFFFFF",
        "statusBar.debuggingBorder": "#C2410C",
        "statusBar.focusBorder": "#FFFFFF",
        "statusBarItem.focusBorder": "#FFFFFF",
        "statusBarItem.hoverBackground": "#FFFFFF26",
        "statusBarItem.hoverForeground": "#FFFFFF",
        "statusBarItem.activeBackground": "#FFFFFF40",
        "statusBarItem.prominentBackground": "#FFFFFF33",
        "statusBarItem.prominentForeground": "#FFFFFF",
        "statusBarItem.remoteBackground": "#0F3D2E",
        "statusBarItem.remoteForeground": "#FFFFFF",
        "statusBarItem.remoteHoverBackground": "#14503C",
        "statusBarItem.remoteHoverForeground": "#FFFFFF",
        "tab.activeBorderTop": "#047857",
        "tab.selectedBorderTop": "#34D399",
        "terminal.tab.activeBorder": "#047857",
        "terminalCursor.foreground": "#047857",
        "textLink.foreground": "#047857",
        "textLink.activeForeground": "#047857",
        "editorGutter.modifiedBackground": "#047857",
        "editor.selectionBackground": "#A7E3C6",
        "charts.orange": "#C2410C",
        "modernActivityBarItem.activeBackground": "#D1F0E0",
    },
}

# App token -> theme key. Names kept short; each token is the color VSCodium
# gives that element, so a component styled as VSCodium's picks the key.
TOKENS = {
    # Work area and chrome
    "bg": "editor.background",
    "fg": "foreground",
    "desc": "descriptionForeground",
    "icon": "icon.foreground",
    "border": "sideBar.border",
    "titlebar": "titleBar.activeBackground",
    "titlebar-fg": "titleBar.activeForeground",
    "titlebar-border": "titleBar.border",
    "activitybar": "activityBar.background",
    "activitybar-fg": "activityBar.foreground",
    "activitybar-inactive": "activityBar.inactiveForeground",
    "activitybar-border": "activityBar.activeBorder",
    "activitybar-edge": "activityBar.border",
    "activitybar-badge": "activityBarBadge.background",
    "activitybar-badge-fg": "activityBarBadge.foreground",
    "sidebar": "sideBar.background",
    "sidebar-fg": "sideBar.foreground",
    "sidebar-title": "sideBarTitle.foreground",
    "section": "sideBarSectionHeader.background",
    "section-fg": "sideBarSectionHeader.foreground",
    "section-border": "sideBarSectionHeader.border",
    "panel": "panel.background",
    "panel-border": "panel.border",
    "panel-title": "panelTitle.activeForeground",
    "panel-title-inactive": "panelTitle.inactiveForeground",
    "panel-title-border": "panelTitle.activeBorder",
    "statusbar": "statusBar.background",
    "statusbar-fg": "statusBar.foreground",
    "statusbar-border": "statusBar.border",
    "statusbar-hover": "statusBarItem.hoverBackground",
    "statusbar-hover-fg": "statusBarItem.hoverForeground",
    "statusbar-remote": "statusBarItem.remoteBackground",
    "statusbar-remote-fg": "statusBarItem.remoteForeground",
    "statusbar-remote-hover": "statusBarItem.remoteHoverBackground",
    "statusbar-active": "statusBarItem.activeBackground",
    "statusbar-error": "statusBarItem.errorBackground",
    "statusbar-error-fg": "statusBarItem.errorForeground",
    "tab-border-top": "tab.activeBorderTop",
    "tab-active": "tab.activeBackground",
    "tab-active-fg": "tab.activeForeground",
    "tab-inactive": "tab.inactiveBackground",
    "tab-inactive-fg": "tab.inactiveForeground",
    "tab-border": "tab.border",
    "tabs": "editorGroupHeader.tabsBackground",
    "tabs-border": "editorGroupHeader.tabsBorder",
    "settings-header": "settings.headerForeground",
    "settings-modified": "settings.modifiedItemIndicator",
    # VSCodium 1.140's "modern UI": floating parts on the workbench color
    "surface-border": "modernActivityBar.border",
    "activitybar-item-active": "modernActivityBarItem.activeBackground",
    "activitybar-item-hover": "modernActivityBarItem.hoverBackground",
    # Controls
    "focus": "focusBorder",
    "button": "button.background",
    "button-hover": "button.hoverBackground",
    "button-fg": "button.foreground",
    "button-border": "button.border",
    "button2": "button.secondaryBackground",
    "button2-hover": "button.secondaryHoverBackground",
    "button2-fg": "button.secondaryForeground",
    "input": "input.background",
    "input-fg": "input.foreground",
    "input-border": "input.border",
    "placeholder": "input.placeholderForeground",
    "option-active": "inputOption.activeBackground",
    "option-active-border": "inputOption.activeBorder",
    "dropdown": "dropdown.background",
    "dropdown-list": "dropdown.listBackground",
    "dropdown-border": "dropdown.border",
    "checkbox": "checkbox.background",
    "checkbox-border": "checkbox.border",
    "checkbox-fg": "checkbox.foreground",
    "badge": "badge.background",
    "badge-fg": "badge.foreground",
    "progress": "progressBar.background",
    "toolbar-hover": "toolbar.hoverBackground",
    "keybinding": "keybindingLabel.background",
    "keybinding-fg": "keybindingLabel.foreground",
    "keybinding-border": "keybindingLabel.border",
    "keybinding-bottom": "keybindingLabel.bottomBorder",
    # Lists
    "list-hover": "list.hoverBackground",
    "list-active": "list.activeSelectionBackground",
    "list-active-fg": "list.activeSelectionForeground",
    "list-inactive": "list.inactiveSelectionBackground",
    "list-focus": "list.focusOutline",
    "list-highlight": "list.highlightForeground",
    # Floating surfaces
    "widget": "editorWidget.background",
    "widget-border": "widget.border",
    "shadow": "widget.shadow",
    "menu": "menu.background",
    "menu-fg": "menu.foreground",
    "menu-border": "menu.border",
    "menu-selection": "menu.selectionBackground",
    "menu-selection-fg": "menu.selectionForeground",
    "menu-separator": "menu.separatorBackground",
    "notification": "notifications.background",
    "notification-fg": "notifications.foreground",
    "notification-border": "notifications.border",
    "quickinput": "quickInput.background",
    # Text
    "link": "textLink.foreground",
    "link-active": "textLink.activeForeground",
    "code-block": "textCodeBlock.background",
    "preformat": "textPreformat.background",
    "preformat-fg": "textPreformat.foreground",
    "quote": "textBlockQuote.background",
    "quote-border": "textBlockQuote.border",
    "selection": "editor.selectionBackground",
    # States
    "error": "errorForeground",
    "warning": "editorWarning.foreground",
    "info": "editorInfo.foreground",
    "success": "gitDecoration.untrackedResourceForeground",
    "orange": "charts.orange",
    # Scroll bars
    "scrollbar": "scrollbarSlider.background",
    "scrollbar-hover": "scrollbarSlider.hoverBackground",
    "scrollbar-active": "scrollbarSlider.activeBackground",
}


def chain(name):
    """The theme with its `include`s resolved, as VS Code merges them: the
    included theme first, colors and semantic colors overridden key by key,
    token colors appended."""
    theme = json.loads((THEMES / name).read_text(encoding="utf-8"))
    out = {"colors": {}, "tokenColors": [], "semanticTokenColors": {}}
    if "include" in theme:
        out = chain(theme["include"].removeprefix("./"))
    out["colors"].update(theme.get("colors", {}))
    out["tokenColors"] += theme.get("tokenColors", [])
    out["semanticTokenColors"].update(theme.get("semanticTokenColors", {}))
    if "semanticHighlighting" in theme:
        out["semanticHighlighting"] = theme["semanticHighlighting"]
    return out


def load(name):
    return chain(name)["colors"]


def resolve(kind):
    colors = dict(BUILTIN[kind])
    colors.update(load(f"{kind}_modern.json"))
    colors.update(BRAND[kind])
    return colors


def css_block(kind, colors):
    lines = []
    for token, key in TOKENS.items():
        value = colors.get(key)
        if value is None:
            raise SystemExit(f"{kind}: no color for {key}")
        lines.append(f"  --vsc-{token}: {value.lower()}; /* {key} */")
    return "\n".join(lines)


EXTENSION = ROOT / "tauri" / "editor-theme"
LABELS = {"dark": "Atollo Dark", "light": "Atollo Light"}


def editor_theme(kind):
    theme = chain(f"{kind}_modern.json")
    theme["colors"].update(BRAND[kind])
    return {
        "$schema": "vscode://schemas/color-theme",
        "name": LABELS[kind],
        "type": kind,
        "semanticHighlighting": theme.get("semanticHighlighting", True),
        "colors": dict(sorted(theme["colors"].items())),
        "tokenColors": theme["tokenColors"],
        "semanticTokenColors": theme["semanticTokenColors"],
    }


def write_extension():
    themes = {
        kind: json.dumps(editor_theme(kind), indent=2) + "\n"
        for kind in ("dark", "light")
    }
    # The version follows the content: a new theme is a new version, and
    # the kit reinstalls it (editor_theme.rs).
    digest = hashlib.sha256("".join(themes.values()).encode()).hexdigest()
    package = {
        "name": "theme",
        "displayName": "Atollo Theme",
        "description": "VSCodium's Dark Modern and Light Modern with the Atollo colors.",
        "publisher": "atollo",
        "version": f"1.0.{int(digest[:8], 16)}",
        "license": "MIT",
        "engines": {"vscode": "^1.80.0"},
        "categories": ["Themes"],
        "contributes": {
            "themes": [
                {
                    "id": LABELS[kind],
                    "label": LABELS[kind],
                    "uiTheme": "vs-dark" if kind == "dark" else "vs",
                    "path": f"./themes/{kind}.json",
                }
                for kind in ("dark", "light")
            ]
        },
    }
    (EXTENSION / "themes").mkdir(parents=True, exist_ok=True)
    for kind, text in themes.items():
        (EXTENSION / "themes" / f"{kind}.json").write_text(text, encoding="utf-8")
    (EXTENSION / "package.json").write_text(
        json.dumps(package, indent=2) + "\n", encoding="utf-8"
    )


def main():
    dark, light = resolve("dark"), resolve("light")
    css = f"""/* Generated by docs/brand/themes.py from VSCodium's Dark Modern and Light
   Modern with the Atollo emphasis: do not edit, run the script. */

:root,
:root[data-theme="dark"] {{
{css_block("dark", dark)}
}}

@media (prefers-color-scheme: light) {{
  :root:not([data-theme="dark"]) {{
{css_block("light", light).replace(chr(10), chr(10) + "  ").replace("  --", "    --", 1)}
  }}
}}

/* Tailwind utilities: bg-vsc-bg, text-vsc-desc, border-vsc-border... */
@theme inline {{
{chr(10).join(f"  --color-vsc-{t}: var(--vsc-{t});" for t in TOKENS)}
}}
"""
    (ROOT / "ui" / "src" / "theme-colors.css").write_text(css, encoding="utf-8")
    write_extension()
    print("ok")


if __name__ == "__main__":
    main()
