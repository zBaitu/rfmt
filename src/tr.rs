use std::any::Any;
use std::cmp::Ordering;
use std::collections::HashMap;

use syntax::parse::ParseSess;
use syntax::ThinVec;

use crate::ast;
use crate::ir::*;

const MAX_BLANK_LINE: u8 = 1;

fn trans_comments(cmnts: Vec<ast::Comment>) -> Vec<Comment> {
    let mut pre_blank_line_pos = 0;
    let mut blank_line = 0;

    cmnts.into_iter().fold(Vec::new(), |mut cmnts, cmnt| {
        if cmnt.style == ast::CommentStyle::BlankLine {
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
fn trans_comment(cmnt: ast::Comment) -> Comment {
    let kind = match cmnt.style {
        ast::CommentStyle::Trailing => CommentKind::Trailing,
        _ => CommentKind::Leading,
    };

    Comment {
        pos: cmnt.pos.0,
        kind,
        lines: cmnt.lines,
    }
}

#[inline]
fn span(s: u32, e: u32) -> ast::Span {
    ast::Span::new(ast::BytePos(s), ast::BytePos(e), ast::NO_EXPANSION)
}

#[inline]
fn is_inner(style: ast::AttrStyle) -> bool {
    style == ast::AttrStyle::Inner
}

#[inline]
fn symbol_to_string(symbol: &ast::Symbol) -> String {
    symbol.as_str().to_string()
}

#[inline]
fn token_lit_to_string(lit: &ast::TokenLit) -> String {
    symbol_to_string(&lit.symbol)
}

#[inline]
fn sugared_doc_to_string(tokens: &ast::TokenStream) -> String {
    let token_tree = &tokens.0.as_ref().unwrap()[1];
    if let ast::TokenTree::Token(ref token) = token_tree.0 {
        if let ast::TokenKind::Literal(ref lit) = token.kind {
            return token_lit_to_string(lit);
        }
    }
    unreachable!()
}

#[inline]
fn ident_to_string(ident: &ast::Ident) -> String {
    if (*ident).name != ast::kw::PathRoot {
        symbol_to_string(&ident.name)
    } else {
        "".to_string()
    }
}

#[inline]
fn path_to_string(path: &ast::Path) -> String {
    let mut first = true;
    path.segments.iter().fold(String::new(), |mut s, e| {
        if !first {
            s.push_str("::");
        }
        first = false;
        s.push_str(&ident_to_string(&e.ident));
        s
    })
}

#[inline]
fn is_sized(modifier: ast::TraitBoundModifier) -> bool {
    modifier == ast::TraitBoundModifier::Maybe
}

#[inline]
fn is_mut(mutbl: ast::Mutability) -> bool {
    mutbl == ast::Mutability::Mutable
}

#[inline]
fn is_dyn(syntax: ast::TraitObjectSyntax) -> bool {
    syntax == ast::TraitObjectSyntax::Dyn
}

#[inline]
fn is_unsafe(safety: ast::Unsafety) -> bool {
    safety == ast::Unsafety::Unsafe
}

#[inline]
fn is_async(asyncness: ast::IsAsync) -> bool {
    match asyncness {
        ast::IsAsync::Async { .. } => true,
        _ => false,
    }
}

#[inline]
fn is_const(constness: ast::Constness) -> bool {
    constness == ast::Constness::Const
}

#[inline]
fn is_auto(autoness: ast::IsAuto) -> bool {
    return autoness == ast::IsAuto::Yes;
}

#[inline]
fn abi_to_string(abi: ast::Abi) -> String {
    format!(r#""{:?}""#, abi)
}

/*
#[inline]
fn is_neg(polarity: ast::ImplPolarity) -> bool {
    polarity == ast::ImplPolarity::Negative
}

#[inline]
fn is_block_unsafe(rules: ast::BlockCheckMode) -> bool {
    match rules {
        ast::BlockCheckMode::Unsafe(source) => source == ast::UnsafeSource::UserProvided,
        _ => false,
    }
}

#[inline]
fn is_move(capture: ast::CaptureBy) -> bool {
    capture == ast::CaptureBy::Value
}

#[inline]
fn is_ref_mut(mode: ast::BindingMode) -> (bool, bool) {
    match mode {
        ast::BindingMode::ByRef(mutbl) => (true, is_mut(mutbl)),
        ast::BindingMode::ByValue(mutbl) => (false, is_mut(mutbl)),
    }
}

#[inline]
fn is_halfopen(range_limit: ast::RangeLimits) -> bool {
    range_limit == ast::RangeLimits::HalfOpen
}

#[inline]
fn is_default(defaultness: ast::Defaultness) -> bool {
    defaultness == ast::Defaultness::Default
}

#[inline]
fn uop_to_string(op: ast::UnOp) -> &'static str {
    ast::UnOp::to_string(op)
}

#[inline]
fn is_macro_semi(style: &ast::MacStmtStyle) -> bool {
    match *style {
        ast::MacStmtStyle::Semicolon => true,
        _ => false,
    }
}

#[inline]
fn token_to_macro_expr_sep(token: &ast::Token) -> Option<MacroExprSep> {
    let (is_sep, s) = match *token {
        ast::Token::Comma => (true, ","),
        ast::Token::Semi => (true, ";"),
        ast::Token::FatArrow => (true, " =>"),
        ast::Token::DotDotDot => (false, "..."),
        _ => return None,
    };

    Some(MacroExprSep {
        is_sep: is_sep,
        s: s,
    })
}
*/

#[inline]
fn map_ref_mut<T, F, R>(opt: &Option<T>, mut f: F) -> Option<R> where F: FnMut(&T) -> R {
    match *opt {
        Some(ref v) => Some(f(v)),
        None => None,
    }
}


pub struct TrResult {
    pub krate: Crate,
    pub leading_cmnts: HashMap<Pos, Vec<String>>,
    pub trailing_cmnts: HashMap<Pos, String>,
}

pub fn trans(sess: ParseSess, krate: ast::Crate, cmnts: Vec<ast::Comment>) -> TrResult {
    Translator::new(sess, trans_comments(cmnts)).trans_crate(krate)
}

struct Translator {
    sess: ParseSess,
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
    fn new(sess: ParseSess, cmnts: Vec<Comment>) -> Translator {
        Translator {
            sess,
            cmnts,
            cmnt_idx: 0,
            last_loc: Default::default(),
            leading_cmnts: HashMap::new(),
            trailing_cmnts: HashMap::new(),
        }
    }

    fn trans_crate(mut self, krate: ast::Crate) -> TrResult {
        self.last_loc.start = krate.span.lo().0;

        let loc = self.loc(&krate.span);
        let attrs = self.trans_attrs(&krate.attrs);
        let crate_mod_name = self.crate_mod_name();
        let module = self.trans_mod(crate_mod_name, &krate.module);

        let crate_file_end = self.crate_file_end();
        self.trans_comments(crate_file_end);

        TrResult {
            krate: Crate {
                loc,
                attrs,
                module,
            },
            leading_cmnts: self.leading_cmnts,
            trailing_cmnts: self.trailing_cmnts,
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

    fn trans_attrs(&mut self, attrs: &Vec<ast::Attribute>) -> Vec<AttrKind> {
        trans_list!(self, attrs, trans_attr_kind)
    }

    fn trans_thin_attrs(&mut self, attrs: &ThinVec<ast::Attribute>) -> Vec<AttrKind> {
        trans_list!(self, attrs, trans_attr_kind)
    }

    #[inline]
    fn trans_attr_kind(&mut self, attr: &ast::Attribute) -> AttrKind {
        if attr.is_sugared_doc {
            AttrKind::Doc(self.trans_doc(attr))
        } else {
            AttrKind::Attr(self.trans_attr(attr))
        }
    }

    fn trans_doc(&mut self, attr: &ast::Attribute) -> Doc {
        Doc {
            loc: self.leaf_loc(&attr.span),
            s: sugared_doc_to_string(&attr.tokens),
        }
    }

    fn trans_attr(&mut self, attr: &ast::Attribute) -> Attr {
        let loc = self.loc(&attr.span);
        let is_inner = is_inner(attr.style);
        let item = self.trans_meta_item(&attr.meta().unwrap());
        self.set_loc(&loc);

        Attr {
            loc,
            is_inner,
            item,
        }
    }

    fn trans_meta_item(&mut self, meta_item: &ast::MetaItem) -> MetaItem {
        let name = path_to_string(&meta_item.path);
        match meta_item.node {
            ast::MetaItemKind::Word => {
                MetaItem {
                    loc: self.leaf_loc(&meta_item.span),
                    name,
                    items: None,
                }
            }
            ast::MetaItemKind::NameValue(ref lit) => {
                let s = format!("{} = {}", name, self.literal_to_string(lit));
                MetaItem {
                    loc: self.leaf_loc(&meta_item.span),
                    name: s,
                    items: None,
                }
            }
            ast::MetaItemKind::List(ref meta_items) => {
                let loc = self.loc(&meta_item.span);
                let items = self.trans_nested_meta_items(meta_items);
                self.set_loc(&loc);

                MetaItem {
                    loc,
                    name,
                    items: Some(items),
                }
            }
        }
    }

    fn trans_nested_meta_items(&mut self, nested_meta_items: &Vec<ast::NestedMetaItem>) -> Vec<MetaItem> {
        trans_list!(self, nested_meta_items, trans_nested_meta_item)
    }

    #[inline]
    fn trans_nested_meta_item(&mut self, nested_meta_item: &ast::NestedMetaItem) -> MetaItem {
        match nested_meta_item {
            ast::NestedMetaItem::Literal(ref lit) => {
                MetaItem {
                    loc: self.leaf_loc(&nested_meta_item.span()),
                    name: self.literal_to_string(lit),
                    items: None,
                }
            }
            ast::NestedMetaItem::MetaItem(ref meta_iten) => {
                self.trans_meta_item(meta_iten)
            }
        }
    }

    fn crate_file_name(&self) -> String {
        self.sess.source_map().files().first().unwrap().name.to_string()
    }

    fn crate_mod_name(&self) -> String {
        let mut name = self.crate_file_name();
        if let Some(pos) = name.rfind('.') {
            name.truncate(pos);
        }
        name
    }

    fn trans_mod(&mut self, name: String, module: &ast::Mod) -> Mod {
        let loc = self.loc(&module.inner);
        let items = self.trans_items(&module.items);
        self.set_loc(&loc);

        Mod {
            loc,
            name,
            items,
        }
    }

    fn trans_items(&mut self, items: &Vec<ast::P<ast::Item>>) -> Vec<Item> {
        trans_list!(self, items, trans_item)
    }

    #[inline]
    fn trans_item(&mut self, item: &ast::Item) -> Item {
        let loc = self.loc(&item.span);
        let attrs = self.trans_attrs(&item.attrs);
        let vis = self.trans_vis(&item.vis);
        let ident = ident_to_string(&item.ident);
        let item = match item.node {
            ast::ItemKind::Mod(ref module) => {
                if module.inline {
                    ItemKind::Mod(self.trans_mod(ident, module))
                } else {
                    ItemKind::ModDecl(self.trans_mod_decl(ident))
                }
            }
            ast::ItemKind::ExternCrate(ref rename) => ItemKind::ExternCrate(self.trans_extren_crate(ident, rename)),
            ast::ItemKind::Use(ref tree) => ItemKind::Use(self.trans_use(tree)),
            ast::ItemKind::Ty(ref ty, ref generics) => ItemKind::TypeAlias(self.trans_type_alias(ident, generics, ty)),
            ast::ItemKind::TraitAlias(ref generics, ref bounds) => {
                ItemKind::TraitAlias(self.trans_trait_alias(ident, generics, bounds))
            }
            ast::ItemKind::Const(ref ty, ref expr) => ItemKind::Const(self.trans_const(ident, ty, expr)),
            ast::ItemKind::Static(ref ty, mutbl, ref expr) => {
                ItemKind::Static(self.trans_static(mutbl, ident, ty, expr))
            }
            ast::ItemKind::Struct(ref var, ref generics) => {
                ItemKind::Struct(self.trans_struct(ident, generics, var))
            }
            ast::ItemKind::Union(ref var, ref generics) => {
                ItemKind::Union(self.trans_union(ident, generics, var))
            }
            ast::ItemKind::Enum(ref enum_def, ref generics) => {
                ItemKind::Enum(self.trans_enum(ident, generics, enum_def))
            }
            ast::ItemKind::ForeignMod(ref module) => ItemKind::ForeignMod(self.trans_foreign_mod(module)),
            ast::ItemKind::Fn(ref decl, ref header, ref generics, ref block) => {
                ItemKind::Fn(self.trans_fn(header, ident, generics, decl, block))
            }
            ast::ItemKind::Trait(autoness, unsafety, ref generics, ref bounds, ref items) => {
                ItemKind::Trait(self.trans_trait(autoness, unsafety, ident, generics, bounds, items))
            }
            /*
            ast::ItemKind::DefaultImpl(unsafety, ref trait_ref) => {
                ItemKind::ImplDefault(self.trans_impl_default(is_unsafe(unsafety), trait_ref))
            }
            ast::ItemKind::Impl(unsafety, polarity, ref generics, ref trait_ref, ref ty,
                                ref items) => {
                ItemKind::Impl(self.trans_impl(is_unsafe(unsafety), is_neg(polarity), generics,
                                               trait_ref, ty, items))
            }
            ast::ItemKind::Mac(ref mac) => ItemKind::Macro(self.trans_macro_raw(mac)),
            */
            _ => unreachable!(),
        };

        self.set_loc(&loc);
        Item {
            loc,
            attrs,
            vis,
            item,
        }
    }

    #[inline]
    fn trans_vis(&mut self, vis: &ast::Visibility) -> Vis {
        let vis = match vis.node {
            ast::VisibilityKind::Public => "pub".to_string(),
            ast::VisibilityKind::Crate(sugar) => match sugar {
                ast::CrateSugar::PubCrate => "pub(crate)".to_string(),
                ast::CrateSugar::JustCrate => "crate".to_string(),
            }
            ast::VisibilityKind::Restricted { ref path, .. } => {
                let path = path_to_string(path);
                if path == "self" || path == "super" {
                    format!("pub({})", path)
                } else {
                    format!("pub(in {})", path)
                }
            }
            ast::VisibilityKind::Inherited => "".to_string(),
        };

        vis
    }

    fn trans_mod_decl(&mut self, ident: String) -> ModDecl {
        ModDecl {
            name: ident,
        }
    }

    fn trans_extren_crate(&mut self, ident: String, rename: &Option<ast::Symbol>) -> ExternCrate {
        let name = match *rename {
            Some(ref rename) => format!("{} as {}", symbol_to_string(rename), ident),
            None => ident,
        };

        ExternCrate {
            name,
        }
    }

    fn trans_use(&mut self, tree: &ast::UseTree) -> Use {
        let (path, trees) = match tree.kind {
            ast::UseTreeKind::Simple(rename, ..) => {
                self.leaf_loc(&tree.span);
                let mut path = path_to_string(&tree.prefix);
                if let Some(ref rename) = rename {
                    path = format!("{} as {}", path, ident_to_string(rename));
                }
                (path, None)
            }
            ast::UseTreeKind::Glob => {
                self.leaf_loc(&tree.span);
                let mut path = String::new();
                if !tree.prefix.segments.is_empty() {
                    path.push_str(&path_to_string(&tree.prefix));
                    path.push_str("::");
                }
                path.push_str("*");
                (path, None)
            }
            ast::UseTreeKind::Nested(ref trees) => {
                let loc = self.loc(&tree.span);
                let path = path_to_string(&tree.prefix);
                let mut trees = self.trans_use_trees(trees);
                self.set_loc(&loc);
                trees.sort_by(|a, b| {
                    if a.path.starts_with("self") {
                        Ordering::Less
                    } else if b.path.starts_with("self") {
                        Ordering::Greater
                    } else {
                        a.path.cmp(&b.path)
                    }
                });
                (path, Some(trees))
            }
        };

        Use {
            path,
            trees,
        }
    }

    fn trans_use_trees(&mut self, trees: &Vec<(ast::UseTree, ast::NodeId)>) -> Vec<Use> {
        trans_list!(self, trees, trans_use_tree)
    }

    #[inline]
    fn trans_use_tree(&mut self, tree: &(ast::UseTree, ast::NodeId)) -> Use {
        self.trans_use(&tree.0)
    }

    fn trans_type_alias(&mut self, ident: String, generics: &ast::Generics, ty: &ast::Ty) -> TypeAlias {
        TypeAlias {
            name: ident,
            generics: self.trans_generics(generics),
            ty: self.trans_type(ty),
        }
    }

    fn trans_trait_alias(&mut self, ident: String, generics: &ast::Generics, bounds: &ast::GenericBounds)
                         -> TraitAlias {
        TraitAlias {
            name: ident,
            generics: self.trans_generics(generics),
            bounds: self.trans_type_param_bounds(bounds),
        }
    }

    fn trans_generics(&mut self, generics: &ast::Generics) -> Generics {
        Generics {
            lifetime_defs: self.trans_lifetime_defs(&generics.params),
            type_params: self.trans_type_params(&generics.params),
            wh: self.trans_where(&generics.where_clause.predicates),
        }
    }

    fn trans_lifetime_defs(&mut self, params: &Vec<ast::GenericParam>) -> Vec<LifetimeDef> {
        params.into_iter().fold(Vec::new(), |mut lifetime_defs, param| {
            if let ast::GenericParamKind::Lifetime = param.kind {
                lifetime_defs.push(self.trans_lifetime_def(param));
            }
            lifetime_defs
        })
    }

    fn trans_lifetime_def(&mut self, param: &ast::GenericParam) -> LifetimeDef {
        let lifetime = self.trans_lifetime(&param.ident);
        LifetimeDef {
            loc: lifetime.loc,
            lifetime,
            bounds: self.trans_lifetimes(&param.bounds),
        }
    }

    fn trans_lifetime(&mut self, ident: &ast::Ident) -> Lifetime {
        Lifetime {
            loc: self.leaf_loc(&ident.span),
            s: symbol_to_string(&ident.name),
        }
    }

    fn trans_lifetimes(&mut self, bounds: &ast::GenericBounds) -> Vec<Lifetime> {
        bounds.into_iter().fold(Vec::new(), |mut lifetimes, bound| {
            if let ast::GenericBound::Outlives(ref lifetime) = bound {
                lifetimes.push(self.trans_lifetime(&lifetime.ident));
            }
            lifetimes
        })
    }

    fn trans_type_params(&mut self, params: &Vec<ast::GenericParam>) -> Vec<TypeParam> {
        params.into_iter().fold(Vec::new(), |mut type_params, param| {
            if let ast::GenericParamKind::Type { .. } = param.kind {
                type_params.push(self.trans_type_param(param));
            }
            type_params
        })
    }

    fn trans_type_param(&mut self, param: &ast::GenericParam) -> TypeParam {
        let loc = self.loc(&param.ident.span);
        let name = ident_to_string(&param.ident);
        let bounds = self.trans_type_param_bounds(&param.bounds);
        let default = match param.kind {
            ast::GenericParamKind::Type { ref default } => map_ref_mut(default, |ty| self.trans_type(ty)),
            _ => None,
        };
        self.set_loc(&loc);

        TypeParam {
            loc,
            name,
            bounds,
            default,
        }
    }

    fn trans_type_param_bounds(&mut self, bounds: &ast::GenericBounds) -> Vec<TypeParamBound> {
        trans_list!(self, bounds, trans_type_param_bound)
    }

    #[inline]
    fn trans_type_param_bound(&mut self, bound: &ast::GenericBound) -> TypeParamBound {
        match *bound {
            ast::GenericBound::Outlives(ref lifetime) => {
                TypeParamBound::Lifetime(self.trans_lifetime(&lifetime.ident))
            }
            ast::GenericBound::Trait(ref poly_trait_ref, modifier) => {
                TypeParamBound::PolyTraitRef(self.trans_poly_trait_ref(poly_trait_ref, modifier))
            }
        }
    }

    fn trans_poly_trait_ref(&mut self, poly_trait_ref: &ast::PolyTraitRef, modifier: ast::TraitBoundModifier)
                            -> PolyTraitRef {
        if is_sized(modifier) {
            return PolyTraitRef::new_sized(self.leaf_loc(&poly_trait_ref.span));
        }

        let loc = self.loc(&poly_trait_ref.span);
        let lifetime_defs = self.trans_lifetime_defs(&poly_trait_ref.bound_generic_params);
        let trait_ref = self.trans_trait_ref(&poly_trait_ref.trait_ref);
        self.set_loc(&loc);

        PolyTraitRef {
            loc,
            lifetime_defs,
            trait_ref,
        }
    }

    fn trans_trait_ref(&mut self, trait_ref: &ast::TraitRef) -> TraitRef {
        self.trans_path(&trait_ref.path)
    }

    fn trans_where(&mut self, predicates: &Vec<ast::WherePredicate>) -> Where {
        Where {
            clauses: self.trans_where_clauses(predicates),
        }
    }

    fn trans_where_clauses(&mut self, predicates: &Vec<ast::WherePredicate>) -> Vec<WhereClause> {
        trans_list!(self, predicates, trans_where_clause)
    }

    #[inline]
    fn trans_where_clause(&mut self, predicate: &ast::WherePredicate) -> WhereClause {
        match *predicate {
            ast::WherePredicate::RegionPredicate(ref region) => self.trans_where_lifetime(region),
            ast::WherePredicate::BoundPredicate(ref bound) => self.trans_where_bound(bound),
            _ => unreachable!(),
        }
    }

    fn trans_where_lifetime(&mut self, region: &ast::WhereRegionPredicate) -> WhereClause {
        let loc = self.loc(&region.span);
        let lifetime = self.trans_lifetime(&region.lifetime.ident);
        let bounds = self.trans_lifetimes(&region.bounds);
        self.set_loc(&loc);

        WhereClause {
            loc,
            clause: WhereKind::LifetimeDef(LifetimeDef {
                loc: lifetime.loc,
                lifetime,
                bounds,
            }),
        }
    }

    fn trans_where_bound(&mut self, bound: &ast::WhereBoundPredicate) -> WhereClause {
        let loc = self.loc(&bound.span);
        let lifetime_defs = self.trans_lifetime_defs(&bound.bound_generic_params);
        let ty = self.trans_type(&bound.bounded_ty);
        let bounds = self.trans_type_param_bounds(&bound.bounds);
        self.set_loc(&loc);

        WhereClause {
            loc,
            clause: WhereKind::Bound(WhereBound {
                lifetime_defs,
                ty,
                bounds,
            }),
        }
    }

    fn trans_path(&mut self, path: &ast::Path) -> Path {
        let loc = self.loc(&path.span);
        let segments = self.trans_path_segments(&path.segments);
        self.set_loc(&loc);

        Path {
            loc,
            segments,
        }
    }

    fn trans_path_segments(&mut self, segments: &Vec<ast::PathSegment>) -> Vec<PathSegment> {
        trans_list!(self, segments, trans_path_segment)
    }

    #[inline]
    fn trans_path_segment(&mut self, seg: &ast::PathSegment) -> PathSegment {
        PathSegment {
            name: ident_to_string(&seg.ident),
            param: self.trans_generics_args_or_none(&seg.args),
        }
    }

    fn trans_generics_args_or_none(&mut self, args: &Option<ast::P<ast::GenericArgs>>) -> PathParam {
        match *args {
            Some(ref args) => self.trans_generic_args(args),
            None => PathParam::Angle(Default::default()),
        }
    }

    fn trans_generic_args(&mut self, args: &ast::GenericArgs) -> PathParam {
        match *args {
            ast::AngleBracketed(ref param) => PathParam::Angle(self.trans_angle_param(param)),
            ast::Parenthesized(ref param) => PathParam::Paren(self.trans_paren_param(param)),
        }
    }

    fn trans_angle_param(&mut self, param: &ast::AngleBracketedArgs) -> AngleParam {
        AngleParam {
            lifetimes: self.trans_generic_args_to_lifetime(&param.args),
            types: self.trans_generic_args_to_types(&param.args),
            bindings: self.trans_type_bindings(&param.constraints),
        }
    }

    fn trans_generic_args_to_lifetime(&mut self, args: &Vec<ast::GenericArg>) -> Vec<Lifetime> {
        args.into_iter().fold(Vec::new(), |mut lifetimes, arg| {
            if let ast::GenericArg::Lifetime(ref lifetime) = arg {
                lifetimes.push(self.trans_lifetime(&lifetime.ident));
            }
            lifetimes
        })
    }

    fn trans_generic_args_to_types(&mut self, args: &Vec<ast::GenericArg>) -> Vec<Type> {
        args.into_iter().fold(Vec::new(), |mut types, arg| {
            if let ast::GenericArg::Type(ref ty) = arg {
                types.push(self.trans_type(ty));
            }
            types
        })
    }

    fn trans_type_bindings(&mut self, bindings: &Vec<ast::AssocTyConstraint>) -> Vec<TypeBinding> {
        trans_list!(self, bindings, trans_type_binding)
    }

    #[inline]
    fn trans_type_binding(&mut self, binding: &ast::AssocTyConstraint) -> TypeBinding {
        let loc = self.loc(&binding.span);
        let name = ident_to_string(&binding.ident);
        let binding = match binding.kind {
            ast::AssocTyConstraintKind::Equality { ref ty } =>
                TypeBindingKind::Eq(self.trans_type(ty)),
            ast::AssocTyConstraintKind::Bound { ref bounds } =>
                TypeBindingKind::Bound(self.trans_type_param_bounds(bounds)),
        };
        self.set_loc(&loc);

        TypeBinding {
            loc,
            name,
            binding,
        }
    }

    fn trans_paren_param(&mut self, args: &ast::ParenthesizedArgs) -> ParenParam {
        let loc = self.loc(&args.span);
        let inputs = self.trans_types(&args.inputs);
        let output = map_ref_mut(&args.output, |ty| self.trans_type(ty));
        self.set_loc(&loc);

        ParenParam {
            loc,
            inputs,
            output,
        }
    }

    fn trans_qself(&mut self, qself: &ast::QSelf) -> QSelf {
        QSelf {
            ty: self.trans_type(&qself.ty),
            pos: qself.position,
        }
    }

    fn trans_types(&mut self, types: &Vec<ast::P<ast::Ty>>) -> Vec<Type> {
        trans_list!(self, types, trans_type)
    }

    #[inline]
    fn trans_type(&mut self, ty: &ast::Ty) -> Type {
        let loc = self.loc(&ty.span);

        let ty = match ty.node {
            ast::TyKind::Infer => TypeKind::Symbol("_"),
            ast::TyKind::Never => TypeKind::Symbol("!"),
            ast::TyKind::CVarArgs => TypeKind::Symbol("..."),
            ast::TyKind::ImplicitSelf => TypeKind::Symbol("self"),
            ast::TyKind::Path(ref qself, ref path) => TypeKind::Path(Box::new(self.trans_path_type(qself, path))),
            ast::TyKind::Ptr(ref mut_type) => TypeKind::Ptr(Box::new(self.trans_ptr_type(mut_type))),
            ast::TyKind::Rptr(ref lifetime, ref mut_type) => {
                TypeKind::Ref(Box::new(self.trans_ref_type(lifetime, mut_type)))
            }
            ast::TyKind::Tup(ref types) => TypeKind::Tuple(Box::new(self.trans_tuple_type(types))),
            ast::TyKind::Paren(ref ty) => TypeKind::Tuple(Box::new(self.trans_tuple_type(&vec![ty.clone()]))),
            ast::TyKind::Slice(ref ty) => TypeKind::Slice(Box::new(self.trans_slice_type(ty))),
            ast::TyKind::Array(ref ty, ref ac) => TypeKind::Array(Box::new(self.trans_array_type(ty, &ac.value))),
            ast::TyKind::TraitObject(ref bounds, syntax) => {
                TypeKind::Trait(Box::new(self.trans_trait_type(is_dyn(syntax), false, bounds)))
            }
            ast::TyKind::ImplTrait(_, ref bounds) => {
                TypeKind::Trait(Box::new(self.trans_trait_type(false, true, bounds)))
            }
            ast::TyKind::BareFn(ref bare_fn) => {
                TypeKind::BareFn(Box::new(self.trans_bare_fn_type(bare_fn)))
            }
            /*
            ast::TyKind::Mac(ref mac) => TypeKind::Macro(Box::new(self.trans_macro(mac))),
            */
            _ => {
                println!("{:#?}", ty.node);
                unreachable!()
            }
        };

        self.set_loc(&loc);
        Type {
            loc,
            ty,
        }
    }

    fn trans_path_type(&mut self, qself: &Option<ast::QSelf>, path: &ast::Path) -> PathType {
        PathType {
            qself: map_ref_mut(qself, |qself| self.trans_qself(qself)),
            path: self.trans_path(path),
        }
    }

    fn trans_ptr_type(&mut self, mut_type: &ast::MutTy) -> PtrType {
        PtrType {
            is_mut: is_mut(mut_type.mutbl),
            ty: self.trans_type(&mut_type.ty),
        }
    }

    fn trans_ref_type(&mut self, lifetime: &Option<ast::Lifetime>, mut_type: &ast::MutTy) -> RefType {
        RefType {
            lifetime: map_ref_mut(lifetime, |lifetime| self.trans_lifetime(&lifetime.ident)),
            is_mut: is_mut(mut_type.mutbl),
            ty: self.trans_type(&mut_type.ty),
        }
    }

    fn trans_tuple_type(&mut self, types: &Vec<ast::P<ast::Ty>>) -> TupleType {
        TupleType {
            types: self.trans_types(types),
        }
    }

    fn trans_slice_type(&mut self, ty: &ast::Ty) -> SliceType {
        SliceType {
            ty: self.trans_type(ty),
        }
    }

    fn trans_array_type(&mut self, ty: &ast::Ty, expr: &ast::Expr) -> ArrayType {
        ArrayType {
            ty: self.trans_type(ty),
            expr: self.trans_expr(expr),
        }
    }

    fn trans_trait_type(&mut self, is_dyn: bool, is_impl: bool, bounds: &ast::GenericBounds) -> TraitType {
        TraitType {
            is_dyn,
            is_impl,
            bounds: self.trans_type_param_bounds(bounds),
        }
    }

    fn trans_const(&mut self, ident: String, ty: &ast::Ty, expr: &ast::Expr) -> Const {
        Const {
            name: ident,
            ty: self.trans_type(ty),
            expr: self.trans_expr(expr),
        }
    }

    fn trans_static(&mut self, mutbl: ast::Mutability, ident: String, ty: &ast::Ty, expr: &ast::Expr)
                    -> Static {
        Static {
            is_mut: is_mut(mutbl),
            name: ident,
            ty: self.trans_type(ty),
            expr: self.trans_expr(expr),
        }
    }

    fn trans_struct(&mut self, ident: String, generics: &ast::Generics, var: &ast::VariantData) -> Struct {
        Struct {
            name: ident,
            generics: self.trans_generics(generics),
            body: self.trans_struct_body(var),
        }
    }

    fn trans_struct_body(&mut self, var: &ast::VariantData) -> StructBody {
        match *var {
            ast::VariantData::Struct(ref fields, _) => {
                StructBody::Struct(self.trans_struct_fields(fields))
            }
            ast::VariantData::Tuple(ref fields, _) => {
                StructBody::Tuple(self.trans_tuple_fields(fields))
            }
            ast::VariantData::Unit(_) => StructBody::Unit,
        }
    }

    fn trans_struct_fields(&mut self, fields: &Vec<ast::StructField>) -> Vec<StructField> {
        trans_list!(self, fields, trans_struct_field)
    }

    #[inline]
    fn trans_struct_field(&mut self, field: &ast::StructField) -> StructField {
        let loc = self.loc(&field.span);
        let attrs = self.trans_attrs(&field.attrs);
        let vis = self.trans_vis(&field.vis);
        let name = ident_to_string(&field.ident.unwrap());
        let ty = self.trans_type(&field.ty);
        self.set_loc(&loc);

        StructField {
            loc,
            attrs,
            vis,
            name,
            ty,
        }
    }

    fn trans_tuple_fields(&mut self, fields: &Vec<ast::StructField>) -> Vec<TupleField> {
        trans_list!(self, fields, trans_tuple_field)
    }

    #[inline]
    fn trans_tuple_field(&mut self, field: &ast::StructField) -> TupleField {
        let loc = self.loc(&field.span);
        let attrs = self.trans_attrs(&field.attrs);
        let vis = self.trans_vis(&field.vis);
        let ty = self.trans_type(&field.ty);
        self.set_loc(&loc);

        TupleField {
            loc,
            attrs,
            vis,
            ty,
        }
    }

    fn trans_union(&mut self, ident: String, generics: &ast::Generics, var: &ast::VariantData) -> Union {
        let fields = match *var {
            ast::VariantData::Struct(ref fields, _) => {
                self.trans_struct_fields(fields)
            }
            _ => unreachable!(),
        };

        Union {
            name: ident,
            generics: self.trans_generics(generics),
            fields,
        }
    }

    fn trans_enum(&mut self, ident: String, generics: &ast::Generics, enum_def: &ast::EnumDef) -> Enum {
        Enum {
            name: ident,
            generics: self.trans_generics(generics),
            body: self.trans_enum_body(enum_def),
        }
    }

    fn trans_enum_body(&mut self, enum_def: &ast::EnumDef) -> EnumBody {
        EnumBody {
            fields: self.trans_enum_fields(&enum_def.variants),
        }
    }

    fn trans_enum_fields(&mut self, vars: &Vec<ast::Variant>) -> Vec<EnumField> {
        trans_list!(self, vars, trans_enum_field)
    }

    #[inline]
    fn trans_enum_field(&mut self, var: &ast::Variant) -> EnumField {
        let loc = self.loc(&var.span);
        let attrs = self.trans_attrs(&var.node.attrs);
        let name = ident_to_string(&var.node.ident);
        let body = self.trans_struct_body(&var.node.data);
        let expr = map_ref_mut(&var.node.disr_expr, |ac| self.trans_expr(&ac.value));
        self.set_loc(&loc);

        EnumField {
            loc,
            attrs,
            name,
            body,
            expr,
        }
    }

    fn trans_bare_fn_type(&mut self, bare_fn: &ast::BareFnTy) -> BareFnType {
        BareFnType {
            lifetime_defs: self.trans_lifetime_defs(&bare_fn.generic_params),
            is_unsafe: is_unsafe(bare_fn.unsafety),
            abi: abi_to_string(bare_fn.abi),
            sig: self.trans_fn_sig(&bare_fn.decl),
        }
    }

    fn trans_fn_sig(&mut self, decl: &ast::FnDecl) -> FnSig {
        FnSig {
            args: self.trans_args(&decl.inputs),
            ret: self.trans_return(&decl.output),
        }
    }

    fn trans_args(&mut self, inputs: &Vec<ast::Arg>) -> Vec<Arg> {
        trans_list!(self, inputs, trans_arg)
    }

    #[inline]
    fn trans_arg(&mut self, arg: &ast::Arg) -> Arg {
        let pat = self.trans_patten(&arg.pat);
        Arg {
            loc: pat.loc.clone(),
            pat,
            ty: self.trans_type(&arg.ty),
        }
    }

    fn trans_return(&mut self, output: &ast::FunctionRetTy) -> Return {
        let (nl, ret) = match *output {
            ast::FunctionRetTy::Default(_) => (false, None),
            ast::FunctionRetTy::Ty(ref ty) => (self.is_return_nl(ty.span.lo().0), Some(self.trans_type(ty))),
        };

        Return {
            nl,
            ret,
        }
    }

    fn is_return_nl(&self, pos: Pos) -> bool {
        let snippet = self.span_to_snippet(span(self.last_loc.end, pos)).unwrap();
        let right_arrow_pos = self.last_loc.end + snippet.find("->").unwrap() as Pos;
        self.is_nl(right_arrow_pos)
    }

    fn trans_foreign_mod(&mut self, module: &ast::ForeignMod) -> ForeignMod {
        ForeignMod {
            abi: abi_to_string(module.abi),
            items: self.trans_foreign_items(&module.items),
        }
    }

    fn trans_foreign_items(&mut self, items: &Vec<ast::ForeignItem>) -> Vec<ForeignItem> {
        trans_list!(self, items, trans_foreign_item)
    }

    #[inline]
    fn trans_foreign_item(&mut self, item: &ast::ForeignItem) -> ForeignItem {
        let loc = self.loc(&item.span);
        let attrs = self.trans_attrs(&item.attrs);
        let vis = self.trans_vis(&item.vis);
        let ident = ident_to_string(&item.ident);
        let item = match item.node {
            ast::ForeignItemKind::Ty => ForeignKind::Type(ident),
            ast::ForeignItemKind::Static(ref ty, mutbl) => {
                ForeignKind::Static(self.trans_foreign_static(mutbl, ident, ty))
            }
            ast::ForeignItemKind::Fn(ref decl, ref generics) => {
                ForeignKind::Fn(self.trans_foreign_fn(ident, generics, decl))
            }
            //ast::ForeignItemKind::Macro(_) => {}
            _ => unreachable!(),
        };
        self.set_loc(&loc);

        ForeignItem {
            loc,
            attrs,
            vis,
            item,
        }
    }

    fn trans_foreign_static(&mut self, mutbl: ast::Mutability, ident: String, ty: &ast::Ty) -> ForeignStatic {
        ForeignStatic {
            is_mut: is_mut(mutbl),
            name: ident,
            ty: self.trans_type(ty),
        }
    }

    fn trans_foreign_fn(&mut self, ident: String, generics: &ast::Generics, decl: &ast::FnDecl) -> ForeignFn {
        ForeignFn {
            name: ident,
            generics: self.trans_generics(generics),
            sig: self.trans_fn_sig(decl),
        }
    }

    fn trans_fn_header(&mut self, header: &ast::FnHeader) -> FnHeader {
        FnHeader {
            is_unsafe: is_unsafe(header.unsafety),
            is_async: is_async(header.asyncness.node),
            is_const: is_const(header.constness.node),
            abi: abi_to_string(header.abi),
        }
    }

    fn trans_fn(&mut self, header: &ast::FnHeader, ident: String, generics: &ast::Generics,
                decl: &ast::FnDecl, block: &ast::Block) -> Fn {
        Fn {
            header: self.trans_fn_header(header),
            name: ident,
            generics: self.trans_generics(generics),
            sig: self.trans_fn_sig(decl),
            //block: self.trans_block(block),
        }
    }

    fn trans_trait(&mut self, autoness: ast::IsAuto, unsafety: ast::Unsafety, ident: String,
                   generics: &ast::Generics, bounds: &ast::GenericBounds, items: &Vec<ast::TraitItem>) -> Trait {
        Trait {
            is_auto: is_auto(autoness),
            is_unsafe: is_unsafe(unsafety),
            name: ident,
            generics: self.trans_generics(generics),
            bounds: self.trans_type_param_bounds(bounds),
            items: self.trans_trait_items(items),
        }
    }

    fn trans_trait_items(&mut self, items: &Vec<ast::TraitItem>) -> Vec<TraitItem> {
        trans_list!(self, items, trans_trait_item)
    }

    #[inline]
    fn trans_trait_item(&mut self, item: &ast::TraitItem) -> TraitItem {
        let loc = self.loc(&item.span);
        let attrs = self.trans_attrs(&item.attrs);
        let ident = ident_to_string(&item.ident);
        let generics = self.trans_generics(&item.generics);
        let item = match item.node {
            ast::TraitItemKind::Const(ref ty, ref expr) => {
                TraitItemKind::Const(self.trans_const_trait_item(ident, ty, expr))
            }
            ast::TraitItemKind::Type(ref bounds, ref ty) => {
                TraitItemKind::Type(self.trans_type_trait_item(ident, bounds, ty))
            }
            ast::TraitItemKind::Method(ref sig, ref block) => {
                TraitItemKind::Method(self.trans_method_trait_item(ident, sig, block))
            }
            _ => unreachable!(),
        };
        self.set_loc(&loc);

        TraitItem {
            loc,
            attrs,
            generics,
            item,
        }
    }

    fn trans_const_trait_item(&mut self, ident: String, ty: &ast::Ty, expr: &Option<ast::P<ast::Expr>>)
                              -> ConstTraitItem {
        ConstTraitItem {
            name: ident,
            ty: self.trans_type(ty),
            expr: map_ref_mut(expr, |expr| self.trans_expr(expr)),
        }
    }

    fn trans_type_trait_item(&mut self, ident: String, bounds: &ast::GenericBounds, ty: &Option<ast::P<ast::Ty>>)
                             -> TypeTraitItem {
        TypeTraitItem {
            name: ident,
            bounds: self.trans_type_param_bounds(bounds),
            ty: map_ref_mut(ty, |ty| self.trans_type(ty)),
        }
    }

    fn trans_method_trait_item(&mut self, ident: String, sig: &ast::MethodSig,
                               block: &Option<ast::P<ast::Block>>)
                               -> MethodTraitItem {
        MethodTraitItem {
            sig: self.trans_method_sig(ident, sig),
        }
    }

    fn trans_method_sig(&mut self, ident: String, sig: &ast::MethodSig) -> MethodSig {
        MethodSig {
            header: self.trans_fn_header(&sig.header),
            name: ident,
            sig: self.trans_fn_sig(&sig.decl),
        }
    }

    /*
    fn trans_impl_default(&mut self, is_unsafe: bool, trait_ref: &ast::TraitRef) -> ImplDefault {
        ImplDefault {
            is_unsafe: is_unsafe,
            trait_ref: self.trans_trait_ref(trait_ref),
        }
    }

    fn trans_impl(&mut self, is_unsafe: bool, is_neg: bool, generics: &ast::Generics,
                  trait_ref: &Option<ast::TraitRef>, ty: &ast::Ty, items: &Vec<ast::ImplItem>)
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

    fn trans_impl_items(&mut self, items: &Vec<ast::ImplItem>) -> Vec<ImplItem> {
        trans_list!(self, items, trans_impl_item)
    }

    fn trans_impl_item(&mut self, item: &ast::ImplItem) -> ImplItem {
        let loc = self.loc(&item.span);
        let attrs = self.trans_attrs(&item.attrs);

        let is_pub = is_pub(&item.vis);
        let is_default = is_default(item.defaultness);
        let ident = ident_to_string(&item.ident);
        let item = match item.node {
            ast::ImplItemKind::Const(ref ty, ref expr) => {
                ImplItemKind::Const(self.trans_const_impl_item(ident, ty, expr))
            }
            ast::ImplItemKind::Type(ref ty) => {
                ImplItemKind::Type(self.trans_type_impl_item(ident, ty))
            }
            ast::ImplItemKind::Method(ref method_sig, ref block) => {
                ImplItemKind::Method(self.trans_method_impl_item(ident, method_sig, block))
            }
            ast::ImplItemKind::Macro(ref mac) => ImplItemKind::Macro(self.trans_macro(mac)),
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

    fn trans_const_impl_item(&mut self, ident: String, ty: &ast::Ty, expr: &ast::Expr)
                             -> ConstImplItem {
        ConstImplItem {
            name: ident,
            ty: self.trans_type(ty),
            expr: self.trans_expr(expr),
        }
    }

    fn trans_type_impl_item(&mut self, ident: String, ty: &ast::Ty) -> TypeImplItem {
        TypeImplItem {
            name: ident,
            ty: self.trans_type(ty),
        }
    }

    fn trans_method_impl_item(&mut self, ident: String, method_sig: &ast::MethodSig,
                              block: &ast::P<ast::Block>)
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


    fn trans_method_sig(&mut self, method_sig: &ast::MethodSig) -> MethodSig {
        MethodSig {
            generics: self.trans_generics(&method_sig.generics),
            sf: self.trans_self(&method_sig.explicit_self.node, &method_sig.decl),
            fn_sig: self.trans_fn_sig(&method_sig.decl),
        }
    }

    fn trans_self(&mut self, self_kind: &ast::SelfKind, fn_sig: &ast::FnDecl) -> Option<Sf> {
        match *self_kind {
            ast::SelfKind::Static => None,
            ast::SelfKind::Value(_) => {
                let arg = &fn_sig.inputs[0];
                let is_mut = match arg.pat.node {
                    ast::PatKind::Ident(mode, _, _) => {
                        let (_, is_mut) = is_ref_mut(mode);
                        is_mut
                    }
                    _ => unreachable!(),
                };
                let sf = if is_mut {
                    "mut self"
                } else {
                    "self"
                }.to_string();
                Some(Sf::String(sf))
            }
            ast::SelfKind::Region(lifetime, mutbl, _) => {
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
            ast::SelfKind::Explicit(ref ty, _) => Some(Sf::Type(self.trans_type(ty))),
        }
    }

    fn trans_block(&mut self, block: &ast::Block) -> Block {
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

    fn trans_stmts(&mut self, stmts: &Vec<ast::Stmt>) -> Vec<Stmt> {
        trans_list!(self, stmts, trans_stmt)
    }

    #[inline]
    fn trans_stmt(&mut self, stmt: &ast::Stmt) -> Stmt {
        let loc = self.loc(&stmt.span);
        let stmt = match stmt.node {
            ast::StmtKind::Decl(ref decl, _) => StmtKind::Decl(self.trans_decl(decl)),
            ast::StmtKind::Semi(ref expr, _) => StmtKind::Expr(self.trans_expr(expr), true),
            ast::StmtKind::Expr(ref expr, _) => StmtKind::Expr(self.trans_expr(expr), false),
            ast::StmtKind::Mac(ref mac, ref style, ref attrs) => {
                StmtKind::Macro(self.trans_macro_stmt(attrs, mac), is_macro_semi(style))
            }
        };
        self.set_loc(&loc);

        Stmt {
            loc: loc,
            stmt: stmt,
        }
    }

    #[inline]
    fn trans_decl(&mut self, decl: &ast::Decl) -> Decl {
        let loc = self.loc(&decl.span);
        let decl = match decl.node {
            ast::DeclKind::Local(ref local) => DeclKind::Local(self.trans_local(local)),
            ast::DeclKind::Item(ref item) => DeclKind::Item(self.trans_item(item)),
        };
        self.set_loc(&loc);

        Decl {
            loc: loc,
            decl: decl,
        }
    }

    #[inline]
    fn trans_local(&mut self, local: &ast::Local) -> Local {
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
    */

    fn trans_pattens(&mut self, pats: &Vec<ast::P<ast::Pat>>) -> Vec<Patten> {
        trans_list!(self, pats, trans_patten)
    }

    #[inline]
    fn trans_patten(&mut self, pat: &ast::P<ast::Pat>) -> Patten {
        let loc = self.loc(&pat.span);
        /*
        let pat = match pat.node {
            ast::PatKind::Wild => PattenKind::Wildcard,
            ast::PatKind::Lit(ref expr) => PattenKind::Literal(self.trans_expr(expr)),
            ast::PatKind::Range(ref start, ref end) => {
                PattenKind::Range(self.trans_range_patten(start, end))
            }
            ast::PatKind::Ident(mode, ref ident, ref binding) => {
                PattenKind::Ident(Box::new(self.trans_ident_patten(mode, ident, binding)))
            }
            ast::PatKind::Ref(ref pat, mutbl) => {
                PattenKind::Ref(Box::new(self.trans_ref_patten(is_mut(mutbl), pat)))
            }
            ast::PatKind::QPath(ref qself, ref path) => {
                PattenKind::Path(self.trans_path_patten(Some(qself), path))
            }
            ast::PatKind::Path(ref path) => {
                PattenKind::Path(self.trans_path_patten(None, path))
            }
            ast::PatKind::TupleStruct(ref path, ref pats) => {
                PattenKind::Enum(self.trans_enum_patten(path, pats))
            }
            ast::PatKind::Struct(ref path, ref fields, etc) => {
                PattenKind::Struct(Box::new(self.trans_struct_patten(path, fields, etc)))
            }
            ast::PatKind::Vec(ref start, ref emit, ref end) => {
                PattenKind::Vec(Box::new(self.trans_vec_patten(start, emit, end)))
            }
            ast::PatKind::Tup(ref pats)
            => PattenKind::Tuple(Box::new(self.trans_tuple_patten(pats))),
            ast::PatKind::Box(ref pat) => PattenKind::Box(Box::new(self.trans_patten(pat))),
            ast::PatKind::Mac(ref mac) => PattenKind::Macro(self.trans_macro(mac)),
        };
        */
        self.set_loc(&loc);

        Patten {
            loc,
            s: self.span_to_snippet(pat.span).unwrap(),
        }
    }

    /*
    #[inline]
    fn trans_range_patten(&mut self, start: &ast::Expr, end: &ast::Expr) -> RangePatten {
        RangePatten {
            start: self.trans_expr(start),
            end: self.trans_expr(end),
        }
    }

    #[inline]
    fn trans_ident_patten(&mut self, mode: ast::BindingMode, ident: &ast::SpannedIdent,
                          binding: &Option<ast::P<ast::Pat>>)
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
    fn trans_ref_patten(&mut self, is_mut: bool, pat: &ast::P<ast::Pat>) -> RefPatten {
        RefPatten {
            is_mut: is_mut,
            pat: self.trans_patten(pat),
        }
    }

    #[inline]
    fn trans_path_patten(&mut self, qself: Option<&ast::QSelf>, path: &ast::Path) -> PathPatten {
        PathPatten {
            qself: map_ref_mut(&qself, |qself| self.trans_qself(qself)),
            path: self.trans_path(path),
        }
    }

    #[inline]
    fn trans_enum_patten(&mut self, path: &ast::Path, pats: &Option<Vec<ast::P<ast::Pat>>>)
                         -> EnumPatten {
        EnumPatten {
            path: self.trans_path(path),
            pats: map_ref_mut(pats, |pats| self.trans_pattens(pats)),
        }
    }

    #[inline]
    fn trans_struct_patten(&mut self, path: &ast::Path,
                           fields: &Vec<ast::Spanned<ast::FieldPat>>,
                           etc: bool)
                           -> StructPatten {
        StructPatten {
            path: self.trans_path(path),
            fields: self.trans_struct_field_pattens(fields),
            etc: etc,
        }
    }

    #[inline]
    fn trans_struct_field_pattens(&mut self, fields: &Vec<ast::Spanned<ast::FieldPat>>)
                                  -> Vec<StructFieldPatten> {
        trans_list!(self, fields, trans_struct_field_patten)
    }

    #[inline]
    fn trans_struct_field_patten(&mut self, field: &ast::Spanned<ast::FieldPat>)
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
    fn trans_vec_patten(&mut self, start: &Vec<ast::P<ast::Pat>>,
                        emit: &Option<ast::P<ast::Pat>>,
                        end: &Vec<ast::P<ast::Pat>>)
                        -> VecPatten {
        VecPatten {
            start: self.trans_pattens(start),
            emit: map_ref_mut(emit, |pat| self.trans_patten(pat)),
            end: self.trans_pattens(end),
        }
    }

    #[inline]
    fn trans_tuple_patten(&mut self, pats: &Vec<ast::P<ast::Pat>>) -> TuplePatten {
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
    */

    fn trans_exprs(&mut self, exprs: &[ast::P<ast::Expr>]) -> Vec<Expr> {
        trans_list!(self, exprs, trans_expr)
    }

    fn trans_expr(&mut self, expr: &ast::Expr) -> Expr {
        let loc = self.loc(&expr.span);
        let attrs = self.trans_thin_attrs(&expr.attrs);
        /*
        let expr = match expr.node {
            ast::ExprKind::Lit(ref lit) => ExprKind::Literal(self.trans_literal_expr(lit)),
            ast::ExprKind::Path(ref qself, ref path)
            => ExprKind::Path(self.trans_path_type(qself, path)),
            ast::ExprKind::Unary(op, ref expr) => {
                ExprKind::Unary(Box::new(self.trans_unary_expr(op, expr)))
            }
            ast::ExprKind::AddrOf(mutble, ref expr) => {
                ExprKind::Ref(Box::new(self.trans_ref_expr(mutble, expr)))
            }
            ast::ExprKind::Binary(ref op, ref left, ref right) => {
                ExprKind::List(Box::new(self.trans_binary_expr(left, op, right)))
            }
            ast::ExprKind::Assign(ref left, ref right) => {
                ExprKind::List(Box::new(self.trans_assign_expr(left, right)))
            }
            ast::ExprKind::AssignOp(ref op, ref left, ref right) => {
                ExprKind::List(Box::new(self.trans_op_assign_expr(left, op, right)))
            }
            ast::ExprKind::InPlace(ref left, ref right) => {
                ExprKind::List(Box::new(self.trans_in_place_expr(left, right)))
            }
            ast::ExprKind::Repeat(ref init, ref len) => {
                ExprKind::FixedSizeArray(Box::new(self.trans_fixed_size_array_expr(init, len)))
            }
            ast::ExprKind::Vec(ref exprs) => ExprKind::Vec(Box::new(self.trans_exprs(exprs))),
            ast::ExprKind::Tup(ref exprs) => ExprKind::Tuple(Box::new(self.trans_exprs(exprs))),
            ast::ExprKind::Paren(ref expr)
            => ExprKind::Tuple(Box::new(vec![self.trans_expr(expr)])),
            ast::ExprKind::Field(ref expr, ref ident) => {
                ExprKind::FieldAccess(Box::new(self.trans_struct_field_access_expr(expr, ident)))
            }
            ast::ExprKind::TupField(ref expr, ref pos) => {
                ExprKind::FieldAccess(Box::new(self.trans_tuple_field_access_expr(expr, pos)))
            }
            ast::ExprKind::Struct(ref path, ref fields, ref base) => {
                ExprKind::Struct(Box::new(self.trans_struct_expr(path, fields, base)))
            }
            ast::ExprKind::Index(ref obj, ref index) => {
                ExprKind::Index(Box::new(self.trans_index_expr(obj, index)))
            }
            ast::ExprKind::Range(ref start, ref end, limit) => {
                ExprKind::Range(Box::new(self.trans_range_expr(start, end, limit)))
            }
            ast::ExprKind::Box(ref expr) => ExprKind::Box(Box::new(self.trans_box_expr(expr))),
            ast::ExprKind::Cast(ref expr, ref ty) => {
                ExprKind::Cast(Box::new(self.trans_cast_expr(expr, ty)))
            }
            ast::ExprKind::Type(ref expr, ref ty) => {
                ExprKind::Type(Box::new(self.trans_type_expr(expr, ty)))
            }
            ast::ExprKind::Block(ref block)
            => ExprKind::Block(Box::new(self.trans_block(block))),
            ast::ExprKind::If(ref expr, ref block, ref br) => {
                ExprKind::If(Box::new(self.trans_if_expr(expr, block, br)))
            }
            ast::ExprKind::IfLet(ref pat, ref expr, ref block, ref br) => {
                ExprKind::IfLet(Box::new(self.trans_if_let_expr(pat, expr, block, br)))
            }
            ast::ExprKind::While(ref expr, ref block, ref label) => {
                ExprKind::While(Box::new(self.trans_while_expr(expr, block, label)))
            }
            ast::ExprKind::WhileLet(ref pat, ref expr, ref block, ref label) => {
                ExprKind::WhileLet(Box::new(self.trans_while_let_expr(pat, expr, block, label)))
            }
            ast::ExprKind::ForLoop(ref pat, ref expr, ref block, ref label) => {
                ExprKind::For(Box::new(self.trans_for_expr(pat, expr, block, label)))
            }
            ast::ExprKind::Loop(ref block, ref label) => {
                ExprKind::Loop(Box::new(self.trans_loop_expr(block, label)))
            }
            ast::ExprKind::Break(ref ident)
            => ExprKind::Break(Box::new(self.trans_break_expr(ident))),
            ast::ExprKind::Again(ref ident) => {
                ExprKind::Continue(Box::new(self.trans_continue_expr(ident)))
            }
            ast::ExprKind::Match(ref expr, ref arms) => {
                ExprKind::Match(Box::new(self.trans_match_expr(expr, arms)))
            }
            ast::ExprKind::Call(ref fn_name, ref args) => {
                ExprKind::FnCall(Box::new(self.trans_fn_call_expr(fn_name, args)))
            }
            ast::ExprKind::MethodCall(ref ident, ref types, ref args) => {
                ExprKind::MethodCall(Box::new(self.trans_method_call_expr(ident, types, args)))
            }
            ast::ExprKind::Closure(capture, ref fn_decl, ref block) => {
                ExprKind::Closure(Box::new(self.trans_closure_expr(capture, fn_decl, block)))
            }
            ast::ExprKind::Ret(ref expr)
            => ExprKind::Return(Box::new(self.trans_return_expr(expr))),
            ast::ExprKind::Mac(ref mac) => ExprKind::Macro(self.trans_macro(mac)),
            ast::ExprKind::Try(ref expr) => ExprKind::Try(Box::new(self.trans_expr(expr))),
            ast::ExprKind::InlineAsm(_) => unreachable!(),
        };
        */
        self.set_loc(&loc);

        Expr {
            loc,
            attrs,
            s: self.span_to_snippet(expr.span).unwrap(),
        }
    }

    /*
    #[inline]
    fn trans_ident(&mut self, ident: &ast::SpannedIdent) -> Chunk {
        Chunk {
            loc: self.leaf_loc(&ident.span),
            s: ident_to_string(&ident.node),
        }
    }

    #[inline]
    fn trans_method_ident(&mut self, ident: &ast::SpannedIdent) -> Chunk {
        let s = ident_to_string(&ident.node);
        let mut span = ident.span;
        span.hi = ast::BytePos(span.lo.0 + s.len() as u32);

        Chunk {
            loc: self.leaf_loc(&span),
            s: s,
        }
    }

    #[inline]
    fn trans_pos(&mut self, pos: &ast::Spanned<usize>) -> Chunk {
        Chunk {
            loc: self.leaf_loc(&pos.span),
            s: pos.node.to_string(),
        }
    }

    #[inline]
    fn trans_literal_expr(&mut self, lit: &ast::Lit) -> Chunk {
        Chunk {
            loc: self.leaf_loc(&lit.span),
            s: self.literal_to_string(lit),
        }
    }

    #[inline]
    fn trans_unary_expr(&mut self, op: ast::UnOp, expr: &ast::Expr) -> UnaryExpr {
        UnaryExpr {
            op: uop_to_string(op),
            expr: self.trans_expr(expr),
        }
    }

    #[inline]
    fn trans_ref_expr(&mut self, mutble: ast::Mutability, expr: &ast::Expr) -> RefExpr {
        RefExpr {
            is_mut: is_mut(mutble),
            expr: self.trans_expr(expr),
        }
    }

    #[inline]
    fn trans_bop(&mut self, op: &ast::BinOp) -> Chunk {
        Chunk {
            loc: self.leaf_loc(&op.span),
            s: op.node.to_string().to_string(),
        }
    }

    #[inline]
    fn trans_bop_assign(&mut self, op: &ast::BinOp) -> Chunk {
        Chunk {
            loc: self.leaf_loc(&op.span),
            s: format!("{}=", op.node.to_string()),
        }
    }

    #[inline]
    fn trans_binary_expr(&mut self, left: &ast::Expr, op: &ast::BinOp, right: &ast::Expr)
                         -> ListExpr {
        ListExpr {
            exprs: vec![self.trans_expr(left), self.trans_expr(right)],
            sep: self.trans_bop(op),
        }
    }

    #[inline]
    fn trans_assign_expr(&mut self, left: &ast::Expr, right: &ast::Expr) -> ListExpr {
        ListExpr {
            exprs: vec![self.trans_expr(left), self.trans_expr(right)],
            sep: Chunk::new("="),
        }
    }

    #[inline]
    fn trans_op_assign_expr(&mut self, left: &ast::Expr, op: &ast::BinOp, right: &ast::Expr)
                            -> ListExpr {
        ListExpr {
            exprs: vec![self.trans_expr(left), self.trans_expr(right)],
            sep: self.trans_bop_assign(op),
        }
    }

    #[inline]
    fn trans_in_place_expr(&mut self, left: &ast::Expr, right: &ast::Expr) -> ListExpr {
        ListExpr {
            exprs: vec![self.trans_expr(left), self.trans_expr(right)],
            sep: Chunk::new("<-"),
        }
    }

    #[inline]
    fn trans_fixed_size_array_expr(&mut self, init: &ast::Expr, len: &ast::Expr)
                                   -> FixedSizeArrayExpr {
        FixedSizeArrayExpr {
            init: self.trans_expr(init),
            len: self.trans_expr(len),
        }
    }

    #[inline]
    fn trans_struct_field_access_expr(&mut self, expr: &ast::Expr, ident: &ast::SpannedIdent)
                                      -> FieldAccessExpr {
        FieldAccessExpr {
            expr: self.trans_expr(expr),
            field: self.trans_ident(ident),
        }
    }

    #[inline]
    fn trans_tuple_field_access_expr(&mut self, expr: &ast::Expr, pos: &ast::Spanned<usize>)
                                     -> FieldAccessExpr {
        FieldAccessExpr {
            expr: self.trans_expr(expr),
            field: self.trans_pos(pos),
        }
    }

    #[inline]
    fn trans_struct_expr(&mut self, path: &ast::Path, fields: &Vec<ast::Field>,
                         base: &Option<ast::P<ast::Expr>>)
                         -> StructExpr {
        StructExpr {
            path: self.trans_path(path),
            fields: self.trans_struct_field_exprs(fields),
            base: map_ref_mut(base, |expr| self.trans_expr(expr)),
        }
    }

    #[inline]
    fn trans_struct_field_exprs(&mut self, fields: &Vec<ast::Field>) -> Vec<StructFieldExpr> {
        trans_list!(self, fields, trans_struct_field_expr)
    }

    #[inline]
    fn trans_struct_field_expr(&mut self, field: &ast::Field) -> StructFieldExpr {
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
    fn trans_index_expr(&mut self, obj: &ast::Expr, index: &ast::Expr) -> IndexExpr {
        IndexExpr {
            obj: self.trans_expr(obj),
            index: self.trans_expr(index),
        }
    }

    #[inline]
    fn trans_range_expr(&mut self, start: &Option<ast::P<ast::Expr>>,
                        end: &Option<ast::P<ast::Expr>>, limit: ast::RangeLimits)
                        -> RangeExpr {
        RangeExpr {
            start: map_ref_mut(start, |expr| self.trans_expr(expr)),
            end: map_ref_mut(end, |expr| self.trans_expr(expr)),
            is_halfopen: is_halfopen(limit),
        }
    }

    #[inline]
    fn trans_box_expr(&mut self, expr: &ast::Expr) -> BoxExpr {
        BoxExpr {
            expr: self.trans_expr(expr),
        }
    }

    #[inline]
    fn trans_cast_expr(&mut self, expr: &ast::Expr, ty: &ast::Ty) -> CastExpr {
        CastExpr {
            expr: self.trans_expr(expr),
            ty: self.trans_type(ty),
        }
    }

    #[inline]
    fn trans_type_expr(&mut self, expr: &ast::Expr, ty: &ast::Ty) -> TypeExpr {
        TypeExpr {
            expr: self.trans_expr(expr),
            ty: self.trans_type(ty),
        }
    }

    #[inline]
    fn trans_if_expr(&mut self, expr: &ast::Expr, block: &ast::Block,
                     br: &Option<ast::P<ast::Expr>>)
                     -> IfExpr {
        IfExpr {
            expr: self.trans_expr(expr),
            block: self.trans_block(block),
            br: map_ref_mut(br, |expr| self.trans_expr(expr)),
        }
    }

    #[inline]
    fn trans_if_let_expr(&mut self, pat: &ast::P<ast::Pat>, expr: &ast::Expr,
                         block: &ast::Block,
                         br: &Option<ast::P<ast::Expr>>)
                         -> IfLetExpr {
        IfLetExpr {
            pat: self.trans_patten(pat),
            expr: self.trans_expr(expr),
            block: self.trans_block(block),
            br: map_ref_mut(br, |expr| self.trans_expr(expr)),
        }
    }

    #[inline]
    fn trans_while_expr(&mut self, expr: &ast::Expr, block: &ast::Block,
                        label: &Option<ast::Ident>)
                        -> WhileExpr {
        WhileExpr {
            label: map_ref_mut(label, |ident| ident_to_string(ident)),
            expr: self.trans_expr(expr),
            block: self.trans_block(block),
        }
    }

    #[inline]
    fn trans_while_let_expr(&mut self, pat: &ast::P<ast::Pat>, expr: &ast::Expr,
                            block: &ast::Block, label: &Option<ast::Ident>)
                            -> WhileLetExpr {
        WhileLetExpr {
            label: map_ref_mut(label, |ident| ident_to_string(ident)),
            pat: self.trans_patten(pat),
            expr: self.trans_expr(expr),
            block: self.trans_block(block),
        }
    }

    #[inline]
    fn trans_for_expr(&mut self, pat: &ast::P<ast::Pat>, expr: &ast::Expr, block: &ast::Block,
                      label: &Option<ast::Ident>)
                      -> ForExpr {
        ForExpr {
            label: map_ref_mut(label, |ident| ident_to_string(ident)),
            pat: self.trans_patten(pat),
            expr: self.trans_expr(expr),
            block: self.trans_block(block),
        }
    }

    #[inline]
    fn trans_loop_expr(&mut self, block: &ast::Block, label: &Option<ast::Ident>) -> LoopExpr {
        LoopExpr {
            label: map_ref_mut(label, |ident| ident_to_string(ident)),
            block: self.trans_block(block),
        }
    }

    #[inline]
    fn trans_break_expr(&mut self, ident: &Option<ast::SpannedIdent>) -> BreakExpr {
        BreakExpr {
            label: map_ref_mut(ident, |ident| self.trans_ident(ident)),
        }
    }

    #[inline]
    fn trans_continue_expr(&mut self, ident: &Option<ast::SpannedIdent>) -> ContinueExpr {
        ContinueExpr {
            label: map_ref_mut(ident, |ident| self.trans_ident(ident)),
        }
    }

    #[inline]
    fn trans_match_expr(&mut self, expr: &ast::Expr, arms: &Vec<ast::Arm>) -> MatchExpr {
        MatchExpr {
            expr: self.trans_expr(expr),
            arms: self.trans_arms(arms),
        }
    }

    #[inline]
    fn trans_arms(&mut self, arms: &Vec<ast::Arm>) -> Vec<Arm> {
        trans_list!(self, arms, trans_arm)
    }

    #[inline]
    fn trans_arm(&mut self, arm: &ast::Arm) -> Arm {
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
    fn trans_fn_call_expr(&mut self, fn_name: &ast::Expr, args: &Vec<ast::P<ast::Expr>>)
                          -> FnCallExpr {
        FnCallExpr {
            name: self.trans_expr(fn_name),
            args: self.trans_exprs(args),
        }
    }

    #[inline]
    fn trans_method_call_expr(&mut self, ident: &ast::SpannedIdent,
                              types: &Vec<ast::P<ast::Ty>>,
                              args: &Vec<ast::P<ast::Expr>>)
                              -> MethodCallExpr {
        MethodCallExpr {
            obj: self.trans_expr(&args[0]),
            name: self.trans_method_ident(ident),
            types: self.trans_types(types),
            args: self.trans_exprs(&args[1..]),
        }
    }

    #[inline]
    fn trans_closure_expr(&mut self, capture: ast::CaptureBy, fn_decl: &ast::FnDecl,
                          block: &ast::Block)
                          -> ClosureExpr {
        ClosureExpr {
            moved: is_move(capture),
            fn_sig: self.trans_fn_sig(fn_decl),
            block: self.trans_block(block),
        }
    }

    #[inline]
    fn trans_return_expr(&mut self, expr: &Option<ast::P<ast::Expr>>) -> ReturnExpr {
        ReturnExpr {
            ret: map_ref_mut(expr, |expr| self.trans_expr(expr)),
        }
    }

    #[inline]
    fn trans_macro_raw(&mut self, mac: &ast::Mac) -> MacroRaw {
        MacroRaw {
            style: self.macro_style(mac),
            s: Chunk {
                loc: self.leaf_loc(&mac.span),
                s: self.span_to_snippet(mac.span).unwrap(),
            },
        }
    }

    #[inline]
    fn trans_macro_stmt(&mut self, attrs: &ast::ThinAttributes, mac: &ast::Mac) -> MacroStmt {
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
    fn trans_macro(&mut self, mac: &ast::Mac) -> Macro {
        match self.trans_macro_expr(mac) {
            Some(macro_expr) => Macro::Expr(macro_expr),
            None => Macro::Raw(self.trans_macro_raw(mac)),
        }
    }

    #[inline]
    fn trans_macro_expr(&mut self, mac: &ast::Mac) -> Option<MacroExpr> {
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
    fn trans_macro_exprs(&self, mac: &ast::Mac)
                         -> Option<(Vec<ast::P<ast::Expr>>, Vec<MacroExprSep>)> {
        let mut exprs = Vec::new();
        let mut seps = Vec::new();

        if mac.node.tts.is_empty() {
            return Some((exprs, seps));
        }

        let mut parser
                = ast::parse::tts_to_parser(&self.sess, mac.node.tts.clone(), Vec::new());
        loop {
            exprs.push(match parser.parse_expr() {
                Ok(expr) => expr,
                Err(mut e) => {
                    e.cancel();
                    return None;
                }
            });

            match parser.token {
                ast::Token::Eof => break,
                ref other => seps.push(match token_to_macro_expr_sep(other) {
                    Some(sep) => sep,
                    None => return None,
                }),
            }

            parser.bump();
            if parser.token == ast::parse::token::Token::Eof {
                return None;
            }
        }
        Some((exprs, seps))
    }

    #[inline]
    fn macro_style(&self, mac: &ast::Mac) -> MacroStyle {
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
    */

    #[inline]
    fn loc(&mut self, sp: &ast::Span) -> Loc {
        self.trans_comments(sp.lo().0);

        Loc {
            start: sp.lo().0,
            end: sp.hi().0,
            nl: self.is_nl(sp.lo().0),
        }
    }

    #[inline]
    fn set_loc(&mut self, loc: &Loc) {
        self.trans_comments(loc.end);
        self.last_loc = *loc;
    }

    #[inline]
    fn leaf_loc(&mut self, sp: &ast::Span) -> Loc {
        let loc = self.loc(sp);
        self.set_loc(&loc);
        loc
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
    fn span_to_snippet(&self, sp: ast::Span) -> Result<String, ast::SpanSnippetError> {
        self.sess.source_map().span_to_snippet(sp)
    }

    #[inline]
    fn literal_to_string(&self, lit: &ast::Lit) -> String {
        self.span_to_snippet(lit.span).unwrap()
    }

    fn crate_file_end(&self) -> Pos {
        self.sess.source_map().files().last().unwrap().end_pos.0
    }
}
