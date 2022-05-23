use crate::ast::{Expression, FunctionCall, IdentifierType, Var, VarDeclaration};
use crate::symbol_table::{
    get_child_table, ArraySymbolTableElement, ExpressionType, FunctionSymbolTableElement, Param,
    ParameterArraySymbolTableElement, SymbolTable, SymbolTableElement, VariableSymbolTableElement,
};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(PartialEq, Debug)]
pub enum TypecheckProgramResult {
    Success,
    Failure(String),
}

pub fn typecheck_program(program: &crate::ast::Program) -> TypecheckProgramResult {
    let mut symbol_table = SymbolTable {
        current_scope: HashMap::new(),
        parent_scope: None,
        depth: 0,
    };
    let was_successful = handle_variable_declarations(&program.var_declarations, &mut symbol_table);
    if !was_successful {
        return TypecheckProgramResult::Failure("Duplicate variable declaration".to_string());
    }
    let immutable_symbol_table = Arc::new(symbol_table);
    for function in &program.fun_declarations {
        let result = handle_function(function, immutable_symbol_table.clone());
        match result {
            TypecheckFunctionResult::Success => {}
            TypecheckFunctionResult::Failure(reason) => {
                return TypecheckProgramResult::Failure(reason);
            }
        };
    }
    TypecheckProgramResult::Success
}

fn handle_variable_declarations(
    declarations: &Vec<VarDeclaration>,
    symbol_table: &mut SymbolTable,
) -> bool {
    for declaration in declarations {
        let element;
        let name;
        match &declaration {
            VarDeclaration::VarDeclaration(_, p_name) => {
                element = SymbolTableElement::Variable(VariableSymbolTableElement {
                    element_type: IdentifierType::Int,
                    depth: 0,
                });
                name = p_name;
            }
            VarDeclaration::ArrDeclaration(_, p_name, size) => {
                element = SymbolTableElement::Array(ArraySymbolTableElement {
                    element_type: IdentifierType::Int,
                    size: *size,
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

pub enum TypecheckFunctionResult {
    Success,
    Failure(String),
}

fn handle_function(
    input_function: &crate::ast::FunctionDeclaration,
    symbol_table: Arc<SymbolTable>,
) -> TypecheckFunctionResult {
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
    let is_name_valid = new_symbol_table.symbol_insert(&input_function.function_name, element);
    if !is_name_valid {
        return TypecheckFunctionResult::Failure("Invalid function name".to_string());
    }

    for param in &input_function.params {
        match param {
            crate::ast::Param::Var(_, name) => {
                new_symbol_table.symbol_insert(
                    &name,
                    SymbolTableElement::Variable(VariableSymbolTableElement {
                        element_type: IdentifierType::Int,
                        depth: 1,
                    }),
                );
            }
            crate::ast::Param::ArrVar(_, name) => {
                new_symbol_table.symbol_insert(
                    &name,
                    SymbolTableElement::ParameterArray(ParameterArraySymbolTableElement {
                        element_type: IdentifierType::Int,
                        depth: 1,
                    }),
                );
            }
        }
    }

    let immutable_symbol_table = Arc::new(new_symbol_table);
    let body = handle_compound_statement(&input_function.body, immutable_symbol_table);
    return match body {
        TypecheckCompoundStatementResult::Success => TypecheckFunctionResult::Success,
        TypecheckCompoundStatementResult::Failure(reason) => {
            TypecheckFunctionResult::Failure(reason)
        }
    };
}

pub enum TypecheckCompoundStatementResult {
    Success,
    Failure(String),
}

fn handle_compound_statement(
    input_statement: &crate::ast::CompoundStatement,
    symbol_table: Arc<SymbolTable>,
) -> TypecheckCompoundStatementResult {
    match &input_statement {
        crate::ast::CompoundStatement {
            declarations,
            statements,
        } => {
            let mut new_symbol_table = get_child_table(symbol_table.clone());
            handle_variable_declarations(&declarations, &mut new_symbol_table);
            let immutable_symbol_table = Arc::new(new_symbol_table);
            for statement in statements {
                match handle_statement(statement, immutable_symbol_table.clone()) {
                    HandleStatementResult::Success => {}
                    HandleStatementResult::Failure(reason) => {
                        return TypecheckCompoundStatementResult::Failure(reason)
                    }
                }
            }
            TypecheckCompoundStatementResult::Success
        }
    }
}

enum HandleStatementResult {
    Success,
    Failure(String),
}

fn handle_statement(
    statement: &crate::ast::Statement,
    symbol_table: Arc<SymbolTable>,
) -> HandleStatementResult {
    match statement {
        crate::ast::Statement::Expression(expression) => {
            match handle_expression(&expression, &symbol_table) {
                HandleExpressionResult::Success(_) => return HandleStatementResult::Success,
                HandleExpressionResult::Failure(reason) => {
                    return HandleStatementResult::Failure(reason)
                }
            }
        }
        crate::ast::Statement::ReturnStatement(expression) => match expression {
            Option::Some(expression) => {
                return match handle_expression(&expression, &symbol_table) {
                    HandleExpressionResult::Success(expression_result) => {
                        if expression_result != ExpressionType::Int {
                            return HandleStatementResult::Failure(
                                "Attempt to return a non-Int value".to_string(),
                            );
                        }
                        HandleStatementResult::Success
                    }
                    HandleExpressionResult::Failure(reason) => {
                        HandleStatementResult::Failure(reason)
                    }
                }
            }
            Option::None => {
                return HandleStatementResult::Success;
            }
        },
        crate::ast::Statement::WhileStatement(while_statement) => {
            match handle_expression(&while_statement.condition, &symbol_table) {
                HandleExpressionResult::Success(expression_type) => {
                    match handle_statement(&while_statement.statement, symbol_table) {
                        HandleStatementResult::Success => {
                            if expression_type != ExpressionType::Int {
                                return HandleStatementResult::Failure(
                                    "Use of non-Int in while statement condition".to_string(),
                                );
                            }
                            HandleStatementResult::Success
                        }
                        HandleStatementResult::Failure(reason) => {
                            HandleStatementResult::Failure(reason)
                        }
                    }
                }
                HandleExpressionResult::Failure(reason) => HandleStatementResult::Failure(reason),
            }
        }
        crate::ast::Statement::IfStatement(if_statement) => match &**if_statement {
            crate::ast::IfStatement::IfStmt(condition, statement) => {
                match handle_expression(&condition, &symbol_table) {
                    HandleExpressionResult::Success(expression_type) => {
                        match handle_statement(&statement, symbol_table) {
                            HandleStatementResult::Success => {
                                if expression_type != ExpressionType::Int {
                                    return HandleStatementResult::Failure(
                                        "Use of non-Int in if statement condition".to_string(),
                                    );
                                }
                                HandleStatementResult::Success
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
                match handle_expression(&condition, &symbol_table) {
                    HandleExpressionResult::Success(expression_type) => {
                        match handle_statement(&statement, symbol_table.clone()) {
                            HandleStatementResult::Success => {
                                match handle_statement(&statement2, symbol_table) {
                                    HandleStatementResult::Success => {
                                        if expression_type != ExpressionType::Int {
                                            return HandleStatementResult::Failure(
                                                "Use of non-Int in if statement condition"
                                                    .to_string(),
                                            );
                                        }
                                        HandleStatementResult::Success
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
        crate::ast::Statement::EmptyStatement => HandleStatementResult::Success,
        crate::ast::Statement::CompoundStatement(compound_statement) => {
            match handle_compound_statement(&compound_statement, symbol_table) {
                TypecheckCompoundStatementResult::Success => HandleStatementResult::Success,
                TypecheckCompoundStatementResult::Failure(reason) => {
                    HandleStatementResult::Failure(reason)
                }
            }
        }
    }
}

enum HandleExpressionResult {
    Success(ExpressionType),
    Failure(String),
}

fn handle_expression(
    expression: &Expression,
    symbol_table: &Arc<SymbolTable>,
) -> HandleExpressionResult {
    match expression {
        Expression::Assignment(var, expression) => {
            handle_assignment(&var, &expression, symbol_table)
        }
        Expression::Operation(expression, _, expression2) => {
            handle_operation(&expression, &expression2, symbol_table)
        }
        Expression::Var(var) => match handle_var(&var, symbol_table) {
            HandleVarResult::Success(expression_type) => {
                HandleExpressionResult::Success(expression_type)
            }
            HandleVarResult::Failure(reason) => HandleExpressionResult::Failure(reason),
        },
        Expression::Call(function_call) => handle_function_call(&function_call, symbol_table),
        Expression::IntegerLiteral(_) => HandleExpressionResult::Success(ExpressionType::Int),
    }
}

fn handle_function_call(
    function_call: &FunctionCall,
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
        FunctionCall { name: _, args } => {
            let mut len = 0;
            for arg in args {
                match handle_expression(&arg, symbol_table) {
                    HandleExpressionResult::Failure(reason) => {
                        return HandleExpressionResult::Failure(reason)
                    }
                    HandleExpressionResult::Success(expression_type) => {
                        match function.argument_types[len] {
                            Param::Var(_) => {
                                if expression_type != ExpressionType::Int {
                                    return HandleExpressionResult::Failure(
                                        "Attempted to pass non-int, but int was expected"
                                            .to_string(),
                                    );
                                }
                            }
                            Param::Arr(_) => {
                                if expression_type != ExpressionType::Array {
                                    return HandleExpressionResult::Failure(
                                        "Attempted to pass int, but array was expected".to_string(),
                                    );
                                }
                            }
                        }
                    }
                }
                len = len + 1;
            }
            if len != function.argument_types.len() {
                return HandleExpressionResult::Failure(
                    "Incorrect number of arguments in function call".to_string(),
                );
            }
            match function.return_type {
                IdentifierType::Int => HandleExpressionResult::Success(ExpressionType::Int),
                IdentifierType::Void => HandleExpressionResult::Success(ExpressionType::Void),
            }
        }
    }
}

fn handle_operation(
    expression1: &Expression,
    expression2: &Expression,
    symbol_table: &Arc<SymbolTable>,
) -> HandleExpressionResult {
    match handle_expression(expression1, symbol_table) {
        HandleExpressionResult::Success(expression_type) => {
            match handle_expression(expression2, symbol_table) {
                HandleExpressionResult::Success(expression2_type) => {
                    if expression_type != ExpressionType::Int
                        || expression2_type != ExpressionType::Int
                    {
                        return HandleExpressionResult::Failure(
                            "Attempt to perform an operation with non-Ints".to_string(),
                        );
                    } else {
                        return HandleExpressionResult::Success(ExpressionType::Int);
                    }
                }
                HandleExpressionResult::Failure(reason) => HandleExpressionResult::Failure(reason),
            }
        }
        HandleExpressionResult::Failure(reason) => HandleExpressionResult::Failure(reason),
    }
}

fn handle_assignment(
    var: &Var,
    expression: &Expression,
    symbol_table: &Arc<SymbolTable>,
) -> HandleExpressionResult {
    match handle_assignment_left(var, symbol_table) {
        HandleAssignmentLeftResult::Int => match handle_expression(expression, symbol_table) {
            HandleExpressionResult::Success(expression_type) => match expression_type {
                ExpressionType::Int => HandleExpressionResult::Success(ExpressionType::Int),
                ExpressionType::Array => HandleExpressionResult::Failure(
                    "Attempt to assign an array to an Int".to_string(),
                ),
                _ => HandleExpressionResult::Failure(
                    "Attempt to assign void to a variable".to_string(),
                ),
            },
            HandleExpressionResult::Failure(reason) => HandleExpressionResult::Failure(reason),
        },
        HandleAssignmentLeftResult::Array => match handle_expression(expression, symbol_table) {
            HandleExpressionResult::Success(expression_type) => match expression_type {
                ExpressionType::Int => HandleExpressionResult::Failure(
                    "Attempt to assign an array to an Int".to_string(),
                ),
                ExpressionType::Array => HandleExpressionResult::Success(ExpressionType::Array),
                _ => HandleExpressionResult::Failure(
                    "Attempt to assign void to a variable".to_string(),
                ),
            },
            HandleExpressionResult::Failure(reason) => HandleExpressionResult::Failure(reason),
        },
        HandleAssignmentLeftResult::Failure(reason) => HandleExpressionResult::Failure(reason),
    }
}

enum HandleVarResult {
    Success(ExpressionType),
    Failure(String),
}

enum HandleAssignmentLeftResult {
    Array,
    Int,
    Failure(String),
}

fn handle_assignment_left(
    var: &Var,
    symbol_table: &Arc<SymbolTable>,
) -> HandleAssignmentLeftResult {
    match &var {
        Var::Var(name) => match symbol_table.symbol_lookup(&name) {
            Some(symbol_table_element) => match symbol_table_element {
                SymbolTableElement::Variable(_) => HandleAssignmentLeftResult::Int,
                SymbolTableElement::Array(_) => HandleAssignmentLeftResult::Array,
                _ => HandleAssignmentLeftResult::Failure(
                    "Attempted to assign to either a function or array".to_string(),
                ),
            },
            None => {
                HandleAssignmentLeftResult::Failure("Assignment to undeclared variable".to_string())
            }
        },
        Var::ArrayAccess(name, expression) => {
            match handle_expression(expression, symbol_table) {
                HandleExpressionResult::Success(expression_type) => match expression_type {
                    ExpressionType::Int => {}
                    _ => {
                        return HandleAssignmentLeftResult::Failure(
                            "Attempt to index array by non-Int".to_string(),
                        )
                    }
                },
                HandleExpressionResult::Failure(reason) => {
                    return HandleAssignmentLeftResult::Failure(reason)
                }
            }
            match symbol_table.symbol_lookup(&name) {
                Some(symbol_table_element) => match symbol_table_element {
                    SymbolTableElement::Array(_) => HandleAssignmentLeftResult::Int,
                    SymbolTableElement::ParameterArray(_) => HandleAssignmentLeftResult::Int,
                    _ => HandleAssignmentLeftResult::Failure(
                        "Attempted to assign to either a function or a variable as an array"
                            .to_string(),
                    ),
                },
                None => HandleAssignmentLeftResult::Failure(
                    "Assignment to undeclared array".to_string(),
                ),
            }
        }
    }
}

fn handle_var(var: &Var, symbol_table: &Arc<SymbolTable>) -> HandleVarResult {
    match &var {
        Var::Var(name) => match symbol_table.symbol_lookup(&name) {
            Some(symbol_table_element) => match symbol_table_element {
                SymbolTableElement::Function(_) => HandleVarResult::Failure(
                    "Attempted to use a function as a variable".to_string(),
                ),
                SymbolTableElement::Variable(_) => HandleVarResult::Success(ExpressionType::Int),
                SymbolTableElement::Array(_) => HandleVarResult::Success(ExpressionType::Array),
                SymbolTableElement::ParameterArray(_) => {
                    HandleVarResult::Success(ExpressionType::Array)
                }
            },
            None => HandleVarResult::Failure("Assignment to undeclared variable".to_string()),
        },
        Var::ArrayAccess(name, expression) => {
            match handle_expression(expression, symbol_table) {
                HandleExpressionResult::Success(expression_type) => match expression_type {
                    ExpressionType::Int => {}
                    _ => {
                        return HandleVarResult::Failure(
                            "Attempt to index array by non-Int".to_string(),
                        )
                    }
                },
                HandleExpressionResult::Failure(reason) => return HandleVarResult::Failure(reason),
            }
            match symbol_table.symbol_lookup(&name) {
                Some(symbol_table_element) => match symbol_table_element {
                    SymbolTableElement::Array(_) => HandleVarResult::Success(ExpressionType::Int),
                    SymbolTableElement::ParameterArray(_) => {
                        return HandleVarResult::Success(ExpressionType::Int);
                    }
                    _ => HandleVarResult::Failure(
                        "Attempted to use either a function or a variable as an array".to_string(),
                    ),
                },
                None => HandleVarResult::Failure("Assignment to undeclared array".to_string()),
            }
        }
    }
}
