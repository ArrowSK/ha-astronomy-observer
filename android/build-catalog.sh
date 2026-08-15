#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
ANDROID_DIR="$ROOT/android"
COMMIT=da90466031b0372c896588b85be6016c617e205b
WORK="$ANDROID_DIR/generated/upstream/openngc"
OUT="$ANDROID_DIR/generated/assets/catalog.tsv"
mkdir -p "$WORK" "$ANDROID_DIR/generated/assets"

fetch() {
  url=$1
  out=$2
  if [ ! -s "$out" ]; then
    curl --fail --silent --show-error --location "$url" -o "$out"
  fi
}

fetch "https://raw.githubusercontent.com/mattiaverga/OpenNGC/${COMMIT}/database_files/NGC.csv" "$WORK/NGC.csv"
fetch "https://raw.githubusercontent.com/mattiaverga/OpenNGC/${COMMIT}/database_files/addendum.csv" "$WORK/addendum.csv"
python3 "$ROOT/astronomy_observer/scripts/build_catalog.py" "$WORK/NGC.csv" "$WORK/addendum.csv" "$OUT"
test -s "$OUT"
echo "Android OpenNGC observing catalogue built"
