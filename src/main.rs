use frame_core::parser::parse_project;
use frame_core::compiler::compile;
use frame_core::testing::run_tests;
use std::fs;
use std::process::Command;
use websocket::sync::Server;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
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
        _ => println!("Usage: frame [build|test|deploy|preview]"),
    }
}