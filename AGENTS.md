# AGENTS.md — QOI → Rust port (qoi_rust)

Governing protocol: `CLAUDE.md` (rules, workflow loop, logging protocol) + `ledger.md` (work queue). This file only adds operational facts CLAUDE.md doesn't state; `scripts/verify.sh` is the executable authority for pass/fail — never self-report success.

## Environment
- Everything runs in **WSL2 Ubuntu-22.04 as root** — never native PowerShell. Working dir `/mnt/d/Code/PortMorterm` (= `D:\Code\PortMorterm`).
- `cargo`/`rustc` (1.97, at `/root/.cargo`) and `gcc`/`sha256sum`/`chmod` exist only in WSL; run `source /root/.cargo/env` before cargo. Don't call them from PowerShell.

## Shell-quoting gotcha (hard-earned)
- PowerShell→`wsl bash -lc "..."` corrupts `$var`, `$?`, `$(...)`, and embedded double quotes. Put logic needing those in a `.sh` file and run `bash file.sh`, or use single-quoted PS strings with no embedded `"`. Never read `$?` through the pipe — echo it from inside a script or use `&&`/`||`.

## Entrypoint & CLI
- Built entrypoint is `src/main.rs`. Root-level `main.rs` is an orphan decoy (deliberately-wrong stub, **not compiled** by cargo) — don't edit or "fix" it.
- Package `qoi_rust`, edition 2024, zero deps, bin only (no lib, no `cargo test`).
- Binary `target/release/qoi_rust`; contract: `encode <in.raw> <out.qoi>`, `decode <in.qoi> <out.decoded>`. Path must match `RUST_BIN` in `scripts/verify.sh`.
- Build: `cargo build --release` from repo root. Done-criterion is `scripts/verify.sh <image_stem>` (e.g. `dice`) or `--all` exiting 0.

## Raw dump format (the pixel interchange — never PNG)
`4-byte BE width, 4-byte BE height, 1-byte channels (3|4), then width*height*channels bytes, row-major, uncompressed`.
- Encode fixtures: `oracle/raw/<stem>.raw`; decode oracles: `oracle/outputs/<stem>.decoded` (same format; byte-identical to the matching `.raw`).
- `.decoded` files were historically PNG — that was a pipeline bug, now fixed. If you see PNG magic (`89 50 4e 47`) in a `.decoded`, something regressed.
- The oracle `.qoi` files are correct as-is; never regenerate them.

## verify.sh facts
- Sole pass/fail authority (CLAUDE.md rule 3). First run (re)builds `oracle/build/mkraw` from `mkraw.c` and, if `oracle/raw/` is empty, generates `.raw` fixtures via `mkraw png2raw` (channels forced like qoiconv: 3 for RGB PNGs, else 4).
- `verify.sh <arg>` takes an IMAGE stem, not a ledger unit. Unit-level checks (e.g. `write_32` byte order) have no script target — write throwaway assertions yourself.
- Each check writes `tmp_verify/<stem>.*` and `cmp`s against `oracle/outputs/`. `edgecase.qoi` has the known channels=03-vs-04 discrepancy (CLAUDE.md rule 9) — expected, never "fix".

## Read-only trees
- `reference/qoi/`, `oracle-source/`, `oracle/outputs/` are `chmod a-w` + read-only attribute. Never edit/move/rename; if something looks wrong, log to `logs/questions.md` and move on (CLAUDE.md rule 1).
- The rest of `oracle/` (stb, build, raw) is writable build tooling.

## Git state
- No commits exist yet; everything is untracked. `.gitignore` only covers `/target` — `git add .` would stage the locked trees, `oracle/raw/`, and `tmp_verify/`. Don't stage/commit those without explicit instruction.

## Missing pieces referenced elsewhere
- CLAUDE.md's workflow says read `research/spec.md` — it doesn't exist; the format authority is `reference/qoi/qoi.h` + the oracle byte outputs.
- `logs/questions.md` and `logs/regressions.md` are referenced but not created; create on first use.
