//! Import resolver and type checker for the Frame framework.
//!
//! This module is responsible for:
//! - Resolving import paths across `.fr` files
//! - Detecting circular dependencies
//! - Type-checking AST nodes

pub mod types;

/// Placeholder for the resolved AST and error types.
/// Full implementation is handled in Task 6.
pub struct FrameError {
    pub message: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
}

/// Resolve imports and validate the AST.
/// Returns the resolved AST or a list of errors.
///
/// Full implementation: Task 6.
pub fn resolve() -> Result<(), Vec<FrameError>> {
    Ok(())
}
