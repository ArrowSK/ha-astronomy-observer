#!/usr/bin/env python3
"""Build a compact location-lookup grid from the Falchi World Atlas GeoTIFF."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import struct
from pathlib import Path

MAGIC = b"AOATLS1\0"
HEADER = struct.Struct("<8sIIdddddd")
NODATA = 0xFFFF
LOG_SCALE = 8000.0
RADIANCE_FLOOR = 0.0001


def encode_radiance(value: float) -> int:
    if not math.isfinite(value) or value < 0:
        return NODATA
    code = round(math.log10(1.0 + value / RADIANCE_FLOOR) * LOG_SCALE)
    return max(0, min(NODATA - 1, code))


def decode_radiance(code: int) -> float | None:
    if code == NODATA:
        return None
    return RADIANCE_FLOOR * (10.0 ** (code / LOG_SCALE) - 1.0)


def self_test() -> None:
    if HEADER.size != 64:
        raise AssertionError(f"unexpected atlas header size: {HEADER.size}")
    for value in (0.0, 0.01, 0.174, 1.0, 10.0, 100.0):
        code = encode_radiance(value)
        decoded = decode_radiance(code)
        if decoded is None:
            raise AssertionError("valid radiance encoded as no-data")
        tolerance = max(0.00002, value * 0.001)
        if abs(decoded - value) > tolerance:
            raise AssertionError(
                f"radiance round-trip too inaccurate: {value} -> {code} -> {decoded}"
            )
    if decode_radiance(NODATA) is not None:
        raise AssertionError("no-data code should decode to None")


def build(input_path: Path, output_path: Path, metadata_path: Path, factor: int) -> None:
    if factor < 1:
        raise ValueError("factor must be at least 1")

    import numpy as np
    import rasterio
    from rasterio.enums import Resampling

    with rasterio.open(input_path) as src:
        if src.count < 1:
            raise RuntimeError("source GeoTIFF has no raster band")
        if src.width < 1000 or src.height < 1000:
            raise RuntimeError("source raster is unexpectedly small")
        if src.transform.b != 0 or src.transform.d != 0:
            raise RuntimeError("rotated source rasters are not supported")

        out_width = math.ceil(src.width / factor)
        out_height = math.ceil(src.height / factor)
        data = src.read(
            1,
            out_shape=(out_height, out_width),
            resampling=Resampling.average,
            masked=True,
        )
        west, south, east, north = src.bounds
        cell_lon = (east - west) / out_width
        cell_lat = (north - south) / out_height

    values = np.asarray(data.filled(np.nan), dtype=np.float64)
    valid = np.isfinite(values) & (values >= 0.0)
    codes = np.full(values.shape, NODATA, dtype="<u2")
    if valid.any():
        encoded = np.rint(
            np.log10(1.0 + values[valid] / RADIANCE_FLOOR) * LOG_SCALE
        )
        encoded = np.clip(encoded, 0, NODATA - 1).astype(np.uint16)
        codes[valid] = encoded

    output_path.parent.mkdir(parents=True, exist_ok=True)
    header = HEADER.pack(
        MAGIC,
        out_width,
        out_height,
        float(west),
        float(north),
        float(cell_lon),
        float(cell_lat),
        LOG_SCALE,
        RADIANCE_FLOOR,
    )
    with output_path.open("wb") as handle:
        handle.write(header)
        handle.write(codes.tobytes(order="C"))

    expected_size = HEADER.size + out_width * out_height * 2
    actual_size = output_path.stat().st_size
    if actual_size != expected_size:
        raise RuntimeError(
            f"atlas size mismatch: expected {expected_size}, wrote {actual_size}"
        )

    digest = hashlib.sha256()
    with output_path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    sha256 = digest.hexdigest()

    metadata = {
        "format": "Astronomy Observer atlas grid v1",
        "source": "Falchi et al. (2016), New World Atlas of Artificial Night Sky Brightness",
        "source_doi": "10.5880/GFZ.1.4.2016.001",
        "source_reference_year": 2015,
        "source_grid_arcsec": 30,
        "license": "CC BY-NC 4.0",
        "downsample_factor": factor,
        "width": out_width,
        "height": out_height,
        "west": west,
        "south": south,
        "east": east,
        "north": north,
        "cell_lon_deg": cell_lon,
        "cell_lat_deg": cell_lat,
        "cell_arcmin_nominal": cell_lon * 60.0,
        "encoding": {
            "type": "uint16 logarithmic artificial zenith luminance",
            "unit": "mcd/m2",
            "no_data": NODATA,
            "log_scale": LOG_SCALE,
            "radiance_floor": RADIANCE_FLOOR,
        },
        "bytes": actual_size,
        "sha256": sha256,
    }
    metadata_path.write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")

    print(
        f"wrote {out_width}x{out_height} atlas ({actual_size / 1024 / 1024:.1f} MiB), "
        f"cell {cell_lon * 60:.2f} x {cell_lat * 60:.2f} arcmin"
    )
    print(f"sha256 {sha256}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--metadata", type=Path)
    parser.add_argument("--factor", type=int, default=6)
    parser.add_argument("--self-test", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    self_test()
    if args.self_test:
        print("atlas builder self-test passed")
        return 0
    if args.input is None or args.output is None or args.metadata is None:
        raise SystemExit("--input, --output and --metadata are required")
    build(args.input, args.output, args.metadata, args.factor)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
