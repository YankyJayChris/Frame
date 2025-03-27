use frame::parser::parse_project;
use frame::compiler::compile;
use frame::testing::run_tests;
use std::fs;
use std::process::Command;
use websocket::sync::Server;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("start") => {
            let name = args.get(2).unwrap_or(&"my-frame-app".to_string());
            println!("Creating new Frame app: {}", name);
            create_app_template(name).expect("Failed to create app template");
        }
        Some("build") => {
            let ast = parse_project(".").expect("Failed to parse project");
            let rust_code = compile(ast);
            fs::write("src/generated.rs", rust_code).expect("Failed to write generated code");
            Command::new("cargo").arg("build").status().expect("Failed to build");
        }
        Some("test") => {
            let ast = parse_project(".").expect("Failed to parse project");
            run_tests(&ast);
        }
        Some("deploy") => {
            let target = args.get(2).unwrap_or(&"desktop".to_string());
            let ast = parse_project(".").expect("Failed to parse project");
            let rust_code = compile(ast);
            fs::write("src/generated.rs", rust_code).expect("Failed to write generated code");
            match target.as_str() {
                "android" => {
                    Command::new("cargo").args(&["ndk", "-t", "arm64-v8a", "build", "--release"]).status().expect("Failed to build APK");
                    Command::new("adb").args(&["install", "target/aarch64-linux-android/release/my-frame-app.apk"]).status().expect("Failed to install APK");
                }
                "ios" => {
                    Command::new("cargo").args(&["ios", "build", "--release"]).status().expect("Failed to build IPA");
                    Command::new("xcodebuild").args(&["-scheme", "my-frame-app", "-destination", "generic/platform=iOS"]).status().expect("Failed to deploy IPA");
                }
                "desktop" => {
                    Command::new("cargo").arg("build").arg("--release").status().expect("Failed to build desktop binary");
                }
                _ => println!("Unsupported target: {}", target),
            }
            println!("Deployed to {}", target);
        }
        Some("preview") => {
            let mut server = Server::bind("127.0.0.1:9001").unwrap();
            for client in server.filter_map(Result::ok) {
                let ast = parse_project(".").expect("Failed to parse project");
                let rust_code = compile(ast);
                fs::write("src/generated.rs", rust_code).expect("Failed to write generated code");
                client.send_message(&websocket::Message::text("reload")).unwrap();
            }
        }
        _ => println!("Usage: frame [start|build|test|deploy|preview]"),
    }
}

fn create_app_template(name: &str) -> std::io::Result<()> {
    let app_dir = Path::new(name);
    fs::create_dir(app_dir)?;

    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
frame = {{ path = "../frame" }} # Replace with "0.1.0" for crates.io

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
"#,
        name
    );
    fs::write(app_dir.join("Cargo.toml"), cargo_toml)?;

    fs::create_dir(app_dir.join("src"))?;
    let project_fr = r#":vars {
    $primary: "#007BFF";
    $spacing: "10dp";
}

:i18n {
    app_title: "My Frame App";
}

import "components.fr"

page: {
    name: "Home"
    route: "/"
    styles: { background: "#F5F5F5" }
    children: [
        AppBar: {},
        Text: {
            content: "Welcome to Frame!"
            styles: { x: "$spacing"; y: "80dp"; font_size: "20dp"; color: "$primary" }
        }
    ]
}

fn navigate:(path:string) => {
    navigate(path)
}
"#;
    fs::write(app_dir.join("src/project.fr"), project_fr)?;

    let components_fr = r#"component AppBar: {
    styles: {
        background: "$primary";
        height: "60dp";
        width: "100%";
        x: "0dp";
        y: "0dp";
    }
    content: t:"app_title"
    animate: {
        from: { opacity: "0" }
        to: { opacity: "1" }
        duration: 500ms
        easing: "ease-in"
    }
}
"#;
    fs::write(app_dir.join("src/components.fr"), components_fr)?;

    fs::create_dir(app_dir.join("assets"))?;

    println!("Created app '{}' successfully! Run 'cd {} && frame build && cargo run' to start.", name, name);
    Ok(())
}