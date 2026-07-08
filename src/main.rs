use clap::Parser as ClapParser;
use frame::cli::{
    Cli, Commands, PluginCommands, scaffold_project, Architecture, run_check,
    run_build, deploy_android, deploy_ios, run_tests, run_preview,
    run_lint, LintConfig,
};
use std::path::Path;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        // ── frame start <name> ───────────────────────────────────────────────
        Commands::Start { name } => {
            println!("Select architecture:");
            println!("  1) Clean Architecture  (recommended)");
            println!("  2) MVC");
            print!("Choice [1]: ");
            let mut input = String::new();
            let _ = std::io::stdin().read_line(&mut input);
            let arch = match input.trim() {
                "2" | "mvc" | "MVC" => Architecture::Mvc,
                _                   => Architecture::CleanArchitecture,
            };
            scaffold_project(&name, arch).unwrap_or_else(|e| {
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
            PluginCommands::Add { name }    => println!("Installing plugin: {name} (Task 19)"),
            PluginCommands::Remove { name } => println!("Removing plugin: {name} (Task 19)"),
            PluginCommands::Install         => println!("Installing all plugins from frame.config.json (Task 19)"),
            PluginCommands::List            => println!("Listing installed plugins (Task 19)"),
            PluginCommands::Create { name } => println!("Scaffolding plugin: {name} (Task 19)"),
            PluginCommands::Publish         => println!("Publishing plugin (Task 19)"),
        },
    }
}
