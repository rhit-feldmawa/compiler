#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub grammar); // synthesized by LALRPOP
mod ast;
mod symbol_table;
mod symbol_table_ast;
mod test;
mod codegen;

use std::env;
use std::fs;
use crate::symbol_table_ast::{typecheck_program};

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    let contents = fs::read_to_string(file_path).unwrap();
    let program = grammar::ProgramParser::new().parse(&contents).unwrap();
    let typecheck_result = typecheck_program(&program);
    // println!("{:?}", tree);
    // println!("{}", *identifier_type);
}
