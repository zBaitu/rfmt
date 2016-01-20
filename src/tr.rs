use std::collections::HashMap;

use rst;

use ir::*;

pub fn trans(krate: rst::Crate, cmnts: Vec<rst::Comment>, lits: Vec<rst::Literal>) {
    let ts = Trans::new(krate, cmnts, to_lit_map(lits));
    ts.trans();
}

fn to_lit_map(lits: Vec<rst::Literal>) -> HashMap<rst::BytePos, String> {
    lits.into_iter().fold(HashMap::new(), |mut map, e| {
        map.insert(e.pos, e.lit);
        map
    })
}

#[inline]
fn span(sp: &rst::Span) -> Span {
    Span::new(sp.lo.0, sp.hi.0)
}

struct Trans {
    krate: rst::Crate,
    cmnts: Vec<rst::Comment>,
    lits: HashMap<rst::BytePos, String>,
}

impl Trans {
    fn new(krate: rst::Crate, cmnts: Vec<rst::Comment>, lits: HashMap<rst::BytePos, String>) -> Trans {
        Trans {
            krate: krate,
            cmnts: cmnts,
            lits: lits,
        }
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
                return Doc::new(s.to_string(), span(&attr.span));
            }
        }

        unreachable!()
    }

    fn trans_attr_attr(&self, attr: &rst::Attribute) -> Attr {
        let is_outer = attr.node.style == rst::AttrStyle::Outer;
        let mi = self.trans_meta_item(&attr.node.value);
        Attr::new(is_outer, mi, span(&attr.span))
    }

    fn trans_meta_item(&self, mi: &rst::MetaItem) -> MetaItem {
        match mi.node {
            rst::MetaWord(ref ident) => {
                MetaItem::Single(Chunk::new(ident.to_string(), span(&mi.span)))
            }
            rst::MetaNameValue(ref ident, ref lit) => {
                let s = format!("{} = {}", ident, self.lit(lit.span.lo));
                MetaItem::Single(Chunk::new(s, span(&mi.span)))
            }
            rst::MetaList(ref ident, ref mis) => {
                MetaItem::List(ident.to_string(),
                               mis.iter().map(|mi| self.trans_meta_item(mi)).collect(),
                               span(&mi.span))
            }
        }
    }
}
