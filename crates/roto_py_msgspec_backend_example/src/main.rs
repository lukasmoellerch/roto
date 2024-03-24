use std::collections::HashSet;
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::process;

use roto_core::frontend::IRCompiler;
use roto_core::frontend::TypePrototype;
use roto_core::ir::NamedIRType;
use roto_core::ir::TypeName;
use roto_core::parser;
use roto_py_msgspec_backend::PrimitiveTypeWriter;
use roto_py_msgspec_backend::TypeNameAllocator;

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

    for (_i, NamedIRType { name, type_: t }) in compiler.iter_types() {
        match name {
            TypeName::Variable(_) => {
                primitive_type_writer.stack.push_back(NamedIRType {
                    name: name.clone(),
                    type_: t.clone(),
                });
            }
            _ => {}
        }
    }

    while !primitive_type_writer.stack.is_empty() {
        let NamedIRType { name, type_: t } = primitive_type_writer.stack.pop_front().unwrap();
        if primitive_type_writer.compiled.contains(&name) {
            continue;
        }
        primitive_type_writer.compiled.insert(name.clone());

        let py_name = primitive_type_writer.allocate_name(&name);
        let q = compiler.resolve_ir_type(&t);
        let py_type = primitive_type_writer.convert_named_ir_type(&py_name, &q.into());
        println!("{}", py_type);
    }
}
