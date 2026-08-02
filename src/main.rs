use std::env;
use std::process::ExitCode;

const QOI_MAGIC: u32 = 0x716f6966; // 'q' << 24 | 'o' << 16 | 'i' << 8 | 'f'
const QOI_HEADER_SIZE: usize = 14;
const QOI_PIXELS_MAX: u32 = 400000000;
const QOI_SRGB: u8 = 0;
#[cfg(test)]
const QOI_LINEAR: u8 = 1;
const QOI_PADDING: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 1];
const QOI_OP_INDEX: u8 = 0x00; // qoi.h:313 — 00xxxxxx
const QOI_OP_DIFF: u8 = 0x40; // qoi.h:314 — 01xxxxxx
const QOI_OP_LUMA: u8 = 0x80; // qoi.h:315 — 10xxxxxx
const QOI_OP_RUN: u8 = 0xc0; // qoi.h:316 — 11xxxxxx
const QOI_OP_RGB: u8 = 0xfe; // qoi.h:317 — 11111110
const QOI_OP_RGBA: u8 = 0xff; // qoi.h:318 — 11111111
const QOI_MASK_2: u8 = 0xc0; // qoi.h:320 — 11000000

#[derive(Debug, PartialEq, Eq)]
struct Header {
    width: u32,
    height: u32,
    channels: u8,
    colorspace: u8,
}

fn parse_header(bytes: &[u8]) -> Option<Header> {
    if bytes.len() < QOI_HEADER_SIZE {
        return None;
    }
    let mut p = 0usize;
    let magic = read_32(bytes, &mut p);
    let width = read_32(bytes, &mut p);
    let height = read_32(bytes, &mut p);
    let channels = bytes[p];
    p += 1;
    let colorspace = bytes[p];
    if width == 0
        || height == 0
        || !(3..=4).contains(&channels)
        || colorspace > 1
        || magic != QOI_MAGIC
        || height >= QOI_PIXELS_MAX / width
    {
        return None;
    }
    Some(Header {
        width,
        height,
        channels,
        colorspace,
    })
}

fn validate_desc(h: &Header) -> bool {
    !(h.width == 0
        || h.height == 0
        || h.channels < 3
        || h.channels > 4
        || h.colorspace > 1
        || h.height >= QOI_PIXELS_MAX / h.width)
}

fn write_header(h: &Header) -> Vec<u8> {
    let mut bytes = [0u8; QOI_HEADER_SIZE];
    let mut p = 0usize;
    write_32(&mut bytes, &mut p, QOI_MAGIC);
    write_32(&mut bytes, &mut p, h.width);
    write_32(&mut bytes, &mut p, h.height);
    bytes[p] = h.channels;
    p += 1;
    bytes[p] = h.colorspace;
    p += 1;
    assert_eq!(p, QOI_HEADER_SIZE);
    bytes.to_vec()
}

#[cfg(test)]
fn write_end_marker(bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&QOI_PADDING);
}

fn write_32(bytes: &mut [u8], p: &mut usize, v: u32) {
    bytes[*p] = ((0xff000000 & v) >> 24) as u8;
    *p += 1;
    bytes[*p] = ((0x00ff0000 & v) >> 16) as u8;
    *p += 1;
    bytes[*p] = ((0x0000ff00 & v) >> 8) as u8;
    *p += 1;
    bytes[*p] = (0x000000ff & v) as u8;
    *p += 1;
}

fn read_32(bytes: &[u8], p: &mut usize) -> u32 {
    let a = bytes[*p] as u32;
    *p += 1;
    let b = bytes[*p] as u32;
    *p += 1;
    let c = bytes[*p] as u32;
    *p += 1;
    let d = bytes[*p] as u32;
    *p += 1;
    a << 24 | b << 16 | c << 8 | d
}

// qoi.h:322 — QOI_COLOR_HASH(C) = C.rgba.r*3 + C.rgba.g*5 + C.rgba.b*7 + C.rgba.a*11.
// Index-table slot is hash & (64 - 1) (encode write: qoi.h:430, decode write: qoi.h:577).
// C promotes the uint8_t channels to int before multiplying; compute in u32 so
// e.g. 255*11 cannot overflow. Shared by the encode AND decode index tables — the
// decoder updates the table after every decoded pixel (qoi.h:577), not just index ops.
fn color_hash(r: u8, g: u8, b: u8, a: u8) -> usize {
    ((r as u32 * 3 + g as u32 * 5 + b as u32 * 7 + a as u32 * 11) % 64) as usize
}

// qoi.h qoi_rgba_t — one pixel, channels accessed individually (.rgba member form).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Px {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

// Decode-side running state of qoi_decode (qoi.h:492-495): the 64-entry color index,
// the current pixel, and the run-length countdown.
struct Decoder {
    index: [Px; 64],
    px: Px,
    run: u8,
}

impl Decoder {
    // qoi.h:533-537 — QOI_ZEROARR(index) (memset 0, qoi.h:310), then px = {0,0,0,255};
    // run = 0 (qoi.h:495).
    fn new() -> Self {
        Decoder {
            index: [Px { r: 0, g: 0, b: 0, a: 0 }; 64],
            px: Px { r: 0, g: 0, b: 0, a: 255 },
            run: 0,
        }
    }

    // qoi.h:577 — after EVERY decoded pixel (any opcode): index[hash(px)] = px.
    // Lives inside each op handler (not the loop driver) so decode_full cannot
    // forget it; run-length-skipped pixels correctly never reach it (qoi.h:541-543).
    fn index_update(&mut self) {
        let px = self.px;
        self.index[color_hash(px.r, px.g, px.b, px.a)] = px;
    }

    // qoi.h:558-559 — QOI_OP_INDEX: b1's top 2 bits are 00, so b1 itself is the slot.
    fn decode_op_index(&mut self, b1: u8) -> Px {
        debug_assert_eq!(b1 & QOI_MASK_2, QOI_OP_INDEX, "not a QOI_OP_INDEX byte");
        self.px = self.index[b1 as usize];
        self.index_update(); // qoi.h:577 — same-slot rewrite for this op
        self.px
    }

    // qoi.h:561-565 — QOI_OP_DIFF: 2-bit fields dr/dg/db, bias -2 (so each delta is
    // in {-2,-1,0,1}); alpha unchanged. C's uint8_t += wraps mod 256 (0 + -2 = 254),
    // so the adds must be wrapping. Encode-side inverse: qoi.h:446-451.
    fn decode_op_diff(&mut self, b1: u8) -> Px {
        debug_assert_eq!(b1 & QOI_MASK_2, QOI_OP_DIFF, "not a QOI_OP_DIFF byte");
        let dr = ((b1 >> 4) & 0x03) as i8 - 2;
        let dg = ((b1 >> 2) & 0x03) as i8 - 2;
        let db = (b1 & 0x03) as i8 - 2;
        self.px.r = self.px.r.wrapping_add(dr as u8);
        self.px.g = self.px.g.wrapping_add(dg as u8);
        self.px.b = self.px.b.wrapping_add(db as u8);
        self.index_update(); // qoi.h:577
        self.px
    }

    // qoi.h:566-572 — QOI_OP_LUMA: two bytes. vg = (b1 & 0x3f) - 32 in [-32,31] is
    // applied to green; red/blue get vg plus a 4-bit nibble from b2, bias -8 (so
    // vg_r/vg_b in [-8,7]). Alpha unchanged; channel adds wrap mod 256 like C
    // uint8_t. Encode-side inverse: qoi.h:453-460.
    fn decode_op_luma(&mut self, b1: u8, b2: u8) -> Px {
        debug_assert_eq!(b1 & QOI_MASK_2, QOI_OP_LUMA, "not a QOI_OP_LUMA byte");
        let vg = (b1 & 0x3f) as i8 - 32;
        let vg_r = ((b2 >> 4) & 0x0f) as i8 - 8;
        let vg_b = (b2 & 0x0f) as i8 - 8;
        self.px.r = self.px.r.wrapping_add((vg + vg_r) as u8);
        self.px.g = self.px.g.wrapping_add(vg as u8);
        self.px.b = self.px.b.wrapping_add((vg + vg_b) as u8);
        self.index_update(); // qoi.h:577
        self.px
    }

    // qoi.h:573-575 — QOI_OP_RUN: run = b1 & 0x3f; px itself is unchanged (it IS the
    // repeated pixel). The qoi.h:577 index write still happens for this pixel — the
    // RUN branch sits inside the chunk-reading else-if (qoi.h:544). Total repeated
    // pixels per RUN chunk = 1 (this pixel) + run, matching the encoder's run-1
    // field (qoi.h:418/426). Note: 0xfe/0xff match the RUN mask but are dispatched
    // as QOI_OP_RGB/QOI_OP_RGBA first (qoi.h:547/552 precede 573).
    fn decode_op_run(&mut self, b1: u8) -> Px {
        debug_assert_eq!(b1 & QOI_MASK_2, QOI_OP_RUN, "not a QOI_OP_RUN byte");
        self.run = b1 & 0x3f;
        self.index_update(); // qoi.h:577
        self.px
    }

    // qoi.h:541-543 — while run > 0, each pixel just repeats px and decrements run:
    // no chunk is read and the qoi.h:577 index write does NOT happen (it lives
    // inside the else-if branch that read the chunk).
    fn decode_run_skip(&mut self) -> Px {
        debug_assert!(self.run > 0, "run skip with no active run");
        self.run -= 1;
        self.px
    }

    // qoi.h:547-551 — QOI_OP_RGB (0xfe, qoi.h:317): the next 3 bytes are the literal
    // r/g/b; alpha is not in the stream and stays unchanged. qoi.h:577 index write
    // applies. Dispatch note: the equality check for 0xfe must run BEFORE the
    // mask-based checks (qoi.h:547 precedes 558-573) because 0xfe & QOI_MASK_2
    // == QOI_OP_RUN.
    fn decode_op_rgb(&mut self, r: u8, g: u8, b: u8) -> Px {
        self.px.r = r;
        self.px.g = g;
        self.px.b = b;
        self.index_update(); // qoi.h:577
        self.px
    }

    // qoi.h:552-557 — QOI_OP_RGBA (0xff, qoi.h:318): the next 4 bytes are the literal
    // r/g/b/a. Same dispatch-order caveat as RGB: 0xff & QOI_MASK_2 == QOI_OP_RUN,
    // so the 0xff equality check must precede the mask checks (qoi.h:552 before
    // 558-573). qoi.h:577 index write applies.
    fn decode_op_rgba(&mut self, r: u8, g: u8, b: u8, a: u8) -> Px {
        self.px.r = r;
        self.px.g = g;
        self.px.b = b;
        self.px.a = a;
        self.index_update(); // qoi.h:577
        self.px
    }
}

// Encode-side running state of qoi_encode (qoi.h:357-362): the 64-entry color index,
// the previous pixel, and the run-length counter.
struct Encoder {
    index: [Px; 64],
    px_prev: Px, // qoi.h:362
    run: u8,     // qoi.h:357
}

impl Encoder {
    // qoi.h:393-400 — QOI_ZEROARR(index) (memset 0, qoi.h:310), px_prev = {0,0,0,255},
    // run = 0 (qoi.h:395).
    fn new() -> Self {
        Encoder {
            index: [Px { r: 0, g: 0, b: 0, a: 0 }; 64],
            px_prev: Px { r: 0, g: 0, b: 0, a: 255 },
            run: 0,
        }
    }

    // qoi.h:430-436 — slot = QOI_COLOR_HASH(px) & 63. On a hit (index[slot] holds
    // exactly px — full 4-channel compare, qoi.h:432 .v) return Some(slot) so the
    // caller emits QOI_OP_INDEX | slot (qoi.h:433). On a miss, store px at the slot
    // (qoi.h:436 — replaces whatever was there, no chaining) and return None so the
    // caller falls through to DIFF/LUMA/RGB/RGBA selection.
    fn encode_index_lookup(&mut self, px: Px) -> Option<u8> {
        let slot = color_hash(px.r, px.g, px.b, px.a);
        if self.index[slot] == px {
            Some(slot as u8)
        } else {
            self.index[slot] = px;
            None
        }
    }

    // qoi.h:430-474 — opcode decision for one non-run pixel transition. Contract:
    // px != px_prev (the reference routes px == px_prev into the run counter first,
    // qoi.h:415). Returns the emitted chunk bytes. Priority order, must match the
    // reference exactly: index hit -> QOI_OP_INDEX (qoi.h:432-433); alpha changed ->
    // QOI_OP_RGBA (qoi.h:468-474); else DIFF if vr/vg/vb in {-2..1} (qoi.h:446-451);
    // else LUMA if vg_r/vg/vg_b in range (qoi.h:453-459); else QOI_OP_RGB
    // (qoi.h:461-466). Deltas use C signed-char wrapping (qoi.h:439-444).
    fn encode_choose_op(&mut self, px: Px) -> Vec<u8> {
        // qoi.h:430-433 — a table hit wins over everything else (the miss path
        // already stored px at its slot, qoi.h:436).
        if let Some(slot) = self.encode_index_lookup(px) {
            return vec![QOI_OP_INDEX | slot];
        }
        if px.a != self.px_prev.a {
            // qoi.h:468-474 — alpha changed: full RGBA literal.
            return vec![QOI_OP_RGBA, px.r, px.g, px.b, px.a];
        }
        // qoi.h:439-444 — signed-char wrapped deltas (e.g. r=0 vs prev=255 -> vr=+1).
        let vr = (px.r as i8).wrapping_sub(self.px_prev.r as i8);
        let vg = (px.g as i8).wrapping_sub(self.px_prev.g as i8);
        let vb = (px.b as i8).wrapping_sub(self.px_prev.b as i8);
        let vg_r = vr.wrapping_sub(vg);
        let vg_b = vb.wrapping_sub(vg);
        // qoi.h:446-451 — QOI_OP_DIFF.
        if vr > -3 && vr < 2 && vg > -3 && vg < 2 && vb > -3 && vb < 2 {
            let b1 =
                QOI_OP_DIFF | (((vr + 2) as u8) << 4) | (((vg + 2) as u8) << 2) | ((vb + 2) as u8);
            return vec![b1];
        }
        // qoi.h:453-459 — QOI_OP_LUMA.
        if vg_r > -9 && vg_r < 8 && vg > -33 && vg < 32 && vg_b > -9 && vg_b < 8 {
            let b1 = QOI_OP_LUMA | ((vg + 32) as u8);
            let b2 = (((vg_r + 8) as u8) << 4) | ((vg_b + 8) as u8);
            return vec![b1, b2];
        }
        // qoi.h:461-466 — QOI_OP_RGB.
        vec![QOI_OP_RGB, px.r, px.g, px.b]
    }

    // qoi.h:415-421 — the caller routes px == px_prev here (the run branch). The
    // counter counts repeats after the first occurrence; the field emitted is
    // run - 1 (qoi.h:418/426), so a chunk covers run pixels: run=1 -> 0xc0 (1 px),
    // run=62 -> 0xfd (62 px, the cap). Returns true if a chunk was emitted (run
    // capped at 62, or is_last); after a flush the run continues with the next
    // equal pixel.
    fn encode_run_repeat(&mut self, is_last: bool, out: &mut Vec<u8>) -> bool {
        self.run += 1;
        if self.run == 62 || is_last {
            out.push(QOI_OP_RUN | (self.run - 1));
            self.run = 0;
            true
        } else {
            false
        }
    }

    // qoi.h:425-428 — before encoding a pixel that differs from px_prev, flush any
    // pending run (run > 0), emitting QOI_OP_RUN | (run - 1).
    fn encode_run_flush(&mut self, out: &mut Vec<u8>) {
        if self.run > 0 {
            out.push(QOI_OP_RUN | (self.run - 1));
            self.run = 0;
        }
    }
}

// qoi.h:488-590 qoi_decode — decode a .qoi byte stream into the raw-dump pixel
// format (4-byte BE width, 4-byte BE height, 1-byte channels, then
// width*height*channels bytes; CLAUDE.md CLI contract). Output channels come from
// the .qoi header (qoiconv decodes with channels=0, qoi.h:523-525). Returns None
// where qoi_decode returns NULL (qoi.h:497-503, 513-521).
fn decode_to_raw(bytes: &[u8]) -> Option<Vec<u8>> {
    // qoi.h:500 — size < QOI_HEADER_SIZE + sizeof(qoi_padding) -> NULL
    if bytes.len() < QOI_HEADER_SIZE + QOI_PADDING.len() {
        return None;
    }
    let h = parse_header(bytes)?; // qoi.h:507-518 field validation
    let channels = h.channels as usize;
    let px_count = (h.width * h.height) as usize;

    let mut out = Vec::with_capacity(9 + px_count * channels);
    let mut hdr = [0u8; 8];
    let mut hp = 0usize;
    write_32(&mut hdr, &mut hp, h.width);
    write_32(&mut hdr, &mut hp, h.height);
    out.extend_from_slice(&hdr);
    out.push(h.channels);

    let mut d = Decoder::new();
    let chunks_len = bytes.len() - QOI_PADDING.len(); // qoi.h:539 (absolute offset)
    let mut p = QOI_HEADER_SIZE;
    for _ in 0..px_count {
        // qoi.h:540-578
        if d.run > 0 {
            d.decode_run_skip(); // qoi.h:541-543
        } else if p < chunks_len {
            // qoi.h:544-575 — equality checks for RGB/RGBA BEFORE the mask checks:
            // 0xfe/0xff would otherwise read as QOI_OP_RUN.
            let b1 = bytes[p];
            p += 1;
            if b1 == QOI_OP_RGB {
                d.decode_op_rgb(bytes[p], bytes[p + 1], bytes[p + 2]);
                p += 3;
            } else if b1 == QOI_OP_RGBA {
                d.decode_op_rgba(bytes[p], bytes[p + 1], bytes[p + 2], bytes[p + 3]);
                p += 4;
            } else if (b1 & QOI_MASK_2) == QOI_OP_INDEX {
                d.decode_op_index(b1);
            } else if (b1 & QOI_MASK_2) == QOI_OP_DIFF {
                d.decode_op_diff(b1);
            } else if (b1 & QOI_MASK_2) == QOI_OP_LUMA {
                let b2 = bytes[p];
                p += 1;
                d.decode_op_luma(b1, b2);
            } else if (b1 & QOI_MASK_2) == QOI_OP_RUN {
                d.decode_op_run(b1);
            }
        }
        // qoi.h:580-586 — emit the current pixel, alpha only for 4 channels.
        let px = d.px;
        out.push(px.r);
        out.push(px.g);
        out.push(px.b);
        if channels == 4 {
            out.push(px.a);
        }
    }
    Some(out)
}

// qoi.h:356-483 qoi_encode — encode raw-dump pixels (4-byte BE width/height, 1-byte
// channels, then width*height*channels bytes; CLAUDE.md CLI contract) into a .qoi
// byte stream. Colorspace is always QOI_SRGB: the raw-dump format carries no
// colorspace field. Returns None where qoi_encode returns NULL (qoi.h:364-372).
fn encode_from_raw(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.len() < 9 {
        return None;
    }
    let mut p = 0usize;
    let width = read_32(bytes, &mut p);
    let height = read_32(bytes, &mut p);
    let channels = bytes[8];
    let h = Header {
        width,
        height,
        channels,
        colorspace: QOI_SRGB,
    };
    if !validate_desc(&h) {
        return None; // qoi.h:364-372
    }
    let channels = channels as usize;
    let px_len = width as usize * height as usize * channels;
    if bytes.len() < 9 + px_len {
        return None; // truncated pixel data (Rust safety guard; valid raws never hit it)
    }

    // qoi.h:374-376 — max_size = w*h*(channels+1) + header + padding.
    let mut out = Vec::with_capacity(px_len + width as usize * height as usize + 14 + 8);
    out.extend_from_slice(&write_header(&h)); // qoi.h:384-388

    let mut e = Encoder::new();
    let pixel_count = width as usize * height as usize;
    let last_px = pixel_count - 1; // qoi.h:403 px_end (byte-level, equivalent for the last pixel)
    let mut px_pos = 9usize;
    for i in 0..pixel_count {
        // qoi.h:407-413 — for 3-channel input alpha is never read and stays 255
        // (px starts as px_prev = {0,0,0,255}, qoi.h:400, and is never reassigned).
        let a = if channels == 4 { bytes[px_pos + 3] } else { 255 };
        let px = Px { r: bytes[px_pos], g: bytes[px_pos + 1], b: bytes[px_pos + 2], a };
        px_pos += channels;
        if px == e.px_prev {
            e.encode_run_repeat(i == last_px, &mut out); // qoi.h:415-421
        } else {
            e.encode_run_flush(&mut out); // qoi.h:425-428
            out.extend_from_slice(&e.encode_choose_op(px)); // qoi.h:430-474
        }
        e.px_prev = px; // qoi.h:477
    }
    out.extend_from_slice(&QOI_PADDING); // qoi.h:480-482 (end marker)
    Some(out)
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("usage: qoi_rust <encode|decode> <in> <out>");
        return ExitCode::from(2);
    }
    let in_path = &args[2];
    let out_path = &args[3];
    match args[1].as_str() {
        "encode" => {
            let Ok(data) = std::fs::read(in_path) else {
                eprintln!("encode: cannot read {in_path}");
                return ExitCode::FAILURE;
            };
            match encode_from_raw(&data) {
                Some(qoi) => match std::fs::write(out_path, &qoi) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("encode: cannot write {out_path}: {e}");
                        ExitCode::FAILURE
                    }
                },
                None => {
                    eprintln!("encode: invalid raw input in {in_path}");
                    ExitCode::FAILURE
                }
            }
        }
        "decode" => {
            let Ok(data) = std::fs::read(in_path) else {
                eprintln!("decode: cannot read {in_path}");
                return ExitCode::FAILURE;
            };
            match decode_to_raw(&data) {
                Some(raw) => match std::fs::write(out_path, &raw) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("decode: cannot write {out_path}: {e}");
                        ExitCode::FAILURE
                    }
                },
                None => {
                    eprintln!("decode: invalid qoi data in {in_path}");
                    ExitCode::FAILURE
                }
            }
        }
        cmd => {
            eprintln!("unknown command: {cmd}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{color_hash, decode_to_raw, encode_from_raw, parse_header, read_32, validate_desc, write_32, write_end_marker, write_header, Decoder, Encoder, Header, Px, QOI_HEADER_SIZE, QOI_LINEAR, QOI_MAGIC, QOI_MASK_2, QOI_OP_DIFF, QOI_OP_INDEX, QOI_OP_LUMA, QOI_OP_RGB, QOI_OP_RGBA, QOI_OP_RUN, QOI_PADDING, QOI_PIXELS_MAX, QOI_SRGB};

    fn header_bytes(width: u32, height: u32, channels: u8, colorspace: u8) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(QOI_HEADER_SIZE);
        let mut p = 0usize;
        let mut buf = [0u8; 4];
        write_32(&mut buf, &mut p, QOI_MAGIC);
        bytes.extend_from_slice(&buf);
        let mut p = 0usize;
        write_32(&mut buf, &mut p, width);
        bytes.extend_from_slice(&buf);
        let mut p = 0usize;
        write_32(&mut buf, &mut p, height);
        bytes.extend_from_slice(&buf);
        bytes.push(channels);
        bytes.push(colorspace);
        bytes
    }

    #[test]
    fn write_32_writes_big_endian() {
        let mut bytes = [0u8; 4];
        let mut p = 0usize;
        write_32(&mut bytes, &mut p, 0x00000320);
        assert_eq!(p, 4);
        assert_eq!(&bytes, &[0x00, 0x00, 0x03, 0x20]);
    }

    #[test]
    fn write_32_advances_incrementally() {
        let mut bytes = [0u8; 16];
        let mut p = 0usize;
        write_32(&mut bytes, &mut p, 0x01020304);
        assert_eq!(p, 4);
        write_32(&mut bytes, &mut p, 0xAABBCCDD);
        assert_eq!(p, 8);
        assert_eq!(&bytes[..8], &[0x01, 0x02, 0x03, 0x04, 0xAA, 0xBB, 0xCC, 0xDD]);
    }

    #[test]
    fn write_32_writes_from_offset() {
        let mut bytes = [0xEEu8; 8];
        let mut p = 4usize;
        write_32(&mut bytes, &mut p, 0xFFFFFFFF);
        assert_eq!(p, 8);
        assert_eq!(&bytes[..4], &[0xEE, 0xEE, 0xEE, 0xEE]);
        assert_eq!(&bytes[4..], &[0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn write_32_zero() {
        let mut bytes = [1u8; 4];
        let mut p = 0usize;
        write_32(&mut bytes, &mut p, 0x00000000);
        assert_eq!(&bytes, &[0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn read_32_reads_big_endian() {
        let bytes = [0x00, 0x00, 0x03, 0x20];
        let mut p = 0usize;
        assert_eq!(read_32(&bytes, &mut p), 0x00000320);
        assert_eq!(p, 4);
    }

    #[test]
    fn read_32_from_offset() {
        let bytes = [0xEE, 0xFF, 0xFF, 0xFF, 0xFF, 0xEE];
        let mut p = 1usize;
        assert_eq!(read_32(&bytes, &mut p), 0xFFFFFFFF);
        assert_eq!(p, 5);
    }

    #[test]
    fn read_32_max_value() {
        let bytes = [0xFF, 0xFF, 0xFF, 0xFF];
        let mut p = 0usize;
        assert_eq!(read_32(&bytes, &mut p), 0xFFFFFFFF);
        assert_eq!(p, 4);
    }

    #[test]
    fn write_read_32_roundtrip() {
        let values = [0x00000000, 0x00000320, 0x00000258, 0x80000001, 0xFFFFFFFF];
        for v in values {
            let mut bytes = [0u8; 4];
            let mut p = 0usize;
            write_32(&mut bytes, &mut p, v);
            assert_eq!(p, 4);
            let mut p = 0usize;
            assert_eq!(read_32(&bytes, &mut p), v);
            assert_eq!(p, 4);
        }
    }

    #[test]
    fn parse_header_valid_srgb() {
        let bytes = header_bytes(800, 600, 4, QOI_SRGB);
        assert_eq!(
            parse_header(&bytes),
            Some(Header {
                width: 800,
                height: 600,
                channels: 4,
                colorspace: QOI_SRGB,
            })
        );
    }

    #[test]
    fn parse_header_valid_linear() {
        let bytes = header_bytes(256, 64, 3, QOI_LINEAR);
        assert_eq!(
            parse_header(&bytes),
            Some(Header {
                width: 256,
                height: 64,
                channels: 3,
                colorspace: QOI_LINEAR,
            })
        );
    }

    #[test]
    fn parse_header_rejects_bad_magic() {
        let mut bytes = header_bytes(1, 1, 4, QOI_SRGB);
        bytes[0] = b'x';
        assert_eq!(parse_header(&bytes), None);
    }

    #[test]
    fn parse_header_rejects_zero_width() {
        let bytes = header_bytes(0, 100, 4, QOI_SRGB);
        assert_eq!(parse_header(&bytes), None);
    }

    #[test]
    fn parse_header_rejects_zero_height() {
        let bytes = header_bytes(100, 0, 4, QOI_SRGB);
        assert_eq!(parse_header(&bytes), None);
    }

    #[test]
    fn parse_header_rejects_bad_channels() {
        assert_eq!(parse_header(&header_bytes(1, 1, 2, QOI_SRGB)), None);
        assert_eq!(parse_header(&header_bytes(1, 1, 5, QOI_SRGB)), None);
    }

    #[test]
    fn parse_header_rejects_bad_colorspace() {
        let bytes = header_bytes(1, 1, 4, 2);
        assert_eq!(parse_header(&bytes), None);
    }

    #[test]
    fn parse_header_rejects_overflow_pixels() {
        assert_eq!(parse_header(&header_bytes(1, QOI_PIXELS_MAX, 4, QOI_SRGB)), None);
    }

    #[test]
    fn parse_header_accepts_max_pixels() {
        assert_eq!(
            parse_header(&header_bytes(1, QOI_PIXELS_MAX - 1, 4, QOI_SRGB)),
            Some(Header {
                width: 1,
                height: QOI_PIXELS_MAX - 1,
                channels: 4,
                colorspace: QOI_SRGB,
            })
        );
    }

    #[test]
    fn parse_header_rejects_short_input() {
        let bytes = header_bytes(1, 1, 4, QOI_SRGB);
        assert_eq!(parse_header(&bytes[..QOI_HEADER_SIZE - 1]), None);
    }

    #[test]
    fn write_header_writes_exact_14_bytes() {
        let h = Header {
            width: 800,
            height: 600,
            channels: 4,
            colorspace: QOI_SRGB,
        };
        let bytes = write_header(&h);
        assert_eq!(
            bytes,
            vec![0x71, 0x6f, 0x69, 0x66, 0x00, 0x00, 0x03, 0x20, 0x00, 0x00, 0x02, 0x58, 0x04, 0x00]
        );
    }

    #[test]
    fn write_header_matches_oracle_dice() {
        let h = Header {
            width: 800,
            height: 600,
            channels: 4,
            colorspace: QOI_SRGB,
        };
        let bytes = write_header(&h);
        assert_eq!(&bytes[..4], b"qoif");
        assert_eq!(&bytes[4..8], &[0x00, 0x00, 0x03, 0x20]);
        assert_eq!(&bytes[8..12], &[0x00, 0x00, 0x02, 0x58]);
        assert_eq!(bytes[12], 4);
        assert_eq!(bytes[13], 0);
    }

    #[test]
    fn write_header_roundtrips_through_parse() {
        let h = Header {
            width: 3,
            height: 5,
            channels: 3,
            colorspace: QOI_LINEAR,
        };
        let bytes = write_header(&h);
        assert_eq!(bytes.len(), QOI_HEADER_SIZE);
        assert_eq!(parse_header(&bytes), Some(h));
    }

    #[test]
    fn end_marker_is_seven_zeros_then_one() {
        let mut bytes = Vec::new();
        write_end_marker(&mut bytes);
        assert_eq!(bytes, vec![0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(bytes, QOI_PADDING.to_vec());
    }

    #[test]
    fn end_marker_appends_after_existing_bytes() {
        let mut bytes = vec![0xAB; 3];
        write_end_marker(&mut bytes);
        assert_eq!(bytes.len(), 11);
        assert_eq!(&bytes[..3], &[0xAB, 0xAB, 0xAB]);
        assert_eq!(&bytes[3..], &[0, 0, 0, 0, 0, 0, 0, 1]);
    }

    #[test]
    fn end_marker_matches_oracle_dice_tail() {
        let data = std::fs::read("oracle/outputs/dice.qoi").unwrap();
        let tail = &data[data.len() - 8..];
        assert_eq!(tail, QOI_PADDING);
    }

    #[test]
    fn validate_desc_accepts_valid() {
        for h in [
            Header { width: 1, height: 1, channels: 3, colorspace: QOI_SRGB },
            Header { width: 1, height: 1, channels: 4, colorspace: QOI_SRGB },
            Header { width: 1, height: 1, channels: 3, colorspace: QOI_LINEAR },
            Header { width: 1, height: 399999999, channels: 4, colorspace: QOI_SRGB },
            Header { width: 200000000, height: 1, channels: 4, colorspace: QOI_SRGB },
            Header { width: 800, height: 600, channels: 4, colorspace: QOI_SRGB },
        ] {
            assert!(validate_desc(&h), "should accept: {h:?}");
        }
    }

    #[test]
    fn validate_desc_rejects_zero_width() {
        assert!(!validate_desc(&Header { width: 0, height: 1, channels: 4, colorspace: QOI_SRGB }));
    }

    #[test]
    fn validate_desc_rejects_zero_height() {
        assert!(!validate_desc(&Header { width: 1, height: 0, channels: 4, colorspace: QOI_SRGB }));
    }

    #[test]
    fn validate_desc_rejects_channels_out_of_3_4() {
        assert!(!validate_desc(&Header { width: 1, height: 1, channels: 2, colorspace: QOI_SRGB }));
        assert!(!validate_desc(&Header { width: 1, height: 1, channels: 5, colorspace: QOI_SRGB }));
        assert!(!validate_desc(&Header { width: 1, height: 1, channels: 0, colorspace: QOI_SRGB }));
    }

    #[test]
    fn validate_desc_rejects_colorspace_out_of_0_1() {
        assert!(!validate_desc(&Header { width: 1, height: 1, channels: 4, colorspace: 2 }));
        assert!(!validate_desc(&Header { width: 1, height: 1, channels: 4, colorspace: 255 }));
    }

    #[test]
    fn validate_desc_rejects_pixel_count_overflow() {
        assert!(!validate_desc(&Header { width: 1, height: 400000000, channels: 4, colorspace: QOI_SRGB }));
        assert!(!validate_desc(&Header { width: 400000000, height: 1, channels: 4, colorspace: QOI_SRGB }));
        assert!(!validate_desc(&Header { width: 200000000, height: 2, channels: 4, colorspace: QOI_SRGB }));
    }

    // qoi.h:322 — 0*3+0*5+0*7+0*11 = 0 -> slot 0. Also the state of every index[]
    // entry after QOI_ZEROARR (qoi.h:361 encode, qoi.h:533 decode), so slot 0 is
    // where the transparent-black pixel lands in a freshly-reset table.
    #[test]
    fn color_hash_zero_pixel_is_slot_zero() {
        assert_eq!(color_hash(0, 0, 0, 0), 0);
    }

    // qoi.h:322 — channel weights are exactly r*3, g*5, b*7, a*11.
    #[test]
    fn color_hash_channel_weights() {
        assert_eq!(color_hash(1, 0, 0, 0), 3);
        assert_eq!(color_hash(0, 1, 0, 0), 5);
        assert_eq!(color_hash(0, 0, 1, 0), 7);
        assert_eq!(color_hash(0, 0, 0, 1), 11);
    }

    // qoi.h:322 — full-scale channels; would overflow u8 without the C-style int
    // promotion (255*11 = 2805). Exact slots: 765%64=61, 1275%64=59, 1785%64=57,
    // 2805%64=53. The last one is also the decoder's starting px {0,0,0,255}
    // (qoi.h:534-537) and encoder's px_prev (qoi.h:396-399).
    #[test]
    fn color_hash_single_channel_255() {
        assert_eq!(color_hash(255, 0, 0, 0), 61);
        assert_eq!(color_hash(0, 255, 0, 0), 59);
        assert_eq!(color_hash(0, 0, 255, 0), 57);
        assert_eq!(color_hash(0, 0, 0, 255), 53);
    }

    // qoi.h:322 — 255*26 = 6630, slot = 6630 % 64 = 38.
    #[test]
    fn color_hash_white_opaque() {
        assert_eq!(color_hash(255, 255, 255, 255), 38);
    }

    // Slot is hash & (64 - 1) (qoi.h:430 encode, qoi.h:577 decode), not the raw sum:
    // (0,4,0,4): 0+20+0+44 = 64 -> wraps to slot 0; (10,20,30,40): 780 % 64 = 12.
    #[test]
    fn color_hash_wraps_mod_64() {
        assert_eq!(color_hash(0, 4, 0, 4), 0);
        assert_eq!(color_hash(10, 20, 30, 40), 12);
    }

    // Hash is a slot, not an identity: (2,1,0,0) = 2*3+1*5 = 11 collides with
    // (0,0,0,1) = 11. The reference resolves collisions by comparing the full pixel
    // value (qoi.h:432), so color_hash must NOT be treated as unique.
    #[test]
    fn color_hash_collisions_are_expected() {
        assert_eq!(color_hash(2, 1, 0, 0), 11);
        assert_eq!(color_hash(0, 0, 0, 1), 11);
    }

    // qoi.h:533 (QOI_ZEROARR = memset 0, qoi.h:310) + qoi.h:534-537: a fresh decoder
    // has an all-zero index table and px = {0,0,0,255}.
    #[test]
    fn decode_op_index_initial_decoder_state() {
        let d = Decoder::new();
        assert_eq!(d.px, Px { r: 0, g: 0, b: 0, a: 255 });
        assert!(d.index.iter().all(|p| *p == Px { r: 0, g: 0, b: 0, a: 0 }));
    }

    // qoi.h:559 — px = index[b1]: the op returns exactly what the table holds at
    // slot b1. Slots precomputed from the qoi.h:322 formula: {10,20,30,255} sums to
    // 3145, 3145%64 = 9; {200,100,50,255} sums to 4255, 4255%64 = 31.
    #[test]
    fn decode_op_index_reads_seeded_slot() {
        let mut d = Decoder::new();
        d.px = Px { r: 10, g: 20, b: 30, a: 255 };
        d.index_update(); // qoi.h:577 -> slot 9
        d.px = Px { r: 200, g: 100, b: 50, a: 255 };
        d.index_update(); // qoi.h:577 -> slot 31
        assert_eq!(d.decode_op_index(9), Px { r: 10, g: 20, b: 30, a: 255 });
        assert_eq!(d.decode_op_index(31), Px { r: 200, g: 100, b: 50, a: 255 });
    }

    // qoi.h:313/320 — a QOI_OP_INDEX byte has top 2 bits 00, so b1 IS the slot
    // number (0..=63); qoi.h:559 indexes with the whole byte. Max slot 63 seeded
    // via {0,2,0,255}: 0+10+0+2805 = 2815, 2815%64 = 63.
    #[test]
    fn decode_op_index_byte_is_slot_number() {
        let mut d = Decoder::new();
        d.px = Px { r: 0, g: 2, b: 0, a: 255 };
        d.index_update(); // slot 63
        assert_eq!(d.decode_op_index(0x3f), Px { r: 0, g: 2, b: 0, a: 255 });
    }

    // qoi.h:533 — an INDEX op hitting a never-written slot yields the ZEROARR zero
    // pixel {0,0,0,0}, NOT the initial px {0,0,0,255}. Table state, not the running
    // px, drives INDEX lookups.
    #[test]
    fn decode_op_index_unwritten_slot_is_zero_pixel() {
        let mut d = Decoder::new();
        assert_eq!(d.decode_op_index(0), Px { r: 0, g: 0, b: 0, a: 0 });
        assert_eq!(d.decode_op_index(17), Px { r: 0, g: 0, b: 0, a: 0 });
    }

    // qoi.h:559 — the op also advances the decoder's running px (later ops are
    // relative to it: DIFF/LUMA deltas at qoi.h:561-572).
    #[test]
    fn decode_op_index_sets_running_px() {
        let mut d = Decoder::new();
        d.px = Px { r: 10, g: 20, b: 30, a: 255 };
        d.index_update(); // slot 9
        d.px = Px { r: 1, g: 1, b: 1, a: 1 }; // clobber running px (not written to table)
        assert_eq!(d.decode_op_index(9), Px { r: 10, g: 20, b: 30, a: 255 });
        assert_eq!(d.px, Px { r: 10, g: 20, b: 30, a: 255 });
    }

    // qoi.h:577 — the index write after an INDEX op rewrites the same slot the pixel
    // was just read from (hash is deterministic), so the table is unchanged. The
    // write exists for the OTHER opcodes; this pins its no-op nature for INDEX.
    #[test]
    fn decode_op_index_table_write_is_same_slot_noop() {
        let mut d = Decoder::new();
        d.px = Px { r: 10, g: 20, b: 30, a: 255 };
        d.index_update(); // slot 9
        let before = d.index;
        d.decode_op_index(9);
        assert_eq!(d.index, before);
    }

    // qoi.h:558 with qoi.h:313/320 — a byte is QOI_OP_INDEX iff
    // (b1 & QOI_MASK_2) == QOI_OP_INDEX, i.e. exactly 0x00..=0x3f; 0x40 is already
    // QOI_OP_DIFF (qoi.h:314). Pins the two constants' bit layout.
    #[test]
    fn decode_op_index_opcode_range() {
        for b1 in 0x00u8..=0x3f {
            assert_eq!(b1 & QOI_MASK_2, QOI_OP_INDEX);
        }
        assert_ne!(0x40 & QOI_MASK_2, QOI_OP_INDEX);
    }

    // qoi.h:561-565 — b1 = 0x40 is 01_00_00_00, so dr=dg=db=-2 (bias: 0-2). From the
    // initial px {0,0,0,255}, C's uint8_t wrap gives 0-2 = 254 per channel.
    #[test]
    fn decode_op_diff_min_deltas_wrap() {
        let mut d = Decoder::new();
        assert_eq!(d.decode_op_diff(0x40), Px { r: 254, g: 254, b: 254, a: 255 });
    }

    // qoi.h:561-565 — b1 = 0x7f is 01_11_11_11, the largest DIFF byte: each field is
    // 3, delta = 3-2 = +1. From {0,0,0,255} -> {1,1,1,255}.
    #[test]
    fn decode_op_diff_max_deltas() {
        let mut d = Decoder::new();
        assert_eq!(d.decode_op_diff(0x7f), Px { r: 1, g: 1, b: 1, a: 255 });
    }

    // qoi.h:561-565 + encode emit qoi.h:451 — zero deltas encode as
    // QOI_OP_DIFF | 2<<4 | 2<<2 | 2 = 0x6a; the pixel must come through unchanged.
    #[test]
    fn decode_op_diff_zero_deltas() {
        let mut d = Decoder::new();
        d.px = Px { r: 10, g: 20, b: 30, a: 255 };
        assert_eq!(d.decode_op_diff(0x6a), Px { r: 10, g: 20, b: 30, a: 255 });
    }

    // qoi.h:561-565 — 0x74 = 01_11_01_00: dr=3-2=+1, dg=1-2=-1, db=0-2=-2.
    // From {10,20,30,255} -> {11,19,28,255}.
    #[test]
    fn decode_op_diff_mixed_deltas() {
        let mut d = Decoder::new();
        d.px = Px { r: 10, g: 20, b: 30, a: 255 };
        assert_eq!(d.decode_op_diff(0x74), Px { r: 11, g: 19, b: 28, a: 255 });
    }

    // qoi.h:561-565 — wrap in the other direction: 255 + 1 = 0 per channel.
    #[test]
    fn decode_op_diff_wraps_upward() {
        let mut d = Decoder::new();
        d.px = Px { r: 255, g: 255, b: 255, a: 255 };
        assert_eq!(d.decode_op_diff(0x7f), Px { r: 0, g: 0, b: 0, a: 255 });
    }

    // qoi.h:561-565 — the op touches r/g/b only; alpha passes through untouched
    // even when it is not 255 (encode only emits DIFF when alpha matches, qoi.h:438,
    // but the decoder itself never reads/writes a here).
    #[test]
    fn decode_op_diff_alpha_unchanged() {
        let mut d = Decoder::new();
        d.px = Px { r: 10, g: 20, b: 30, a: 7 };
        assert_eq!(d.decode_op_diff(0x7f), Px { r: 11, g: 21, b: 31, a: 7 });
    }

    // qoi.h:577 — after a DIFF op the new pixel must land in the index table at
    // color_hash(px): {1,1,1,255} sums to 3+5+7+2805 = 2820, 2820%64 = 4, so a
    // subsequent INDEX op to slot 4 must return it. (This is the per-pixel table
    // maintenance that is a no-op for INDEX but observable here.)
    #[test]
    fn decode_op_diff_updates_index_table() {
        let mut d = Decoder::new();
        d.decode_op_diff(0x7f); // px becomes {1,1,1,255} -> slot 4
        assert_eq!(d.decode_op_index(4), Px { r: 1, g: 1, b: 1, a: 255 });
    }

    // qoi.h:561 with qoi.h:314/320 — a byte is QOI_OP_DIFF iff
    // (b1 & QOI_MASK_2) == QOI_OP_DIFF, i.e. exactly 0x40..=0x7f; 0x80 is already
    // QOI_OP_LUMA (qoi.h:315).
    #[test]
    fn decode_op_diff_opcode_range() {
        for b1 in 0x40u8..=0x7f {
            assert_eq!(b1 & QOI_MASK_2, QOI_OP_DIFF);
        }
        assert_ne!(0x80 & QOI_MASK_2, QOI_OP_DIFF);
    }

    // qoi.h:566-572 — b1 = 0x80 is 10_000000: vg = 0-32 = -32 (min). b2 = 0x00:
    // vg_r = vg_b = 0-8 = -8 (min). From {0,0,0,255}: r/b = 0-40 wraps to 216,
    // g = 0-32 wraps to 224.
    #[test]
    fn decode_op_luma_min_deltas_wrap() {
        let mut d = Decoder::new();
        assert_eq!(d.decode_op_luma(0x80, 0x00), Px { r: 216, g: 224, b: 216, a: 255 });
    }

    // qoi.h:566-572 — b1 = 0xbf (10_111111): vg = 63-32 = 31 (max). b2 = 0xff:
    // vg_r = vg_b = 15-8 = 7 (max). From {0,0,0,255} -> {38,31,38,255}.
    #[test]
    fn decode_op_luma_max_deltas() {
        let mut d = Decoder::new();
        assert_eq!(d.decode_op_luma(0xbf, 0xff), Px { r: 38, g: 31, b: 38, a: 255 });
    }

    // qoi.h:566-572 + encode emit qoi.h:458-459 — zero deltas: vg=0 -> b1 = 0x80|32
    // = 0xa0; vg_r=vg_b=0 -> b2 = 8<<4|8 = 0x88. Pixel unchanged.
    #[test]
    fn decode_op_luma_zero_deltas() {
        let mut d = Decoder::new();
        d.px = Px { r: 10, g: 20, b: 30, a: 255 };
        assert_eq!(d.decode_op_luma(0xa0, 0x88), Px { r: 10, g: 20, b: 30, a: 255 });
    }

    // qoi.h:566-572 — vg=10 -> b1 = 0x80|42 = 0xaa; vg_r=-5, vg_b=3 -> b2 =
    // 3<<4|11 = 0x3b. From {100,100,100,255}: g=110, r=100+10-5=105, b=100+10+3=113.
    #[test]
    fn decode_op_luma_mixed_deltas() {
        let mut d = Decoder::new();
        d.px = Px { r: 100, g: 100, b: 100, a: 255 };
        assert_eq!(d.decode_op_luma(0xaa, 0x3b), Px { r: 105, g: 110, b: 113, a: 255 });
    }

    // qoi.h:566-572 — upward wrap: from {250,250,250,255} with max deltas,
    // r/b = 250+38 = 288 mod 256 = 32, g = 250+31 = 281 mod 256 = 25.
    #[test]
    fn decode_op_luma_wraps_upward() {
        let mut d = Decoder::new();
        d.px = Px { r: 250, g: 250, b: 250, a: 255 };
        assert_eq!(d.decode_op_luma(0xbf, 0xff), Px { r: 32, g: 25, b: 32, a: 255 });
    }

    // qoi.h:566-572 — alpha is never read or written by LUMA. b1=0x81 (vg=-31),
    // b2=0x00 (vg_r=vg_b=-8): r=50-39=11, g=60-31=29, b=70-39=31, a stays 9.
    #[test]
    fn decode_op_luma_alpha_unchanged() {
        let mut d = Decoder::new();
        d.px = Px { r: 50, g: 60, b: 70, a: 9 };
        assert_eq!(d.decode_op_luma(0x81, 0x00), Px { r: 11, g: 29, b: 31, a: 9 });
    }

    // qoi.h:577 — the LUMA result lands in the index table at color_hash(px):
    // {38,31,38,255} sums to 3340, 3340%64 = 12, so INDEX slot 12 returns it next.
    #[test]
    fn decode_op_luma_updates_index_table() {
        let mut d = Decoder::new();
        d.decode_op_luma(0xbf, 0xff); // px becomes {38,31,38,255} -> slot 12
        assert_eq!(d.decode_op_index(12), Px { r: 38, g: 31, b: 38, a: 255 });
    }

    // qoi.h:566 with qoi.h:315/320 — a byte is QOI_OP_LUMA iff
    // (b1 & QOI_MASK_2) == QOI_OP_LUMA, i.e. exactly 0x80..=0xbf; 0xc0 is already
    // QOI_OP_RUN (qoi.h:316).
    #[test]
    fn decode_op_luma_opcode_range() {
        for b1 in 0x80u8..=0xbf {
            assert_eq!(b1 & QOI_MASK_2, QOI_OP_LUMA);
        }
        assert_ne!(0xc0 & QOI_MASK_2, QOI_OP_LUMA);
    }

    // qoi.h:573-575 — run = b1 & 0x3f: 0xc0 -> 0, 0xc5 -> 5, 0xfd -> 61 (max the
    // encoder can emit: run-1 with run capped at 62, qoi.h:417-418). px unchanged;
    // a fresh decoder has run = 0 (qoi.h:495).
    #[test]
    fn decode_op_run_sets_run_count() {
        let mut d = Decoder::new();
        assert_eq!(d.run, 0);
        d.px = Px { r: 10, g: 20, b: 30, a: 255 };
        assert_eq!(d.decode_op_run(0xc5), Px { r: 10, g: 20, b: 30, a: 255 });
        assert_eq!(d.run, 5);
        d.decode_op_run(0xc0);
        assert_eq!(d.run, 0);
        d.decode_op_run(0xfd);
        assert_eq!(d.run, 61);
    }

    // qoi.h:573-575 + qoi.h:541-543 — a RUN chunk emits the current pixel, then run
    // more copies follow via skips: 1 + (b1 & 0x3f) total, matching the encoder's
    // run-1 field (qoi.h:418). 0xc2 -> 3 copies of {10,20,30,255}.
    #[test]
    fn decode_op_run_repeats_previous_pixel() {
        let mut d = Decoder::new();
        d.px = Px { r: 10, g: 20, b: 30, a: 255 };
        let px = Px { r: 10, g: 20, b: 30, a: 255 };
        assert_eq!(d.decode_op_run(0xc2), px); // chunk pixel
        assert_eq!(d.decode_run_skip(), px); // run 2 -> 1
        assert_eq!(d.decode_run_skip(), px); // run 1 -> 0
        assert_eq!(d.run, 0);
    }

    // qoi.h:577 — the RUN-chunk pixel IS written to the index table (the RUN branch
    // is inside the chunk-reading else-if, qoi.h:544). Fresh decoder: px {0,0,0,255}
    // hashes to slot 53 (2805%64), which initially holds the ZEROARR {0,0,0,0};
    // after the RUN chunk, slot 53 must hold {0,0,0,255}.
    #[test]
    fn decode_op_run_chunk_pixel_updates_index() {
        let mut d = Decoder::new();
        d.decode_op_run(0xc0);
        assert_eq!(d.decode_op_index(53), Px { r: 0, g: 0, b: 0, a: 255 });
    }

    // qoi.h:541-543 — run skips read no chunk and touch nothing: px and the whole
    // index table are byte-identical before and after (the qoi.h:577 write lives
    // inside the else-if the skip bypasses).
    #[test]
    fn decode_op_run_skip_touches_nothing() {
        let mut d = Decoder::new();
        d.px = Px { r: 7, g: 8, b: 9, a: 10 };
        d.index_update(); // seed table with current px
        d.decode_op_run(0xc2);
        let table_before = d.index;
        d.decode_run_skip();
        d.decode_run_skip();
        assert_eq!(d.index, table_before);
        assert_eq!(d.px, Px { r: 7, g: 8, b: 9, a: 10 });
        assert_eq!(d.run, 0);
    }

    // qoi.h:573 with qoi.h:316/320 — bytes 0xc0..=0xfd are RUN. 0xfe/0xff match the
    // mask too, but the reference dispatch checks QOI_OP_RGB (0xfe, qoi.h:317) and
    // QOI_OP_RGBA (0xff, qoi.h:318) by equality FIRST (qoi.h:547/552 precede 573),
    // so they never reach the RUN branch.
    #[test]
    fn decode_op_run_opcode_range() {
        for b1 in 0xc0u8..=0xfd {
            assert_eq!(b1 & QOI_MASK_2, QOI_OP_RUN);
        }
        assert_eq!(0xfe & QOI_MASK_2, QOI_OP_RUN); // masked out by earlier RGB check
        assert_eq!(QOI_OP_RGBA & QOI_MASK_2, QOI_OP_RUN); // masked out by earlier RGBA check
    }

    // qoi.h:547-551 — the 3 bytes after 0xfe become r/g/b verbatim; alpha untouched.
    // From {1,2,3,255}: (200,100,50) -> {200,100,50,255}.
    #[test]
    fn decode_op_rgb_sets_rgb_keeps_alpha() {
        let mut d = Decoder::new();
        d.px = Px { r: 1, g: 2, b: 3, a: 255 };
        assert_eq!(d.decode_op_rgb(200, 100, 50), Px { r: 200, g: 100, b: 50, a: 255 });
    }

    // qoi.h:547-551 — alpha passes through even when it is not 255 (the stream
    // simply has no alpha byte for RGB).
    #[test]
    fn decode_op_rgb_alpha_unchanged() {
        let mut d = Decoder::new();
        d.px = Px { r: 1, g: 2, b: 3, a: 7 };
        assert_eq!(d.decode_op_rgb(9, 8, 7), Px { r: 9, g: 8, b: 7, a: 7 });
    }

    // qoi.h:547-551 + qoi.h:534-537 — on a fresh decoder, RGB (0,0,0) yields
    // {0,0,0,255}: the initial alpha 255 survives, so this is NOT the ZEROARR
    // zero pixel {0,0,0,0}.
    #[test]
    fn decode_op_rgb_black_keeps_initial_alpha() {
        let mut d = Decoder::new();
        assert_eq!(d.decode_op_rgb(0, 0, 0), Px { r: 0, g: 0, b: 0, a: 255 });
    }

    // qoi.h:577 — the literal pixel lands in the index table: {200,100,50,255}
    // sums to 4255, 4255%64 = 31, so INDEX slot 31 returns it next.
    #[test]
    fn decode_op_rgb_updates_index_table() {
        let mut d = Decoder::new();
        d.decode_op_rgb(200, 100, 50); // px becomes {200,100,50,255} -> slot 31
        assert_eq!(d.decode_op_index(31), Px { r: 200, g: 100, b: 50, a: 255 });
    }

    // qoi.h:317 — QOI_OP_RGB is exactly 0xfe, and (0xfe & QOI_MASK_2) == QOI_OP_RUN:
    // the equality check MUST precede the mask checks (qoi.h:547 before 558-573) or
    // RGB bytes would be misread as runs. Pins the constant and the dispatch order.
    #[test]
    fn decode_op_rgb_opcode_byte() {
        assert_eq!(QOI_OP_RGB, 0xfe);
        assert_eq!(QOI_OP_RGB & QOI_MASK_2, QOI_OP_RUN);
    }

    // qoi.h:552-557 — the 4 bytes after 0xff become r/g/b/a verbatim, including
    // alpha (unlike RGB). From {1,2,3,4}: (200,100,50,25) -> {200,100,50,25}.
    #[test]
    fn decode_op_rgba_sets_all_channels() {
        let mut d = Decoder::new();
        d.px = Px { r: 1, g: 2, b: 3, a: 4 };
        assert_eq!(d.decode_op_rgba(200, 100, 50, 25), Px { r: 200, g: 100, b: 50, a: 25 });
    }

    // qoi.h:552-557 + qoi.h:534-537 — the only op that can change alpha away from
    // the initial 255: on a fresh decoder, (0,0,0,0) yields the true zero pixel.
    #[test]
    fn decode_op_rgba_can_set_zero_alpha() {
        let mut d = Decoder::new();
        assert_eq!(d.decode_op_rgba(0, 0, 0, 0), Px { r: 0, g: 0, b: 0, a: 0 });
    }

    // qoi.h:577 — the literal pixel lands in the index table: {200,100,50,25} sums
    // to 600+500+350+275 = 1725, 1725%64 = 61, so INDEX slot 61 returns it next.
    #[test]
    fn decode_op_rgba_updates_index_table() {
        let mut d = Decoder::new();
        d.decode_op_rgba(200, 100, 50, 25); // px becomes {200,100,50,25} -> slot 61
        assert_eq!(d.decode_op_index(61), Px { r: 200, g: 100, b: 50, a: 25 });
    }

    // qoi.h:318 — QOI_OP_RGBA is exactly 0xff, and (0xff & QOI_MASK_2) == QOI_OP_RUN
    // (0xff would read as a run of 63!): the equality check MUST precede the mask
    // checks (qoi.h:552 before 558-573). Pins the constant and the dispatch order.
    #[test]
    fn decode_op_rgba_opcode_byte() {
        assert_eq!(QOI_OP_RGBA, 0xff);
        assert_eq!(QOI_OP_RGBA & QOI_MASK_2, QOI_OP_RUN);
    }

    // qoi.h:540-578 — one chunk sequence through every op. 7x1, 4 channels:
    //   fe 0a 14 1e    RGB  -> {10,20,30,255}          (qoi.h:547-551)
    //   a0 88          LUMA vg=0, vg_r=vg_b=0 -> same  (qoi.h:566-572)
    //   7f             DIFF +1/+1/+1 -> {11,21,31,255} (qoi.h:561-565)
    //   18             INDEX slot 24 -> {11,21,31,255} (qoi.h:558-559; slot written
    //                  by the DIFF's qoi.h:577 update: 3*11+5*21+7*31+11*255 = 3160,
    //                  3160%64 = 24)
    //   c1             RUN run=1 -> {11,21,31,255} x2  (qoi.h:573-575 + 541-543)
    //   ff 05 06 07 08 RGBA -> {5,6,7,8}               (qoi.h:552-557)
    #[test]
    fn decode_full_decodes_handcrafted_all_ops_stream() {
        let mut bytes = header_bytes(7, 1, 4, QOI_SRGB);
        bytes.extend_from_slice(&[
            0xfe, 0x0a, 0x14, 0x1e,
            0xa0, 0x88,
            0x7f,
            0x18,
            0xc1,
            0xff, 0x05, 0x06, 0x07, 0x08,
        ]);
        bytes.extend_from_slice(&QOI_PADDING);
        let raw = decode_to_raw(&bytes).unwrap();
        let mut expected = vec![0, 0, 0, 7, 0, 0, 0, 1, 4]; // raw dump header
        for px in [
            [10, 20, 30, 255],
            [10, 20, 30, 255],
            [11, 21, 31, 255],
            [11, 21, 31, 255],
            [11, 21, 31, 255],
            [11, 21, 31, 255],
            [5, 6, 7, 8],
        ] {
            expected.extend_from_slice(&px);
        }
        assert_eq!(raw, expected);
    }

    // qoi.h:580-586 — with channels=3 the output rows carry no alpha byte.
    // 2x1 3ch: fe 01 02 03 (RGB), c0 (RUN run=0) -> {1,2,3},{1,2,3}.
    #[test]
    fn decode_full_three_channel_output() {
        let mut bytes = header_bytes(2, 1, 3, QOI_SRGB);
        bytes.extend_from_slice(&[0xfe, 0x01, 0x02, 0x03, 0xc0]);
        bytes.extend_from_slice(&QOI_PADDING);
        assert_eq!(
            decode_to_raw(&bytes).unwrap(),
            vec![0, 0, 0, 2, 0, 0, 0, 1, 3, 1, 2, 3, 1, 2, 3]
        );
    }

    // qoi.h:500 — input smaller than header + padding is rejected (NULL).
    #[test]
    fn decode_full_rejects_short_input() {
        let mut bytes = header_bytes(1, 1, 4, QOI_SRGB);
        bytes.extend_from_slice(&[0xfe, 0, 0, 0]); // 18 bytes < 14 + 8
        assert_eq!(decode_to_raw(&bytes), None);
    }

    // qoi.h:544 — once p reaches chunks_len the loop just re-emits px. 3x1 4ch with
    // a single RGB chunk: pixels 2 and 3 repeat {1,2,3,255}.
    #[test]
    fn decode_full_exhausted_stream_repeats_last_px() {
        let mut bytes = header_bytes(3, 1, 4, QOI_SRGB);
        bytes.extend_from_slice(&[0xfe, 0x01, 0x02, 0x03]);
        bytes.extend_from_slice(&QOI_PADDING);
        let mut expected = vec![0, 0, 0, 3, 0, 0, 0, 1, 4];
        for _ in 0..3 {
            expected.extend_from_slice(&[1, 2, 3, 255]);
        }
        assert_eq!(decode_to_raw(&bytes).unwrap(), expected);
    }

    // qoi.h:393 + qoi.h:430-433 — a fresh encoder's table is all-zero (QOI_ZEROARR),
    // so the zero pixel {0,0,0,0} hits slot 0 immediately (hash of zero = 0). This
    // is exactly why oracle dice.qoi's first chunk is 0x00 (INDEX slot 0).
    #[test]
    fn encode_index_table_zero_pixel_hits_fresh_table() {
        let mut e = Encoder::new();
        assert_eq!(e.encode_index_lookup(Px { r: 0, g: 0, b: 0, a: 0 }), Some(0));
    }

    // qoi.h:430-436 — a miss stores the pixel (returning None), a later identical
    // pixel hits its slot (returning Some(slot)). {10,20,30,255} hashes to
    // 3145%64 = 9.
    #[test]
    fn encode_index_table_miss_then_hit() {
        let mut e = Encoder::new();
        let px = Px { r: 10, g: 20, b: 30, a: 255 };
        assert_eq!(e.encode_index_lookup(px), None); // stored at slot 9
        assert_eq!(e.encode_index_lookup(px), Some(9));
    }

    // qoi.h:436 — the store REPLACES whatever was at the slot; the compare at
    // qoi.h:432 is on the full 4-channel value (.v), so alpha participates. Both
    // {2,1,0,0} and {0,0,0,1} hash to slot 11 (2*3+1*5 = 11*1 = 11): every lookup
    // overwrites the other entry and misses — with two colliders on one slot,
    // neither ever hits.
    #[test]
    fn encode_index_table_collision_replaces_entry() {
        let mut e = Encoder::new();
        let a = Px { r: 2, g: 1, b: 0, a: 0 };
        let b = Px { r: 0, g: 0, b: 0, a: 1 };
        assert_eq!(e.encode_index_lookup(a), None); // slot 11: zero -> a
        assert_eq!(e.encode_index_lookup(b), None); // slot 11: a -> b (b != a)
        assert_eq!(e.encode_index_lookup(a), None); // slot 11: b -> a (a != b)
        assert_eq!(e.encode_index_lookup(b), None); // slot 11: a -> b (b != a) again
        assert_eq!(e.index[11], b); // ends holding the last-written pixel
    }

    // qoi.h:430/433 — Some() carries the true slot number (used as QOI_OP_INDEX | slot).
    // {200,100,50,255} sums to 4255, 4255%64 = 31.
    #[test]
    fn encode_index_table_returns_actual_slot() {
        let mut e = Encoder::new();
        let px = Px { r: 200, g: 100, b: 50, a: 255 };
        assert_eq!(e.encode_index_lookup(px), None);
        assert_eq!(e.encode_index_lookup(px), Some(31));
    }

    // qoi.h:432-433 — a table hit wins over every other op, even one the transition
    // would otherwise take: px {10,20,30,255} was stored at slot 9 by a prior miss,
    // and from px_prev {10,20,30,254} the alpha-diff would force RGBA (qoi.h:468),
    // yet the hit emits QOI_OP_INDEX | 9 = 0x09.
    #[test]
    fn encode_choose_op_index_hit_wins() {
        let mut e = Encoder::new();
        e.encode_index_lookup(Px { r: 10, g: 20, b: 30, a: 255 }); // miss, stored at slot 9
        e.px_prev = Px { r: 10, g: 20, b: 30, a: 254 };
        assert_eq!(e.encode_choose_op(Px { r: 10, g: 20, b: 30, a: 255 }), vec![0x09]);
    }

    // qoi.h:446-451 — deltas +1/+1/+1 encode as QOI_OP_DIFF | 3<<4 | 3<<2 | 3 = 0x7f.
    // (Expected bytes independently verified against a PS simulation of qoi.h.)
    #[test]
    fn encode_choose_op_diff_positive() {
        let mut e = Encoder::new();
        e.px_prev = Px { r: 10, g: 20, b: 30, a: 255 };
        assert_eq!(e.encode_choose_op(Px { r: 11, g: 21, b: 31, a: 255 }), vec![0x7f]);
    }

    // qoi.h:446-451 — deltas -2/-1/+1 encode as QOI_OP_DIFF | 0<<4 | 1<<2 | 3 = 0x47.
    #[test]
    fn encode_choose_op_diff_negative_mixed() {
        let mut e = Encoder::new();
        e.px_prev = Px { r: 100, g: 100, b: 100, a: 255 };
        assert_eq!(e.encode_choose_op(Px { r: 98, g: 99, b: 101, a: 255 }), vec![0x47]);
    }

    // qoi.h:439-444 — C signed-char wrap: r 0 vs prev 255 -> vr = +1 (not -255), so
    // this fits DIFF: 0x40 | 3<<4 | 2<<2 | 2 = 0x7a.
    #[test]
    fn encode_choose_op_diff_signed_char_wrap() {
        let mut e = Encoder::new();
        e.px_prev = Px { r: 255, g: 20, b: 30, a: 255 };
        assert_eq!(e.encode_choose_op(Px { r: 0, g: 20, b: 30, a: 255 }), vec![0x7a]);
    }

    // qoi.h:453-459 — vr=5, vg=10, vb=5: vg_r = vg_b = -5, all in LUMA range, but
    // vr=5 fails DIFF (< 2). Emits 0x80|(10+32)=0xaa, (3<<4)|3 = 0x33.
    #[test]
    fn encode_choose_op_luma() {
        let mut e = Encoder::new();
        e.px_prev = Px { r: 10, g: 20, b: 30, a: 255 };
        assert_eq!(e.encode_choose_op(Px { r: 15, g: 30, b: 35, a: 255 }), vec![0xaa, 0x33]);
    }

    // qoi.h:453-459 — lower LUMA boundary: vg = -31, vg_r = vg_b = 0.
    // 0x80|(-31+32)=0x81, (8<<4)|8 = 0x88.
    #[test]
    fn encode_choose_op_luma_lower_boundary() {
        let mut e = Encoder::new();
        e.px_prev = Px { r: 10, g: 20, b: 30, a: 255 };
        assert_eq!(e.encode_choose_op(Px { r: 235, g: 245, b: 255, a: 255 }), vec![0x81, 0x88]);
    }

    // qoi.h:461-466 — vr=20 forces RGB (DIFF range is {-2..1}; vg_r = 30 also blows
    // LUMA's < 8). Emits 0xfe + literal r/g/b.
    #[test]
    fn encode_choose_op_rgb_fallback() {
        let mut e = Encoder::new();
        e.px_prev = Px { r: 10, g: 20, b: 30, a: 255 };
        assert_eq!(e.encode_choose_op(Px { r: 30, g: 10, b: 20, a: 255 }), vec![0xfe, 30, 10, 20]);
    }

    // qoi.h:468-474 — alpha changed forces full RGBA even though RGB is identical.
    #[test]
    fn encode_choose_op_rgba_on_alpha_change() {
        let mut e = Encoder::new();
        e.px_prev = Px { r: 10, g: 20, b: 30, a: 255 };
        assert_eq!(e.encode_choose_op(Px { r: 10, g: 20, b: 30, a: 7 }), vec![0xff, 10, 20, 30, 7]);
    }

    // qoi.h:415-421 — repeats accumulate without emitting until forced. 3 repeats:
    // run=3, then encode_run_flush emits QOI_OP_RUN | (3-1) = 0xc2.
    #[test]
    fn encode_run_accumulates_then_flushes() {
        let mut e = Encoder::new();
        let mut out = Vec::new();
        assert!(!e.encode_run_repeat(false, &mut out)); // run 1
        assert!(!e.encode_run_repeat(false, &mut out)); // run 2
        assert!(!e.encode_run_repeat(false, &mut out)); // run 3
        assert_eq!(out, Vec::<u8>::new());
        assert_eq!(e.run, 3);
        e.encode_run_flush(&mut out); // qoi.h:425-428
        assert_eq!(out, vec![0xc2]);
        assert_eq!(e.run, 0);
    }

    // qoi.h:417-418 — the cap: 61 repeats accumulate, the 62nd forces a flush
    // emitting QOI_OP_RUN | 61 = 0xfd (covers exactly 62 pixels) and resets; a new
    // run then restarts from 1.
    #[test]
    fn encode_run_cap_at_62() {
        let mut e = Encoder::new();
        let mut out = Vec::new();
        for _ in 0..61 {
            assert!(!e.encode_run_repeat(false, &mut out));
        }
        assert_eq!(e.run, 61);
        assert!(e.encode_run_repeat(false, &mut out)); // run 62 -> cap flush
        assert_eq!(out, vec![0xfd]);
        assert_eq!(e.run, 0);
        assert!(!e.encode_run_repeat(false, &mut out)); // next equal pixel restarts
        assert_eq!(e.run, 1);
    }

    // qoi.h:417 — is_last (px_pos == px_end) flushes a short run immediately:
    // run=1 on the final pixel emits 0xc0 (single-pixel run). A pre-built run of 3
    // on the final pixel becomes run=4 -> 0xc3 (covers 4 pixels).
    #[test]
    fn encode_run_is_last_flushes_short_run() {
        let mut e = Encoder::new();
        let mut out = Vec::new();
        assert!(e.encode_run_repeat(true, &mut out)); // run 0->1 on final pixel
        assert_eq!(out, vec![0xc0]);
        assert_eq!(e.run, 0);
        let mut e = Encoder::new();
        let mut out = Vec::new();
        for _ in 0..3 {
            assert!(!e.encode_run_repeat(false, &mut out)); // run 3
        }
        assert!(e.encode_run_repeat(true, &mut out)); // final pixel: run 3->4
        assert_eq!(out, vec![0xc3]);
        assert_eq!(e.run, 0);
    }

    // qoi.h:425-428 — flush with nothing pending emits nothing.
    #[test]
    fn encode_run_flush_with_empty_run_is_noop() {
        let mut e = Encoder::new();
        let mut out = Vec::new();
        e.encode_run_flush(&mut out);
        assert_eq!(out, Vec::<u8>::new());
    }

    // qoi.h:425-428 + 430-433 — the full 62-run pixel counts: first occurrence via
    // some op, then 62 repeats via 0xfd. Simulates a first pixel + 62 identical
    // follow-ups, showing run-1 covers run pixels (1 + 62 = 63 total, matching the
    // decoder's 1 + (b1 & 0x3f) per RUN chunk, qoi.h:573-575).
    #[test]
    fn encode_run_covers_exact_pixel_count() {
        let mut e = Encoder::new();
        let mut out = Vec::new();
        e.encode_choose_op(Px { r: 5, g: 5, b: 5, a: 255 }); // first occurrence
        e.px_prev = Px { r: 5, g: 5, b: 5, a: 255 };
        for _ in 0..61 {
            assert!(!e.encode_run_repeat(false, &mut out));
        }
        assert!(e.encode_run_repeat(false, &mut out)); // 62nd repeat -> cap flush
        assert_eq!(out, vec![0xfd]);
    }

    // qoi.h:415-421 + 417 — is_last flushes even a single repeat. On the final
    // pixel of a 2x2 all-{5,5,5,255} image the encoder must emit the exact oracle
    // bytes: first pixel LUMA (from px_prev {0,0,0,255}: vr=vg=vb=5, so vg_r=vg_b=0;
    // byte1 0x80|(5+32)=0xa5, byte2 0x88 — qoi.h:453-459), then a run of 3
    // (run=3, qoi.h:418 -> 0xc2). Whole file = 14-byte header + chunks + padding.
    #[test]
    fn encode_full_known_bytes_single_run() {
        let mut raw = vec![0, 0, 0, 2, 0, 0, 0, 2, 4]; // 2x2, 4ch
        for _ in 0..4 {
            raw.extend_from_slice(&[5, 5, 5, 255]);
        }
        let qoi = encode_from_raw(&raw).unwrap();
        let mut expected = header_bytes(2, 2, 4, QOI_SRGB);
        expected.extend_from_slice(&[0xa5, 0x88, 0xc2]);
        expected.extend_from_slice(&QOI_PADDING);
        assert_eq!(qoi, expected);
    }

    // qoi.h:356-483 — encode then decode is identity for a pattern exercising every
    // op: RGB (first px), DIFF, RGBA (alpha change), RUN. 4x1 4ch:
    // {10,20,30,255} -> RGB [fe 0a 14 1e]; {11,21,31,255} -> DIFF 0x7f;
    // {5,6,7,8} -> RGBA [ff 05 06 07 08]; {5,6,7,8} (equal, last) -> RUN 0xc0.
    #[test]
    fn encode_full_roundtrips_via_decode() {
        let mut raw = vec![0, 0, 0, 4, 0, 0, 0, 1, 4]; // 4x1, 4ch
        raw.extend_from_slice(&[10, 20, 30, 255]);
        raw.extend_from_slice(&[11, 21, 31, 255]);
        raw.extend_from_slice(&[5, 6, 7, 8]);
        raw.extend_from_slice(&[5, 6, 7, 8]);
        let qoi = encode_from_raw(&raw).unwrap();
        assert_eq!(decode_to_raw(&qoi).unwrap(), raw);
    }

    // qoi.h:364-372 — invalid descriptors are rejected, not encoded.
    #[test]
    fn encode_full_rejects_invalid_input() {
        assert_eq!(encode_from_raw(&[0, 0, 0, 0, 0, 0, 0, 1, 4]), None); // zero width
        assert_eq!(encode_from_raw(&[0, 0, 0, 1, 0, 0, 0, 0, 4]), None); // zero height
        assert_eq!(encode_from_raw(&[0, 0, 0, 1, 0, 0, 0, 1, 2]), None); // channels=2
        assert_eq!(encode_from_raw(&[0, 0, 0, 1, 0, 0, 0, 1, 5]), None); // channels=5
        assert_eq!(encode_from_raw(&[0, 0, 0, 1, 0, 0, 0, 1, 4]), None); // no pixel data
        assert_eq!(encode_from_raw(&[0, 0, 0, 1, 0, 0, 0, 1, 4, 9, 9]), None); // truncated
    }
}

