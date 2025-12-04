#[cfg(test)]
mod tests {
    use aetos::parser::Parser;
    use aetos::optimize::Optimizer;

    fn parse_and_optimize(code: &str) -> aetos::ast::Program {
        let mut parser = Parser::new(code);
        let mut program = parser.parse_program().unwrap();
        
        let optimizer = Optimizer::default();
        optimizer.optimize(&mut program);
        
        program
    }

    #[test]
    fn test_constant_folding() {
        let code = r#"
            fn main() -> i32 {
                let x: i32 = 5 + 3 * 2;
                return x;
            }
        "#;
        
        let program = parse_and_optimize(code);
        let main_fn = program.functions.iter().find(|f| f.name == "main").unwrap();
        
        // После оптимизации должно остаться объявление переменной и return
        // Но переменная x будет иметь значение 11 вместо выражения 5 + 3 * 2
        assert_eq!(main_fn.body.len(), 2); // Исправлено с 1 на 2
        
        // Проверяем что первое statement - объявление переменной с константой
        if let aetos::ast::Statement::VariableDeclaration { value, .. } = &main_fn.body[0] {
            if let aetos::ast::Expression::IntegerLiteral(result) = value {
                assert_eq!(*result, 11); // 5 + (3 * 2) = 11
            } else {
                panic!("Expected integer literal in variable declaration after optimization");
            }
        } else {
            panic!("Expected variable declaration after optimization");
        }
        
        // Проверяем что второе statement - return переменной
        if let aetos::ast::Statement::Return { value } = &main_fn.body[1] {
            if let aetos::ast::Expression::Variable(name) = value {
                assert_eq!(name, "x");
            } else {
                panic!("Expected variable in return statement after optimization");
            }
        } else {
            panic!("Expected return statement after optimization");
        }
    }

    #[test]
    fn test_dead_code_elimination() {
        let code = r#"
            fn main() -> i32 {
                let unused: i32 = 42;
                let used: i32 = 10;
                return used;
            }
        "#;
        
        let program = parse_and_optimize(code);
        let main_fn = program.functions.iter().find(|f| f.name == "main").unwrap();
        
        // Должна остаться только used переменная и return
        let var_declarations: Vec<_> = main_fn.body.iter()
            .filter(|stmt| matches!(stmt, aetos::ast::Statement::VariableDeclaration { .. }))
            .collect();
            
        assert_eq!(var_declarations.len(), 1);
        
        if let aetos::ast::Statement::VariableDeclaration { name, .. } = &var_declarations[0] {
            assert_eq!(name, "used");
        }
    }

    #[test]
    fn test_binary_constant_folding() {
        let code = r#"
            fn main() -> i32 {
                return 2 + 3 * 4;
            }
        "#;
        
        let program = parse_and_optimize(code);
        let main_fn = program.functions.iter().find(|f| f.name == "main").unwrap();
        
        // После оптимизации должен остаться только return с константой
        assert_eq!(main_fn.body.len(), 1);
        
        if let aetos::ast::Statement::Return { value: aetos::ast::Expression::IntegerLiteral(result) } = &main_fn.body[0] {
            assert_eq!(*result, 14); // 2 + (3 * 4) = 14
        } else {
            panic!("Expected return with integer literal after optimization");
        }
    }

    #[test]
    fn test_boolean_folding() {
        let code = r#"
            fn main() -> bool {
                return true && false || true;
            }
        "#;
        
        let program = parse_and_optimize(code);
        let main_fn = program.functions.iter().find(|f| f.name == "main").unwrap();
        
        if let aetos::ast::Statement::Return { value: aetos::ast::Expression::BoolLiteral(result) } = &main_fn.body[0] {
            assert!(*result); // true && false || true = true
        } else {
            panic!("Expected return with boolean literal after optimization");
        }
    }
}