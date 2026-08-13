#!/usr/bin/env python3
from pathlib import Path
import re
import tomllib

ROOT = Path(__file__).resolve().parents[1]
APP = ROOT / "astronomy_observer"
WEB = ROOT / "webapp"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    required = [
        WEB / "Dockerfile",
        WEB / "src/main.rs",
        WEB / "src/ha.rs",
        WEB / "src/server.rs",
        WEB / "src/calculator.rs",
        WEB / "src/ui.rs",
        WEB / "ui/manual.js",
        ROOT / "railway.toml",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    require(not missing, f"missing web deployment files: {', '.join(missing)}")

    ha_docker = (APP / "Dockerfile").read_text(encoding="utf-8")
    helper_script = (WEB / "build-astro-helper.sh").read_text(encoding="utf-8")
    catalog_script = (WEB / "build-catalog.sh").read_text(encoding="utf-8")
    astronomy_pin = re.search(r"ARG ASTRONOMY_ENGINE_COMMIT=([0-9a-f]{40})", ha_docker)
    openngc_pin = re.search(r"ARG OPENNGC_COMMIT=([0-9a-f]{40})", ha_docker)
    require(astronomy_pin is not None, "Home Assistant Astronomy Engine pin missing")
    require(openngc_pin is not None, "Home Assistant OpenNGC pin missing")
    require(astronomy_pin.group(1) in helper_script, "web helper pin differs from Home Assistant")
    require(openngc_pin.group(1) in catalog_script, "web catalogue pin differs from Home Assistant")

    shared_modules = (WEB / "src/main.rs").read_text(encoding="utf-8")
    for module in [
        "astro", "aurora", "comets", "config", "coordinates", "engine",
        "light_pollution", "meteors", "models", "satellites", "scoring",
        "targets", "weather",
    ]:
        require(
            f'../../astronomy_observer/src/{module}.rs' in shared_modules,
            f"web adapter is not compiling shared {module}.rs",
        )

    base_ui = (APP / "web/index.html").read_text(encoding="utf-8")
    for marker in [
        'id="person-select"',
        "load();\nupdateBottomNav();\nsetInterval(load, 60000);",
        "<span>Forecast</span>",
    ]:
        require(marker in base_ui, f"shared UI marker changed: {marker}")

    railway = tomllib.loads((ROOT / "railway.toml").read_text(encoding="utf-8"))
    require(railway.get("build", {}).get("builder") == "DOCKERFILE", "Railway must use Dockerfile builder")
    require(
        railway.get("build", {}).get("dockerfilePath") == "webapp/Dockerfile",
        "Railway Dockerfile path mismatch",
    )
    require(railway.get("deploy", {}).get("healthcheckPath") == "/health", "Railway health check mismatch")

    print("web deployment validation passed")


if __name__ == "__main__":
    main()
