// interpreter.rs - исправленная версия

use crate::ast::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::graphics_engine::GraphicsEngine;
use minifb::Key;

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Integer(i32),
    Float(f32),
    Boolean(bool),
    String(String),
    Struct(String, HashMap<String, RuntimeValue>),
    Void,
}

// В interpreter.rs добавьте поле start_time
pub struct Interpreter {
    variables: HashMap<String, RuntimeValue>,
    functions: HashMap<String, Function>,
    graphics_engine: Option<GraphicsEngine>,
    pub should_exit: bool,
    start_time: std::time::Instant, // Добавьте это поле
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            graphics_engine: None,
            should_exit: false,
            start_time: std::time::Instant::now(), // Инициализируйте здесь
        }
    }

    pub fn interpret_program(&mut self, program: &Program, width: usize, height: usize, title: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Сначала собираем все пользовательские функции
        for function in &program.functions {
            self.functions.insert(function.name.clone(), function.clone());
        }

        // Ищем функцию main
        let main_function = self.functions.get("main")
            .ok_or("No main function found")?;

        // Инициализируем графику если есть графические функции
        if self.has_graphics_functions(program) {
            self.graphics_engine = Some(GraphicsEngine::new(width, height, title)?);
        }

        // Выполняем main функцию - клонируем функцию чтобы избежать проблем с заимствованиями
        let main_function_clone = main_function.clone();
        self.interpret_function(&main_function_clone, &[])?;

        Ok(())
    }

    fn has_graphics_functions(&self, program: &Program) -> bool {
        // Проверяем, используются ли графические функции
        let graphics_functions = [
            "init_graphics", "clear_screen", "draw_pixel", "draw_rect", 
            "draw_circle", "draw_line", "render", "get_time", "sleep"
        ];

        for function in &program.functions {
            if self.contains_graphics_calls(&function.body, &graphics_functions) {
                return true;
            }
        }
        false
    }

    fn contains_graphics_calls(&self, statements: &[Statement], graphics_functions: &[&str]) -> bool {
        for statement in statements {
            match statement {
                Statement::Expression(expr) => {
                    if self.expression_contains_graphics(expr, graphics_functions) {
                        return true;
                    }
                }
                Statement::Block { statements } => {
                    if self.contains_graphics_calls(statements, graphics_functions) {
                        return true;
                    }
                }
                Statement::If { then_branch, else_branch, .. } => {
                    if self.contains_graphics_calls(then_branch, graphics_functions) {
                        return true;
                    }
                    if let Some(else_branch) = else_branch {
                        if self.contains_graphics_calls(else_branch, graphics_functions) {
                            return true;
                        }
                    }
                }
                Statement::While { body, .. } => {
                    if self.contains_graphics_calls(body, graphics_functions) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn expression_contains_graphics(&self, expr: &Expression, graphics_functions: &[&str]) -> bool {
        match expr {
            Expression::FunctionCall { name, .. } => {
                graphics_functions.contains(&name.as_str())
            }
            Expression::BinaryExpression { left, right, .. } => {
                self.expression_contains_graphics(left, graphics_functions) ||
                self.expression_contains_graphics(right, graphics_functions)
            }
            _ => false
        }
    }

    fn interpret_function(&mut self, function: &Function, args: &[RuntimeValue]) -> Result<RuntimeValue, Box<dyn std::error::Error>> {
        // Сохраняем текущие переменные
        let old_variables = std::mem::take(&mut self.variables);

        // Устанавливаем параметры
        for (i, param) in function.params.iter().enumerate() {
            if i < args.len() {
                self.variables.insert(param.name.clone(), args[i].clone());
            }
        }

        // Выполняем тело функции
        let mut result = RuntimeValue::Void;
        for statement in &function.body {
            result = self.interpret_statement(statement)?;
            
            // Если встретили return, прерываем выполнение
            if let Statement::Return { .. } = statement {
                break;
            }
        }

        // Восстанавливаем переменные
        self.variables = old_variables;

        Ok(result)
    }

    fn interpret_statement(&mut self, statement: &Statement) -> Result<RuntimeValue, Box<dyn std::error::Error>> {
        match statement {
            // interpreter.rs - в функции interpret_statement
            Statement::VariableDeclaration { name, var_type: _, value, mutable } => {
                let value = self.interpret_expression(value)?;
                self.variables.insert(name.clone(), value);
                Ok(RuntimeValue::Void)
            }

            // interpreter.rs - добавьте в interpret_statement
            Statement::Assignment { name, value } => {
                let new_value = self.interpret_expression(value)?;
                // interpreter.rs - исправьте строку 163
                if let Some(old_value) = self.variables.get_mut(name) {
                    // Убрали лишний & перед name
                    *old_value = new_value;
                } else {
                    return Err(format!("Undefined variable: {}", name).into());
                }
                Ok(RuntimeValue::Void)
            }
            
            Statement::Return { value } => {
                let result = self.interpret_expression(value)?;
                Ok(result)
            }
            
            Statement::Expression(expr) => {
                self.interpret_expression(expr)?;
                Ok(RuntimeValue::Void)
            }
            
            Statement::Block { statements } => {
                // Сохраняем текущие переменные
                let old_variables = self.variables.clone();
                
                let mut result = RuntimeValue::Void;
                for stmt in statements {
                    result = self.interpret_statement(stmt)?;
                }
                
                // Восстанавливаем переменные
                self.variables = old_variables;
                Ok(result)
            }

            // interpreter.rs - в interpret_statement для VariableDeclaration
            Statement::VariableDeclaration { name, var_type: _, value, mutable } => {
                if self.variables.contains_key(name) && !mutable {
                    return Err(format!("Cannot reassign immutable variable: {}", name).into());
                }
    
                let value = self.interpret_expression(value)?;
                self.variables.insert(name.clone(), value);
                Ok(RuntimeValue::Void)
            }
            
            Statement::While { condition, body } => {
                // ВАЖНО: сохраняем переменные перед циклом
                let old_variables = self.variables.clone();
            
                loop {
                    // Вычисляем условие
                    let condition_result = self.interpret_expression(condition)?;
                    let condition_value = self.is_truthy(&condition_result);
                
                    if !condition_value {
                        break;
                    }
                    
                    // ВАЖНО: НЕ сохраняем переменные перед выполнением тела
                    // Это позволяет переменным сохраняться между итерациями
                    
                    // Выполняем тело цикла
                    for stmt in body {
                        self.interpret_statement(stmt)?;
                    }
                    
                    // Проверяем выход из графического цикла
                    if let Some(engine) = &mut self.graphics_engine {
                        if !engine.update() {
                            self.should_exit = true;
                            break;
                        }
                    }
                    
                    if self.should_exit {
                        break;
                    }
                }
                
                // ВАЖНО: НЕ восстанавливаем исходные переменные после цикла
                // Это позволяет изменениям переменных сохраняться после цикла
                
                Ok(RuntimeValue::Void)
            }
            
            Statement::If { condition, then_branch, else_branch } => {
                let condition_result = self.interpret_expression(condition)?;
                
                // Сохраняем переменные перед ветвлением
                let old_variables = self.variables.clone();
                
                if self.is_truthy(&condition_result) {
                    for stmt in then_branch {
                        self.interpret_statement(stmt)?;
                    }
                } else if let Some(else_branch) = else_branch {
                    for stmt in else_branch {
                        self.interpret_statement(stmt)?;
                    }
                }
                
                // Восстанавливаем переменные после ветвления
                self.variables = old_variables;
                Ok(RuntimeValue::Void)
            }
        }
    }

    fn interpret_expression(&mut self, expr: &Expression) -> Result<RuntimeValue, Box<dyn std::error::Error>> {
        match expr {
            Expression::IntegerLiteral(value) => Ok(RuntimeValue::Integer(*value)),
            Expression::FloatLiteral(value) => Ok(RuntimeValue::Float(*value)),
            Expression::StringLiteral(value) => Ok(RuntimeValue::String(value.clone())),
            Expression::BoolLiteral(value) => Ok(RuntimeValue::Boolean(*value)),
            
            Expression::Variable(name) => {
                self.variables.get(name)
                    .cloned()
                    .ok_or_else(|| format!("Undefined variable: {}", name).into())
            }
            
            Expression::BinaryExpression { left, operator, right } => {
                let left_val = self.interpret_expression(left)?;
                let right_val = self.interpret_expression(right)?;
                
                self.evaluate_binary_operation(&left_val, operator, &right_val)
            }
            
            Expression::FunctionCall { name, args } => {
                let arg_values: Vec<RuntimeValue> = args.iter()
                    .map(|arg| self.interpret_expression(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                
                // Сначала проверяем встроенные функции
                if self.is_builtin_function(name) {
                    self.call_builtin_function(name, &arg_values)
                } else {
                    // Затем пользовательские функции
                    if let Some(function) = self.functions.get(name) {
                        // Клонируем функцию чтобы избежать проблем с заимствованиями
                        let function_clone = function.clone();
                        self.interpret_function(&function_clone, &arg_values)
                    } else {
                        Err(format!("Undefined function: {}", name).into())
                    }
                }
            }
            
            Expression::StructInitialization { struct_name, fields } => {
                let mut field_values = HashMap::new();
                for (field_name, field_expr) in fields {
                    let value = self.interpret_expression(field_expr)?;
                    field_values.insert(field_name.clone(), value);
                }
                Ok(RuntimeValue::Struct(struct_name.clone(), field_values))
            }
            
            Expression::FieldAccess { expression, field_name } => {
                let struct_val = self.interpret_expression(expression)?;
                if let RuntimeValue::Struct(_, fields) = struct_val {
                    fields.get(field_name)
                        .cloned()
                        .ok_or_else(|| format!("Undefined field: {}", field_name).into())
                } else {
                    Err("Field access on non-struct value".into())
                }
            }

            Expression::TypeCast { expression, target_type } => {
                let value = self.interpret_expression(expression)?;
    
                match (value, target_type) {
                    (RuntimeValue::Integer(i), Type::F32) => Ok(RuntimeValue::Float(i as f32)),
                    (RuntimeValue::Float(f), Type::I32) => Ok(RuntimeValue::Integer(f as i32)),
                    (RuntimeValue::Integer(i), Type::I64) => Ok(RuntimeValue::Integer(i)), // временно
                    (RuntimeValue::Integer(i), Type::F64) => Ok(RuntimeValue::Float(i as f32)), // временно
                    (value, _) => Ok(value), // Если типы совпадают или преобразование не нужно
                }
            }
            
            // Пока упрощенно обрабатываем move и borrow
            Expression::Move { expression } => self.interpret_expression(expression),
            Expression::Borrow { expression, .. } => self.interpret_expression(expression),
        }
    }

    fn is_builtin_function(&self, name: &str) -> bool {
        matches!(name, 
            "print_i32" | "print_string" | "print" |
            "gpio_set" | "gpio_toggle" | "delay" |
            // Графические функции
            "init_graphics" | "clear_screen" | "draw_pixel" | "draw_rect" | 
            "draw_circle" | "draw_line" | "render" | "get_time" | "sleep" | "is_key_pressed"
        )
    }   

    fn call_builtin_function(&mut self, name: &str, args: &[RuntimeValue]) -> Result<RuntimeValue, Box<dyn std::error::Error>> {
        match name {
            // Встроенные функции вывода
            "print_i32" => {
                if let RuntimeValue::Integer(value) = &args[0] {
                    println!("{}", value);
                }
                Ok(RuntimeValue::Void)
            }
            "print" => {
                if let RuntimeValue::Integer(value) = &args[0] {
                    println!("{}", value);
                }
                Ok(RuntimeValue::Void)
            }
            "print_string" => {
                if let RuntimeValue::String(value) = &args[0] {
                    println!("{}", value);
                }
                Ok(RuntimeValue::Void)
            }
            
            // GPIO функции (заглушки)
            "gpio_set" => {
                // Игнорируем GPIO операции
                Ok(RuntimeValue::Void)
            }
            "gpio_toggle" => {
                // Игнорируем GPIO операции
                Ok(RuntimeValue::Void)
            }
            "delay" => {
                if let RuntimeValue::Integer(ms) = args[0] {
                    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
                }
                Ok(RuntimeValue::Void)
            }
            
            // Графические функции
            "init_graphics" => {
                // Уже инициализировано при запуске
                Ok(RuntimeValue::Void)
            }
            "clear_screen" => {
                if let (RuntimeValue::Integer(r), RuntimeValue::Integer(g), RuntimeValue::Integer(b)) = (&args[0], &args[1], &args[2]) {
                    if let Some(engine) = &mut self.graphics_engine {
                        engine.clear(*r as u8, *g as u8, *b as u8);
                    }
                }
                Ok(RuntimeValue::Void)
            }
            "draw_pixel" => {
                if let (RuntimeValue::Integer(x), RuntimeValue::Integer(y), RuntimeValue::Integer(r), RuntimeValue::Integer(g), RuntimeValue::Integer(b)) = 
                    (&args[0], &args[1], &args[2], &args[3], &args[4]) {
                    if let Some(engine) = &mut self.graphics_engine {
                        engine.draw_pixel(*x, *y, *r as u8, *g as u8, *b as u8);
                    }
                }
                Ok(RuntimeValue::Void)
            }
            "draw_rect" => {
                if let (RuntimeValue::Integer(x), RuntimeValue::Integer(y), RuntimeValue::Integer(w), RuntimeValue::Integer(h), RuntimeValue::Integer(r), RuntimeValue::Integer(g), RuntimeValue::Integer(b)) = 
                    (&args[0], &args[1], &args[2], &args[3], &args[4], &args[5], &args[6]) {
                    if let Some(engine) = &mut self.graphics_engine {
                        engine.draw_rect(*x, *y, *w, *h, *r as u8, *g as u8, *b as u8);
                    }
                }
                Ok(RuntimeValue::Void)
            }
            "draw_circle" => {
                if let (RuntimeValue::Integer(x), RuntimeValue::Integer(y), RuntimeValue::Integer(radius), RuntimeValue::Integer(r), RuntimeValue::Integer(g), RuntimeValue::Integer(b)) = 
                    (&args[0], &args[1], &args[2], &args[3], &args[4], &args[5]) {
                    if let Some(engine) = &mut self.graphics_engine {
                        engine.draw_circle(*x, *y, *radius, *r as u8, *g as u8, *b as u8);
                    }
                }
                Ok(RuntimeValue::Void)
            }
            "draw_line" => {
                if let (RuntimeValue::Integer(x1), RuntimeValue::Integer(y1), RuntimeValue::Integer(x2), RuntimeValue::Integer(y2), RuntimeValue::Integer(r), RuntimeValue::Integer(g), RuntimeValue::Integer(b)) = 
                    (&args[0], &args[1], &args[2], &args[3], &args[4], &args[5], &args[6]) {
                    if let Some(engine) = &mut self.graphics_engine {
                        engine.draw_line(*x1, *y1, *x2, *y2, *r as u8, *g as u8, *b as u8);
                    }
                }
                Ok(RuntimeValue::Void)
            }
            "render" => {
                if let Some(engine) = &mut self.graphics_engine {
                    engine.render();
                }
                Ok(RuntimeValue::Void)
            }
            // Затем в call_builtin_function
            "get_time" => {
                let elapsed = self.start_time.elapsed();
                Ok(RuntimeValue::Float(elapsed.as_secs_f32()))
            }
            "sleep" => {
                if let RuntimeValue::Integer(ms) = args[0] {
                    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
                }
                Ok(RuntimeValue::Void)
            }
            "is_key_pressed" => {
                if let RuntimeValue::Integer(key_code) = args[0] {
                    if let Some(engine) = &self.graphics_engine {
                        let key = match key_code {
                            87 => Key::W,     // W
                            83 => Key::S,     // S
                            65 => Key::A,     // A
                            68 => Key::D,     // D
                            37 => Key::Left,  // Left arrow
                            38 => Key::Up,    // Up arrow
                            39 => Key::Right, // Right arrow
                            40 => Key::Down,  // Down arrow
                            32 => Key::Space, // Space
                            _ => return Ok(RuntimeValue::Boolean(false)),
                        };
                        return Ok(RuntimeValue::Boolean(engine.is_key_pressed(key)));
                    }
                }
                Ok(RuntimeValue::Boolean(false))
            }
            
            _ => Err(format!("Unknown builtin function: {}", name).into())
        }
    }

    fn evaluate_binary_operation(&self, left: &RuntimeValue, operator: &BinaryOperator, right: &RuntimeValue) -> Result<RuntimeValue, Box<dyn std::error::Error>> {
        println!("DEBUG INTERPRETER: Binary operation - left: {:?}, operator: {:?}, right: {:?}", left, operator, right);
    
        match (left, operator, right) {
            (RuntimeValue::Integer(l), op, RuntimeValue::Integer(r)) => {
                println!("DEBUG INTERPRETER: Integer operation: {} {:?} {}", l, op, r);
                match op {
                    BinaryOperator::Add => Ok(RuntimeValue::Integer(l + r)),
                    BinaryOperator::Subtract => Ok(RuntimeValue::Integer(l - r)),
                    BinaryOperator::Multiply => Ok(RuntimeValue::Integer(l * r)),
                    BinaryOperator::Divide => {
                        if *r == 0 {
                            Err("Division by zero".into())
                        } else {
                            Ok(RuntimeValue::Integer(l / r))
                        }
                    }
                    BinaryOperator::Eq => Ok(RuntimeValue::Boolean(l == r)),
                    BinaryOperator::Neq => Ok(RuntimeValue::Boolean(l != r)),
                    BinaryOperator::Lt => Ok(RuntimeValue::Boolean(l < r)),
                    BinaryOperator::Gt => Ok(RuntimeValue::Boolean(l > r)),
                    BinaryOperator::Lte => Ok(RuntimeValue::Boolean(l <= r)),
                    BinaryOperator::Gte => Ok(RuntimeValue::Boolean(l >= r)),
                    BinaryOperator::And | BinaryOperator::Or => {
                        Err("Logical operations not supported for integers".into())
                    }
                }
            }
            (RuntimeValue::Float(l), op, RuntimeValue::Float(r)) => {
                println!("DEBUG INTERPRETER: Float operation: {} {:?} {}", l, op, r);
                match op {
                    BinaryOperator::Add => Ok(RuntimeValue::Float(l + r)),
                    BinaryOperator::Subtract => Ok(RuntimeValue::Float(l - r)),
                    BinaryOperator::Multiply => Ok(RuntimeValue::Float(l * r)),
                    BinaryOperator::Divide => {
                        if *r == 0.0 {
                            Err("Division by zero".into())
                        } else {
                            Ok(RuntimeValue::Float(l / r))
                        }
                    }
                    BinaryOperator::Eq => Ok(RuntimeValue::Boolean(l == r)),
                    BinaryOperator::Neq => Ok(RuntimeValue::Boolean(l != r)),
                    BinaryOperator::Lt => Ok(RuntimeValue::Boolean(l < r)),
                    BinaryOperator::Gt => Ok(RuntimeValue::Boolean(l > r)),
                    BinaryOperator::Lte => Ok(RuntimeValue::Boolean(l <= r)),
                    BinaryOperator::Gte => Ok(RuntimeValue::Boolean(l >= r)),
                    BinaryOperator::And | BinaryOperator::Or => {
                        Err("Logical operations not supported for floats".into())
                    }
                }
            }
            (RuntimeValue::Boolean(l), op, RuntimeValue::Boolean(r)) => {
                println!("DEBUG INTERPRETER: Boolean operation: {} {:?} {}", l, op, r);
                match op {
                    BinaryOperator::And => Ok(RuntimeValue::Boolean(*l && *r)),
                    BinaryOperator::Or => Ok(RuntimeValue::Boolean(*l || *r)),
                    BinaryOperator::Eq => Ok(RuntimeValue::Boolean(l == r)),
                    BinaryOperator::Neq => Ok(RuntimeValue::Boolean(l != r)),
                    _ => {
                        println!("DEBUG INTERPRETER: Unsupported operation for booleans: {:?}", op);
                        Err("Unsupported operation for booleans".into())
                    }
                }
            }
            // Смешанные типы: Integer и Float
            (RuntimeValue::Integer(l), op, RuntimeValue::Float(r)) => {
                println!("DEBUG INTERPRETER: Mixed operation (int, float): {} {:?} {}", l, op, r);
                let l_float = *l as f32;
                match op {
                    BinaryOperator::Add => Ok(RuntimeValue::Float(l_float + r)),
                    BinaryOperator::Subtract => Ok(RuntimeValue::Float(l_float - r)),
                    BinaryOperator::Multiply => Ok(RuntimeValue::Float(l_float * r)),
                    BinaryOperator::Divide => {
                        if *r == 0.0 {
                            Err("Division by zero".into())
                        } else {
                            Ok(RuntimeValue::Float(l_float / r))
                        }
                    }
                    BinaryOperator::Eq => Ok(RuntimeValue::Boolean(l_float == *r)),
                    BinaryOperator::Neq => Ok(RuntimeValue::Boolean(l_float != *r)),
                    BinaryOperator::Lt => Ok(RuntimeValue::Boolean(l_float < *r)),
                    BinaryOperator::Gt => Ok(RuntimeValue::Boolean(l_float > *r)),
                    BinaryOperator::Lte => Ok(RuntimeValue::Boolean(l_float <= *r)),
                    BinaryOperator::Gte => Ok(RuntimeValue::Boolean(l_float >= *r)),
                    BinaryOperator::And | BinaryOperator::Or => {
                        Err("Logical operations not supported for mixed types".into())
                    }
                }
            }
            (RuntimeValue::Float(l), op, RuntimeValue::Integer(r)) => {
                println!("DEBUG INTERPRETER: Mixed operation (float, int): {} {:?} {}", l, op, r);
                let r_float = *r as f32;
                match op {
                    BinaryOperator::Add => Ok(RuntimeValue::Float(l + r_float)),
                    BinaryOperator::Subtract => Ok(RuntimeValue::Float(l - r_float)),
                    BinaryOperator::Multiply => Ok(RuntimeValue::Float(l * r_float)),
                    BinaryOperator::Divide => {
                        if r_float == 0.0 {
                            Err("Division by zero".into())
                        } else {
                            Ok(RuntimeValue::Float(l / r_float))
                        }
                    }
                    BinaryOperator::Eq => Ok(RuntimeValue::Boolean(*l == r_float)),
                    BinaryOperator::Neq => Ok(RuntimeValue::Boolean(*l != r_float)),
                    BinaryOperator::Lt => Ok(RuntimeValue::Boolean(*l < r_float)),
                    BinaryOperator::Gt => Ok(RuntimeValue::Boolean(*l > r_float)),
                    BinaryOperator::Lte => Ok(RuntimeValue::Boolean(*l <= r_float)),
                    BinaryOperator::Gte => Ok(RuntimeValue::Boolean(*l >= r_float)),
                    BinaryOperator::And | BinaryOperator::Or => {
                        Err("Logical operations not supported for mixed types".into())
                    }
                }
            }
            _ => {
                println!("DEBUG INTERPRETER: Type mismatch - left: {:?}, right: {:?}", left, right);
                Err("Type mismatch in binary operation".into())
            }
        }
    }

    fn is_truthy(&self, value: &RuntimeValue) -> bool {
        match value {
            RuntimeValue::Boolean(b) => *b,
            RuntimeValue::Integer(i) => *i != 0,
            RuntimeValue::Float(f) => *f != 0.0,
            _ => false,
        }
    }
}