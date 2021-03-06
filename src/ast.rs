use std::fmt::{Debug};

#[derive(PartialEq, Debug)]
pub struct Program {
    pub var_declarations: Vec<VarDeclaration>,
    pub fun_declarations: Vec<FunctionDeclaration>
}

#[derive(PartialEq, Debug)]
pub struct FunctionDeclaration {
    pub return_type: IdentifierType,
    pub function_name: String,
    pub params: Vec<Param>,
    pub body: Box<CompoundStatement>,
}

#[derive(PartialEq, Debug)]
pub enum Param {
    Var(IdentifierType, String),
    ArrVar(IdentifierType, String)
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum IdentifierType {
    Int,
    Void
}

#[derive(PartialEq, Debug)]
pub enum Statement {
    Expression(Box<Expression>),
    CompoundStatement(Box<CompoundStatement>),
    IfStatement(Box<IfStatement>),
    WhileStatement(Box<WhileStatement>),
    ReturnStatement(Option<Box<Expression>>),
    EmptyStatement
}

#[derive(PartialEq, Debug)]
pub struct CompoundStatement {
    pub declarations: Vec<VarDeclaration>,
    pub statements: Vec<Box<Statement>>
}

#[derive(PartialEq, Debug)]
pub enum VarDeclaration {
    VarDeclaration(IdentifierType, String),
    ArrDeclaration(IdentifierType, String, i32)
}

#[derive(PartialEq, Debug)]
pub enum IfStatement {
    IfStmt(Box<Expression>, Box<Statement>),
    IfElseStmt(Box<Expression>, Box<Statement>, Box<Statement>)
}

#[derive(PartialEq, Debug)]
pub struct WhileStatement {
    pub condition: Box<Expression>,
    pub statement: Box<Statement>,
}

#[derive(PartialEq, Debug)]
pub enum Expression {
    Assignment(Box<Var>, Box<Expression>),
    Operation(Box<Expression>, Operator, Box<Expression>),
    Var(Box<Var>),
    Call(Box<FunctionCall>),
    IntegerLiteral(i32)
}

#[derive(PartialEq, Debug)]
pub enum Operator {
    Mul,
    Div,
    Add,
    Sub,
    Gt,
    Ge,
    Lt,
    Le,
    Ne,
    Eq,
    As
}

#[derive(PartialEq, Debug)]
pub enum Var {
    Var(String),
    ArrayAccess(String, Box<Expression>),
}

#[derive(PartialEq, Debug)]
pub struct FunctionCall {
    pub name: String,
    pub args: Vec<Box<Expression>>,
}

pub enum IdentifierFollow {
    FunctionCall(Vec<Box<Expression>>),
    ArrayAccess(Box<Expression>)
}

// impl Debug for FunctionCall {
//     fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
//         write!(fmt, "Function call")
//     }
// }
