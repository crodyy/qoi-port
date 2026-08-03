#!/usr/bin/env bash
# Throwaway evidence for ledger unit roundtrip_all_qoi_bytes (not a pass/fail
# authority — verify.sh is). Encodes every oracle/raw fixture and byte-compares
# the result against oracle/outputs/<stem>.qoi.
set -u
cd "$(dirname "${BASH_SOURCE[0]}")/.."
BIN=target/release/qoi_rust
mkdir -p tmp_verify
fail=0
for f in oracle/raw/*.raw; do
    stem=$(basename "$f" .raw)
    "$BIN" encode "oracle/raw/$stem.raw" "tmp_verify/qb_$stem.qoi" \
        && cmp -s "tmp_verify/qb_$stem.qoi" "oracle/outputs/$stem.qoi" \
        && echo "QB-OK [$stem]" || { echo "QB-FAIL [$stem]"; fail=1; }
done
exit $fail
