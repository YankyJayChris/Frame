//! FR language parser.
//!
//! Parses `.fr` source files into an AST using pest PEG grammar.

pub mod ast;
pub use ast::*;

use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser as PestParser;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(PestParser)]
#[grammar = "parser/grammar.pest"]
struct FrameParser;

// ─── Error types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum ErrorCategory {
    ParseError,
    TypeMismatchError,
    UnresolvedImportError,
    MissingPropError,
    MissingAssetError,
    CircularDependencyError,
    UnsupportedPlatformError,
}

#[derive(Debug, Clone)]
pub struct FrameError {
    pub category: ErrorCategory,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl FrameError {
    fn parse(file: &str, line: usize, col: usize, msg: impl Into<String>) -> Self {
        FrameError {
            category: ErrorCategory::ParseError,
            file: file.to_string(),
            line,
            column: col,
            message: msg.into(),
        }
    }
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}:{}:{} — {}", self.category, self.file, self.line, self.column, self.message)
    }
}

// ─── parse_source_str (public helper for round-trip testing) ─────────────────

/// Parse a single .fr source string (for testing and pretty-printer round-trip).
pub fn parse_source_str(source: &str) -> Result<AST, Vec<FrameError>> {
    let mut errors = Vec::new();
    let ast = parse_source(source, "<string>", &mut errors);
    if errors.is_empty() { Ok(ast) } else { Err(errors) }
}

// ─── parse_project ────────────────────────────────────────────────────────────

/// Parse a Frame project rooted at `dir`.
///
/// Starts from `{dir}/src/project.fr`, walks imports recursively,
/// and merges all file ASTs.
pub fn parse_project(dir: &str) -> Result<AST, Vec<FrameError>> {
    let root = Path::new(dir);
    let entry = root.join("src").join("project.fr");
    let mut errors: Vec<FrameError> = Vec::new();
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut ast = AST::default();

    parse_file_recursive(&entry, root, &mut ast, &mut errors, &mut visited);

    if errors.is_empty() {
        Ok(ast)
    } else {
        Err(errors)
    }
}

fn parse_file_recursive(
    path: &Path,
    project_root: &Path,
    ast: &mut AST,
    errors: &mut Vec<FrameError>,
    visited: &mut HashSet<PathBuf>,
) {
    let canonical = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            // File doesn't exist — record error and skip
            errors.push(FrameError::parse(
                &path.to_string_lossy(),
                0, 0,
                format!("File not found: {}", path.display()),
            ));
            return;
        }
    };

    if visited.contains(&canonical) {
        return; // already parsed
    }
    visited.insert(canonical.clone());

    let source = match std::fs::read_to_string(&canonical) {
        Ok(s) => s,
        Err(e) => {
            errors.push(FrameError::parse(
                &canonical.to_string_lossy(),
                0, 0,
                format!("Cannot read file: {}", e),
            ));
            return;
        }
    };

    let file_str = canonical.to_string_lossy().to_string();
    let file_ast = parse_source(&source, &file_str, errors);

    // Resolve and recurse into imports
    for imp in &file_ast.imports {
        if imp.path == "frame-core" {
            // Built-in — skip file resolution
        } else if imp.path.starts_with("frame-") {
            // Check if this is an installed plugin: frame-camera → frame_modules/frame_camera/src/index.fr
            let plugin_name = imp.path.strip_prefix("frame-").unwrap_or("");
            let plugin_index = project_root
                .join("frame_modules")
                .join(format!("frame_{}", plugin_name.replace('-', "_")))
                .join("src")
                .join("index.fr");
            if plugin_index.exists() {
                parse_file_recursive(&plugin_index, project_root, ast, errors, visited);
            }
            // If plugin doesn't exist, silently skip (resolver will report missing imports)
        } else {
            let base = canonical.parent().unwrap_or(Path::new("."));
            let imp_path = base.join(&imp.path);
            parse_file_recursive(&imp_path, project_root, ast, errors, visited);
        }
    }

    merge_ast(ast, file_ast);
}

fn merge_ast(target: &mut AST, src: AST) {
    target.vars.extend(src.vars);
    target.i18n.extend(src.i18n);
    target.enums.extend(src.enums);
    target.type_aliases.extend(src.type_aliases);
    target.validations.extend(src.validations);
    target.stores.extend(src.stores);
    target.imports.extend(src.imports);
    target.consts.extend(src.consts);
    target.pages.extend(src.pages);
    target.components.extend(src.components);
    target.functions.extend(src.functions);
    target.tests.extend(src.tests);
    target.breakpoints.extend(src.breakpoints);
    target.typography.extend(src.typography);
    target.objects.extend(src.objects);
}

// ─── Top-level file parser ────────────────────────────────────────────────────

pub(crate) fn parse_source(source: &str, file_path: &str, errors: &mut Vec<FrameError>) -> AST {
    match FrameParser::parse(Rule::file, source) {
        Ok(pairs) => {
            let mut ast = AST::default();
            for pair in pairs {
                if pair.as_rule() == Rule::file {
                    for inner in pair.into_inner() {
                        match inner.as_rule() {
                            Rule::vars_block => {
                                ast.vars.extend(parse_vars_block(inner));
                            }
                            Rule::i18n_block => {
                                ast.i18n.extend(parse_i18n_block(inner));
                            }
                            Rule::store_block => {
                                let s = parse_store_block(inner, errors, file_path);
                                ast.stores.insert(s.name.clone(), s);
                            }
                            Rule::enum_block => {
                                let e = parse_enum_block(inner);
                                ast.enums.insert(e.name.clone(), e);
                            }
                            Rule::type_alias => {
                                let ta = parse_type_alias(inner);
                                ast.type_aliases.insert(ta.alias.clone(), ta);
                            }
                            Rule::validation_block => {
                                let vs = parse_validation_block(inner);
                                ast.validations.insert(vs.name.clone(), vs);
                            }
                            Rule::obj_block => {
                                let o = parse_obj_block(inner, errors, file_path);
                                ast.objects.insert(o.name.clone(), o);
                            }
                            Rule::import_decl => {
                                ast.imports.push(parse_import_decl(inner));
                            }
                            Rule::const_decl => {
                                let (k, v) = parse_const_decl(inner);
                                ast.consts.insert(k, v);
                            }
                            Rule::breakpoints_block => {
                                ast.breakpoints.extend(parse_breakpoints_block(inner));
                            }
                            Rule::typography_block => {
                                ast.typography.extend(parse_typography_block(inner));
                            }
                            Rule::page_decl => {
                                ast.pages.push(parse_page_decl(inner, errors, file_path));
                            }
                            Rule::component_decl => {
                                let c = parse_component_decl(inner, errors, file_path);
                                ast.components.insert(c.name.clone(), c);
                            }
                            Rule::fn_def => {
                                let f = parse_fn_def(inner, errors, file_path);
                                ast.functions.insert(f.name.clone(), f);
                            }
                            Rule::test_suite => {
                                ast.tests.push(parse_test_suite(inner, errors, file_path));
                            }
                            Rule::app_lifecycle_block => {
                                for entry in inner.into_inner() {
                                    match entry.as_rule() {
                                        Rule::app_on_launch => {
                                            ast.on_launch = entry.into_inner().next()
                                                .map(|p| p.as_str().to_string());
                                        }
                                        Rule::app_on_foreground => {
                                            ast.on_foreground = entry.into_inner().next()
                                                .map(|p| p.as_str().to_string());
                                        }
                                        Rule::app_on_background => {
                                            ast.on_background = entry.into_inner().next()
                                                .map(|p| p.as_str().to_string());
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            Rule::EOI => {}
                            _ => {}
                        }
                    }
                }
            }
            ast
        }
        Err(e) => {
            let (line, col) = match e.line_col {
                pest::error::LineColLocation::Pos((l, c)) => (l, c),
                pest::error::LineColLocation::Span((l, c), _) => (l, c),
            };
            errors.push(FrameError::parse(file_path, line, col, e.to_string()));
            AST::default()
        }
    }
}

// ─── :vars block ─────────────────────────────────────────────────────────────

fn parse_vars_block(pair: Pair<Rule>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for entry in pair.into_inner() {
        if entry.as_rule() == Rule::vars_entry {
            let mut inner = entry.into_inner();
            let raw = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let key = raw.strip_prefix('$').unwrap_or(&raw).to_string();
            let val = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            map.insert(key, val);
        }
    }
    map
}

// ─── :i18n block ─────────────────────────────────────────────────────────────

fn parse_i18n_block(pair: Pair<Rule>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for entry in pair.into_inner() {
        if entry.as_rule() == Rule::i18n_entry {
            let mut inner = entry.into_inner();
            let key = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let raw = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let val = strip_quotes(&raw);
            map.insert(key, val);
        }
    }
    map
}

// ─── import_decl ─────────────────────────────────────────────────────────────

fn parse_import_decl(pair: Pair<Rule>) -> Import {
    let mut names = Vec::new();
    let mut path = String::new();
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::import_name_list => {
                for name_pair in child.into_inner() {
                    if name_pair.as_rule() == Rule::import_name {
                        let mut ni = name_pair.into_inner();
                        let original = ni.next().map(|p| p.as_str().to_string()).unwrap_or_default();
                        let alias = ni.next().map(|p| p.as_str().to_string());
                        names.push((original, alias));
                    }
                }
            }
            Rule::string => {
                path = strip_quotes(child.as_str());
            }
            _ => {}
        }
    }
    Import { names, path }
}

// ─── const_decl ──────────────────────────────────────────────────────────────

fn parse_const_decl(pair: Pair<Rule>) -> (String, ConstValue) {
    let mut inner = pair.into_inner();
    let name = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    let val_pair = inner.next();
    let val = match val_pair {
        Some(p) => match p.as_rule() {
            Rule::string  => ConstValue::Str(strip_quotes(p.as_str())),
            Rule::float   => ConstValue::Float(p.as_str().parse().unwrap_or(0.0)),
            Rule::integer => ConstValue::Int(p.as_str().parse().unwrap_or(0)),
            Rule::bool_lit => ConstValue::Bool(p.as_str() == "true"),
            _ => ConstValue::Str(p.as_str().to_string()),
        },
        None => ConstValue::Str(String::new()),
    };
    (name, val)
}

// ─── :breakpoints block ───────────────────────────────────────────────────────

fn parse_breakpoints_block(pair: Pair<Rule>) -> Vec<Breakpoint> {
    let mut bps = Vec::new();
    for entry in pair.into_inner() {
        if entry.as_rule() == Rule::breakpoint_entry {
            let mut inner = entry.into_inner();
            let name = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let dim  = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let min_width_dp = parse_dimension_to_f32(&dim);
            bps.push(Breakpoint { name, min_width_dp });
        }
    }
    bps
}

// ─── :typography block ────────────────────────────────────────────────────────

fn parse_typography_block(pair: Pair<Rule>) -> HashMap<String, TypographyStyle> {
    let mut map = HashMap::new();
    for entry in pair.into_inner() {
        if entry.as_rule() == Rule::typography_entry {
            let mut inner = entry.into_inner();
            let name = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let mut style = TypographyStyle { name: name.clone(), ..Default::default() };
            for prop in inner {
                match prop.as_rule() {
                    Rule::typography_prop => {
                        // In pest, string literal keywords in a rule don't produce child pairs,
                        // so we extract key/value by splitting the raw text on ':'.
                        let raw = prop.as_str();
                        // Skip breakpoint_override alternatives handled by outer match
                        if raw.trim_start().starts_with('@') {
                            // handled by breakpoint_override arm below
                        } else {
                            let colon_pos = raw.find(':');
                            if let Some(pos) = colon_pos {
                                let key = raw[..pos].trim();
                                let val_raw = raw[pos+1..].trim();
                                let val = strip_quotes(val_raw);
                                match key {
                                    "font_size"      => { style.font_size = val; }
                                    "font_weight"    => { style.font_weight = Some(val); }
                                    "font_family"    => { style.font_family = Some(val); }
                                    "line_height"    => { style.line_height = Some(val); }
                                    "letter_spacing" => { style.letter_spacing = Some(val); }
                                    "color"          => { style.color = Some(val); }
                                    _ => {}
                                }
                            }
                        }
                    }
                    Rule::breakpoint_override => {
                        let (bp_name, bp_style) = parse_bp_override_as_typography(prop);
                        style.breakpoint_overrides.insert(bp_name, Box::new(bp_style));
                    }
                    _ => {}
                }
            }
            map.insert(name, style);
        }
    }
    map
}

fn parse_bp_override_as_typography(pair: Pair<Rule>) -> (String, TypographyStyle) {
    let mut inner = pair.into_inner();
    let name = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    let mut style = TypographyStyle::default();
    for prop in inner {
        if prop.as_rule() == Rule::generic_style_prop {
            let mut pi = prop.into_inner();
            let key = pi.next().map(|p| p.as_str()).unwrap_or("");
            let val = pi.next().map(|p| strip_quotes(p.as_str())).unwrap_or_default();
            match key {
                "font_size"   => style.font_size = val,
                "font_weight" => style.font_weight = Some(val),
                "color"       => style.color = Some(val),
                _ => {}
            }
        }
    }
    (name, style)
}

// ─── :obj block ───────────────────────────────────────────────────────────────

/// Parse `:obj TypeName { field: type  other: type? }`
fn parse_obj_block(pair: Pair<Rule>, _errors: &mut Vec<FrameError>, _file: &str) -> ObjDef {
    let mut inner = pair.into_inner();
    let name = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    let mut fields = Vec::new();

    for field_pair in inner {
        if field_pair.as_rule() == Rule::obj_field {
            let mut fp = field_pair.into_inner();
            let field_name = fp.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let type_str   = fp.next().map(|p| p.as_str().trim().to_string()).unwrap_or_default();
            // Check for trailing ? (optional marker) — grammar captures it as remaining token
            let optional   = fp.next().map(|p| p.as_str() == "?").unwrap_or(false);
            let type_ = parse_type_name(&type_str);
            fields.push(ObjField { name: field_name, type_, optional });
        }
    }

    ObjDef { name, fields }
}

fn parse_validation_block(pair: Pair<Rule>) -> ValidationSchema {
    let mut inner = pair.into_inner();
    let name = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    let mut fields = Vec::new();
    for child in inner {
        if child.as_rule() == Rule::validation_field {
            let mut fp = child.into_inner();
            let field_name = fp.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let mut rules = Vec::new();
            for rp in fp {
                if rp.as_rule() == Rule::validation_rule {
                    let mut r_inner = rp.into_inner();
                    let rule_name = r_inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
                    let arg = r_inner.next().map(parse_expr);
                    rules.push(ValidationRule { name: rule_name, arg });
                }
            }
            fields.push(ValidationField { field: field_name, rules });
        }
    }
    ValidationSchema { name, fields }
}

fn parse_enum_block(pair: Pair<Rule>) -> EnumDef {
    let mut inner = pair.into_inner();
    let name = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    let mut variants = Vec::new();
    for v in inner {
        if v.as_rule() == Rule::enum_variant {
            let mut vp = v.into_inner();
            let vname = vp.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let value = vp.next().map(|p| strip_quotes(p.as_str()));
            variants.push(EnumVariant { name: vname, value });
        }
    }
    EnumDef { name, variants }
}

fn parse_type_alias(pair: Pair<Rule>) -> TypeAlias {
    let mut inner = pair.into_inner();
    let alias = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    let target_str = inner.next().map(|p| p.as_str().trim().to_string()).unwrap_or_default();
    let target = parse_type_name(&target_str);
    TypeAlias { alias, target }
}

// ─── :store block ─────────────────────────────────────────────────────────────

fn parse_store_block(pair: Pair<Rule>, errors: &mut Vec<FrameError>, file: &str) -> StoreSlice {
    let mut inner = pair.into_inner();
    let name = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    let mut slice = StoreSlice { name: name.clone(), ..Default::default() };

    for member in inner {
        match member.as_rule() {
            Rule::store_field => {
                let f = parse_store_field(member);
                slice.fields.insert(f.name.clone(), f);
            }
            Rule::persist_block => {
                for entry in member.into_inner() {
                    if entry.as_rule() == Rule::persist_entry {
                        let raw = entry.as_str(); // e.g., "token: secure"
                        let mut pi = entry.into_inner();
                        let field_name = pi.next().map(|p| p.as_str().to_string()).unwrap_or_default();
                        // "secure" or "local" are inline string literals — not named child pairs
                        // Detect from raw text
                        let strategy = if raw.contains("secure") {
                            PersistStrategy::Secure
                        } else {
                            PersistStrategy::Local
                        };
                        slice.persist.insert(field_name, strategy);
                    }
                }
            }
            Rule::store_fn => {
                let f = parse_store_fn(member, errors, file);
                slice.actions.insert(f.name.clone(), f);
            }
            _ => {}
        }
    }
    slice
}

fn parse_store_field(pair: Pair<Rule>) -> StoreField {
    let mut inner = pair.into_inner();
    let name = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    let type_ = inner.next().map(|p| parse_type_name(p.as_str())).unwrap_or_default();
    let default = inner.next().map(parse_expr);
    StoreField { name, type_, default }
}

fn parse_store_fn(pair: Pair<Rule>, errors: &mut Vec<FrameError>, file: &str) -> Function {
    let raw = pair.as_str();
    let is_async = raw.contains("async");
    let mut inner = pair.into_inner();
    let name = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    let mut params = Vec::new();
    let mut body = Vec::new();
    for child in inner {
        match child.as_rule() {
            Rule::fn_params => { params = parse_fn_params(child); }
            Rule::assign_stmt | Rule::return_stmt | Rule::if_stmt | Rule::for_stmt
            | Rule::switch_stmt | Rule::try_catch_stmt | Rule::wait_fetch_stmt
            | Rule::wait_call_stmt | Rule::method_call_stmt | Rule::call_stmt
            | Rule::navigate_stmt | Rule::navigate_replace_stmt | Rule::navigate_back_stmt
            | Rule::navigate_back_to_stmt | Rule::navigate_modal_stmt | Rule::navigate_dismiss_stmt
            | Rule::plugin_call | Rule::var_decl => {
                if let Some(s) = parse_stmt(child, errors, file) { body.push(s); }
            }
            _ => {}
        }
    }
    Function { name, is_async, params, return_type: None, body }
}

// ─── fn_def ───────────────────────────────────────────────────────────────────

fn parse_fn_def(pair: Pair<Rule>, errors: &mut Vec<FrameError>, file: &str) -> Function {
    let raw = pair.as_str();
    let is_async = raw.contains("async");

    let mut name = String::new();
    let mut params = Vec::new();
    let mut body = Vec::new();
    let mut saw_name = false;

    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::ident if !saw_name => {
                name = child.as_str().to_string();
                saw_name = true;
            }
            Rule::fn_params => {
                params = parse_fn_params(child);
            }
            // stmt is a silent rule (_) so children are the actual statement rule pairs
            Rule::assign_stmt | Rule::return_stmt | Rule::if_stmt | Rule::for_stmt
            | Rule::switch_stmt | Rule::try_catch_stmt | Rule::wait_fetch_stmt
            | Rule::wait_call_stmt | Rule::method_call_stmt | Rule::call_stmt
            | Rule::navigate_stmt | Rule::navigate_replace_stmt | Rule::navigate_back_stmt
            | Rule::navigate_back_to_stmt | Rule::navigate_modal_stmt | Rule::navigate_dismiss_stmt
            | Rule::plugin_call | Rule::var_decl => {
                if let Some(s) = parse_stmt(child, errors, file) { body.push(s); }
            }
            _ => {}
        }
    }
    Function { name, is_async, params, return_type: None, body }
}

fn parse_fn_params(pair: Pair<Rule>) -> Vec<(String, FRType, Option<Expr>)> {
    let mut params = Vec::new();
    for child in pair.into_inner() {
        if child.as_rule() == Rule::param {
            let mut pi = child.into_inner();
            let pname   = pi.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let ptype   = pi.next().map(|p| parse_type_name(p.as_str())).unwrap_or_default();
            // plan §1e — optional default value after `=`
            let default = pi.next().map(parse_expr);
            params.push((pname, ptype, default));
        }
    }
    params
}

fn parse_type_name(s: &str) -> FRType {
    match s {
        "int"    => FRType::Int,
        "float"  => FRType::Float,
        "bool"   => FRType::Bool,
        "string" => FRType::String_,
        "object" => FRType::Object,
        "list"   => FRType::List,
        "null"   => FRType::Nullable(Box::new(FRType::String_)),
        _        => FRType::Custom(s.to_string()),
    }
}

// ─── page_decl ────────────────────────────────────────────────────────────────

fn parse_page_decl(pair: Pair<Rule>, errors: &mut Vec<FrameError>, file: &str) -> Page {
    let mut page = Page::default();
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::page_name => {
                let raw = child.into_inner().next().map(|p| p.as_str().to_string()).unwrap_or_default();
                page.name = strip_quotes(&raw);
            }
            Rule::page_route => {
                let raw = child.into_inner().next().map(|p| p.as_str().to_string()).unwrap_or_default();
                page.route = strip_quotes(&raw);
            }
            Rule::page_params => {
                for entry in child.into_inner() {
                    if entry.as_rule() == Rule::page_param_entry {
                        let mut ei = entry.into_inner();
                        let pname = ei.next().map(|p| p.as_str().to_string()).unwrap_or_default();
                        let ptype = ei.next().map(|p| parse_type_name(p.as_str())).unwrap_or_default();
                        page.params.push((pname, ptype));
                    }
                }
            }
            // before_enter / before_leave now accept full event_handler expressions
            Rule::page_before_enter => {
                page.before_enter = child.into_inner().next().map(parse_expr);
            }
            Rule::page_before_leave => {
                page.before_leave = child.into_inner().next().map(parse_expr);
            }
            Rule::page_on_mount => {
                page.on_mount = child.into_inner().next().map(parse_expr);
            }
            Rule::page_on_unmount => {
                page.on_unmount = child.into_inner().next().map(parse_expr);
            }
            Rule::page_on_background => {
                page.on_background = child.into_inner().next().map(parse_expr);
            }
            Rule::page_on_foreground => {
                page.on_foreground = child.into_inner().next().map(parse_expr);
            }
            Rule::styles_block => {
                page.styles = parse_styles_block(child);
            }
            Rule::state_decl => {
                for sf in child.into_inner() {
                    if sf.as_rule() == Rule::state_field {
                        let f = parse_state_field(sf);
                        page.state.insert(f.name.clone(), f);
                    }
                }
            }
            Rule::children_block => {
                page.children = parse_children_block(child, errors, file);
            }
            Rule::component_node => {
                page.children.push(parse_component_node(child, errors, file));
            }
            Rule::fn_def => {
                let f = parse_fn_def(child, errors, file);
                page.functions.insert(f.name.clone(), f);
            }
            _ => {}
        }
    }
    page
}

fn parse_state_field(pair: Pair<Rule>) -> StateField {
    let mut inner = pair.into_inner();
    let name  = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    let type_ = inner.next().map(|p| parse_type_name(p.as_str())).unwrap_or_default();
    let default = inner.next().map(parse_expr);
    StateField { name, type_, default }
}

// ─── component_decl ───────────────────────────────────────────────────────────

fn parse_component_decl(pair: Pair<Rule>, errors: &mut Vec<FrameError>, file: &str) -> ComponentDef {
    let mut inner = pair.into_inner();
    let name = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    let mut def = ComponentDef { name, ..Default::default() };
    for child in inner {
        match child.as_rule() {
            Rule::props_block => {
                for pd in child.into_inner() {
                    if pd.as_rule() == Rule::prop_def {
                        let p = parse_prop_def(pd);
                        def.props.insert(p.name.clone(), p);
                    }
                }
            }
            Rule::state_decl => {
                for sf in child.into_inner() {
                    if sf.as_rule() == Rule::state_field {
                        let f = parse_state_field(sf);
                        def.state.insert(f.name.clone(), f);
                    }
                }
            }
            Rule::styles_block => {
                def.styles = parse_styles_block(child);
            }
            Rule::children_block => {
                def.children = parse_children_block(child, errors, file);
            }
            Rule::animate_block => {
                def.animate.push(parse_animate_block(child));
            }
            Rule::show_if_prop => {} // handled on nodes
            Rule::fn_def => {
                let f = parse_fn_def(child, errors, file);
                def.functions.insert(f.name.clone(), f);
            }
            Rule::component_node => {
                def.children.push(parse_component_node(child, errors, file));
            }
            _ => {
                apply_event_prop_to_event_map(child, &mut def.events);
            }
        }
    }
    def
}

fn parse_prop_def(pair: Pair<Rule>) -> PropDef {
    let mut inner = pair.into_inner();
    let name  = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    let type_ = inner.next().map(|p| parse_type_name(p.as_str())).unwrap_or_default();
    let default = inner.next().map(parse_expr);
    PropDef { name, type_, required: default.is_none(), default }
}

// ─── component_node ───────────────────────────────────────────────────────────

fn parse_children_block(pair: Pair<Rule>, errors: &mut Vec<FrameError>, file: &str) -> Vec<ComponentNode> {
    pair.into_inner()
        .filter(|p| p.as_rule() == Rule::component_node)
        .map(|p| parse_component_node(p, errors, file))
        .collect()
}

fn parse_component_node(pair: Pair<Rule>, errors: &mut Vec<FrameError>, file: &str) -> ComponentNode {
    let mut node = ComponentNode::default();

    // Extract the component kind from the raw text (before the first ':' or '(')
    // component_kind is a silent rule (_), so it won't appear as a named child.
    let raw = pair.as_str().trim();
    let kind_end = raw.find(':').or_else(|| raw.find('(')).unwrap_or(0);
    node.kind = raw[..kind_end].trim().to_string();

    let inner_pairs: Vec<Pair<Rule>> = pair.into_inner().collect();

    // If the first child is a pascal_ident (custom component), it IS a named pair
    // and its text is the kind. Override if so.
    let start_idx = if let Some(first) = inner_pairs.first() {
        if first.as_rule() == Rule::pascal_ident {
            node.kind = first.as_str().to_string();
            1
        } else {
            0
        }
    } else {
        0
    };

    for child in &inner_pairs[start_idx..] {
        match child.as_rule() {
            Rule::styles_block => {
                node.styles = parse_styles_block(child.clone());
            }
            Rule::children_block => {
                node.children = parse_children_block(child.clone(), errors, file);
            }
            Rule::animate_block => {
                node.animate.push(parse_animate_block(child.clone()));
            }
            Rule::show_if_prop => {
                node.show_if = child.clone().into_inner().next().map(parse_expr);
            }
            Rule::data_prop => {
                node.data = child.clone().into_inner().next().map(parse_expr);
            }
            Rule::build_prop => {
                node.build = Some(parse_build_prop(child.clone(), errors, file));
            }
            Rule::alignment_prop => {
                node.alignment = parse_alignment_prop(child.clone());
            }
            Rule::positioned_prop => {
                node.positioned = Some(parse_positioned_prop(child.clone()));
            }
            // ── Original named props ──────────────────────────────────────
            Rule::content_prop | Rule::src_prop | Rule::value_prop
            | Rule::title_prop | Rule::icon_prop | Rule::direction_prop
            | Rule::current_prop | Rule::validation_prop => {
                let key = match child.as_rule() {
                    Rule::content_prop    => "content",
                    Rule::src_prop        => "src",
                    Rule::value_prop      => "value",
                    Rule::title_prop      => "title",
                    Rule::icon_prop       => "icon",
                    Rule::direction_prop  => "direction",
                    Rule::current_prop    => "current",
                    Rule::validation_prop => "validation",
                    _ => "unknown",
                };
                if let Some(expr_pair) = child.clone().into_inner().next() {
                    node.props.insert(key.to_string(), parse_expr(expr_pair));
                }
            }
            // ── New first-class named props ───────────────────────────────
            Rule::min_prop | Rule::max_prop | Rule::placeholder_prop
            | Rule::url_prop | Rule::lat_prop | Rule::lng_prop
            | Rule::lines_prop | Rule::length_prop | Rule::count_prop
            | Rule::message_prop | Rule::selected_prop | Rule::checked_prop
            | Rule::refreshing_prop | Rule::duration_prop
            | Rule::label_prop | Rule::text_prop => {
                let key = match child.as_rule() {
                    Rule::min_prop         => "min",
                    Rule::max_prop         => "max",
                    Rule::placeholder_prop => "placeholder",
                    Rule::url_prop         => "url",
                    Rule::lat_prop         => "lat",
                    Rule::lng_prop         => "lng",
                    Rule::lines_prop       => "lines",
                    Rule::length_prop      => "length",
                    Rule::count_prop       => "count",
                    Rule::message_prop     => "message",
                    Rule::selected_prop    => "selected",
                    Rule::checked_prop     => "checked",
                    Rule::refreshing_prop  => "refreshing",
                    Rule::duration_prop    => "duration",
                    Rule::label_prop       => "label",
                    Rule::text_prop        => "text",
                    _ => "unknown",
                };
                if let Some(expr_pair) = child.clone().into_inner().next() {
                    node.props.insert(key.to_string(), parse_expr(expr_pair));
                }
            }
            // ── New first-class event props stored in node.props ──────────
            Rule::on_scan_prop | Rule::on_complete_prop
            | Rule::on_scan | Rule::on_refresh
            | Rule::on_increment | Rule::on_decrement
            | Rule::on_complete | Rule::on_long_press
            | Rule::on_drag | Rule::on_swipe => {
                apply_extended_event_to_props(child.clone(), &mut node.props);
            }
            Rule::columns_prop => {
                if let Some(ra) = child.clone().into_inner().next() {
                    let val = ra.as_str().to_string();
                    node.props.insert("columns".to_string(), Expr::Literal(Value::Str(val)));
                }
            }
            Rule::items_prop => {
                let kids = child.clone().into_inner()
                    .filter(|p| p.as_rule() == Rule::component_node)
                    .map(|p| parse_component_node(p, errors, file))
                    .collect::<Vec<_>>();
                node.children.extend(kids);
            }
            Rule::fit_prop => {
                if let Some(v) = child.clone().into_inner().next() {
                    node.props.insert("fit".to_string(), Expr::Literal(Value::Str(v.as_str().to_string())));
                }
            }
            Rule::clip_behavior_prop => {
                if let Some(v) = child.clone().into_inner().next() {
                    node.props.insert("clip_behavior".to_string(), Expr::Literal(Value::Str(v.as_str().to_string())));
                }
            }
            Rule::generic_prop => {
                let mut pi = child.clone().into_inner();
                let key = pi.next().map(|p| p.as_str().to_string()).unwrap_or_default();
                if let Some(val_p) = pi.next() {
                    node.props.insert(key, parse_expr(val_p));
                }
            }
            Rule::props_block => {}
            _ => {
                apply_event_prop_to_event_map(child.clone(), &mut node.events);
                apply_extended_event_to_props(child.clone(), &mut node.props);
            }
        }
    }
    node
}

fn parse_build_prop(pair: Pair<Rule>, errors: &mut Vec<FrameError>, file: &str) -> Function {
    // build: (item) => { component_node }  or  build: lambda_expr
    let mut param = String::from("item");
    let mut body = Vec::new();
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::ident => { param = child.as_str().to_string(); }
            Rule::component_node => {
                // Wrap node as a return stmt
                let node = parse_component_node(child, errors, file);
                body.push(Stmt::Return(Expr::Literal(Value::Str(format!("component:{}", node.kind)))));
            }
            Rule::lambda_expr => {
                let (params, stmts) = parse_lambda(child, errors, file);
                return Function {
                    name: "build".to_string(),
                    is_async: false,
                    params: params.iter().map(|p| (p.clone(), FRType::Object, None)).collect(),
                    return_type: None,
                    body: stmts,
                };
            }
            _ => {}
        }
    }
    Function {
        name: "build".to_string(),
        is_async: false,
        params: vec![(param, FRType::Object, None)],
        return_type: None,
        body,
    }
}

// ─── alignment + positioned ───────────────────────────────────────────────────

fn parse_alignment_prop(pair: Pair<Rule>) -> StackAlignment {
    let val = pair.into_inner().next().map(|p| p.as_str()).unwrap_or("");
    match val {
        "top_left"      => StackAlignment::TopLeft,
        "top_center"    => StackAlignment::TopCenter,
        "top_right"     => StackAlignment::TopRight,
        "center_left"   => StackAlignment::CenterLeft,
        "center"        => StackAlignment::Center,
        "center_right"  => StackAlignment::CenterRight,
        "bottom_left"   => StackAlignment::BottomLeft,
        "bottom_center" => StackAlignment::BottomCenter,
        "bottom_right"  => StackAlignment::BottomRight,
        _               => StackAlignment::TopLeft,
    }
}

fn parse_positioned_prop(pair: Pair<Rule>) -> PositionedProps {
    let mut pp = PositionedProps::default();
    for entry in pair.into_inner() {
        if entry.as_rule() == Rule::positioned_entry {
            let text = entry.as_str();
            let mut pi = entry.into_inner();
            // The grammar matches ("top"|"bottom"|...) ~ ":" ~ (dimension | dollar_var)
            // pest captures the whole string; we need to split manually
            let parts: Vec<&str> = text.splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim();
                let val_str = parts[1].trim().to_string();
                let val = Some(val_str);
                match key {
                    "top"    => pp.top    = val,
                    "bottom" => pp.bottom = val,
                    "left"   => pp.left   = val,
                    "right"  => pp.right  = val,
                    "width"  => pp.width  = val,
                    "height" => pp.height = val,
                    _ => {}
                }
            } else {
                // fallback: use inner pairs
                let key_p = pi.next().map(|p| p.as_str().to_string()).unwrap_or_default();
                let val_p = pi.next().map(|p| p.as_str().to_string());
                match key_p.as_str() {
                    "top"    => pp.top    = val_p,
                    "bottom" => pp.bottom = val_p,
                    "left"   => pp.left   = val_p,
                    "right"  => pp.right  = val_p,
                    "width"  => pp.width  = val_p,
                    "height" => pp.height = val_p,
                    _ => {}
                }
            }
        }
    }
    pp
}

// ─── styles_block ─────────────────────────────────────────────────────────────

fn parse_styles_block(pair: Pair<Rule>) -> Styles {
    let mut s = Styles::default();
    for prop in pair.into_inner() {
        match prop.as_rule() {
            Rule::breakpoint_override => {
                let (bp, sub) = parse_breakpoint_override(prop);
                s.breakpoint_overrides.insert(bp, Box::new(sub));
            }
            Rule::overflow_prop => {
                if let Some(v) = prop.into_inner().next() {
                    s.overflow = parse_overflow_value(v.as_str());
                }
            }
            Rule::overflow_x_prop => {
                if let Some(v) = prop.into_inner().next() {
                    s.overflow_x = Some(parse_overflow_value(v.as_str()));
                }
            }
            Rule::overflow_y_prop => {
                if let Some(v) = prop.into_inner().next() {
                    s.overflow_y = Some(parse_overflow_value(v.as_str()));
                }
            }
            Rule::text_overflow_prop => {
                if let Some(v) = prop.into_inner().next() {
                    s.text_overflow = parse_text_overflow_value(v.as_str());
                }
            }
            Rule::max_lines_prop => {
                if let Some(v) = prop.into_inner().next() {
                    s.max_lines = v.as_str().parse().ok();
                }
            }
            Rule::line_clamp_prop => {
                if let Some(v) = prop.into_inner().next() {
                    s.line_clamp = v.as_str().parse().ok();
                }
            }
            Rule::scroll_indicator_prop => {
                if let Some(v) = prop.into_inner().next() {
                    s.scroll_indicator = match v.as_str() { "true" => Some(true), "false" => Some(false), _ => None };
                }
            }
            Rule::scroll_snap_prop => {
                if let Some(v) = prop.into_inner().next() {
                    s.scroll_snap = parse_scroll_snap_value(v.as_str());
                }
            }
            Rule::scroll_enabled_prop => {
                if let Some(v) = prop.into_inner().next() {
                    s.scroll_enabled = Some(v.as_str().to_string());
                }
            }
            Rule::on_scroll => {
                if let Some(v) = prop.into_inner().next() {
                    s.on_scroll = Some(v.as_str().to_string());
                }
            }
            Rule::on_scroll_end => {
                if let Some(v) = prop.into_inner().next() {
                    s.on_scroll_end = Some(v.as_str().to_string());
                }
            }
            Rule::clip_behavior_prop => {
                if let Some(v) = prop.into_inner().next() {
                    s.clip_behavior = parse_clip_behavior_value(v.as_str());
                }
            }
            Rule::fit_prop => {
                if let Some(v) = prop.into_inner().next() {
                    s.image_fit = parse_fit_value(v.as_str());
                }
            }
            Rule::generic_style_prop => {
                apply_generic_style_prop(prop, &mut s);
            }
            _ => {}
        }
    }
    s
}

fn parse_breakpoint_override(pair: Pair<Rule>) -> (String, Styles) {
    let mut inner = pair.into_inner();
    let name = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    let mut s = Styles::default();
    for prop in inner {
        if prop.as_rule() == Rule::generic_style_prop {
            apply_generic_style_prop(prop, &mut s);
        }
    }
    (name, s)
}

fn apply_generic_style_prop(prop: Pair<Rule>, s: &mut Styles) {
    let mut pi = prop.into_inner();
    let key = pi.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    let val_raw = pi.next().map(|p| {
        // could be responsive_array or style_value
        p.as_str().to_string()
    }).unwrap_or_default();
    let val = strip_quotes(&val_raw);

    match key.as_str() {
        "width"          => s.width = Some(val),
        "height"         => s.height = Some(val),
        "min_width"      => s.min_width = Some(val),
        "max_width"      => s.max_width = Some(val),
        "min_height"     => s.min_height = Some(val),
        "max_height"     => s.max_height = Some(val),
        "x"              => s.x = Some(val),
        "y"              => s.y = Some(val),
        "flex"           => s.flex = Some(val),
        "flex_wrap"      => s.flex_wrap = Some(val),
        "direction"      => s.direction = Some(val),
        "align"          => s.align = Some(val),
        "justify"        => s.justify = Some(val),
        "gap"            => s.gap = Some(val),
        "aspect_ratio"   => s.aspect_ratio = Some(val),
        "margin"         => s.margin = Some(val),
        "margin_top"     => s.margin_top = Some(val),
        "margin_bottom"  => s.margin_bottom = Some(val),
        "margin_left"    => s.margin_left = Some(val),
        "margin_right"   => s.margin_right = Some(val),
        "padding"        => s.padding = Some(val),
        "padding_top"    => s.padding_top = Some(val),
        "padding_bottom" => s.padding_bottom = Some(val),
        "padding_left"   => s.padding_left = Some(val),
        "padding_right"  => s.padding_right = Some(val),
        "background"     => s.background = Some(val),
        "color"          => s.color = Some(val),
        "font_size"      => s.font_size = Some(val),
        "font_weight"    => s.font_weight = Some(val),
        "font_family"    => s.font_family = Some(val),
        "border"         => s.border = Some(val),
        "border_radius"  => s.border_radius = Some(val),
        "opacity"        => s.opacity = Some(val),
        "visible"        => s.visible = match val.as_str() { "true" => Some(true), "false" => Some(false), _ => None },
        "safe_area"      => s.safe_area = match val.as_str() { "true" => Some(true), "false" => Some(false), _ => None },
        _ => { s.extra.insert(key, val); }
    }
}

fn parse_overflow_value(s: &str) -> OverflowValue {
    match s {
        "scroll"   => OverflowValue::Scroll,
        "scroll_x" => OverflowValue::ScrollX,
        "scroll_y" => OverflowValue::ScrollY,
        "hidden"   => OverflowValue::Hidden,
        "auto"     => OverflowValue::Auto,
        _          => OverflowValue::Visible,
    }
}

fn parse_text_overflow_value(s: &str) -> TextOverflowValue {
    match s {
        "ellipsis" => TextOverflowValue::Ellipsis,
        "fade"     => TextOverflowValue::Fade,
        _          => TextOverflowValue::Clip,
    }
}

fn parse_scroll_snap_value(s: &str) -> ScrollSnap {
    match s {
        "start"  => ScrollSnap::Start,
        "center" => ScrollSnap::Center,
        "end"    => ScrollSnap::End,
        _        => ScrollSnap::None_,
    }
}

fn parse_clip_behavior_value(s: &str) -> ClipBehavior {
    match s {
        "anti_aliased" => ClipBehavior::AntiAliased,
        "hard"         => ClipBehavior::Hard,
        _              => ClipBehavior::None_,
    }
}

fn parse_fit_value(s: &str) -> ImageFitValue {
    match s {
        "cover"      => ImageFitValue::Cover,
        "fill"       => ImageFitValue::Fill,
        "scale_down" => ImageFitValue::ScaleDown,
        "none"       => ImageFitValue::None_,
        _            => ImageFitValue::Contain,
    }
}

// ─── animate_block ────────────────────────────────────────────────────────────

fn parse_animate_block(pair: Pair<Rule>) -> Animation {
    let mut anim = Animation::default();
    for prop in pair.into_inner() {
        match prop.as_rule() {
            Rule::anim_property    => { anim.property = prop.into_inner().next().map(|p| strip_quotes(p.as_str())).unwrap_or_default(); }
            Rule::anim_from        => { anim.from = prop.into_inner().next().map(|p| p.as_str().to_string()).unwrap_or_default(); }
            Rule::anim_to          => { anim.to = prop.into_inner().next().map(|p| p.as_str().to_string()).unwrap_or_default(); }
            Rule::anim_duration    => { anim.duration_ms = prop.into_inner().next().map(|p| parse_duration_to_ms(p.as_str())).unwrap_or(0); }
            Rule::anim_delay       => { anim.delay_ms = prop.into_inner().next().map(|p| parse_duration_to_ms(p.as_str())).unwrap_or(0); }
            Rule::anim_easing      => { anim.easing = prop.into_inner().next().map(|p| parse_easing(p.as_str())).unwrap_or_default(); }
            Rule::anim_repeat      => { anim.repeat = prop.into_inner().next().map(|p| p.as_str() == "true").unwrap_or(false); }
            Rule::anim_auto_reverse => { anim.auto_reverse = prop.into_inner().next().map(|p| p.as_str() == "true").unwrap_or(false); }
            _ => {}
        }
    }
    anim
}

fn parse_easing(s: &str) -> EasingType {
    match s {
        "ease_in"     => EasingType::EaseIn,
        "ease_out"    => EasingType::EaseOut,
        "ease_in_out" => EasingType::EaseInOut,
        "bounce"      => EasingType::Bounce,
        "spring"      => EasingType::Spring,
        _             => EasingType::Linear,
    }
}

fn parse_duration_to_ms(s: &str) -> u32 {
    let s = s.trim();
    if let Some(n) = s.strip_suffix("ms") {
        n.parse().unwrap_or(0)
    } else if let Some(n) = s.strip_suffix('s') {
        (n.parse::<f32>().unwrap_or(0.0) * 1000.0) as u32
    } else {
        s.parse().unwrap_or(0)
    }
}

// ─── event handlers ──────────────────────────────────────────────────────────

fn apply_event_prop_to_event_map(pair: Pair<Rule>, events: &mut EventMap) {
    let rule = pair.as_rule();
    let expr_opt = pair.into_inner().next().map(parse_expr);
    match rule {
        Rule::on_click        => events.on_click        = expr_opt,
        Rule::on_change       => events.on_change       = expr_opt,
        Rule::on_submit       => events.on_submit       = expr_opt,
        Rule::on_select       => events.on_select       = expr_opt,
        Rule::on_touch_start  => events.on_touch_start  = expr_opt,
        Rule::on_touch_move   => events.on_touch_move   = expr_opt,
        Rule::on_touch_end    => events.on_touch_end    = expr_opt,
        Rule::on_mount        => events.on_mount        = expr_opt,
        Rule::on_update       => events.on_update       = expr_opt,
        Rule::on_unmount      => events.on_unmount      = expr_opt,
        // watch: expr — dependency key for on_update
        Rule::on_watch        => events.watch           = expr_opt,
        _ => {}
    }
}

/// Like `apply_event_prop_to_event_map` but also stores extended events as props.
fn apply_extended_event_to_props(pair: Pair<Rule>, props: &mut std::collections::HashMap<String, Expr>) {
    let key = match pair.as_rule() {
        Rule::on_scan       => Some("on_scan"),
        Rule::on_refresh    => Some("on_refresh"),
        Rule::on_increment  => Some("on_increment"),
        Rule::on_decrement  => Some("on_decrement"),
        Rule::on_complete   => Some("on_complete"),
        Rule::on_long_press => Some("on_long_press"),
        Rule::on_drag       => Some("on_drag"),
        Rule::on_swipe      => Some("on_swipe"),
        Rule::on_complete_prop => Some("on_complete"),
        Rule::on_scan_prop     => Some("on_scan"),
        _ => None,
    };
    if let Some(k) = key {
        if let Some(expr) = pair.into_inner().next().map(parse_expr) {
            props.insert(k.to_string(), expr);
        }
    }
}

// ─── parse_stmt ───────────────────────────────────────────────────────────────

fn parse_stmt(pair: Pair<Rule>, errors: &mut Vec<FrameError>, file: &str) -> Option<Stmt> {
    // stmt is a SILENT rule (_) in the grammar, so it never shows up as a pair.
    // We receive the actual statement rule pairs directly.
    match pair.as_rule() {
        Rule::var_decl => {
            Some(parse_var_decl(pair))
        }
        Rule::assign_stmt => {
            let mut pi = pair.into_inner();
            let mut parts: Vec<String> = Vec::new();
            loop {
                let token = pi.next();
                match token {
                    Some(t) if t.as_rule() == Rule::ident => {
                        parts.push(t.as_str().to_string());
                    }
                    Some(t) => {
                        let name = parts.join(".");
                        let expr = parse_expr(t);
                        return Some(Stmt::Assign(name, expr));
                    }
                    None => {
                        let name = parts.join(".");
                        return Some(Stmt::Assign(name, Expr::Literal(Value::Null)));
                    }
                }
            }
        }
        Rule::return_stmt => {
            let expr = pair.into_inner().next().map(parse_expr).unwrap_or_default();
            Some(Stmt::Return(expr))
        }
        Rule::if_stmt => {
            Some(parse_if_stmt(pair, errors, file))
        }
        Rule::for_stmt => {
            Some(parse_for_stmt(pair, errors, file))
        }
        Rule::switch_stmt => {
            Some(parse_switch_stmt(pair, errors, file))
        }
        Rule::try_catch_stmt => {
            Some(parse_try_catch_stmt(pair, errors, file))
        }
        Rule::wait_fetch_stmt => {
            let mut pi = pair.into_inner();
            let _var = pi.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let fetch = pi.next().map(parse_wait_fetch_expr).unwrap_or_default();
            Some(Stmt::WaitFetch(fetch))
        }
        Rule::wait_call_stmt => {
            let mut parts = Vec::new();
            let mut args = Vec::new();
            for child in pair.into_inner() {
                match child.as_rule() {
                    Rule::ident => parts.push(child.as_str().to_string()),
                    Rule::call_args => {
                        args = child.into_inner().map(parse_expr).collect();
                    }
                    _ => {}
                }
            }
            let func = parts.join(".");
            Some(Stmt::Wait(CallExpr { func, args, named_args: vec![] }))
        }
        Rule::call_stmt => {
            let mut pi = pair.into_inner();
            let func = pi.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let args = pi.next()
                .map(|ca| ca.into_inner().map(parse_expr).collect())
                .unwrap_or_default();
            Some(Stmt::Call(CallExpr { func, args, named_args: vec![] }))
        }
        Rule::method_call_stmt => {
            // ident ("." ident)+ "(" call_args ")"
            // e.g. log.info("msg"), list.push(item)
            let mut parts: Vec<String> = Vec::new();
            let mut args: Vec<Expr> = Vec::new();
            for child in pair.into_inner() {
                match child.as_rule() {
                    Rule::ident => parts.push(child.as_str().to_string()),
                    Rule::call_args => {
                        args = child.into_inner().map(parse_expr).collect();
                    }
                    _ => {}
                }
            }
            let func = parts.join(".");
            Some(Stmt::Call(CallExpr { func, args, named_args: vec![] }))
        }
        Rule::plugin_call => {
            Some(parse_plugin_call(pair))
        }
        // ── Navigate statements ────────────────────────────────────────────
        Rule::navigate_stmt => {
            let expr = parse_expr(pair);
            Some(Stmt::Call(CallExpr { func: "__navigate__".to_string(), args: vec![expr], named_args: vec![] }))
        }
        Rule::navigate_replace_stmt => {
            let arg = pair.into_inner().next().map(|p| strip_quotes(p.as_str())).unwrap_or_default();
            Some(Stmt::Call(CallExpr { func: "__navigate_replace__".to_string(), args: vec![Expr::Literal(Value::Str(arg))], named_args: vec![] }))
        }
        Rule::navigate_back_stmt => {
            Some(Stmt::Call(CallExpr { func: "__navigate_back__".to_string(), args: vec![], named_args: vec![] }))
        }
        Rule::navigate_back_to_stmt => {
            let arg = pair.into_inner().next().map(|p| strip_quotes(p.as_str())).unwrap_or_default();
            Some(Stmt::Call(CallExpr { func: "__navigate_back_to__".to_string(), args: vec![Expr::Literal(Value::Str(arg))], named_args: vec![] }))
        }
        Rule::navigate_modal_stmt => {
            let arg = pair.into_inner().next().map(|p| strip_quotes(p.as_str())).unwrap_or_default();
            Some(Stmt::Call(CallExpr { func: "__navigate_modal__".to_string(), args: vec![Expr::Literal(Value::Str(arg))], named_args: vec![] }))
        }
        Rule::navigate_dismiss_stmt => {
            Some(Stmt::Call(CallExpr { func: "__navigate_dismiss__".to_string(), args: vec![], named_args: vec![] }))
        }
        _ => None,
    }
}

fn parse_plugin_call(pair: Pair<Rule>) -> Stmt {
    let inner: Vec<_> = pair.into_inner().collect();
    // plugin_call = { "plugin:" ~ "{" ~ "name" ~ ":" ~ string ~ "method" ~ ":" ~ ident ~ ("params" ~ ":" ~ "{" ~ plugin_param* ~ "}")? ~ "}" }
    // children: string, ident, [plugin_param*]
    let mut i = inner.into_iter();
    let plugin_name = i.next()
        .map(|p| strip_quotes(p.as_str()))
        .unwrap_or_default();
    let method = i.next()
        .map(|p| p.as_str().to_string())
        .unwrap_or_default();
    let mut params = HashMap::new();
    for param in i {
        let mut pi = param.into_inner();
        let key = pi.next().map(|p| p.as_str().to_string()).unwrap_or_default();
        let val = pi.next().map(parse_expr).unwrap_or_default();
        params.insert(key, val);
    }
    Stmt::PluginCall(PluginCall { plugin_name, method, params })
}

fn parse_var_decl(pair: Pair<Rule>) -> Stmt {
    let mut pi = pair.into_inner();
    // var_decl = { ":var" ~ "mut"? ~ ident ~ (":" ~ type_name)? ~ ("=" ~ expr)? }
    // children: ["mut"?], ident, [type_name], [expr]
    let mut mutable = false;
    // Peek at first child — it's either "mut" keyword or the ident
    let first = pi.peek().map(|p| p.as_str());
    if first == Some("mut") {
        mutable = true;
        pi.next(); // consume "mut"
    }
    let name = pi.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    let type_ = pi.next().map(|p| parse_type_name(p.as_str()));
    let initializer = pi.next().map(parse_expr);
    Stmt::VarDecl(VarDecl { name, mutable, type_, initializer })
}

fn parse_if_stmt(pair: Pair<Rule>, errors: &mut Vec<FrameError>, file: &str) -> Stmt {
    let mut children = pair.into_inner();
    // Grammar: "if" ~ expr ~ if_then_block ~ ("else" ~ if_else_block)?
    // Children: expr, if_then_block, [if_else_block]
    let cond = children.next().map(parse_expr).unwrap_or_default();

    let then_body: Vec<Stmt> = children.next()
        .map(|b| b.into_inner().filter_map(|c| parse_stmt(c, errors, file)).collect())
        .unwrap_or_default();

    let else_body: Option<Vec<Stmt>> = children.next()
        .map(|b| b.into_inner().filter_map(|c| parse_stmt(c, errors, file)).collect());

    Stmt::If(cond, then_body, else_body)
}

fn parse_for_stmt(pair: Pair<Rule>, errors: &mut Vec<FrameError>, file: &str) -> Stmt {
    let mut pi = pair.into_inner();
    let var  = pi.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    let iter = pi.next().map(parse_expr).unwrap_or_default();
    let body = pi.filter_map(|p| parse_stmt(p, errors, file)).collect();
    Stmt::For(var, iter, body)
}

fn parse_switch_stmt(pair: Pair<Rule>, errors: &mut Vec<FrameError>, file: &str) -> Stmt {
    let mut pi = pair.into_inner();
    let discriminant = pi.next().map(parse_expr).unwrap_or_default();
    let mut cases = Vec::new();
    for case in pi {
        if case.as_rule() == Rule::switch_case {
            let mut ci = case.into_inner();
            let case_val = ci.next().map(parse_expr).unwrap_or_default();
            let case_body = ci.filter_map(|p| parse_stmt(p, errors, file)).collect();
            cases.push((case_val, case_body));
        }
    }
    Stmt::Switch(discriminant, cases)
}

fn parse_try_catch_stmt(pair: Pair<Rule>, errors: &mut Vec<FrameError>, file: &str) -> Stmt {
    // Grammar: "try" ~ try_body_block ~ "catch" ~ "(" ~ ident ~ ")" ~ catch_body_block ~ ("finally" ~ finally_body_block)?
    // Children: try_body_block, ident(catch_param), catch_body_block, [finally_body_block]
    let mut children = pair.into_inner();

    let try_body: Vec<Stmt> = children.next()
        .map(|b| b.into_inner().filter_map(|c| parse_stmt(c, errors, file)).collect())
        .unwrap_or_default();

    let catch_param = children.next()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| {
            errors.push(FrameError::parse(file, 0, 0,
                "try block is missing a required catch clause"));
            "err".to_string()
        });

    let catch_body: Vec<Stmt> = children.next()
        .map(|b| b.into_inner().filter_map(|c| parse_stmt(c, errors, file)).collect())
        .unwrap_or_default();

    let finally_body: Option<Vec<Stmt>> = children.next()
        .map(|b| b.into_inner().filter_map(|c| parse_stmt(c, errors, file)).collect());

    Stmt::TryCatch {
        body: try_body,
        catch_param,
        catch_body,
        finally_body,
    }
}

// ─── parse_expr ───────────────────────────────────────────────────────────────

fn parse_expr(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::expr => {
            // expr = { null_coalesce_expr }
            pair.into_inner().next().map(parse_expr).unwrap_or_default()
        }
        Rule::null_coalesce_expr => {
            let mut pi = pair.into_inner();
            let first = pi.next().map(parse_expr).unwrap_or_default();
            if let Some(second) = pi.next() {
                Expr::NullCoalesce(Box::new(first), Box::new(parse_expr(second)))
            } else {
                first
            }
        }
        Rule::compare_expr => {
            let mut pi = pair.into_inner();
            let mut left = pi.next().map(parse_expr).unwrap_or_default();
            while let (Some(op_text), Some(right_pair)) = (pi.next(), pi.next()) {
                let op = parse_op(op_text.as_str());
                left = Expr::BinOp(Box::new(left), op, Box::new(parse_expr(right_pair)));
            }
            left
        }
        Rule::add_expr => {
            let mut pi = pair.into_inner();
            let mut left = pi.next().map(parse_expr).unwrap_or_default();
            while let (Some(op_pair), Some(right_pair)) = (pi.next(), pi.next()) {
                let op = parse_op(op_pair.as_str());
                left = Expr::BinOp(Box::new(left), op, Box::new(parse_expr(right_pair)));
            }
            left
        }
        Rule::mul_expr => {
            let mut pi = pair.into_inner();
            let mut left = pi.next().map(parse_expr).unwrap_or_default();
            while let (Some(op_pair), Some(right_pair)) = (pi.next(), pi.next()) {
                let op = parse_op(op_pair.as_str());
                left = Expr::BinOp(Box::new(left), op, Box::new(parse_expr(right_pair)));
            }
            left
        }
        Rule::paren_expr => {
            pair.into_inner().next().map(parse_expr).unwrap_or_default()
        }
        Rule::wait_fetch_expr => parse_wait_fetch_to_expr(pair),
        Rule::wait_call_expr  => parse_wait_call_to_expr(pair),
        Rule::func_call_expr  => parse_func_call_to_expr(pair),
        Rule::method_call_expr => parse_method_call_to_expr(pair),
        Rule::safe_nav_expr   => {
            let parts: Vec<String> = pair.into_inner().map(|p| p.as_str().to_string()).collect();
            Expr::SafeNav(parts)
        }
        Rule::navigate_expr => {
            let mut inner = pair.into_inner();
            let route_pair = inner.next().unwrap();
            let route = strip_quotes(route_pair.as_str());
            // Parse nav options: replace, clear_stack, single_top, transition
            let mut opts = crate::parser::ast::NavOptions::default();
            for opt in inner {
                if opt.as_rule() == Rule::nav_option {
                    let mut oi = opt.into_inner();
                    let key = oi.next().map(|p| p.as_str()).unwrap_or("");
                    let val = oi.next().map(|p| p.as_str()).unwrap_or("false");
                    match key {
                        "replace"     => opts.replace     = val == "true",
                        "clear_stack" => opts.clear_stack = val == "true",
                        "single_top"  => opts.single_top  = val == "true",
                        "transition"  => opts.transition  = Some(strip_quotes(val)),
                        _ => {}
                    }
                }
            }
            Expr::Navigate(Box::new(Expr::Literal(Value::Str(route))), opts)
        }
        Rule::navigate_replace_expr => {
            let arg = pair.into_inner().next().map(|p| strip_quotes(p.as_str())).unwrap_or_default();
            Expr::NavigateReplace(Box::new(Expr::Literal(Value::Str(arg))))
        }
        Rule::navigate_back_expr => {
            Expr::NavigateBack
        }
        Rule::navigate_back_to_expr => {
            let arg = pair.into_inner().next().map(|p| strip_quotes(p.as_str())).unwrap_or_default();
            Expr::NavigateBackTo(Box::new(Expr::Literal(Value::Str(arg))))
        }
        Rule::navigate_modal_expr => {
            let arg = pair.into_inner().next().map(|p| strip_quotes(p.as_str())).unwrap_or_default();
            Expr::NavigateModal(Box::new(Expr::Literal(Value::Str(arg))))
        }
        Rule::navigate_dismiss_expr => {
            Expr::NavigateDismiss
        }
        Rule::lambda_expr => {
            let (params, body) = parse_lambda(pair, &mut vec![], "");
            Expr::Lambda(params, body)
        }
        Rule::state_field_expr => {
            // "state.field.nested" — drop the "state." prefix
            let full = pair.as_str();
            let field = full.strip_prefix("state.").unwrap_or(full).to_string();
            Expr::StateField(field)
        }
        Rule::store_field_expr => {
            let full = pair.as_str();
            let parts: Vec<&str> = full.splitn(2, '.').collect();
            let store = parts.get(0).map(|s| s.to_string()).unwrap_or_default();
            let field = parts.get(1).map(|s| s.to_string()).unwrap_or_default();
            Expr::StoreField(store, field)
        }
        Rule::dollar_var => {
            Expr::Var(pair.as_str().to_string())
        }
        Rule::string => {
            let raw = pair.as_str();
            let inner: Vec<_> = pair.into_inner().collect();
            if inner.is_empty() {
                Expr::Literal(Value::Str(strip_quotes(raw)))
            } else {
                let mut segments = Vec::new();
                for child in inner {
                    match child.as_rule() {
                        Rule::string_literal_char => {
                            segments.push(InterpolatedSegment::Literal(child.as_str().to_string()));
                        }
                        Rule::string_interp_paren => {
                            if let Some(expr) = child.into_inner().next() {
                                segments.push(InterpolatedSegment::Expr(Box::new(parse_expr(expr))));
                            }
                        }
                        Rule::string_interp_brace => {
                            if let Some(expr) = child.into_inner().next() {
                                segments.push(InterpolatedSegment::Expr(Box::new(parse_expr(expr))));
                            }
                        }
                        _ => {}
                    }
                }
                Expr::Interpolated(segments)
            }
        }
        Rule::float => {
            Expr::Literal(Value::Float(pair.as_str().parse().unwrap_or(0.0)))
        }
        Rule::integer => {
            Expr::Literal(Value::Int(pair.as_str().parse().unwrap_or(0)))
        }
        Rule::dimension => {
            Expr::Literal(Value::Str(pair.as_str().to_string()))
        }
        Rule::bool_lit => {
            Expr::Literal(Value::Bool(pair.as_str() == "true"))
        }
        Rule::null_lit => {
            Expr::Literal(Value::Null)
        }
        Rule::ident => {
            Expr::Var(pair.as_str().to_string())
        }
        _ => {
            // Try descending into inner
            if let Some(inner) = pair.into_inner().next() {
                parse_expr(inner)
            } else {
                Expr::default()
            }
        }
    }
}

fn parse_op(s: &str) -> Op {
    match s {
        "+"  => Op::Add,
        "-"  => Op::Sub,
        "*"  => Op::Mul,
        "/"  => Op::Div,
        "%"  => Op::Mod,
        "==" => Op::Eq,
        "!=" => Op::Ne,
        "<"  => Op::Lt,
        "<=" => Op::Le,
        ">"  => Op::Gt,
        ">=" => Op::Ge,
        "&&" => Op::And,
        "||" => Op::Or,
        _    => Op::Add,
    }
}

// ─── wait:fetch expr helpers ──────────────────────────────────────────────────

fn parse_wait_fetch_to_expr(pair: Pair<Rule>) -> Expr {
    let fetch = parse_wait_fetch_expr(pair);
    Expr::Call(CallExpr {
        func: "wait:fetch".to_string(),
        args: vec![fetch.url],
        named_args: vec![],
    })
}

fn parse_wait_fetch_expr(pair: Pair<Rule>) -> FetchExpr {
    let mut fe = FetchExpr::default();
    fe.method = "GET".to_string();
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::expr => { fe.url = parse_expr(child); }
            Rule::fetch_options => {
                for opt in child.into_inner() {
                    match opt.as_rule() {
                        Rule::fetch_method => {
                            fe.method = opt.into_inner().next()
                                .map(|p| strip_quotes(p.as_str()))
                                .unwrap_or_default();
                        }
                        Rule::fetch_timeout => {
                            fe.timeout_ms = opt.into_inner().next()
                                .and_then(|p| p.as_str().parse().ok());
                        }
                        Rule::fetch_headers => {
                            for entry in opt.into_inner() {
                                if entry.as_rule() == Rule::header_entry {
                                    let mut ei = entry.into_inner();
                                    let k = ei.next().map(|p| p.as_str().to_string()).unwrap_or_default();
                                    let v = ei.next().map(parse_expr).unwrap_or_default();
                                    fe.headers.insert(k, v);
                                }
                            }
                        }
                        Rule::fetch_body => {
                            let entries: Vec<(String, Expr)> = opt.into_inner()
                                .filter(|p| p.as_rule() == Rule::body_entry)
                                .map(|e| {
                                    let mut ei = e.into_inner();
                                    let k = ei.next().map(|p| p.as_str().to_string()).unwrap_or_default();
                                    let v = ei.next().map(parse_expr).unwrap_or_default();
                                    (k, v)
                                }).collect();
                            if !entries.is_empty() {
                                // Preserve the actual expression types, not just strings
                                let map: HashMap<String, Value> = entries.iter().map(|(k, v)| {
                                    let val = match v {
                                        Expr::Literal(lit) => lit.clone(),
                                        _ => Value::Str(String::new()), // non-literal; handled at codegen
                                    };
                                    (k.clone(), val)
                                }).collect();
                                fe.body = Some(Expr::Literal(Value::Object(map)));
                            }
                        }
                        // content_type: "..." — shorthand that inserts Content-Type header
                        Rule::fetch_content_type => {
                            let ct = opt.into_inner().next()
                                .map(|p| strip_quotes(p.as_str()))
                                .unwrap_or_else(|| "application/json".to_string());
                            fe.headers.insert(
                                "Content-Type".to_string(),
                                Expr::Literal(Value::Str(ct)),
                            );
                        }
                        _ => {}
                    }
                }
            }
            Rule::then_chain => {
                for tc in child.into_inner() {
                    if tc.as_rule() == Rule::lambda_expr {
                        let (_, body) = parse_lambda(tc, &mut vec![], "");
                        if fe.then_branch.is_empty() {
                            fe.then_branch = body;
                        } else {
                            fe.catch_branch = body;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    fe
}

fn parse_wait_call_to_expr(pair: Pair<Rule>) -> Expr {
    let mut parts = Vec::new();
    let mut args = Vec::new();
    let mut named_args = Vec::new();
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::ident => parts.push(child.as_str().to_string()),
            Rule::call_args => {
                for item in child.into_inner() {
                    if item.as_rule() == Rule::named_arg {
                        let mut ni = item.into_inner();
                        let k = ni.next().map(|p| p.as_str().to_string()).unwrap_or_default();
                        let v = ni.next().map(parse_expr).unwrap_or_default();
                        named_args.push((k, v));
                    } else {
                        args.push(parse_expr(item));
                    }
                }
            }
            _ => {}
        }
    }
    Expr::Call(CallExpr { func: format!("wait:{}", parts.join(".")), args, named_args })
}

fn parse_func_call_to_expr(pair: Pair<Rule>) -> Expr {
    let mut pi = pair.into_inner();
    let func = pi.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    let mut args = Vec::new();
    let mut named_args = Vec::new();
    if let Some(ca) = pi.next() {
        for item in ca.into_inner() {
            if item.as_rule() == Rule::named_arg {
                let mut ni = item.into_inner();
                let k = ni.next().map(|p| p.as_str().to_string()).unwrap_or_default();
                let v = ni.next().map(parse_expr).unwrap_or_default();
                named_args.push((k, v));
            } else {
                args.push(parse_expr(item));
            }
        }
    }
    Expr::Call(CallExpr { func, args, named_args })
}

fn parse_method_call_to_expr(pair: Pair<Rule>) -> Expr {
    let mut pi = pair.into_inner();
    let receiver = pi.next().map(parse_expr).unwrap_or_default();
    let method   = pi.next().map(|p| p.as_str().to_string()).unwrap_or_default();
    let args     = pi.next()
        .map(|ca| ca.into_inner().map(parse_expr).collect())
        .unwrap_or_default();
    Expr::MethodCall(Box::new(receiver), method, args)
}

fn parse_lambda(pair: Pair<Rule>, errors: &mut Vec<FrameError>, file: &str) -> (Vec<String>, Vec<Stmt>) {
    let mut params = Vec::new();
    let mut body   = Vec::new();
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::lambda_params => {
                params = child.into_inner()
                    .filter(|p| p.as_rule() == Rule::ident)
                    .map(|p| p.as_str().to_string())
                    .collect();
            }
            Rule::stmt => {
                if let Some(s) = parse_stmt(child, errors, file) { body.push(s); }
            }
            _ => {}
        }
    }
    (params, body)
}

// ─── test_suite ───────────────────────────────────────────────────────────────

fn parse_test_suite(pair: Pair<Rule>, errors: &mut Vec<FrameError>, file: &str) -> TestSuite {
    let mut suite = TestSuite::default();
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::string => {
                suite.name = strip_quotes(child.as_str());
            }
            Rule::test_case => {
                suite.cases.push(parse_test_case(child, errors, file));
            }
            _ => {}
        }
    }
    suite
}

fn parse_test_case(pair: Pair<Rule>, errors: &mut Vec<FrameError>, file: &str) -> TestCase {
    let mut tc = TestCase::default();
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::string => {
                tc.name = strip_quotes(child.as_str());
            }
            Rule::mock_block => {
                tc.mocks.push(parse_mock_block(child));
            }
            Rule::assertion => {
                tc.assertions.push(parse_assertion(child));
            }
            // test_body_stmt is silent, so we get actual stmt rule pairs directly
            Rule::wait_fetch_stmt | Rule::wait_call_stmt | Rule::method_call_stmt
            | Rule::assign_stmt | Rule::call_stmt
            | Rule::navigate_stmt | Rule::navigate_replace_stmt | Rule::navigate_back_stmt
            | Rule::navigate_back_to_stmt | Rule::navigate_modal_stmt | Rule::navigate_dismiss_stmt
            | Rule::if_stmt | Rule::for_stmt | Rule::switch_stmt
            | Rule::try_catch_stmt | Rule::return_stmt | Rule::var_decl => {
                if let Some(s) = parse_stmt(child, errors, file) { tc.body.push(s); }
            }
            _ => {}
        }
    }
    tc
}

fn parse_mock_block(pair: Pair<Rule>) -> MockConfig {
    let mut mc = MockConfig::default();
    for prop in pair.into_inner() {
        match prop.as_rule() {
            Rule::mock_url => {
                mc.url_pattern = prop.into_inner().next()
                    .map(|p| strip_quotes(p.as_str())).unwrap_or_default();
            }
            Rule::mock_status => {
                mc.status_code = prop.into_inner().next()
                    .and_then(|p| p.as_str().parse().ok()).unwrap_or(200);
            }
            Rule::mock_response => {
                // parse object_literal into a Value
                mc.response = Value::Object(HashMap::new()); // simplified
            }
            _ => {}
        }
    }
    mc
}

fn parse_assertion(pair: Pair<Rule>) -> Assertion {
    let mut assertion = Assertion::default();
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::expr => {
                assertion.expr = parse_expr(child);
            }
            Rule::matcher_to_be => {
                assertion.matcher = Matcher::ToBe;
                assertion.expected = child.into_inner().next().map(parse_expr);
            }
            Rule::matcher_to_equal => {
                assertion.matcher = Matcher::ToEqual;
                assertion.expected = child.into_inner().next().map(parse_expr);
            }
            Rule::matcher_to_contain => {
                assertion.matcher = Matcher::ToContain;
                assertion.expected = child.into_inner().next().map(parse_expr);
            }
            Rule::matcher_to_be_null  => { assertion.matcher = Matcher::ToBeNull; }
            Rule::matcher_to_be_true  => { assertion.matcher = Matcher::ToBeTrue; }
            Rule::matcher_to_be_false => { assertion.matcher = Matcher::ToBeFalse; }
            Rule::matcher_to_throw    => { assertion.matcher = Matcher::ToThrow; }
            _ => {}
        }
    }
    assertion
}

// ─── Utilities ────────────────────────────────────────────────────────────────

fn strip_quotes(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) ||
       (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len()-1].to_string()
    } else {
        s.to_string()
    }
}

fn parse_dimension_to_f32(s: &str) -> f32 {
    let s = s.trim();
    // Strip known units
    for unit in &["dp", "sp", "px", "%", "ms", "s"] {
        if let Some(n) = s.strip_suffix(unit) {
            return n.parse().unwrap_or(0.0);
        }
    }
    s.parse().unwrap_or(0.0)
}

// ─── Unit Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: parse source and expect no fatal errors
    fn parse_ok(src: &str) -> AST {
        let mut errors = Vec::new();
        let ast = parse_source(src, "test.fr", &mut errors);
        for e in &errors {
            eprintln!("Error: {}", e);
        }
        ast
    }

    // Helper: parse source and collect errors
    fn parse_errors(src: &str) -> Vec<FrameError> {
        let mut errors = Vec::new();
        parse_source(src, "test.fr", &mut errors);
        errors
    }

    // ── :vars block ──────────────────────────────────────────────────────────

    #[test]
    fn test_parse_vars_block() {
        let src = ":vars { $primary: \"#FF0000\"; $spacing: \"10dp\"; }";
        let ast = parse_ok(src);
        assert!(ast.vars.contains_key("primary"));
        assert!(ast.vars.contains_key("spacing"));
    }

    #[test]
    fn test_parse_vars_block_no_dollar() {
        let src = ":vars { primary: \"#FF0000\"; spacing: \"10dp\"; }";
        let ast = parse_ok(src);
        assert!(ast.vars.contains_key("primary"));
        assert!(ast.vars.contains_key("spacing"));
    }

    #[test]
    fn test_parse_vars_block_dimensions() {
        let src = ":vars { $gap: 8dp }";
        let ast = parse_ok(src);
        assert!(ast.vars.contains_key("gap"));
    }

    // ── :var declaration ─────────────────────────────────────────────────────

    #[test]
    fn test_var_decl_with_type_and_init() {
        let src = "fn test: () => {\n    :var x: int = 42\n    return x\n}";
        let ast = parse_ok(src);
        assert!(ast.functions.contains_key("test"));
    }

    #[test]
    fn test_var_decl_no_init() {
        let src = "fn test: () => {\n    :var name: string\n}";
        let ast = parse_ok(src);
        assert!(ast.functions.contains_key("test"));
    }

    #[test]
    fn test_var_decl_mut() {
        let src = "fn test: () => {\n    :var mut x: int = 0\n    x = 1\n}";
        let ast = parse_ok(src);
        assert!(ast.functions.contains_key("test"));
    }

    #[test]
    fn test_var_decl_inferred() {
        let src = "fn test: () => {\n    :var x = 42\n}";
        let ast = parse_ok(src);
        assert!(ast.functions.contains_key("test"));
    }

    #[test]
    fn test_var_decl_mut_inferred() {
        let src = "fn test: () => {\n    :var mut x = \"hello\"\n}";
        let ast = parse_ok(src);
        assert!(ast.functions.contains_key("test"));
    }

    // ── :i18n block ──────────────────────────────────────────────────────────

    #[test]
    fn test_parse_i18n_block() {
        let src = ":i18n { welcome: \"Welcome\"; logout: \"Log out\" }";
        let ast = parse_ok(src);
        assert_eq!(ast.i18n.get("welcome"), Some(&"Welcome".to_string()));
        assert_eq!(ast.i18n.get("logout"),  Some(&"Log out".to_string()));
    }

    // ── import_decl ──────────────────────────────────────────────────────────

    #[test]
    fn test_parse_import_single() {
        let src = "import { AppBar } \"frame-core\"";
        let ast = parse_ok(src);
        assert_eq!(ast.imports.len(), 1);
        assert_eq!(ast.imports[0].path, "frame-core");
        assert_eq!(ast.imports[0].names[0].0, "AppBar");
    }

    #[test]
    fn test_parse_import_with_alias() {
        let src = "import { Card as MyCard } \"./components/Card.fr\"";
        let ast = parse_ok(src);
        assert_eq!(ast.imports[0].names[0].0, "Card");
        assert_eq!(ast.imports[0].names[0].1, Some("MyCard".to_string()));
    }

    #[test]
    fn test_parse_import_multiple_names() {
        let src = "import { Foo, Bar } \"some-lib\"";
        let ast = parse_ok(src);
        assert_eq!(ast.imports[0].names.len(), 2);
    }

    // ── const_decl ───────────────────────────────────────────────────────────

    #[test]
    fn test_parse_const_string() {
        let src = "const author = \"john doe\"";
        let ast = parse_ok(src);
        match ast.consts.get("author") {
            Some(ConstValue::Str(s)) => assert_eq!(s, "john doe"),
            _ => panic!("expected string const"),
        }
    }

    #[test]
    fn test_parse_const_int() {
        let src = "const max_items = 10";
        let ast = parse_ok(src);
        match ast.consts.get("max_items") {
            Some(ConstValue::Int(n)) => assert_eq!(*n, 10),
            _ => panic!("expected int const"),
        }
    }

    #[test]
    fn test_parse_const_bool() {
        let src = "const debug = true";
        let ast = parse_ok(src);
        match ast.consts.get("debug") {
            Some(ConstValue::Bool(b)) => assert!(*b),
            _ => panic!("expected bool const"),
        }
    }

    // ── page_decl ────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_page_basic() {
        let src = "page: {\n  name: \"Home\"\n  route: \"/\"\n  styles: { background: \"#FFF\" }\n  children: [\n    text: { content: \"Hello\" }\n  ]\n}";
        let ast = parse_ok(src);
        assert_eq!(ast.pages.len(), 1);
        let page = &ast.pages[0];
        assert_eq!(page.name, "Home");
        assert_eq!(page.route, "/");
        assert_eq!(page.children.len(), 1);
        assert_eq!(page.children[0].kind, "text");
    }

    #[test]
    fn test_parse_page_before_enter() {
        let src = "page: {\n  name: \"Profile\"\n  route: \"/profile\"\n  before_enter: checkAuth\n}";
        let ast = parse_ok(src);
        // before_enter is now Option<Expr> — just verify it was parsed (is Some)
        assert!(ast.pages[0].before_enter.is_some());
    }

    // ── component_decl ───────────────────────────────────────────────────────

    #[test]
    fn test_parse_component_def() {
        let src = "component Card: {\n  props: { title: string\n    count: int = 0\n  }\n  children: [\n    text: { content: $title }\n  ]\n}";
        let ast = parse_ok(src);
        assert!(ast.components.contains_key("Card"));
        let card = &ast.components["Card"];
        assert!(card.props.contains_key("title"));
        assert!(card.props.contains_key("count"));
    }

    // ── fn_def ───────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_fn_sync() {
        let src = "fn greet: (name: string) => {\n  x = name\n}";
        let ast = parse_ok(src);
        assert!(ast.functions.contains_key("greet"));
        let f = &ast.functions["greet"];
        assert!(!f.is_async);
        assert_eq!(f.params.len(), 1);
        assert_eq!(f.params[0].0, "name");
    }

    #[test]
    fn test_parse_fn_async() {
        let src = "fn fetchData: async () => {\n  x = 1\n}";
        let ast = parse_ok(src);
        let f = &ast.functions["fetchData"];
        assert!(f.is_async);
    }

    #[test]
    fn test_param_default_value() {
        let src = "fn greet: (name: string = \"World\") => {\n  return name\n}";
        let ast = parse_ok(src);
        let f = &ast.functions["greet"];
        assert_eq!(f.params.len(), 1);
        assert_eq!(f.params[0].0, "name");
        assert!(f.params[0].2.is_some());
    }

    #[test]
    fn test_param_no_default() {
        let src = "fn greet: (name: string) => {\n  return name\n}";
        let ast = parse_ok(src);
        let f = &ast.functions["greet"];
        assert_eq!(f.params.len(), 1);
        assert!(f.params[0].2.is_none());
    }

    #[test]
    fn test_param_multiple_with_mixed_defaults() {
        let src = "fn f: (a: int = 1, b: string, c: float = 3.0) => {\n  return a\n}";
        let ast = parse_ok(src);
        let f = &ast.functions["f"];
        assert_eq!(f.params.len(), 3);
        assert!(f.params[0].2.is_some());
        assert!(f.params[1].2.is_none());
        assert!(f.params[2].2.is_some());
    }

    // ── string interpolation tests ───────────────────────────────────────────

    #[test]
    fn test_string_interpol_paren() {
        let src = "fn greet: (name: string) => {\n  return \"Hello \\(name)!\"\n}";
        let ast = parse_ok(src);
        let f = &ast.functions["greet"];
        assert_eq!(f.body.len(), 1);
    }

    #[test]
    fn test_string_interpol_brace() {
        let src = "fn greet: (name: string) => {\n  return \"Hello ${name}!\"\n}";
        let ast = parse_ok(src);
        let f = &ast.functions["greet"];
        assert_eq!(f.body.len(), 1);
    }

    #[test]
    fn test_string_interpol_mixed() {
        let src = "fn f: () => {\n  return \"a \\(b) c ${d} e\"\n}";
        let ast = parse_ok(src);
        let f = &ast.functions["f"];
        assert_eq!(f.body.len(), 1);
    }

    // ── named args tests ─────────────────────────────────────────────────────

    #[test]
    fn test_named_arg_single() {
        let src = "fn test: () => {\n  foo:(name: \"bar\")\n}";
        let ast = parse_ok(src);
        assert!(ast.functions.contains_key("test"));
    }

    #[test]
    fn test_named_arg_mixed() {
        let src = "fn test: () => {\n  foo:(\"pos\", name: \"bar\", count: 42)\n}";
        let ast = parse_ok(src);
        assert!(ast.functions.contains_key("test"));
    }

    #[test]
    fn test_named_arg_expr() {
        let src = "fn test: (x: int) => {\n  foo:(value: x)\n}";
        let ast = parse_ok(src);
        assert!(ast.functions.contains_key("test"));
    }

    // ── fetch headers tests ──────────────────────────────────────────────────

    #[test]
    fn test_fetch_with_headers() {
        let src = "fn fetchData: async () => {\n  x = wait:fetch(\"https://api.example.com\", { headers: { Authorization: \"Bearer token123\" } })\n}";
        let ast = parse_ok(src);
        assert!(ast.functions.contains_key("fetchData"));
    }

    #[test]
    fn test_fetch_without_headers() {
        let src = "fn fetchData: async () => {\n  x = wait:fetch(\"https://api.example.com\")\n}";
        let ast = parse_ok(src);
        assert!(ast.functions.contains_key("fetchData"));
    }

    #[test]
    fn test_fetch_header_with_var_ref() {
        let src = "fn fetchData: (token: string) => {\n  x = wait:fetch(\"https://api.example.com\", { headers: { Authorization: token } })\n}";
        let ast = parse_ok(src);
        assert!(ast.functions.contains_key("fetchData"));
    }

    // ── store_block ──────────────────────────────────────────────────────────

    #[test]
    fn test_parse_store_block() {
        let src = ":store AuthStore {\n  token: string = \"\"\n  is_logged_in: bool = false\n  persist: { token: secure }\n}";
        let ast = parse_ok(src);
        assert!(ast.stores.contains_key("AuthStore"));
        let store = &ast.stores["AuthStore"];
        assert!(store.fields.contains_key("token"));
        assert!(store.fields.contains_key("is_logged_in"));
        assert_eq!(store.persist.get("token"), Some(&PersistStrategy::Secure));
    }

    // ── styles with overflow props and breakpoint overrides ──────────────────

    #[test]
    fn test_parse_styles_overflow() {
        let src = "page: {\n  name: \"X\"\n  route: \"/x\"\n  styles: {\n    overflow: scroll\n    text_overflow: ellipsis\n    max_lines: 3\n    scroll_snap: center\n  }\n}";
        let ast = parse_ok(src);
        let styles = &ast.pages[0].styles;
        assert_eq!(styles.overflow, OverflowValue::Scroll);
        assert_eq!(styles.text_overflow, TextOverflowValue::Ellipsis);
        assert_eq!(styles.max_lines, Some(3));
        assert_eq!(styles.scroll_snap, ScrollSnap::Center);
    }

    #[test]
    fn test_parse_styles_breakpoint_override() {
        let src = "page: {\n  name: \"X\"\n  route: \"/x\"\n  styles: {\n    width: 100%\n    @md { width: 75% }\n  }\n}";
        let ast = parse_ok(src);
        let styles = &ast.pages[0].styles;
        assert!(styles.breakpoint_overrides.contains_key("md"));
    }

    // ── try/catch ────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_try_catch() {
        let src = "fn safe: () => {\n  try { x = 1 } catch (err) { x = 0 }\n}";
        let ast = parse_ok(src);
        let f = &ast.functions["safe"];
        assert_eq!(f.body.len(), 1);
        match &f.body[0] {
            Stmt::TryCatch { catch_param, .. } => {
                assert_eq!(catch_param, "err");
            }
            _ => panic!("expected TryCatch"),
        }
    }

    #[test]
    fn test_parse_try_catch_finally() {
        let src = "fn safe: () => {\n  try { x = 1 } catch (e) { x = 0 } finally { x = 2 }\n}";
        let ast = parse_ok(src);
        let f = &ast.functions["safe"];
        match &f.body[0] {
            Stmt::TryCatch { finally_body, .. } => {
                assert!(finally_body.is_some());
            }
            _ => panic!("expected TryCatch"),
        }
    }

    // ── if/else ───────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_if_else() {
        let src = "fn test: () => {\n  if true { x = 1 } else { x = 2 }\n}";
        let ast = parse_ok(src);
        let f = &ast.functions["test"];
        assert_eq!(f.body.len(), 1);
        match &f.body[0] {
            Stmt::If(_, then_body, Some(else_body)) => {
                assert_eq!(then_body.len(), 1);
                assert_eq!(else_body.len(), 1);
            }
            _ => panic!("expected If with else branch"),
        }
    }

    #[test]
    fn test_parse_if_no_else() {
        let src = "fn test: () => {\n  if true { x = 1 }\n}";
        let ast = parse_ok(src);
        let f = &ast.functions["test"];
        match &f.body[0] {
            Stmt::If(_, then_body, None) => {
                assert_eq!(then_body.len(), 1);
            }
            _ => panic!("expected If without else branch"),
        }
    }

    #[test]
    fn test_parse_if_else_multiple_stmts() {
        let src = "fn test: () => {\n  if true { x = 1  y = 2 } else { x = 3  y = 4 }\n}";
        let ast = parse_ok(src);
        let f = &ast.functions["test"];
        match &f.body[0] {
            Stmt::If(_, then_body, Some(else_body)) => {
                assert_eq!(then_body.len(), 2);
                assert_eq!(else_body.len(), 2);
            }
            _ => panic!("expected If with else branch and multiple stmts"),
        }
    }

    #[test]
    fn test_parse_try_missing_catch_emits_error() {
        // Grammar requires catch; missing it is a parse-level failure
        let src = "fn safe: () => {\n  try { x = 1 }\n}";
        let errors = parse_errors(src);
        // pest itself will reject this since catch is mandatory in the grammar
        assert!(!errors.is_empty(), "expected parse errors for missing catch");
    }

    // ── stack: alignment + positioned ────────────────────────────────────────

    #[test]
    fn test_parse_stack_alignment() {
        let src = "page: {\n  name: \"X\"\n  route: \"/x\"\n  children: [\n    stack: {\n      alignment: center\n      children: [\n        text: { content: \"hi\" }\n      ]\n    }\n  ]\n}";
        let ast = parse_ok(src);
        let stack = &ast.pages[0].children[0];
        assert_eq!(stack.kind, "stack");
        assert_eq!(stack.alignment, StackAlignment::Center);
    }

    #[test]
    fn test_parse_stack_positioned_child() {
        let src = "page: {\n  name: \"X\"\n  route: \"/x\"\n  children: [\n    stack: {\n      children: [\n        text: {\n          content: \"overlay\"\n          positioned: { top: 10dp left: 20dp }\n        }\n      ]\n    }\n  ]\n}";
        let ast = parse_ok(src);
        let stack = &ast.pages[0].children[0];
        let child = &stack.children[0];
        assert!(child.positioned.is_some());
        let pos = child.positioned.as_ref().unwrap();
        assert!(pos.top.is_some());
        assert!(pos.left.is_some());
    }

    // ── wait:fetch statement ─────────────────────────────────────────────────

    #[test]
    fn test_parse_wait_fetch_stmt() {
        let src = "fn load: async () => {\n  result = wait:fetch(\"https://api.example.com/data\", { method: \"GET\" })\n}";
        let ast = parse_ok(src);
        let f = &ast.functions["load"];
        assert_eq!(f.body.len(), 1);
        match &f.body[0] {
            Stmt::WaitFetch(fe) => {
                assert_eq!(fe.method, "GET");
            }
            _ => panic!("expected WaitFetch"),
        }
    }

    // ── test_suite (describe:) ───────────────────────────────────────────────

    #[test]
    fn test_parse_test_suite() {
        let src = "describe: \"Auth tests\" => {\n  it: \"logs in\" => {\n    expect: true .toBeTrue:()\n  }\n}";
        let ast = parse_ok(src);
        assert_eq!(ast.tests.len(), 1);
        let suite = &ast.tests[0];
        assert_eq!(suite.name, "Auth tests");
        assert_eq!(suite.cases.len(), 1);
        assert_eq!(suite.cases[0].name, "logs in");
    }

    #[test]
    fn test_parse_test_suite_with_mock() {
        let src = "describe: \"API\" => {\n  it: \"fetches posts\" => {\n    mock: { url: \"https://api.example.com/posts\" status: 200 }\n    expect: true .toBe: true\n  }\n}";
        let ast = parse_ok(src);
        assert_eq!(ast.tests[0].cases[0].mocks.len(), 1);
    }

    // ── multi-file: parse project.fr with import ─────────────────────────────

    #[test]
    fn test_parse_project_missing_entry() {
        // A non-existent directory should return Err
        let result = parse_project("/tmp/nonexistent_frame_project_xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_project_with_temp_dir() {
        use std::fs;
        let dir = std::env::temp_dir().join("frame_test_project");
        let src_dir = dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let fr_content = ":vars { $color: \"#000\" }\npage: { name: \"Home\" route: \"/\" }";
        fs::write(src_dir.join("project.fr"), fr_content).unwrap();
        let result = parse_project(dir.to_str().unwrap());
        fs::remove_dir_all(&dir).ok();
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.pages.len(), 1);
        assert_eq!(ast.pages[0].name, "Home");
    }

    #[test]
    fn test_parse_project_with_import() {
        use std::fs;
        let dir = std::env::temp_dir().join("frame_test_project_import");
        let src_dir = dir.join("src");
        let comp_dir = src_dir.join("component");
        fs::create_dir_all(&comp_dir).unwrap();

        let card_content = "component Card: {\n  props: { title: string }\n  children: [ text: { content: $title } ]\n}";
        fs::write(comp_dir.join("Card.fr"), card_content).unwrap();

        let project_content = "import { Card } \"./component/Card.fr\"\npage: { name: \"Home\" route: \"/\" children: [ text: { content: \"hi\" } ] }";
        fs::write(src_dir.join("project.fr"), project_content).unwrap();

        let result = parse_project(dir.to_str().unwrap());
        fs::remove_dir_all(&dir).ok();
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert!(ast.components.contains_key("Card"));
    }

    // ── :obj block ─────────────────────────────────────────────────────────

    #[test]
    fn test_parse_obj_block() {
        let src = ":obj User { id: string, name: string, email: string }";
        let ast = parse_ok(src);
        assert!(ast.objects.contains_key("User"));
        let obj = &ast.objects["User"];
        assert_eq!(obj.fields.len(), 3);
        assert_eq!(obj.fields[0].name, "id");
        assert_eq!(obj.fields[1].name, "name");
        assert_eq!(obj.fields[2].name, "email");
    }

    #[test]
    fn test_parse_obj_block_no_commas() {
        let src = ":obj User { id: string name: string email: string }";
        let ast = parse_ok(src);
        assert!(ast.objects.contains_key("User"));
        let obj = &ast.objects["User"];
        assert_eq!(obj.fields.len(), 3);
    }

    #[test]
    fn test_parse_obj_block_optional() {
        let src = ":obj Profile { bio: string? age: int? }";
        let ast = parse_ok(src);
        let obj = &ast.objects["Profile"];
        assert_eq!(obj.fields.len(), 2);
        assert!(obj.fields[0].optional, "first field 'bio' should be optional");
        assert!(obj.fields[1].optional, "second field 'age' should be optional");
    }

    // ── Comments ──────────────────────────────────────────────────────────────

    #[test]
    fn test_single_line_comment() {
        let src = ":vars { $primary: \"red\" }\n// this is a comment\nimport { text } \"frame-core\"";
        let ast = parse_ok(src);
        assert!(ast.vars.contains_key("primary"));
        assert_eq!(ast.imports.len(), 1);
    }

    #[test]
    fn test_multi_line_comment() {
        let src = ":vars { $primary: \"red\" }\n/* block\n   comment */\nimport { text } \"frame-core\"";
        let ast = parse_ok(src);
        assert!(ast.vars.contains_key("primary"));
        assert_eq!(ast.imports.len(), 1);
    }

    #[test]
    fn test_comment_in_fn_body() {
        let src = "fn greet: () => {\n    // say hello\n    return 1\n}";
        let ast = parse_ok(src);
        assert!(ast.functions.contains_key("greet"));
    }

    #[test]
    fn test_trailing_comment_after_stmt() {
        let src = "import { text } \"frame-core\" // importing text\npage: { name: \"Home\" route: \"/\" }";
        let ast = parse_ok(src);
        assert_eq!(ast.imports.len(), 1);
        assert_eq!(ast.pages.len(), 1);
    }

    // ── :enum tests ──────────────────────────────────────────────────────────

    #[test]
    fn test_enum_basic() {
        let src = ":enum Status { Active, Inactive, Pending }";
        let ast = parse_ok(src);
        let e = ast.enums.get("Status").unwrap();
        assert_eq!(e.variants.len(), 3);
        assert_eq!(e.variants[0].name, "Active");
        assert_eq!(e.variants[1].name, "Inactive");
        assert_eq!(e.variants[2].name, "Pending");
        assert!(e.variants[0].value.is_none());
    }

    #[test]
    fn test_enum_with_values() {
        let src = ":enum Color { Red = \"#FF0000\", Green = \"#00FF00\", Blue = \"#0000FF\" }";
        let ast = parse_ok(src);
        let e = ast.enums.get("Color").unwrap();
        assert_eq!(e.variants.len(), 3);
        assert_eq!(e.variants[0].value.as_deref(), Some("#FF0000"));
        assert_eq!(e.variants[1].value.as_deref(), Some("#00FF00"));
        assert_eq!(e.variants[2].value.as_deref(), Some("#0000FF"));
    }

    #[test]
    fn test_enum_semicolon_separators() {
        let src = ":enum Role { Admin; User; Guest; }";
        let ast = parse_ok(src);
        let e = ast.enums.get("Role").unwrap();
        assert_eq!(e.variants.len(), 3);
        assert_eq!(e.variants[0].name, "Admin");
        assert_eq!(e.variants[1].name, "User");
        assert_eq!(e.variants[2].name, "Guest");
    }

    #[test]
    fn test_enum_empty() {
        let src = ":enum Empty { }";
        let ast = parse_ok(src);
        let e = ast.enums.get("Empty").unwrap();
        assert_eq!(e.variants.len(), 0);
    }

    #[test]
    fn test_enum_with_trailing_import() {
        let src = ":enum Status { Active, Inactive }\nimport { text } \"frame-core\"";
        let ast = parse_ok(src);
        assert!(ast.enums.contains_key("Status"));
        assert_eq!(ast.imports.len(), 1);
    }

    #[test]
    fn test_enum_multiple() {
        let src = ":enum A { X, Y }\n:enum B { Z }";
        let ast = parse_ok(src);
        assert!(ast.enums.contains_key("A"));
        assert!(ast.enums.contains_key("B"));
    }

    // ── :type tests ──────────────────────────────────────────────────────────

    #[test]
    fn test_type_alias_basic() {
        let src = ":type UserId = string";
        let ast = parse_ok(src);
        let ta = ast.type_aliases.get("UserId").unwrap();
        assert_eq!(ta.alias, "UserId");
        assert_eq!(ta.target, FRType::String_);
    }

    #[test]
    fn test_type_alias_to_int() {
        let src = ":type Count = int";
        let ast = parse_ok(src);
        let ta = ast.type_aliases.get("Count").unwrap();
        assert_eq!(ta.target, FRType::Int);
    }

    #[test]
    fn test_type_alias_to_float() {
        let src = ":type Price = float";
        let ast = parse_ok(src);
        let ta = ast.type_aliases.get("Price").unwrap();
        assert_eq!(ta.target, FRType::Float);
    }

    #[test]
    fn test_type_alias_to_bool() {
        let src = ":type Flag = bool";
        let ast = parse_ok(src);
        let ta = ast.type_aliases.get("Flag").unwrap();
        assert_eq!(ta.target, FRType::Bool);
    }

    #[test]
    fn test_type_alias_to_enum_type() {
        let src = ":enum Status { Active, Inactive }\n:type MyStatus = Status";
        let ast = parse_ok(src);
        assert!(ast.type_aliases.contains_key("MyStatus"));
    }

    #[test]
    fn test_type_alias_multiple() {
        let src = ":type A = string\n:type B = int";
        let ast = parse_ok(src);
        assert!(ast.type_aliases.contains_key("A"));
        assert!(ast.type_aliases.contains_key("B"));
    }
}
