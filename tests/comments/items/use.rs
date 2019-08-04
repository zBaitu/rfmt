
// aa
//
// bb
use *; // cc
use ::*; // dd
use ::f;
use a::b::{c, d, e::f, g::h::i}; // ee
use a::b::{self, c, d::e}; // ff
use a::b::{self as ab, c as abc}; // gg
use a::b::*; // hh
use a::b::{ self as ab, c, d::{*, e::f}, };
use p::q::r as x;

//
// jj
// 
// kk
use crate::aa as x; // ll
// mm
// nn

