use clap::Parser as ClapParser;
use frame::cli::{
    Cli, Commands, PluginCommands, IconCommands, scaffold_project, Architecture, run_check,
    run_build, deploy_android, deploy_ios, run_tests, run_preview,
    run_lint, LintConfig,
    plugin_add, plugin_remove, plugin_install, plugin_list, plugin_create, plugin_publish,
    run_init_examples, run_icon_add,
};
use std::path::Path;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        // ── frame start <name> ───────────────────────────────────────────────
        Commands::Start { name, arch } => {
            let chosen_arch = if let Some(a) = arch {
                match a.to_lowercase().as_str() {
                    "mvc" => Architecture::Mvc,
                    "clean" | "clean-architecture" | "clean_architecture" => Architecture::CleanArchitecture,
                    other => {
                        eprintln!("Unknown architecture: {other}. Use --arch clean or --arch mvc");
                        std::process::exit(1);
                    }
                }
            } else {
                // Interactive prompt only when --arch not provided
                println!("Select architecture:");
                println!("  1) Clean Architecture  (recommended)");
                println!("  2) MVC");
                print!("Choice [1]: ");
                let mut input = String::new();
                let _ = std::io::stdin().read_line(&mut input);
                match input.trim() {
                    "2" | "mvc" | "MVC" => Architecture::Mvc,
                    _                   => Architecture::CleanArchitecture,
                }
            };
            scaffold_project(&name, chosen_arch).unwrap_or_else(|e| {
                eprintln!("Error creating project: {e}");
                std::process::exit(1);
            });
        }

        // ── frame check ──────────────────────────────────────────────────────
        Commands::Check { fix, target } => {
            let ok = run_check(&target, fix);
            if fix && !ok {
                frame::cli::check::run_fix(&target);
            }
            if !ok { std::process::exit(1); }
        }

        // ── frame build ──────────────────────────────────────────────────────
        Commands::Build { watch, strict, locale } => {
            let ok = run_build(watch, strict, locale);
            if !ok { std::process::exit(1); }
        }

        // ── frame deploy ─────────────────────────────────────────────────────
        Commands::Deploy { target } => {
            match target.as_str() {
                "android" => {
                    let ok = deploy_android(Path::new("."));
                    if !ok { std::process::exit(1); }
                }
                "ios" => {
                    let ok = deploy_ios(Path::new("."));
                    if !ok { std::process::exit(1); }
                }
                other => {
                    eprintln!("Unknown deploy target: {other}");
                    eprintln!("Use: frame deploy android  OR  frame deploy ios");
                    std::process::exit(1);
                }
            }
        }

        // ── frame test ───────────────────────────────────────────────────────
        Commands::Test { filter, coverage, pbt: _ } => {
            let ok = run_tests(filter, coverage);
            if !ok { std::process::exit(1); }
        }

        // ── frame preview ────────────────────────────────────────────────────
        Commands::Preview { port } => {
            run_preview(port);
        }

        // ── frame lint ───────────────────────────────────────────────────────
        Commands::Lint { rules, skip, strict } => {
            let only_rules = rules.map(|r| {
                r.split(',').map(|s| s.trim().to_uppercase()).collect::<Vec<_>>()
            });
            let skip_rules = skip.map(|s| {
                s.split(',').map(|r| r.trim().to_uppercase()).collect::<Vec<_>>()
            }).unwrap_or_default();
            let cfg = LintConfig { only_rules, skip_rules, strict };
            let ok = run_lint(Path::new("."), &cfg);
            if !ok { std::process::exit(1); }
        }

        // ── frame plugin ─────────────────────────────────────────────────────
        Commands::Plugin { action } => match action {
            PluginCommands::Add { name } => {
                let ok = plugin_add(&name, Path::new("."));
                if !ok { std::process::exit(1); }
            }
            PluginCommands::Remove { name } => {
                let ok = plugin_remove(&name, Path::new("."));
                if !ok { std::process::exit(1); }
            }
            PluginCommands::Install => {
                let ok = plugin_install(Path::new("."));
                if !ok { std::process::exit(1); }
            }
            PluginCommands::List => {
                let ok = plugin_list(Path::new("."));
                if !ok { std::process::exit(1); }
            }
            PluginCommands::Create { name } => {
                let ok = plugin_create(&name, Path::new("."));
                if !ok { std::process::exit(1); }
            }
            PluginCommands::Publish => {
                let ok = plugin_publish(Path::new("."));
                if !ok { std::process::exit(1); }
            }
        },

        // ── frame icon ────────────────────────────────────────────────────────
        Commands::Icon { action } => match action {
            IconCommands::Add { path, name } => {
                run_icon_add(&path, name.as_deref(), Path::new(".")).unwrap_or_else(|e| {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                });
            }
        },

        // ── frame init-examples ───────────────────────────────────────────────
        Commands::InitExamples => {
            run_init_examples().unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1);
            });
        }
    }
}
