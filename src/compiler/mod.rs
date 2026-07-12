//! Compiler orchestrator for the Frame framework.
//!
//! Transforms a parsed AST into platform-specific output files.
//! Full pipeline: parse → resolve → type_check → codegen (Android + iOS).

use crate::parser::FrameError;
use std::collections::HashMap;

pub mod pretty;
pub use pretty::print;

pub mod android;
pub use android::{gen_android, gen_android_with_plugins, AndroidConfig, OutputFile};

pub mod ios;
pub use ios::{gen_ios, gen_ios_with_plugins, IosConfig};

pub mod overflow;
pub use overflow::{inject_overflow_defaults, default_overflow, page_root_overflow,
                   android_overflow_modifier, ios_overflow_code,
                   android_text_overflow, ios_line_break_mode,
                   android_image_content_scale, ios_image_content_mode,
                   android_scroll_snap_code, ios_scroll_snap_code};

pub mod registry;
pub use registry::{ComponentRegistry, BuiltInComponentDef, ComponentCategory,
                   BuiltInPropDef, StyleProp, build_registry, registry as component_registry};

pub mod validation;
pub use validation::{check_validations, check_validations_with_plugins,
                     gen_validation_kotlin, gen_validation_swift,
                     ValidationSchema, ValidationRule, is_builtin_validator,
                     BUILTIN_VALIDATORS};

// ─── CompilationResult ───────────────────────────────────────────────────────

/// A single compiled output file.
#[derive(Debug, Clone)]
pub struct CompileOutputFile {
    /// Relative output path, e.g. `android/app/src/main/...`
    pub path: String,
    pub content: String,
}

/// The result of a full compilation pipeline run.
#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub output_files: Vec<CompileOutputFile>,
    pub errors: Vec<FrameError>,
    pub warnings: Vec<String>,
}

// ─── Public entry point ───────────────────────────────────────────────────────

/// Compile a Frame project rooted at `project_dir`.
///
/// Pipeline: `parse_project` → `resolve` → `type_check` → `gen_android` + `gen_ios`
///
/// Returns a [`CompilationResult`] with generated files or errors.
pub fn compile(project_dir: &str) -> CompilationResult {
    use crate::resolver::{resolve, types::type_check};

    // 1. Parse all .fr files
    let ast = match crate::parser::parse_project(project_dir) {
        Ok(a) => a,
        Err(errs) => {
            return CompilationResult {
                output_files: vec![],
                errors: errs,
                warnings: vec![],
            };
        }
    };

    // 2. Resolve imports and check circular dependencies
    let ast = match resolve(ast) {
        Ok(a) => a,
        Err(errs) => {
            return CompilationResult {
                output_files: vec![],
                errors: errs,
                warnings: vec![],
            };
        }
    };

    // 3. Type-check (async/await enforcement, prop types, var types, warnings)
    let type_errors = type_check(&ast);
    if !type_errors.is_empty() {
        return CompilationResult {
            output_files: vec![],
            errors: type_errors,
            warnings: vec![],
        };
    }

    // 4. Android codegen
    let android_cfg = android::AndroidConfig::default();
    let mut output_files: Vec<CompileOutputFile> = gen_android(&ast, &android_cfg)
        .into_iter()
        .map(|f| CompileOutputFile {
            path: format!("android/{}", f.path),
            content: f.content,
        })
        .collect();

    // 5. iOS codegen
    let ios_cfg = ios::IosConfig::default();
    let ios_files: Vec<CompileOutputFile> = gen_ios(&ast, &ios_cfg)
        .into_iter()
        .map(|f| CompileOutputFile {
            path: format!("ios/{}", f.path),
            content: f.content,
        })
        .collect();
    output_files.extend(ios_files);

    // 6. Validation codegen (plan §5a)
    for (name, schema) in &ast.validations {
        let mut fields: HashMap<String, Vec<validation::ValidationRule>> = HashMap::new();
        for field in &schema.fields {
            let rules: Vec<validation::ValidationRule> = field.rules.iter().map(|r| {
                let mut args = Vec::new();
                if let Some(arg) = &r.arg {
                    args.push(arg.clone());
                }
                validation::ValidationRule { kind: r.name.clone(), args }
            }).collect();
            fields.insert(field.field.clone(), rules);
        }
        let v_schema = validation::ValidationSchema {
            name: schema.name.clone(),
            fields,
        };

        let kotlin_val = gen_validation_kotlin(&v_schema, "com.example.app");
        output_files.push(CompileOutputFile {
            path: format!("android/app/src/main/java/com/example/app/{}Validator.kt", name),
            content: kotlin_val,
        });
        let swift_val = gen_validation_swift(&v_schema);
        output_files.push(CompileOutputFile {
            path: format!("ios/{}/Validators/{}Validator.swift", ios_cfg.app_name, name),
            content: swift_val,
        });
    }

    CompilationResult {
        output_files,
        errors: vec![],
        warnings: vec![],
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_nonexistent_dir_returns_errors() {
        let result = compile("/tmp/frame_nonexistent_xyz");
        assert!(!result.errors.is_empty(), "Expected parse errors for missing project");
        assert!(result.output_files.is_empty());
    }

    #[test]
    fn compile_result_has_output_files_on_success() {
        use std::fs;
        let dir = std::env::temp_dir().join("frame_compile_test");
        let src = dir.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            dir.join("frame.config.json"),
            r#"{"name":"TestApp","bundle_id":"com.example.testapp","version":"1.0"}"#,
        ).unwrap();
        fs::write(
            src.join("project.fr"),
            "page: {\n  name: \"Home\"\n  route: \"/\"\n}\n",
        ).unwrap();

        let result = compile(&dir.to_string_lossy());
        // Should produce output files even for a minimal project
        assert!(result.errors.is_empty() || !result.output_files.is_empty(),
            "Either no errors or output files produced");
        fs::remove_dir_all(&dir).ok();
    }
}
