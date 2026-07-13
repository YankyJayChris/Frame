# IDE Support

Frame ships with a complete VS Code extension (`frame-syntax`) that provides syntax highlighting, code snippets, LSP integration, icon browsing, and project management tools.

## Installation

### From VS Code Marketplace

Search for "Frame Syntax" in the VS Code extensions panel and install.

### Manual Install

```bash
cd frame-syntax
npm install -g @vscode/vsce
vsce package
code --install-extension frame-syntax-*.vsix
```

## Features

### LSP Integration

The extension launches `frame lsp` as a child process and connects to it via stdio. All LSP features are available:

- Real-time diagnostics (red squiggles for errors, yellow for warnings)
- Context-aware completions (25+ contexts)
- Hover documentation
- Go-to-definition
- Find references
- Outline / document symbols
- Code actions (quick fixes)
- Formatting
- Rename symbols
- Semantic token highlighting
- Document colors (inline color picker)

### VS Code Commands

The extension registers 18 commands, all prefixed with `Frame:`:

| Command | Title | Description |
|---------|-------|-------------|
| `frame.build` | Frame: Build Project | Run `frame build` on the current project |
| `frame.buildWatch` | Frame: Build & Watch | Run `frame build --watch` |
| `frame.test` | Frame: Run Tests | Run `frame test` |
| `frame.testFilter` | Frame: Run Test (Filter) | Run `frame test --filter` with a prompt |
| `frame.deploy` | Frame: Deploy App | Run `frame deploy` with platform prompt |
| `frame.lint` | Frame: Lint Project | Run `frame lint` on the full project |
| `frame.lintFile` | Frame: Lint Current File | Run `frame lint` on the active file |
| `frame.pluginAdd` | Frame: Add Plugin | Prompt for plugin name and run `frame plugin add` |
| `frame.pluginList` | Frame: List Plugins | Run `frame plugin list` and show output |
| `frame.iconList` | Frame: List Icons | Run `frame icon list` and show output |
| `frame.iconGenerate` | Frame: Generate Icons | Run `frame icon generate` |
| `frame.newPage` | Frame: New Page | Scaffold a new `.fr` page file |
| `frame.newComponent` | Frame: New Component | Scaffold a new `.fr` component file |
| `frame.newStore` | Frame: New Store | Scaffold a new store definition |
| `frame.openDocs` | Frame: Open Documentation | Open the Frame documentation |
| `frame.restartLsp` | Frame: Restart Language Server | Restart the `frame lsp` process |
| `frame.showIcons` | Frame: Browse Icons | Open the icon browser view |
| `frameExplorer.refresh` | Frame: Refresh Explorer | Refresh the Frame Explorer tree view |

### Auto-import

When you type a PascalCase component name (a user-defined component) in a `children: [...]` block, the extension detects it's unimported and offers to add the `import` statement automatically. Supports `@/` path aliases:

```fr
// Typing MyComponent: { ... } in children
// Suggests: import { MyComponent } "@/components/MyComponent"
```

### Icon Browser

Browse all 330+ bundled icons by category, with SF Symbol and Material Icon mappings. Accessible via the `Frame: Browse Icons` command or the Frame Explorer view.

### Frame Explorer

The Frame Explorer is a tree view in the VS Code explorer panel that shows:

- **Pages** — all page declarations in the project
- **Components** — all user-defined components
- **Stores** — all `:store` declarations
- **Functions** — all top-level functions
- **Icons by Category** — all icons organized by category
- Quick action buttons for common commands

### Status Bar Indicators

- **Diagnostic count** — number of errors and warnings in the current file
- **Build status** — last build result (success/failure)
- **LSP status** — whether the LSP server is connected

### Decorations

- **Color swatches** — inline color swatch beside hex color values
- **Unit dimming** — dimmed display of unit suffixes on dimension properties

### Supplementary Completions

In addition to LSP-provided completions, the extension provides:

- **Icon names** — all registered icons from `frame-icon-lookup.json`
- **CSS color names** — common CSS color names
- **File paths** — relative and absolute file paths
- **Page routes** — all registered page route paths

### TextMate Grammar

The extension includes a TextMate grammar (`syntaxes/frame.tmLanguage.json`) that provides syntax highlighting for:

- 64+ built-in component names
- Language keywords (`page:`, `component`, `fn`, `import`, `:store`, etc.)
- Style properties (`width`, `height`, `padding`, `background`, etc.)
- Event handlers (`on_click`, `on_change`, etc.)
- Test matchers (`.toBe:`, `.toContain:`, etc.)

### Code Snippets

90+ code snippets covering every built-in component, declaration type, control flow pattern, and testing construct:

```
// Component snippets
row + Tab       → row: { children: [ ] }
column + Tab    → column: { children: [ ] }
text + Tab      → text: { content: "" }
button + Tab    → button: { content: ""  on_click: () }

// Declaration snippets
page + Tab      → page: { name: ""  route: ""  children: [ ] }
store + Tab     → :store Name { }
component + Tab → component Name: { props: {}  children: [] }
fn + Tab        → fn name: () => { }

// Control flow snippets
if + Tab        → if condition { } else { }
for + Tab       → for item in items { }
try + Tab       → try { } catch (err) { }
switch + Tab    → switch expr { case val => { } }

// Test snippets
describe + Tab  → describe: "" => { }
it + Tab        → it: "" => { }
```

## Configuration

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `frame.path` | String | `""` | Path to the Frame CLI binary. Leave empty to search PATH and common locations. |
| `frame.workspaceRoot` | String | `""` | Override workspace root path for Frame projects. Leave empty to use VS Code workspace root. |

## Troubleshooting

### LSP Server Not Starting

1. Ensure `frame` is installed and in your PATH: `frame --version`
2. Set `frame.path` in VS Code settings to the absolute path of the `frame` binary
3. Check the VS Code output panel for `Frame Language Server` logs
4. Run `frame lsp` manually to verify the CLI works

### Diagnostics Not Showing

1. Make sure your project has a `frame.config.json` file
2. Check that `frame.build` succeeds on your project
3. Restart the LSP server via `Frame: Restart Language Server` command

### Icon Browser Empty

1. Ensure you have at least one `.frameicons` file in `assets/icons/`
2. Run `frame icon list` to verify icons are registered
3. Run `Frame: Generate Icons` from the command palette
