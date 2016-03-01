#![feature(rustc_private)]
#![feature(custom_derive)]
#![feature(iter_arith)]

#[macro_use]
extern crate zbase;
extern crate rst;

use std::env;
use std::path::Path;

#[macro_use]
mod ts;

mod ft;
mod ir;
mod tr;

mod rfmt;

fn main() {
    let mut args = env::args();
    args.next();
    let file = args.next().unwrap();
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

    let result = rfmt::fmt(path);
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
