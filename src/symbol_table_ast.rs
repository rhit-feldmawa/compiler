use crate::ast::{Program, Expression, FunctionCall, IdentifierType, Operator, Var, VarDeclaration};
use crate::symbol_table::{
    get_child_table, ArraySymbolTableElement, FunctionSymbolTableElement, Param,
    ParameterArraySymbolTableElement, SymbolTable, SymbolTableElement, VariableSymbolTableElement,
};
use std::collections::HashMap;
use std::sync::Arc;

// pub struct Program {
//     pub fun_declarations: Vec<FunctionDeclaration>,
//     pub scope: Arc<SymbolTable>,
// }

// pub struct FunctionDeclaration {
//     compound_statement: CompoundStatement
// }

// pub enum Statement {
//     CompoundStatement(Box<CompoundStatement>),
//     Expression(Box<Expression>),
//     IfStatement(Box<IfStatement>),
//     WhileStatement(Box<WhileStatement>),
//     ReturnStatement(Box<Expression>),
//     EmptyStatement,
// }

// pub struct WhileStatement {
//     condition: Expression,
//     statement: Box<Statement>,
// }

// pub enum IfStatement {
//     IfStmt(Expression, Box<Statement>),
//     IfElseStmt(Expression, Box<Statement>, Box<Statement>),
// }

// pub struct CompoundStatement {
//     statements: Vec<Box<Statement>>,
//     scope: Arc<SymbolTable>,
// }

pub enum TypecheckProgramResult {
    Success(Program),
    Failure(String),
}

fn generate_symbol_table_ast(program: crate::ast::Program) -> TypecheckProgramResult {
    let mut symbol_table = SymbolTable {
        current_scope: HashMap::new(),
        parent_scope: None,
        depth: 0,
    };
    let wasSuccessful = handle_variable_declarations(program.var_declarations, &mut symbol_table);
    if !wasSuccessful {
        return TypecheckProgramResult::Failure("Duplicate variable declaration".to_string());
    }
    let mut fun_declarations = Vec::new();
    let mut immutable_symbol_table = Arc::new(symbol_table);
    for function in program.fun_declarations {
        let result = handle_function(function, immutable_symbol_table);
        match result {
            GenerateSymbolTableFunctionResult::Success(function_declaration, new_table) => {
                fun_declarations.push(function_declaration);
                immutable_symbol_table = new_table;
            }
            GenerateSymbolTableFunctionResult::Failure(reason) => {
                return TypecheckProgramResult::Failure(reason);
            }
        };
    }
    let output = Program {
        var_declarations: program.var_declarations,
        fun_declarations: fun_declarations,
    };
    TypecheckProgramResult::Success(output)
}

fn handle_variable_declarations(
    declarations: Vec<Box<VarDeclaration>>,
    symbol_table: &mut SymbolTable,
) -> bool {
    for declaration in declarations {
        let element;
        let name;
        match *declaration {
            VarDeclaration::VarDeclaration(identifier_type, p_name) => {
                element = SymbolTableElement::Variable(VariableSymbolTableElement {
                    element_type: identifier_type,
                    depth: 0,
                });
                name = p_name;
            }
            VarDeclaration::ArrDeclaration(identifier_type, p_name, size) => {
                element = SymbolTableElement::Array(ArraySymbolTableElement {
                    element_type: identifier_type,
                    size: size,
                    depth: 0,
                });
                name = p_name;
            }
        }
        let was_successful = symbol_table.symbol_insert(&name, element);
        if !was_successful {
            return false;
        }
    }
    return true;
}

pub enum GenerateSymbolTableFunctionResult {
    Success(FunctionDeclaration),
    Failure(String),
}

fn handle_function(
    input_function: crate::ast::FunctionDeclaration,
    symbol_table: Arc<SymbolTable>,
) -> GenerateSymbolTableFunctionResult {
    let element = SymbolTableElement::Function(FunctionSymbolTableElement {
        return_type: input_function.return_type,
        argument_types: input_function
            .params
            .iter()
            .map(|arg| match arg {
                crate::ast::Param::Var(identifier_type, _) => Param::Var(*identifier_type),
                crate::ast::Param::ArrVar(identifier_type, _) => Param::Arr(*identifier_type),
            })
            .collect(),
        depth: 0,
    });

    let mut new_symbol_table = get_child_table(symbol_table);
    let isNameValid = new_symbol_table.symbol_insert(&input_function.function_name, element);
    if !isNameValid {
        return GenerateSymbolTableFunctionResult::Failure("Invalid function name".to_string());
    }

    for param in input_function.params {
        match param {
            crate::ast::Param::Var(identifier_type, name) => {
                new_symbol_table.symbol_insert(
                    &name,
                    SymbolTableElement::Variable(VariableSymbolTableElement {
                        element_type: identifier_type,
                        depth: 1,
                    }),
                );
            }
            crate::ast::Param::ArrVar(identifier_type, name) => {
                new_symbol_table.symbol_insert(
                    &name,
                    SymbolTableElement::ParameterArray(ParameterArraySymbolTableElement {
                        element_type: identifier_type,
                        depth: 1,
                    }),
                );
            }
        }
    }

    let immutable_symbol_table = Arc::new(new_symbol_table);
    let body = handle_compound_statement(*input_function.body, immutable_symbol_table);
    return match body {
        GenerateSymbolTableCompoundStatementResult::Success(compound_statement, symbol_table) => {
            GenerateSymbolTableFunctionResult::Success(
                FunctionDeclaration {
                    compound_statement,
                },
                symbol_table,
            )
        }
        GenerateSymbolTableCompoundStatementResult::Failure(reason) => {
            GenerateSymbolTableFunctionResult::Failure(reason)
        }
    };
}

pub enum GenerateSymbolTableCompoundStatementResult {
    Success(CompoundStatement, Arc<SymbolTable>),
    Failure(String),
}

fn handle_compound_statement(
    input_statement: crate::ast::CompoundStatement,
    symbol_table: Arc<SymbolTable>,
) -> GenerateSymbolTableCompoundStatementResult {
    match input_statement {
        crate::ast::CompoundStatement {declarations, statements} => {
            let mut new_symbol_table = get_child_table(symbol_table.clone());
            let mut new_statements: Vec<Box<Statement>> = Vec::new();
            handle_variable_declarations(declarations, &mut new_symbol_table);
            let mut immutable_symbol_table = Arc::new(new_symbol_table);
            for statement in statements {
                match handle_statement(*statement, immutable_symbol_table) {
                    HandleStatementResult::Success(statement, symbol_table) => {
                        new_statements.push(Box::new(statement));
                        immutable_symbol_table = symbol_table;
                    },
                    HandleStatementResult::Failure(reason) => return GenerateSymbolTableCompoundStatementResult::Failure(reason)
                }
            }
            GenerateSymbolTableCompoundStatementResult::Success(CompoundStatement { statements: new_statements, scope: immutable_symbol_table}, symbol_table.clone())
        }
    }
}

enum HandleStatementResult {
    Success(Statement, Arc<SymbolTable>),
    Failure(String),
}

fn handle_statement(
    statement: crate::ast::Statement,
    symbol_table: Arc<SymbolTable>,
) -> HandleStatementResult {
    match statement {
        crate::ast::Statement::Expression(expression) => {
            match handle_expression(*expression, &symbol_table) {
                HandleExpressionResult::Success(expression) => {
                    return HandleStatementResult::Success(
                        Statement::Expression(Box::new(expression)),
                        symbol_table,
                    );
                }
                HandleExpressionResult::Failure(reason) => {
                    return HandleStatementResult::Failure(reason)
                }
            }
        }
        crate::ast::Statement::ReturnStatement(expression) => {
            return match handle_expression(*expression, &symbol_table) {
                HandleExpressionResult::Success(expression) => HandleStatementResult::Success(
                    Statement::ReturnStatement(Box::new(expression)),
                    symbol_table,
                ),
                HandleExpressionResult::Failure(reason) => HandleStatementResult::Failure(reason),
            }
        }
        crate::ast::Statement::WhileStatement(while_statement) => {
            match handle_expression(*while_statement.condition, &symbol_table) {
                HandleExpressionResult::Success(expression) => {
                    match handle_statement(*while_statement.statement, symbol_table) {
                        HandleStatementResult::Success(statement, symbol_table) => {
                            HandleStatementResult::Success(
                                Statement::WhileStatement(Box::new(WhileStatement {
                                    condition: expression,
                                    statement: Box::new(statement),
                                })),
                                symbol_table,
                            )
                        }
                        HandleStatementResult::Failure(reason) => {
                            HandleStatementResult::Failure(reason)
                        }
                    }
                }
                HandleExpressionResult::Failure(reason) => HandleStatementResult::Failure(reason),
            }
        }
        crate::ast::Statement::IfStatement(if_statement) => match *if_statement {
            crate::ast::IfStatement::IfStmt(condition, statement) => {
                match handle_expression(*condition, &symbol_table) {
                    HandleExpressionResult::Success(condition) => {
                        match handle_statement(*statement, symbol_table) {
                            HandleStatementResult::Success(statement, symbol_table) => {
                                HandleStatementResult::Success(
                                    Statement::IfStatement(Box::new(IfStatement::IfStmt(
                                        condition,
                                        Box::new(statement),
                                    ))),
                                    symbol_table,
                                )
                            }
                            HandleStatementResult::Failure(reason) => {
                                HandleStatementResult::Failure(reason)
                            }
                        }
                    }
                    HandleExpressionResult::Failure(reason) => {
                        HandleStatementResult::Failure(reason)
                    }
                }
            }
            crate::ast::IfStatement::IfElseStmt(condition, statement, statement2) => {
                match handle_expression(*condition, &symbol_table) {
                    HandleExpressionResult::Success(condition) => {
                        match handle_statement(*statement, symbol_table) {
                            HandleStatementResult::Success(statement1, symbol_table) => {
                                match handle_statement(*statement2, symbol_table) {
                                    HandleStatementResult::Success(statement, symbol_table) => {
                                        HandleStatementResult::Success(
                                            Statement::IfStatement(Box::new(IfStatement::IfStmt(
                                                condition,
                                                Box::new(statement),
                                            ))),
                                            symbol_table,
                                        )
                                    }
                                    HandleStatementResult::Failure(reason) => {
                                        HandleStatementResult::Failure(reason)
                                    }
                                }
                            }
                            HandleStatementResult::Failure(reason) => {
                                HandleStatementResult::Failure(reason)
                            }
                        }
                    }
                    HandleExpressionResult::Failure(reason) => {
                        HandleStatementResult::Failure(reason)
                    }
                }
            }
        },
        crate::ast::Statement::EmptyStatement => {
            HandleStatementResult::Success(Statement::EmptyStatement, symbol_table)
        }
        crate::ast::Statement::CompoundStatement(compoundStatement) => {
            match handle_compound_statement(*compoundStatement, symbol_table) {
                GenerateSymbolTableCompoundStatementResult::Success(
                    compoundStatement,
                    symbol_table,
                ) => HandleStatementResult::Success(
                    Statement::CompoundStatement(Box::new(compoundStatement)),
                    symbol_table,
                ),
                GenerateSymbolTableCompoundStatementResult::Failure(reason) => HandleStatementResult::Failure(reason)
            }
        }
    }
}

enum HandleExpressionResult {
    Success(Expression),
    Failure(String),
}

fn handle_expression(
    expression: Expression,
    symbol_table: &Arc<SymbolTable>,
) -> HandleExpressionResult {
    match expression {
        Expression::Assignment(var, expression) => {
            handle_assignment(*var, *expression, symbol_table)
        }
        Expression::Operation(expression, operator, expression2) => {
            handle_operation(*expression, operator, *expression2, symbol_table)
        }
        Expression::Var(var) => match handle_var(*var, symbol_table) {
            HandleVarResult::Success(var) => {
                HandleExpressionResult::Success(Expression::Var(Box::new(var)))
            }
            HandleVarResult::Failure(reason) => HandleExpressionResult::Failure(reason),
        },
        Expression::Call(functionCall) => handle_function_call(*functionCall, symbol_table),
        Expression::IntegerLiteral(value) => {
            HandleExpressionResult::Success(Expression::IntegerLiteral(value))
        }
    }
}

fn handle_function_call(
    function_call: FunctionCall,
    symbol_table: &Arc<SymbolTable>,
) -> HandleExpressionResult {
    let function;
    match symbol_table.symbol_lookup(&function_call.name) {
        Some(symbol_table_element) => match symbol_table_element {
            SymbolTableElement::Function(element) => function = element,
            _ => {
                return HandleExpressionResult::Failure("Attempt to call non-function".to_string())
            }
        },
        _ => return HandleExpressionResult::Failure("Attempt to call non-function".to_string()),
    };
    match function_call {
        FunctionCall {name, args} => {
            let mut len = 0;
            let mut new_args: Vec<Box<Expression>> = Vec::new();
            for arg in args {
                match handle_expression(*arg, symbol_table) {
                    HandleExpressionResult::Failure(reason) => {
                        return HandleExpressionResult::Failure(reason)
                    }
                    HandleExpressionResult::Success(expression) => new_args.push(Box::new(expression)),
                }
                len = len + 1;
            }
            if len != function.argument_types.len() {
                return HandleExpressionResult::Failure(
                    "Incorrect number of arguments in function call".to_string(),
                );
            }
            return HandleExpressionResult::Success(Expression::Call(Box::new(FunctionCall { name, args: new_args })))
        }
    }
}

fn handle_operation(
    expression1: Expression,
    operator: Operator,
    expression2: Expression,
    symbol_table: &Arc<SymbolTable>,
) -> HandleExpressionResult {
    match handle_expression(expression1, symbol_table) {
        HandleExpressionResult::Success(expression1) => {
            match handle_expression(expression2, symbol_table) {
                HandleExpressionResult::Success(expression2) => HandleExpressionResult::Success(
                    Expression::Operation(Box::new(expression1), operator, Box::new(expression2)),
                ),
                HandleExpressionResult::Failure(reason) => HandleExpressionResult::Failure(reason),
            }
        }
        HandleExpressionResult::Failure(reason) => HandleExpressionResult::Failure(reason),
    }
}

fn handle_assignment(
    var: Var,
    expression: Expression,
    symbol_table: &Arc<SymbolTable>,
) -> HandleExpressionResult {
    match handle_var(var, symbol_table) {
        HandleVarResult::Success(var) => match handle_expression(expression, symbol_table) {
            HandleExpressionResult::Success(expression) => HandleExpressionResult::Success(
                Expression::Assignment(Box::new(var), Box::new(expression)),
            ),
            HandleExpressionResult::Failure(reason) => HandleExpressionResult::Failure(reason),
        },
        HandleVarResult::Failure(reason) => HandleExpressionResult::Failure(reason),
    }
}

enum HandleVarResult {
    Success(Var),
    Failure(String),
}

fn handle_var(var: Var, symbol_table: &Arc<SymbolTable>) -> HandleVarResult {
    match &var {
        Var::Var(name) => match symbol_table.symbol_lookup(&name) {
            Some(symbol_table_element) => match symbol_table_element {
                SymbolTableElement::Variable(_) => HandleVarResult::Success(var),
                _ => HandleVarResult::Failure(
                    "Attempted to assign to either a function or array".to_string(),
                ),
            },
            None => HandleVarResult::Failure("Assignment to undeclared variable".to_string()),
        },
        Var::ArrayAccess(name, expression) => match symbol_table.symbol_lookup(&name) {
            Some(symbol_table_element) => match symbol_table_element {
                SymbolTableElement::Array(_) => HandleVarResult::Success(var),
                SymbolTableElement::ParameterArray(_) => HandleVarResult::Success(var),
                _ => HandleVarResult::Failure(
                    "Attempted to assign to either a function or a variable as an array"
                        .to_string(),
                ),
            },
            None => HandleVarResult::Failure("Assignment to undeclared array".to_string()),
        },
    }
}
