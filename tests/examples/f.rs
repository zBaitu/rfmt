#![feature(custom_derive)]
#![deny(warnings)]
#![feature(question_mark)]
#![feature(iter_arith)]
#![feature(rustc_private)]

extern crate rst;
extern crate getopts;
extern crate walkdir;

use std::env;
use getopts::Options;

#[macro_use]
mod ts;

mod ir;
mod ft;
mod tr;
mod rfmt;
