#![feature(rustc_private)]
extern crate syntax;

use syntax::ast::CrateConfig;
use syntax::parse::{self, ParseSess};
use syntax::parse::lexer::comments;

use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

fn main() {
    let mut args = env::args();
    args.next();
    let src = args.next().unwrap();
    let path = Path::new(&src);
    let cfg = CrateConfig::new();

    let session = ParseSess::new();
    let krate = parse::parse_crate_from_file(&path, cfg, &session);

    let mut file = File::open(path).unwrap();
    let mut rs = String::new();
    file.read_to_string(&mut rs).unwrap();
    let mut input = &rs.as_bytes().to_vec()[..];

    let (cmts, lits) = comments::gather_comments_and_literals(&session.span_diagnostic,
                                                              path.file_name()
                                                                  .unwrap()
                                                                  .to_str()
                                                                  .unwrap()
                                                                  .to_string(),
                                                              &mut input);
    for cmt in cmts {
        println!("{:?}", cmt.lines);
        println!("{:?}", cmt.pos);
    }
    for lit in lits {
        println!("{}", lit.lit);
        println!("{:?}", lit.pos);
    }
}
