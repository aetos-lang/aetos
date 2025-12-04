// src/codegen/wasm.rs
use crate::ast::*;
use std::collections::HashMap;

pub struct WasmGenerator {
    type_section: String,
    function_section: String,
    export_section: String,
    code_section: String,
    function_types: HashMap<String, (Vec<String>, String)>,
    current_function: String,
    locals: HashMap<String, String>,
    strings: Vec<String>,
    code: String,
}

impl WasmGenerator {
    pub fn new() -> Self {
        Self {
            type_section: String::new(),
            function_section: String::new(),
            export_section: String::new(),
            code_section: String::new(),
            function_types: HashMap::new(),
            current_function: String::new(),
            locals: HashMap::new(),
            strings: Vec::new(),
            code: String::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        // Сначала собираем информацию о типах функций
        for function in &program.functions {
            let param_types: Vec<String> = function.params
                .iter()
                .map(|p| self.type_to_wasm(&p.param_type))
                .collect();
            let return_type = self.type_to_wasm(&function.return_type);
            self.function_types.insert(function.name.clone(), (param_types, return_type));
        }

        // Генерируем секции
        self.generate_type_section(&program);
        self.generate_function_section(&program);
        self.generate_export_section(&program);
        self.generate_code_section(&program);

        // Собираем итоговый модуль WASM
        let mut wasm_module = String::new();
        wasm_module.push_str("(module\n");
        wasm_module.push_str(&self.type_section);
        wasm_module.push_str(&self.function_section);
        wasm_module.push_str(&self.export_section);
        wasm_module.push_str(&self.code_section);
        wasm_module.push_str(")\n");
        
        wasm_module
    }

    fn generate_type_section(&mut self, program: &Program) {
        self.type_section.push_str("  (type (func");
        
        for (i, function) in program.functions.iter().enumerate() {
            let (param_types, return_type) = self.function_types.get(&function.name).unwrap();
            
            self.type_section.push_str(&format!(
                "\n    (type ${} (func (param {}) (result {})))",
                i,
                param_types.join(" "),
                return_type
            ));
        }
        
        self.type_section.push_str("\n  ))\n");
    }

    fn generate_function_section(&mut self, program: &Program) {
        self.function_section.push_str("  (func");
        
        for (i, function) in program.functions.iter().enumerate() {
            self.function_section.push_str(&format!(
                "\n    (func ${} (type ${})",
                function.name, i
            ));
            
            // Объявляем параметры
            for param in &function.params {
                let wasm_type = self.type_to_wasm(&param.param_type);
                self.function_section.push_str(&format!(
                    "\n      (param ${} {})",
                    param.name, wasm_type
                ));
            }
            
            // Объявляем возвращаемый тип
            let return_type = self.type_to_wasm(&function.return_type);
            if return_type != "void" {
                self.function_section.push_str(&format!("\n      (result {})", return_type));
            }
            
            self.function_section.push_str("\n    )");
        }
        
        self.function_section.push_str("\n  )\n");
    }

    fn generate_export_section(&mut self, program: &Program) {
        self.export_section.push_str("  (export");
        
        for function in &program.functions {
            if function.name == "main" {
                self.export_section.push_str(&format!(
                    "\n    (export \"main\" (func ${}))",
                    function.name
                ));
            }
        }
        
        self.export_section.push_str("\n  )\n");
    }

    fn generate_code_section(&mut self, program: &Program) {
        self.code_section.push_str("  (code");
        
        for function in &program.functions {
            self.current_function = function.name.clone();
            self.locals.clear();
            self.code.clear();
            
            // Генерируем код функции
            for statement in &function.body {
                self.generate_statement(statement);
            }
            
            // Добавляем неявный возврат для void функций
            if function.return_type == Type::Void {
                self.code.push_str("return\n");
            }
            
            self.code_section.push_str(&format!(
                "\n    (func ${}\n      {}\n    )",
                function.name, self.code
            ));
        }
        
        self.code_section.push_str("\n  )\n");
    }

    pub fn generate_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::VariableDeclaration { name, var_type, value, mutable: _ } => {
                // Генерируем значение выражения
                self.generate_expression(value);
                
                // Определяем тип для WebAssembly
                let wasm_type = match var_type {
                    Type::I32 | Type::Bool => "i32",
                    Type::I64 => "i64",
                    Type::F32 => "f32",
                    Type::F64 => "f64",
                    Type::String => "i32", // указатель на строку
                    Type::Void => unreachable!("Cannot declare variable of type void"),
                    Type::Struct(_) => "i32", // указатель на структуру
                };
                
                // Сохраняем переменную в локальной области видимости
                self.locals.insert(name.clone(), wasm_type.to_string());
                
                // Сохраняем значение в локальной переменной
                self.code.push_str(&format!("local.set ${}\n", name));
            }
            
            Statement::Assignment { name, value } => {
                // Проверяем, что переменная существует
                if !self.locals.contains_key(name) {
                    panic!("Assignment to undefined variable: {}", name);
                }
                
                // Генерируем значение выражения
                self.generate_expression(value);
                
                // Сохраняем значение в существующей переменной
                self.code.push_str(&format!("local.set ${}\n", name));
            }
            
            Statement::Return { value } => {
                self.generate_expression(value);
                self.code.push_str("return\n");
            }
            
            Statement::Expression(expr) => {
                self.generate_expression(expr);
                // Для выражений, которые не используются, выбрасываем результат
                self.code.push_str("drop\n");
            }
            
            Statement::Block { statements } => {
                // Сохраняем текущие локальные переменные
                let old_locals = self.locals.clone();
                
                // Генерируем все операторы в блоке
                for stmt in statements {
                    self.generate_statement(stmt);
                }
                
                // Восстанавливаем локальные переменные (убираем те, что были объявлены в блоке)
                self.locals = old_locals;
            }
            
            Statement::While { condition, body } => {
                // Начало цикла
                self.code.push_str("block\n");
                self.code.push_str("loop\n");
                
                // Генерируем условие
                self.generate_expression(condition);
                self.code.push_str("i32.eqz\n");
                self.code.push_str("br_if 1\n"); // Выход из цикла если условие ложно
                
                // Тело цикла
                for stmt in body {
                    self.generate_statement(stmt);
                }
                
                self.code.push_str("br 0\n"); // Возврат к началу цикла
                self.code.push_str("end\n");
                self.code.push_str("end\n");
            }
            
            Statement::If { condition, then_branch, else_branch } => {
                // Генерируем условие
                self.generate_expression(condition);
                
                self.code.push_str("if\n");
                
                // Ветка then
                for stmt in then_branch {
                    self.generate_statement(stmt);
                }
                
                if let Some(else_branch) = else_branch {
                    self.code.push_str("else\n");
                    // Ветка else
                    for stmt in else_branch {
                        self.generate_statement(stmt);
                    }
                }
                
                self.code.push_str("end\n");
            }
        }
    }

    fn generate_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::IntegerLiteral(value) => {
                self.code.push_str(&format!("i32.const {}\n", value));
            }
            
            Expression::FloatLiteral(value) => {
                self.code.push_str(&format!("f32.const {}\n", value));
            }
            
            Expression::BoolLiteral(value) => {
                self.code.push_str(&format!("i32.const {}\n", if *value { 1 } else { 0 }));
            }
            
            Expression::StringLiteral(value) => {
                // Сохраняем строку в памяти и возвращаем указатель
                let ptr = self.strings.len() as i32;
                self.strings.push(value.clone());
                self.code.push_str(&format!("i32.const {}\n", ptr));
            }
            
            Expression::Variable(name) => {
                // Загружаем значение переменной
                self.code.push_str(&format!("local.get ${}\n", name));
            }
            
            Expression::BinaryExpression { left, operator, right } => {
                // Генерируем левый операнд
                self.generate_expression(left);
                // Генерируем правый операнд
                self.generate_expression(right);
                
                // Генерируем операцию
                match operator {
                    BinaryOperator::Add => self.code.push_str("i32.add\n"),
                    BinaryOperator::Subtract => self.code.push_str("i32.sub\n"),
                    BinaryOperator::Multiply => self.code.push_str("i32.mul\n"),
                    BinaryOperator::Divide => self.code.push_str("i32.div_s\n"),
                    BinaryOperator::Eq => self.code.push_str("i32.eq\n"),
                    BinaryOperator::Neq => self.code.push_str("i32.ne\n"),
                    BinaryOperator::Lt => self.code.push_str("i32.lt_s\n"),
                    BinaryOperator::Gt => self.code.push_str("i32.gt_s\n"),
                    BinaryOperator::Lte => self.code.push_str("i32.le_s\n"),
                    BinaryOperator::Gte => self.code.push_str("i32.ge_s\n"),
                    BinaryOperator::And => {
                        // Логическое И: a && b эквивалентно (a != 0) && (b != 0)
                        self.code.push_str("i32.and\n");
                        self.code.push_str("i32.const 0\n");
                        self.code.push_str("i32.ne\n");
                    }
                    BinaryOperator::Or => {
                        // Логическое ИЛИ: a || b эквивалентно (a != 0) || (b != 0)
                        self.code.push_str("i32.or\n");
                        self.code.push_str("i32.const 0\n");
                        self.code.push_str("i32.ne\n");
                    }
                }
            }
            
            Expression::FunctionCall { name, args } => {
                // Генерируем аргументы
                for arg in args.iter().rev() {
                    self.generate_expression(arg);
                }
                
                // Вызываем функцию
                self.code.push_str(&format!("call ${}\n", name));
            }
            
            Expression::StructInitialization { struct_name: _, fields: _ } => {
                // TODO: Реализовать инициализацию структур
                panic!("Struct initialization not implemented in WASM backend");
            }
            
            Expression::FieldAccess { expression: _, field_name: _ } => {
                // TODO: Реализовать доступ к полям структур
                panic!("Field access not implemented in WASM backend");
            }
            
            Expression::TypeCast { expression, target_type } => {
                // Генерируем выражение
                self.generate_expression(expression);
                
                // Генерируем приведение типа
                match target_type {
                    Type::I32 => self.code.push_str("i32.trunc_f32_s\n"),
                    Type::F32 => self.code.push_str("f32.convert_i32_s\n"),
                    _ => panic!("Unsupported type cast in WASM: {:?}", target_type),
                }
            }
            
            Expression::Move { expression: _ } => {
                // В WebAssembly нет семантики перемещения
                panic!("Move semantics not implemented in WASM backend");
            }
            
            Expression::Borrow { expression: _, mutable: _ } => {
                // В WebAssembly нет семантики заимствования
                panic!("Borrow semantics not implemented in WASM backend");
            }
        }
    }

    fn type_to_wasm(&self, ty: &Type) -> String {
        match ty {
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Bool => "i32".to_string(), // bool представляется как i32
            Type::String => "i32".to_string(), // указатель на строку
            Type::Void => "void".to_string(),
            Type::Struct(_) => "i32".to_string(), // указатель на структуру
        }
    }
}