#!/bin/bash
# Hash the oracle outputs. Generated 2026-07-31.
set -e
cd /mnt/d/Code/PortMorterm
printf '# 2026-07-31\n' > oracle_outputs_hash.txt
find oracle/outputs -type f | sort | xargs sha256sum >> oracle_outputs_hash.txt
wc -l oracle_outputs_hash.txt
sha256sum -c oracle_outputs_hash.txt
echo HASH_OUTPUTS_DONE
