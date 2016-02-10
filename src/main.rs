#![feature(rustc_private)]
#![feature(custom_derive)]

extern crate zbase;
extern crate rst;

use rst::ast::CrateConfig;
use rst::parse::{self, ParseSess};
use rst::parse::lexer::comments;

use std::collections::HashSet;
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
    let src = args.next().unwrap();
    let path = Path::new(&src);
    let cfg = CrateConfig::new();
    let session = ParseSess::new();

    let mut file = File::open(path).unwrap();
    let mut rs = String::new();
    file.read_to_string(&mut rs).unwrap();
    let mut input = &rs.as_bytes().to_vec()[..];

    let krate = parse::parse_crate_from_source_str(path.file_name()
                                                   .unwrap()
                                                   .to_str()
                                                   .unwrap()
                                                   .to_string(),
                                                   rs,
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
    println!("{:#?}", krate);
    println!("\n----------");
    for cmnt in &cmnts {
        println!("{:?}", cmnt.pos);
        println!("{:?}", cmnt.lines);
    }

    let cmnt_pos_set: HashSet<u32> = cmnts.iter().map(|cmnt| cmnt.pos).collect();
    println!("{:?}", cmnt_pos_set);

    ft::fmt_crate(&krate, &cmnts, &cmnt_pos_set);
}
