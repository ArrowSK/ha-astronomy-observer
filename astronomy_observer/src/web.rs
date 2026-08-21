use crate::config::{AppConfig, UiSettings};
use crate::coordinates::HorizonMask;
use crate::ha::HaClient;
use crate::models::Snapshot;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::{mpsc::Sender, Arc, RwLock};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

const INDEX: &str = include_str!("../web/index.html");
const TARGET_IMAGE_SCRIPT: &str = include_str!("../web/target-images.js");
const DASHBOARD: &str = include_str!("../dashboard/astronomy-dashboard.yaml");
const OBJECT_IMAGE_DIR: &str = "/usr/share/astronomy-observer/object-images";
const MAX_REQUEST_BODY: usize = 16 * 1024;

#[derive(Debug, Deserialize)]
struct ObservationInput {
    #[serde(default)]
    sqm: Option<f64>,
    #[serde(default)]
    seeing_arcsec: Option<f64>,
    #[serde(default)]
    transparency: Option<u8>,
    #[serde(default)]
    limiting_magnitude: Option<f64>,
    #[serde(default)]
    notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ObservationRecord {
    recorded_at: String,
    location: String,
    sqm: Option<f64>,
    seeing_arcsec: Option<f64>,
    transparency: Option<u8>,
    limiting_magnitude: Option<f64>,
    notes: String,
    forecast_score: Option<f64>,
    forecast_transparency: Option<f64>,
    forecast_seeing_proxy: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct ObservationDeleteInput {
    recorded_at: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SettingsInput {
    #[serde(default)]
    primary_person: String,
    minimum_target_altitude: f64,
    horizon_mask: String,
}

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("valid static header")
}

fn allowed(ip: IpAddr) -> bool {
    if ip.is_loopback() {
        return true;
    }
    match ip {
        IpAddr::V4(address) => address.octets() == [172, 30, 32, 2],
        IpAddr::V6(address) => address
            .to_ipv4_mapped()
            .map(|mapped| mapped.octets() == [172, 30, 32, 2])
            .unwrap_or(false),
    }
}

fn with_security_headers(
    response: Response<std::io::Cursor<Vec<u8>>>,
) -> Response<std::io::Cursor<Vec<u8>>> {
    response
        .with_header(header("Cache-Control", "no-store"))
        .with_header(header("X-Content-Type-Options", "nosniff"))
        .with_header(header("Referrer-Policy", "no-referrer"))
}

fn enhanced_index() -> String {
    let marker = "</body>";
    let injection = format!("<script src=\"target-images.js?v=0.3.4\"></script>\n{marker}");
    INDEX.replacen(marker, &injection, 1)
}

fn html_response(body: String) -> Response<std::io::Cursor<Vec<u8>>> {
    with_security_headers(
        Response::from_string(body)
            .with_header(header("Content-Type", "text/html; charset=utf-8"))
            .with_header(header(
                "Content-Security-Policy",
                "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https://upload.wikimedia.org; connect-src 'self' https://en.wikipedia.org https://commons.wikimedia.org; frame-ancestors 'self'",
            )),
    )
}

fn javascript_response() -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(TARGET_IMAGE_SCRIPT)
        .with_header(header("Content-Type", "text/javascript; charset=utf-8"))
        .with_header(header("Cache-Control", "public, max-age=604800"))
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

fn object_asset_response(name: &str) -> Option<Response<std::io::Cursor<Vec<u8>>>> {
    if name.is_empty()
        || name.contains("..")
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return None;
    }
    let content_type = if name.ends_with(".webp") {
        "image/webp"
    } else if name.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if name.ends_with(".json") {
        "application/json; charset=utf-8"
    } else if name.ends_with(".txt") {
        "text/plain; charset=utf-8"
    } else {
        return None;
    };
    let data = fs::read(Path::new(OBJECT_IMAGE_DIR).join(name)).ok()?;
    Some(
        Response::from_data(data)
            .with_header(header("Content-Type", content_type))
            .with_header(header("Cache-Control", "public, max-age=604800"))
            .with_header(header("X-Content-Type-Options", "nosniff"))
            .with_header(header("Referrer-Policy", "no-referrer")),
    )
}

fn validate_observation(input: &ObservationInput) -> Result<(), &'static str> {
    if input.sqm.is_some_and(|v| !(10.0..=25.0).contains(&v)) {
        return Err("SQM must be between 10 and 25 mag/arcsec²");
    }
    if input
        .seeing_arcsec
        .is_some_and(|v| !(0.1..=20.0).contains(&v))
    {
        return Err("Seeing must be between 0.1 and 20 arcseconds");
    }
    if input.transparency.is_some_and(|v| !(1..=5).contains(&v)) {
        return Err("Transparency must be between 1 and 5");
    }
    if input
        .limiting_magnitude
        .is_some_and(|v| !(-2.0..=9.0).contains(&v))
    {
        return Err("Limiting magnitude must be between -2 and 9");
    }
    if input.notes.chars().count() > 1000 {
        return Err("Notes are limited to 1000 characters");
    }
    if input.sqm.is_none()
        && input.seeing_arcsec.is_none()
        && input.transparency.is_none()
        && input.limiting_magnitude.is_none()
        && input.notes.trim().is_empty()
    {
        return Err("Record at least one observation");
    }
    Ok(())
}

fn validate_settings(input: &SettingsInput) -> Result<(), String> {
    if !input.primary_person.trim().is_empty()
        && !input.primary_person.trim().starts_with("person.")
    {
        return Err("Observer must be Home or a person.* entity".to_string());
    }
    if !(0.0..=60.0).contains(&input.minimum_target_altitude) {
        return Err("Lowest useful altitude must be between 0° and 60°".to_string());
    }
    HorizonMask::parse(input.horizon_mask.trim())
        .map(|_| ())
        .map_err(|error| format!("Directional horizon mask is invalid: {error}"))
}

fn read_limited_body(request: &mut Request) -> Result<String, String> {
    if request
        .body_length()
        .is_some_and(|length| length > MAX_REQUEST_BODY)
    {
        return Err("request body too large".to_string());
    }
    let mut bytes = Vec::new();
    request
        .as_reader()
        .take((MAX_REQUEST_BODY + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > MAX_REQUEST_BODY {
        return Err("request body too large".to_string());
    }
    String::from_utf8(bytes).map_err(|_| "request body is not UTF-8".to_string())
}

fn observation_path(data_dir: &Path) -> PathBuf {
    data_dir.join("observations.jsonl")
}

fn settings_path(data_dir: &Path) -> PathBuf {
    data_dir.join("ui_settings.json")
}

fn append_observation(path: &Path, record: &ObservationRecord) -> Result<(), String> {
    let line = serde_json::to_string(record).map_err(|error| error.to_string())?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    writeln!(file, "{line}").map_err(|error| error.to_string())
}

fn recent_observations(path: &Path, limit: usize) -> Vec<ObservationRecord> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut values: Vec<ObservationRecord> = text
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    if values.len() > limit {
        values.drain(0..values.len() - limit);
    }
    values.reverse();
    values
}

fn delete_observations(path: &Path, recorded_at: &HashSet<String>) -> Result<usize, String> {
    if recorded_at.is_empty() {
        return Ok(0);
    }
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error.to_string()),
    };

    let mut kept = Vec::new();
    let mut deleted = 0usize;
    for line in text.lines() {
        match serde_json::from_str::<ObservationRecord>(line) {
            Ok(record) if recorded_at.contains(record.recorded_at.as_str()) => {
                deleted += 1;
            }
            _ => kept.push(line),
        }
    }

    if deleted == 0 {
        return Ok(0);
    }

    let temporary = path.with_extension("jsonl.tmp");
    let body = if kept.is_empty() {
        String::new()
    } else {
        format!("{}\n", kept.join("\n"))
    };
    fs::write(&temporary, body).map_err(|error| error.to_string())?;
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(error.to_string());
    }
    Ok(deleted)
}

fn handle_delete_observations(mut request: Request, data_dir: &Path) {
    let text = match read_limited_body(&mut request) {
        Ok(text) => text,
        Err(error) => {
            let _ = request.respond(json_response(json!({"error": error}), 413));
            return;
        }
    };
    let input: ObservationDeleteInput = match serde_json::from_str(&text) {
        Ok(input) => input,
        Err(_) => {
            let _ = request.respond(json_response(json!({"error": "invalid JSON"}), 400));
            return;
        }
    };
    if input.recorded_at.len() > 100 {
        let _ = request.respond(json_response(
            json!({"error": "at most 100 observations can be deleted at once"}),
            400,
        ));
        return;
    }
    let recorded_at: HashSet<String> = input
        .recorded_at
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect();
    if recorded_at.is_empty() {
        let _ = request.respond(json_response(
            json!({"error": "select at least one observation to delete"}),
            400,
        ));
        return;
    }

    match delete_observations(&observation_path(data_dir), &recorded_at) {
        Ok(deleted) => {
            let _ = request.respond(json_response(json!({"deleted": deleted}), 200));
        }
        Err(error) => {
            eprintln!("Could not delete observation: {error}");
            let _ = request.respond(json_response(
                json!({"error": "could not delete observation"}),
                500,
            ));
        }
    }
}

fn handle_observation(
    mut request: Request,
    snapshot: &Arc<RwLock<Option<Snapshot>>>,
    data_dir: &Path,
) {
    let text = match read_limited_body(&mut request) {
        Ok(text) => text,
        Err(error) => {
            let _ = request.respond(json_response(json!({"error": error}), 413));
            return;
        }
    };
    let input: ObservationInput = match serde_json::from_str(&text) {
        Ok(input) => input,
        Err(_) => {
            let _ = request.respond(json_response(json!({"error": "invalid JSON"}), 400));
            return;
        }
    };
    if let Err(error) = validate_observation(&input) {
        let _ = request.respond(json_response(json!({"error": error}), 400));
        return;
    }

    let current = snapshot.read().ok().and_then(|guard| guard.clone());
    let record = ObservationRecord {
        recorded_at: Utc::now().to_rfc3339(),
        location: current
            .as_ref()
            .map(|value| value.location.label.clone())
            .unwrap_or_else(|| "Unknown".to_string()),
        sqm: input.sqm,
        seeing_arcsec: input.seeing_arcsec,
        transparency: input.transparency,
        limiting_magnitude: input.limiting_magnitude,
        notes: input.notes.trim().to_string(),
        forecast_score: current.as_ref().map(|value| value.conditions.overall),
        forecast_transparency: current.as_ref().map(|value| value.conditions.transparency),
        forecast_seeing_proxy: current
            .as_ref()
            .map(|value| value.conditions.seeing_estimate),
    };
    match append_observation(&observation_path(data_dir), &record) {
        Ok(()) => {
            let _ = request.respond(json_response(json!({"saved": true, "record": record}), 201));
        }
        Err(error) => {
            eprintln!("Could not save observation: {error}");
            let _ = request.respond(json_response(
                json!({"error": "could not save observation"}),
                500,
            ));
        }
    }
}

fn handle_settings(mut request: Request, data_dir: &Path, refresh_tx: &Sender<()>) {
    let text = match read_limited_body(&mut request) {
        Ok(text) => text,
        Err(error) => {
            let _ = request.respond(json_response(json!({"error": error}), 413));
            return;
        }
    };
    let input: SettingsInput = match serde_json::from_str(&text) {
        Ok(input) => input,
        Err(_) => {
            let _ = request.respond(json_response(json!({"error": "invalid JSON"}), 400));
            return;
        }
    };
    if let Err(error) = validate_settings(&input) {
        let _ = request.respond(json_response(json!({"error": error}), 400));
        return;
    }

    let settings = UiSettings {
        primary_person: Some(input.primary_person.trim().to_string()),
        minimum_target_altitude: Some(input.minimum_target_altitude),
        horizon_mask: Some(input.horizon_mask.trim().to_string()),
    };
    let body = match serde_json::to_string_pretty(&settings) {
        Ok(body) => body,
        Err(error) => {
            let _ = request.respond(json_response(json!({"error": error.to_string()}), 500));
            return;
        }
    };
    let path = settings_path(data_dir);
    let temporary = data_dir.join("ui_settings.json.tmp");
    let result = fs::write(&temporary, body).and_then(|_| fs::rename(&temporary, &path));
    match result {
        Ok(()) => {
            let _ = refresh_tx.send(());
            let _ = request.respond(json_response(json!({"saved": true}), 200));
        }
        Err(error) => {
            let _ = fs::remove_file(&temporary);
            eprintln!("Could not save setup: {error}");
            let _ = request.respond(json_response(json!({"error": "could not save setup"}), 500));
        }
    }
}

pub fn serve(
    snapshot: Arc<RwLock<Option<Snapshot>>>,
    refresh_tx: Sender<()>,
    data_dir: PathBuf,
    options_path: PathBuf,
    config_dir: PathBuf,
    ha: HaClient,
) {
    std::thread::spawn(move || {
        let server = match Server::http("0.0.0.0:8099") {
            Ok(server) => server,
            Err(error) => {
                eprintln!("Ingress server failed: {error}");
                return;
            }
        };

        for request in server.incoming_requests() {
            if request
                .remote_addr()
                .map(|address| !allowed(address.ip()))
                .unwrap_or(true)
            {
                let _ = request
                    .respond(Response::from_string("forbidden").with_status_code(StatusCode(403)));
                continue;
            }

            let path = request.url().split('?').next().unwrap_or("/");
            let method = request.method().clone();
            match (method, path) {
                (Method::Get, "/") | (Method::Get, "/index.html") => {
                    let _ = request.respond(html_response(enhanced_index()));
                }
                (Method::Get, "/target-images.js") => {
                    let _ = request.respond(javascript_response());
                }
                (Method::Get, value) if value.starts_with("/object-images/") => {
                    let name = value.trim_start_matches("/object-images/");
                    match object_asset_response(name) {
                        Some(response) => {
                            let _ = request.respond(response);
                        }
                        None => {
                            let _ = request.respond(
                                Response::from_string("not found")
                                    .with_status_code(StatusCode(404)),
                            );
                        }
                    }
                }
                (Method::Get, "/health") => {
                    let ready = snapshot
                        .read()
                        .ok()
                        .and_then(|value| value.as_ref().map(|_| ()))
                        .is_some();
                    let code = if ready { 200 } else { 503 };
                    let _ = request.respond(
                        Response::from_string(if ready { "ok" } else { "starting" })
                            .with_status_code(StatusCode(code)),
                    );
                }
                (Method::Get, "/api/snapshot") => {
                    let value = snapshot.read().ok().and_then(|value| value.clone());
                    let body = serde_json::to_string(&value).unwrap_or_else(|_| "null".to_string());
                    let _ = request.respond(with_security_headers(
                        Response::from_string(body)
                            .with_header(header("Content-Type", "application/json; charset=utf-8")),
                    ));
                }
                (Method::Get, "/api/dashboard") => {
                    let _ = request.respond(with_security_headers(
                        Response::from_string(DASHBOARD)
                            .with_header(header("Content-Type", "text/yaml; charset=utf-8")),
                    ));
                }
                (Method::Get, "/api/people") => match ha.people() {
                    Ok(people) => {
                        let _ = request.respond(json_response(json!(people), 200));
                    }
                    Err(error) => {
                        let _ = request
                            .respond(json_response(json!({"error": error.to_string()}), 502));
                    }
                },
                (Method::Get, "/api/settings") => {
                    match AppConfig::load(&options_path, &data_dir, &config_dir) {
                        Ok(config) => {
                            let _ = request.respond(json_response(
                                json!({
                                    "primary_person": config.options.primary_person,
                                    "minimum_target_altitude": config.options.minimum_target_altitude,
                                    "horizon_mask": config.options.horizon_mask
                                }),
                                200,
                            ));
                        }
                        Err(error) => {
                            let _ = request
                                .respond(json_response(json!({"error": error.to_string()}), 500));
                        }
                    }
                }
                (Method::Post, "/api/settings") => {
                    handle_settings(request, &data_dir, &refresh_tx);
                }
                (Method::Get, "/api/observations") => {
                    let records = recent_observations(&observation_path(&data_dir), 50);
                    let _ = request.respond(json_response(json!(records), 200));
                }
                (Method::Delete, "/api/observations") => {
                    handle_delete_observations(request, &data_dir);
                }
                (Method::Post, "/api/observation") => {
                    handle_observation(request, &snapshot, &data_dir);
                }
                (Method::Post, "/api/refresh") => {
                    let _ = refresh_tx.send(());
                    let _ = request.respond(json_response(json!({"accepted": true}), 202));
                }
                _ => {
                    let _ = request.respond(
                        Response::from_string("not found").with_status_code(StatusCode(404)),
                    );
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ingress_source_filter() {
        assert!(allowed("127.0.0.1".parse().unwrap()));
        assert!(allowed("172.30.32.2".parse().unwrap()));
        assert!(!allowed("172.30.32.3".parse().unwrap()));
    }

    #[test]
    fn target_image_enhancement_is_injected() {
        let html = enhanced_index();
        assert!(html.contains("target-images.js?v=0.3.4"));
        assert!(TARGET_IMAGE_SCRIPT.contains("resolveCommons"));
    }

    #[test]
    fn validates_observation_ranges() {
        let valid = ObservationInput {
            sqm: Some(20.4),
            seeing_arcsec: Some(1.5),
            transparency: Some(4),
            limiting_magnitude: Some(5.8),
            notes: "clear and steady".to_string(),
        };
        assert!(validate_observation(&valid).is_ok());
    }

    #[test]
    fn deletes_selected_observations_without_touching_other_lines() {
        let path = std::env::temp_dir().join(format!(
            "astronomy-observer-delete-test-{}-{}.jsonl",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let first = r#"{"recorded_at":"2026-08-13T01:00:00Z","location":"A","sqm":null,"seeing_arcsec":null,"transparency":null,"limiting_magnitude":null,"notes":"first","forecast_score":null,"forecast_transparency":null,"forecast_seeing_proxy":null}"#;
        let second = r#"{"recorded_at":"2026-08-13T02:00:00Z","location":"B","sqm":null,"seeing_arcsec":null,"transparency":null,"limiting_magnitude":null,"notes":"second","forecast_score":null,"forecast_transparency":null,"forecast_seeing_proxy":null}"#;
        fs::write(&path, format!("{first}\n{second}\n")).unwrap();
        let selected = HashSet::from(["2026-08-13T01:00:00Z".to_string()]);
        assert_eq!(delete_observations(&path, &selected).unwrap(), 1);
        let remaining = fs::read_to_string(&path).unwrap();
        assert!(!remaining.contains("first"));
        assert!(remaining.contains("second"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn validates_simple_setup() {
        let valid = SettingsInput {
            primary_person: "person.alex".to_string(),
            minimum_target_altitude: 20.0,
            horizon_mask: "0:0,90:0,180:0,270:0".to_string(),
        };
        assert!(validate_settings(&valid).is_ok());
    }
}
