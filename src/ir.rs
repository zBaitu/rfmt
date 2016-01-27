use std::fmt::{self, Debug, Display};

#[derive(Clone, Copy, Default)]
pub struct Loc {
    pub start: u32,
    pub end: u32,
    pub wrapped: bool,
}

impl Loc {
    pub fn new(start: u32, end: u32, wrapped: bool) -> Loc {
        Loc {
            start: start,
            end: end,
            wrapped: wrapped,
        }
    }
}

impl Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.wrapped {
            write!(f, "Loc({}, {}, wrapped)", self.start, self.end)
        } else {
            write!(f, "Loc({}, {})", self.start, self.end)
        }
    }
}

impl Debug for Loc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

#[derive(Debug)]
pub struct Chunk {
    loc: Loc,
    s: String,
}

impl Chunk {
    pub fn new(loc: Loc, s: String) -> Chunk {
        Chunk {
            loc: loc,
            s: s,
        }
    }
}

#[derive(Debug)]
pub struct Crate {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub module: Mod,
}

impl Crate {
    pub fn new(loc: Loc, attrs: Vec<AttrKind>, module: Mod) -> Crate {
        Crate {
            loc: loc,
            attrs: attrs,
            module: module,
        }
    }
}

#[derive(Debug)]
pub enum AttrKind {
    Attr(Attr),
    Doc(Doc),
}

#[derive(Debug)]
pub struct Attr {
    pub loc: Loc,
    pub head: &'static str,
    pub meta_item: MetaItem,
}

#[derive(Debug)]
pub enum MetaItem {
    Single(Chunk),
    List(Loc, String, Vec<MetaItem>),
}

#[inline]
fn attr_head(is_outer: bool) -> &'static str {
    static HASH: &'static str = "#";
    static HASH_BANG: &'static str = "#!";

    if is_outer {
        HASH
    } else {
        HASH_BANG
    }
}

impl Attr {
    pub fn new(loc: Loc, is_outer: bool, meta_item: MetaItem) -> Attr {
        Attr {
            loc: loc,
            head: attr_head(is_outer),
            meta_item: meta_item,
        }
    }
}

pub type Doc = Chunk;

#[derive(Debug)]
pub struct Mod {
    pub loc: Loc,
    pub items: Vec<Item>,
}

impl Mod {
    pub fn new(loc: Loc, items: Vec<Item>) -> Mod {
        Mod {
            loc: loc,
            items: items,
        }
    }
}

#[derive(Debug)]
pub struct Item {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub item: ItemKind,
}

impl Item {
    pub fn new(loc: Loc, attrs: Vec<AttrKind>, item: ItemKind) -> Item {
        Item {
            loc: loc,
            attrs: attrs,
            item: item,
        }
    }
}

#[derive(Debug)]
pub enum ItemKind {
    ExternCrate(ExternCrate),
    Use(Use),
    ModDecl(ModDecl),
    Mod(Mod),
    TypeAlias(TypeAlias),
}

#[derive(Debug)]
pub struct ExternCrate {
    pub head: &'static str,
    pub krate: String,
}

impl ExternCrate {
    pub fn new(krate: String) -> ExternCrate {
        static HEAD: &'static str = "extern crate ";
        ExternCrate {
            head: HEAD,
            krate: krate,
        }
    }
}

#[derive(Debug)]
pub struct Use {
    pub head: &'static str,
    pub path: String,
    pub used_items: Vec<Chunk>,
}

#[inline]
fn use_head(is_pub: bool) -> &'static str {
    static HEAD: &'static str = "use";
    static PUB_HEAD: &'static str = "pub use";

    if is_pub {
        PUB_HEAD
    } else {
        HEAD
    }
}

impl Use {
    pub fn new(is_pub: bool, path: String, used_items: Vec<Chunk>) -> Use {
        Use {
            head: use_head(is_pub),
            path: path,
            used_items: used_items,
        }
    }
}

#[inline]
fn mod_head(is_pub: bool) -> &'static str {
    static HEAD: &'static str = "mod";
    static PUB_HEAD: &'static str = "pub mod";

    if is_pub {
        PUB_HEAD
    } else {
        HEAD
    }
}

#[derive(Debug)]
pub struct ModDecl {
    pub head: &'static str,
    pub module: String,
}

impl ModDecl {
    pub fn new(is_pub: bool, module: String) -> ModDecl {
        ModDecl {
            head: mod_head(is_pub),
            module: module,
        }
    }
}

#[derive(Debug)]
pub struct TypeAlias {
    pub generics: Generics,
    pub ty: Type,
}

impl TypeAlias {
    pub fn new(generics: Generics, ty: Type) -> TypeAlias {
        TypeAlias {
            generics: generics,
            ty: ty,
        }
    }
}

#[derive(Debug)]
pub struct Generics {
    pub lifetime_defs: Vec<LifetimeDef>,
    pub type_params: Vec<TypeParam>,
}

impl Generics {
    pub fn new(lifetime_defs: Vec<LifetimeDef>, type_params: Vec<TypeParam>) -> Generics {
        Generics {
            lifetime_defs: lifetime_defs,
            type_params: type_params,
        }
    }
}

pub type Lifetime = Chunk;

#[derive(Debug)]
pub struct LifetimeDef {
    pub lifetime: Lifetime,
    pub bounds: Vec<Lifetime>,
}

impl LifetimeDef {
    pub fn new(lifetime: Lifetime, bounds: Vec<Lifetime>) -> LifetimeDef {
        LifetimeDef {
            lifetime: lifetime,
            bounds: bounds,
        }
    }
}

#[derive(Debug)]
pub struct TypeParam {
    pub loc: Loc,
    pub name: String,
    pub bounds: Vec<TypeParamBound>,
    pub default: Option<Type>,
}

impl TypeParam {
    pub fn new(loc: Loc, name: String, bounds: Vec<TypeParamBound>, default: Option<Type>)
        -> TypeParam {
        TypeParam {
            loc: loc,
            name: name,
            bounds: bounds,
            default: default,
        }
    }
}

#[derive(Debug)]
pub enum TypeParamBound {
    Lifetime(Lifetime),
    PolyTraitRef(PolyTraitRef),
}

#[derive(Debug)]
pub struct PolyTraitRef {
    pub loc: Loc,
    pub lifetimes: Vec<LifetimeDef>,
    pub trait_ref: TraitRef,
}

impl PolyTraitRef {
    pub fn new(loc: Loc, lifetimes: Vec<LifetimeDef>, trait_ref: TraitRef) -> PolyTraitRef {
        PolyTraitRef {
            loc: loc,
            lifetimes: lifetimes,
            trait_ref: trait_ref,
        }
    }

    pub fn new_maybe_sized(loc: Loc) -> PolyTraitRef {
        PolyTraitRef {
            loc: loc,
            lifetimes: Vec::new(),
            trait_ref: TraitRef::new_maybe_sized(loc),
        }
    }
}

pub type TraitRef = Path;

#[inline]
fn path_head(global: bool) -> &'static str {
    static HEAD: &'static str = "";
    static GLOBAL_HEAD: &'static str = "::";

    if global {
        GLOBAL_HEAD
    } else {
        HEAD
    }
}

#[derive(Debug)]
pub struct Path {
    pub loc: Loc,
    pub head: &'static str,
    pub segs: Vec<PathSegment>,
}

impl Path {
    pub fn new(loc: Loc, global: bool, segs: Vec<PathSegment>) -> Path {
        Path {
            loc: loc,
            head: path_head(global),
            segs: segs,
        }
    }

    pub fn new_maybe_sized(loc: Loc) -> Path {
        Path {
            loc: loc,
            head: path_head(false),
            segs: vec![PathSegment::new_maybe_sized()],
        }
    }
}

#[derive(Debug)]
pub struct PathSegment {
    pub name: String,
    pub param: PathParam,
}

impl PathSegment {
    pub fn new(name: String, param: PathParam) -> PathSegment {
        PathSegment {
            name: name,
            param: param,
        }
    }

    pub fn new_maybe_sized() -> PathSegment {
        PathSegment {
            name: "?Sized".to_string(),
            param: PathParam::Angle(Default::default()),
        }
    }
}

#[derive(Debug)]
pub enum PathParam {
    Angle(AngleParam),
    Paren(ParenParam),
}

#[derive(Debug, Default)]
pub struct AngleParam {
    pub lifetimes: Vec<Lifetime>,
    pub types: Vec<Type>,
    pub bindings: Vec<TypeBinding>,
}

impl AngleParam {
    pub fn new(lifetimes: Vec<Lifetime>, types: Vec<Type>, bindings: Vec<TypeBinding>) -> AngleParam {
        AngleParam {
            lifetimes: lifetimes,
            types: types,
            bindings: bindings,
        }
    }
}

#[derive(Debug)]
pub struct TypeBinding {
    pub loc: Loc,
    pub name: String,
    pub ty: Type,
}

impl TypeBinding {
    pub fn new(loc: Loc, name: String, ty: Type) -> TypeBinding {
        TypeBinding {
            loc: loc,
            name: name,
            ty: ty,
        }
    }
}

#[derive(Debug)]
pub struct ParenParam {
    pub loc: Loc,
    pub inputs: Vec<Type>,
    pub output: Option<Type>,
}

impl ParenParam {
    pub fn new(loc: Loc, inputs: Vec<Type>, output: Option<Type>) -> ParenParam {
        ParenParam {
            loc: loc,
            inputs: inputs,
            output: output,
        }
    }
}

#[derive(Debug)]
pub struct Type;

impl Type {
    pub fn new() -> Type {
        Type
    }
}
