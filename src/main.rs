//! `txt2-rs`: txt2 to HTML, and its problems. See `usage`.

use std::process::ExitCode;

fn usage() {
    eprintln!("usage: txt2-rs <file.txt2>            a whole HTML page, to stdout");
    eprintln!("       txt2-rs --check <file.txt2>    what's wrong (an unclosed block ...)");
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (check, path) = match args.as_slice() {
        [p] if !p.starts_with('-') => (false, p),
        [c, p] if c == "--check" => (true, p),
        _ => {
            usage();
            return ExitCode::FAILURE;
        }
    };
    let text = match std::fs::read(path) {
        Ok(b) => String::from_utf8_lossy(&b).into_owned(),
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let doc = txt2_rs::parse(&text);
    if check {
        for e in &doc.errors {
            println!("{path}:{}:{}: {}", e.line, e.col, e.message);
        }
        return if doc.errors.is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE };
    }
    print!("{}", txt2_rs::html::page(&doc, path));
    ExitCode::SUCCESS
}
