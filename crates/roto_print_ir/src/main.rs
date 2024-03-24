use std::env;
use std::fs;
use std::process;

use roto_core::frontend::IRCompiler;
use roto_core::frontend::TypePrototype;
use roto_core::ir::NamedIRType;
use roto_core::parser;

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

    for (i, NamedIRType { name, t }) in compiler.iter_types() {
        println!("type {}#{} = {}", name, i, t);
    }


}
