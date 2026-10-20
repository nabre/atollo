"""VSCodium's "Dark Modern" and "Light Modern", applied to Atollo.

The theme files in vscodium-themes/ are copied unchanged from the VSCodium
server 1.140 (extensions/theme-defaults/themes, MIT license, Microsoft).
This script resolves their `include` chain, fills the colors the themes
leave to VS Code's built-in defaults, swaps the blue emphasis for the brand
colors (make.py, brand.png) and writes:

- tauri/editor-theme/: the editor's "Atollo Dark" and "Atollo Light",
  the two themes whole (colors, token colors) with the same emphasis, as
  the extension `atollo.theme` that the kit installs (editor/theme.rs);
- ui/src/theme/theme-vscode.css: the same colors as the `--vscode-*` variables
  that VS Code gives a webview, which @vscode-elements/elements reads;
- ui/src/theme/theme-colors.css: the app's short tokens (`--vsc-*`), each an
  alias of one of those variables, and their Tailwind utilities;
- ui/src/theme/theme-sizes.css: VSCodium's sizes (`--vscode-spacing-*`,
  `--vscode-cornerRadius-*`, shadows, `--pane-header-size`...), the app's
  measure tokens (`--vsc-size-*`) and their Tailwind utilities.

The theme files are newer than the editor Atollo ships, VSCodium 1.135:
RENAMED gives their keys the names it reads. The sizes come from
vscodium-1.135/workbench-ref.css, the part of that VSCodium's workbench CSS
the app mirrors, written by scripts/vscodium-css/extract.mjs.

The colors a theme leaves to VS Code's defaults come from vscode-defaults/,
the "Dark Modern" and "Light Modern" variables of a VS Code webview as
published in @vscode-elements/webview-playground 1.9.0 (dist/themes/
dark-v2.js and light-v2.js, MIT license, LICENSE there). To refresh them:

  npm pack @vscode-elements/webview-playground@1.9.0
  tar -xzf vscode-elements-webview-playground-1.9.0.tgz
  python docs/brand/themes.py --vendor package

Run `python docs/brand/themes.py` after changing anything here.
"""

import hashlib
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
THEMES = Path(__file__).with_name("vscodium-themes")
DEFAULTS = Path(__file__).with_name("vscode-defaults")
PLAYGROUND = "@vscode-elements/webview-playground 1.9.0"

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
        # list.inactiveSelectionForeground, else foreground: Dark Modern
        # sets neither, so the registry's foreground.
        "modernActivityBar.activeForeground": "#CCCCCC",
        "modernActivityBar.hoverForeground": "#CCCCCC",
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

# Keys of the theme files that VSCodium 1.135, the editor Atollo ships
# (tauri/catalog/components.json), reads under another name: the activity
# bar's items, and the floating parts' border, which its themes give as
# surface.border (kept where a theme has it).
RENAMED = {
    "modernActivityBarItem.activeBackground": "modernActivityBar.activeBackground",
    "modernActivityBarItem.activeForeground": "modernActivityBar.activeForeground",
    "modernActivityBarItem.hoverBackground": "modernActivityBar.hoverBackground",
    "modernActivityBarItem.hoverForeground": "modernActivityBar.hoverForeground",
    "modernActivityBar.border": "surface.border",
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
        "modernActivityBar.activeBackground": "#10B98133",
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
        # Darker, not lighter, than the band: white text keeps 4.5:1 on
        # hover and on the active item (a white veil drops it to 3.4:1).
        "statusBarItem.hoverBackground": "#04140D26",
        "statusBarItem.hoverForeground": "#FFFFFF",
        "statusBarItem.activeBackground": "#04140D40",
        "statusBarItem.prominentBackground": "#04140D33",
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
        "modernActivityBar.activeBackground": "#D1F0E0",
    },
}

# Colors VS Code computes from another key when the theme doesn't set them
# (the colorRegistry's defaults): with the brand emphasis they follow it,
# as in the editor. Each pair is checked against vscode-defaults/, where
# the key holds the source's value (with the alpha, if any) in Dark Modern
# and Light Modern.
DERIVED = {
    "sash.hoverBorder": "focusBorder",
    "settings.focusedRowBorder": "focusBorder",
    "notebook.focusedCellBorder": "focusBorder",
    "notebook.focusedEditorBorder": "focusBorder",
    "notebook.cellInsertionIndicator": "focusBorder",
    "inlineChatInput.focusBorder": "focusBorder",
    "editorCommentsWidget.unresolvedBorder": "focusBorder",
    "commentsView.unresolvedIcon": "focusBorder",
    "welcomePage.progress.foreground": "focusBorder",
    "list.focusHighlightForeground": "list.highlightForeground",
    "editorSuggestWidget.highlightForeground": "list.highlightForeground",
    "editorSuggestWidget.focusHighlightForeground": "list.focusHighlightForeground",
    "editorHoverWidget.highlightForeground": "list.highlightForeground",
    "editorSuggestWidget.selectedBackground": "list.activeSelectionBackground",
    "quickInputList.focusBackground": "list.activeSelectionBackground",
    "editorActionList.focusBackground": "list.activeSelectionBackground",
    "extensionButton.background": "button.background",
    "extensionButton.foreground": "button.foreground",
    "extensionButton.hoverBackground": "button.hoverBackground",
    "extensionButton.prominentBackground": "button.background",
    "extensionButton.prominentForeground": "button.foreground",
    "extensionButton.prominentHoverBackground": "button.hoverBackground",
    "inlineEdit.gutterIndicator.primaryBorder": "button.background",
    "inlineEdit.gutterIndicator.successfulBorder": "button.background",
    "inlineEdit.gutterIndicator.successfulBackground": "button.background",
    "inlineEdit.gutterIndicator.primaryForeground": "button.foreground",
    "inlineEdit.gutterIndicator.successfulForeground": "button.foreground",
    "extensionBadge.remoteBackground": "activityBarBadge.background",
    "extensionBadge.remoteForeground": "activityBarBadge.foreground",
    "panelTitleBadge.background": "activityBarBadge.background",
    "panelTitleBadge.foreground": "activityBarBadge.foreground",
    "radio.activeBackground": "inputOption.activeBackground",
    "radio.activeBorder": "inputOption.activeBorder",
    "radio.activeForeground": "inputOption.activeForeground",
    "notificationLink.foreground": "textLink.foreground",
    "minimapGutter.modifiedBackground": "editorGutter.modifiedBackground",
    "terminal.selectionBackground": "editor.selectionBackground",
    "minimap.selectionHighlight": "editor.selectionBackground",
    "settings.checkboxBackground": "checkbox.background",
    "settings.checkboxBorder": "checkbox.border",
    "settings.checkboxForeground": "checkbox.foreground",
    "settings.dropdownBackground": "dropdown.background",
    "settings.dropdownBorder": "dropdown.border",
    "settings.dropdownForeground": "dropdown.foreground",
    "settings.dropdownListBorder": "editorWidget.border",
    "settings.textInputBackground": "input.background",
    "settings.textInputBorder": "input.border",
    "settings.textInputForeground": "input.foreground",
    "settings.numberInputBackground": "input.background",
    "settings.numberInputBorder": "input.border",
    "settings.numberInputForeground": "input.foreground",
    "extensionIcon.verifiedForeground": "textLink.foreground",
    "chat.requestBubbleHoverBackground": "editor.selectionBackground",
    # transparent(source, alpha): the source's alpha times this one
    "editorOverviewRuler.modifiedForeground": ("editorGutter.modifiedBackground", 0.6),
    "editorCommentsWidget.rangeBackground": ("editorCommentsWidget.unresolvedBorder", 0.1),
    "editorCommentsWidget.rangeActiveBackground": (
        "editorCommentsWidget.unresolvedBorder",
        0.1,
    ),
    "commandCenter.debuggingBackground": ("statusBar.debuggingBackground", 0.258),
    "editor.foldBackground": ("editor.selectionBackground", 0.3),
    "chat.requestBubbleBackground": ("editor.selectionBackground", 0.3),
}

# The same, where the registry derives a key in one theme kind only (or
# with a different alpha per kind).
DERIVED_KIND = {
    "dark": {
        "banner.background": "list.activeSelectionBackground",
        "inlineEdit.gutterIndicator.primaryBackground": (
            "inlineEdit.gutterIndicator.primaryBorder",
            0.4,
        ),
    },
    "light": {
        "inlineEdit.gutterIndicator.primaryBackground": (
            "inlineEdit.gutterIndicator.primaryBorder",
            0.5,
        ),
    },
}

# Keys VS Code leaves without a value outside high contrast: a webview has
# no variable for them. Declared `initial` (the guaranteed-invalid value),
# a var() of them falls back exactly as there.
UNSET = [
    "contrastBorder",
    "list.hoverForeground",
    "menu.selectionBorder",
    "toolbar.hoverOutline",
]

# VS Code's fonts on Windows, the same in both themes; the defaults hold the
# sizes and weights too, and main() checks that they agree.
FONTS = {
    "--vscode-font-family": '"Segoe WPC", "Segoe UI", sans-serif',
    "--vscode-font-weight": "normal",
    "--vscode-font-size": "13px",
    "--vscode-editor-font-family": 'Consolas, "Courier New", monospace',
    "--vscode-editor-font-weight": "normal",
    "--vscode-editor-font-size": "14px",
    "--vscode-editor-font-feature-settings": '"liga" off, "calt" off',
}

# Atollo's typefaces over VS Code's (MARKETING.md, section 6): Geist and
# Geist Mono, self-hosted by the app (@fontsource-variable, ui/src/main.tsx)
# and by the editor (tauri/src/editor/window.rs), with the system's fonts
# behind them. Sizes and weights stay VS Code's.
FONT_FAMILIES = {
    "--vscode-font-family": '"Geist Variable", "Geist", system-ui, "Segoe UI", sans-serif',
    "--vscode-editor-font-family": '"Geist Mono Variable", "Geist Mono", ui-monospace, Consolas, monospace',
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
    # VSCodium's "modern UI": floating parts on the workbench color, and
    # the activity bar's item under the pointer and of the page shown
    "surface-border": "surface.border",
    "activitybar-item-active": "modernActivityBar.activeBackground",
    "activitybar-item-active-fg": "modernActivityBar.activeForeground",
    "activitybar-item-hover": "modernActivityBar.hoverBackground",
    "activitybar-item-hover-fg": "modernActivityBar.hoverForeground",
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
    # States. Error and warning are also text: the list's colors (as the
    # explorer's file names), 4.5:1 in both themes, where errorForeground
    # and editorWarning.foreground of Light Modern are near 3:1.
    "error": "list.errorForeground",
    "warning": "list.warningForeground",
    "info": "editorInfo.foreground",
    "success": "gitDecoration.untrackedResourceForeground",
    "orange": "charts.orange",
    # Scroll bars
    "scrollbar": "scrollbarSlider.background",
    "scrollbar-hover": "scrollbarSlider.hoverBackground",
    "scrollbar-active": "scrollbarSlider.activeBackground",
}

# VSCodium 1.135's workbench as vendored by scripts/vscodium-css/extract.mjs:
# the sizes its registry declares (the :root block) and the rules Atollo
# mirrors, one per line.
REFERENCE = Path(__file__).with_name("vscodium-1.135") / "workbench-ref.css"

# The modern UI's measures (workbench.experimental.modernUI: the floating
# parts Atollo imitates) under VSCodium's own variables. The activity bar's
# are set by workbench.js with the floating parts on (ActivitybarPart:
# baseWidth, actionHeight, actionGap and the icon size, 48px, 48px, 0 and
# 24px without them); the others come from the reference's rules
# (.modern-ui .monaco-pane-view, .modern-ui-tabs ... .title.tabs).
LAYOUT = {
    "--activity-bar-width": "36px",
    "--activity-bar-action-height": "36px",
    "--activity-bar-action-gap": "8px",
    "--activity-bar-icon-size": "24px",
    "--pane-header-size": "28px",
    "--editor-group-tab-height": "24px",
}

# App measure token -> value: the height or width VSCodium gives that part
# in the modern UI, as a VSCodium variable where it has one. The others are
# written in its CSS or code:
# - titlebar: TitlebarPart.minimumHeight with the command center (30px
#   without), the .window-appicon's width;
# - window-control: .window-controls-container>.window-icon;
# - title: .modern-ui.monaco-workbench .part>.title (35px in the classic UI),
#   the side bar's and the panel's title with its tabs;
# - row: the lists' and trees' row (sidebarSize22 in workbench.js);
# - tabs: the editor's tab bar, EDITOR_TAB_HEIGHT.modernUI (35px classic),
#   a tab of --editor-group-tab-height and 4px above and below;
# - statusbar: .monaco-workbench .part.statusbar.
SIZE_TOKENS = {
    "titlebar": "35px",
    "window-control": "46px",
    "activitybar": "var(--activity-bar-width)",
    "activitybar-action": "var(--activity-bar-action-height)",
    "activitybar-gap": "var(--activity-bar-action-gap)",
    "activitybar-icon": "var(--activity-bar-icon-size)",
    "title": "32px",
    "pane-header": "var(--pane-header-size)",
    "row": "22px",
    "tabs": "32px",
    "tab": "var(--editor-group-tab-height)",
    "statusbar": "22px",
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


def var_name(key):
    """VS Code's variable for a color key, as a webview gets it: the first
    dot becomes a dash, any other dot stays (`--vscode-welcomePage-progress.
    foreground`)."""
    return "--vscode-" + key.replace(".", "-", 1)


def css_name(name):
    return name.replace(".", "\\.")


def write(path, text):
    """UTF-8 with LF endings, as the repository keeps them."""
    path.write_text(text, encoding="utf-8", newline="\n")


def unescape(text):
    return re.sub(r"\\(.)", r"\1", text)


def vendor(package):
    """Copies the Dark Modern and Light Modern tables of an unpacked
    webview-playground into vscode-defaults/ (see the module notes)."""
    package = Path(package)
    meta = json.loads((package / "package.json").read_text(encoding="utf-8"))
    found = f"{meta['name']} {meta['version']}"
    if found != PLAYGROUND:
        raise SystemExit(f"expected {PLAYGROUND}, found {found}")
    DEFAULTS.mkdir(exist_ok=True)
    write(DEFAULTS / "LICENSE", (package / "LICENSE").read_text(encoding="utf-8"))
    for kind in ("dark", "light"):
        source = f"dist/themes/{kind}-v2.js"
        text = (package / source).read_text(encoding="utf-8")
        # [name, value] pairs of JavaScript strings (`\.`, `\"` escaped).
        pairs = re.findall(r'\["((?:[^"\\]|\\.)*)",\s*"((?:[^"\\]|\\.)*)"\]', text)
        data = {
            "_source": f"{PLAYGROUND}, {source} (MIT, LICENSE)",
            "variables": {unescape(n): unescape(v) for n, v in pairs},
        }
        write(DEFAULTS / f"{kind}.json", json.dumps(data, indent=2) + "\n")


def defaults(kind):
    data = json.loads((DEFAULTS / f"{kind}.json").read_text(encoding="utf-8"))
    return data["variables"]


def derived(kind):
    """DERIVED and DERIVED_KIND for a theme kind, as key -> (source, alpha)."""
    pairs = {**DERIVED, **DERIVED_KIND[kind]}
    return {k: v if isinstance(v, tuple) else (v, 1) for k, v in pairs.items()}


def rgba(value):
    """A `#rgb[a]`, `#rrggbb[aa]` or `rgb[a](...)` color as (r, g, b, a)."""
    m = re.fullmatch(r"rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)", value)
    if m:
        r, g, b, a = m.groups()
        return int(r), int(g), int(b), float(a or 1)
    digits = value.removeprefix("#")
    if len(digits) in (3, 4):
        digits = "".join(c * 2 for c in digits)
    a = int(digits[6:8], 16) / 255 if len(digits) == 8 else 1
    return *(int(digits[i : i + 2], 16) for i in (0, 2, 4)), a


def transparent(value, alpha):
    """VS Code's transparent(): the color with its alpha scaled."""
    if alpha == 1:
        return value
    r, g, b, a = rgba(value)
    return f"#{r:02x}{g:02x}{b:02x}{round(a * alpha * 255):02x}"


def same_color(a, b):
    """Equal up to the rounding of the alpha in vscode-defaults."""
    if a is None or b is None:
        return a == b
    (*x, xa), (*y, ya) = rgba(a), rgba(b)
    return x == y and abs(xa - ya) < 0.01


def check_derived():
    for kind in ("dark", "light"):
        base = defaults(kind)
        for key, (source, alpha) in derived(kind).items():
            value = base.get(var_name(source))
            if value is None or not same_color(
                base.get(var_name(key)), transparent(value, alpha)
            ):
                raise SystemExit(f"{kind}: {key} is not {source} in vscode-defaults")


def vscode_vars(kind):
    """Every color variable of a webview, by name: VS Code's defaults, the
    registry defaults in BUILTIN, the derived keys, and over them the
    editor theme whole (the `include` chain with BRAND)."""
    out = {n: v for n, v in defaults(kind).items() if n not in FONTS}
    leftover = [n for n in out if "-font-" in n]
    if leftover:
        raise SystemExit(f"{kind}: font variables missing from FONTS: {leftover}")
    for key, value in BUILTIN[kind].items():
        out[var_name(key)] = value
    theme = editor_theme(kind)["colors"]
    for key, (source, alpha) in derived(kind).items():
        if key not in theme:
            value = theme.get(source, out[var_name(source)])
            out[var_name(key)] = transparent(value, alpha)
    for key, value in theme.items():
        out[var_name(key)] = value
    for key in UNSET:
        out.setdefault(var_name(key), "initial")
    return {n: v.lower() if v.startswith("#") else v for n, v in out.items()}


def check_fonts():
    for kind in ("dark", "light"):
        for name, value in defaults(kind).items():
            if name in FONTS and FONTS[name] != value:
                raise SystemExit(f"{kind}: {name} is {value} in vscode-defaults")


def var_lines(values, indent):
    return "\n".join(f"{indent}{css_name(n)}: {v};" for n, v in values.items())


EXTENSION = ROOT / "tauri" / "editor-theme"
LABELS = {"dark": "Atollo Dark", "light": "Atollo Light"}


def editor_theme(kind):
    theme = chain(f"{kind}_modern.json")
    for old, new in RENAMED.items():
        if old in theme["colors"]:
            theme["colors"].setdefault(new, theme["colors"].pop(old))
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
    # the kit reinstalls it (editor/theme.rs).
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
        write(EXTENSION / "themes" / f"{kind}.json", text)
    write(EXTENSION / "package.json", json.dumps(package, indent=2) + "\n")


HEADER = """/* Generated by docs/brand/themes.py from VSCodium's Dark Modern and Light
   Modern with the Atollo emphasis: do not edit, run the script. */"""


def write_vscode_css():
    check_derived()
    check_fonts()
    dark, light = vscode_vars("dark"), vscode_vars("light")
    # Both themes declare the same names: a light value never falls through
    # to the dark one.
    names = sorted(dark.keys() | light.keys())
    dark = {n: dark.get(n, "initial") for n in names}
    light = {n: light.get(n, "initial") for n in names}
    css = f"""{HEADER}

/* The variables VS Code gives a webview, read by @vscode-elements/elements
   and by the app's tokens (theme-colors.css). Defaults from
   {PLAYGROUND} (MIT, docs/brand/vscode-defaults/LICENSE). */

:root {{
{var_lines(FONTS | FONT_FAMILIES, "  ")}
}}

:root,
:root[data-theme="dark"] {{
{var_lines(dark, "  ")}
}}

@media (prefers-color-scheme: light) {{
  :root:not([data-theme="dark"]) {{
{var_lines(light, "    ")}
  }}
}}
"""
    write(ROOT / "ui" / "src" / "theme" / "theme-vscode.css", css)
    return dark, light


def write_tokens_css(dark, light):
    for token, key in TOKENS.items():
        for kind, values in (("dark", dark), ("light", light)):
            if values.get(var_name(key), "initial") == "initial":
                raise SystemExit(f"{kind}: no color for {key}")
    tokens = "\n".join(
        f"  --vsc-{t}: var({css_name(var_name(k))});" for t, k in TOKENS.items()
    )
    css = f"""{HEADER}

/* The app's tokens, one per theme key: aliases of the variables in
   theme-vscode.css, so they follow its theme. */
:root {{
{tokens}
}}

/* Tailwind utilities: bg-vsc-bg, text-vsc-desc, border-vsc-border... */
@theme inline {{
{chr(10).join(f"  --color-vsc-{t}: var(--vsc-{t});" for t in TOKENS)}
}}
"""
    write(ROOT / "ui" / "src" / "theme" / "theme-colors.css", css)


def reference_sizes():
    """The registry's sizes and the modern UI's shadows, from REFERENCE:
    the :root block, then the shadows of .monaco-workbench with the ones
    .modern-ui.monaco-workbench turns off."""
    text = REFERENCE.read_text(encoding="utf-8")
    root = re.search(r"^:root \{\n(.*?)^\}", text, re.S | re.M)
    if not root:
        raise SystemExit(f"{REFERENCE.name}: no :root block")
    sizes = dict(re.findall(r"^  (--vscode-[\w-]+): ([^;]+);", root[1], re.M))
    for selector in (".monaco-workbench", ".modern-ui.monaco-workbench"):
        for line in re.findall(rf"^{re.escape(selector)}\{{([^}}]*)\}}$", text, re.M):
            for name, value in re.findall(r"(--vscode-shadow-[\w-]+):\s*([^;]+)", line):
                sizes[name] = value.strip()
    if not any(n.startswith("--vscode-shadow-") for n in sizes):
        raise SystemExit(f"{REFERENCE.name}: no shadows")
    # Prettier's numbers, as ui/ keeps its CSS: 0.08, not .08.
    return {n: re.sub(r"(?<![\d.])\.(\d)", r"0.\1", v) for n, v in sizes.items()}


def tailwind_name(name):
    """The Tailwind theme variable of a VSCodium size, or None: spacing
    (p-vsc-size40), radius (rounded-vsc-small), text (text-vsc-body-small,
    text-vsc-label1, text-vsc-codicon), font weight (font-vsc-semibold),
    shadow (shadow-vsc-md)."""
    rules = [
        (r"--vscode-spacing-(size\d+)", "--spacing-vsc-{}"),
        (r"--vscode-cornerRadius-(\w+)", "--radius-vsc-{}"),
        (r"--vscode-bodyFontSize(-\w+)?", "--text-vsc-body{}"),
        (r"--vscode-fontSize-(\w+)", "--text-vsc-{}"),
        (r"--vscode-codiconFontSize(-\w+)?", "--text-vsc-codicon{}"),
        (r"--vscode-fontWeight-(\w+)", "--font-weight-vsc-{}"),
        (r"--vscode-shadow-([\w-]+)", "--shadow-vsc-{}"),
    ]
    for pattern, out in rules:
        m = re.fullmatch(pattern, name)
        if m:
            return out.format((m[1] or "").lower())
    return None


def write_sizes_css():
    sizes = reference_sizes()
    root = var_lines(sizes | LAYOUT, "  ")
    tokens = "\n".join(f"  --vsc-size-{t}: {v};" for t, v in SIZE_TOKENS.items())
    utilities = [f"  {tailwind_name(n)}: var({n});" for n in sizes if tailwind_name(n)]
    utilities += [f"  --spacing-vsc-{t}: var(--vsc-size-{t});" for t in SIZE_TOKENS]
    css = f"""/* Generated by docs/brand/themes.py from VSCodium 1.135's workbench
   (docs/brand/vscodium-1.135/workbench-ref.css): do not edit, run the script. */

/* VSCodium's sizes, the same in every theme: its registry's (spacing,
   corner radius, font sizes), the modern UI's shadows and the workbench's
   own measures. */
:root {{
{root}
}}

/* The app's measure tokens: each part's height or width in VSCodium's
   modern UI. */
:root {{
{tokens}
}}

/* Tailwind utilities: h-vsc-row, w-vsc-activitybar, p-vsc-size40,
   rounded-vsc-small, shadow-vsc-xl, text-vsc-body-small... */
@theme inline {{
{chr(10).join(utilities)}
}}
"""
    write(ROOT / "ui" / "src" / "theme" / "theme-sizes.css", css)


def main():
    if sys.argv[1:2] == ["--vendor"]:
        vendor(sys.argv[2])
    write_sizes_css()
    write_tokens_css(*write_vscode_css())
    write_extension()
    print("ok")


if __name__ == "__main__":
    main()
