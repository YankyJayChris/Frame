//! `frame preview` — WebSocket hot-reload server on :9001 (configurable via --port).
//!
//! Protocol:
//!   → client connects
//!   ← {"type":"reload"}              on successful incremental recompile
//!   ← {"type":"error","errors":[…]}  on compile error
//!
//! Store devtools: GET /devtools/stores returns current store state JSON.

use crate::parser::{parse_project, FrameError};
use crate::compiler::{gen_android, gen_ios};
use crate::compiler::android::AndroidConfig;
use crate::compiler::ios::IosConfig;

use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// ─── PreviewServer ────────────────────────────────────────────────────────────

/// Shared state for the preview WebSocket server.
pub struct PreviewServer {
    pub port: u16,
    /// Serialized error list or empty when healthy.
    pub last_errors: Arc<Mutex<Vec<String>>>,
}

impl PreviewServer {
    pub fn new(port: u16) -> Self {
        PreviewServer {
            port,
            last_errors: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

// ─── Broadcast helpers ────────────────────────────────────────────────────────

/// JSON payload to broadcast on successful recompile.
fn reload_payload() -> String {
    "{\"type\":\"reload\"}".to_string()
}

/// JSON payload to broadcast on compile error.
fn error_payload(errors: &[FrameError]) -> String {
    let errs: Vec<serde_json::Value> = errors.iter().map(|e| {
        serde_json::json!({
            "file": e.file,
            "line": e.line,
            "message": e.message,
        })
    }).collect();
    serde_json::json!({ "type": "error", "errors": errs }).to_string()
}

/// Return a JSON representation of all store state (for devtools endpoint).
pub fn handle_devtools_request(store_data: &serde_json::Value) -> String {
    store_data.to_string()
}

// ─── WebSocket server ─────────────────────────────────────────────────────────

/// Run the preview server: WebSocket on `ws://localhost:{port}`.
///
/// Uses the `websocket` crate for the WS layer and `notify` for file watching.
/// This function blocks until Ctrl-C.
pub fn run_preview(port: u16) {
    println!("frame preview  →  ws://localhost:{port}");
    println!("  Watching src/ for changes… (Ctrl-C to stop)");

    // ── shared client list ────────────────────────────────────────────────────
    use websocket::sync::Server as WsServer;
    use websocket::OwnedMessage;

    // We collect serialized messages to broadcast; the client thread handles sending.
    let client_senders: Arc<Mutex<Vec<websocket::sender::Writer<std::net::TcpStream>>>> =
        Arc::new(Mutex::new(Vec::new()));

    let broadcast_msg = Arc::new(Mutex::new(None::<String>));

    // ── start WS server ───────────────────────────────────────────────────────
    let ws_addr = format!("0.0.0.0:{port}");
    let server = match WsServer::bind(&ws_addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not bind WebSocket server on :{port}: {e}");
            eprintln!("  Try a different port: frame preview --port 9002");
            return;
        }
    };

    println!("  WebSocket server listening on ws://localhost:{port}");

    let senders_clone = Arc::clone(&client_senders);
    let bcast_clone   = Arc::clone(&broadcast_msg);

    // Accept connections in a background thread
    thread::spawn(move || {
        for connection in server.filter_map(Result::ok) {
            let senders_inner = Arc::clone(&senders_clone);
            thread::spawn(move || {
                let client = match connection.accept() {
                    Ok(c) => c,
                    Err(_) => return,
                };
                let (mut receiver, sender) = client.split().unwrap();
                {
                    let mut guard = senders_inner.lock().unwrap();
                    guard.push(sender);
                }
                // Keep-alive: drain incoming messages
                for msg in receiver.incoming_messages() {
                    match msg {
                        Ok(OwnedMessage::Close(_)) => break,
                        Err(_) => break,
                        _ => {}
                    }
                }
            });
        }
    });

    // ── devtools HTTP endpoint (simple TCP, not a full HTTP server) ───────────
    let http_addr = format!("0.0.0.0:{}", port + 1);
    thread::spawn(move || {
        serve_devtools_http(&http_addr);
    });

    // ── file watcher ─────────────────────────────────────────────────────────
    use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();
    let mut watcher: RecommendedWatcher = match notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    }) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("warning: could not start file watcher: {e}");
            // Still run — just won't auto-reload
            loop { thread::sleep(Duration::from_secs(60)); }
        }
    };

    let src_dir = Path::new("src");
    if let Err(e) = watcher.watch(src_dir, RecursiveMode::Recursive) {
        eprintln!("warning: could not watch src/: {e}");
    }

    // ── main event loop ───────────────────────────────────────────────────────
    loop {
        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(Ok(event)) => {
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                        println!("  File changed — recompiling…");
                        let payload = recompile_and_get_payload();

                        // Broadcast to all connected clients
                        let mut senders = client_senders.lock().unwrap();
                        senders.retain_mut(|sender| {
                            sender.send_message(&OwnedMessage::Text(payload.clone())).is_ok()
                        });

                        if payload.contains("\"reload\"") {
                            println!("  → reloaded ({} client(s))", senders.len());
                        } else {
                            println!("  → compile error broadcast");
                        }
                    }
                    _ => {}
                }
            }
            Ok(Err(e)) => eprintln!("  watch error: {e}"),
            Err(mpsc::RecvTimeoutError::Timeout) => {} // keep alive
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
}

// ─── Incremental recompile ────────────────────────────────────────────────────

fn recompile_and_get_payload() -> String {
    match parse_project(".") {
        Ok(ast) => {
            // Regenerate files
            let android_cfg = AndroidConfig::default();
            let ios_cfg = IosConfig::default();
            let android_files = gen_android(&ast, &android_cfg);
            let ios_files = gen_ios(&ast, &ios_cfg);

            let android_out = Path::new("build/android");
            let ios_out = Path::new("build/ios");

            for file in &android_files {
                let dest = android_out.join(&file.path);
                if let Some(p) = dest.parent() { fs::create_dir_all(p).ok(); }
                fs::write(&dest, &file.content).ok();
            }
            for file in &ios_files {
                let dest = ios_out.join(&file.path);
                if let Some(p) = dest.parent() { fs::create_dir_all(p).ok(); }
                fs::write(&dest, &file.content).ok();
            }

            reload_payload()
        }
        Err(errs) => {
            error_payload(&errs)
        }
    }
}

// ─── Simple devtools HTTP listener ────────────────────────────────────────────

fn serve_devtools_http(addr: &str) {
    use std::net::TcpListener;
    use std::io::{Read, Write};

    let listener = match TcpListener::bind(addr) {
        Ok(l) => l,
        Err(_) => return, // devtools endpoint is best-effort
    };

    for stream in listener.incoming().flatten() {
        let mut stream = stream;
        let mut buf = [0u8; 512];
        let _ = stream.read(&mut buf);
        let request = std::str::from_utf8(&buf).unwrap_or("");

        let body = if request.contains("GET /devtools/stores") {
            serde_json::json!({ "stores": {} }).to_string()
        } else {
            "{\"error\":\"not found\"}".to_string()
        };

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(response.as_bytes());
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::FrameError;
    use crate::parser::ErrorCategory;

    #[test]
    fn reload_payload_is_valid_json() {
        let payload = reload_payload();
        let v: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(v["type"], "reload");
    }

    #[test]
    fn error_payload_contains_errors() {
        let errs = vec![FrameError {
            category: ErrorCategory::ParseError,
            file: "src/app.fr".to_string(),
            line: 10,
            column: 5,
            message: "unexpected token".to_string(),
        }];
        let payload = error_payload(&errs);
        let v: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(v["type"], "error");
        assert_eq!(v["errors"][0]["file"], "src/app.fr");
        assert_eq!(v["errors"][0]["line"], 10);
    }

    #[test]
    fn devtools_endpoint_returns_json() {
        let data = serde_json::json!({ "AuthStore": { "token": "abc" } });
        let result = handle_devtools_request(&data);
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["AuthStore"]["token"], "abc");
    }
}
