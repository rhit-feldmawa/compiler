#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub grammar); // synthesized by LALRPOP
mod ast;
mod symbol_table;
mod test;

fn main() {
    // let args: Vec<String> = env::args().collect();
    // let file_path = &args[1];
    // let contents = fs::read_to_string(file_path).unwrap();
    // let tree = grammar::ProgramParser::new().parse(&contents);
    // println!("{:?}", tree);
    // println!("{}", *identifier_type);
}
