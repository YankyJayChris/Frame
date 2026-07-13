use clap::Parser as ClapParser;
use frame::cli::{
    Cli, Commands, PluginCommands, IconCommands, scaffold_project, Architecture, run_check,
    run_build, deploy_android, deploy_ios, run_tests, run_preview,
    run_lint, LintConfig,
    plugin_add, plugin_remove, plugin_install, plugin_list, plugin_create, plugin_publish,
    run_init_examples, run_icon_add, run_icon_load_bundle,
    collect_icon_assets, generate_ios_icon_assets, generate_android_icon_assets,
    log_icon_summary, write_icon_lookup_table,
};
use frame::lsp;
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
            IconCommands::LoadBundle { path } => {
                run_icon_load_bundle(&path, Path::new(".")).unwrap_or_else(|e| {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                });
            }
            IconCommands::List => {
                log_icon_summary(Path::new("."));
            }
            IconCommands::Generate { target } => {
                let project_dir = Path::new(".");
                let _ = std::fs::create_dir_all(project_dir.join("build"));
                let icons = collect_icon_assets(project_dir);
                println!("Generating {} icon asset(s) for target: {target}", icons.len());

                match target.as_str() {
                    "ios" => {
                        let ios_dst = project_dir.join("build/ios/Assets.xcassets/Resources");
                        let written = generate_ios_icon_assets(project_dir, &ios_dst);
                        println!("  iOS: {} PDF icon(s) written to {}", written.len(), ios_dst.display());
                    }
                    "android" => {
                        let android_dst = project_dir.join("build/android/app/src/main/res/drawable");
                        let written = generate_android_icon_assets(project_dir, &android_dst);
                        println!("  Android: {} XML icon(s) written to {}", written.len(), android_dst.display());
                    }
                    "all" => {
                        let ios_dst = project_dir.join("build/ios/Assets.xcassets/Resources");
                        let ios_written = generate_ios_icon_assets(project_dir, &ios_dst);
                        println!("  iOS: {} PDF icon(s) written to {}", ios_written.len(), ios_dst.display());
                        let android_dst = project_dir.join("build/android/app/src/main/res/drawable");
                        let android_written = generate_android_icon_assets(project_dir, &android_dst);
                        println!("  Android: {} XML icon(s) written to {}", android_written.len(), android_dst.display());
                    }
                    other => {
                        eprintln!("Unknown target: {other}. Use ios, android, or all");
                        std::process::exit(1);
                    }
                }

                write_icon_lookup_table(project_dir, &project_dir.join("build")).unwrap_or_else(|e| {
                    eprintln!("Warning: could not write icon lookup table: {e}");
                });
                println!("✓ Icon generation complete");
            }
        },

        // ── frame lsp ─────────────────────────────────────────────────────────
        Commands::Lsp { workspace_root } => {
            let root = std::path::Path::new(&workspace_root);
            let root = frame::lsp::server::FrameLanguageServer::find_project_root(root)
                .unwrap_or_else(|| root.to_path_buf());
            let rt = tokio::runtime::Runtime::new().unwrap_or_else(|e| {
                eprintln!("Error creating tokio runtime: {e}");
                std::process::exit(1);
            });
            rt.block_on(async {
                lsp::run_lsp_server(root).await;
            });
        }

        // ── frame init-examples ───────────────────────────────────────────────
        Commands::InitExamples => {
            run_init_examples().unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1);
            });
        }
    }
}
