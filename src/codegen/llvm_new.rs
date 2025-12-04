use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{InitializationConfig, Target, TargetMachine};
use inkwell::types::BasicType;
use inkwell::values::{BasicValue, FunctionValue};
use inkwell::OptimizationLevel;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

use crate::ast;
// В начале файла llvm_new.rs
use crate::ast::Type;

#[derive(Error, Debug)]
pub enum LLVMCodeGenError {
    #[error("LLVM code generation error: {message}")]
    LLVMError { message: String },
    
    #[error("Unsupported type for LLVM: {ty}")]
    UnsupportedType { ty: ast::Type },
    
    #[error("Undefined function: {name}")]
    UndefinedFunction { name: String },
    
    #[error("Undefined variable: {name}")]
    UndefinedVariable { name: String },
}

type LLVMCodeGenResult<T> = Result<T, LLVMCodeGenError>;

pub struct LLVMGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: inkwell::builder::Builder<'ctx>,
    function_values: HashMap<String, FunctionValue<'ctx>>,
    variable_values: HashMap<String, inkwell::values::BasicValueEnum<'ctx>>,
    current_function: Option<FunctionValue<'ctx>>,
}

impl<'ctx> LLVMGenerator<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        
        Self {
            context,
            module,
            builder,
            function_values: HashMap::new(),
            variable_values: HashMap::new(),
            current_function: None,
        }
    }
    
    pub fn generate(program: &ast::Program, output_path: &str) -> LLVMCodeGenResult<()> {
        // Инициализируем LLVM targets
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| LLVMCodeGenError::LLVMError {
                message: format!("Failed to initialize LLVM targets: {}", e),
            })?;
        
        let context = Context::create();
        let mut generator = LLVMGenerator::new(&context, "aetos_module");
        
        // Генерируем код для программы
        generator.generate_program(program)?;
        
        // Валидируем модуль
        if let Err(e) = generator.module.verify() {
            return Err(LLVMCodeGenError::LLVMError {
                message: format!("Module verification failed: {}", e),
            });
        }
        
        // Печатаем IR для отладки
        generator.module.print_to_stderr();
        
        // Компилируем в объектный файл
        generator.compile_to_object(output_path)?;
        
        Ok(())
    }
    
    fn generate_program(&mut self, program: &ast::Program) -> LLVMCodeGenResult<()> {
        // Сначала объявляем все функции
        for function in &program.functions {
            self.declare_function(function)?;
        }
        
        // Затем генерируем тела функций
        for function in &program.functions {
            self.generate_function(function)?;
        }
        
        Ok(())
    }
    
    fn declare_function(&mut self, function: &ast::Function) -> LLVMCodeGenResult<()> {
        let return_type = self.type_to_llvm_type(&function.return_type)?;
        let param_types: Vec<inkwell::types::BasicTypeEnum<'ctx>> = function
            .params
            .iter()
            .map(|p| self.type_to_llvm_type(&p.param_type))
            .collect::<Result<Vec<_>, _>>()?;
        
        let fn_type = return_type.fn_type(&param_types, false);
        let function_value = self.module.add_function(&function.name, fn_type, None);
        
        // Устанавливаем имена параметров
        for (i, param) in function.params.iter().enumerate() {
            function_value.get_nth_param(i as u32)
                .unwrap()
                .set_name(&param.name);
        }
        
        self.function_values.insert(function.name.clone(), function_value);
        Ok(())
    }
    
    fn generate_function(&mut self, function: &ast::Function) -> LLVMCodeGenResult<()> {
        let function_value = self.function_values.get(&function.name)
            .ok_or_else(|| LLVMCodeGenError::UndefinedFunction {
                name: function.name.clone(),
            })?;
        
        self.current_function = Some(*function_value);
        self.variable_values.clear();
        
        // Создаем базовый блок
        let basic_block = self.context.append_basic_block(*function_value, "entry");
        self.builder.position_at_end(basic_block);
        
        // Устанавливаем значения параметров
        for (i, param) in function.params.iter().enumerate() {
            let param_value = function_value.get_nth_param(i as u32).unwrap();
            param_value.set_name(&param.name);
            
            // Создаем аллокацию для параметра
            let alloca = self.build_alloca(param_value.get_type(), &param.name);
            self.builder.build_store(alloca, param_value).unwrap();
            
            self.variable_values.insert(param.name.clone(), alloca.as_basic_value_enum());
        }
        
        // Генерируем тело функции
        for statement in &function.body {
            self.generate_statement(statement)?;
        }
        
        // Если функция не заканчивается return, добавляем return
        if let Some(ast::Statement::Return { .. }) = function.body.last() {
            // Уже есть return
        } else {
            match function.return_type {
                ast::Type::Void => {
                    self.builder.build_return(None).unwrap();
                }
                _ => {
                    let default_val = match function.return_type {
                        ast::Type::I32 => self.context.i32_type().const_int(0, false),
                        ast::Type::I64 => self.context.i64_type().const_int(0, false),
                        ast::Type::F32 => self.context.f32_type().const_float(0.0),
                        ast::Type::F64 => self.context.f64_type().const_float(0.0),
                        ast::Type::Bool => self.context.bool_type().const_int(0, false),
                        ast::Type::String => self.context.i8_type().ptr_type(inkwell::AddressSpace::Generic).const_null(),
                        ast::Type::Struct(_) => self.context.i8_type().ptr_type(inkwell::AddressSpace::Generic).const_null(),
                        ast::Type::Void => unreachable!(),
                    };
                    self.builder.build_return(Some(&default_val)).unwrap();
                }
            }
        }
        
        Ok(())
    }
    
    fn generate_statement(&mut self, statement: &ast::Statement) -> LLVMCodeGenResult<()> {
        match statement {
            ast::Statement::VariableDeclaration { name, var_type, value } => {
                let value_llvm = self.generate_expression(value)?;
                let alloca = self.build_alloca(value_llvm.get_type(), name);
                self.builder.build_store(alloca, value_llvm).unwrap();
                
                self.variable_values.insert(name.clone(), alloca.as_basic_value_enum());
                Ok(())
            }
            
            ast::Statement::Return { value } => {
                let return_value = self.generate_expression(value)?;
                self.builder.build_return(Some(&return_value)).unwrap();
                Ok(())
            }
            
            ast::Statement::Expression(expr) => {
                self.generate_expression(expr)?;
                Ok(())
            }
            
            ast::Statement::Block { statements } => {
                for stmt in statements {
                    self.generate_statement(stmt)?;
                }
                Ok(())
            }
            
            ast::Statement::While { condition, body } => {
                let function = self.current_function.unwrap();
                let condition_block = self.context.append_basic_block(function, "while_condition");
                let body_block = self.context.append_basic_block(function, "while_body");
                let end_block = self.context.append_basic_block(function, "while_end");
                
                // Переходим к блоку условия
                self.builder.build_unconditional_branch(condition_block).unwrap();
                self.builder.position_at_end(condition_block);
                
                // Генерируем условие
                let cond_value = self.generate_expression(condition)?;
                let bool_cond = match cond_value.get_type() {
                    t if t.is_int_type() => {
                        self.builder.build_int_compare(
                            inkwell::IntPredicate::NE,
                            cond_value.into_int_value(),
                            self.context.i32_type().const_int(0, false),
                            "while_cond",
                        ).unwrap()
                    }
                    _ => cond_value.into_int_value(), // предполагаем, что уже bool
                };
                
                self.builder.build_conditional_branch(bool_cond, body_block, end_block).unwrap();
                
                // Тело цикла
                self.builder.position_at_end(body_block);
                for stmt in body {
                    self.generate_statement(stmt)?;
                }
                self.builder.build_unconditional_branch(condition_block).unwrap();
                
                // Конец цикла
                self.builder.position_at_end(end_block);
                Ok(())
            }
            
            ast::Statement::If { condition, then_branch, else_branch } => {
                let function = self.current_function.unwrap();
                let then_block = self.context.append_basic_block(function, "if_then");
                let else_block = self.context.append_basic_block(function, "if_else");
                let end_block = self.context.append_basic_block(function, "if_end");
                
                // Генерируем условие
                let cond_value = self.generate_expression(condition)?;
                let bool_cond = match cond_value.get_type() {
                    t if t.is_int_type() => {
                        self.builder.build_int_compare(
                            inkwell::IntPredicate::NE,
                            cond_value.into_int_value(),
                            self.context.i32_type().const_int(0, false),
                            "if_cond",
                        ).unwrap()
                    }
                    _ => cond_value.into_int_value(),
                };
                
                if else_branch.is_some() {
                    self.builder.build_conditional_branch(bool_cond, then_block, else_block).unwrap();
                } else {
                    self.builder.build_conditional_branch(bool_cond, then_block, end_block).unwrap();
                }
                
                // Then branch
                self.builder.position_at_end(then_block);
                for stmt in then_branch {
                    self.generate_statement(stmt)?;
                }
                self.builder.build_unconditional_branch(end_block).unwrap();
                
                // Else branch
                if let Some(else_branch) = else_branch {
                    self.builder.position_at_end(else_block);
                    for stmt in else_branch {
                        self.generate_statement(stmt)?;
                    }
                    self.builder.build_unconditional_branch(end_block).unwrap();
                }
                
                // End block
                self.builder.position_at_end(end_block);
                Ok(())
            }
        }
    }
    
    fn generate_expression(&self, expression: &ast::Expression) -> LLVMCodeGenResult<inkwell::values::BasicValueEnum<'ctx>> {
        match expression {
            ast::Expression::IntegerLiteral(value) => {
                Ok(self.context.i32_type().const_int(*value as u64, false).into())
            }
            
            ast::Expression::FloatLiteral(value) => {
                Ok(self.context.f32_type().const_float(*value as f64).into())
            }
            
            ast::Expression::StringLiteral(_value) => {
                // Пока возвращаем null pointer для строк
                Ok(self.context.i8_type().ptr_type(inkwell::AddressSpace::Generic).const_null().into())
            }
            
            ast::Expression::BoolLiteral(value) => {
                Ok(self.context.bool_type().const_int(if *value { 1 } else { 0 }, false).into())
            }
            
            ast::Expression::Variable(name) => {
                let variable = self.variable_values.get(name)
                    .ok_or_else(|| LLVMCodeGenError::UndefinedVariable {
                        name: name.clone(),
                    })?;
                
                // Загружаем значение из аллокации
                Ok(self.builder.build_load(variable.get_type(), *variable, name).unwrap())
            }
            
            ast::Expression::BinaryExpression { left, operator, right } => {
                let left_val = self.generate_expression(left)?;
                let right_val = self.generate_expression(right)?;
                
                match operator {
                    ast::BinaryOperator::Add => {
                        if left_val.get_type().is_int_type() {
                            Ok(self.builder.build_int_add(
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "addtmp",
                            ).unwrap().into())
                        } else {
                            Ok(self.builder.build_float_add(
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "faddtmp",
                            ).unwrap().into())
                        }
                    }
                    
                    ast::BinaryOperator::Subtract => {
                        if left_val.get_type().is_int_type() {
                            Ok(self.builder.build_int_sub(
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "subtmp",
                            ).unwrap().into())
                        } else {
                            Ok(self.builder.build_float_sub(
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "fsubtmp",
                            ).unwrap().into())
                        }
                    }
                    
                    ast::BinaryOperator::Multiply => {
                        if left_val.get_type().is_int_type() {
                            Ok(self.builder.build_int_mul(
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "multmp",
                            ).unwrap().into())
                        } else {
                            Ok(self.builder.build_float_mul(
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "fmultmp",
                            ).unwrap().into())
                        }
                    }
                    
                    ast::BinaryOperator::Divide => {
                        if left_val.get_type().is_int_type() {
                            Ok(self.builder.build_int_signed_div(
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "divtmp",
                            ).unwrap().into())
                        } else {
                            Ok(self.builder.build_float_div(
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "fdivtmp",
                            ).unwrap().into())
                        }
                    }
                    
                    ast::BinaryOperator::Eq => {
                        if left_val.get_type().is_int_type() {
                            Ok(self.builder.build_int_compare(
                                inkwell::IntPredicate::EQ,
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "eqtmp",
                            ).unwrap().into())
                        } else {
                            Ok(self.builder.build_float_compare(
                                inkwell::FloatPredicate::OEQ,
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "feqtmp",
                            ).unwrap().into())
                        }
                    }
                    
                    ast::BinaryOperator::Neq => {
                        if left_val.get_type().is_int_type() {
                            Ok(self.builder.build_int_compare(
                                inkwell::IntPredicate::NE,
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "neqtmp",
                            ).unwrap().into())
                        } else {
                            Ok(self.builder.build_float_compare(
                                inkwell::FloatPredicate::ONE,
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "fneqtmp",
                            ).unwrap().into())
                        }
                    }
                    
                    ast::BinaryOperator::Lt => {
                        if left_val.get_type().is_int_type() {
                            Ok(self.builder.build_int_compare(
                                inkwell::IntPredicate::SLT,
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "lttmp",
                            ).unwrap().into())
                        } else {
                            Ok(self.builder.build_float_compare(
                                inkwell::FloatPredicate::OLT,
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "flttmp",
                            ).unwrap().into())
                        }
                    }
                    
                    ast::BinaryOperator::Gt => {
                        if left_val.get_type().is_int_type() {
                            Ok(self.builder.build_int_compare(
                                inkwell::IntPredicate::SGT,
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "gttmp",
                            ).unwrap().into())
                        } else {
                            Ok(self.builder.build_float_compare(
                                inkwell::FloatPredicate::OGT,
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "fgttmp",
                            ).unwrap().into())
                        }
                    }
                    
                    ast::BinaryOperator::Lte => {
                        if left_val.get_type().is_int_type() {
                            Ok(self.builder.build_int_compare(
                                inkwell::IntPredicate::SLE,
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "ltetmp",
                            ).unwrap().into())
                        } else {
                            Ok(self.builder.build_float_compare(
                                inkwell::FloatPredicate::OLE,
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "fltetmp",
                            ).unwrap().into())
                        }
                    }
                    
                    ast::BinaryOperator::Gte => {
                        if left_val.get_type().is_int_type() {
                            Ok(self.builder.build_int_compare(
                                inkwell::IntPredicate::SGE,
                                left_val.into_int_value(),
                                right_val.into_int_value(),
                                "gtetmp",
                            ).unwrap().into())
                        } else {
                            Ok(self.builder.build_float_compare(
                                inkwell::FloatPredicate::OGE,
                                left_val.into_float_value(),
                                right_val.into_float_value(),
                                "fgtetmp",
                            ).unwrap().into())
                        }
                    }
                    
                    ast::BinaryOperator::And => {
                        Ok(self.builder.build_and(
                            left_val.into_int_value(),
                            right_val.into_int_value(),
                            "andtmp",
                        ).unwrap().into())
                    }
                    
                    ast::BinaryOperator::Or => {
                        Ok(self.builder.build_or(
                            left_val.into_int_value(),
                            right_val.into_int_value(),
                            "ortmp",
                        ).unwrap().into())
                    }
                }
            }

            ast::Expression::TypeCast { expression, target_type } => {
                let value = self.generate_expression(expression)?;
                let target_llvm_type = self.type_to_llvm_type(target_type)?;
    
                match (value.get_type(), target_llvm_type) {
                    (t1, t2) if t1 == t2 => Ok(value), // Типы совпадают
                    (t1, t2) if t1.is_int_type() && t2.is_float_type() => {
                        Ok(self.builder.build_signed_int_to_float(
                            value.into_int_value(),
                            target_llvm_type.into_float_type(),
                            "cast",
                        ).unwrap().into())
                    }
                    (t1, t2) if t1.is_float_type() && t2.is_int_type() => {
                        Ok(self.builder.build_float_to_signed_int(
                            value.into_float_value(),
                            target_llvm_type.into_int_type(),
                            "cast",
                        ).unwrap().into())
                    }
                    _ => Ok(value), // Пока просто возвращаем значение для других случаев
                }
            }
            
            ast::Expression::FunctionCall { name, args } => {
                let function = self.function_values.get(name)
                    .ok_or_else(|| LLVMCodeGenError::UndefinedFunction {
                        name: name.clone(),
                    })?;
                
                let arg_values: Vec<inkwell::values::BasicValueEnum<'ctx>> = args
                    .iter()
                    .map(|arg| self.generate_expression(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                
                Ok(self.builder.build_call(*function, &arg_values, "calltmp")
                   .unwrap()
                   .try_as_basic_value()
                   .left()
                   .unwrap())
            }
            
            // Пока упрощенные реализации для остальных выражений
            ast::Expression::StructInitialization { .. } => {
                Ok(self.context.i32_type().const_int(0, false).into())
            }
            
            ast::Expression::FieldAccess { .. } => {
                Ok(self.context.i32_type().const_int(0, false).into())
            }
            
            ast::Expression::Move { expression } => {
                self.generate_expression(expression)
            }
            
            ast::Expression::Borrow { expression, .. } => {
                self.generate_expression(expression)
            }
        }
    }
    
    fn build_alloca(&self, ty: inkwell::types::BasicTypeEnum<'ctx>, name: &str) -> inkwell::values::PointerValue<'ctx> {
        let builder = self.context.create_builder();
        let entry_block = self.current_function.unwrap().get_first_basic_block().unwrap();
        
        if let Some(first_instr) = entry_block.get_first_instruction() {
            builder.position_before(&first_instr);
        } else {
            builder.position_at_end(entry_block);
        }
        
        builder.build_alloca(ty, name).unwrap()
    }
    
    fn type_to_llvm_type(&self, ty: &ast::Type) -> LLVMCodeGenResult<inkwell::types::BasicTypeEnum<'ctx>> {
        match ty {
            ast::Type::I32 => Ok(self.context.i32_type().as_basic_type_enum()),
            ast::Type::I64 => Ok(self.context.i64_type().as_basic_type_enum()),
            ast::Type::F32 => Ok(self.context.f32_type().as_basic_type_enum()),
            ast::Type::F64 => Ok(self.context.f64_type().as_basic_type_enum()),
            ast::Type::Bool => Ok(self.context.bool_type().as_basic_type_enum()),
            ast::Type::String => Ok(self.context.i8_type().ptr_type(inkwell::AddressSpace::Generic).as_basic_type_enum()),
            ast::Type::Struct(_) => Ok(self.context.i8_type().ptr_type(inkwell::AddressSpace::Generic).as_basic_type_enum()),
            ast::Type::Void => Ok(self.context.void_type().as_basic_type_enum()),
        }
    }
    
    fn compile_to_object(&self, output_path: &str) -> LLVMCodeGenResult<()> {
        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple)
            .map_err(|e| LLVMCodeGenError::LLVMError {
                message: format!("Failed to get target: {}", e),
            })?;
        
        let cpu = TargetMachine::get_host_cpu_name().to_string();
        let features = TargetMachine::get_host_cpu_features().to_string();
        
        let target_machine = target
            .create_target_machine(
                &target_triple,
                &cpu,
                &features,
                OptimizationLevel::Default,
                inkwell::targets::RelocMode::Default,
                inkwell::targets::CodeModel::Default,
            )
            .ok_or_else(|| LLVMCodeGenError::LLVMError {
                message: "Failed to create target machine".to_string(),
            })?;
        
        // Компилируем в объектный файл
        target_machine
            .write_to_file(&self.module, inkwell::targets::FileType::Object, Path::new(output_path))
            .map_err(|e| LLVMCodeGenError::LLVMError {
                message: format!("Failed to write object file: {}", e),
            })?;
        
        println!("Generated native object file: {}", output_path);
        Ok(())
    }
}