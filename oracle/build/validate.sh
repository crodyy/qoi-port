#!/bin/bash
# Validate oracle outputs: decoded PNG magic + qoi encode parity with reference. Generated 2026-07-31.
cd /mnt/d/Code/PortMorterm
echo "--- decoded magic bytes (expect 8950 4e47 = PNG) ---"
for f in oracle/outputs/*.decoded; do
  magic=$(od -A n -t x1 -N 4 "$f" | tr -d ' \n')
  echo "$f -> $magic"
done
echo "--- qoi encode vs official reference (cmp) ---"
for q in oracle-source/qoi_test_images/*.qoi; do
  b=$(basename "$q")
  if cmp -s "oracle/outputs/$b" "$q"; then echo "MATCH $b"; else echo "DIFF  $b"; fi
done
echo VALIDATE_DONE
