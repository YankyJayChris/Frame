# Introduction

Welcome to **Frame** — a cross-platform mobile framework for building native iOS and Android apps using a single, strictly-typed declarative language.

## What is Frame?

Frame is a complete development toolkit that lets you write mobile app UI and logic once in a purpose-built language (`.fr` files) and compile it to real native code — **Kotlin/Compose** for Android and **UIKit/Swift** for iOS. There is no WebView, no JavaScript bridge, and no runtime interpreter. Your app compiles down to the same platform UI primitives that hand-written native apps use.

The Frame language is **declarative** and **strictly-typed**. You describe *what* your UI should look like, not *how* to build it. The type system catches mismatches, missing props, and invalid states at compile time — before your app ever runs.

## Key Features

- **71+ built-in components** — Layout, input, navigation, media, feedback, gestures, and more. Every component maps to a native platform view.
- **Native compilation** — `.fr` → PEG Parser → AST → Resolver → Type Checker → Platform-specific codegen. Android output uses Kotlin with Jetpack Compose; iOS output uses Swift with UIKit.
- **Plugin system** — Extend Frame with community or private plugins. Plugins can expose components, functions, and native platform APIs through a stable ABI.
- **Hot-reload** — `frame preview` starts a WebSocket server that watches your source files and pushes incremental updates to running simulators/devices for instant feedback.
- **Icon bundle system** — Declare icons in a `frame-icons.json` manifest. Frame bundles the selected icons (from SF Symbols, Material Icons, or custom SVGs) into each platform's native format.
- **Built-in LSP server** — Frame ships an LSP server that provides diagnostics, completions, hover info, go-to-definition, and more in any LSP-compatible editor.
- **Store / State management** — Declarative state stores with persistence (secure keychain on iOS, encrypted shared preferences on Android).
- **Standard library** — Built-in functions for strings, numbers, lists, math, dates, JSON, and utilities.
- **Validation system** — Schema-based form validation with composable validators (`required`, `email`, `min`, `max`, `pattern`, etc.).
- **Responsive design** — Built-in breakpoint system (`@sm`, `@md`, `@lg`, `@xl`) for adapting layouts across screen sizes.

## Who is Frame For?

- **Mobile developers** who want to target both iOS and Android from a single codebase without sacrificing native performance or platform idiomatics.
- **Teams** that need strict typing and compile-time guarantees to ship reliable mobile apps.
- **Design-conscious developers** who want to build beautiful, responsive UIs with a component model that maps directly to native platform views.
- **Prototypers** who need hot-reload and a fast edit-preview loop.

## Philosophy

Frame was built on a few core beliefs:

| Principle | Why |
|-----------|-----|
| **No runtime** | The framework compiles away completely. Your app is pure native code with zero framework overhead at runtime. |
| **Static typing** | Every expression has a known type at compile time. No `nil` exceptions, no type mismatches in production. |
| **Native icons** | Icons are resolved at compile time to platform-native representations (SF Symbols on iOS, Material Icons on Android). No icon font bloat. |
| **Bundle-based icons** | Only the icons you actually use are included in the final binary. This keeps app size minimal. |
| **Plugin architecture** | The core framework is intentionally focused. Plugins extend functionality without bloating the compiler or standard library. |
| **Built-in LSP** | Developer tooling is a first-class concern, not an afterthought. The LSP server ships with the CLI. |
| **`@/` path aliases** | Import from a project-root-relative `@/` prefix instead of brittle relative paths like `../../components/`. |

## The Pipeline

Frame source code goes through a well-defined pipeline:

```
.fr file → PEG Parser → AST → Resolver → Type Checker → Codegen → Kotlin/Swift
```

The parser uses **pest** grammar (defined in `frame-syntax/src/parser.pest`). After parsing, the resolver connects component references, imports, and path aliases. The type checker validates the entire tree against the component registry (71+ built-in components). Finally, the codegen phase emits either Kotlin or Swift source files, which are then compiled by the platform SDK toolchains.

## Next Steps

- [Getting Started](getting-started.md) — Install Frame and build your first app in minutes.
- [Language Guide](language-guide/overview.md) — Learn the Frame language from the ground up.
- [Architecture](architecture.md) — Understand how Frame works under the hood.
