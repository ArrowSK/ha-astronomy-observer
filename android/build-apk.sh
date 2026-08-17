#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
ANDROID_DIR="$ROOT/android"

command -v cargo >/dev/null 2>&1 || { echo "cargo is required" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "python3 is required" >&2; exit 2; }
command -v gradle >/dev/null 2>&1 || { echo "Gradle 8.13 is required" >&2; exit 2; }

"$ANDROID_DIR/build-catalog.sh"
"$ANDROID_DIR/build-native.sh"
python3 "$ANDROID_DIR/prepare_assets.py"
python3 "$ANDROID_DIR/validate.py"

gradle -p "$ANDROID_DIR" --no-daemon clean assembleDebug lintDebug

APK="$ANDROID_DIR/app/build/outputs/apk/debug/app-debug.apk"
test -s "$APK"
mkdir -p "$ANDROID_DIR/generated/apk"
cp "$APK" "$ANDROID_DIR/generated/apk/astronomy-observer-0.3.1-debug.apk"

echo "Installable debug-signed APK: android/generated/apk/astronomy-observer-0.3.1-debug.apk"
