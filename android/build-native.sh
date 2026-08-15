#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
ANDROID_DIR="$ROOT/android"
NDK=${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-}}
if [ -z "$NDK" ] || [ ! -d "$NDK" ]; then
  echo "ANDROID_NDK_HOME must point to Android NDK 27.0.12077973 or a compatible NDK" >&2
  exit 2
fi

case "$(uname -s)" in
  Linux) HOST=linux-x86_64 ;;
  Darwin) HOST=darwin-x86_64 ;;
  *) echo "Unsupported build host" >&2; exit 2 ;;
esac
TOOLCHAIN="$NDK/toolchains/llvm/prebuilt/$HOST"
if [ ! -d "$TOOLCHAIN" ]; then
  echo "Android NDK LLVM toolchain not found at $TOOLCHAIN" >&2
  exit 2
fi

ASTRONOMY_ENGINE_COMMIT=61dc07020aaa6885d2c7f688a4d82beaf6edb9ef
UPSTREAM="$ANDROID_DIR/generated/upstream/astronomy-engine"
mkdir -p "$UPSTREAM" "$ANDROID_DIR/generated/assets" "$ANDROID_DIR/generated/jniLibs"

fetch() {
  url=$1
  out=$2
  if [ ! -s "$out" ]; then
    curl --fail --silent --show-error --location "$url" -o "$out"
  fi
}

fetch "https://raw.githubusercontent.com/cosinekitty/astronomy/${ASTRONOMY_ENGINE_COMMIT}/source/c/astronomy.c" "$UPSTREAM/astronomy.c"
fetch "https://raw.githubusercontent.com/cosinekitty/astronomy/${ASTRONOMY_ENGINE_COMMIT}/source/c/astronomy.h" "$UPSTREAM/astronomy.h"
fetch "https://raw.githubusercontent.com/cosinekitty/astronomy/${ASTRONOMY_ENGINE_COMMIT}/LICENSE" "$ANDROID_DIR/generated/assets/astronomy-engine-license.txt"

export CARGO_TARGET_DIR="$ANDROID_DIR/generated/cargo-target"
AR="$TOOLCHAIN/bin/llvm-ar"

build_target() {
  rust_target=$1
  abi=$2
  clang_prefix=$3
  cc="$TOOLCHAIN/bin/${clang_prefix}28-clang"
  libdir="$ANDROID_DIR/generated/c/$rust_target"
  mkdir -p "$libdir" "$ANDROID_DIR/generated/jniLibs/$abi"

  "$cc" -O2 -fPIC -Wall -Wextra -Werror \
    -Wno-error=unused-parameter -Wno-error=missing-field-initializers \
    -I"$UPSTREAM" -c "$UPSTREAM/astronomy.c" -o "$libdir/astronomy.o"
  "$cc" -O2 -fPIC -Wall -Wextra -Werror \
    -I"$UPSTREAM" -c "$ANDROID_DIR/native/astro_bridge.c" -o "$libdir/astro_bridge.o"
  "$AR" rcs "$libdir/libao_android_c.a" "$libdir/astronomy.o" "$libdir/astro_bridge.o"

  case "$rust_target" in
    aarch64-linux-android)
      export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$cc"
      export CC_aarch64_linux_android="$cc"
      export AR_aarch64_linux_android="$AR"
      ;;
    x86_64-linux-android)
      export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$cc"
      export CC_x86_64_linux_android="$cc"
      export AR_x86_64_linux_android="$AR"
      ;;
  esac

  AO_ANDROID_C_LIB_DIR="$libdir" cargo build --locked \
    --manifest-path "$ANDROID_DIR/native-rust/Cargo.toml" \
    --target "$rust_target" --release
  cp "$CARGO_TARGET_DIR/$rust_target/release/libastronomy_observer_android.so" \
    "$ANDROID_DIR/generated/jniLibs/$abi/libastronomy_observer_android.so"
}

build_target aarch64-linux-android arm64-v8a aarch64-linux-android
build_target x86_64-linux-android x86_64 x86_64-linux-android

for library in \
  "$ANDROID_DIR/generated/jniLibs/arm64-v8a/libastronomy_observer_android.so" \
  "$ANDROID_DIR/generated/jniLibs/x86_64/libastronomy_observer_android.so"; do
  test -s "$library"
done

echo "Android native runtime built for arm64-v8a and x86_64"
