use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Debug, Display};

use ir::*;
use ts::*;

pub fn fmt_crate(krate: &Crate, cmnts: &Vec<Comment>) -> (String, BTreeSet<u32>, BTreeSet<u32>) {
    Formatter::new(cmnts).fmt_crate(krate)
}

macro_rules! select_str {
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
select_str!(ptr_head, is_mut, "*mut ", "*const ");
select_str!(static_head, is_mut, "static mut ", "static ");

fn ref_head(lifetime: &Option<Lifetime>, is_mut: bool) -> String {
    let mut head = String::new();
    head.push_str("&");

    if let Some(ref lifetime) = *lifetime {
        head.push_str(&lifetime.s);
        head.push_str(" ");
    }
    if is_mut {
        head.push_str("mut ");
    }

    head
}

fn foreign_head(abi: &str) -> String {
    let mut head = String::new();

    if abi != r#""Rust""# {
        head.push_str("extern");
        head.push_str(&abi_head(abi));
        head.push_str(" ");
    }

    head
}

fn abi_head(abi: &str) -> String {
    let mut head = String::new();
    if abi != r#""C""# {
        head.push_str(" ");
        head.push_str(abi);
    }
    head
}

fn fn_head(is_unsafe: bool, is_const: bool, abi: &str) -> String {
    let mut head = String::new();
    if is_unsafe {
        head.push_str("unsafe ");
    }
    if is_const {
        head.push_str("const ");
    }
    head.push_str(&foreign_head(abi));
    head.push_str("fn");
    head
}

macro_rules! display_lists {
   ($f: expr, $open: expr, $sep: expr, $close: expr, $($lists: expr),+) => ({
        try!(write!($f, $open));

        let mut first = true;
        $(for e in $lists {
            if !first {
                try!(write!($f, $sep));
            }
            try!(Display::fmt(e, $f));
            first = false;
        })+

        write!($f, $close)
    });
}

impl Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.s, f)
    }
}

impl Display for Attr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "#"));
        if self.is_inner {
            try!(write!(f, "!"));
        }

        write!(f, "[{}]", self.item)
    }
}

impl Display for MetaItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(Display::fmt(&self.name, f));
        if let Some(ref items) = self.items {
            try!(display_meta_items(f, &**items));
        }
        Ok(())
    }
}

#[inline]
fn display_meta_items(f: &mut fmt::Formatter, items: &Vec<MetaItem>) -> fmt::Result {
    display_lists!(f, "(", ", ", ")", items)
}

impl Display for ExternCrate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "extern crate {};", self.name)
    }
}

impl Display for Use {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "use {}", self.base));

        if !self.names.is_empty() {
            try!(write!(f, "::"));
            if self.names.len() == 1 {
                try!(write!(f, "{}", self.names[0]))
            } else {
                try!(display_use_names(f, &self.names));
            }
        }

        write!(f, ";")
    }
}

#[inline]
fn display_use_names(f: &mut fmt::Formatter, names: &Vec<Chunk>) -> fmt::Result {
    display_lists!(f, "{{", ", ", "}}", names)
}

impl Display for ModDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mod {};", self.name)
    }
}

impl Display for TypeAlias {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "type {}{} = {};", self.name, self.generics, self.ty)
    }
}

impl Display for Generics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_empty() {
            return Ok(());
        } else {
            try!(display_lists!(f, "<", ", ", ">", &self.lifetime_defs, &self.type_params));
        }

        if !self.wh.is_empty() {
            try!(write!(f, " "));
            try!(Display::fmt(&self.wh, f));
        }

        Ok(())
    }
}

impl Display for LifetimeDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}", self.lifetime));
        if !self.bounds.is_empty() {
            try!(write!(f, ": "));
            try!(display_lifetime_def_bounds(f, &self.bounds));
        }
        Ok(())
    }
}

#[inline]
fn display_lifetime_def_bounds(f: &mut fmt::Formatter, bounds: &Vec<Lifetime>) -> fmt::Result {
    display_lists!(f, "", " + ", "", bounds)
}

impl Display for TypeParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}", self.name));

        if !self.bounds.is_empty() {
            try!(write!(f, ": "));
            try!(display_type_param_bounds(f, &self.bounds));
        }

        if let Some(ref ty) = self.default {
            try!(write!(f, " = {}", ty));
        }

        Ok(())
    }
}

#[inline]
fn display_type_param_bounds(f: &mut fmt::Formatter, bounds: &Vec<TypeParamBound>) -> fmt::Result {
    display_lists!(f, "", " + ", "", bounds)
}

impl Display for TypeParamBound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TypeParamBound::Lifetime(ref bound) => Display::fmt(bound, f),
            TypeParamBound::PolyTraitRef(ref bound) => Display::fmt(bound, f),
        }
    }
}

impl Display for PolyTraitRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(display_for_liftime_defs(f, &self.lifetime_defs));
        Display::fmt(&self.trait_ref, f)
    }
}

#[inline]
fn display_for_liftime_defs(f: &mut fmt::Formatter, lifetime_defs: &Vec<LifetimeDef>) -> fmt::Result {
    if !lifetime_defs.is_empty() {
        display_lists!(f, "for<", ", ", "> ", lifetime_defs)
    } else {
        Ok(())
    }
}

impl Display for Where {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_empty() {
            return Ok(());
        }

        try!(write!(f, "where "));
        display_where_clauses(f, &self.clauses)
    }
}

#[inline]
fn display_where_clauses(f: &mut fmt::Formatter, wh: &Vec<WhereClause>) -> fmt::Result {
    display_lists!(f, "", ", ", "", wh)
}

impl Display for WhereClause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.clause {
            WhereKind::LifetimeDef(ref wh) => Display::fmt(wh, f),
            WhereKind::Bound(ref wh) => Display::fmt(wh, f),
        }
    }
}

impl Display for WhereBound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(display_for_liftime_defs(f, &self.lifetime_defs));
        try!(write!(f, "{}: ", &self.ty));
        display_type_param_bounds(f, &self.bounds)
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.global {
            try!(write!(f, "::"));
        }
        display_path_segments(f, &self.segs)
    }
}

#[inline]
fn display_path_segments(f: &mut fmt::Formatter, segs: &[PathSegment]) -> fmt::Result {
    display_lists!(f, "", "::", "", segs)
}

impl Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.name, self.param)
    }
}

impl Display for PathParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PathParam::Angle(ref param) => Display::fmt(param, f),
            PathParam::Paren(ref param) => Display::fmt(param, f),
        }
    }
}

impl Display for AngleParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_empty() {
            Ok(())
        } else {
            display_lists!(f, "<", ", ", ">", &self.lifetimes, &self.types, &self.bindings)
        }
    }
}

impl Display for TypeBinding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} = {}", self.name, self.ty)
    }
}

impl Display for ParenParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(display_paren_param_inputs(f, &self.inputs));
        if let Some(ref output) = self.output {
            try!(write!(f, " -> {}", output));
        }
        Ok(())
    }
}

#[inline]
fn display_paren_param_inputs(f: &mut fmt::Formatter, inputs: &Vec<Type>) -> fmt::Result {
    display_lists!(f, "(", ", ", ")", inputs)
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.ty {
            TypeKind::Path(ref ty) => Display::fmt(ty, f),
            TypeKind::Ptr(ref ty) => Display::fmt(ty, f),
            TypeKind::Ref(ref ty) => Display::fmt(ty, f),
            TypeKind::Array(ref ty) => Display::fmt(ty, f),
            TypeKind::FixedSizeArray(ref ty) => Display::fmt(ty, f),
            TypeKind::Tuple(ref ty) => Display::fmt(ty, f),
            TypeKind::BareFn(ref ty) => Display::fmt(ty, f),
            TypeKind::Sum(ref ty) => Display::fmt(ty, f),
            TypeKind::PolyTraitRef(ref ty) => Display::fmt(ty, f),
            TypeKind::Macro(ref ty) => Display::fmt(ty, f),
            TypeKind::Infer => display_infer_type(f),
        }
    }
}

impl Display for PathType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.qself {
            Some(ref qself) => display_qself(f, qself, &self.path),
            None => Display::fmt(&self.path, f),
        }
    }
}

fn display_qself(f: &mut fmt::Formatter, qself: &QSelf, path: &Path) -> fmt::Result {
    try!(write!(f, "<{}", qself.ty));
    if qself.pos > 0 {
        try!(write!(f, " as "));
        if path.global {
            try!(write!(f, "::"));
        }
        try!(display_path_segments(f, &path.segs[0..qself.pos]));
    }
    try!(write!(f, ">"));

    try!(write!(f, "::"));
    display_path_segments(f, &path.segs[qself.pos..])
}

impl Display for PtrType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", ptr_head(self.is_mut), self.ty)
    }
}

impl Display for RefType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", ref_head(&self.lifetime, self.is_mut), self.ty)
    }
}

impl Display for ArrayType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.ty)
    }
}

impl Display for FixedSizeArrayType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}; {}]", self.ty, self.expr)
    }
}

impl Display for TupleType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_lists!(f, "(", ", ", ")", &self.types)
    }
}

impl Display for BareFnType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(display_for_liftime_defs(f, &self.lifetime_defs));
        write!(f, "{}{}", fn_head(self.is_unsafe, false, &self.abi), self.fn_sig)
    }
}

impl Display for SumType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(Display::fmt(&self.ty, f));
        if !self.bounds.is_empty() {
            try!(write!(f, " + "));
            try!(display_type_param_bounds(f, &self.bounds));
        }
        Ok(())
    }
}

impl Display for PolyTraitRefType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_type_param_bounds(f, &self.bounds)
    }
}

fn display_infer_type(f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "_")
}

impl Display for ForeignStatic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}: {}", static_head(self.is_mut), self.name, self.ty)
    }
}

impl Display for ForeignFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "fn {}{}{}", self.name, self.generics, self.fn_sig)
    }
}

impl Display for TupleField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_pub {
            try!(write!(f, "pub "));
        }
        Display::fmt(&self.ty, f)
    }
}

impl Display for FnSig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.arg, self.ret)
    }
}

impl Display for FnArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.va {
            try!(write!(f, "("));

            let mut first = true;
            for e in &self.args {
                if !first {
                    try!(write!(f, ", "));
                }
                try!(Display::fmt(e, f));
                first = false;
            }

            write!(f, ", ...)")
        } else {
            display_lists!(f, "(", ", ", ")", &self.args)
        }
    }
}

impl Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.pat, self.ty)
    }
}

impl Display for FnReturn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FnReturn::Unit => Ok(()),
            FnReturn::Diverge => write!(f, " -> !"),
            FnReturn::Normal(ref ty) => write!(f, " -> {}", ty),
        }
    }
}

impl Display for Patten {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

macro_rules! fmt_attr_group {
    ($sf: expr, $group: expr, $ty: ty, $fmt_attr: ident) => ({
        let map: BTreeMap<String, $ty> = $group.into_iter().map(|e| (e.to_string(), *e)).collect();

        for (_, e) in map {
            $sf.insert_indent();
            $sf.$fmt_attr(e);
            $sf.ts.nl();
        }
    })
}

macro_rules! fmt_item_group {
    ($sf: expr, $group: expr, $ty: ty, $fmt_item: ident) => ({
        let map: BTreeMap<String, (&Vec<AttrKind>, bool, $ty)>
                = $group.into_iter().map(|e| (e.2.to_string(), *e)).collect();

        for (_, e) in map {
            $sf.fmt_attrs(e.0);

            $sf.insert_indent();
            if e.1 {
                $sf.insert("pub ");
            }
            $sf.$fmt_item(e.2);

            $sf.raw_insert(";");
            $sf.ts.nl();
        }
    })
}

macro_rules! fmt_item_groups {
    ($sf: expr, $items: expr, $item_kind: path, $item_type: ty, $fmt_item: ident) => ({
        let mut group: Vec<(&Vec<AttrKind>, bool, $item_type)> = Vec::new();

        for item in $items {
            match item.item {
                $item_kind(ref e) => {
                    if $sf.is_after_comment(&item.loc) {
                        fmt_item_group!($sf, &group, $item_type, $fmt_item);
                        group.clear();

                        $sf.fmt_comments(&item.loc);
                    }

                    group.push((&item.attrs, item.is_pub, e));
                }
                _ => {
                    fmt_item_group!($sf, &group, $item_type, $fmt_item);
                    group.clear();
                }
            }
        }

        fmt_item_group!($sf, &group, $item_type, $fmt_item);
    })
}

macro_rules! insert_sep {
    ($sf: expr, $sep: expr, $e: expr) => ({
        $sf.raw_insert($sep);
        if !$e.loc.nl && !need_wrap!($sf.ts, &$e.to_string()) {
            $sf.raw_insert(" ");
        }
    });
}

macro_rules! maybe_nl {
    ($sf: expr, $e: ident) => ({
        if $e.loc.nl {
            $sf.wrap();
        }
    })
}

macro_rules! maybe_wrap {
    ($sf: expr, $sep: expr, $wrap_sep: expr, $e: expr) => ({
        if !need_wrap!($sf.ts, $sep, &$e.to_string()) {
            $sf.insert($sep);
        } else {
            $sf.wrap();
            $sf.insert($wrap_sep);
        }
    });
    ($sf: expr, $e: expr) => ({
        maybe_wrap!($sf, "", "", $e);
    });
    ($sf: expr, $sep: expr, $wrap_sep: expr, $e: expr, $act: ident) => ({
        maybe_wrap!($sf, $sep, $wrap_sep, $e);
        $sf.$act(&$e);
    });
}

macro_rules! maybe_wrap_len {
    ($sf: expr, $e: expr, $len: expr) => ({
        if $sf.ts.need_wrap_len($e.to_string().len() + $len) {
            $sf.wrap();
        }
    });
}

macro_rules! maybe_nl_non_wrap {
    ($sf: expr, $sep: expr, $e: expr) => ({
        if !need_wrap!($sf.ts, $sep, &$e.to_string()) {
            $sf.insert($sep);
        } else {
            $sf.nl_indent();
        }
    });
}

macro_rules! fmt_comma_lists {
    ($sf: expr, $open: expr, $close: expr, $($list: expr, $act: ident),+) => ({
        $sf.insert_mark_align($open);

        let mut first = true;
        $(for e in $list {
            if !first {
                insert_sep!($sf, ",", e);
            }

            $sf.$act(e);
            first = false;
        })+

        $sf.insert_unmark_align($close);
    });
    ($sf: expr, $($list: expr, $act: ident),+) => ({
        fmt_comma_lists!($sf, "", "", $($list, $act)+);
    });
}

macro_rules! fmt_lists {
    ($sf: expr, $sep: expr, $wrap_sep: expr, $($list: expr, $act: ident),+) => ({
        let mut first = true;
        $(for e in $list {
            if !first {
                maybe_wrap!($sf, $sep, $wrap_sep, e, $act);
            } else {
                $sf.$act(e);
            }

            first = false;
        })+
    })
}

macro_rules! fmt_block {
    ($sf: expr, $act: ident, $item: expr) => ({
        $sf.insert(" {");
        $sf.ts.indent();
        $sf.ts.nl();

        $sf.$act($item);

        $sf.ts.outdent();
        $sf.insert_indent();
        $sf.insert("}");
    });
}

macro_rules! fmt_items {
    ($sf: ident, $items: expr, $fmt_item: ident) => ({
        for item in $items {
            $sf.try_fmt_comments(&item.loc);
            $sf.fmt_attrs(&item.attrs);
            $sf.insert_indent();

            $sf.$fmt_item(item);
        }
    });
}

struct Formatter<'a> {
    ts: Typesetter,

    cmnts: &'a Vec<Comment>,
    cmnt_idx: usize,
    after_indent: bool,
    after_wrap: bool,
}

impl<'a> Formatter<'a> {
    fn new(cmnts: &'a Vec<Comment>) -> Formatter<'a> {
        Formatter {
            ts: Typesetter::new(),

            cmnts: cmnts,
            cmnt_idx: 0,
            after_indent: false,
            after_wrap: false,
        }
    }

    #[inline]
    fn clear_flag(&mut self) {
        self.after_indent = false;
        self.after_wrap = false;
    }

    #[inline]
    fn raw_insert(&mut self, s: &str) {
        self.ts.raw_insert(s);
        self.clear_flag();
    }

    fn insert(&mut self, s: &str) {
        self.ts.insert(s);
        self.clear_flag();
    }

    fn wrap(&mut self) {
        if !self.after_indent && !self.after_wrap {
            self.ts.wrap();
            self.after_wrap = true;
        }
    }

    fn insert_mark_align(&mut self, s: &str) {
        self.ts.insert_mark_align(s);
        self.clear_flag();
    }

    fn insert_unmark_align(&mut self, s: &str) {
        self.ts.insert_unmark_align(s);
        self.clear_flag();
    }

    fn nl_indent(&mut self) {
        if !self.after_indent {
            self.ts.nl_indent();
            self.after_indent = true;
        }
    }

    fn insert_indent(&mut self) {
        self.ts.insert_indent();
        self.after_indent = true;
    }

    #[inline]
    fn fmt_chunk(&mut self, chunk: &Chunk) {
        maybe_nl!(self, chunk);
        self.insert(&chunk.s);
    }

    #[inline]
    fn is_after_comment(&self, loc: &Loc) -> bool {
        self.cmnt_idx < self.cmnts.len() && self.cmnts[self.cmnt_idx].pos < loc.start
    }

    #[inline]
    fn try_fmt_comments(&mut self, loc: &Loc) {
        if self.is_after_comment(loc) {
            self.fmt_comments(loc);
        }
    }

    fn fmt_comments(&mut self, loc: &Loc) {
        while self.cmnt_idx < self.cmnts.len() && self.cmnts[self.cmnt_idx].pos < loc.start {
            let idx = self.cmnt_idx;
            self.fmt_comment(&self.cmnts[idx]);
            self.cmnt_idx += 1;
        }
    }

    fn fmt_comment(&mut self, cmnt: &Comment) {
        p!("---------- comment ----------");
        p!("{:#?}", cmnt);

        if cmnt.lines.is_empty() {
            self.ts.nl();
            return;
        }

        for line in &cmnt.lines {
            self.insert_indent();
            self.raw_insert(line);
            self.ts.nl();
        }
    }

    fn fmt_crate(mut self, krate: &Crate) -> (String, BTreeSet<u32>, BTreeSet<u32>) {
        self.try_fmt_comments(&krate.loc);
        self.fmt_attrs(&krate.attrs);
        self.fmt_mod(&krate.module);

        p!("{:?}", self.ts);
        self.ts.result()
    }

    fn fmt_attrs(&mut self, attrs: &Vec<AttrKind>) {
        let mut attr_group: Vec<&Attr> = Vec::new();

        for attr in attrs {
            match *attr {
                AttrKind::Doc(ref doc) => {
                    self.fmt_attr_group(&attr_group);
                    attr_group.clear();

                    self.fmt_doc(doc);
                }
                AttrKind::Attr(ref attr) => {
                    if self.is_after_comment(&attr.loc) {
                        self.fmt_attr_group(&attr_group);
                        attr_group.clear();

                        self.fmt_comments(&attr.loc);
                    }
                    attr_group.push(attr);
                }
            }
        }

        self.fmt_attr_group(&attr_group);
    }

    fn fmt_doc(&mut self, doc: &Doc) {
        self.try_fmt_comments(&doc.loc);
        p!("---------- doc ----------");
        p!("{:#?}", doc);

        self.insert_indent();
        self.raw_insert(&doc.s);
        self.ts.nl();
    }

    #[inline]
    fn fmt_attr_group(&mut self, attr_group: &Vec<&Attr>) {
        p!("---------- attr ----------");
        fmt_attr_group!(self, attr_group, &Attr, fmt_attr);
    }

    fn fmt_attr(&mut self, attr: &Attr) {
        p!(attr);

        self.insert("#");
        if attr.is_inner {
            self.insert("!");
        }
        self.insert("[");
        self.fmt_attr_meta_item(&attr.item);
        self.insert("]");
    }

    fn fmt_attr_meta_item(&mut self, item: &MetaItem) {
        maybe_nl!(self, item);
        self.insert(&item.name);

        if let Some(ref items) = item.items {
            self.fmt_attr_meta_items(&**items);
        }
    }

    fn fmt_attr_meta_items(&mut self, items: &Vec<MetaItem>) {
        fmt_comma_lists!(self, "(", ")", items, fmt_attr_meta_item);
    }

    fn fmt_mod(&mut self, module: &Mod) {
        p!("---------- mod ----------");
        p!(module.name);

        self.fmt_group_items(&module.items);
        self.fmt_items(&module.items);
    }

    fn fmt_group_items(&mut self, items: &Vec<Item>) {
        p!("---------- group items begin ----------");

        self.fmt_extern_crate_items(items);
        self.fmt_use_items(items);
        self.fmt_mod_decl_items(items);

        p!("---------- group items end ----------");
    }

    fn fmt_extern_crate_items(&mut self, items: &Vec<Item>) {
        p!("---------- extern crate ----------");
        fmt_item_groups!(self, items, ItemKind::ExternCrate, &ExternCrate, fmt_extern_crate);
    }

    #[inline]
    fn fmt_extern_crate(&mut self, item: &ExternCrate) {
        p!(item);

        self.insert(&format!("extern crate {}", &item.name));
    }

    fn fmt_use_items(&mut self, items: &Vec<Item>) {
        p!("---------- use ----------");
        fmt_item_groups!(self, items, ItemKind::Use, &Use, fmt_use);
    }

    #[inline]
    fn fmt_use(&mut self, item: &Use) {
        p!(item);

        self.insert(&format!("use {}", &item.base));
        self.fmt_use_names(&item.names);
    }

    fn fmt_use_names(&mut self, names: &Vec<Chunk>) {
        if names.is_empty() {
            return;
        }

        self.insert("::");
        if names.len() == 1 {
            self.insert(&names[0].s);
            return;
        }

        fmt_comma_lists!(self, "{", "}", names, fmt_chunk);
    }

    fn fmt_mod_decl_items(&mut self, items: &Vec<Item>) {
        p!("---------- mod decl ----------");
        fmt_item_groups!(self, items, ItemKind::ModDecl, &ModDecl, fmt_mod_decl);
    }

    #[inline]
    fn fmt_mod_decl(&mut self, item: &ModDecl) {
        p!(item);

        self.insert(&format!("mod {}", &item.name));
    }

    #[inline]
    fn fmt_items(&mut self, items: &Vec<Item>) {
        for item in items {
            match item.item {
                ItemKind::ExternCrate(_) | ItemKind::Use(_) | ItemKind::ModDecl(_) => (),
                _ => {
                    self.try_fmt_comments(&item.loc);
                    self.fmt_attrs(&item.attrs);
                    self.insert_indent();

                    self.fmt_item(item);
                }
            }
        }
    }

    fn fmt_item(&mut self, item: &Item) {
        p!("---------- item ----------");

        if item.is_pub {
            self.insert("pub ");
        }

        match item.item {
            ItemKind::ExternCrate(_) | ItemKind::Use(_) | ItemKind::ModDecl(_) => unreachable!(),
            ItemKind::Mod(ref item) => self.fmt_sub_mod(item),
            ItemKind::TypeAlias(ref item) => self.fmt_type_alias(item),
            ItemKind::ForeignMod(ref item) => self.fmt_foreign_mod(item),
            ItemKind::Const(ref item) => self.fmt_const(item),
            ItemKind::Static(ref item) => self.fmt_static(item),
            ItemKind::Struct(ref item) => self.fmt_struct(item),
            ItemKind::Enum(ref item) => self.fmt_enum(item),
            ItemKind::Fn(ref item) => self.fmt_fn(item),
            ItemKind::Trait(ref item) => self.fmt_trait(item),
            ItemKind::ImplDefault(ref item) => self.fmt_impl_default(item),
            ItemKind::Impl(ref item) => self.fmt_impl(item),
            ItemKind::Macro(ref item) => self.fmt_macro(item),
        }

        self.ts.nl();
    }

    fn fmt_sub_mod(&mut self, item: &Mod) {
        p!("---------- sub mod ----------");
        p!(item.name);

        self.insert(&format!("mod {}", &item.name));

        if item.items.is_empty() {
            self.insert("{}");
            return;
        }

        fmt_block!(self, fmt_mod, item);
    }

    fn fmt_type_alias(&mut self, item: &TypeAlias) {
        p!("---------- type alias ----------");
        p!(item);

        self.insert(&format!("type {}", &item.name));

        self.fmt_generics(&item.generics);
        self.fmt_where(&item.generics.wh);

        maybe_wrap!(self, " = ", "= ", item.ty, fmt_type);
        self.raw_insert(";");
    }

    fn fmt_generics(&mut self, generics: &Generics) {
        if !generics.is_empty() {
            fmt_comma_lists!(self,
                             "<",
                             ">",
                             &generics.lifetime_defs,
                             fmt_lifetime_def,
                             &generics.type_params,
                             fmt_type_param);
        }
    }

    fn fmt_lifetime_def(&mut self, lifetime_def: &LifetimeDef) {
        self.fmt_lifetime(&lifetime_def.lifetime);
        if !lifetime_def.bounds.is_empty() {
            self.insert(": ");
            fmt_lists!(self, " + ", "+ ", &lifetime_def.bounds, fmt_lifetime)
        }
    }

    fn fmt_lifetime(&mut self, lifetime: &Lifetime) {
        self.fmt_chunk(lifetime);
    }

    fn fmt_type_param(&mut self, type_param: &TypeParam) {
        maybe_nl!(self, type_param);
        self.insert(&type_param.name);

        if let Some(ref ty) = type_param.default {
            maybe_wrap!(self, " = ", "= ", ty, fmt_type);
            return;
        }

        if !type_param.bounds.is_empty() {
            self.insert(": ");
            self.fmt_type_param_bounds(&type_param.bounds);
        }
    }

    fn fmt_type_param_bounds(&mut self, bounds: &Vec<TypeParamBound>) {
        fmt_lists!(self, " + ", "+ ", bounds, fmt_type_param_bound)
    }

    fn fmt_type_param_bound(&mut self, bound: &TypeParamBound) {
        match *bound {
            TypeParamBound::Lifetime(ref lifetime) => self.fmt_lifetime(lifetime),
            TypeParamBound::PolyTraitRef(ref poly_trait_ref) => {
                self.fmt_poly_trait_ref(poly_trait_ref)
            }
        }
    }

    fn fmt_poly_trait_ref(&mut self, poly_trait_ref: &PolyTraitRef) {
        self.fmt_for_lifetime_defs(&poly_trait_ref.lifetime_defs);
        self.fmt_trait_ref(&poly_trait_ref.trait_ref);
    }

    fn fmt_for_lifetime_defs(&mut self, lifetime_defs: &Vec<LifetimeDef>) {
        if !lifetime_defs.is_empty() {
            fmt_comma_lists!(self, "for<", "> ", lifetime_defs, fmt_lifetime_def);
        }
    }

    fn fmt_trait_ref(&mut self, trait_ref: &TraitRef) {
        self.fmt_path(trait_ref);
    }

    fn fmt_where(&mut self, wh: &Where) {
        if !wh.is_empty() {
            maybe_nl_non_wrap!(self, " ", wh);
            self.insert("where ");
            self.fmt_where_clauses(&wh.clauses);
        }
    }

    fn fmt_where_clauses(&mut self, clauses: &Vec<WhereClause>) {
        fmt_comma_lists!(self, clauses, fmt_where_clause);
    }

    fn fmt_where_clause(&mut self, clause: &WhereClause) {
        match clause.clause {
            WhereKind::LifetimeDef(ref lifetime_def) => self.fmt_lifetime_def(lifetime_def),
            WhereKind::Bound(ref bound) => self.fmt_where_bound(bound),
        }
    }

    fn fmt_where_bound(&mut self, bound: &WhereBound) {
        self.fmt_for_lifetime_defs(&bound.lifetime_defs);
        self.fmt_type(&bound.ty);
        self.insert(": ");
        self.fmt_type_param_bounds(&bound.bounds);
    }

    fn fmt_path(&mut self, path: &Path) {
        maybe_nl!(self, path);

        if path.global {
            self.insert("::");
        }

        self.fmt_path_segments(&path.segs);
    }

    fn fmt_path_segments(&mut self, segs: &[PathSegment]) {
        fmt_lists!(self, "::", "::", segs, fmt_path_segment)
    }

    fn fmt_path_segment(&mut self, seg: &PathSegment) {
        self.insert(&seg.name);
        self.fmt_path_param(&seg.param);
    }

    fn fmt_path_param(&mut self, param: &PathParam) {
        match *param {
            PathParam::Angle(ref param) => self.fmt_angle_param(param),
            PathParam::Paren(ref param) => self.fmt_paren_param(param),
        }
    }

    fn fmt_angle_param(&mut self, param: &AngleParam) {
        if !param.is_empty() {
            fmt_comma_lists!(self,
                             "<",
                             ">",
                             &param.lifetimes,
                             fmt_lifetime,
                             &param.types,
                             fmt_type,
                             &param.bindings,
                             fmt_type_binding);
        }
    }

    fn fmt_type_binding(&mut self, binding: &TypeBinding) {
        maybe_nl!(self, binding);
        self.insert(&format!("{} = ", binding.name));
        self.fmt_type(&binding.ty);
    }

    fn fmt_paren_param(&mut self, param: &ParenParam) {
        fmt_comma_lists!(self, "(", ")", &param.inputs, fmt_type);
        if let Some(ref output) = param.output {
            maybe_wrap!(self, " -> ", "-> ", output, fmt_type);
        }
    }

    fn fmt_type(&mut self, ty: &Type) {
        maybe_nl!(self, ty);
        match ty.ty {
            TypeKind::Path(ref ty) => self.fmt_path_type(ty),
            TypeKind::Ptr(ref ty) => self.fmt_ptr_type(ty),
            TypeKind::Ref(ref ty) => self.fmt_ref_type(ty),
            TypeKind::Array(ref ty) => self.fmt_array_type(ty),
            TypeKind::FixedSizeArray(ref ty) => self.fmt_fixed_size_array_type(ty),
            TypeKind::Tuple(ref ty) => self.fmt_tuple_type(ty),
            TypeKind::BareFn(ref ty) => self.fmt_bare_fn_type(ty),
            TypeKind::Sum(ref ty) => self.fmt_sum_type(ty),
            TypeKind::PolyTraitRef(ref ty) => self.fmt_poly_trait_ref_type(ty),
            TypeKind::Macro(ref ty) => self.fmt_macro(ty),
            TypeKind::Infer => self.fmt_infer_type(),
        }
    }

    fn fmt_path_type(&mut self, ty: &PathType) {
        match ty.qself {
            Some(ref qself) => {
                maybe_wrap!(self, ty);
                self.fmt_qself_path(qself, &ty.path);
            }
            None => self.fmt_path(&ty.path),
        }
    }

    fn fmt_qself_path(&mut self, qself: &QSelf, path: &Path) {
        self.insert_mark_align("<");
        self.fmt_type(&qself.ty);
        if qself.pos > 0 {
            self.insert(" as ");
            if path.global {
                self.insert("::");
            }
            self.fmt_path_segments(&path.segs[0..qself.pos]);
        }
        self.insert_unmark_align(">");

        self.insert("::");
        self.fmt_path_segments(&path.segs[qself.pos..]);
    }

    fn fmt_ptr_type(&mut self, ty: &PtrType) {
        let head = ptr_head(ty.is_mut);
        maybe_wrap!(self, head, head, ty.ty, fmt_type);
    }

    fn fmt_ref_type(&mut self, ty: &RefType) {
        let head = &ref_head(&ty.lifetime, ty.is_mut);
        maybe_wrap!(self, head, head, ty.ty, fmt_type);
    }

    fn fmt_array_type(&mut self, ty: &ArrayType) {
        maybe_wrap_len!(self, ty.ty, 2);
        self.insert_mark_align("[");
        self.fmt_type(&ty.ty);
        self.insert_unmark_align("]");
    }

    fn fmt_fixed_size_array_type(&mut self, ty: &FixedSizeArrayType) {
        maybe_wrap_len!(self, ty.ty, 4);
        self.insert_mark_align("[");
        self.fmt_type(&ty.ty);
        insert_sep!(self, ";", ty.expr);
        self.fmt_expr(&ty.expr);
        self.insert_unmark_align("]");
    }

    fn fmt_tuple_type(&mut self, ty: &TupleType) {
        fmt_comma_lists!(self, "(", ")", &ty.types, fmt_type);
    }

    fn fmt_bare_fn_type(&mut self, ty: &BareFnType) {
        self.fmt_for_lifetime_defs(&ty.lifetime_defs);
        self.insert(&fn_head(ty.is_unsafe, false, &ty.abi));
        self.fmt_fn_sig(&ty.fn_sig);
    }

    fn fmt_sum_type(&mut self, ty: &SumType) {
        self.fmt_type(&ty.ty);
        if !ty.bounds.is_empty() {
            self.insert(": ");
            self.fmt_type_param_bounds(&ty.bounds);
        }
    }

    fn fmt_poly_trait_ref_type(&mut self, ty: &PolyTraitRefType) {
        self.fmt_type_param_bounds(&ty.bounds);
    }

    fn fmt_infer_type(&mut self) {
        self.insert("_");
    }

    fn fmt_foreign_mod(&mut self, item: &ForeignMod) {
        p!("---------- foreign mod ----------");

        self.insert(&format!("extern{}", abi_head(&item.abi)));

        if item.items.is_empty() {
            self.insert(" {}");
            return;
        }

        fmt_block!(self, fmt_foreign_items, &item.items);
    }

    fn fmt_foreign_items(&mut self, items: &Vec<ForeignItem>) {
        fmt_items!(self, items, fmt_foreign_item);
    }

    fn fmt_foreign_item(&mut self, item: &ForeignItem) {
        p!("---------- foreign item ----------");

        if item.is_pub {
            self.insert("pub ");
        }

        match item.item {
            ForeignKind::Static(ref item) => self.fmt_foreign_static(item),
            ForeignKind::Fn(ref item) => self.fmt_foreign_fn(item),
        }

        self.raw_insert(";");
        self.ts.nl();
    }

    fn fmt_foreign_static(&mut self, item: &ForeignStatic) {
        p!("---------- foreign static ----------");
        p!(item);

        self.insert(&format!("{}{}", static_head(item.is_mut), item.name));
        insert_sep!(self, ":", item.ty);
        self.fmt_type(&item.ty);
    }

    fn fmt_foreign_fn(&mut self, item: &ForeignFn) {
        p!("---------- foreign fn ----------");
        p!(item);

        self.insert(&format!("fn {}", item.name));
        self.fmt_generics(&item.generics);
        self.fmt_fn_sig(&item.fn_sig);
    }

    fn fmt_const(&mut self, item: &Const) {
        self.insert(&format!("const {}", item.name));
        insert_sep!(self, ":", item.ty);
        self.fmt_type(&item.ty);
        maybe_wrap!(self, " = ", "= ", item.expr, fmt_expr);
    }

    fn fmt_static(&mut self, item: &Static) {
        self.insert(&format!("{}{}", static_head(item.is_mut), item.name));
        insert_sep!(self, ":", item.ty);
        self.fmt_type(&item.ty);
        maybe_wrap!(self, " = ", "= ", item.expr, fmt_expr);
    }

    fn fmt_struct(&mut self, item: &Struct) {
        self.insert(&format!("struct {}", item.name));
        self.fmt_generics(&item.generics);
        self.fmt_struct_body(&item.body);

        match item.body {
            StructBody::Tuple(_) | StructBody::Unit => self.raw_insert(";"),
            _ => (),
        }
    }

    fn fmt_struct_body(&mut self, body: &StructBody) {
        match *body {
            StructBody::Struct(ref fields) => self.fmt_struct_field_block(fields),
            StructBody::Tuple(ref fields) => self.fmt_tuple_fields(fields),
            StructBody::Unit => (),
        }
    }

    fn fmt_struct_field_block(&mut self, fields: &Vec<StructField>) {
        if fields.is_empty() {
            self.insert(" {}");
            return;
        }

        fmt_block!(self, fmt_struct_fields, &fields);
    }

    fn fmt_struct_fields(&mut self, fields: &Vec<StructField>) {
        fmt_items!(self, fields, fmt_struct_field);
    }

    fn fmt_struct_field(&mut self, field: &StructField) {
        if field.is_pub {
            self.insert("pub ");
        }
        self.insert(&field.name);
        insert_sep!(self, ":", field.ty);
        self.fmt_type(&field.ty);

        self.raw_insert(",");
        self.ts.nl();
    }

    fn fmt_tuple_fields(&mut self, fields: &Vec<TupleField>) {
        fmt_comma_lists!(self, "(", ")", fields, fmt_tuple_field);
    }

    fn fmt_tuple_field(&mut self, field: &TupleField) {
        self.try_fmt_comments(&field.loc);
        self.fmt_attrs(&field.attrs);

        if field.is_pub {
            self.insert("pub ");
        }
        self.fmt_type(&field.ty);
    }

    fn fmt_enum(&mut self, item: &Enum) {
        self.insert(&format!("enum {}", item.name));
        self.fmt_generics(&item.generics);
        self.fmt_enum_body(&item.body);
    }

    fn fmt_enum_body(&mut self, body: &EnumBody) {
        if body.fields.is_empty() {
            self.insert(" {}");
            return;
        }

        fmt_block!(self, fmt_enum_fields, &body.fields);
    }

    fn fmt_enum_fields(&mut self, fields: &Vec<EnumField>) {
        fmt_items!(self, fields, fmt_enum_field);
    }

    fn fmt_enum_field(&mut self, field: &EnumField) {
        self.insert(&field.name);
        self.fmt_struct_body(&field.body);
        if let Some(ref expr) = field.expr {
            maybe_wrap!(self, " = ", "= ", expr, fmt_expr);
        }

        self.raw_insert(",");
        self.ts.nl();
    }

    fn fmt_fn(&mut self, item: &Fn) {
        self.insert(&format!("{} {}",
                             fn_head(item.is_unsafe, item.is_const, &item.abi),
                             item.name));
        self.fmt_generics(&item.generics);
        self.fmt_fn_sig(&item.fn_sig);
        self.fmt_where(&item.generics.wh);
        self.fmt_block(&item.block);
    }

    fn fmt_trait(&mut self, item: &Trait) {
        if item.is_unsafe {
            self.insert("unsafe ");
        }
        self.insert(&format!("trait {}", item.name));
        self.fmt_generics(&item.generics);
        if !item.bounds.is_empty() {
            self.insert(": ");
            self.fmt_type_param_bounds(&item.bounds);
        }
        self.fmt_where(&item.generics.wh);

        if item.items.is_empty() {
            self.insert(" {}");
            return;
        }

        fmt_block!(self, fmt_trait_items, &item.items);
    }

    fn fmt_trait_items(&mut self, items: &Vec<TraitItem>) {
        fmt_items!(self, items, fmt_trait_item);
    }

    fn fmt_trait_item(&mut self, item: &TraitItem) {
        match item.item {
            TraitItemKind::Const(ref item) => self.fmt_const_trait_item(item),
            TraitItemKind::Type(ref item) => self.fmt_type_trait_item(item),
            TraitItemKind::Method(ref item) => self.fmt_method_trait_item(item),
        }

        self.raw_insert(";");
        self.ts.nl();
    }

    fn fmt_const_trait_item(&mut self, item: &ConstTraitItem) {
        self.insert(&format!("const {}", item.name));
        insert_sep!(self, ":", item.ty);
        self.fmt_type(&item.ty);
    }

    fn fmt_type_trait_item(&mut self, item: &TypeTraitItem) {
        self.insert(&format!("type {}", item.name));
        if !item.bounds.is_empty() {
            self.insert(": ");
            self.fmt_type_param_bounds(&item.bounds);
        }
        if let Some(ref ty) = item.ty {
            maybe_wrap!(self, " = ", "= ", ty, fmt_type);
        }
    }

    fn fmt_method_trait_item(&mut self, item: &MethodTraitItem) {
        self.insert(&format!("{} {}",
                             fn_head(item.is_unsafe, item.is_const, &item.abi),
                             item.name));
        self.fmt_method_sig(&item.method_sig);
        if let Some(ref block) = item.block {
            self.fmt_block(block);
        }
    }

    fn fmt_impl_default(&mut self, item: &ImplDefault) {
        if item.is_unsafe {
            self.insert("unsafe ");
        }
        self.insert("impl ");
        self.fmt_trait_ref(&item.trait_ref);
        self.insert(" for .. {}");
    }

    fn fmt_impl(&mut self, item: &Impl) {
        if item.is_unsafe {
            self.insert("unsafe ");
        }

        self.insert("impl");
        self.fmt_generics(&item.generics);
        self.insert(" ");

        if let Some(ref trait_ref) = item.trait_ref {
            if item.is_neg {
                maybe_wrap!(self, "!", "!", trait_ref, fmt_trait_ref);
            } else {
                self.fmt_trait_ref(trait_ref);
            }
            maybe_wrap!(self, " for ", "for ", item.ty, fmt_type);
        } else {
            self.fmt_type(&item.ty);
        }

        self.fmt_where(&item.generics.wh);

        if item.items.is_empty() {
            self.insert(" {}");
            return;
        }

        fmt_block!(self, fmt_impl_items, &item.items);
    }

    fn fmt_impl_items(&mut self, items: &Vec<ImplItem>) {
        fmt_items!(self, items, fmt_impl_item);
    }

    fn fmt_impl_item(&mut self, item: &ImplItem) {
        match item.item {
            ImplItemKind::Const(ref item) => {
                self.fmt_const_impl_item(item);
                self.raw_insert(";");
            }
            ImplItemKind::Type(ref item) => {
                self.fmt_type_impl_item(item);
                self.raw_insert(";");
            }
            ImplItemKind::Method(ref item) => self.fmt_method_impl_item(item),
            ImplItemKind::Macro(ref item) => self.fmt_macro(item),
        }

        self.ts.nl();
    }

    fn fmt_const_impl_item(&mut self, item: &ConstImplItem) {
        self.insert(&format!("const {}", item.name));
        insert_sep!(self, ":", item.ty);
        self.fmt_type(&item.ty);
        maybe_wrap!(self, " = ", "= ", item.expr, fmt_expr);
    }

    fn fmt_type_impl_item(&mut self, item: &TypeImplItem) {
        self.insert(&format!("type {}", item.name));
        maybe_wrap!(self, " = ", "= ", item.ty, fmt_type);
    }

    fn fmt_method_impl_item(&mut self, item: &MethodImplItem) {
        self.insert(&format!("{} {}",
                             fn_head(item.is_unsafe, item.is_const, &item.abi),
                             item.name));
        self.fmt_method_sig(&item.method_sig);
        self.fmt_block(&item.block);
    }

    fn fmt_fn_sig(&mut self, fn_sig: &FnSig) {
        self.fmt_fn_arg(&fn_sig.arg);
        self.fmt_fn_return(&fn_sig.ret);
    }

    fn fmt_fn_arg(&mut self, arg: &FnArg) {
        if arg.va {
            self.insert_mark_align("(");

            let mut first = true;
            for e in &arg.args {
                if !first {
                    insert_sep!(self, ",", e);
                }

                self.fmt_arg(e);
                first = false;
            }

            self.insert(", ...");
            self.insert_unmark_align(")");
        } else {
            fmt_comma_lists!(self, "(", ")", &arg.args, fmt_arg);
        }
    }

    fn fmt_arg(&mut self, arg: &Arg) {
        maybe_wrap!(self, arg);
        self.fmt_patten(&arg.pat);
        maybe_wrap!(self, ": ", ":", arg.ty, fmt_type);
    }

    fn fmt_fn_return(&mut self, ret: &FnReturn) {
        match *ret {
            FnReturn::Unit => (),
            FnReturn::Diverge => {
                maybe_nl_non_wrap!(self, " ", ret);
                self.insert("-> !");
            }
            FnReturn::Normal(ref ty) => {
                maybe_nl_non_wrap!(self, " ", ret);
                self.insert("-> ");
                self.fmt_type(ty);
            }
        }
    }

    fn fmt_method_sig(&mut self, sig: &MethodSig) {
        self.fmt_generics(&sig.generics);

        if let Some(ref sf) = sig.sf {
            self.fmt_method_fn_sig(sf, sig);
        } else {
            self.fmt_fn_sig(&sig.fn_sig);
        }

        self.fmt_where(&sig.generics.wh);
    }

    fn fmt_method_fn_sig(&mut self, sf: &Sf, sig: &MethodSig) {
        self.insert_mark_align("(");
        match *sf {
            Sf::String(ref s) => self.insert(s),
            Sf::Type(ref ty) => {
                maybe_wrap_len!(self, ty, 6);
                self.insert("self: ");
                self.fmt_type(ty);
            }
        }

        for e in &sig.fn_sig.arg.args[1..] {
            insert_sep!(self, ",", e);
            self.fmt_arg(e);
        }
        self.insert_unmark_align(")");

        self.fmt_fn_return(&sig.fn_sig.ret);
    }

    fn fmt_block(&mut self, block: &Block) {
        if block.is_unsafe {
            self.insert("unsafe");
        }

        if block.stmts.is_empty() {
            self.insert(" {}");
            return;
        }

        fmt_block!(self, fmt_stmts, &block.stmts);
    }

    fn fmt_stmts(&mut self, stmts: &Vec<Stmt>) {
        for stmt in stmts {
            self.fmt_stmt(stmt);
        }
    }

    fn fmt_stmt(&mut self, stmt: &Stmt) {
        match stmt.stmt {
            StmtKind::Decl(ref decl) => self.fmt_decl_stmt(decl),
            StmtKind::Expr(ref expr, is_semi) => self.fmt_expr_stmt(expr, is_semi),
            StmtKind::Macro(ref mac) => self.fmt_macro(mac),
        }

        self.ts.nl();
    }

    fn fmt_decl_stmt(&mut self, decl: &Decl) {
        match decl.decl {
            DeclKind::Local(ref local) => self.fmt_local(local),
            DeclKind::Item(ref item) => {
                self.try_fmt_comments(&item.loc);
                self.fmt_attrs(&item.attrs);
                self.insert_indent();

                self.fmt_item(item);
            }
        }
        self.raw_insert(";");
    }

    fn fmt_local(&mut self, local: &Local) {
        self.try_fmt_comments(&local.loc);
        self.fmt_attrs(&local.attrs);
        self.insert_indent();

        self.insert("let ");
        self.fmt_patten(&local.pat);
        if let Some(ref ty) = local.ty {
            maybe_wrap!(self, ": ", ":", ty, fmt_type);
        }
        if let Some(ref expr) = local.init {
            maybe_wrap!(self, " = ", "= ", expr, fmt_expr);
        }
    }

    fn fmt_patten(&mut self, pat: &Patten) {
        maybe_nl!(self, pat);
        match pat.pat {
            PattenKind::Wildcard => self.insert("_"),
            PattenKind::Literal(ref pat) => self.fmt_expr(pat),
            PattenKind::Range(ref pat) => self.fmt_range_patten(pat),
            PattenKind::Ident(ref pat) => self.fmt_ident_patten(pat),
            PattenKind::Ref(ref pat) => self.fmt_ref_patten(pat),
            PattenKind::Path(ref pat) => self.fmt_path_patten(pat),
            PattenKind::Enum(ref pat) => self.fmt_enum_patten(pat),
            PattenKind::Struct(ref pat) => self.fmt_struct_patten(pat),
            PattenKind::Vec(ref pat) => self.fmt_vec_patten(pat),
            PattenKind::Tuple(ref pat) => self.fmt_tuple_patten(pat),
            PattenKind::Box(ref pat) => self.fmt_box_patten(pat),
            PattenKind::Macro(ref pat) => self.fmt_macro(pat),
        }
    }

    fn fmt_range_patten(&mut self, pat: &RangePatten) {
        self.fmt_expr(&pat.start);
        self.insert("...");
        self.fmt_expr(&pat.end);
    }

    fn fmt_ident_patten(&mut self, pat: &IdentPatten) {
        let mut head = String::new();
        if pat.is_ref {
            head.push_str("ref ");
        }
        if pat.is_mut {
            head.push_str("mut ");
        }

        self.insert(&format!("{}{}", head, pat.name.s));
        if let Some(ref binding) = pat.binding {
            maybe_wrap!(self, " @ ", "@ ", binding, fmt_patten);
        }
    }

    fn fmt_ref_patten(&mut self, pat: &RefPatten) {
        self.insert(&ref_head(&None, pat.is_mut));
        self.fmt_patten(&pat.pat);
    }

    fn fmt_path_patten(&mut self, pat: &PathPatten) {
        self.fmt_qself_path(&pat.qself, &pat.path);
    }

    fn fmt_enum_patten(&mut self, pat: &EnumPatten) {
        self.fmt_path(&pat.path);
        match pat.pats {
            Some(ref pats) => fmt_comma_lists!(self, "(", ")", pats, fmt_patten),
            None => self.insert("(..)"),
        }
    }

    fn fmt_struct_patten(&mut self, pat: &StructPatten) {
        self.fmt_path(&pat.path);

        if pat.fields.is_empty() {
            self.insert(" {}");
            return;
        }

        self.insert(" {");
        self.ts.indent();
        self.ts.nl();

        self.fmt_struct_field_pattens(&pat.fields);
        if pat.etc {
            self.insert_indent();
            self.insert("...");
            self.ts.nl();
        }

        self.ts.outdent();
        self.insert_indent();
        self.insert("}");
    }

    fn fmt_struct_field_pattens(&mut self, fields: &Vec<StructFieldPatten>) {
        for field in fields {
            self.try_fmt_comments(&field.loc);
            self.insert_indent();

            self.fmt_struct_field_patten(field);
        }
    }

    fn fmt_struct_field_patten(&mut self, field: &StructFieldPatten) {
        if field.shorthand {
            self.fmt_patten(&field.pat);
        } else {
            self.insert(&field.name);
            maybe_wrap!(self, ": ", ":", field.pat, fmt_patten);
        }

        self.raw_insert(",");
        self.ts.nl();
    }

    fn fmt_vec_patten(&mut self, pat: &VecPatten) {
        let emit = if let Some(_) = pat.emit {
            vec![Chunk::new("..")]
        } else {
            Vec::new()
        };
        fmt_comma_lists!(self,
                         "[",
                         "]",
                         &pat.start,
                         fmt_patten,
                         &emit,
                         fmt_vec_emit_patten,
                         &pat.end,
                         fmt_patten);
    }

    fn fmt_vec_emit_patten(&mut self, emit: &Chunk) {
        self.insert(&emit.s);
    }

    fn fmt_tuple_patten(&mut self, pat: &TuplePatten) {
        fmt_comma_lists!(self, "(", ")", &pat.pats, fmt_patten);
    }

    fn fmt_box_patten(&mut self, pat: &Patten) {
        self.insert("box ");
        self.fmt_patten(pat);
    }

    fn fmt_expr_stmt(&mut self, expr: &Expr, is_semi: bool) {
        self.try_fmt_comments(&expr.loc);
        self.fmt_attrs(&expr.attrs);
        self.insert_indent();

        self.fmt_expr(expr);
        if is_semi {
            self.raw_insert(";");
        }
    }

    fn fmt_expr(&mut self, expr: &Expr) {
        maybe_nl!(self, expr);
        match expr.expr {
            ExprKind::Literal(ref expr) => self.fmt_literal_expr(expr),
            ExprKind::Path(ref expr) => self.fmt_path_expr(expr),
            ExprKind::Unary(ref expr) => self.fmt_unary_expr(expr),
            ExprKind::Ref(ref expr) => self.fmt_ref_expr(expr),
            ExprKind::List(ref expr) => self.fmt_list_expr(expr),
            ExprKind::FixedSizeArray(ref expr) => self.fmt_fixed_size_array_expr(expr),
            ExprKind::Vec(ref exprs) => self.fmt_vec_expr(exprs),
            ExprKind::Tuple(ref exprs) => self.fmt_tuple_expr(exprs),
            ExprKind::FieldAccess(ref expr) => self.fmt_field_access_expr(expr),
            ExprKind::Struct(ref expr) => self.fmt_struct_expr(expr),
            ExprKind::Index(ref expr) => self.fmt_index_expr(expr),
            ExprKind::Range(ref expr) => self.fmt_range_expr(expr),
            ExprKind::Box(ref expr) => self.fmt_box_expr(expr),
            ExprKind::Cast(ref expr) => self.fmt_cast_expr(expr),
            ExprKind::Type(ref expr) => self.fmt_type_expr(expr),
            ExprKind::Block(ref expr) => self.fmt_block(expr),
            ExprKind::If(ref expr) => self.fmt_if_expr(expr),
            ExprKind::IfLet(ref expr) => self.fmt_if_let_expr(expr),
            ExprKind::While(ref expr) => self.fmt_while_expr(expr),
            ExprKind::WhileLet(ref expr) => self.fmt_while_let_expr(expr),
            ExprKind::For(ref expr) => self.fmt_for_expr(expr),
            ExprKind::Loop(ref expr) => self.fmt_loop_expr(expr),
            ExprKind::Break(ref expr) => self.fmt_break_expr(expr),
            ExprKind::Continue(ref expr) => self.fmt_continue_expr(expr),
            ExprKind::Match(ref expr) => self.fmt_match_expr(expr),
            ExprKind::FnCall(ref expr) => self.fmt_fn_call_expr(expr),
            ExprKind::MethodCall(ref expr) => self.fmt_method_call_expr(expr),
            ExprKind::Closure(ref expr) => self.fmt_closure_expr(expr),
            ExprKind::Return(ref expr) => self.fmt_return_expr(expr),
            ExprKind::Macro(ref expr) => self.fmt_macro(expr),
        }
    }

    fn fmt_literal_expr(&mut self, expr: &Chunk) {
        self.fmt_chunk(expr);
    }

    fn fmt_path_expr(&mut self, expr: &PathExpr) {
        self.fmt_path_type(expr);
    }

    fn fmt_unary_expr(&mut self, expr: &UnaryExpr) {
        maybe_wrap!(self, &expr.op.s, &expr.op.s, expr.expr, fmt_expr);
    }

    fn fmt_ref_expr(&mut self, expr: &RefExpr) {
        let head = &ref_head(&None, expr.is_mut);
        maybe_wrap!(self, head, head, expr.expr, fmt_expr);
    }

    fn fmt_list_expr(&mut self, expr: &ListExpr) {
        let sep = format!(" {} ", expr.sep);
        let wrap_sep = format!("{} ", expr.sep);
        fmt_lists!(self, &sep, &wrap_sep, &expr.exprs, fmt_expr);
    }

    fn fmt_fixed_size_array_expr(&mut self, expr: &FixedSizeArrayExpr) {
        self.insert_mark_align("[");
        self.fmt_expr(&expr.init);
        insert_sep!(self, ";", expr.len);
        self.fmt_expr(&expr.len);
        self.insert_unmark_align("]");
    }

    fn fmt_vec_expr(&mut self, exprs: &Vec<Expr>) {
        fmt_comma_lists!(self, "[", "]", exprs, fmt_expr);
    }

    fn fmt_tuple_expr(&mut self, exprs: &Vec<Expr>) {
        fmt_comma_lists!(self, "(", ")", exprs, fmt_expr);
    }

    fn fmt_field_access_expr(&mut self, expr: &FieldAccessExpr) {
        self.fmt_expr(&expr.expr);
        self.insert(&format!(".{}", &expr.field.s));
    }

    fn fmt_struct_expr(&mut self, expr: &StructExpr) {
        self.fmt_path(&expr.path);

        if expr.fields.is_empty() {
            self.insert(" {}");
            return;
        }

        self.insert(" {");
        self.ts.indent();
        self.ts.nl();

        self.fmt_struct_field_exprs(&expr.fields);
        if let Some(ref base) = expr.base {
            self.insert_indent();
            self.insert("..");
            self.fmt_expr(base);
            self.ts.nl();
        }

        self.ts.outdent();
        self.insert_indent();
        self.insert("}");
    }

    fn fmt_struct_field_exprs(&mut self, fields: &Vec<StructFieldExpr>) {
        for field in fields {
            self.try_fmt_comments(&field.loc);
            self.insert_indent();

            self.fmt_struct_field_expr(field);
        }
    }

    fn fmt_struct_field_expr(&mut self, field: &StructFieldExpr) {
        self.insert(&field.name.s);
        maybe_wrap!(self, ": ", ":", &field.value, fmt_expr);
        self.raw_insert(",");
        self.ts.nl();
    }

    fn fmt_index_expr(&mut self, expr: &IndexExpr) {
        self.fmt_expr(&expr.obj);
        self.insert_mark_align("[");
        self.fmt_expr(&expr.index);
        self.insert_unmark_align("]");
    }

    fn fmt_range_expr(&mut self, expr: &RangeExpr) {
        if let Some(ref start) = expr.start {
            self.fmt_expr(start);
        }
        self.insert("..");
        if let Some(ref end) = expr.end {
            self.fmt_expr(end);
        }
    }

    fn fmt_box_expr(&mut self, expr: &BoxExpr) {
        maybe_wrap!(self, "box ", "box ", expr.expr, fmt_expr);
    }

    fn fmt_cast_expr(&mut self, expr: &CastExpr) {
        self.fmt_expr(&expr.expr);
        maybe_wrap!(self, " as ", "as ", expr.ty, fmt_type);
    }

    fn fmt_type_expr(&mut self, expr: &TypeExpr) {
        self.fmt_expr(&expr.expr);
        maybe_wrap!(self, ": ", ":", expr.ty, fmt_type);
    }

    fn fmt_if_expr(&mut self, expr: &IfExpr) {
        self.insert("if ");
        self.fmt_expr(&expr.expr);
        self.fmt_block(&expr.block);
        if let Some(ref br) = expr.br {
            self.insert(" else ");
            self.fmt_expr(br);
        }
    }

    fn fmt_if_let_expr(&mut self, expr: &IfLetExpr) {
        self.insert("if let ");
        self.fmt_patten(&expr.pat);
        maybe_wrap!(self, " = ", "= ", expr.expr, fmt_expr);
        self.fmt_block(&expr.block);
        if let Some(ref br) = expr.br {
            self.insert(" else ");
            self.fmt_expr(br);
        }
    }

    fn fmt_label(&mut self, label: &Option<String>) {
        if let Some(ref label) = *label {
            self.insert(&format!("{}:", label));
            self.ts.nl();
            self.insert_indent();
        }
    }

    fn fmt_while_expr(&mut self, expr: &WhileExpr) {
        self.fmt_label(&expr.label);
        self.insert("while ");
        self.fmt_expr(&expr.expr);
        self.fmt_block(&expr.block);
    }

    fn fmt_while_let_expr(&mut self, expr: &WhileLetExpr) {
        self.fmt_label(&expr.label);
        self.insert("while let ");
        self.fmt_patten(&expr.pat);
        maybe_wrap!(self, " = ", "= ", expr.expr, fmt_expr);
        self.fmt_block(&expr.block);
    }

    fn fmt_for_expr(&mut self, expr: &ForExpr) {
        self.fmt_label(&expr.label);
        self.insert("for ");
        self.fmt_patten(&expr.pat);
        maybe_wrap!(self, " in ", "in ", expr.expr, fmt_expr);
        self.fmt_block(&expr.block);
    }

    fn fmt_loop_expr(&mut self, expr: &LoopExpr) {
        self.fmt_label(&expr.label);
        self.insert("loop ");
        self.fmt_block(&expr.block);
    }

    fn fmt_jump_label(&mut self, keyword: &str, label: &Option<Chunk>) {
        let label = if let Some(ref label) = *label {
            format!(" {}", label.s)
        } else {
            String::new()
        };
        self.insert(&format!("{}{}", keyword, label));
    }

    fn fmt_break_expr(&mut self, expr: &BreakExpr) {
        self.fmt_jump_label("break", &expr.label);
    }

    fn fmt_continue_expr(&mut self, expr: &ContinueExpr) {
        self.fmt_jump_label("continue", &expr.label);
    }

    fn fmt_match_expr(&mut self, expr: &MatchExpr) {
        self.insert("match ");
        self.fmt_expr(&expr.expr);
        fmt_block!(self, fmt_arms, &expr.arms);
    }

    fn fmt_arms(&mut self, arms: &Vec<Arm>) {
        for arm in arms {
            self.fmt_arm(arm);
        }
    }

    fn fmt_arm(&mut self, arm: &Arm) {
        self.try_fmt_comments(&arm.loc);
        self.fmt_attrs(&arm.attrs);
        self.insert_indent();

        fmt_lists!(self, " | ", "| ", &arm.pats, fmt_patten);
        if let Some(ref guard) = arm.guard {
            maybe_wrap!(self, " if ", "if ", guard, fmt_expr);
        }
        maybe_wrap!(self, " => ", "=> ", arm.body, fmt_expr);

        self.raw_insert(",");
        self.ts.nl();
    }

    fn fmt_fn_call_expr(&mut self, expr: &FnCallExpr) {
        self.fmt_expr(&expr.name);
        fmt_comma_lists!(self, "(", ")", &expr.args, fmt_expr);
    }

    fn fmt_method_call_expr(&mut self, expr: &MethodCallExpr) {
        self.fmt_expr(&expr.obj);
        self.insert(&format!(".{}", &expr.name.s));
        if !expr.types.is_empty() {
            self.insert("::");
            fmt_comma_lists!(self, "<", ">", &expr.types, fmt_type);
        }
        fmt_comma_lists!(self, "(", ")", &expr.args, fmt_expr);
    }

    fn fmt_closure_expr(&mut self, expr: &ClosureExpr) {
        if expr.moved {
            self.insert("move ");
        }

        self.fmt_closure_fn_arg(&expr.fn_sig.arg);
        self.fmt_fn_return(&expr.fn_sig.ret);
        self.fmt_block(&expr.block);
    }

    fn fmt_closure_fn_arg(&mut self, arg: &FnArg) {
        if arg.va {
            self.insert_mark_align("|");

            let mut first = true;
            for e in &arg.args {
                if !first {
                    insert_sep!(self, ",", e);
                }

                self.fmt_arg(e);
                first = false;
            }

            self.insert(", ...");
            self.insert_unmark_align("|");
        } else {
            fmt_comma_lists!(self, "|", "|", &arg.args, fmt_closure_arg);
        }
    }

    fn fmt_closure_arg(&mut self, arg: &Arg) {
        maybe_wrap!(self, arg);
        self.fmt_patten(&arg.pat);
        if let TypeKind::Infer = arg.ty.ty {
            return;
        } else {
            maybe_wrap!(self, ": ", ":", arg.ty, fmt_type);
        }
    }

    fn fmt_return_expr(&mut self, expr: &ReturnExpr) {
        self.insert("return");
        if let Some(ref expr) = expr.ret {
            maybe_wrap!(self, " ", "", expr, fmt_expr);
        }
    }

    fn fmt_macro(&mut self, mac: &Macro) {
        self.fmt_chunk(mac);
    }
}
