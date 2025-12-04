// typecheck.rs - исправленная версия

use crate::ast::*;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TypeCheckError {
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: Type, found: Type },
    
    #[error("Undefined variable: {name}")]
    UndefinedVariable { name: String },
    
    #[error("Undefined function: {name}")]
    UndefinedFunction { name: String },
    
    #[error("Undefined struct: {name}")]
    UndefinedStruct { name: String },
    
    #[error("Undefined field: {field} in struct {struct_name}")]
    UndefinedField { struct_name: String, field: String },
    
    #[error("Function parameter count mismatch: expected {expected}, found {found}")]
    ParameterCountMismatch { expected: usize, found: usize },
    
    #[error("Duplicate variable definition: {name}")]
    DuplicateVariable { name: String },
    
    #[error("Duplicate function definition: {name}")]
    DuplicateFunction { name: String },
    
    #[error("Duplicate struct definition: {name}")]
    DuplicateStruct { name: String },
    
    #[error("Invalid return type: expected {expected}, found {found}")]
    InvalidReturnType { expected: Type, found: Type },
    
    #[error("Cannot move variable: {name} - already moved")]
    VariableAlreadyMoved { name: String },
    
    #[error("Cannot use variable after move: {name}")]
    UseAfterMove { name: String },
    
    #[error("Condition must be boolean, found {found}")]
    NonBooleanCondition { found: Type },
}

type TypeCheckResult<T> = Result<T, TypeCheckError>;

#[derive(Debug, Clone)]
enum VariableState {
    Available,
    Moved,
    Borrowed,
}

#[derive(Debug, Clone)]
struct VariableInfo {
    var_type: Type,
    state: VariableState,
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    return_type: Type,
    params: Vec<Type>,
}

#[derive(Debug, Clone)]
struct StructInfo {
    fields: HashMap<String, Type>,
}

pub struct TypeChecker {
    variables: HashMap<String, VariableInfo>,
    functions: HashMap<String, FunctionInfo>,
    structs: HashMap<String, StructInfo>,
    current_function_return: Option<Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut checker = Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            current_function_return: None,
        };
        
        checker.add_builtin_functions();
        checker
    }
    
    fn add_builtin_functions(&mut self) {
        self.functions.insert(
            "print_i32".to_string(),
            FunctionInfo {
                return_type: Type::Void,
                params: vec![Type::I32],
            },
        );
    
        self.functions.insert(
            "print_string".to_string(),
            FunctionInfo {
                return_type: Type::Void,
                params: vec![Type::String],
            },
        );
        
        self.functions.insert(
            "print".to_string(),
            FunctionInfo {
                return_type: Type::Void,
                params: vec![Type::I32],
            },
        );
        
        // Embedded functions
        self.functions.insert(
            "gpio_set".to_string(),
            FunctionInfo {
                return_type: Type::Void,
                params: vec![Type::I32, Type::I32],
            },
        );
        
        self.functions.insert(
            "gpio_toggle".to_string(),
            FunctionInfo {
                return_type: Type::Void,
                params: vec![Type::I32],
            },
        );
        
        self.functions.insert(
            "delay".to_string(),
            FunctionInfo {
                return_type: Type::Void,
                params: vec![Type::I32],
            },
        );
        
        // Графические функции
        self.functions.insert(
            "init_graphics".to_string(),
            FunctionInfo {
                return_type: Type::Void,
                params: vec![Type::I32, Type::I32, Type::String],
            },
        );
    
        self.functions.insert(
            "clear_screen".to_string(),
            FunctionInfo {
                return_type: Type::Void,
                params: vec![Type::I32, Type::I32, Type::I32],
            },
        );
        
        self.functions.insert(
            "draw_circle".to_string(),
            FunctionInfo {
                return_type: Type::Void,
                params: vec![Type::I32, Type::I32, Type::I32, Type::I32, Type::I32, Type::I32],
            },
        );
        
        self.functions.insert(
            "draw_line".to_string(),
            FunctionInfo {
                return_type: Type::Void,
                params: vec![Type::I32, Type::I32, Type::I32, Type::I32, Type::I32, Type::I32, Type::I32],
            },
        );
        
        self.functions.insert(
            "render".to_string(),
            FunctionInfo {
                return_type: Type::Void,
                params: vec![],
            },
        );
        
        // В функции add_builtin_functions в typecheck.rs
        self.functions.insert(
            "get_time".to_string(),
            FunctionInfo {
                return_type: Type::F32,  // Изменено с F64 на F32
                params: vec![],
            },
        );
        
        self.functions.insert(
            "sleep".to_string(),
            FunctionInfo {
                return_type: Type::Void,
                params: vec![Type::I32],
            },
        );
    }
    
    pub fn check_program(&mut self, program: &Program) -> TypeCheckResult<()> {
        // Сначала собираем информацию о структурах
        for struct_def in &program.structs {
            if self.structs.contains_key(&struct_def.name) {
                return Err(TypeCheckError::DuplicateStruct {
                    name: struct_def.name.clone(),
                });
            }
            
            let mut fields = HashMap::new();
            for field in &struct_def.fields {
                fields.insert(field.name.clone(), field.field_type.clone());
            }
            
            self.structs.insert(
                struct_def.name.clone(),
                StructInfo { fields },
            );
        }
        
        // Сначала собираем информацию о ВСЕХ функциях (включая пользовательские)
        let mut function_info = HashMap::new();
        for function in &program.functions {
            if function_info.contains_key(&function.name) {
                return Err(TypeCheckError::DuplicateFunction {
                    name: function.name.clone(),
                });
            }
            
            let param_types: Vec<Type> = function.params.iter()
                .map(|p| p.param_type.clone())
                .collect();

            function_info.insert(
                function.name.clone(),
                FunctionInfo {
                    return_type: function.return_type.clone(),
                    params: param_types,
                },
            );
        }
        
        // Добавляем встроенные функции к пользовательским
        for (name, info) in &function_info {
            self.functions.insert(name.clone(), info.clone());
        }
        
        // Проверяем функции
        for function in &program.functions {
            self.check_function(function)?;
        }
        
        Ok(())
    }
    
    fn check_function(&mut self, function: &Function) -> TypeCheckResult<()> {
        self.variables.clear();
        self.current_function_return = Some(function.return_type.clone());
        
        for param in &function.params {
            if self.variables.contains_key(&param.name) {
                return Err(TypeCheckError::DuplicateVariable {
                    name: param.name.clone(),
                });
            }
            
            self.variables.insert(
                param.name.clone(),
                VariableInfo {
                    var_type: param.param_type.clone(),
                    state: VariableState::Available,
                },
            );
        }
        
        for statement in &function.body {
            self.check_statement(statement)?;
        }
        
        Ok(())
    }
    
    fn check_statement(&mut self, statement: &Statement) -> TypeCheckResult<()> {
        match statement {
            Statement::VariableDeclaration { name, var_type, value, mutable: _ } => {
                if self.variables.contains_key(name) {
                    return Err(TypeCheckError::DuplicateVariable {
                        name: name.clone(),
                    });
                }
                
                let expr_type = self.check_expression(value)?;
                
                // Разрешаем неявное приведение i32 -> f32
                if !self.types_are_compatible(var_type, &expr_type) {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: var_type.clone(),
                        found: expr_type,
                    });
                }
                
                self.variables.insert(
                    name.clone(),
                    VariableInfo {
                        var_type: var_type.clone(),
                        state: VariableState::Available,
                    },
                );
                
                Ok(())
            }

            // В функции check_assignment (около строки 327):
            Statement::Assignment { name, value } => {
                // Сначала получаем тип выражения
                let expr_type = self.check_expression(value)?;
    
                // Затем получаем тип переменной
                let var_type = {
                    let var_info = self.variables.get(name)
                        .ok_or_else(|| TypeCheckError::UndefinedVariable {
                            name: name.clone(),
                        })?;
                    var_info.var_type.clone()
                };
    
                // Проверяем совместимость типов
                if !self.types_are_compatible(&var_type, &expr_type) {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: var_type,
                        found: expr_type,
                    });
                }
    
                Ok(())
            }
            
            Statement::Return { value } => {
                let return_type = self.current_function_return
                    .as_ref()
                    .expect("Return outside of function")
                    .clone();
                
                let expr_type = self.check_expression(value)?;
                
                // Разрешаем неявное приведение типов для возвращаемых значений
                if !self.types_are_compatible(&return_type, &expr_type) {
                    return Err(TypeCheckError::InvalidReturnType {
                        expected: return_type,
                        found: expr_type,
                    });
                }
                
                Ok(())
            }
            
            Statement::Expression(expr) => {
                self.check_expression(expr)?;
                Ok(())
            }
            
            Statement::Block { statements } => {
                let old_variables = self.variables.clone();
                
                for stmt in statements {
                    self.check_statement(stmt)?;
                }
                
                self.variables = old_variables;
                Ok(())
            }
            
            Statement::While { condition, body } => {
                let cond_type = self.check_expression(condition)?;
                if cond_type != Type::Bool {
                    return Err(TypeCheckError::NonBooleanCondition {
                        found: cond_type,
                    });
                }
                
                let old_variables = self.variables.clone();
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                self.variables = old_variables;
                
                Ok(())
            }
            
            Statement::If { condition, then_branch, else_branch } => {
                let cond_type = self.check_expression(condition)?;
                if cond_type != Type::Bool {
                    return Err(TypeCheckError::NonBooleanCondition {
                        found: cond_type,
                    });
                }
                
                let old_variables = self.variables.clone();
                for stmt in then_branch {
                    self.check_statement(stmt)?;
                }
                self.variables = old_variables.clone();
                
                if let Some(else_branch) = else_branch {
                    for stmt in else_branch {
                        self.check_statement(stmt)?;
                    }
                }
                self.variables = old_variables;
                
                Ok(())
            }
        }
    }

    fn types_are_compatible(&self, expected: &Type, actual: &Type) -> bool {
        match (expected, actual) {
            // Тот же тип - всегда совместим
            (a, b) if a == b => true,
            
            // Числовые преобразования
            (Type::F32, Type::I32) => true,  // i32 -> f32
            (Type::F64, Type::I32) => true,  // i32 -> f64
            (Type::F64, Type::F32) => true,  // f32 -> f64
            (Type::I64, Type::I32) => true,  // i32 -> i64
            
            // Для арифметических операций
            (Type::F32, Type::F32) => true,
            (Type::I32, Type::I32) => true,
            (Type::F64, Type::F64) => true,
            (Type::I64, Type::I64) => true,
            
            // Во всех остальных случаях - не совместимы
            _ => false,
        }
    }

    fn get_common_numeric_type(&self, left: &Type, right: &Type) -> Option<Type> {
        match (left, right) {
            // Если типы одинаковые - возвращаем тот же тип
            (a, b) if a == b => Some(a.clone()),
            
            // Смешанные числовые типы
            (Type::F32, Type::I32) | (Type::I32, Type::F32) => Some(Type::F32),
            (Type::F64, Type::I32) | (Type::I32, Type::F64) => Some(Type::F64),
            (Type::F64, Type::F32) | (Type::F32, Type::F64) => Some(Type::F64),
            (Type::I64, Type::I32) | (Type::I32, Type::I64) => Some(Type::I64),
            
            // Несовместимые типы
            _ => None,
        }
    }
    
    fn check_expression(&mut self, expression: &Expression) -> TypeCheckResult<Type> {
        match expression {
            Expression::IntegerLiteral(_) => Ok(Type::I32),
            Expression::FloatLiteral(_) => Ok(Type::F32),
            Expression::StringLiteral(_) => Ok(Type::String),
            Expression::BoolLiteral(_) => Ok(Type::Bool),
            
            Expression::Variable(name) => {
                let var_info = self.variables.get(name)
                    .ok_or_else(|| TypeCheckError::UndefinedVariable {
                        name: name.clone(),
                    })?;
                
                if let VariableState::Moved = var_info.state {
                    return Err(TypeCheckError::UseAfterMove {
                        name: name.clone(),
                    });
                }
                
                Ok(var_info.var_type.clone())
            }
            
            Expression::BinaryExpression { left, operator, right } => {
                let left_type = self.check_expression(left)?;
                let right_type = self.check_expression(right)?;
                
                // Проверяем совместимость типов для операторов
                match operator {
                    BinaryOperator::Add |
                    BinaryOperator::Subtract |
                    BinaryOperator::Multiply |
                    BinaryOperator::Divide => {
                        // Для арифметических операций находим общий тип
                        if let Some(common_type) = self.get_common_numeric_type(&left_type, &right_type) {
                            Ok(common_type)
                        } else {
                            Err(TypeCheckError::TypeMismatch {
                                expected: left_type.clone(),
                                found: right_type,
                            })
                        }
                    }
                    
                    BinaryOperator::Eq | 
                    BinaryOperator::Neq |
                    BinaryOperator::Lt |
                    BinaryOperator::Gt |
                    BinaryOperator::Lte |
                    BinaryOperator::Gte => {
                        // Для операторов сравнения типы должны быть совместимы
                        if self.get_common_numeric_type(&left_type, &right_type).is_some() {
                            Ok(Type::Bool)
                        } else {
                            Err(TypeCheckError::TypeMismatch {
                                expected: left_type.clone(),
                                found: right_type,
                            })
                        }
                    }
                    
                    BinaryOperator::And | BinaryOperator::Or => {
                        if left_type != Type::Bool || right_type != Type::Bool {
                            return Err(TypeCheckError::TypeMismatch {
                                expected: Type::Bool,
                                found: if left_type != Type::Bool { left_type } else { right_type },
                            });
                        }
                        Ok(Type::Bool)
                    }
                }
            }
            
            Expression::FunctionCall { name, args } => {
                let function_info = self.functions.get(name)
                    .ok_or_else(|| TypeCheckError::UndefinedFunction {
                        name: name.clone(),
                    })?
                    .clone();
                
                if args.len() != function_info.params.len() {
                    return Err(TypeCheckError::ParameterCountMismatch {
                        expected: function_info.params.len(),
                        found: args.len(),
                    });
                }
                
                for (arg, expected_type) in args.iter().zip(&function_info.params) {
                    let arg_type = self.check_expression(arg)?;
                    if !self.types_are_compatible(expected_type, &arg_type) {
                        return Err(TypeCheckError::TypeMismatch {
                            expected: expected_type.clone(),
                            found: arg_type,
                        });
                    }
                }
                
                Ok(function_info.return_type.clone())
            }
            
            Expression::StructInitialization { struct_name, fields } => {
                let struct_info = self.structs.get(struct_name)
                    .ok_or_else(|| TypeCheckError::UndefinedStruct {
                        name: struct_name.clone(),
                    })?;
                
                // Создаем копию информации о структуре для использования в цикле
                let struct_fields = struct_info.fields.clone();
                
                // Проверяем, что все поля присутствуют и типы совпадают
                for (field_name, field_expr) in fields {
                    let expected_type = struct_fields.get(field_name)
                        .ok_or_else(|| TypeCheckError::UndefinedField {
                            struct_name: struct_name.clone(),
                            field: field_name.clone(),
                        })?;
                    
                    let actual_type = self.check_expression(field_expr)?;
                    if !self.types_are_compatible(expected_type, &actual_type) {
                        return Err(TypeCheckError::TypeMismatch {
                            expected: expected_type.clone(),
                            found: actual_type,
                        });
                    }
                }
                
                Ok(Type::Struct(struct_name.clone()))
            }
            
            Expression::FieldAccess { expression, field_name } => {
                let expr_type = self.check_expression(expression)?;
                
                if let Type::Struct(struct_name) = expr_type {
                    let struct_info = self.structs.get(&struct_name)
                        .ok_or_else(|| TypeCheckError::UndefinedStruct {
                            name: struct_name.clone(),
                        })?;
                    
                    let field_type = struct_info.fields.get(field_name)
                        .ok_or_else(|| TypeCheckError::UndefinedField {
                            struct_name: struct_name.clone(),
                            field: field_name.clone(),
                        })?;
                    
                    Ok(field_type.clone())
                } else {
                    Err(TypeCheckError::TypeMismatch {
                        expected: Type::Struct("any".to_string()),
                        found: expr_type,
                    })
                }
            }
            
            Expression::Move { expression } => {
                let expr_type = self.check_expression(expression)?;
                
                if let Expression::Variable(name) = expression.as_ref() {
                    if let Some(var_info) = self.variables.get_mut(name) {
                        var_info.state = VariableState::Moved;
                    }
                }
                
                Ok(expr_type)
            }
            
            Expression::Borrow { expression, mutable: _ } => {
                let expr_type = self.check_expression(expression)?;
                
                if let Expression::Variable(name) = expression.as_ref() {
                    if let Some(var_info) = self.variables.get_mut(name) {
                        var_info.state = VariableState::Borrowed;
                    }
                }
                
                Ok(expr_type)
            }
            Expression::TypeCast { expression, target_type } => {
                let expr_type = self.check_expression(expression)?;
    
                // Проверяем допустимые преобразования типов
                match (&expr_type, target_type) {
                    (Type::I32, Type::F32) => Ok(Type::F32),
                    (Type::F32, Type::I32) => Ok(Type::I32),
                    (Type::I32, Type::I64) => Ok(Type::I64),
                    (Type::I64, Type::I32) => Ok(Type::I32),
                    (Type::F32, Type::F64) => Ok(Type::F64),
                    (Type::F64, Type::F32) => Ok(Type::F32),
                    (same, same2) if same == same2 => Ok(target_type.clone()), // Тот же тип
                    _ => Err(TypeCheckError::TypeMismatch {
                        expected: target_type.clone(),
                        found: expr_type,
                    }),
                }
            }
        }
    }
}