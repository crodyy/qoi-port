# Run Log

Every `scripts/verify.sh` run, pass or fail, appended here. No exceptions.

---

### 2026-07-31 16:59 IST — verify.sh --all — FAIL
- Command: `scripts/verify.sh --all`
- Exit code: 1
- Pipeline state: **post raw-dump migration**. verify.sh now uses `oracle/build/mkraw` (png2raw/qoi2raw) — no PNG in the loop. `oracle/raw/*.raw` fixtures generated; `oracle/outputs/*.decoded` regenerated as raw dumps (same format, byte-identical to the corresponding `.raw`); `oracle_outputs_hash.txt` re-hashed (16/16 OK); `oracle/outputs` re-locked (WSL + Windows write blocked).
- Result: FAIL [dice] encode/decode, [edgecase] encode/decode (+known channels=03-vs-04 note), [kodim10], [kodim23], [qoi_logo], [testcard], [testcard_rgba], [wikipedia_008] — 16 failures.
- Expected at this stage: yes. `src/main.rs` stub still exits 1 with "not implemented yet"; no QOI logic exists yet.
- Next: begin ledger unit work (first pending item: write_32).

### 2026-07-31 16:43 IST — verify.sh --all — FAIL
- Command: `scripts/verify.sh --all`
- Exit code: 1
- Scope: scaffolding check (CLI stub only — encode/decode unimplemented)
- Result: FAIL [dice] encode, FAIL [dice] decode, FAIL [edgecase] encode (+known discrepancy note), FAIL [edgecase] decode, FAIL [kodim10] encode/decode, FAIL [kodim23] encode/decode, FAIL [qoi_logo] encode/decode, FAIL [testcard] encode/decode, FAIL [testcard_rgba] encode/decode, FAIL [wikipedia_008] encode/decode (16 failures total)
- Expected at this stage: yes. `src/main.rs` is a placeholder CLI that prints "not implemented yet" and exits 1; no QOI logic exists yet.
- Next: begin ledger unit work (first pending item: write_32).
