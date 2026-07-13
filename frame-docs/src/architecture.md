# Architecture

Frame's architecture is designed around a **compile-time pipeline** that transforms a single declarative source language into platform-native code. There is no runtime, no interpreter, and no virtual machine — the framework compiles away completely.

## Compilation Pipeline

The core pipeline transforms `.fr` source files into native platform code:

```
.fr file → PEG Parser → AST → Resolver → Type Checker → Codegen → Kotlin/Swift
```

### 1. PEG Parser

The parser uses **pest** (a PEG parser generator for Rust) with the grammar defined in `frame-syntax/src/parser.pest`. It reads `.fr` files and produces a concrete syntax tree (CST).

Key grammar rules include:

- **`page`** — Page definitions with route, lifecycle hooks, params, and children
- **`component`** — Reusable component definitions with typed props, styles, and children
- **`:store`** — State management declarations with persistable fields
- **`:vars`** — Theme variable definitions
- **`:enum`** / **`:type`** — Enum and type alias declarations
- **`:validation`** — Validation schema definitions
- **`fn`** — Function declarations (sync and async)
- **`import`** / **`export`** — Module import/export statements

### 2. AST (Abstract Syntax Tree)

The CST is lowered into a typed AST defined in [`src/ast/`](https://github.com/frame-lang/frame/tree/main/src/ast). The AST represents the program in a structured, tree-based form that is easier to analyze and transform.

### 3. Resolver

The resolver phase connects references across files:

- **Import resolution** — Maps import paths (including `@/` path aliases) to concrete file paths
- **Component resolution** — Connects component usage to component definitions in the component registry
- **Store resolution** — Links store field accesses to their declarations
- **Plugin resolution** — Maps plugin component and function references to plugin registrations in `frame.config.json`
- **Icon resolution** — Resolves icon names to their bundle definitions in `frame-icons.json`

The resolver also handles the `@/` path alias system, which maps `@/` prefixed imports to the project's `src/` directory.

### 4. Type Checker

The type checker validates the entire resolved AST before code generation:

- **Type inference** — Infers types for variables, function return values, and expressions
- **Type validation** — Checks that component props match their declared types, function arguments match parameter types, and assignments are type-safe
- **Store field validation** — Ensures store field types are valid (`int`, `float`, `bool`, `string`, `object`, `list`)
- **Lifecycle hook validation** — Verifies that lifecycle hooks reference valid function names
- **Import validation** — Checks that all imported symbols exist in their source modules
- **Async validation** — Ensures `wait:` prefix is used on async function calls and that async functions are not called synchronously

The type checker references the **Component Registry** (71+ built-in components) to validate component usage — every prop, style, event, and child slot is checked against the component's type signature.

### 5. Codegen

The codegen phase emits platform-specific source code:

#### Android Codegen

Outputs **Kotlin** source files using **Jetpack Compose**:

- Pages become `@Composable` functions with `NavController` integration
- Components become `@Composable` functions accepting typed parameters
- Stores become state holder classes with `MutableState` / `StateFlow`
- `scaffold` maps to `Scaffold` with `TopAppBar` / `BottomAppBar`
- Layout components (`column`, `row`, `stack`, `grid`) map to Compose `Column`, `Row`, `Box`, `LazyVerticalGrid`
- Input components map to `TextField`, `Switch`, `Checkbox`, `Slider`, etc.
- Animation blocks map to `Animatable` / `animate*AsState`

#### iOS Codegen

Outputs **Swift** source files using **UIKit** (with programmatic constraints):

- Pages become `UIViewController` subclasses
- Components become `UIView` subclasses or factory functions
- Stores become `ObservableObject` classes with `@Published` properties
- `scaffold` maps to a view controller hierarchy with `UINavigationController`
- Layout components map to `UIStackView` with configured axis, spacing, and alignment
- `stack` maps to a `UIView` with subviews positioned via constraints
- Input components map to `UITextField`, `UISwitch`, `UISlider`, etc.
- Animation blocks map to `UIViewPropertyAnimator` / `UIView.animate`

#### Platform Mapping Table

| Frame Component | Android (Compose) | iOS (UIKit) |
|-----------------|-------------------|-------------|
| `column` | `Column` | `UIStackView` (vertical) |
| `row` | `Row` | `UIStackView` (horizontal) |
| `stack` | `Box` | `UIView` (constraints) |
| `scaffold` | `Scaffold` | `UINavigationController` |
| `text` | `Text` | `UILabel` |
| `button` | `Button` | `UIButton` |
| `image` | `AsyncImage` / `Image` | `UIImageView` |
| `input` | `TextField` | `UITextField` |
| `switch` | `Switch` | `UISwitch` |
| `slider` | `Slider` | `UISlider` |
| `list` | `LazyColumn` | `UITableView` |
| `grid` | `LazyVerticalGrid` | `UICollectionView` |
| `app_bar` | `TopAppBar` | `UINavigationBar` |
| `icon` | `Icon` (Material) | `UIImageView` (SF Symbol) |

## LSP Server

The LSP server is built on the **tower-lsp** framework and runs as a subprocess of the `frame` CLI. It shares the same parser, AST types, and type checker as the compiler.

Features provided by the LSP server:

| Feature | Description |
|---------|-------------|
| **Diagnostics** | Real-time parsing and type-checking errors as you type |
| **Completions** | Component names, props, style properties, store fields, functions |
| **Hover info** | Type information and documentation on hover |
| **Go-to-definition** | Navigate to component, function, store, or variable definitions |
| **Code actions** | Quick fixes for common errors |
| **Document symbols** | Outline of pages, components, stores, functions in the current file |
| **Workspace symbols** | Search across the entire project |

The LSP server is launched automatically by the VS Code extension. For manual use:

```bash
frame lsp
```

## Component Registry

The component registry is a central catalog of all 71+ built-in components. Each component entry defines:

- **Name** — The component identifier (e.g., `column`, `button`, `text`)
- **Props** — Typed properties with defaults and required/optional flags
- **Styles** — Valid style properties for this component
- **Events** — Supported event handlers (e.g., `on_click`, `on_change`)
- **Children** — Whether the component accepts children
- **Platform mappings** — Target platform view types

Components are registered at compile time in [`src/component_registry/`](https://github.com/frame-lang/frame/tree/main/src/component_registry). The registry is also used by the type checker and the LSP server for validation and completions.

## Plugin Registry

The plugin registry manages third-party and private plugins. Plugins are declared in `frame.config.json`:

```json
{
  "plugins": [
    { "name": "frame-camera", "version": "1.0" },
    { "name": "frame-storage", "version": "1.0" }
  ]
}
```

Plugins can provide:

- **Custom components** — New component types with typed props and platform implementations
- **Custom functions** — Native functions callable from Frame code
- **Platform SDK integrations** — Access to device features (camera, storage, connectivity, etc.)

Each plugin contains a `.frameplugin` manifest and platform-specific implementation code (Kotlin for Android, Swift for iOS). The compiler loads plugin definitions at build time and includes them in the codegen output.

## Icon Bundle System

The icon system works differently from traditional icon fonts. Icons are declared in a `frame-icons.json` manifest:

```json
{
  "sets": {
    "default": [
      "heart",
      "gearshape",
      "bell",
      "chevron.left",
      "line.3.horizontal"
    ]
  }
}
```

During compilation, Frame:

1. Resolves each icon name against the platform-native icon set (SF Symbols on iOS, Material Icons on Android)
2. For custom SVG icons in `assets/icons/`, converts them to platform-native representations
3. Generates platform-specific icon code (Swift `UIImage(systemName:)` calls or Compose `Icons.Default` references)
4. Includes only the used icons in the final binary — no icon font bloat

This means `icon: { name: "heart" }` becomes `UIImage(systemName: "heart")` on iOS and `Icons.Default.Favorite` on Android.

## Stdlib Translator

The standard library functions (strings, numbers, lists, math, dates, JSON, etc.) are translated to platform-native equivalents during codegen:

| Frame Stdlib | Android | iOS |
|--------------|---------|-----|
| `string.upper(s)` | `s.uppercase()` | `s.uppercased()` |
| `list.length(l)` | `l.size` | `l.count` |
| `math.sqrt(n)` | `kotlin.math.sqrt(n)` | `sqrt(n)` |
| `date.now()` | `System.currentTimeMillis()` | `Date()` |
| `to_json(obj)` | `Json.encodeToString(obj)` | `JSONEncoder().encode(obj)` |
| `log.info(msg)` | `Log.i(...)` | `os_log(.info, ...)` |

The translation is defined in [`src/stdlib/`](https://github.com/frame-lang/frame/tree/main/src/stdlib) with separate mappings for each platform.

## Path Alias Resolution

The `@/` path alias system lets you import from a project-root-relative path:

```fr
import { UserCard } "@/components/UserCard.fr"
import { loadUser } "@/controllers/UserController.fr"
```

During resolution, `@/` is expanded to the project's `src/` directory. This avoids brittle relative paths like `../../components/UserCard.fr` and makes refactoring safer.

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **No runtime** | Zero framework overhead. Your app is pure native code with nothing extra to load or interpret at runtime. |
| **Static typing** | All type checking happens at compile time. No runtime type errors, no nil/null exceptions. |
| **Native icons** | Icons are resolved to platform-native representations at compile time. No icon font downloads or runtime mapping. |
| **Bundle-based icons** | Only used icons are included in the final binary. Keeps app size minimal. |
| **Plugin architecture** | Core framework stays focused and lightweight. Plugins handle platform-specific or third-party integrations. |
| **Built-in LSP** | Developer tooling is a first-class concern. The LSP server is built alongside the compiler, sharing the same parser and type checker. |
| **`@/` path aliases** | Project-root-relative imports make code more refactorable and eliminate brittle relative paths. |
| **PEG parser (pest)** | PEG grammars are declarative, composable, and produce unambiguous parse trees. pest was chosen for its Rust integration and performance. |
| **MVC project structure** | Standardized separation of concerns (`views/pages/`, `views/components/`, `models/`, `controllers/`) makes project organization predictable across all Frame apps. |

## Source Code Layout

The Frame compiler source is organized as follows (from `Cargo.toml`):

| Path | Purpose |
|------|---------|
| `src/parser/` | PEG-based parser (pest grammar in `frame-syntax/`) |
| `src/ast/` | Abstract syntax tree types |
| `src/resolver/` | Import, component, store, and path alias resolution |
| `src/type_checker/` | Type inference and validation |
| `src/codegen/` | Platform-specific code generation (Android + iOS) |
| `src/component_registry/` | Built-in component type signatures |
| `src/plugin_system/` | Plugin loading and registry |
| `src/stdlib/` | Standard library type signatures and platform translations |
| `src/lsp/` | LSP server (tower-lsp based) |
| `src/cli/` | CLI subcommands (build, check, test, deploy, preview, lsp) |
| `src/icons/` | Icon bundle resolution |
| `src/hot_reload/` | WebSocket-based hot-reload server |
