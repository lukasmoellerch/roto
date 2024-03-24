use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

use roto_core::ast;
use roto_core::frontend::TypeAllocator;
use roto_core::ir::IRType;
use roto_core::ir::NamedIRType;
use roto_core::ir::PrimitiveType;
use roto_core::ir::TypeName;

pub struct TypeNameAllocator {
    next_id: usize,
    names: HashMap<TypeName, usize>,
}

impl TypeNameAllocator {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            names: HashMap::new(),
        }
    }

    pub fn allocate_name(&mut self, type_name: &TypeName) -> String {
        match type_name {
            TypeName::Variable(name) => name.clone(),
            TypeName::Generic(name, params) => {
                let existing = self
                    .names
                    .get(&TypeName::Generic(name.clone(), params.clone()));
                match existing {
                    Some(id) => format!("{}{}", name, id),
                    None => {
                        let id = self.next_id;
                        self.next_id += 1;
                        self.names
                            .insert(TypeName::Generic(name.clone(), params.clone()), id);
                        format!("{}{}", name, id)
                    }
                }
            }
            TypeName::Temporary(id) => format!("T{}", id),
        }
    }
}

pub struct PrimitiveTypeWriter<'a> {
    pub name_allocator: &'a mut TypeNameAllocator,
    pub allocator: &'a TypeAllocator,
    //
    pub compiled: HashSet<TypeName>,
    pub stack: VecDeque<NamedIRType>,
}

impl<'a> PrimitiveTypeWriter<'a> {
    pub fn allocate_name(&mut self, type_name: &TypeName) -> String {
        self.name_allocator.allocate_name(type_name)
    }

    fn convert_builtin(&self, t: &ast::Builtin) -> String {
        match t {
            ast::Builtin::Int => "int".to_string(),
            ast::Builtin::Float => "float".to_string(),
            ast::Builtin::String => "str".to_string(),
            ast::Builtin::Bool => "bool".to_string(),
            ast::Builtin::Unit => "None".to_string(),
        }
    }

    fn convert_primitive_type(&mut self, t: &PrimitiveType) -> String {
        match t {
            PrimitiveType::Builtin(builtin) => self.convert_builtin(builtin),
            PrimitiveType::Reference(name) => {
                let r = self.allocator.types.get(name).unwrap();
                if !self.compiled.contains(&r.name) {
                    self.stack.push_front(r.clone());
                }
                self.allocate_name(&r.name)
            }
        }
    }

    pub fn convert_named_ir_type(&mut self, name: &str, t: &IRType) -> String {
        match t {
            IRType::Struct(struct_type) => {
                let mut result = "class ".to_string();
                result.push_str(name);
                result.push_str("(msgspec.Struct):\n");
                for field in struct_type.fields.iter() {
                    if let Some(comment) = &field.comment {
                        result.push_str(&format!("    # {}\n", comment));
                    }
                    result.push_str(&format!(
                        "    {}: {}\n",
                        field.name,
                        self.convert_primitive_type(&field.type_)
                    ));
                }
                result
            }
            IRType::Reference(reference) => {
                let rhs = self.allocator.types.get(reference).unwrap();
                let rhs_name = self.allocate_name(&rhs.name);
                format!("{}: TypeAlias = {}\n", name, rhs_name)
            }
            _ => {
                let rhs = self.convert_primitive_type(&PrimitiveType::Builtin(ast::Builtin::Unit));
                format!("{}: TypeAlias = {}\n", name, rhs)
            }
        }
    }
}
