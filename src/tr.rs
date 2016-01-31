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
    lits.into_iter().map(|e| (e.pos, e.lit)).collect()
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
fn is_mut(mutbl: rst::Mutability) -> bool {
    match mutbl {
        rst::Mutability::MutMutable => true,
        _ => false,
    }
}

#[inline]
fn is_unsafe(safety: rst::Unsafety) -> bool {
    match safety {
        rst::Unsafety::Unsafe => true,
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

#[inline]
fn abi_to_string(abi: rst::abi::Abi) -> String {
    format!("{:?}", abi)
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

    #[inline]
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
    fn file_name(&self) -> String {
        self.sess.codemap().files.borrow().first().unwrap().name.clone()
    }

    #[inline]
    fn mod_name(&self) -> String {
        let mut name = self.file_name();
        if let Some(dot_pos) = name.rfind('.') {
            name.truncate(dot_pos);
        }
        name
    }

    fn is_wrapped(&self, sp: &rst::Span) -> bool {
        let start = self.last_loc.get().end;
        let end = sp.lo.0;
        if start > end {
            return false;
        }

        let snippet = self.sess.codemap().span_to_snippet(span(start, end)).unwrap();
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

    #[inline]
    fn lit(&self, pos: rst::BytePos) -> String {
        self.lits[&pos].clone()
    }

    #[inline]
    fn is_mod_decl(&self, sp: &rst::Span) -> bool {
        sp.lo.0 > self.krate.span.hi.0
    }

    fn trans(&self) -> Crate {
        self.trans_crate()
    }

    fn trans_crate(&self) -> Crate {
        let loc = self.loc(&self.krate.span);
        let attrs = self.trans_attrs(&self.krate.attrs);
        let module = self.trans_mod(self.mod_name(), &self.krate.module);
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
                                               self.trans_meta_items(meta_items));
                self.set_loc(&loc);
                meta_item
            }
        }
    }

    #[inline]
    fn trans_meta_items(&self, meta_items: &Vec<rst::P<rst::MetaItem>>) -> Vec<MetaItem> {
        trans_list!(self, meta_items, trans_meta_item)
    }

    fn trans_mod(&self, name: String, module: &rst::Mod) -> Mod {
        let loc = self.loc(&module.inner);
        let items = self.trans_items(&module.items);
        self.set_loc(&loc);
        Mod::new(loc, name, items)
    }

    #[inline]
    fn trans_items(&self, items: &Vec<rst::P<rst::Item>>) -> Vec<Item> {
        trans_list!(self, items, trans_item)
    }

    fn trans_item(&self, item: &rst::Item) -> Item {
        let loc = self.loc(&item.span);
        let attrs = self.trans_attrs(&item.attrs);

        let is_pub = is_pub(item.vis);
        let ident = ident_to_string(&item.ident);
        let item = match item.node {
            rst::ItemExternCrate(ref name) => {
                ItemKind::ExternCrate(self.trans_extren_crate(ident, name))
            }
            rst::ItemUse(ref view_path) => ItemKind::Use(self.trans_use(is_pub, view_path)),
            rst::ItemMod(ref module) => {
                if self.is_mod_decl(&module.inner) {
                    ItemKind::ModDecl(self.trans_mod_decl(is_pub, ident))
                } else {
                    ItemKind::Mod(self.trans_mod(ident, module))
                }
            }
            rst::ItemTy(ref ty, ref generics) => {
                ItemKind::TypeAlias(self.trans_type_alias(ident, generics, ty))
            }
            rst::ItemForeignMod(ref module) => ItemKind::ForeignMod(self.trans_foreign_mod(module)),
            rst::ItemStatic(ref ty, mutbl, ref expr) => {
                ItemKind::Static(self.trans_static(is_pub, is_mut(mutbl), ident, ty, expr))
            }
            rst::ItemConst(ref ty, ref expr) => {
                ItemKind::Const(self.trans_const(is_pub, ident, ty, expr))
            }
            _ => unreachable!(),
        };

        self.set_loc(&loc);
        Item::new(loc, attrs, item)
    }

    fn trans_extren_crate(&self, ident: String, name: &Option<rst::Name>) -> ExternCrate {
        let mut krate = ident;
        if let Some(ref rename) = *name {
            krate = format!("{} as {}", krate, name_to_string(rename));
        }
        ExternCrate::new(krate)
    }

    fn trans_use(&self, is_pub: bool, view_path: &rst::ViewPath) -> Use {
        match view_path.node {
            rst::ViewPathSimple(ref ident, ref path) => {
                self.loc_leaf(&path.span);
                let mut fullpath = self.use_path_to_string(path);
                if path.segments.last().unwrap().identifier.name != ident.name {
                    fullpath = format!("{} as {}", fullpath, ident_to_string(ident));
                }
                Use::new(is_pub, fullpath, Vec::new())
            }
            rst::ViewPathGlob(ref path) => {
                self.loc_leaf(&path.span);
                let fullpath = format!("{}::*", self.use_path_to_string(path));
                Use::new(is_pub, fullpath, Vec::new())
            }
            rst::ViewPathList(ref path, ref used_items) => {
                let loc = self.loc(&path.span);
                let fullpath = self.use_path_to_string(path);
                let use_item = Use::new(is_pub, fullpath, self.trans_used_items(used_items));
                self.set_loc(&loc);
                use_item
            }
        }
    }

    fn use_path_to_string(&self, path: &rst::Path) -> String {
        path.segments.iter().fold(String::new(), |mut s, e| {
            if !s.is_empty() {
                s.push_str("::");
            }
            s.push_str(&ident_to_string(&e.identifier));
            s
        })
    }

    #[inline]
    fn trans_used_items(&self, used_items: &Vec<rst::PathListItem>) -> Vec<Chunk> {
        trans_list!(self, used_items, trans_used_item)
    }

    fn trans_used_item(&self, used_item: &rst::PathListItem) -> Chunk {
        let loc = self.loc_leaf(&used_item.span);
        let (mut s, rename) = match used_item.node {
            rst::PathListIdent{ ref name, ref rename, .. } => (ident_to_string(name), rename),
            rst::PathListMod{ ref rename, .. } => ("self".to_string(), rename),
        };
        if let Some(ref ident) = *rename {
            s = format!("{} as {}", s, ident_to_string(ident));
        };

        Chunk::new(loc, s)
    }

    fn trans_mod_decl(&self, is_pub: bool, ident: String) -> ModDecl {
        ModDecl::new(is_pub, ident)
    }

    fn trans_type_alias(&self, ident: String, generics: &rst::Generics, ty: &rst::Ty) -> TypeAlias {
        TypeAlias::new(ident,
                       self.trans_generics(generics),
                       self.trans_type(ty))
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
                         self.trans_lifetimes(&lifetime_def.bounds))
    }

    #[inline]
    fn trans_lifetimes(&self, lifetimes: &Vec<rst::Lifetime>) -> Vec<Lifetime> {
        trans_list!(self, lifetimes, trans_lifetime)
    }

    fn trans_lifetime(&self, lifetime: &rst::Lifetime) -> Lifetime {
        Lifetime::new(self.loc_leaf(&lifetime.span), name_to_string(&lifetime.name))
    }

    #[inline]
    fn trans_type_params(&self, type_params: &[rst::TyParam]) -> Vec<TypeParam> {
        trans_list!(self, type_params, trans_type_param)
    }

    fn trans_type_param(&self, type_param: &rst::TyParam) -> TypeParam {
        let loc = self.loc(&type_param.span);
        let name = ident_to_string(&type_param.ident);
        let bounds = self.trans_type_param_bounds(&type_param.bounds);
        let default = match type_param.default {
            Some(ref ty) => Some(self.trans_type(ty)),
            None => None,
        };
        self.set_loc(&loc);
        TypeParam::new(loc, name, bounds, default)
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
            &rst::TraitTyParamBound(ref poly_trait_ref, is_sized) => {
                TypeParamBound::PolyTraitRef(self.trans_poly_trait_ref(poly_trait_ref, is_sized))
            }
        }
    }

    fn trans_poly_trait_ref(&self, poly_trait_ref: &rst::PolyTraitRef,
                            is_sized: rst::TraitBoundModifier)
        -> PolyTraitRef {
        if let rst::TraitBoundModifier::Maybe = is_sized {
            return PolyTraitRef::new_maybe_sized(self.loc_leaf(&poly_trait_ref.span));
        }

        let loc = self.loc(&poly_trait_ref.span);
        let lifetime_defs = self.trans_lifetime_defs(&poly_trait_ref.bound_lifetimes);
        let trait_ref = self.trans_trait_ref(&poly_trait_ref.trait_ref);
        self.set_loc(&loc);
        PolyTraitRef::new(loc, lifetime_defs, trait_ref)
    }

    fn trans_trait_ref(&self, trait_ref: &rst::TraitRef) -> TraitRef {
        self.trans_path(&trait_ref.path)
    }

    fn trans_path(&self, path: &rst::Path) -> Path {
        let loc = self.loc(&path.span);
        let segs = self.trans_path_segments(&path.segments);
        self.set_loc(&loc);
        Path::new(loc, path.global, segs)
    }

    #[inline]
    fn trans_path_segments(&self, segs: &Vec<rst::PathSegment>) -> Vec<PathSegment> {
        trans_list!(self, segs, trans_path_segment)
    }

    fn trans_path_segment(&self, seg: &rst::PathSegment) -> PathSegment {
        PathSegment::new(ident_to_string(&seg.identifier), self.trans_path_param(&seg.parameters))
    }

    fn trans_path_param(&self, param: &rst::PathParameters) -> PathParam {
        match param {
            &rst::AngleBracketed(ref param) => PathParam::Angle(self.trans_angle_param(param)),
            &rst::Parenthesized(ref param) => PathParam::Paren(self.trans_paren_param(param)),
        }
    }

    fn trans_angle_param(&self, param: &rst::AngleBracketedParameterData) -> AngleParam {
        AngleParam::new(self.trans_lifetimes(&param.lifetimes),
                        self.trans_types(&param.types),
                        self.trans_type_bindings(&param.bindings))
    }

    #[inline]
    fn trans_type_bindings(&self, bindings: &[rst::P<rst::TypeBinding>]) -> Vec<TypeBinding> {
        trans_list!(self, bindings, trans_type_binding)
    }

    fn trans_type_binding(&self, binding: &rst::TypeBinding) -> TypeBinding {
        TypeBinding::new(self.loc_leaf(&binding.span),
                         ident_to_string(&binding.ident),
                         self.trans_type(&binding.ty))
    }

    fn trans_paren_param(&self, param: &rst::ParenthesizedParameterData) -> ParenParam {
        let loc = self.loc(&param.span);
        let inputs = self.trans_types(&param.inputs);
        let output = match param.output {
            Some(ref ty) => Some(self.trans_type(ty)),
            None => None,
        };
        self.set_loc(&loc);
        ParenParam::new(loc, inputs, output)
    }

    #[inline]
    fn trans_types(&self, types: &[rst::P<rst::Ty>]) -> Vec<Type> {
        trans_list!(self, types, trans_type)
    }

    fn trans_type(&self, ty: &rst::Ty) -> Type {
        let loc = self.loc(&ty.span);

        let ty = match ty.node {
            rst::TyPath(ref qself, ref path) => {
                TypeKind::Path(Box::new(self.trans_path_type(qself, path)))
            }
            rst::TyPtr(ref mut_type) => TypeKind::Ptr(Box::new(self.trans_ptr_type(mut_type))),
            rst::TyRptr(ref lifetime, ref mut_type) => {
                TypeKind::Ref(Box::new(self.trans_ref_type(lifetime, mut_type)))
            }
            rst::TyVec(ref ty) => TypeKind::Array(Box::new(self.trans_array_type(ty))),
            rst::TyFixedLengthVec(ref ty, ref expr) => {
                TypeKind::FixedSizeArray(Box::new(self.trans_fixed_size_array_type(ty, expr)))
            }
            rst::TyTup(ref types) => TypeKind::Tuple(Box::new(self.trans_tuple_type(types))),
            rst::TyParen(ref ty) => {
                TypeKind::Tuple(Box::new(self.trans_tuple_type(&vec![ty.clone()])))
            }
            rst::TyBareFn(ref bare_fn) => {
                TypeKind::BareFn(Box::new(self.trans_bare_fn_type(bare_fn)))
            }
            rst::TyObjectSum(ref ty, ref bounds) => {
                TypeKind::Sum(Box::new(self.trans_sum_type(ty, bounds)))
            }
            rst::TyPolyTraitRef(ref bounds) => {
                TypeKind::PolyTraitRef(Box::new(self.trans_poly_trait_ref_type(bounds)))
            }
            rst::TyMac(ref mac) => TypeKind::Macro(Box::new(self.trans_macro_type(mac))),
            rst::TyInfer => TypeKind::Infer,
            _ => unreachable!(),
        };

        self.set_loc(&loc);
        Type::new(loc, ty)
    }

    fn trans_path_type(&self, qself: &Option<rst::QSelf>, path: &rst::Path) -> PathType {
        let qself = match qself {
            &Some(ref qself) => Some(self.trans_type(&qself.ty)),
            &None => None,
        };
        let path = self.trans_path(path);
        PathType::new(qself, path)
    }

    fn trans_ptr_type(&self, mut_type: &rst::MutTy) -> PtrType {
        PtrType::new(is_mut(mut_type.mutbl), self.trans_type(&mut_type.ty))
    }

    fn trans_ref_type(&self, lifetime: &Option<rst::Lifetime>, mut_type: &rst::MutTy) -> RefType {
        let lifetime = match lifetime {
            &Some(ref lifetime) => Some(self.trans_lifetime(lifetime)),
            &None => None,
        };
        let is_mut = is_mut(mut_type.mutbl);
        let ty = self.trans_type(&mut_type.ty);
        RefType::new(lifetime, is_mut, ty)
    }

    fn trans_array_type(&self, ty: &rst::Ty) -> ArrayType {
        ArrayType::new(self.trans_type(ty))
    }

    fn trans_fixed_size_array_type(&self, ty: &rst::Ty, expr: &rst::Expr) -> FixedSizeArrayType {
        FixedSizeArrayType::new(self.trans_type(ty), self.trans_expr(expr))
    }

    fn trans_tuple_type(&self, types: &Vec<rst::P<rst::Ty>>) -> TupleType {
        TupleType::new(self.trans_types(types))
    }

    fn trans_bare_fn_type(&self, bare_fn: &rst::BareFnTy) -> BareFnType {
        BareFnType::new(is_unsafe(bare_fn.unsafety),
                        abi_to_string(bare_fn.abi),
                        self.trans_lifetime_defs(&bare_fn.lifetimes),
                        self.trans_fn_decl(&bare_fn.decl))
    }

    fn trans_sum_type(&self, ty: &rst::Ty, bounds: &rst::TyParamBounds) -> SumType {
        SumType::new(self.trans_type(ty), self.trans_type_param_bounds(bounds))
    }

    fn trans_poly_trait_ref_type(&self, bounds: &rst::TyParamBounds) -> PolyTraitRefType {
        PolyTraitRefType::new(self.trans_type_param_bounds(bounds))
    }

    fn trans_macro_type(&self, mac: &rst::Mac) -> MacroType {
        self.trans_macro(mac)
    }

    fn trans_foreign_mod(&self, module: &rst::ForeignMod) -> ForeignMod {
        ForeignMod::new(abi_to_string(module.abi), self.trans_foreign_items(&module.items))
    }

    #[inline]
    fn trans_foreign_items(&self, items: &Vec<rst::P<rst::ForeignItem>>) -> Vec<Foreign> {
        trans_list!(self, items, trans_foreign_item)
    }

    fn trans_foreign_item(&self, item: &rst::ForeignItem) -> Foreign {
        let loc = self.loc(&item.span);
        let attrs = self.trans_attrs(&item.attrs);

        let item = match item.node {
            rst::ForeignItemStatic(ref ty, is_mut) => {
                ForeignKind::Static(self.trans_foreign_static(item, is_mut, ty))
            }
            rst::ForeignItemFn(ref fn_decl, ref generics) => {
                ForeignKind::Fn(self.trans_foreign_fn(is_pub(item.vis), generics, fn_decl))
            }
        };

        self.set_loc(&loc);
        Foreign::new(loc, attrs, item)
    }

    fn trans_foreign_static(&self, item: &rst::ForeignItem, is_mut: bool, ty: &rst::Ty)
        -> ForeignStatic {
        ForeignStatic::new(is_pub(item.vis),
                           is_mut,
                           ident_to_string(&item.ident),
                           self.trans_type(ty))
    }

    fn trans_foreign_fn(&self, is_pub: bool, generics: &rst::Generics, fn_decl: &rst::FnDecl)
        -> ForeignFn {
        ForeignFn::new(is_pub, self.trans_generics(generics), self.trans_fn_decl(fn_decl))
    }

    fn trans_static(&self, is_pub: bool, is_mut: bool, ident: String, ty: &rst::Ty, expr: &rst::Expr) -> Static {
        Static::new(is_pub, is_mut, ident, self.trans_type(ty), self.trans_expr(expr))
    }

    fn trans_const(&self, is_pub: bool, ident: String, ty: &rst::Ty, expr: &rst::Expr) -> Const {
        Const::new(is_pub, ident, self.trans_type(ty), self.trans_expr(expr))
    }

    fn trans_fn_decl(&self, fn_decl: &rst::FnDecl) -> FnDecl {
        FnDecl
    }

    fn trans_macro(&self, mac: &rst::Mac) -> Macro {
        Macro
    }

    fn trans_expr(&self, expr: &rst::Expr) -> Expr {
        Expr
    }
}
