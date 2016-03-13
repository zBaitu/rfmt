use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::Read;
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

pub fn fmt(path: String, check: bool, debug: bool) {
    let path = Path::new(&path);
    if path.is_dir() {
        fmt_dir(path, check, debug);
    } else {
        fmt_file(path, check, debug);
    }
}

#[allow(deprecated)]
fn fmt_dir(path: &Path, check: bool, debug: bool) {
    let walk_dir = fs::walk_dir(path).unwrap();
    for dir in walk_dir {
        let dir = dir.unwrap();
        let path = dir.path();
        let file_type = dir.file_type().unwrap();

        if file_type.is_dir() {
            fmt_dir(&path, check, debug);
        } else if file_type.is_file() {
            let ext = path.extension();
            if let Some(ext) = ext {
                if ext == "rs" {
                    fmt_file(&path, check, debug);
                }
            }
        }
    }
}

fn fmt_file(path: &Path, check: bool, debug: bool) {
    let mut file = File::open(path).unwrap();
    let mut src = String::new();
    file.read_to_string(&mut src).unwrap();
    let mut input = &src.as_bytes().to_vec()[..];

    let file_name = path.file_name() .unwrap() .to_str() .unwrap() .to_string();
    let cfg = CrateConfig::new();
    let sess = ParseSess::new();
    let krate = parse::parse_crate_from_source_str(file_name.clone(), src, cfg, &sess);
    let (cmnts, _) = comments::gather_comments_and_literals(&sess.span_diagnostic, file_name, &mut input);

    let result = tr::trans(sess, krate, cmnts);
    if debug {
        p!("{:#?}", result.krate);
        p!("{:#?}", result.leading_cmnts);
        p!("{:#?}", result.trailing_cmnts);
    }
    let result = ft::fmt(result.krate, result.leading_cmnts, result.trailing_cmnts);
    if check {
        p!("-----------------------------------------------------------------------------------------\
            ----------");
        p!("{:?}", result.exceed_lines);
        p!("{:?}", result.trailing_ws_lines);
    } else {
        p!(result.s);
    }
}
