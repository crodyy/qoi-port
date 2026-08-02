# Run Log

Every `scripts/verify.sh` run, pass or fail, appended here. No exceptions.

---

### [color_hash] attempt 1 — PASS
- Command: `cargo test color_hash` (WSL2, exit 0) — internal unit; no verify.sh script target exists for it (AGENTS.md), done-criterion per CLAUDE.md rule 3.
- Image(s) tested: none (internal unit, tested in isolation).
- Expected: exact slots from QOI_COLOR_HASH = r*3+g*5+b*7+a*11 (qoi.h:322), slot = hash & (64-1) (qoi.h:430/577). Constants verified independently in PowerShell before baking into assertions: (0,0,0,0)->0, (255,0,0,0)->61, (0,255,0,0)->59, (0,0,255,0)->57, (0,0,0,255)->53, (255,255,255,255)->38, (0,4,0,4)->0 (wrap), (10,20,30,40)->12, weights 3/5/7/11, collision pair (2,1,0,0)==(0,0,0,1)==11.
- Actual: 6/6 color_hash tests pass.
- Hypothesis: n/a (first attempt passed).
- Fix applied: added `color_hash(r,g,b,a) -> usize` to src/main.rs computing in u32 (C int promotion, qoi.h:322) + 6 tests citing qoi.h line numbers.
- Full-suite regression check: clean — `cargo test` full run 36 passed, 0 failed.

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
