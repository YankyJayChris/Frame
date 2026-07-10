//! Type checker for the Frame AST.

use crate::parser::{AST, FrameError, ErrorCategory, Function, Stmt};
use crate::parser::ast::{Expr, FRType, OverflowValue, Value, ComponentNode, VarDecl};
use std::collections::HashMap;

// ─── Public entry point ───────────────────────────────────────────────────────

/// Run all type-checking passes on the AST.
/// Returns a list of errors/warnings found.
pub fn type_check(ast: &AST) -> Vec<FrameError> {
    let mut errors = Vec::new();

    check_async_wait_enforcement(ast, &mut errors);
    check_prop_types(ast, &mut errors);
    check_var_types(ast, &mut errors);
    check_overflow_list_warning(ast, &mut errors);

    errors
}

// ─── Pass 1: async/wait enforcement ──────────────────────────────────────────

fn check_async_wait_enforcement(ast: &AST, errors: &mut Vec<FrameError>) {
    // Top-level functions
    for func in ast.functions.values() {
        check_fn_async_wait(func, ast, errors);
    }
    // Store actions
    for store in ast.stores.values() {
        for action in store.actions.values() {
            check_fn_async_wait(action, ast, errors);
        }
    }
    // Component-level functions
    for comp in ast.components.values() {
        for func in comp.functions.values() {
            check_fn_async_wait(func, ast, errors);
        }
    }
}

fn check_fn_async_wait(func: &Function, ast: &AST, errors: &mut Vec<FrameError>) {
    if !func.is_async {
        // Sync function must NOT use wait: or wait:fetch
        for stmt in &func.body {
            check_stmt_for_wait_in_sync(stmt, &func.name, errors);
        }
    } else {
        // Async function: every call to another async fn must use wait:
        for stmt in &func.body {
            check_stmt_for_missing_await(stmt, func, ast, errors);
        }
    }
}

fn check_stmt_for_wait_in_sync(stmt: &Stmt, fn_name: &str, errors: &mut Vec<FrameError>) {
    match stmt {
        Stmt::Wait(_) => {
            errors.push(FrameError {
                category: ErrorCategory::TypeMismatchError,
                file: "<project>".to_string(),
                line: 0,
                column: 0,
                message: format!(
                    "wait: can only be used inside async functions (found in '{}')",
                    fn_name
                ),
            });
        }
        Stmt::WaitFetch(_) => {
            errors.push(FrameError {
                category: ErrorCategory::TypeMismatchError,
                file: "<project>".to_string(),
                line: 0,
                column: 0,
                message: format!(
                    "wait:fetch can only be used inside async functions (found in '{}')",
                    fn_name
                ),
            });
        }
        // Recurse into control-flow bodies
        Stmt::If(_, then_body, else_body) => {
            for s in then_body {
                check_stmt_for_wait_in_sync(s, fn_name, errors);
            }
            if let Some(else_stmts) = else_body {
                for s in else_stmts {
                    check_stmt_for_wait_in_sync(s, fn_name, errors);
                }
            }
        }
        Stmt::For(_, _, body) => {
            for s in body {
                check_stmt_for_wait_in_sync(s, fn_name, errors);
            }
        }
        Stmt::Switch(_, cases) => {
            for (_, case_body) in cases {
                for s in case_body {
                    check_stmt_for_wait_in_sync(s, fn_name, errors);
                }
            }
        }
        Stmt::TryCatch { body, catch_body, finally_body, .. } => {
            for s in body {
                check_stmt_for_wait_in_sync(s, fn_name, errors);
            }
            for s in catch_body {
                check_stmt_for_wait_in_sync(s, fn_name, errors);
            }
            if let Some(fb) = finally_body {
                for s in fb {
                    check_stmt_for_wait_in_sync(s, fn_name, errors);
                }
            }
        }
        _ => {}
    }
}

fn check_stmt_for_missing_await(stmt: &Stmt, caller: &Function, ast: &AST, errors: &mut Vec<FrameError>) {
    match stmt {
        Stmt::Call(call) => {
            // If the callee is an async function, it must be called with wait:
            let callee_is_async = ast
                .functions
                .get(&call.func)
                .map(|f| f.is_async)
                .unwrap_or(false);
            if callee_is_async {
                errors.push(FrameError {
                    category: ErrorCategory::TypeMismatchError,
                    file: "<project>".to_string(),
                    line: 0,
                    column: 0,
                    message: format!(
                        "Async function '{}' must be called with wait:{}() inside async function '{}'",
                        call.func, call.func, caller.name
                    ),
                });
            }
        }
        Stmt::If(_, then_body, else_body) => {
            for s in then_body {
                check_stmt_for_missing_await(s, caller, ast, errors);
            }
            if let Some(else_stmts) = else_body {
                for s in else_stmts {
                    check_stmt_for_missing_await(s, caller, ast, errors);
                }
            }
        }
        Stmt::For(_, _, body) => {
            for s in body {
                check_stmt_for_missing_await(s, caller, ast, errors);
            }
        }
        Stmt::Switch(_, cases) => {
            for (_, case_body) in cases {
                for s in case_body {
                    check_stmt_for_missing_await(s, caller, ast, errors);
                }
            }
        }
        Stmt::TryCatch { body, catch_body, finally_body, .. } => {
            for s in body {
                check_stmt_for_missing_await(s, caller, ast, errors);
            }
            for s in catch_body {
                check_stmt_for_missing_await(s, caller, ast, errors);
            }
            if let Some(fb) = finally_body {
                for s in fb {
                    check_stmt_for_missing_await(s, caller, ast, errors);
                }
            }
        }
        _ => {}
    }
}

// ─── Pass 2: prop type checking ───────────────────────────────────────────────

fn check_prop_types(ast: &AST, errors: &mut Vec<FrameError>) {
    // Check page children
    for page in &ast.pages {
        for node in &page.children {
            check_node_props(node, ast, errors);
        }
    }
    // Check component def children
    for comp in ast.components.values() {
        for node in &comp.children {
            check_node_props(node, ast, errors);
        }
    }
}

fn check_node_props(node: &ComponentNode, ast: &AST, errors: &mut Vec<FrameError>) {
    // Only user-defined components have typed prop declarations
    if let Some(comp_def) = ast.components.get(&node.kind) {
        // Missing required props
        for (prop_name, prop_def) in &comp_def.props {
            if prop_def.required && !node.props.contains_key(prop_name) {
                errors.push(FrameError {
                    category: ErrorCategory::MissingPropError,
                    file: "<project>".to_string(),
                    line: 0,
                    column: 0,
                    message: format!(
                        "Component '{}' is missing required prop '{}'",
                        node.kind, prop_name
                    ),
                });
            }
        }

        // Type mismatch for props that are present
        for (prop_name, prop_expr) in &node.props {
            if let Some(prop_def) = comp_def.props.get(prop_name) {
                check_prop_type_mismatch(
                    &node.kind,
                    prop_name,
                    &prop_def.type_,
                    prop_expr,
                    errors,
                );
            }
        }
    }

    // Recurse into children
    for child in &node.children {
        check_node_props(child, ast, errors);
    }
}

fn check_prop_type_mismatch(
    component: &str,
    prop_name: &str,
    expected: &FRType,
    expr: &Expr,
    errors: &mut Vec<FrameError>,
) {
    if let Expr::Literal(Value::Str(s)) = expr {
        match expected {
            FRType::Int => {
                if s.parse::<i64>().is_err() {
                    errors.push(FrameError {
                        category: ErrorCategory::TypeMismatchError,
                        file: "<project>".to_string(),
                        line: 0,
                        column: 0,
                        message: format!(
                            "Type mismatch: prop '{}' on '{}' expects int, got string value \"{}\"",
                            prop_name, component, s
                        ),
                    });
                }
            }
            FRType::Float => {
                if s.parse::<f64>().is_err() {
                    errors.push(FrameError {
                        category: ErrorCategory::TypeMismatchError,
                        file: "<project>".to_string(),
                        line: 0,
                        column: 0,
                        message: format!(
                            "Type mismatch: prop '{}' on '{}' expects float, got string value \"{}\"",
                            prop_name, component, s
                        ),
                    });
                }
            }
            FRType::Bool => {
                if s != "true" && s != "false" {
                    errors.push(FrameError {
                        category: ErrorCategory::TypeMismatchError,
                        file: "<project>".to_string(),
                        line: 0,
                        column: 0,
                        message: format!(
                            "Type mismatch: prop '{}' on '{}' expects bool, got string value \"{}\"",
                            prop_name, component, s
                        ),
                    });
                }
            }
            _ => {}
        }
    }
}

// ─── Pass 3: list: + overflow:hidden warning ──────────────────────────────────

fn check_overflow_list_warning(ast: &AST, errors: &mut Vec<FrameError>) {
    for page in &ast.pages {
        for node in &page.children {
            check_list_overflow_node(node, errors);
        }
    }
    for comp in ast.components.values() {
        for node in &comp.children {
            check_list_overflow_node(node, errors);
        }
    }
}

fn check_list_overflow_node(node: &ComponentNode, errors: &mut Vec<FrameError>) {
    if node.kind == "list" && node.styles.overflow == OverflowValue::Hidden {
        errors.push(FrameError {
            category: ErrorCategory::ParseError,
            file: "<project>".to_string(),
            line: 0,
            column: 0,
            message: "overflow: hidden has no effect on list: — list always scrolls. \
                      Remove the property or use overflow: scroll_y explicitly."
                .to_string(),
        });
    }
    for child in &node.children {
        check_list_overflow_node(child, errors);
    }
}

// ─── Pass 4: variable type checking ────────────────────────────────────────────

fn check_var_types(ast: &AST, errors: &mut Vec<FrameError>) {
    for func in ast.functions.values() {
        let mut scope = HashMap::new();
        check_stmts(&func.body, &mut scope, errors);
    }
    for store in ast.stores.values() {
        for action in store.actions.values() {
            let mut scope = HashMap::new();
            check_stmts(&action.body, &mut scope, errors);
        }
    }
    for comp in ast.components.values() {
        for func in comp.functions.values() {
            let mut scope = HashMap::new();
            check_stmts(&func.body, &mut scope, errors);
        }
    }
}

fn check_stmts(stmts: &[Stmt], scope: &mut HashMap<String, FRType>, errors: &mut Vec<FrameError>) {
    for stmt in stmts {
        match stmt {
            Stmt::VarDecl(vd) => {
                if let Some(init) = &vd.initializer {
                    if let Some(actual) = infer_literal_type(init, scope) {
                        if !types_compatible(&actual, &vd.type_) {
                            errors.push(FrameError {
                                category: ErrorCategory::TypeMismatchError,
                                file: "<project>".to_string(),
                                line: 0,
                                column: 0,
                                message: format!(
                                    "Type mismatch: '{}' declared as {} but initializer is {}",
                                    vd.name, frtype_name(&vd.type_), frtype_name(&actual)
                                ),
                            });
                        }
                    }
                }
                scope.insert(vd.name.clone(), vd.type_.clone());
            }
            Stmt::Assign(name, expr) => {
                if let Some(expected) = scope.get(name) {
                    if let Some(actual) = infer_literal_type(expr, scope) {
                        if !types_compatible(&actual, expected) {
                            errors.push(FrameError {
                                category: ErrorCategory::TypeMismatchError,
                                file: "<project>".to_string(),
                                line: 0,
                                column: 0,
                                message: format!(
                                    "Type mismatch: assigning {} to '{}' (declared as {})",
                                    frtype_name(&actual), name, frtype_name(expected)
                                ),
                            });
                        }
                    }
                }
            }
            Stmt::If(_, then_body, else_body) => {
                let mut then_scope = scope.clone();
                check_stmts(then_body, &mut then_scope, errors);
                if let Some(else_stmts) = else_body {
                    let mut else_scope = scope.clone();
                    check_stmts(else_stmts, &mut else_scope, errors);
                }
            }
            Stmt::For(_, _, body) => {
                check_stmts(body, scope, errors);
            }
            Stmt::Switch(_, cases) => {
                for (_, body) in cases {
                    let mut case_scope = scope.clone();
                    check_stmts(body, &mut case_scope, errors);
                }
            }
            Stmt::TryCatch { body, catch_body, finally_body, .. } => {
                check_stmts(body, scope, errors);
                let mut catch_scope = scope.clone();
                catch_scope.insert("err".to_string(), FRType::String_);
                check_stmts(catch_body, &mut catch_scope, errors);
                if let Some(fb) = finally_body {
                    check_stmts(fb, scope, errors);
                }
            }
            _ => {}
        }
    }
}

fn infer_literal_type(expr: &Expr, scope: &HashMap<String, FRType>) -> Option<FRType> {
    match expr {
        Expr::Literal(Value::Str(_)) => Some(FRType::String_),
        Expr::Literal(Value::Int(_)) => Some(FRType::Int),
        Expr::Literal(Value::Float(_)) => Some(FRType::Float),
        Expr::Literal(Value::Bool(_)) => Some(FRType::Bool),
        Expr::Literal(Value::Null) => Some(FRType::Nullable(Box::new(FRType::String_))),
        Expr::Literal(Value::List(_)) => Some(FRType::List),
        Expr::Literal(Value::Object(_)) => Some(FRType::Object),
        Expr::Var(name) => scope.get(name).cloned(),
        _ => None,
    }
}

fn types_compatible(actual: &FRType, expected: &FRType) -> bool {
    if actual == expected {
        return true;
    }
    // Null can be assigned to nullable, object, or string
    if matches!(actual, FRType::Nullable(_)) {
        return matches!(expected, FRType::Nullable(_) | FRType::Object | FRType::String_);
    }
    // Int literal can be assigned to float
    if matches!(actual, FRType::Int) && matches!(expected, FRType::Float) {
        return true;
    }
    false
}

fn frtype_name(t: &FRType) -> &'static str {
    match t {
        FRType::String_ => "string",
        FRType::Int => "int",
        FRType::Float => "float",
        FRType::Bool => "bool",
        FRType::Object => "object",
        FRType::List => "list",
        FRType::Nullable(_) => "nullable",
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::*;
    use crate::resolver::resolve;

    fn empty_ast() -> AST {
        AST::default()
    }

    // ── 1. ParseError: valid empty AST doesn't crash ──────────────────────────

    #[test]
    fn test_parse_error_category_empty_ast_no_crash() {
        let ast = empty_ast();
        let errs = type_check(&ast);
        // No errors on a completely empty AST
        assert!(errs.is_empty());
    }

    // ── 2. TypeMismatchError: int prop receives string "abc" ──────────────────

    #[test]
    fn test_type_mismatch_int_prop_with_non_numeric_string() {
        let mut ast = empty_ast();

        // Define component with an int prop
        let mut comp = ComponentDef::default();
        comp.name = "Counter".to_string();
        comp.props.insert(
            "count".to_string(),
            PropDef {
                name: "count".to_string(),
                type_: FRType::Int,
                required: false,
                default: None,
            },
        );
        ast.components.insert("Counter".to_string(), comp);

        // Page uses Counter with count="abc"
        let mut node = ComponentNode::default();
        node.kind = "Counter".to_string();
        node.props.insert(
            "count".to_string(),
            Expr::Literal(Value::Str("abc".to_string())),
        );
        let mut page = Page::default();
        page.name = "Home".to_string();
        page.children = vec![node];
        ast.pages.push(page);

        let errs = type_check(&ast);
        assert!(errs.iter().any(|e| matches!(e.category, ErrorCategory::TypeMismatchError)));
        assert!(errs.iter().any(|e| e.message.contains("count") && e.message.contains("int")));
    }

    // ── 3. UnresolvedImportError: unknown frame-core component ────────────────

    #[test]
    fn test_unresolved_import_unknown_builtin() {
        let mut ast = empty_ast();
        ast.imports.push(Import {
            names: vec![("FakeWidget".to_string(), None)],
            path: "frame-core".to_string(),
        });
        let result = resolve(ast);
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs
            .iter()
            .any(|e| matches!(e.category, ErrorCategory::UnresolvedImportError)));
        assert!(errs.iter().any(|e| e.message.contains("FakeWidget")));
    }

    // ── 4. MissingPropError: required prop not provided ───────────────────────

    #[test]
    fn test_missing_required_prop() {
        let mut ast = empty_ast();

        // Component with required prop
        let mut comp = ComponentDef::default();
        comp.name = "Avatar".to_string();
        comp.props.insert(
            "src".to_string(),
            PropDef {
                name: "src".to_string(),
                type_: FRType::String_,
                required: true,
                default: None,
            },
        );
        ast.components.insert("Avatar".to_string(), comp);

        // Page node doesn't pass "src"
        let mut node = ComponentNode::default();
        node.kind = "Avatar".to_string();
        // No props set at all
        let mut page = Page::default();
        page.name = "Home".to_string();
        page.children = vec![node];
        ast.pages.push(page);

        let errs = type_check(&ast);
        assert!(errs.iter().any(|e| matches!(e.category, ErrorCategory::MissingPropError)));
        assert!(errs.iter().any(|e| e.message.contains("Avatar") && e.message.contains("src")));
    }

    // ── 5. CircularDependencyError: A → B → A ─────────────────────────────────

    #[test]
    fn test_circular_dependency_error() {
        let mut ast = empty_ast();

        let mut comp_a = ComponentDef::default();
        comp_a.name = "CompA".to_string();
        comp_a.children = vec![ComponentNode {
            kind: "CompB".to_string(),
            ..Default::default()
        }];
        ast.components.insert("CompA".to_string(), comp_a);

        let mut comp_b = ComponentDef::default();
        comp_b.name = "CompB".to_string();
        comp_b.children = vec![ComponentNode {
            kind: "CompA".to_string(),
            ..Default::default()
        }];
        ast.components.insert("CompB".to_string(), comp_b);

        let result = resolve(ast);
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs
            .iter()
            .any(|e| matches!(e.category, ErrorCategory::CircularDependencyError)));
    }

    // ── 6. UnsupportedPlatformError: not from resolver; verify clean pass ─────

    #[test]
    fn test_resolve_clean_on_valid_ast_no_unsupported_platform_errors() {
        let mut ast = empty_ast();
        ast.imports.push(Import {
            names: vec![("text".to_string(), None), ("button".to_string(), None)],
            path: "frame-core".to_string(),
        });
        let result = resolve(ast);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
    }

    // ── 7. Async/wait enforcement ─────────────────────────────────────────────

    // Calling an async fn without wait: inside async context → TypeMismatchError
    #[test]
    fn test_async_fn_called_without_wait() {
        let mut ast = empty_ast();

        // Define async helper
        ast.functions.insert(
            "fetchData".to_string(),
            Function {
                name: "fetchData".to_string(),
                is_async: true,
                params: vec![],
                return_type: None,
                body: vec![],
            },
        );

        // Define async caller that calls fetchData via Stmt::Call (not Stmt::Wait)
        ast.functions.insert(
            "loadPage".to_string(),
            Function {
                name: "loadPage".to_string(),
                is_async: true,
                params: vec![],
                return_type: None,
                body: vec![Stmt::Call(CallExpr {
                    func: "fetchData".to_string(),
                    args: vec![],
                })],
            },
        );

        let errs = type_check(&ast);
        assert!(
            errs.iter().any(|e| matches!(e.category, ErrorCategory::TypeMismatchError)),
            "Expected TypeMismatchError for missing wait:, got: {:?}", errs
        );
        assert!(errs.iter().any(|e| e.message.contains("fetchData") && e.message.contains("wait:")));
    }

    // Using wait: inside a sync function → TypeMismatchError
    #[test]
    fn test_wait_in_sync_function_is_error() {
        let mut ast = empty_ast();
        ast.functions.insert(
            "syncFn".to_string(),
            Function {
                name: "syncFn".to_string(),
                is_async: false,
                params: vec![],
                return_type: None,
                body: vec![Stmt::Wait(CallExpr {
                    func: "someOtherFn".to_string(),
                    args: vec![],
                })],
            },
        );

        let errs = type_check(&ast);
        assert!(
            errs.iter().any(|e| matches!(e.category, ErrorCategory::TypeMismatchError)),
            "Expected TypeMismatchError for wait: in sync fn, got: {:?}", errs
        );
        assert!(errs.iter().any(|e| e.message.contains("async")));
    }

    // wait:fetch in sync function → TypeMismatchError
    #[test]
    fn test_wait_fetch_in_sync_function_is_error() {
        let mut ast = empty_ast();
        ast.functions.insert(
            "syncFn".to_string(),
            Function {
                name: "syncFn".to_string(),
                is_async: false,
                params: vec![],
                return_type: None,
                body: vec![Stmt::WaitFetch(FetchExpr {
                    url: Expr::Literal(Value::Str("https://api.example.com".to_string())),
                    method: "GET".to_string(),
                    ..Default::default()
                })],
            },
        );

        let errs = type_check(&ast);
        assert!(
            errs.iter().any(|e| matches!(e.category, ErrorCategory::TypeMismatchError)),
            "Expected TypeMismatchError for wait:fetch in sync fn, got: {:?}", errs
        );
    }

    // ── Overflow/list warning ─────────────────────────────────────────────────

    #[test]
    fn test_list_with_overflow_hidden_emits_warning() {
        let mut ast = empty_ast();
        let mut node = ComponentNode::default();
        node.kind = "list".to_string();
        node.styles.overflow = OverflowValue::Hidden;

        let mut page = Page::default();
        page.name = "Home".to_string();
        page.children = vec![node];
        ast.pages.push(page);

        let errs = type_check(&ast);
        assert!(
            errs.iter().any(|e| e.message.contains("overflow: hidden has no effect on list:")),
            "Expected list+overflow warning, got: {:?}", errs
        );
    }

    #[test]
    fn test_list_without_overflow_hidden_no_warning() {
        let mut ast = empty_ast();
        let mut node = ComponentNode::default();
        node.kind = "list".to_string();
        // Default overflow is Visible

        let mut page = Page::default();
        page.name = "Home".to_string();
        page.children = vec![node];
        ast.pages.push(page);

        let errs = type_check(&ast);
        assert!(
            !errs.iter().any(|e| e.message.contains("overflow: hidden has no effect")),
            "Unexpected list+overflow warning"
        );
    }

    // ── Float and Bool type mismatch ─────────────────────────────────────────

    #[test]
    fn test_type_mismatch_float_prop_with_non_numeric_string() {
        let mut ast = empty_ast();

        let mut comp = ComponentDef::default();
        comp.name = "Slider".to_string();
        comp.props.insert(
            "value".to_string(),
            PropDef {
                name: "value".to_string(),
                type_: FRType::Float,
                required: false,
                default: None,
            },
        );
        ast.components.insert("Slider".to_string(), comp);

        let mut node = ComponentNode::default();
        node.kind = "Slider".to_string();
        node.props.insert(
            "value".to_string(),
            Expr::Literal(Value::Str("not-a-float".to_string())),
        );
        let mut page = Page::default();
        page.name = "Home".to_string();
        page.children = vec![node];
        ast.pages.push(page);

        let errs = type_check(&ast);
        assert!(errs.iter().any(|e| matches!(e.category, ErrorCategory::TypeMismatchError)));
        assert!(errs.iter().any(|e| e.message.contains("float")));
    }

    #[test]
    fn test_type_mismatch_bool_prop_with_invalid_string() {
        let mut ast = empty_ast();

        let mut comp = ComponentDef::default();
        comp.name = "Toggle".to_string();
        comp.props.insert(
            "enabled".to_string(),
            PropDef {
                name: "enabled".to_string(),
                type_: FRType::Bool,
                required: false,
                default: None,
            },
        );
        ast.components.insert("Toggle".to_string(), comp);

        let mut node = ComponentNode::default();
        node.kind = "Toggle".to_string();
        node.props.insert(
            "enabled".to_string(),
            Expr::Literal(Value::Str("yes".to_string())), // not "true"/"false"
        );
        let mut page = Page::default();
        page.name = "Home".to_string();
        page.children = vec![node];
        ast.pages.push(page);

        let errs = type_check(&ast);
        assert!(errs.iter().any(|e| matches!(e.category, ErrorCategory::TypeMismatchError)));
        assert!(errs.iter().any(|e| e.message.contains("bool")));
    }

    // Valid "true"/"false" bool strings should NOT produce errors
    #[test]
    fn test_bool_prop_with_valid_string_no_error() {
        let mut ast = empty_ast();

        let mut comp = ComponentDef::default();
        comp.name = "Toggle".to_string();
        comp.props.insert(
            "enabled".to_string(),
            PropDef {
                name: "enabled".to_string(),
                type_: FRType::Bool,
                required: false,
                default: None,
            },
        );
        ast.components.insert("Toggle".to_string(), comp);

        let mut node = ComponentNode::default();
        node.kind = "Toggle".to_string();
        node.props.insert(
            "enabled".to_string(),
            Expr::Literal(Value::Str("true".to_string())),
        );
        let mut page = Page::default();
        page.name = "Home".to_string();
        page.children = vec![node];
        ast.pages.push(page);

        let errs = type_check(&ast);
        assert!(
            !errs.iter().any(|e| matches!(e.category, ErrorCategory::TypeMismatchError)),
            "Unexpected TypeMismatchError for valid bool string"
        );
    }

    // ── :var type checking ────────────────────────────────────────────────────

    fn make_fn(name: &str, body: Vec<Stmt>) -> Function {
        Function { name: name.to_string(), is_async: false, params: vec![], return_type: None, body }
    }

    #[test]
    fn test_var_decl_int_init_ok() {
        let mut ast = AST::default();
        ast.functions.insert("test".to_string(), make_fn("test", vec![
            Stmt::VarDecl(VarDecl { name: "x".to_string(), type_: FRType::Int, initializer: Some(Expr::Literal(Value::Int(42))) }),
        ]));
        let errs = type_check(&ast);
        assert!(errs.is_empty(), "Expected no errors, got: {:?}", errs);
    }

    #[test]
    fn test_var_decl_no_init_ok() {
        let mut ast = AST::default();
        ast.functions.insert("test".to_string(), make_fn("test", vec![
            Stmt::VarDecl(VarDecl { name: "x".to_string(), type_: FRType::String_, initializer: None }),
        ]));
        let errs = type_check(&ast);
        assert!(errs.is_empty(), "Expected no errors, got: {:?}", errs);
    }

    #[test]
    fn test_var_decl_type_mismatch_string_to_int() {
        let mut ast = AST::default();
        ast.functions.insert("test".to_string(), make_fn("test", vec![
            Stmt::VarDecl(VarDecl { name: "x".to_string(), type_: FRType::Int, initializer: Some(Expr::Literal(Value::Str("hello".to_string()))) }),
        ]));
        let errs = type_check(&ast);
        assert!(errs.iter().any(|e| e.message.contains("Type mismatch")), "Expected Type mismatch error, got: {:?}", errs);
    }

    #[test]
    fn test_var_decl_int_to_float_ok() {
        let mut ast = AST::default();
        ast.functions.insert("test".to_string(), make_fn("test", vec![
            Stmt::VarDecl(VarDecl { name: "x".to_string(), type_: FRType::Float, initializer: Some(Expr::Literal(Value::Int(42))) }),
        ]));
        let errs = type_check(&ast);
        assert!(errs.is_empty(), "Expected no errors (int→float widening), got: {:?}", errs);
    }

    #[test]
    fn test_assign_type_mismatch_after_decl() {
        let mut ast = AST::default();
        ast.functions.insert("test".to_string(), make_fn("test", vec![
            Stmt::VarDecl(VarDecl { name: "x".to_string(), type_: FRType::Int, initializer: None }),
            Stmt::Assign("x".to_string(), Expr::Literal(Value::Str("bad".to_string()))),
        ]));
        let errs = type_check(&ast);
        assert!(errs.iter().any(|e| e.message.contains("Type mismatch")), "Expected Type mismatch error, got: {:?}", errs);
    }

    #[test]
    fn test_assign_same_type_ok() {
        let mut ast = AST::default();
        ast.functions.insert("test".to_string(), make_fn("test", vec![
            Stmt::VarDecl(VarDecl { name: "x".to_string(), type_: FRType::Int, initializer: None }),
            Stmt::Assign("x".to_string(), Expr::Literal(Value::Int(99))),
        ]));
        let errs = type_check(&ast);
        assert!(errs.is_empty(), "Expected no errors, got: {:?}", errs);
    }
}
