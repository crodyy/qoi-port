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

## Decode path
- [in-progress] decode_op_index — QOI_OP_INDEX: pull pixel from the 64-entry running array
- [pending] decode_op_diff — QOI_OP_DIFF: small per-channel delta from previous pixel
- [pending] decode_op_luma — QOI_OP_LUMA: green-biased delta encoding
- [pending] decode_op_run — QOI_OP_RUN: run-length repeat of previous pixel
- [pending] decode_op_rgb — QOI_OP_RGB: full RGB literal, alpha unchanged
- [pending] decode_op_rgba — QOI_OP_RGBA: full RGBA literal
- [pending] decode_full — wire all ops together, decode a full image, compare pixel buffer to oracle .decoded files

## Encode path
- [pending] color_hash — QOI_COLOR_HASH(C) = r*3 + g*5 + b*7 + a*11, mod 64 for index table slot. Test in isolation against known (r,g,b,a) -> index pairs before trusting it inside encode_index_table.
- [pending] encode_index_table — maintain the same 64-entry running color array, using color_hash, same reset-to-zero behavior as reference (qoi.h uses QOI_ZEROARR)
- [pending] encode_choose_op — the decision logic for which opcode to emit for a given pixel transition (this is where subtle bugs live — must match reference's op-selection priority order exactly)
- [pending] encode_run — run-length detection and emission, including the max-run-length boundary case
- [pending] encode_full — wire all encode logic together, compare output bytes to oracle .qoi files byte-for-byte

## Round-trip / integration
- [pending] roundtrip_all_images — run encode then decode on every oracle-source image, confirm output pixels match original PNG pixels exactly
- [pending] roundtrip_all_qoi_bytes — confirm encoded bytes match oracle/outputs/*.qoi byte-for-byte (note: edgecase.qoi has a known channels=03 vs 04 discrepancy from qoiconv itself — this is expected, not a bug, do not "fix" it)

## Explicitly out of scope (do not implement)
- qoiconv.c's PNG reading/writing (that's stb_image, not QOI itself — you already have oracle outputs, no need to re-port this)
- qoibench.c, qoifuzz.c (benchmarking/fuzzing tools, not the format itself)