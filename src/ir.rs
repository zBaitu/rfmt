use std::fmt::{self, Display, Debug};

pub struct Span(pub u32, pub u32);

impl Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Span({}, {})", self.0, self.1)
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
pub struct Doc<'a> {
    pub doc: &'a str,
    pub sp: Span,
}

#[derive(Debug)]
pub enum AttrOrDoc<'a> {
    IsAttr(Attr),
    IsDoc(Doc<'a>),
}
