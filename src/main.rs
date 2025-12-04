use clap::{Arg, Command};
use std::fs;
use std::path::Path;

mod ast;
mod lexer;
mod parser;
mod typecheck;
mod codegen;
mod stdlib;
mod optimize;
mod graphics_engine;
mod interpreter;
mod ide;

use interpreter::Interpreter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("aetosc")
        .version("0.3.0")
        .about("Aetos Language Compiler")
        .subcommand(
            Command::new("graphics")
                .about("Run graphics program in native window")
                .arg(
                    Arg::new("input")
                        .required(true)
                        .help("Input source file"),
                )
                .arg(
                    Arg::new("width")
                        .long("width")
                        .short('W')
                        .default_value("800")
                        .help("Window width"),
                )
                .arg(
                    Arg::new("height")
                        .long("height")
                        .short('H')
                        .default_value("600")
                        .help("Window height"),
                )
        )
        .subcommand(
            Command::new("run")
                .about("Run Aetos program")
                .arg(
                    Arg::new("input")
                        .required(true)
                        .help("Input source file"),
                )
        )
        .subcommand(
            Command::new("compile")
                .about("Compile Aetos program")
                .arg(
                    Arg::new("input")
                        .required(true)
                        .help("Input source file"),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .help("Output file"),
                )
        )
        .subcommand(
            Command::new("ide")
                .about("Start interactive development environment")
        )
        .subcommand(
            Command::new("check")
                .about("Check syntax and types without running")
                .arg(
                    Arg::new("input")
                        .required(true)
                        .help("Input source file"),
                )
        )
        .get_matches();

    match matches.subcommand() {
        Some(("graphics", sub_matches)) => {
            let input_file = sub_matches.get_one::<String>("input").unwrap();
            let width = sub_matches.get_one::<String>("width").unwrap().parse::<usize>()?;
            let height = sub_matches.get_one::<String>("height").unwrap().parse::<usize>()?;
            
            run_aetos_program(input_file, width, height)
        }
        Some(("run", sub_matches)) => {
            let input_file = sub_matches.get_one::<String>("input").unwrap();
            run_aetos_program(input_file, 800, 600)
        }
        Some(("compile", sub_matches)) => {
            let input_file = sub_matches.get_one::<String>("input").unwrap();
            compile_aetos_program(input_file, sub_matches.get_one::<String>("output"))
        }
        Some(("check", sub_matches)) => {
            let input_file = sub_matches.get_one::<String>("input").unwrap();
            check_aetos_program(input_file)
        }
        Some(("ide", _)) => {
            println!("Starting Aetos Interactive Development Environment...\n");
            ide::run_ide()
        }
        _ => {
            show_help();
            Ok(())
        }
    }
}

fn run_aetos_program(input_file: &str, width: usize, height: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running Aetos program: {}", input_file);
    
    let source_code = fs::read_to_string(input_file)?;
    
    // Парсим программу
    let mut parser = parser::Parser::new(&source_code);
    let program = parser.parse_program()?;
    
    println!("Parsed {} functions and {} structs", program.functions.len(), program.structs.len());
    
    // Проверяем типы
    let mut type_checker = typecheck::TypeChecker::new();
    type_checker.check_program(&program)?;
    println!("Type checking passed!");
    
    // Применяем оптимизации
    let optimizer = optimize::Optimizer::default();
    let mut optimized_program = program;
    optimizer.optimize(&mut optimized_program);
    
    // Запускаем интерпретатор
    let mut interpreter = Interpreter::new();
    
    // Извлекаем имя файла для заголовка окна
    let title = input_file
        .split('/')
        .last()
        .unwrap_or("AetOS Program")
        .replace(".aetos", "");
    
    match interpreter.interpret_program(&optimized_program, width, height, &title) {
        Ok(_) => println!("Program finished successfully"),
        Err(e) => eprintln!("Runtime error: {}", e),
    }
    
    Ok(())
}

fn compile_aetos_program(input_file: &str, output_file: Option<&String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Compiling Aetos program: {}", input_file);
    
    let source_code = fs::read_to_string(input_file)?;
    
    // Парсим программу
    let mut parser = parser::Parser::new(&source_code);
    let program = parser.parse_program()?;
    
    println!("Parsed {} functions and {} structs", program.functions.len(), program.structs.len());
    
    // Проверяем типы
    let mut type_checker = typecheck::TypeChecker::new();
    type_checker.check_program(&program)?;
    println!("Type checking passed!");
    
    // Определяем выходной файл
    let output_path = if let Some(output) = output_file {
        output.clone()
    } else {
        let input_path = Path::new(input_file);
        let mut output = input_path.with_extension("wasm").to_string_lossy().to_string();
        if output == input_file {
            output = format!("{}.wasm", input_file);
        }
        output
    };
    
    // Компилируем в WASM
    println!("Compiling to WASM: {}", output_path);
    
    // TODO: Реализовать компиляцию в WASM
    println!("WASM compilation not yet implemented");
    
    Ok(())
}

fn check_aetos_program(input_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking Aetos program: {}", input_file);
    
    let source_code = fs::read_to_string(input_file)?;
    
    // Парсим программу
    let mut parser = parser::Parser::new(&source_code);
    let program = parser.parse_program()?;
    
    println!("✓ Parsed {} functions and {} structs", program.functions.len(), program.structs.len());
    
    // Проверяем типы
    let mut type_checker = typecheck::TypeChecker::new();
    type_checker.check_program(&program)?;
    println!("✓ Type checking passed!");
    
    // Проверяем оптимизации
    let optimizer = optimize::Optimizer::default();
    let mut optimized_program = program.clone();
    optimizer.optimize(&mut optimized_program);
    
    if program.functions.len() != optimized_program.functions.len() {
        println!("⚠  Optimization may have removed some code");
    }
    
    println!("✓ Program is valid Aetos code");
    
    Ok(())
}

fn show_help() {
    println!("Aetos Language Compiler v0.3.0");
    println!();
    println!("Usage:");
    println!("  aetosc graphics <file.aetos>    - Run graphics program");
    println!("  aetosc run <file.aetos>         - Run console program");
    println!("  aetosc compile <file.aetos>     - Compile to WASM");
    println!("  aetosc check <file.aetos>       - Check syntax and types");
    println!("  aetosc ide                      - Start interactive IDE");
    println!("  aetosc help                     - Show this help");
    println!();
    println!("Examples:");
    println!("  aetosc run examples/hello.aetos");
    println!("  aetosc graphics examples/graphics_demo.aetos");
    println!("  aetosc ide");
}