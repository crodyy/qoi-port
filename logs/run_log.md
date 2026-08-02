# Run Log

Every `scripts/verify.sh` run, pass or fail, appended here. No exceptions.

---

### [clippy_cleanup] verify.sh --all — PASS
- Command: `cargo clippy --all-targets -- -D warnings` then `scripts/verify.sh --all` (WSL2, exit 0).
- Reason for run: post-completion code-quality gate (not a ledger unit). Manual check by user: no FFI (0 `extern "C"`/`#[link]`/build.rs/cc/bindgen; binary links only libc/libgcc_s), 0 `unwrap()/expect(/panic!` outside `#[cfg(test)]`.
- Clippy findings: 2 warnings. (1) `manual_range_contains` at validate_desc — `channels < 3 || channels > 4` → `!(3..=4).contains(&channels)` (behavior identical). (2) `identity_op` at decode_op_run_opcode_range — **test was previously vacuous**: `0xff & QOI_MASK_2` is `0xff & 0xc0 == 0xc0` regardless of any input, so the original assertion was mathematically always-true and never tested anything real. It was intended to prove "RGBA's tag byte masks into the RUN range, so the equality-check must dispatch first," but as written it could never fail. Replaced the literal with `QOI_OP_RGBA` (== 0xff); now the assertion genuinely pins the invariant that matters: RGBA and RUN share the `11` top-bit mask (QOI_MASK_2), and the decoder must rely on equality-check-before-mask ordering (qoi.h:552 precedes 573).
- Actual: clippy 0 warnings; `cargo test` 97 passed, 0 failed; `PASS` all 8 images; script exit 0.
- Hypothesis: n/a.
- Fix applied: (1) behavior-identical rewrite of the channels check; (2) replaced a vacuous always-true assertion with one that asserts the real dispatch-ordering invariant — a genuine (if small) test-quality improvement surfaced by clippy.
- Full-suite regression check: clean — `cargo test` 97 passed, 0 failed; `verify.sh --all` exit 0.

### [roundtrip_all_images] attempt 1 — PASS
- Command: `bash scripts/roundtrip_check.sh` (WSL2, exit 0) — chained encode->decode of every oracle/raw fixture, decoded bytes cmp'd against the original .raw. (Also implied by `verify.sh --all` exit 0 since .decoded oracle == .raw, but run explicitly for ledger evidence.)
- Image(s) tested: all 8 (dice, edgecase, kodim10, kodim23, qoi_logo, testcard, testcard_rgba, wikipedia_008).
- Expected: decoded pixels byte-identical to oracle/raw/<stem>.raw (== original PNG pixels per AGENTS.md).
- Actual: RT-OK for all 8, exit 0.
- Hypothesis: n/a (passed first attempt).
- Fix applied: none (throwaway evidence script scripts/roundtrip_check.sh created; not a pass/fail authority).
- Full-suite regression check: clean — `cargo test` 97 passed, 0 failed; `verify.sh --all` exit 0.

### [roundtrip_all_qoi_bytes] attempt 1 — PASS
- Command: `bash scripts/qoi_bytes_check.sh` (WSL2, exit 0) — encode of every oracle/raw fixture cmp'd byte-for-byte against oracle/outputs/<stem>.qoi. (Also proven by the encode checks in `verify.sh --all` exit 0.)
- Image(s) tested: all 8.
- Expected: encoded bytes byte-identical to oracle .qoi.
- Actual: QB-OK for all 8, exit 0.
- Hypothesis: n/a (passed first attempt).
- Fix applied: none (throwaway evidence script scripts/qoi_bytes_check.sh created).
- Full-suite regression check: clean — `cargo test` 97 passed, 0 failed; `verify.sh --all` exit 0.

### [encode_full] attempt 1 — PASS
- Command: `bash scripts/verify.sh --all` (WSL2) — CLI-exposed unit; done-criterion per CLAUDE.md rule 3.
- Image(s) tested: all 8 (dice, edgecase, kodim10, kodim23, qoi_logo, testcard, testcard_rgba, wikipedia_008).
- Expected (oracle bytes): encode of every oracle/raw/<stem>.raw must byte-match oracle/outputs/<stem>.qoi.
- Actual: `PASS [dice] PASS [edgecase] PASS [kodim10] PASS [kodim23] PASS [qoi_logo] PASS [testcard] PASS [testcard_rgba] PASS [wikipedia_008]`, script exit 0 (confirmed via `&&/||`: ALL_PASS). edgecase passes because edgecase.raw is 3-channel (header byte 03) matching the oracle .qoi's 03 — the documented "reference 04" was a qoiconv-pipeline artifact that does not manifest against this 3-channel raw fixture.
- Hypothesis: n/a (first attempt passed).
- Fix applied: encode_from_raw mirroring qoi.h:356-483 (raw-dump 9-byte header parse, validate_desc guard qoi.h:364-372, write_header qoi.h:384-388, run branch qoi.h:415-421, flush-before-differing-pixel qoi.h:425-428, choose_op qoi.h:430-474, px_prev update qoi.h:477, end marker qoi.h:480-482) + CLI encode branch. 3 wiring tests (byte-exact single-run file, encode->decode identity across all ops, invalid-descriptor rejection).
- Full-suite regression check: clean — `cargo test` full run 97 passed, 0 failed.
- Note: build warnings remain for QOI_LINEAR/write_end_marker (used only by tests) and an unused `p += 1` in encode_from_raw (header offset). Cosmetic, no behavioral effect.

### [encode_run] attempt 1 — PASS
- Command: `cargo test encode_run` (WSL2, exit 0) — internal unit; no verify.sh script target (AGENTS.md), done-criterion per CLAUDE.md rule 3.
- Image(s) tested: none (internal unit, tested in isolation).
- Expected: qoi.h:415-421 (px == px_prev -> run++, flush at run==62 or is_last emitting QOI_OP_RUN | (run-1)) + qoi.h:425-428 (flush pending run before a differing pixel). Chunk field = run-1 covers exactly run pixels (run=1 -> 0xc0, run=62 -> 0xfd cap). Vectors: 3 repeats then flush -> 0xc2; 61 accumulate + 62nd cap flush -> 0xfd then run restarts at 1; is_last with run=1 -> 0xc0, with run=3 -> 0xc3; empty flush noop; 62-repeat chunk after a first occurrence.
- Actual: 5/5 encode_run tests pass. (One test-assertion bug of mine mid-run: called repeat(true) after repeat(false) forgetting the is_last call also increments run, producing 0xc1; corrected.)
- Hypothesis: n/a (first attempt passed).
- Fix applied: Encoder.run field (qoi.h:357, init 0 qoi.h:395), encode_run_repeat(is_last) -> bool, encode_run_flush (qoi.h:415-428); 5 tests citing qoi.h lines.
- Full-suite regression check: clean — `cargo test` full run 94 passed, 0 failed.

### [encode_choose_op] attempt 1 — PASS
- Command: `cargo test encode_choose_op` (WSL2, exit 0) — internal unit; no verify.sh script target (AGENTS.md), done-criterion per CLAUDE.md rule 3.
- Image(s) tested: none (internal unit, tested in isolation).
- Expected: qoi.h:430-474 priority order (index hit qoi.h:432-433 > RGBA alpha-change qoi.h:468-474 > DIFF vr/vg/vb in {-2..1} qoi.h:446-451 > LUMA vg_r/vg/vg_b in range qoi.h:453-459 > RGB qoi.h:461-466); C signed-char wrap on deltas (qoi.h:439-444). All 7 transition vectors independently verified via a PowerShell simulation of the reference C before baking into tests: +1/+1/+1 -> DIFF 0x7f; -2/-1/+1 -> DIFF 0x47; wrap r 0 vs 255 -> vr=+1 -> DIFF 0x7a; LUMA [0xaa,0x33]; LUMA low-bound vg=-31 [0x81,0x88]; RGB [0xfe,30,10,20]; RGBA on alpha change [0xff,10,20,30,7]; plus index-hit-beats-RGBA -> 0x09.
- Actual: 8/8 encode_choose_op tests pass.
- Hypothesis: n/a (first attempt passed).
- Fix applied: added px_prev to Encoder (qoi.h:362, init {0,0,0,255} qoi.h:396-399) + encode_choose_op -> Vec<u8> (qoi.h:430-474); 8 tests citing qoi.h lines.
- Full-suite regression check: clean — `cargo test` full run 89 passed, 0 failed.

### [encode_index_table] attempt 1 — PASS
- Command: `cargo test encode_index_table` (WSL2, exit 0) — internal unit; no verify.sh script target (AGENTS.md), done-criterion per CLAUDE.md rule 3.
- Image(s) tested: none (internal unit, tested in isolation).
- Expected: qoi.h:393 QOI_ZEROARR(index) (all-zero table), qoi.h:430 slot = QOI_COLOR_HASH & 63, qoi.h:432 full-.v compare, qoi.h:433 Some(slot) on hit, qoi.h:436 store-on-miss (replace, no chaining). Vectors: fresh table -> {0,0,0,0} hits slot 0 (explains dice.qoi first chunk 0x00); {10,20,30,255} miss-then-hit slot 9; colliders {2,1,0,0}/{0,0,0,1} both slot 11 alternate-overwrite, neither hits (alpha participates in .v compare).
- Actual: 4/4 encode_index_table tests pass.
- Hypothesis: n/a (first attempt; one test assertion bug of mine fixed mid-run — asserted Some(11) on a 4th alternating collision lookup, corrected to None + index[11] == last-written).
- Fix applied: Encoder{index} struct (qoi.h:361), Encoder::new (qoi.h:393 zeroarr), encode_index_lookup -> Option<u8> (qoi.h:430-436); 4 tests citing qoi.h lines.
- Full-suite regression check: clean — `cargo test` full run 81 passed, 0 failed.

### [decode_full] attempt 1 — verify.sh run (decode checks green; encode stub fails as expected)
- Command: `bash scripts/verify.sh --all` (WSL2) — plus follow-up probes `verify.sh dice`, `verify.sh --all` (redirected), `bash -x verify.sh dice`, `verify.sh dice && echo PASS || echo FAIL` while checking exit-code behavior.
- Image(s) tested: all 8 (dice, edgecase, kodim10, kodim23, qoi_logo, testcard, testcard_rgba, wikipedia_008).
- Expected (oracle pixels): decode of every oracle .qoi must byte-match oracle/outputs/<stem>.decoded.
- Actual: zero `FAIL [<stem>] decode` lines — all 8 decode checks PASS, including edgecase. All 8 `FAIL [<stem>] encode` lines present (encode is still the CLI stub that exits FAILURE without writing output) — expected at this stage.
- Hypothesis: n/a (decode side correct on first full run; 4 wiring tests also green: all-ops handcrafted stream, 3-channel output, short-input reject qoi.h:500, exhausted-stream px repeat qoi.h:544).
- Fix applied: decode_to_raw mirroring qoi.h:488-590 (header validation via parse_header, Decoder dispatch with RGB/RGBA equality checks before mask checks per qoi.h:547/552, run-skip per qoi.h:541-543, per-op qoi.h:577 index write via op handlers) + `decode` CLI branch writing the 9-byte-header raw dump.
- Exit-code note: script exit is nonzero (encode FAILs). An earlier probe printed EXIT=0 — false alarm caused by reading $? through the PowerShell→WSL pipe (documented AGENTS.md gotcha); `bash -x` trace shows `exit 1` and `&&/||` probe confirms VERIFY_DICE_FAIL. Script authority is sound.
- Full-suite regression check: clean — `cargo test` full run 77 passed, 0 failed.
- Done-criterion caveat: CLAUDE.md rule 3 requires verify.sh exit 0 for CLI-exposed units, which is impossible for decode_full while the encode stub fails every encode check. RESOLVED: user approved marking decode_full passing on the strength of all-green decode checks, with the caveat noted in ledger.md.

### [decode_op_rgba] attempt 1 — PASS
- Command: `cargo test decode_op_rgba` (WSL2, exit 0) — internal unit; no verify.sh script target (AGENTS.md), done-criterion per CLAUDE.md rule 3.
- Image(s) tested: none (internal unit, tested in isolation).
- Expected: QOI_OP_RGBA per qoi.h:552-557 — 0xff (qoi.h:318) followed by literal r/g/b/a; only op that can set alpha (fresh decoder (0,0,0,0) -> true zero pixel); qoi.h:577 table write: {200,100,50,25} -> slot 1725%64 = 61; dispatch order: 0xff & QOI_MASK_2 == QOI_OP_RUN (would read as run of 63) so the 0xff equality check must precede mask checks (qoi.h:552 before 558-573).
- Actual: 4/4 decode_op_rgba tests pass.
- Hypothesis: n/a (first attempt passed).
- Fix applied: Decoder::decode_op_rgba(r,g,b,a); 4 tests citing qoi.h lines. QOI_OP_RGBA dead-code warning now resolved.
- Full-suite regression check: clean — `cargo test` full run 73 passed, 0 failed.

### [decode_op_rgb] attempt 1 — PASS
- Command: `cargo test decode_op_rgb` (WSL2, exit 0) — internal unit; no verify.sh script target (AGENTS.md), done-criterion per CLAUDE.md rule 3.
- Image(s) tested: none (internal unit, tested in isolation).
- Expected: QOI_OP_RGB per qoi.h:547-551 — 0xfe (qoi.h:317) followed by literal r/g/b; alpha not in stream, unchanged (fresh decoder: (0,0,0) -> {0,0,0,255}, not the ZEROARR {0,0,0,0}); qoi.h:577 table write: {200,100,50,255} -> slot 4255%64 = 31; dispatch order: 0xfe & QOI_MASK_2 == QOI_OP_RUN so the 0xfe equality check must precede mask checks (qoi.h:547 before 558-573).
- Actual: 5/5 decode_op_rgb tests pass.
- Hypothesis: n/a (first attempt passed).
- Fix applied: added QOI_OP_RGB/QOI_OP_RGBA consts (qoi.h:317/318) + Decoder::decode_op_rgb(r,g,b); 5 tests citing qoi.h lines. (One dead-code warning on QOI_OP_RGBA until its unit lands next — expected.)
- Full-suite regression check: clean — `cargo test` full run 69 passed, 0 failed.

### [decode_op_run] attempt 1 — PASS
- Command: `cargo test decode_op_run` (WSL2, exit 0) — internal unit; no verify.sh script target (AGENTS.md), done-criterion per CLAUDE.md rule 3.
- Image(s) tested: none (internal unit, tested in isolation).
- Expected: QOI_OP_RUN per qoi.h:573-575 (run = b1 & 0x3f, px unchanged) + loop-top skip qoi.h:541-543 (repeat px, run--, no chunk read, NO qoi.h:577 index write) + the subtlety that the RUN-chunk pixel DOES get the qoi.h:577 write (branch inside the else-if, qoi.h:544). Run field 0..=61 (encoder emits run-1, cap 62: qoi.h:417-418); 0xfe/0xff match the mask but dispatch as RGB/RGBA first (qoi.h:547/552). Vectors: 0xc0->run 0, 0xc5->5, 0xfd->61; 0xc2 = 3 total copies; RUN chunk writes px {0,0,0,255} to slot 53 over ZEROARR {0,0,0,0}.
- Actual: 5/5 decode_op_run tests pass.
- Hypothesis: n/a (first attempt passed).
- Fix applied: added QOI_OP_RUN const (qoi.h:316), run: u8 field to Decoder (qoi.h:495, init 0), decode_op_run (sets run + qoi.h:577 write), decode_run_skip (qoi.h:541-543, no table touch); 5 tests citing qoi.h lines.
- Full-suite regression check: clean — `cargo test` full run 64 passed, 0 failed.

### [decode_op_luma] attempt 1 — PASS
- Command: `cargo test decode_op_luma` (WSL2, exit 0) — internal unit; no verify.sh script target (AGENTS.md), done-criterion per CLAUDE.md rule 3.
- Image(s) tested: none (internal unit, tested in isolation).
- Expected: QOI_OP_LUMA per qoi.h:566-572 — vg = (b1&0x3f)-32 in [-32,31] to green; r/b get vg + 4-bit nibble of b2, bias -8 (in [-8,7]); alpha untouched; C uint8_t wrap. Vectors verified by independent PS simulation of the reference math: (0,0,0,255)+0x80/0x00 -> (216,224,216,255); +0xbf/0xff -> (38,31,38,255); (100,100,100,255)+0xaa/0x3b -> (105,110,113,255); (250,250,250,255)+0xbf/0xff -> (32,25,32,255); (50,60,70,9)+0x81/0x00 -> (11,29,31,9); zero-delta b1=0xa0/b2=0x88 (emit form qoi.h:458-459); post-op table write qoi.h:577: {38,31,38,255} -> slot 3340%64 = 12.
- Actual: 8/8 decode_op_luma tests pass.
- Hypothesis: n/a (first attempt passed).
- Fix applied: added QOI_OP_LUMA const (qoi.h:315) + Decoder::decode_op_luma(b1,b2) with i8 deltas and wrapping_add; 8 tests citing qoi.h lines.
- Full-suite regression check: clean — `cargo test` full run 59 passed, 0 failed.

### [decode_op_diff] attempt 1 — PASS
- Command: `cargo test decode_op_diff` (WSL2, exit 0) — internal unit; no verify.sh script target (AGENTS.md), done-criterion per CLAUDE.md rule 3.
- Image(s) tested: none (internal unit, tested in isolation).
- Expected: QOI_OP_DIFF per qoi.h:561-565 — 01_dr_dg_db, 2-bit fields, bias -2, alpha untouched, C uint8_t wrap (0 + -2 = 254, 255 + 1 = 0); emit inverse qoi.h:451, eligibility qoi.h:446-450, opcode range qoi.h:314/320. Vectors: 0x40->{254,254,254,255} from {0,0,0,255}; 0x7f->{1,1,1,255}; 0x6a zero-deltas; 0x74 = +1/-1/-2 from {10,20,30,255} -> {11,19,28,255}; post-op table write qoi.h:577 observable: {1,1,1,255} -> slot 2820%64 = 4 retrievable via INDEX.
- Actual: 8/8 decode_op_diff tests pass.
- Hypothesis: n/a (first attempt passed).
- Fix applied: added QOI_OP_DIFF const (qoi.h:314) + Decoder::decode_op_diff using i8 deltas with wrapping_add (C uint8_t wrap semantics); 8 tests citing qoi.h lines.
- Full-suite regression check: clean — `cargo test` full run 51 passed, 0 failed.

### [decode_op_index] attempt 1 — PASS
- Command: `cargo test decode_op_index` (WSL2, exit 0) — internal unit; no verify.sh script target (AGENTS.md), done-criterion per CLAUDE.md rule 3.
- Image(s) tested: none (internal unit, tested in isolation).
- Expected: QOI_OP_INDEX semantics per qoi.h:558-559 (px = index[b1], b1's top bits 00 so b1 IS the slot; qoi.h:313/320), fresh-table state per qoi.h:533/310 (QOI_ZEROARR) + qoi.h:534-537 (px={0,0,0,255}), post-op table write per qoi.h:577. Seed slots precomputed via qoi.h:322 formula and verified independently in PowerShell: {10,20,30,255}->9, {200,100,50,255}->31, {0,2,0,255}->63.
- Actual: 7/7 decode_op_index tests pass.
- Hypothesis: n/a (first attempt passed).
- Fix applied: added Px struct, Decoder{index,px} (qoi.h:492-495), Decoder::new (qoi.h:533-537), index_update (qoi.h:577, kept inside op handlers so decode_full can't skip it), decode_op_index (qoi.h:558-559), consts QOI_OP_INDEX (qoi.h:313) + QOI_MASK_2 (qoi.h:320); 7 tests citing qoi.h lines. Built on the passing color_hash — no inline hash duplication.
- Full-suite regression check: clean — `cargo test` full run 43 passed, 0 failed.

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
