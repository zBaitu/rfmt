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

#[derive(Debug)]
pub struct Crate {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub module: Mod,
}

impl Crate {
    pub fn new(loc: Loc, attrs: Vec<AttrKind>, module: Mod) -> Crate {
        Crate {
            loc: loc,
            attrs: attrs,
            module: module,
        }
    }
}

#[derive(Debug)]
pub enum AttrKind {
    Attr(Attr),
    Doc(Doc),
}

#[derive(Debug)]
pub struct Attr {
    pub loc: Loc,
    pub head: &'static str,
    pub mi: MetaItem,
}

#[derive(Debug)]
pub enum MetaItem {
    Single(Chunk),
    List(Loc, String, Vec<MetaItem>),
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

pub type Doc = Chunk;

#[derive(Debug)]
pub struct Mod {
    pub loc: Loc,
    pub items: Vec<Item>,
}

impl Mod {
    pub fn new(loc: Loc, items: Vec<Item>) -> Mod {
        Mod {
            loc: loc,
            items: items,
        }
    }
}

#[derive(Debug)]
pub struct Item {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub item: ItemKind,
}

impl Item {
    pub fn new(loc: Loc, attrs: Vec<AttrKind>, item: ItemKind) -> Item {
        Item {
            loc: loc,
            attrs: attrs,
            item: item,
        }
    }
}

#[derive(Debug)]
pub enum ItemKind {
    ExternCrate(ExternCrate),
}

#[derive(Debug)]
pub struct ExternCrate {
    pub head: &'static str,
    pub krate: String,
}

impl ExternCrate {
    pub fn new(krate: String) -> ExternCrate {
        static HEAD: &'static str = "extern crate ";
        ExternCrate {
            head: HEAD,
            krate: krate,
        }
    }
}
