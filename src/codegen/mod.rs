// file name: mod.rs
pub mod wasm;

use crate::ast::Program;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodeGenError {
    #[error("WASM error")]
    WASM(String),
}

pub trait CodeGenerator {
    fn generate(program: &Program, output_path: &str) -> Result<(), CodeGenError>;
}