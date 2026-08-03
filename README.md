# QOI Rust Port (Porting Hackathon)

This repository is a Rust port of the reference QOI implementation from [phoboslab/qoi](https://github.com/phoboslab/qoi).

The goal is strict equivalence with `qoi.h`:
- **Encode:** byte-identical `.qoi` output
- **Decode:** pixel-identical decoded output

## What this project includes

- Rust CLI implementation in `/home/runner/work/qoi-port/qoi-port/src/main.rs`
- Oracle test corpus and expected outputs
- Verification script that diffs Rust output against oracle files

## First-time setup

1. Open a shell in the repo root:
   - `/home/runner/work/qoi-port/qoi-port`
2. Build the release binary:
   ```bash
   cargo build --release
   ```
3. Run unit tests:
   ```bash
   cargo test
   ```
4. Run full oracle verification:
   ```bash
   /home/runner/work/qoi-port/qoi-port/scripts/verify.sh --all
   ```

## CLI usage

The binary path is:
- `/home/runner/work/qoi-port/qoi-port/target/release/qoi_rust`

Commands:
```bash
qoi_rust encode <input.raw> <output.qoi>
qoi_rust decode <input.qoi> <output.decoded>
```

Raw input/output format:
- 4-byte big-endian width
- 4-byte big-endian height
- 1-byte channels (`3` or `4`)
- `width * height * channels` bytes of row-major pixel data

## Testing and verification policy (hackathon note)

For this porting hackathon, correctness is judged by exact oracle matching:
- Use `scripts/verify.sh` as the pass/fail authority
- Keep oracle fixtures and expected outputs unchanged
- Maintain exact behavior on edge cases from the reference implementation

Known expected quirk:
- `edgecase.qoi` has a documented channel-field mismatch (`03` vs `04`) from oracle generation. Treat this as known historical behavior, not a target for special-casing.

## Repository layout

- `/home/runner/work/qoi-port/qoi-port/src/main.rs` — Rust implementation + unit tests
- `/home/runner/work/qoi-port/qoi-port/scripts/verify.sh` — oracle comparison runner
- `/home/runner/work/qoi-port/qoi-port/oracle/raw` — raw encode fixtures
- `/home/runner/work/qoi-port/qoi-port/oracle/outputs` — expected `.qoi` and `.decoded` outputs
- `/home/runner/work/qoi-port/qoi-port/oracle-source/qoi_test_images` — original test-image corpus
- `/home/runner/work/qoi-port/qoi-port/ledger.md` — unit-by-unit port tracking
- `/home/runner/work/qoi-port/qoi-port/CLAUDE.md` and `/home/runner/work/qoi-port/qoi-port/AGENTS.md` — workflow and guardrails

## Important constraints

Do not edit, move, or regenerate read-only reference data under:
- `/home/runner/work/qoi-port/qoi-port/reference/qoi/`
- `/home/runner/work/qoi-port/qoi-port/oracle-source/`
- `/home/runner/work/qoi-port/qoi-port/oracle/outputs/`

These are part of the reproducibility contract for the port.
