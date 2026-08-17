#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ANDROID = ROOT / "android"
HA = ROOT / "astronomy_observer"


def require(value: bool, message: str) -> None:
    if not value:
        raise AssertionError(message)


def main() -> None:
    required = [
        ANDROID / "app/build.gradle.kts",
        ANDROID / "app/src/main/AndroidManifest.xml",
        ANDROID / "app/src/main/java/com/arrowsk/astronomyobserver/MainActivity.java",
        ANDROID / "app/src/main/java/com/arrowsk/astronomyobserver/AndroidBridge.java",
        ANDROID / "app/src/main/java/com/arrowsk/astronomyobserver/NativeBridge.java",
        ANDROID / "native/astro_bridge.c",
        ANDROID / "native-rust/Cargo.toml",
        ANDROID / "native-rust/src/lib.rs",
        ANDROID / "ui/android.js",
        ANDROID / "prepare_assets.py",
        ANDROID / "build-native.sh",
        ANDROID / "build-catalog.sh",
        ANDROID / "build-apk.sh",
        ANDROID / "app/src/main/assets/legal-summary.txt",
        ROOT / "docs/ANDROID.md",
        HA / "data/object_images/manifest.json",
        HA / "data/object_images/m031.webp",
        HA / "data/object_images/credits.html",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    require(not missing, "missing Android files: " + ", ".join(missing))

    gradle = (ANDROID / "app/build.gradle.kts").read_text(encoding="utf-8")
    require('applicationId = "com.arrowsk.astronomyobserver"' in gradle, "Android application ID changed")
    require("compileSdk = 36" in gradle and "targetSdk = 36" in gradle, "Android SDK target must be 36")
    require('versionName = "0.3.1"' in gradle, "Android version must match 0.3.1 release")
    require('"arm64-v8a", "x86_64"' in gradle, "Android ABI set changed")

    manifest = (ANDROID / "app/src/main/AndroidManifest.xml").read_text(encoding="utf-8")
    require('android:usesCleartextTraffic="false"' in manifest, "Android cleartext traffic must stay disabled")
    require('android:allowBackup="false"' in manifest, "Android backup must stay disabled for local location/journal data")
    require("ACCESS_FINE_LOCATION" in manifest and "INTERNET" in manifest, "Android permissions incomplete")

    activity = (ANDROID / "app/src/main/java/com/arrowsk/astronomyobserver/MainActivity.java").read_text(encoding="utf-8")
    for marker in [
        "setBlockNetworkLoads(true)",
        "setAllowUniversalAccessFromFileURLs(false)",
        "MIXED_CONTENT_NEVER_ALLOW",
        "file:///android_asset/index.html",
        'addJavascriptInterface(bridge, "AstronomyAndroid")',
    ]:
        require(marker in activity, f"Android WebView safety marker missing: {marker}")
    require("http://" not in activity and "https://" not in activity, "Android WebView shell must not hard-code a remote app server")

    native = (ANDROID / "native-rust/src/lib.rs").read_text(encoding="utf-8")
    for module in [
        "astro", "aurora", "comets", "config", "coordinates", "engine",
        "light_pollution", "meteors", "models", "satellites", "scoring", "targets", "weather",
    ]:
        require(
            f'../../../astronomy_observer/src/{module}.rs' in native,
            f"Android runtime is not compiling shared {module}.rs",
        )

    astro = (HA / "src/astro.rs").read_text(encoding="utf-8")
    engine = (HA / "src/engine.rs").read_text(encoding="utf-8")
    light = (HA / "src/light_pollution.rs").read_text(encoding="utf-8")
    weather = (HA / "src/weather.rs").read_text(encoding="utf-8")
    require('cfg(target_os = "android")' in astro and "ao_astro_calculate" in astro, "Android astronomy FFI missing")
    require("ASTRONOMY_RESOURCE_DIR" in engine, "shared engine does not support packaged Android resources")
    require("ASTRONOMY_RESOURCE_DIR" in light, "light-pollution atlas path is not Android-aware")
    require("offline fallback — no live weather" in weather, "Android offline weather fallback missing")
    require(
        '#[cfg(not(target_os = "android"))]\n    match fetch_met_norway' in weather,
        "Android must not call MET Norway directly without a mobile proxy",
    )

    ha_config = (HA / "config.yaml").read_text(encoding="utf-8")
    require('version: "0.3.1"' in ha_config, "Home Assistant version must match Android release")
    require("ARG BUILD_VERSION=0.3.1" in (HA / "Dockerfile").read_text(encoding="utf-8"), "container version mismatch")

    native_build = (ANDROID / "build-native.sh").read_text(encoding="utf-8")
    catalogue_build = (ANDROID / "build-catalog.sh").read_text(encoding="utf-8")
    require("61dc07020aaa6885d2c7f688a4d82beaf6edb9ef" in native_build, "Astronomy Engine pin changed")
    require("da90466031b0372c896588b85be6016c617e205b" in catalogue_build, "OpenNGC pin changed")
    require("astronomy-engine-license.txt" in native_build, "upstream MIT notice is not bundled")

    legal = (ANDROID / "app/src/main/assets/legal-summary.txt").read_text(encoding="utf-8")
    for marker in [
        "CC BY-SA 4.0", "CC BY-NC 4.0", "CC BY 4.0", "Data from MET Norway",
        "Data from Open-Meteo", "not relicensed under the project licence",
    ]:
        require(marker in legal, f"Android legal attribution missing: {marker}")

    generated = ANDROID / "generated/assets/index.html"
    if generated.exists():
        html = generated.read_text(encoding="utf-8")
        require("Use current location" in html, "generated Android UI lacks device location action")
        require("About &amp; licences" in html, "generated Android UI lacks licence access")
        require("AstronomyAndroid.calculate" in html, "generated Android UI does not call native runtime")
        require("Weather data by Open-Meteo.com" in html, "Open-Meteo attribution is not visible beside Android source data")
        require("Copy dashboard YAML" in html, "shared dashboard marker unexpectedly disappeared")
        require("object-images/" in html, "generated Android UI lacks bundled target thumbnails")
        require((ANDROID / "generated/assets/object-images/m031.webp").is_file(), "generated Android assets lack M31 thumbnail")

    print("Android standalone validation passed")


if __name__ == "__main__":
    main()
