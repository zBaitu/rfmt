#![feature(rustc_private)]
extern crate syntax;

mod rfmt;

fn main() {
    rfmt::fmt();
}
