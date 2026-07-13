//! `frame lint` — static analysis and style validation for `.fr` files.
//!
//! Modelled after Flutter's `flutter analyze` and Dart's linter.
//! Each rule has an ID, severity, category, and a plain-English message + fix hint.
//!
//! Rules:
//!   NAMING
//!     FR001  component-name-pascal-case   — component names must be PascalCase
//!     FR002  function-name-camel-case     — fn names must be camelCase
//!     FR003  var-name-snake-case          — :vars keys must be snake_case
//!   STYLE
//!     FR010  magic-color                  — bare hex literals should use :vars
//!     FR011  magic-number                 — bare numeric literals >4 in styles should use :vars
//!     FR012  missing-key-prop             — list: build: items should have a unique key:
//!   COMPLEXITY
//!     FR020  deep-nesting                 — component trees deeper than 8 levels
//!     FR021  large-component              — component with >25 direct+indirect children
//!   PERFORMANCE
//!     FR030  unbounded-list               — list: without max_height or flex on parent
//!     FR031  image-no-dimensions          — image: without width/height set
//!   BEST PRACTICE
//!     FR040  async-no-error-handling      — async fn with wait:fetch but no try/catch
//!     FR041  empty-catch                  — try/catch with an empty catch body
//!     FR042  hardcoded-string             — content: with a literal string (use :i18n)
//!     FR043  unused-var                   — :vars entry never referenced in the project
//!     FR044  store-field-no-type          — store field declared without explicit type
//!     FR050  missing-accessibility-label  — image: or icon: without an alt: or label: prop

use crate::parser::{AST, parse_project};
use crate::parser::ast::*;
use std::collections::HashSet;
use std::path::Path;

// ─── Severity / Diagnostic ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl Severity {
    fn label(&self) -> &'static str {
        match self {
            Severity::Info    => "info",
            Severity::Warning => "warning",
            Severity::Error   => "error",
        }
    }
    fn color_code(&self) -> &'static str {
        match self {
            Severity::Info    => "\x1b[36m", // cyan
            Severity::Warning => "\x1b[33m", // yellow
            Severity::Error   => "\x1b[31m", // red
        }
    }
}

#[derive(Debug, Clone)]
pub struct LintDiagnostic {
    /// Rule ID, e.g. "FR001"
    pub rule: &'static str,
    /// One-line rule name, e.g. "component-name-pascal-case"
    pub rule_name: &'static str,
    pub severity: Severity,
    /// Human-readable explanation
    pub message: String,
    /// Optional quick-fix hint
    pub hint: Option<&'static str>,
    /// Which file / context (best-effort)
    pub context: String,
}

impl LintDiagnostic {
    fn error(rule: &'static str, rule_name: &'static str,
             message: impl Into<String>, hint: Option<&'static str>,
             context: impl Into<String>) -> Self {
        LintDiagnostic { rule, rule_name, severity: Severity::Error,
            message: message.into(), hint, context: context.into() }
    }
    fn warning(rule: &'static str, rule_name: &'static str,
               message: impl Into<String>, hint: Option<&'static str>,
               context: impl Into<String>) -> Self {
        LintDiagnostic { rule, rule_name, severity: Severity::Warning,
            message: message.into(), hint, context: context.into() }
    }
    fn info(rule: &'static str, rule_name: &'static str,
            message: impl Into<String>, hint: Option<&'static str>,
            context: impl Into<String>) -> Self {
        LintDiagnostic { rule, rule_name, severity: Severity::Info,
            message: message.into(), hint, context: context.into() }
    }
}

// ─── LintConfig ──────────────────────────────────────────────────────────────

pub struct LintConfig {
    /// If Some, only run these rule IDs (e.g. ["FR001", "FR010"])
    pub only_rules: Option<Vec<String>>,
    /// Rules to skip
    pub skip_rules: Vec<String>,
    /// Treat warnings as errors (--strict)
    pub strict: bool,
}

impl Default for LintConfig {
    fn default() -> Self {
        LintConfig { only_rules: None, skip_rules: vec![], strict: false }
    }
}

impl LintConfig {
    fn should_run(&self, rule: &str) -> bool {
        if self.skip_rules.iter().any(|r| r == rule) { return false; }
        match &self.only_rules {
            Some(list) => list.iter().any(|r| r == rule),
            None => true,
        }
    }
}

// ─── Public entry point ───────────────────────────────────────────────────────

/// Lint the project at `project_dir`. Returns `true` if no errors (or warnings
/// when `strict = true`) were found.
pub fn run_lint(project_dir: &Path, config: &LintConfig) -> bool {
    println!("Analyzing Frame project…\n");

    let ast = match parse_project(&project_dir.to_string_lossy()) {
        Ok(a) => a,
        Err(errs) => {
            eprintln!("Cannot lint: project has {} parse error(s):", errs.len());
            for e in &errs { eprintln!("  {e}"); }
            eprintln!("\n  Fix parse errors first, then run: frame lint");
            return false;
        }
    };

    let mut diags: Vec<LintDiagnostic> = Vec::new();
    lint_naming(&ast, config, &mut diags);
    lint_style(&ast, config, &mut diags);
    lint_complexity(&ast, config, &mut diags);
    lint_performance(&ast, config, &mut diags);
    lint_best_practice(&ast, config, &mut diags);

    // Sort by severity desc, then rule, then context
    diags.sort_by(|a, b| b.severity.cmp(&a.severity).then(a.rule.cmp(b.rule)));

    print_diagnostics(&diags);

    let error_count   = diags.iter().filter(|d| d.severity == Severity::Error).count();
    let warning_count = diags.iter().filter(|d| d.severity == Severity::Warning).count();
    let info_count    = diags.iter().filter(|d| d.severity == Severity::Info).count();

    println!("\n{}", "─".repeat(60));
    if diags.is_empty() {
        println!("✓  No issues found. Your Frame project looks great!");
        return true;
    }

    println!("  {} error(s)  {} warning(s)  {} info",
             error_count, warning_count, info_count);

    if error_count > 0 {
        println!("\n✗  {} issue(s) must be fixed before building.", error_count);
    } else {
        println!("\n!  Run: frame lint --rules FR001,FR010  to focus on specific rules.");
    }
    println!("   Docs: https://frame.dev/docs/lint");
    println!();

    if config.strict { error_count == 0 && warning_count == 0 } else { error_count == 0 }
}

// ─── Output formatter ─────────────────────────────────────────────────────────

fn print_diagnostics(diags: &[LintDiagnostic]) {
    if diags.is_empty() { return; }
    for d in diags {
        let reset  = "\x1b[0m";
        let bold   = "\x1b[1m";
        let dim    = "\x1b[2m";
        let col    = d.severity.color_code();
        println!("{col}{bold}[{}]{reset} {bold}{}{reset}  {dim}({}){reset}",
                 d.rule, d.rule_name, d.severity.label());
        println!("       {}", d.message);
        if let Some(h) = d.hint {
            println!("  {dim}→ Fix: {h}{reset}");
        }
        if !d.context.is_empty() {
            println!("  {dim}@ {}{reset}", d.context);
        }
        println!();
    }
}

// ─── NAMING rules ─────────────────────────────────────────────────────────────

pub(crate) fn lint_naming(ast: &AST, cfg: &LintConfig, out: &mut Vec<LintDiagnostic>) {
    // FR001 — component names must be PascalCase
    if cfg.should_run("FR001") {
        for name in ast.components.keys() {
            if !is_pascal_case(name) {
                out.push(LintDiagnostic::error(
                    "FR001", "component-name-pascal-case",
                    format!("Component '{}' must be PascalCase (e.g. '{}').",
                            name, to_pascal_case(name)),
                    Some("Rename the component and update all call sites."),
                    format!("component {name}"),
                ));
            }
        }
    }

    // FR002 — function names must be camelCase
    if cfg.should_run("FR002") {
        for name in ast.functions.keys() {
            if !is_camel_case(name) {
                out.push(LintDiagnostic::warning(
                    "FR002", "function-name-camel-case",
                    format!("Function '{}' should be camelCase (e.g. '{}').",
                            name, to_camel_case(name)),
                    Some("Rename the function and update all call sites."),
                    format!("fn {name}"),
                ));
            }
        }
    }

    // FR003 — :vars keys must be $snake_case
    if cfg.should_run("FR003") {
        for key in ast.vars.keys() {
            let bare = key.trim_start_matches('$');
            if !is_snake_case(bare) {
                out.push(LintDiagnostic::warning(
                    "FR003", "var-name-snake-case",
                    format!(":vars key '{}' should be snake_case (e.g. '${}').",
                            key, to_snake_case(bare)),
                    Some("Rename the variable in :vars and all usage sites."),
                    format!(":vars {{ ${bare} }}"),
                ));
            }
        }
    }
}

// ─── STYLE rules ──────────────────────────────────────────────────────────────

pub(crate) fn lint_style(ast: &AST, cfg: &LintConfig, out: &mut Vec<LintDiagnostic>) {
    // FR010 — bare hex color literals in styles should use :vars
    if cfg.should_run("FR010") {
        for page in &ast.pages {
            check_nodes_for_magic_color(&page.children, out);
        }
        for comp in ast.components.values() {
            check_nodes_for_magic_color(&comp.children, out);
        }
    }

    // FR011 — bare numeric literals >4 in style values should use :vars
    if cfg.should_run("FR011") {
        for page in &ast.pages {
            check_nodes_for_magic_number(&page.children, out);
        }
        for comp in ast.components.values() {
            check_nodes_for_magic_number(&comp.children, out);
        }
    }

    // FR012 — list: with a build: lambda should have key: on each item
    if cfg.should_run("FR012") {
        for page in &ast.pages {
            check_nodes_for_missing_key(&page.children, out);
        }
        for comp in ast.components.values() {
            check_nodes_for_missing_key(&comp.children, out);
        }
    }
}

fn check_nodes_for_magic_color(nodes: &[ComponentNode], out: &mut Vec<LintDiagnostic>) {
    for node in nodes {
        if let Some(bg) = &node.styles.background {
            if is_bare_hex(bg) {
                out.push(LintDiagnostic::warning(
                    "FR010", "magic-color",
                    format!("Bare hex color '{}' found on '{}'. Extract it to :vars.", bg, node.kind),
                    Some("Add to :vars block: $myColor: \"#RRGGBB\" and use $myColor."),
                    format!("{} {{ background: {} }}", node.kind, bg),
                ));
            }
        }
        if let Some(col) = &node.styles.color {
            if is_bare_hex(col) {
                out.push(LintDiagnostic::warning(
                    "FR010", "magic-color",
                    format!("Bare hex color '{}' found on '{}'. Extract it to :vars.", col, node.kind),
                    Some("Add to :vars block: $myColor: \"#RRGGBB\" and use $myColor."),
                    format!("{} {{ color: {} }}", node.kind, col),
                ));
            }
        }
        check_nodes_for_magic_color(&node.children, out);
    }
}

fn check_nodes_for_magic_number(nodes: &[ComponentNode], out: &mut Vec<LintDiagnostic>) {
    for node in nodes {
        for (prop, val) in [
            ("padding", &node.styles.padding),
            ("margin",  &node.styles.margin),
            ("gap",     &node.styles.gap),
        ] {
            if let Some(v) = val {
                if let Some(n) = parse_dp_value(v) {
                    if n > 4.0 {
                        out.push(LintDiagnostic::info(
                            "FR011", "magic-number",
                            format!("Magic number '{}' in '{}' styles:{}. Consider a :vars entry.", v, node.kind, prop),
                            Some("Define in :vars: $spacing: \"16dp\" and reference as $spacing."),
                            format!("{} {{ {}: {} }}", node.kind, prop, v),
                        ));
                    }
                }
            }
        }
        check_nodes_for_magic_number(&node.children, out);
    }
}

fn check_nodes_for_missing_key(nodes: &[ComponentNode], out: &mut Vec<LintDiagnostic>) {
    for node in nodes {
        if node.kind == "list" && node.build.is_some() {
            if !node.props.contains_key("key") {
                out.push(LintDiagnostic::warning(
                    "FR012", "missing-key-prop",
                    "list: with build: should have a key: prop for efficient diffing.".to_string(),
                    Some("Add key: $item.id (or another unique field) to the list: node."),
                    "list: { build: ... }".to_string(),
                ));
            }
        }
        check_nodes_for_missing_key(&node.children, out);
    }
}

// ─── COMPLEXITY rules ─────────────────────────────────────────────────────────

pub(crate) fn lint_complexity(ast: &AST, cfg: &LintConfig, out: &mut Vec<LintDiagnostic>) {
    const MAX_DEPTH: usize = 8;
    const MAX_CHILDREN: usize = 25;

    for page in &ast.pages {
        if cfg.should_run("FR020") {
            let depth = max_tree_depth(&page.children, 0);
            if depth > MAX_DEPTH {
                out.push(LintDiagnostic::warning(
                    "FR020", "deep-nesting",
                    format!("Page '{}' has a component tree {depth} levels deep (max {MAX_DEPTH}). \
                             Deep nesting hurts readability and performance.", page.name),
                    Some("Extract deeply nested subtrees into named components."),
                    format!("page: {} (route: {})", page.name, page.route),
                ));
            }
        }
        if cfg.should_run("FR021") {
            let count = total_node_count(&page.children);
            if count > MAX_CHILDREN {
                out.push(LintDiagnostic::info(
                    "FR021", "large-component",
                    format!("Page '{}' contains {count} component nodes (max {MAX_CHILDREN}). \
                             Consider splitting into sub-components.", page.name),
                    Some("Move sections of the page into named components in separate .fr files."),
                    format!("page: {}", page.name),
                ));
            }
        }
    }

    for (name, comp) in &ast.components {
        if cfg.should_run("FR020") {
            let depth = max_tree_depth(&comp.children, 0);
            if depth > MAX_DEPTH {
                out.push(LintDiagnostic::warning(
                    "FR020", "deep-nesting",
                    format!("Component '{name}' has a tree {depth} levels deep (max {MAX_DEPTH}).",),
                    Some("Extract deeply nested subtrees into named sub-components."),
                    format!("component {name}"),
                ));
            }
        }
        if cfg.should_run("FR021") {
            let count = total_node_count(&comp.children);
            if count > MAX_CHILDREN {
                out.push(LintDiagnostic::info(
                    "FR021", "large-component",
                    format!("Component '{name}' contains {count} nodes (max {MAX_CHILDREN}).",),
                    Some("Split the component into smaller, focused sub-components."),
                    format!("component {name}"),
                ));
            }
        }
    }
}

fn max_tree_depth(nodes: &[ComponentNode], current: usize) -> usize {
    if nodes.is_empty() { return current; }
    nodes.iter().map(|n| max_tree_depth(&n.children, current + 1)).max().unwrap_or(current)
}

fn total_node_count(nodes: &[ComponentNode]) -> usize {
    nodes.iter().map(|n| 1 + total_node_count(&n.children)).sum()
}

// ─── PERFORMANCE rules ────────────────────────────────────────────────────────

pub(crate) fn lint_performance(ast: &AST, cfg: &LintConfig, out: &mut Vec<LintDiagnostic>) {
    let all_nodes = collect_all_nodes(ast);

    for node in &all_nodes {
        // FR030 — list: without any height/flex constraint
        if cfg.should_run("FR030") && node.kind == "list" {
            let has_height = node.styles.height.is_some()
                || node.styles.max_height.is_some()
                || node.styles.flex.is_some();
            if !has_height {
                out.push(LintDiagnostic::warning(
                    "FR030", "unbounded-list",
                    "list: has no height, max_height, or flex set. \
                     An unbounded list may cause layout overflow or render all items at once.".to_string(),
                    Some("Add styles: { height: 300dp } or flex: 1 to constrain the list."),
                    "list: { ... }".to_string(),
                ));
            }
        }

        // FR031 — image: without explicit width and height
        if cfg.should_run("FR031") && node.kind == "image" {
            let has_w = node.styles.width.is_some();
            let has_h = node.styles.height.is_some();
            if !has_w || !has_h {
                out.push(LintDiagnostic::info(
                    "FR031", "image-no-dimensions",
                    "image: has no explicit width and/or height. \
                     This can cause layout jumps while the image loads.".to_string(),
                    Some("Add styles: { width: 100dp  height: 100dp } to reserve space."),
                    "image: { src: ... }".to_string(),
                ));
            }
        }
    }
}

// ─── BEST PRACTICE rules ──────────────────────────────────────────────────────

pub(crate) fn lint_best_practice(ast: &AST, cfg: &LintConfig, out: &mut Vec<LintDiagnostic>) {
    // FR040 — async fn with wait:fetch but no try/catch
    if cfg.should_run("FR040") {
        let all_fns = collect_all_functions(ast);
        for func in &all_fns {
            if func.is_async && stmts_have_fetch(&func.body) && !stmts_have_try_catch(&func.body) {
                out.push(LintDiagnostic::warning(
                    "FR040", "async-no-error-handling",
                    format!("Async function '{}' uses wait:fetch but has no try/catch. \
                             Network errors will silently crash the action.", func.name),
                    Some("Wrap the wait:fetch call in try { } catch (e) { } to handle errors."),
                    format!("fn {}: async", func.name),
                ));
            }
        }
    }

    // FR041 — try/catch with an empty catch body
    if cfg.should_run("FR041") {
        let all_fns = collect_all_functions(ast);
        for func in &all_fns {
            check_empty_catch_in_stmts(&func.body, &func.name, out);
        }
    }

    // FR042 — hardcoded content: string (should use :i18n)
    if cfg.should_run("FR042") {
        let has_i18n = !ast.i18n.is_empty();
        if has_i18n {
            // Only warn when the project already uses :i18n (otherwise noise)
            let all_nodes = collect_all_nodes(ast);
            for node in &all_nodes {
                if let Some(Expr::Literal(Value::Str(s))) = node.props.get("content") {
                    if s.len() > 2 && !s.starts_with('$') && !s.starts_with("t:") {
                        out.push(LintDiagnostic::info(
                            "FR042", "hardcoded-string",
                            format!("Hardcoded string {:?} on '{}'. \
                                     Use t:\"key\" and add to :i18n for i18n support.", s, node.kind),
                            Some("Add an entry to :i18n { my_key: \"...\" } and use t:\"my_key\"."),
                            format!("{} {{ content: {:?} }}", node.kind, s),
                        ));
                    }
                }
            }
        }
    }

    // FR043 — :vars entries never referenced anywhere
    if cfg.should_run("FR043") {
        let referenced = collect_var_references(ast);
        for key in ast.vars.keys() {
            let bare = key.trim_start_matches('$');
            if !referenced.contains(bare) {
                out.push(LintDiagnostic::info(
                    "FR043", "unused-var",
                    format!(":vars entry '${bare}' is declared but never used in any style or prop."),
                    Some("Remove the unused variable or reference it somewhere."),
                    format!(":vars {{ ${bare}: ... }}"),
                ));
            }
        }
    }

    // FR044 — store field without explicit type (defaults to string silently)
    if cfg.should_run("FR044") {
        for (store_name, store) in &ast.stores {
            for (field_name, field) in &store.fields {
                if field.type_ == FRType::String_ {
                    // Heuristic: if the default is null or 0 but type is string, likely untyped
                    if let Some(Expr::Literal(Value::Null)) = &field.default {
                        out.push(LintDiagnostic::info(
                            "FR044", "store-field-no-type",
                            format!("Store '{store_name}' field '{field_name}' has no explicit type \
                                     annotation (defaulting to string). Add a type for clarity."),
                            Some("Change to: fieldName: object = null  or  fieldName: string = \"\""),
                            format!(":store {store_name} {{ {field_name} }}"),
                        ));
                    }
                }
            }
        }
    }

    // FR050 — image: or icon: without accessibility label
    if cfg.should_run("FR050") {
        let all_nodes = collect_all_nodes(ast);
        for node in &all_nodes {
            if node.kind == "image" || node.kind == "icon" {
                let has_label = node.props.contains_key("alt")
                    || node.props.contains_key("label")
                    || node.props.contains_key("content_description");
                if !has_label {
                    out.push(LintDiagnostic::warning(
                        "FR050", "missing-accessibility-label",
                        format!("'{}' has no alt:, label:, or content_description: prop. \
                                 Screen readers cannot describe this element.", node.kind),
                        Some("Add alt: \"Descriptive text\" so assistive technologies can read it."),
                        format!("{}: {{ ... }}", node.kind),
                    ));
                }
            }
        }
    }
}

// ─── AST traversal helpers ────────────────────────────────────────────────────

fn collect_all_nodes(ast: &AST) -> Vec<&ComponentNode> {
    let mut out = Vec::new();
    for page in &ast.pages {
        collect_nodes_recursive(&page.children, &mut out);
    }
    for comp in ast.components.values() {
        collect_nodes_recursive(&comp.children, &mut out);
    }
    out
}

fn collect_nodes_recursive<'a>(nodes: &'a [ComponentNode], out: &mut Vec<&'a ComponentNode>) {
    for node in nodes {
        out.push(node);
        collect_nodes_recursive(&node.children, out);
    }
}

fn collect_all_functions(ast: &AST) -> Vec<&Function> {
    let mut out: Vec<&Function> = Vec::new();
    out.extend(ast.functions.values());
    for store in ast.stores.values() {
        out.extend(store.actions.values());
    }
    for comp in ast.components.values() {
        out.extend(comp.functions.values());
    }
    out
}

fn stmts_have_fetch(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|s| match s {
        Stmt::WaitFetch(_) => true,
        Stmt::If(_, a, b) => stmts_have_fetch(a)
            || b.as_ref().map(|v| stmts_have_fetch(v)).unwrap_or(false),
        Stmt::For(_, _, body) => stmts_have_fetch(body),
        Stmt::Switch(_, cases) => cases.iter().any(|(_, b)| stmts_have_fetch(b)),
        Stmt::TryCatch { body, catch_body, finally_body, .. } =>
            stmts_have_fetch(body) || stmts_have_fetch(catch_body)
            || finally_body.as_ref().map(|f| stmts_have_fetch(f)).unwrap_or(false),
        _ => false,
    })
}

fn stmts_have_try_catch(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|s| match s {
        Stmt::TryCatch { .. } => true,
        Stmt::If(_, a, b) => stmts_have_try_catch(a)
            || b.as_ref().map(|v| stmts_have_try_catch(v)).unwrap_or(false),
        Stmt::For(_, _, body) => stmts_have_try_catch(body),
        _ => false,
    })
}

fn check_empty_catch_in_stmts(stmts: &[Stmt], fn_name: &str, out: &mut Vec<LintDiagnostic>) {
    for stmt in stmts {
        if let Stmt::TryCatch { catch_body, .. } = stmt {
            if catch_body.is_empty() {
                out.push(LintDiagnostic::warning(
                    "FR041", "empty-catch",
                    format!("Empty catch block in function '{}'. \
                             Swallowed errors make bugs invisible.", fn_name),
                    Some("Log or re-throw the error: catch (e) { print(e) }"),
                    format!("fn {fn_name} {{ try {{ }} catch (e) {{ }} }}"),
                ));
            }
            // Recurse into catch body too
            check_empty_catch_in_stmts(catch_body, fn_name, out);
        }
        // Recurse into other blocks
        match stmt {
            Stmt::If(_, a, b) => {
                check_empty_catch_in_stmts(a, fn_name, out);
                if let Some(v) = b { check_empty_catch_in_stmts(v, fn_name, out); }
            }
            Stmt::For(_, _, body) => check_empty_catch_in_stmts(body, fn_name, out),
            Stmt::Switch(_, cases) => {
                for (_, body) in cases { check_empty_catch_in_stmts(body, fn_name, out); }
            }
            _ => {}
        }
    }
}

/// Collect all `$varName` references used anywhere in styles, props, and i18n.
fn collect_var_references(ast: &AST) -> HashSet<String> {
    let mut refs = HashSet::new();
    // styles fields
    let all_nodes = collect_all_nodes(ast);
    for node in all_nodes {
        for field in [
            &node.styles.background, &node.styles.color, &node.styles.width,
            &node.styles.height, &node.styles.padding, &node.styles.margin,
            &node.styles.font_size, &node.styles.border_radius, &node.styles.gap,
        ].iter().filter_map(|o| o.as_ref()) {
            collect_var_refs_from_str(field, &mut refs);
        }
        for val in node.props.values() {
            collect_var_refs_from_expr(val, &mut refs);
        }
    }
    refs
}

fn collect_var_refs_from_str(s: &str, refs: &mut HashSet<String>) {
    // Match $identifier patterns
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '$' {
            let name: String = chars.by_ref()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() { refs.insert(name); }
        }
    }
}

fn collect_var_refs_from_expr(expr: &Expr, refs: &mut HashSet<String>) {
    match expr {
        Expr::Var(name) => { refs.insert(name.trim_start_matches('$').to_string()); }
        Expr::Literal(Value::Str(s)) => collect_var_refs_from_str(s, refs),
        _ => {}
    }
}

// ─── String / naming helpers ──────────────────────────────────────────────────

fn is_pascal_case(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_uppercase() => chars.all(|c| c.is_alphanumeric()),
        _ => false,
    }
}

fn is_camel_case(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_lowercase() => chars.all(|c| c.is_alphanumeric()),
        _ => false,
    }
}

fn is_snake_case(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn to_pascal_case(s: &str) -> String {
    s.split('_').map(|w| {
        let mut c = w.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }).collect()
}

fn to_camel_case(s: &str) -> String {
    let pascal = to_pascal_case(s);
    let mut c = pascal.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 { result.push('_'); }
        result.push(ch.to_ascii_lowercase());
    }
    result
}

fn is_bare_hex(s: &str) -> bool {
    let s = s.trim_matches('"');
    s.starts_with('#') && s.len() >= 4 && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

fn parse_dp_value(s: &str) -> Option<f64> {
    let s = s.trim_matches('"');
    let s = s.trim_end_matches("dp").trim_end_matches("px").trim_end_matches('%');
    s.parse::<f64>().ok()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::*;

    fn empty_ast() -> AST { AST::default() }
    fn default_cfg() -> LintConfig { LintConfig::default() }

    // ── Naming ───────────────────────────────────────────────────────────────

    #[test]
    fn fr001_catches_non_pascal_component() {
        let mut ast = empty_ast();
        ast.components.insert("myCard".to_string(), ComponentDef::default());
        let mut out = vec![];
        lint_naming(&ast, &default_cfg(), &mut out);
        assert!(out.iter().any(|d| d.rule == "FR001" && d.message.contains("myCard")));
    }

    #[test]
    fn fr001_passes_pascal_component() {
        let mut ast = empty_ast();
        ast.components.insert("MyCard".to_string(), ComponentDef::default());
        let mut out = vec![];
        lint_naming(&ast, &default_cfg(), &mut out);
        assert!(!out.iter().any(|d| d.rule == "FR001"));
    }

    #[test]
    fn fr002_catches_non_camel_function() {
        let mut ast = empty_ast();
        ast.functions.insert("LoadData".to_string(), Function {
            name: "LoadData".to_string(), is_async: false,
            params: vec![], return_type: None, body: vec![],
        });
        let mut out = vec![];
        lint_naming(&ast, &default_cfg(), &mut out);
        assert!(out.iter().any(|d| d.rule == "FR002"));
    }

    #[test]
    fn fr003_catches_non_snake_var() {
        let mut ast = empty_ast();
        ast.vars.insert("$primaryColor".to_string(), "#007BFF".to_string());
        let mut out = vec![];
        lint_naming(&ast, &default_cfg(), &mut out);
        assert!(out.iter().any(|d| d.rule == "FR003"));
    }

    // ── Style ────────────────────────────────────────────────────────────────

    #[test]
    fn fr010_catches_bare_hex_in_background() {
        let mut ast = empty_ast();
        let mut node = ComponentNode::default();
        node.kind = "container".to_string();
        node.styles.background = Some("#FF0000".to_string());
        let mut page = Page::default();
        page.children = vec![node];
        ast.pages.push(page);
        let mut out = vec![];
        lint_style(&ast, &default_cfg(), &mut out);
        assert!(out.iter().any(|d| d.rule == "FR010"));
    }

    #[test]
    fn fr010_passes_var_reference() {
        let mut ast = empty_ast();
        let mut node = ComponentNode::default();
        node.kind = "container".to_string();
        node.styles.background = Some("$primary".to_string());
        let mut page = Page::default();
        page.children = vec![node];
        ast.pages.push(page);
        let mut out = vec![];
        lint_style(&ast, &default_cfg(), &mut out);
        assert!(!out.iter().any(|d| d.rule == "FR010"));
    }

    // ── Best practice ────────────────────────────────────────────────────────

    #[test]
    fn fr040_catches_async_fetch_without_try_catch() {
        let mut ast = empty_ast();
        ast.functions.insert("loadData".to_string(), Function {
            name: "loadData".to_string(),
            is_async: true,
            params: vec![],
            return_type: None,
            body: vec![Stmt::WaitFetch(FetchExpr {
                url: Expr::Literal(Value::Str("https://api.example.com".to_string())),
                method: "GET".to_string(),
                ..Default::default()
            })],
        });
        let mut out = vec![];
        lint_best_practice(&ast, &default_cfg(), &mut out);
        assert!(out.iter().any(|d| d.rule == "FR040" && d.message.contains("loadData")));
    }

    #[test]
    fn fr040_passes_when_try_catch_present() {
        let mut ast = empty_ast();
        ast.functions.insert("loadData".to_string(), Function {
            name: "loadData".to_string(),
            is_async: true,
            params: vec![],
            return_type: None,
            body: vec![Stmt::TryCatch {
                body: vec![Stmt::WaitFetch(FetchExpr {
                    url: Expr::Literal(Value::Str("https://api.example.com".to_string())),
                    method: "GET".to_string(),
                    ..Default::default()
                })],
                catch_param: "e".to_string(),
                catch_body: vec![Stmt::Call(CallExpr { func: "print".to_string(), args: vec![], named_args: vec![] })],
                finally_body: None,
            }],
        });
        let mut out = vec![];
        lint_best_practice(&ast, &default_cfg(), &mut out);
        assert!(!out.iter().any(|d| d.rule == "FR040"));
    }

    #[test]
    fn fr041_catches_empty_catch() {
        let mut ast = empty_ast();
        ast.functions.insert("doThing".to_string(), Function {
            name: "doThing".to_string(), is_async: false, params: vec![],
            return_type: None,
            body: vec![Stmt::TryCatch {
                body: vec![],
                catch_param: "e".to_string(),
                catch_body: vec![], // empty!
                finally_body: None,
            }],
        });
        let mut out = vec![];
        lint_best_practice(&ast, &default_cfg(), &mut out);
        assert!(out.iter().any(|d| d.rule == "FR041"));
    }

    #[test]
    fn fr043_catches_unused_var() {
        let mut ast = empty_ast();
        ast.vars.insert("primary".to_string(), "#007BFF".to_string());
        // No nodes reference $primary
        let mut out = vec![];
        lint_best_practice(&ast, &default_cfg(), &mut out);
        assert!(out.iter().any(|d| d.rule == "FR043" && d.message.contains("primary")));
    }

    #[test]
    fn fr050_catches_image_without_alt() {
        let mut ast = empty_ast();
        let mut node = ComponentNode::default();
        node.kind = "image".to_string();
        node.props.insert("src".to_string(), Expr::Literal(Value::Str("logo.png".to_string())));
        // no alt:
        let mut page = Page::default();
        page.children = vec![node];
        ast.pages.push(page);
        let mut out = vec![];
        lint_best_practice(&ast, &default_cfg(), &mut out);
        assert!(out.iter().any(|d| d.rule == "FR050"));
    }

    #[test]
    fn fr050_passes_image_with_alt() {
        let mut ast = empty_ast();
        let mut node = ComponentNode::default();
        node.kind = "image".to_string();
        node.props.insert("src".to_string(), Expr::Literal(Value::Str("logo.png".to_string())));
        node.props.insert("alt".to_string(), Expr::Literal(Value::Str("App logo".to_string())));
        let mut page = Page::default();
        page.children = vec![node];
        ast.pages.push(page);
        let mut out = vec![];
        lint_best_practice(&ast, &default_cfg(), &mut out);
        assert!(!out.iter().any(|d| d.rule == "FR050"));
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    #[test]
    fn is_bare_hex_detects_hex() {
        assert!(is_bare_hex("#FF0000"));
        assert!(is_bare_hex("#fff"));
        assert!(!is_bare_hex("$primary"));
        assert!(!is_bare_hex("red"));
    }

    #[test]
    fn naming_conversions() {
        assert_eq!(to_pascal_case("my_card"), "MyCard");
        assert_eq!(to_camel_case("my_card"), "myCard");
        assert_eq!(to_snake_case("MyCard"), "my_card");
    }

    #[test]
    fn rule_filtering_works() {
        let cfg = LintConfig {
            only_rules: Some(vec!["FR001".to_string()]),
            skip_rules: vec![],
            strict: false,
        };
        assert!(cfg.should_run("FR001"));
        assert!(!cfg.should_run("FR010"));
    }

    #[test]
    fn skip_rules_works() {
        let cfg = LintConfig {
            only_rules: None,
            skip_rules: vec!["FR010".to_string()],
            strict: false,
        };
        assert!(cfg.should_run("FR001"));
        assert!(!cfg.should_run("FR010"));
    }
}
