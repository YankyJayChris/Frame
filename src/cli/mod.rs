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

pub mod start;
pub mod check;
pub mod build;
pub mod deploy;
pub mod test_runner;
pub mod preview;
pub mod lint;
pub mod plugin;
pub mod icon;
pub mod font;

pub use start::{scaffold_project, scaffold_project_in, Architecture, run_init_examples};
pub use check::run_check;
pub use build::run_build;
pub use deploy::{deploy_android, deploy_ios};
pub use test_runner::run_tests;
pub use preview::run_preview;
pub use lint::{run_lint, LintConfig};
pub use plugin::{plugin_add, plugin_remove, plugin_install, plugin_list, plugin_create, plugin_publish};
pub use icon::{run_icon_add, IconManifest};

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
        /// Architecture pattern: `clean` (default) or `mvc`. Skips the interactive prompt.
        #[arg(long, value_name = "ARCH")]
        arch: Option<String>,
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

    /// Analyze `.fr` source files for style, naming, and best-practice issues.
    Lint {
        /// Only run specific rule IDs, comma-separated (e.g. FR001,FR010).
        #[arg(long)]
        rules: Option<String>,

        /// Skip specific rule IDs, comma-separated (e.g. FR042,FR043).
        #[arg(long)]
        skip: Option<String>,

        /// Treat warnings as errors (non-zero exit if any warnings found).
        #[arg(long)]
        strict: bool,
    },

    /// Manage Frame plugins.
    Plugin {
        #[command(subcommand)]
        action: PluginCommands,
    },

    /// Check the development environment (like Flutter Doctor).
    /// Verifies all required tools are installed for building Frame apps.
    Check {
        /// Automatically install missing tools where possible.
        #[arg(long)]
        fix: bool,

        /// Target platform to check: `android`, `ios`, or `all` (default).
        #[arg(long, default_value = "all")]
        target: String,
    },

    /// Regenerate example projects (examples/blog-app MVC + examples/profile Clean Architecture).
    InitExamples,

    /// Manage icons in the project.
    Icon {
        #[command(subcommand)]
        action: IconCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum IconCommands {
    /// Add an SVG icon to the project.
    Add {
        /// Path to the SVG file.
        path: String,
        /// Optional name for the icon (defaults to filename without extension).
        #[arg(long)]
        name: Option<String>,
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
