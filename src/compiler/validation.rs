//! Validation system for the Frame framework (plan §Phase 5).
//!
//! Implements:
//! - §5a  `:validation` schema blocks (grammar already added; AST/codegen here)
//! - §5b  Inline `validate:` prop on input components
//! - §5c  Plugin custom validators

use crate::parser::ast::{AST, Expr, Value};
use crate::parser::{FrameError, ErrorCategory};
use crate::plugins::PluginRegistry;
use std::collections::HashMap;
use std::path::Path;

// ─── ValidationRule ───────────────────────────────────────────────────────────

/// A single validation rule, e.g. `required`, `min(2)`, `email`.
#[derive(Debug, Clone)]
pub struct ValidationRule {
    /// Rule identifier: "required", "optional", "email", "min", "max", etc.
    pub kind: String,
    /// Arguments to the rule, e.g. `[2]` for `min(2)`.
    pub args: Vec<Expr>,
}

// ─── ValidationSchema ─────────────────────────────────────────────────────────

/// A complete `:validation SchemeName { field: rules... }` block.
#[derive(Debug, Clone, Default)]
pub struct ValidationSchema {
    pub name: String,
    /// field_name → list of rules
    pub fields: HashMap<String, Vec<ValidationRule>>,
}

// ─── Built-in rule registry ───────────────────────────────────────────────────

/// All built-in validator names (plan §5a).
pub const BUILTIN_VALIDATORS: &[&str] = &[
    "required", "optional", "email", "min", "max", "length",
    "pattern", "int", "float", "matches", "custom",
    "url", "phone", "alpha", "alphanumeric", "numeric",
];

pub fn is_builtin_validator(name: &str) -> bool {
    BUILTIN_VALIDATORS.contains(&name)
}

/// Check if a validator name is provided by any installed plugin.
pub fn is_plugin_validator(name: &str, project_root: Option<&Path>) -> bool {
    let root = match project_root {
        Some(r) => r,
        None => return false,
    };
    let registry = PluginRegistry::load(root);
    registry.validator_providers.iter().any(|p| p.validators.iter().any(|v| v == name))
}

// ─── Resolver pass ───────────────────────────────────────────────────────────

/// Validate that all `:validation` blocks and `validate:` props in the AST
/// reference known schemas, built-in rules, or plugin validators.  Returns a list of errors.
///
/// Optionally pass `project_root` to check plugin custom validators (Phase 5c).
pub fn check_validations(ast: &AST) -> Vec<FrameError> {
    let mut errors = Vec::new();

    // Check that inline validate: props on components reference built-in rules
    // (full schema references are checked below)
    let supported_components = [
        "input", "text_area", "dropdown", "date_picker", "time_picker", "otp_input",
    ];

    for page in &ast.pages {
        check_nodes_for_validate(&page.children, ast, &supported_components, &mut errors);
    }
    for comp in ast.components.values() {
        check_nodes_for_validate(&comp.children, ast, &supported_components, &mut errors);
    }

    errors
}

/// Like check_validations but also considers plugin custom validators (Phase 5c).
pub fn check_validations_with_plugins(ast: &AST, project_root: &Path) -> Vec<FrameError> {
    let mut errors = Vec::new();
    let registry = PluginRegistry::load(project_root);
    let plugin_validators: Vec<String> = registry.validator_providers.iter()
        .flat_map(|p| p.validators.iter().cloned())
        .collect();

    let supported_components = [
        "input", "text_area", "dropdown", "date_picker", "time_picker", "otp_input",
    ];

    for page in &ast.pages {
        check_nodes_for_validate_with_plugins(
            &page.children, ast, &supported_components, &plugin_validators, &mut errors,
        );
    }
    for comp in ast.components.values() {
        check_nodes_for_validate_with_plugins(
            &comp.children, ast, &supported_components, &plugin_validators, &mut errors,
        );
    }

    errors
}

fn check_nodes_for_validate(
    nodes: &[crate::parser::ast::ComponentNode],
    ast: &AST,
    supported: &[&str],
    errors: &mut Vec<FrameError>,
) {
    for node in nodes {
        if let Some(validate_expr) = node.props.get("validate") {
            if !supported.contains(&node.kind.as_str()) {
                errors.push(FrameError {
                    category: ErrorCategory::TypeMismatchError,
                    file: "<project>".to_string(),
                    line: 0, column: 0,
                    message: format!(
                        "Component '{}' does not support the 'validate:' prop. \
                         Supported: {}",
                        node.kind, supported.join(", ")
                    ),
                });
            }
            // If validate references a schema name, check it exists
            if let Expr::Var(schema_name) | Expr::Literal(Value::Str(schema_name)) = validate_expr {
                let schema_name = schema_name.trim_start_matches('$');
                if !schema_name.contains('|')
                    && !is_builtin_validator(schema_name)
                    && !ast.validations.contains_key(schema_name)
                {
                    errors.push(FrameError {
                        category: ErrorCategory::TypeMismatchError,
                        file: "<project>".to_string(),
                        line: 0, column: 0,
                        message: format!(
                            "Validation schema '{}' is not defined. \
                             Add a :validation {} {{ ... }} block.",
                            schema_name, schema_name
                        ),
                    });
                }
            }
        }
        check_nodes_for_validate(&node.children, ast, supported, errors);
    }
}

/// Like `check_nodes_for_validate` but also accepts plugin custom validators.
fn check_nodes_for_validate_with_plugins(
    nodes: &[crate::parser::ast::ComponentNode],
    ast: &AST,
    supported: &[&str],
    plugin_validators: &[String],
    errors: &mut Vec<FrameError>,
) {
    for node in nodes {
        if let Some(validate_expr) = node.props.get("validate") {
            if !supported.contains(&node.kind.as_str()) {
                errors.push(FrameError {
                    category: ErrorCategory::TypeMismatchError,
                    file: "<project>".to_string(),
                    line: 0, column: 0,
                    message: format!(
                        "Component '{}' does not support the 'validate:' prop. \
                         Supported: {}",
                        node.kind, supported.join(", ")
                    ),
                });
            }
            if let Expr::Var(schema_name) | Expr::Literal(Value::Str(schema_name)) = validate_expr {
                let schema_name = schema_name.trim_start_matches('$');
                if !schema_name.contains('|')
                    && !is_builtin_validator(schema_name)
                    && !plugin_validators.iter().any(|v| v == schema_name)
                    && !ast.validations.contains_key(schema_name)
                {
                    errors.push(FrameError {
                        category: ErrorCategory::TypeMismatchError,
                        file: "<project>".to_string(),
                        line: 0, column: 0,
                        message: format!(
                            "Validation schema '{}' is not defined. \
                             Add a :validation {} {{ ... }} block, or install a plugin \
                             that provides this validator.",
                            schema_name, schema_name
                        ),
                    });
                }
            }
        }
        check_nodes_for_validate_with_plugins(&node.children, ast, supported, plugin_validators, errors);
    }
}

// ─── Codegen helpers ──────────────────────────────────────────────────────────

/// Emit Kotlin validation extension function for a schema (plan §5a codegen).
pub fn gen_validation_kotlin(schema: &ValidationSchema, pkg: &str) -> String {
    let mut out = format!("package {pkg}\n\n");
    out.push_str(&format!("// {} — generated from :validation block. Do not edit.\n", schema.name));
    out.push_str(&format!("object {}Validator {{\n", schema.name));

    let mut sorted_fields: Vec<_> = schema.fields.iter().collect();
    sorted_fields.sort_by_key(|(k, _)| k.as_str());

    for (field, rules) in &sorted_fields {
        out.push_str(&format!("    fun validate{}(value: String): List<String> {{\n", pascal(field)));
        out.push_str("        val errors = mutableListOf<String>()\n");
        for rule in rules.iter() {
            match rule.kind.as_str() {
                "required"      => out.push_str(&format!("        if (value.isBlank()) errors.add(\"{field} is required\")\n")),
                "email"         => out.push_str(&format!("        if (!value.contains('@')) errors.add(\"{field} must be a valid email\")\n")),
                "min"           => {
                    let n = rule.args.first().map(|e| expr_to_string(e)).unwrap_or_else(|| "0".to_string());
                    out.push_str(&format!("        if (value.length < {n}) errors.add(\"{field} must be at least {n} characters\")\n"));
                }
                "max"           => {
                    let n = rule.args.first().map(|e| expr_to_string(e)).unwrap_or_else(|| "0".to_string());
                    out.push_str(&format!("        if (value.length > {n}) errors.add(\"{field} must be at most {n} characters\")\n"));
                }
                "int"           => out.push_str(&format!("        if (value.toIntOrNull() == null) errors.add(\"{field} must be a whole number\")\n")),
                "float"         => out.push_str(&format!("        if (value.toDoubleOrNull() == null) errors.add(\"{field} must be a number\")\n")),
                "url"           => out.push_str(&format!("        if (!value.startsWith(\"http\")) errors.add(\"{field} must be a valid URL\")\n")),
                "alpha"         => out.push_str(&format!("        if (!value.all {{ it.isLetter() }}) errors.add(\"{field} must contain only letters\")\n")),
                "numeric"       => out.push_str(&format!("        if (!value.all {{ it.isDigit() }}) errors.add(\"{field} must contain only digits\")\n")),
                _               => {} // custom / optional / matches — skip in base codegen
            }
        }
        out.push_str("        return errors\n    }\n\n");
    }

    out.push_str("    fun validate(data: Map<String, String>): Map<String, List<String>> {\n");
    out.push_str("        val result = mutableMapOf<String, List<String>>()\n");
    for (field, _) in &sorted_fields {
        out.push_str(&format!("        result[\"{field}\"] = validate{}(data[\"{field}\"] ?: \"\")\n", pascal(field)));
    }
    out.push_str("        return result\n    }\n}\n");
    out
}

/// Emit Swift validation struct for a schema (plan §5a codegen).
pub fn gen_validation_swift(schema: &ValidationSchema) -> String {
    let mut out = format!("import Foundation\n\n");
    out.push_str(&format!("// {} — generated from :validation block. Do not edit.\n", schema.name));
    out.push_str(&format!("struct {}Validator {{\n", schema.name));

    let mut sorted_fields: Vec<_> = schema.fields.iter().collect();
    sorted_fields.sort_by_key(|(k, _)| k.as_str());

    for (field, rules) in &sorted_fields {
        out.push_str(&format!("    static func validate{}(_ value: String) -> [String] {{\n", pascal(field)));
        out.push_str("        var errors: [String] = []\n");
        for rule in rules.iter() {
            match rule.kind.as_str() {
                "required" => out.push_str(&format!("        if value.trimmingCharacters(in: .whitespaces).isEmpty {{ errors.append(\"{field} is required\") }}\n")),
                "email"    => out.push_str(&format!("        if !value.contains(\"@\") {{ errors.append(\"{field} must be a valid email\") }}\n")),
                "min"      => {
                    let n = rule.args.first().map(|e| expr_to_string(e)).unwrap_or_else(|| "0".to_string());
                    out.push_str(&format!("        if value.count < {n} {{ errors.append(\"{field} must be at least {n} characters\") }}\n"));
                }
                "max"      => {
                    let n = rule.args.first().map(|e| expr_to_string(e)).unwrap_or_else(|| "0".to_string());
                    out.push_str(&format!("        if value.count > {n} {{ errors.append(\"{field} must be at most {n} characters\") }}\n"));
                }
                "int"      => out.push_str(&format!("        if Int(value) == nil {{ errors.append(\"{field} must be a whole number\") }}\n")),
                "float"    => out.push_str(&format!("        if Double(value) == nil {{ errors.append(\"{field} must be a number\") }}\n")),
                "url"      => out.push_str(&format!("        if !value.hasPrefix(\"http\") {{ errors.append(\"{field} must be a valid URL\") }}\n")),
                "alpha"    => out.push_str(&format!("        if !value.allSatisfy({{ $0.isLetter }}) {{ errors.append(\"{field} must contain only letters\") }}\n")),
                "numeric"  => out.push_str(&format!("        if !value.allSatisfy({{ $0.isNumber }}) {{ errors.append(\"{field} must contain only digits\") }}\n")),
                _          => {}
            }
        }
        out.push_str("        return errors\n    }\n\n");
    }

    out.push_str("    static func validate(_ data: [String: String]) -> [String: [String]] {\n");
    out.push_str("        var result: [String: [String]] = [:]\n");
    for (field, _) in &sorted_fields {
        out.push_str(&format!("        result[\"{field}\"] = validate{}(data[\"{field}\"] ?? \"\")\n", pascal(field)));
    }
    out.push_str("        return result\n    }\n}\n");
    out
}

fn expr_to_string(e: &Expr) -> String {
    match e {
        Expr::Literal(Value::Int(n))   => n.to_string(),
        Expr::Literal(Value::Float(f)) => f.to_string(),
        Expr::Literal(Value::Str(s))   => s.clone(),
        _                              => "0".to_string(),
    }
}

fn pascal(s: &str) -> String {
    s.split('_').map(|w| {
        let mut c = w.chars();
        match c.next() { None => String::new(), Some(f) => f.to_uppercase().collect::<String>() + c.as_str() }
    }).collect()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_schema(name: &str, fields: Vec<(&str, Vec<&str>)>) -> ValidationSchema {
        let mut schema = ValidationSchema { name: name.to_string(), fields: HashMap::new() };
        for (field, rules) in fields {
            let rule_list = rules.iter().map(|r| ValidationRule {
                kind: r.to_string(), args: vec![],
            }).collect();
            schema.fields.insert(field.to_string(), rule_list);
        }
        schema
    }

    #[test]
    fn is_builtin_validator_known_rules() {
        assert!(is_builtin_validator("required"));
        assert!(is_builtin_validator("email"));
        assert!(is_builtin_validator("min"));
        assert!(is_builtin_validator("max"));
        assert!(!is_builtin_validator("nonexistent_rule"));
    }

    #[test]
    fn gen_validation_kotlin_required_and_email() {
        let schema = make_schema("UserSchema", vec![
            ("email", vec!["required", "email"]),
            ("name",  vec!["required"]),
        ]);
        let code = gen_validation_kotlin(&schema, "com.example.app");
        assert!(code.contains("object UserSchemaValidator"));
        assert!(code.contains("validateEmail"));
        assert!(code.contains("validateName"));
        assert!(code.contains("isBlank()"));
        assert!(code.contains("contains('@')"));
    }

    #[test]
    fn gen_validation_swift_required_and_email() {
        let schema = make_schema("UserSchema", vec![
            ("email", vec!["required", "email"]),
        ]);
        let code = gen_validation_swift(&schema);
        assert!(code.contains("struct UserSchemaValidator"));
        assert!(code.contains("validateEmail"));
        assert!(code.contains("trimmingCharacters"));
        assert!(code.contains("contains(\"@\")"));
    }

    #[test]
    fn check_validations_unknown_schema_reference() {
        use crate::parser::ast::{AST, ComponentNode, Page};
        let mut ast = AST::default();
        let mut node = ComponentNode::default();
        node.kind = "input".to_string();
        node.props.insert("validate".to_string(),
            Expr::Var("NonExistentSchema".to_string()));
        let mut page = Page::default();
        page.name = "Test".to_string();
        page.children = vec![node];
        ast.pages.push(page);

        let errors = check_validations(&ast);
        assert!(errors.iter().any(|e| e.message.contains("NonExistentSchema")),
            "Expected error for unknown schema reference");
    }

    #[test]
    fn check_validations_builtin_rule_no_error() {
        use crate::parser::ast::{AST, ComponentNode, Page};
        let mut ast = AST::default();
        let mut node = ComponentNode::default();
        node.kind = "input".to_string();
        node.props.insert("validate".to_string(),
            Expr::Var("required".to_string()));
        let mut page = Page::default();
        page.name = "Test".to_string();
        page.children = vec![node];
        ast.pages.push(page);

        let errors = check_validations(&ast);
        assert!(errors.is_empty(), "Built-in rule should not produce error: {:?}", errors);
    }
}
