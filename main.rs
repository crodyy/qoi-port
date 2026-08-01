use std::env;
use std::fs;
use std::process::exit;

// STUB — deliberately wrong. Purpose: prove verify.sh detects failure correctly
// before any real port logic exists. Do not "fix" this to pass — that's the loop's job.

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        eprintln!("usage: {} <encode|decode> <input> <output>", args[0]);
        exit(2);
    }

    let mode = &args[1];
    let output_path = &args[3];

    match mode.as_str() {
        "encode" => {
            // deliberately wrong: writes garbage bytes instead of real QOI encoding
            fs::write(output_path, b"NOT_A_REAL_QOI_ENCODE_YET")
                .expect("stub: failed to write dummy encode output");
        }
        "decode" => {
            // deliberately wrong: writes garbage bytes instead of real decoded pixels
            fs::write(output_path, b"NOT_REAL_DECODED_PIXELS_YET")
                .expect("stub: failed to write dummy decode output");
        }
        _ => {
            eprintln!("unknown mode: {}", mode);
            exit(2);
        }
    }

    // exits 0 on purpose — a crashing stub would also make verify.sh report FAIL,
    // but a clean-exit-with-wrong-output stub is the more realistic case to test against
}
