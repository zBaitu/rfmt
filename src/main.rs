#![feature(rustc_private)]
#![feature(custom_derive)]

#[macro_use]
extern crate zbase;
extern crate rst;

use rst::ast::CrateConfig;
use rst::parse::{ParseSess, self};
use rst::parse::lexer::comments;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

mod ir;
mod tr;
mod ft;
mod ts;

fn main() {
    let mut args = env::args();
    args.next();
    let file = args.next().unwrap();
    let path = Path::new(&file);

    let dir = path.parent();
    if let Some(dir) = dir {
        if let Some(dir) = dir.to_str() {
            if !dir.is_empty() {
                env::set_current_dir(dir).unwrap();
            }
        }
    }

    let file_name = path.file_name().unwrap();
    let mut path = env::current_dir().unwrap();
    path.push(file_name);

    let mut file = File::open(&path).unwrap();
    let mut src = String::new();
    file.read_to_string(&mut src).unwrap();
    let mut input = &src.as_bytes().to_vec()[..];

    let cfg = CrateConfig::new();
    let session = ParseSess::new();
    let krate = parse::parse_crate_from_source_str(path.file_name()
                                                   .unwrap()
                                                   .to_str()
                                                   .unwrap()
                                                   .to_string(),
                                                   src,
                                                   cfg,
                                                   &session);
    let (cmnts, lits) = comments::gather_comments_and_literals(&session.span_diagnostic,
                                                               path.file_name()
                                                               .unwrap()
                                                               .to_str()
                                                               .unwrap()
                                                               .to_string(),
                                                               &mut input);

    let (krate, cmnts) = tr::trans(session, krate, lits, cmnts);
    p!("{:#?}", krate);
    for cmnt in &cmnts {
        p!("{:?}", cmnt.pos);
        p!("{:?}", cmnt.lines);
    }
    p!("");
    p!("");

    ft::fmt_crate(&krate, &cmnts);
}
