use rst;
use rst::ast::CrateConfig;
use rst::parse::{self, ParseSess};
use rst::parse::lexer::comments;

use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::path::{self, PathBuf};

use tr::Translator;
use ft;

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
    let (cmnts, lits) = comments::gather_comments_and_literals(&sess.span_diagnostic,
                                                               path.file_name()
                                                                   .unwrap()
                                                                   .to_str()
                                                                   .unwrap()
                                                                   .to_string(),
                                                               &mut input);

    for cmnt in &cmnts {
        p!("{:?}", cmnt.pos);
        p!("{:?}", cmnt.lines);
    }
    RustFmt::new(sess, lits, cmnts, recursive).fmt(krate);
}

pub struct Result {
    pub s: String,
    pub exceed_lines: BTreeSet<u32>,
    pub trailing_ws_lines: BTreeSet<u32>,
}

struct RustFmt {
    trans: Translator,
    recursive: bool,
    files: BTreeSet<String>,
    mod_paths: Vec<String>,
}

#[inline]
fn name_to_string(name: &rst::Name) -> String {
    name.as_str().to_string()
}

#[inline]
fn ident_to_string(ident: &rst::Ident) -> String {
    name_to_string(&ident.name)
}

impl RustFmt {
    pub fn new(sess: rst::ParseSess, lits: Vec<rst::Literal>, cmnts: Vec<rst::Comment>,
               recursive: bool)
        -> RustFmt {
        let files = if recursive {
            sess.codemap().files.borrow().iter().map(|ref file| file.name.clone()).collect()
        } else {
            BTreeSet::new()
        };

        RustFmt {
            trans: Translator::new(sess, lits, cmnts),
            recursive: recursive,
            files: files,
            mod_paths: Vec::new(),
        }
    }

    pub fn fmt(&mut self, krate: rst::Crate) {
        self.fmt_crate(&krate);
        if self.recursive {
            p!("{:?}", self.files);
            self.fmt_sub_mods(&krate.module);
        }
    }

    fn fmt_crate(&mut self, krate: &rst::Crate) {
        let result = self.trans.trans_crate(&krate);
        p!("=====================================================================================\
            ===============");
        p!("{:#?}", result.krate);
        p!("{:#?}", result.leading_cmnts);
        p!("{:#?}", result.trailing_cmnts);
        p!("-------------------------------------------------------------------------------------\
            --------------");
        let result = ft::fmt_crate(result.krate, result.leading_cmnts, result.trailing_cmnts);
        p!(result.s);
        p!("-------------------------------------------------------------------------------------\
            --------------");
        p!("{:?}", result.exceed_lines);
        p!("{:?}", result.trailing_ws_lines);
        p!();
        p!();
    }

    fn fmt_sub_mods(&mut self, module: &rst::Mod) {
        for item in &module.items {
            if let rst::ItemMod(ref module) = item.node {
                let ident = ident_to_string(&item.ident);
                self.mod_paths.push(ident.clone());
                self.fmt_mod(ident, module);
                self.mod_paths.pop();
            }
        }
    }

    fn is_sub_mod(&self) -> bool {
        self.files.contains(&self.mod_full_file_name())
    }

    fn mod_full_file_name(&self) -> String {
        let file = self.mod_full_name() + ".rs";
        if self.files.contains(&file) {
            file
        } else {
            self.mod_full_name() + &path::MAIN_SEPARATOR.to_string() + "mod.rs"
        }
    }

    fn mod_full_name(&self) -> String {
        join_path_list!(&self.mod_paths)
    }

    fn fmt_mod(&mut self, name: String, module: &rst::Mod) {
        if self.is_sub_mod() {
            let mod_full_file_name = self.mod_full_file_name();
            self.trans.set_mod_full_file_name(mod_full_file_name);
            let result = self.trans.trans_mod(name, &module);
            p!("=================================================================================\
                ===================");
            p!("{:#?}", result.module);
            p!("{:#?}", result.leading_cmnts);
            p!("{:#?}", result.trailing_cmnts);
            p!("---------------------------------------------------------------------------------\
                ------------------");
            let result = ft::fmt_mod(result.module, result.leading_cmnts, result.trailing_cmnts);
            p!(result.s);
            p!("---------------------------------------------------------------------------------\
                ------------------");
            p!("{:?}", result.exceed_lines);
            p!("{:?}", result.trailing_ws_lines);
            p!();
            p!();
        }

        self.fmt_sub_mods(module);
    }
}
