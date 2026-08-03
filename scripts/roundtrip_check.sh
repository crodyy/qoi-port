#!/usr/bin/env bash
# Throwaway evidence for ledger unit roundtrip_all_images (not a pass/fail
# authority — verify.sh is). Encodes every oracle/raw fixture, decodes it back,
# and compares pixels against the original .raw.
set -u
cd "$(dirname "${BASH_SOURCE[0]}")/.."
BIN=target/release/qoi_rust
mkdir -p tmp_verify
fail=0
for f in oracle/raw/*.raw; do
    stem=$(basename "$f" .raw)
    "$BIN" encode "oracle/raw/$stem.raw" "tmp_verify/rt_$stem.qoi" \
        && "$BIN" decode "tmp_verify/rt_$stem.qoi" "tmp_verify/rt_$stem.decoded" \
        && cmp -s "tmp_verify/rt_$stem.decoded" "oracle/raw/$stem.raw" \
        && echo "RT-OK [$stem]" || { echo "RT-FAIL [$stem]"; fail=1; }
done
exit $fail
