use inkwell::context::Context;
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::types::BasicType;
use inkwell::values::{BasicValue, BasicValueEnum, FunctionValue};
use inkwell::AddressSpace;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use thiserror::Error;

use crate::ast::*;

#[derive(Error, Debug)]
pub enum CodeGenError {
    #[error("LLVM code generation error: {message}")]
    LLVMError { message: String },
    
    #[error("Undefined function: {name}")]
    UndefinedFunction { name: String },
    
    #[error("Undefined variable: {name}")]
    UndefinedVariable { name: String },
    
    #[error("Invalid type for code generation: {ty}")]
    InvalidType { ty: Type },
}

type CodeGenResult<T> = Result<T, CodeGenError>;

pub struct LLVMGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: inkwell::builder::Builder<'ctx>,
    named_values: HashMap<String, inkwell::values::BasicValueEnum<'ctx>>,
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
            named_values: HashMap::new(),
            current_function: None,
        }
    }
    
    pub fn generate(program: &Program, output_path: &str) -> CodeGenResult<()> {
        Target::initialize_native(&InitializationConfig::default())?;
        
        let context = Context::create();
        let mut generator = LLVMGenerator::new(&context, "aetos_module");
        generator.add_builtin_functions();
        
        generator.generate_program(program)?;
        
        if let Err(e) = generator.module.verify() {
            return Err(CodeGenError::LLVMError {
                message: format!("Module verification failed: {}", e),
            });
        }
        
        generator.module.print_to_stderr();
        
        generator.compile_to_object(output_path)?;
        
        Ok(())
    }
    
    pub fn generate_embedded(program: &Program, output_path: &str) -> CodeGenResult<()> {
        let context = Context::create();
        let mut generator = LLVMGenerator::new(&context, "aetos_embedded");
        generator.add_embedded_functions();
        
        generator.generate_program(program)?;
        
        if let Err(e) = generator.module.verify() {
            return Err(CodeGenError::LLVMError {
                message: format!("Module verification failed: {}", e),
            });
        }
        
        generator.compile_to_embedded(output_path)?;
        
        Ok(())
    }
    
    fn generate_program(&mut self, program: &Program) -> CodeGenResult<()> {
        for function in &program.functions {
            self.declare_function(function)?;
        }
        
        for function in &program.functions {
            self.generate_function(function)?;
        }
        
        Ok(())
    }
    
    fn declare_function(&self, function: &Function) -> CodeGenResult<()> {
        let return_type = self.type_to_llvm_type(&function.return_type)?;
        let param_types: Vec<inkwell::types::BasicTypeEnum<'ctx>> = function
            .params
            .iter()
            .map(|p| self.type_to_llvm_type(&p.param_type))
            .collect::<Result<Vec<_>, _>>()?;
        
        let fn_type = return_type.fn_type(&param_types, false);
        let function_value = self.module.add_function(&function.name, fn_type, None);
        
        for (i, param) in function.params.iter().enumerate() {
            function_value.get_nth_param(i as u32)
                .unwrap()
                .set_name(&param.name);
        }
        
        Ok(())
    }
    
    fn generate_function(&mut self, function: &Function) -> CodeGenResult<()> {
        let function_value = self.module.get_function(&function.name)
            .ok_or_else(|| CodeGenError::UndefinedFunction {
                name: function.name.clone(),
            })?;
        
        self.current_function = Some(function_value);
        
        let basic_block = self.context.append_basic_block(function_value, "entry");
        self.builder.position_at_end(basic_block);
        
        self.named_values.clear();
        
        for (i, param) in function.params.iter().enumerate() {
            let param_value = function_value.get_nth_param(i as u32).unwrap();
            param_value.set_name(&param.name);
            
            let alloca = self.build_alloca(param_value.get_type(), &param.name);
            self.builder.build_store(alloca, param_value).unwrap();
            
            self.named_values.insert(param.name.clone(), alloca.as_basic_value_enum());
        }
        
        for statement in &function.body {
            self.generate_statement(statement)?;
        }
        
        if let Type::Void = function.return_type {
            self.builder.build_return(None).unwrap();
        } else {
            if function.name == "main" && matches!(function.return_type, Type::I32) {
                let zero = self.context.i32_type().const_int(0, false);
                self.builder.build_return(Some(&zero)).unwrap();
            }
        }
        
        Ok(())
    }
    
    fn generate_statement(&mut self, statement: &Statement) -> CodeGenResult<()> {
        match statement {
            Statement::VariableDeclaration { name, var_type: _, value } => {
                let value_llvm = self.generate_expression(value)?;
                let alloca = self.build_alloca(value_llvm.get_type(), name);
                self.builder.build_store(alloca, value_llvm).unwrap();
                
                self.named_values.insert(name.clone(), alloca.as_basic_value_enum());
                Ok(())
            }
            
            Statement::Return { value } => {
                let return_value = self.generate_expression(value)?;
                self.builder.build_return(Some(&return_value)).unwrap();
                Ok(())
            }
            
            Statement::Expression(expr) => {
                self.generate_expression(expr)?;
                Ok(())
            }
            
            Statement::Block { statements } => {
                for stmt in statements {
                    self.generate_statement(stmt)?;
                }
                Ok(())
            }
        }
    }
    
    fn generate_expression(&self, expression: &Expression) -> CodeGenResult<BasicValueEnum<'ctx>> {
        match expression {
            Expression::IntegerLiteral(value) => {
                Ok(self.context.i32_type().const_int(*value as u64, false).into())
            }
            
            Expression::Variable(name) => {
                let variable = self.named_values.get(name)
                    .ok_or_else(|| CodeGenError::UndefinedVariable {
                        name: name.clone(),
                    })?;
                
                Ok(self.builder.build_load(variable.get_type(), *variable, name).unwrap())
            }
            
            Expression::BinaryExpression { left, operator, right } => {
                let left_val = self.generate_expression(left)?;
                let right_val = self.generate_expression(right)?;
                
                match operator {
                    BinaryOperator::Add => {
                        Ok(self.builder.build_int_add(
                            left_val.into_int_value(),
                            right_val.into_int_value(),
                            "addtmp",
                        ).unwrap().into())
                    }
                    
                    BinaryOperator::Subtract => {
                        Ok(self.builder.build_int_sub(
                            left_val.into_int_value(),
                            right_val.into_int_value(),
                            "subtmp",
                        ).unwrap().into())
                    }
                    
                    BinaryOperator::Multiply => {
                        Ok(self.builder.build_int_mul(
                            left_val.into_int_value(),
                            right_val.into_int_value(),
                            "multmp",
                        ).unwrap().into())
                    }
                    
                    BinaryOperator::Divide => {
                        Ok(self.builder.build_int_signed_div(
                            left_val.into_int_value(),
                            right_val.into_int_value(),
                            "divtmp",
                        ).unwrap().into())
                    }
                    
                    BinaryOperator::Eq => {
                        Ok(self.builder.build_int_compare(
                            inkwell::IntPredicate::EQ,
                            left_val.into_int_value(),
                            right_val.into_int_value(),
                            "eqtmp",
                        ).unwrap().into())
                    }
                    
                    BinaryOperator::Neq => {
                        Ok(self.builder.build_int_compare(
                            inkwell::IntPredicate::NE,
                            left_val.into_int_value(),
                            right_val.into_int_value(),
                            "neqtmp",
                        ).unwrap().into())
                    }
                }
            }
            
            Expression::FunctionCall { name, args } => {
                let function = self.module.get_function(name)
                    .ok_or_else(|| CodeGenError::UndefinedFunction {
                        name: name.clone(),
                    })?;
                
                let arg_values: Vec<BasicValueEnum<'ctx>> = args
                    .iter()
                    .map(|arg| self.generate_expression(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                
                Ok(self.builder.build_call(function, &arg_values, "calltmp")
                   .unwrap()
                   .try_as_basic_value()
                   .left()
                   .unwrap())
            }
            
            Expression::Move { expression } => {
                self.generate_expression(expression)
            }
            
            Expression::Borrow { expression, mutable: _ } => {
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
    
    fn type_to_llvm_type(&self, ty: &Type) -> CodeGenResult<inkwell::types::BasicTypeEnum<'ctx>> {
        match ty {
            Type::I32 => Ok(self.context.i32_type().as_basic_type_enum()),
            Type::I64 => Ok(self.context.i64_type().as_basic_type_enum()),
            Type::F32 => Ok(self.context.f32_type().as_basic_type_enum()),
            Type::F64 => Ok(self.context.f64_type().as_basic_type_enum()),
            Type::Bool => Ok(self.context.bool_type().as_basic_type_enum()),
            Type::Void => Ok(self.context.void_type().as_basic_type_enum()),
        }
    }
    
    fn add_builtin_functions(&self) {
        // Add standard library functions if needed
    }
    
    fn add_embedded_functions(&self) {
        let void_type = self.context.void_type();
        let i32_type = self.context.i32_type();
        
        let gpio_set_type = void_type.fn_type(&[i32_type.into(), i32_type.into()], false);
        self.module.add_function("gpio_set", gpio_set_type, None);
        
        let gpio_toggle_type = void_type.fn_type(&[i32_type.into()], false);
        self.module.add_function("gpio_toggle", gpio_toggle_type, None);
        
        let delay_type = void_type.fn_type(&[i32_type.into()], false);
        self.module.add_function("delay", delay_type, None);
    }
    
    fn compile_to_object(&self, output_path: &str) -> CodeGenResult<()> {
        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple)
            .map_err(|e| CodeGenError::LLVMError {
                message: format!("Failed to get target: {}", e),
            })?;
        
        let cpu = TargetMachine::get_host_cpu_name().to_string();
        let features = TargetMachine::get_host_cpu_features().to_string();
        
        let target_machine = target
            .create_target_machine(
                &target_triple,
                &cpu,
                &features,
                inkwell::OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| CodeGenError::LLVMError {
                message: "Failed to create target machine".to_string(),
            })?;
        
        target_machine
            .write_to_file(&self.module, FileType::Object, Path::new(output_path))
            .map_err(|e| CodeGenError::LLVMError {
                message: format!("Failed to write object file: {}", e),
            })?;
        
        Ok(())
    }
    
    fn compile_to_embedded(&self, output_path: &str) -> CodeGenResult<()> {
        let target_triple = "arm-none-eabi";
        let target = Target::from_triple(target_triple)
            .map_err(|e| CodeGenError::LLVMError {
                message: format!("Failed to get ARM target: {}", e),
            })?;
        
        let target_machine = target
            .create_target_machine(
                target_triple,
                "cortex-m3",
                "+thumb-mode",
                inkwell::OptimizationLevel::Size,
                RelocMode::Static,
                CodeModel::Small,
            )
            .ok_or_else(|| CodeGenError::LLVMError {
                message: "Failed to create embedded target machine".to_string(),
            })?;
        
        let asm_output = format!("{}.s", output_path);
        target_machine
            .write_to_file(&self.module, FileType::Assembly, Path::new(&asm_output))
            .map_err(|e| CodeGenError::LLVMError {
                message: format!("Failed to write assembly file: {}", e),
            })?;
        
        println!("Generated assembly for embedded: {}", asm_output);
        
        Ok(())
    }
}

impl From<inkwell::support::LLVMString> for CodeGenError {
    fn from(err: inkwell::support::LLVMString) -> Self {
        CodeGenError::LLVMError {
            message: err.to_string(),
        }
    }
}
