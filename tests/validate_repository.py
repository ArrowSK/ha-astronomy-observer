#!/usr/bin/env python3
"""Static repository checks that do not require Home Assistant or network access."""
from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
APP = ROOT / "astronomy_observer"


def fail(message: str) -> None:
    raise AssertionError(message)


def load_yaml(path: Path):
    with path.open("r", encoding="utf-8") as fh:
        return yaml.safe_load(fh)


def required_paths() -> None:
    paths = [
        ROOT / "README.md",
        ROOT / "LICENSE",
        ROOT / "THIRD_PARTY_LICENSES.md",
        ROOT / "CONTRIBUTING.md",
        ROOT / "SECURITY.md",
        ROOT / "repository.yaml",
        ROOT / "dashboard/astronomy-dashboard.yaml",
        ROOT / "docs/SCORING.md",
        ROOT / "docs/DATA_SOURCES.md",
        ROOT / "docs/LIGHT_POLLUTION.md",
        ROOT / "docs/PRIVACY.md",
        ROOT / "docs/ARCHITECTURE.md",
        ROOT / "docs/DEVELOPMENT.md",
        ROOT / "docs/LIMITS_AND_ROADMAP.md",
        ROOT / "tools/light_pollution_tile.py",
        APP / "config.yaml",
        APP / "DOCS.md",
        APP / "README.md",
        APP / "CHANGELOG.md",
        APP / "Dockerfile",
        APP / "apparmor.txt",
        APP / "icon.png",
        APP / "logo.png",
        APP / "run.sh",
        APP / "web/index.html",
        APP / "dashboard/astronomy-dashboard.yaml",
        APP / "translations/en.yaml",
    ]
    missing = [str(path.relative_to(ROOT)) for path in paths if not path.exists()]
    if missing:
        fail(f"missing required files: {', '.join(missing)}")


def validate_manifest() -> None:
    repository = load_yaml(ROOT / "repository.yaml")
    if repository.get("name") != "Astronomy Observer":
        fail("repository.yaml name mismatch")
    if repository.get("url") != "https://github.com/ArrowSK/ha-astronomy-observer":
        fail("repository.yaml URL mismatch")

    config = load_yaml(APP / "config.yaml")
    if config.get("slug") != APP.name:
        fail("app slug must match app directory")
    if config.get("image") != "ghcr.io/arrowsk/ha-astronomy-observer":
        fail("unexpected container image")
    if set(config.get("arch", [])) != {"aarch64", "amd64"}:
        fail("expected aarch64 and amd64 architectures")
    if not config.get("homeassistant_api") or not config.get("ingress"):
        fail("Home Assistant API and Ingress must be enabled")
    if "apparmor" in config:
        fail("custom AppArmor profile must not replace the Supervisor default")
    if config.get("panel_admin") is not True:
        fail("Ingress panel must remain admin-only")
    if not str(config.get("watchdog", "")).startswith("tcp://"):
        fail("watchdog should use TCP because HTTP is restricted to Ingress")
    maps = config.get("map", [])
    if maps != [{"type": "addon_config", "read_only": True}]:
        fail("only read-only addon_config should be mapped")

    options = config.get("options", {})
    schema = config.get("schema", {})
    translations = load_yaml(APP / "translations/en.yaml").get("configuration", {})
    if set(options) != set(schema):
        fail("config options and schema keys differ")
    if set(schema) != set(translations):
        fail("config schema and English translation keys differ")

    cargo = tomllib.loads((APP / "Cargo.toml").read_text(encoding="utf-8"))
    if cargo["package"]["version"] != str(config["version"]):
        fail("Cargo.toml and config.yaml versions differ")


def validate_dashboard() -> None:
    public = (ROOT / "dashboard/astronomy-dashboard.yaml").read_text(encoding="utf-8")
    embedded = (APP / "dashboard/astronomy-dashboard.yaml").read_text(encoding="utf-8")
    if public != embedded:
        fail("public and embedded dashboard presets differ")
    load_yaml(ROOT / "dashboard/astronomy-dashboard.yaml")

    dashboard_entities = set(re.findall(r"entity:\s+([a-z_]+\.[a-z0-9_]+)", public))
    state_source = (APP / "src/state.rs").read_text(encoding="utf-8")
    literal_published = set(re.findall(r'"((?:sensor|binary_sensor)\.astronomy_observer_[a-z0-9_]+)"', state_source))
    missing = []
    for entity in sorted(dashboard_entities):
        if entity in literal_published:
            continue
        if re.fullmatch(r"sensor\.astronomy_observer_target_(?:[1-9]|10)", entity) and "sensor.astronomy_observer_target_{}" in state_source:
            continue
        missing.append(entity)
    if missing:
        fail(f"dashboard references unpublished entities: {', '.join(missing)}")


def validate_docs() -> None:
    markdown_files = list(ROOT.glob("*.md")) + list((ROOT / "docs").glob("*.md")) + [
        APP / "README.md",
        APP / "DOCS.md",
        APP / "CHANGELOG.md",
    ]
    prohibited = [
        re.compile(r"\bChatGPT\b", re.I),
        re.compile(r"\blanguage model\b", re.I),
        re.compile(r"\bartificial intelligence\b", re.I),
        re.compile(r"\bLLM\b"),
        re.compile(r"\bAI\b"),
        re.compile(r"generated by", re.I),
    ]
    link_pattern = re.compile(r"\[[^\]]*\]\(([^)]+)\)")
    for path in markdown_files:
        text = path.read_text(encoding="utf-8")
        for pattern in prohibited:
            if pattern.search(text):
                fail(f"unwanted documentation wording {pattern.pattern!r} in {path.relative_to(ROOT)}")
        for target in link_pattern.findall(text):
            target = target.split("#", 1)[0]
            if not target or "://" in target or target.startswith("mailto:"):
                continue
            resolved = (path.parent / target).resolve()
            try:
                resolved.relative_to(ROOT.resolve())
            except ValueError:
                fail(f"documentation link escapes repository: {path}: {target}")
            if not resolved.exists():
                fail(f"broken documentation link in {path.relative_to(ROOT)}: {target}")


def validate_licensing_and_pins() -> None:
    license_text = (ROOT / "LICENSE").read_text(encoding="utf-8")
    if "https://polyformproject.org/licenses/noncommercial/1.0.0" not in license_text:
        fail("PolyForm canonical URL missing from LICENSE")
    if "Required Notice: Copyright 2026 ArrowSK" not in license_text:
        fail("required copyright notice missing from LICENSE")

    docker = (APP / "Dockerfile").read_text(encoding="utf-8")
    third_party = (ROOT / "THIRD_PARTY_LICENSES.md").read_text(encoding="utf-8")
    pins = re.findall(r"ARG\s+(?:ASTRONOMY_ENGINE_COMMIT|OPENNGC_COMMIT)=([0-9a-f]{40})", docker)
    if len(pins) != 2:
        fail("expected two pinned third-party source commits in Dockerfile")
    for pin in pins:
        if pin not in third_party:
            fail(f"third-party source pin {pin} missing from THIRD_PARTY_LICENSES.md")


def validate_web_and_security() -> None:
    web = (APP / "web/index.html").read_text(encoding="utf-8")
    source = (APP / "src/web.rs").read_text(encoding="utf-8")
    required_ui_calls = ["api/snapshot", "api/dashboard", "api/observations", "api/observation"]
    if any(call not in web for call in required_ui_calls):
        fail("Ingress UI is missing required API calls")
    required_server_paths = [
        '"/api/snapshot"',
        '"/api/dashboard"',
        '"/api/observations"',
        '"/api/observation"',
    ]
    if any(path not in source for path in required_server_paths):
        fail("web server is missing required endpoints")
    if "172, 30, 32, 2" not in source:
        fail("Ingress source-address restriction is missing")

    all_text = "\n".join(
        path.read_text(encoding="utf-8", errors="ignore")
        for path in ROOT.rglob("*")
        if path.is_file() and path.suffix not in {".png", ".jpg", ".jpeg"}
    )
    secret_patterns = [
        re.compile(r"sk-(?:proj-)?[A-Za-z0-9_-]{16,}"),
        re.compile(r"gh[pousr]_[A-Za-z0-9]{20,}"),
        re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----"),
    ]
    for pattern in secret_patterns:
        if pattern.search(all_text):
            fail(f"possible secret found: {pattern.pattern}")


def main() -> int:
    checks = [
        required_paths,
        validate_manifest,
        validate_dashboard,
        validate_docs,
        validate_licensing_and_pins,
        validate_web_and_security,
    ]
    for check in checks:
        check()
        print(f"ok: {check.__name__}")
    print("repository validation passed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except AssertionError as error:
        print(f"validation failed: {error}", file=sys.stderr)
        raise SystemExit(1)
