use ir::*;
use ts::*;

pub fn fmt_crate(krate: &Crate, cmnts: &Vec<Comment>) -> String {
    Formatter::new(cmnts).fmt_crate(krate)
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
        p!("---------- doc ----------");
        p!("{:#?}", doc);
        self.try_fmt_comments(&doc.loc);
    }

    fn fmt_attr_group(&mut self, attr_group: &Vec<&Attr>) {
        p!("---------- attr group ----------");
        p!("{:#?}", attr_group);
    }
}
