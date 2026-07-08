//! Standard library injector for the Frame framework.
//!
//! This module provides:
//! - Built-in string, number, list, object, math, date and utility functions
//! - String interpolation (`$variable` substitution)
//! - Platform-specific (Kotlin / Swift) native method mappings
//!
//! Full implementation: Task 7.

/// Inject stdlib bindings into an AST.
/// Returns the AST unchanged until Task 7 is implemented.
pub fn inject() {}

/// Emit a platform-specific call for a stdlib function.
///
/// # Arguments
/// * `call` - The stdlib function name (e.g. `"string.upper"`)
/// * `platform` - Target platform: `"android"` or `"ios"`
pub fn emit_stdlib_call(call: &str, platform: &str) -> String {
    format!("/* stdlib: {} on {} */", call, platform)
}
