# Frame Syntax — VS Code Extension

A full-featured VS Code extension for the **Frame** language (`.fr` files). Provides LSP-powered editing, syntax highlighting, code snippets, and project management commands — everything you need to build Frame apps in VS Code.

## Features

### LSP-Powered Editing

The extension launches the built-in `frame lsp` server automatically when you open a `.fr` file. All features work out of the box with zero configuration:

| Feature | Details |
|---------|---------|
| **Diagnostics** | Real-time errors (red squiggles) from the parser, type checker, and linter. Warnings for style/correctness issues (FR001–FR050) |
| **Completions** | 25+ context-aware completion contexts — component names in `children: [...]`, props after component kinds, style properties, store fields, events, icon names, page routes, file paths, design tokens, and more |
| **Hover Info** | Documentation for built-in components (props/events/styles/category), function signatures, store fields, action signatures, style property types |
| **Go-to Definition** | Jump to component declarations, store definitions, function definitions, page definitions, object/enum types, and import sources |
| **Find References** | Find all usages of any symbol across the project |
| **Document Symbols** | Outline view showing pages, components, stores, objects, enums, and functions with their children |
| **Workspace Symbols** | Search any symbol across the entire workspace |
| **Code Actions** | Quick-fix lightbulb for missing required props and naming convention violations |
| **Formatting** | Auto-indent with tabs, consistent spacing, trailing newline |
| **Folding Ranges** | Collapse `{}` blocks, `children: [...]` arrays, and `/* */` comments |
| **Semantic Tokens** | AST-driven coloring for keywords, types, strings, numbers, comments |
| **Rename Symbol** | Rename components, stores, functions, and variables across the file |
| **Document Highlights** | Highlight all occurrences of the symbol at the cursor |
| **Document Colors** | Inline color picker for hex values (`#FF0000`, `#FF0`) and `$var` references |

### Commands (22 total)

| Command | Description |
|---------|-------------|
| `frame.build` | Run `frame build` in the integrated terminal |
| `frame.buildWatch` | Run `frame build --watch` for continuous rebuild |
| `frame.test` | Run all `.test.fr` test suites |
| `frame.testFilter` | Pick a test by name and run with filter |
| `frame.deploy` | Pick platform (iOS / Android) and deploy |
| `frame.lint` | Run `frame lint` on the project |
| `frame.lintFile` | Lint only the current file |
| `frame.pluginAdd` | Install a Frame plugin |
| `frame.pluginList` | Quick pick of installed plugins |
| `frame.iconList` | Browse all registered icons by category |
| `frame.iconGenerate` | Generate icon assets for iOS, Android, or both |
| `frame.openAppIconPicker` | Configure app icon (default or custom) |
| `frame.validateIcons` | Validate app icon configuration |
| `frame.previewAppIcon` | Preview and view app icon details |
| `frame.openAppIconDocs` | Open app icon documentation |
| `frame.newPage` | Insert a new page snippet |
| `frame.newComponent` | Insert a new component snippet |
| `frame.newStore` | Insert a new store snippet |
| `frame.openDocs` | Open the Frame documentation |
| `frame.showIcons` | Open the icon browser |
| `frame.restartLsp` | Restart the LSP server |
| `frame.formatDocument` | Format the current document |

### Status Bar

- **Frame logo** (left side) — shows the installed `frame` CLI version
- **Diagnostic count** (right side) — click to open the Problems panel
- **Build status** — spinner during build, checkmark on success, cross on failure
- **LSP status** — connected `/` reconnecting `/` disconnected indicator

### Decorations

- **Color swatches** — small colored circles beside hex color values (`$primary: "#FF0000"`)
- **Unit dimming** — dims unit suffixes like `dp`, `sp`, `px`, `%`, `ms` so you can focus on the numeric value

### Auto-Import

The import manager automatically detects PascalCase identifiers in `children: [...]` blocks that aren't yet imported. It scans your workspace for `.fr` files with matching `component Name:` declarations and offers a quick-fix lightbulb to insert the `import` statement. If the import path would be deep relative, it suggests using the `@/` alias instead.

### Supplementary Completions

These inline providers handle completions the LSP can't easily provide:

- **Icon names** — when typing `icon: { name: "`, suggests names from all `.frameicons` bundles
- **Color values** — when typing `color: "` or `background: "`, suggests `:vars` tokens and CSS named colors
- **File paths** — when typing `import { x } "`, suggests `.fr` files in the project
- **Page routes** — when typing `navigate("`, suggests known page routes from the AST

### Icon Browser

The `frame.showIcons` command opens a quick pick listing all 330+ bundled icons grouped by category (Actions, UI, Navigation, Media, Communication, Social, Devices, Status, Commerce, Files, Security, Weather, Health, Food). Each entry shows the SF Symbol and Material Icon mappings. Select an icon to insert its name at the cursor.

### Frame Explorer

The side panel tree view shows:

1. **Project** — pages (with routes), components (with prop count), stores (with field count), functions
2. **Icons** — category tree with all registered icon names
3. **Plugins** — installed plugins from `frame.config.json`
4. **Quick Actions** — build, test, deploy, preview buttons

## Installation

### From VS Code Marketplace

Search for "Frame Syntax" in the VS Code extensions panel.

### From VSIX

```bash
cd frame-syntax
npm install -g @vscode/vsce
vsce package
# Install the generated .vsix in VS Code:
# Extensions → ... → Install from VSIX...
```

### Manual

Copy the `frame-syntax` folder to `~/.vscode/extensions/` and restart VS Code.

## Requirements

- **Frame CLI** (`frame`) must be installed and on your PATH. The extension runs `frame lsp` as the language server.
- **VS Code** 1.98.0 or later.

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `frame.path` | (auto-detect) | Custom path to the `frame` CLI binary |
| `frame.workspaceRoot` | (auto-detect) | Explicit project root path |

## Syntax Highlighting

The TextMate grammar covers:

- **71+ built-in components** — text, button, image, icon, column, row, container, stack, scaffold, card, divider, spacer, modal, scroll_view, grid, list, app_bar, bottom_navigation_bar, sidebar, tab_bar, tab, bottom_sheet, floating_action_button, input, text_area, dropdown, switch, checkbox, radio, slider, stepper, search_bar, date_picker, time_picker, color_picker, rating, otp_input, form, progress_bar, progress_circle, toast, tooltip, badge, chip, tag, avatar, banner, skeleton, table, accordion, timeline, video_player, audio_player, lottie, web_view, map_view, camera_view, qr_scanner, swipeable, draggable, refresh, long_press, plugin, navigation_rail, drawer, popup_menu, menu, selectable_text, wrap, flow
- **Keywords** — all control flow, declarations, lifecycle, navigation, persistence, testing keywords
- **Style properties** — 60+ layout, typography, visual, animation, scroll, border properties
- **Events** — 25+ event handler names
- **Test matchers** — `.toBe:`, `.toEqual:`, `.toContain:`, `.toBeNull:`, `.toBeTrue:`, `.toBeFalse:`, `.toThrow:`
- **Interpolation** — `\(expr)` and `${expr}` template syntax
- **Types** — `int`, `float`, `bool`, `string`, `object`, `list`, `null`, `void`

## Snippets (90+)

The extension includes code snippets for everything in Frame:

| Category | Snippets |
|----------|----------|
| **Layout** | scaffold, column, row, container, stack, card, divider, spacer, scroll_view, grid, list, accordion, timeline, sidebar, tab_bar, bottom_nav_bar |
| **Text/Content** | text, button, image, icon, avatar, badge, chip, tag, banner, skeleton, table |
| **Input** | input, text_area, dropdown, switch, checkbox, radio, slider, stepper, search_bar, date_picker, time_picker, color_picker, rating, otp_input, form |
| **Navigation** | app_bar, bottom_navigation_bar, fab, modal, bottom_sheet, tab, tab_bar |
| **Feedback** | toast, tooltip, progress_bar, progress_circle |
| **Media** | video_player, audio_player, lottie, web_view, map_view, camera_view, qr_scanner |
| **Gesture** | swipeable, draggable, refresh, long_press |
| **Declarations** | page, component, store, object, enum, function, type, validation, app |
| **Control Flow** | if, for, switch, try, fetch, navigate, import |
| **State** | state, persist, watch, data, build, show_if |
| **Animation** | animate, positioned |
| **Testing** | describe, it, expect, mock |
| **Design System** | :vars, :breakpoints, :typography, :i18n, :var |

## Development

```bash
# Clone the repo
git clone https://github.com/frame-lang/frame.git
cd frame/frame-syntax

# Install dependencies
npm install

# Compile TypeScript
npx tsc -p tsconfig.json

# Package the extension
npm install -g @vscode/vsce
vsce package

# Launch in development mode
# Open this folder in VS Code and press F5
```

## Troubleshooting

- **"Frame CLI not found"** — ensure `frame` is installed and on your PATH. Run `cargo install frame` or build from source.
- **LSP not starting** — check the VS Code Output panel (select "Frame Language Server" from the dropdown). Ensure `frame lsp --help` works in your terminal.
- **Auto-import not suggesting** — the import manager caches component names from workspace `.fr` files. Try reloading the window.
- **Color swatches not showing** — decorations only activate for hex colors (`#FF0000`, `#FF0`) and `$variable` references in style blocks.
