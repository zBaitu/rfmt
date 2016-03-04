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

fn opt() -> (bool, String) {
    let mut opts = Options::new();
    opts.optflag("r", "recursive", ""); 

    let args: Vec<String> = env::args().collect();
    let mut matches = opts.parse(&args[1..]).unwrap();
    let recursive = matches.opt_present("r");
    let file = matches.free.pop().unwrap();
    (recursive, file)
}

fn main() {
    let (recursive, file) = opt();

    let path = Path::new(&file);
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

    rfmt::fmt(path, recursive);
}
