#!/usr/bin/env python3
"""Reduce OpenNGC to the compact catalogue used by Astronomy Observer."""

from __future__ import annotations

import csv
import math
import sys
from pathlib import Path

KEEP_TYPES = {"**", "OCl", "GCl", "Cl+N", "G", "PN", "HII", "EmN", "Neb", "RfN", "SNR"}
OUTPUT_FIELDS = [
    "name",
    "type",
    "ra_deg",
    "dec_deg",
    "constellation",
    "v_mag",
    "b_mag",
    "surface_brightness",
    "major_arcmin",
    "minor_arcmin",
    "messier",
    "common_names",
]


def number(value: str) -> float | None:
    try:
        return float(value) if value else None
    except ValueError:
        return None


def hms_to_deg(value: str) -> float | None:
    """Convert OpenNGC HH:MM:SS.SS right ascension to decimal degrees."""
    try:
        hours, minutes, seconds = (float(part) for part in value.strip().split(":"))
    except (ValueError, TypeError):
        return None
    if not (0.0 <= hours < 24.0 and 0.0 <= minutes < 60.0 and 0.0 <= seconds < 60.0):
        return None
    degrees = 15.0 * (hours + minutes / 60.0 + seconds / 3600.0)
    return degrees if math.isfinite(degrees) else None


def dms_to_deg(value: str) -> float | None:
    """Convert OpenNGC +/-DD:MM:SS.SS declination to decimal degrees."""
    text = value.strip()
    if not text:
        return None
    sign = -1.0 if text.startswith("-") else 1.0
    text = text.lstrip("+-")
    try:
        degrees, minutes, seconds = (float(part) for part in text.split(":"))
    except (ValueError, TypeError):
        return None
    if not (0.0 <= degrees <= 90.0 and 0.0 <= minutes < 60.0 and 0.0 <= seconds < 60.0):
        return None
    result = sign * (degrees + minutes / 60.0 + seconds / 3600.0)
    if not (-90.0 <= result <= 90.0) or not math.isfinite(result):
        return None
    return result


def useful(row: dict[str, str]) -> bool:
    if row.get("Type") not in KEEP_TYPES:
        return False
    if row.get("M"):
        return True
    vmag = number(row.get("V-Mag", ""))
    bmag = number(row.get("B-Mag", ""))
    mag = vmag if vmag is not None else bmag
    major = number(row.get("MajAx", ""))
    if mag is not None and mag <= 13.5:
        return True
    if major is not None and major >= 8.0 and (mag is None or mag <= 15.0):
        return True
    return False


def rows(path: Path):
    with path.open(newline="", encoding="utf-8") as handle:
        yield from csv.DictReader(handle, delimiter=";")


def catalogue_row(row: dict[str, str]) -> list[str] | None:
    ra = hms_to_deg(row.get("RA", ""))
    dec = dms_to_deg(row.get("Dec", ""))
    if ra is None or dec is None:
        return None
    return [
        row.get("Name", "").strip(),
        row.get("Type", "").strip(),
        f"{ra:.8f}",
        f"{dec:.8f}",
        row.get("Const", "").strip(),
        row.get("V-Mag", "").strip(),
        row.get("B-Mag", "").strip(),
        row.get("SurfBr", "").strip(),
        row.get("MajAx", "").strip(),
        row.get("MinAx", "").strip(),
        row.get("M", "").strip(),
        row.get("Common names", "").replace("\t", " ").replace("\n", " ").strip(),
    ]


def self_test() -> int:
    assert abs((hms_to_deg("12:00:00.00") or 0.0) - 180.0) < 1e-9
    assert abs((hms_to_deg("00:30:00.00") or 0.0) - 7.5) < 1e-9
    assert abs((dms_to_deg("+27:30:00.00") or 0.0) - 27.5) < 1e-9
    assert abs((dms_to_deg("-29:00:28.10") or 0.0) + 29.00780556) < 1e-6
    sample = {
        "Name": "NGC0001",
        "Type": "G",
        "RA": "00:07:15.84",
        "Dec": "+27:42:29.10",
        "Const": "Peg",
        "V-Mag": "12.86",
        "B-Mag": "13.65",
        "SurfBr": "21.49",
        "MajAx": "1.57",
        "MinAx": "1.07",
        "M": "",
        "Common names": "",
    }
    converted = catalogue_row(sample)
    assert converted is not None and len(converted) == len(OUTPUT_FIELDS)
    assert 0.0 <= float(converted[2]) < 360.0
    assert -90.0 <= float(converted[3]) <= 90.0
    assert converted[5] == "12.86"
    assert converted[7] == "21.49"
    assert converted[8] == "1.57"
    print("catalogue builder self-test passed")
    return 0


def main() -> int:
    if len(sys.argv) == 2 and sys.argv[1] == "--self-test":
        return self_test()
    if len(sys.argv) != 4:
        print("usage: build_catalog.py NGC.csv addendum.csv output.tsv", file=sys.stderr)
        return 2

    sources = [Path(sys.argv[1]), Path(sys.argv[2])]
    output = Path(sys.argv[3])
    selected: list[list[str]] = []
    seen: set[str] = set()
    skipped_coordinates = 0

    for source in sources:
        for row in rows(source):
            name = row.get("Name", "").strip()
            if not name or name in seen or not useful(row):
                continue
            converted = catalogue_row(row)
            if converted is None:
                skipped_coordinates += 1
                continue
            seen.add(name)
            selected.append(converted)

    if len(selected) < 100:
        raise RuntimeError(f"unexpectedly small observing catalogue: {len(selected)} rows")

    with output.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle, delimiter="\t", lineterminator="\n")
        writer.writerow(OUTPUT_FIELDS)
        writer.writerows(selected)

    print(
        f"wrote {len(selected)} observing targets to {output}"
        + (f"; skipped {skipped_coordinates} rows without usable coordinates" if skipped_coordinates else "")
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
