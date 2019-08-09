use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Display};

use ir::*;
use ts::*;

use crate::ir;
use crate::ts;
use crate::{need_nl_indent, need_wrap};

macro_rules! display_lists {
    ($f:expr, $open:expr, $sep:expr, $close:expr, $($lists:expr),+) => ({
        write!($f, $open)?;

        let mut first = true;
        $(for e in $lists {
            if !first {
                write!($f, "{}", $sep)?;
            }
            Display::fmt(e, $f)?;
            first = false;
        })+

        write!($f, $close)
    });

    ($f:expr, $sep:expr, $($lists:expr),+) => ({
       display_lists!($f, "", $sep, "", $($lists)+)
    });
}

macro_rules! display_fields_block {
    ($f:expr, $fields: expr) => ({
        writeln!($f, " {{")?;
        display_fields!($f, $fields)?;
        write!($f, "}}")
    })
}

macro_rules! display_decls_block {
    ($f:expr, $items: expr) => ({
        writeln!($f, " {{")?;
        display_decls!($f, $items)?;
        write!($f, "}}")
    })
}

macro_rules! display_block {
    ($f:expr, $items: expr) => ({
        writeln!($f, " {{")?;
        display_items!($f, $items)?;
        write!($f, "}}")
    })
}

macro_rules! display_fields {
    ($f:expr, $fields:expr) => ({
        display_lines!($f, $fields, ",")
    });
}

macro_rules! display_decls {
    ($f:expr, $items:expr) => ({
        display_lines!($f, $items, ";")
    });
}

macro_rules! display_items {
    ($f:expr, $items: expr) => ({
        display_lines!($f, $items, "")
    })
}

macro_rules! display_lines {
    ($f:expr, $lines:expr, $sep:expr) => ({
        for line in $lines {
            writeln!($f, "{}{}", line, $sep)?;
        }
        Ok(())
    });
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
select_str!(range, is_inclusive, "..=", "..");

impl Display for Crate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_attrs(f, &self.attrs)?;
        Display::fmt(&self.module, f)
    }
}

impl Display for AttrKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AttrKind::Doc(ref doc) => Display::fmt(doc, f),
            AttrKind::Attr(ref attr) => Display::fmt(attr, f),
        }
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        for line in self.s.split('\n') {
            if !first {
                writeln!(f)?
            }

            write!(f, "{}", line)?;
            first = false;
        }
        Ok(())
    }
}

impl Display for Attr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#")?;
        if self.is_inner {
            write!(f, "!")?;
        }
        write!(f, "[{}]", self.item)
    }
}

impl Display for MetaItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(ref items) = self.items {
            display_lists!(f, "(", ", ", ")", &**items)?;
        }
        Ok(())
    }
}

impl Display for Mod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for item in &self.items {
            writeln!(f, "{}", item)?;
        }
        Ok(())
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_attrs(f, &self.attrs)?;
        display_vis(f, &self.vis)?;
        match self.item {
            ItemKind::Mod(ref item) => {
                writeln!(f, "mod {} {{", item.name)?;
                Display::fmt(item, f)?;
                return write!(f, "}}");
            },
            ItemKind::ModDecl(ref item) => Display::fmt(item, f)?,
            ItemKind::ExternCrate(ref item) => Display::fmt(item, f)?,
            ItemKind::Use(ref item) => Display::fmt(item, f)?,
            ItemKind::TypeAlias(ref item) => Display::fmt(item, f)?,
            ItemKind::TraitAlias(ref item) => Display::fmt(item, f)?,
            ItemKind::Existential(ref item) => Display::fmt(item, f)?,
            ItemKind::Const(ref item) => Display::fmt(item, f)?,
            ItemKind::Static(ref item) => Display::fmt(item, f)?,
            ItemKind::Struct(ref item) => Display::fmt(item, f)?,
            ItemKind::Union(ref item) => Display::fmt(item, f)?,
            ItemKind::Enum(ref item) => Display::fmt(item, f)?,
            ItemKind::ForeignMod(ref item) => Display::fmt(item, f)?,
            ItemKind::Fn(ref item) => Display::fmt(item, f)?,
            ItemKind::Trait(ref item) => Display::fmt(item, f)?,
            ItemKind::Impl(ref item) => Display::fmt(item, f)?,
            ItemKind::MacroDef(ref item) => Display::fmt(item, f)?,
            ItemKind::Macro(ref item) => Display::fmt(item, f)?,
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
        write!(f, "use {}", self.path)?;
        fmt_use_trees(f, &self.trees)
    }
}

impl Display for UseTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path)?;
        fmt_use_trees(f, &self.trees)
    }
}

impl Display for ModDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mod {}", self.name)
    }
}

impl Display for Generics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.is_empty() {
            display_lists!(f, "<", ", ", ">", &self.lifetime_defs, &self.type_params)?;
        }
        if !self.wh.is_empty() {
            write!(f, " where {}", self.wh)?;
        }
        Ok(())
    }
}

impl Display for LifetimeDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.lifetime, f)?;
        if !self.bounds.is_empty() {
            write!(f, ": ")?;
            display_lists!(f, " + ", &self.bounds)?
        }
        Ok(())
    }
}

impl Display for TypeParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        try_display_type_param_bounds(f, &self.bounds)?;
        if let Some(ref ty) = self.default {
            write!(f, " = {}", ty)?;
        }

        Ok(())
    }
}

impl Display for TypeParamBound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TypeParamBound::Lifetime(ref bound) => Display::fmt(bound, f),
            TypeParamBound::PolyTraitRef(ref bound) => Display::fmt(bound, f),
        }
    }
}

impl Display for TypeParamBounds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_lists!(f, " + ", &self.0)
    }
}

impl Display for PolyTraitRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_for_liftime_defs(f, &self.lifetime_defs)?;
        Display::fmt(&self.trait_ref, f)
    }
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
        display_for_liftime_defs(f, &self.lifetime_defs)?;
        write!(f, "{}: ", &self.ty)?;
        display_type_param_bounds(f, &self.bounds)
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_path_segments(f, &self.segments)
    }
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
            display_lists!(f, "<", ", ", ">", &self.lifetimes, &self.types, &self.bindings)?;
        }
        Ok(())
    }
}

impl Display for TypeBinding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.binding {
            TypeBindingKind::Eq(ref ty) => write!(f, "{}={}", self.name, ty),
            TypeBindingKind::Bound(ref bounds) => {
                write!(f, "{}: ", self.name)?;
                display_lists!(f, "+", &bounds.0)
            },
        }
    }
}

impl Display for ParenParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_paren_param_inputs(f, &self.inputs)?;
        if let Some(ref output) = self.output {
            write!(f, " -> {}", output)?;
        }
        Ok(())
    }
}

impl Display for TypeAlias {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "type {}{} = {}", self.name, self.generics, self.ty)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.ty {
            TypeKind::Symbol(ref ty) => Display::fmt(ty, f),
            TypeKind::Path(ref ty) => Display::fmt(ty, f),
            TypeKind::Ptr(ref ty) => Display::fmt(ty, f),
            TypeKind::Ref(ref ty) => Display::fmt(ty, f),
            TypeKind::Tuple(ref ty) => Display::fmt(ty, f),
            TypeKind::Slice(ref ty) => Display::fmt(ty, f),
            TypeKind::Array(ref ty) => Display::fmt(ty, f),
            TypeKind::Trait(ref ty) => Display::fmt(ty, f),
            TypeKind::BareFn(ref ty) => Display::fmt(ty, f),
            TypeKind::Macro(ref ty) => Display::fmt(ty, f),
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

impl Display for TupleType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_lists!(f, "(", ", ", ")", &self.types)
    }
}

impl Display for SliceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.ty)
    }
}

impl Display for ArrayType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}; {}]", self.ty, self.expr)
    }
}

impl Display for TraitType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", trait_type_head(self.is_dyn, self.is_impl))?;
        display_type_param_bounds(f, &self.bounds)
    }
}

impl Display for BareFnType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_for_liftime_defs(f, &self.lifetime_defs)?;
        write!(f, "{}{}", fn_head(&self.header), self.sig)
    }
}

impl Display for TupleField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", vis_head(&self.vis), self.ty)
    }
}

impl Display for FnSig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_args(f, &self.args)?;
        Display::fmt(&self.ret, f)
    }
}

impl Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.has_patten {
            write!(f, "{}", self.patten)?;
            match self.ty.ty {
                TypeKind::Symbol(s) if s == "_" => Ok(()),
                _ => write!(f, ": {}", self.ty),
            }
        } else {
            if let PattenKind::Ident(ref patten) = self.patten.patten {
                write!(f, "{}{}", ident_patten_head(patten.is_ref, patten.is_mut), self.ty)
            } else {
                Display::fmt(&self.ty, f)
            }
        }
    }
}

impl Display for Return {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.ret {
            Some(ref ty) => write!(f, " -> {}", ty),
            None => Ok(()),
        }
    }
}

impl Display for TraitAlias {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "trait {}{} = {}", self.name, self.generics, self.bounds)
    }
}

impl Display for Existential {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "existential type {}{}: {}", self.name, self.generics, self.bounds)
    }
}

impl Display for Const {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "const {}: {} = {}", self.name, self.ty, self.expr)
    }
}

impl Display for Static {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}: {} = {}", static_head(self.is_mut), self.name, self.ty, self.expr)
    }
}

impl Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "struct {}{}{}", self.name, self.generics, self.body)
    }
}

impl Display for StructBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StructBody::Struct(ref fields) => {
                display_fields_block!(f, fields)
            },
            StructBody::Tuple(ref fields) => {
                display_lists!(f, "(", ", ", ")", fields)
            },
            StructBody::Unit => Ok(()),
        }
    }
}

impl Display for StructField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_attrs(f, &self.attrs)?;
        display_vis(f, &self.vis)?;
        write!(f, "{}: {}", self.name, self.ty)
    }
}

impl Display for Union {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "union {}{}", self.name, self.generics)?;
        display_fields_block!(f, &self.fields)
    }
}

impl Display for Enum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "enum {}{}{}", self.name, self.generics, self.body)
    }
}

impl Display for EnumBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_fields_block!(f, &self.fields)
    }
}

impl Display for EnumField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_attrs(f, &self.attrs)?;
        write!(f, "{}{}", self.name, self.body)?;
        if let Some(ref expr) = self.expr {
            write!(f, " = {}", expr)?;
        }
        Ok(())
    }
}

impl Display for ForeignMod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", foreign_head(&self.abi))?;
        display_decls_block!(f, &self.items)
    }
}

impl Display for ForeignItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_attrs(f, &self.attrs)?;
        display_vis(f, &self.vis)?;
        Display::fmt(&self.item, f)
    }
}

impl Display for ForeignKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ForeignKind::Type(ref item) => write!(f, "type {}", item),
            ForeignKind::Static(ref item) => Display::fmt(item, f),
            ForeignKind::Fn(ref item) => Display::fmt(item, f),
            ForeignKind::Macro(ref item) => Display::fmt(item, f),
        }
    }
}

impl Display for ForeignStatic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}: {}", static_head(self.is_mut), self.name, self.ty)
    }
}

impl Display for ForeignFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "fn {}", self.name)?;
        display_generics(f, &self.generics)?;
        Display::fmt(&self.sig, f)?;
        display_where(f, &self.generics)
    }
}

impl Display for Fn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", fn_head(&self.header), self.name)?;
        display_generics(f, &self.generics)?;
        Display::fmt(&self.sig, f)?;
        display_where(f, &self.generics)?;
        Display::fmt(&self.block, f)
    }
}

impl Display for Trait {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", trait_head(self.is_auto, self.is_unsafe), self.name)?;
        display_generics(f, &self.generics)?;
        try_display_type_param_bounds(f, &self.bounds)?;
        display_where(f, &self.generics)?;
        display_decls_block!(f, &self.items)
    }
}

impl Display for TraitItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_attrs(f, &self.attrs)?;
        match self.item {
            TraitItemKind::Const(ref item) => Display::fmt(item, f),
            TraitItemKind::Type(ref item) => Display::fmt(item, f),
            TraitItemKind::Method(ref item) => Display::fmt(item, f),
            TraitItemKind::Macro(ref item) => Display::fmt(item, f),
        }
    }
}

impl Display for ConstTraitItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "const {}: {}", self.name, self.ty)?;
        if let Some(ref expr) = self.expr {
            Display::fmt(expr, f)?;
        }
        Ok(())
    }
}

impl Display for TypeTraitItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "type {}", self.name)?;
        display_generics(f, &self.generics)?;
        try_display_type_param_bounds(f, &self.bounds)?;
        display_where(f, &self.generics)?;
        if let Some(ref ty) = self.ty {
            write!(f, " = {}", ty)?;
        }
        Ok(())
    }
}

impl Display for MethodTraitItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.sig, f)?;
        if let Some(ref block) = self.block {
            Display::fmt(block, f)?;
        }
        Ok(())
    }
}

impl Display for MethodSig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", fn_head(&self.header), self.name)?;
        display_generics(f, &self.generics)?;
        Display::fmt(&self.sig, f)?;
        display_where(f, &self.generics)
    }
}

impl Display for Impl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", impl_head(self.is_unsafe, self.is_default))?;
        display_generics(f, &self.generics)?;
        write!(f, " ")?;

        if self.is_neg {
            write!(f, "!")?;
        }
        if let Some(ref trait_ref) = self.trait_ref {
            Display::fmt(trait_ref, f)?;
            write!(f, " for {}", self.ty)?;
        } else {
            Display::fmt(&self.ty, f)?;
        }
        display_where(f, &self.generics)?;

        display_block!(f, &self.items)
    }
}

impl Display for ImplItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_attrs(f, &self.attrs)?;
        display_vis(f, &self.vis)?;
        write!(f, "{}", impl_item_head(self.is_default))?;

        let mut is_method = false;
        match self.item {
            ImplItemKind::Const(ref item) => Display::fmt(item, f)?,
            ImplItemKind::Type(ref item) => Display::fmt(item, f)?,
            ImplItemKind::Existential(ref item) => Display::fmt(item, f)?,
            ImplItemKind::Method(ref item) => {
                is_method = true;
                Display::fmt(item, f)?
            },
            ImplItemKind::Macro(ref item) => Display::fmt(item, f)?,
        }
        if !is_method {
            write!(f, ";")?;
        }
        Ok(())
    }
}

impl Display for TypeImplItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "type {}{} = {}", self.name, self.generics, self.ty)
    }
}

impl Display for ExistentialImplItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "existential type {}{}: {}", self.name, self.generics, self.bounds)
    }
}

impl Display for MethodImplItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.sig, f)?;
        Display::fmt(&self.block, f)
    }
}

impl Display for Patten {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.patten {
            PattenKind::Wildcard => write!(f, "_"),
            PattenKind::Symbol(ref patten) => Display::fmt(patten, f),
            PattenKind::Literal(ref patten) => Display::fmt(patten, f),
            PattenKind::Range(ref patten) => Display::fmt(patten, f),
            PattenKind::Ref(ref patten) => Display::fmt(patten, f),
            PattenKind::Path(ref patten) => Display::fmt(patten, f),
            PattenKind::Ident(ref patten) => Display::fmt(patten, f),
            PattenKind::Struct(ref patten) => Display::fmt(patten, f),
            PattenKind::Enum(ref patten) => Display::fmt(patten, f),
            PattenKind::Tuple(ref patten) => Display::fmt(patten, f),
            PattenKind::Slice(ref patten) => Display::fmt(patten, f),
            PattenKind::Macro(ref patten) => Display::fmt(patten, f),
        }
    }
}

impl Display for RangePatten {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}{}", self.start, range(self.is_inclusive), self.end)
    }
}

impl Display for RefPatten {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", ref_head(&None, self.is_mut), self.patten)
    }
}

impl Display for IdentPatten {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", ident_patten_head(self.is_ref, self.is_mut), self.name)?;
        if let Some(ref patten) = self.patten {
            write!(f, " @ {}", patten)?;
        }
        Ok(())
    }
}

impl Display for StructPatten {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.path, f)?;

        if self.fields.is_empty() {
            if self.omit {
                return write!(f, " {{..}}");
            } else {
                return write!(f, " {{}}");
            }
        }

        write!(f, " {{ ")?;
        display_lists!(f, ", ", &self.fields)?;
        if self.omit {
            write!(f, ", ..")?;
        }
        write!(f, " }}")
    }
}

impl Display for StructFieldPatten {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.shorthand {
            Display::fmt(&self.patten, f)
        } else {
            write!(f, "{}: {}", self.name, self.patten)
        }
    }
}

impl Display for EnumPatten {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.path, f)?;
        display_lists!(f, "(", ", ", ")", &self.pattens)
    }
}

impl Display for TuplePatten {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_lists!(f, "(", ", ", ")", &self.pattens)
    }
}

impl Display for SlicePatten {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_lists!(f, "[", ", ", "]", &self.pattens)
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", block_head(self.is_unsafe))?;
        display_block!(f, &self.stmts)
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.stmt {
            StmtKind::Item(ref item) => Display::fmt(item, f),
            StmtKind::Let(ref item) => Display::fmt(item, f),
            StmtKind::Expr(ref item, is_semi) => display_expr(f, item, is_semi),
            StmtKind::Macro(ref item) => Display::fmt(item, f),
        }
    }
}

impl Display for Let {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_attrs(f, &self.attrs)?;
        write!(f, "let {}", self.patten)?;
        if let Some(ref ty) = self.ty {
            write!(f, ":{}", ty)?;
        }
        if let Some(ref expr) = self.init {
            write!(f, " = {}", expr)?;
        }
        Ok(())
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.expr {
            ExprKind::Literal(ref expr) => Display::fmt(expr, f),
            ExprKind::Path(ref expr) => Display::fmt(expr, f),
            ExprKind::Ref(ref expr) => Display::fmt(expr, f),
            ExprKind::UnaryOp(ref expr) => Display::fmt(expr, f),
            ExprKind::Try(ref expr) => Display::fmt(expr, f),
            ExprKind::ListOp(ref expr) => Display::fmt(expr, f),
            ExprKind::Repeat(ref expr) => Display::fmt(expr, f),
            ExprKind::Array(ref exprs) => display_lists!(f, "[", ", ", "]", &**exprs),
            ExprKind::Tuple(ref exprs) => display_lists!(f, "(", ", ", ")", &**exprs),
            ExprKind::Index(ref expr) => Display::fmt(expr, f),
            ExprKind::Struct(ref expr) => Display::fmt(expr, f),
            ExprKind::Field(ref expr) => Display::fmt(expr, f),
            ExprKind::Type(ref expr) => Display::fmt(expr, f),
            ExprKind::Cast(ref expr) => Display::fmt(expr, f),
            ExprKind::Range(ref expr) => Display::fmt(expr, f),
            ExprKind::Block(ref expr) => Display::fmt(expr, f),
            ExprKind::If(ref expr) => Display::fmt(expr, f),
            ExprKind::While(ref expr) => Display::fmt(expr, f),
            ExprKind::Let(ref expr) => Display::fmt(expr, f),
            ExprKind::For(ref expr) => Display::fmt(expr, f),
            ExprKind::Loop(ref expr) => Display::fmt(expr, f),
            ExprKind::Break(ref expr) => Display::fmt(expr, f),
            ExprKind::Continue(ref expr) => Display::fmt(expr, f),
            ExprKind::Match(ref expr) => Display::fmt(expr, f),
            ExprKind::FnCall(ref expr) => Display::fmt(expr, f),
            ExprKind::MethodCall(ref expr) => Display::fmt(expr, f),
            ExprKind::Closure(ref expr) => Display::fmt(expr, f),
            ExprKind::Return(ref expr) => Display::fmt(expr, f),
            ExprKind::Macro(ref expr) => Display::fmt(expr, f),
        }
    }
}

impl Display for RefExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", ref_head(&None, self.is_mut), self.expr)
    }
}

impl Display for UnaryOpExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.op, self.expr)
    }
}

impl Display for ListOpExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sep = format!(" {} ", self.op);
        display_lists!(f, sep, &self.exprs)
    }
}

impl Display for RepeatExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}; {}]", &self.value, &self.len)
    }
}

impl Display for IndexExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}[{}]", self.obj, self.index)
    }
}

impl Display for StructExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.path, f)?;
        writeln!(f, " {{")?;
        display_fields!(f, &self.fields)?;
        if let Some(ref base) = self.base {
            writeln!(f, "..{}", base)?;
        }
        write!(f, "}}")
    }
}

impl Display for StructFieldExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = self.value.to_string();
        if self.name == value {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}: {}", self.name, value)
        }
    }
}

impl Display for FieldExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.expr, self.field)
    }
}

impl Display for TypeExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.expr, self.ty)
    }
}

impl Display for CastExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} as {}", self.expr, self.ty)
    }
}

impl Display for RangeExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref start) = self.start {
            Display::fmt(start, f)?;
        }
        if self.is_inclusive {
            write!(f, "..=")?;
        } else {
            write!(f, "..")?;
        }
        if let Some(ref end) = self.end {
            Display::fmt(end, f)?;
        }
        Ok(())
    }
}

impl Display for BlockExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref label) = self.label {
            write!(f, "{}:", label)?;
        }
        Display::fmt(&self.block, f)
    }
}

impl Display for IfExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if is_if_one_line(self) {
            let (if_value, else_value) = exract_if_else_value(self);
            return write!(f, "if {} {{ {} }} else {{ {} }}", self.expr, if_value, else_value);
        }

        write!(f, "if {}", self.expr)?;
        Display::fmt(&self.block, f)?;
        if let Some(ref br) = self.br {
            write!(f, " else {}", br)?
        }
        Ok(())
    }
}

impl Display for WhileExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref label) = self.label {
            writeln!(f, "{}:", label)?;
        }
        write!(f, "while {}", self.expr)?;
        Display::fmt(&self.block, f)
    }
}

impl Display for LetExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_pattens(f, &self.pattens)?;
        write!(f, " = {}", self.expr)
    }
}

impl Display for ForExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref label) = self.label {
            writeln!(f, "{}:", label)?;
        }
        write!(f, "for {} in {}", self.patten, self.expr)?;
        Display::fmt(&self.block, f)
    }
}

impl Display for LoopExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref label) = self.label {
            writeln!(f, "{}:", label)?;
        }
        write!(f, "loop{}", self.block)
    }
}

impl Display for BreakExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "break ")?;
        if let Some(ref label) = self.label {
            Display::fmt(&label, f)?;
        }
        if let Some(ref expr) = self.expr {
            Display::fmt(&expr, f)?;
        }
        Ok(())
    }
}

impl Display for ContinueExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "continue ")?;
        if let Some(ref label) = self.label {
            Display::fmt(&label, f)?;
        }
        Ok(())
    }
}

impl Display for MatchExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "match {}", self.expr)?;
        display_fields_block!(f, &self.arms)
    }
}

impl Display for Arm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_pattens(f, &self.pattens)?;
        if let Some(ref guard) = self.guard {
            write!(f, " if {}", guard)?;
        }
        write!(f, " => {}", self.body)
    }
}

impl Display for FnCallExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        display_lists!(f, "(", ", ", ")", &self.args)
    }
}

impl Display for MethodCallExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", &self.args[0], self.path.name)?;
        if !self.path.is_empty() {
            write!(f, "::{}", &self.path)?;
        }
        display_lists!(f, "(", ", ", ")", &self.args[1..])
    }
}

impl Display for ClosureExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", closure_head(self.is_static, self.is_async, self.is_move))?;
        display_lists!(f, "|", ", ", "|", &self.sig.args)?;
        Display::fmt(&self.sig.ret, f)?;

        match self.expr.expr {
            ExprKind::Block(ref block) => Display::fmt(block, f),
            _ => write!(f, " {}", self.expr),
        }
    }
}

impl Display for ReturnExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "return")?;
        if let Some(ref expr) = self.ret {
            write!(f, " {}", expr)?;
        }
        Ok(())
    }
}

impl Display for MacroDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "macro_rules! {} {{\n    {}\n}}", self.name, self.def)
    }
}

impl Display for MacroStmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_attrs(f, &self.attrs)?;
        Display::fmt(&self.mac, f)?;
        if self.is_semi {
            write!(f, ";")?;
        }
        Ok(())
    }
}

impl Display for Macro {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (open, close) = match self.style {
            MacroStyle::Paren => ("(", ")"),
            MacroStyle::Bracket => ("[", "]"),
            MacroStyle::Brace => ("{", "}"),
        };

        write!(f, "{}!", self.name)?;
        write!(f, "{}", open)?;
        let expr_len = self.exprs.len();
        for i in 0..expr_len {
            let expr = &self.exprs[i];
            if i > 0 {
                let sep = &self.seps[i - 1];
                if sep.is_sep {
                    write!(f, "{} ", sep.s)?;
                } else {
                    write!(f, "{}", sep.s)?;
                }
            }
            Display::fmt(expr, f)?;
        }
        write!(f, "{}", close)
    }
}

#[inline]
fn fmt_use_trees(f: &mut fmt::Formatter, trees: &Option<Vec<UseTree>>) -> fmt::Result {
    if trees.is_none() {
        return Ok(());
    }

    let trees: &Vec<UseTree> = &trees.as_ref().unwrap();
    write!(f, "::")?;
    if trees.len() == 1 {
        write!(f, "{}", trees[0])
    } else {
        display_lists!(f, "{{", ", ", "}}", trees)
    }
}

#[inline]
fn display_attrs(f: &mut fmt::Formatter, attrs: &Vec<AttrKind>) -> fmt::Result {
    for attr in attrs {
        writeln!(f, "{}", attr)?;
    }
    Ok(())
}

#[inline]
fn display_vis(f: &mut fmt::Formatter, vis: &Vis) -> fmt::Result {
    write!(f, "{}", vis_head(vis))
}

#[inline]
fn try_display_type_param_bounds(f: &mut fmt::Formatter, bounds: &TypeParamBounds) -> fmt::Result {
    if !bounds.is_empty() {
        write!(f, ": ")?;
        display_type_param_bounds(f, bounds)?;
    }
    Ok(())
}

#[inline]
fn display_type_param_bounds(f: &mut fmt::Formatter, bounds: &TypeParamBounds) -> fmt::Result {
    display_lists!(f, " + ", &bounds.0)
}

#[inline]
fn display_for_liftime_defs(f: &mut fmt::Formatter, lifetime_defs: &Vec<LifetimeDef>) -> fmt::Result {
    if !lifetime_defs.is_empty() {
        display_lists!(f, "for<", ", ", "> ", lifetime_defs)?;
    }
    Ok(())
}

#[inline]
fn display_paren_param_inputs(f: &mut fmt::Formatter, inputs: &Vec<Type>) -> fmt::Result {
    display_lists!(f, "(", ", ", ")", inputs)
}

#[inline]
fn display_qself(f: &mut fmt::Formatter, qself: &QSelf, path: &Path) -> fmt::Result {
    write!(f, "<{}", qself.ty)?;
    if qself.pos > 0 {
        write!(f, " as ")?;
        display_path_segments(f, &path.segments[0..qself.pos])?;
    }
    write!(f, ">")?;

    write!(f, "::")?;
    display_path_segments(f, &path.segments[qself.pos..])
}

#[inline]
fn display_path_segments(f: &mut fmt::Formatter, segments: &[PathSegment]) -> fmt::Result {
    display_lists!(f, "", "::", "", segments)
}

#[inline]
fn display_generics(f: &mut fmt::Formatter, generics: &Generics) -> fmt::Result {
    if !generics.is_empty() {
        display_lists!(f, "<", ", ", ">", &generics.lifetime_defs, &generics.type_params)?;
    }
    Ok(())
}

#[inline]
fn display_where(f: &mut fmt::Formatter, generics: &Generics) -> fmt::Result {
    if !generics.wh.is_empty() {
        write!(f, " where {}", generics.wh)?;
    }
    Ok(())
}

#[inline]
fn display_args(f: &mut fmt::Formatter, args: &Vec<Arg>) -> fmt::Result {
    display_lists!(f, "(", ", ", ")", args)
}

#[inline]
fn display_pattens(f: &mut fmt::Formatter, pattens: &Vec<Patten>) -> fmt::Result {
    display_lists!(f, " | ", pattens)
}

#[inline]
fn display_expr(f: &mut fmt::Formatter, expr: &Expr, is_semi: bool) -> fmt::Result {
    Display::fmt(expr, f)?;
    if is_semi {
        write!(f, ";")?;
    }
    Ok(())
}

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
fn trait_type_head(is_dyn: bool, is_impl: bool) -> String {
    let mut head = String::new();
    if is_dyn {
        head.push_str("dyn ");
    }
    if is_impl {
        head.push_str("impl ");
    }
    head
}

#[inline]
fn fn_head(header: &FnHeader) -> String {
    let mut head = String::new();
    if header.is_async {
        head.push_str("async ");
    }
    if header.is_unsafe {
        head.push_str("unsafe ");
    }
    if header.is_const {
        head.push_str("const ");
    }
    head.push_str(&extern_head(&header.abi));
    head.push_str("fn");
    head
}

#[inline]
fn extern_head(abi: &str) -> String {
    let mut head = String::new();
    if abi != r#""Rust""# {
        head.push_str("extern ");
        head.push_str(abi);
        head.push_str(" ");
    }
    head
}

#[inline]
fn trait_head(is_auto: bool, is_unsafe: bool) -> String {
    let mut head = String::new();
    if is_auto {
        head.push_str("auto ");
    }
    if is_unsafe {
        head.push_str("unsafe ");
    }
    head.push_str("trait ");
    head
}

#[inline]
fn impl_head(is_unsafe: bool, is_default: bool) -> String {
    let mut head = String::new();
    if is_unsafe {
        head.push_str("unsafe ");
    }
    if is_default {
        head.push_str("default ");
    }
    head.push_str("impl");
    head
}

#[inline]
fn impl_item_head(is_default: bool) -> String {
    let mut head = String::new();
    if is_default {
        head.push_str("default ");
    }
    head
}

#[inline]
fn vis_head(vis: &Vis) -> String {
    let mut head = String::new();
    if !vis.is_empty() {
        head.push_str(vis);
        head.push_str(" ");
    }
    head
}

#[inline]
fn foreign_head(abi: &str) -> String {
    let mut head = String::new();
    head.push_str("extern");
    head.push_str(" ");
    head.push_str(abi);
    head
}

#[inline]
fn block_head(is_unsafe: bool) -> String {
    let mut head = String::new();
    if is_unsafe {
        head.push_str("unsafe ");
    }
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

#[inline]
fn closure_head(is_static: bool, is_async: bool, is_move: bool) -> String {
    let mut head = String::new();
    if is_static {
        head.push_str("static ");
    }
    if is_async {
        head.push_str("async ");
    }
    if is_move {
        head.push_str("move ");
    }
    head
}

#[inline]
fn is_if_one_line(expr: &IfExpr) -> bool {
    if expr.br.is_none() {
        return false;
    }

    if !expr.block.is_one_literal_expr() {
        return false;
    }

    match expr.br.as_ref().unwrap().expr {
        ExprKind::Block(ref block) => block.block.is_one_literal_expr(),
        _ => false,
    }
}

#[inline]
fn exract_if_else_value(expr: &IfExpr) -> (&Expr, &Expr) {
    let if_value = match &expr.block.stmts[0].stmt {
        StmtKind::Expr(ref expr, _) => expr,
        _ => unreachable!(),
    };

    let else_value = match expr.br.as_ref().unwrap().expr {
        ExprKind::Block(ref block) => {
            match &block.block.stmts[0].stmt {
                StmtKind::Expr(ref expr, _) => expr,
                _ => unreachable!(),
            }
        },
        _ => unreachable!(),
    };

    (if_value, else_value)
}

macro_rules! can_one_line {
    ($sf:expr, $e:expr) => ({
        $sf.ts.can_one_line(&$e.to_string())
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
            false
        } else {
            $sf.wrap();
            true
        }
    });
}

macro_rules! fmt_comma_lists {
    ($sf:expr, $open:expr, $close:expr, $($list:expr, $fmt:ident),+) => ({
        let mut is_wrap = false;
        $sf.insert_mark_align($open);

        let mut first = true;
        $(for e in $list {
            if !first {
                is_wrap |= insert_sep!($sf, ",", e);
            }

            $sf.$fmt(e);
            first = false;
        })+

        $sf.insert_unmark_align($close);
        is_wrap
    });

    ($sf:expr, $($list:expr, $fmt:ident),+) => ({
        fmt_comma_lists!($sf, "", "", $($list, $fmt)+);
    });
}

macro_rules! fmt_item_groups {
    ($sf:expr, $items:expr, $item_kind:path, $item_type:ty, $fmt_item:ident) => ({
        let mut group: Vec<(&Loc, &String, &Vec<AttrKind>, $item_type)> = Vec::new();

        for item in $items {
            match item.item {
                $item_kind(ref e) => {
                    if $sf.has_leading_comments(&item.loc) {
                        fmt_item_group!($sf, &group, $item_type, $fmt_item);
                        group.clear();

                        $sf.fmt_leading_comments(&item.loc);
                    }
                    group.push((&item.loc, &item.vis, &item.attrs, e));
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
        let map: BTreeMap<String, (&Loc, &String, &Vec<AttrKind>, $ty)>
                = $group.into_iter().map(|e| (e.3.to_string(), *e)).collect();

        for (_, e) in map {
            $sf.fmt_attrs(e.2);

            $sf.insert_indent();
            $sf.fmt_vis(e.1);
            $sf.$fmt_item(e.3);

            $sf.try_fmt_trailing_comment(e.0);
            $sf.nl();
        }
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

    ($sf:expr, $($list:expr, $act:ident),+) => ({
        fmt_lists!($sf, " ", " ", $($list, $act)+);
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

        let loc = Loc {
            start: $sf.block_locs.last().unwrap().end,
            ..Default::default()
        };
        $sf.indent();
        $sf.nl();
        $sf.$fmt($block);
        $sf.try_fmt_leading_comments(&loc);

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

pub fn fmt(krate: Crate, leading_cmnts: HashMap<Pos, Vec<String>>, trailing_cmnts: HashMap<Pos, String>) -> TsResult {
    Formatter::new(leading_cmnts, trailing_cmnts).fmt_crate(krate)
}

struct Formatter {
    ts: Typesetter,

    leading_cmnts: HashMap<Pos, Vec<String>>,
    trailing_cmnts: HashMap<Pos, String>,
    block_locs: Vec<Loc>,

    after_indent: bool,
    after_wrap: bool,
    block_non_sep: bool,
}

impl Formatter {
    fn new(leading_cmnts: HashMap<Pos, Vec<String>>, trailing_cmnts: HashMap<Pos, String>) -> Formatter {
        Formatter {
            ts: Typesetter::new(),

            leading_cmnts,
            trailing_cmnts,
            block_locs: Vec::new(),

            after_indent: false,
            after_wrap: false,
            block_non_sep: false,
        }
    }

    fn fmt_crate(mut self, krate: Crate) -> TsResult {
        self.try_fmt_leading_comments(&krate.loc);
        self.fmt_attrs(&krate.attrs);
        self.fmt_mod(&krate.module);
        self.fmt_left_comments(&krate.module.loc);
        self.ts.result()
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
        self.fmt_chunk(doc);
        self.try_fmt_trailing_comment(&doc.loc);
        self.nl();
    }

    #[inline]
    fn fmt_chunk(&mut self, chunk: &Chunk) {
        maybe_nl!(self, chunk);
        self.fmt_long_str(&chunk.s);
    }

    #[inline]
    fn fmt_long_str(&mut self, s: &str) {
        let mut first = true;
        for line in s.split('\n') {
            if !first {
                self.nl();
            }

            self.insert(line);
            first = false;
        }
    }

    fn fmt_attr_group(&mut self, attr_group: &Vec<&Attr>) {
        let sorted_attrs: BTreeMap<_, _> = attr_group.into_iter().map(|e| (e.to_string(), *e)).collect();
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

    fn fmt_meta_items(&mut self, items: &Vec<MetaItem>) {
        fmt_comma_lists!(self, "(", ")", items, fmt_meta_item);
    }

    #[inline]
    fn fmt_meta_item(&mut self, item: &MetaItem) {
        maybe_nl!(self, item);
        self.fmt_long_str(&item.name);

        if let Some(ref items) = item.items {
            self.fmt_meta_items(items);
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

    #[inline]
    fn fmt_use(&mut self, item: &Use) {
        self.insert(&format!("use {}", &item.path));
        self.fmt_use_trees(&item.trees);
        self.raw_insert(";");
    }

    fn fmt_use_tree(&mut self, item: &UseTree) {
        self.insert(&item.path);
        self.fmt_use_trees(&item.trees);
    }

    fn fmt_use_trees(&mut self, trees: &Option<Vec<UseTree>>) {
        if trees.is_none() {
            return;
        }

        self.insert("::");
        let trees: &Vec<UseTree> = trees.as_ref().unwrap();
        if trees.len() == 1 {
            self.fmt_use_tree(&trees[0]);
        } else {
            fmt_comma_lists!(self, "{", "}", trees, fmt_use_tree);
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
        self.fmt_vis(&item.vis);

        self.block_locs.push(item.loc);
        match item.item {
            ItemKind::ExternCrate(..) | ItemKind::Use(..) | ItemKind::ModDecl(..) => unreachable!(),
            ItemKind::Mod(ref item) => self.fmt_sub_mod(item),
            ItemKind::TypeAlias(ref item) => self.fmt_type_alias(item),
            ItemKind::TraitAlias(ref item) => self.fmt_trait_alias(item),
            ItemKind::Existential(ref item) => self.fmt_existential(item),
            ItemKind::Const(ref item) => self.fmt_const(item),
            ItemKind::Static(ref item) => self.fmt_static(item),
            ItemKind::Struct(ref item) => self.fmt_struct(item),
            ItemKind::Union(ref item) => self.fmt_union(item),
            ItemKind::Enum(ref item) => self.fmt_enum(item),
            ItemKind::ForeignMod(ref item) => self.fmt_foreign_mod(item),
            ItemKind::Fn(ref item) => self.fmt_fn(item),
            ItemKind::Trait(ref item) => self.fmt_trait(item),
            ItemKind::Impl(ref item) => self.fmt_impl(item),
            ItemKind::MacroDef(ref item) => self.fmt_macro_def(item),
            ItemKind::Macro(ref item) => self.fmt_macro_item(item),
        }
        self.block_locs.pop();

        self.try_fmt_trailing_comment(&item.loc);
        self.nl();
    }

    fn fmt_sub_mod(&mut self, item: &Mod) {
        self.insert(&format!("mod {}", &item.name));
        fmt_block!(self, item.items, item, fmt_mod);
    }

    fn fmt_type_alias(&mut self, item: &TypeAlias) {
        self.insert(&format!("type {}", &item.name));
        self.fmt_generics_and_where(&item.generics);
        maybe_wrap!(self, " = ", "= ", item.ty, fmt_type);
        self.raw_insert(";");
    }

    fn fmt_generics(&mut self, generics: &Generics) {
        if !generics.is_empty() {
            fmt_comma_lists!(self, "<", ">", &generics.lifetime_defs, fmt_lifetime_def,
                             &generics.type_params, fmt_type_param);
        }
    }

    #[inline]
    fn fmt_lifetime_def(&mut self, lifetime_def: &LifetimeDef) {
        maybe_nl!(self, lifetime_def);
        maybe_wrap!(self, lifetime_def);

        self.fmt_lifetime(&lifetime_def.lifetime);
        if !lifetime_def.bounds.is_empty() {
            self.raw_insert(": ");
            fmt_lists!(self, " + ", "+ ", &lifetime_def.bounds, fmt_lifetime)
        }
    }

    #[inline]
    fn fmt_lifetime(&mut self, lifetime: &Lifetime) {
        self.fmt_chunk(lifetime);
    }

    #[inline]
    fn fmt_type_param(&mut self, type_param: &TypeParam) {
        maybe_nl!(self, type_param);
        maybe_wrap!(self, type_param);

        self.insert(&type_param.name);
        if let Some(ref ty) = type_param.default {
            maybe_wrap!(self, " = ", "= ", ty, fmt_type);
        } else {
            self.try_fmt_type_param_bounds(&type_param.bounds);
        }
    }

    fn try_fmt_type_param_bounds(&mut self, bounds: &TypeParamBounds) {
        if !bounds.is_empty() {
            self.raw_insert(": ");
            self.fmt_type_param_bounds(bounds);
        }
    }

    fn fmt_type_param_bounds(&mut self, bounds: &TypeParamBounds) {
        fmt_lists!(self, " + ", "+ ", &bounds.0, fmt_type_param_bound)
    }

    #[inline]
    fn fmt_type_param_bound(&mut self, bound: &TypeParamBound) {
        match *bound {
            TypeParamBound::Lifetime(ref lifetime) => self.fmt_lifetime(lifetime),
            TypeParamBound::PolyTraitRef(ref poly_trait_ref) => self.fmt_poly_trait_ref(poly_trait_ref),
        }
    }

    #[inline]
    fn fmt_poly_trait_ref(&mut self, poly_trait_ref: &PolyTraitRef) {
        self.fmt_for_lifetime_defs(&poly_trait_ref.lifetime_defs);
        self.fmt_trait_ref(&poly_trait_ref.trait_ref);
    }

    #[inline]
    fn fmt_for_lifetime_defs(&mut self, lifetime_defs: &Vec<LifetimeDef>) {
        if !lifetime_defs.is_empty() {
            fmt_comma_lists!(self, "for<", "> ", lifetime_defs, fmt_lifetime_def);
        }
    }

    #[inline]
    fn fmt_trait_ref(&mut self, trait_ref: &TraitRef) {
        self.fmt_path(trait_ref, false);
    }

    #[inline]
    fn fmt_where(&mut self, generics: &Generics) {
        let wh = &generics.wh;
        if !wh.is_empty() {
            maybe_nl_indent!(self, " where ", "where ", wh);
            self.fmt_where_clauses(&wh.clauses);
        }
    }

    fn fmt_where_clauses(&mut self, clauses: &Vec<WhereClause>) {
        fmt_comma_lists!(self, clauses, fmt_where_clause);
    }

    #[inline]
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
        self.try_fmt_type_param_bounds(&bound.bounds);
    }

    #[inline]
    fn fmt_generics_and_where(&mut self, generics: &Generics) {
        self.fmt_generics(generics);
        self.fmt_where(generics);
    }

    #[inline]
    fn fmt_path(&mut self, path: &Path, from_expr: bool) {
        maybe_nl!(self, path);
        self.fmt_path_segments(&path.segments, from_expr);
    }

    fn fmt_path_segments(&mut self, segments: &[PathSegment], from_expr: bool) {
        let mut first = true;
        for seg in segments {
            if !first {
                maybe_wrap!(self, "::", "::", seg);
            }

            self.fmt_path_segment(seg, from_expr);
            first = false;
        };
    }

    #[inline]
    fn fmt_path_segment(&mut self, seg: &PathSegment, from_expr: bool) {
        self.insert(&seg.name);
        self.fmt_path_param(&seg.param, from_expr);
    }

    #[inline]
    fn fmt_path_param(&mut self, param: &PathParam, from_expr: bool) {
        match *param {
            PathParam::Angle(ref param) => self.fmt_angle_param(param, from_expr),
            PathParam::Paren(ref param) => self.fmt_paren_param(param),
        }
    }

    fn fmt_angle_param(&mut self, param: &AngleParam, from_expr: bool) {
        if !param.is_empty() {
            if from_expr {
                self.insert("::");
            }
            fmt_comma_lists!(self, "<", ">", &param.lifetimes, fmt_lifetime, &param.types,
                             fmt_type, &param.bindings, fmt_type_binding);
        }
    }

    fn fmt_type_binding(&mut self, binding: &TypeBinding) {
        maybe_nl!(self, binding);
        maybe_wrap!(self, binding);

        self.insert(&binding.name);
        match binding.binding {
            TypeBindingKind::Eq(ref ty) => {
                self.raw_insert("=");
                self.fmt_type(ty);
            },
            TypeBindingKind::Bound(ref bounds) => {
                self.raw_insert(": ");
                fmt_lists!(self, "+", "+", &bounds.0, fmt_type_param_bound);
            },
        }
    }

    fn fmt_paren_param(&mut self, param: &ParenParam) {
        fmt_comma_lists!(self, "(", ")", &param.inputs, fmt_type);
        if let Some(ref output) = param.output {
            maybe_wrap!(self, " -> ", "-> ", output, fmt_type);
        }
    }

    fn fmt_qself_path(&mut self, qself: &QSelf, path: &Path, from_expr: bool) {
        self.insert_mark_align("<");
        self.fmt_type(&qself.ty);
        if qself.pos > 0 {
            self.raw_insert(" as ");
            self.fmt_path_segments(&path.segments[0..qself.pos], from_expr);
        }
        self.insert_unmark_align(">");

        self.insert("::");
        self.fmt_path_segments(&path.segments[qself.pos..], from_expr);
    }

    fn fmt_type(&mut self, ty: &Type) {
        maybe_nl!(self, ty);
        match ty.ty {
            TypeKind::Symbol(ref ty) => self.fmt_symbol_type(ty),
            TypeKind::Path(ref ty) => self.fmt_path_type(ty, false),
            TypeKind::Ptr(ref ty) => self.fmt_ptr_type(ty),
            TypeKind::Ref(ref ty) => self.fmt_ref_type(ty),
            TypeKind::Tuple(ref ty) => self.fmt_tuple_type(ty),
            TypeKind::Slice(ref ty) => self.fmt_slice_type(ty),
            TypeKind::Array(ref ty) => self.fmt_array_type(ty),
            TypeKind::Trait(ref ty) => self.fmt_trait_type(ty),
            TypeKind::BareFn(ref ty) => self.fmt_bare_fn_type(ty),
            TypeKind::Macro(ref ty) => self.fmt_macro(ty),
        }
    }

    #[inline]
    fn fmt_symbol_type(&mut self, ty: &str) {
        self.raw_insert(ty)
    }

    #[inline]
    fn fmt_path_type(&mut self, ty: &PathType, from_expr: bool) {
        match ty.qself {
            Some(ref qself) => {
                maybe_wrap!(self, ty);
                self.fmt_qself_path(qself, &ty.path, from_expr);
            },
            None => self.fmt_path(&ty.path, from_expr),
        }
    }

    #[inline]
    fn fmt_ptr_type(&mut self, ty: &PtrType) {
        let head = ptr_head(ty.is_mut);
        maybe_wrap!(self, head, head, ty.ty, fmt_type);
    }

    #[inline]
    fn fmt_ref_type(&mut self, ty: &RefType) {
        let head = &ref_head(&ty.lifetime, ty.is_mut);
        maybe_wrap!(self, head, head, ty.ty, fmt_type);
    }

    #[inline]
    fn fmt_tuple_type(&mut self, ty: &TupleType) {
        fmt_comma_lists!(self, "(", ")", &ty.types, fmt_type);
    }

    #[inline]
    fn fmt_slice_type(&mut self, ty: &SliceType) {
        self.insert_mark_align("[");
        self.fmt_type(&ty.ty);
        self.insert_unmark_align("]");
    }

    #[inline]
    fn fmt_array_type(&mut self, ty: &ArrayType) {
        self.insert_mark_align("[");
        self.fmt_type(&ty.ty);
        insert_sep!(self, ";", ty.expr);
        self.fmt_expr(&ty.expr);
        self.insert_unmark_align("]");
    }

    #[inline]
    fn fmt_trait_type(&mut self, ty: &TraitType) {
        let head = &trait_type_head(ty.is_dyn, ty.is_impl);
        self.insert(head);
        self.fmt_type_param_bounds(&ty.bounds);
    }

    #[inline]
    fn fmt_bare_fn_type(&mut self, ty: &BareFnType) {
        self.fmt_for_lifetime_defs(&ty.lifetime_defs);
        self.insert(&fn_head(&ty.header));
        self.fmt_fn_sig(&ty.sig);
    }

    #[inline]
    fn fmt_trait_alias(&mut self, item: &TraitAlias) {
        self.insert(&format!("trait {}", &item.name));
        self.fmt_generics_and_where(&item.generics);
        maybe_wrap!(self, " = ", "= ", item.bounds, fmt_type_param_bounds);
        self.raw_insert(";");
    }

    #[inline]
    fn fmt_existential(&mut self, item: &Existential) {
        self.insert(&format!("existential type {}", &item.name));
        self.fmt_generics_and_where(&item.generics);
        maybe_wrap!(self, ": ", ": ", item.bounds, fmt_type_param_bounds);
        self.raw_insert(";");
    }

    #[inline]
    fn fmt_const(&mut self, item: &Const) {
        self.insert(&format!("const {}", item.name));
        insert_sep!(self, ":", item.ty);
        self.fmt_type(&item.ty);
        maybe_wrap!(self, " = ", "= ", item.expr, fmt_expr);
        self.raw_insert(";");
    }

    #[inline]
    fn fmt_static(&mut self, item: &Static) {
        self.insert(&format!("{}{}", static_head(item.is_mut), item.name));
        insert_sep!(self, ":", item.ty);
        self.fmt_type(&item.ty);
        maybe_wrap!(self, " = ", "= ", item.expr, fmt_expr);
        self.raw_insert(";");
    }

    fn fmt_struct(&mut self, item: &Struct) {
        self.insert(&format!("struct {}", item.name));
        self.fmt_generics_and_where(&item.generics);
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

    #[inline]
    fn fmt_struct_field(&mut self, field: &StructField) {
        self.fmt_vis(&field.vis);
        self.insert(&field.name);
        insert_sep!(self, ":", field.ty);
        self.fmt_type(&field.ty);
        self.raw_insert(",");
    }

    fn fmt_tuple_fields(&mut self, fields: &Vec<TupleField>) {
        fmt_comma_lists!(self, "(", ")", fields, fmt_tuple_field);
    }

    #[inline]
    fn fmt_tuple_field(&mut self, field: &TupleField) {
        maybe_nl!(self, field);
        self.try_fmt_leading_comments(&field.loc);
        self.fmt_attrs(&field.attrs);
        self.fmt_vis(&field.vis);
        self.fmt_type(&field.ty);
    }

    fn fmt_union(&mut self, item: &Union) {
        self.insert(&format!("union {}", item.name));
        self.fmt_generics_and_where(&item.generics);
        fmt_block!(self, &item.fields, fmt_struct_fields);
    }

    fn fmt_enum(&mut self, item: &Enum) {
        self.insert(&format!("enum {}", item.name));
        self.fmt_generics_and_where(&item.generics);
        fmt_block!(self, &item.body.fields, fmt_enum_fields);
    }

    fn fmt_enum_fields(&mut self, fields: &Vec<EnumField>) {
        fmt_items!(self, fields, fmt_enum_field);
    }

    #[inline]
    fn fmt_enum_field(&mut self, field: &EnumField) {
        self.insert(&field.name);
        self.fmt_struct_body(&field.body);
        if let Some(ref expr) = field.expr {
            maybe_wrap!(self, " = ", "= ", expr, fmt_expr);
        }
        self.raw_insert(",");
    }

    fn fmt_foreign_mod(&mut self, item: &ForeignMod) {
        self.insert(&foreign_head(&item.abi));
        fmt_block!(self, &item.items, fmt_foreign_items);
    }

    fn fmt_foreign_items(&mut self, items: &Vec<ForeignItem>) {
        fmt_items!(self, items, fmt_foreign_item);
    }

    #[inline]
    fn fmt_foreign_item(&mut self, item: &ForeignItem) {
        self.fmt_vis(&item.vis);
        match item.item {
            ForeignKind::Type(ref item) => self.fmt_foreign_type(item),
            ForeignKind::Static(ref item) => self.fmt_foreign_static(item),
            ForeignKind::Fn(ref item) => self.fmt_foreign_fn(item),
            ForeignKind::Macro(ref item) => self.fmt_macro(item),
        }
        self.raw_insert(";");
    }

    #[inline]
    fn fmt_foreign_type(&mut self, item: &ForeignType) {
        self.insert(&format!("type {};", item));
    }

    #[inline]
    fn fmt_foreign_static(&mut self, item: &ForeignStatic) {
        self.insert(&format!("{}{}", static_head(item.is_mut), item.name));
        insert_sep!(self, ":", item.ty);
        self.fmt_type(&item.ty);
    }

    #[inline]
    fn fmt_foreign_fn(&mut self, item: &ForeignFn) {
        self.insert(&format!("fn {}", item.name));
        self.fmt_generics(&item.generics);
        self.fmt_fn_sig(&item.sig);
        self.fmt_where(&item.generics);
    }

    fn fmt_fn(&mut self, item: &Fn) {
        self.insert(&format!("{} {}", fn_head(&item.header), item.name));
        self.fmt_generics(&item.generics);
        self.fmt_fn_sig(&item.sig);
        self.fmt_where(&item.generics);
        self.fmt_block(&item.block);
    }

    fn fmt_trait(&mut self, item: &Trait) {
        self.insert(&format!("{}{}", trait_head(item.is_auto, item.is_unsafe), item.name));
        self.fmt_generics(&item.generics);
        self.try_fmt_type_param_bounds(&item.bounds);
        self.fmt_where(&item.generics);
        fmt_block!(self, &item.items, fmt_trait_items);
    }

    fn fmt_trait_items(&mut self, items: &Vec<TraitItem>) {
        fmt_items!(self, items, fmt_trait_item);
    }

    #[inline]
    fn fmt_trait_item(&mut self, item: &TraitItem) {
        self.fmt_attrs(&item.attrs);
        match item.item {
            TraitItemKind::Const(ref item) => self.fmt_const_trait_item(item),
            TraitItemKind::Type(ref item) => self.fmt_type_trait_item(item),
            TraitItemKind::Method(ref item) => self.fmt_method_trait_item(item),
            TraitItemKind::Macro(ref item) => self.fmt_macro(item),
        }
    }

    #[inline]
    fn fmt_const_trait_item(&mut self, item: &ConstTraitItem) {
        self.insert(&format!("const {}", item.name));
        insert_sep!(self, ":", item.ty);
        self.fmt_type(&item.ty);
        if let Some(ref expr) = item.expr {
            maybe_wrap!(self, " = ", "= ", expr, fmt_expr);
        }
        self.raw_insert(";");
    }

    #[inline]
    fn fmt_type_trait_item(&mut self, item: &TypeTraitItem) {
        self.insert(&format!("type {}", item.name));
        self.fmt_generics(&item.generics);
        self.try_fmt_type_param_bounds(&item.bounds);
        self.fmt_where(&item.generics);
        if let Some(ref ty) = item.ty {
            maybe_wrap!(self, " = ", "= ", ty, fmt_type);
        }
        self.raw_insert(";");
    }

    #[inline]
    fn fmt_method_trait_item(&mut self, item: &MethodTraitItem) {
        self.fmt_method_sig(&item.sig);
        if let Some(ref block) = item.block {
            self.fmt_block(block);
        } else {
            self.raw_insert(";");
        }
    }

    fn fmt_impl(&mut self, item: &Impl) {
        self.insert(&impl_head(item.is_unsafe, item.is_default));
        self.fmt_generics(&item.generics);
        self.raw_insert(" ");
        if item.is_neg {
            self.raw_insert("!");
        }

        if let Some(ref trait_ref) = item.trait_ref {
            self.fmt_trait_ref(trait_ref);
            maybe_wrap!(self, " for ", "for ", item.ty, fmt_type);
        } else {
            self.fmt_type(&item.ty);
        }
        self.fmt_where(&item.generics);
        fmt_block!(self, &item.items, fmt_impl_items);
    }

    fn fmt_impl_items(&mut self, items: &Vec<ImplItem>) {
        fmt_items!(self, items, fmt_impl_item);
    }

    #[inline]
    fn fmt_impl_item(&mut self, item: &ImplItem) {
        self.fmt_vis(&item.vis);
        self.insert(&impl_item_head(item.is_default));

        let mut is_method = false;
        match item.item {
            ImplItemKind::Const(ref item) => self.fmt_const_impl_item(item),
            ImplItemKind::Type(ref item) => self.fmt_type_impl_item(item),
            ImplItemKind::Existential(ref item) => self.fmt_existential_impl_item(item),
            ImplItemKind::Method(ref item) => {
                is_method = true;
                self.fmt_method_impl_item(item);
            },
            ImplItemKind::Macro(ref item) => self.fmt_macro(item),
        }
        if !is_method {
            self.raw_insert(";");
        }
    }

    #[inline]
    fn fmt_const_impl_item(&mut self, item: &ConstImplItem) {
        self.insert(&format!("const {}", item.name));
        insert_sep!(self, ":", item.ty);
        self.fmt_type(&item.ty);
        maybe_wrap!(self, " = ", "= ", item.expr, fmt_expr);
    }

    #[inline]
    fn fmt_type_impl_item(&mut self, item: &TypeImplItem) {
        self.insert(&format!("type {}", item.name));
        self.fmt_generics_and_where(&item.generics);
        maybe_wrap!(self, " = ", "= ", item.ty, fmt_type);
    }

    #[inline]
    fn fmt_existential_impl_item(&mut self, item: &ExistentialImplItem) {
        self.insert(&format!("existential type {}", &item.name));
        self.fmt_generics_and_where(&item.generics);
        maybe_wrap!(self, ": ", ": ", item.bounds, fmt_type_param_bounds);
    }

    #[inline]
    fn fmt_method_impl_item(&mut self, item: &MethodImplItem) {
        self.fmt_method_sig(&item.sig);
        self.fmt_block(&item.block);
    }

    #[inline]
    fn fmt_fn_sig(&mut self, sig: &FnSig) {
        self.fmt_fn_args(&sig.args);
        self.fmt_fn_return(&sig.ret);
    }

    #[inline]
    fn fmt_fn_args(&mut self, args: &Vec<Arg>) -> bool {
        fmt_comma_lists!(self, "(", ")", args, fmt_arg)
    }

    #[inline]
    fn fmt_arg(&mut self, arg: &Arg) {
        maybe_nl!(self, arg);
        maybe_wrap!(self, arg);

        if arg.has_patten {
            self.fmt_patten(&arg.patten);
            self.raw_insert(": ");
        } else {
            if let PattenKind::Ident(ref patten) = arg.patten.patten {
                self.insert(&ident_patten_head(patten.is_ref, patten.is_mut));
            }
        }
        self.fmt_type(&arg.ty);
    }

    #[inline]
    fn fmt_fn_return(&mut self, ret: &Return) {
        if let Some(ref ty) = ret.ret {
            if ret.nl {
                self.nl_indent();
                self.raw_insert("-> ");
            } else {
                maybe_nl_indent!(self, " -> ", "-> ", ty);
            }
            self.fmt_type(ty);
        }
    }

    #[inline]
    fn fmt_method_sig(&mut self, sig: &MethodSig) {
        self.insert(&format!("{} {}", fn_head(&sig.header), sig.name));
        self.fmt_generics(&sig.generics);
        self.fmt_fn_sig(&sig.sig);
        self.fmt_where(&sig.generics);
    }

    fn fmt_block(&mut self, block: &Block) {
        self.block_locs.push(block.loc);
        self.insert(&block_head(block.is_unsafe));
        fmt_block!(self, &block.stmts, fmt_stmts);
        self.block_locs.pop();
    }

    fn fmt_stmts(&mut self, stmts: &Vec<Stmt>) {
        for stmt in stmts {
            self.fmt_stmt(stmt);
        }
    }

    #[inline]
    fn fmt_stmt(&mut self, stmt: &Stmt) {
        self.block_locs.push(stmt.loc);
        self.try_fmt_leading_comments(&stmt.loc);
        match stmt.stmt {
            StmtKind::Item(ref item) => self.fmt_item(item),
            StmtKind::Let(ref local) => self.fmt_let(local),
            StmtKind::Expr(ref expr, is_semi) => self.fmt_expr_stmt(expr, is_semi),
            StmtKind::Macro(ref mac) => self.fmt_macro_stmt(mac),
        }
        self.block_locs.pop();
    }

    fn fmt_let(&mut self, local: &Let) {
        self.try_fmt_leading_comments(&local.loc);
        self.fmt_attrs(&local.attrs);
        self.insert_indent();

        self.raw_insert("let ");
        self.fmt_patten(&local.patten);
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

    fn fmt_patten(&mut self, patten: &Patten) {
        maybe_nl!(self, patten);
        match patten.patten {
            PattenKind::Wildcard => self.insert("_"),
            PattenKind::Symbol(ref patten) => self.insert(patten),
            PattenKind::Literal(ref patten) => self.fmt_expr(patten),
            PattenKind::Range(ref patten) => self.fmt_range_patten(patten),
            PattenKind::Ref(ref patten) => self.fmt_ref_patten(patten),
            PattenKind::Path(ref patten) => self.fmt_path_type(patten, true),
            PattenKind::Ident(ref patten) => self.fmt_ident_patten(patten),
            PattenKind::Struct(ref patten) => self.fmt_struct_patten(patten),
            PattenKind::Enum(ref patten) => self.fmt_enum_patten(patten),
            PattenKind::Tuple(ref patten) => self.fmt_tuple_patten(patten),
            PattenKind::Slice(ref patten) => self.fmt_slice_patten(patten),
            PattenKind::Macro(ref patten) => self.fmt_macro(patten),
        }
    }

    fn fmt_pattens(&mut self, pattens: &Vec<Patten>) {
        fmt_lists!(self, " | ", "| ", pattens, fmt_patten);
    }

    #[inline]
    fn fmt_range_patten(&mut self, patten: &RangePatten) {
        self.fmt_expr(&patten.start);
        self.insert(range(patten.is_inclusive));
        self.fmt_expr(&patten.end);
    }

    #[inline]
    fn fmt_ref_patten(&mut self, patten: &RefPatten) {
        self.insert(&ref_head(&None, patten.is_mut));
        self.fmt_patten(&patten.patten);
    }

    #[inline]
    fn fmt_ident_patten(&mut self, patten: &IdentPatten) {
        self.insert(&format!("{}{}", ident_patten_head(patten.is_ref, patten.is_mut), patten.name));
        if let Some(ref patten) = patten.patten {
            maybe_wrap!(self, " @ ", "@ ", patten, fmt_patten);
        }
    }

    #[inline]
    fn fmt_struct_patten(&mut self, patten: &StructPatten) {
        self.fmt_path(&patten.path, true);

        if patten.fields.is_empty() {
            if patten.omit {
                self.raw_insert(" {..}");
            } else {
                self.raw_insert(" {}");
            }
            return;
        }

        if can_one_line!(self, patten) {
            self.fmt_struct_patten_one_line(patten);
            return;
        }

        self.open_brace();
        let loc = Loc {
            start: self.block_locs.last().unwrap().end,
            ..Default::default()
        };

        self.fmt_struct_field_pattens(&patten.fields);
        if patten.omit {
            self.insert_indent();
            self.raw_insert("..");
            self.nl();
        }

        self.try_fmt_leading_comments(&loc);
        self.close_brace();
    }

    fn fmt_struct_patten_one_line(&mut self, patten: &StructPatten) {
        self.raw_insert(" { ");
        fmt_lists!(self, &patten.fields, fmt_struct_field_patten);
        if patten.omit {
            self.raw_insert(" ..");
        }
        self.raw_insert(" }");
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
            self.fmt_patten(&field.patten);
        } else {
            self.insert(&field.name);
            maybe_wrap!(self, ": ", ":", field.patten, fmt_patten);
        }
        self.raw_insert(",");
    }

    #[inline]
    fn fmt_enum_patten(&mut self, patten: &EnumPatten) {
        self.fmt_path(&patten.path, true);
        fmt_comma_lists!(self, "(", ")", &patten.pattens, fmt_patten);
    }

    #[inline]
    fn fmt_tuple_patten(&mut self, patten: &TuplePatten) {
        fmt_comma_lists!(self, "(", ")", &patten.pattens, fmt_patten);
    }

    #[inline]
    fn fmt_slice_patten(&mut self, patten: &SlicePatten) {
        fmt_comma_lists!(self, "[", "]", &patten.pattens, fmt_patten);
    }

    #[inline]
    fn fmt_expr_stmt(&mut self, expr: &Expr, is_semi: bool) {
        self.block_locs.push(expr.loc);
        self.try_fmt_leading_comments(&expr.loc);
        self.fmt_attrs(&expr.attrs);
        self.insert_indent();

        self.fmt_expr(expr);
        if is_semi {
            self.raw_insert(";");
        }

        self.try_fmt_trailing_comment(&expr.loc);
        self.nl();
        self.block_locs.pop();
    }

    fn fmt_expr(&mut self, expr: &Expr) {
        self.block_locs.push(expr.loc);
        maybe_nl!(self, expr);
        match expr.expr {
            ExprKind::Literal(ref expr) => self.fmt_literal_expr(expr),
            ExprKind::Path(ref expr) => self.fmt_path_expr(expr),
            ExprKind::Ref(ref expr) => self.fmt_ref_expr(expr),
            ExprKind::UnaryOp(ref expr) => self.fmt_unary_op_expr(expr),
            ExprKind::Try(ref expr) => self.fmt_try_expr(expr),
            ExprKind::ListOp(ref expr) => self.fmt_list_op_expr(expr),
            ExprKind::Repeat(ref expr) => self.fmt_repeat_expr(expr),
            ExprKind::Array(ref exprs) => self.fmt_array_expr(exprs),
            ExprKind::Tuple(ref exprs) => self.fmt_tuple_expr(exprs),
            ExprKind::Index(ref expr) => self.fmt_index_expr(expr),
            ExprKind::Struct(ref expr) => self.fmt_struct_expr(expr),
            ExprKind::Field(ref expr) => self.fmt_field_expr(expr),
            ExprKind::Type(ref expr) => self.fmt_type_expr(expr),
            ExprKind::Cast(ref expr) => self.fmt_cast_expr(expr),
            ExprKind::Range(ref expr) => self.fmt_range_expr(expr),
            ExprKind::Block(ref expr) => self.fmt_block_expr(expr),
            ExprKind::If(ref expr) => self.fmt_if_expr(expr),
            ExprKind::While(ref expr) => self.fmt_while_expr(expr),
            ExprKind::Let(ref expr) => self.fmt_let_expr(expr),
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
        self.block_locs.pop();
    }

    #[inline]
    fn fmt_literal_expr(&mut self, expr: &Chunk) {
        self.fmt_chunk(expr);
    }

    #[inline]
    fn fmt_path_expr(&mut self, expr: &PathExpr) {
        self.fmt_path_type(expr, true);
    }

    #[inline]
    fn fmt_ref_expr(&mut self, expr: &RefExpr) {
        let head = &ref_head(&None, expr.is_mut);
        maybe_wrap!(self, head, head, expr.expr, fmt_expr);
    }

    #[inline]
    fn fmt_unary_op_expr(&mut self, expr: &UnaryOpExpr) {
        maybe_wrap!(self, &expr.op, &expr.op, expr.expr, fmt_expr);
    }

    #[inline]
    fn fmt_try_expr(&mut self, expr: &Expr) {
        self.fmt_expr(expr);
        self.raw_insert("?");
    }

    #[inline]
    fn fmt_list_op_expr(&mut self, expr: &ListOpExpr) {
        let sep = format!(" {} ", expr.op);
        let wrap_sep = format!("{} ", expr.op);
        fmt_lists!(self, &sep, &wrap_sep, &expr.exprs, fmt_expr);
    }

    #[inline]
    fn fmt_repeat_expr(&mut self, expr: &RepeatExpr) {
        self.insert_mark_align("[");
        self.fmt_expr(&expr.value);
        insert_sep!(self, ";", expr.len);
        self.fmt_expr(&expr.len);
        self.insert_unmark_align("]");
    }

    #[inline]
    fn fmt_array_expr(&mut self, exprs: &Vec<Expr>) {
        fmt_comma_lists!(self, "[", "]", exprs, fmt_expr);
    }

    #[inline]
    fn fmt_tuple_expr(&mut self, exprs: &Vec<Expr>) {
        fmt_comma_lists!(self, "(", ")", exprs, fmt_expr);
    }

    #[inline]
    fn fmt_index_expr(&mut self, expr: &IndexExpr) {
        self.fmt_expr(&expr.obj);
        self.insert_mark_align("[");
        self.fmt_expr(&expr.index);
        self.insert_unmark_align("]");
    }

    #[inline]
    fn fmt_struct_expr(&mut self, expr: &StructExpr) {
        self.fmt_path(&expr.path, true);

        if expr.fields.is_empty() {
            self.insert(" {}");
            return;
        }

        self.open_brace();
        let loc = Loc {
            start: self.block_locs.last().unwrap().end,
            ..Default::default()
        };

        self.fmt_struct_field_exprs(&expr.fields);
        if let Some(ref base) = expr.base {
            self.insert_indent();
            self.insert("..");
            self.fmt_expr(base);
            self.try_fmt_trailing_comment(&base.loc);
            self.nl();
        }

        self.try_fmt_leading_comments(&loc);
        self.close_brace();
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
        self.insert(&field.name);
        let value = field.value.to_string();
        if field.name != value {
            insert_sep!(self, ":", field.value);
            self.fmt_expr(&field.value);
        }
        self.raw_insert(",");
    }

    #[inline]
    fn fmt_field_expr(&mut self, expr: &FieldExpr) {
        maybe_wrap!(self, expr);
        self.fmt_expr(&expr.expr);
        self.insert(&format!(".{}", &expr.field));
    }

    #[inline]
    fn fmt_type_expr(&mut self, expr: &TypeExpr) {
        self.fmt_expr(&expr.expr);
        maybe_wrap!(self, ": ", ":", expr.ty, fmt_type);
    }

    #[inline]
    fn fmt_cast_expr(&mut self, expr: &CastExpr) {
        self.fmt_expr(&expr.expr);
        maybe_wrap!(self, " as ", "as ", expr.ty, fmt_type);
    }

    #[inline]
    fn fmt_range_expr(&mut self, expr: &RangeExpr) {
        maybe_wrap!(self, expr);

        if let Some(ref start) = expr.start {
            self.fmt_expr(start);
        }
        if expr.is_inclusive {
            self.insert("..=");
        } else {
            self.insert("..");
        }
        if let Some(ref end) = expr.end {
            self.fmt_expr(end);
        }
    }

    #[inline]
    fn fmt_block_expr(&mut self, expr: &BlockExpr) {
        self.block_non_sep = true;
        if let Some(ref label) = expr.label {
            self.insert(&format!("{}: ", label));
        }
        self.fmt_block(&expr.block);
    }

    #[inline]
    fn fmt_if_expr(&mut self, expr: &IfExpr) {
        if is_if_one_line(expr) {
            return self.fmt_if_expr_one_line(expr);
        }

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
    fn fmt_if_expr_one_line(&mut self, expr: &IfExpr) {
        self.block_non_sep = false;
        self.raw_insert("if ");
        self.fmt_expr(&expr.expr);

        let (if_value, else_value) = exract_if_else_value(expr);
        self.raw_insert(" { ");
        self.fmt_expr(if_value);
        self.raw_insert(" } else { ");
        self.fmt_expr(else_value);
        self.raw_insert(" }");
    }

    #[inline]
    fn fmt_while_expr(&mut self, expr: &WhileExpr) {
        self.fmt_label(&expr.label);
        self.raw_insert("while ");
        self.fmt_expr(&expr.expr);
        self.fmt_block(&expr.block);
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
    fn fmt_let_expr(&mut self, expr: &LetExpr) {
        self.raw_insert("let ");
        self.fmt_pattens(&expr.pattens);
        maybe_wrap!(self, " = ", "= ", expr.expr, fmt_expr);
    }

    #[inline]
    fn fmt_for_expr(&mut self, expr: &ForExpr) {
        self.fmt_label(&expr.label);
        self.raw_insert("for ");
        self.fmt_patten(&expr.patten);
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
    fn fmt_break_expr(&mut self, expr: &BreakExpr) {
        self.fmt_jump_label("break", &expr.label);
        if let Some(ref expr) = expr.expr {
            self.raw_insert(" ");
            self.fmt_expr(expr);
        }
    }

    #[inline]
    fn fmt_continue_expr(&mut self, expr: &ContinueExpr) {
        self.fmt_jump_label("continue", &expr.label);
    }

    #[inline]
    fn fmt_jump_label(&mut self, keyword: &str, label: &Option<String>) {
        let label = if let Some(ref label) = *label {
            format!(" {}", label)
        } else {
            String::new()
        };
        self.insert(&format!("{}{}", keyword, label));
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
        fmt_lists!(self, " | ", "| ", &arm.pattens, fmt_patten);
        if let Some(ref guard) = arm.guard {
            maybe_wrap!(self, " if ", "if ", guard, fmt_expr);
        }

        match arm.body.expr {
            ExprKind::Block(..) => {
                self.raw_insert(" => ");
                self.fmt_expr(&arm.body);
            },
            _ => {
                self.raw_insert(" =>");
                maybe_wrap!(self, " ", "", &arm.body, fmt_expr);
            },
        }
        self.raw_insert(",");
    }

    #[inline]
    fn fmt_fn_call_expr(&mut self, expr: &FnCallExpr) {
        self.fmt_expr(&expr.name);
        fmt_comma_lists!(self, "(", ")", &expr.args, fmt_expr);
    }

    #[inline]
    fn fmt_method_call_expr(&mut self, expr: &MethodCallExpr) {
        self.fmt_expr(&expr.args[0]);
        self.raw_insert(".");
        self.fmt_path_segment(&expr.path, true);
        fmt_comma_lists!(self, "(", ")", &expr.args[1..], fmt_expr);
    }

    #[inline]
    fn fmt_closure_expr(&mut self, expr: &ClosureExpr) {
        self.insert(&closure_head(expr.is_static, expr.is_async, expr.is_move));
        self.fmt_closure_args(&expr.sig.args);
        self.fmt_fn_return(&expr.sig.ret);

        match expr.expr.expr {
            ExprKind::Block(ref block) => self.fmt_block(&block.block),
            _ => {
                self.raw_insert(" ");
                self.fmt_expr(&expr.expr);
            },
        }
    }

    #[inline]
    fn fmt_closure_args(&mut self, args: &Vec<Arg>) {
        fmt_comma_lists!(self, "|", "|", args, fmt_closure_arg);
    }

    #[inline]
    fn fmt_closure_arg(&mut self, arg: &Arg) {
        maybe_nl!(self, arg);
        maybe_wrap!(self, arg);

        self.fmt_patten(&arg.patten);
        match arg.ty.ty {
            TypeKind::Symbol(ref s) if s == &"_" => {},
            _ => {
                self.raw_insert(": ");
                self.fmt_type(&arg.ty)
            },
        }
    }

    #[inline]
    fn fmt_return_expr(&mut self, expr: &ReturnExpr) {
        self.raw_insert("return");
        if let Some(ref expr) = expr.ret {
            maybe_wrap!(self, " ", "", expr, fmt_expr);
        }
    }

    #[inline]
    fn fmt_macro_def(&mut self, item: &MacroDef) {
        self.raw_insert(&format!("macro_rules! {}", item.name));
        self.open_brace();
        self.insert_indent();
        self.force_insert(&item.def);
        self.nl();
        self.close_brace();
    }

    #[inline]
    fn fmt_macro_item(&mut self, item: &Macro) {
        self.fmt_macro(item);
        self.raw_insert(";");
    }

    #[inline]
    fn fmt_macro_stmt(&mut self, mac: &MacroStmt) {
        self.try_fmt_leading_comments(&mac.loc);
        self.fmt_attrs(&mac.attrs);
        self.insert_indent();

        self.fmt_macro(&mac.mac);
        if mac.is_semi {
            self.raw_insert(";");
        }

        self.try_fmt_trailing_comment(&mac.loc);
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
                let sep = &mac.seps[i - 1];
                if sep.is_sep {
                    insert_sep!(self, sep.s, expr);
                } else {
                    self.insert(sep.s);
                }
            }
            self.fmt_expr(expr);
        }
        self.insert_unmark_align(close);
    }

    #[inline]
    fn clear_flag(&mut self) {
        self.after_indent = false;
        self.after_wrap = false;
    }

    #[inline]
    fn insert_indent(&mut self) {
        self.ts.insert_indent();
        self.after_indent = true;
    }

    #[inline]
    fn force_insert(&mut self, s: &str) {
        self.ts.force_insert(s);
        self.clear_flag();
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
    fn nl(&mut self) {
        self.ts.nl();
        self.clear_flag();
    }

    #[inline]
    fn wrap(&mut self) {
        if !self.after_indent && !self.after_wrap {
            self.ts.wrap();
            self.after_wrap = true;
        }
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
    fn fmt_vis(&mut self, vis: &Vis) {
        if !vis.is_empty() {
            self.raw_insert(vis);
            self.raw_insert(" ");
        }
    }

    #[inline]
    fn open_brace(&mut self) {
        self.raw_insert(" {");
        self.indent();
        self.nl();
    }

    #[inline]
    fn close_brace(&mut self) {
        self.outdent();
        self.insert_indent();
        self.raw_insert("}");
    }
}
