use crate::models::Snapshot;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::{mpsc::Sender, Arc, RwLock};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

const INDEX: &str = include_str!("../web/index.html");
const DASHBOARD: &str = include_str!("../dashboard/astronomy-dashboard.yaml");
const MAX_OBSERVATION_BODY: usize = 16 * 1024;

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

fn html_response(body: &'static str) -> Response<std::io::Cursor<Vec<u8>>> {
    with_security_headers(
        Response::from_string(body)
            .with_header(header("Content-Type", "text/html; charset=utf-8"))
            .with_header(header(
                "Content-Security-Policy",
                "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'self'",
            )),
    )
}

fn json_response(value: serde_json::Value, status: u16) -> Response<std::io::Cursor<Vec<u8>>> {
    with_security_headers(
        Response::from_string(value.to_string())
            .with_status_code(StatusCode(status))
            .with_header(header("Content-Type", "application/json; charset=utf-8")),
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
    if input
        .transparency
        .is_some_and(|v| !(1..=5).contains(&v))
    {
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

fn read_limited_body(request: &mut Request) -> Result<String, String> {
    if request
        .body_length()
        .is_some_and(|length| length > MAX_OBSERVATION_BODY)
    {
        return Err("request body too large".to_string());
    }
    let mut bytes = Vec::new();
    request
        .as_reader()
        .take((MAX_OBSERVATION_BODY + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > MAX_OBSERVATION_BODY {
        return Err("request body too large".to_string());
    }
    String::from_utf8(bytes).map_err(|_| "request body is not UTF-8".to_string())
}

fn observation_path(data_dir: &Path) -> PathBuf {
    data_dir.join("observations.jsonl")
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
        forecast_transparency: current
            .as_ref()
            .map(|value| value.conditions.transparency),
        forecast_seeing_proxy: current.as_ref().map(|value| value.conditions.seeing_estimate),
    };
    match append_observation(&observation_path(data_dir), &record) {
        Ok(()) => {
            let _ = request.respond(json_response(json!({"saved": true, "record": record}), 201));
        }
        Err(error) => {
            eprintln!("Could not save observation: {error}");
            let _ = request.respond(json_response(json!({"error": "could not save observation"}), 500));
        }
    }
}

pub fn serve(
    snapshot: Arc<RwLock<Option<Snapshot>>>,
    refresh_tx: Sender<()>,
    data_dir: PathBuf,
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
                let _ = request.respond(
                    Response::from_string("forbidden").with_status_code(StatusCode(403)),
                );
                continue;
            }

            let path = request.url().split('?').next().unwrap_or("/");
            let method = request.method().clone();
            match (method, path) {
                (Method::Get, "/") | (Method::Get, "/index.html") => {
                    let _ = request.respond(html_response(INDEX));
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
                (Method::Get, "/api/observations") => {
                    let records = recent_observations(&observation_path(&data_dir), 50);
                    let _ = request.respond(json_response(json!(records), 200));
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
}
