#!/usr/bin/env bash
# verify.sh — sole authority on pass/fail. Agent must never self-report success;
# this script's exit code is the only thing that counts.
#
# Usage:
#   scripts/verify.sh --all              # check every image against both .qoi and .decoded oracle
#   scripts/verify.sh <image_stem>       # check one image only, e.g. `scripts/verify.sh dice`
#
# NOTE ON INPUT FORMAT:
# No PNG anywhere in this loop. Everything is a "raw dump": 4-byte big-endian width,
# 4-byte big-endian height, 1-byte channels (3 or 4), then width*height*channels bytes
# of pixel data, row-major, uncompressed. Encode fixtures live in oracle/raw/*.raw
# (generated from the PNGs with oracle/build/mkraw once, cached); decode oracles are
# oracle/outputs/*.decoded in the same format. Your Rust port handles raw dumps only:
#   encode reads a .raw and writes .qoi; decode reads a .qoi and writes .decoded.

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ORACLE_OUT="$ROOT/oracle/outputs"
ORACLE_SRC="$ROOT/oracle-source/qoi_test_images"
RAW_DIR="$ROOT/oracle/raw"          # pre-decoded raw RGBA, generated once, cached
BUILD_DIR="$ROOT/target/release"
RUST_BIN="$BUILD_DIR/qoi_rust"      # adjust to match your actual Cargo package/bin name
FAIL=0

mkdir -p "$RAW_DIR" "$ROOT/tmp_verify" "$ROOT/oracle/build"

echo "== Building Rust port =="
cargo build --release --manifest-path "$ROOT/Cargo.toml" || { echo "BUILD FAILED"; exit 1; }

# Ensure the raw-fixture generator is built
if [ ! -x "$ROOT/oracle/build/mkraw" ]; then
  echo "== Building oracle/build/mkraw =="
  gcc "$ROOT/scripts/mkraw.c" -std=c99 -O2 \
      -I"$ROOT/oracle/stb" -I"$ROOT/reference/qoi" \
      -o "$ROOT/oracle/build/mkraw" || { echo "MKRAW BUILD FAILED"; exit 1; }
fi

# One-time: ensure raw RGBA fixtures exist for every test image
if [ -z "$(ls -A "$RAW_DIR" 2>/dev/null)" ]; then
  echo "== Generating raw RGBA fixtures into oracle/raw (one-time) =="
  for png in "$ORACLE_SRC"/*.png; do
    stem="$(basename "$png" .png)"
    "$ROOT/oracle/build/mkraw" png2raw "$png" "$RAW_DIR/$stem.raw" || { echo "RAW GEN FAILED [$stem]"; exit 1; }
  done
fi

check_one() {
  local stem="$1"
  local ok=1

  # ENCODE CHECK: raw pixels -> rust encode -> compare bytes to oracle .qoi
  "$RUST_BIN" encode "$RAW_DIR/$stem.raw" "$ROOT/tmp_verify/$stem.qoi" 2>"$ROOT/tmp_verify/$stem.encode.err"
  if ! cmp -s "$ROOT/tmp_verify/$stem.qoi" "$ORACLE_OUT/$stem.qoi"; then
    echo "FAIL [$stem] encode: byte mismatch vs oracle/outputs/$stem.qoi"
    if [ "$stem" = "edgecase" ]; then
      echo "  (known reference discrepancy: channels=03 vs 04 — confirm this is the SAME expected mismatch, not a new one)"
    fi
    ok=0
  fi

  # DECODE CHECK: oracle .qoi -> rust decode -> compare pixels to oracle .decoded
  "$RUST_BIN" decode "$ORACLE_OUT/$stem.qoi" "$ROOT/tmp_verify/$stem.decoded" 2>"$ROOT/tmp_verify/$stem.decode.err"
  if ! cmp -s "$ROOT/tmp_verify/$stem.decoded" "$ORACLE_OUT/$stem.decoded"; then
    echo "FAIL [$stem] decode: pixel mismatch vs oracle/outputs/$stem.decoded"
    ok=0
  fi

  if [ "$ok" -eq 1 ]; then
    echo "PASS [$stem]"
  else
    FAIL=1
  fi
}

if [ "${1:-}" = "--all" ]; then
  for f in "$ORACLE_OUT"/*.qoi; do
    check_one "$(basename "$f" .qoi)"
  done
else
  check_one "$1"
fi

exit $FAIL
