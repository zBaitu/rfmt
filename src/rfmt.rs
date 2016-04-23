use std::collections::BTreeSet;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use rst::ast::CrateConfig;
use rst::parse::lexer::comments;
use rst::parse::{self, ParseSess};
use walkdir::WalkDir;

use ft;
use tr;

macro_rules! p {
    () => ({print!("\n")});
    ($arg:expr) => ({print!("{}\n", $arg)});
    ($fmt:expr, $($arg:tt)*) => ({print!(concat!($fmt, "\n"), $($arg)*)});
    ($($arg:tt)+) => ({print!("{}\n", $($arg)+)});
}

const SEP: &'static str = "----------------------------------------";

pub fn dump_ast(path: &str) {
    let mut file = File::open(path).unwrap();
    let mut src = String::new();
    file.read_to_string(&mut src).unwrap();
    let mut input = &src.as_bytes().to_vec()[..];

    let cfg = CrateConfig::new();
    let session = ParseSess::new();
    let krate = parse::parse_crate_from_source_str(path.to_string(), src, cfg, &session);
    p!("{:#?}", krate);
    p!(SEP);

    let (cmnts, lits) = comments::gather_comments_and_literals(&session.span_diagnostic,
            path.to_string(), &mut input);
    for cmnt in cmnts {
        p!("{}: {:#?} {:#?}", cmnt.pos.0, cmnt.style, cmnt.lines);
    }
    p!(SEP);
    for lit in lits {
        p!("{}: {}", lit.pos.0, lit.lit);
    }
}

pub struct Result {
    pub s: String,
    pub exceed_lines: BTreeSet<u32>,
    pub trailing_ws_lines: BTreeSet<u32>,
}

pub fn fmt_from_stdin() {
    let mut src = String::new();
    io::stdin().read_to_string(&mut src).unwrap();
    fmt_str(src, "stdin", false, false, false);
}

pub fn fmt(path: &str, check: bool, debug: bool, overwrite: bool) {
    let path = Path::new(path);
    if path.is_dir() {
        fmt_dir(path, check, debug, overwrite);
    } else {
        fmt_file(path, check, debug, overwrite);
    }
}

#[allow(deprecated)]
fn fmt_dir(path: &Path, check: bool, debug: bool, overwrite: bool) {
    for entry in WalkDir::new(path) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let path = entry.path();
            let ext = path.extension();
            if let Some(ext) = ext {
                if ext == "rs" {
                    fmt_file(&path, check, debug, overwrite);
                }
            }
        }
    }
}

fn fmt_file(path: &Path, check: bool, debug: bool, overwrite: bool) {
    let mut file = File::open(path).unwrap();
    let mut src = String::new();
    file.read_to_string(&mut src).unwrap();
    fmt_str(src, path.to_str().unwrap(), check, debug, overwrite);
}

fn fmt_str(src: String, path: &str, check: bool, debug: bool, overwrite: bool) {
    let cfg = CrateConfig::new();
    let sess = ParseSess::new();
    let mut input = &src.as_bytes().to_vec()[..];
    let krate = parse::parse_crate_from_source_str(path.to_string(), src, cfg, &sess).unwrap();
    let (cmnts, _) = comments::gather_comments_and_literals(&sess.span_diagnostic,
            path.to_string(), &mut input);

    let result = tr::trans(sess, krate, cmnts);
    if debug {
        p!("{:#?}", result.krate);
        p!("{:#?}", result.leading_cmnts);
        p!("{:#?}", result.trailing_cmnts);
        p!(SEP);
    }

    let result = ft::fmt(result.krate, result.leading_cmnts, result.trailing_cmnts);
    if overwrite {
        let mut file = File::create(path).unwrap();
        file.write_all(result.s.as_bytes()).unwrap();
    } else if check {
        if !result.exceed_lines.is_empty() || !result.trailing_ws_lines.is_empty() {
            p!("{}", path);
            if !result.exceed_lines.is_empty() {
                p!("exceed_lines: {:?}", result.exceed_lines);
            }
            if !result.trailing_ws_lines.is_empty() {
                p!("trailing_ws_lines: {:?}", result.trailing_ws_lines);
            }
            p!(SEP);
        }
    } else {
        p!(result.s);
    }
}
