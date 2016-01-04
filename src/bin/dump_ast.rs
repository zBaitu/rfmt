#![feature(rustc_private)]
extern crate syntax;

use syntax::ast::CrateConfig;
use syntax::parse::{self, ParseSess};

use std::env;
use std::path::Path;

fn main() {
    let mut args = env::args();
    args.next();
    let src = args.next().unwrap();
    let path = Path::new(&src);
    let cfg = CrateConfig::new();
    let session = ParseSess::new();

    let krate = parse::parse_crate_from_file(&path, cfg, &session);
    println!("{:#?}", krate);

    println!("{:#?}", session.codemap().span_to_snippet(krate.span));
    println!("{:#?}", session.codemap().span_to_string(krate.span));
    println!("{:#?}", session.codemap().span_to_expanded_string(krate.span));
    println!("{:#?}", session.codemap().span_to_filename(krate.span));
}
