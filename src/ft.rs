use rst;

use std::collections::HashSet;

use ir::*;
use ts::*;

pub fn fmt_crate(krate: &Crate, cmnts: &Vec<Comment>, cmnt_pos_set: &HashSet<u32>) -> String {
    Formatter::new(cmnts, cmnt_pos_set).fmt_crate(krate)
}

struct Formatter<'a> {
    cmnts: &'a Vec<Comment>,
    cmnt_pos_set: &'a HashSet<u32>,
    ts: Typesetter,
}

impl<'a> Formatter<'a> {
    pub fn new(cmnts: &'a Vec<Comment>, cmnt_pos_set: &'a HashSet<u32>) -> Formatter<'a> {
        Formatter {
            cmnts: cmnts,
            cmnt_pos_set: cmnt_pos_set,
            ts: Typesetter::new(),
        }
    }

    pub fn fmt_crate(&self, krate: &Crate) -> String {
        "".to_string()
    }
}
