#[cfg(test)]
mod tests {
    use aetos::ast::*;
    use aetos::parser::Parser;

    #[test]
    fn test_basic_parsing() {
        let code = r#"
            fn main() -> i32 {
                return 42;
            }
        "#;
        
        let mut parser = Parser::new(code);
        let program = parser.parse_program();
        assert!(program.is_ok());
    }

    #[test]
    fn test_function_parsing() {
        let code = r#"
            fn add(a: i32, b: i32) -> i32 {
                return a + b;
            }
        "#;
        
        let mut parser = Parser::new(code);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].name, "add");
    }
}