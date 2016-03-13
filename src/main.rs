#![feature(fs_walk)] 
#![feature(rustc_private)]
#![feature(custom_derive)]
#![feature(iter_arith)]

extern crate getopts;
#[macro_use]
extern crate zbase;
extern crate rst;

use std::env;

use getopts::Options;

#[macro_use]
mod ts;

mod ft;
mod ir;
mod tr;

mod rfmt;

struct CmdArg {
    check: bool,
    debug: bool,
    overwrite: bool,
    path: String,
}

fn cmd_arg() -> CmdArg {
    let mut opts = Options::new();
    opts.optflag("c", "check", ""); 
    opts.optflag("d", "debug", ""); 
    opts.optflag("o", "overwrite", ""); 

    let args: Vec<String> = env::args().collect();
    let mut matches = opts.parse(&args[1..]).unwrap();
    let check = matches.opt_present("c");
    let debug = matches.opt_present("d");
    let overwrite = matches.opt_present("o");
    let path = matches.free.pop().unwrap();

    CmdArg {
        check: check,
        debug: debug,
        overwrite: overwrite,
        path: path,
    }
}

fn main() {
    let cmd_arg = cmd_arg();
    rfmt::fmt(cmd_arg.path, cmd_arg.check, cmd_arg.debug, cmd_arg.overwrite);
}
