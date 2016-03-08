use rst::ast::CrateConfig;
use rst::parse::{self, ParseSess};
use rst::parse::lexer::comments;

use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use ft;
use tr;

pub struct Result {
    pub s: String,
    pub exceed_lines: BTreeSet<u32>,
    pub trailing_ws_lines: BTreeSet<u32>,
}

pub fn fmt(path: PathBuf, recursive: bool) {
    let mut file = File::open(&path).unwrap();
    let mut src = String::new();
    file.read_to_string(&mut src).unwrap();
    let mut input = &src.as_bytes().to_vec()[..];

    let cfg = CrateConfig::new();
    let sess = ParseSess::new();
    let krate = parse::parse_crate_from_source_str(path.file_name()
                                                       .unwrap()
                                                       .to_str()
                                                       .unwrap()
                                                       .to_string(),
                                                   src,
                                                   cfg,
                                                   &sess);
    let (cmnts, _) = comments::gather_comments_and_literals(&sess.span_diagnostic,
                                                            path.file_name()
                                                                .unwrap()
                                                                .to_str()
                                                                .unwrap()
                                                                .to_string(),
                                                            &mut input);

    p!("{:#?}", sess.codemap().files.borrow());
    let result = tr::trans(sess, krate, cmnts);
    p!("=========================================================================================\
        ===========");
    p!("{:#?}", result.krate);
    p!("{:#?}", result.leading_cmnts);
    p!("{:#?}", result.trailing_cmnts);
    p!("-----------------------------------------------------------------------------------------\
        ----------");
    let result = ft::fmt(result.krate, result.leading_cmnts, result.trailing_cmnts);
    p!(result.s);
    p!("-----------------------------------------------------------------------------------------\
        ----------");
    p!("{:?}", result.exceed_lines);
    p!("{:?}", result.trailing_ws_lines);
    p!();
    p!();

    // RustFmt::new(sess, lits, cmnts, recursive).fmt(krate);
}

//
// struct RustFmt {
// recursive: bool,
// files: BTreeSet<String>,
// mod_paths: Vec<String>,
// }
//
// impl RustFmt {
// pub fn new(sess: rst::ParseSess, lits: Vec<rst::Literal>, cmnts: Vec<rst::Comment>,
// recursive: bool)
// -> RustFmt {
// let files = if recursive {
// sess.codemap().files.borrow().iter().map(|ref file| file.name.clone()).collect()
// } else {
// BTreeSet::new()
// };
//
// RustFmt {
// recursive: recursive,
// files: files,
// mod_paths: Vec::new(),
// }
// }
//
// pub fn fmt(&mut self, krate: rst::Crate) {
// self.fmt_crate(&krate);
// }
//
// fn fmt_crate(&mut self, krate: &rst::Crate) {
// let result = self.trans.trans(&krate);
// p!("=====================================================================================\
// ===============");
// p!("{:#?}", result.krate);
// p!("{:#?}", result.leading_cmnts);
// p!("{:#?}", result.trailing_cmnts);
// p!("-------------------------------------------------------------------------------------\
// --------------");
// let result = ft::fmt_crate(result.krate, result.leading_cmnts, result.trailing_cmnts);
// p!(result.s);
// p!("-------------------------------------------------------------------------------------\
// --------------");
// p!("{:?}", result.exceed_lines);
// p!("{:?}", result.trailing_ws_lines);
// p!();
// p!();
// }
// }
//
