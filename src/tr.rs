use std::cell::Cell;
use std::collections::HashMap;

use rst;

use ir::*;

pub fn trans(sess: rst::ParseSess, krate: rst::Crate, cmnts: Vec<rst::Comment>,
             lits: Vec<rst::Literal>) {
    let ts = Trans::new(sess, krate, cmnts, to_lit_map(lits));
    ts.trans();
}

fn to_lit_map(lits: Vec<rst::Literal>) -> HashMap<rst::BytePos, String> {
    lits.into_iter().fold(HashMap::new(), |mut map, e| {
        map.insert(e.pos, e.lit);
        map
    })
}

#[inline]
fn span(s: u32, e: u32) -> rst::Span {
    rst::codemap::mk_sp(rst::BytePos(s), rst::BytePos(e))
}

struct Trans {
    sess: rst::ParseSess,
    krate: rst::Crate,
    cmnts: Vec<rst::Comment>,
    lits: HashMap<rst::BytePos, String>,

    last_loc: Cell<Loc>,
}

impl Trans {
    fn new(sess: rst::ParseSess, krate: rst::Crate, cmnts: Vec<rst::Comment>,
           lits: HashMap<rst::BytePos, String>)
        -> Trans {
        Trans {
            sess: sess,
            krate: krate,
            cmnts: cmnts,
            lits: lits,

            last_loc: Cell::default(),
        }
    }

    fn loc(&self, sp: &rst::Span) -> Loc {
        Loc::new(sp.lo.0, sp.hi.0, self.is_wrapped(sp))
    }

    #[inline]
    fn loc_leaf(&self, sp: &rst::Span) -> Loc {
        let loc = Loc::new(sp.lo.0, sp.hi.0, self.is_wrapped(sp));
        self.last_loc.set(loc);
        loc
    }

    #[inline]
    fn is_wrapped(&self, sp: &rst::Span) -> bool {
        let snippet = self.sess
                          .codemap()
                          .span_to_snippet(span(self.last_loc.get().e, sp.lo.0))
                          .unwrap();

        let mut wrapped = false;
        let mut in_comment = false;
        let mut chars = snippet.chars();
        while let Some(ch) = chars.next() {
            if !in_comment {
                if ch == '/' {
                    in_comment = true;
                } else if ch == '\n' {
                    wrapped = true;
                } else if ch != ',' && !ch.is_whitespace() {
                    wrapped = false;
                    break;
                }
            } else if ch == '/' {
                in_comment = false;
            }
        }

        wrapped
    }

    fn lit(&self, pos: rst::BytePos) -> String {
        self.lits[&pos].clone()
    }

    fn trans(&self) {
        self.trans_crate();
    }

    fn trans_crate(&self) {
        let attrs = self.trans_attrs(&self.krate.attrs);
        println!("{:#?}", attrs);
        //
        // let s = self.sess.codemap().span_to_snippet(rst::codemap::mk_sp(rst::BytePos(7), rst::BytePos(90))).unwrap();
        // println!("{:?}", s);
        // println!("{}", s);
        // println!("{:?}", s.find('\n'));
        // println!("{:?}", s.rfind('\n'));
        //
    }

    fn trans_attrs(&self, attrs: &Vec<rst::Attribute>) -> Vec<AttrOrDoc> {
        attrs.iter().map(|attr| self.trans_attr(attr)).collect()
    }

    fn trans_attr(&self, attr: &rst::Attribute) -> AttrOrDoc {
        if attr.node.is_sugared_doc {
            AttrOrDoc::Doc(self.trans_attr_doc(attr))
        } else {
            AttrOrDoc::Attr(self.trans_attr_attr(attr))
        }
    }

    fn trans_attr_doc(&self, attr: &rst::Attribute) -> Doc {
        if let rst::MetaNameValue(_, ref value) = attr.node.value.node {
            if let rst::LitStr(ref s, _) = value.node {
                return Doc::new(self.loc_leaf(&attr.span), s.to_string());
            }
        }

        unreachable!()
    }

    fn trans_attr_attr(&self, attr: &rst::Attribute) -> Attr {
        let loc = self.loc(&attr.span);
        let is_outer = attr.node.style == rst::AttrStyle::Outer;
        let mi = self.trans_meta_item(&attr.node.value);
        let attr = Attr::new(loc, is_outer, mi);
        self.last_loc.set(loc);
        attr
    }

    fn trans_meta_item(&self, mi: &rst::MetaItem) -> MetaItem {
        match mi.node {
            rst::MetaWord(ref ident) => {
                MetaItem::Single(Chunk::new(self.loc_leaf(&mi.span), ident.to_string()))
            }
            rst::MetaNameValue(ref ident, ref lit) => {
                let s = format!("{} = {}", ident, self.lit(lit.span.lo));
                MetaItem::Single(Chunk::new(self.loc_leaf(&mi.span), s))
            }
            rst::MetaList(ref ident, ref mis) => {
                let loc = self.loc(&mi.span);
                let mi_list = MetaItem::List(loc,
                                             ident.to_string(),
                                             mis.iter()
                                                .map(|mi| self.trans_meta_item(mi))
                                                .collect());
                self.last_loc.set(loc);
                mi_list
            }
        }
    }
}
