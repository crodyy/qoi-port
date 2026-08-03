# QOI → Rust Port

A Rust port of [phoboslab/qoi](https://github.com/phoboslab/qoi) — the "Quite OK Image Format"
reference C implementation — built for Port Mortem 2026 (Track: C → Rust).

This port targets byte-identical encode output and pixel-identical decode output against the
original `qoi.h`, verified through an automated oracle-diff loop, not just "it compiles and the
included tests pass."

## Build

```bash
cargo build --release
```

Single command, no other build path. Binary at `target/release/qoi_rust`.

## Verify

```bash
scripts/verify.sh --all
```

Runs the built binary's `encode`/`decode` against every image in `oracle-source/qoi_test_images/`
and byte-diffs the output against pre-generated golden files in `oracle/outputs/`. Exits 0 only if
every image matches on both encode and decode.

```bash
cargo test
```

97 unit tests, each asserting exact values/byte-order/edge cases cited against specific `qoi.h`
line numbers (see `CLAUDE.md` rule 3) — not just internal self-consistency.

## Process

This port was built with a deliberate emphasis on *proving* equivalence, not just achieving it:

1. **Kickoff hash** — `reference/qoi_hash.txt` and `oracle-source_hash.txt` were generated
   immediately after cloning, before any code was written, as proof the original source and test
   images were never modified.
2. **Oracle-first** — golden encode/decode outputs (`oracle/outputs/`) were generated from the
   real, compiled reference `qoi.h` before any Rust was written, so every unit of work had a
   ground truth to diff against from the start.
3. **Ledger-driven, one unit at a time** — `ledger.md` tracks every function/opcode as its own
   unit (`pending` → `in-progress` → `passing`/`stuck`), enforcing dependency order (e.g.
   `color_hash` is shared by encode and decode and had to pass before either could rely on it).
4. **`scripts/verify.sh` is the sole pass/fail authority** — no unit is marked done based on the
   implementing agent's own judgment; only a script exit code counts. See `CLAUDE.md` for the full
   rule set this port was built under, including an explicit anti-hardcoding rule and a 5-attempt
   retry cap per unit before escalating to `stuck`.
5. **Full audit trail** — every verification run, pass or fail, is logged in `logs/run_log.md`
   (append-only); regressions are tracked separately in `logs/regressions.md`.

### A real bug found and fixed via this process

Differential fuzz-testing (2000 random/malformed byte buffers, decoded by both the real `qoi_decode`
and this port, comparing crash behavior and output) found one genuine divergence: the Rust decoder
accepted 14–21 byte truncated streams that the reference correctly rejects (`qoi.h:500` requires
`size >= QOI_HEADER_SIZE + sizeof(qoi_padding)`, i.e. 22 bytes minimum). Zero panics, zero pixel
mismatches, and zero other divergences were found across the fuzz run — see `logs/run_log.md` and
`ledger.md`'s "Differential fuzz findings" section for the full report and fix.

## Repository layout

| Path | Purpose |
|---|---|
| `src/main.rs` | The port itself + inline unit tests |
| `ledger.md` | Unit-by-unit port status, the source of truth for scope and progress |
| `CLAUDE.md` | Rules the porting agent operated under |
| `scripts/verify.sh` | Sole pass/fail authority; encode/decode diff against the oracle |
| `oracle/outputs/`, `oracle/raw/` | Golden files generated from the real reference implementation |
| `oracle-source/qoi_test_images/` | Official QOI test image corpus |
| `logs/run_log.md`, `logs/regressions.md` | Append-only verification history |
| `command_log.md` | Chronological log of ported units |
| `reference/` | Hash files + `FETCH.md` — see below, source not embedded |

## Reference source

The original `qoi.h` is **not embedded** in this repo — see `reference/FETCH.md` for how to fetch
the exact commit this port was verified against and confirm its integrity via checksum. This is
intentional: it lets you (or a judge) pull pristine upstream independently rather than trusting a
bundled copy.

## Scope

In scope: `qoi_encode`, `qoi_decode` and all six opcodes (`QOI_OP_INDEX/DIFF/LUMA/RUN/RGB/RGBA`),
header parsing/writing, the shared color-index table, and input validation matching the reference's
`qoi_desc` checks. Out of scope: `qoiconv.c`'s PNG handling, `qoibench.c`, `qoifuzz.c` (tooling
around the format, not the format itself — see `ledger.md` for the explicit exclusion list).
