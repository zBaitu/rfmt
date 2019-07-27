use std::fs;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use syntax::parse::{self, lexer::comments, ParseSess};
use syntax::source_map::FilePathMapping;
use syntax_pos::FileName;
use walkdir::WalkDir;

use crate::ft;
use crate::Opt;
use crate::tr::{self, TrResult};

macro_rules! p {
    () => ({println!()});
    ($arg:expr) => ({println!("{}", $arg)});
    ($fmt:expr, $($arg:tt)*) => ({println!($fmt, $($arg)*)});
    ($($arg:tt)+) => ({println!("{}", $($arg)+)});
}

macro_rules! d {
    ($arg:expr) => ({println!("{:#?}", $arg)});
}

const SEP: &str = r#"
------------------------------------------------------------------------------------------------------------------------
"#;

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

pub fn fmt_from_stdin(opt: Opt) {
    let mut src = String::new();
    io::stdin().read_to_string(&mut src).unwrap();
    fmt_str(src, &PathBuf::from("stdin"), &opt);
}

pub fn debug(path: &PathBuf) {
    let src = fs::read_to_string(path).unwrap();
    let result = trans(src, path);

    d!(result.krate);
    p!(SEP);
    d!(result.leading_cmnts);
    d!(result.trailing_cmnts);
}

pub fn fmt(opt: Opt) {
    let path = opt.input.as_ref().unwrap();
    if path.is_dir() {
        fmt_dir(&path, &opt);
    } else {
        fmt_file(&path, &opt);
    }
}

fn fmt_dir(path: &Path, opt: &Opt) {
    for entry in WalkDir::new(path) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let path = entry.into_path();
            let ext = path.extension();
            if let Some(ext) = ext {
                if ext == "rs" {
                    fmt_file(&path, opt);
                }
            }
        }
    }
}

fn fmt_file(path: &PathBuf, opt: &Opt) {
    let src = fs::read_to_string(path).unwrap();
    fmt_str(src, path, opt);
}

fn fmt_str(src: String, path: &PathBuf, opt: &Opt) {
    let tr_result = trans(src, path);
    let ft_result = ft::fmt(tr_result.krate, tr_result.leading_cmnts, tr_result.trailing_cmnts);
    if opt.overwrite {
        let mut file = File::create(path).unwrap();
        file.write_all(ft_result.s.as_bytes()).unwrap();
    } else if opt.check {
        if !ft_result.exceed_lines.is_empty() || !ft_result.trailing_ws_lines.is_empty() {
            p!("{:?}", path);
            if !ft_result.exceed_lines.is_empty() {
                p!("exceed_lines: {:?}", ft_result.exceed_lines);
            }
            if !ft_result.trailing_ws_lines.is_empty() {
                p!("trailing_ws_lines: {:?}", ft_result.trailing_ws_lines);
            }
            p!(SEP);
        }
    } else {
        p!(ft_result.s);
    }
}

fn trans(src: String, path: &PathBuf) -> TrResult {
    syntax::with_default_globals(|| {
        let mut input = &src.as_bytes().to_vec()[..];
        let sess = ParseSess::new(FilePathMapping::empty());
        let krate = parse::parse_crate_from_source_str(FileName::from(path.to_path_buf()), src, &sess).unwrap();
        let cmnts = comments::gather_comments(&sess, FileName::from(path.to_path_buf()), &mut input);
        tr::trans(sess, krate, cmnts)
    })
}

