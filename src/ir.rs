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
head_fn!(pub_head, is_pub, "pub ", "");
head_fn!(use_head, is_pub, "pub use", "use");
head_fn!(mod_head, is_pub, "pub mod", "mod");
head_fn!(path_head, global, "::", "");
head_fn!(ptr_head, is_mut, "*mut", "*const");
head_fn!(const_head, is_pub, "pub const", "const");
head_fn!(struct_head, is_pub, "pub struct", "struct");
head_fn!(enum_head, is_pub, "pub enum", "enum");

#[inline]
fn foreign_head(abi: &str) -> String {
    format!("extern {}", abi)
}

#[inline]
fn static_head(is_pub: bool, is_mut: bool) -> String {
    let mut head = String::new();
    if is_pub {
        head.push_str("pub ");
    }
    if is_mut {
        head.push_str("mut ");
    }
    head.push_str("static ");
    head
}

#[inline]
fn fn_head(is_pub: bool, is_unsafe: bool, is_const: bool, abi: Option<&str>) -> String {
    let mut head = String::new();
    if is_pub {
        head.push_str("pub ");
    }
    if is_unsafe {
        head.push_str("unsafe ");
    }
    if is_const {
        head.push_str("const ");
    }
    if let Some(abi) = abi {
        if abi != "Rust" {
            head.push_str(&foreign_head(abi));
            head.push_str(" ");
        }
    }
    head.push_str("fn ");
    head
}

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
    pub name: String,
    pub items: Vec<Item>,
}

impl Mod {
    pub fn new(loc: Loc, name: String, items: Vec<Item>) -> Mod {
        Mod {
            loc: loc,
            name: name,
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
    ForeignMod(ForeignMod),
    Const(Const),
    Static(Static),
    Struct(Struct),
    Enum(Enum),
    Fn(Fn),
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
    Infer,
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
    pub fn new(is_unsafe: bool, abi: String, lifetimes: Vec<LifetimeDef>, fn_decl: FnDecl)
        -> BareFnType {
        BareFnType {
            head: fn_head(false, is_unsafe, false, Some(&abi)),
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

#[derive(Debug)]
pub struct ForeignMod {
    pub head: String,
    pub items: Vec<Foreign>,
}

impl ForeignMod {
    pub fn new(abi: String, items: Vec<Foreign>) -> ForeignMod {
        ForeignMod {
            head: foreign_head(&abi),
            items: items,
        }
    }
}

#[derive(Debug)]
pub struct Foreign {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub item: ForeignKind,
}

impl Foreign {
    pub fn new(loc: Loc, attrs: Vec<AttrKind>, item: ForeignKind) -> Foreign {
        Foreign {
            loc: loc,
            attrs: attrs,
            item: item,
        }
    }
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

impl ForeignStatic {
    pub fn new(is_pub: bool, is_mut: bool, name: String, ty: Type) -> ForeignStatic {
        ForeignStatic {
            head: static_head(is_pub, is_mut),
            name: name,
            ty: ty,
        }
    }
}

#[derive(Debug)]
pub struct ForeignFn {
    pub head: String,
    pub name: String,
    pub generics: Generics,
    pub fn_decl: FnDecl,
}

impl ForeignFn {
    pub fn new(is_pub: bool, name: String, generics: Generics, fn_decl: FnDecl) -> ForeignFn {
        ForeignFn {
            head: fn_head(is_pub, false, false, None),
            name: name,
            generics: generics,
            fn_decl: fn_decl,
        }
    }
}

#[derive(Debug)]
pub struct Const {
    pub head: &'static str,
    pub name: String,
    pub ty: Type,
    pub expr: Expr,
}

impl Const {
    pub fn new(is_pub: bool, name: String, ty: Type, expr: Expr) -> Const {
        Const {
            head: const_head(is_pub),
            name: name,
            ty: ty,
            expr: Expr,
        }
    }
}

#[derive(Debug)]
pub struct Static {
    pub head: String,
    pub name: String,
    pub ty: Type,
    pub expr: Expr,
}

impl Static {
    pub fn new(is_pub: bool, is_mut: bool, name: String, ty: Type, expr: Expr) -> Static {
        Static {
            head: static_head(is_pub, is_mut),
            name: name,
            ty: ty,
            expr: Expr,
        }
    }
}

#[derive(Debug)]
pub struct Struct {
    pub head: &'static str,
    pub name: String,
    pub generics: Generics,
    pub body: StructBody,
}

impl Struct {
    pub fn new(is_pub: bool, name: String, generics: Generics, body: StructBody) -> Struct {
        Struct {
            head: struct_head(is_pub),
            name: name,
            generics: generics,
            body: body,
        }
    }
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

impl StructField {
    pub fn new(loc: Loc, attrs: Vec<AttrKind>, is_pub: bool, name: String, ty: Type) -> StructField {
        StructField {
            loc: loc,
            attrs: attrs,
            head: pub_head(is_pub),
            name: name,
            ty: ty,
        }
    }
}

#[derive(Debug)]
pub struct TupleField {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub head: &'static str,
    pub ty: Type,
}

impl TupleField {
    pub fn new(loc: Loc, attrs: Vec<AttrKind>, is_pub: bool, ty: Type) -> TupleField {
        TupleField {
            loc: loc,
            attrs: attrs,
            head: pub_head(is_pub),
            ty: ty,
        }
    }
}

#[derive(Debug)]
pub struct Enum {
    pub head: &'static str,
    pub name: String,
    pub generics: Generics,
    pub body: EnumBody,
}

impl Enum {
    pub fn new(is_pub: bool, name: String, generics: Generics, body: EnumBody) -> Enum {
        Enum {
            head: enum_head(is_pub),
            name: name,
            generics: generics,
            body: body,
        }
    }
}

#[derive(Debug)]
pub struct EnumBody {
    pub fields: Vec<EnumField>,
}

impl EnumBody {
    pub fn new(fields: Vec<EnumField>) -> EnumBody {
        EnumBody {
            fields: fields,
        }
    }
}

#[derive(Debug)]
pub struct EnumField {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub name: String,
    pub body: StructBody,
    pub expr: Option<Expr>,
}

impl EnumField {
    pub fn new(loc: Loc, attrs: Vec<AttrKind>, name: String, body: StructBody, expr: Option<Expr>)
        -> EnumField {
        EnumField {
            loc: loc,
            attrs: attrs,
            name: name,
            body: body,
            expr: expr,
        }
    }
}

#[derive(Debug)]
pub struct Fn {
    pub head: String,
    pub name: String,
    pub generics: Generics,
    pub fn_decl: FnDecl,
    pub block: Block,
}

impl Fn {
    pub fn new(is_pub: bool, is_unsafe: bool, is_const: bool, abi: String, name: String,
               generics: Generics, fn_decl: FnDecl, block: Block)
        -> Fn {
        Fn {
            head: fn_head(is_pub, is_unsafe, is_const, Some(&abi)),
            name: name,
            generics: generics,
            fn_decl: fn_decl,
            block: block,
        }
    }
}

#[derive(Debug)]
pub struct FnDecl;

#[derive(Debug)]
pub struct Block;

#[derive(Debug)]
pub struct Expr;

#[derive(Debug)]
pub struct Macro;

pub type MacroType = Macro;
