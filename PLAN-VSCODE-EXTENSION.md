# Frame VS Code Extension — Production Plan

## Architecture Decisions (Confirmed)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| LSP binary | `frame lsp` subcommand | Built into existing `frame` CLI — zero extra install steps for users |
| LSP language | Rust (`tower-lsp`) | Reuses existing PEG grammar (1084 lines), AST (721 lines), type checker (1063 lines), linter (15 rules) directly |
| LSP transport | stdio | Simplest, works cross-platform, VS Code's built-in support |
| Project root detection | Walk up from open file to nearest `frame.config.json` | Standard pattern, works even when VS Code workspace root isn't the project root |
| @/ alias config | `frame.config.json` → `"paths": { "@": "./src" }` | Consistent with existing config pattern |
| Parsing strategy | Full re-parse on save | Project files are small, PEG parser is fast (<50ms), incremental parsing unnecessary |
| TextMate + Semantic Tokens | Both | TextMate for instant highlight; semantic tokens for precise AST-driven coloring |

---

## File Manifest (What Gets Created/Modified)

### New Files

```
frame-lsp/Cargo.toml                        # LSP server crate manifest
frame-lsp/src/main.rs                       # LSP entrypoint: frame lsp subcommand
frame-lsp/src/server.rs                     # FrameLanguageServer (tower-lsp)
frame-lsp/src/capabilities.rs               # Server capabilities registration
frame-lsp/src/completion.rs                 # TextDocument completion handler
frame-lsp/src/hover.rs                      # Hover documentation handler
frame-lsp/src/definition.rs                 # Go-to-definition handler
frame-lsp/src/references.rs                 # Find-all-references handler
frame-lsp/src/diagnostics.rs               # Publish diagnostics from parser/linter/typechecker
frame-lsp/src/symbols.rs                    # Document/workspace symbols
frame-lsp/src/code_action.rs                # Code actions / quick fixes
frame-lsp/src/formatting.rs                 # Document formatting
frame-lsp/src/folding.rs                    # Folding ranges
frame-lsp/src/semantic_tokens.rs            # Semantic token provider
frame-lsp/src/rename.rs                     # Rename symbol
frame-lsp/src/highlights.rs                 # Document highlight
frame-lsp/src/completion_data.rs            # Static completion data (component names, props, etc.)

frame-syntax/src/extension.ts              # VS Code extension entrypoint (LSP client)
frame-syntax/src/commands.ts               # Custom VS Code commands (build, test, deploy, etc.)
frame-syntax/src/status-bar.ts             # Status bar indicators
frame-syntax/src/decorations.ts            # Decoration renderers (color swatches, etc.)
frame-syntax/src/import-manager.ts         # Auto-import for components + @/ alias resolution
frame-syntax/src/completion-provider.ts     # Supplementary providers (icons, colors, routes, files)
frame-syntax/src/icon-picker.ts            # Quick pick UI for icon selection
frame-syntax/src/side-panel.ts             # Frame Explorer tree view
frame-syntax/tsconfig.json                 # TypeScript config
frame-syntax/eslint.config.mjs             # ESLint config
```

### Modified Files

```
src/main.rs                                 # Add `frame lsp` subcommand
src/cli/mod.rs                              # Add LspCommand variant
src/parser/mod.rs                           # Add error-recovery parse mode (partial AST on failure)
src/resolver/mod.rs                         # Add @/ path alias resolution
```

### Files to Update

```
frame-syntax/package.json                   # Full extension manifest
frame-syntax/syntaxes/frame.tmLanguage.json # Complete TextMate grammar
frame-syntax/snippets/frame.code-snippets   # All missing snippets
frame-syntax/language-configuration.json    # Minor indentation fixes
```

---

## Phase 1: Frame CLI — `frame lsp` Subcommand

### `src/cli/mod.rs` — New LspCommand

```rust
#[derive(clap::Subcommand)]
pub enum Command {
    // ... existing commands ...
    /// Start the Frame Language Server (LSP) over stdio
    Lsp(LspArgs),
}

#[derive(clap::Args)]
pub struct LspArgs {
    /// Workspace root directory
    #[arg(long, default_value = ".")]
    pub workspace_root: PathBuf,
}
```

### `src/main.rs` — Route to LSP

```rust
Command::Lsp(args) => {
    frame_lsp::run_server(args.workspace_root).await;
}
```

### `src/parser/mod.rs` — Error-Recovery Parse Mode

Add a flag/function `parse_project_recover(dir)` that collects multiple parse errors instead of failing on the first one. Returns `(AST, Vec<FrameError>)`. Critical for providing rich diagnostics — the LSP needs to report *all* errors in a file, not just the first one.

### `src/resolver/mod.rs` — @/ Path Alias

In the import resolution logic:
1. Detect paths starting with `@/`
2. Look up `frame.config.json` in the project root (walk up from current file)
3. Read `"paths": { "@": "<relative_path>" }` — default `"./src"`
4. Replace `@/` with the resolved absolute path
5. Continue with existing relative-path resolution logic

Registration in `frame.config.json`:
```json
{
  "paths": {
    "@": "./src"
  }
}
```

### `frame-lsp/Cargo.toml`

```toml
[package]
name = "frame-lsp"
version = "0.1.0"
edition = "2021"

[dependencies]
frame = { path = ".." }
tower-lsp = "0.9"
tokio = { version = "1", features = ["full"] }
serde_json = "1"
serde = { version = "1", features = ["derive"] }
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### `frame-lsp/src/main.rs`

Standard `tower-lsp` entrypoint. Reads LSP messages from stdin, writes to stdout. Receives `workspace_root` from `--workspace-root` CLI arg (passed by `frame lsp` subcommand).

```rust
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|client| FrameLanguageServer::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
```

### `frame-lsp/src/server.rs`

`FrameLanguageServer` struct:

```rust
pub struct FrameLanguageServer {
    client: Client,
    workspace_root: Arc<Mutex<Option<PathBuf>>>,
    ast: Arc<Mutex<Option<AST>>>,
    config: Arc<Mutex<Option<FrameConfig>>>,
}
```

Capabilities:
- `textDocumentSync`: Full (re-parse on every change)
- `completionProvider`: trigger chars `:`, `.`, `"`, `@`, `$`
- `hoverProvider`: true
- `definitionProvider`: true
- `referencesProvider`: true
- `documentSymbolProvider`: true
- `workspaceSymbolProvider`: true
- `codeActionProvider`: true (with `CodeActionKind::QuickFix`)
- `documentFormattingProvider`: true
- `foldingRangeProvider`: true
- `semanticTokensProvider`: full
- `renameProvider`: true (prepare support)
- `documentHighlightProvider`: true
- `colorProvider`: true

---

## Phase 2: LSP Features — Detailed Spec

### 2.1 Diagnostics (`diagnostics.rs`)

On every file open/change/save, rebuild the project AST (error-recovery mode), then run:

1. **Parse errors** from `parse_project_recover()` — syntax errors with precise spans
2. **Import resolution errors** from `resolver/mod.rs` — unresolved imports, circular dependencies
3. **Type errors** from `resolver/types.rs` — type mismatches, missing required props, async violations
4. **Lint rules** from `cli/lint.rs` — all 15 rules (FR001–FR050)

Each `FrameError` / `LintDiagnostic` converts to VS Code `Diagnostic`:
- `range`: line/column from error span (use character offsets, map to UTF-16 for VS Code)
- `severity`: Error / Warning / Information / Hint
- `code`: error code like `FR001`, `E0001`
- `source`: `"frame"`
- `message`: human-readable error
- `relatedInformation`: for multi-span errors (e.g., circular dependency)
- `data`: for code action resolution

**Result**: Red squiggles on syntax errors, yellow on warnings, blue on info. All three layers fire on save.

### 2.2 Completions (`completion.rs`)

Context detection: walk the AST to determine what the cursor position is inside.

#### Contexts and Their Completions

| Cursor Context | Completions |
|----------------|-------------|
| Top-level (outside any block) | `page:`, `component`, `fn`, `import`, `:store`, `:obj`, `:enum`, `:type`, `:vars`, `:i18n`, `:validation`, `:breakpoints`, `:typography`, `:app`, `const`, `describe:`, `[:` (for test blocks that sit at top) |
| Inside `children: [...]` | 57 built-in component names (`text`, `button`, `icon`, etc.) + user-defined PascalCase components in scope + imported components |
| After `component_kind :` + enter `{` | Known props for that component from `registry.rs` (e.g., `text:` → `content:`, `image:` → `src:`, `icon:` → `name:`, `path:`). Include all event props if component supports them. |
| Inside `styles: { }` | 50+ style property names, sorted by relevance: `width`, `height`, `padding`, `background`, `font_size`, `color`, `margin`, `border_radius`, etc. |
| After style property `:` | Value hints: `dimension` for sizes, `"string"` for colors/text, `true/false` for booleans, numeric for opacity/elevation |
| Inside `props: { }` (component def) | Type keywords: `string`, `int`, `float`, `bool`, `list`, `object`, `void` |
| Inside `state: { }` | Type keywords same as props |
| After `import {` | Known component names from `frame-core` registry, plus names from `frame.config.json` plugin references |
| After `"import { ... } "` (path) | File path completions: `.fr` files relative to current file, `@/` → aliased root, `frame-` → known plugin modules |
| After `wait:` | `fetch(`, `StoreName.action(`, `ComponentName.method(` |
| After `StoreName.` (in expr) | Store fields and function names (from parsed AST) |
| After `state.` | Current page/component state field names |
| After `$` | Known `:vars` design tokens |
| In `on_click:` / `on_change:` etc. | Known function names in scope, `() => { }` lambda snippet, `navigate(` / `navigate_back(` / `navigate_modal(` |
| Inside string `\\(` | Variables in scope (identifiers, store fields) |
| After number in style | Unit suffixes: `dp`, `sp`, `px`, `%`, `ms` |
| After `@` in styles | Breakpoint names from `:breakpoints` block |
| In `navigate("` | Page routes from all parsed pages |
| In `icon: { name: "` | Icon names from `.frameicons` + SVG files |
| In `color: "` / `background: "` | `:vars` dollar tokens + common CSS colors |
| In `image: { src: "` / `avatar: { src: "` | Asset file paths |
| In `import { X } "@/"` | Files/directories under the @ alias root |
| Inside `animate: { }` | `property:`, `from:`, `to:`, `duration:`, `delay:`, `easing:`, `repeat:`, `auto_reverse:` |
| Inside `positioned: { }` | `top:`, `bottom:`, `left:`, `right:`, `width:`, `height:` |
| After `allows_children_kinds` restricted parent | Only allowed child kinds (e.g., `tab_bar:` → only `tab:` completions) |
| After `build: (` | Variable name suggestion (e.g., `item`, `element`) |

Implementation approach:

```rust
enum CompletionContext {
    TopLevel,
    InsideChildren,
    ComponentProp { component_kind: String },
    StyleProp,
    StyleValue { style_prop: String },
    ImportName,
    ImportPath,
    WaitPrefix,
    StoreField { store_name: String },
    StateField,
    VarsToken,
    EventHandler,
    // ... etc
}

fn determine_context(ast: &AST, file_path: &Path, position: Position) -> CompletionContext;

fn completions_for_context(ctx: &CompletionContext, ast: &AST, registry: &ComponentRegistry) -> Vec<CompletionItem>;
```

### 2.3 Hover (`hover.rs`)

For each AST node at the cursor position, produce documentation:

| Node | Hover Content |
|------|---------------|
| Built-in component kind | Category, prop table (required/optional), events, children rules, style props |
| User-defined component | Props, state, events (from ComponentDef) |
| Store name | Fields, actions, persist config |
| Store field | Type, default value |
| Store action | Signature (params + return type) |
| Function name | Signature, async flag, body summary |
| Import name | Source module path |
| Style property | Accepted types, description |
| Event handler | Description, expected signature |
| `:vars` token | Value |
| `:obj` field | Type, optional flag |
| `:enum` variant | Enum name, optional string value |
| `type_name` reference | Resolved type definition |
| `navigate("route")` | Target page name |
| Stdlib call (e.g., `string.upper`) | Description + platform mapping for both Android/iOS |

Format: Markdown with syntax-highlighted code blocks. Example for `text:` component:

```
**text** (Text/Content)

A text display element.

**Props:**
| Prop | Type | Required | Default |
|------|------|----------|---------|
| content | string | no | - |

**Events:** on_click

**Styles:** color, font_size, font_weight, font_family, text_overflow, max_lines, line_clamp, width, height, margin, padding, opacity
```

### 2.4 Go-to-Definition (`definition.rs`)

| Symbol | Target |
|--------|--------|
| PascalCase component in children | `component Name: { ... }` declaration |
| `fn Name:` call | `fn Name: ...` definition |
| `StoreName.field` | Field definition in `:store StoreName { ... }` |
| Store action call | `fn actionName:` inside store |
| `:obj` type reference | `:obj TypeName { ... }` |
| `:enum` variant reference | `:enum EnumName { ... }` |
| `:vars $token` | Entry in `:vars { $token: ... }` |
| Import name | The file where it's defined (cross-file) |
| `state.field` | Field in current page/component `state:` block |
| `navigate("/route")` | `page: { name: "..." route: "/route" ... }` |
| `import { Name } "path"` | The target `.fr` file |

### 2.5 Document Symbols (`symbols.rs`)

Walk the AST, produce:

| Symbol | Kind | Children |
|--------|------|----------|
| `page: { name: "..." }` | Package | params, state fields, lifecycle hooks |
| `component Name:` | Class | props, state fields |
| `:store StoreName` | Namespace | fields (Variable), actions (Method) |
| `:obj ObjName` | Struct | fields (Field) |
| `:enum EnumName` | Enum | variants (EnumMember) |
| `fn Name:` | Function | |
| `:vars` | Constant | each token |
| `import` | Module | imported names |
| `describe:` | Package | `it:` (Method) |
| `:type Alias` | Interface | |

### 2.6 Code Actions (`code_action.rs`)

| Diagnostic | Quick Fix |
|------------|-----------|
| Missing required prop (e.g., `image:` no `src:`) | "Add required prop `src:`" |
| FR040 — async no error handling | "Wrap `wait:` call in try/catch" |
| FR042 — hardcoded string | "Move to `:i18n` block" |
| FR050 — missing accessibility label | "Add `label: \"...\"` prop" |
| FR001 — component name not PascalCase | "Rename to `PascalCase`" |
| FR002 — function name not camelCase | "Rename to `camelCase`" |
| FR003 — var key not snake_case | "Rename to `snake_case`" |
| FR031 — image no dimensions | "Add `width:` and `height:` props" |
| Unresolved import | "Install plugin `frame-name`" / "Create file `path`" |
| Unrecognized PascalCase identifier | "Import `Name` from frame-core" (auto-adds `import { Name } "frame-core"`) |
| Missing newline at EOF | "Insert final newline" |

### 2.7 Formatting (`formatting.rs`)

Rules:
- Indent: 1 tab per nesting level
- `{` on same line as keyword/component kind
- `children: [` items indented one level
- `styles:` properties: space after `key:`, no space before
- Align `:` in consecutive style props (soft — space padding to align colons)
- Consistent spacing around `=>` in lambdas and build
- Remove trailing whitespace
- Single space between tokens inside blocks
- `,` after last child item optional (preserve or remove based on setting)
- Empty lines between top-level declarations (pages, components, stores)
- One blank line after `import` block

Uses the round-trip approach: parse → pretty-print from AST → output. Falls back to text-based formatting if AST parse fails.

### 2.8 Folding Ranges (`folding.rs`)

| Region | Foldable |
|--------|----------|
| `{ }` multi-line block | Yes |
| `children: [ ... ]` | Yes (collapse to `children: [...]`) |
| `styles: { ... }` | Yes (collapse single line) |
| `/* ... */` block comments | Yes |
| `import { ... } "..."` | Yes (if multi-line names) |
| `describe: "..." => { ... }` | Yes |
| `props: { ... }` | Yes |
| `state: { ... }` | Yes |
| `animate: { ... }` | Yes |

### 2.9 Semantic Tokens (`semantic_tokens.rs`)

Token types mapped to AST nodes:
- `keyword` — `page`, `component`, `fn`, `import`, `if`, `else`, `for`, `switch`, `return`, `try`, `catch`, `show_if`
- `keyword` with modifier `async` — `wait:`, `async`
- `namespace` — `:store` name, `:obj` name, `:enum` name
- `type` — `string`, `int`, `float`, `bool`, `list`, `object`, `void`, `null`
- `class` — PascalCase component invocations
- `struct` — `:obj` type references
- `enum` — `:enum` type references
- `parameter` — function parameters, lambda parameters, build parameter
- `variable` — `:var` names, `$` variables, store fields when referenced
- `property` — style keys, prop keys, `store_field_expr` field part
- `event` — `on_click:`, `on_change:`, etc.
- `decorator` — `@breakpoint` overrides, `:var`, `:vars`, `:i18n`, `:type`, `:validation`
- `modifier` `readonly` — `const` declarations
- `function` — `fn` names, store action names
- `method` — stdlib calls (`string.upper`, `list.push`)
- `string` — string literals (including interpolation base)
- `number` — integers, floats, dimensions
- `comment` — `//`, `/* */`
- `operator` — `=`, `==`, `!=`, `<=`, `>=`, `&&`, `||`, `+`, `-`, `*`, `/`, `%`, `??`, `?.`
- `macro` — `.toBe:`, `.toEqual:`, etc. in tests

---

## Phase 3: VS Code Extension Client

### 3.1 `extension.ts` — Entrypoint

```typescript
import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    // Find the frame binary
    const framePath = findFrameBinary();
    if (!framePath) {
        vscode.window.showErrorMessage(
            'Frame CLI not found. Install with: cargo install frame'
        );
        return;
    }

    const serverOptions: ServerOptions = {
        command: framePath,
        args: ['lsp', '--workspace-root', vscode.workspace.rootPath ?? '.'],
        options: { env: { ...process.env, RUST_LOG: 'info' } }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'frame' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.fr')
        },
        initializationOptions: {
            workspaceRoot: vscode.workspace.rootPath
        }
    };

    client = new LanguageClient('frame', 'Frame Language Server', serverOptions, clientOptions);
    client.start();

    // Register commands
    registerCommands(context);
    // Register status bar
    registerStatusBar(context);
    // Register decorators
    registerDecorators(context);
    // Register import manager
    registerImportManager(context);
    // Register side panel
    registerSidePanel(context);
}

export function deactivate(): Thenable<void> | undefined {
    return client?.stop();
}
```

### 3.2 `commands.ts` — Custom Commands

| Command | Description | Implementation |
|---------|-------------|----------------|
| `frame.build` | Run `frame build` | Open integrated terminal, execute `frame build` |
| `frame.buildWatch` | Run `frame build --watch` | Open terminal, execute `frame build --watch` |
| `frame.test` | Run `frame test` | Open terminal, execute `frame test` |
| `frame.testFilter` | Run test with filter | Quick pick for test names, then `frame test --filter` |
| `frame.deploy` | Deploy (pick platform) | Quick pick: iOS / Android → `frame deploy ios` or `frame deploy android` |
| `frame.lint` | Run lint | Execute `frame lint`, show problems in Problems panel |
| `frame.lintFile` | Lint current file | Execute `frame lint` with focus on current file |
| `frame.pluginAdd` | Add plugin | Input box for plugin name, then `frame plugin add <name>` |
| `frame.pluginList` | List installed | Quick pick showing installed plugins with versions |
| `frame.iconList` | List icons | Quick pick showing all registered icons with categories |
| `frame.iconGenerate` | Generate icon assets | Quick pick: iOS / Android / All, then `frame icon generate --target ...` |
| `frame.newPage` | Scaffold new page | Input box for page name/route, then snippet insertion |
| `frame.newComponent` | Scaffold new component | Input box for component name, then snippet insertion |
| `frame.newStore` | Scaffold new store | Input box for store name, then snippet insertion |
| `frame.newObj` | Scaffold new object model | Input box for object name, then snippet insertion |
| `frame.openDocs` | Open Frame docs | Open `https://frame-lang.dev/docs` (or local README) |
| `frame.showPreview` | Open preview | Run `frame preview`, show URL in notification |
| `frame.runFile` | Run current test file | Execute `frame test --filter <current-file>` |
| `frame.addImport` | Auto-import component | Quick pick of available components, inserts import |
| `frame.showIcons` | Icon browser | Quick pick grid of icons with preview |
| `frame.formatDocument` | Format current file | Trigger `textDocument/formatting` |
| `frame.restartLsp` | Restart LSP server | Stop and restart the language client |

### 3.3 `status-bar.ts`

Items:
- Left side: `Frame` logo with version (from `frame --version`)
- Right side: 
  - Diagnostic count badge (errors/warnings) — click to open Problems
  - Build status indicator (spinner during build, checkmark on success, X on failure)
  - LSP connection status (connected/reconnecting/disconnected)

### 3.4 `decorations.ts`

| Decoration | Rule |
|------------|------|
| Color swatch | `$primary: "#FF0000"` — render a colored circle before the value |
| Dimension unit dimming | `16sp`, `24dp`, `100%` — dim the unit suffix |
| Deprecated strikethrough | Items marked `@deprecated` in comments (future) |
| TODO/FIXME highlight | `TODO:`, `FIXME:`, `HACK:` in comments |
| Matching bracket highlight | `{ }` `[ ]` `( )` |

### 3.5 `import-manager.ts`

Auto-import logic:
1. Watch for PascalCase identifiers typed inside `children: [...]` that aren't in scope
2. Check against `frame-core` registry (57 built-in PascalCase? No — built-ins are lowercase. PascalCase = user-defined)
3. Actually, PascalCase components are user-defined. For built-ins, they're lowercase keywords. So auto-import is for user-defined components from other files + plugins.
4. Scan workspace for `.fr` files that export matching `component Name:` declarations
5. Show lightbulb: "Import `Name` from `./path/to/file`"
6. On accept, insert `import { Name } "./path/to/file"` at top of file
7. **@/ resolution**: if the path would be deep relative path, suggest using `@/` alias instead
8. Cache scanned components in workspace for performance

### 3.6 `completion-provider.ts` — Supplementary Inline Providers

These handle completions the LSP can't easily do (file-system dependent or UI-driven):

1. **Icon name completion**: When typing `icon: { name: "`, scan `assets/icons/*.frameicons` and `assets/icons/*.svg` to suggest names with category badges
2. **Color value completion**: When typing `color: "` or `background: "`, suggest from `:vars` tokens + CSS named colors + recent colors
3. **File path completion**: When typing `import { x } "`, suggest `.fr` files in project; when typing `image: { src: "`, suggest asset files
4. **Route completion**: When typing `navigate("`, suggest from known page routes

### 3.7 `icon-picker.ts`

Icon browser UI:
- Command: `frame.showIcons`
- Quick pick with categories as separators
- Each item shows: icon name, category badge, SF Symbol / Material mapping
- On select: inserts `"name"` into the editor at cursor
- Filterable by name

### 3.8 `side-panel.ts` — Frame Explorer

Tree view with sections:
1. **Project** (top-level declarations grouped by type)
   - Pages (with route shown)
   - Components (with prop count)
   - Stores (with field count)
   - Objects
   - Enums
   - Functions
2. **Icons** (category tree)
   - Category → icon names
3. **Plugins** (from frame.config.json + frame_modules/)
   - Plugin name → version
4. **Diagnostics** (active problems from current file)
5. **Quick Actions** (build, test, deploy, preview buttons)

---

## Phase 4: TextMate Grammar Updates

### Missing Component Kinds to Add

Add these 20+ to the `builtin_kind` match pattern in `frame.tmLanguage.json`:

```
accordion, timeline, skeleton, table,
map_view, camera_view, qr_scanner,
swipeable, draggable, refresh, long_press,
floating_action_button, otp_input,
color_picker, time_picker, date_picker,
search_bar, text_area, stepper, rating,
avatar, chip, tag, banner, tooltip,
lottie, audio_player, video_player, web_view
```

### Missing Keywords to Add

```
describe:, it:, expect:, mock:,
.toBe:, .toEqual:, .toContain:, .toBeNull:, .toBeTrue:, .toBeFalse:, .toThrow:,
navigate_back, navigate_back_to, navigate_modal, navigate_dismiss,
show_if, animate, positioned,
before_leave, on_background, on_foreground, params,
:app, on_launch, on_foreground, on_background,
:validation, :type,
persist:, secure, local,
async, watch:,
data:, build:, columns:,
\.then, \.catch,
\?\?,
\?\.,
\$\{ (brace interpolation),
\\\( (paren interpolation, already partial)
```

### Missing Style Properties to Add

```
safe_area, min_width, max_width, min_height, max_height,
padding_start, padding_end, margin_start, margin_end,
font_style, text_decoration, line_height, letter_spacing, word_spacing,
visibility, elevation, shadow_color, shadow_offset,
shadow_blur_radius, shadow_spread_radius,
align_self, aspect_ratio, weight, position,
top, bottom, left, right, start, end, z_index,
transform, rotate, scale, translate_x, translate_y,
animation, transition_duration, transition_curve,
scroll_indicator, scroll_snap, scroll_enabled,
fit, clip_behavior,
overflow_x, overflow_y, text_overflow, max_lines, line_clamp,
border, border_width, border_color,
direction, align, justify, gap, flex, wrap
```

### Missing Event Names to Add

```
on_watch, on_focus, on_blur, on_key_press, on_hover, on_animation_end,
on_touch_start, on_touch_move, on_touch_end, on_touch_cancel,
on_increment, on_decrement, on_complete,
on_long_press, on_drag, on_swipe,
on_scan, on_refresh,
on_scroll, on_scroll_end
```

---

## Phase 5: Snippet Updates

### New Snippets to Add (20+)

| Prefix | Component / Construct |
|--------|-----------------------|
| `accordion` | Collapsible section |
| `timeline` | Event timeline |
| `skeleton` | Loading placeholder |
| `table` | Data table |
| `map_view` | Map display |
| `camera_view` | Camera preview |
| `qr_scanner` | QR/barcode scanner |
| `video_player` | Video playback |
| `audio_player` | Audio player |
| `lottie` | Lottie animation |
| `web_view` | Embedded browser |
| `swipeable` | Swipe gesture wrapper |
| `draggable` | Drag gesture wrapper |
| `refresh` | Pull-to-refresh |
| `long_press` | Long press gesture |
| `otp_input` | OTP/PIN input |
| `color_picker` | Color picker |
| `date_picker` | Date picker |
| `time_picker` | Time picker |
| `search_bar` | Search input |
| `text_area` | Multi-line text |
| `avatar` | Circular avatar |
| `chip` | Compact filter chip |
| `tag` | Static label pill |
| `banner` | Info banner |
| `tooltip` | Tooltip popup |
| `stepper` | Increment/decrement |
| `rating` | Star rating |
| `floating_action_button` (alias: `fab`) | FAB |
| `bottom_nav_bar` | Bottom navigation |
| `tab` | Tab entry (for tab_bar) |
| `item` | List item |
| `animate:` | Animation block |
| `positioned:` | Absolute positioning |
| `show_if:` | Conditional display |
| `state:` | Page/component state |
| `describe:` / `it:` | Test suite |
| `expect:` | Test assertion |
| `mock:` | HTTP mock |
| `:validation` | Validation schema |
| `:type` | Type alias |
| `:app` | App lifecycle |
| `persist:` | Store persistence |
| `build:` | List/grid builder |
| `data:` | Data binding |
| `watch:` | Watch dependency |
| `wait:fetch` with then/catch | HTTP request |
| `plugin:` node | Plugin component |
| `plugin:` call statement | Plugin call |
| `navigate` with opts | Navigation with options |
| `for` loop | Iterator |

---

## Phase 6: @/ Path Alias — Full Implementation

### Resolver Changes (`src/resolver/mod.rs`)

```rust
fn resolve_import_path(import_path: &str, current_file: &Path, config: &FrameConfig) -> PathBuf {
    if import_path.starts_with("@/") {
        // Look up the alias in config
        let alias_root = config.paths.get("@")
            .map(|p| PathBuf::from(p))
            .unwrap_or_else(|| PathBuf::from("./src"));
        
        // Resolve relative to the project root (where frame.config.json lives)
        let project_root = find_project_root(current_file);
        let resolved = project_root.join(alias_root).join(&import_path[2..]);
        return resolved.with_extension("fr");
    }
    
    // Existing relative path resolution...
    if import_path.starts_with("./") || import_path.starts_with("../") {
        let parent = current_file.parent().unwrap();
        return parent.join(import_path).with_extension("fr");
    }
    
    // frame-module resolution
    if import_path.starts_with("frame-") {
        // look in frame_modules/
    }
    
    // frame-core is built-in (no file to resolve)
    PathBuf::from(import_path)
}
```

### Config Loading

Walk up directory tree from the current file's directory to find `frame.config.json`:

```rust
fn find_project_root(start: &Path) -> PathBuf {
    let mut dir = Some(start.to_path_buf());
    while let Some(d) = dir {
        if d.join("frame.config.json").exists() {
            return d;
        }
        dir = d.parent().map(|p| p.to_path_buf());
    }
    // Fall back to current directory
    std::env::current_dir().unwrap()
}
```

### FrameConfig Paths Support

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct FrameConfig {
    pub plugins: Option<HashMap<String, String>>,
    pub paths: Option<HashMap<String, String>>,  // NEW
}
```

---

## Implementation Phasing

### Phase A — Foundation (Week 1)
1. `frame-lsp/Cargo.toml` + crate structure
2. `main.rs` + `server.rs` — bare LSP that can initialize and respond to `didOpen`/`didChange`
3. `diagnostics.rs` — wire up parser errors → VS Code diagnostics
4. `src/main.rs` + `src/cli/mod.rs` — add `frame lsp` subcommand
5. `extension.ts` — basic LSP client that launches `frame lsp`

### Phase B — Core LSP Features (Week 2)
6. `completion.rs` — all 25+ completion contexts
7. `hover.rs` — component registry docs, function signatures, store info
8. `definition.rs` — go to component/function/store/import definitions
9. `symbols.rs` — document outlines

### Phase C — Polish (Week 3)
10. `code_action.rs` — quick fixes for all lint rules
11. `formatting.rs` — document formatter
12. `folding.rs` — all foldable regions
13. `rename.rs` — symbol rename
14. `highlights.rs` — document highlights
15. `references.rs` — find all references
16. `semantic_tokens.rs` — AST-driven token coloring

### Phase D — Extension UI (Week 3-4)
17. `commands.ts` — all VS Code commands
18. `status-bar.ts` — status indicators
19. `decorations.ts` — color swatches, unit dimming
20. `import-manager.ts` — auto-import with @/
21. `icon-picker.ts` — icon browser
22. `completion-provider.ts` — supplementary providers
23. `side-panel.ts` — Frame Explorer

### Phase E — Grammar & Snippets (Week 4)
24. Update `frame.tmLanguage.json` — all missing tokens
25. Update `frame.code-snippets` — all missing snippets

### Phase F — @/ Resolution & Config (Week 4)
26. `resolver/mod.rs` — @/ path alias
27. `frame.config.json` — `paths` config support
28. `frame start` scaffold — generate `paths` in config

---

## Test Plan

| Layer | Tests | Approach |
|-------|-------|----------|
| LSP completions | 50+ unit tests | Mock AST + registry, assert completion items match expected labels/kinds |
| LSP diagnostics | 30+ unit tests | Parse malformed files, assert diagnostic codes and ranges |
| LSP hover | 20+ unit tests | Assert markdown content contains expected section headers |
| LSP definitions | 20+ unit tests | Assert target URI and range match expected |
| LSP code actions | 15+ unit tests | Assert actions exist and edits are correct |
| LSP formatting | 10+ unit tests | Round-trip: format → parse → format, assert idempotence |
| @/ resolution | 10+ unit tests | Mock config, assert import paths resolve correctly |
| Extension client | Integration | Manual via VS Code launch config (existing pattern) |

---

## Questions for Review

1. **LSP binary approach**: Built into `frame` CLI as `frame lsp` subcommand — zero extra install steps. Users already have `frame`. No new binary, no PATH changes. ✓ Confirmed

2. **@/ alias config**: `frame.config.json` → `"paths": { "@": "./src" }`. ✓ Confirmed

3. **Project root detection**: Walk up from current file to nearest `frame.config.json`. ✓ Confirmed

4. **LSP language**: Rust with `tower-lsp`, reusing the existing parser crate. ✓ Confirmed

5. Any additional features you'd like prioritized?

---

*Generated: July 2026*
