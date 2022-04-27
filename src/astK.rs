use std::fmt::{Debug, Error, Formatter};

pub type Program = Vec<Box<TopLevelStatement>>;

pub enum TopLevelStatement {
    Expr(Box<Expr>),
    Func(Box<Func>),
    Extern(Box<Extern>),
}

pub enum Expr {
    Number(i32),
    Variable(std::string::String),
    FunctionCall(String, Vec<Expr>),
    Op(Box<Expr>, Opcode, Box<Expr>),
}

pub struct FuncSignature {
    pub name: String,
    pub args: Vec<String>,
}

pub struct Func {
    pub signature: Box<FuncSignature>,
    pub body: Box<Expr>
}

pub struct Extern {
    pub signature: Box<FuncSignature>,
}

#[derive(Copy, Clone)]
pub enum Opcode {
    Mul,
    Div,
    Add,
    Sub,
    Gt,
    Ge,
    Lt,
    Le,
}

impl Debug for Expr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Expr::*;
        match &*self {
            Number(n) => write!(fmt, "expr: number: {:?}", n),
            Op(ref l, op, ref r) => write!(fmt, "expr: operation: ({:?} {:?} {:?})", l, op, r),
            Variable(v) => write!(fmt, "expr: variable: {:?}", v),
            FunctionCall(name, args) => write!(fmt, "function call: name: {:?}, args: {:?}", name, args),
        }
    }
}

impl Debug for Opcode {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Opcode::*;
        match *self {
            Mul => write!(fmt, "*"),
            Div => write!(fmt, "/"),
            Add => write!(fmt, "+"),
            Sub => write!(fmt, "-"),
            Gt => write!(fmt, ">"),
            Ge => write!(fmt, ">="),
            Lt => write!(fmt, "<"),
            Le => write!(fmt, "<="),
        }
    }
}

impl Debug for Func {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        // use self::Func::*;
        write!(fmt, "function definition: \n\t{:?} \n\t{:?}", self.signature, self.body)
    }
}

impl Debug for FuncSignature {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        // use self::FuncSignature::*;
        write!(fmt, "signature: \n\t\tfunction name: {:?}, \n\t\targs: {:?}", self.name, self.args)
    }
}

impl Debug for Extern {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        // use self::Extern::*;
        write!(fmt, "Extern: \n\t{:?}", self.signature)
    }
}

impl Debug for TopLevelStatement {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::TopLevelStatement::*;
        match &*self {
            Expr(expr) => write!(fmt, "{:?}", expr),
            Extern(e) => write!(fmt, "{:?}", e),
            Func(func) => write!(fmt, "{:?}", func),
        }
    }
}
