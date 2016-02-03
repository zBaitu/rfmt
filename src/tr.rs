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
fn is_outer(style: rst::AttrStyle) -> bool {
    style == rst::AttrStyle::Outer
}

#[inline]
fn is_pub(vis: rst::Visibility) -> bool {
    vis == rst::Visibility::Public
}

#[inline]
fn is_sized(modifier: rst::TraitBoundModifier) -> bool {
    modifier == rst::TraitBoundModifier::Maybe
}

#[inline]
fn is_mut(mutbl: rst::Mutability) -> bool {
    mutbl == rst::Mutability::MutMutable
}

#[inline]
fn is_unsafe(safety: rst::Unsafety) -> bool {
    safety == rst::Unsafety::Unsafe
}

#[inline]
fn is_const(constness: rst::Constness) -> bool {
    constness == rst::Constness::Const
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
    format!(r#""{:?}""#, abi)
}

struct Trans {
    sess: rst::ParseSess,
    krate: rst::Crate,
    cmnts: Vec<rst::Comment>,
    cmnt_idx: u32,
    lits: HashMap<rst::BytePos, String>,

    last_loc: Cell<Loc>,
}

macro_rules! head_fn {
    ($fn_name:ident, $flag:ident, $true_value:expr, $false_value:expr) => (
        #[inline]
        fn $fn_name($flag: bool) -> &'static str {
            static TRUE_HEAD: &'static str = $true_value;
            static FALSE_HEAD: &'static str = $false_value;

            if $flag {
                TRUE_HEAD
            } else {
                FALSE_HEAD
            }
        }
    );
}
head_fn!(attr_head, is_outer, "#", "#!");
head_fn!(pub_head, is_pub, "pub ", "");
head_fn!(mut_head, is_mut, "mut ", "");
head_fn!(use_head, is_pub, "pub use", "use");
head_fn!(mod_head, is_pub, "pub mod", "mod");
head_fn!(type_head, is_pub, "pub type", "type");
head_fn!(path_head, global, "::", "");
head_fn!(ptr_head, is_mut, "*mut", "*const");
head_fn!(const_head, is_pub, "pub const", "const");
head_fn!(struct_head, is_pub, "pub struct", "struct");
head_fn!(enum_head, is_pub, "pub enum", "enum");
head_fn!(unsafe_head, is_unsafe, "unsafe ", "");
head_fn!(fn_const_head, is_const, "const ", "");

#[inline]
fn ref_head(lifetime: Option<Lifetime>, is_mut: bool) -> String {
    let mut head = String::new();
    head.push_str("&");

    if let Some(lifetime) = lifetime {
        head.push_str(&lifetime.s);
        head.push_str(" ");
    }

    head.push_str(mut_head(is_mut));
    head
}

#[inline]
fn static_head(is_pub: bool, is_mut: bool) -> String {
    format!("{}{}static ", pub_head(is_pub), mut_head(is_mut))
}

#[inline]
fn foreign_head(abi: &str) -> String {
    format!("extern {}", abi)
}

#[inline]
fn fn_head(is_pub: bool, is_unsafe: bool, is_const: bool, abi: Option<&str>) -> String {
    let mut head = format!("{}{}{}",
                           pub_head(is_pub),
                           unsafe_head(is_unsafe),
                           fn_const_head(is_const));
    if let Some(abi) = abi {
        if abi != r#""Rust""# {
            head.push_str(&foreign_head(abi));
            head.push_str(" ");
        }
    }
    head.push_str("fn ");
    head
}

#[inline]
fn trait_head(is_pub: bool, is_unsafe: bool) -> String {
    format!("{}{}trait ", pub_head(is_pub), unsafe_head(is_unsafe))
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
        Loc {
            start: sp.lo.0,
            end: sp.hi.0,
            wrapped: self.is_wrapped(sp),
        }
    }

    #[inline]
    fn loc_leaf(&self, sp: &rst::Span) -> Loc {
        let loc = self.loc(sp);
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

        Crate {
            loc: loc,
            attrs: attrs,
            module: module,
        }
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
                return Doc {
                    loc: self.loc_leaf(&attr.span),
                    s: s.to_string(),
                };
            }
        }

        unreachable!()
    }

    fn trans_attr_attr(&self, attr: &rst::Attribute) -> Attr {
        let loc = self.loc(&attr.span);
        let is_outer = is_outer(attr.node.style);
        let item = self.trans_meta_item(&attr.node.value);
        self.set_loc(&loc);

        Attr {
            loc: loc,
            head: attr_head(is_outer),
            item: item,
        }
    }

    fn trans_meta_item(&self, meta_item: &rst::MetaItem) -> MetaItem {
        match meta_item.node {
            rst::MetaWord(ref ident) => {
                // MetaItem::Single(Chunk::new(self.loc_leaf(&meta_item.span), ident.to_string()))
                MetaItem::Single(Chunk {
                    loc: self.loc_leaf(&meta_item.span),
                    s: ident.to_string(),
                })
            }
            rst::MetaNameValue(ref ident, ref lit) => {
                let s = format!("{} = {}", ident, self.lit(lit.span.lo));
                // MetaItem::Single(Chunk::new(self.loc_leaf(&meta_item.span), s))
                MetaItem::Single(Chunk {
                    loc: self.loc_leaf(&meta_item.span),
                    s: s,
                })
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

        Mod {
            loc: loc,
            name: name,
            items: items,
        }
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
                ItemKind::TypeAlias(self.trans_type_alias(is_pub, ident, generics, ty))
            }
            rst::ItemForeignMod(ref module) => ItemKind::ForeignMod(self.trans_foreign_mod(module)),
            rst::ItemConst(ref ty, ref expr) => {
                ItemKind::Const(self.trans_const(is_pub, ident, ty, expr))
            }
            rst::ItemStatic(ref ty, mutbl, ref expr) => {
                ItemKind::Static(self.trans_static(is_pub, is_mut(mutbl), ident, ty, expr))
            }
            rst::ItemStruct(ref variant, ref generics) => {
                ItemKind::Struct(self.trans_struct(is_pub, ident, generics, variant))
            }
            rst::ItemEnum(ref enum_def, ref generics) => {
                ItemKind::Enum(self.trans_enum(is_pub, ident, generics, enum_def))
            }
            rst::ItemFn(ref fn_decl, unsafety, constness, abi, ref generics, ref block) => {
                ItemKind::Fn(self.trans_fn(is_pub,
                                           is_unsafe(unsafety),
                                           is_const(constness),
                                           abi_to_string(abi),
                                           ident,
                                           generics,
                                           fn_decl,
                                           block))
            }
            rst::ItemTrait(unsafety, ref generics, ref bounds, ref items) => {
                ItemKind::Trait(self.trans_trait(is_pub,
                                                 is_unsafe(unsafety),
                                                 ident,
                                                 generics,
                                                 bounds,
                                                 items))
            }
            _ => unreachable!(),
        };

        self.set_loc(&loc);
        Item {
            loc: loc,
            attrs: attrs,
            item: item,
        }
    }

    fn trans_extren_crate(&self, ident: String, rename: &Option<rst::Name>) -> ExternCrate {
        let name = match *rename {
            Some(ref name) => format!("{} as {}", ident, name_to_string(name)),
            None => ident,
        };

        ExternCrate {
            head: "extern crate ",
            name: name,
        }
    }

    fn trans_use(&self, is_pub: bool, view_path: &rst::ViewPath) -> Use {
        let (full_path, items) = match view_path.node {
            rst::ViewPathSimple(ref ident, ref path) => {
                self.loc_leaf(&path.span);
                let mut full_path = self.use_path_to_string(path);
                if path.segments.last().unwrap().identifier.name != ident.name {
                    full_path = format!("{} as {}", full_path, ident_to_string(ident));
                }
                (full_path, Vec::new())
            }
            rst::ViewPathGlob(ref path) => {
                self.loc_leaf(&path.span);
                let full_path = format!("{}::*", self.use_path_to_string(path));
                (full_path, Vec::new())
            }
            rst::ViewPathList(ref path, ref items) => {
                let loc = self.loc(&path.span);
                let full_path = self.use_path_to_string(path);
                let items = self.trans_use_items(items);
                self.set_loc(&loc);
                (full_path, items)
            }
        };

        Use {
            head: use_head(is_pub),
            path: full_path,
            items: items,
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
    fn trans_use_items(&self, items: &Vec<rst::PathListItem>) -> Vec<Chunk> {
        trans_list!(self, items, trans_use_item)
    }

    fn trans_use_item(&self, item: &rst::PathListItem) -> Chunk {
        let loc = self.loc_leaf(&item.span);
        let (mut s, rename) = match item.node {
            rst::PathListIdent{ ref name, ref rename, .. } => (ident_to_string(name), rename),
            rst::PathListMod{ ref rename, .. } => ("self".to_string(), rename),
        };
        if let Some(ref ident) = *rename {
            s = format!("{} as {}", s, ident_to_string(ident));
        };

        Chunk {
            loc: loc,
            s: s,
        }
    }

    fn trans_mod_decl(&self, is_pub: bool, ident: String) -> ModDecl {
        ModDecl {
            head: mod_head(is_pub),
            name: ident,
        }
    }

    fn trans_type_alias(&self, is_pub: bool, ident: String, generics: &rst::Generics, ty: &rst::Ty)
        -> TypeAlias {
        TypeAlias {
            head: type_head(is_pub),
            name: ident,
            generics: self.trans_generics(generics),
            ty: self.trans_type(ty),
        }
    }

    fn trans_generics(&self, generics: &rst::Generics) -> Generics {
        Generics {
            lifetime_defs: self.trans_lifetime_defs(&generics.lifetimes),
            type_params: self.trans_type_params(&generics.ty_params),
        }
    }

    #[inline]
    fn trans_lifetime_defs(&self, lifetime_defs: &Vec<rst::LifetimeDef>) -> Vec<LifetimeDef> {
        trans_list!(self, lifetime_defs, trans_lifetime_def)
    }

    fn trans_lifetime_def(&self, lifetime_def: &rst::LifetimeDef) -> LifetimeDef {
        LifetimeDef {
            lifetime: self.trans_lifetime(&lifetime_def.lifetime),
            bounds: self.trans_lifetimes(&lifetime_def.bounds),
        }
    }

    #[inline]
    fn trans_lifetimes(&self, lifetimes: &Vec<rst::Lifetime>) -> Vec<Lifetime> {
        trans_list!(self, lifetimes, trans_lifetime)
    }

    fn trans_lifetime(&self, lifetime: &rst::Lifetime) -> Lifetime {
        Lifetime {
            loc: self.loc_leaf(&lifetime.span),
            s: name_to_string(&lifetime.name),
        }
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

        TypeParam {
            loc: loc,
            name: name,
            bounds: bounds,
            default: default,
        }
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
            &rst::TraitTyParamBound(ref poly_trait_ref, modifier) => {
                TypeParamBound::PolyTraitRef(self.trans_poly_trait_ref(poly_trait_ref, modifier))
            }
        }
    }

    fn trans_poly_trait_ref(&self, poly_trait_ref: &rst::PolyTraitRef,
                            modifier: rst::TraitBoundModifier)
        -> PolyTraitRef {
        if is_sized(modifier) {
            return PolyTraitRef::new_sized(self.loc_leaf(&poly_trait_ref.span));
        }

        let loc = self.loc(&poly_trait_ref.span);
        let lifetime_defs = self.trans_lifetime_defs(&poly_trait_ref.bound_lifetimes);
        let trait_ref = self.trans_trait_ref(&poly_trait_ref.trait_ref);
        self.set_loc(&loc);

        PolyTraitRef {
            loc: loc,
            lifetime_defs: lifetime_defs,
            trait_ref: trait_ref,
        }
    }

    fn trans_trait_ref(&self, trait_ref: &rst::TraitRef) -> TraitRef {
        self.trans_path(&trait_ref.path)
    }

    fn trans_path(&self, path: &rst::Path) -> Path {
        let loc = self.loc(&path.span);
        let segs = self.trans_path_segments(&path.segments);
        self.set_loc(&loc);

        Path {
            loc: loc,
            head: path_head(path.global),
            segs: segs,
        }
    }

    #[inline]
    fn trans_path_segments(&self, segs: &Vec<rst::PathSegment>) -> Vec<PathSegment> {
        trans_list!(self, segs, trans_path_segment)
    }

    fn trans_path_segment(&self, seg: &rst::PathSegment) -> PathSegment {
        PathSegment {
            name: ident_to_string(&seg.identifier),
            param: self.trans_path_param(&seg.parameters),
        }
    }

    fn trans_path_param(&self, params: &rst::PathParameters) -> PathParam {
        match params {
            &rst::AngleBracketed(ref param) => PathParam::Angle(self.trans_angle_param(param)),
            &rst::Parenthesized(ref param) => PathParam::Paren(self.trans_paren_param(param)),
        }
    }

    fn trans_angle_param(&self, param: &rst::AngleBracketedParameterData) -> AngleParam {
        AngleParam {
            lifetimes: self.trans_lifetimes(&param.lifetimes),
            types: self.trans_types(&param.types),
            bindings: self.trans_type_bindings(&param.bindings),
        }
    }

    #[inline]
    fn trans_type_bindings(&self, bindings: &[rst::P<rst::TypeBinding>]) -> Vec<TypeBinding> {
        trans_list!(self, bindings, trans_type_binding)
    }

    fn trans_type_binding(&self, binding: &rst::TypeBinding) -> TypeBinding {
        TypeBinding {
            loc: self.loc_leaf(&binding.span),
            name: ident_to_string(&binding.ident),
            ty: self.trans_type(&binding.ty),
        }
    }

    fn trans_paren_param(&self, param: &rst::ParenthesizedParameterData) -> ParenParam {
        let loc = self.loc(&param.span);
        let inputs = self.trans_types(&param.inputs);
        let output = match param.output {
            Some(ref ty) => Some(self.trans_type(ty)),
            None => None,
        };
        self.set_loc(&loc);

        ParenParam {
            loc: loc,
            inputs: inputs,
            output: output,
        }
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
        Type {
            loc: loc,
            ty: ty,
        }
    }

    fn trans_path_type(&self, qself: &Option<rst::QSelf>, path: &rst::Path) -> PathType {
        let qself = match qself {
            &Some(ref qself) => Some(self.trans_type(&qself.ty)),
            &None => None,
        };
        let path = self.trans_path(path);

        PathType {
            qself: qself,
            path: path,
        }
    }

    fn trans_ptr_type(&self, mut_type: &rst::MutTy) -> PtrType {
        PtrType {
            head: ptr_head(is_mut(mut_type.mutbl)),
            ty: self.trans_type(&mut_type.ty),
        }
    }

    fn trans_ref_type(&self, lifetime: &Option<rst::Lifetime>, mut_type: &rst::MutTy) -> RefType {
        let lifetime = match lifetime {
            &Some(ref lifetime) => Some(self.trans_lifetime(lifetime)),
            &None => None,
        };
        let is_mut = is_mut(mut_type.mutbl);
        let ty = self.trans_type(&mut_type.ty);

        RefType {
            head: ref_head(lifetime, is_mut),
            ty: ty,
        }
    }

    fn trans_array_type(&self, ty: &rst::Ty) -> ArrayType {
        ArrayType {
            ty: self.trans_type(ty),
        }
    }

    fn trans_fixed_size_array_type(&self, ty: &rst::Ty, expr: &rst::Expr) -> FixedSizeArrayType {
        FixedSizeArrayType {
            ty: self.trans_type(ty),
            expr: self.trans_expr(expr),
        }
    }

    fn trans_tuple_type(&self, types: &Vec<rst::P<rst::Ty>>) -> TupleType {
        TupleType {
            types: self.trans_types(types),
        }
    }

    fn trans_bare_fn_type(&self, bare_fn: &rst::BareFnTy) -> BareFnType {
        BareFnType {
            head: fn_head(false,
                          is_unsafe(bare_fn.unsafety),
                          false,
                          Some(&abi_to_string(bare_fn.abi))),
            lifetime_defs: self.trans_lifetime_defs(&bare_fn.lifetimes),
            fn_sig: self.trans_fn_sig(&bare_fn.decl),
        }
    }

    fn trans_sum_type(&self, ty: &rst::Ty, bounds: &rst::TyParamBounds) -> SumType {
        SumType {
            ty: self.trans_type(ty),
            bounds: self.trans_type_param_bounds(bounds),
        }
    }

    fn trans_poly_trait_ref_type(&self, bounds: &rst::TyParamBounds) -> PolyTraitRefType {
        PolyTraitRefType {
            bounds: self.trans_type_param_bounds(bounds),
        }
    }

    fn trans_foreign_mod(&self, module: &rst::ForeignMod) -> ForeignMod {
        ForeignMod {
            head: foreign_head(&abi_to_string(module.abi)),
            items: self.trans_foreign_items(&module.items),
        }
    }

    #[inline]
    fn trans_foreign_items(&self, items: &Vec<rst::P<rst::ForeignItem>>) -> Vec<Foreign> {
        trans_list!(self, items, trans_foreign_item)
    }

    fn trans_foreign_item(&self, item: &rst::ForeignItem) -> Foreign {
        let loc = self.loc(&item.span);
        let attrs = self.trans_attrs(&item.attrs);

        let is_pub = is_pub(item.vis);
        let ident = ident_to_string(&item.ident);
        let item = match item.node {
            rst::ForeignItemStatic(ref ty, is_mut) => {
                ForeignKind::Static(self.trans_foreign_static(is_pub, is_mut, ident, ty))
            }
            rst::ForeignItemFn(ref fn_decl, ref generics) => {
                ForeignKind::Fn(self.trans_foreign_fn(is_pub, ident, generics, fn_decl))
            }
        };

        self.set_loc(&loc);
        Foreign {
            loc: loc,
            attrs: attrs,
            item: item,
        }
    }

    fn trans_foreign_static(&self, is_pub: bool, is_mut: bool, ident: String, ty: &rst::Ty)
        -> ForeignStatic {
        ForeignStatic {
            head: static_head(is_pub, is_mut),
            name: ident,
            ty: self.trans_type(ty),
        }
    }

    fn trans_foreign_fn(&self, is_pub: bool, ident: String, generics: &rst::Generics,
                        fn_decl: &rst::FnDecl)
        -> ForeignFn {
        ForeignFn {
            head: fn_head(is_pub, false, false, None),
            name: ident,
            generics: self.trans_generics(generics),
            fn_sig: self.trans_fn_sig(fn_decl),
        }
    }

    fn trans_const(&self, is_pub: bool, ident: String, ty: &rst::Ty, expr: &rst::Expr) -> Const {
        Const {
            head: const_head(is_pub),
            name: ident,
            ty: self.trans_type(ty),
            expr: self.trans_expr(expr),
        }
    }

    fn trans_static(&self, is_pub: bool, is_mut: bool, ident: String, ty: &rst::Ty,
                    expr: &rst::Expr)
        -> Static {
        Static {
            head: static_head(is_pub, is_mut),
            name: ident,
            ty: self.trans_type(ty),
            expr: self.trans_expr(expr),
        }
    }

    fn trans_struct(&self, is_pub: bool, ident: String, generics: &rst::Generics,
                    variant: &rst::VariantData)
        -> Struct {
        Struct {
            head: struct_head(is_pub),
            name: ident,
            generics: self.trans_generics(generics),
            body: self.trans_struct_body(variant),
        }
    }

    fn trans_struct_body(&self, variant: &rst::VariantData) -> StructBody {
        match variant {
            &rst::VariantData::Struct(ref fields, _) => {
                StructBody::Struct(self.trans_struct_fields(fields))
            }
            &rst::VariantData::Tuple(ref fields, _) => {
                StructBody::Tuple(self.trans_tuple_fields(fields))
            }
            &rst::VariantData::Unit(_) => StructBody::Unit,
        }
    }

    #[inline]
    fn trans_struct_fields(&self, fields: &Vec<rst::StructField>) -> Vec<StructField> {
        trans_list!(self, fields, trans_struct_field)
    }

    fn trans_struct_field(&self, field: &rst::StructField) -> StructField {
        let (is_pub, name) = match field.node.kind {
            rst::StructFieldKind::NamedField(ref ident, vis) => {
                (is_pub(vis), ident_to_string(ident))
            }
            _ => unreachable!(),
        };

        let loc = self.loc(&field.span);
        let attrs = self.trans_attrs(&field.node.attrs);
        let ty = self.trans_type(&field.node.ty);
        self.set_loc(&loc);

        StructField {
            loc: loc,
            attrs: attrs,
            head: pub_head(is_pub),
            name: name,
            ty: ty,
        }
    }

    #[inline]
    fn trans_tuple_fields(&self, fields: &Vec<rst::StructField>) -> Vec<TupleField> {
        trans_list!(self, fields, trans_tuple_field)
    }

    fn trans_tuple_field(&self, field: &rst::StructField) -> TupleField {
        let is_pub = match field.node.kind {
            rst::StructFieldKind::UnnamedField(vis) => is_pub(vis),
            _ => unreachable!(),
        };

        let loc = self.loc(&field.span);
        let attrs = self.trans_attrs(&field.node.attrs);
        let ty = self.trans_type(&field.node.ty);
        self.set_loc(&loc);

        TupleField {
            loc: loc,
            attrs: attrs,
            head: pub_head(is_pub),
            ty: ty,
        }
    }

    fn trans_enum(&self, is_pub: bool, ident: String, generics: &rst::Generics,
                  enum_def: &rst::EnumDef)
        -> Enum {
        Enum {
            head: enum_head(is_pub),
            name: ident,
            generics: self.trans_generics(generics),
            body: self.trans_enum_body(enum_def),
        }
    }

    fn trans_enum_body(&self, enum_def: &rst::EnumDef) -> EnumBody {
        EnumBody {
            fields: self.trans_enum_fields(&enum_def.variants),
        }
    }

    #[inline]
    fn trans_enum_fields(&self, variants: &Vec<rst::P<rst::Variant>>) -> Vec<EnumField> {
        trans_list!(self, variants, trans_enum_field)
    }

    fn trans_enum_field(&self, variant: &rst::Variant) -> EnumField {
        let loc = self.loc(&variant.span);
        let attrs = self.trans_attrs(&variant.node.attrs);
        let name = ident_to_string(&variant.node.name);
        let body = self.trans_struct_body(&variant.node.data);
        let expr = match variant.node.disr_expr {
            Some(ref expr) => Some(self.trans_expr(expr)),
            None => None,
        };
        self.set_loc(&loc);

        EnumField {
            loc: loc,
            attrs: attrs,
            name: name,
            body: body,
            expr: expr,
        }
    }

    fn trans_fn(&self, is_pub: bool, is_unsafe: bool, is_const: bool, abi: String, ident: String,
                generics: &rst::Generics, fn_decl: &rst::FnDecl, block: &rst::Block)
        -> Fn {
        Fn {
            head: fn_head(is_pub, is_unsafe, is_const, Some(&abi)),
            name: ident,
            generics: self.trans_generics(generics),
            fn_sig: self.trans_fn_sig(fn_decl),
            block: self.trans_block(block),
        }
    }

    fn trans_trait(&self, is_pub: bool, is_unsafe: bool, ident: String, generics: &rst::Generics,
                   bounds: &rst::TyParamBounds, items: &Vec<rst::P<rst::TraitItem>>)
        -> Trait {
        Trait {
            head: trait_head(is_pub, is_unsafe),
            name: ident,
            generics: self.trans_generics(generics),
            bounds: self.trans_type_param_bounds(bounds),
            items: self.trans_trait_items(items),
        }
    }

    #[inline]
    fn trans_trait_items(&self, items: &Vec<rst::P<rst::TraitItem>>) -> Vec<TraitItem> {
        trans_list!(self, items, trans_trait_item)
    }

    fn trans_trait_item(&self, item: &rst::TraitItem) -> TraitItem {
        let loc = self.loc(&item.span);
        let attrs = self.trans_attrs(&item.attrs);
        let ident = ident_to_string(&item.ident);
        let item = match item.node {
            rst::ConstTraitItem(ref ty, ref expr) => {
                TraitItemKind::Const(self.trans_const_trait_item(ident, ty, expr))
            }
            rst::TypeTraitItem(ref bounds, ref ty) => {
                TraitItemKind::Type(self.trans_type_trait_item(ident, bounds, ty))
            }
            rst::MethodTraitItem(ref method_sig, ref block) => {
                TraitItemKind::Method(self.trans_method_trait_item(ident, method_sig, block))
            }
        };
        self.set_loc(&loc);

        TraitItem {
            loc: loc,
            attrs: attrs,
            item: item,
        }
    }

    fn trans_const_trait_item(&self, ident: String, ty: &rst::Ty, expr: &Option<rst::P<rst::Expr>>)
        -> ConstTraitItem {
        let expr = match *expr {
            Some(ref expr) => Some(self.trans_expr(expr)),
            None => None,
        };

        ConstTraitItem {
            head: const_head(false),
            name: ident,
            ty: self.trans_type(ty),
            expr: expr,
        }
    }

    fn trans_type_trait_item(&self, ident: String, bounds: &rst::TyParamBounds,
                             ty: &Option<rst::P<rst::Ty>>)
        -> TypeTraitItem {
        let ty = match *ty {
            Some(ref ty) => Some(self.trans_type(ty)),
            None => None,
        };

        TypeTraitItem {
            head: type_head(false),
            name: ident,
            bounds: self.trans_type_param_bounds(bounds),
            ty: ty,
        }
    }

    fn trans_method_trait_item(&self, ident: String, method_sig: &rst::MethodSig,
                               block: &Option<rst::P<rst::Block>>)
        -> MethodTraitItem {
        let block = match *block {
            Some(ref block) => Some(self.trans_block(block)),
            None => None,
        };

        MethodTraitItem {
            head: fn_head(false,
                          is_unsafe(method_sig.unsafety),
                          is_const(method_sig.constness),
                          Some(&abi_to_string(method_sig.abi))),
            name: ident,
            method_sig: self.trans_method_sig(method_sig),
            block: block,
        }
    }

    fn trans_fn_sig(&self, fn_decl: &rst::FnDecl) -> FnSig {
        FnSig
    }

    fn trans_method_sig(&self, method_sig: &rst::MethodSig) -> MethodSig {
        MethodSig {
            generics: self.trans_generics(&method_sig.generics),
            fn_sig: self.trans_fn_sig(&method_sig.decl),
            slf: self.trans_self(&method_sig.explicit_self.node),
        }
    }

    fn trans_self(&self, explicit_self: &rst::ExplicitSelf_) -> Option<String> {
        match *explicit_self {
            rst::SelfStatic => None,
            rst::SelfValue(_) => Some("self".to_string()),
            rst::SelfRegion(lifetime, mutbl, _) => {
                let mut s = String::new();
                s.push_str("&");

                if let Some(ref lifetime) = lifetime {
                    let lifetime = self.trans_lifetime(lifetime);
                    s.push_str(&lifetime.s);
                    s.push_str(" ");
                }

                if is_mut(mutbl) {
                    s.push_str("mut ");
                }

                s.push_str("self");
                Some(s)
            }
            _ => unreachable!(),
        }
    }

    fn trans_block(&self, block: &rst::Block) -> Block {
        Block
    }

    fn trans_expr(&self, expr: &rst::Expr) -> Expr {
        Expr
    }

    fn trans_macro(&self, mac: &rst::Mac) -> Macro {
        Macro
    }

    fn trans_macro_type(&self, mac: &rst::Mac) -> MacroType {
        self.trans_macro(mac)
    }
}
