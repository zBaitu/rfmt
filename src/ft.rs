use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Debug, Display};

use ir::*;
use ts::*;

pub fn fmt_crate(krate: &Crate, cmnts: &Vec<Comment>) -> (String, BTreeSet<u32>) {
    Formatter::new(cmnts).fmt_crate(krate)
}

macro_rules! display_list {
    ($f: expr, $list: expr, $open: expr, $sep: expr, $close: expr) => ({
        try!(write!($f, $open));

        let mut first = true;
        for e in $list {
            if !first {
                try!(write!($f, $sep));
            }
            try!(Display::fmt(e, $f));
            first = false;
        }

        write!($f, $close)
    })
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
    })
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
    display_list!(f, items, "(", ", ", ")")
}

impl Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_pub {
            try!(write!(f, "pub "));
        }
        Display::fmt(&self.item, f)
    }
}

impl Display for ItemKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ItemKind::ExternCrate(ref item) => Display::fmt(item, f),
            ItemKind::Use(ref item) => Display::fmt(item, f),
            ItemKind::ModDecl(ref item) => Display::fmt(item, f),
            _ => Debug::fmt(self, f),
        }
    }
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
    display_list!(f, names, "{{", ", ", "}}")
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
            try!(write!(f, " where "));
            try!(display_where_clauses(f, &self.wh));
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
    display_list!(f, bounds, "", " + ", "")
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
    display_list!(f, bounds, "", " + ", "")
}

impl Display for TypeParamBound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TypeParamBound::Lifetime(ref lifetime) => Display::fmt(lifetime, f),
            TypeParamBound::PolyTraitRef(ref poly_trait_ref) => Display::fmt(poly_trait_ref, f),
        }
    }
}

impl Display for PolyTraitRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.lifetime_defs.is_empty() {
            try!(display_for_liftime_defs(f, &self.lifetime_defs));
        }
        Display::fmt(&self.trait_ref, f)
    }
}

#[inline]
fn display_for_liftime_defs(f: &mut fmt::Formatter, lifetime_defs: &Vec<LifetimeDef>) -> fmt::Result {
    display_list!(f, lifetime_defs, "for<", ", ", "> ")
}

#[inline]
fn display_where_clauses(f: &mut fmt::Formatter, wh: &Vec<WhereClause>) -> fmt::Result {
    display_list!(f, wh, "", ", ", "")
}

impl Display for WhereClause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.clause {
            WhereKind::Lifetime(ref lifetime_def) => Display::fmt(lifetime_def, f),
            WhereKind::Bound(ref bound) => Display::fmt(bound, f),
        }
    }
}

impl Display for WhereBound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.lifetime_defs.is_empty() {
            try!(display_for_liftime_defs(f, &self.lifetime_defs));
        }
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
fn display_path_segments(f: &mut fmt::Formatter, segs: &Vec<PathSegment>) -> fmt::Result {
    display_list!(f, segs, "", "::", "")
}

impl Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}", self.name));
        Display::fmt(&self.param, f)
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
    display_list!(f, inputs, "(", ", ", ")")
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

macro_rules! fmt_attr_group {
    ($sf: expr, $group: expr, $ty: ty, $fmt_attr: ident) => ({
        let map: BTreeMap<String, $ty> = $group.into_iter()
        .map(|e| (e.to_string(), *e))
        .collect();

        for (_, e) in map {
            $sf.ts.insert_indent();
            $sf.$fmt_attr(e);
            $sf.ts.nl();
        }
    })
}

macro_rules! fmt_item_group {
    ($sf: ident, $group: expr, $ty: ty, $fmt_item: ident) => ({
        let map: BTreeMap<String, (&Vec<AttrKind>, bool, $ty)> = $group.into_iter()
        .map(|e| (e.2.to_string(), *e))
        .collect();

    for (_, e) in map {
        $sf.fmt_attrs(e.0);

        $sf.ts.insert_indent();
        if e.1 {
            $sf.ts.insert("pub ");
        }
        $sf.$fmt_item(e.2);

        $sf.ts.raw_insert(";");
        $sf.ts.nl();
    }
    })
}

macro_rules! fmt_item_groups {
    ($sf: ident, $items: expr, $item_kind: path, $item_type: ty, $fmt_item: ident) => ({
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

macro_rules! fmt_list {
    ($sf: ident, $list: expr, $open: expr, $close: expr, $act: expr) => ({
        $sf.ts.insert_mark_align($open);

        let mut first = true;
        for e in $list {
            if !first {
                $sf.ts.raw_insert(",");
                if !e.loc.nl && !$sf.ts.need_wrap(&e.to_string()) {
                    $sf.ts.raw_insert(" ");
                }
            }

            $act(e);
            first = false;
        }

        $sf.ts.insert_unmark_align($close);
    })
}

struct Formatter<'a> {
    cmnts: &'a Vec<Comment>,
    cmnt_idx: usize,

    ts: Typesetter,
}

impl<'a> Formatter<'a> {
    fn new(cmnts: &'a Vec<Comment>) -> Formatter<'a> {
        Formatter {
            cmnts: cmnts,
            cmnt_idx: 0,

            ts: Typesetter::new(),
        }
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
            self.ts.raw_insert(line);
            self.ts.nl();
        }
    }

    fn fmt_crate(mut self, krate: &Crate) -> (String, BTreeSet<u32>) {
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

        self.ts.raw_insert(&doc.s);
        self.ts.nl();
    }

    #[inline]
    fn fmt_attr_group(&mut self, attr_group: &Vec<&Attr>) {
        p!("---------- attr ----------");
        fmt_attr_group!(self, attr_group, &Attr, fmt_attr);
    }

    fn fmt_attr(&mut self, attr: &Attr) {
        p!(attr);

        self.ts.insert("#");
        if attr.is_inner {
            self.ts.insert("!");
        }
        self.ts.insert("[");
        self.fmt_attr_meta_item(&attr.item);
        self.ts.insert("]");
    }

    fn fmt_attr_meta_item(&mut self, item: &MetaItem) {
        if item.loc.nl {
            self.ts.wrap();
        }
        self.ts.insert(&item.name);

        if let Some(ref items) = item.items {
            self.fmt_attr_meta_items(&**items);
        }
    }

    fn fmt_attr_meta_items(&mut self, items: &Vec<MetaItem>) {
        fmt_list!(self, items, "(", ")", |item: &MetaItem| self.fmt_attr_meta_item(item));
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

        self.ts.insert(&format!("extern crate {}", &item.name));
    }

    fn fmt_use_items(&mut self, items: &Vec<Item>) {
        p!("---------- use ----------");
        fmt_item_groups!(self, items, ItemKind::Use, &Use, fmt_use);
    }

    #[inline]
    fn fmt_use(&mut self, item: &Use) {
        p!(item);

        self.ts.insert(&format!("use {}", &item.base));
        self.fmt_use_names(&item.names);
    }

    fn fmt_use_names(&mut self, names: &Vec<Chunk>) {
        if names.is_empty() {
            return;
        }

        self.ts.insert("::");
        if names.len() == 1 {
            self.ts.insert(&names[0].s);
            return;
        }

        fmt_list!(self, names, "{", "}", |name: &Chunk| self.ts.insert(&name.s));
    }

    fn fmt_mod_decl_items(&mut self, items: &Vec<Item>) {
        p!("---------- mod decl ----------");
        fmt_item_groups!(self, items, ItemKind::ModDecl, &ModDecl, fmt_mod_decl);
    }

    #[inline]
    fn fmt_mod_decl(&mut self, item: &ModDecl) {
        p!(item);

        self.ts.insert(&format!("mod {}", &item.name));
    }

    #[inline]
    fn fmt_items(&mut self, items: &Vec<Item>) {
        for item in items {
            match item.item {
                ItemKind::ExternCrate(_) | ItemKind::Use(_) | ItemKind::ModDecl(_) => (),
                _ => self.fmt_item(item),
            }
        }
    }

    fn fmt_item(&mut self, item: &Item) {
        self.try_fmt_comments(&item.loc);
        p!("---------- item ----------");

        self.fmt_attrs(&item.attrs);

        self.ts.insert_indent();
        if item.is_pub {
            self.ts.insert("pub ");
        }

        match item.item {
            ItemKind::ExternCrate(_) | ItemKind::Use(_) | ItemKind::ModDecl(_) => unreachable!(),
            ItemKind::Mod(ref item) => self.fmt_sub_mod(item),
            ItemKind::TypeAlias(ref item) => self.fmt_type_alias(item),
            _ => (),
        }
    }

    fn fmt_sub_mod(&mut self, item: &Mod) {
        p!("---------- sub mod ----------");
        p!(item.name);

        self.ts.insert(&format!("mod {} ", &item.name));

        if item.items.is_empty() {
            self.ts.insert("{}");
            self.ts.nl();
            return;
        }

        self.ts.insert("{");
        self.ts.indent();
        self.ts.nl();

        self.fmt_mod(item);

        self.ts.outdent();
        self.ts.insert_indent();
        self.ts.insert("}");
    }

    fn fmt_type_alias(&mut self, item: &TypeAlias) {
        p!("---------- type alias ----------");
        p!(item);
    }
}
