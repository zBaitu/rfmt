use rst;

use ir::*;
use ts::*;

pub fn fmt_crate(krate: &Crate, cmnts: &Vec<rst::Comment>) -> String {
    Formatter::new(cmnts).fmt_crate(krate)
}

struct Formatter<'a> {
    cmnts: &'a Vec<rst::Comment>,
    ts: Typesetter,
}

impl<'a> Formatter<'a> {
    pub fn new(cmnts: &Vec<rst::Comment>) -> Formatter {
        Formatter {
            cmnts: cmnts,
            ts: Typesetter::new(),
        }
    }

    pub fn fmt_crate(&self, krate: &Crate) -> String {
        "".to_string()
    }
}
