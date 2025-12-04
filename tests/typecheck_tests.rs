#[cfg(test)]
mod tests {
    use aetos::parser::Parser;
    use aetos::typecheck::TypeChecker;

    fn parse_and_check(code: &str) -> Result<(), aetos::typecheck::TypeCheckError> {
        let mut parser = Parser::new(code);
        let program = parser.parse_program().unwrap();
        let mut checker = TypeChecker::new(); // Добавили mut
        checker.check_program(&program)
    }

    #[test]
    fn test_basic_type_checking() {
        let code = r#"
            fn main() -> i32 {
                let x: i32 = 5;
                let y: i32 = 3;
                return x + y;
            }
        "#;
        
        assert!(parse_and_check(code).is_ok());
    }

    #[test]
    fn test_type_mismatch() {
        let code = r#"
            fn main() -> i32 {
                let x: i32 = 5;
                let y: bool = true;
                return x + y;
            }
        "#;
        
        assert!(parse_and_check(code).is_err());
    }

    #[test]
    fn test_undefined_variable() {
        let code = r#"
            fn main() -> i32 {
                return x;
            }
        "#;
        
        assert!(parse_and_check(code).is_err());
    }

    #[test]
    fn test_function_call_validation() {
        let code = r#"
            fn add(a: i32, b: i32) -> i32 {
                return a + b;
            }
            
            fn main() -> i32 {
                return add(5);
            }
        "#;
        
        assert!(parse_and_check(code).is_err());
    }
}