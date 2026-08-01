# Project: QOI (phoboslab/qoi) → Rust port

## Objective
Port qoi.h's encode/decode logic to Rust. Behavior must be byte-identical (encode) and pixel-identical (decode) to the reference for every image in oracle-source/qoi_test_images. Not "make it compile" — "prove equivalence."

## Non-negotiable rules
1. Never edit, move, rename, or delete anything under `reference/qoi/`, `oracle-source/`, or `oracle/outputs/`. All three are read-only (filesystem-enforced, not just by instruction). If something there looks wrong, log it in `logs/questions.md` and move on.
2. Never special-case a specific test image's filename or literal bytes to force a match. If you can't solve a case, log it as stuck — do not hardcode around it.
3. You do not decide when a task is done.
    CLI-exposed units (anything reachable through encode/decode on the full binary — header_parse, header_write, encode_full, decode_full, roundtrip units) are done only when scripts/verify.sh <unit> exits 0.
    Internal units not reachable from the CLI on their own (write_32, read_32, color_hash, encode_choose_op, individual decode_op_* cases before they're wired into decode_full) are done only when cargo test <unit> passes, AND the test cases assert the exact values/byte order/edge cases specified by the reference implementation in qoi.h — not merely that the code is internally consistent with itself. Cite the qoi.h line number each assertion is checking against in a comment.
    Either way, paste the full real command output verbatim. Do not summarize it as "passing" in your own words.
4. Scope is exactly the units listed in `ledger.md`. Do not port `qoiconv.c`'s PNG handling, `qoibench.c`, or `qoifuzz.c` — these are tooling around the format, not the format itself.
5. One unit of work per turn = one ledger item marked `in-progress`. Do not start a second before the current one is `passing` or `stuck`.
6. Max 5 attempts per unit. On attempt 5 failing, mark `stuck` in `ledger.md` with your hypothesis, move to the next `pending` item.
7. After every `verify.sh` run — pass or fail — append one entry to `logs/run_log.md`. Every run, no exceptions.
8. If a `passing` unit fails again later, that's a regression — log to `logs/regressions.md`, treat as higher priority than any `pending` item.
9. Known, expected discrepancy: `edgecase.qoi`'s header channels field (03 vs reference 04) is a documented artifact of the oracle-generation tool, not a bug to fix or hide. Do not special-case around it either — if your port produces a *different* mismatch on this file, that's real and must be logged normally.

## Rust CLI contract (verify.sh depends on this exactly)
```
your_bin encode <input.raw> <output.qoi>
your_bin decode <input.qoi> <output.decoded>
```
The pixel interchange format is a **raw dump**, never PNG: 4-byte big-endian width, 4-byte big-endian height, 1-byte channels (3 or 4), then `width*height*channels` bytes of pixel data, row-major, uncompressed. Encode fixtures are `oracle/raw/*.raw`; decode oracles are `oracle/outputs/*.decoded` (same format). PNG reading/writing is explicitly out of scope — nothing in the loop touches PNG.
Binary name/path must match `RUST_BIN` in `scripts/verify.sh` — update one, update the other, don't let them drift silently.

## Workflow loop (repeat until ledger has no pending or in-progress items)
1. Read `ledger.md`, pick first `pending` item, mark `in-progress`.
2. Read `research/spec.md` and the relevant opcode section for that item.
3. Write/modify Rust code for that unit only.
4. Run `scripts/verify.sh <unit>`. Paste full output.
5. Exit 0 → mark `passing`, log success, go to 1. Nonzero → log failure with diff, hypothesis, fix, go to 3 (or mark `stuck` if this was attempt 5).

## Logging protocol — every entry needs these fields
```
### [unit] attempt [N] — [PASS|FAIL|STUCK]
- Image(s) tested: ...
- Expected (oracle bytes/pixels): ...
- Actual (port output): ...
- Hypothesis: ...
- Fix applied: ...
- Full-suite regression check: [clean | N regressions — see regressions.md]
```

## Definition of done
Every `ledger.md` item is `passing` or `stuck` with a logged reason. `scripts/verify.sh --all` exits 0 (except the documented edgecase.qoi discrepancy). `logs/regressions.md` is empty.

## Build
Single command: `cargo build --release` from repo root. No second build path.
