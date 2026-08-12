#!/usr/bin/env python3
"""Reduce OpenNGC to a compact catalogue used by Astronomy Observer."""

from __future__ import annotations

import csv
import sys
from pathlib import Path

KEEP_TYPES = {"**", "OCl", "GCl", "Cl+N", "G", "PN", "HII", "EmN", "Neb", "RfN", "SNR"}


def number(value: str) -> float | None:
    try:
        return float(value) if value else None
    except ValueError:
        return None


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


def main() -> int:
    if len(sys.argv) != 4:
        print("usage: build_catalog.py NGC.csv addendum.csv output.tsv", file=sys.stderr)
        return 2
    sources = [Path(sys.argv[1]), Path(sys.argv[2])]
    output = Path(sys.argv[3])
    selected: list[dict[str, str]] = []
    seen: set[str] = set()
    for source in sources:
        for row in rows(source):
            name = row.get("Name", "").strip()
            if not name or name in seen or not useful(row):
                continue
            seen.add(name)
            selected.append(row)
    if len(selected) < 100:
        raise RuntimeError(f"unexpectedly small observing catalogue: {len(selected)} rows")

    fields = ["Name", "Type", "RA", "Dec", "Const", "MajAx", "MinAx", "B-Mag", "V-Mag", "SurfBr", "M", "Common names"]
    with output.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle, delimiter="\t", lineterminator="\n")
        writer.writerow(fields)
        for row in selected:
            writer.writerow([row.get(field, "").replace("\t", " ").replace("\n", " ") for field in fields])
    print(f"wrote {len(selected)} observing targets to {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
