#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub grammar); // synthesized by LALRPOP
mod ast;
mod symbol_table;
mod typecheck;
mod test;
mod codegen;

use std::env;
use std::fs;
use crate::typecheck::{typecheck_program, TypecheckProgramResult};
use crate::codegen::{codegen};

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    let contents = fs::read_to_string(file_path).unwrap();
    let program = grammar::ProgramParser::new().parse(&contents).unwrap();
    let typecheck_result = typecheck_program(&program);
    match typecheck_result {
        TypecheckProgramResult::Success => {
            codegen(*program, file_path);
        },
        TypecheckProgramResult::Failure(reason) => {
            println!("Error: {}", reason);
        }
    }
    // println!("{:?}", tree);
    // println!("{}", *identifier_type);
}
