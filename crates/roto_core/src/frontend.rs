use std::{
    collections::{BTreeMap, HashMap, HashSet},
    vec,
};

use crate::{
    ast,
    ir::{
        IRType, NamedIRType, PrimitiveStruct, PrimitiveStructField, PrimitiveType,
        PrimitiveVariant, PrimitiveVariantOption, ResolvedIRType, TypeName,
    },
};

pub struct TypeAllocator {
    pub types: BTreeMap<usize, NamedIRType>,
    pub named_types: HashMap<ast::TypeExpression, usize>,
}

impl TypeAllocator {
    pub fn new() -> Self {
        TypeAllocator {
            types: BTreeMap::new(),
            named_types: HashMap::new(),
        }
    }

    fn alloc(&mut self, t: &ast::TypeExpression) -> (usize, bool) {
        if let Some(&id) = self.named_types.get(t) {
            (id, false)
        } else {
            let id = self.named_types.len();
            self.named_types.insert(t.clone(), id);
            (id, true)
        }
    }

    fn set(&mut self, id: usize, name: TypeName, t: IRType) {
        self.types.insert(id, NamedIRType { name: name, t });
    }
}

pub struct TypePrototype {
    pub params: Vec<String>,
    pub type_: ast::TypeExpression,
}

impl TypePrototype {
    pub fn unify(&self, args: &BTreeMap<String, ast::TypeExpression>) -> ast::TypeExpression {
        for param in &self.params {
            args.get(param)
                .expect("Type parameter not found in arguments");
        }
        for (k, _v) in args {
            if !self.params.contains(k) {
                panic!("Type parameter {} not found in type prototype", k);
            }
        }
        self.type_.unify(args)
    }
}

pub struct IRCompiler {
    pub allocator: TypeAllocator,
    type_env: HashMap<String, TypePrototype>,
    next_temporary_id: usize,
}

impl IRCompiler {
    pub fn new() -> Self {
        IRCompiler {
            allocator: TypeAllocator::new(),
            type_env: HashMap::new(),
            next_temporary_id: 0,
        }
    }

    pub fn register_global_type(&mut self, name: String, t: TypePrototype) {
        self.type_env.insert(name, t);
    }

    pub fn iter_types(&self) -> impl Iterator<Item = (&usize, &NamedIRType)> {
        self.allocator.types.iter()
    }

    pub fn iter_globals(&self) -> impl Iterator<Item = (&String, &TypePrototype)> {
        self.type_env.iter()
    }

    pub fn resolve_ir_type(&self, t: &IRType) -> ResolvedIRType {
        match t {
            IRType::Reference(id) => {
                let named_type = self.allocator.types.get(id).unwrap();
                self.resolve_ir_type(&named_type.t)
            }
            IRType::Builtin(builtin) => ResolvedIRType::Builtin(builtin.clone()),
            IRType::Struct(fields) => ResolvedIRType::Struct(fields.clone()),
            IRType::Variant(variants) => ResolvedIRType::Variant(variants.clone()),
        }
    }

    pub fn compile_force_allocation(
        &mut self,
        name: TypeName,
        type_var: &ast::TypeExpression,
        t: &ast::TypeExpression,
    ) -> (usize, bool) {
        let (alloc_id, new) = self.allocator.alloc(type_var);
        if new {
            let inner_primitive = self.compile_to_primitive_type(t);
            self.allocator.set(alloc_id, name, inner_primitive.into());
        }
        return (alloc_id, new);
    }

    pub fn eager_emit_temporary(&mut self, t: &ast::TypeExpression, p: IRType) -> (usize, bool) {
        let (alloc_id, new) = self.allocator.alloc(t);
        if new {
            self.allocator
                .set(alloc_id, TypeName::Temporary(self.next_temporary_id), p);
            self.next_temporary_id += 1;
        }
        return (alloc_id, new);
    }

    pub fn compile_global(&mut self, name: String, t: &ast::TypeExpression) -> (usize, bool) {
        let var_expression = ast::TypeExpression::Variable(name.clone());
        self.compile_force_allocation(TypeName::Variable(name.clone()), &var_expression, &t)
    }

    // primitive type, resolved primitive type
    pub fn compile_to_primitive_type(&mut self, t: &ast::TypeExpression) -> PrimitiveType {
        match t {
            ast::TypeExpression::Variable(name) => {
                let inner_type = self
                    .type_env
                    .get(name)
                    .expect(
                        format!("Type variable {} not found in type environment", name).as_str(),
                    )
                    .unify(&BTreeMap::new());

                let (alloc_id, _new) =
                    self.compile_force_allocation(TypeName::Variable(name.clone()), t, &inner_type);
                PrimitiveType::Reference(alloc_id)
            }
            ast::TypeExpression::Builtin(name) => PrimitiveType::Builtin(name.clone()),
            ast::TypeExpression::Generic(name, args) => {
                let inner_type = self
                    .type_env
                    .get(name)
                    .expect(
                        format!("Type variable {} not found in type environment", name).as_str(),
                    )
                    .unify(args);
                let (alloc_id, _new) = self.compile_force_allocation(
                    TypeName::Generic(name.clone(), args.clone()),
                    t,
                    &inner_type,
                );
                PrimitiveType::Reference(alloc_id)
            }
            ast::TypeExpression::Struct(ast::StructTypeExpression { fields }) => {
                let primitive_fields = fields
                    .iter()
                    .map(|v| PrimitiveStructField {
                        name: v.name.clone(),
                        type_: self.compile_to_primitive_type(&v.type_),
                        comment: v.comment.clone(),
                    })
                    .collect();
                let (alloc_id, _new) = self.eager_emit_temporary(
                    t,
                    IRType::Struct(PrimitiveStruct {
                        fields: primitive_fields,
                    }),
                );
                PrimitiveType::Reference(alloc_id)
            }
            ast::TypeExpression::Variant(ast::VariantTypeExpression { variants }) => {
                let primitive_variants = variants
                    .iter()
                    .map(|v| PrimitiveVariantOption {
                        name: v.name.clone(),
                        type_: self.compile_to_primitive_type(&v.type_),
                        comment: v.comment.clone(),
                    })
                    .collect();
                let (alloc_id, _new) = self.eager_emit_temporary(
                    t,
                    IRType::Variant(PrimitiveVariant {
                        variants: primitive_variants,
                    }),
                );
                PrimitiveType::Reference(alloc_id)
            }
            ast::TypeExpression::Intersection(a, b) => {
                let ax = self.compile_to_primitive_type(&a);
                let a = self.resolve_ir_type(&ax.into());
                let bx = self.compile_to_primitive_type(&b);
                let b = self.resolve_ir_type(&bx.into());
                match (a, b) {
                    (
                        ResolvedIRType::Struct(PrimitiveStruct { fields: a }),
                        ResolvedIRType::Struct(PrimitiveStruct { fields: b }),
                    ) => {
                        let mut fields: Vec<PrimitiveStructField> = vec![];

                        let b_set: HashSet<String> = b.iter().map(|f| f.name.clone()).collect();

                        for f in a {
                            if b_set.contains(&f.name) {
                                panic!("Intersection of structs with overlapping fields");
                            }
                            fields.push(f);
                        }
                        for f in b {
                            fields.push(f);
                        }
                        let (alloc_id, _new) = self
                            .eager_emit_temporary(t, IRType::Struct(PrimitiveStruct { fields }));
                        PrimitiveType::Reference(alloc_id)
                    }
                    (
                        ResolvedIRType::Variant(PrimitiveVariant { variants: a }),
                        ResolvedIRType::Variant(PrimitiveVariant { variants: b }),
                    ) => {
                        let mut variants: Vec<PrimitiveVariantOption> = vec![];

                        let b_set: HashSet<String> = b.iter().map(|f| f.name.clone()).collect();

                        for f in a {
                            if b_set.contains(&f.name) {
                                panic!("Intersection of variants with overlapping fields");
                            }
                            variants.push(f);
                        }

                        for f in b {
                            variants.push(f);
                        }

                        let (alloc_id, _new) = self.eager_emit_temporary(
                            t,
                            IRType::Variant(PrimitiveVariant { variants }),
                        );
                        PrimitiveType::Reference(alloc_id)
                    }
                    _ => panic!("Intersection of non-structs"),
                }
            }
        }
    }
}
