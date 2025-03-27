use std::fs;
use std::path::Path;
use std::process::Command;
use notify_debouncer_mini::{new_debouncer, notify::{RecommendedWatcher, RecursiveMode, Watcher}};
use std::sync::mpsc::channel;

fn main() {
    println!("cargo:rerun-if-changed=src/");
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = new_debouncer(std::time::Duration::from_secs(1), None, move |res| {
        if let Ok(event) = res {
            if event.paths.iter().any(|p| p.extension().map_or(false, |e| e == "fr")) {
                tx.send(()).unwrap();
            }
        }
    }).unwrap();

    watcher.watch(Path::new("src"), RecursiveMode::Recursive).unwrap();

    loop {
        if rx.recv().is_ok() {
            println!("Detected change, recompiling...");
            Command::new("cargo").args(&["build"]).status().unwrap();
        }
    }
}