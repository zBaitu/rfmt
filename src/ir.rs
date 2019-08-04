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
    pub items: Option<Vec<MetaItem>>,
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

pub type Vis = String;

#[derive(Debug)]
pub enum ItemKind {
    Mod(Mod),
    ModDecl(ModDecl),
    ExternCrate(ExternCrate),
    Use(Use),
    TypeAlias(TypeAlias),
    TraitAlias(TraitAlias),
    Existential(Existential),
    Const(Const),
    Static(Static),
    Struct(Struct),
    Union(Union),
    Enum(Enum),
    ForeignMod(ForeignMod),
    Fn(Fn),
    Trait(Trait),
    Impl(Impl),
    MacroDef(MacroDef),
    Macro(Macro),
}

#[derive(Debug)]
pub struct ModDecl {
    pub name: String,
}

#[derive(Debug)]
pub struct ExternCrate {
    pub name: String,
}

#[derive(Debug)]
pub struct Use {
    pub path: String,
    pub trees: Option<Vec<UseTree>>,
}

#[derive(Debug)]
pub struct UseTree {
    pub loc: Loc,
    pub path: String,
    pub trees: Option<Vec<UseTree>>,
}

#[derive(Debug)]
pub struct TypeAlias {
    pub name: String,
    pub generics: Generics,
    pub ty: Type,
}

#[derive(Debug)]
pub struct TraitAlias {
    pub name: String,
    pub generics: Generics,
    pub bounds: TypeParamBounds,
}

#[derive(Debug)]
pub struct Existential {
    pub name: String,
    pub generics: Generics,
    pub bounds: TypeParamBounds,
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
    pub bounds: TypeParamBounds,
    pub default: Option<Type>,
}

#[derive(Debug)]
pub enum TypeParamBound {
    Lifetime(Lifetime),
    PolyTraitRef(PolyTraitRef),
}

#[derive(Debug)]
pub struct TypeParamBounds(pub Vec<TypeParamBound>);

impl TypeParamBounds {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
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
            loc,
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
    pub bounds: TypeParamBounds,
}

#[derive(Debug)]
pub struct Path {
    pub loc: Loc,
    pub segments: Vec<PathSegment>,
}

impl Path {
    pub fn new_sized(loc: Loc) -> Path {
        Path {
            loc,
            segments: vec![PathSegment::new_sized()],
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

    #[inline]
    pub fn is_empty(&self) -> bool {
        match self.param {
            PathParam::Angle(ref param) => param.is_empty(),
            PathParam::Paren(ref param) => param.is_empty(),
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
pub enum TypeBindingKind {
    Eq(Type),
    Bound(TypeParamBounds),
}

#[derive(Debug)]
pub struct TypeBinding {
    pub loc: Loc,
    pub name: String,
    pub binding: TypeBindingKind,
}

#[derive(Debug)]
pub struct ParenParam {
    pub loc: Loc,
    pub inputs: Vec<Type>,
    pub output: Option<Type>,
}

impl ParenParam {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inputs.is_empty()
    }
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
    Symbol(&'static str),
    Path(Box<PathType>),
    Ptr(Box<PtrType>),
    Ref(Box<RefType>),
    Tuple(Box<TupleType>),
    Slice(Box<SliceType>),
    Array(Box<ArrayType>),
    Trait(Box<TraitType>),
    BareFn(Box<BareFnType>),
    Macro(Macro),
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
pub struct TupleType {
    pub types: Vec<Type>,
}

#[derive(Debug)]
pub struct SliceType {
    pub ty: Type,
}

#[derive(Debug)]
pub struct ArrayType {
    pub ty: Type,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct TraitType {
    pub is_dyn: bool,
    pub is_impl: bool,
    pub bounds: TypeParamBounds,
}

#[derive(Debug)]
pub struct BareFnType {
    pub lifetime_defs: Vec<LifetimeDef>,
    pub header: FnHeader,
    pub sig: FnSig,
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
    pub vis: Vis,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug)]
pub struct TupleField {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub vis: Vis,
    pub ty: Type,
}

#[derive(Debug)]
pub struct Union {
    pub name: String,
    pub generics: Generics,
    pub fields: Vec<StructField>,
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
pub struct FnSig {
    pub args: Vec<Arg>,
    pub ret: Return,
}

#[derive(Debug)]
pub struct Arg {
    pub loc: Loc,
    pub patten: Patten,
    pub ty: Type,
    pub has_patten: bool,
}

#[derive(Debug)]
pub struct Return {
    pub nl: bool,
    pub ret: Option<Type>,
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
    pub vis: Vis,
    pub item: ForeignKind,
}

#[derive(Debug)]
pub enum ForeignKind {
    Type(ForeignType),
    Static(ForeignStatic),
    Fn(ForeignFn),
    Macro(Macro),
}

pub type ForeignType = String;

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
    pub sig: FnSig,
}

#[derive(Debug, Default)]
pub struct FnHeader {
    pub is_unsafe: bool,
    pub is_async: bool,
    pub is_const: bool,
    pub abi: String,
}

#[derive(Debug)]
pub struct Fn {
    pub header: FnHeader,
    pub name: String,
    pub generics: Generics,
    pub sig: FnSig,
    pub block: Block,
}

#[derive(Debug)]
pub struct MethodSig {
    pub header: FnHeader,
    pub name: String,
    pub generics: Generics,
    pub sig: FnSig,
}

#[derive(Debug)]
pub struct Trait {
    pub is_auto: bool,
    pub is_unsafe: bool,
    pub name: String,
    pub generics: Generics,
    pub bounds: TypeParamBounds,
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
    Macro(Macro),
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
    pub generics: Generics,
    pub bounds: TypeParamBounds,
    pub ty: Option<Type>,
}

#[derive(Debug)]
pub struct MethodTraitItem {
    pub sig: MethodSig,
    pub block: Option<Block>,
}

#[derive(Debug)]
pub struct Impl {
    pub is_unsafe: bool,
    pub is_default: bool,
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
    pub vis: Vis,
    pub is_default: bool,
    pub item: ImplItemKind,
}

#[derive(Debug)]
pub enum ImplItemKind {
    Const(ConstImplItem),
    Type(TypeImplItem),
    Existential(ExistentialImplItem),
    Method(MethodImplItem),
    Macro(Macro),
}

pub type ConstImplItem = Const;

#[derive(Debug)]
pub struct TypeImplItem {
    pub name: String,
    pub generics: Generics,
    pub ty: Type,
}

#[derive(Debug)]
pub struct ExistentialImplItem {
    pub name: String,
    pub generics: Generics,
    pub bounds: TypeParamBounds,
}

#[derive(Debug)]
pub struct MethodImplItem {
    pub sig: MethodSig,
    pub block: Block,
}

#[derive(Debug)]
pub struct Block {
    pub loc: Loc,
    pub is_unsafe: bool,
    pub stmts: Vec<Stmt>,
}

impl Block {
    #[inline]
    pub fn is_one_literal_expr(&self) -> bool {
        if self.stmts.len() != 1 {
            return false;
        }

        match &self.stmts[0].stmt {
            StmtKind::Expr(ref expr, _) => {
                if let ExprKind::Literal(_) = expr.expr {
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Stmt {
    pub loc: Loc,
    pub stmt: StmtKind,
}

#[derive(Debug)]
pub enum StmtKind {
    Item(Item),
    Let(Let),
    Expr(Expr, bool),
    Macro(MacroStmt),
}

#[derive(Debug)]
pub struct Let {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub patten: Patten,
    pub ty: Option<Type>,
    pub init: Option<Expr>,
}

#[derive(Debug)]
pub struct Patten {
    pub loc: Loc,
    pub patten: PattenKind,
}

#[derive(Debug)]
pub enum PattenKind {
    Wildcard,
    Symbol(&'static str),
    Literal(Expr),
    Range(RangePatten),
    Ref(Box<RefPatten>),
    Path(PathPatten),
    Ident(Box<IdentPatten>),
    Struct(StructPatten),
    Enum(EnumPatten),
    Tuple(TuplePatten),
    Slice(Box<SlicePatten>),
    Macro(Macro),
}

#[derive(Debug)]
pub struct RangePatten {
    pub start: Expr,
    pub end: Expr,
    pub is_inclusive: bool,
}

#[derive(Debug)]
pub struct RefPatten {
    pub is_mut: bool,
    pub patten: Patten,
}

pub type PathPatten = PathType;

#[derive(Debug)]
pub struct IdentPatten {
    pub is_ref: bool,
    pub is_mut: bool,
    pub name: String,
    pub patten: Option<Patten>,
}

#[derive(Debug)]
pub struct StructPatten {
    pub path: Path,
    pub fields: Vec<StructFieldPatten>,
    pub omit: bool,
}

#[derive(Debug)]
pub struct StructFieldPatten {
    pub loc: Loc,
    pub name: String,
    pub patten: Patten,
    pub shorthand: bool,
}

#[derive(Debug)]
pub struct EnumPatten {
    pub path: Path,
    pub pattens: Vec<Patten>,
}

#[derive(Debug)]
pub struct TuplePatten {
    pub pattens: Vec<Patten>,
}

#[derive(Debug)]
pub struct SlicePatten {
    pub pattens: Vec<Patten>,
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
    Ref(Box<RefExpr>),
    UnaryOp(Box<UnaryOpExpr>),
    Try(Box<Expr>),
    ListOp(Box<ListOpExpr>),
    Repeat(Box<RepeatExpr>),
    Array(Box<Vec<Expr>>),
    Tuple(Box<Vec<Expr>>),
    Index(Box<IndexExpr>),
    Struct(Box<StructExpr>),
    Field(Box<FieldExpr>),
    Type(Box<TypeExpr>),
    Cast(Box<CastExpr>),
    Range(Box<RangeExpr>),
    Block(Box<BlockExpr>),
    If(Box<IfExpr>),
    While(Box<WhileExpr>),
    Let(Box<LetExpr>),
    For(Box<ForExpr>),
    Loop(Box<LoopExpr>),
    Break(Box<BreakExpr>),
    Continue(Box<ContinueExpr>),
    Match(Box<MatchExpr>),
    FnCall(Box<FnCallExpr>),
    MethodCall(Box<MethodCallExpr>),
    Closure(Box<ClosureExpr>),
    Return(Box<ReturnExpr>),
    Macro(Macro),
}

pub type PathExpr = PathType;

#[derive(Debug)]
pub struct RefExpr {
    pub is_mut: bool,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct UnaryOpExpr {
    pub op: &'static str,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct ListOpExpr {
    pub op: Chunk,
    pub exprs: Vec<Expr>,
}

#[derive(Debug)]
pub struct RepeatExpr {
    pub value: Expr,
    pub len: Expr,
}

#[derive(Debug)]
pub struct IndexExpr {
    pub obj: Expr,
    pub index: Expr,
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
    pub name: String,
    pub value: Expr,
}

#[derive(Debug)]
pub struct FieldExpr {
    pub expr: Expr,
    pub field: String,
}

#[derive(Debug)]
pub struct TypeExpr {
    pub expr: Expr,
    pub ty: Type,
}

#[derive(Debug)]
pub struct CastExpr {
    pub expr: Expr,
    pub ty: Type,
}

#[derive(Debug)]
pub struct RangeExpr {
    pub start: Option<Expr>,
    pub end: Option<Expr>,
    pub is_inclusive: bool,
}

#[derive(Debug)]
pub struct BlockExpr {
    pub label: Option<String>,
    pub block: Block,
}

#[derive(Debug)]
pub struct IfExpr {
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
pub struct LetExpr {
    pub pattens: Vec<Patten>,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct ForExpr {
    pub label: Option<String>,
    pub patten: Patten,
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
    pub label: Option<String>,
    pub expr: Option<Expr>,
}

#[derive(Debug)]
pub struct ContinueExpr {
    pub label: Option<String>,
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
    pub pattens: Vec<Patten>,
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
    pub path: PathSegment,
    pub args: Vec<Expr>,
}

#[derive(Debug)]
pub struct ClosureExpr {
    pub is_static: bool,
    pub is_async: bool,
    pub is_move: bool,
    pub sig: FnSig,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct ReturnExpr {
    pub ret: Option<Expr>,
}

#[derive(Debug)]
pub struct MacroDef {
    pub name: String,
    pub def: String,
}

#[derive(Debug)]
pub struct MacroStmt {
    pub loc: Loc,
    pub attrs: Vec<AttrKind>,
    pub mac: Macro,
    pub is_semi: bool,
}

#[derive(Debug)]
pub enum MacroStyle {
    Paren,
    Bracket,
    Brace,
}

#[derive(Debug)]
pub struct MacroSep {
    pub is_sep: bool,
    pub s: &'static str,
}

#[derive(Debug)]
pub struct Macro {
    pub name: String,
    pub style: MacroStyle,
    pub exprs: Vec<Expr>,
    pub seps: Vec<MacroSep>,
}
