#[cfg(test)]
mod tests {
    use crate::ast::{
        CompoundStatement, Expression, FunctionCall, FunctionDeclaration, IdentifierType,
        IfStatement, Operator, Param, Statement, Var, VarDeclaration, WhileStatement,
    };
    use crate::symbol_table_ast::{typecheck_program, TypecheckProgramResult};

    lalrpop_mod!(pub grammar); // synthesized by LALRPOP

    static EXPRESSION_TEST: &str = "0*(1+2)+(3*4-e(1))";

    static ASSOCIATIVITY_TEST: &str = "0+1-2";

    static VAR_DECLARATIONS_TEST: &str = "
        int test;
        int test2[4];
    ";

    static FUN_DECLARATIONS_TEST: &str = "
        int ident(int ab) {}
        void ident2(void a, int b[]) {}
    ";

    static ASSIGNMENT_TEST: &str = "
        int func() {
            a = 5;
        }
";

    static WHILE_TEST: &str = "
        int func() {
            while (a) {}
        }
";

    static IF_TEST: &str = "
        int func() {
            if (a) {
                return 0;
            } else {
                return 1;
            }
        }
";

    static DANGLING_ELSE: &str = "
        int func() {
            if (a)
                if (b)
                    ;
                else
                    ;
        }
";

    #[test]
    fn dangling_else() {
        let program = grammar::ProgramParser::new()
            .parse(DANGLING_ELSE)
            .unwrap();
        let isTypeCorrectly = typecheck_program(&program);
        assert_eq!(
            isTypeCorrectly,
            TypecheckProgramResult::Success
        );
        assert_eq!(
            program.fun_declarations[0].body.statements[0],
            Box::new(Statement::IfStatement(Box::new(IfStatement::IfStmt(
                Box::new(Expression::Var(
                    Box::new(Var::Var("a".to_string()))
                )),
                Box::new(Statement::IfStatement(Box::new(IfStatement::IfElseStmt(
                    Box::new(Expression::Var(
                        Box::new(Var::Var("b".to_string()))
                    )),
                    Box::new(Statement::EmptyStatement),
                    Box::new(Statement::EmptyStatement)
                ))))
        )))));
    }

    #[test]
    fn assignment() {
        let program = grammar::ProgramParser::new()
            .parse(ASSIGNMENT_TEST)
            .unwrap();
        assert_eq!(
            program.fun_declarations[0].body.statements[0],
            Box::new(Statement::Expression(Box::new(Expression::Assignment(
                Box::new(Var::Var("a".to_string())),
                Box::new(Expression::IntegerLiteral(5))
            ))))
        );
    }

    #[test]
    fn while_statement() {
        let program = grammar::ProgramParser::new().parse(WHILE_TEST).unwrap();
        assert_eq!(
            program.fun_declarations[0].body.statements[0],
            Box::new(Statement::WhileStatement(Box::new(WhileStatement {
                condition: Box::new(Expression::Var(Box::new(Var::Var("a".to_string())))),
                statement: Box::new(Statement::CompoundStatement(Box::new(CompoundStatement {
                    declarations: Vec::new(),
                    statements: Vec::new()
                })))
            })))
        );
    }
    #[test]
    fn if_statement() {
        let program = grammar::ProgramParser::new().parse(IF_TEST).unwrap();
        assert_eq!(
            *program.fun_declarations[0].body.statements[0],
            Statement::IfStatement(Box::new(IfStatement::IfElseStmt(
                Box::new(Expression::Var(Box::new(Var::Var("a".to_string())))),
                Box::new(Statement::CompoundStatement(Box::new(CompoundStatement {
                    declarations: Vec::new(),
                    statements: vec![Box::new(Statement::ReturnStatement(Box::new(
                        Expression::IntegerLiteral(0)
                    )))]
                }))),
                Box::new(Statement::CompoundStatement(Box::new(CompoundStatement {
                    declarations: Vec::new(),
                    statements: vec![Box::new(Statement::ReturnStatement(Box::new(
                        Expression::IntegerLiteral(1)
                    )))]
                }))),
            )))
        );
    }

    #[test]
    fn var_declarations() {
        let program = grammar::ProgramParser::new()
            .parse(VAR_DECLARATIONS_TEST)
            .unwrap();
        assert_eq!(
            program.var_declarations[0],
            VarDeclaration::VarDeclaration(IdentifierType::Int, "test".to_string())
        );
        assert_eq!(
            program.var_declarations[1],
            VarDeclaration::ArrDeclaration(IdentifierType::Void, "test2".to_string(), 4)
        );
        assert_eq!(program.fun_declarations.len(), 0)
    }

    #[test]
    fn expression() {
        let expression = grammar::ExpressionParser::new()
            .parse(EXPRESSION_TEST)
            .unwrap();
        use Expression::*;
        assert_eq!(
            *expression,
            Operation(
                Box::new(Operation(
                    Box::new(IntegerLiteral(0)),
                    Operator::Mul,
                    Box::new(Operation(
                        Box::new(IntegerLiteral(1)),
                        Operator::Add,
                        Box::new(IntegerLiteral(2))
                    ))
                )),
                Operator::Add,
                Box::new(Operation(
                    Box::new(Operation(
                        Box::new(IntegerLiteral(3)),
                        Operator::Mul,
                        Box::new(IntegerLiteral(4))
                    )),
                    Operator::Sub,
                    Box::new(Call(Box::new(FunctionCall {
                        name: "e".to_string(),
                        args: vec![Box::new(IntegerLiteral(1))]
                    })))
                ))
            )
        );
    }

    #[test]
    fn left_associative_expression() {
        let expression = grammar::ExpressionParser::new()
            .parse(ASSOCIATIVITY_TEST)
            .unwrap();
        use Expression::*;
        assert_eq!(
            *expression,
            Operation(
                Box::new(Operation(
                    Box::new(IntegerLiteral(0)),
                    Operator::Add,
                    Box::new(IntegerLiteral(1))
                )),
                Operator::Sub,
                Box::new(IntegerLiteral(2))
            )
        );
    }

    #[test]
    fn function_declarations() {
        let program = grammar::ProgramParser::new()
            .parse(FUN_DECLARATIONS_TEST)
            .unwrap();
        assert_eq!(program.var_declarations.len(), 0);
        assert_eq!(
            program.fun_declarations[0],
            FunctionDeclaration {
                return_type: IdentifierType::Int,
                function_name: "ident".to_string(),
                params: vec![Param::Var(
                    IdentifierType::Int,
                    "ab".to_string()
                )],
                body: Box::new(CompoundStatement {
                    declarations: Vec::new(),
                    statements: vec![]
                })
            }
        );
        assert_eq!(
            program.fun_declarations[1],
            FunctionDeclaration {
                return_type: IdentifierType::Void,
                function_name: "ident2".to_string(),
                params: vec![
                    Param::Var(IdentifierType::Void, "a".to_string()),
                    Param::ArrVar(
                        IdentifierType::Int,
                        "b".to_string()
                    ),
                ],
                body: Box::new(CompoundStatement {
                    declarations: Vec::new(),
                    statements: vec![]
                })
            }
        );
    }
}
