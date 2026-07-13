# Contributing

Thank you for your interest in contributing to Frame! This guide will help you set up your development environment and understand how to make changes to the project.

## Development Setup

### Prerequisites

- **Rust** (1.70+) — Install via [rustup](https://rustup.rs/)
- **Git**
- **Optional**: Android SDK (for testing codegen output), Xcode (for iOS codegen testing)

### Clone and Build

```bash
git clone https://github.com/frame-lang/frame.git
cd frame
cargo build --release
```

The release build produces the `frame` binary in `target/release/frame`.

### Development Build

For faster iteration during development:

```bash
cargo build
```

This produces a debug build in `target/debug/frame`. Development builds are faster to compile but may be slower at runtime.

## Running Tests

Run the full test suite:

```bash
cargo test
```

Run specific test modules:

```bash
# Parser tests
cargo test parser

# Type checker tests
cargo test type_checker

# Codegen tests
cargo test codegen

# Resolver tests
cargo test resolver
```

Frame uses property-based testing with `proptest` for parser and type checker tests. If you add new parsing or type-checking logic, consider adding property-based tests that generate random valid programs and verify they parse and type-check correctly.

## Code Style

- Format all Rust code with `rustfmt` before committing:
  ```bash
  cargo fmt
  ```
- Run clippy to catch common issues:
  ```bash
  cargo clippy --all-targets --all-features
  ```
- Follow existing naming conventions:
  - Types: `PascalCase` (e.g., `AstNode`, `ComponentDef`)
  - Functions: `snake_case` (e.g., `parse_file`, `resolve_imports`)
  - Variables: `snake_case`
  - Module names: `snake_case`
- Avoid unwrap/expect in production code — propagate errors with `Result` types
- Document public APIs with doc comments (`///`)
- Keep functions focused and small — prefer composition over large functions

## PR Process

1. **Fork** the repository and create a feature branch from `main`.
2. **Make your changes** — follow the code style and add tests.
3. **Run tests** — ensure `cargo test` passes and `cargo clippy` reports no new warnings.
4. **Run `cargo fmt`** — ensure code is formatted.
5. **Open a pull request** against `main` with a clear title and description.

### PR Checklist

- [ ] Tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy --all-targets --all-features`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] New features include tests
- [ ] Public API changes are documented
- [ ] Grammar changes update `frame-syntax/src/parser.pest`

## How to Add a New Component

Adding a new built-in component requires changes in several places:

### 1. Define the Component Type Signature

Add a component entry in `src/component_registry/`. Each component needs:

```rust
ComponentDef {
    name: "my_component",
    props: vec![
        PropDef { name: "title", prop_type: Type::String, required: false, default: Some("\"\"") },
        PropDef { name: "on_click", prop_type: Type::EventHandler, required: false, default: None },
    ],
    styles: StyleSet::Layout,  // Which style category this component supports
    events: vec!["on_click"],
    accepts_children: true,
}
```

### 2. Add Parser Grammar Support

If the component introduces new syntax (not just props and styles you pass), add grammar rules in `frame-syntax/src/parser.pest`.

### 3. Add Type Checker Rules

If the component has unique validation rules, add them in `src/type_checker/`.

### 4. Add Codegen for Both Platforms

Implement the platform output in `src/codegen/`:

- **Android codegen** (`src/codegen/android/`) — Emit the Compose composable function call
- **iOS codegen** (`src/codegen/ios/`) — Emit the Swift UIKit view construction

### 5. Add Tests

- Unit tests for parsing
- Unit tests for type checking
- Snapshot tests for codegen output
- Integration tests with `frame build`

### 6. Add Documentation

- Add the component to the [Component Reference](component-reference/overview.md) with a code snippet, prop table, and platform mappings.

## How to Add a Lint Rule

Lint rules are defined in `src/lint/`. To add a new rule:

### 1. Create a Lint Rule Module

```rust
// src/lint/rules/my_rule.rs
use crate::ast::*;
use crate::lint::*;

pub struct MyRule;

impl LintRule for MyRule {
    fn name(&self) -> &'static str {
        "my-rule"
    }
    fn category(&self) -> LintCategory {
        LintCategory::Style
    }
    fn check(&self, node: &AstNode) -> Vec<LintDiagnostic> {
        // Your lint logic here
        vec![]
    }
}
```

### 2. Register the Rule

Add the rule to the lint registry in `src/lint/mod.rs`:

```rust
mod rules {
    pub mod my_rule;
}

pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(rules::my_rule::MyRule),
        // ... existing rules
    ]
}
```

### 3. Add a Test

Add a test case in `src/lint/tests/` that verifies your rule fires on the expected patterns and does not fire on valid code.

### 4. Document the Rule

Add the rule to the CLI's `--help` output or a future lint documentation page.

## How to Update the Grammar / Syntax Definitions

The grammar is defined in `frame-syntax/src/parser.pest` using PEG notation. To modify it:

### 1. Understand the Grammar Structure

The grammar is organized into sections:

```pest
// Top-level declarations
page = { "page:" ~ "{" ~ page_body ~ "}" }
component = { "component" ~ ident ~ ":" ~ "{" ~ component_body ~ "}" }
store = { ":store" ~ ident ~ "{" ~ store_body ~ "}" }
fn_decl = { "fn" ~ ident ~ ":" ~ fn_sig ~ "=>" ~ "{" ~ stmt* ~ "}" }
```

### 2. Make Your Changes

- Add new rules following existing patterns
- Ensure the grammar is unambiguous (PEG grammars must be deterministic)
- Test with edge cases

### 3. Update the Syntax Highlighting Snippets

If you change syntax keywords or structure, update:

- **VS Code extension** — `frame-syntax/syntaxes/frame.tmLanguage.json`
- **TextMate grammar** (if applicable)

### 4. Update `frame-syntax/README.md`

Document the grammar changes so that editor integrations and tooling can stay in sync.

### 5. Run Grammar Tests

```bash
cargo test parser
```

## Project Structure

The Frame project is organized as follows:

| Directory | Purpose |
|-----------|---------|
| `src/` | Compiler source (parser, AST, resolver, type checker, codegen) |
| `frame-syntax/` | VS Code extension, grammar files, and syntax definitions |
| `frame-docs/` | This documentation site (mdBook) |
| `examples/` | Example Frame projects |
| `docs/` | Additional documentation and plugin specs |
| `assets/` | Shared assets for the repository |

## Getting Help

- Open an [issue](https://github.com/frame-lang/frame/issues) for bugs, feature requests, or questions
- Check existing issues and discussions before opening a new one
- For design discussions, open a discussion thread rather than an issue

## License

By contributing to Frame, you agree that your contributions will be licensed under the same license as the project.
