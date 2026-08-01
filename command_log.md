# QOI Port Kickoff — Command Log

Date: 2026-07-31
Environment: WSL2 Ubuntu-22.04 (root), project root `/mnt/d/Code` (Windows `D:\Code`)
Logging: every operational command, verbatim output, exit code, and any fix applied.
RELOCATION 2026-07-31: all artifacts (reference/, oracle-source/, oracle/, the 3 hash files, and this log) moved from `D:\Code\` into `D:\Code\PortMorterm\`. All three hash files re-verified with `sha256sum -c` from the new location (0 non-OK each). Helper script paths in `oracle/build/*.sh` updated from `/mnt/d/Code` to `/mnt/d/Code/PortMorterm`.

---

## [1.1] mkdir -p reference/qoi oracle-source oracle/outputs && git clone https://github.com/phoboslab/qoi reference/qoi
Exit: 0
```text
Cloning into 'reference/qoi'...
```

## [1.2] Generate reference/qoi_hash.txt (date comment + recursive sha256sum of reference/qoi/)
Command: `echo '# 2026-07-31' > reference/qoi_hash.txt && find reference/qoi -type f | sort | xargs sha256sum >> reference/qoi_hash.txt`
Exit: 0 (35 lines total: 1 comment + 34 hashes)

Verification — `sha256sum -c reference/qoi_hash.txt` (all OK, 34/34):
```text
reference/qoi/.git/HEAD: OK
reference/qoi/.git/config: OK
reference/qoi/.git/description: OK
reference/qoi/.git/hooks/applypatch-msg.sample: OK
reference/qoi/.git/hooks/commit-msg.sample: OK
reference/qoi/.git/hooks/fsmonitor-watchman.sample: OK
reference/qoi/.git/hooks/post-update.sample: OK
reference/qoi/.git/hooks/pre-applypatch.sample: OK
reference/qoi/.git/hooks/pre-commit.sample: OK
reference/qoi/.git/hooks/pre-merge-commit.sample: OK
reference/qoi/.git/hooks/pre-push.sample: OK
reference/qoi/.git/hooks/pre-rebase.sample: OK
reference/qoi/.git/hooks/pre-receive.sample: OK
reference/qoi/.git/hooks/prepare-commit-msg.sample: OK
reference/qoi/.git/hooks/push-to-checkout.sample: OK
reference/qoi/.git/hooks/update.sample: OK
reference/qoi/.git/index: OK
reference/qoi/.git/info/exclude: OK
reference/qoi/.git/logs/HEAD: OK
reference/qoi/.git/logs/refs/heads/master: OK
reference/qoi/.git/logs/refs/remotes/origin/HEAD: OK
reference/qoi/.git/objects/pack/pack-3a432be8bc999c9efbb111541873aa6e3ba9cca6.idx: OK
reference/qoi/.git/objects/pack/pack-3a432be8bc999c9efbb111541873aa6e3ba9cca6.pack: OK
reference/qoi/.git/packed-refs: OK
reference/qoi/.git/refs/heads/master: OK
reference/qoi/.git/refs/remotes/origin/HEAD: OK
reference/qoi/.gitignore: OK
reference/qoi/LICENSE: OK
reference/qoi/Makefile: OK
reference/qoi/README.md: OK
reference/qoi/qoi.h: OK
reference/qoi/qoibench.c: OK
reference/qoi/qoiconv.c: OK
reference/qoi/qoifuzz.c: OK
```
Full `reference/qoi_hash.txt` (35 lines):
```text
# 2026-07-31
f6f2b945f6c411b02ba3da9c7ace88dcf71b6af65ba2e0d89aa82900042b5a10  reference/qoi/.git/HEAD
b0e06e962b314ebd0e331384cfeea38855906a2905db11cf0fdab90c7603b1a1  reference/qoi/.git/config
85ab6c163d43a17ea9cf7788308bca1466f1b0a8d1cc92e26e9bf63da4062aee  reference/qoi/.git/description
0223497a0b8b033aa58a3a521b8629869386cf7ab0e2f101963d328aa62193f7  reference/qoi/.git/hooks/applypatch-msg.sample
1f74d5e9292979b573ebd59741d46cb93ff391acdd083d340b94370753d92437  reference/qoi/.git/hooks/commit-msg.sample
f3c0228d8e827f1c5260ac59fdd92c3d425c46e54711ef713c5a54ae0a4db2b4  reference/qoi/.git/hooks/fsmonitor-watchman.sample
81765af2daef323061dcbc5e61fc16481cb74b3bac9ad8a174b186523586f6c5  reference/qoi/.git/hooks/post-update.sample
e15c5b469ea3e0a695bea6f2c82bcf8e62821074939ddd85b77e0007ff165475  reference/qoi/.git/hooks/pre-applypatch.sample
f9af7d95eb1231ecf2eba9770fedfa8d4797a12b02d7240e98d568201251244a  reference/qoi/.git/hooks/pre-commit.sample
d3825a70337940ebbd0a5c072984e13245920cdf8898bd225c8d27a6dfc9cb53  reference/qoi/.git/hooks/pre-merge-commit.sample
ecce9c7e04d3f5dd9d8ada81753dd1d549a9634b26770042b58dda00217d086a  reference/qoi/.git/hooks/pre-push.sample
4febce867790052338076f4e66cc47efb14879d18097d1d61c8261859eaaa7b3  reference/qoi/.git/hooks/pre-rebase.sample
a4c3d2b9c7bb3fd8d1441c31bd4ee71a595d66b44fcf49ddb310252320169989  reference/qoi/.git/hooks/pre-receive.sample
e9ddcaa4189fddd25ed97fc8c789eca7b6ca16390b2392ae3276f0c8e1aa4619  reference/qoi/.git/hooks/prepare-commit-msg.sample
a53d0741798b287c6dd7afa64aee473f305e65d3f49463bb9d7408ec3b12bf5f  reference/qoi/.git/hooks/push-to-checkout.sample
8d5f2fa83e103cf08b57eaa67521df9194f45cbdbcb37da52ad586097a14d106  reference/qoi/.git/hooks/update.sample
d5a37a32020cb6adbca6f2b7eef082e9ecbefc16493bcbc8b72d869d444c2e48  reference/qoi/.git/index
6671fe83b7a07c8932ee89164d1f2793b2318058eb8b98dc5c06ee0a5a3b0ec1  reference/qoi/.git/info/exclude
1318150b3c42bcfb33fe14d10eb4d25384a627bf5f92d0de2990984fe6da46b5  reference/qoi/.git/logs/HEAD
1318150b3c42bcfb33fe14d10eb4d25384a627bf5f92d0de2990984fe6da46b5  reference/qoi/.git/logs/refs/heads/master
1318150b3c42bcfb33fe14d10eb4d25384a627bf5f92d0de2990984fe6da46b5  reference/qoi/.git/logs/refs/remotes/origin/HEAD
874b10cb984cb31c07fd2fea9dd26969970036ec802e2a0c3e1bf6f18c35ea81  reference/qoi/.git/objects/pack/pack-3a432be8bc999c9efbb111541873aa6e3ba9cca6.idx
c4be0cbf155b15d394c94ebb2d5ca4866426472c0b137d067f5fedeba58ba34b  reference/qoi/.git/objects/pack/pack-3a432be8bc999c9efbb111541873aa6e3ba9cca6.pack
80a1d5d1997417b52dfd386df4a2856dcdec4fd0edf41a5b8ab149025ed2de58  reference/qoi/.git/packed-refs
6bf1125e05f0ba28768741fd2f580604e3c57e5105eb4d12d6d07d294b8e783b  reference/qoi/.git/refs/heads/master
cdc65e67690c4c6475174e5ec662b70655246a2f3924354778835ab3be70aa76  reference/qoi/.git/refs/remotes/origin/HEAD
3525edbddf663d969a390fdd126499783b4eb5da8a1a41323c62c25462a80302  reference/qoi/.gitignore
0caf25d92ae0e7e12107e144a86782ccaad37ddaa3ee51e86e193c7db9ed1487  reference/qoi/LICENSE
0ee44a4ab9e71762f4036b34b7bdfbbc732d3d25dfb91e19423ad2b0037e4478  reference/qoi/Makefile
e7ab8e80f59c07b072ef1bfa98a8a9b9e53bb9eeadf75411c8377fbd2ca9e9f7  reference/qoi/README.md
7de6fca1a285b1c20d38f2723dec8b774eb9f144edb9710800a95feeea09375a  reference/qoi/qoi.h
fa76356f7a6e84321c22c24b6e949597c9af662eba28bfcefb8736ab12d6bf95  reference/qoi/qoibench.c
6abba2e650d93429c32b55ff5cc27ba18c56607385f5dfd4aed5d5bd017132ed  reference/qoi/qoiconv.c
3b1bc1fa161d770cca3f10438b6251000dab586fec85861e15804d7a828151ac  reference/qoi/qoifuzz.c
```

---

## [1.3] curl -fsSL -o oracle-source/qoi_test_images.zip https://qoiformat.org/qoi_test_images.zip && unzip -q ... -d oracle-source/
Exit: 0
```text
-rwxrwxrwx 1 root root 5654501 Jul 31  2026 oracle-source/qoi_test_images.zip
UNZIP_OK
8
```
Note: `8` = number of `.png` test images. The zip also ships official reference `.qoi` files (8 of them), total 16 extracted files, flat (no subdirs).

## [1.4] Generate oracle-source_hash.txt (date comment + recursive sha256sum of oracle-source/)
Command: `echo '# 2026-07-31' > oracle-source_hash.txt && find oracle-source -type f | sort | xargs sha256sum >> oracle-source_hash.txt`
Exit: 0 (18 lines: 1 comment + 17 hashes). `sha256sum -c`: all OK.

Full `oracle-source_hash.txt`:
```text
# 2026-07-31
bd557fb208222478d9eefcae59fb473d10e047fd7a8885fcff48861f86599165  oracle-source/qoi_test_images.zip
98ad39b64a723e62053d39cb069ff51febbf6fdeb7ff0f67a3ca191d43f74ab6  oracle-source/qoi_test_images/dice.png
b05a622813eff15ce64f33ab76eee3f9d144f5cf24386e13ddf17c27f6310a01  oracle-source/qoi_test_images/dice.qoi
45e8f7afb7a15b4c4d1ccd25f2e213c45f48ca56f49140828b64d4b9e7a073ae  oracle-source/qoi_test_images/edgecase.png
3cae50b533fbc796171a0763c29a576eaac475d04b6a95fe46b02d440f609e11  oracle-source/qoi_test_images/edgecase.qoi
9dfb70f5867c29ff9ed6313683f19b3d867849e40fbc0c4c54a4a89df341cf23  oracle-source/qoi_test_images/kodim10.png
e330cc81299a2641386f32bdf4b7070b8d5f8f2f76d899ced389b5a1469e65b0  oracle-source/qoi_test_images/kodim10.qoi
e3111a2fd4da24af15d6459ef9eacfe54106b38e27b4a21821b75c3f5d2d5baf  oracle-source/qoi_test_images/kodim23.png
d225e987dc07262be2acee5dee164b5f48d3a49dd0e03f426b3111b52f265548  oracle-source/qoi_test_images/kodim23.qoi
7d23d538ca18f74b3a261f157abe479b79462909c2fc606105228465e5778bd6  oracle-source/qoi_test_images/qoi_logo.png
e6519746939c2b6bc6776a65ce87b1dbd769069c2d2c11295453e9f35160ba57  oracle-source/qoi_test_images/qoi_logo.qoi
77b430a8329304b77c4e67626328cc17f000fa74a1646c50cfddaacaa6024fb3  oracle-source/qoi_test_images/testcard.png
de309646439d2e49c51d9921eb1faff9af4cb33f0019a24ccb57dce1ef00dbab  oracle-source/qoi_test_images/testcard.qoi
e6a2f17f012d66c6f0ed1097f18553bd4e3ad680e96a5da16cfc231def01d7d7  oracle-source/qoi_test_images/testcard_rgba.png
b284ed810a892bca34e89a956b7f8bf21afae4826197a8f3eaef90e470e2149e  oracle-source/qoi_test_images/testcard_rgba.qoi
8ba03c5f635dea6d0aa961efd9b393033c21a84c612f32cff72c98b85c90c2e6  oracle-source/qoi_test_images/wikipedia_008.png
a289c12cd96cc3ff65fcafa1a6d55c5cace0095a45bc570ca1a4d8b79a20b4df  oracle-source/qoi_test_images/wikipedia_008.qoi
```

---

## [1.5] chmod -R a-w reference/ oracle-source/
Exit: 0 — perms now show `-r-xr-xr-x` (e.g. `reference/qoi/README.md`).
ISSUE: On this host WSL runs as **root** and `/mnt/d` is a 9P/drvfs mount; root bypasses Unix permission checks and the emulated perms aren't enforced server-side. `touch reference/qoi/README.md` (mtime update) **succeeded**.
FIX: Also set the real DOS read-only attribute from Windows: `attrib +R /S /D D:\Code\reference` and `attrib +R /S /D D:\Code\oracle-source`.

## [1.6] Re-verification of the lock (after attrib +R fix)
```text
Windows:  Set-Content reference\qoi\README.md -> WINDOWS_WRITE_BLOCKED: Access to the path 'D:\Code\reference\qoi\README.md' is denied.
WSL:      echo probe > reference/qoi/README.md  -> bash: line 1: reference/qoi/README.md: Permission denied  (CONTENT_WRITE_FAILED)
WSL:      touch reference/qoi/README.md         -> succeeds (mtime only). Root has CAP_FOWNER; utimensat needs no write permission even on native Linux. Distro has no non-root user to test with.
Integrity: sha256sum -c reference/qoi_hash.txt     -> 0 non-OK lines (all good)
           sha256sum -c oracle-source_hash.txt     -> 0 non-OK lines (all good)
```
Conclusion: content modification is blocked (Windows `Access denied`, WSL `Permission denied`); only a root-only mtime poke is possible, which does not alter file content or hashes.

---

## [2.1] Download stb headers into oracle/stb/ (deps required by qoiconv.c, not vendored in the qoi repo)
`curl -fsSL -o oracle/stb/stb_image.h https://raw.githubusercontent.com/nothings/stb/master/stb_image.h` and same for stb_image_write.h
Exit: 0
```text
  7988 oracle/stb/stb_image.h
  1724 oracle/stb/stb_image_write.h
  9712 total
```

## [2.2] Compile qoiconv.c unmodified from reference/qoi/
`gcc reference/qoi/qoiconv.c -std=c99 -O3 -Ioracle/stb -o oracle/build/qoiconv`
Exit: 0
```text
oracle/build/qoiconv: ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, BuildID[sha1]=6cdbe6cc2b543a05fec18f323199bcd711fc624a, for GNU/Linux 3.2.0, not stripped
-rwxrwxrwx 1 root root 138776 Jul 31  2026 oracle/build/qoiconv
```

## [2.3-ERR] Inline bash encode/decode loop (first attempt)
ERROR: nested quoting through PowerShell→wsl→bash mangled `$()`/`basename`; PowerShell tried to execute `basename` itself and the bash command ended in `unexpected EOF while looking for matching quote`.
FIX: move the loop into a script file `oracle/build/run_oracle.sh` and execute `bash oracle/build/run_oracle.sh`.

## [2.4-ERR] run_oracle.sh run #1 — encodes OK, decodes failed
Exit: 0 (loop ran); decode step errors:
```text
== dice ==
  encoded -> oracle/outputs/dice.qoi
Couldn't write/encode oracle/outputs/dice.decoded
  ... (same for all 8 images)
```
ERROR: qoiconv only writes outputs whose filename ends in `.png` or `.qoi`; a `.decoded` name matches neither branch, so `encoded` stays 0 and it aborts.
FIX: decode to `oracle/outputs/<name>.png` first, then `mv` it to `<name>.decoded` (keeps PNG bytes, satisfies the required `.decoded` filename).

## [2.5] run_oracle.sh run #2 — all encodes + decodes succeed
Exit: 0
```text
== dice ==
  encoded -> oracle/outputs/dice.qoi
  decoded -> oracle/outputs/dice.decoded
== edgecase ==
  encoded -> oracle/outputs/edgecase.qoi
  decoded -> oracle/outputs/edgecase.decoded
== kodim10 ==
  encoded -> oracle/outputs/kodim10.qoi
  decoded -> oracle/outputs/kodim10.decoded
== kodim23 ==
  encoded -> oracle/outputs/kodim23.qoi
  decoded -> oracle/outputs/kodim23.decoded
== qoi_logo ==
  encoded -> oracle/outputs/qoi_logo.qoi
  decoded -> oracle/outputs/qoi_logo.decoded
== testcard ==
  encoded -> oracle/outputs/testcard.qoi
  decoded -> oracle/outputs/testcard.decoded
== testcard_rgba ==
  encoded -> oracle/outputs/testcard_rgba.qoi
  decoded -> oracle/outputs/testcard_rgba.decoded
== wikipedia_008 ==
  encoded -> oracle/outputs/wikipedia_008.qoi
  decoded -> oracle/outputs/wikipedia_008.decoded
DONE
```

## [2.6] Count check
```text
png_count=8
out_count=16
```
16 output files = 2 × 8 test images → **pass**.

## [2.7-ERR] Hash oracle/outputs inline (first attempt)
ERROR: inline `echo '# 2026-07-31' > ...` produced no output and no file (shell-quoting layers swallowed it).
FIX: moved to script `oracle/build/hash_outputs.sh`. Result: 17 lines, `sha256sum -c` all OK (16/16).

## [2.8] Oracle quality validation
```text
--- decoded magic bytes (expect 8950 4e47 = PNG) ---
oracle/outputs/dice.decoded -> 89504e47
oracle/outputs/edgecase.decoded -> 89504e47
oracle/outputs/kodim10.decoded -> 89504e47
oracle/outputs/kodim23.decoded -> 89504e47
oracle/outputs/qoi_logo.decoded -> 89504e47
oracle/outputs/testcard.decoded -> 89504e47
oracle/outputs/testcard_rgba.decoded -> 89504e47
oracle/outputs/wikipedia_008.decoded -> 89504e47
--- qoi encode vs official reference (cmp) ---
MATCH dice.qoi
DIFF  edgecase.qoi
MATCH kodim10.qoi
MATCH kodim23.qoi
MATCH qoi_logo.qoi
MATCH testcard.qoi
MATCH testcard_rgba.qoi
MATCH wikipedia_008.qoi
```
NOTE (not an error): `edgecase.png` is 8-bit RGB (3ch); qoiconv encodes it with `channels=03`, the official reference uses `channels=04`. Oracle is qoiconv's exact output; round-trip intact.

## [2.9] chmod -R a-w oracle/outputs/ + lock verification
`chmod -R a-w oracle/outputs/` → perms `-r-xr-xr-x`. Then, as before, enforced with `attrib +R /S /D D:\Code\oracle\outputs`.
```text
Windows: Set-Content oracle\outputs\dice.qoi -> WINDOWS_WRITE_BLOCKED: Access to the path ... is denied.
WSL:     echo probe > oracle/outputs/dice.qoi -> bash: line 1: oracle/outputs/dice.qoi: Permission denied (WSL_CONTENT_WRITE_BLOCKED)
Integrity: sha256sum -c oracle_outputs_hash.txt -> 16 OK lines (all good)
```

---

## [3.1-ERR] curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
ERROR: piped stdin is not interactive → `error: Unable to run interactively. Run with -y to accept defaults, --help for additional options`
FIX: append `-s -- -y` so rustup-init accepts defaults non-interactively: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y`

## [3.2] Rust install (fixed command)
`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y`
Exit: 0
```text
info: downloading installer
warn: It looks like you have an existing rustup settings file at:
warn: /root/.rustup/settings.toml
warn: Rustup will install the default toolchain as specified in the settings file,
warn: instead of the one inferred from the default host triple.
info: profile set to default
info: default host triple is x86_64-unknown-linux-gnu
info: syncing channel updates for stable-x86_64-unknown-linux-gnu
info: latest update on 2026-07-16 for version 1.97.1 (8bab26f4f 2026-07-14)
info: downloading 6 components

info: default toolchain set to stable-x86_64-unknown-linux-gnu
  stable-x86_64-unknown-linux-gnu installed - rustc 1.97.1 (8bab26f4f 2026-07-14)


Rust is installed now. Great!
```

## [3.3] Verification
`source /root/.cargo/env && rustc --version && cargo --version && rustup show active-toolchain`
Exit: 0
```text
rustc 1.97.1 (8bab26f4f 2026-07-14)
cargo 1.97.1 (c980f4866 2026-06-30)
stable-x86_64-unknown-linux-gnu (default)
```

---

## [4.x] RAW-DUMP MIGRATION (fixes the PNG-interchange bugs in the oracle pipeline)
Two bugs, one root cause: PNG used as pixel interchange instead of raw bytes.
- Bug 1: verify.sh RAW-gen wrote `$RAW_DIR/<stem>.qoi.tmp` — qoiconv rejects output names not ending `.png`/`.qoi`, so generation failed silently (stderr→`/dev/null`) and `oracle/raw/` stayed empty.
- Bug 2: `oracle/outputs/*.decoded` were PNG files, so decode checks compared raw pixels against PNG bytes — unmatchable without porting PNG encode.

Fix — new helper `oracle/build/mkraw.c` (stb_image.h + qoi.h), raw dump format:
`4-byte BE width, 4-byte BE height, 1-byte channels, then width*height*channels raw bytes`.

## [4.1] Compile mkraw
`gcc oracle/build/mkraw.c -std=c99 -O2 -Ioracle/stb -Ireference/qoi -o oracle/build/mkraw`
Exit: 0 → `ELF 64-bit LSB pie executable, x86-64` (50,336 bytes)

## [4.2] Unlock oracle/outputs + regenerate fixtures and decoded files
`attrib -R /S /D` on `oracle/outputs`; `chmod -R u+w`; `bash oracle/build/mkraw_all.sh`
Exit: 0 — generated `oracle/raw/<stem>.raw` (png2raw) and replaced `oracle/outputs/<stem>.decoded` (qoi2raw) for all 8 images.

## [4.3] Verify raw format (sizes + headers + raw==decoded)
- dice: 800x600 ch4 → 1,920,009 B (`00 00 03 20 00 00 02 58 04` header)
- edgecase: 256x64 ch3 → 49,161 B; kodim10/kodim23: 512x768/768x512 ch3 → 1,179,657 B
- qoi_logo: 448x220 ch4 → 394,249 B; testcard/testcard_rgba: 256x256 ch4 → 262,153 B
- wikipedia_008: 1152x858 ch3 → 2,965,257 B
- `cmp oracle/raw/<stem>.raw oracle/outputs/<stem>.decoded` → **IDENTICAL for all 8**

## [4.4] Re-hash oracle/outputs + re-lock
- `oracle_outputs_hash.txt` regenerated (17 lines); `sha256sum -c` 16/16 OK.
- Re-lock: `chmod -R a-w` + `attrib +R /S /D`. WSL content write → `Permission denied`; Windows write → `Access denied`. Hashes still 16/16 OK.

## [4.5] Update scripts/verify.sh + CLAUDE.md
- verify.sh: mkraw auto-build if missing; one-time `oracle/raw` generation via `mkraw png2raw`; encode input `$RAW_DIR/$stem.raw` (was `.png`); header comment documents raw dump format.
- CLAUDE.md: CLI contract now `encode <in.raw> <out.qoi>` / `decode <in.qoi> <out.decoded>` with raw-dump spec; PNG explicitly out of scope in the loop.

## [4.6] verify.sh --all post-migration
Exit: 1 (all 16 checks FAIL — Rust stub still unimplemented; expected). No generation/build errors. Logged in `logs/run_log.md`.

## [4.7] Created AGENTS.md
Operational facts: WSL-only toolchain, PowerShell→wsl quoting corruption, src/main.rs vs root main.rs decoy, raw dump format, verify.sh semantics (image stems, not ledger units), locked trees, git state (no commits), missing `research/spec.md`.

---

## [0.1] apt-get update && apt-get install -y gcc git unzip curl
Exit: 0
```text
Get:1 http://security.ubuntu.com/ubuntu jammy-security InRelease [129 kB]
Hit:2 http://archive.ubuntu.com/ubuntu jammy InRelease
Get:3 http://archive.ubuntu.com/ubuntu jammy-updates InRelease [128 kB]
Get:4 http://security.ubuntu.com/ubuntu jammy-security/main amd64 Packages [3411 kB]
Get:5 http://archive.ubuntu.com/ubuntu jammy-backports InRelease [127 kB]
Get:6 http://archive.ubuntu.com/ubuntu jammy/universe amd64 Packages [14.1 MB]
... (apt fetch + install of 29 packages completed cleanly)
Setting up gcc-11-base:amd64 (11.4.0-1ubuntu1~22.04.3) ...
Setting up manpages-dev (5.10-1ubuntu1) ...
Setting up libxpm4:amd64 (1:3.5.12-1ubuntu0.22.04.3) ...
Setting up unzip (6.0-26ubuntu3.2) ...
Setting up linux-libc-dev:amd64 (5.15.0-186.196) ...
Setting up libgomp1:amd64 (12.3.0-1ubuntu1~22.04.3) ...
Setting up libasan6:amd64 (11.4.0-1ubuntu1~22.04.3) ...
Setting up libtirpc-dev:amd64 (1.3.2-2ubuntu0.1) ...
Setting up rpcsvc-proto (1.4.2-0ubuntu6) ...
Setting up libquadmath0:amd64 (12.3.0-1ubuntu1~22.04.3) ...
Setting up libgd3:amd64 (2.3.0-2ubuntu2.3) ...
Setting up libmpc3:amd64 (1.2.1-2build1) ...
Setting up libatomic1:amd64 (12.3.0-1ubuntu1~22.04.3) ...
Setting up libubsan1:amd64 (12.3.0-1ubuntu1~22.04.3) ...
Setting up libnsl-dev:amd64 (1.3.0-2build2) ...
Setting up libcrypt-dev:amd64 (1:4.4.27-1) ...
Setting up libcurl4:amd64 (7.81.0-1ubuntu1.25) ...
Setting up curl (7.81.0-1ubuntu1.25) ...
Setting up libisl23:amd64 (0.24-2build1) ...
Setting up libc-dev-bin (2.35-0ubuntu3.14) ...
Setting up libcc1-0:amd64 (12.3.0-1ubuntu1~22.04.3) ...
Setting up liblsan0:amd64 (12.3.0-1ubuntu1~22.04.3) ...
Setting up libitm1:amd64 (12.3.0-1ubuntu1~22.04.3) ...
Setting up libc-devtools (2.35-0ubuntu3.14) ...
Setting up libtsan0:amd64 (12.3.0-1ubuntu1~22.04.3) ...
Setting up cpp-11 (11.4.0-1ubuntu1~22.04.3) ...
Setting up libgcc-11-dev:amd64 (11.4.0-1ubuntu1~22.04.3) ...
Setting up gcc-11 (11.4.0-1ubuntu1~22.04.3) ...
Setting up cpp (4:11.2.0-1ubuntu1) ...
Setting up libc6-dev:amd64 (2.35-0ubuntu3.14) ...
Setting up gcc (4:11.2.0-1ubuntu1) ...
Processing triggers for man-db (2.10.2-1) ...
Processing triggers for libc-bin (2.35-0ubuntu3.13) ...
```

## [0.2] Toolchain verification (gcc / git / unzip / curl versions)
Exit: 0
```text
gcc (Ubuntu 11.4.0-1ubuntu1~22.04.3) 11.4.0
Copyright (C) 2021 Free Software Foundation, Inc.
This is free software; see the source for copying conditions.  There is NO
warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.

---
git version 2.34.1
---
UnZip 6.00 of 20 April 2009, by Debian. Original by Info-ZIP.

---
curl 7.81.0 (x86_64-pc-linux-gnu) libcurl/7.81.0 OpenSSL/3.0.2 zlib/1.2.11 brotli/1.0.9 zstd/1.4.8 libidn2/2.3.2 libpsl/0.21.0 (+libidn2/2.3.2) libssh/0.9.6/openssl/zlib nghttp2/1.43.0 librtmp/2.3 OpenLDAP/2.5.20
```

---

## [5.1] write_32 unit (ledger item 1)
- Added `write_32(bytes: &mut [u8], p: &mut usize, v: u32)` to `src/main.rs`, ported verbatim from qoi.h:341-346 (big-endian, advances `*p` by 4).
- Added `#[cfg(test)] mod tests` with 4 assertions (byte order of 0x00000320, incremental p across two writes, offset write, zero value).
- `cargo test`: **4 passed, 0 failed**.
- `cargo build --release`: OK, one expected `dead_code` warning (write_32 unused until header_write/end_marker land).
- Ledger: `write_32` → `[passing]`. This unit has no `verify.sh` target (per AGENTS.md, unit-level checks use throwaway assertions).

## [5.2] read_32 unit (ledger item 2)
- Added `read_32(bytes: &[u8], p: &mut usize) -> u32` to `src/main.rs`, ported verbatim from qoi.h:348-354 (big-endian; C's `unsigned int a=...` promotion reproduced via `bytes[*p] as u32` before shifting).
- Added 4 new assertions (big-endian read, offset read, max value, and a `write_32`→`read_32` roundtrip over edge values incl. 0x80000001 and 0xFFFFFFFF).
- `cargo test`: **8 passed, 0 failed** (4 write_32 + 4 read_32).
- Ledger: `read_32` → `[passing]`. No `verify.sh` target.

## [5.3] header_parse unit (ledger item 3)
- Added constants (`QOI_MAGIC`=0x716f6966, `QOI_HEADER_SIZE`=14, `QOI_PIXELS_MAX`=400000000, `QOI_SRGB`=0, `QOI_LINEAR`=1), `struct Header {width,height,channels,colorspace}`, and `parse_header(bytes) -> Option<Header>` mirroring qoi.h:507-521 exactly — including the guard order `width==0 || height==0 || channels<3 || channels>4 || colorspace>1 || magic!=QOI_MAGIC || height >= QOI_PIXELS_MAX/width` (width==0 short-circuits before the division, as in C).
- 10 new assertions: valid sRGB, valid linear, bad magic, zero width, zero height, channels 2/5, colorspace 2, overflow guard (1×400000000 rejected), max-valid (1×399999999 accepted), short input.
- `cargo test`: **18 passed, 0 failed**.
- Ledger: `header_parse` → `[passing]`. No `verify.sh` target.

## [5.4] header_write unit (ledger item 4)
- Added `write_header(h: &Header) -> Vec<u8>` to `src/main.rs`, mirroring qoi.h:384-388 (write_32 magic, write_32 width, write_32 height, then channels, colorspace bytes; asserts final p == 14).
- 3 new assertions: exact 14-byte output for dice-style header (bytes `71 6f 69 66 00 00 03 20 00 00 02 58 04 00`, matching the real `oracle/outputs/dice.qoi` header), header byte fields vs the actual oracle dice header, and a write→parse roundtrip (3×5 RGB linear).
- `cargo test`: **21 passed, 0 failed**.
- Ledger: `header_write` → `[passing]`. No `verify.sh` target.

## [5.5] end_marker unit (ledger item 5)
- Added `QOI_PADDING` const `[0;7] + [1]` (mirrors `qoi_padding[8]` qoi.h:339) and `write_end_marker(bytes: &mut Vec<u8>)` (qoi.h:480-482). Decode-side reservation of the last 8 bytes (`chunks_len = size - sizeof(qoi_padding)`, qoi.h:539) is decode_full's job; reference never validates padding content.
- 3 new assertions: exact `[0,0,0,0,0,0,0,1]` output, append-after-existing behavior, and a real-file check that the tail of `oracle/outputs/dice.qoi` equals the padding.
- `cargo test`: **24 passed, 0 failed**.
- Ledger: `end_marker` → `[passing]`. No `verify.sh` target.

## [5.6] desc_validation unit (ledger item 6)
- Added `validate_desc(h: &Header) -> bool` to `src/main.rs`, mirroring qoi.h:364-372 exactly (width==0, height==0, channels<3, channels>4, colorspace>1, height >= QOI_PIXELS_MAX/width; same guard order as C).
- 6 new assertions: accepts 6 valid cases (incl. boundary 1×399999999 and 200000000×1), rejects zero width, zero height, channels 0/2/5, colorspace 2/255, and pixel-count overflow cases (1×400000000, 400000000×1, 200000000×2).
- TEST-FIX DURING RUN: initially asserted 400000000×1 valid — wrong. Reference `height >= QOI_PIXELS_MAX/width`: 400000000/400000000=1, 1>=1 → REJECTED. Corrected test; implementation was always right.
- `cargo test`: **30 passed, 0 failed**.
- Ledger: `desc_validation` → `[passing]`. No `verify.sh` target.

---

