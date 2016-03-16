use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Debug, Display};

use ir::*;
use rfmt;
use ts::*;

pub fn fmt(krate: Crate, leading_cmnts: HashMap<Pos, Vec<String>>,
           trailing_cmnts: HashMap<Pos, String>)
-> rfmt::Result {
    Formatter::new(leading_cmnts, trailing_cmnts).fmt_crate(krate)
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

#[inline]
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

#[inline]
fn foreign_head(abi: &str) -> String {
    let mut head = String::new();
    head.push_str("extern");
    if abi != r#""C""# {
        head.push_str(" ");
        head.push_str(abi);
    }
    head
}

#[inline]
fn extern_head(abi: &str) -> String {
    let mut head = String::new();
    if abi != r#""Rust""# {
        head.push_str("extern ");
        if abi != r#""C""# {
            head.push_str(abi);
            head.push_str(" ");
        }
    }
    head
}

#[inline]
fn fn_head(is_unsafe: bool, is_const: bool, abi: &str) -> String {
    let mut head = String::new();
    if is_unsafe {
        head.push_str("unsafe ");
    }
    if is_const {
        head.push_str("const ");
    }
    head.push_str(&extern_head(abi));
    head.push_str("fn");
    head
}

#[inline]
fn ident_patten_head(is_ref: bool, is_mut: bool) -> String {
    let mut head = String::new();
    if is_ref {
        head.push_str("ref ");
    }
    if is_mut {
        head.push_str("mut ");
    }
    head
}

macro_rules! display_lists {
    ($f:expr, $open:expr, $sep:expr, $close:expr, $($lists:expr),+) => ({
        try!(write!($f, $open));

        let mut first = true;
        $(for e in $lists {
            if !first {
                try!(write!($f, "{}", $sep));
            }
            try!(Display::fmt(e, $f));
            first = false;
        })+

        write!($f, $close)
    });

    ($f:expr, $sep:expr, $($lists:expr),+) => ({
       display_lists!($f, "", $sep, "", $($lists)+)
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
            try!(display_lists!(f, "(", ", ", ")", &**items));
        }
        Ok(())
    }
}

impl Display for ExternCrate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "extern crate {}", self.name)
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
                try!(display_lists!(f, "{{", ", ", "}}", &self.names));
            }
        }
        Ok(())
    }
}

impl Display for ModDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mod {}", self.name)
    }
}

impl Display for LifetimeDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}", self.lifetime));
        if !self.bounds.is_empty() {
            try!(write!(f, ": "));
            try!(display_lists!(f, " + ", &self.bounds))
        }
        Ok(())
    }
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
fn display_type_param_bounds(f: &mut fmt::Formatter, bounds: &Vec<TypeParamBound>)
-> fmt::Result {
    display_lists!(f, " + ", bounds)
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
fn display_for_liftime_defs(f: &mut fmt::Formatter, lifetime_defs: &Vec<LifetimeDef>)
-> fmt::Result {
    if !lifetime_defs.is_empty() {
        try!(display_lists!(f, "for<", ", ", "> ", lifetime_defs));
    }
    Ok(())
}

impl Display for Where {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_lists!(f, ", ", &self.clauses)
    }
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
        if !self.is_empty() {
            try!(display_lists!(f, "<", ", ", ">", &self.lifetimes, &self.types, &self.bindings));
        }
        Ok(())
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
            TypeKind::Macro(ref ty) => Debug::fmt(ty, f),
            TypeKind::Infer => write!(f, "_"),
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
            try!(write!(f, ": "));
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
        match self.ret {
            FnReturnKind::Unit => Ok(()),
            FnReturnKind::Diverge => write!(f, " -> !"),
            FnReturnKind::Normal(ref ty) => write!(f, " -> {}", ty),
        }
    }
}

impl Display for Sf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Sf::String(ref s) => Display::fmt(s, f),
            Sf::Type(ref ty) => write!(f, "self: {}", ty),
        }
    }
}

impl Display for Patten {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.pat {
            PattenKind::Wildcard => write!(f, "_"),
            PattenKind::Literal(ref pat) => Display::fmt(pat, f),
            PattenKind::Range(ref pat) => Display::fmt(pat, f),
            PattenKind::Ident(ref pat) => Display::fmt(pat, f),
            PattenKind::Ref(ref pat) => Display::fmt(pat, f),
            PattenKind::Path(ref pat) => Display::fmt(pat, f),
            PattenKind::Enum(ref pat) => Display::fmt(pat, f),
            PattenKind::Struct(ref pat) => Debug::fmt(pat, f),
            PattenKind::Vec(ref pat) => Debug::fmt(pat, f),
            PattenKind::Tuple(ref pat) => Display::fmt(pat, f),
            PattenKind::Box(ref pat) => write!(f, "box {}", pat),
            PattenKind::Macro(ref pat) => Display::fmt(pat, f),
        }
    }
}

impl Display for RangePatten {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}...{}", self.start, self.end)
    }
}

impl Display for IdentPatten {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}{}", ident_patten_head(self.is_ref, self.is_mut), self.name));
        if let Some(ref pat) = self.binding {
            try!(write!(f, " @ {}", pat));
        }
        Ok(())
    }
}

impl Display for RefPatten {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", ref_head(&None, self.is_mut), self.pat)
    }
}

impl Display for PathPatten {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_qself(f, &self.qself, &self.path)
    }
}

impl Display for EnumPatten {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(Display::fmt(&self.path, f));
        match self.pats {
            Some(ref pats) if !pats.is_empty() => display_lists!(f, "(", ", ", ")", pats),
            None => write!(f, "(..)"),
            _ => Ok(()),
        }
    }
}

impl Display for VecPatten {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let emit = if let Some(_) = self.emit {
            vec![Chunk::new("..")]
        } else {
            Vec::new()
        };
        display_lists!(f, "[", ", ", "]", &self.start, &emit, &self.end)
    }
}

impl Display for TuplePatten {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_lists!(f, "(", ", ", ")", &self.pats)
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.expr {
            ExprKind::Literal(ref expr) => Display::fmt(expr, f),
            ExprKind::Path(ref expr) => Display::fmt(expr, f),
            ExprKind::Unary(ref expr) => Display::fmt(expr, f),
            ExprKind::Ref(ref expr) => Display::fmt(expr, f),
            ExprKind::List(ref expr) => Display::fmt(expr, f),
            ExprKind::FixedSizeArray(ref expr) => Display::fmt(expr, f),
            ExprKind::Vec(ref exprs) => display_lists!(f, "[", ", ", "]", &**exprs),
            ExprKind::Tuple(ref exprs) => display_lists!(f, "(", ", ", ")", &**exprs),
            ExprKind::FieldAccess(ref expr) => Display::fmt(expr, f),
            ExprKind::Range(ref expr) => Display::fmt(expr, f),
            ExprKind::Index(ref expr) => Display::fmt(expr, f),
            ExprKind::Box(ref expr) => Display::fmt(expr, f),
            ExprKind::Cast(ref expr) => Display::fmt(expr, f),
            ExprKind::Type(ref expr) => Display::fmt(expr, f),
            ExprKind::FnCall(ref expr) => Display::fmt(expr, f),
            ExprKind::MethodCall(ref expr) => Display::fmt(expr, f),
            ExprKind::Closure(ref expr) => Display::fmt(expr, f),
            ExprKind::Macro(ref expr) => Display::fmt(expr, f),
            _ => Debug::fmt(self, f),
        }
    }
}

impl Display for UnaryExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.op, self.expr)
    }
}

impl Display for RefExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", ref_head(&None, self.is_mut), self.expr)
    }
}

impl Display for ListExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sep = format!(" {} ", self.sep);
        display_lists!(f, sep, &self.exprs)
    }
}

impl Display for FixedSizeArrayExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}; {}]", &self.init, &self.len)
    }
}

impl Display for FieldAccessExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.expr, self.field.s)
    }
}

impl Display for IndexExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}[{}]", self.obj, self.index)
    }
}

impl Display for RangeExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref start) = self.start {
            try!(write!(f, "{}", start));
        }
        try!(write!(f, ".."));
        if let Some(ref end) = self.end {
            try!(write!(f, "{}", end));
        }
        Ok(())
    }
}

impl Display for BoxExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "box {}", self.expr)
    }
}

impl Display for CastExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} as {}", self.expr, self.ty)
    }
}

impl Display for TypeExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.expr, self.ty)
    }
}

impl Display for FnCallExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}", self.name));
        display_lists!(f, "(", ", ", ")", &self.args)
    }
}

impl Display for MethodCallExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}.{}", self.obj, self.name));
        if !self.types.is_empty() {
            try!(write!(f, "::"));
            try!(display_lists!(f, "<", ", ", ">", &self.types));
        }
        display_lists!(f, "(", ", ", ")", &self.args)
    }
}

impl Display for ClosureExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.moved {
            try!(write!(f, "move "));
        }

        try!(write!(f, "|"));
        let mut first = true;
        for arg in &self.fn_sig.arg.args {
            if !first {
                try!(write!(f, ", "));
            }

            try!(write!(f, "{}", arg.pat));
            match arg.ty.ty {
                TypeKind::Infer => (),
                _ => try!(write!(f, ": {}", arg.ty)),
            }
            first = false;
        }

        if self.fn_sig.arg.va {
            try!(write!(f, ", ..."));
        }
        try!(write!(f, "|"));
        try!(Display::fmt(&self.fn_sig.ret, f));

        if self.block.stmts.len() > 1 {
            Debug::fmt(&self.block, f)
        } else {
            match self.block.stmts[0].stmt {
                StmtKind::Expr(ref expr, is_semi) if !is_semi => {
                    write!(f, " {}", expr)
                },
                _ => unreachable!(),
            }
        }
    }
}

impl Display for Macro {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (open, close) = match self.style {
            MacroStyle::Paren => ("(", ")"),
            MacroStyle::Bracket => ("[", "]"),
            MacroStyle::Brace => ("{", "}"),
        };

        try!(write!(f, "{}!", self.name));
        try!(write!(f, "{}", open));
        let expr_len = self.exprs.len();
        for i in 0..expr_len {
            let expr = &self.exprs[i];
            if i > 0 {
                try!(write!(f, "{}", self.seps[i - 1]));
            }
            try!(Display::fmt(expr, f));
        }
        write!(f, "{}", close)
    }
}

macro_rules! fmt_item_groups {
    ($sf:expr, $items:expr, $item_kind:path, $item_type:ty, $fmt_item:ident) => ({
        let mut group: Vec<(&Loc, bool, &Vec<AttrKind>, $item_type)> = Vec::new();

        for item in $items {
            match item.item {
                $item_kind(ref e) => {
                    if $sf.has_leading_comments(&item.loc) {
                        fmt_item_group!($sf, &group, $item_type, $fmt_item);
                        group.clear();

                        $sf.fmt_leading_comments(&item.loc);
                    }
                    group.push((&item.loc, item.is_pub, &item.attrs, e));
                }
                _ => {
                    fmt_item_group!($sf, &group, $item_type, $fmt_item);
                    group.clear();
                }
            }
        }

        fmt_item_group!($sf, &group, $item_type, $fmt_item);
    });
}

macro_rules! fmt_item_group {
    ($sf:expr, $group:expr, $ty:ty, $fmt_item:ident) => ({
        let map: BTreeMap<String, (&Loc, bool, &Vec<AttrKind>, $ty)>
                = $group.into_iter().map(|e| (e.3.to_string(), *e)).collect();

        for (_, e) in map {
            $sf.fmt_attrs(e.2);

            $sf.insert_indent();
            if e.1 {
                $sf.raw_insert("pub ");
            }
            $sf.$fmt_item(e.3);

            $sf.try_fmt_trailing_comment(e.0);
            $sf.nl();
        }
    });
}

macro_rules! maybe_nl {
    ($sf:expr, $e:ident) => ({
        if $e.loc.nl {
            $sf.wrap();
        }
    });
}

macro_rules! maybe_wrap {
    ($sf:expr, $sep:expr, $wrap_sep:expr, $e:expr) => ({
        if !need_wrap!($sf.ts, $sep, &$e.to_string()) {
            $sf.raw_insert($sep);
        } else {
            $sf.wrap();
            $sf.raw_insert($wrap_sep);
        }
    });

    ($sf:expr, $e:expr) => ({
        maybe_wrap!($sf, "", "", $e);
    });

    ($sf:expr, $sep:expr, $wrap_sep:expr, $e:expr, $fmt:ident) => ({
        maybe_wrap!($sf, $sep, $wrap_sep, $e);
        $sf.$fmt(&$e);
    });
}

macro_rules! maybe_nl_indent {
    ($sf:expr, $sep:expr, $wrap_sep:expr, $e:expr) => ({
        if !need_nl_indent!($sf.ts, $sep, &$e.to_string()) {
            $sf.raw_insert($sep);
        } else {
            $sf.nl_indent();
            $sf.raw_insert($wrap_sep);
        }
    });

    ($sf:expr, $sep:expr, $wrap_sep:expr, $e:expr, $fmt:ident) => ({
        maybe_nl_indent!($sf, $sep, $wrap_sep, $e);
        $sf.$fmt($e);
    });
}

macro_rules! insert_sep {
    ($sf:expr, $sep:expr, $e:expr) => ({
        $sf.raw_insert($sep);
        if !$e.loc.nl && !need_wrap!($sf.ts, " ", &$e.to_string()) {
            $sf.raw_insert(" ");
        } else {
            $sf.wrap();
        }
    });
}

macro_rules! fmt_comma_lists {
    ($sf:expr, $open:expr, $close:expr, $($list:expr, $fmt:ident),+) => ({
        $sf.insert_mark_align($open);

        let mut first = true;
        $(for e in $list {
            if !first {
                insert_sep!($sf, ",", e);
            }

            $sf.$fmt(e);
            first = false;
        })+

        $sf.insert_unmark_align($close);
    });

    ($sf:expr, $($list:expr, $fmt:ident),+) => ({
        fmt_comma_lists!($sf, "", "", $($list, $fmt)+);
    });
}

macro_rules! fmt_lists {
    ($sf:expr, $sep:expr, $wrap_sep:expr, $($list:expr, $act:ident),+) => ({
        let mut first = true;
        $(for e in $list {
            if !first {
                maybe_wrap!($sf, $sep, $wrap_sep, e, $act);
            } else {
                $sf.$act(e);
            }

            first = false;
        })+
    });
}

macro_rules! fmt_block {
    ($sf:expr, $items: expr, $block:expr, $fmt:ident) => ({
        if $items.is_empty() {
            if $sf.block_non_sep {
                $sf.raw_insert("{}");
                $sf.block_non_sep = false;
            } else {
                $sf.raw_insert(" {}");
            }
            return;
        }

        if $sf.block_non_sep {
            $sf.raw_insert("{");
            $sf.block_non_sep = false;
        } else {
            $sf.raw_insert(" {");
        }
        $sf.indent();
        $sf.nl();

        $sf.$fmt($block);

        $sf.outdent();
        $sf.insert_indent();
        $sf.raw_insert("}");
    });

    ($sf:expr, $items:expr, $fmt:ident) => ({
        fmt_block!($sf, $items, $items, $fmt);
    })
}

macro_rules! fmt_items {
    ($sf:ident, $items:expr, $fmt_item:ident) => ({
        for item in $items {
            $sf.try_fmt_leading_comments(&item.loc);
            $sf.fmt_attrs(&item.attrs);
            $sf.insert_indent();

            $sf.$fmt_item(item);

            $sf.try_fmt_trailing_comment(&item.loc);
            $sf.nl();
        }
    });
}

struct Formatter {
    ts: Typesetter,

    leading_cmnts: HashMap<Pos, Vec<String>>,
    trailing_cmnts: HashMap<Pos, String>,

    after_indent: bool,
    after_wrap: bool,
    block_non_sep: bool,
}

impl Formatter {
    fn new(leading_cmnts: HashMap<Pos, Vec<String>>, trailing_cmnts: HashMap<Pos, String>)
    -> Formatter {
        Formatter {
            ts: Typesetter::new(),

            leading_cmnts: leading_cmnts,
            trailing_cmnts: trailing_cmnts,

            after_indent: false,
            after_wrap: false,
            block_non_sep: false,
        }
    }

    fn fmt_crate(mut self, krate: Crate) -> rfmt::Result {
        self.try_fmt_leading_comments(&krate.loc);
        self.fmt_attrs(&krate.attrs);
        self.fmt_mod(&krate.module);
        self.fmt_left_comments(&krate.module.loc);
        self.ts.result()
    }

    #[inline]
    fn clear_flag(&mut self) {
        self.after_indent = false;
        self.after_wrap = false;
    }

    #[inline]
    fn raw_insert(&mut self, s: &str) {
        if !s.is_empty() {
            self.ts.raw_insert(s);
            self.clear_flag();
        }
    }

    #[inline]
    fn insert(&mut self, s: &str) {
        if !s.is_empty() {
            self.ts.insert(s);
            self.clear_flag();
        }
    }

    #[inline]
    fn wrap(&mut self) {
        if !self.after_indent && !self.after_wrap {
            self.ts.wrap();
            self.after_wrap = true;
        }
    }

    #[inline]
    fn nl(&mut self) {
        self.ts.nl();
        self.clear_flag();
    }

    #[inline]
    fn nl_indent(&mut self) {
        if !self.after_indent {
            self.ts.nl_indent();
            self.after_indent = true;
        }
    }

    #[inline]
    fn indent(&mut self) {
        self.ts.indent();
    }

    #[inline]
    fn outdent(&mut self) {
        self.ts.outdent();
    }

    #[inline]
    fn insert_indent(&mut self) {
        self.ts.insert_indent();
        self.after_indent = true;
    }

    #[inline]
    fn insert_mark_align(&mut self, s: &str) {
        self.ts.insert_mark_align(s);
        self.clear_flag();
    }

    #[inline]
    fn insert_unmark_align(&mut self, s: &str) {
        self.ts.insert_unmark_align(s);
        self.clear_flag();
    }

    #[inline]
    fn has_leading_comments(&self, loc: &Loc) -> bool {
        self.leading_cmnts.contains_key(&loc.start)
    }

    #[inline]
    fn try_fmt_leading_comments(&mut self, loc: &Loc) {
        if self.has_leading_comments(loc) {
            self.fmt_leading_comments(loc);
        }
    }

    #[inline]
    fn fmt_leading_comments(&mut self, loc: &Loc) {
        for cmnt in &self.leading_cmnts.remove(&loc.start).unwrap() {
            if !cmnt.is_empty() {
                self.insert_indent();
                self.raw_insert(cmnt);
            }
            self.nl();
        }
    }

    #[inline]
    fn fmt_left_comments(&mut self, loc: &Loc) {
        let poses: Vec<_> = self.leading_cmnts.keys().cloned().collect();
        for pos in poses {
            for cmnt in &self.leading_cmnts.remove(&pos).unwrap() {
                if pos > loc.end {
                    self.raw_insert(cmnt);
                    self.nl();
                }
            }
        }
    }

    #[inline]
    fn has_trailing_comment(&self, loc: &Loc) -> bool {
        self.trailing_cmnts.contains_key(&loc.end)
    }

    #[inline]
    fn try_fmt_trailing_comment(&mut self, loc: &Loc) {
        if self.has_trailing_comment(loc) {
            self.fmt_trailing_comment(loc);
        }
    }

    #[inline]
    fn fmt_trailing_comment(&mut self, loc: &Loc) {
        self.raw_insert(" ");
        let cmnt = self.trailing_cmnts.remove(&loc.end).unwrap();
        self.raw_insert(&cmnt);
    }

    #[inline]
    fn fmt_chunk(&mut self, chunk: &Chunk) {
        maybe_nl!(self, chunk);
        self.insert(&chunk.s);
    }

    #[inline]
    fn fmt_attrs(&mut self, attrs: &Vec<AttrKind>) {
        let mut attr_group: Vec<&Attr> = Vec::new();

        for attr in attrs {
            match *attr {
                AttrKind::Doc(ref doc) => {
                    self.fmt_attr_group(&attr_group);
                    attr_group.clear();

                    self.fmt_doc(doc);
                },
                AttrKind::Attr(ref attr) => {
                    if self.has_leading_comments(&attr.loc) {
                        self.fmt_attr_group(&attr_group);
                        attr_group.clear();

                        self.fmt_leading_comments(&attr.loc);
                    }
                    attr_group.push(attr);
                },
            }
        }

        self.fmt_attr_group(&attr_group);
    }

    #[inline]
    fn fmt_doc(&mut self, doc: &Doc) {
        self.try_fmt_leading_comments(&doc.loc);
        self.insert_indent();
        self.raw_insert(&doc.s);
        self.try_fmt_trailing_comment(&doc.loc);
        self.nl();
    }

    #[inline]
    fn fmt_attr_group(&mut self, attr_group: &Vec<&Attr>) {
        let sorted_attrs: BTreeMap<_, _>
                = attr_group.into_iter().map(|e| (e.to_string(), *e)).collect();
        for attr in sorted_attrs.values() {
            self.insert_indent();
            self.fmt_attr(attr);
            self.try_fmt_trailing_comment(&attr.loc);
            self.nl();
        }
    }

    #[inline]
    fn fmt_attr(&mut self, attr: &Attr) {
        self.raw_insert("#");
        if attr.is_inner {
            self.raw_insert("!");
        }
        self.raw_insert("[");
        self.fmt_meta_item(&attr.item);
        self.raw_insert("]");
    }

    #[inline]
    fn fmt_meta_items(&mut self, items: &Vec<MetaItem>) {
        fmt_comma_lists!(self, "(", ")", items, fmt_meta_item);
    }

    #[inline]
    fn fmt_meta_item(&mut self, item: &MetaItem) {
        maybe_nl!(self, item);
        self.insert(&item.name);

        if let Some(ref items) = item.items {
            self.fmt_meta_items(&**items);
        }
    }

    fn fmt_mod(&mut self, module: &Mod) {
        self.fmt_group_items(&module.items);
        self.fmt_items(&module.items);
    }

    fn fmt_group_items(&mut self, items: &Vec<Item>) {
        self.fmt_extern_crate_items(items);
        self.fmt_use_items(items);
        self.fmt_mod_decl_items(items);
    }

    fn fmt_extern_crate_items(&mut self, items: &Vec<Item>) {
        fmt_item_groups!(self, items, ItemKind::ExternCrate, &ExternCrate, fmt_extern_crate);
    }

    fn fmt_extern_crate(&mut self, item: &ExternCrate) {
        self.insert(&format!("extern crate {};", &item.name));
    }

    fn fmt_use_items(&mut self, items: &Vec<Item>) {
        fmt_item_groups!(self, items, ItemKind::Use, &Use, fmt_use);
    }

    fn fmt_use(&mut self, item: &Use) {
        self.insert(&format!("use {}", &item.base));
        self.fmt_use_names(&item.names);
        self.raw_insert(";");
    }

    fn fmt_use_names(&mut self, names: &Vec<Chunk>) {
        if names.is_empty() {
            return;
        }

        self.insert("::");
        if names.len() == 1 {
            self.insert(&names[0].s);
        } else {
            fmt_comma_lists!(self, "{", "}", names, fmt_chunk);
        }
    }

    fn fmt_mod_decl_items(&mut self, items: &Vec<Item>) {
        fmt_item_groups!(self, items, ItemKind::ModDecl, &ModDecl, fmt_mod_decl);
    }

    fn fmt_mod_decl(&mut self, item: &ModDecl) {
        self.insert(&format!("mod {};", &item.name));
    }

    fn fmt_items(&mut self, items: &Vec<Item>) {
        for item in items {
            match item.item {
                ItemKind::ExternCrate(_) | ItemKind::Use(_) | ItemKind::ModDecl(_) => (),
                _ => self.fmt_item(item),
            }
        }
    }

    fn fmt_item(&mut self, item: &Item) {
        self.try_fmt_leading_comments(&item.loc);
        self.fmt_attrs(&item.attrs);
        self.insert_indent();

        if item.is_pub {
            self.raw_insert("pub ");
        }
        match item.item {
            ItemKind::ExternCrate(ref item) => self.fmt_extern_crate(item),
            ItemKind::Use(ref item) => self.fmt_use(item),
            ItemKind::ModDecl(ref item) => self.fmt_mod_decl(item),
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
            ItemKind::Macro(ref item) => self.fmt_macro_item(item),
        }

        self.try_fmt_trailing_comment(&item.loc);
        self.nl();
    }

    fn fmt_sub_mod(&mut self, item: &Mod) {
        self.insert(&format!("mod {}", &item.name));
        fmt_block!(self, item.items, item, fmt_mod);
    }

    fn fmt_type_alias(&mut self, item: &TypeAlias) {
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
        maybe_nl!(self, lifetime_def);
        maybe_wrap!(self, lifetime_def);

        self.fmt_lifetime(&lifetime_def.lifetime);
        if !lifetime_def.bounds.is_empty() {
            self.raw_insert(": ");
            fmt_lists!(self, " + ", "+ ", &lifetime_def.bounds, fmt_lifetime)
        }
    }

    fn fmt_lifetime(&mut self, lifetime: &Lifetime) {
        self.fmt_chunk(lifetime);
    }

    fn fmt_type_param(&mut self, type_param: &TypeParam) {
        maybe_nl!(self, type_param);
        maybe_wrap!(self, type_param);

        self.insert(&type_param.name);
        if let Some(ref ty) = type_param.default {
            maybe_wrap!(self, " = ", "= ", ty, fmt_type);
        } else if !type_param.bounds.is_empty() {
            self.raw_insert(": ");
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
            },
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
            maybe_nl_indent!(self, " where ", "where ", wh);
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
        maybe_wrap!(self, bound);
        self.fmt_for_lifetime_defs(&bound.lifetime_defs);
        self.fmt_type(&bound.ty);
        self.raw_insert(": ");
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
        maybe_wrap!(self, binding);

        self.insert(&format!("{} = ", binding.name));
        self.fmt_type(&binding.ty);
    }

    fn fmt_paren_param(&mut self, param: &ParenParam) {
        fmt_comma_lists!(self, "(", ")", &param.inputs, fmt_type);
        if let Some(ref output) = param.output {
            maybe_wrap!(self, " -> ", "-> ", output, fmt_type);
        }
    }

    fn fmt_qself_path(&mut self, qself: &QSelf, path: &Path) {
        self.insert_mark_align("<");
        self.fmt_type(&qself.ty);
        if qself.pos > 0 {
            self.raw_insert(" as ");
            if path.global {
                self.insert("::");
            }
            self.fmt_path_segments(&path.segs[0..qself.pos]);
        }
        self.insert_unmark_align(">");

        self.insert("::");
        self.fmt_path_segments(&path.segs[qself.pos..]);
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
            },
            None => self.fmt_path(&ty.path),
        }
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
        self.insert_mark_align("[");
        self.fmt_type(&ty.ty);
        self.insert_unmark_align("]");
    }

    fn fmt_fixed_size_array_type(&mut self, ty: &FixedSizeArrayType) {
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
            self.raw_insert(": ");
            self.fmt_type_param_bounds(&ty.bounds);
        }
    }

    fn fmt_poly_trait_ref_type(&mut self, ty: &PolyTraitRefType) {
        self.fmt_type_param_bounds(&ty.bounds);
    }

    fn fmt_infer_type(&mut self) {
        self.raw_insert("_");
    }

    fn fmt_foreign_mod(&mut self, item: &ForeignMod) {
        self.insert(&foreign_head(&item.abi));
        fmt_block!(self, &item.items, fmt_foreign_items);
    }

    fn fmt_foreign_items(&mut self, items: &Vec<ForeignItem>) {
        fmt_items!(self, items, fmt_foreign_item);
    }

    fn fmt_foreign_item(&mut self, item: &ForeignItem) {
        if item.is_pub {
            self.raw_insert("pub ");
        }

        match item.item {
            ForeignKind::Static(ref item) => self.fmt_foreign_static(item),
            ForeignKind::Fn(ref item) => self.fmt_foreign_fn(item),
        }

        self.raw_insert(";");
    }

    fn fmt_foreign_static(&mut self, item: &ForeignStatic) {
        self.insert(&format!("{}{}", static_head(item.is_mut), item.name));
        insert_sep!(self, ":", item.ty);
        self.fmt_type(&item.ty);
    }

    fn fmt_foreign_fn(&mut self, item: &ForeignFn) {
        self.insert(&format!("fn {}", item.name));
        self.fmt_generics(&item.generics);
        self.fmt_fn_sig(&item.fn_sig);
    }

    fn fmt_const(&mut self, item: &Const) {
        self.insert(&format!("const {}", item.name));
        insert_sep!(self, ":", item.ty);
        self.fmt_type(&item.ty);
        maybe_wrap!(self, " = ", "= ", item.expr, fmt_expr);
        self.raw_insert(";");
    }

    fn fmt_static(&mut self, item: &Static) {
        self.insert(&format!("{}{}", static_head(item.is_mut), item.name));
        insert_sep!(self, ":", item.ty);
        self.fmt_type(&item.ty);
        maybe_wrap!(self, " = ", "= ", item.expr, fmt_expr);
        self.raw_insert(";");
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
            StructBody::Struct(ref fields) => fmt_block!(self, fields, fmt_struct_fields),
            StructBody::Tuple(ref fields) => self.fmt_tuple_fields(fields),
            StructBody::Unit => (),
        }
    }

    fn fmt_struct_fields(&mut self, fields: &Vec<StructField>) {
        fmt_items!(self, fields, fmt_struct_field);
    }

    fn fmt_struct_field(&mut self, field: &StructField) {
        if field.is_pub {
            self.raw_insert("pub ");
        }

        self.insert(&field.name);
        insert_sep!(self, ":", field.ty);
        self.fmt_type(&field.ty);

        self.raw_insert(",");
    }

    fn fmt_tuple_fields(&mut self, fields: &Vec<TupleField>) {
        fmt_comma_lists!(self, "(", ")", fields, fmt_tuple_field);
    }

    fn fmt_tuple_field(&mut self, field: &TupleField) {
        maybe_nl!(self, field);
        self.try_fmt_leading_comments(&field.loc);
        self.fmt_attrs(&field.attrs);

        if field.is_pub {
            self.raw_insert("pub ");
        }
        self.fmt_type(&field.ty);
    }

    fn fmt_enum(&mut self, item: &Enum) {
        self.insert(&format!("enum {}", item.name));
        self.fmt_generics(&item.generics);
        fmt_block!(self, &item.body.fields, fmt_enum_fields);
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
            self.raw_insert("unsafe ");
        }
        self.insert(&format!("trait {}", item.name));
        self.fmt_generics(&item.generics);
        if !item.bounds.is_empty() {
            self.raw_insert(": ");
            self.fmt_type_param_bounds(&item.bounds);
        }
        self.fmt_where(&item.generics.wh);
        fmt_block!(self, &item.items, fmt_trait_items);
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
    }

    fn fmt_const_trait_item(&mut self, item: &ConstTraitItem) {
        self.insert(&format!("const {}", item.name));
        insert_sep!(self, ":", item.ty);
        self.fmt_type(&item.ty);
    }

    fn fmt_type_trait_item(&mut self, item: &TypeTraitItem) {
        self.insert(&format!("type {}", item.name));
        if !item.bounds.is_empty() {
            self.raw_insert(": ");
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
            self.raw_insert("unsafe ");
        }
        self.raw_insert("impl ");
        self.fmt_trait_ref(&item.trait_ref);
        self.raw_insert(" for .. {}");
    }

    fn fmt_impl(&mut self, item: &Impl) {
        if item.is_unsafe {
            self.raw_insert("unsafe ");
        }

        self.raw_insert("impl");
        self.fmt_generics(&item.generics);
        self.raw_insert(" ");

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
        fmt_block!(self, &item.items, fmt_impl_items);
    }

    fn fmt_impl_items(&mut self, items: &Vec<ImplItem>) {
        fmt_items!(self, items, fmt_impl_item);
    }

    fn fmt_impl_item(&mut self, item: &ImplItem) {
        if item.is_pub {
            self.raw_insert("pub ");
        }

        match item.item {
            ImplItemKind::Const(ref item) => {
                self.fmt_const_impl_item(item);
                self.raw_insert(";");
            },
            ImplItemKind::Type(ref item) => {
                self.fmt_type_impl_item(item);
                self.raw_insert(";");
            },
            ImplItemKind::Method(ref item) => self.fmt_method_impl_item(item),
            ImplItemKind::Macro(ref item) => {
                self.fmt_macro(item);
                self.raw_insert(";");
            },
        }
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
        maybe_nl!(self, arg);
        maybe_wrap!(self, arg);

        if !arg.pat.to_string().is_empty() {
            self.fmt_patten(&arg.pat);
            self.raw_insert(": ");
        }
        self.fmt_type(&arg.ty);
    }

    fn fmt_fn_return(&mut self, ret: &FnReturn) {
        match ret.ret {
            FnReturnKind::Unit => (),
            FnReturnKind::Diverge => {
                if ret.nl {
                    self.nl_indent();
                    self.raw_insert("-> !");
                } else {
                    maybe_nl_indent!(self, " -> !", "-> !", "");
                }
            },
            FnReturnKind::Normal(ref ty) => {
                if ret.nl {
                    self.nl_indent();
                    self.raw_insert("-> ");
                } else {
                    maybe_nl_indent!(self, " -> ", "-> ", ty);
                }
                self.fmt_type(ty);
            },
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
        self.fmt_sf(sf);
        for arg in &sig.fn_sig.arg.args[1..] {
            insert_sep!(self, ",", arg);
            self.fmt_arg(arg);
        }
        self.insert_unmark_align(")");

        self.fmt_fn_return(&sig.fn_sig.ret);
    }

    fn fmt_sf(&mut self, sf: &Sf) {
        maybe_wrap!(self, sf);
        match *sf {
            Sf::String(ref s) => self.raw_insert(s),
            Sf::Type(ref ty) => {
                self.raw_insert("self: ");
                self.fmt_type(ty);
            },
        }
    }

    fn fmt_block(&mut self, block: &Block) {
        if block.is_unsafe {
            self.raw_insert("unsafe");
        }
        fmt_block!(self, &block.stmts, fmt_stmts);
    }

    fn fmt_stmts(&mut self, stmts: &Vec<Stmt>) {
        for stmt in stmts {
            self.fmt_stmt(stmt);
        }
    }

    #[inline]
    fn fmt_stmt(&mut self, stmt: &Stmt) {
        self.try_fmt_leading_comments(&stmt.loc);
        match stmt.stmt {
            StmtKind::Decl(ref decl) => self.fmt_decl_stmt(decl),
            StmtKind::Expr(ref expr, is_semi) => self.fmt_expr_stmt(expr, is_semi),
            StmtKind::Macro(ref mac, is_semi) => self.fmt_macro_stmt(mac, is_semi),
        }
    }

    fn fmt_decl_stmt(&mut self, decl: &Decl) {
        match decl.decl {
            DeclKind::Local(ref local) => self.fmt_local(local),
            DeclKind::Item(ref item) => self.fmt_item(item),
        }
    }

    fn fmt_local(&mut self, local: &Local) {
        self.try_fmt_leading_comments(&local.loc);
        self.fmt_attrs(&local.attrs);
        self.insert_indent();

        self.raw_insert("let ");
        self.fmt_patten(&local.pat);
        if let Some(ref ty) = local.ty {
            maybe_wrap!(self, ": ", ":", ty, fmt_type);
        }
        if let Some(ref expr) = local.init {
            maybe_wrap!(self, " = ", "= ", expr, fmt_expr);
        }

        self.raw_insert(";");
        self.try_fmt_trailing_comment(&local.loc);
        self.nl();
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

    #[inline]
    fn fmt_range_patten(&mut self, pat: &RangePatten) {
        self.fmt_expr(&pat.start);
        self.insert("...");
        self.fmt_expr(&pat.end);
    }

    #[inline]
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

    #[inline]
    fn fmt_ref_patten(&mut self, pat: &RefPatten) {
        self.insert(&ref_head(&None, pat.is_mut));
        self.fmt_patten(&pat.pat);
    }

    #[inline]
    fn fmt_path_patten(&mut self, pat: &PathPatten) {
        self.fmt_qself_path(&pat.qself, &pat.path);
    }

    #[inline]
    fn fmt_enum_patten(&mut self, pat: &EnumPatten) {
        self.fmt_path(&pat.path);
        match pat.pats {
            Some(ref pats) if !pats.is_empty() => fmt_comma_lists!(self, "(", ")", pats,
                    fmt_patten),
            None => self.insert("(..)"),
            _ => (),
        }
    }

    #[inline]
    fn fmt_struct_patten(&mut self, pat: &StructPatten) {
        self.fmt_path(&pat.path);

        if pat.fields.is_empty() {
            self.raw_insert("{}");
            return;
        }

        self.raw_insert(" {");
        self.indent();
        self.nl();

        self.fmt_struct_field_pattens(&pat.fields);
        if pat.etc {
            self.insert_indent();
            self.raw_insert("..");
            self.nl();
        }

        self.outdent();
        self.insert_indent();
        self.raw_insert("}");
    }

    #[inline]
    fn fmt_struct_field_pattens(&mut self, fields: &Vec<StructFieldPatten>) {
        for field in fields {
            self.try_fmt_leading_comments(&field.loc);
            self.insert_indent();
            self.fmt_struct_field_patten(field);
            self.try_fmt_trailing_comment(&field.loc);
            self.nl();
        }
    }

    #[inline]
    fn fmt_struct_field_patten(&mut self, field: &StructFieldPatten) {
        if field.shorthand {
            self.fmt_patten(&field.pat);
        } else {
            self.insert(&field.name);
            maybe_wrap!(self, ": ", ":", field.pat, fmt_patten);
        }
        self.raw_insert(",");
    }

    #[inline]
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

    #[inline]
    fn fmt_vec_emit_patten(&mut self, emit: &Chunk) {
        self.insert(&emit.s);
    }

    #[inline]
    fn fmt_tuple_patten(&mut self, pat: &TuplePatten) {
        fmt_comma_lists!(self, "(", ")", &pat.pats, fmt_patten);
    }

    #[inline]
    fn fmt_box_patten(&mut self, pat: &Patten) {
        self.raw_insert("box ");
        self.fmt_patten(pat);
    }

    #[inline]
    fn fmt_expr_stmt(&mut self, expr: &Expr, is_semi: bool) {
        self.try_fmt_leading_comments(&expr.loc);
        self.fmt_attrs(&expr.attrs);
        self.insert_indent();

        self.fmt_expr(expr);
        if is_semi {
            self.raw_insert(";");
        }

        self.try_fmt_trailing_comment(&expr.loc);
        self.nl();
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
            ExprKind::Block(ref expr) => self.fmt_block_expr(expr),
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

    #[inline]
    fn fmt_literal_expr(&mut self, expr: &Chunk) {
        self.fmt_chunk(expr);
    }

    #[inline]
    fn fmt_path_expr(&mut self, expr: &PathExpr) {
        self.fmt_path_type(expr);
    }

    #[inline]
    fn fmt_unary_expr(&mut self, expr: &UnaryExpr) {
        maybe_wrap!(self, &expr.op, &expr.op, expr.expr, fmt_expr);
    }

    #[inline]
    fn fmt_ref_expr(&mut self, expr: &RefExpr) {
        let head = &ref_head(&None, expr.is_mut);
        maybe_wrap!(self, head, head, expr.expr, fmt_expr);
    }

    #[inline]
    fn fmt_list_expr(&mut self, expr: &ListExpr) {
        let sep = format!(" {} ", expr.sep);
        let wrap_sep = format!("{} ", expr.sep);
        fmt_lists!(self, &sep, &wrap_sep, &expr.exprs, fmt_expr);
    }

    #[inline]
    fn fmt_fixed_size_array_expr(&mut self, expr: &FixedSizeArrayExpr) {
        self.insert_mark_align("[");
        self.fmt_expr(&expr.init);
        insert_sep!(self, ";", expr.len);
        self.fmt_expr(&expr.len);
        self.insert_unmark_align("]");
    }

    #[inline]
    fn fmt_vec_expr(&mut self, exprs: &Vec<Expr>) {
        fmt_comma_lists!(self, "[", "]", exprs, fmt_expr);
    }

    #[inline]
    fn fmt_tuple_expr(&mut self, exprs: &Vec<Expr>) {
        fmt_comma_lists!(self, "(", ")", exprs, fmt_expr);
    }

    #[inline]
    fn fmt_field_access_expr(&mut self, expr: &FieldAccessExpr) {
        maybe_wrap!(self, expr);
        self.fmt_expr(&expr.expr);
        self.insert(&format!(".{}", &expr.field.s));
    }

    #[inline]
    fn fmt_struct_expr(&mut self, expr: &StructExpr) {
        self.fmt_path(&expr.path);

        if expr.fields.is_empty() {
            self.insert(" {}");
            return;
        }

        self.raw_insert(" {");
        self.indent();
        self.nl();

        self.fmt_struct_field_exprs(&expr.fields);
        if let Some(ref base) = expr.base {
            self.insert_indent();
            self.insert("..");
            self.fmt_expr(base);
            self.try_fmt_trailing_comment(&base.loc);
            self.nl();
        }

        self.outdent();
        self.insert_indent();
        self.raw_insert("}");
    }

    #[inline]
    fn fmt_struct_field_exprs(&mut self, fields: &Vec<StructFieldExpr>) {
        for field in fields {
            self.try_fmt_leading_comments(&field.loc);
            self.insert_indent();
            self.fmt_struct_field_expr(field);
            self.try_fmt_trailing_comment(&field.loc);
            self.nl();
        }
    }

    #[inline]
    fn fmt_struct_field_expr(&mut self, field: &StructFieldExpr) {
        self.insert(&field.name.s);
        insert_sep!(self, ":", field.value);
        self.fmt_expr(&field.value);
        self.raw_insert(",");
    }

    #[inline]
    fn fmt_index_expr(&mut self, expr: &IndexExpr) {
        self.fmt_expr(&expr.obj);
        self.insert_mark_align("[");
        self.fmt_expr(&expr.index);
        self.insert_unmark_align("]");
    }

    #[inline]
    fn fmt_range_expr(&mut self, expr: &RangeExpr) {
        maybe_wrap!(self, expr);
        if let Some(ref start) = expr.start {
            self.fmt_expr(start);
        }
        self.insert("..");
        if let Some(ref end) = expr.end {
            self.fmt_expr(end);
        }
    }

    #[inline]
    fn fmt_box_expr(&mut self, expr: &BoxExpr) {
        maybe_wrap!(self, "box ", "box ", expr.expr, fmt_expr);
    }

    #[inline]
    fn fmt_cast_expr(&mut self, expr: &CastExpr) {
        self.fmt_expr(&expr.expr);
        maybe_wrap!(self, " as ", "as ", expr.ty, fmt_type);
    }

    #[inline]
    fn fmt_type_expr(&mut self, expr: &TypeExpr) {
        self.fmt_expr(&expr.expr);
        maybe_wrap!(self, ": ", ":", expr.ty, fmt_type);
    }

    #[inline]
    fn fmt_block_expr(&mut self, expr: &Block) {
        self.block_non_sep = true;
        self.fmt_block(expr);
    }

    #[inline]
    fn fmt_if_expr(&mut self, expr: &IfExpr) {
        self.block_non_sep = false;
        self.raw_insert("if ");
        self.fmt_expr(&expr.expr);
        self.fmt_block(&expr.block);
        if let Some(ref br) = expr.br {
            self.block_non_sep = true;
            self.raw_insert(" else ");
            self.fmt_expr(br);
        }
    }

    #[inline]
    fn fmt_if_let_expr(&mut self, expr: &IfLetExpr) {
        self.block_non_sep = false;
        self.raw_insert("if let ");
        self.fmt_patten(&expr.pat);
        maybe_wrap!(self, " = ", "= ", expr.expr, fmt_expr);
        self.fmt_block(&expr.block);
        if let Some(ref br) = expr.br {
            self.block_non_sep = true;
            self.raw_insert(" else ");
            self.fmt_expr(br);
        }
    }

    #[inline]
    fn fmt_label(&mut self, label: &Option<String>) {
        if let Some(ref label) = *label {
            self.insert(&format!("{}:", label));
            self.nl();
            self.insert_indent();
        }
    }

    #[inline]
    fn fmt_while_expr(&mut self, expr: &WhileExpr) {
        self.fmt_label(&expr.label);
        self.raw_insert("while ");
        self.fmt_expr(&expr.expr);
        self.fmt_block(&expr.block);
    }

    #[inline]
    fn fmt_while_let_expr(&mut self, expr: &WhileLetExpr) {
        self.fmt_label(&expr.label);
        self.raw_insert("while let ");
        self.fmt_patten(&expr.pat);
        maybe_wrap!(self, " = ", "= ", expr.expr, fmt_expr);
        self.fmt_block(&expr.block);
    }

    #[inline]
    fn fmt_for_expr(&mut self, expr: &ForExpr) {
        self.fmt_label(&expr.label);
        self.raw_insert("for ");
        self.fmt_patten(&expr.pat);
        maybe_wrap!(self, " in ", "in ", expr.expr, fmt_expr);
        self.fmt_block(&expr.block);
    }

    #[inline]
    fn fmt_loop_expr(&mut self, expr: &LoopExpr) {
        self.fmt_label(&expr.label);
        self.raw_insert("loop");
        self.fmt_block(&expr.block);
    }

    #[inline]
    fn fmt_jump_label(&mut self, keyword: &str, label: &Option<Chunk>) {
        let label = if let Some(ref label) = *label {
            format!(" {}", label.s)
        } else {
            String::new()
        };
        self.insert(&format!("{}{}", keyword, label));
    }

    #[inline]
    fn fmt_break_expr(&mut self, expr: &BreakExpr) {
        self.fmt_jump_label("break", &expr.label);
    }

    #[inline]
    fn fmt_continue_expr(&mut self, expr: &ContinueExpr) {
        self.fmt_jump_label("continue", &expr.label);
    }

    #[inline]
    fn fmt_match_expr(&mut self, expr: &MatchExpr) {
        self.raw_insert("match ");
        self.fmt_expr(&expr.expr);
        fmt_block!(self, &expr.arms, fmt_arms);
    }

    #[inline]
    fn fmt_arms(&mut self, arms: &Vec<Arm>) {
        fmt_items!(self, arms, fmt_arm);
    }

    #[inline]
    fn fmt_arm(&mut self, arm: &Arm) {
        fmt_lists!(self, " | ", "| ", &arm.pats, fmt_patten);
        if let Some(ref guard) = arm.guard {
            maybe_wrap!(self, " if ", "if ", guard, fmt_expr);
        }
        maybe_wrap!(self, " => ", "=> ", arm.body, fmt_expr);
        self.raw_insert(",");
    }

    #[inline]
    fn fmt_fn_call_expr(&mut self, expr: &FnCallExpr) {
        self.fmt_expr(&expr.name);
        fmt_comma_lists!(self, "(", ")", &expr.args, fmt_expr);
    }

    #[inline]
    fn fmt_method_call_expr(&mut self, expr: &MethodCallExpr) {
        self.fmt_expr(&expr.obj);
        self.insert(&format!(".{}", &expr.name.s));
        if !expr.types.is_empty() {
            self.insert("::");
            fmt_comma_lists!(self, "<", ">", &expr.types, fmt_type);
        }
        fmt_comma_lists!(self, "(", ")", &expr.args, fmt_expr);
    }

    #[inline]
    fn fmt_closure_expr(&mut self, expr: &ClosureExpr) {
        if expr.moved {
            self.insert("move ");
        }

        self.fmt_closure_fn_arg(&expr.fn_sig.arg);
        self.fmt_fn_return(&expr.fn_sig.ret);

        if expr.block.stmts.len() > 1 {
            self.fmt_block(&expr.block);
        } else {
            self.fmt_closure_stmt(&expr.block.stmts[0]);
        }
    }

    #[inline]
    fn fmt_closure_fn_arg(&mut self, arg: &FnArg) {
        if arg.va {
            self.insert_mark_align("|");

            let mut first = true;
            for e in &arg.args {
                if !first {
                    insert_sep!(self, ",", e);
                }

                self.fmt_closure_arg(e);
                first = false;
            }

            self.insert(", ...");
            self.insert_unmark_align("|");
        } else {
            fmt_comma_lists!(self, "|", "|", &arg.args, fmt_closure_arg);
        }
    }

    #[inline]
    fn fmt_closure_arg(&mut self, arg: &Arg) {
        maybe_nl!(self, arg);
        maybe_wrap!(self, arg);

        self.fmt_patten(&arg.pat);
        if let TypeKind::Infer = arg.ty.ty {
            return;
        } else {
            maybe_wrap!(self, ": ", ":", arg.ty, fmt_type);
        }
    }

    #[inline]
    fn fmt_closure_stmt(&mut self, stmt: &Stmt) {
        self.try_fmt_leading_comments(&stmt.loc);
        match stmt.stmt {
            StmtKind::Expr(ref expr, is_semi) if !is_semi => {
                maybe_wrap!(self, " ", "", expr, fmt_expr)
            },
            _ => unreachable!(),
        }
        self.try_fmt_trailing_comment(&stmt.loc);
    }

    #[inline]
    fn fmt_return_expr(&mut self, expr: &ReturnExpr) {
        self.raw_insert("return");
        if let Some(ref expr) = expr.ret {
            maybe_wrap!(self, " ", "", expr, fmt_expr);
        }
    }

    #[inline]
    fn fmt_macro_item(&mut self, item: &MacroItem) {
        let lines = item.s.s.split('\n').collect::<Vec<_>>();
        let len = lines.len();
        for idx in 0..len {
            if idx > 0 {
                self.nl();
            }

            self.raw_insert(lines[idx]);
            if idx == len - 1 {
                match item.style {
                    MacroStyle::Paren | MacroStyle::Bracket => self.raw_insert(";"),
                    _ => (),
                }
            }
        }
    }

    #[inline]
    fn fmt_macro_stmt(&mut self, stmt: &MacroStmt, is_semi: bool) {
        self.try_fmt_leading_comments(&stmt.loc);
        self.fmt_attrs(&stmt.attrs);
        self.insert_indent();

        self.fmt_macro(&stmt.mac);
        if is_semi {
            self.raw_insert(";");
        }

        self.try_fmt_trailing_comment(&stmt.loc);
        self.nl();
    }

    #[inline]
    fn fmt_macro(&mut self, mac: &Macro) {
        let (open, close) = match mac.style {
            MacroStyle::Paren => ("(", ")"),
            MacroStyle::Bracket => ("[", "]"),
            MacroStyle::Brace => ("{", "}"),
        };

        self.insert(&format!("{}!", mac.name));
        self.insert_mark_align(open);
        let expr_len = mac.exprs.len();
        for i in 0..expr_len {
            let expr = &mac.exprs[i];
            if i > 0 {
                insert_sep!(self, mac.seps[i - 1], expr);
            }
            self.fmt_expr(expr);
        }
        self.insert_unmark_align(close);
    }
}
