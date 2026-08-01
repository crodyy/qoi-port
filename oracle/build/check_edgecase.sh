#!/bin/bash
cd /mnt/d/Code/PortMorterm
file oracle-source/qoi_test_images/*.png
echo "--- qoi headers: offset 0-13 (magic w h channels colorspace) ---"
for f in oracle/outputs/edgecase.qoi oracle-source/qoi_test_images/edgecase.qoi; do
  echo "$f:"
  od -A d -t x1 -N 14 "$f"
done
echo HEADER_CHECK_DONE
