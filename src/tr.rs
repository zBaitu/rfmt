use std::cmp::Ordering;
use std::collections::HashMap;
use std::result;

use rst;

use ir::*;

const MAX_BLANK_LINE: u8 = 1;

pub struct Result {
    pub krate: Crate,
    pub leading_cmnts: HashMap<Pos, Vec<String>>,
    pub trailing_cmnts: HashMap<Pos, String>,
}

pub fn trans(sess: rst::ParseSess, krate: rst::Crate, cmnts: Vec<rst::Comment>) -> Result {
    Translator::new(sess, trans_comments(cmnts)).trans_crate(krate)
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

#[inline]
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
fn is_pub(vis: &rst::Visibility) -> bool {
    *vis == rst::Visibility::Public
}

#[inline]
fn is_sized(modifier: rst::TraitBoundModifier) -> bool {
    modifier == rst::TraitBoundModifier::Maybe
}

#[inline]
fn is_mut(mutbl: rst::Mutability) -> bool {
    mutbl == rst::Mutability::Mutable
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
        rst::BlockCheckMode::Unsafe(source) => source == rst::UnsafeSource::UserProvided,
        _ => false,
    }
}

#[inline]
fn is_move(capture: rst::CaptureBy) -> bool {
    capture == rst::CaptureBy::Value
}

#[inline]
fn is_ref_mut(mode: rst::BindingMode) -> (bool, bool) {
    match mode {
        rst::BindingMode::ByRef(mutbl) => (true, is_mut(mutbl)),
        rst::BindingMode::ByValue(mutbl) => (false, is_mut(mutbl)),
    }
}

#[inline]
fn is_halfopen(range_limit: rst::RangeLimits) -> bool {
    range_limit == rst::RangeLimits::HalfOpen
}

#[inline]
fn is_default(defaultness: rst::Defaultness) -> bool {
    defaultness == rst::Defaultness::Default
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

#[inline]
fn is_macro_semi(style: &rst::MacStmtStyle) -> bool {
    match *style {
        rst::MacStmtStyle::Semicolon => true,
        _ => false,
    }
}

#[inline]
fn token_to_macro_expr_sep(token: &rst::Token) -> Option<MacroExprSep> {
    let (is_sep, s) = match *token {
        rst::Token::Comma => (true, ","),
        rst::Token::Semi => (true, ";"),
        rst::Token::FatArrow => (true, " =>"),
        rst::Token::DotDotDot => (false, "..."),
        _ => return None,
    };

    Some(MacroExprSep {
        is_sep: is_sep,
        s: s,
    })
}

#[inline]
fn map_ref_mut<T, F, R>(opt: &Option<T>, mut f: F) -> Option<R> where F: FnMut(&T) -> R {
    match *opt {
        Some(ref v) => Some(f(v)),
        None => None,
    }
}

struct Translator {
    sess: rst::ParseSess,
    cmnts: Vec<Comment>,
    cmnt_idx: usize,
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
    fn new(sess: rst::ParseSess, cmnts: Vec<Comment>) -> Translator {
        Translator {
            sess: sess,
            cmnts: cmnts,
            cmnt_idx: 0,
            last_loc: Default::default(),

            leading_cmnts: HashMap::new(),
            trailing_cmnts: HashMap::new(),
        }
    }

    fn trans_crate(mut self, krate: rst::Crate) -> Result {
        self.last_loc.start = krate.span.lo.0;

        let loc = self.loc(&krate.span);
        let attrs = self.trans_attrs(&krate.attrs);
        let crate_mod_name = self.crate_mod_name();
        let module = self.trans_mod(crate_mod_name, &krate.module);

        let crate_file_end = self.crate_file_end();
        self.trans_comments(crate_file_end);

        Result {
            krate: Crate {
                loc: loc,
                attrs: attrs,
                module: module,
            },
            leading_cmnts: self.leading_cmnts,
            trailing_cmnts: self.trailing_cmnts,
        }
    }

    fn crate_file_name(&self) -> String {
        self.sess.codemap().files.borrow().first().unwrap().name.clone()
    }

    fn crate_mod_name(&self) -> String {
        let mut name = self.crate_file_name();
        if let Some(pos) = name.rfind('.') {
            name.truncate(pos);
        }
        name
    }

    fn crate_file_end(&self) -> Pos {
        self.sess.codemap().files.borrow().last().unwrap().end_pos.0
    }

    #[inline]
    fn span_to_snippet(&self, sp: rst::Span) -> result::Result<String, rst::SpanSnippetError> {
        self.sess.codemap().span_to_snippet(sp)
    }

    #[inline]
    fn literal_to_string(&self, lit: &rst::Lit) -> String {
        self.span_to_snippet(lit.span).unwrap()
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
        self.trans_comments(loc.end);
        self.last_loc = *loc;
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
        if cmnt.pos > pos || cmnt.kind != CommentKind::Trailing {
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
    fn trans_attrs(&mut self, attrs: &Vec<rst::Attribute>) -> Vec<AttrKind> {
        trans_list!(self, attrs, trans_attr_kind)
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
        if let rst::MetaItemKind::NameValue(_, ref value) = attr.node.value.node {
            if let rst::LitKind::Str(ref s, _) = value.node {
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
            rst::MetaItemKind::Word(ref ident) => {
                MetaItem {
                    loc: self.leaf_loc(&meta_item.span),
                    name: ident.to_string(),
                    items: None,
                }
            },
            rst::MetaItemKind::NameValue(ref ident, ref lit) => {
                let s = format!("{} = {}", ident, self.literal_to_string(lit));
                MetaItem {
                    loc: self.leaf_loc(&meta_item.span),
                    name: s,
                    items: None,
                }
            },
            rst::MetaItemKind::List(ref ident, ref meta_items) => {
                let loc = self.loc(&meta_item.span);
                let items = self.trans_meta_items(meta_items);
                self.set_loc(&loc);

                MetaItem {
                    loc: loc,
                    name: ident.to_string(),
                    items: Some(Box::new(items)),
                }
            },
        }
    }

    fn trans_mod(&mut self, name: String, module: &rst::Mod) -> Mod {
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
    fn is_mod_decl(&self, module: &rst::Mod) -> bool {
        module.inner.lo == module.inner.hi
    }

    fn trans_items(&mut self, items: &Vec<rst::P<rst::Item>>) -> Vec<Item> {
        trans_list!(self, items, trans_item)
    }

    fn trans_item(&mut self, item: &rst::Item) -> Item {
        let loc = self.loc(&item.span);
        let attrs = self.trans_attrs(&item.attrs);

        let is_pub = is_pub(&item.vis);
        let ident = ident_to_string(&item.ident);
        let item = match item.node {
            rst::ItemKind::ExternCrate(ref rename) => {
                ItemKind::ExternCrate(self.trans_extren_crate(ident, rename))
            },
            rst::ItemKind::Use(ref view_path) => ItemKind::Use(self.trans_use(view_path)),
            rst::ItemKind::Mod(ref module) => {
                if self.is_mod_decl(module) {
                    ItemKind::ModDecl(self.trans_mod_decl(ident))
                } else {
                    ItemKind::Mod(self.trans_mod(ident, module))
                }
            },
            rst::ItemKind::Ty(ref ty, ref generics) => {
                ItemKind::TypeAlias(self.trans_type_alias(ident, generics, ty))
            },
            rst::ItemKind::ForeignMod(ref module)
                    => ItemKind::ForeignMod(self.trans_foreign_mod(module)),
            rst::ItemKind::Const(ref ty, ref expr)
                    => ItemKind::Const(self.trans_const(ident, ty, expr)),
            rst::ItemKind::Static(ref ty, mutbl, ref expr) => {
                ItemKind::Static(self.trans_static(is_mut(mutbl), ident, ty, expr))
            },
            rst::ItemKind::Struct(ref variant, ref generics) => {
                ItemKind::Struct(self.trans_struct(ident, generics, variant))
            },
            rst::ItemKind::Enum(ref enum_def, ref generics) => {
                ItemKind::Enum(self.trans_enum(ident, generics, enum_def))
            },
            rst::ItemKind::Fn(ref fn_decl, unsafety, constness, abi, ref generics, ref block)
                    => {
                ItemKind::Fn(self.trans_fn(is_unsafe(unsafety), is_const(constness),
                                           abi_to_string(abi), ident, generics, fn_decl, block))
            },
            rst::ItemKind::Trait(unsafety, ref generics, ref bounds, ref items) => {
                ItemKind::Trait(self.trans_trait(is_unsafe(unsafety), ident, generics, bounds,
                                                 items))
            },
            rst::ItemKind::DefaultImpl(unsafety, ref trait_ref) => {
                ItemKind::ImplDefault(self.trans_impl_default(is_unsafe(unsafety), trait_ref))
            },
            rst::ItemKind::Impl(unsafety, polarity, ref generics, ref trait_ref, ref ty,
                                ref items) => {
                ItemKind::Impl(self.trans_impl(is_unsafe(unsafety), is_neg(polarity), generics,
                                               trait_ref, ty, items))
            },
            rst::ItemKind::Mac(ref mac) => ItemKind::Macro(self.trans_macro_raw(mac)),
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
            Some(ref name) => format!("{} as {}", name_to_string(name), ident),
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
            },
            rst::ViewPathGlob(ref path) => {
                self.leaf_loc(&path.span);
                let base = format!("{}::*", path_to_string(path));
                (base, Vec::new())
            },
            rst::ViewPathList(ref path, ref items) => {
                let loc = self.loc(&path.span);
                let base = path_to_string(path);
                let names = self.trans_use_names(items);
                self.set_loc(&loc);
                (base, names)
            },
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

    fn trans_use_names(&mut self, items: &Vec<rst::PathListItem>) -> Vec<Chunk> {
        trans_list!(self, items, trans_use_name)
    }

    fn trans_use_name(&mut self, item: &rst::PathListItem) -> Chunk {
        let loc = self.leaf_loc(&item.span);
        let (mut s, rename) = match item.node {
            rst::PathListItemKind::Ident {
                ref name,
                ref rename,
                ..
            } => (ident_to_string(name), rename),
            rst::PathListItemKind::Mod {
                ref rename,
                ..
            } => ("self".to_string(), rename),
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

    fn trans_lifetime_defs(&mut self, lifetime_defs: &Vec<rst::LifetimeDef>)
    -> Vec<LifetimeDef> {
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

    fn trans_lifetimes(&mut self, lifetimes: &Vec<rst::Lifetime>) -> Vec<Lifetime> {
        trans_list!(self, lifetimes, trans_lifetime)
    }

    fn trans_lifetime(&mut self, lifetime: &rst::Lifetime) -> Lifetime {
        Lifetime {
            loc: self.leaf_loc(&lifetime.span),
            s: name_to_string(&lifetime.name),
        }
    }

    fn trans_type_params(&mut self, type_params: &[rst::TyParam]) -> Vec<TypeParam> {
        trans_list!(self, type_params, trans_type_param)
    }

    fn trans_type_param(&mut self, type_param: &rst::TyParam) -> TypeParam {
        let loc = self.loc(&type_param.span);
        let name = ident_to_string(&type_param.ident);
        let bounds = self.trans_type_param_bounds(&type_param.bounds);
        let default = map_ref_mut(&type_param.default, |ty| self.trans_type(ty));
        self.set_loc(&loc);

        TypeParam {
            loc: loc,
            name: name,
            bounds: bounds,
            default: default,
        }
    }

    fn trans_type_param_bounds(&mut self, bounds: &[rst::TyParamBound]) -> Vec<TypeParamBound> {
        trans_list!(self, bounds, trans_type_param_bound)
    }

    fn trans_type_param_bound(&mut self, bound: &rst::TyParamBound) -> TypeParamBound {
        match *bound {
            rst::RegionTyParamBound(ref lifetime) => {
                TypeParamBound::Lifetime(self.trans_lifetime(lifetime))
            },
            rst::TraitTyParamBound(ref poly_trait_ref, modifier) => {
                TypeParamBound::PolyTraitRef(self.trans_poly_trait_ref(poly_trait_ref, modifier))
            },
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

    fn trans_where(&mut self, predicates: &Vec<rst::WherePredicate>) -> Where {
        Where {
            clauses: self.trans_where_clauses(predicates),
        }
    }

    fn trans_where_clauses(&mut self, predicates: &Vec<rst::WherePredicate>)
    -> Vec<WhereClause> {
        trans_list!(self, predicates, trans_where_clause)
    }

    fn trans_where_clause(&mut self, predicate: &rst::WherePredicate) -> WhereClause {
        match *predicate {
            rst::WherePredicate::RegionPredicate(ref region)
                    => self.trans_where_lifetime(region),
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

    fn trans_type_bindings(&mut self, bindings: &[rst::TypeBinding]) -> Vec<TypeBinding> {
        trans_list!(self, bindings, trans_type_binding)
    }

    fn trans_type_binding(&mut self, binding: &rst::TypeBinding) -> TypeBinding {
        let loc = self.loc(&binding.span);
        let name = ident_to_string(&binding.ident);
        let ty = self.trans_type(&binding.ty);
        self.set_loc(&loc);

        TypeBinding {
            loc: loc,
            name: name,
            ty: ty,
        }
    }

    fn trans_paren_param(&mut self, param: &rst::ParenthesizedParameterData) -> ParenParam {
        let loc = self.loc(&param.span);
        let inputs = self.trans_types(&param.inputs);
        let output = map_ref_mut(&param.output, |ty| self.trans_type(ty));
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

    fn trans_types(&mut self, types: &[rst::P<rst::Ty>]) -> Vec<Type> {
        trans_list!(self, types, trans_type)
    }

    fn trans_type(&mut self, ty: &rst::Ty) -> Type {
        let loc = self.loc(&ty.span);

        let ty = match ty.node {
            rst::TyKind::Path(ref qself, ref path) => {
                TypeKind::Path(Box::new(self.trans_path_type(qself, path)))
            },
            rst::TyKind::Ptr(ref mut_type)
                    => TypeKind::Ptr(Box::new(self.trans_ptr_type(mut_type))),
            rst::TyKind::Rptr(ref lifetime, ref mut_type) => {
                TypeKind::Ref(Box::new(self.trans_ref_type(lifetime, mut_type)))
            },
            rst::TyKind::Vec(ref ty) => TypeKind::Array(Box::new(self.trans_array_type(ty))),
            rst::TyKind::FixedLengthVec(ref ty, ref expr) => {
                TypeKind::FixedSizeArray(Box::new(self.trans_fixed_size_array_type(ty, expr)))
            },
            rst::TyKind::Tup(ref types)
                    => TypeKind::Tuple(Box::new(self.trans_tuple_type(types))),
            rst::TyKind::Paren(ref ty) => {
                TypeKind::Tuple(Box::new(self.trans_tuple_type(&vec![ty.clone()])))
            },
            rst::TyKind::BareFn(ref bare_fn) => {
                TypeKind::BareFn(Box::new(self.trans_bare_fn_type(bare_fn)))
            },
            rst::TyKind::ObjectSum(ref ty, ref bounds) => {
                TypeKind::Sum(Box::new(self.trans_sum_type(ty, bounds)))
            },
            rst::TyKind::PolyTraitRef(ref bounds) => {
                TypeKind::PolyTraitRef(Box::new(self.trans_poly_trait_ref_type(bounds)))
            },
            rst::TyKind::Mac(ref mac) => TypeKind::Macro(Box::new(self.trans_macro(mac))),
            rst::TyKind::Infer => TypeKind::Infer,
            rst::TyKind::Typeof(_) => unreachable!(),
        };

        self.set_loc(&loc);
        Type {
            loc: loc,
            ty: ty,
        }
    }

    #[inline]
    fn trans_path_type(&mut self, qself: &Option<rst::QSelf>, path: &rst::Path) -> PathType {
        PathType {
            qself: map_ref_mut(qself, |qself| self.trans_qself(qself)),
            path: self.trans_path(path),
        }
    }

    #[inline]
    fn trans_ptr_type(&mut self, mut_type: &rst::MutTy) -> PtrType {
        PtrType {
            is_mut: is_mut(mut_type.mutbl),
            ty: self.trans_type(&mut_type.ty),
        }
    }

    #[inline]
    fn trans_ref_type(&mut self, lifetime: &Option<rst::Lifetime>, mut_type: &rst::MutTy)
    -> RefType {
        RefType {
            lifetime: map_ref_mut(lifetime, |lifetime| self.trans_lifetime(lifetime)),
            is_mut: is_mut(mut_type.mutbl),
            ty: self.trans_type(&mut_type.ty),
        }
    }

    #[inline]
    fn trans_array_type(&mut self, ty: &rst::Ty) -> ArrayType {
        ArrayType {
            ty: self.trans_type(ty),
        }
    }

    #[inline]
    fn trans_fixed_size_array_type(&mut self, ty: &rst::Ty, expr: &rst::Expr)
    -> FixedSizeArrayType {
        FixedSizeArrayType {
            ty: self.trans_type(ty),
            expr: self.trans_expr(expr),
        }
    }

    #[inline]
    fn trans_tuple_type(&mut self, types: &Vec<rst::P<rst::Ty>>) -> TupleType {
        TupleType {
            types: self.trans_types(types),
        }
    }

    #[inline]
    fn trans_bare_fn_type(&mut self, bare_fn: &rst::BareFnTy) -> BareFnType {
        BareFnType {
            lifetime_defs: self.trans_lifetime_defs(&bare_fn.lifetimes),
            is_unsafe: is_unsafe(bare_fn.unsafety),
            abi: abi_to_string(bare_fn.abi),
            fn_sig: self.trans_fn_sig(&bare_fn.decl),
        }
    }

    #[inline]
    fn trans_sum_type(&mut self, ty: &rst::Ty, bounds: &rst::TyParamBounds) -> SumType {
        SumType {
            ty: self.trans_type(ty),
            bounds: self.trans_type_param_bounds(bounds),
        }
    }

    #[inline]
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

    fn trans_foreign_items(&mut self, items: &Vec<rst::ForeignItem>) -> Vec<ForeignItem> {
        trans_list!(self, items, trans_foreign_item)
    }

    fn trans_foreign_item(&mut self, item: &rst::ForeignItem) -> ForeignItem {
        let loc = self.loc(&item.span);
        let attrs = self.trans_attrs(&item.attrs);

        let is_pub = is_pub(&item.vis);
        let ident = ident_to_string(&item.ident);
        let item = match item.node {
            rst::ForeignItemKind::Static(ref ty, is_mut) => {
                ForeignKind::Static(self.trans_foreign_static(is_mut, ident, ty))
            },
            rst::ForeignItemKind::Fn(ref fn_decl, ref generics) => {
                ForeignKind::Fn(self.trans_foreign_fn(ident, generics, fn_decl))
            },
        };

        self.set_loc(&loc);
        ForeignItem {
            loc: loc,
            attrs: attrs,
            is_pub: is_pub,
            item: item,
        }
    }

    fn trans_foreign_static(&mut self, is_mut: bool, ident: String, ty: &rst::Ty)
    -> ForeignStatic {
        ForeignStatic {
            is_mut: is_mut,
            name: ident,
            ty: self.trans_type(ty),
        }
    }

    fn trans_foreign_fn(&mut self, ident: String, generics: &rst::Generics,
                        fn_decl: &rst::FnDecl)
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

    fn trans_static(&mut self, is_mut: bool, ident: String, ty: &rst::Ty, expr: &rst::Expr)
    -> Static {
        Static {
            is_mut: is_mut,
            name: ident,
            ty: self.trans_type(ty),
            expr: self.trans_expr(expr),
        }
    }

    fn trans_struct(&mut self, ident: String, generics: &rst::Generics,
                    variant: &rst::VariantData)
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
            },
            rst::VariantData::Tuple(ref fields, _) => {
                StructBody::Tuple(self.trans_tuple_fields(fields))
            },
            rst::VariantData::Unit(_) => StructBody::Unit,
        }
    }

    fn trans_struct_fields(&mut self, fields: &Vec<rst::StructField>) -> Vec<StructField> {
        trans_list!(self, fields, trans_struct_field)
    }

    fn trans_struct_field(&mut self, field: &rst::StructField) -> StructField {
        let loc = self.loc(&field.span);
        let attrs = self.trans_attrs(&field.attrs);
        let is_pub = is_pub(&field.vis);
        let name = ident_to_string(&field.ident.unwrap());
        let ty = self.trans_type(&field.ty);
        self.set_loc(&loc);

        StructField {
            loc: loc,
            attrs: attrs,
            is_pub: is_pub,
            name: name,
            ty: ty,
        }
    }

    fn trans_tuple_fields(&mut self, fields: &Vec<rst::StructField>) -> Vec<TupleField> {
        trans_list!(self, fields, trans_tuple_field)
    }

    fn trans_tuple_field(&mut self, field: &rst::StructField) -> TupleField {
        let loc = self.loc(&field.span);
        let attrs = self.trans_attrs(&field.attrs);
        let is_pub = is_pub(&field.vis);
        let ty = self.trans_type(&field.ty);
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

    fn trans_enum_fields(&mut self, variants: &Vec<rst::Variant>) -> Vec<EnumField> {
        trans_list!(self, variants, trans_enum_field)
    }

    fn trans_enum_field(&mut self, variant: &rst::Variant) -> EnumField {
        let loc = self.loc(&variant.span);
        let attrs = self.trans_attrs(&variant.node.attrs);
        let name = ident_to_string(&variant.node.name);
        let body = self.trans_struct_body(&variant.node.data);
        let expr = map_ref_mut(&variant.node.disr_expr, |expr| self.trans_expr(expr));
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
                   bounds: &rst::TyParamBounds, items: &Vec<rst::TraitItem>)
    -> Trait {
        Trait {
            is_unsafe: is_unsafe,
            name: ident,
            generics: self.trans_generics(generics),
            bounds: self.trans_type_param_bounds(bounds),
            items: self.trans_trait_items(items),
        }
    }

    fn trans_trait_items(&mut self, items: &Vec<rst::TraitItem>) -> Vec<TraitItem> {
        trans_list!(self, items, trans_trait_item)
    }

    fn trans_trait_item(&mut self, item: &rst::TraitItem) -> TraitItem {
        let loc = self.loc(&item.span);
        let attrs = self.trans_attrs(&item.attrs);
        let ident = ident_to_string(&item.ident);
        let item = match item.node {
            rst::TraitItemKind::Const(ref ty, ref expr) => {
                TraitItemKind::Const(self.trans_const_trait_item(ident, ty, expr))
            },
            rst::TraitItemKind::Type(ref bounds, ref ty) => {
                TraitItemKind::Type(self.trans_type_trait_item(ident, bounds, ty))
            },
            rst::TraitItemKind::Method(ref method_sig, ref block) => {
                TraitItemKind::Method(self.trans_method_trait_item(ident, method_sig, block))
            },
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
        ConstTraitItem {
            name: ident,
            ty: self.trans_type(ty),
            expr: map_ref_mut(expr, |expr| self.trans_expr(expr)),
        }
    }

    fn trans_type_trait_item(&mut self, ident: String, bounds: &rst::TyParamBounds,
                             ty: &Option<rst::P<rst::Ty>>)
    -> TypeTraitItem {
        TypeTraitItem {
            name: ident,
            bounds: self.trans_type_param_bounds(bounds),
            ty: map_ref_mut(ty, |ty| self.trans_type(ty)),
        }
    }

    fn trans_method_trait_item(&mut self, ident: String, method_sig: &rst::MethodSig,
                               block: &Option<rst::P<rst::Block>>)
    -> MethodTraitItem {
        MethodTraitItem {
            is_unsafe: is_unsafe(method_sig.unsafety),
            is_const: is_const(method_sig.constness),
            abi: abi_to_string(method_sig.abi),
            name: ident,
            method_sig: self.trans_method_sig(method_sig),
            block: map_ref_mut(block, |block| self.trans_block(block)),
        }
    }

    fn trans_impl_default(&mut self, is_unsafe: bool, trait_ref: &rst::TraitRef) -> ImplDefault {
        ImplDefault {
            is_unsafe: is_unsafe,
            trait_ref: self.trans_trait_ref(trait_ref),
        }
    }

    fn trans_impl(&mut self, is_unsafe: bool, is_neg: bool, generics: &rst::Generics,
                  trait_ref: &Option<rst::TraitRef>, ty: &rst::Ty, items: &Vec<rst::ImplItem>)
    -> Impl {
        Impl {
            is_unsafe: is_unsafe,
            is_neg: is_neg,
            generics: self.trans_generics(generics),
            trait_ref: map_ref_mut(trait_ref, |trait_ref| self.trans_trait_ref(trait_ref)),
            ty: self.trans_type(ty),
            items: self.trans_impl_items(items),
        }
    }

    fn trans_impl_items(&mut self, items: &Vec<rst::ImplItem>) -> Vec<ImplItem> {
        trans_list!(self, items, trans_impl_item)
    }

    fn trans_impl_item(&mut self, item: &rst::ImplItem) -> ImplItem {
        let loc = self.loc(&item.span);
        let attrs = self.trans_attrs(&item.attrs);

        let is_pub = is_pub(&item.vis);
        let is_default = is_default(item.defaultness);
        let ident = ident_to_string(&item.ident);
        let item = match item.node {
            rst::ImplItemKind::Const(ref ty, ref expr) => {
                ImplItemKind::Const(self.trans_const_impl_item(ident, ty, expr))
            },
            rst::ImplItemKind::Type(ref ty) => {
                ImplItemKind::Type(self.trans_type_impl_item(ident, ty))
            },
            rst::ImplItemKind::Method(ref method_sig, ref block) => {
                ImplItemKind::Method(self.trans_method_impl_item(ident, method_sig, block))
            },
            rst::ImplItemKind::Macro(ref mac) => ImplItemKind::Macro(self.trans_macro(mac)),
        };
        self.set_loc(&loc);

        ImplItem {
            loc: loc,
            attrs: attrs,
            is_pub: is_pub,
            is_default: is_default,
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
        let (nl, ret) = match *output {
            rst::FunctionRetTy::Default(_) => (false, FnReturnKind::Unit),
            rst::FunctionRetTy::None(ref span)
                    => (self.is_fn_return_nl(span.lo.0), FnReturnKind::Diverge),
            rst::FunctionRetTy::Ty(ref ty) => (self.is_fn_return_nl(ty.span.lo.0),
                                               FnReturnKind::Normal(self.trans_type(ty))),
        };

        FnReturn {
            nl: nl,
            ret: ret,
        }
    }

    fn is_fn_return_nl(&self, pos: Pos) -> bool {
        let snippet = self.span_to_snippet(span(self.last_loc.end, pos)).unwrap();
        let right_arrow_pos = self.last_loc.end + snippet.find("->").unwrap() as Pos;
        self.is_nl(right_arrow_pos)
    }

    fn trans_method_sig(&mut self, method_sig: &rst::MethodSig) -> MethodSig {
        MethodSig {
            generics: self.trans_generics(&method_sig.generics),
            sf: self.trans_self(&method_sig.explicit_self.node, &method_sig.decl),
            fn_sig: self.trans_fn_sig(&method_sig.decl),
        }
    }

    fn trans_self(&mut self, self_kind: &rst::SelfKind, fn_sig: &rst::FnDecl) -> Option<Sf> {
        match *self_kind {
            rst::SelfKind::Static => None,
            rst::SelfKind::Value(_) => {
                let arg = &fn_sig.inputs[0];
                let is_mut = match arg.pat.node {
                    rst::PatKind::Ident(mode, _, _) => {
                        let (_, is_mut) = is_ref_mut(mode);
                        is_mut
                    },
                    _ => unreachable!(),
                };
                let sf = if is_mut {
                    "mut self"
                } else {
                    "self"
                }.to_string();
                Some(Sf::String(sf))
            },
            rst::SelfKind::Region(lifetime, mutbl, _) => {
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
            },
            rst::SelfKind::Explicit(ref ty, _) => Some(Sf::Type(self.trans_type(ty))),
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

    fn trans_stmts(&mut self, stmts: &Vec<rst::Stmt>) -> Vec<Stmt> {
        trans_list!(self, stmts, trans_stmt)
    }

    #[inline]
    fn trans_stmt(&mut self, stmt: &rst::Stmt) -> Stmt {
        let loc = self.loc(&stmt.span);
        let stmt = match stmt.node {
            rst::StmtKind::Decl(ref decl, _) => StmtKind::Decl(self.trans_decl(decl)),
            rst::StmtKind::Semi(ref expr, _) => StmtKind::Expr(self.trans_expr(expr), true),
            rst::StmtKind::Expr(ref expr, _) => StmtKind::Expr(self.trans_expr(expr), false),
            rst::StmtKind::Mac(ref mac, ref style, ref attrs) => {
                StmtKind::Macro(self.trans_macro_stmt(attrs, mac), is_macro_semi(style))
            },
        };
        self.set_loc(&loc);

        Stmt {
            loc: loc,
            stmt: stmt,
        }
    }

    #[inline]
    fn trans_decl(&mut self, decl: &rst::Decl) -> Decl {
        let loc = self.loc(&decl.span);
        let decl = match decl.node {
            rst::DeclKind::Local(ref local) => DeclKind::Local(self.trans_local(local)),
            rst::DeclKind::Item(ref item) => DeclKind::Item(self.trans_item(item)),
        };
        self.set_loc(&loc);

        Decl {
            loc: loc,
            decl: decl,
        }
    }

    #[inline]
    fn trans_local(&mut self, local: &rst::Local) -> Local {
        let loc = self.loc(&local.span);
        let attrs = self.trans_thin_attrs(&local.attrs);
        let pat = self.trans_patten(&local.pat);
        let ty = map_ref_mut(&local.ty, |ty| self.trans_type(ty));
        let init = map_ref_mut(&local.init, |expr| self.trans_expr(expr));
        self.set_loc(&loc);

        Local {
            loc: loc,
            attrs: attrs,
            pat: pat,
            ty: ty,
            init: init,
        }
    }

    #[inline]
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
            rst::PatKind::Wild => PattenKind::Wildcard,
            rst::PatKind::Lit(ref expr) => PattenKind::Literal(self.trans_expr(expr)),
            rst::PatKind::Range(ref start, ref end) => {
                PattenKind::Range(self.trans_range_patten(start, end))
            },
            rst::PatKind::Ident(mode, ref ident, ref binding) => {
                PattenKind::Ident(Box::new(self.trans_ident_patten(mode, ident, binding)))
            },
            rst::PatKind::Ref(ref pat, mutbl) => {
                PattenKind::Ref(Box::new(self.trans_ref_patten(is_mut(mutbl), pat)))
            },
            rst::PatKind::QPath(ref qself, ref path) => {
                PattenKind::Path(self.trans_path_patten(Some(qself), path))
            },
            rst::PatKind::Path(ref path) => {
                PattenKind::Path(self.trans_path_patten(None, path))
            },
            rst::PatKind::TupleStruct(ref path, ref pats) => {
                PattenKind::Enum(self.trans_enum_patten(path, pats))
            },
            rst::PatKind::Struct(ref path, ref fields, etc) => {
                PattenKind::Struct(Box::new(self.trans_struct_patten(path, fields, etc)))
            },
            rst::PatKind::Vec(ref start, ref emit, ref end) => {
                PattenKind::Vec(Box::new(self.trans_vec_patten(start, emit, end)))
            },
            rst::PatKind::Tup(ref pats)
                    => PattenKind::Tuple(Box::new(self.trans_tuple_patten(pats))),
            rst::PatKind::Box(ref pat) => PattenKind::Box(Box::new(self.trans_patten(pat))),
            rst::PatKind::Mac(ref mac) => PattenKind::Macro(self.trans_macro(mac)),
        };
        self.set_loc(&loc);

        Patten {
            loc: loc,
            pat: pat,
        }
    }

    #[inline]
    fn trans_range_patten(&mut self, start: &rst::Expr, end: &rst::Expr) -> RangePatten {
        RangePatten {
            start: self.trans_expr(start),
            end: self.trans_expr(end),
        }
    }

    #[inline]
    fn trans_ident_patten(&mut self, mode: rst::BindingMode, ident: &rst::SpannedIdent,
                          binding: &Option<rst::P<rst::Pat>>)
    -> IdentPatten {
        let (is_ref, is_mut) = is_ref_mut(mode);
        IdentPatten {
            is_ref: is_ref,
            is_mut: is_mut,
            name: self.trans_ident(ident),
            binding: map_ref_mut(binding, |pat| self.trans_patten(pat)),
        }
    }

    #[inline]
    fn trans_ref_patten(&mut self, is_mut: bool, pat: &rst::P<rst::Pat>) -> RefPatten {
        RefPatten {
            is_mut: is_mut,
            pat: self.trans_patten(pat),
        }
    }

    #[inline]
    fn trans_path_patten(&mut self, qself: Option<&rst::QSelf>, path: &rst::Path) -> PathPatten {
        PathPatten {
            qself: map_ref_mut(&qself, |qself| self.trans_qself(qself)),
            path: self.trans_path(path),
        }
    }

    #[inline]
    fn trans_enum_patten(&mut self, path: &rst::Path, pats: &Option<Vec<rst::P<rst::Pat>>>)
    -> EnumPatten {
        EnumPatten {
            path: self.trans_path(path),
            pats: map_ref_mut(pats, |pats| self.trans_pattens(pats)),
        }
    }

    #[inline]
    fn trans_struct_patten(&mut self, path: &rst::Path,
                           fields: &Vec<rst::Spanned<rst::FieldPat>>,
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

    #[inline]
    fn trans_struct_field_patten(&mut self, field: &rst::Spanned<rst::FieldPat>)
    -> StructFieldPatten {
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

    #[inline]
    fn trans_vec_patten(&mut self, start: &Vec<rst::P<rst::Pat>>,
                        emit: &Option<rst::P<rst::Pat>>,
                        end: &Vec<rst::P<rst::Pat>>)
    -> VecPatten {
        VecPatten {
            start: self.trans_pattens(start),
            emit: map_ref_mut(emit, |pat| self.trans_patten(pat)),
            end: self.trans_pattens(end),
        }
    }

    #[inline]
    fn trans_tuple_patten(&mut self, pats: &Vec<rst::P<rst::Pat>>) -> TuplePatten {
        TuplePatten {
            pats: self.trans_pattens(pats),
        }
    }

    #[inline]
    fn expr_to_stmt(&mut self, expr: Expr) -> Stmt {
        Stmt {
            loc: expr.loc,
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
            rst::ExprKind::Lit(ref lit) => ExprKind::Literal(self.trans_literal_expr(lit)),
            rst::ExprKind::Path(ref qself, ref path)
                    => ExprKind::Path(self.trans_path_type(qself, path)),
            rst::ExprKind::Unary(op, ref expr) => {
                ExprKind::Unary(Box::new(self.trans_unary_expr(op, expr)))
            },
            rst::ExprKind::AddrOf(mutble, ref expr) => {
                ExprKind::Ref(Box::new(self.trans_ref_expr(mutble, expr)))
            },
            rst::ExprKind::Binary(ref op, ref left, ref right) => {
                ExprKind::List(Box::new(self.trans_binary_expr(left, op, right)))
            },
            rst::ExprKind::Assign(ref left, ref right) => {
                ExprKind::List(Box::new(self.trans_assign_expr(left, right)))
            },
            rst::ExprKind::AssignOp(ref op, ref left, ref right) => {
                ExprKind::List(Box::new(self.trans_op_assign_expr(left, op, right)))
            },
            rst::ExprKind::InPlace(ref left, ref right) => {
                ExprKind::List(Box::new(self.trans_in_place_expr(left, right)))
            },
            rst::ExprKind::Repeat(ref init, ref len) => {
                ExprKind::FixedSizeArray(Box::new(self.trans_fixed_size_array_expr(init, len)))
            },
            rst::ExprKind::Vec(ref exprs) => ExprKind::Vec(Box::new(self.trans_exprs(exprs))),
            rst::ExprKind::Tup(ref exprs) => ExprKind::Tuple(Box::new(self.trans_exprs(exprs))),
            rst::ExprKind::Paren(ref expr)
                    => ExprKind::Tuple(Box::new(vec![self.trans_expr(expr)])),
            rst::ExprKind::Field(ref expr, ref ident) => {
                ExprKind::FieldAccess(Box::new(self.trans_struct_field_access_expr(expr, ident)))
            },
            rst::ExprKind::TupField(ref expr, ref pos) => {
                ExprKind::FieldAccess(Box::new(self.trans_tuple_field_access_expr(expr, pos)))
            },
            rst::ExprKind::Struct(ref path, ref fields, ref base) => {
                ExprKind::Struct(Box::new(self.trans_struct_expr(path, fields, base)))
            },
            rst::ExprKind::Index(ref obj, ref index) => {
                ExprKind::Index(Box::new(self.trans_index_expr(obj, index)))
            },
            rst::ExprKind::Range(ref start, ref end, limit) => {
                ExprKind::Range(Box::new(self.trans_range_expr(start, end, limit)))
            },
            rst::ExprKind::Box(ref expr) => ExprKind::Box(Box::new(self.trans_box_expr(expr))),
            rst::ExprKind::Cast(ref expr, ref ty) => {
                ExprKind::Cast(Box::new(self.trans_cast_expr(expr, ty)))
            },
            rst::ExprKind::Type(ref expr, ref ty) => {
                ExprKind::Type(Box::new(self.trans_type_expr(expr, ty)))
            },
            rst::ExprKind::Block(ref block)
                    => ExprKind::Block(Box::new(self.trans_block(block))),
            rst::ExprKind::If(ref expr, ref block, ref br) => {
                ExprKind::If(Box::new(self.trans_if_expr(expr, block, br)))
            },
            rst::ExprKind::IfLet(ref pat, ref expr, ref block, ref br) => {
                ExprKind::IfLet(Box::new(self.trans_if_let_expr(pat, expr, block, br)))
            },
            rst::ExprKind::While(ref expr, ref block, ref label) => {
                ExprKind::While(Box::new(self.trans_while_expr(expr, block, label)))
            },
            rst::ExprKind::WhileLet(ref pat, ref expr, ref block, ref label) => {
                ExprKind::WhileLet(Box::new(self.trans_while_let_expr(pat, expr, block, label)))
            },
            rst::ExprKind::ForLoop(ref pat, ref expr, ref block, ref label) => {
                ExprKind::For(Box::new(self.trans_for_expr(pat, expr, block, label)))
            },
            rst::ExprKind::Loop(ref block, ref label) => {
                ExprKind::Loop(Box::new(self.trans_loop_expr(block, label)))
            },
            rst::ExprKind::Break(ref ident)
                    => ExprKind::Break(Box::new(self.trans_break_expr(ident))),
            rst::ExprKind::Again(ref ident) => {
                ExprKind::Continue(Box::new(self.trans_continue_expr(ident)))
            },
            rst::ExprKind::Match(ref expr, ref arms) => {
                ExprKind::Match(Box::new(self.trans_match_expr(expr, arms)))
            },
            rst::ExprKind::Call(ref fn_name, ref args) => {
                ExprKind::FnCall(Box::new(self.trans_fn_call_expr(fn_name, args)))
            },
            rst::ExprKind::MethodCall(ref ident, ref types, ref args) => {
                ExprKind::MethodCall(Box::new(self.trans_method_call_expr(ident, types, args)))
            },
            rst::ExprKind::Closure(capture, ref fn_decl, ref block) => {
                ExprKind::Closure(Box::new(self.trans_closure_expr(capture, fn_decl, block)))
            },
            rst::ExprKind::Ret(ref expr)
                    => ExprKind::Return(Box::new(self.trans_return_expr(expr))),
            rst::ExprKind::Mac(ref mac) => ExprKind::Macro(self.trans_macro(mac)),
            rst::ExprKind::Try(ref expr) => ExprKind::Try(Box::new(self.trans_expr(expr))),
            rst::ExprKind::InlineAsm(_) => unreachable!(),
        };
        self.set_loc(&loc);

        Expr {
            loc: loc,
            attrs: attrs,
            expr: expr,
        }
    }

    #[inline]
    fn trans_ident(&mut self, ident: &rst::SpannedIdent) -> Chunk {
        Chunk {
            loc: self.leaf_loc(&ident.span),
            s: ident_to_string(&ident.node),
        }
    }

    #[inline]
    fn trans_method_ident(&mut self, ident: &rst::SpannedIdent) -> Chunk {
        let s = ident_to_string(&ident.node);
        let mut span = ident.span;
        span.hi = rst::BytePos(span.lo.0 + s.len() as u32);

        Chunk {
            loc: self.leaf_loc(&span),
            s: s,
        }
    }

    #[inline]
    fn trans_pos(&mut self, pos: &rst::Spanned<usize>) -> Chunk {
        Chunk {
            loc: self.leaf_loc(&pos.span),
            s: pos.node.to_string(),
        }
    }

    #[inline]
    fn trans_literal_expr(&mut self, lit: &rst::Lit) -> Chunk {
        Chunk {
            loc: self.leaf_loc(&lit.span),
            s: self.literal_to_string(lit),
        }
    }

    #[inline]
    fn trans_unary_expr(&mut self, op: rst::UnOp, expr: &rst::Expr) -> UnaryExpr {
        UnaryExpr {
            op: uop_to_string(op),
            expr: self.trans_expr(expr),
        }
    }

    #[inline]
    fn trans_ref_expr(&mut self, mutble: rst::Mutability, expr: &rst::Expr) -> RefExpr {
        RefExpr {
            is_mut: is_mut(mutble),
            expr: self.trans_expr(expr),
        }
    }

    #[inline]
    fn trans_bop(&mut self, op: &rst::BinOp) -> Chunk {
        Chunk {
            loc: self.leaf_loc(&op.span),
            s: op.node.to_string().to_string(),
        }
    }

    #[inline]
    fn trans_bop_assign(&mut self, op: &rst::BinOp) -> Chunk {
        Chunk {
            loc: self.leaf_loc(&op.span),
            s: format!("{}=", op.node.to_string()),
        }
    }

    #[inline]
    fn trans_binary_expr(&mut self, left: &rst::Expr, op: &rst::BinOp, right: &rst::Expr)
    -> ListExpr {
        ListExpr {
            exprs: vec![self.trans_expr(left), self.trans_expr(right)],
            sep: self.trans_bop(op),
        }
    }

    #[inline]
    fn trans_assign_expr(&mut self, left: &rst::Expr, right: &rst::Expr) -> ListExpr {
        ListExpr {
            exprs: vec![self.trans_expr(left), self.trans_expr(right)],
            sep: Chunk::new("="),
        }
    }

    #[inline]
    fn trans_op_assign_expr(&mut self, left: &rst::Expr, op: &rst::BinOp, right: &rst::Expr)
    -> ListExpr {
        ListExpr {
            exprs: vec![self.trans_expr(left), self.trans_expr(right)],
            sep: self.trans_bop_assign(op),
        }
    }

    #[inline]
    fn trans_in_place_expr(&mut self, left: &rst::Expr, right: &rst::Expr) -> ListExpr {
        ListExpr {
            exprs: vec![self.trans_expr(left), self.trans_expr(right)],
            sep: Chunk::new("<-"),
        }
    }

    #[inline]
    fn trans_fixed_size_array_expr(&mut self, init: &rst::Expr, len: &rst::Expr)
    -> FixedSizeArrayExpr {
        FixedSizeArrayExpr {
            init: self.trans_expr(init),
            len: self.trans_expr(len),
        }
    }

    #[inline]
    fn trans_struct_field_access_expr(&mut self, expr: &rst::Expr, ident: &rst::SpannedIdent)
    -> FieldAccessExpr {
        FieldAccessExpr {
            expr: self.trans_expr(expr),
            field: self.trans_ident(ident),
        }
    }

    #[inline]
    fn trans_tuple_field_access_expr(&mut self, expr: &rst::Expr, pos: &rst::Spanned<usize>)
    -> FieldAccessExpr {
        FieldAccessExpr {
            expr: self.trans_expr(expr),
            field: self.trans_pos(pos),
        }
    }

    #[inline]
    fn trans_struct_expr(&mut self, path: &rst::Path, fields: &Vec<rst::Field>,
                         base: &Option<rst::P<rst::Expr>>)
    -> StructExpr {
        StructExpr {
            path: self.trans_path(path),
            fields: self.trans_struct_field_exprs(fields),
            base: map_ref_mut(base, |expr| self.trans_expr(expr)),
        }
    }

    #[inline]
    fn trans_struct_field_exprs(&mut self, fields: &Vec<rst::Field>) -> Vec<StructFieldExpr> {
        trans_list!(self, fields, trans_struct_field_expr)
    }

    #[inline]
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

    #[inline]
    fn trans_index_expr(&mut self, obj: &rst::Expr, index: &rst::Expr) -> IndexExpr {
        IndexExpr {
            obj: self.trans_expr(obj),
            index: self.trans_expr(index),
        }
    }

    #[inline]
    fn trans_range_expr(&mut self, start: &Option<rst::P<rst::Expr>>,
                        end: &Option<rst::P<rst::Expr>>, limit: rst::RangeLimits)
    -> RangeExpr {
        RangeExpr {
            start: map_ref_mut(start, |expr| self.trans_expr(expr)),
            end: map_ref_mut(end, |expr| self.trans_expr(expr)),
            is_halfopen: is_halfopen(limit),
        }
    }

    #[inline]
    fn trans_box_expr(&mut self, expr: &rst::Expr) -> BoxExpr {
        BoxExpr {
            expr: self.trans_expr(expr),
        }
    }

    #[inline]
    fn trans_cast_expr(&mut self, expr: &rst::Expr, ty: &rst::Ty) -> CastExpr {
        CastExpr {
            expr: self.trans_expr(expr),
            ty: self.trans_type(ty),
        }
    }

    #[inline]
    fn trans_type_expr(&mut self, expr: &rst::Expr, ty: &rst::Ty) -> TypeExpr {
        TypeExpr {
            expr: self.trans_expr(expr),
            ty: self.trans_type(ty),
        }
    }

    #[inline]
    fn trans_if_expr(&mut self, expr: &rst::Expr, block: &rst::Block,
                     br: &Option<rst::P<rst::Expr>>)
    -> IfExpr {
        IfExpr {
            expr: self.trans_expr(expr),
            block: self.trans_block(block),
            br: map_ref_mut(br, |expr| self.trans_expr(expr)),
        }
    }

    #[inline]
    fn trans_if_let_expr(&mut self, pat: &rst::P<rst::Pat>, expr: &rst::Expr,
                         block: &rst::Block,
                         br: &Option<rst::P<rst::Expr>>)
    -> IfLetExpr {
        IfLetExpr {
            pat: self.trans_patten(pat),
            expr: self.trans_expr(expr),
            block: self.trans_block(block),
            br: map_ref_mut(br, |expr| self.trans_expr(expr)),
        }
    }

    #[inline]
    fn trans_while_expr(&mut self, expr: &rst::Expr, block: &rst::Block,
                        label: &Option<rst::Ident>)
    -> WhileExpr {
        WhileExpr {
            label: map_ref_mut(label, |ident| ident_to_string(ident)),
            expr: self.trans_expr(expr),
            block: self.trans_block(block),
        }
    }

    #[inline]
    fn trans_while_let_expr(&mut self, pat: &rst::P<rst::Pat>, expr: &rst::Expr,
                            block: &rst::Block, label: &Option<rst::Ident>)
    -> WhileLetExpr {
        WhileLetExpr {
            label: map_ref_mut(label, |ident| ident_to_string(ident)),
            pat: self.trans_patten(pat),
            expr: self.trans_expr(expr),
            block: self.trans_block(block),
        }
    }

    #[inline]
    fn trans_for_expr(&mut self, pat: &rst::P<rst::Pat>, expr: &rst::Expr, block: &rst::Block,
                      label: &Option<rst::Ident>)
    -> ForExpr {
        ForExpr {
            label: map_ref_mut(label, |ident| ident_to_string(ident)),
            pat: self.trans_patten(pat),
            expr: self.trans_expr(expr),
            block: self.trans_block(block),
        }
    }

    #[inline]
    fn trans_loop_expr(&mut self, block: &rst::Block, label: &Option<rst::Ident>) -> LoopExpr {
        LoopExpr {
            label: map_ref_mut(label, |ident| ident_to_string(ident)),
            block: self.trans_block(block),
        }
    }

    #[inline]
    fn trans_break_expr(&mut self, ident: &Option<rst::SpannedIdent>) -> BreakExpr {
        BreakExpr {
            label: map_ref_mut(ident, |ident| self.trans_ident(ident)),
        }
    }

    #[inline]
    fn trans_continue_expr(&mut self, ident: &Option<rst::SpannedIdent>) -> ContinueExpr {
        ContinueExpr {
            label: map_ref_mut(ident, |ident| self.trans_ident(ident)),
        }
    }

    #[inline]
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

    #[inline]
    fn trans_arm(&mut self, arm: &rst::Arm) -> Arm {
        let attrs = self.trans_attrs(&arm.attrs);
        let pats = self.trans_pattens(&arm.pats);
        let guard = map_ref_mut(&arm.guard, |expr| self.trans_expr(expr));
        let body = self.trans_expr(&arm.body);

        Arm {
            loc: Loc {
                start: pats[0].loc.start,
                end: body.loc.end,
                nl: false,
            },
            attrs: attrs,
            pats: pats,
            guard: guard,
            body: body,
        }
    }

    #[inline]
    fn trans_fn_call_expr(&mut self, fn_name: &rst::Expr, args: &Vec<rst::P<rst::Expr>>)
    -> FnCallExpr {
        FnCallExpr {
            name: self.trans_expr(fn_name),
            args: self.trans_exprs(args),
        }
    }

    #[inline]
    fn trans_method_call_expr(&mut self, ident: &rst::SpannedIdent,
                              types: &Vec<rst::P<rst::Ty>>,
                              args: &Vec<rst::P<rst::Expr>>)
    -> MethodCallExpr {
        MethodCallExpr {
            obj: self.trans_expr(&args[0]),
            name: self.trans_method_ident(ident),
            types: self.trans_types(types),
            args: self.trans_exprs(&args[1..]),
        }
    }

    #[inline]
    fn trans_closure_expr(&mut self, capture: rst::CaptureBy, fn_decl: &rst::FnDecl,
                          block: &rst::Block)
    -> ClosureExpr {
        ClosureExpr {
            moved: is_move(capture),
            fn_sig: self.trans_fn_sig(fn_decl),
            block: self.trans_block(block),
        }
    }

    #[inline]
    fn trans_return_expr(&mut self, expr: &Option<rst::P<rst::Expr>>) -> ReturnExpr {
        ReturnExpr {
            ret: map_ref_mut(expr, |expr| self.trans_expr(expr)),
        }
    }

    #[inline]
    fn trans_macro_raw(&mut self, mac: &rst::Mac) -> MacroRaw {
        MacroRaw {
            style: self.macro_style(mac),
            s: Chunk {
                loc: self.leaf_loc(&mac.span),
                s: self.span_to_snippet(mac.span).unwrap(),
            },
        }
    }

    #[inline]
    fn trans_macro_stmt(&mut self, attrs: &rst::ThinAttributes, mac: &rst::Mac) -> MacroStmt {
        let loc = self.loc(&mac.span);
        let attrs = self.trans_thin_attrs(attrs);
        let mac = self.trans_macro(mac);
        self.set_loc(&loc);

        MacroStmt {
            loc: loc,
            attrs: attrs,
            mac: mac,
        }
    }

    #[inline]
    fn trans_macro(&mut self, mac: &rst::Mac) -> Macro {
        match self.trans_macro_expr(mac) {
            Some(macro_expr) => Macro::Expr(macro_expr),
            None => Macro::Raw(self.trans_macro_raw(mac)),
        }
    }

    #[inline]
    fn trans_macro_expr(&mut self, mac: &rst::Mac) -> Option<MacroExpr> {
        let macro_exprs = self.trans_macro_exprs(&mac);
        if macro_exprs.is_none() {
            return None;
        }

        let (exprs, seps) = macro_exprs.unwrap();
        let name = path_to_string(&mac.node.path);
        let style = self.macro_style(&mac);
        let exprs = self.trans_exprs(&exprs);
        Some(MacroExpr {
            name: name,
            style: style,
            exprs: exprs,
            seps: seps,
        })
    }

    #[inline]
    fn trans_macro_exprs(&self, mac: &rst::Mac)
    -> Option<(Vec<rst::P<rst::Expr>>, Vec<MacroExprSep>)> {
        let mut exprs = Vec::new();
        let mut seps = Vec::new();

        if mac.node.tts.is_empty() {
            return Some((exprs, seps));
        }

        let mut parser
                = rst::parse::tts_to_parser(&self.sess, mac.node.tts.clone(), Vec::new());
        loop {
            exprs.push(match parser.parse_expr() {
                Ok(expr) => expr,
                Err(mut e) => {
                    e.cancel();
                    return None;
                },
            });

            match parser.token {
                rst::Token::Eof => break,
                ref other => seps.push(match token_to_macro_expr_sep(other) {
                    Some(sep) => sep,
                    None => return None,
                }),
            }

            parser.bump();
            if parser.token == rst::parse::token::Token::Eof {
                return None;
            }
        }
        Some((exprs, seps))
    }

    #[inline]
    fn macro_style(&self, mac: &rst::Mac) -> MacroStyle {
        let s = self.span_to_snippet(mac.span).unwrap();
        let paren_pos = s.find('(').unwrap_or(usize::max_value());
        let bracket_pos = s.find('[').unwrap_or(usize::max_value());
        let brace_pos = s.find('{').unwrap_or(usize::max_value());

        if paren_pos < bracket_pos && paren_pos < brace_pos {
            MacroStyle::Paren
        } else if bracket_pos < brace_pos {
            MacroStyle::Bracket
        } else {
            MacroStyle::Brace
        }
    }
}
