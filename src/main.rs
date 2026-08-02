use std::env;
use std::process::ExitCode;

const QOI_MAGIC: u32 = 0x716f6966; // 'q' << 24 | 'o' << 16 | 'i' << 8 | 'f'
const QOI_HEADER_SIZE: usize = 14;
const QOI_PIXELS_MAX: u32 = 400000000;
const QOI_SRGB: u8 = 0;
const QOI_LINEAR: u8 = 1;
const QOI_PADDING: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 1];

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
        || channels < 3
        || channels > 4
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
            let _ = (in_path, out_path);
            eprintln!("encode: not implemented yet");
            ExitCode::FAILURE
        }
        "decode" => {
            let _ = (in_path, out_path);
            eprintln!("decode: not implemented yet");
            ExitCode::FAILURE
        }
        cmd => {
            eprintln!("unknown command: {cmd}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{color_hash, parse_header, read_32, validate_desc, write_32, write_end_marker, write_header, Header, QOI_HEADER_SIZE, QOI_LINEAR, QOI_MAGIC, QOI_PADDING, QOI_PIXELS_MAX, QOI_SRGB};

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
}

