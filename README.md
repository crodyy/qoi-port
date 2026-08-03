# QOI Rust Port (Porting Hackathon)

This project is a **Rust port** of the original QOI reference implementation from [phoboslab/qoi](https://github.com/phoboslab/qoi).

QOI (Quite OK Image) is a compact, lossless image format designed to be much simpler than PNG while still keeping good compression speed and quality for many image types.

## What this tool does

This repository provides a command-line tool that can:
- **Encode** raw pixel dumps into `.qoi`
- **Decode** `.qoi` files back into raw pixel dumps

The port is built to mirror the C reference behavior as closely as possible:
- **Encode target:** byte-identical output
- **Decode target:** pixel-identical output

## Quick start (first-time users)

From the repository root:
`/home/runner/work/qoi-port/qoi-port`

1. Build the binary:
   ```bash
   cargo build --release
   ```
2. Run tests:
   ```bash
   cargo test
   ```
3. Run full oracle verification:
   ```bash
   /home/runner/work/qoi-port/qoi-port/scripts/verify.sh --all
   ```

If verification succeeds, your binary matches expected behavior on the official test corpus used in this repository.

## CLI usage

Binary location:
- `/home/runner/work/qoi-port/qoi-port/target/release/qoi_rust`

Command contract:
```bash
qoi_rust encode <input.raw> <output.qoi>
qoi_rust decode <input.qoi> <output.decoded>
```

## Input/output format (important)

This project uses a **raw dump format** (not PNG I/O in the CLI):

1. 4-byte big-endian width
2. 4-byte big-endian height
3. 1-byte channels (`3` or `4`)
4. `width * height * channels` bytes of row-major pixel data

So:
- `encode` reads `.raw`, writes `.qoi`
- `decode` reads `.qoi`, writes `.decoded` (same raw layout)

## Example with included test assets

Encode one provided sample:
```bash
/home/runner/work/qoi-port/qoi-port/target/release/qoi_rust \
  encode \
  /home/runner/work/qoi-port/qoi-port/oracle/raw/dice.raw \
  /home/runner/work/qoi-port/qoi-port/tmp_verify/dice.qoi
```

Decode the oracle QOI sample:
```bash
/home/runner/work/qoi-port/qoi-port/target/release/qoi_rust \
  decode \
  /home/runner/work/qoi-port/qoi-port/oracle/outputs/dice.qoi \
  /home/runner/work/qoi-port/qoi-port/tmp_verify/dice.decoded
```

## Hackathon correctness note

For this porting hackathon, **exact test-case and edge-case behavior matters**.

- `scripts/verify.sh` is the pass/fail authority.
- Oracle files in `oracle/outputs` are treated as fixed expected outputs.
- Edge cases must match reference behavior exactly; avoid “close enough” changes.

Known expected quirk:
- `edgecase.qoi` has a documented channel-field mismatch (`03` vs `04`) from historical oracle generation. Keep this behavior documented; do not special-case file-specific logic.

## Project paths you will use most

- `/home/runner/work/qoi-port/qoi-port/src/main.rs` — Rust implementation
- `/home/runner/work/qoi-port/qoi-port/scripts/verify.sh` — full verification script
- `/home/runner/work/qoi-port/qoi-port/oracle/raw` — raw encode fixtures
- `/home/runner/work/qoi-port/qoi-port/oracle/outputs` — expected `.qoi` and `.decoded` outputs
- `/home/runner/work/qoi-port/qoi-port/oracle-source/qoi_test_images` — source test-image set

## Read-only data policy

Do not modify these directories:
- `/home/runner/work/qoi-port/qoi-port/reference/qoi/`
- `/home/runner/work/qoi-port/qoi-port/oracle-source/`
- `/home/runner/work/qoi-port/qoi-port/oracle/outputs/`

They are part of the reproducibility contract for this QOI port.
