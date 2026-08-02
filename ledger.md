# Ledger — QOI → Rust port
# Status values: pending | in-progress | passing | stuck
# Format: - [status] unit_name — one-line note

## Header / framing
- [passing] write_32 — big-endian 32-bit write helper (qoi.h:341), used for header width/height and end marker. Verify byte order explicitly, not just via header_write's overall output.
- [passing] read_32 — big-endian 32-bit read helper (qoi.h:348), decode-side counterpart.
- [passing] header_parse — read "qoif" magic (QOI_MAGIC, 4 bytes), width, height, channels, colorspace (QOI_SRGB=0/QOI_LINEAR=1) from encoded bytes. QOI_HEADER_SIZE=14.
- [passing] header_write — write the same 14-byte header on encode, using write_32.
- [passing] end_marker — verify/emit the 8-byte end-of-stream padding (7 zero bytes + 0x01)
- [passing] desc_validation — qoi_encode must return NULL/fail for invalid qoi_desc: width or height == 0, channels not in {3,4}, colorspace not in {0,1}, or width*height exceeding QOI_PIXELS_MAX (400000000). Confirm exact reference behavior on each invalid case, not just "some kind of error."

## Shared index table (encode AND decode)
- [passing] color_hash — QOI_COLOR_HASH(C) = r*3 + g*5 + b*7 + a*11, mod 64 for index-table slot (qoi.h:322). NOT encode-only: the encoder maintains the running 64-entry index array with it (qoi.h:430) and the decoder maintains the same array, updating after EVERY decoded pixel regardless of opcode (qoi.h:577). Both encode_index_table and decode_op_index depend on it. Test in isolation against known (r,g,b,a) -> index pairs (citing qoi.h:322) before trusting it in either path.

## Decode path
- [passing] decode_op_index — QOI_OP_INDEX: pull pixel from the 64-entry running array (qoi.h:559). BLOCKED on color_hash: the table this reads is populated via QOI_COLOR_HASH after every decoded pixel (qoi.h:577) — do not start until color_hash is passing. Note the index write runs per-pixel (every opcode), not only after index ops.
- [passing] decode_op_diff — QOI_OP_DIFF: small per-channel delta from previous pixel
- [passing] decode_op_luma — QOI_OP_LUMA: green-biased delta encoding
- [passing] decode_op_run — QOI_OP_RUN: run-length repeat of previous pixel
- [passing] decode_op_rgb — QOI_OP_RGB: full RGB literal, alpha unchanged
- [passing] decode_op_rgba — QOI_OP_RGBA: full RGBA literal
- [passing] decode_full — wire all ops together, decode a full image, compare pixel buffer to oracle .decoded files. Must run the index write after every decoded pixel (qoi.h:577), not just after index ops — a hash bug and an index-maintenance bug look identical here in verify.sh (wrong pixel color at some position), so keep color_hash tested separately first. Verified: all 8 decode checks in `verify.sh --all` green; script exit 0 deferred to encode_full (encode stub fails every encode check until then) — marking approved by user.

## Encode path
- [passing] encode_index_table — maintain the same 64-entry running color array, using color_hash (Shared index table section; encode write at qoi.h:430), same reset-to-zero behavior as reference (qoi.h uses QOI_ZEROARR)
- [passing] encode_choose_op — the decision logic for which opcode to emit for a given pixel transition (this is where subtle bugs live — must match reference's op-selection priority order exactly)
- [passing] encode_run — run-length detection and emission, including the max-run-length boundary case
- [passing] encode_full — wire all encode logic together, compare output bytes to oracle .qoi files byte-for-byte

## Round-trip / integration
- [passing] roundtrip_all_images — run encode then decode on every oracle-source image, confirm output pixels match original PNG pixels exactly
- [passing] roundtrip_all_qoi_bytes — confirm encoded bytes match oracle/outputs/*.qoi byte-for-byte (note: edgecase.qoi has a known channels=03 vs 04 discrepancy from qoiconv itself — this is expected, not a bug, do not "fix" it)

## Explicitly out of scope (do not implement)
- qoiconv.c's PNG reading/writing (that's stb_image, not QOI itself — you already have oracle outputs, no need to re-port this)
- qoibench.c, qoifuzz.c (benchmarking/fuzzing tools, not the format itself)