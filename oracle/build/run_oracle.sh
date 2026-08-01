#!/bin/bash
# QOI port kickoff: encode/decode all test images with the qoiconv oracle.
# Generated 2026-07-31. Build tooling; lives outside the locked trees.
set -u
cd /mnt/d/Code/PortMorterm
for png in oracle-source/qoi_test_images/*.png; do
  base=$(basename "$png" .png)
  echo "== $base =="
  /mnt/d/Code/PortMorterm/oracle/build/qoiconv "$png" "oracle/outputs/$base.qoi" && echo "  encoded -> oracle/outputs/$base.qoi"
  /mnt/d/Code/PortMorterm/oracle/build/qoiconv "oracle/outputs/$base.qoi" "oracle/outputs/$base.png" && mv "oracle/outputs/$base.png" "oracle/outputs/$base.decoded" && echo "  decoded -> oracle/outputs/$base.decoded"
done
echo DONE
