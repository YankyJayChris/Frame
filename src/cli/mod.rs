//! CLI command definitions for the Frame framework.
//!
//! All commands are implemented via `clap` with the `derive` feature.
//! Full command implementations live in `src/main.rs`.
//!
//! Commands (full implementations: Tasks 17 & 18):
//!   frame start <name>       — scaffold a new Frame project
//!   frame build              — compile `.fr` files, with optional --watch / --strict
//!   frame deploy android     — write generated Android project to build/android/
//!   frame deploy ios         — write generated iOS project to build/ios/
//!   frame test               — run *.test.fr test suites
//!   frame preview            — start hot-reload WebSocket dev server

use clap::{Parser, Subcommand};

/// The Frame framework CLI.
#[derive(Parser, Debug)]
#[command(name = "frame", version, about = "Frame — cross-platform mobile framework")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scaffold a new Frame project.
    Start {
        /// Project name / directory to create.
        name: String,
    },

    /// Compile the current Frame project.
    Build {
        /// Rebuild on file changes.
        #[arg(long)]
        watch: bool,

        /// Treat warnings as errors.
        #[arg(long)]
        strict: bool,

        /// Compile for a specific locale.
        #[arg(long)]
        locale: Option<String>,
    },

    /// Deploy the compiled project to a target platform.
    Deploy {
        /// Target platform: `android` or `ios`.
        target: String,
    },

    /// Run `.test.fr` test suites.
    Test {
        /// Only run tests whose name matches this filter.
        #[arg(long)]
        filter: Option<String>,

        /// Report test coverage.
        #[arg(long)]
        coverage: bool,

        /// Run property-based tests.
        #[arg(long)]
        pbt: bool,
    },

    /// Start a hot-reload WebSocket preview server.
    Preview {
        /// Port to listen on (default: 9001).
        #[arg(long, default_value_t = 9001)]
        port: u16,
    },

    /// Manage Frame plugins.
    Plugin {
        #[command(subcommand)]
        action: PluginCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum PluginCommands {
    /// Install a plugin from the Frame Plugin Registry.
    Add { name: String },
    /// Remove an installed plugin.
    Remove { name: String },
    /// Install all plugins listed in frame.config.json.
    Install,
    /// List installed plugins and their versions.
    List,
    /// Scaffold a new plugin package.
    Create { name: String },
    /// Publish a plugin to the Frame Plugin Registry.
    Publish,
}
