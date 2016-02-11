use std::collections::BTreeMap;
use std::fmt::{self, Display};

use ir::*;
use ts::*;

pub fn fmt_crate(krate: &Crate, cmnts: &Vec<Comment>) -> String {
    Formatter::new(cmnts).fmt_crate(krate)
}

//
// macro_rules! fmt_list {
// ($list: ident, $bop: expr, $op: expr, $sop: expr, $eop: expr) => ({
// $bop;
// $list.iter().fold(true, |first, e| {
// if !first {
// $sop;
// }
// $op(e);
// false
// });
// $eop;
// })
// }
//

impl Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.s, f)
    }
}

impl Display for Attr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "#"));
        if self.is_inner {
            try!(write!(f, "!"));
        }

        write!(f, "[{}]", self.item)
    }
}

impl Display for MetaItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MetaItem::Single(ref chunk) => Display::fmt(chunk, f),
            MetaItem::List(_, ref name, ref items) => {
                try!(write!(f, "{}(", name));

                let mut first = true;
                for item in items {
                    if !first {
                        try!(write!(f, ", "));
                    }
                    try!(Display::fmt(item, f));
                    first = false;
                }

                write!(f, ")")
            }
        }
    }
}

struct Formatter<'a> {
    cmnts: &'a Vec<Comment>,
    cmnt_idx: usize,

    ts: Typesetter,
}

impl<'a> Formatter<'a> {
    fn new(cmnts: &'a Vec<Comment>) -> Formatter<'a> {
        Formatter {
            cmnts: cmnts,
            cmnt_idx: 0,

            ts: Typesetter::new(),
        }
    }

    #[inline]
    fn is_after_comment(&self, loc: &Loc) -> bool {
        self.cmnt_idx < self.cmnts.len() && self.cmnts[self.cmnt_idx].pos < loc.start
    }

    #[inline]
    fn try_fmt_comments(&mut self, loc: &Loc) {
        if self.is_after_comment(loc) {
            self.fmt_comments(loc);
        }
    }

    fn fmt_comments(&mut self, loc: &Loc) {
        while self.cmnt_idx < self.cmnts.len() && self.cmnts[self.cmnt_idx].pos < loc.start {
            self.fmt_comment(&self.cmnts[self.cmnt_idx]);
            self.cmnt_idx += 1;
        }
    }

    fn fmt_comment(&self, cmnt: &Comment) {
        p!("---------- comment ----------");
        p!("{:#?}", cmnt);
    }

    fn fmt_crate(mut self, krate: &Crate) -> String {
        self.try_fmt_comments(&krate.loc);
        self.fmt_attrs(&krate.attrs);

        self.ts.string()
    }

    fn fmt_attrs(&mut self, attrs: &Vec<AttrKind>) {
        let mut attr_group: Vec<&Attr> = Vec::new();

        for attr in attrs {
            match *attr {
                AttrKind::Doc(ref doc) => {
                    self.fmt_attr_group(&attr_group);
                    attr_group.clear();

                    self.fmt_doc(doc);
                }
                AttrKind::Attr(ref attr) => {
                    if self.is_after_comment(&attr.loc) {
                        self.fmt_attr_group(&attr_group);
                        attr_group.clear();

                        self.fmt_comments(&attr.loc);
                    }
                    attr_group.push(attr);
                }
            }
        }

        self.fmt_attr_group(&attr_group);
    }

    fn fmt_doc(&mut self, doc: &Doc) {
        self.try_fmt_comments(&doc.loc);
        p!("---------- doc ----------");
        p!("{:#?}", doc);
    }

    fn fmt_attr_group(&mut self, attr_group: &Vec<&Attr>) {
        let mut attrs: BTreeMap<String, &Attr> = attr_group.into_iter()
                                                    .map(|e| (e.to_string(), *e))
                                                    .collect();
        p!("---------- attr group ----------");
        for (_, attr) in attrs {
            p!("{}", attr);
        }
    }
}
