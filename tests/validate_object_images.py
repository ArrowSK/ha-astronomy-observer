#!/usr/bin/env python3
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
IMAGES = ROOT / "astronomy_observer" / "data" / "object_images"
MANIFEST = IMAGES / "manifest.json"


def require(value: bool, message: str) -> None:
    if not value:
        raise AssertionError(message)


def main() -> None:
    require(MANIFEST.is_file(), "object thumbnail manifest is missing")
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    require(manifest.get("format") == "Astronomy Observer object thumbnails v1", "unexpected thumbnail manifest format")
    items = manifest.get("items", [])
    require(len(items) >= 80, "unexpectedly sparse object thumbnail bundle")
    keys = {item.get("key") for item in items}
    require("m031" in keys and "planet-saturn" in keys, "required thumbnail smoke-test targets missing")

    allowed = re.compile(r"^(Public domain|CC0(?:\b.*)?|CC BY(?:-SA)?\s)", re.I)
    for item in items:
        key = item.get("key", "")
        require(re.fullmatch(r"[a-z0-9-]+", key) is not None, f"unsafe thumbnail key: {key}")
        require(allowed.match(item.get("license", "")) is not None, f"non-allow-listed thumbnail licence for {key}")
        require(item.get("source_url", "").startswith("https://commons.wikimedia.org/wiki/File:"), f"non-Commons thumbnail source for {key}")
        require(item.get("creator") or "See source file page" in (IMAGES / "credits.html").read_text(encoding="utf-8"), f"missing creator metadata for {key}")
        image = IMAGES / item.get("local_path", "")
        require(image.is_file() and image.stat().st_size > 200, f"thumbnail missing or empty: {key}")
        require(image.read_bytes()[:4] == b"RIFF", f"thumbnail is not WebP/RIFF: {key}")

    credits = (IMAGES / "credits.html").read_text(encoding="utf-8")
    notice = (IMAGES / "NOTICE.txt").read_text(encoding="utf-8")
    for marker in ["Wikimedia Commons", "not relicensed", "CC BY", "Public domain"]:
        require(marker in credits + notice + MANIFEST.read_text(encoding="utf-8"), f"thumbnail attribution marker missing: {marker}")

    print(f"Object thumbnail validation passed ({len(items)} licensed thumbnails)")


if __name__ == "__main__":
    main()
