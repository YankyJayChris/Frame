# LSP Server

Frame ships with a built-in Language Server Protocol (LSP) server as part of the `frame` CLI binary. It communicates over stdio and provides a rich editing experience in any LSP-compatible editor.

## Starting the Server

```bash
# Start LSP server (auto-detects workspace root)
frame lsp

# Start with explicit workspace root
frame lsp --workspace-root /path/to/project
```

The server is automatically launched by the [VS Code extension](ide-support.md). It communicates over **stdin/stdout** using the LSP protocol.

## Features

| Feature | Description |
|---------|-------------|
| **Diagnostics** | Real-time error reporting from the parser, type checker, and linter — red squiggles for syntax errors, yellow for warnings |
| **Completions** | 25+ context-aware completion contexts — component names inside `children: [...]`, props after component kind, style property names, store fields, events, icons, routes, and more |
| **Hover Info** | Documentation for built-in components (props, events, style props, category), function signatures, store fields/actions, style properties |
| **Go-to Definition** | Jump to component declarations, store definitions, function definitions, page declarations, object/enum types, imports |
| **Find References** | Find all usages of a symbol across the project |
| **Document Symbols** | Outline view showing pages, components, stores, objects, enums, functions with their children |
| **Workspace Symbols** | Search for any symbol across the entire project |
| **Code Actions** | Quick fixes for common issues — missing required props, naming convention violations |
| **Formatting** | Auto-indent with tabs, consistent spacing, trailing newline |
| **Folding Ranges** | Collapsible regions for blocks, arrays, comments |
| **Semantic Tokens** | AST-driven syntax highlighting for keywords, types, strings, numbers, comments |
| **Rename Symbol** | Rename components, stores, functions, and variables across the file |
| **Document Highlights** | Highlight all occurrences of the symbol at the cursor |
| **Document Colors** | Color picker for hex color values (`#FF0000`, `#FF0`, `$primary`) |

## Completion Contexts

The LSP server provides completions in 25+ contexts, including:

| Context | Triggers | Examples |
|---------|----------|---------|
| Top-level declarations | At file scope | `page:`, `:store`, `component`, `fn`, `import` |
| Component names | Inside `children: [...]` | `row`, `column`, `text`, `button`, user-defined components |
| Component props | After component kind + `{` | `content:`, `src:`, `value:`, `on_click:` |
| Style properties | Inside `styles: { }` | `width`, `height`, `padding`, `background` |
| Import names | After `import {` | Component names, store names, object types |
| Theme variables | After `$` | `$primary`, `$background`, etc. |
| Store fields | After `StoreName.` | User-defined store fields and actions |
| Breakpoints | After `@` | `@sm`, `@md`, `@lg`, `@xl` |
| Async calls | After `wait:` | `fetch()`, store actions |
| Routes | In string context | Page route paths |
| Icons | In string context for icon `name:` | All registered icon names |

## Technical Details

- **Built on**: `tower-lsp` (Tokio-based LSP framework)
- **Reuses**: PEG grammar parser, AST, type checker, linter from the Frame compiler
- **Project root discovery**: Walks up from the current file looking for `frame.config.json` or `src/project.fr`
- **Sync mode**: Full text document sync (sends entire file content on each change)
- **Semantic token types**: namespace, type, class, enum, interface, struct, parameter, variable, property, enumMember, event, function, method, macro, keyword, modifier, comment, string, number, regexp, operator, decorator

## Architecture

```
VS Code Extension              frame CLI
┌─────────────────┐    LSP     ┌──────────────────┐
│  LSP Client      │ ◄────────►│  frame lsp        │
│  (vscode-language│   stdio   │  ┌────────────┐   │
│   client)        │           │  │ Parser      │   │
│                  │           │  │ TypeChecker │   │
│  Auto-launches   │           │  │ Linter      │   │
│  frame lsp       │           │  │ Registry    │   │
└─────────────────┘           └──────────────────┘
```

The LSP server is auto-detected by the VS Code extension when opening `.fr` files. The extension finds the `frame` binary in PATH (or uses the `frame.path` setting) and launches the LSP server as a child process.
