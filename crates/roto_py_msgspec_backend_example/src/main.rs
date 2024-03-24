use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::process;

use roto_core::ast;
use roto_core::frontend::IRCompiler;
use roto_core::frontend::TypeAllocator;
use roto_core::frontend::TypePrototype;
use roto_core::ir::IRType;
use roto_core::ir::NamedIRType;
use roto_core::ir::PrimitiveType;
use roto_core::ir::TypeName;
use roto_core::parser;

struct TypeNameAllocator {
    next_id: usize,
    names: HashMap<TypeName, usize>,
}

impl TypeNameAllocator {
    fn new() -> Self {
        Self {
            next_id: 0,
            names: HashMap::new(),
        }
    }

    fn allocate_name(&mut self, type_name: &TypeName) -> String {
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

struct PrimitiveTypeWriter<'a> {
    name_allocator: &'a mut TypeNameAllocator,
    allocator: &'a TypeAllocator,
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

    fn convert_named_primitive(&mut self, name: &str, t: &IRType) -> String {
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

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        process::exit(1);
    }

    let file_path = &args[1];
    let file_contents = fs::read_to_string(file_path).expect("Failed to read file");

    let parsed = parser::ProgramParser::new()
        .parse(&file_contents)
        .expect("Failed to parse content");

    let mut compiler = IRCompiler::new();
    for decl in parsed {
        compiler.register_global_type(
            decl.name,
            TypePrototype {
                params: decl.params,
                type_: decl.type_,
            },
        );
    }

    let globals = compiler
        .iter_globals()
        .filter(|(_, t)| t.params.is_empty())
        .map(|(name, t)| (name.clone(), t.type_.clone()))
        .collect::<Vec<_>>();
    for (name, expr) in globals {
        compiler.compile_global(name.clone(), &expr);
    }

    let mut type_name_allocator = TypeNameAllocator::new();
    let mut primitive_type_writer = PrimitiveTypeWriter {
        name_allocator: &mut type_name_allocator,
        allocator: &compiler.allocator,
        compiled: HashSet::new(),
        stack: VecDeque::new(),
    };

    for (_i, NamedIRType { name, t }) in compiler.iter_types() {
        match name {
            TypeName::Variable(_) => {
                primitive_type_writer.stack.push_back(NamedIRType {
                    name: name.clone(),
                    t: t.clone(),
                });
            }
            _ => {}
        }
    }

    while !primitive_type_writer.stack.is_empty() {
        let NamedIRType { name, t } = primitive_type_writer.stack.pop_front().unwrap();
        if primitive_type_writer.compiled.contains(&name) {
            continue;
        }
        primitive_type_writer.compiled.insert(name.clone());

        let py_name = primitive_type_writer.allocate_name(&name);
        let q = compiler.resolve_ir_type(&t);
        let py_type = primitive_type_writer.convert_named_primitive(&py_name, &q.into());
        println!("{}", py_type);
    }
}
