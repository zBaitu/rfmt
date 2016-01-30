use std::fmt::{self, Debug, Display};

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
head_fn!(use_head, is_pub, "pub use", "use");
head_fn!(mod_head, is_pub, "pub mod", "mod");
head_fn!(path_head, global, "::", "");
head_fn!(ptr_head, is_mut, "*mut", "*const");
head_fn!(mut_deco, is_mut, "mut", "");

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

impl Use {
    pub fn new(is_pub: bool, path: String, used_items: Vec<Chunk>) -> Use {
        Use {
            head: use_head(is_pub),
            path: path,
            used_items: used_items,
        }
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
    pub name: String,
    pub generics: Generics,
    pub ty: Type,
}

impl TypeAlias {
    pub fn new(name: String, generics: Generics, ty: Type) -> TypeAlias {
        TypeAlias {
            name: name,
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
pub struct Type {
    loc: Loc,
    ty: TypeKind,
}

impl Type {
    pub fn new(loc: Loc, ty: TypeKind) -> Type {
        Type {
            loc: loc,
            ty: ty,
        }
    }
}

#[derive(Debug)]
pub enum TypeKind {
    Path(Box<PathType>),
    Ptr(Box<PtrType>),
    Ref(Box<RefType>),
    Array(Box<ArrayType>),
    FixedSizeArray(Box<FixedSizeArrayType>),
    Tuple(Box<TupleType>),
    BareFn(Box<BareFnType>),
    Sum(Box<SumType>),
    PolyTraitRef(Box<PolyTraitRefType>),
    Macro(Box<MacroType>),
    Infer
}

#[derive(Debug)]
pub struct PathType {
    pub qself: Option<Type>,
    pub path: Path,
}

impl PathType {
    pub fn new(qself: Option<Type>, path: Path) -> PathType {
        PathType {
            qself: qself,
            path: path,
        }
    }
}

#[derive(Debug)]
pub struct PtrType {
    pub head: &'static str,
    pub ty: Type,
}

impl PtrType {
    pub fn new(is_mut: bool, ty: Type) -> PtrType {
        PtrType {
            head: ptr_head(is_mut),
            ty: ty,
        }
    }
}

#[derive(Debug)]
pub struct RefType {
    pub lifetime: Option<Lifetime>,
    pub is_mut: bool,
    pub ty: Type,
}

impl RefType {
    pub fn new(lifetime: Option<Lifetime>, is_mut: bool, ty: Type) -> RefType {
        RefType {
            lifetime: lifetime,
            is_mut: is_mut,
            ty: ty,
        }
    }
}

#[derive(Debug)]
pub struct ArrayType {
    pub ty: Type,
}

impl ArrayType {
    pub fn new(ty: Type) -> ArrayType {
        ArrayType {
            ty: ty,
        }
    }
}

#[derive(Debug)]
pub struct FixedSizeArrayType {
    pub ty: Type,
    pub expr: Expr,
}

impl FixedSizeArrayType {
    pub fn new(ty: Type, expr: Expr) -> FixedSizeArrayType {
        FixedSizeArrayType {
            ty: ty,
            expr: expr,
        }
    }
}

#[derive(Debug)]
pub struct TupleType {
    pub types: Vec<Type>,
}

impl TupleType {
    pub fn new(types: Vec<Type>) -> TupleType {
        TupleType {
            types: types,
        }
    }
}

#[derive(Debug)]
pub struct BareFnType {
    pub head: String,
    pub lifetimes: Vec<LifetimeDef>,
    pub fn_decl: FnDecl,
}

impl BareFnType {
    pub fn new(is_unsafe: bool, abi: String, lifetimes: Vec<LifetimeDef>, fn_decl: FnDecl) -> BareFnType {
        BareFnType {
            head: fn_head(is_unsafe, false, &abi),
            lifetimes: lifetimes,
            fn_decl: fn_decl,
        }
    }
}

#[derive(Debug)]
pub struct SumType {
    pub ty: Type,
    pub bounds: Vec<TypeParamBound>,
}

impl SumType {
    pub fn new(ty: Type, bounds: Vec<TypeParamBound>) -> SumType {
        SumType {
            ty: ty,
            bounds: bounds,
        }
    }
}

#[derive(Debug)]
pub struct PolyTraitRefType {
    pub bounds: Vec<TypeParamBound>,
}

impl PolyTraitRefType {
    pub fn new(bounds: Vec<TypeParamBound>) -> PolyTraitRefType {
        PolyTraitRefType {
            bounds: bounds,
        }
    }
}

pub type MacroType = Macro;

fn fn_head(is_unsafe: bool, is_const: bool, abi: &str) -> String {
    let mut head = String::new();
    if is_unsafe {
        head.push_str("unsafe ");
    }
    if is_const {
        head.push_str("const ");
    }
    if abi != "Rust" {
        head.push_str(abi);
        head.push_str(" ");
    }
    head
}

#[derive(Debug)]
pub struct FnDecl;

#[derive(Debug)]
pub struct Macro;

#[derive(Debug)]
pub struct Expr;
