//! Frame runtime module.
//!
//! Provides traits, types, and implementations used by compiled Frame apps
//! at runtime on the target platform.
//!
//! Full implementations: Tasks 8, 15, 16.

pub mod state;
pub mod navigation;
pub mod animation;

// canvas and plugins have heavy external deps — stubbed until Task 9+
pub mod canvas;
pub mod plugins;
pub mod store;
pub mod responsive;

pub use state::{State, I18n, Reactive};
pub use navigation::Navigation;
pub use animation::Animation;
pub use canvas::{Canvas, CanvasApp, run_app};
pub use store::{StoreRegistry, StoreSlice, StoreValue, PersistStrategy, auto_persist_strategy};
pub use responsive::{ResponsiveEngine, DEFAULT_BREAKPOINTS};
