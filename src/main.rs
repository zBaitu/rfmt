use std::path::PathBuf;

use structopt::StructOpt;

//mod ast;
//mod ft;
//mod ir;
mod rfmt;
//mod tr;
//mod ts;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(long, short)]
    /// Print the rust original syntax ast debug info
    ast: bool,

    #[structopt(long, short)]
    /// Check exceed lines and trailing white space lines
    check: bool,

    #[structopt(long, short)]
    /// Print the rfmt ir debug info
    debug: bool,

    #[structopt(long, short)]
    /// Print the rfmt ir simple format
    print: bool,

    #[structopt(long, short)]
    /// Overwrite the source file
    overwrite: bool,

    /// Input file or dir.
    /// If `input` is a dir, rfmt will do action for all files in this dir recursively.
    /// If neither `options` nor `input` is specified, rfmt will format source code from stdin.
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();
    rfmt::dump_ast(&opt.input.unwrap());
    /*
    if opt.input.is_none() {
        rfmt::fmt_from_stdin(opt);
    } else if opt.ast {
        rfmt::dump_ast(&opt.input.unwrap());
    } else if opt.debug {
        rfmt::debug(&opt.input.unwrap());
    } else if opt.print {
        rfmt::print(&opt.input.unwrap());
    } else {
        rfmt::fmt(opt);
    }
     */
}
