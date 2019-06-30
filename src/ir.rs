use std::fmt::{self, Debug, Display};

pub type Pos = u32;

#[derive(Clone, Copy, Default)]
pub struct Loc {
    pub start: Pos,
    pub end: Pos,
    pub nl: bool,
}

impl Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.nl {
            write!(f, "Loc({}, {}, nl)", self.start, self.end)
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

#[derive(Debug, PartialEq)]
pub enum CommentKind {
    Leading,
    Trailing,
}

#[derive(Debug)]
pub struct Comment {
    pub pos: Pos,
    pub kind: CommentKind,
    pub lines: Vec<String>,
}


#[derive(Debug, Default)]
pub struct Chunk {
    pub loc: Loc,
    pub s: String,
}

impl Chunk {
    pub fn new<S>(s: S) -> Chunk where S: Into<String> {
        Chunk {
            loc: Default::default(),
            s: s.into(),
        }
    }
}

#[derive(Debug)]
pub struct Crate {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub module: Mod,
}

pub type Doc = Chunk;

#[derive(Debug)]
pub enum AttrKind {
    Doc(Doc),
    Attr(Attr),
}

#[derive(Debug)]
pub struct Attr {
    pub loc: Loc,
    pub is_inner: bool,
    pub item: MetaItem,
}

#[derive(Debug)]
pub struct MetaItem {
    pub loc: Loc,
    pub name: String,
    pub items: Option<Box<Vec<MetaItem>>>,
}

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
    pub vis: Vis,
    pub item: ItemKind,
}

#[derive(Debug)]
pub struct Vis {
    pub name: String,
}

#[derive(Debug)]
pub enum ItemKind {
    Mod(Mod),
    ModDecl(ModDecl),
    ExternCrate(ExternCrate),
    /*
    Use(Use),
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
    Macro(MacroRaw),
    */
}

#[derive(Debug)]
pub struct ModDecl {
    pub name: String,
}

#[derive(Debug)]
pub struct ExternCrate {
    pub name: String,
}

/*
#[derive(Debug)]
pub struct Use {
    pub base: String,
    pub names: Vec<Chunk>,
}

#[derive(Debug)]
pub struct TypeAlias {
    pub name: String,
    pub generics: Generics,
    pub ty: Type,
}

#[derive(Debug)]
pub struct Generics {
    pub lifetime_defs: Vec<LifetimeDef>,
    pub type_params: Vec<TypeParam>,
    pub wh: Where,
}

impl Generics {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.lifetime_defs.is_empty() && self.type_params.is_empty()
    }
}

#[derive(Debug)]
pub struct LifetimeDef {
    pub loc: Loc,
    pub lifetime: Lifetime,
    pub bounds: Vec<Lifetime>,
}

pub type Lifetime = Chunk;

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
pub struct Where {
    pub clauses: Vec<WhereClause>,
}

impl Where {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }
}

#[derive(Debug)]
pub struct WhereClause {
    pub loc: Loc,
    pub clause: WhereKind,
}

#[derive(Debug)]
pub enum WhereKind {
    LifetimeDef(LifetimeDef),
    Bound(WhereBound),
}

#[derive(Debug)]
pub struct WhereBound {
    pub lifetime_defs: Vec<LifetimeDef>,
    pub ty: Type,
    pub bounds: Vec<TypeParamBound>,
}

#[derive(Debug)]
pub struct Path {
    pub loc: Loc,
    pub global: bool,
    pub segs: Vec<PathSegment>,
}

impl Path {
    pub fn new_sized(loc: Loc) -> Path {
        Path {
            loc: loc,
            global: Default::default(),
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

impl AngleParam {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.lifetimes.is_empty() && self.types.is_empty() && self.bindings.is_empty()
    }
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
pub struct QSelf {
    pub ty: Type,
    pub pos: usize,
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
    Macro(Box<Macro>),
    Infer,
}

#[derive(Debug)]
pub struct PathType {
    pub qself: Option<QSelf>,
    pub path: Path,
}

#[derive(Debug)]
pub struct PtrType {
    pub is_mut: bool,
    pub ty: Type,
}

#[derive(Debug)]
pub struct RefType {
    pub lifetime: Option<Lifetime>,
    pub is_mut: bool,
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
    pub lifetime_defs: Vec<LifetimeDef>,
    pub is_unsafe: bool,
    pub abi: String,
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
    pub abi: String,
    pub items: Vec<ForeignItem>,
}

#[derive(Debug)]
pub struct ForeignItem {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub is_pub: bool,
    pub item: ForeignKind,
}

#[derive(Debug)]
pub enum ForeignKind {
    Static(ForeignStatic),
    Fn(ForeignFn),
}

#[derive(Debug)]
pub struct ForeignStatic {
    pub is_mut: bool,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug)]
pub struct ForeignFn {
    pub name: String,
    pub generics: Generics,
    pub fn_sig: FnSig,
}

#[derive(Debug)]
pub struct Const {
    pub name: String,
    pub ty: Type,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct Static {
    pub is_mut: bool,
    pub name: String,
    pub ty: Type,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct Struct {
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
    pub is_pub: bool,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug)]
pub struct TupleField {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub is_pub: bool,
    pub ty: Type,
}

#[derive(Debug)]
pub struct Enum {
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
    pub is_unsafe: bool,
    pub is_const: bool,
    pub abi: String,
    pub name: String,
    pub generics: Generics,
    pub fn_sig: FnSig,
    pub block: Block,
}

#[derive(Debug)]
pub struct Trait {
    pub is_unsafe: bool,
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
    pub name: String,
    pub ty: Type,
    pub expr: Option<Expr>,
}

#[derive(Debug)]
pub struct TypeTraitItem {
    pub name: String,
    pub bounds: Vec<TypeParamBound>,
    pub ty: Option<Type>,
}

#[derive(Debug)]
pub struct MethodTraitItem {
    pub is_unsafe: bool,
    pub is_const: bool,
    pub abi: String,
    pub name: String,
    pub method_sig: MethodSig,
    pub block: Option<Block>,
}

#[derive(Debug)]
pub struct ImplDefault {
    pub is_unsafe: bool,
    pub trait_ref: TraitRef,
}

#[derive(Debug)]
pub struct Impl {
    pub is_unsafe: bool,
    pub is_neg: bool,
    pub generics: Generics,
    pub trait_ref: Option<TraitRef>,
    pub ty: Type,
    pub items: Vec<ImplItem>,
}

#[derive(Debug)]
pub struct ImplItem {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub is_pub: bool,
    pub is_default: bool,
    pub item: ImplItemKind,
}

#[derive(Debug)]
pub enum ImplItemKind {
    Const(ConstImplItem),
    Type(TypeImplItem),
    Method(MethodImplItem),
    Macro(Macro),
}

#[derive(Debug)]
pub struct ConstImplItem {
    pub name: String,
    pub ty: Type,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct TypeImplItem {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug)]
pub struct MethodImplItem {
    pub is_unsafe: bool,
    pub is_const: bool,
    pub abi: String,
    pub name: String,
    pub method_sig: MethodSig,
    pub block: Block,
}

#[derive(Debug)]
pub struct FnSig {
    pub arg: FnArg,
    pub ret: FnReturn,
}

#[derive(Debug)]
pub struct FnArg {
    pub args: Vec<Arg>,
    pub va: bool,
}

#[derive(Debug)]
pub struct Arg {
    pub loc: Loc,
    pub pat: Patten,
    pub ty: Type,
}

#[derive(Debug)]
pub struct FnReturn {
    pub nl: bool,
    pub ret: FnReturnKind,
}

#[derive(Debug)]
pub enum FnReturnKind {
    Unit,
    Diverge,
    Normal(Type),
}

#[derive(Debug)]
pub struct MethodSig {
    pub generics: Generics,
    pub sf: Option<Sf>,
    pub fn_sig: FnSig,
}

#[derive(Debug)]
pub enum Sf {
    String(String),
    Type(Type),
}

#[derive(Debug)]
pub struct Block {
    pub loc: Loc,
    pub is_unsafe: bool,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub struct Stmt {
    pub loc: Loc,
    pub stmt: StmtKind,
}

#[derive(Debug)]
pub enum StmtKind {
    Decl(Decl),
    Expr(Expr, bool),
    Macro(MacroStmt, bool),
}

#[derive(Debug)]
pub struct Decl {
    pub loc: Loc,
    pub decl: DeclKind,
}

#[derive(Debug)]
pub enum DeclKind {
    Local(Local),
    Item(Item),
}

#[derive(Debug)]
pub struct Local {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub pat: Patten,
    pub ty: Option<Type>,
    pub init: Option<Expr>,
}

#[derive(Debug)]
pub struct Patten {
    pub loc: Loc,
    pub pat: PattenKind,
}

#[derive(Debug)]
pub enum PattenKind {
    Wildcard,
    Literal(Expr),
    Range(RangePatten),
    Ident(Box<IdentPatten>),
    Ref(Box<RefPatten>),
    Path(PathPatten),
    Enum(EnumPatten),
    Struct(Box<StructPatten>),
    Vec(Box<VecPatten>),
    Tuple(Box<TuplePatten>),
    Box(Box<Patten>),
    Macro(Macro),
}

#[derive(Debug)]
pub struct RangePatten {
    pub start: Expr,
    pub end: Expr,
}

#[derive(Debug)]
pub struct IdentPatten {
    pub is_ref: bool,
    pub is_mut: bool,
    pub name: Chunk,
    pub binding: Option<Patten>,
}

#[derive(Debug)]
pub struct RefPatten {
    pub is_mut: bool,
    pub pat: Patten,
}

#[derive(Debug)]
pub struct PathPatten {
    pub qself: Option<QSelf>,
    pub path: Path,
}

#[derive(Debug)]
pub struct EnumPatten {
    pub path: Path,
    pub pats: Option<Vec<Patten>>,
}

#[derive(Debug)]
pub struct StructPatten {
    pub path: Path,
    pub fields: Vec<StructFieldPatten>,
    pub etc: bool,
}

#[derive(Debug)]
pub struct StructFieldPatten {
    pub loc: Loc,
    pub name: String,
    pub pat: Patten,
    pub shorthand: bool,
}

#[derive(Debug)]
pub struct VecPatten {
    pub start: Vec<Patten>,
    pub emit: Option<Patten>,
    pub end: Vec<Patten>,
}

#[derive(Debug)]
pub struct TuplePatten {
    pub pats: Vec<Patten>,
}

#[derive(Debug)]
pub struct Expr {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub expr: ExprKind,
}

#[derive(Debug)]
pub enum ExprKind {
    Literal(Chunk),
    Path(PathExpr),
    Unary(Box<UnaryExpr>),
    Ref(Box<RefExpr>),
    List(Box<ListExpr>),
    FixedSizeArray(Box<FixedSizeArrayExpr>),
    Vec(Box<Vec<Expr>>),
    Tuple(Box<Vec<Expr>>),
    FieldAccess(Box<FieldAccessExpr>),
    Struct(Box<StructExpr>),
    Index(Box<IndexExpr>),
    Range(Box<RangeExpr>),
    Box(Box<BoxExpr>),
    Cast(Box<CastExpr>),
    Type(Box<TypeExpr>),
    Block(Box<Block>),
    If(Box<IfExpr>),
    IfLet(Box<IfLetExpr>),
    While(Box<WhileExpr>),
    WhileLet(Box<WhileLetExpr>),
    For(Box<ForExpr>),
    Loop(Box<LoopExpr>),
    Break(Box<BreakExpr>),
    Continue(Box<ContinueExpr>),
    Match(Box<MatchExpr>),
    FnCall(Box<FnCallExpr>),
    MethodCall(Box<MethodCallExpr>),
    Closure(Box<ClosureExpr>),
    Return(Box<ReturnExpr>),
    Try(Box<Expr>),
    Macro(Macro),
}

pub type PathExpr = PathType;

#[derive(Debug)]
pub struct UnaryExpr {
    pub op: &'static str,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct RefExpr {
    pub is_mut: bool,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct ListExpr {
    pub exprs: Vec<Expr>,
    pub sep: Chunk,
}

#[derive(Debug)]
pub struct FixedSizeArrayExpr {
    pub init: Expr,
    pub len: Expr,
}

#[derive(Debug)]
pub struct FieldAccessExpr {
    pub expr: Expr,
    pub field: Chunk,
}

#[derive(Debug)]
pub struct StructExpr {
    pub path: Path,
    pub fields: Vec<StructFieldExpr>,
    pub base: Option<Expr>,
}

#[derive(Debug)]
pub struct StructFieldExpr {
    pub loc: Loc,
    pub name: Chunk,
    pub value: Expr,
}

#[derive(Debug)]
pub struct IndexExpr {
    pub obj: Expr,
    pub index: Expr,
}

#[derive(Debug)]
pub struct RangeExpr {
    pub start: Option<Expr>,
    pub end: Option<Expr>,
    pub is_halfopen: bool,
}

#[derive(Debug)]
pub struct BoxExpr {
    pub expr: Expr,
}

#[derive(Debug)]
pub struct CastExpr {
    pub expr: Expr,
    pub ty: Type,
}

#[derive(Debug)]
pub struct TypeExpr {
    pub expr: Expr,
    pub ty: Type,
}

#[derive(Debug)]
pub struct IfExpr {
    pub expr: Expr,
    pub block: Block,
    pub br: Option<Expr>,
}

#[derive(Debug)]
pub struct IfLetExpr {
    pub pat: Patten,
    pub expr: Expr,
    pub block: Block,
    pub br: Option<Expr>,
}

#[derive(Debug)]
pub struct WhileExpr {
    pub label: Option<String>,
    pub expr: Expr,
    pub block: Block,
}

#[derive(Debug)]
pub struct WhileLetExpr {
    pub label: Option<String>,
    pub pat: Patten,
    pub expr: Expr,
    pub block: Block,
}

#[derive(Debug)]
pub struct ForExpr {
    pub label: Option<String>,
    pub pat: Patten,
    pub expr: Expr,
    pub block: Block,
}

#[derive(Debug)]
pub struct LoopExpr {
    pub label: Option<String>,
    pub block: Block,
}

#[derive(Debug)]
pub struct BreakExpr {
    pub label: Option<Chunk>,
}

#[derive(Debug)]
pub struct ContinueExpr {
    pub label: Option<Chunk>,
}

#[derive(Debug)]
pub struct MatchExpr {
    pub expr: Expr,
    pub arms: Vec<Arm>,
}

#[derive(Debug)]
pub struct Arm {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub pats: Vec<Patten>,
    pub guard: Option<Expr>,
    pub body: Expr,
}

#[derive(Debug)]
pub struct FnCallExpr {
    pub name: Expr,
    pub args: Vec<Expr>,
}

#[derive(Debug)]
pub struct MethodCallExpr {
    pub obj: Expr,
    pub name: Chunk,
    pub types: Vec<Type>,
    pub args: Vec<Expr>,
}

#[derive(Debug)]
pub struct ClosureExpr {
    pub moved: bool,
    pub fn_sig: FnSig,
    pub block: Block,
}

#[derive(Debug)]
pub struct ReturnExpr {
    pub ret: Option<Expr>,
}

#[derive(Debug)]
pub enum MacroStyle {
    Paren,
    Bracket,
    Brace,
}

#[derive(Debug)]
pub struct MacroRaw {
    pub style: MacroStyle,
    pub s: Chunk,
}

#[derive(Debug)]
pub struct MacroStmt {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub mac: Macro,
}

#[derive(Debug)]
pub enum Macro {
    Raw(MacroRaw),
    Expr(MacroExpr),
}

#[derive(Debug)]
pub struct MacroExprSep {
    pub is_sep: bool,
    pub s: &'static str,
}

#[derive(Debug)]
pub struct MacroExpr {
    pub name: String,
    pub style: MacroStyle,
    pub exprs: Vec<Expr>,
    pub seps: Vec<MacroExprSep>,
}
*/
