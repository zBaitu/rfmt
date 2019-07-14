use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use syntax::parse::{self, lexer::comments, ParseSess};
use syntax::source_map::FilePathMapping;
use syntax_pos::FileName;
use walkdir::WalkDir;

use crate::Opt;
use crate::tr;

const SEP: &str = "--------------------------------------------------------------------------------";


pub fn dump_ast(path: &PathBuf) {
    let src = fs::read_to_string(path).unwrap();
    let mut input = &src.as_bytes().to_vec()[..];

    syntax::with_default_globals(|| {
        let sess = ParseSess::new(FilePathMapping::empty());
        let krate = match parse::parse_crate_from_source_str(FileName::from(path.clone()), src, &sess) {
            Ok(krate) => krate,
            Err(mut e) => {
                e.emit();
                return;
            }
        };
        d!(krate);

        p!("\n{}\n", SEP);

        let cmnts = comments::gather_comments(&sess, FileName::from(path.clone()), &mut input);
        for cmnt in cmnts {
            p!("{}: {:#?} {:#?}", cmnt.pos.0, cmnt.style, cmnt.lines);
        }
    });
}

pub fn fmt_from_stdin() {
    let mut src = String::new();
    io::stdin().read_to_string(&mut src).unwrap();
    fmt_str(src, &PathBuf::from("stdin"), false, false, false);
}

pub fn fmt(opt: Opt) {
    let path = opt.input.unwrap();
    if path.is_dir() {
        fmt_dir(&path, opt.check, opt.debug, opt.overwrite);
    } else {
        fmt_file(&path, opt.check, opt.debug, opt.overwrite);
    }
}

fn fmt_dir(path: &Path, check: bool, debug: bool, overwrite: bool) {
    for entry in WalkDir::new(path) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let path = entry.into_path();
            let ext = path.extension();
            if let Some(ext) = ext {
                if ext == "rs" {
                    fmt_file(&path, check, debug, overwrite);
                }
            }
        }
    }
}

fn fmt_file(path: &PathBuf, check: bool, debug: bool, overwrite: bool) {
    let src = fs::read_to_string(path).unwrap();
    fmt_str(src, path, check, debug, overwrite);
}

fn fmt_str(src: String, path: &PathBuf, check: bool, debug: bool, overwrite: bool) {
    let result = syntax::with_default_globals(|| {
        let mut input = &src.as_bytes().to_vec()[..];
        let sess = ParseSess::new(FilePathMapping::empty());
        let krate = parse::parse_crate_from_source_str(FileName::from(path.to_path_buf()), src, &sess).unwrap();
        let cmnts = comments::gather_comments(&sess, FileName::from(path.to_path_buf()), &mut input);
        tr::trans(sess, krate, cmnts)
    });

    if debug {
        d!(result.krate);
        d!(result.leading_cmnts);
        d!(result.trailing_cmnts);
        p!("{}\n", SEP);
    }

    /*
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
    */
}

