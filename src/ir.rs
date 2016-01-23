use std::fmt::{self, Debug, Display};

#[derive(Clone, Copy, Default)]
pub struct Loc {
    pub s: u32,
    pub e: u32,
    pub w: bool,
}

impl Loc {
    pub fn new(s: u32, e: u32, w: bool) -> Loc {
        Loc {
            s: s,
            e: e,
            w: w,
        }
    }
}

impl Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.w {
            write!(f, "Loc({}, {}, wrapped)", self.s, self.e)
        } else {
            write!(f, "Loc({}, {})", self.s, self.e)
        }
    }
}

impl Debug for Loc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

#[derive(Debug)]
pub struct Chunk {
    loc: Loc,
    s: String,
}

impl Chunk {
    pub fn new(loc: Loc, s: String) -> Chunk {
        Chunk {
            loc: loc,
            s: s,
        }
    }
}

#[inline]
fn attr_head(is_outer: bool) -> &'static str {
    static HASH: &'static str = "#";
    static HASH_BANG: &'static str = "#!";

    if is_outer {
        HASH
    } else {
        HASH_BANG
    }
}

#[derive(Debug)]
pub enum MetaItem {
    Single(Chunk),
    List(Loc, String, Vec<MetaItem>),
}

#[derive(Debug)]
pub struct Attr {
    pub loc: Loc,
    pub head: &'static str,
    pub mi: MetaItem,
}

impl Attr {
    pub fn new(loc: Loc, is_outer: bool, mi: MetaItem) -> Attr {
        Attr {
            loc: loc,
            head: attr_head(is_outer),
            mi: mi,
        }
    }
}

pub type Doc = Chunk;

#[derive(Debug)]
pub enum AttrOrDoc {
    Attr(Attr),
    Doc(Doc),
}
