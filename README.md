# QOI → Rust Port

A Rust port of [phoboslab/qoi](https://github.com/phoboslab/qoi) — the "Quite OK Image Format" —
built for Port Mortem 2026 (Track: C → Rust).

**QOI** a lossless image format that compresses close to PNG but encodes/decodes
20-50x faster, because the format is small enough to fit in ~300 lines of C.

This port targets byte-identical encode output and pixel-identical decode output against the
original `qoi.h`, verified through an automated oracle-diff loop — not just "it compiles."

## Quick start

```bash
cargo build --release
target/release/qoi_rust encode oracle/raw/kodim23.raw /tmp/out.qoi
target/release/qoi_rust decode /tmp/out.qoi /tmp/decoded.raw
```

## Test stats

| Check | Result |
|---|---|
| Unit tests | 97 passing, 0 failing |
| Oracle image suite | 8/8 images, byte-identical on encode, pixel-identical on decode |
| Differential fuzz | 2000 random/malformed byte buffers, 0 panics, 0 crashes, 0 output mismatches |
| Clippy | 0 warnings (`cargo clippy --all-targets -- -D warnings`) |
| Fresh-clone build | Verified from a clean `git clone`, no local state reused |

## Bug fixes found during development

**Truncated-input leniency (fixed).** Differential fuzzing found the decoder initially accepted
14–21 byte truncated streams that the reference implementation correctly rejects — `qoi.h:500`
requires a minimum of 22 bytes (14-byte header + 8-byte end marker) before accepting a stream as
valid. The port was missing this length guard, so it would decode garbage instead of failing.
Fixed by mirroring the exact reference check; regression test added using the original fuzz-found
reproducer bytes. Zero other divergences found across the 2000-buffer fuzz run, before or after.

Full audit trail of every verification run (pass and fail) is in `logs/run_log.md`; unit-by-unit
port history is in `ledger.md` and `command_log.md`.

## Input/output format

- **Encode** takes a raw, uncompressed pixel buffer (not PNG/JPG — decoding those formats is
  explicitly out of scope for this port, see Scope below) and writes a `.qoi` file.
- **Decode** takes a `.qoi` file and writes the raw pixel buffer back out.
- Sample raw pixel files are already in `oracle/raw/` — no need to create your own to try the tool.

**Raw file format used by this tool** *(confirm exact byte layout before treating as final)*:
`<width: 4 bytes><height: 4 bytes><channels: 1 byte><raw pixel bytes, row-major, no padding>`

**QOI file format** (what `.qoi` output actually contains, per the
[official spec](https://qoiformat.org/qoi-specification.pdf)): a 14-byte header (`"qoif"` magic,
width, height, channels, colorspace), followed by a stream of variable-length opcodes encoding
runs, small deltas, index lookups, or full pixel values, ending in an 8-byte marker.

## Verify

```bash
scripts/verify.sh --all
```

Runs the binary's `encode`/`decode` against every image in `oracle-source/qoi_test_images/` and
byte-diffs the output against pre-generated golden files in `oracle/outputs/`. Exits 0 only if
every image matches on both encode and decode.

```bash
cargo test
```

97 unit tests, each asserting exact values/byte-order/edge cases cited against specific `qoi.h`
line numbers (see `CLAUDE.md` rule 3) — not just internal self-consistency.

## Process

Built with an emphasis on *proving* equivalence, not just achieving it:

- **Kickoff hash** (`reference/qoi_hash.txt`, `oracle-source_hash.txt`) taken before any code was
  written, proving the original source and test images were never modified.
- **Oracle-first** — golden encode/decode outputs (`oracle/outputs/`) generated from the real,
  compiled reference `qoi.h` before any Rust existed, so every unit had ground truth to diff against.
- **Ledger-driven** (`ledger.md`) — every function/opcode tracked as its own unit
  (`pending → in-progress → passing/stuck`), with dependency order enforced (e.g. the shared color
  hash/index table had to pass before either encode or decode could rely on it).
- **`scripts/verify.sh` is the sole pass/fail authority** — no unit is marked done on the
  implementer's own judgment, only a script exit code counts. Full rules in `CLAUDE.md`.
- **Full audit trail** — every verification run, pass or fail, logged in `logs/run_log.md`
  (append-only); regressions tracked separately in `logs/regressions.md`.
- **Differential fuzz-tested** — 2000 random/malformed byte buffers, decoded by both this port and
  the real `qoi_decode`, comparing crash behavior and output. Found and fixed one real divergence:
  the port initially accepted 14–21 byte truncated streams the reference correctly rejects
  (`qoi.h:500` requires ≥22 bytes). Zero panics, zero other mismatches across the run — see
  `ledger.md`'s "Differential fuzz findings" section.
- **Manually sanity-checked** — encode/decode run by hand on a real image, output verified against
  the QOI magic-byte header and round-tripped byte-for-byte, independent of the automated suite.

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
