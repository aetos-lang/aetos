use aetos::parser::Parser;
use aetos::typecheck::TypeChecker;
use aetos::interpreter::Interpreter;
use rustyline::error::ReadlineError;
use rustyline::{Editor, Config};
use std::error::Error;

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
"#;

struct AetosREPL {
    interpreter: Interpreter,
    variables: Vec<(String, String)>,
    history_file: String,
}

impl AetosREPL {
    fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
            variables: Vec::new(),
            history_file: format!("{}/.aetos_history", std::env::var("HOME").unwrap_or(".".to_string())),
        }
    }

    fn eval(&mut self, input: &str) -> Result<String, Box<dyn Error>> {
        let trimmed = input.trim();
        
        // Handle commands
        if trimmed.starts_with('.') {
            return self.handle_command(trimmed);
        }
        
        // Try to parse as expression
        if let Ok(expr) = self.parse_expression(trimmed) {
            return self.eval_expression(&expr);
        }
        
        // Try to parse as statement
        if let Ok(stmt) = self.parse_statement(trimmed) {
            return self.eval_statement(&stmt);
        }
        
        Err("Invalid syntax. Use '.help' for help.".into())
    }

    fn handle_command(&mut self, cmd: &str) -> Result<String, Box<dyn Error>> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        
        match parts[0] {
            ".help" => Ok(HELP_TEXT.to_string()),
            ".exit" => {
                println!("Goodbye!");
                std::process::exit(0);
            }
            ".clear" => {
                print!("\x1B[2J\x1B[1;1H");
                Ok("".to_string())
            }
            ".vars" => {
                let mut output = String::new();
                output.push_str("Variables:\n");
                for (name, value) in &self.variables {
                    output.push_str(&format!("  {} = {}\n", name, value));
                }
                Ok(output)
            }
            ".reset" => {
                self.interpreter = Interpreter::new();
                self.variables.clear();
                Ok("Environment reset".to_string())
            }
            ".run" if parts.len() > 1 => {
                let filename = parts[1];
                self.run_file(filename)
            }
            _ => Err(format!("Unknown command: {}", cmd).into())
        }
    }

    fn parse_expression(&self, input: &str) -> Result<aetos::ast::Expression, Box<dyn Error>> {
        let mut parser = Parser::new(input);
        // Note: We need to add expression parsing to the parser
        Err("Expression parsing not implemented yet".into())
    }

    fn parse_statement(&self, input: &str) -> Result<aetos::ast::Statement, Box<dyn Error>> {
        let mut parser = Parser::new(input);
        // Wrap input in a function for parsing
        let wrapped = format!("fn __repl_fn() -> i32 {{ {} ; 0 }}", input);
        let mut parser = Parser::new(&wrapped);
        let program = parser.parse_program()?;
        
        if let Some(func) = program.functions.first() {
            if let Some(stmt) = func.body.first() {
                return Ok(stmt.clone());
            }
        }
        
        Err("Could not parse statement".into())
    }

    fn eval_expression(&mut self, _expr: &aetos::ast::Expression) -> Result<String, Box<dyn Error>> {
        // TODO: Implement expression evaluation
        Ok("Expression evaluation not implemented yet".to_string())
    }

    fn eval_statement(&mut self, _stmt: &aetos::ast::Statement) -> Result<String, Box<dyn Error>> {
        // TODO: Implement statement evaluation
        Ok("Statement evaluation not implemented yet".to_string())
    }

    fn run_file(&self, filename: &str) -> Result<String, Box<dyn Error>> {
        let source = std::fs::read_to_string(filename)?;
        let mut parser = Parser::new(&source);
        let program = parser.parse_program()?;
        
        let mut type_checker = TypeChecker::new();
        type_checker.check_program(&program)?;
        
        // TODO: Run the program
        Ok(format!("Successfully parsed {}", filename))
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Welcome to Aetos Interactive Environment!");
        println!("Type '.help' for help, '.exit' to quit\n");
        
        let config = Config::builder()
            .history_ignore_space(true)
            .build();
            
        let mut rl = Editor::<()>::with_config(config)?;
        
        if rl.load_history(&self.history_file).is_err() {
            println!("No previous history found.");
        }
        
        loop {
            let readline = rl.readline("aetos> ");
            match readline {
                Ok(line) => {
                    rl.add_history_entry(&line)?;
                    
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    match self.eval(&line) {
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
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }
        
        rl.save_history(&self.history_file)?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut repl = AetosREPL::new();
    repl.run()
}