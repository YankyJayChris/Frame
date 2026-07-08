//! Compiler orchestrator for the Frame framework.
//!
//! Transforms a parsed AST into platform-specific output files.
//! Full implementation: Tasks 9–16.

use crate::parser::AST;

/// Compile an AST to a Rust source string (legacy desktop target).
///
/// Full Android/iOS code generation: Tasks 9–14.
pub fn compile(_ast: AST) -> String {
    String::new()
}
