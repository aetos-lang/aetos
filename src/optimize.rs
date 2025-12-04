use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Optimizer {
    pub constant_folding: bool,
    pub dead_code_elimination: bool,
    pub inline_functions: bool,
}

impl Default for Optimizer {
    fn default() -> Self {
        Self {
            constant_folding: true,
            dead_code_elimination: true,
            inline_functions: true,
        }
    }
}

impl Optimizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn optimize(&self, program: &mut Program) {
        if self.constant_folding {
            self.constant_folding(program);
        }
        if self.dead_code_elimination {
            self.dead_code_elimination(program);
        }
        if self.inline_functions {
            self.inline_small_functions(program);
        }
    }

    // Constant Folding
    fn constant_folding(&self, program: &mut Program) {
        for function in &mut program.functions {
            self.fold_constants_in_function(function);
        }
    }

    fn fold_constants_in_function(&self, function: &mut Function) {
        let mut new_body = Vec::new();
        for statement in function.body.drain(..) {
            new_body.push(self.fold_constants_in_statement(statement));
        }
        function.body = new_body;
    }

    fn fold_constants_in_statement(&self, statement: Statement) -> Statement {
        match statement {
            Statement::VariableDeclaration { name, var_type, value, mutable } => {
                Statement::VariableDeclaration {
                    name,
                    var_type,
                    value: self.fold_constants_in_expression(value),
                    mutable,
                }
            }
            Statement::Assignment { name, value } => {
                Statement::Assignment {
                    name,
                    value: self.fold_constants_in_expression(value),
                }
            }
            Statement::Return { value } => {
                Statement::Return {
                    value: self.fold_constants_in_expression(value),
                }
            }
            Statement::Expression(expr) => {
                Statement::Expression(self.fold_constants_in_expression(expr))
            }
            Statement::Block { statements } => {
                Statement::Block {
                    statements: statements.into_iter()
                        .map(|s| self.fold_constants_in_statement(s))
                        .collect(),
                }
            }
            Statement::While { condition, body } => {
                Statement::While {
                    condition: self.fold_constants_in_expression(condition),
                    body: body.into_iter()
                        .map(|s| self.fold_constants_in_statement(s))
                        .collect(),
                }
            }
            Statement::If { condition, then_branch, else_branch } => {
                Statement::If {
                    condition: self.fold_constants_in_expression(condition),
                    then_branch: then_branch.into_iter()
                        .map(|s| self.fold_constants_in_statement(s))
                        .collect(),
                    else_branch: else_branch.map(|branch| {
                        branch.into_iter()
                            .map(|s| self.fold_constants_in_statement(s))
                            .collect()
                    }),
                }
            }
        }
    }

    fn analyze_variable_usage(&self, statement: &Statement, used_variables: &mut HashMap<String, usize>) {
        match statement {
            Statement::VariableDeclaration { value, .. } => {
                self.analyze_expression_usage(value, used_variables);
            }
            Statement::Assignment { name, value } => {
                // При присваивании переменная используется (пишется)
                *used_variables.entry(name.clone()).or_insert(0) += 1;
                self.analyze_expression_usage(value, used_variables);
            }
            Statement::Return { value } => {
                self.analyze_expression_usage(value, used_variables);
            }
            Statement::Expression(expr) => {
                self.analyze_expression_usage(expr, used_variables);
            }
            Statement::Block { statements } => {
                for stmt in statements {
                    self.analyze_variable_usage(stmt, used_variables);
                }
            }
            Statement::While { condition, body } => {
                self.analyze_expression_usage(condition, used_variables);
                for stmt in body {
                    self.analyze_variable_usage(stmt, used_variables);
                }
            }
            Statement::If { condition, then_branch, else_branch } => {
                self.analyze_expression_usage(condition, used_variables);
                for stmt in then_branch {
                    self.analyze_variable_usage(stmt, used_variables);
                }
                if let Some(else_branch) = else_branch {
                    for stmt in else_branch {
                        self.analyze_variable_usage(stmt, used_variables);
                    }
                }
            }
        }
    }

    fn try_inline_statement(&self, statement: &Statement, function_map: &HashMap<String, Function>) -> Option<Vec<Statement>> {
        if let Statement::Expression(Expression::FunctionCall { name, args }) = statement {
            if let Some(target_function) = function_map.get(name) {
                return self.inline_function_call(target_function, args);
            }
        }
        None
    }

    fn inline_function_call(&self, target_function: &Function, args: &[Expression]) -> Option<Vec<Statement>> {
        if target_function.params.len() != args.len() {
            return None;
        }

        let mut inlined_body = Vec::new();
        
        // Создаем переменные для параметров
        for (param, arg) in target_function.params.iter().zip(args) {
            inlined_body.push(Statement::VariableDeclaration {
                name: param.name.clone(),
                var_type: param.param_type.clone(),
                value: arg.clone(),
                mutable: false,
            });
        }
        
        // Копируем тело функции
        for statement in &target_function.body {
            inlined_body.push(statement.clone());
        }
        
        Some(inlined_body)
    }

    fn fold_constants_in_expression(&self, expr: Expression) -> Expression {
        match expr {
            Expression::BinaryExpression { left, operator, right } => {
                let left = Box::new(self.fold_constants_in_expression(*left));
                let right = Box::new(self.fold_constants_in_expression(*right));

                // Попробуем свернуть константы
                if let (Expression::IntegerLiteral(left_val), Expression::IntegerLiteral(right_val)) = (&*left, &*right) {
                    match operator {
                        BinaryOperator::Add => {
                            return Expression::IntegerLiteral(left_val + right_val);
                        }
                        BinaryOperator::Subtract => {
                            return Expression::IntegerLiteral(left_val - right_val);
                        }
                        BinaryOperator::Multiply => {
                            return Expression::IntegerLiteral(left_val * right_val);
                        }
                        BinaryOperator::Divide if *right_val != 0 => {
                            return Expression::IntegerLiteral(left_val / right_val);
                        }
                        BinaryOperator::Eq => {
                            return Expression::BoolLiteral(left_val == right_val);
                        }
                        BinaryOperator::Neq => {
                            return Expression::BoolLiteral(left_val != right_val);
                        }
                        BinaryOperator::Lt => {
                            return Expression::BoolLiteral(left_val < right_val);
                        }
                        BinaryOperator::Gt => {
                            return Expression::BoolLiteral(left_val > right_val);
                        }
                        BinaryOperator::Lte => {
                            return Expression::BoolLiteral(left_val <= right_val);
                        }
                        BinaryOperator::Gte => {
                            return Expression::BoolLiteral(left_val >= right_val);
                        }
                        _ => {}
                    }
                }

                // Для логических операций с bool литералами
                if let (Expression::BoolLiteral(left_val), Expression::BoolLiteral(right_val)) = (&*left, &*right) {
                    match operator {
                        BinaryOperator::And => {
                            return Expression::BoolLiteral(*left_val && *right_val);
                        }
                        BinaryOperator::Or => {
                            return Expression::BoolLiteral(*left_val || *right_val);
                        }
                        _ => {}
                    }
                }

                Expression::BinaryExpression { left, operator, right }
            }

            Expression::TypeCast { expression, target_type } => {
                Expression::TypeCast {
                    expression: Box::new(self.fold_constants_in_expression(*expression)),
                    target_type: target_type.clone(),
                }
            }

            // Рекурсивно обрабатываем другие выражения
            Expression::FunctionCall { name, args } => {
                Expression::FunctionCall {
                    name,
                    args: args.into_iter()
                        .map(|arg| self.fold_constants_in_expression(arg))
                        .collect(),
                }
            }
            Expression::StructInitialization { struct_name, fields } => {
                Expression::StructInitialization {
                    struct_name,
                    fields: fields.into_iter()
                        .map(|(name, expr)| (name, self.fold_constants_in_expression(expr)))
                        .collect(),
                }
            }
            Expression::FieldAccess { expression, field_name } => {
                Expression::FieldAccess {
                    expression: Box::new(self.fold_constants_in_expression(*expression)),
                    field_name,
                }
            }
            Expression::Move { expression } => {
                Expression::Move {
                    expression: Box::new(self.fold_constants_in_expression(*expression)),
                }
            }
            Expression::Borrow { expression, mutable } => {
                Expression::Borrow {
                    expression: Box::new(self.fold_constants_in_expression(*expression)),
                    mutable,
                }
            }
            other => other,
        }
    }

    // Dead Code Elimination
    fn dead_code_elimination(&self, program: &mut Program) {
        for function in &mut program.functions {
            self.eliminate_dead_code_in_function(function);
        }
    }

    fn eliminate_dead_code_in_function(&self, function: &mut Function) {
        let mut used_variables = HashMap::new();
        let mut new_body = Vec::new();

        // Анализ использования переменных
        for statement in &function.body {
            self.analyze_variable_usage(statement, &mut used_variables);
        }

        // Удаляем неиспользуемые объявления переменных
        for statement in function.body.drain(..) {
            if let Statement::VariableDeclaration { name, .. } = &statement {
                if used_variables.get(name).map_or(false, |&count| count > 0) {
                    new_body.push(statement);
                }
            } else {
                new_body.push(statement);
            }
        }

        function.body = new_body;
    }

    fn analyze_expression_usage(&self, expr: &Expression, used_variables: &mut HashMap<String, usize>) {
        match expr {
            Expression::Variable(name) => {
                *used_variables.entry(name.clone()).or_insert(0) += 1;
            }
            Expression::BinaryExpression { left, right, .. } => {
                self.analyze_expression_usage(left, used_variables);
                self.analyze_expression_usage(right, used_variables);
            }
            Expression::FunctionCall { args, .. } => {
                for arg in args {
                    self.analyze_expression_usage(arg, used_variables);
                }
            }
            Expression::StructInitialization { fields, .. } => {
                for (_, expr) in fields {
                    self.analyze_expression_usage(expr, used_variables);
                }
            }
            Expression::FieldAccess { expression, .. } => {
                self.analyze_expression_usage(expression, used_variables);
            }
            Expression::Move { expression } => {
                self.analyze_expression_usage(expression, used_variables);
            }
            Expression::Borrow { expression, .. } => {
                self.analyze_expression_usage(expression, used_variables);
            }
            _ => {}
        }
    }

    // Function Inlining
    fn inline_small_functions(&self, program: &mut Program) {
        let mut function_map = HashMap::new();
        for function in &program.functions {
            if self.should_inline_function(function) {
                function_map.insert(function.name.clone(), function.clone());
            }
        }

        for function in &mut program.functions {
            self.inline_functions_in_body(function, &function_map);
        }
    }

    fn should_inline_function(&self, function: &Function) -> bool {
        // Инлайним только маленькие функции (до 5 statements)
        function.body.len() <= 5 && 
        !function.name.starts_with("print") && // Не инлайним функции ввода-вывода
        function.params.len() <= 3
    }

    fn inline_functions_in_body(&self, function: &mut Function, function_map: &HashMap<String, Function>) {
        let mut new_body = Vec::new();
        
        for statement in function.body.drain(..) {
            if let Some(inlined) = self.try_inline_statement(&statement, function_map) {
                new_body.extend(inlined);
            } else {
                new_body.push(statement);
            }
        }
        
        function.body = new_body;
    }
}