use std::cell::Cell;
use std::collections::HashMap;

use rst;

use ir::*;

pub fn trans(sess: rst::ParseSess, krate: rst::Crate, cmnts: Vec<rst::Comment>,
             lits: Vec<rst::Literal>) {
                 let ts = Trans::new(sess, krate, cmnts, to_lit_map(lits));
                 let krate = ts.trans();
                 println!("{:#?}", krate);
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
    cmnt_idx: u32,
    lits: HashMap<rst::BytePos, String>,

    last_loc: Cell<Loc>,
}

impl Trans {
    fn new(sess: rst::ParseSess, krate: rst::Crate, cmnts: Vec<rst::Comment>,
           lits: HashMap<rst::BytePos, String>)
        -> Trans {
            let crate_start = krate.span.lo.0;
            Trans {
                sess: sess,
                krate: krate,
                cmnts: cmnts,
                cmnt_idx: crate_start,
                lits: lits,

                last_loc: Cell::new(Loc {
                    e: crate_start,
                    ..Default::default()
                }),
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
        for ch in snippet.chars() {
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

    fn trans(&self) -> Crate {
        self.trans_crate()
    }

    fn trans_crate(&self) -> Crate {
        let loc = self.loc(&self.krate.span);
        let attrs = self.trans_attrs(&self.krate.attrs);
        let module = self.trans_mod(&self.krate.module);
        Crate::new(loc, attrs, module)
    }

    fn trans_attrs(&self, attrs: &Vec<rst::Attribute>) -> Vec<AttrKind> {
        attrs.iter().map(|attr| self.trans_attr(attr)).collect()
    }

    fn trans_attr(&self, attr: &rst::Attribute) -> AttrKind {
        if attr.node.is_sugared_doc {
            AttrKind::Doc(self.trans_attr_doc(attr))
        } else {
            AttrKind::Attr(self.trans_attr_attr(attr))
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
        self.last_loc.set(loc);
        Attr::new(loc, is_outer, mi)
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

    fn trans_mod(&self, module: &rst::Mod) -> Mod {
        let loc = self.loc(&module.inner);
        let items = module.items.iter().map(|item| self.trans_item(item)).collect();
        self.last_loc.set(loc);
        Mod::new(loc, items)
    }

    fn trans_item(&self, item: &rst::Item) -> Item {
        let loc = self.loc(&item.span);
        let attrs = self.trans_attrs(&item.attrs);
        let item = match item.node {
            rst::ItemExternCrate(ref name) => {
                ItemKind::ExternCrate(self.trans_extren_crate(item, name))
            }
            _ => unreachable!(),
        };
        self.last_loc.set(loc);

        Item::new(loc, attrs, item)
    }

    fn trans_extren_crate(&self, item: &rst::Item, name: &Option<rst::Name>) -> ExternCrate {
        let mut krate = item.ident.name.as_str().to_string();
        if let Some(rename) = *name {
            krate = format!("{} as {}", krate, rename.as_str().to_string());
        }
        ExternCrate::new(krate)
    }
}
