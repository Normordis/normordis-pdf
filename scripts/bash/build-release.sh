#!/usr/bin/env bash
# build-release.sh — Build normordis-pdf library and CLI tools in release mode.
#
# Usage:
#   ./scripts/bash/build-release.sh [--target <triple>] [--out-dir <dir>]
#
# Options:
#   --target <triple>   Cross-compile target (e.g. x86_64-unknown-linux-musl).
#                       Defaults to the host target.
#   --out-dir <dir>     Directory where binaries are copied after build.
#                       Defaults to ./dist/
#
# Output:
#   dist/dotx2ndt[.exe]
#   dist/ndt-tools[.exe]

set -euo pipefail

# ── Defaults ──────────────────────────────────────────────────────────────────

TARGET=""
OUT_DIR="dist"

# ── Argument parsing ──────────────────────────────────────────────────────────

while [[ $# -gt 0 ]]; do
    case "$1" in
        --target)   TARGET="$2"; shift 2 ;;
        --out-dir)  OUT_DIR="$2"; shift 2 ;;
        -h|--help)
            sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
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
TARGET_DIR="$ROOT/target/release"

if [[ -n "$TARGET" ]]; then
    CARGO_ARGS+=(--target "$TARGET")
    TARGET_DIR="$ROOT/target/$TARGET/release"
fi

# Detect Windows host (Git Bash / MSYS2 / Cygwin)
EXT=""
if [[ "${OS:-}" == "Windows_NT" ]] || [[ "$(uname -s)" == MINGW* ]] \
    || [[ "$(uname -s)" == CYGWIN* ]] || [[ "$(uname -s)" == MSYS* ]]; then
    EXT=".exe"
fi

# ── Build ─────────────────────────────────────────────────────────────────────

echo "==> Building normordis-pdf workspace (release)…"
cargo build "${CARGO_ARGS[@]}" \
    -p normordis-pdf \
    -p dotx2ndt \
    -p ndt-tools

# ── Copy binaries to output directory ─────────────────────────────────────────

mkdir -p "$OUT_DIR"

BINS=(dotx2ndt ndt-tools)
for bin in "${BINS[@]}"; do
    src="$TARGET_DIR/${bin}${EXT}"
    dst="$OUT_DIR/${bin}${EXT}"
    if [[ -f "$src" ]]; then
        cp "$src" "$dst"
        echo "    $dst  ($(du -sh "$dst" | cut -f1))"
    else
        echo "WARNING: expected binary not found: $src" >&2
    fi
done

echo ""
echo "Done. Binaries in $OUT_DIR/"
