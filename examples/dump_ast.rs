#![feature(rustc_private)]
extern crate rst;

use rst::ast::CrateConfig;
use rst::parse::{self, ParseSess};
use rst::parse::lexer::comments;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn main() {
    let mut args = env::args();
    args.next();
    let src = args.next().unwrap();
    let path = Path::new(&src);

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
    println!("{:#?}", krate);

    println!("\n----------");
    println!("{:?}", session.codemap().files);
    println!("\n----------");
    println!("{:#?}", session.codemap().span_to_snippet(krate.span));
    println!("{:#?}", session.codemap().span_to_string(krate.span));
    println!("{:#?}", session.codemap().span_to_expanded_string(krate.span));
    println!("{:#?}", session.codemap().span_to_filename(krate.span));

    let (cmts, lits) = comments::gather_comments_and_literals(&session.span_diagnostic,
                                                              path.file_name()
                                                                  .unwrap()
                                                                  .to_str()
                                                                  .unwrap()
                                                                  .to_string(),
                                                              &mut input);
    println!("\n----------");
    for cmt in cmts {
        println!("{:?}", cmt.style);
        println!("{:?}", cmt.pos);
        println!("{:?}", cmt.lines);
    }
    println!("----------");
    for lit in lits {
        println!("{:?}", lit.pos);
        println!("{}", lit.lit);
    }
}
