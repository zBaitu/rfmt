#![deny(warnings)]
#![feature(custom_derive)]
#![feature(iter_arith)]
#![feature(question_mark)]
#![feature(rustc_private)]

extern crate getopts;
extern crate rst;
extern crate walkdir;

use getopts::Options;

use std::env;

#[macro_use]
mod ts;

mod ft;
mod ir;
mod rfmt;
mod tr;

struct CmdArg {
    opts: Options,
    ast: bool,
    check: bool,
    debug: bool,
    overwrite: bool,
    path: String,
    v: bool,
    h: bool,
}

fn cmd_arg() -> CmdArg {
    let mut opts = Options::new();
    opts.optflag("a", "ast", "print the rust original syntax ast debug info");
    opts.optflag("c", "check", "check exceed lines and trailing white space lines");
    opts.optflag("d", "debug", "print the rfmt ir debug info");
    opts.optflag("o", "overwrite", "overwrite the source file");
    opts.optflag("v", "version", "show version");
    opts.optflag("h", "help", "show help");

    let mut matches = opts.parse(env::args().skip(1)).unwrap();
    let ast = matches.opt_present("a");
    let check = matches.opt_present("c");
    let debug = matches.opt_present("d");
    let overwrite = matches.opt_present("o");
    let v = matches.opt_present("v");
    let h = matches.opt_present("h");
    let path = match matches.free.pop() {
        Some(path) => path,
        None => ".".to_string(),
    };

    CmdArg {
        opts: opts,
        ast: ast,
        check: check,
        debug: debug,
        overwrite: overwrite,
        path: path,
        v: v,
        h: h,
    }
}

fn print_version() {
    println!("rfmt 0.1.0");
}

fn print_help(opts: &Options) {
    let brief = r#"
Usage: rfmt [options] [path]
    If `path` is a dir, rfmt will do action for all files in this dir recursively.
    If `path` is not specified, use the current dir by default.
    If neither `options` nor `path` is specified, rfmt will format source code from stdin."#;

    println!("{}", opts.usage(brief));
}

fn main() {
    if env::args().len() == 1 {
        rfmt::fmt_from_stdin();
        return;
    }

    let cmd_arg = cmd_arg();
    if cmd_arg.v {
        print_version();
    } else if cmd_arg.h {
        print_help(&cmd_arg.opts);
    } else if cmd_arg.ast {
        rfmt::dump_ast(&cmd_arg.path);
    } else {
        rfmt::fmt(&cmd_arg.path, cmd_arg.check, cmd_arg.debug, cmd_arg.overwrite);
    }
}
