#![feature(rustc_private)]
extern crate syntax;

use syntax::ast::CrateConfig;
use syntax::parse::{self, ParseSess};

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

    let ann = pprust::NoAnn;
    let mut vec = Vec::new();
    {
        let out: &mut Write = &mut vec;
        pprust::print_crate(session.codemap(),
                            &session.span_diagnostic,
                            &krate,
                            path.file_name().unwrap().to_str().unwrap().to_string(),
                            &mut input,
                            Box::new(out),
                            &ann,
                            false)
            .unwrap();
    }
    println!("{}", String::from_utf8(vec).unwrap());
}
