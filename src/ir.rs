use std::fmt::{self, Debug, Display};

pub struct Span {
    s: u32,
    e: u32,
}

impl Span {
    pub fn new(s: u32, e: u32) -> Span {
        Span {
            s: s,
            e: e,
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Span({}, {})", self.s, self.e)
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

#[derive(Debug)]
pub struct Attr {
    pub sp: Span,
}

#[derive(Debug)]
pub struct Doc {
    pub doc: String,
    pub sp: Span,
}

#[derive(Debug)]
pub enum AttrOrDoc {
    IsAttr(Attr),
    IsDoc(Doc),
}
