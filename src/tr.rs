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

#[inline]
fn is_pub(vis: rst::Visibility) -> bool {
    match vis {
        rst::Visibility::Public => true,
        _ => false,
    }
}

#[inline]
fn name_to_string(name: &rst::Name) -> String {
    name.as_str().to_string()
}

#[inline]
fn ident_to_string(ident: &rst::Ident) -> String {
    name_to_string(&ident.name)
}

struct Trans {
    sess: rst::ParseSess,
    krate: rst::Crate,
    cmnts: Vec<rst::Comment>,
    cmnt_idx: u32,
    lits: HashMap<rst::BytePos, String>,

    last_loc: Cell<Loc>,
}

macro_rules! trans_list {
    ($self_: ident, $list: ident, $trans_single: ident) => ({
        $list.iter().map(|ref e| $self_.$trans_single(e)).collect()
    })
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
                end: crate_start,
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
        self.set_loc(&loc);
        loc
    }

    #[inline]
    fn set_loc(&self, loc: &Loc) {
        self.last_loc.set(*loc)
    }

    #[inline]
    fn is_wrapped(&self, sp: &rst::Span) -> bool {
        let snippet = self.sess
                          .codemap()
                          .span_to_snippet(span(self.last_loc.get().end, sp.lo.0))
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

    fn is_mod_decl(&self, sp: &rst::Span) -> bool {
        sp.lo.0 > self.krate.span.hi.0
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

    #[inline]
    fn trans_attrs(&self, attrs: &Vec<rst::Attribute>) -> Vec<AttrKind> {
        trans_list!(self, attrs, trans_attr)
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
        let meta_item = self.trans_meta_item(&attr.node.value);
        self.set_loc(&loc);
        Attr::new(loc, is_outer, meta_item)
    }

    fn trans_meta_item(&self, meta_item: &rst::MetaItem) -> MetaItem {
        match meta_item.node {
            rst::MetaWord(ref ident) => {
                MetaItem::Single(Chunk::new(self.loc_leaf(&meta_item.span), ident.to_string()))
            }
            rst::MetaNameValue(ref ident, ref lit) => {
                let s = format!("{} = {}", ident, self.lit(lit.span.lo));
                MetaItem::Single(Chunk::new(self.loc_leaf(&meta_item.span), s))
            }
            rst::MetaList(ref ident, ref meta_items) => {
                let loc = self.loc(&meta_item.span);
                let meta_item = MetaItem::List(loc,
                                             ident.to_string(),
                                             trans_list!(self, meta_items, trans_meta_item));
                self.set_loc(&loc);
                meta_item
            }
        }
    }

    fn trans_mod(&self, module: &rst::Mod) -> Mod {
        let loc = self.loc(&module.inner);
        let items = self.trans_items(&module.items);
        self.set_loc(&loc);
        Mod::new(loc, items)
    }

    #[inline]
    fn trans_items(&self, items: &Vec<rst::P<rst::Item>>) -> Vec<Item> {
        trans_list!(self, items, trans_item)
    }

    fn trans_item(&self, item: &rst::Item) -> Item {
        let loc = self.loc(&item.span);
        let attrs = self.trans_attrs(&item.attrs);

        let item = match item.node {
            rst::ItemExternCrate(ref name) => {
                ItemKind::ExternCrate(self.trans_extren_crate(item, name))
            }
            rst::ItemUse(ref view_path) => ItemKind::Use(self.trans_use(item, view_path)),
            rst::ItemMod(ref module) => {
                if self.is_mod_decl(&module.inner) {
                    ItemKind::ModDecl(self.trans_mod_decl(&item))
                } else {
                    ItemKind::Mod(self.trans_mod(module))
                }
            }
            rst::ItemTy(ref ty, ref generics) => {
                ItemKind::Type(self.trans_type_alias(ty, generics))
            }
            _ => unreachable!(),
        };

        self.set_loc(&loc);
        Item::new(loc, attrs, item)
    }

    fn trans_extren_crate(&self, item: &rst::Item, name: &Option<rst::Name>) -> ExternCrate {
        let mut krate = ident_to_string(&item.ident);
        if let Some(ref rename) = *name {
            krate = format!("{} as {}", krate, name_to_string(rename));
        }
        ExternCrate::new(krate)
    }

    fn trans_use(&self, item: &rst::Item, view_path: &rst::ViewPath) -> Use {
        match view_path.node {
            rst::ViewPathSimple(ident, ref path) => {
                self.loc_leaf(&path.span);
                let mut fullpath = self.use_view_path_to_string(path);
                if path.segments.last().unwrap().identifier.name != ident.name {
                    fullpath = format!("{} as {}", fullpath, ident_to_string(&ident));
                }
                Use::new(is_pub(item.vis), fullpath, Vec::new())
            }
            rst::ViewPathGlob(ref path) => {
                self.loc_leaf(&path.span);
                let fullpath = format!("{}::*", self.use_view_path_to_string(path));
                Use::new(is_pub(item.vis), fullpath, Vec::new())
            }
            rst::ViewPathList(ref path, ref list) => {
                let loc = self.loc(&path.span);
                let fullpath = self.use_view_path_to_string(path);
                let use_item = Use::new(is_pub(item.vis), fullpath, self.trans_use_paths(list));
                self.set_loc(&loc);
                use_item
            }
        }
    }

    fn use_view_path_to_string(&self, path: &rst::Path) -> String {
        path.segments.iter().fold(String::new(), |mut s, e| {
            if !s.is_empty() {
                s.push_str("::");
            }
            s.push_str(&ident_to_string(&e.identifier));
            s
        })
    }

    fn trans_use_paths(&self, list: &Vec<rst::PathListItem>) -> Vec<Chunk> {
        list.iter().fold(Vec::new(), |mut vec, ref e| {
            vec.push(self.trans_use_path(e));
            vec
        })
    }

    fn trans_use_path(&self, use_path: &rst::PathListItem) -> Chunk {
        let loc = self.loc_leaf(&use_path.span);
        let (mut s, rename) = match use_path.node {
            rst::PathListIdent{ ref name, ref rename, .. } => (ident_to_string(name), rename),
            rst::PathListMod{ ref rename, .. } => ("self".to_string(), rename),
        };
        if let Some(ref ident) = *rename {
            s = format!("{} as {}", s, ident_to_string(ident));
        };

        Chunk::new(loc, s)
    }

    fn trans_mod_decl(&self, item: &rst::Item) -> ModDecl {
        ModDecl::new(is_pub(item.vis), ident_to_string(&item.ident))
    }

    fn trans_type_alias(&self, ty: &rst::Ty, generics: &rst::Generics) -> Type {
        Type::new(self.trans_generics(generics))
    }

    fn trans_generics(&self, generics: &rst::Generics) -> Generics {
        Generics::new(self.trans_lifetime_defs(&generics.lifetimes),
                      self.trans_type_params(&generics.ty_params))
    }

    #[inline]
    fn trans_lifetime_defs(&self, lifetime_defs: &Vec<rst::LifetimeDef>) -> Vec<LifetimeDef> {
        trans_list!(self, lifetime_defs, trans_lifetime_def)
    }

    fn trans_lifetime_def(&self, lifetime_def: &rst::LifetimeDef) -> LifetimeDef {
        LifetimeDef::new(self.trans_lifetime(&lifetime_def.lifetime),
                         self.trans_lifetime_bounds(&lifetime_def.bounds))
    }

    fn trans_lifetime(&self, lifetime: &rst::Lifetime) -> Lifetime {
        Lifetime::new(self.loc_leaf(&lifetime.span), name_to_string(&lifetime.name))
    }

    #[inline]
    fn trans_lifetime_bounds(&self, bounds: &Vec<rst::Lifetime>) -> Vec<Lifetime> {
        trans_list!(self, bounds, trans_lifetime)
    }

    #[inline]
    fn trans_type_params(&self, type_params: &[rst::TyParam]) -> Vec<TypeParam> {
        trans_list!(self, type_params, trans_type_param)
    }

    fn trans_type_param(&self, type_param: &rst::TyParam) -> TypeParam {
        let loc = self.loc(&type_param.span);
        let name = ident_to_string(&type_param.ident);
        let bounds = self.trans_type_param_bounds(&type_param.bounds);
        let default = None;
        self.set_loc(&loc);
        TypeParam::new(loc, name, bounds, default)
        //
        // let default = match type_param.default {
        // Some(ref ty) => Some(self.trans_type(ty)),
        // None => None,
        // }
        //
    }

    #[inline]
    fn trans_type_param_bounds(&self, bounds: &[rst::TyParamBound]) -> Vec<TypeParamBound> {
        trans_list!(self, bounds, trans_type_param_bound)
    }

    fn trans_type_param_bound(&self, bound: &rst::TyParamBound) -> TypeParamBound {
        match bound {
            &rst::RegionTyParamBound(ref lifetime) => {
                TypeParamBound::Lifetime(self.trans_lifetime(lifetime))
            }
            _ => unreachable!(),
        }
    }
}
