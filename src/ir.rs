use std::fmt::{self, Debug, Display};

#[derive(Clone, Copy, Default)]
pub struct Loc {
    pub start: u32,
    pub end: u32,
    pub wrapped: bool,
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
    pub loc: Loc,
    pub s: String,
}

#[derive(Debug)]
pub struct Crate {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub module: Mod,
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
    pub item: MetaItem,
}

#[derive(Debug)]
pub enum MetaItem {
    Single(Chunk),
    List(Loc, String, Vec<MetaItem>),
}

pub type Doc = Chunk;

#[derive(Debug)]
pub struct Mod {
    pub loc: Loc,
    pub name: String,
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub struct Item {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub item: ItemKind,
}

#[derive(Debug)]
pub enum ItemKind {
    ExternCrate(ExternCrate),
    Use(Use),
    ModDecl(ModDecl),
    Mod(Mod),
    TypeAlias(TypeAlias),
    ForeignMod(ForeignMod),
    Const(Const),
    Static(Static),
    Struct(Struct),
    Enum(Enum),
    Fn(Fn),
    Trait(Trait),
    ImplDefault(ImplDefault),
    Impl(Impl),
    Macro(Macro),
}

#[derive(Debug)]
pub struct ExternCrate {
    pub head: &'static str,
    pub name: String,
}

#[derive(Debug)]
pub struct Use {
    pub head: &'static str,
    pub path: String,
    pub items: Vec<Chunk>,
}

#[derive(Debug)]
pub struct ModDecl {
    pub head: &'static str,
    pub name: String,
}

#[derive(Debug)]
pub struct TypeAlias {
    pub head: &'static str,
    pub name: String,
    pub generics: Generics,
    pub ty: Type,
}

#[derive(Debug)]
pub struct Generics {
    pub lifetime_defs: Vec<LifetimeDef>,
    pub type_params: Vec<TypeParam>,
}

pub type Lifetime = Chunk;

#[derive(Debug)]
pub struct LifetimeDef {
    pub lifetime: Lifetime,
    pub bounds: Vec<Lifetime>,
}

#[derive(Debug)]
pub struct TypeParam {
    pub loc: Loc,
    pub name: String,
    pub bounds: Vec<TypeParamBound>,
    pub default: Option<Type>,
}

#[derive(Debug)]
pub enum TypeParamBound {
    Lifetime(Lifetime),
    PolyTraitRef(PolyTraitRef),
}

#[derive(Debug)]
pub struct PolyTraitRef {
    pub loc: Loc,
    pub lifetime_defs: Vec<LifetimeDef>,
    pub trait_ref: TraitRef,
}

impl PolyTraitRef {
    pub fn new_sized(loc: Loc) -> PolyTraitRef {
        PolyTraitRef {
            loc: loc,
            lifetime_defs: Vec::new(),
            trait_ref: TraitRef::new_sized(loc),
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
    pub fn new_sized(loc: Loc) -> Path {
        Path {
            loc: loc,
            head: Default::default(),
            segs: vec![PathSegment::new_sized()],
        }
    }
}

#[derive(Debug)]
pub struct PathSegment {
    pub name: String,
    pub param: PathParam,
}

impl PathSegment {
    pub fn new_sized() -> PathSegment {
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

#[derive(Debug)]
pub struct TypeBinding {
    pub loc: Loc,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug)]
pub struct ParenParam {
    pub loc: Loc,
    pub inputs: Vec<Type>,
    pub output: Option<Type>,
}

#[derive(Debug)]
pub struct Type {
    pub loc: Loc,
    pub ty: TypeKind,
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
    Infer,
}

#[derive(Debug)]
pub struct PathType {
    pub qself: Option<Type>,
    pub path: Path,
}

#[derive(Debug)]
pub struct PtrType {
    pub head: &'static str,
    pub ty: Type,
}

#[derive(Debug)]
pub struct RefType {
    pub head: String,
    pub ty: Type,
}

#[derive(Debug)]
pub struct ArrayType {
    pub ty: Type,
}

#[derive(Debug)]
pub struct FixedSizeArrayType {
    pub ty: Type,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct TupleType {
    pub types: Vec<Type>,
}

#[derive(Debug)]
pub struct BareFnType {
    pub head: String,
    pub lifetime_defs: Vec<LifetimeDef>,
    pub fn_sig: FnSig,
}

#[derive(Debug)]
pub struct SumType {
    pub ty: Type,
    pub bounds: Vec<TypeParamBound>,
}

#[derive(Debug)]
pub struct PolyTraitRefType {
    pub bounds: Vec<TypeParamBound>,
}

#[derive(Debug)]
pub struct ForeignMod {
    pub head: String,
    pub items: Vec<Foreign>,
}

#[derive(Debug)]
pub struct Foreign {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub item: ForeignKind,
}

#[derive(Debug)]
pub enum ForeignKind {
    Static(ForeignStatic),
    Fn(ForeignFn),
}

#[derive(Debug)]
pub struct ForeignStatic {
    pub head: String,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug)]
pub struct ForeignFn {
    pub head: String,
    pub name: String,
    pub generics: Generics,
    pub fn_sig: FnSig,
}

#[derive(Debug)]
pub struct Const {
    pub head: &'static str,
    pub name: String,
    pub ty: Type,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct Static {
    pub head: String,
    pub name: String,
    pub ty: Type,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct Struct {
    pub head: &'static str,
    pub name: String,
    pub generics: Generics,
    pub body: StructBody,
}

#[derive(Debug)]
pub enum StructBody {
    Struct(Vec<StructField>),
    Tuple(Vec<TupleField>),
    Unit,
}

#[derive(Debug)]
pub struct StructField {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub head: &'static str,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug)]
pub struct TupleField {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub head: &'static str,
    pub ty: Type,
}

#[derive(Debug)]
pub struct Enum {
    pub head: &'static str,
    pub name: String,
    pub generics: Generics,
    pub body: EnumBody,
}

#[derive(Debug)]
pub struct EnumBody {
    pub fields: Vec<EnumField>,
}

#[derive(Debug)]
pub struct EnumField {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub name: String,
    pub body: StructBody,
    pub expr: Option<Expr>,
}

#[derive(Debug)]
pub struct Fn {
    pub head: String,
    pub name: String,
    pub generics: Generics,
    pub fn_sig: FnSig,
    pub block: Block,
}

#[derive(Debug)]
pub struct Trait {
    pub head: String,
    pub name: String,
    pub generics: Generics,
    pub bounds: Vec<TypeParamBound>,
    pub items: Vec<TraitItem>,
}

#[derive(Debug)]
pub struct TraitItem {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub item: TraitItemKind,
}

#[derive(Debug)]
pub enum TraitItemKind {
    Const(ConstTraitItem),
    Type(TypeTraitItem),
    Method(MethodTraitItem),
}

#[derive(Debug)]
pub struct ConstTraitItem {
    pub head: &'static str,
    pub name: String,
    pub ty: Type,
    pub expr: Option<Expr>,
}

#[derive(Debug)]
pub struct TypeTraitItem {
    pub head: &'static str,
    pub name: String,
    pub bounds: Vec<TypeParamBound>,
    pub ty: Option<Type>,
}

#[derive(Debug)]
pub struct MethodTraitItem {
    pub head: String,
    pub name: String,
    pub method_sig: MethodSig,
    pub block: Option<Block>,
}

#[derive(Debug)]
pub struct MethodSig {
    pub generics: Generics,
    pub fn_sig: FnSig,
    pub slf: Option<String>,
}

#[derive(Debug)]
pub struct ImplDefault {
    pub head: String,
    pub trait_ref: TraitRef,
    pub tail: &'static str,
}

#[derive(Debug)]
pub struct Impl {
    pub head: String,
    pub polarity: &'static str,
    pub generics: Generics,
    pub trait_ref: Option<TraitRef>,
    pub ty: Type,
    pub items: Vec<ImplItem>,
}

#[derive(Debug)]
pub struct ImplItem {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub item: ImplItemKind,
}

#[derive(Debug)]
pub enum ImplItemKind {
    Const(ConstImplItem),
    Type(TypeImplItem),
    Method(MethodImplItem),
    Macro(MacroImplItem),
}

#[derive(Debug)]
pub struct ConstImplItem {
    pub head: &'static str,
    pub name: String,
    pub ty: Type,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct TypeImplItem {
    pub head: &'static str,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug)]
pub struct MethodImplItem {
    pub head: String,
    pub name: String,
    pub method_sig: MethodSig,
    pub block: Block,
}

#[derive(Debug)]
pub struct FnSig;

#[derive(Debug)]
pub struct Block;

#[derive(Debug)]
pub struct Expr;

pub type MacroType = Macro;
pub type MacroImplItem = Macro;

#[derive(Debug)]
pub struct Macro;
