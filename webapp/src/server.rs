#[path = "calculator.rs"]
mod calculator;

use crate::error::{err, AppResult};
use crate::session::STATELESS_WEB;
use crate::ui;
use serde_json::json;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

const APP_CONFIG: &str = include_str!("../../astronomy_observer/config.yaml");
const MAX_BODY: usize = 16 * 1024;
static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("valid static header")
}

fn app_version() -> String {
    APP_CONFIG
        .lines()
        .find_map(|line| line.trim().strip_prefix("version:"))
        .map(|value| value.trim().trim_matches('"').to_string())
        .unwrap_or_else(|| "development".to_string())
}

fn with_security_headers(
    response: Response<std::io::Cursor<Vec<u8>>>,
) -> Response<std::io::Cursor<Vec<u8>>> {
    response
        .with_header(header("Cache-Control", "no-store"))
        .with_header(header("X-Content-Type-Options", "nosniff"))
        .with_header(header("Referrer-Policy", "no-referrer"))
}

fn json_response(value: serde_json::Value, status: u16) -> Response<std::io::Cursor<Vec<u8>>> {
    with_security_headers(
        Response::from_string(value.to_string())
            .with_status_code(StatusCode(status))
            .with_header(header("Content-Type", "application/json; charset=utf-8")),
    )
}

fn html_response(body: String) -> Response<std::io::Cursor<Vec<u8>>> {
    with_security_headers(
        Response::from_string(body)
            .with_header(header("Content-Type", "text/html; charset=utf-8"))
            .with_header(header(
                "Content-Security-Policy",
                "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'self'",
            )),
    )
}

fn read_body(request: &mut Request) -> Result<String, String> {
    if request
        .body_length()
        .is_some_and(|length| length > MAX_BODY)
    {
        return Err("request body too large".to_string());
    }
    let mut bytes = Vec::new();
    request
        .as_reader()
        .take((MAX_BODY + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > MAX_BODY {
        return Err("request body too large".to_string());
    }
    String::from_utf8(bytes).map_err(|_| "request body is not UTF-8".to_string())
}

fn request_work_dir(root: &Path) -> PathBuf {
    let id = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    root.join(format!("{}-{id}", std::process::id()))
}

fn handle_snapshot(mut request: Request, temp_root: &Path, config_dir: &Path) {
    let body = match read_body(&mut request) {
        Ok(value) => value,
        Err(error) => {
            let _ = request.respond(json_response(json!({"error": error}), 413));
            return;
        }
    };
    let input: calculator::SnapshotInput = match serde_json::from_str(&body) {
        Ok(value) => value,
        Err(_) => {
            let _ = request.respond(json_response(json!({"error": "invalid JSON"}), 400));
            return;
        }
    };
    if let Err(error) = calculator::validate(&input) {
        let _ = request.respond(json_response(json!({"error": error}), 400));
        return;
    }

    let work_dir = request_work_dir(temp_root);
    let result = calculator::calculate(input, &work_dir, config_dir);
    let _ = fs::remove_dir_all(&work_dir);
    match result {
        Ok(snapshot) => {
            let _ = request.respond(json_response(snapshot, 200));
        }
        Err(error) => {
            eprintln!("web calculation failed: {error}");
            let _ = request.respond(json_response(json!({"error": error.to_string()}), 502));
        }
    }
}

fn handle_request(request: Request, index: &str, temp_root: &Path, config_dir: &Path) {
    let path = request.url().split('?').next().unwrap_or("/").to_string();
    let method = request.method().clone();
    match (method, path.as_str()) {
        (Method::Get, "/") | (Method::Get, "/index.html") => {
            let _ = request.respond(html_response(index.to_string()));
        }
        (Method::Get, "/health") => {
            let _ = request.respond(Response::from_string("ok").with_status_code(StatusCode(200)));
        }
        (Method::Get, "/api/platform") => {
            let _ = request.respond(json_response(
                json!({
                    "platform": "web",
                    "version": app_version(),
                    "shared_runtime": true,
                    "stateless": STATELESS_WEB
                }),
                200,
            ));
        }
        (Method::Post, "/api/web/snapshot") => {
            handle_snapshot(request, temp_root, config_dir);
        }
        (Method::Post, "/api/refresh") => {
            let _ = request.respond(json_response(json!({"accepted": true}), 202));
        }
        _ => {
            let _ = request
                .respond(Response::from_string("not found").with_status_code(StatusCode(404)));
        }
    }
}

pub fn run() -> AppResult<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|value| value == "--self-test") {
        let html = ui::web_index()?;
        if !html.contains("api/web/snapshot") || !html.contains("web-location-lat") {
            return Err(err("standalone web interface self-test failed"));
        }
        println!("web self-test passed");
        return Ok(());
    }

    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(8080);
    let temp_root = PathBuf::from(
        std::env::var("ASTRONOMY_TEMP_DIR")
            .unwrap_or_else(|_| "/tmp/astronomy-observer-web".to_string()),
    );
    let config_dir = PathBuf::from(
        std::env::var("ASTRONOMY_CONFIG_DIR").unwrap_or_else(|_| "/config".to_string()),
    );
    fs::create_dir_all(&temp_root)?;
    fs::create_dir_all(&config_dir)?;

    let index = Arc::new(ui::web_index()?);
    let server = Server::http(format!("0.0.0.0:{port}"))
        .map_err(|error| err(format!("could not start web server: {error}")))?;
    println!(
        "Astronomy Observer Web {} listening on port {port}",
        app_version()
    );

    for request in server.incoming_requests() {
        let index = index.clone();
        let temp_root = temp_root.clone();
        let config_dir = config_dir.clone();
        std::thread::spawn(move || {
            handle_request(request, &index, &temp_root, &config_dir);
        });
    }
    Ok(())
}
