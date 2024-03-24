use core::fmt;
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

use crate::ast;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeName {
    Variable(String),
    Generic(String, BTreeMap<String, ast::TypeExpression>),
    Temporary(usize),
}

#[derive(Debug, Clone)]
pub struct PrimitiveStruct {
    pub fields: Vec<PrimitiveStructField>,
}

#[derive(Debug, Clone)]
pub struct PrimitiveVariant {
    pub variants: Vec<PrimitiveVariantOption>,
}

#[derive(Debug, Clone)]
pub enum PrimitiveType {
    Reference(usize),
    Builtin(ast::Builtin),
}

#[derive(Debug, Clone)]
pub enum IRType {
    Struct(PrimitiveStruct),
    Variant(PrimitiveVariant),
    Reference(usize),
    Builtin(ast::Builtin),
}

pub enum ResolvedIRType {
    Struct(PrimitiveStruct),
    Variant(PrimitiveVariant),
    Builtin(ast::Builtin),
}

impl Display for TypeName {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TypeName::Variable(name) => write!(f, "{}", name),
            TypeName::Generic(name, args) => {
                write!(f, "{}<", name)?;
                for (i, (k, v)) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}={:?}", k, v)?;
                }
                write!(f, ">")
            }
            TypeName::Temporary(id) => write!(f, "T{}", id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NamedIRType {
    pub name: TypeName,
    pub t: IRType,
}

pub struct NamedPrimitiveType {
    pub name: TypeName,
    pub t: PrimitiveType,
}

#[derive(Debug, Clone)]
pub struct PrimitiveStructField {
    pub name: String,
    pub type_: PrimitiveType,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PrimitiveVariantOption {
    pub name: String,
    pub type_: PrimitiveType,
    pub comment: Option<String>,
}

impl Display for IRType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            IRType::Struct(PrimitiveStruct { fields }) => {
                write!(f, "struct {{")?;
                for PrimitiveStructField {
                    name: k,
                    type_: v,
                    comment,
                } in fields.iter()
                {
                    if let Some(comment) = comment {
                        for line in comment.lines() {
                            write!(f, "\n  // {}", line)?;
                        }
                    }
                    write!(f, "\n  {}: {},", k, v)?;
                }
                write!(f, "\n}}")
            }
            IRType::Variant(PrimitiveVariant { variants }) => {
                write!(f, "enum {{")?;
                for PrimitiveVariantOption {
                    name: k,
                    type_: v,
                    comment,
                } in variants.iter()
                {
                    if let Some(comment) = comment {
                        for line in comment.lines() {
                            write!(f, "\n  // {}", line)?;
                        }
                    }
                    write!(f, "\n  {}({}),", k, v)?;
                }
                write!(f, "\n}}")
            }
            IRType::Reference(id) => write!(f, "reference {}", id),
            IRType::Builtin(builtin) => write!(f, "{}", builtin),
        }
    }
}

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            PrimitiveType::Reference(id) => write!(f, "reference {}", id),
            PrimitiveType::Builtin(builtin) => write!(f, "{}", builtin),
        }
    }
}

impl Into<IRType> for PrimitiveType {
    fn into(self) -> IRType {
        match self {
            PrimitiveType::Reference(id) => IRType::Reference(id),
            PrimitiveType::Builtin(builtin) => IRType::Builtin(builtin),
        }
    }
}

impl Into<IRType> for ResolvedIRType {
    fn into(self) -> IRType {
        match self {
            ResolvedIRType::Struct(fields) => IRType::Struct(fields),
            ResolvedIRType::Variant(variants) => IRType::Variant(variants),
            ResolvedIRType::Builtin(builtin) => IRType::Builtin(builtin),
        }
    }
}
