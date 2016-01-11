#![feature(rustc_private)]
extern crate syntax;

use syntax::ast::CrateConfig;
use syntax::parse::{self, ParseSess};

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn main() {
    let mut args = env::args();
    args.next();
    let src = args.next().unwrap();
    let path = Path::new(&src);
    let cfg = CrateConfig::new();
    let session = ParseSess::new();

    let mut file = File::open(path).unwrap();
    let mut input = String::new();
    file.read_to_string(&mut input).unwrap();

    let krate = parse::parse_crate_from_source_str(path.file_name()
                                                       .unwrap()
                                                       .to_str()
                                                       .unwrap()
                                                       .to_string(),
                                                   input,
                                                   cfg,
                                                   &session);
    println!("{:#?}", krate);

    println!("{:#?}", session.codemap().span_to_snippet(krate.span));
    println!("{:#?}", session.codemap().span_to_string(krate.span));
    println!("{:#?}", session.codemap().span_to_expanded_string(krate.span));
    println!("{:#?}", session.codemap().span_to_filename(krate.span));
}
