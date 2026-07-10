//! Compiler orchestrator for the Frame framework.
//!
//! Transforms a parsed AST into platform-specific output files.
//! Full implementation: Tasks 9–16.

use crate::parser::AST;

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

/// Compile an AST to a Rust source string (legacy desktop target).
///
/// Full Android/iOS code generation: Tasks 9–14.
pub fn compile(_ast: AST) -> String {
    String::new()
}
