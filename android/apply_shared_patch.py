#!/usr/bin/env python3
from pathlib import Path


def replace_once(path: Path, old: str, new: str, label: str) -> None:
    text = path.read_text(encoding="utf-8")
    if old not in text:
        raise SystemExit(f"marker changed for {label}: {path}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


root = Path(__file__).resolve().parents[1]

astro = root / "astronomy_observer/src/astro.rs"
text = astro.read_text(encoding="utf-8")
old_imports = "use std::io::Write;\nuse std::process::{Command, Stdio};"
new_imports = "#[cfg(not(target_os = \"android\"))]\nuse std::io::Write;\n#[cfg(not(target_os = \"android\"))]\nuse std::process::{Command, Stdio};"
if old_imports not in text:
    raise SystemExit("astronomy imports marker changed")
text = text.replace(old_imports, new_imports, 1)
marker = "pub fn calculate(location: &Location, times: &[DateTime<Utc>]) -> AppResult<Vec<AstronomySample>> {"
if marker not in text:
    raise SystemExit("astronomy calculate marker changed")
text = text.replace(marker, "#[cfg(not(target_os = \"android\"))]\n" + marker, 1)
sample_marker = "pub fn sample_nearest(samples: &[AstronomySample], time: DateTime<Utc>) -> Option<&AstronomySample> {"
if sample_marker not in text:
    raise SystemExit("astronomy sample marker changed")
android_calc = '''#[cfg(target_os = "android")]
pub fn calculate(location: &Location, times: &[DateTime<Utc>]) -> AppResult<Vec<AstronomySample>> {
    use std::ffi::CStr;
    use std::os::raw::c_char;

    unsafe extern "C" {
        fn ao_astro_calculate(
            latitude: f64,
            longitude: f64,
            elevation: f64,
            epochs: *const i64,
            count: usize,
        ) -> *mut c_char;
        fn ao_astro_free(value: *mut c_char);
    }

    if times.is_empty() {
        return Ok(Vec::new());
    }
    let epochs: Vec<i64> = times.iter().map(|time| time.timestamp()).collect();
    let value = unsafe {
        ao_astro_calculate(
            location.latitude,
            location.longitude,
            location.elevation_m,
            epochs.as_ptr(),
            epochs.len(),
        )
    };
    if value.is_null() {
        return Err(err("embedded Astronomy Engine calculation failed"));
    }
    let output = unsafe { CStr::from_ptr(value) }
        .to_string_lossy()
        .into_owned();
    unsafe { ao_astro_free(value) };
    parse_output(&output)
}

'''
text = text.replace(sample_marker, android_calc + sample_marker, 1)
astro.write_text(text, encoding="utf-8")

engine = root / "astronomy_observer/src/engine.rs"
replace_once(engine, "use std::path::Path;", "use std::path::PathBuf;", "engine path import")
text = engine.read_text(encoding="utf-8")
refresh_marker = "pub fn refresh(cfg: &AppConfig, ha: &HaClient) -> AppResult<Snapshot> {"
helper = '''fn resource_path(name: &str) -> PathBuf {
    std::env::var_os("ASTRONOMY_RESOURCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/share/astronomy-observer"))
        .join(name)
}

'''
if refresh_marker not in text:
    raise SystemExit("engine refresh marker changed")
text = text.replace(refresh_marker, helper + refresh_marker, 1)
old_resources = '    let catalog = Path::new("/usr/share/astronomy-observer/catalog.tsv");\n    let meteor_showers = Path::new("/usr/share/astronomy-observer/meteor_showers.csv");'
if old_resources not in text:
    raise SystemExit("engine resource markers changed")
text = text.replace(old_resources, '    let catalog = resource_path("catalog.tsv");\n    let meteor_showers = resource_path("meteor_showers.csv");', 1)
text = text.replace("        catalog,", "        &catalog,", 1)
text = text.replace("        meteor_showers,", "        &meteor_showers,", 1)
engine.write_text(text, encoding="utf-8")

light = root / "astronomy_observer/src/light_pollution.rs"
replace_once(light, "use std::path::Path;", "use std::path::{Path, PathBuf};", "light pollution path import")
replace_once(
    light,
    'const BUNDLED_ATLAS_PATH: &str = "/usr/share/astronomy-observer/world_atlas_3min.bin";',
    '''const DEFAULT_BUNDLED_ATLAS_PATH: &str = "/usr/share/astronomy-observer/world_atlas_3min.bin";

fn bundled_atlas_path() -> PathBuf {
    std::env::var_os("ASTRONOMY_RESOURCE_DIR")
        .map(PathBuf::from)
        .map(|path| path.join("world_atlas_3min.bin"))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_BUNDLED_ATLAS_PATH))
}''',
    "light pollution resource path",
)
replace_once(
    light,
    "        binary_lookup(location, Path::new(BUNDLED_ATLAS_PATH), dark_radius_km);",
    "        binary_lookup(location, &bundled_atlas_path(), dark_radius_km);",
    "light pollution atlas lookup",
)

weather = root / "astronomy_observer/src/weather.rs"
old_weather = '''    load_cache(&cache)
        .ok_or_else(|| err("weather providers failed and no recent cache is available"))
}'''
new_weather = '''    if let Some(series) = load_cache(&cache) {
        return Ok(series);
    }

    #[cfg(target_os = "android")]
    {
        let now = Utc::now();
        let hours = (0..=days * 24)
            .map(|offset| HourlyWeather {
                time: now + chrono::Duration::hours(offset as i64),
                ..Default::default()
            })
            .collect();
        return Ok(WeatherSeries {
            source: "offline fallback — no live weather".to_string(),
            retrieved_at: now,
            stale: true,
            hours,
        });
    }

    #[cfg(not(target_os = "android"))]
    {
        Err(err("weather providers failed and no recent cache is available"))
    }
}'''
replace_once(weather, old_weather, new_weather, "Android weather fallback")

config = root / "astronomy_observer/config.yaml"
replace_once(config, 'version: "0.2.3"', 'version: "0.3.0"', "app version")
docker = root / "astronomy_observer/Dockerfile"
replace_once(docker, "ARG BUILD_VERSION=0.2.3", "ARG BUILD_VERSION=0.3.0", "container version")

ci = root / ".github/workflows/ci.yaml"
text = ci.read_text(encoding="utf-8")
ci_marker = "      - name: Validate standalone web packaging\n        run: python3 webapp/validate.py\n"
if ci_marker not in text:
    raise SystemExit("CI web validation marker changed")
text = text.replace(ci_marker, ci_marker + "\n      - name: Validate standalone Android packaging\n        run: python3 android/validate.py\n", 1)
old_compile = "tests/validate_repository.py tests/validate_atlas.py webapp/validate.py"
if old_compile not in text:
    raise SystemExit("CI Python compile marker changed")
text = text.replace(old_compile, old_compile + " android/validate.py android/prepare_assets.py", 1)
ci.write_text(text, encoding="utf-8")

print("shared Android adaptations applied")
