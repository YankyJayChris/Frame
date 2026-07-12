//! Component registry integration for the Frame framework.
//!
//! The old `Component` trait with empty `render()` stubs has been removed per
//! plan §7d / §7e.  Component rendering is handled entirely by the codegen
//! compilers in `src/compiler/android.rs` and `src/compiler/ios.rs` via
//! `emit_composable` / `emit_uikit_view`.
//!
//! This module is kept as a re-export shim for any external code that imports
//! `frame::components`.

pub use crate::resolver::{is_builtin_component, builtin_components};
