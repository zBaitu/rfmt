use rst::ast::CrateConfig;
use rst::parse::{self, ParseSess};
use rst::parse::lexer::comments;

use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use tr;
use ft;

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

    let result = tr::trans(session, krate, lits, cmnts);
    p!("{:#?}", result.krate);
    p!("{:#?}", result.leading_cmnts);
    p!("{:#?}", result.trailing_cmnts);

    let result = ft::fmt_crate(result.krate, result.leading_cmnts, result.trailing_cmnts);

    p!("{}", recursive);
    p!();
    p!();
    p!("=========================================================================================\
        ===========");
    p!(result.s);
    p!("=========================================================================================\
        ===========");
    p!("{:?}", result.exceed_lines);
    p!("{:?}", result.trailing_ws_lines);
}
