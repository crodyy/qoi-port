# QOI Rust Port (Porting Hackathon)

This project is a **Rust port** of the original QOI reference implementation from [phoboslab/qoi](https://github.com/phoboslab/qoi).

QOI (Quite OK Image) is a compact, lossless image format designed to be simpler than PNG while keeping strong practical performance.

## What this tool does

This command-line tool can:
- **Encode** raw pixel dumps into `.qoi`
- **Decode** `.qoi` files back into raw pixel dumps

Port target behavior:
- **Encode target:** byte-identical output to reference oracle files
- **Decode target:** pixel-identical output to reference oracle files

## Quick start (first-time users)

Repository root:
- `/home/runner/work/qoi-port/qoi-port`

1. Build:
   ```bash
   cargo build --release
   ```
2. Run unit tests:
   ```bash
   cargo test
   ```
3. Run full oracle verification:
   ```bash
   /home/runner/work/qoi-port/qoi-port/scripts/verify.sh --all
   ```

## CLI usage

Binary location:
- `/home/runner/work/qoi-port/qoi-port/target/release/qoi_rust`

Commands:
```bash
/home/runner/work/qoi-port/qoi-port/target/release/qoi_rust encode <input.raw> <output.qoi>
/home/runner/work/qoi-port/qoi-port/target/release/qoi_rust decode <input.qoi> <output.decoded>
```

## Input/output format (important)

This CLI uses a **raw dump** pixel format (PNG I/O is out of scope for this binary):

1. 4-byte big-endian width
2. 4-byte big-endian height
3. 1-byte channels (`3` or `4`)
4. `width * height * channels` bytes of row-major pixel data

So:
- `encode` reads `.raw`, writes `.qoi`
- `decode` reads `.qoi`, writes `.decoded` (same raw layout)

## Example with included test assets

Encode one included raw fixture:
```bash
/home/runner/work/qoi-port/qoi-port/target/release/qoi_rust \
  encode \
  /home/runner/work/qoi-port/qoi-port/oracle/raw/dice.raw \
  /home/runner/work/qoi-port/qoi-port/tmp_verify/dice.qoi
```

Decode one included oracle QOI:
```bash
/home/runner/work/qoi-port/qoi-port/target/release/qoi_rust \
  decode \
  /home/runner/work/qoi-port/qoi-port/oracle/outputs/dice.qoi \
  /home/runner/work/qoi-port/qoi-port/tmp_verify/dice.decoded
```

## Bugs fixed through this porting process

The process fixed important correctness and pipeline issues:

1. **Raw fixture generation bug fixed**
   - The old pipeline used an invalid temp extension (`.qoi.tmp`) for conversion output, which caused raw fixture generation to fail silently.
2. **Decoded oracle format bug fixed**
   - `oracle/outputs/*.decoded` were previously PNG files; they were migrated to the required raw-dump format so decode checks compare raw pixels correctly.
3. **Test-quality bug fixed**
   - A vacuous always-true decoder assertion was replaced with a real dispatch-order invariant check (RGBA-vs-RUN tag precedence).

## Test statistics for organizers

From final validation records:

- **Unit tests:** `cargo test` → **97 passed, 0 failed**
- **Oracle verification:** `scripts/verify.sh --all` → **8/8 images passed** on both encode and decode (**16 checks total**)
- **Lint gate:** `cargo clippy --all-targets -- -D warnings` → **0 warnings**
- **Run log summary:** `logs/run_log.md` currently records **14 PASS entries** and **2 FAIL entries** (early scaffolding stage)

## Hackathon correctness note

For this hackathon, exact behavior on test and edge cases is required:
- `scripts/verify.sh` is the pass/fail authority
- Oracle outputs in `/home/runner/work/qoi-port/qoi-port/oracle/outputs` are fixed targets
- Edge cases must match reference behavior (no file-specific hardcoding)

Known expected quirk:
- `edgecase.qoi` has a documented channel-field mismatch (`03` vs `04`) from historical oracle generation; keep it documented, do not special-case around filenames.

## Project paths you will use most

- `/home/runner/work/qoi-port/qoi-port/src/main.rs` — Rust implementation
- `/home/runner/work/qoi-port/qoi-port/scripts/verify.sh` — oracle verification script
- `/home/runner/work/qoi-port/qoi-port/oracle/raw` — raw encode fixtures
- `/home/runner/work/qoi-port/qoi-port/oracle/outputs` — expected `.qoi` and `.decoded` outputs
- `/home/runner/work/qoi-port/qoi-port/oracle-source/qoi_test_images` — source test-image set
- `/home/runner/work/qoi-port/qoi-port/logs/run_log.md` — verification run history

## Read-only data policy

Do not modify these directories:
- `/home/runner/work/qoi-port/qoi-port/reference/qoi/`
- `/home/runner/work/qoi-port/qoi-port/oracle-source/`
- `/home/runner/work/qoi-port/qoi-port/oracle/outputs/`

They are part of the reproducibility contract for this port.
