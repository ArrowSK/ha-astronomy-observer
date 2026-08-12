#!/usr/bin/env python3
"""Create a local Astronomy Observer light-pollution CSV from a Falchi-style GeoTIFF."""
from __future__ import annotations

import argparse
import csv
import math
from pathlib import Path


def parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        description="Extract a local latitude/longitude/artificial-luminance grid from a GeoTIFF."
    )
    p.add_argument("--input", required=True, type=Path, help="Source GeoTIFF")
    p.add_argument("--output", required=True, type=Path, help="Output CSV")
    p.add_argument("--latitude", required=True, type=float, help="Centre latitude")
    p.add_argument("--longitude", required=True, type=float, help="Centre longitude")
    p.add_argument("--radius-km", required=True, type=float, help="Approximate extraction radius")
    p.add_argument("--step", type=int, default=1, help="Keep every Nth source pixel (default: 1)")
    return p


def main() -> int:
    args = parser().parse_args()
    if not -90 <= args.latitude <= 90 or not -180 <= args.longitude <= 180:
        raise SystemExit("invalid latitude/longitude")
    if args.radius_km <= 0 or args.radius_km > 1000:
        raise SystemExit("radius must be greater than 0 and no more than 1000 km")
    if args.step < 1 or args.step > 100:
        raise SystemExit("step must be between 1 and 100")

    try:
        import rasterio
        from rasterio.windows import Window
        from rasterio.warp import transform
    except ImportError as exc:
        raise SystemExit("Rasterio is required: python3 -m pip install rasterio") from exc

    lat_radius = args.radius_km / 111.32
    lon_scale = max(math.cos(math.radians(args.latitude)), 0.05)
    lon_radius = args.radius_km / (111.32 * lon_scale)
    min_lat, max_lat = args.latitude - lat_radius, args.latitude + lat_radius
    min_lon, max_lon = args.longitude - lon_radius, args.longitude + lon_radius

    count = 0
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with rasterio.open(args.input) as src:
        if src.count < 1:
            raise SystemExit("source raster has no bands")
        if src.crs is None:
            raise SystemExit("source raster has no CRS")

        xs, ys = transform("EPSG:4326", src.crs, [min_lon, max_lon], [min_lat, max_lat])
        row_a, col_a = src.index(xs[0], ys[0])
        row_b, col_b = src.index(xs[1], ys[1])
        row0, row1 = sorted((max(0, row_a), min(src.height - 1, row_b)))
        col0, col1 = sorted((max(0, col_a), min(src.width - 1, col_b)))
        window = Window.from_slices((row0, row1 + 1), (col0, col1 + 1))
        data = src.read(1, window=window, masked=True)
        transform_window = src.window_transform(window)

        with args.output.open("w", encoding="utf-8", newline="") as fh:
            writer = csv.writer(fh)
            writer.writerow(["latitude", "longitude", "artificial_mcd_m2"])
            for local_row in range(0, data.shape[0], args.step):
                for local_col in range(0, data.shape[1], args.step):
                    value = data[local_row, local_col]
                    if getattr(value, "mask", False):
                        continue
                    artificial = float(value)
                    if not math.isfinite(artificial) or artificial < 0:
                        continue
                    x, y = rasterio.transform.xy(
                        transform_window, local_row, local_col, offset="center"
                    )
                    lon, lat = transform(src.crs, "EPSG:4326", [x], [y])
                    if not (min_lat <= lat[0] <= max_lat and min_lon <= lon[0] <= max_lon):
                        continue
                    # Keep an approximate circle rather than the full bounding box.
                    dy = (lat[0] - args.latitude) * 111.32
                    dx = (lon[0] - args.longitude) * 111.32 * lon_scale
                    if math.hypot(dx, dy) > args.radius_km:
                        continue
                    writer.writerow([f"{lat[0]:.7f}", f"{lon[0]:.7f}", f"{artificial:.8f}"])
                    count += 1

    if count == 0:
        args.output.unlink(missing_ok=True)
        raise SystemExit("no valid pixels were written; check the raster, CRS and selected location")
    print(f"wrote {count} points to {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
