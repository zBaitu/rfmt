use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Debug, Display};

use ir::*;
use ts::*;

pub fn fmt_crate(krate: &Crate, cmnts: &Vec<Comment>) -> (String, BTreeSet<u32>) {
    Formatter::new(cmnts).fmt_crate(krate)
}

macro_rules! display_list {
    ($f: expr, $list: expr, $begin: expr, $sep: expr, $end: expr) => ({
        try!(write!($f, $begin));

        let mut first = true;
        for e in $list {
            if !first {
                try!(write!($f, $sep));
            }
            try!(Display::fmt(e, $f));
            first = false;
        }

        write!($f, $end)
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
        match *self {
            MetaItem::Single(ref chunk) => Display::fmt(chunk, f),
            MetaItem::List(_, ref name, ref items) => {
                try!(write!(f, "{}", name));
                display_list!(f, items, "(", ", ", ")")
            }
        }
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
        try!(write!(f, "use {}", self.path));

        if !self.items.is_empty() {
            try!(write!(f, "::"));
            if self.items.len() == 1 {
                try!(write!(f, "{}", self.items[0]))
            } else {
                try!(display_list!(f, &self.items, "{{", ", ", "}}"));
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

macro_rules! fmt_group_items {
    ($sf: ident, $items: expr, $item_kind: path, $fmt_item: ident) => ({
        let mut group: Vec<&Item> = Vec::new();

        for item in $items {
            match item.item {
                $item_kind(_) => {
                    if $sf.is_after_comment(&item.loc) {
                        fmt_group!($sf, &group, &Item, $fmt_item);
                        group.clear();

                        $sf.fmt_comments(&item.loc);
                    }

                    group.push(item);
                }
                _ => {
                    fmt_group!($sf, &group, &Item, $fmt_item);
                    group.clear();
                }
            }
        }
    })
}

macro_rules! fmt_group {
    ($sf: expr, $group: expr, $ty: ty, $fmt_item: ident) => ({
        let map: BTreeMap<String, $ty> = $group.into_iter()
            .map(|e| (e.to_string(), *e))
            .collect();
        
        for (_, e) in map {
            $sf.$fmt_item(e);
        }
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
            self.fmt_comment(&self.cmnts[self.cmnt_idx]);
            self.cmnt_idx += 1;
        }
    }

    fn fmt_comment(&self, cmnt: &Comment) {
        p!("---------- comment ----------");
        p!("{:#?}", cmnt);
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
    }

    #[inline]
    fn fmt_attr_group(&mut self, attr_group: &Vec<&Attr>) {
        p!("---------- attr ----------");
        fmt_group!(self, attr_group, &Attr, fmt_attr);
    }

    fn fmt_attr(&mut self, attr: &Attr) {
        p!("{}", attr);
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
        fmt_group_items!(self, items, ItemKind::ExternCrate, fmt_extern_crate);
    }

    fn fmt_extern_crate(&mut self, item: &Item) {
        p!("{}", item);
    }

    fn fmt_use_items(&mut self, items: &Vec<Item>) {
        p!("---------- use ----------");
        fmt_group_items!(self, items, ItemKind::Use, fmt_use);
    }

    fn fmt_use(&mut self, item: &Item) {
        p!("{}", item);
    }

    fn fmt_mod_decl_items(&mut self, items: &Vec<Item>) {
        p!("---------- mod decl ----------");
        fmt_group_items!(self, items, ItemKind::ModDecl, fmt_mod_decl);
    }

    fn fmt_mod_decl(&mut self, item: &Item) {
        p!("{}", item);
    }

    fn fmt_non_group_items(&mut self, items: &Vec<Item>) {}
}
