//! Import resolver for the Frame framework.

use crate::parser::{AST, FrameError, ErrorCategory};
use crate::parser::ast::ComponentNode;
use crate::compiler::component_registry;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub mod types;
pub use types::type_check;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct FrameConfig {
    pub plugins: Option<HashMap<String, String>>,
    pub paths: Option<HashMap<String, String>>,
}

pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start.to_path_buf());
    while let Some(ref dir) = current {
        if dir.join("frame.config.json").exists() {
            return Some(dir.clone());
        }
        current = dir.parent().map(|p| p.to_path_buf());
    }
    None
}

pub fn load_frame_config(project_root: &Path) -> FrameConfig {
    let config_path = project_root.join("frame.config.json");
    if config_path.exists() {
        std::fs::read_to_string(config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        FrameConfig::default()
    }
}

pub fn resolve_import_path(import_path: &str, current_file: &Path, config: &FrameConfig, project_root: &Path) -> PathBuf {
    if import_path.starts_with("@/") {
        let alias_root = config.paths.as_ref()
            .and_then(|p| p.get("@"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("./src"));
        project_root.join(alias_root).join(&import_path[2..]).with_extension("fr")
    } else if import_path.starts_with("./") || import_path.starts_with("../") {
        let parent = current_file.parent().unwrap_or(Path::new("."));
        parent.join(import_path).with_extension("fr")
    } else if import_path.starts_with("frame-") {
        project_root.join("frame_modules").join(import_path).with_extension("fr")
    } else {
        PathBuf::from(import_path)
    }
}

/// Returns true if `name` is a known built-in component.
pub fn is_builtin_component(name: &str) -> bool {
    component_registry().contains(name)
}

/// All built-in component names.
pub fn builtin_components() -> Vec<&'static str> {
    component_registry().names()
}

/// Resolve imports and validate an AST.
///
/// This function:
/// 1. Checks all imports resolve to known files or frame-core builtins
/// 2. Checks for circular import dependencies
/// 3. Validates that imported identifiers exist in their source
///
/// NOTE: File-level circular dependency detection happens in the parser
/// (parse_project visits files). Here we detect circular *component* dependencies.
pub fn resolve(ast: AST, project_root: &Path) -> Result<AST, Vec<FrameError>> {
    let config = load_frame_config(project_root);
    let mut errors = Vec::new();

    // 0. Validate :app {} block — required in project.fr
    check_app_block(&ast, &mut errors);

    // 1. Validate imports
    check_imports(&ast, &mut errors, project_root, &config);

    // 2. Check for circular component dependencies
    check_circular_component_deps(&ast, &mut errors);

    if errors.is_empty() {
        Ok(ast)
    } else {
        Err(errors)
    }
}

/// Validate that the project has a :app {} block with a default_route.
/// This is required in every project.fr — without it the app has no entry point.
/// Skipped for minimal/test ASTs that have no pages.
fn check_app_block(ast: &AST, errors: &mut Vec<FrameError>) {
    // Skip for empty/test ASTs — only enforce when pages are declared
    if ast.pages.is_empty() {
        return;
    }

    // :app {} not declared at all
    if ast.default_route.is_none() && ast.on_launch.is_none()
        && ast.on_foreground.is_none() && ast.on_background.is_none()
    {
        errors.push(FrameError {
            category: ErrorCategory::ParseError,
            file: "src/project.fr".to_string(),
            line: 0,
            column: 0,
            message: concat!(
                "Missing required :app {} block in project.fr.\n",
                "\n",
                "  Every Frame project must declare :app {} with a default_route.\n",
                "  Add this to src/project.fr:\n",
                "\n",
                "    :app {\n",
                "        default_route: \"/\"      // route of the first screen\n",
                "        on_launch:     appInit  // optional\n",
                "    }\n",
                "\n",
                "  The default_route must match the route of one of your page: declarations."
            ).to_string(),
        });
        return;
    }

    // :app {} declared but default_route missing
    if ast.default_route.is_none() {
        errors.push(FrameError {
            category: ErrorCategory::ParseError,
            file: "src/project.fr".to_string(),
            line: 0,
            column: 0,
            message: concat!(
                "Missing required default_route in :app {} block.\n",
                "\n",
                "  Add default_route to your :app {} block in src/project.fr:\n",
                "\n",
                "    :app {\n",
                "        default_route: \"/\"      // route of the first screen\n",
                "    }\n",
                "\n",
                "  The default_route must match the route of one of your page: declarations."
            ).to_string(),
        });
        return;
    }

    // default_route declared — verify it matches an existing page route
    let declared = ast.default_route.as_deref().unwrap_or("/");
    let route_exists = ast.pages.iter().any(|p| p.route == declared);
    if !route_exists {
        let known: Vec<&str> = ast.pages.iter().map(|p| p.route.as_str()).collect();
        errors.push(FrameError {
            category: ErrorCategory::ParseError,
            file: "src/project.fr".to_string(),
            line: 0,
            column: 0,
            message: format!(
                "default_route \"{}\" does not match any declared page route.\n\nKnown routes: {}\n\nUpdate default_route in :app {{}} to one of these.",
                declared,
                known.join(", ")
            ),
        });
    }
}

fn check_imports(ast: &AST, errors: &mut Vec<FrameError>, project_root: &Path, config: &FrameConfig) {
    for imp in &ast.imports {
        if imp.path == "frame-core" {
            // Built-in — check each named import against the built-in registry
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
                if !is_builtin_component(&lower) && !is_builtin_component(&snake) {
                    let avail = component_registry().names().join(", ");
                    errors.push(FrameError {
                        category: ErrorCategory::UnresolvedImportError,
                        file: "<project>".to_string(),
                        line: 0,
                        column: 0,
                        message: format!(
                            "Unresolved import: '{}' is not a built-in component in frame-core. \
                             Available components: {avail}",
                            name,
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
        } else if imp.path.starts_with("@/") {
            // @/ alias import: resolve path and validate file exists
            let resolved = resolve_import_path(&imp.path, Path::new(""), config, project_root);
            if !resolved.exists() {
                errors.push(FrameError {
                    category: ErrorCategory::UnresolvedImportError,
                    file: "<project>".to_string(),
                    line: 0,
                    column: 0,
                    message: format!(
                        "Unresolved import: '@/' alias path '{}' does not resolve to an existing file. \
                         Expected: {}",
                        imp.path, resolved.display()
                    ),
                });
            }
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
                            "Unresolved import: '{}' not found in '@/' path '{}'.",
                            name, imp.path
                        ),
                    });
                }
            }
        } else if imp.path.starts_with("frame-") {
            // Plugin import: check frame_modules/<name>/ exists
            let plugin_folder = project_root.join("frame_modules").join(&imp.path);
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
            for (name, _alias) in &imp.names {
                let in_ast = ast.functions.contains_key(name)
                    || ast.components.contains_key(name)
                    || ast.objects.contains_key(name)
                    || ast.stores.contains_key(name)
                    || ast.pages.iter().any(|p| p.name == *name)
                    || ast.consts.contains_key(name);
                let lower = name.to_lowercase();
                let snake = pascal_to_snake(name);
                if !in_ast && !is_builtin_component(&lower) && !is_builtin_component(&snake) {
                    let avail = component_registry().names().join(", ");
                    errors.push(FrameError {
                        category: ErrorCategory::UnresolvedImportError,
                        file: "<project>".to_string(),
                        line: 0,
                        column: 0,
                        message: format!(
                            "Unresolved import: '{}' is not exported by plugin '{}'. \
                             Available built-in components: {avail}",
                            name, imp.path
                        ),
                    });
                }
            }
        }
        // Other plugin packages (frame_maps, etc.) — validated with PluginRegistry
        else {
            let plugin_folder = project_root.join("frame_modules").join(&imp.path);
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

    fn project_root() -> &'static Path {
        Path::new(".")
    }

    // resolve() returns Ok on a valid empty AST
    #[test]
    fn test_resolve_empty_ast_ok() {
        let ast = empty_ast();
        assert!(resolve(ast, project_root()).is_ok());
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
        assert!(resolve(ast, project_root()).is_ok());
    }

    // resolve() returns Err for unknown frame-core import
    #[test]
    fn test_resolve_unknown_frame_core_import_err() {
        let mut ast = empty_ast();
        ast.imports.push(Import {
            names: vec![("NonExistentWidget".to_string(), None)],
            path: "frame-core".to_string(),
        });
        let result = resolve(ast, project_root());
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

        let result = resolve(ast, project_root());
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

    // All builtins are resolvable
    #[test]
    fn test_all_builtins_resolve_ok() {
        let mut ast = empty_ast();
        let names: Vec<(String, Option<String>)> = component_registry()
            .names()
            .into_iter()
            .map(|s| (s.to_string(), None))
            .collect();
        ast.imports.push(Import {
            names,
            path: "frame-core".to_string(),
        });
        assert!(resolve(ast, project_root()).is_ok());
    }
}
