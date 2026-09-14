//! Compare two raw 8-bit buffers with a per-sample tolerance.
//!
//! Exits non-zero when the buffers differ (length mismatch, IO error, or any
//! sample outside the tolerance), so it can gate CI and shell pipelines.

use std::process::ExitCode;

const USAGE: &str = "\
pictura-diff - compare two raw 8-bit buffers

USAGE:
    pictura-diff [--tolerance N] <a> <b>

ARGS:
    <a>  left / expected raw 8-bit file
    <b>  right / actual raw 8-bit file

OPTIONS:
    -t, --tolerance N   per-sample absolute delta treated as equal (default 0)
    -h, --help          print this help
";

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<bool, String> {
    let mut tolerance: u8 = 0;
    let mut files: Vec<String> = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return Ok(true);
            }
            "-t" | "--tolerance" => {
                let value = args.next().ok_or("--tolerance requires a value")?;
                tolerance = value
                    .parse()
                    .map_err(|_| format!("invalid tolerance: {value}"))?;
            }
            other if other.starts_with('-') => return Err(format!("unknown option: {other}")),
            other => files.push(other.to_string()),
        }
    }

    if files.len() != 2 {
        return Err(format!(
            "expected exactly two files, got {}\n\n{USAGE}",
            files.len()
        ));
    }

    let a = std::fs::read(&files[0]).map_err(|e| format!("{}: {e}", files[0]))?;
    let b = std::fs::read(&files[1]).map_err(|e| format!("{}: {e}", files[1]))?;
    let diff = pictura_testkit::compare(&a, &b, tolerance)?;
    println!(
        "samples={} differing={} max_delta={} mean_delta={:.6}",
        diff.samples,
        diff.differing,
        diff.max_delta,
        diff.mean_delta()
    );
    Ok(diff.is_empty())
}
