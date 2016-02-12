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
            try!(display_list!(f, &**items, "(", ", ", ")"));
        }
        Ok(())
    }
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
                try!(display_list!(f, &self.names, "{{", ", ", "}}"));
            }
        }

        write!(f, ";")
    }
}

impl Display for ModDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mod {};", self.name)
    }
}

macro_rules! fmt_attr_group {
    ($sf: expr, $group: expr, $ty: ty, $fmt_item: ident) => ({
        let map: BTreeMap<String, $ty> = $group.into_iter()
            .map(|e| (e.to_string(), *e))
            .collect();

        for (_, e) in map {
            $sf.$fmt_item(e);
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
    })
}

macro_rules! fmt_list {
    ($sf: ident, $list: expr, $open: expr, $close: expr, $act: expr) => ({
        $sf.ts.insert_mark_align($open);

        let mut first = true;
        for e in $list {
            if !first {
                $sf.ts.raw_insert(",");
                if !e.loc.wrapped && !$sf.ts.need_wrap(&e.to_string()) {
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
        p!("{}", attr);

        self.ts.insert("#");
        if attr.is_inner {
            self.ts.insert("!");
        }
        self.ts.insert("[");
        self.fmt_attr_meta_item(&attr.item);
        self.ts.insert("]");
    }

    fn fmt_attr_meta_item(&mut self, item: &MetaItem) {
        if item.loc.wrapped {
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
        self.fmt_group_items(&module.items);
        self.fmt_non_group_items(&module.items);
    }

    fn fmt_group_items(&mut self, items: &Vec<Item>) {
        self.fmt_extern_crate_items(items);
        self.fmt_use_items(items);
        self.fmt_mod_decl_items(items);
    }

    fn fmt_extern_crate_items(&mut self, items: &Vec<Item>) {
        p!("---------- extern crate ----------");
        fmt_item_groups!(self, items, ItemKind::ExternCrate, &ExternCrate, fmt_extern_crate);
    }

    fn fmt_extern_crate(&mut self, item: &ExternCrate) {
        p!("{}", item);

        self.ts.insert("extern crate ");
        self.ts.insert(&item.name);
    }

    fn fmt_use_items(&mut self, items: &Vec<Item>) {
        p!("---------- use ----------");
        fmt_item_groups!(self, items, ItemKind::Use, &Use, fmt_use);
    }

    fn fmt_use(&mut self, item: &Use) {
        p!("{}", item);

        self.ts.insert("use ");
        self.ts.insert(&item.base);
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

    fn fmt_mod_decl(&mut self, item: &ModDecl) {
        p!("{}", item);

        self.ts.insert("mod ");
        self.ts.insert(&item.name);
    }

    fn fmt_non_group_items(&mut self, items: &Vec<Item>) {}
}
