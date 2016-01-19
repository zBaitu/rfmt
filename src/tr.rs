use rst::ast;

use ir::*;

pub fn trans(krate: &ast::Crate, cmnts: &Vec<ast::Comment>, lits: &Vec<ast::Literal>) {
    let ts = Trans::new(krate, cmnts, lits);
    ts.trans();
}

struct Trans<'a> {
    krate: &'a ast::Crate,
    cmnts: &'a Vec<ast::Comment>,
    lits: &'a Vec<ast::Literal>,
}

impl<'a> Trans<'a> {
    fn new(krate: &'a ast::Crate, cmnts: &'a Vec<ast::Comment>, lits: &'a Vec<ast::Literal>)
        -> Trans<'a> {
        Trans {
            krate: krate,
            cmnts: cmnts,
            lits: lits,
        }
    }

    fn trans(&self) {
        self.trans_crate();
    }

    fn trans_crate(&self) {
        let attrs = self.trans_attrs(&self.krate.attrs);
        println!("{:#?}", attrs);
    }

    fn trans_attrs<'tr>(&self, attrs: &'tr Vec<ast::Attribute>) -> Vec<AttrOrDoc<'tr>> {
        attrs.iter().map(|attr| self.trans_attr(attr)).collect()
    }

    fn trans_attr<'tr>(&self, attr: &'tr ast::Attribute) -> AttrOrDoc<'tr> {
        if attr.node.is_sugared_doc {
            AttrOrDoc::IsDoc(self.trans_attr_doc(attr))
        } else {
            AttrOrDoc::IsAttr(self.trans_attr_attr(attr))
        }
    }

    fn trans_attr_doc<'tr>(&self, attr: &'tr ast::Attribute) -> Doc<'tr> {
        if let ast::MetaNameValue(_, ref value) = attr.node.value.node {
            if let ast::LitStr(ref s, _) = value.node {
                return Doc {
                    doc: &s,
                    sp: Span(attr.span.lo.0, attr.span.hi.0),
                };
            }
        }

        unreachable!()
    }

    fn trans_attr_attr(&self, attr: &ast::Attribute) -> Attr {
        unreachable!()
    }
}
