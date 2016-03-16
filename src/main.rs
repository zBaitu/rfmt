#![feature(custom_derive)]
#![feature(iter_arith)]
#![feature(rustc_private)]

extern crate getopts;
extern crate rst;
extern crate walkdir;
#[macro_use]
extern crate zbase;

use std::env;

use getopts::Options;

#[macro_use]
mod ts;

mod ft;
mod ir;
mod tr;

mod rfmt;

struct CmdArg {
    ast: bool,
    check: bool,
    debug: bool,
    overwrite: bool,
    path: String,
}

fn cmd_arg() -> CmdArg {
    let mut opts = Options::new();
    opts.optflag("a", "ast", "print rust original ast debug info");
    opts.optflag("c", "check", "only output exceed lines and trailing white space lines");
    opts.optflag("d", "debug", "print ir debug info");
    opts.optflag("o", "overwrite", "overwrite the source file");

    let mut matches = opts.parse(env::args().skip(1)).unwrap();
    let ast = matches.opt_present("a");
    let check = matches.opt_present("c");
    let debug = matches.opt_present("d");
    let overwrite = matches.opt_present("o");
    let path = matches.free.pop().unwrap();

    CmdArg {
        ast: ast,
        check: check,
        debug: debug,
        overwrite: overwrite,
        path: path,
    }
}

fn main() {
    if env::args().len() == 1 {
        rfmt::fmt_from_stdin();
    } else {
        let cmd_arg = cmd_arg();
        if cmd_arg.ast {
            rfmt::dump_ast(&cmd_arg.path);
        } else {
            rfmt::fmt(&cmd_arg.path, cmd_arg.check, cmd_arg.debug, cmd_arg.overwrite);
        }
    }
}
