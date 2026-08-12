#!/usr/bin/env python3
"""Validate the bundled light-pollution atlas and its runtime/scoring wiring."""
from __future__ import annotations

import hashlib
import json
import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
APP = ROOT / "astronomy_observer"
ATLAS = APP / "data/world_atlas_3min.bin"
METADATA = APP / "data/world_atlas_3min.json"
NOTICE = APP / "data/WORLD_ATLAS_NOTICE.md"


def fail(message: str) -> None:
    raise AssertionError(message)


def validate_files() -> None:
    for path in [ATLAS, METADATA, NOTICE, APP / "scripts/build_global_atlas.py"]:
        if not path.exists():
            fail(f"missing bundled-atlas file: {path.relative_to(ROOT)}")


def validate_binary() -> None:
    metadata = json.loads(METADATA.read_text(encoding="utf-8"))
    size = ATLAS.stat().st_size
    if not 20_000_000 < size < 60_000_000:
        fail(f"bundled atlas size is outside the expected compact range: {size}")
    if metadata.get("bytes") != size:
        fail("bundled atlas metadata size does not match the file")
    if metadata.get("license") != "CC BY-NC 4.0":
        fail("bundled atlas metadata licence is missing or incorrect")
    if metadata.get("source_doi") != "10.5880/GFZ.1.4.2016.001":
        fail("bundled atlas source DOI is missing or incorrect")
    if metadata.get("source_reference_year") != 2015:
        fail("bundled atlas source reference year is missing or incorrect")
    cell_arcmin = float(metadata.get("cell_arcmin_nominal", 0))
    if not 2.5 <= cell_arcmin <= 3.5:
        fail("bundled atlas is not at the expected approximately 3-arcminute resolution")

    digest = hashlib.sha256()
    with ATLAS.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    if digest.hexdigest() != metadata.get("sha256"):
        fail("bundled atlas SHA-256 does not match metadata")

    with ATLAS.open("rb") as handle:
        header = handle.read(64)
    if len(header) != 64 or header[:8] != b"AOATLS1\0":
        fail("bundled atlas header magic is invalid")
    _, width, height, west, north, cell_lon, cell_lat, scale, floor = struct.unpack(
        "<8sIIdddddd", header
    )
    if width != metadata.get("width") or height != metadata.get("height"):
        fail("bundled atlas binary dimensions do not match metadata")
    if size != 64 + width * height * 2:
        fail("bundled atlas binary length does not match its dimensions")
    if not (-181 < west < -179 and 84 < north < 86):
        fail("bundled atlas geographic origin is unexpected")
    if not (0.04 < cell_lon < 0.06 and 0.04 < cell_lat < 0.06):
        fail("bundled atlas cell size is unexpected")
    if scale <= 0 or floor <= 0:
        fail("bundled atlas encoding parameters are invalid")


def validate_runtime_wiring() -> None:
    docker = (APP / "Dockerfile").read_text(encoding="utf-8")
    if "COPY data/world_atlas_3min.bin /usr/share/astronomy-observer/world_atlas_3min.bin" not in docker:
        fail("Dockerfile does not install the bundled atlas")

    light_pollution = (APP / "src/light_pollution.rs").read_text(encoding="utf-8")
    for marker in ["BUNDLED_ATLAS_PATH", "BinaryAtlas", "binary_lookup", "darker_site"]:
        if marker not in light_pollution:
            fail(f"runtime atlas lookup is missing: {marker}")

    scoring = (APP / "src/scoring.rs").read_text(encoding="utf-8")
    for marker in ["fn darkness_factor", ".sqm_mag_arcsec2", "(dark, 0.18)", "(dark, 0.24)", "(dark, 0.16)"]:
        if marker not in scoring:
            fail(f"sky brightness is not wired into the expected score path: {marker}")


def validate_attribution() -> None:
    third_party = (ROOT / "THIRD_PARTY_LICENSES.md").read_text(encoding="utf-8")
    notice = NOTICE.read_text(encoding="utf-8")
    for marker in ["10.5880/GFZ.1.4.2016.001", "CC BY-NC 4.0", "world_atlas_3min.bin"]:
        if marker not in third_party or marker not in notice:
            fail(f"World Atlas attribution is incomplete: {marker}")


def main() -> int:
    for check in [validate_files, validate_binary, validate_runtime_wiring, validate_attribution]:
        check()
        print(f"ok: {check.__name__}")
    print("bundled atlas validation passed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except AssertionError as error:
        print(f"atlas validation failed: {error}", file=sys.stderr)
        raise SystemExit(1)
