#!/usr/bin/env bash
# build-release.sh — Build normordis-pdf library and CLI tools in release mode.
#
# Usage:
#   ./scripts/bash/build-release.sh [--target <triple>] [--out-dir <dir>]
#                                      [--cargo-target-dir <dir>]
#
# Options:
#   --target <triple>   Target Rust (e.g. x86_64-pc-windows-gnu).
#                       Defaults to the host target.
#   --out-dir <dir>     Directory where the library, header and CLIs are copied.
#                       Defaults to ./dist/
#   --cargo-target-dir <dir>
#                       Rust build cache. Defaults to the local machine cache,
#                       outside the repository.
#
# Output:
#   dist/normordis_pdf.{so,dll,dylib}
#   dist/include/normordis_pdf.h
#   dist/dotx2ndt[.exe], dist/ndt-tools[.exe]

set -euo pipefail

# ── Defaults ──────────────────────────────────────────────────────────────────

TARGET=""
OUT_DIR="dist"
CARGO_TARGET_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/normordis-pdf/target"

# ── Argument parsing ──────────────────────────────────────────────────────────

while [[ $# -gt 0 ]]; do
    case "$1" in
        --target)
            [[ $# -ge 2 ]] || { echo "Missing value after --target" >&2; exit 2; }
            TARGET="$2"; shift 2 ;;
        --out-dir)
            [[ $# -ge 2 ]] || { echo "Missing value after --out-dir" >&2; exit 2; }
            OUT_DIR="$2"; shift 2 ;;
        --cargo-target-dir)
            [[ $# -ge 2 ]] || { echo "Missing value after --cargo-target-dir" >&2; exit 2; }
            CARGO_TARGET_DIR="$2"; shift 2 ;;
        -h|--help)
            sed -n '2,17p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *) echo "Unknown option: $1" >&2; exit 1 ;;
    esac
done

# ── Resolve workspace root ────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$ROOT"

# ── Build arguments ───────────────────────────────────────────────────────────

CARGO_ARGS=(--release)
TARGET_DIR="$CARGO_TARGET_DIR/release"

if [[ -n "$TARGET" ]]; then
    CARGO_ARGS+=(--target "$TARGET")
    TARGET_DIR="$CARGO_TARGET_DIR/$TARGET/release"
fi

# Derive artifact names from the requested target, not from the build host.
# This keeps Linux-hosted Windows cross builds and Windows-hosted Linux builds
# correct when collecting release artifacts.
PLATFORM="$TARGET"
if [[ -z "$PLATFORM" ]]; then
    PLATFORM="$(rustc -vV | awk '/^host: / { print $2 }')"
fi

BIN_EXT=""
LIBRARY="libnormordis_pdf.so"
case "$PLATFORM" in
    *windows*) BIN_EXT=".exe"; LIBRARY="normordis_pdf.dll" ;;
    *apple-darwin*) LIBRARY="libnormordis_pdf.dylib" ;;
esac

# ── Build ─────────────────────────────────────────────────────────────────────

echo "==> Building normordis-pdf workspace (release)…"
echo "    Cargo target directory: $CARGO_TARGET_DIR"
CARGO_TARGET_DIR="$CARGO_TARGET_DIR" cargo build "${CARGO_ARGS[@]}" \
    -p normordis-pdf \
    -p dotx2ndt \
    -p ndt-tools

# ── Copy binaries to output directory ─────────────────────────────────────────

mkdir -p "$OUT_DIR/include"

BINS=(dotx2ndt ndt-tools)
for bin in "${BINS[@]}"; do
    src="$TARGET_DIR/${bin}${BIN_EXT}"
    dst="$OUT_DIR/${bin}${BIN_EXT}"
    if [[ -f "$src" ]]; then
        cp "$src" "$dst"
        echo "    $dst  ($(du -sh "$dst" | cut -f1))"
    else
        echo "WARNING: expected binary not found: $src" >&2
    fi
done

library_src="$TARGET_DIR/$LIBRARY"
library_dst="$OUT_DIR/$LIBRARY"
if [[ -f "$library_src" ]]; then
    cp "$library_src" "$library_dst"
    echo "    $library_dst  ($(du -sh "$library_dst" | cut -f1))"
else
    echo "WARNING: expected C library not found: $library_src" >&2
fi

cp "$ROOT/normordis_pdf.h" "$OUT_DIR/include/normordis_pdf.h"
echo "    $OUT_DIR/include/normordis_pdf.h"

echo ""
echo "Done. Release artifacts for $PLATFORM in $OUT_DIR/"
