#!/bin/bash
# Generate raw pixel fixtures with the raw dump format (4-byte BE w, 4-byte BE h,
# 1-byte channels, then width*height*channels raw bytes). No PNG anywhere.
# Generated 2026-07-31. Run AFTER unlocking oracle/outputs (if replacing .decoded).
cd /mnt/d/Code/PortMorterm
set -e
mkdir -p oracle/raw
for png in oracle-source/qoi_test_images/*.png; do
  stem=$(basename "$png" .png)
  ./oracle/build/mkraw png2raw "$png" "oracle/raw/$stem.raw"
  echo "raw $stem"
done
for q in oracle/outputs/*.qoi; do
  stem=$(basename "$q" .qoi)
  ./oracle/build/mkraw qoi2raw "$q" "oracle/outputs/$stem.decoded"
  echo "decoded $stem"
done
echo MKRAW_ALL_DONE
