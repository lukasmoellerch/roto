use core::fmt;
use std::{
    collections::{BTreeMap, HashSet},
    fmt::{Display, Formatter},
};

use crate::ast;

pub trait Intersectable<A, B> {
    fn intersect(&self, other: &B) -> A;
}

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

impl PrimitiveStruct {
    pub fn new() -> Self {
        PrimitiveStruct { fields: Vec::new() }
    }
    pub fn add_field(&mut self, name: String, type_: PrimitiveType, comment: Option<String>) {
        self.fields.push(PrimitiveStructField {
            name,
            type_,
            comment,
        });
    }
}

impl Intersectable<PrimitiveStruct, PrimitiveStruct> for PrimitiveStruct {
    fn intersect(&self, other: &PrimitiveStruct) -> PrimitiveStruct {
        let mut out = PrimitiveStruct::new();
        let b_fields_names = other
            .fields
            .iter()
            .map(|f| f.name.clone())
            .collect::<HashSet<_>>();
        
        for f in &self.fields {
            if b_fields_names.contains(&f.name) {
                panic!("Intersection of structs with overlapping fields");
            }

            out.fields.push(f.clone());
        }

        for f in &other.fields {
            out.fields.push(f.clone());
        }

        out

    }
}

#[derive(Debug, Clone)]
pub struct PrimitiveVariant {
    pub variants: Vec<PrimitiveVariantOption>,
}


impl Intersectable<PrimitiveVariant, PrimitiveVariant> for PrimitiveVariant {
    fn intersect(&self, other: &PrimitiveVariant) -> PrimitiveVariant {
        let mut out = PrimitiveVariant::new();
        let b_variants_names = other
            .variants
            .iter()
            .map(|v| v.name.clone())
            .collect::<HashSet<_>>();

        for v in &self.variants {
            if b_variants_names.contains(&v.name) {
                panic!("Intersection of variants with overlapping fields");
            }

            out.variants.push(v.clone());
        }

        for v in &other.variants {
            out.variants.push(v.clone());
        }

        out
    }
}


impl PrimitiveVariant {
    pub fn new() -> Self {
        PrimitiveVariant {
            variants: Vec::new(),
        }
    }
    pub fn add_variant(&mut self, name: String, type_: PrimitiveType, comment: Option<String>) {
        self.variants.push(PrimitiveVariantOption {
            name,
            type_,
            comment,
        });
    }
}

/// IRType is the most generate type of type - it can represent any type that can be used in the
/// IR. This includes structs, variants, references, and builtins.
#[derive(Debug, Clone)]
pub enum IRType {
    Struct(PrimitiveStruct),
    Variant(PrimitiveVariant),
    Reference(usize),
    Builtin(ast::Builtin),
}

/// A primitive type is a type that is "constant" in size, i.e. it does not have any direct
/// nesting, but can reference other types. This is used to represent the type of values in the
/// ir.
#[derive(Debug, Clone)]
pub enum PrimitiveType {
    Reference(usize),
    Builtin(ast::Builtin),
}

/// A resolved IR type is a type that has been resolved to a specific type. There are no direct references
/// to other types.
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
