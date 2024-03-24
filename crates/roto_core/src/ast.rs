use core::fmt;
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Annotation {
    pub name: String,
    pub args: Vec<(String, String)>
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StructField {
    pub name: String,
    pub type_: TypeExpression,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StructTypeExpression {
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VariantOption {
    pub name: String,
    pub type_: TypeExpression,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VariantTypeExpression {
    pub variants: Vec<VariantOption>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Builtin {
    Int,
    Float,
    String,
    Bool,
    Unit,
}

impl Display for Builtin {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Builtin::Int => write!(f, "int"),
            Builtin::Float => write!(f, "float"),
            Builtin::String => write!(f, "string"),
            Builtin::Bool => write!(f, "bool"),
            Builtin::Unit => write!(f, "unit"),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum TypeExpression {
    Variable(String),
    Builtin(Builtin),
    Generic(String, BTreeMap<String, TypeExpression>),
    Struct(StructTypeExpression),
    Variant(VariantTypeExpression),
    Intersection(Box<TypeExpression>, Box<TypeExpression>),
}

impl StructField {
    pub fn unify(&self, mapping: &BTreeMap<String, TypeExpression>) -> StructField {
        StructField {
            name: self.name.clone(),
            type_: self.type_.unify(mapping),
            comment: self.comment.clone(),
        }
    }
}

impl VariantOption {
    pub fn unify(&self, mapping: &BTreeMap<String, TypeExpression>) -> VariantOption {
        VariantOption {
            name: self.name.clone(),
            type_: self.type_.unify(mapping),
            comment: self.comment.clone(),
        }
    }
}

impl TypeExpression {
    pub fn unify(&self, mapping: &BTreeMap<String, TypeExpression>) -> TypeExpression {
        match self {
            TypeExpression::Variable(name) => mapping
                .get(name)
                .cloned()
                .unwrap_or_else(|| TypeExpression::Variable(name.clone())),
            TypeExpression::Builtin(_) => self.clone(),
            TypeExpression::Generic(name, args) => TypeExpression::Generic(
                name.clone(),
                args.iter()
                    .map(|(k, v)| (k.clone(), v.unify(mapping)))
                    .collect(),
            ),
            TypeExpression::Struct(struct_type) => TypeExpression::Struct(StructTypeExpression {
                fields: struct_type
                    .fields
                    .iter()
                    .map(|v| v.unify(mapping))
                    .collect(),
            }),
            TypeExpression::Variant(variant_type) => {
                TypeExpression::Variant(VariantTypeExpression {
                    variants: variant_type
                        .variants
                        .iter()
                        .map(|v| v.unify(mapping))
                        .collect(),
                })
            }
            TypeExpression::Intersection(a, b) => {
                TypeExpression::Intersection(Box::new(a.unify(mapping)), Box::new(b.unify(mapping)))
            }
        }
    }
}

#[derive(Debug)]
pub struct TypeAliasDeclaration {
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub params: Vec<String>,
    pub type_: TypeExpression,
}
