use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

use rst::ast::CrateConfig;
use rst::parse::lexer::comments;
use rst::parse::{self, ParseSess};

use ft;
use tr;

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
    let walk_dir = fs::walk_dir(path).unwrap();
    for dir in walk_dir {
        let dir = dir.unwrap();
        let path = dir.path();
        let file_type = dir.file_type().unwrap();

        if file_type.is_dir() {
            fmt_dir(&path, check, debug, overwrite);
        } else if file_type.is_file() {
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
    let file_name = path.file_name().unwrap().to_str().unwrap();
    fmt_str(src, file_name, check, debug, overwrite);
}

fn fmt_str(src: String, file_name: &str, check: bool, debug: bool, overwrite: bool) {
    let cfg = CrateConfig::new();
    let sess = ParseSess::new();
    let mut input = &src.as_bytes().to_vec()[..];
    let krate = parse::parse_crate_from_source_str(file_name.to_string(), src, cfg, &sess);
    let (cmnts, _) = comments::gather_comments_and_literals(&sess.span_diagnostic,
            file_name.to_string(), &mut input);

    let result = tr::trans(sess, krate, cmnts);
    if debug {
        p!("{:#?}", result.krate);
        p!("{:#?}", result.leading_cmnts);
        p!("{:#?}", result.trailing_cmnts);
        p!(SEP);
    }

    let result = ft::fmt(result.krate, result.leading_cmnts, result.trailing_cmnts);
    if overwrite {
        let mut file = File::create(file_name).unwrap();
        file.write_all(result.s.as_bytes()).unwrap();
    } else if !check {
        p!(result.s);
        p!(SEP);
    }

    if !result.exceed_lines.is_empty() || !result.trailing_ws_lines.is_empty() {
        pe!("{}", file_name);
        if !result.exceed_lines.is_empty() {
            pe!("exceed_lines: {:?}", result.exceed_lines);
        }
        if !result.trailing_ws_lines.is_empty() {
            pe!("trailing_ws_lines: {:?}", result.trailing_ws_lines);
        }
        p!(SEP);
    }
}
