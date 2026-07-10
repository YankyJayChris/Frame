//! Import resolver for the Frame framework.

use crate::parser::{AST, FrameError, ErrorCategory};
use crate::parser::ast::ComponentNode;
use std::collections::{HashMap, HashSet};

pub mod types;
pub use types::type_check;

/// All built-in components provided by frame-core.
pub const BUILTIN_COMPONENTS: &[&str] = &[
    // Original 21
    "text", "button", "image", "icon", "row", "column", "container", "stack",
    "list", "input", "dropdown", "form", "app_bar", "bottom_navigation_bar",
    "scaffold", "card", "divider", "spacer", "modal", "scroll_view", "grid",
    "plugin", "item",
    // Feedback
    "toast", "tooltip", "badge", "progress_bar", "progress_circle",
    // Navigation
    "tab_bar", "tab", "bottom_sheet",
    // Inputs
    "switch", "checkbox", "radio", "slider", "stepper", "text_area", "search_bar",
    "date_picker", "time_picker", "color_picker", "rating", "otp_input",
    // Display
    "avatar", "chip", "tag", "banner", "table", "accordion", "timeline", "skeleton",
    // Media
    "video_player", "audio_player", "lottie", "web_view", "map_view",
    "camera_view", "qr_scanner",
    // Gestures
    "swipeable", "draggable", "refresh", "long_press",
];

/// Resolve imports and validate an AST.
///
/// This function:
/// 1. Checks all imports resolve to known files or frame-core builtins
/// 2. Checks for circular import dependencies
/// 3. Validates that imported identifiers exist in their source
///
/// NOTE: File-level circular dependency detection happens in the parser
/// (parse_project visits files). Here we detect circular *component* dependencies.
pub fn resolve(ast: AST) -> Result<AST, Vec<FrameError>> {
    let mut errors = Vec::new();

    // 1. Validate imports
    check_imports(&ast, &mut errors);

    // 2. Check for circular component dependencies
    check_circular_component_deps(&ast, &mut errors);

    if errors.is_empty() {
        Ok(ast)
    } else {
        Err(errors)
    }
}

fn check_imports(ast: &AST, errors: &mut Vec<FrameError>) {
    for imp in &ast.imports {
        if imp.path == "frame-core" || imp.path.starts_with("frame-") {
            // Check each named import — first in merged AST (from plugin files),
            // then against the built-in registry
            for (name, _alias) in &imp.names {
                let in_ast = ast.functions.contains_key(name)
                    || ast.components.contains_key(name)
                    || ast.objects.contains_key(name)
                    || ast.stores.contains_key(name)
                    || ast.pages.iter().any(|p| p.name == *name)
                    || ast.consts.contains_key(name);
                if in_ast {
                    continue; // resolved from plugin file
                }
                let lower = name.to_lowercase();
                let snake = pascal_to_snake(name);
                if !BUILTIN_COMPONENTS.contains(&lower.as_str())
                    && !BUILTIN_COMPONENTS.contains(&snake.as_str())
                {
                    errors.push(FrameError {
                        category: ErrorCategory::UnresolvedImportError,
                        file: "<project>".to_string(),
                        line: 0,
                        column: 0,
                        message: format!(
                            "Unresolved import: '{}' is not a built-in component in frame-core. \
                             Available components: {}",
                            name,
                            BUILTIN_COMPONENTS.join(", ")
                        ),
                    });
                }
            }
        } else if imp.path.starts_with("./") || imp.path.starts_with("../") {
            // Relative path imports: validate named identifier exists in merged AST
            for (name, _alias) in &imp.names {
                if !ast.components.contains_key(name)
                    && !ast.functions.contains_key(name)
                    && !ast.objects.contains_key(name)
                    && !ast.stores.contains_key(name)
                    && !ast.pages.iter().any(|p| p.name == *name)
                    && !ast.consts.contains_key(name)
                {
                    errors.push(FrameError {
                        category: ErrorCategory::UnresolvedImportError,
                        file: "<project>".to_string(),
                        line: 0,
                        column: 0,
                        message: format!(
                            "Unresolved import: '{}' not found in '{}'. \
                             Make sure the file exports a component, function, :obj, :store, page, or const named '{}'.",
                            name, imp.path, name
                        ),
                    });
                }
            }
        }
        // Plugin package imports (frame_maps, etc.) — validated with PluginRegistry
        else {
            // Plugin package import: check frame_modules/<name>/ exists
            let plugin_folder = std::path::PathBuf::from("frame_modules").join(&imp.path);
            // We check relative to CWD; if the caller passes a full AST with a
            // project root annotation we'd use that. For now we use a best-effort check.
            if !plugin_folder.exists() {
                errors.push(FrameError {
                    category: ErrorCategory::UnresolvedImportError,
                    file: "<project>".to_string(),
                    line: 0,
                    column: 0,
                    message: format!(
                        "Plugin '{}' is not installed. Run: frame plugin add {}",
                        imp.path, imp.path
                    ),
                });
            }
        }
    }
}

fn check_circular_component_deps(ast: &AST, errors: &mut Vec<FrameError>) {
    // Build dependency graph: component -> set of components it uses
    let mut graph: HashMap<String, HashSet<String>> = HashMap::new();
    for (name, comp) in &ast.components {
        let mut deps = HashSet::new();
        collect_component_refs_from_nodes(&comp.children, &mut deps);
        graph.insert(name.clone(), deps);
    }

    // DFS cycle detection
    let mut visited: HashSet<String> = HashSet::new();
    let mut in_stack: HashSet<String> = HashSet::new();
    let names: Vec<String> = graph.keys().cloned().collect();

    for name in &names {
        if !visited.contains(name) {
            let mut path = Vec::new();
            if dfs_detect_cycle(&graph, name, &mut visited, &mut in_stack, &mut path) {
                errors.push(FrameError {
                    category: ErrorCategory::CircularDependencyError,
                    file: "<project>".to_string(),
                    line: 0,
                    column: 0,
                    message: format!(
                        "Circular component dependency detected: {}",
                        path.join(" → ")
                    ),
                });
            }
        }
    }
}

fn dfs_detect_cycle(
    graph: &HashMap<String, HashSet<String>>,
    node: &str,
    visited: &mut HashSet<String>,
    in_stack: &mut HashSet<String>,
    path: &mut Vec<String>,
) -> bool {
    visited.insert(node.to_string());
    in_stack.insert(node.to_string());
    path.push(node.to_string());

    if let Some(deps) = graph.get(node) {
        for dep in deps {
            if !visited.contains(dep) {
                if dfs_detect_cycle(graph, dep, visited, in_stack, path) {
                    return true;
                }
            } else if in_stack.contains(dep) {
                path.push(dep.clone());
                return true;
            }
        }
    }

    in_stack.remove(node);
    path.pop();
    false
}

fn collect_component_refs_from_nodes(nodes: &[ComponentNode], deps: &mut HashSet<String>) {
    for node in nodes {
        // PascalCase kind = user-defined component reference
        if node.kind.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
            deps.insert(node.kind.clone());
        }
        collect_component_refs_from_nodes(&node.children, deps);
    }
}

fn pascal_to_snake(name: &str) -> String {
    let mut result = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_ascii_lowercase());
    }
    result
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::*;

    fn empty_ast() -> AST {
        AST::default()
    }

    // resolve() returns Ok on a valid empty AST
    #[test]
    fn test_resolve_empty_ast_ok() {
        let ast = empty_ast();
        assert!(resolve(ast).is_ok());
    }

    // resolve() returns Ok on an AST with frame-core imports of known components
    #[test]
    fn test_resolve_known_frame_core_imports_ok() {
        let mut ast = empty_ast();
        ast.imports.push(Import {
            names: vec![
                ("text".to_string(), None),
                ("button".to_string(), None),
                ("column".to_string(), None),
            ],
            path: "frame-core".to_string(),
        });
        assert!(resolve(ast).is_ok());
    }

    // resolve() returns Err for unknown frame-core import
    #[test]
    fn test_resolve_unknown_frame_core_import_err() {
        let mut ast = empty_ast();
        ast.imports.push(Import {
            names: vec![("NonExistentWidget".to_string(), None)],
            path: "frame-core".to_string(),
        });
        let result = resolve(ast);
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs.iter().any(|e| matches!(e.category, ErrorCategory::UnresolvedImportError)));
        assert!(errs.iter().any(|e| e.message.contains("NonExistentWidget")));
    }

    // resolve() detects circular component dependency
    #[test]
    fn test_resolve_circular_component_dependency() {
        let mut ast = empty_ast();

        // Component A uses B
        let mut comp_a = ComponentDef::default();
        comp_a.name = "CompA".to_string();
        comp_a.children = vec![ComponentNode {
            kind: "CompB".to_string(),
            ..Default::default()
        }];
        ast.components.insert("CompA".to_string(), comp_a);

        // Component B uses A  (circular!)
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

    // pascal_to_snake helper
    #[test]
    fn test_pascal_to_snake() {
        assert_eq!(pascal_to_snake("AppBar"), "app_bar");
        assert_eq!(pascal_to_snake("ScrollView"), "scroll_view");
        assert_eq!(pascal_to_snake("text"), "text");
        assert_eq!(pascal_to_snake("BottomNavigationBar"), "bottom_navigation_bar");
    }

    // All 21 builtins are resolvable
    #[test]
    fn test_all_builtins_resolve_ok() {
        let mut ast = empty_ast();
        ast.imports.push(Import {
            names: BUILTIN_COMPONENTS
                .iter()
                .map(|&s| (s.to_string(), None))
                .collect(),
            path: "frame-core".to_string(),
        });
        assert!(resolve(ast).is_ok());
    }
}
