use crate::ast::Program;
use crate::parser::Parser;
use crate::typecheck::TypeChecker;
use crate::interpreter::Interpreter;
use std::error::Error;
use std::io::{self, Write};
use std::fs;

const HELP_TEXT: &str = r#"
Aetos Interactive Environment (v0.3.0)
Type '.help' for help, '.exit' to quit

Available commands:
  .help           - Show this help
  .exit           - Exit the REPL
  .clear          - Clear the screen
  .run <file>     - Run an Aetos file
  .vars           - Show all variables
  .reset          - Reset the environment
  .ast            - Show AST of last parsed code
  .parse <code>   - Parse code and show AST
  .history        - Show command history
  .load <file>    - Load and display file content
"#;

pub struct AetosIDE {
    interpreter: Interpreter,
    last_program: Option<Program>,
    variables: Vec<(String, String)>,
}

impl AetosIDE {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
            last_program: None,
            variables: Vec::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Welcome to Aetos Interactive Development Environment!");
        println!("Type '.help' for help, '.exit' to quit\n");
        
        let mut history = Vec::new();
        
        loop {
            // Show prompt
            print!("aetos> ");
            io::stdout().flush()?;
            
            // Read input
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();
            
            // Save to history
            if !input.is_empty() {
                history.push(input.to_string());
                if history.len() > 100 {
                    history.remove(0);
                }
            }
            
            if input.is_empty() {
                continue;
            }
            
            // Handle commands
            if input.starts_with('.') {
                match self.handle_command(input, &history) {
                    Ok(should_continue) => {
                        if !should_continue {
                            break;
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
                continue;
            }
            
            // Try to evaluate as Aetos code
            match self.evaluate_input(input) {
                Ok(result) => {
                    if !result.is_empty() {
                        println!("{}", result);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
        
        println!("Goodbye!");
        Ok(())
    }

    fn handle_command(&mut self, cmd: &str, history: &[String]) -> Result<bool, Box<dyn Error>> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        
        match parts[0] {
            ".help" => {
                println!("{}", HELP_TEXT);
                Ok(true)
            }
            ".exit" => Ok(false),
            ".clear" => {
                print!("\x1B[2J\x1B[1;1H");
                Ok(true)
            }
            ".vars" => {
                if self.variables.is_empty() {
                    println!("No variables defined.");
                } else {
                    println!("Variables:");
                    for (name, value) in &self.variables {
                        println!("  {} = {}", name, value);
                    }
                }
                Ok(true)
            }
            ".reset" => {
                self.interpreter = Interpreter::new();
                self.variables.clear();
                println!("Environment reset.");
                Ok(true)
            }
            ".history" => {
                println!("Command history (last {}):", history.len());
                for (i, cmd) in history.iter().enumerate() {
                    println!("  {}: {}", i + 1, cmd);
                }
                Ok(true)
            }
            ".ast" => {
                if let Some(program) = &self.last_program {
                    println!("Last parsed program:");
                    println!("  Functions: {}", program.functions.len());
                    println!("  Structs: {}", program.structs.len());
                    for func in &program.functions {
                        println!("  Function: {}", func.name);
                    }
                } else {
                    println!("No program parsed yet.");
                }
                Ok(true)
            }
            ".parse" if parts.len() > 1 => {
                let code = parts[1..].join(" ");
                match Parser::new(&code).parse_program() {
                    Ok(program) => {
                        self.last_program = Some(program.clone());
                        println!("Successfully parsed.");
                        println!("AST contains {} function(s).", program.functions.len());
                    }
                    Err(e) => eprintln!("Parse error: {}", e),
                }
                Ok(true)
            }
            ".run" if parts.len() > 1 => {
                let filename = parts[1];
                match self.run_file(filename) {
                    Ok(msg) => println!("{}", msg),
                    Err(e) => eprintln!("Error: {}", e),
                }
                Ok(true)
            }
            ".load" if parts.len() > 1 => {
                let filename = parts[1];
                match fs::read_to_string(filename) {
                    Ok(content) => {
                        println!("Loaded {} ({} bytes)", filename, content.len());
                        // Show first few lines
                        let lines: Vec<&str> = content.lines().take(5).collect();
                        for line in lines {
                            println!("  {}", line);
                        }
                        if content.lines().count() > 5 {
                            println!("  ... ({} more lines)", content.lines().count() - 5);
                        }
                    }
                    Err(e) => eprintln!("Error loading file: {}", e),
                }
                Ok(true)
            }
            _ => {
                println!("Unknown command: {}", cmd);
                println!("Type '.help' for available commands.");
                Ok(true)
            }
        }
    }

    fn evaluate_input(&mut self, input: &str) -> Result<String, Box<dyn Error>> {
        // Try to wrap input in a function if it looks like an expression
        let wrapped = if input.contains(';') || input.contains('{') {
            // Already looks like a statement/block
            format!("fn __repl_eval() -> i32 {{ {} ; 0 }}", input)
        } else {
            // Treat as expression
            format!("fn __repl_eval() -> i32 {{ return {}; }}", input)
        };
        
        let mut parser = Parser::new(&wrapped);
        let program = parser.parse_program()?;
        
        let mut type_checker = TypeChecker::new();
        type_checker.check_program(&program)?;
        
        self.last_program = Some(program.clone());
        
        // Extract the function name from the parsed program
        if let Some(func) = program.functions.first() {
            if func.name == "__repl_eval" {
                return Ok("✓ Valid Aetos code".to_string());
            }
        }
        
        Ok("✓ Parsed successfully".to_string())
    }

    fn run_file(&self, filename: &str) -> Result<String, Box<dyn Error>> {
        let source = fs::read_to_string(filename)?;
        let mut parser = Parser::new(&source);
        let program = parser.parse_program()?;
        
        let mut type_checker = TypeChecker::new();
        type_checker.check_program(&program)?;
        
        Ok(format!("✓ Successfully parsed {} ({} functions)", 
                  filename, program.functions.len()))
    }
}

pub fn run_ide() -> Result<(), Box<dyn Error>> {
    let mut ide = AetosIDE::new();
    ide.run()
}