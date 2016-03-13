#![feature(rustc_private)]
#![feature(custom_derive)]
#![feature(iter_arith)]

extern crate getopts;
#[macro_use]
extern crate zbase;
extern crate rst;

use std::env;
use std::path::Path;

use getopts::Options;

#[macro_use]
mod ts;

mod ft;
mod ir;
mod tr;

mod rfmt;

struct Opt {
    check: bool,
    debug: bool,
    file: String,
}

fn opt() -> Opt {
    let mut opts = Options::new();
    opts.optflag("c", "check", ""); 
    opts.optflag("d", "debug", ""); 

    let args: Vec<String> = env::args().collect();
    let mut matches = opts.parse(&args[1..]).unwrap();
    let check = matches.opt_present("c");
    let debug = matches.opt_present("d");
    let file = matches.free.pop().unwrap();

    Opt {
        check: check,
        debug: debug,
        file: file,
    }
}

fn main() {
    let opt = opt();

    let path = Path::new(&opt.file);
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

    rfmt::fmt(path, opt.check, opt.debug);
}
