use clap::Parser as ClapParser;
use frame::cli::{Cli, Commands, PluginCommands};
use frame::parser::parse_project;
use frame::compiler::compile;
use std::fs;
use std::process::Command;
use std::path::Path;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { name } => {
            println!("Creating new Frame app: {}", name);
            create_app_template(&name).expect("Failed to create app template");
        }

        Commands::Build { watch: _, strict: _, locale: _ } => {
            let ast = parse_project(".").unwrap_or_else(|errs| {
                for e in &errs { eprintln!("{}", e); }
                panic!("Failed to parse project ({} error(s))", errs.len());
            });
            let generated = compile(ast);
            fs::write("src/generated.rs", generated).expect("Failed to write generated code");
            Command::new("cargo")
                .arg("build")
                .status()
                .expect("Failed to build");
        }

        Commands::Deploy { target } => {
            let ast = parse_project(".").unwrap_or_else(|errs| {
                for e in &errs { eprintln!("{}", e); }
                panic!("Failed to parse project ({} error(s))", errs.len());
            });
            let generated = compile(ast);
            fs::write("src/generated.rs", generated).expect("Failed to write generated code");
            match target.as_str() {
                "android" => {
                    Command::new("cargo")
                        .args(["ndk", "-t", "arm64-v8a", "build", "--release"])
                        .status()
                        .expect("Failed to build APK");
                }
                "ios" => {
                    Command::new("xcodebuild")
                        .args(["-scheme", "frame", "-destination", "generic/platform=iOS"])
                        .status()
                        .expect("Failed to deploy IPA");
                }
                _ => println!("Unsupported target: {}", target),
            }
            println!("Deployed to {}", target);
        }

        Commands::Test { filter: _, coverage: _, pbt: _ } => {
            // Full test runner implementation: Task 18
            println!("frame test — full implementation coming in Task 18");
        }

        Commands::Preview { port } => {
            // Full hot-reload server implementation: Task 18
            println!("frame preview — listening on :{} (full implementation in Task 18)", port);
        }

        Commands::Plugin { action } => match action {
            PluginCommands::Add { name } => println!("plugin add {} — Task 19", name),
            PluginCommands::Remove { name } => println!("plugin remove {} — Task 19", name),
            PluginCommands::Install => println!("plugin install — Task 19"),
            PluginCommands::List => println!("plugin list — Task 19"),
            PluginCommands::Create { name } => println!("plugin create {} — Task 19", name),
            PluginCommands::Publish => println!("plugin publish — Task 19"),
        },
    }
}

fn create_app_template(name: &str) -> std::io::Result<()> {
    let app_dir = Path::new(name);
    fs::create_dir_all(app_dir)?;

    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
frame = {{ path = "../" }}
"#,
        name
    );
    fs::write(app_dir.join("Cargo.toml"), cargo_toml)?;

    fs::create_dir_all(app_dir.join("src"))?;

    let project_fr = concat!(
        ":vars {\n",
        "    $primary: \"#007BFF\";\n",
        "    $spacing: \"10dp\";\n",
        "}\n",
        "\n",
        ":i18n {\n",
        "    app_title: \"My Frame App\";\n",
        "}\n",
        "\n",
        "page: {\n",
        "    name: \"Home\"\n",
        "    route: \"/\"\n",
        "    styles: { background: \"#F5F5F5\" }\n",
        "    children: [\n",
        "        text: {\n",
        "            content: \"Welcome to Frame!\"\n",
        "            styles: { font_size: \"20dp\"; color: \"$primary\" }\n",
        "        }\n",
        "    ]\n",
        "}\n"
    );
    fs::write(app_dir.join("src/project.fr"), project_fr)?;
    fs::create_dir_all(app_dir.join("assets"))?;

    println!(
        "Created '{}'. Run: cd {} && frame build",
        name, name
    );
    Ok(())
}
