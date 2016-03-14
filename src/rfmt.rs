use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

use rst::ast::CrateConfig;
use rst::parse::{self, ParseSess};
use rst::parse::lexer::comments;

use ft;
use tr;

pub struct Result {
    pub s: String,
    pub exceed_lines: BTreeSet<u32>,
    pub trailing_ws_lines: BTreeSet<u32>,
}

pub fn fmt_from_stdin() {
    let mut src = String::new();
    io::stdin().read_to_string(&mut src).unwrap();
    fmt_str(src, "stdin".to_string(), false, false, false);
}

pub fn fmt(path: String, check: bool, debug: bool, overwrite: bool) {
    let path = Path::new(&path);
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
    let file_name = path.file_name() .unwrap() .to_str() .unwrap() .to_string();
    fmt_str(src, file_name, check, debug, overwrite);
}

fn fmt_str(src: String, file_name: String, check: bool, debug: bool, overwrite: bool) {
    let cfg = CrateConfig::new();
    let sess = ParseSess::new();
    let mut input = &src.as_bytes().to_vec()[..];
    let krate = parse::parse_crate_from_source_str(file_name.clone(), src, cfg, &sess);
    let (cmnts, _) = comments::gather_comments_and_literals(&sess.span_diagnostic, file_name.clone(), &mut input);

    let result = tr::trans(sess, krate, cmnts);
    if debug {
        p!("{:#?}", result.krate);
        p!("{:#?}", result.leading_cmnts);
        p!("{:#?}", result.trailing_cmnts);
    }
    let result = ft::fmt(result.krate, result.leading_cmnts, result.trailing_cmnts);
    if overwrite {
        let mut file = File::create(&file_name).unwrap();
        file.write_all(result.s.as_bytes()).unwrap();
    } else if !check {
        p!(result.s);
    }
    if !result.exceed_lines.is_empty() || !result.trailing_ws_lines.is_empty() {
        pe!("{}", file_name);
        if !result.exceed_lines.is_empty() {
            pe!("exceed_lines: {:?}", result.exceed_lines);
        }
        if !result.trailing_ws_lines.is_empty() {
            pe!("trailing_ws_lines: {:?}", result.trailing_ws_lines);
        }
        pe!();
    }
}
