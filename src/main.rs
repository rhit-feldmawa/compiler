#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub grammar); // synthesized by LALRPOP
mod ast;
mod test;

// #[test]
// fn var_declarations() {
//     let test_string = "
//         int test;
//         void test2[4];
//     ";
//     let program = grammar::ProgramParser::new().parse(test_string).unwrap();
//     assert_eq!(
//         *program.var_declarations[0],
//         VarDeclaration::VarDeclaration(Box::new(IdentifierType::Int), "test".to_string())
//     );
//     assert_eq!(
//         *program.var_declarations[1],
//         VarDeclaration::ArrDeclaration(Box::new(IdentifierType::Void), "test2".to_string(), 4)
//     );
//     assert_eq!(program.fun_declarations.len(), 0)
// }

// #[test]
// fn expression() {
//     let test_string = "0*(1+2)+(3*4-e(1))";
//     let expression = grammar::ExpressionParser::new().parse(test_string).unwrap();
//     use Expression::*;
//     assert_eq!(
//         *expression,
//         Operation(
//             Box::new(Operation(
//                 Box::new(IntegerLiteral(0)),
//                 Operator::Mul,
//                 Box::new(Operation(
//                     Box::new(IntegerLiteral(1)),
//                     Operator::Add,
//                     Box::new(IntegerLiteral(2))
//                 ))
//             )),
//             Operator::Add,
//             Box::new(Operation(
//                 Box::new(Operation(
//                     Box::new(IntegerLiteral(3)),
//                     Operator::Mul,
//                     Box::new(IntegerLiteral(4))
//                 )),
//                 Operator::Sub,
//                 Box::new(Call(Box::new(FunctionCall {
//                     name: "e".to_string(),
//                     args: vec![Box::new(IntegerLiteral(1))]
//                 })))
//             ))
//         )
//     );
// }

// #[test]
// fn left_associative_expression() {
//     let test_string = "0+1-2";
//     let expression = grammar::ExpressionParser::new().parse(test_string).unwrap();
//     use Expression::*;
//     assert_eq!(
//         *expression,
//         Operation(
//             Box::new(Operation(
//                 Box::new(IntegerLiteral(0)),
//                 Operator::Add,
//                 Box::new(IntegerLiteral(1))
//             )),
//             Operator::Sub,
//             Box::new(IntegerLiteral(2))
//         )
//     );
// }

// #[test]
// fn function_declarations() {
//     let test_string = "
//         int ident(int ab) {}
//         void ident2(void a, int b[]) {}
//     ";
//     let program = grammar::ProgramParser::new().parse(test_string).unwrap();
//     assert_eq!(program.var_declarations.len(), 0);
//     assert_eq!(
//         *program.fun_declarations[0],
//         FunctionDeclaration {
//             return_type: Box::new(IdentifierType::Int),
//             function_name: "ident".to_string(),
//             params: vec![Box::new(Param::Var(
//                 Box::new(IdentifierType::Int),
//                 "ab".to_string()
//             ))],
//             body: Box::new(CompoundStatement {
//                 declarations: Vec::new(),
//                 statements: vec![]
//             })
//         }
//     );
//     assert_eq!(
//         *program.fun_declarations[1],
//         FunctionDeclaration {
//             return_type: Box::new(IdentifierType::Void),
//             function_name: "ident2".to_string(),
//             params: vec![
//                 Box::new(Param::Var(Box::new(IdentifierType::Void), "a".to_string())),
//                 Box::new(Param::ArrVar(
//                     Box::new(IdentifierType::Int),
//                     "b".to_string()
//                 )),
//             ],
//             body: Box::new(CompoundStatement {
//                 declarations: Vec::new(),
//                 statements: vec![]
//             })
//         }
//     );
// }
fn main() {
    // let args: Vec<String> = env::args().collect();
    // let file_path = &args[1];
    // let contents = fs::read_to_string(file_path).unwrap();
    // let tree = grammar::ProgramParser::new().parse(&contents);
    // println!("{:?}", tree);
    // println!("{}", *identifier_type);
}
