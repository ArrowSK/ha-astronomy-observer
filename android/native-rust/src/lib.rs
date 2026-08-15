#[path = "../../../astronomy_observer/src/astro.rs"]
mod astro;
#[path = "../../../astronomy_observer/src/aurora.rs"]
mod aurora;
#[path = "../../../astronomy_observer/src/comets.rs"]
mod comets;
#[path = "../../../astronomy_observer/src/config.rs"]
mod config;
#[path = "../../../astronomy_observer/src/coordinates.rs"]
mod coordinates;
#[path = "../../../astronomy_observer/src/engine.rs"]
mod engine;
#[path = "../../../astronomy_observer/src/error.rs"]
mod error;
#[path = "../../../astronomy_observer/src/light_pollution.rs"]
mod light_pollution;
#[path = "../../../astronomy_observer/src/meteors.rs"]
mod meteors;
#[path = "../../../astronomy_observer/src/models.rs"]
mod models;
#[path = "../../../astronomy_observer/src/satellites.rs"]
mod satellites;
#[path = "../../../astronomy_observer/src/scoring.rs"]
mod scoring;
#[path = "../../../astronomy_observer/src/targets.rs"]
mod targets;
#[path = "../../../astronomy_observer/src/weather.rs"]
mod weather;

mod calculator;
mod ha;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::OnceLock;

static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();
static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();

fn c_string(value: String) -> *mut c_char {
    CString::new(value)
        .unwrap_or_else(|_| CString::new("invalid string").expect("static string"))
        .into_raw()
}

fn read_c_string(value: *const c_char) -> Result<String, String> {
    if value.is_null() {
        return Err("native bridge received a null string".to_string());
    }
    let text = unsafe { CStr::from_ptr(value) };
    text.to_str()
        .map(str::to_string)
        .map_err(|_| "native bridge received invalid UTF-8".to_string())
}

#[no_mangle]
pub extern "C" fn ao_initialize(
    resource_dir: *const c_char,
    data_dir: *const c_char,
    config_dir: *const c_char,
) -> *mut c_char {
    let result = (|| -> Result<(), String> {
        let resource_dir = read_c_string(resource_dir)?;
        let data_dir = PathBuf::from(read_c_string(data_dir)?);
        let config_dir = PathBuf::from(read_c_string(config_dir)?);
        if !PathBuf::from(&resource_dir).is_dir() {
            return Err("Android resource directory is missing".to_string());
        }
        std::fs::create_dir_all(&data_dir).map_err(|error| error.to_string())?;
        std::fs::create_dir_all(&config_dir).map_err(|error| error.to_string())?;
        std::env::set_var("ASTRONOMY_RESOURCE_DIR", &resource_dir);
        let _ = DATA_DIR.set(data_dir);
        let _ = CONFIG_DIR.set(config_dir);
        Ok(())
    })();
    match result {
        Ok(()) => c_string(String::new()),
        Err(error) => c_string(error),
    }
}

#[no_mangle]
pub extern "C" fn ao_calculate_json(input: *const c_char) -> *mut c_char {
    let response = (|| -> Result<serde_json::Value, String> {
        let input = read_c_string(input)?;
        let request: calculator::SnapshotInput =
            serde_json::from_str(&input).map_err(|error| format!("Invalid request: {error}"))?;
        let data_dir = DATA_DIR
            .get()
            .ok_or_else(|| "Android runtime is not initialized".to_string())?;
        let config_dir = CONFIG_DIR
            .get()
            .ok_or_else(|| "Android runtime is not initialized".to_string())?;
        calculator::calculate(request, data_dir, config_dir).map_err(|error| error.to_string())
    })();

    let envelope = match response {
        Ok(snapshot) => serde_json::json!({"ok": true, "snapshot": snapshot}),
        Err(error) => serde_json::json!({"ok": false, "error": error}),
    };
    c_string(envelope.to_string())
}

#[no_mangle]
pub extern "C" fn ao_free_string(value: *mut c_char) {
    if !value.is_null() {
        unsafe {
            drop(CString::from_raw(value));
        }
    }
}
