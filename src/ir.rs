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
pub struct Chunk {
    s: String,
    sp: Span,
}

impl Chunk {
    pub fn new(s: String, sp: Span) -> Chunk {
        Chunk {
            s: s,
            sp: sp,
        }
    }
}

#[inline]
fn attr_head(is_outer: bool) -> &'static str {
    static HASH: &'static str = "#";
    static HASH_BANG: &'static str = "#!";

    if is_outer {
        HASH
    }
    else {
        HASH_BANG
    }
}

#[derive(Debug)]
pub enum MetaItem {
    Single(Chunk),
    List(String, Vec<MetaItem>, Span),
}

#[derive(Debug)]
pub struct Attr {
    pub head: &'static str,
    pub mi: MetaItem,
    pub sp: Span,
}

impl Attr {
    pub fn new(is_outer: bool, mi: MetaItem, sp: Span) -> Attr {
        Attr {
            head: attr_head(is_outer),
            mi: mi,
            sp: sp,
        }
    }
}

pub type Doc = Chunk;

#[derive(Debug)]
pub enum AttrOrDoc {
    Attr(Attr),
    Doc(Doc),
}
