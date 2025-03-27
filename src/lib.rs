pub mod parser;
pub mod compiler;
pub mod runtime;
pub mod components;
pub mod icons;
pub mod styles;
pub mod testing;

// Re-export dependencies for user convenience
pub use pest;
pub use pest_derive;
pub use wgpu;
pub use winit;
pub use serde;
pub use serde_json;
pub use tokio;
pub use reqwest;
pub use image;
pub use rusttype;
pub use resvg;
pub use rusqlite;
pub use git2;
pub use websocket;
pub use flate2;
pub use softbuffer;
pub use fontdue;
pub use lazy_static;
pub use minifier;