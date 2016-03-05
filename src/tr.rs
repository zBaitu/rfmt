use std::cmp::Ordering;
use std::collections::HashMap;
use std::mem;
use std::path;
use std::result;

use rst;
use zbase::zopt;

use ir::*;

const MAX_BLANK_LINE: u8 = 2;

fn trans_literal(lits: Vec<rst::Literal>) -> HashMap<rst::BytePos, String> {
    lits.into_iter().map(|e| (e.pos, e.lit)).collect()
}

#[derive(Debug, PartialEq)]
enum CommentKind {
    Leading,
    Trailing,
}

#[derive(Debug)]
struct Comment {
    pos: Pos,
    kind: CommentKind,
    lines: Vec<String>,
}

fn trans_comments(cmnts: Vec<rst::Comment>) -> Vec<Comment> {
    let mut pre_blank_line_pos = 0;
    let mut blank_line = 0;

    cmnts.into_iter().fold(Vec::new(), |mut cmnts, cmnt| {
        if cmnt.style == rst::CommentStyle::BlankLine {
            let cur_pos = cmnt.pos.0;

            if cur_pos != pre_blank_line_pos + 1 {
                blank_line = 1;
                cmnts.push(trans_comment(cmnt));
            } else {
                blank_line += 1;
                if blank_line <= MAX_BLANK_LINE {
                    cmnts.push(trans_comment(cmnt));
                }
            }

            pre_blank_line_pos = cur_pos;
        } else {
            blank_line = 0;
            cmnts.push(trans_comment(cmnt));
        }

        cmnts
    })
}

fn trans_comment(cmnt: rst::Comment) -> Comment {
    let kind = match cmnt.style {
        rst::CommentStyle::Trailing => CommentKind::Trailing,
        _ => CommentKind::Leading,
    };

    Comment {
        pos: cmnt.pos.0,
        kind: kind,
        lines: cmnt.lines,
    }
}

#[inline]
fn span(s: u32, e: u32) -> rst::Span {
    rst::codemap::mk_sp(rst::BytePos(s), rst::BytePos(e))
}

#[inline]
fn is_inner(style: rst::AttrStyle) -> bool {
    style == rst::AttrStyle::Inner
}

#[inline]
fn is_outer(style: rst::AttrStyle) -> bool {
    !is_inner(style)
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
fn is_neg(polarity: rst::ImplPolarity) -> bool {
    polarity == rst::ImplPolarity::Negative
}

#[inline]
fn is_block_unsafe(rules: rst::BlockCheckMode) -> bool {
    match rules {
        rst::BlockCheckMode::UnsafeBlock(source) => source == rst::UnsafeSource::UserProvided,
        _ => false,
    }
}

#[inline]
fn is_move(capture: rst::CaptureClause) -> bool {
    capture == rst::CaptureClause::CaptureByValue
}

#[inline]
fn is_ref_mut(mode: rst::BindingMode) -> (bool, bool) {
    match mode {
        rst::BindingMode::ByRef(mutbl) => (true, is_mut(mutbl)),
        rst::BindingMode::ByValue(mutbl) => (false, is_mut(mutbl)),
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
fn path_to_string(path: &rst::Path) -> String {
    path.segments.iter().fold(String::new(), |mut s, e| {
        if !s.is_empty() {
            s.push_str("::");
        }
        s.push_str(&ident_to_string(&e.identifier));
        s
    })
}

#[inline]
fn abi_to_string(abi: rst::abi::Abi) -> String {
    format!(r#""{:?}""#, abi)
}

#[inline]
fn uop_to_string(op: rst::UnOp) -> &'static str {
    rst::UnOp::to_string(op)
}

pub struct CrateResult {
    pub krate: Crate,
    pub leading_cmnts: HashMap<Pos, Vec<String>>,
    pub trailing_cmnts: HashMap<Pos, String>,
}

pub struct ModResult {
    pub module: Mod,
    pub leading_cmnts: HashMap<Pos, Vec<String>>,
    pub trailing_cmnts: HashMap<Pos, String>,
}

pub struct Translator {
    sess: rst::ParseSess,
    lits: HashMap<rst::BytePos, String>,
    cmnts: Vec<Comment>,
    cmnt_idx: usize,

    mod_full_file_name: String,
    mod_paths: Vec<String>,
    mod_end: Pos,
    last_loc: Loc,

    leading_cmnts: HashMap<Pos, Vec<String>>,
    trailing_cmnts: HashMap<Pos, String>,
}

macro_rules! trans_list {
    ($sf: ident, $list: ident, $trans_single: ident) => ({
        $list.iter().map(|ref e| $sf.$trans_single(e)).collect()
    });
}

impl Translator {
    pub fn new(sess: rst::ParseSess, lits: Vec<rst::Literal>, cmnts: Vec<rst::Comment>) -> Translator {
        Translator {
            sess: sess,
            lits: trans_literal(lits),
            cmnts: trans_comments(cmnts),
            cmnt_idx: 0,

            mod_full_file_name: String::new(),
            mod_paths: Vec::new(),
            mod_end: 0,
            last_loc: Default::default(),

            leading_cmnts: HashMap::new(),
            trailing_cmnts: HashMap::new(),
        }
    }

    pub fn trans_crate(&mut self, krate: &rst::Crate) -> CrateResult {
        self.mod_end = krate.span.hi.0;
        self.last_loc.start = krate.span.lo.0;

        let loc = self.loc(&krate.span);
        let attrs = self.trans_attrs(&krate.attrs);
        let crate_mod_name = self.crate_mod_name();
        let module = self.trans_mod_(crate_mod_name, &krate.module);

        CrateResult {
            krate: Crate {
                loc: loc,
                attrs: attrs,
                module: module,
            },
            leading_cmnts: mem::replace(&mut self.leading_cmnts, HashMap::new()),
            trailing_cmnts: mem::replace(&mut self.trailing_cmnts, HashMap::new()),
        }
    }

    #[inline]
    fn crate_file_name(&self) -> String {
        self.sess.codemap().files.borrow().first().unwrap().name.clone()
    }

    #[inline]
    fn crate_mod_name(&self) -> String {
        let mut name = self.crate_file_name();
        if let Some(pos) = name.rfind('.') {
            name.truncate(pos);
        }
        name
    }

    pub fn trans_mod(&mut self, name: String, module: &rst::Mod) -> ModResult {
        ModResult {
            module: self.trans_mod_(name, module),
            leading_cmnts: mem::replace(&mut self.leading_cmnts, HashMap::new()),
            trailing_cmnts: mem::replace(&mut self.trailing_cmnts, HashMap::new()),
        }
    }

    #[inline]
    fn trans_comments(&mut self, pos: Pos) {
        let cmnts = self.trans_trailing_comments(pos);
        self.trans_leading_comments(pos, cmnts);
    }

    #[inline]
    fn trans_trailing_comments(&mut self, pos: Pos) -> Vec<String> {
        let mut cmnts = Vec::new();

        if self.cmnt_idx >= self.cmnts.len() {
            return cmnts;
        }
        let cmnt = &self.cmnts[self.cmnt_idx];
        if cmnt.pos >= pos || cmnt.kind != CommentKind::Trailing {
            return cmnts;
        }
        self.cmnt_idx += 1;

        self.trailing_cmnts.insert(self.last_loc.end, cmnt.lines[0].clone());
        cmnts.extend_from_slice(&cmnt.lines[1..]);
        cmnts
    }

    #[inline]
    fn trans_leading_comments(&mut self, pos: Pos, mut cmnts: Vec<String>) {
        while self.cmnt_idx < self.cmnts.len() {
            let cmnt = &self.cmnts[self.cmnt_idx];
            if cmnt.pos >= pos {
                break;
            }

            if cmnt.lines.is_empty() {
                cmnts.push(String::new());
            } else {
                cmnts.extend_from_slice(&cmnt.lines);
            }

            self.cmnt_idx += 1;
        }

        if !cmnts.is_empty() {
            self.leading_cmnts.insert(pos, cmnts);
        }
    }

    #[inline]
    fn span_to_snippet(&self, sp: rst::Span) -> result::Result<String, rst::SpanSnippetError> {
        self.sess.codemap().span_to_snippet(sp)
    }

    #[inline]
    fn is_nl(&self, pos: Pos) -> bool {
        let snippet = self.span_to_snippet(span(self.last_loc.end, pos));
        if snippet.is_err() {
            return false;
        }

        let snippet = snippet.unwrap();
        let linefeed = snippet.find('\n');
        if linefeed.is_none() {
            return false;
        }

        let start = linefeed.unwrap() + 1;
        for ch in snippet[start..].chars() {
            if !ch.is_whitespace() {
                return false;
            }
        }
        true
    }

    #[inline]
    fn loc(&mut self, sp: &rst::Span) -> Loc {
        self.trans_comments(sp.lo.0);

        Loc {
            start: sp.lo.0,
            end: sp.hi.0,
            nl: self.is_nl(sp.lo.0),
        }
    }

    #[inline]
    fn leaf_loc(&mut self, sp: &rst::Span) -> Loc {
        let loc = self.loc(sp);
        self.set_loc(&loc);
        loc
    }

    #[inline]
    fn set_loc(&mut self, loc: &Loc) {
        self.last_loc = *loc;
    }

    #[inline]
    fn literal_to_string(&self, lit: &rst::Lit) -> String {
        match lit.node {
            rst::LitBool(b) => b.to_string(),
            _ => self.lits[&lit.span.lo].clone(),
        }
    }

    #[inline]
    fn trans_attrs(&mut self, attrs: &Vec<rst::Attribute>) -> Vec<AttrKind> {
        trans_list!(self, attrs, trans_attr_kind)
    }

    #[inline]
    fn trans_outer_attrs(&mut self, attrs: &Vec<rst::Attribute>) -> Vec<AttrKind> {
        attrs.iter()
             .filter(|ref e| is_outer(e.node.style))
             .map(|ref e| self.trans_attr_kind(e))
             .collect()
    }

    #[inline]
    fn trans_attr_kind(&mut self, attr: &rst::Attribute) -> AttrKind {
        if attr.node.is_sugared_doc {
            AttrKind::Doc(self.trans_doc(attr))
        } else {
            AttrKind::Attr(self.trans_attr(attr))
        }
    }

    #[inline]
    fn trans_doc(&mut self, attr: &rst::Attribute) -> Doc {
        if let rst::MetaNameValue(_, ref value) = attr.node.value.node {
            if let rst::LitStr(ref s, _) = value.node {
                return Doc {
                    loc: self.leaf_loc(&attr.span),
                    s: s.to_string(),
                };
            }
        }

        unreachable!()
    }

    #[inline]
    fn trans_attr(&mut self, attr: &rst::Attribute) -> Attr {
        let loc = self.loc(&attr.span);
        let is_inner = is_inner(attr.node.style);
        let item = self.trans_meta_item(&attr.node.value);
        self.set_loc(&loc);

        Attr {
            loc: loc,
            is_inner: is_inner,
            item: item,
        }
    }

    #[inline]
    fn trans_meta_items(&mut self, meta_items: &Vec<rst::P<rst::MetaItem>>) -> Vec<MetaItem> {
        trans_list!(self, meta_items, trans_meta_item)
    }

    #[inline]
    fn trans_meta_item(&mut self, meta_item: &rst::MetaItem) -> MetaItem {
        match meta_item.node {
            rst::MetaWord(ref ident) => {
                MetaItem {
                    loc: self.leaf_loc(&meta_item.span),
                    name: ident.to_string(),
                    items: None,
                }
            }
            rst::MetaNameValue(ref ident, ref lit) => {
                let s = format!("{} = {}", ident, self.literal_to_string(lit));
                MetaItem {
                    loc: self.leaf_loc(&meta_item.span),
                    name: s,
                    items: None,
                }
            }
            rst::MetaList(ref ident, ref meta_items) => {
                let loc = self.loc(&meta_item.span);
                let items = self.trans_meta_items(meta_items);
                self.set_loc(&loc);

                MetaItem {
                    loc: loc,
                    name: ident.to_string(),
                    items: Some(Box::new(items)),
                }
            }
        }
    }

    fn trans_mod_(&mut self, name: String, module: &rst::Mod) -> Mod {
        let loc = self.loc(&module.inner);
        let items = self.trans_items(&module.items);
        self.set_loc(&loc);

        let mod_file_end = self.mod_file_end(module);
        self.trans_comments(mod_file_end);

        Mod {
            loc: loc,
            name: name,
            items: items,
        }
    }

    pub fn set_mod_full_file_name(&mut self, name: String) {
        self.mod_full_file_name = name;
    }

    #[inline]
    fn is_mod_decl(&self, sp: &rst::Span) -> bool {
        sp.lo.0 > self.mod_end
    }

    #[inline]
    fn mod_full_name(&self) -> String {
        self.mod_paths.iter().fold(String::new(), |mut s, ref path| {
            if !s.is_empty() {
                s.push(path::MAIN_SEPARATOR);
            }
            s.push_str(path);
            s
        })
    }

    #[inline]
    fn mod_full_file_name(&self) -> String {
        if self.mod_full_file_name.is_empty() {
            join_str!(self.mod_full_name(), ".rs")
        } else {
            self.mod_full_file_name.clone()
        }
    }

    #[inline]
    fn mod_file_end(&mut self, module: &rst::Mod) -> Pos {
        if self.is_mod_decl(&module.inner) {
            let file_name = self.mod_full_file_name();
            p!("{}", file_name);
            self.sess.codemap().files.borrow().iter().find(|e| e.name == file_name).unwrap().end_pos.0
        } else {
            module.inner.hi.0
        }
    }

    #[inline]
    fn is_mod_decl_item(&self, item: &rst::Item) -> bool {
        match item.node {
            rst::ItemMod(ref module) if self.is_mod_decl(&module.inner) => true,
            _ => false,
        }
    }

    #[inline]
    fn trans_items(&mut self, items: &Vec<rst::P<rst::Item>>) -> Vec<Item> {
        trans_list!(self, items, trans_item)
    }

    fn trans_item(&mut self, item: &rst::Item) -> Item {
        let loc = self.loc(&item.span);

        let attrs = if self.is_mod_decl_item(item) {
            self.trans_outer_attrs(&item.attrs)
        } else {
            self.trans_attrs(&item.attrs)
        };

        let is_pub = is_pub(item.vis);
        let ident = ident_to_string(&item.ident);
        let item = match item.node {
            rst::ItemExternCrate(ref name) => {
                ItemKind::ExternCrate(self.trans_extren_crate(ident, name))
            }
            rst::ItemUse(ref view_path) => ItemKind::Use(self.trans_use(view_path)),
            rst::ItemMod(ref module) => {
                if self.is_mod_decl(&module.inner) {
                    ItemKind::ModDecl(self.trans_mod_decl(ident))
                } else {
                    self.mod_paths.push(ident.clone());
                    let item = ItemKind::Mod(self.trans_mod_(ident, module));
                    self.mod_paths.pop();
                    item
                }
            }
            rst::ItemTy(ref ty, ref generics) => {
                ItemKind::TypeAlias(self.trans_type_alias(ident, generics, ty))
            }
            rst::ItemForeignMod(ref module) => ItemKind::ForeignMod(self.trans_foreign_mod(module)),
            rst::ItemConst(ref ty, ref expr) => ItemKind::Const(self.trans_const(ident, ty, expr)),
            rst::ItemStatic(ref ty, mutbl, ref expr) => {
                ItemKind::Static(self.trans_static(is_mut(mutbl), ident, ty, expr))
            }
            rst::ItemStruct(ref variant, ref generics) => {
                ItemKind::Struct(self.trans_struct(ident, generics, variant))
            }
            rst::ItemEnum(ref enum_def, ref generics) => {
                ItemKind::Enum(self.trans_enum(ident, generics, enum_def))
            }
            rst::ItemFn(ref fn_decl, unsafety, constness, abi, ref generics, ref block) => {
                ItemKind::Fn(self.trans_fn(is_unsafe(unsafety),
                                           is_const(constness),
                                           abi_to_string(abi),
                                           ident,
                                           generics,
                                           fn_decl,
                                           block))
            }
            rst::ItemTrait(unsafety, ref generics, ref bounds, ref items) => {
                ItemKind::Trait(self.trans_trait(is_unsafe(unsafety),
                                                 ident,
                                                 generics,
                                                 bounds,
                                                 items))
            }
            rst::ItemDefaultImpl(unsafety, ref trait_ref) => {
                ItemKind::ImplDefault(self.trans_impl_default(is_unsafe(unsafety), trait_ref))
            }
            rst::ItemImpl(unsafety, polarity, ref generics, ref trait_ref, ref ty, ref items) => {
                ItemKind::Impl(self.trans_impl(is_unsafe(unsafety),
                                               is_neg(polarity),
                                               generics,
                                               trait_ref,
                                               ty,
                                               items))
            }
            rst::ItemMac(ref mac) => ItemKind::Macro(self.trans_macro(mac)),
        };

        self.set_loc(&loc);
        Item {
            loc: loc,
            attrs: attrs,
            is_pub: is_pub,
            item: item,
        }
    }

    fn trans_extren_crate(&mut self, ident: String, rename: &Option<rst::Name>) -> ExternCrate {
        let name = match *rename {
            Some(ref name) => format!("{} as {}", ident, name_to_string(name)),
            None => ident,
        };

        ExternCrate {
            name: name,
        }
    }

    fn trans_use(&mut self, view_path: &rst::ViewPath) -> Use {
        let (base, mut names) = match view_path.node {
            rst::ViewPathSimple(ref ident, ref path) => {
                self.leaf_loc(&path.span);
                let mut base = path_to_string(path);
                if path.segments.last().unwrap().identifier.name != ident.name {
                    base = format!("{} as {}", base, ident_to_string(ident));
                }
                (base, Vec::new())
            }
            rst::ViewPathGlob(ref path) => {
                self.leaf_loc(&path.span);
                let base = format!("{}::*", path_to_string(path));
                (base, Vec::new())
            }
            rst::ViewPathList(ref path, ref items) => {
                let loc = self.loc(&path.span);
                let base = path_to_string(path);
                let names = self.trans_use_names(items);
                self.set_loc(&loc);
                (base, names)
            }
        };

        names.sort_by(|a, b| {
            if a.s == "self" {
                Ordering::Less
            } else if b.s == "self" {
                Ordering::Greater
            } else {
                a.s.cmp(&b.s)
            }
        });

        Use {
            base: base,
            names: names,
        }
    }

    #[inline]
    fn trans_use_names(&mut self, items: &Vec<rst::PathListItem>) -> Vec<Chunk> {
        trans_list!(self, items, trans_use_name)
    }

    fn trans_use_name(&mut self, item: &rst::PathListItem) -> Chunk {
        let loc = self.leaf_loc(&item.span);
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

    fn trans_mod_decl(&mut self, ident: String) -> ModDecl {
        ModDecl {
            name: ident,
        }
    }

    fn trans_type_alias(&mut self, ident: String, generics: &rst::Generics, ty: &rst::Ty)
        -> TypeAlias {
        TypeAlias {
            name: ident,
            generics: self.trans_generics(generics),
            ty: self.trans_type(ty),
        }
    }

    fn trans_generics(&mut self, generics: &rst::Generics) -> Generics {
        Generics {
            lifetime_defs: self.trans_lifetime_defs(&generics.lifetimes),
            type_params: self.trans_type_params(&generics.ty_params),
            wh: self.trans_where(&generics.where_clause.predicates),
        }
    }

    #[inline]
    fn trans_lifetime_defs(&mut self, lifetime_defs: &Vec<rst::LifetimeDef>) -> Vec<LifetimeDef> {
        trans_list!(self, lifetime_defs, trans_lifetime_def)
    }

    fn trans_lifetime_def(&mut self, lifetime_def: &rst::LifetimeDef) -> LifetimeDef {
        let lifetime = self.trans_lifetime(&lifetime_def.lifetime);
        LifetimeDef {
            loc: lifetime.loc,
            lifetime: lifetime,
            bounds: self.trans_lifetimes(&lifetime_def.bounds),
        }
    }

    #[inline]
    fn trans_lifetimes(&mut self, lifetimes: &Vec<rst::Lifetime>) -> Vec<Lifetime> {
        trans_list!(self, lifetimes, trans_lifetime)
    }

    fn trans_lifetime(&mut self, lifetime: &rst::Lifetime) -> Lifetime {
        Lifetime {
            loc: self.leaf_loc(&lifetime.span),
            s: name_to_string(&lifetime.name),
        }
    }

    #[inline]
    fn trans_type_params(&mut self, type_params: &[rst::TyParam]) -> Vec<TypeParam> {
        trans_list!(self, type_params, trans_type_param)
    }

    fn trans_type_param(&mut self, type_param: &rst::TyParam) -> TypeParam {
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
    fn trans_type_param_bounds(&mut self, bounds: &[rst::TyParamBound]) -> Vec<TypeParamBound> {
        trans_list!(self, bounds, trans_type_param_bound)
    }

    fn trans_type_param_bound(&mut self, bound: &rst::TyParamBound) -> TypeParamBound {
        match *bound {
            rst::RegionTyParamBound(ref lifetime) => {
                TypeParamBound::Lifetime(self.trans_lifetime(lifetime))
            }
            rst::TraitTyParamBound(ref poly_trait_ref, modifier) => {
                TypeParamBound::PolyTraitRef(self.trans_poly_trait_ref(poly_trait_ref, modifier))
            }
        }
    }

    fn trans_poly_trait_ref(&mut self, poly_trait_ref: &rst::PolyTraitRef,
                            modifier: rst::TraitBoundModifier)
        -> PolyTraitRef {
        if is_sized(modifier) {
            return PolyTraitRef::new_sized(self.leaf_loc(&poly_trait_ref.span));
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

    fn trans_trait_ref(&mut self, trait_ref: &rst::TraitRef) -> TraitRef {
        self.trans_path(&trait_ref.path)
    }

    #[inline]
    fn trans_where(&mut self, predicates: &Vec<rst::WherePredicate>) -> Where {
        Where {
            clauses: self.trans_where_clauses(predicates),
        }
    }

    #[inline]
    fn trans_where_clauses(&mut self, predicates: &Vec<rst::WherePredicate>) -> Vec<WhereClause> {
        trans_list!(self, predicates, trans_where_clause)
    }

    fn trans_where_clause(&mut self, predicate: &rst::WherePredicate) -> WhereClause {
        match *predicate {
            rst::WherePredicate::RegionPredicate(ref region) => self.trans_where_lifetime(region),
            rst::WherePredicate::BoundPredicate(ref bound) => self.trans_where_bound(bound),
            _ => unreachable!(),
        }
    }

    fn trans_where_lifetime(&mut self, region: &rst::WhereRegionPredicate) -> WhereClause {
        let loc = self.loc(&region.span);
        let lifetime = self.trans_lifetime(&region.lifetime);
        let bounds = self.trans_lifetimes(&region.bounds);
        self.set_loc(&loc);

        WhereClause {
            loc: loc,
            clause: WhereKind::LifetimeDef(LifetimeDef {
                loc: lifetime.loc,
                lifetime: lifetime,
                bounds: bounds,
            }),
        }
    }

    fn trans_where_bound(&mut self, bound: &rst::WhereBoundPredicate) -> WhereClause {
        let loc = self.loc(&bound.span);
        let lifetime_defs = self.trans_lifetime_defs(&bound.bound_lifetimes);
        let ty = self.trans_type(&bound.bounded_ty);
        let bounds = self.trans_type_param_bounds(&bound.bounds);
        self.set_loc(&loc);

        WhereClause {
            loc: loc,
            clause: WhereKind::Bound(WhereBound {
                lifetime_defs: lifetime_defs,
                ty: ty,
                bounds: bounds,
            }),
        }
    }

    fn trans_path(&mut self, path: &rst::Path) -> Path {
        let loc = self.loc(&path.span);
        let segs = self.trans_path_segments(&path.segments);
        self.set_loc(&loc);

        Path {
            loc: loc,
            global: path.global,
            segs: segs,
        }
    }

    #[inline]
    fn trans_path_segments(&mut self, segs: &Vec<rst::PathSegment>) -> Vec<PathSegment> {
        trans_list!(self, segs, trans_path_segment)
    }

    fn trans_path_segment(&mut self, seg: &rst::PathSegment) -> PathSegment {
        PathSegment {
            name: ident_to_string(&seg.identifier),
            param: self.trans_path_param(&seg.parameters),
        }
    }

    fn trans_path_param(&mut self, params: &rst::PathParameters) -> PathParam {
        match *params {
            rst::AngleBracketed(ref param) => PathParam::Angle(self.trans_angle_param(param)),
            rst::Parenthesized(ref param) => PathParam::Paren(self.trans_paren_param(param)),
        }
    }

    fn trans_angle_param(&mut self, param: &rst::AngleBracketedParameterData) -> AngleParam {
        AngleParam {
            lifetimes: self.trans_lifetimes(&param.lifetimes),
            types: self.trans_types(&param.types),
            bindings: self.trans_type_bindings(&param.bindings),
        }
    }

    #[inline]
    fn trans_type_bindings(&mut self, bindings: &[rst::P<rst::TypeBinding>]) -> Vec<TypeBinding> {
        trans_list!(self, bindings, trans_type_binding)
    }

    fn trans_type_binding(&mut self, binding: &rst::TypeBinding) -> TypeBinding {
        TypeBinding {
            loc: self.leaf_loc(&binding.span),
            name: ident_to_string(&binding.ident),
            ty: self.trans_type(&binding.ty),
        }
    }

    fn trans_paren_param(&mut self, param: &rst::ParenthesizedParameterData) -> ParenParam {
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

    fn trans_qself(&mut self, qself: &rst::QSelf) -> QSelf {
        QSelf {
            ty: self.trans_type(&qself.ty),
            pos: qself.position,
        }
    }

    #[inline]
    fn trans_types(&mut self, types: &[rst::P<rst::Ty>]) -> Vec<Type> {
        trans_list!(self, types, trans_type)
    }

    fn trans_type(&mut self, ty: &rst::Ty) -> Type {
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

    fn trans_path_type(&mut self, qself: &Option<rst::QSelf>, path: &rst::Path) -> PathType {
        let qself = match *qself {
            Some(ref qself) => Some(self.trans_qself(&qself)),
            None => None,
        };
        let path = self.trans_path(path);

        PathType {
            qself: qself,
            path: path,
        }
    }

    fn trans_ptr_type(&mut self, mut_type: &rst::MutTy) -> PtrType {
        PtrType {
            is_mut: is_mut(mut_type.mutbl),
            ty: self.trans_type(&mut_type.ty),
        }
    }

    fn trans_ref_type(&mut self, lifetime: &Option<rst::Lifetime>, mut_type: &rst::MutTy) -> RefType {
        let lifetime = match *lifetime {
            Some(ref lifetime) => Some(self.trans_lifetime(lifetime)),
            None => None,
        };
        let is_mut = is_mut(mut_type.mutbl);
        let ty = self.trans_type(&mut_type.ty);

        RefType {
            lifetime: lifetime,
            is_mut: is_mut,
            ty: ty,
        }
    }

    fn trans_array_type(&mut self, ty: &rst::Ty) -> ArrayType {
        ArrayType {
            ty: self.trans_type(ty),
        }
    }

    fn trans_fixed_size_array_type(&mut self, ty: &rst::Ty, expr: &rst::Expr) -> FixedSizeArrayType {
        FixedSizeArrayType {
            ty: self.trans_type(ty),
            expr: self.trans_expr(expr),
        }
    }

    fn trans_tuple_type(&mut self, types: &Vec<rst::P<rst::Ty>>) -> TupleType {
        TupleType {
            types: self.trans_types(types),
        }
    }

    fn trans_bare_fn_type(&mut self, bare_fn: &rst::BareFnTy) -> BareFnType {
        BareFnType {
            lifetime_defs: self.trans_lifetime_defs(&bare_fn.lifetimes),
            is_unsafe: is_unsafe(bare_fn.unsafety),
            abi: abi_to_string(bare_fn.abi),
            fn_sig: self.trans_fn_sig(&bare_fn.decl),
        }
    }

    fn trans_sum_type(&mut self, ty: &rst::Ty, bounds: &rst::TyParamBounds) -> SumType {
        SumType {
            ty: self.trans_type(ty),
            bounds: self.trans_type_param_bounds(bounds),
        }
    }

    fn trans_poly_trait_ref_type(&mut self, bounds: &rst::TyParamBounds) -> PolyTraitRefType {
        PolyTraitRefType {
            bounds: self.trans_type_param_bounds(bounds),
        }
    }

    fn trans_foreign_mod(&mut self, module: &rst::ForeignMod) -> ForeignMod {
        ForeignMod {
            abi: abi_to_string(module.abi),
            items: self.trans_foreign_items(&module.items),
        }
    }

    #[inline]
    fn trans_foreign_items(&mut self, items: &Vec<rst::P<rst::ForeignItem>>) -> Vec<ForeignItem> {
        trans_list!(self, items, trans_foreign_item)
    }

    fn trans_foreign_item(&mut self, item: &rst::ForeignItem) -> ForeignItem {
        let loc = self.loc(&item.span);
        let attrs = self.trans_attrs(&item.attrs);

        let is_pub = is_pub(item.vis);
        let ident = ident_to_string(&item.ident);
        let item = match item.node {
            rst::ForeignItemStatic(ref ty, is_mut) => {
                ForeignKind::Static(self.trans_foreign_static(is_mut, ident, ty))
            }
            rst::ForeignItemFn(ref fn_decl, ref generics) => {
                ForeignKind::Fn(self.trans_foreign_fn(ident, generics, fn_decl))
            }
        };

        self.set_loc(&loc);
        ForeignItem {
            loc: loc,
            attrs: attrs,
            is_pub: is_pub,
            item: item,
        }
    }

    fn trans_foreign_static(&mut self, is_mut: bool, ident: String, ty: &rst::Ty) -> ForeignStatic {
        ForeignStatic {
            is_mut: is_mut,
            name: ident,
            ty: self.trans_type(ty),
        }
    }

    fn trans_foreign_fn(&mut self, ident: String, generics: &rst::Generics, fn_decl: &rst::FnDecl)
        -> ForeignFn {
        ForeignFn {
            name: ident,
            generics: self.trans_generics(generics),
            fn_sig: self.trans_fn_sig(fn_decl),
        }
    }

    fn trans_const(&mut self, ident: String, ty: &rst::Ty, expr: &rst::Expr) -> Const {
        Const {
            name: ident,
            ty: self.trans_type(ty),
            expr: self.trans_expr(expr),
        }
    }

    fn trans_static(&mut self, is_mut: bool, ident: String, ty: &rst::Ty, expr: &rst::Expr) -> Static {
        Static {
            is_mut: is_mut,
            name: ident,
            ty: self.trans_type(ty),
            expr: self.trans_expr(expr),
        }
    }

    fn trans_struct(&mut self, ident: String, generics: &rst::Generics, variant: &rst::VariantData)
        -> Struct {
        Struct {
            name: ident,
            generics: self.trans_generics(generics),
            body: self.trans_struct_body(variant),
        }
    }

    fn trans_struct_body(&mut self, variant: &rst::VariantData) -> StructBody {
        match *variant {
            rst::VariantData::Struct(ref fields, _) => {
                StructBody::Struct(self.trans_struct_fields(fields))
            }
            rst::VariantData::Tuple(ref fields, _) => {
                StructBody::Tuple(self.trans_tuple_fields(fields))
            }
            rst::VariantData::Unit(_) => StructBody::Unit,
        }
    }

    #[inline]
    fn trans_struct_fields(&mut self, fields: &Vec<rst::StructField>) -> Vec<StructField> {
        trans_list!(self, fields, trans_struct_field)
    }

    fn trans_struct_field(&mut self, field: &rst::StructField) -> StructField {
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
            is_pub: is_pub,
            name: name,
            ty: ty,
        }
    }

    #[inline]
    fn trans_tuple_fields(&mut self, fields: &Vec<rst::StructField>) -> Vec<TupleField> {
        trans_list!(self, fields, trans_tuple_field)
    }

    fn trans_tuple_field(&mut self, field: &rst::StructField) -> TupleField {
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
            is_pub: is_pub,
            ty: ty,
        }
    }

    fn trans_enum(&mut self, ident: String, generics: &rst::Generics, enum_def: &rst::EnumDef)
        -> Enum {
        Enum {
            name: ident,
            generics: self.trans_generics(generics),
            body: self.trans_enum_body(enum_def),
        }
    }

    fn trans_enum_body(&mut self, enum_def: &rst::EnumDef) -> EnumBody {
        EnumBody {
            fields: self.trans_enum_fields(&enum_def.variants),
        }
    }

    #[inline]
    fn trans_enum_fields(&mut self, variants: &Vec<rst::P<rst::Variant>>) -> Vec<EnumField> {
        trans_list!(self, variants, trans_enum_field)
    }

    fn trans_enum_field(&mut self, variant: &rst::Variant) -> EnumField {
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

    fn trans_fn(&mut self, is_unsafe: bool, is_const: bool, abi: String, ident: String,
                generics: &rst::Generics, fn_decl: &rst::FnDecl, block: &rst::Block)
        -> Fn {
        Fn {
            is_unsafe: is_unsafe,
            is_const: is_const,
            abi: abi,
            name: ident,
            generics: self.trans_generics(generics),
            fn_sig: self.trans_fn_sig(fn_decl),
            block: self.trans_block(block),
        }
    }

    fn trans_trait(&mut self, is_unsafe: bool, ident: String, generics: &rst::Generics,
                   bounds: &rst::TyParamBounds, items: &Vec<rst::P<rst::TraitItem>>)
        -> Trait {
        Trait {
            is_unsafe: is_unsafe,
            name: ident,
            generics: self.trans_generics(generics),
            bounds: self.trans_type_param_bounds(bounds),
            items: self.trans_trait_items(items),
        }
    }

    #[inline]
    fn trans_trait_items(&mut self, items: &Vec<rst::P<rst::TraitItem>>) -> Vec<TraitItem> {
        trans_list!(self, items, trans_trait_item)
    }

    fn trans_trait_item(&mut self, item: &rst::TraitItem) -> TraitItem {
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

    fn trans_const_trait_item(&mut self, ident: String, ty: &rst::Ty,
                              expr: &Option<rst::P<rst::Expr>>)
        -> ConstTraitItem {
        let expr = match *expr {
            Some(ref expr) => Some(self.trans_expr(expr)),
            None => None,
        };

        ConstTraitItem {
            name: ident,
            ty: self.trans_type(ty),
            expr: expr,
        }
    }

    fn trans_type_trait_item(&mut self, ident: String, bounds: &rst::TyParamBounds,
                             ty: &Option<rst::P<rst::Ty>>)
        -> TypeTraitItem {
        let ty = match *ty {
            Some(ref ty) => Some(self.trans_type(ty)),
            None => None,
        };

        TypeTraitItem {
            name: ident,
            bounds: self.trans_type_param_bounds(bounds),
            ty: ty,
        }
    }

    fn trans_method_trait_item(&mut self, ident: String, method_sig: &rst::MethodSig,
                               block: &Option<rst::P<rst::Block>>)
        -> MethodTraitItem {
        let block = match *block {
            Some(ref block) => Some(self.trans_block(block)),
            None => None,
        };

        MethodTraitItem {
            is_unsafe: is_unsafe(method_sig.unsafety),
            is_const: is_const(method_sig.constness),
            abi: abi_to_string(method_sig.abi),
            name: ident,
            method_sig: self.trans_method_sig(method_sig),
            block: block,
        }
    }

    fn trans_method_sig(&mut self, method_sig: &rst::MethodSig) -> MethodSig {
        MethodSig {
            generics: self.trans_generics(&method_sig.generics),
            fn_sig: self.trans_fn_sig(&method_sig.decl),
            sf: self.trans_self(&method_sig.explicit_self.node),
        }
    }

    fn trans_self(&mut self, explicit_self: &rst::ExplicitSelf_) -> Option<Sf> {
        match *explicit_self {
            rst::SelfStatic => None,
            rst::SelfValue(_) => Some(Sf::String("self".to_string())),
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
                Some(Sf::String(s))
            }
            rst::SelfExplicit(ref ty, _) => Some(Sf::Type(self.trans_type(ty))),
        }
    }

    fn trans_impl_default(&mut self, is_unsafe: bool, trait_ref: &rst::TraitRef) -> ImplDefault {
        ImplDefault {
            is_unsafe: is_unsafe,
            trait_ref: self.trans_trait_ref(trait_ref),
        }
    }

    fn trans_impl(&mut self, is_unsafe: bool, is_neg: bool, generics: &rst::Generics,
                  trait_ref: &Option<rst::TraitRef>, ty: &rst::Ty,
                  items: &Vec<rst::P<rst::ImplItem>>)
        -> Impl {
        let trait_ref = match *trait_ref {
            Some(ref trait_ref) => Some(self.trans_trait_ref(trait_ref)),
            None => None,
        };

        Impl {
            is_unsafe: is_unsafe,
            is_neg: is_neg,
            generics: self.trans_generics(generics),
            trait_ref: trait_ref,
            ty: self.trans_type(ty),
            items: self.trans_impl_items(items),
        }
    }

    #[inline]
    fn trans_impl_items(&mut self, items: &Vec<rst::P<rst::ImplItem>>) -> Vec<ImplItem> {
        trans_list!(self, items, trans_impl_item)
    }

    fn trans_impl_item(&mut self, item: &rst::ImplItem) -> ImplItem {
        let loc = self.loc(&item.span);
        let attrs = self.trans_attrs(&item.attrs);

        let is_pub = is_pub(item.vis);
        let ident = ident_to_string(&item.ident);
        let item = match item.node {
            rst::ImplItemKind::Const(ref ty, ref expr) => {
                ImplItemKind::Const(self.trans_const_impl_item(ident, ty, expr))
            }
            rst::ImplItemKind::Type(ref ty) => {
                ImplItemKind::Type(self.trans_type_impl_item(ident, ty))
            }
            rst::ImplItemKind::Method(ref method_sig, ref block) => {
                ImplItemKind::Method(self.trans_method_impl_item(ident, method_sig, block))
            }
            rst::ImplItemKind::Macro(ref mac) => {
                ImplItemKind::Macro(self.trans_macro_impl_item(mac))
            }
        };
        self.set_loc(&loc);

        ImplItem {
            loc: loc,
            attrs: attrs,
            is_pub: is_pub,
            item: item,
        }
    }

    fn trans_const_impl_item(&mut self, ident: String, ty: &rst::Ty, expr: &rst::Expr)
        -> ConstImplItem {
        ConstImplItem {
            name: ident,
            ty: self.trans_type(ty),
            expr: self.trans_expr(expr),
        }
    }

    fn trans_type_impl_item(&mut self, ident: String, ty: &rst::Ty) -> TypeImplItem {
        TypeImplItem {
            name: ident,
            ty: self.trans_type(ty),
        }
    }

    fn trans_method_impl_item(&mut self, ident: String, method_sig: &rst::MethodSig,
                              block: &rst::P<rst::Block>)
        -> MethodImplItem {
        MethodImplItem {
            is_unsafe: is_unsafe(method_sig.unsafety),
            is_const: is_const(method_sig.constness),
            abi: abi_to_string(method_sig.abi),
            name: ident,
            method_sig: self.trans_method_sig(method_sig),
            block: self.trans_block(block),
        }
    }

    fn trans_fn_sig(&mut self, fn_decl: &rst::FnDecl) -> FnSig {
        FnSig {
            arg: self.trans_fn_arg(&fn_decl.inputs, fn_decl.variadic),
            ret: self.trans_fn_return(&fn_decl.output),
        }
    }

    fn trans_fn_arg(&mut self, inputs: &Vec<rst::Arg>, variadic: bool) -> FnArg {
        FnArg {
            args: self.trans_args(inputs),
            va: variadic,
        }
    }

    #[inline]
    fn trans_args(&mut self, inputs: &Vec<rst::Arg>) -> Vec<Arg> {
        trans_list!(self, inputs, trans_arg)
    }

    fn trans_arg(&mut self, arg: &rst::Arg) -> Arg {
        let pat = self.trans_patten(&arg.pat);
        Arg {
            loc: pat.loc.clone(),
            pat: pat,
            ty: self.trans_type(&arg.ty),
        }
    }

    fn trans_fn_return(&mut self, output: &rst::FunctionRetTy) -> FnReturn {
        match *output {
            rst::FunctionRetTy::DefaultReturn(_) => FnReturn::Unit,
            rst::FunctionRetTy::NoReturn(_) => FnReturn::Diverge,
            rst::FunctionRetTy::Return(ref ty) => FnReturn::Normal(self.trans_type(ty)),
        }
    }

    fn trans_block(&mut self, block: &rst::Block) -> Block {
        let loc = self.loc(&block.span);
        let mut stmts = self.trans_stmts(&block.stmts);
        if let Some(ref expr) = block.expr {
            let expr = self.trans_expr(expr);
            stmts.push(self.expr_to_stmt(expr));
        }
        self.set_loc(&loc);

        Block {
            loc: loc,
            is_unsafe: is_block_unsafe(block.rules),
            stmts: stmts,
        }
    }

    #[inline]
    fn trans_stmts(&mut self, stmts: &Vec<rst::P<rst::Stmt>>) -> Vec<Stmt> {
        trans_list!(self, stmts, trans_stmt)
    }

    #[inline]
    fn trans_stmt(&mut self, stmt: &rst::P<rst::Stmt>) -> Stmt {
        let loc = self.loc(&stmt.span);
        let stmt = match stmt.node {
            rst::StmtDecl(ref decl, _) => StmtKind::Decl(self.trans_decl(decl)),
            rst::StmtSemi(ref expr, _) => StmtKind::Expr(self.trans_expr(expr), true),
            rst::StmtExpr(ref expr, _) => StmtKind::Expr(self.trans_expr(expr), false),
            rst::StmtMac(ref mac, _, _) => StmtKind::Macro(self.trans_macro(mac)),
        };
        self.set_loc(&loc);

        Stmt {
            loc: loc,
            stmt: stmt,
        }
    }

    fn trans_decl(&mut self, decl: &rst::Decl) -> Decl {
        let loc = self.loc(&decl.span);
        let decl = match decl.node {
            rst::DeclLocal(ref local) => DeclKind::Local(self.trans_local(local)),
            rst::DeclItem(ref item) => DeclKind::Item(self.trans_item(item)),
        };
        self.set_loc(&loc);

        Decl {
            loc: loc,
            decl: decl,
        }
    }

    fn trans_local(&mut self, local: &rst::Local) -> Local {
        let loc = self.loc(&local.span);
        let attrs = self.trans_thin_attrs(&local.attrs);
        let pat = self.trans_patten(&local.pat);
        let ty = match local.ty {
            Some(ref ty) => Some(self.trans_type(ty)),
            None => None,
        };
        let init = match local.init {
            Some(ref init) => Some(self.trans_expr(init)),
            None => None,
        };

        Local {
            loc: loc,
            attrs: attrs,
            pat: pat,
            ty: ty,
            init: init,
        }
    }

    fn trans_thin_attrs(&mut self, attrs: &rst::ThinAttributes) -> Vec<AttrKind> {
        match *attrs {
            Some(ref attrs) => self.trans_attrs(attrs),
            None => Vec::new(),
        }
    }

    #[inline]
    fn trans_pattens(&mut self, pats: &Vec<rst::P<rst::Pat>>) -> Vec<Patten> {
        trans_list!(self, pats, trans_patten)
    }

    fn trans_patten(&mut self, pat: &rst::P<rst::Pat>) -> Patten {
        let loc = self.loc(&pat.span);
        let pat = match pat.node {
            rst::PatWild => PattenKind::Wildcard,
            rst::PatLit(ref expr) => PattenKind::Literal(self.trans_expr(expr)),
            rst::PatRange(ref start, ref end) => {
                PattenKind::Range(self.trans_range_patten(start, end))
            }
            rst::PatIdent(mode, ref ident, ref binding) => {
                PattenKind::Ident(Box::new(self.trans_ident_patten(mode, ident, binding)))
            }
            rst::PatRegion(ref pat, mutbl) => {
                PattenKind::Ref(Box::new(self.trans_ref_patten(is_mut(mutbl), pat)))
            }
            rst::PatQPath(ref qself, ref path) => {
                PattenKind::Path(self.trans_path_patten(qself, path))
            }
            rst::PatEnum(ref path, ref pats) => {
                PattenKind::Enum(self.trans_enum_patten(path, pats))
            }
            rst::PatStruct(ref path, ref fields, etc) => {
                PattenKind::Struct(Box::new(self.trans_struct_patten(path, fields, etc)))
            }
            rst::PatVec(ref start, ref emit, ref end) => {
                PattenKind::Vec(Box::new(self.trans_vec_patten(start, emit, end)))
            }
            rst::PatTup(ref pats) => PattenKind::Tuple(Box::new(self.trans_tuple_patten(pats))),
            rst::PatBox(ref pat) => PattenKind::Box(Box::new(self.trans_patten(pat))),
            rst::PatMac(ref mac) => PattenKind::Macro(self.trans_macro(mac)),
        };
        self.set_loc(&loc);

        Patten {
            loc: loc,
            pat: pat,
        }
    }

    fn trans_range_patten(&mut self, start: &rst::Expr, end: &rst::Expr) -> RangePatten {
        RangePatten {
            start: self.trans_expr(start),
            end: self.trans_expr(end),
        }
    }

    fn trans_ident_patten(&mut self, mode: rst::BindingMode, ident: &rst::SpannedIdent,
                          binding: &Option<rst::P<rst::Pat>>)
        -> IdentPatten {
        let (is_ref, is_mut) = is_ref_mut(mode);
        IdentPatten {
            is_ref: is_ref,
            is_mut: is_mut,
            name: self.trans_ident(ident),
            binding: zopt::map_ref_mut(binding, |pat| self.trans_patten(pat)),
        }
    }

    fn trans_ref_patten(&mut self, is_mut: bool, pat: &rst::P<rst::Pat>) -> RefPatten {
        RefPatten {
            is_mut: is_mut,
            pat: self.trans_patten(pat),
        }
    }

    fn trans_path_patten(&mut self, qself: &rst::QSelf, path: &rst::Path) -> PathPatten {
        PathPatten {
            qself: self.trans_qself(&qself),
            path: self.trans_path(path),
        }
    }

    fn trans_enum_patten(&mut self, path: &rst::Path, pats: &Option<Vec<rst::P<rst::Pat>>>)
        -> EnumPatten {
        EnumPatten {
            path: self.trans_path(path),
            pats: zopt::map_ref_mut(pats, |pats| self.trans_pattens(pats)),
        }
    }

    fn trans_struct_patten(&mut self, path: &rst::Path, fields: &Vec<rst::Spanned<rst::FieldPat>>,
                           etc: bool)
        -> StructPatten {
        StructPatten {
            path: self.trans_path(path),
            fields: self.trans_struct_field_pattens(fields),
            etc: etc,
        }
    }

    #[inline]
    fn trans_struct_field_pattens(&mut self, fields: &Vec<rst::Spanned<rst::FieldPat>>)
        -> Vec<StructFieldPatten> {
        trans_list!(self, fields, trans_struct_field_patten)
    }

    fn trans_struct_field_patten(&mut self, field: &rst::Spanned<rst::FieldPat>) -> StructFieldPatten {
        let loc = self.loc(&field.span);
        let name = ident_to_string(&field.node.ident);
        let pat = self.trans_patten(&field.node.pat);
        let shorthand = field.node.is_shorthand;
        self.set_loc(&loc);

        StructFieldPatten {
            loc: loc,
            name: name,
            pat: pat,
            shorthand: shorthand,
        }
    }

    fn trans_vec_patten(&mut self, start: &Vec<rst::P<rst::Pat>>, emit: &Option<rst::P<rst::Pat>>,
                        end: &Vec<rst::P<rst::Pat>>)
        -> VecPatten {
        VecPatten {
            start: self.trans_pattens(start),
            emit: zopt::map_ref_mut(emit, |pat| self.trans_patten(pat)),
            end: self.trans_pattens(end),
        }
    }

    fn trans_tuple_patten(&mut self, pats: &Vec<rst::P<rst::Pat>>) -> TuplePatten {
        TuplePatten {
            pats: self.trans_pattens(pats),
        }
    }

    fn expr_to_stmt(&mut self, expr: Expr) -> Stmt {
        Stmt {
            loc: Default::default(),
            stmt: StmtKind::Expr(expr, false),
        }
    }

    #[inline]
    fn trans_exprs(&mut self, exprs: &[rst::P<rst::Expr>]) -> Vec<Expr> {
        trans_list!(self, exprs, trans_expr)
    }

    fn trans_expr(&mut self, expr: &rst::Expr) -> Expr {
        let loc = self.loc(&expr.span);
        let attrs = self.trans_thin_attrs(&expr.attrs);
        let expr = match expr.node {
            rst::ExprLit(ref lit) => ExprKind::Literal(self.trans_literal_expr(lit)),
            rst::ExprPath(ref qself, ref path) => ExprKind::Path(self.trans_path_type(qself, path)),
            rst::ExprUnary(op, ref expr) => {
                ExprKind::Unary(Box::new(self.trans_unary_expr(op, expr)))
            }
            rst::ExprAddrOf(mutble, ref expr) => {
                ExprKind::Ref(Box::new(self.trans_ref_expr(mutble, expr)))
            }
            rst::ExprBinary(ref op, ref left, ref right) => {
                ExprKind::List(Box::new(self.trans_binary_expr(left, op, right)))
            }
            rst::ExprAssign(ref left, ref right) => {
                ExprKind::List(Box::new(self.trans_assign_expr(left, right)))
            }
            rst::ExprAssignOp(ref op, ref left, ref right) => {
                ExprKind::List(Box::new(self.trans_op_assign_expr(left, op, right)))
            }
            rst::ExprInPlace(ref left, ref right) => {
                ExprKind::List(Box::new(self.trans_in_place_expr(left, right)))
            }
            rst::ExprRepeat(ref init, ref len) => {
                ExprKind::FixedSizeArray(Box::new(self.trans_fixed_size_array_expr(init, len)))
            }
            rst::ExprVec(ref exprs) => ExprKind::Vec(Box::new(self.trans_exprs(exprs))),
            rst::ExprTup(ref exprs) => ExprKind::Tuple(Box::new(self.trans_exprs(exprs))),
            rst::ExprParen(ref expr) => ExprKind::Tuple(Box::new(vec![self.trans_expr(expr)])),
            rst::ExprField(ref expr, ref ident) => {
                ExprKind::FieldAccess(Box::new(self.trans_struct_field_access_expr(expr, ident)))
            }
            rst::ExprTupField(ref expr, ref pos) => {
                ExprKind::FieldAccess(Box::new(self.trans_tuple_field_access_expr(expr, pos)))
            }
            rst::ExprStruct(ref path, ref fields, ref base) => {
                ExprKind::Struct(Box::new(self.trans_struct_expr(path, fields, base)))
            }
            rst::ExprIndex(ref obj, ref index) => {
                ExprKind::Index(Box::new(self.trans_index_expr(obj, index)))
            }
            rst::ExprRange(ref start, ref end) => {
                ExprKind::Range(Box::new(self.trans_range_expr(start, end)))
            }
            rst::ExprBox(ref expr) => ExprKind::Box(Box::new(self.trans_box_expr(expr))),
            rst::ExprCast(ref expr, ref ty) => {
                ExprKind::Cast(Box::new(self.trans_cast_expr(expr, ty)))
            }
            rst::ExprType(ref expr, ref ty) => {
                ExprKind::Type(Box::new(self.trans_type_expr(expr, ty)))
            }
            rst::ExprBlock(ref block) => ExprKind::Block(Box::new(self.trans_block(block))),
            rst::ExprIf(ref expr, ref block, ref left) => {
                ExprKind::If(Box::new(self.trans_if_expr(expr, block, left)))
            }
            rst::ExprIfLet(ref pat, ref expr, ref block, ref left) => {
                ExprKind::IfLet(Box::new(self.trans_if_let_expr(pat, expr, block, left)))
            }
            rst::ExprWhile(ref expr, ref block, ref label) => {
                ExprKind::While(Box::new(self.trans_while_expr(expr, block, label)))
            }
            rst::ExprWhileLet(ref pat, ref expr, ref block, ref label) => {
                ExprKind::WhileLet(Box::new(self.trans_while_let_expr(pat, expr, block, label)))
            }
            rst::ExprForLoop(ref pat, ref expr, ref block, ref label) => {
                ExprKind::For(Box::new(self.trans_for_expr(pat, expr, block, label)))
            }
            rst::ExprLoop(ref block, ref label) => {
                ExprKind::Loop(Box::new(self.trans_loop_expr(block, label)))
            }
            rst::ExprBreak(ref ident) => ExprKind::Break(Box::new(self.trans_break_expr(ident))),
            rst::ExprAgain(ref ident) => {
                ExprKind::Continue(Box::new(self.trans_continue_expr(ident)))
            }
            rst::ExprMatch(ref expr, ref arms) => {
                ExprKind::Match(Box::new(self.trans_match_expr(expr, arms)))
            }
            rst::ExprCall(ref fn_name, ref args) => {
                ExprKind::FnCall(Box::new(self.trans_fn_call_expr(fn_name, args)))
            }
            rst::ExprMethodCall(ref ident, ref types, ref args) => {
                ExprKind::MethodCall(Box::new(self.trans_method_call_expr(ident, types, args)))
            }
            rst::ExprClosure(capture, ref fn_decl, ref block) => {
                ExprKind::Closure(Box::new(self.trans_closure_expr(capture, fn_decl, block)))
            }
            rst::ExprRet(ref expr) => ExprKind::Return(Box::new(self.trans_return_expr(expr))),
            rst::ExprMac(ref mac) => ExprKind::Macro(self.trans_macro_expr(mac)),

            // ExprInlineAsm
            _ => unreachable!(),
        };
        self.set_loc(&loc);

        Expr {
            loc: loc,
            attrs: attrs,
            expr: expr,
        }
    }

    fn trans_ident(&mut self, ident: &rst::SpannedIdent) -> Chunk {
        Chunk {
            loc: self.leaf_loc(&ident.span),
            s: ident_to_string(&ident.node),
        }
    }

    fn trans_pos(&mut self, pos: &rst::Spanned<usize>) -> Chunk {
        Chunk {
            loc: self.leaf_loc(&pos.span),
            s: pos.node.to_string(),
        }
    }

    fn trans_literal_expr(&mut self, lit: &rst::Lit) -> Chunk {
        Chunk {
            loc: self.leaf_loc(&lit.span),
            s: self.literal_to_string(lit),
        }
    }

    fn trans_uop(&mut self, op: rst::UnOp) -> Chunk {
        Chunk::new(uop_to_string(op))
    }

    fn trans_unary_expr(&mut self, op: rst::UnOp, expr: &rst::Expr) -> UnaryExpr {
        UnaryExpr {
            op: self.trans_uop(op),
            expr: self.trans_expr(expr),
        }
    }

    fn trans_ref_expr(&mut self, mutble: rst::Mutability, expr: &rst::Expr) -> RefExpr {
        RefExpr {
            is_mut: is_mut(mutble),
            expr: self.trans_expr(expr),
        }
    }

    fn trans_bop(&mut self, op: &rst::BinOp) -> Chunk {
        Chunk {
            loc: self.leaf_loc(&op.span),
            s: op.node.to_string().to_string(),
        }
    }

    fn trans_bop_assign(&mut self, op: &rst::BinOp) -> Chunk {
        Chunk {
            loc: self.leaf_loc(&op.span),
            s: format!("{}=", op.node.to_string()),
        }
    }

    fn trans_binary_expr(&mut self, left: &rst::Expr, op: &rst::BinOp, right: &rst::Expr) -> ListExpr {
        ListExpr {
            exprs: vec![self.trans_expr(left), self.trans_expr(right)],
            sep: self.trans_bop(op),
        }
    }

    fn trans_assign_expr(&mut self, left: &rst::Expr, right: &rst::Expr) -> ListExpr {
        ListExpr {
            exprs: vec![self.trans_expr(left), self.trans_expr(right)],
            sep: Chunk::new("="),
        }
    }

    fn trans_op_assign_expr(&mut self, left: &rst::Expr, op: &rst::BinOp, right: &rst::Expr)
        -> ListExpr {
        ListExpr {
            exprs: vec![self.trans_expr(left), self.trans_expr(right)],
            sep: self.trans_bop_assign(op),
        }
    }

    fn trans_in_place_expr(&mut self, left: &rst::Expr, right: &rst::Expr) -> ListExpr {
        ListExpr {
            exprs: vec![self.trans_expr(left), self.trans_expr(right)],
            sep: Chunk::new("<-"),
        }
    }

    fn trans_fixed_size_array_expr(&mut self, init: &rst::Expr, len: &rst::Expr)
        -> FixedSizeArrayExpr {
        FixedSizeArrayExpr {
            init: self.trans_expr(init),
            len: self.trans_expr(len),
        }
    }

    fn trans_struct_field_access_expr(&mut self, expr: &rst::Expr, ident: &rst::SpannedIdent)
        -> FieldAccessExpr {
        FieldAccessExpr {
            expr: self.trans_expr(expr),
            field: self.trans_ident(ident),
        }
    }

    fn trans_tuple_field_access_expr(&mut self, expr: &rst::Expr, pos: &rst::Spanned<usize>)
        -> FieldAccessExpr {
        FieldAccessExpr {
            expr: self.trans_expr(expr),
            field: self.trans_pos(pos),
        }
    }

    fn trans_struct_expr(&mut self, path: &rst::Path, fields: &Vec<rst::Field>,
                         base: &Option<rst::P<rst::Expr>>)
        -> StructExpr {
        StructExpr {
            path: self.trans_path(path),
            fields: self.trans_struct_field_exprs(fields),
            base: zopt::map_ref_mut(base, |expr| self.trans_expr(expr)),
        }
    }

    #[inline]
    fn trans_struct_field_exprs(&mut self, fields: &Vec<rst::Field>) -> Vec<StructFieldExpr> {
        trans_list!(self, fields, trans_struct_field_expr)
    }

    fn trans_struct_field_expr(&mut self, field: &rst::Field) -> StructFieldExpr {
        let loc = self.loc(&field.span);
        let name = self.trans_ident(&field.ident);
        let value = self.trans_expr(&field.expr);
        self.set_loc(&loc);

        StructFieldExpr {
            loc: loc,
            name: name,
            value: value,
        }
    }

    fn trans_index_expr(&mut self, obj: &rst::Expr, index: &rst::Expr) -> IndexExpr {
        IndexExpr {
            obj: self.trans_expr(obj),
            index: self.trans_expr(index),
        }
    }

    fn trans_range_expr(&mut self, start: &Option<rst::P<rst::Expr>>,
                        end: &Option<rst::P<rst::Expr>>)
        -> RangeExpr {
        RangeExpr {
            start: zopt::map_ref_mut(start, |expr| self.trans_expr(expr)),
            end: zopt::map_ref_mut(end, |expr| self.trans_expr(expr)),
        }
    }

    fn trans_box_expr(&mut self, expr: &rst::Expr) -> BoxExpr {
        BoxExpr {
            expr: self.trans_expr(expr),
        }
    }

    fn trans_cast_expr(&mut self, expr: &rst::Expr, ty: &rst::Ty) -> CastExpr {
        CastExpr {
            expr: self.trans_expr(expr),
            ty: self.trans_type(ty),
        }
    }

    fn trans_type_expr(&mut self, expr: &rst::Expr, ty: &rst::Ty) -> TypeExpr {
        TypeExpr {
            expr: self.trans_expr(expr),
            ty: self.trans_type(ty),
        }
    }

    fn trans_if_expr(&mut self, expr: &rst::Expr, block: &rst::Block,
                     left: &Option<rst::P<rst::Expr>>)
        -> IfExpr {
        let br = match *left {
            Some(ref expr) => Some(self.trans_expr(expr)),
            None => None,
        };

        IfExpr {
            expr: self.trans_expr(expr),
            block: self.trans_block(block),
            br: br,
        }
    }

    fn trans_if_let_expr(&mut self, pat: &rst::P<rst::Pat>, expr: &rst::Expr, block: &rst::Block,
                         left: &Option<rst::P<rst::Expr>>)
        -> IfLetExpr {
        let br = match *left {
            Some(ref expr) => (Some(self.trans_expr(expr))),
            None => None,
        };

        IfLetExpr {
            pat: self.trans_patten(pat),
            expr: self.trans_expr(expr),
            block: self.trans_block(block),
            br: br,
        }
    }

    fn trans_while_expr(&mut self, expr: &rst::Expr, block: &rst::Block, label: &Option<rst::Ident>)
        -> WhileExpr {
        let label = match *label {
            Some(ref ident) => Some(ident_to_string(ident)),
            None => None,
        };

        WhileExpr {
            label: label,
            expr: self.trans_expr(expr),
            block: self.trans_block(block),
        }
    }

    fn trans_while_let_expr(&mut self, pat: &rst::P<rst::Pat>, expr: &rst::Expr,
                            block: &rst::Block, label: &Option<rst::Ident>)
        -> WhileLetExpr {
        let label = match *label {
            Some(ref ident) => Some(ident_to_string(ident)),
            None => None,
        };

        WhileLetExpr {
            label: label,
            pat: self.trans_patten(pat),
            expr: self.trans_expr(expr),
            block: self.trans_block(block),
        }
    }

    fn trans_for_expr(&mut self, pat: &rst::P<rst::Pat>, expr: &rst::Expr, block: &rst::Block,
                      label: &Option<rst::Ident>)
        -> ForExpr {
        let label = match *label {
            Some(ref ident) => Some(ident_to_string(ident)),
            None => None,
        };

        ForExpr {
            label: label,
            pat: self.trans_patten(pat),
            expr: self.trans_expr(expr),
            block: self.trans_block(block),
        }
    }

    fn trans_loop_expr(&mut self, block: &rst::Block, label: &Option<rst::Ident>) -> LoopExpr {
        let label = match *label {
            Some(ref ident) => Some(ident_to_string(ident)),
            None => None,
        };

        LoopExpr {
            label: label,
            block: self.trans_block(block),
        }
    }

    fn trans_break_expr(&mut self, ident: &Option<rst::SpannedIdent>) -> BreakExpr {
        BreakExpr {
            label: zopt::map_ref_mut(ident, |ident| self.trans_ident(ident)),
        }
    }

    fn trans_continue_expr(&mut self, ident: &Option<rst::SpannedIdent>) -> ContinueExpr {
        ContinueExpr {
            label: zopt::map_ref_mut(ident, |ident| self.trans_ident(ident)),
        }
    }

    fn trans_match_expr(&mut self, expr: &rst::Expr, arms: &Vec<rst::Arm>) -> MatchExpr {
        MatchExpr {
            expr: self.trans_expr(expr),
            arms: self.trans_arms(arms),
        }
    }

    #[inline]
    fn trans_arms(&mut self, arms: &Vec<rst::Arm>) -> Vec<Arm> {
        trans_list!(self, arms, trans_arm)
    }

    fn trans_arm(&mut self, arm: &rst::Arm) -> Arm {
        let pats = self.trans_pattens(&arm.pats);
        let loc = if pats.is_empty() {
            Default::default()
        } else {
            pats[0].loc
        };

        Arm {
            loc: loc,
            attrs: self.trans_attrs(&arm.attrs),
            pats: self.trans_pattens(&arm.pats),
            guard: zopt::map_ref_mut(&arm.guard, |expr| self.trans_expr(expr)),
            body: self.trans_expr(&arm.body),
        }
    }

    fn trans_fn_call_expr(&mut self, fn_name: &rst::Expr, args: &Vec<rst::P<rst::Expr>>)
        -> FnCallExpr {
        FnCallExpr {
            name: self.trans_expr(fn_name),
            args: self.trans_exprs(args),
        }
    }

    fn trans_method_call_expr(&mut self, ident: &rst::SpannedIdent, types: &Vec<rst::P<rst::Ty>>,
                              args: &Vec<rst::P<rst::Expr>>)
        -> MethodCallExpr {
        MethodCallExpr {
            obj: self.trans_expr(&args[0]),
            name: self.trans_ident(ident),
            types: self.trans_types(types),
            args: self.trans_exprs(&args[1..]),
        }
    }

    fn trans_closure_expr(&mut self, capture: rst::CaptureClause, fn_decl: &rst::FnDecl,
                          block: &rst::Block)
        -> ClosureExpr {
        ClosureExpr {
            moved: is_move(capture),
            fn_sig: self.trans_fn_sig(fn_decl),
            block: self.trans_block(block),
        }
    }

    fn trans_return_expr(&mut self, expr: &Option<rst::P<rst::Expr>>) -> ReturnExpr {
        ReturnExpr {
            ret: zopt::map_ref_mut(expr, |expr| self.trans_expr(expr)),
        }
    }

    fn trans_macro_type(&mut self, mac: &rst::Mac) -> MacroType {
        self.trans_macro(mac)
    }

    fn trans_macro_impl_item(&mut self, mac: &rst::Mac) -> MacroImplItem {
        self.trans_macro(mac)
    }

    fn trans_macro_expr(&mut self, mac: &rst::Mac) -> MacroExpr {
        self.trans_macro(mac)
    }

    fn trans_macro(&mut self, mac: &rst::Mac) -> Macro {
        Macro {
            loc: self.leaf_loc(&mac.span),
            s: self.span_to_snippet(mac.span).unwrap(),
        }
    }
}
