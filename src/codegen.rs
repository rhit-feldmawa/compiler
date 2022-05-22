use crate::ast::*;
extern crate llvm_sys as llvm;

use llvm::core::*;
use llvm::*;
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr;

fn codegen(program: Program, file_name: &str) {
    unsafe {
        let context = LLVMContextCreate();
        let module = LLVMModuleCreateWithNameInContext(file_name.as_ptr() as *const _, context);
        let builder = LLVMCreateBuilderInContext(context);
        let intType = LLVMInt32Type();
        let mut functions: HashMap<String,prelude::LLVMValueRef> = HashMap::new();
        // let mut namedValues: HashMap<String, prelude::LLVMValueRef> = HashMap::new();
        let mut namedValues = Table {
            scope: HashMap::new(),
            parent: Option::None,
        };
        for var_declaration in &program.var_declarations {
            match &var_declaration {
                VarDeclaration::ArrDeclaration(_, name, size) => {
                    LLVMAddGlobal(
                        module,
                        LLVMArrayType(intType, *size as u32),
                        name.as_ptr() as *const _,
                    );
                    let gVar = LLVMGetNamedGlobal(module, toC_char(&name));
                    LLVMSetLinkage(gVar, LLVMLinkage::LLVMCommonLinkage)
                }
                VarDeclaration::VarDeclaration(_, name) => {
                    LLVMAddGlobal(module, intType, toC_char(&name));
                    let gVar = LLVMGetNamedGlobal(module, toC_char(&name));
                    LLVMSetLinkage(gVar, LLVMLinkage::LLVMCommonLinkage)
                }
            }
        }

        for function in &program.fun_declarations {
            let mut formal_vars: Vec<prelude::LLVMTypeRef> = Vec::new();
            for arg in &function.params {
                match &arg {
                    Param::ArrVar(_, name) => {
                        formal_vars.push(LLVMInt32TypeInContext(context));
                    }
                    _ => {
                        println!("Error, formal variable in not an int");
                    }
                }
            }
            let function_type = match &function.return_type {
                IdentifierType::Void => LLVMFunctionType(
                    LLVMVoidTypeInContext(context),
                    formal_vars.as_mut_ptr(),
                    formal_vars.len().try_into().unwrap(),
                    0,
                ),
                IdentifierType::Int => LLVMFunctionType(
                    LLVMInt32TypeInContext(context),
                    formal_vars.as_mut_ptr(),
                    formal_vars.len().try_into().unwrap(),
                    0,
                ),
            };
            let llvm_function =
                LLVMAddFunction(module, toC_char(&function.function_name), function_type);
            functions.insert(function.function_name.clone(), llvm_function);
        }
        let mut i = 0;
        let mut j = 0;
        for function in &program.fun_declarations {
            namedValues = Table {
                scope: HashMap::new(),
                parent: Option::Some(Box::new(namedValues)),
            };
            let bb = LLVMAppendBasicBlockInContext(
                context,
                *functions.get_mut(&function.function_name).unwrap(),
                b"entry\0".as_ptr() as *const _,
            );
            LLVMPositionBuilderAtEnd(builder, bb);
            for param in &program.fun_declarations[i].params {
                match param {
                    Param::Var(_, name) => {
                        let inst = LLVMBuildAlloca(builder, intType, toC_char(&name));
                        LLVMBuildStore(builder, LLVMGetParam(*functions.get_mut(&function.function_name).unwrap(), j), inst);
                        namedValues.insert(name.clone(), inst);
                    }
                    _ => {
                        println!("Invaldid param");
                    }
                }
                j = j + 1;
            }
            namedValues = codegenCompountStatement(builder, &function.body, namedValues, &mut functions);
            i = i + 1;
            namedValues = *namedValues.parent.unwrap();
        }
    }
}

unsafe fn codegenCompountStatement(
    builder: prelude::LLVMBuilderRef,
    statement: &CompoundStatement,
    mut named_values: Table,
    functions: &mut HashMap<String, prelude::LLVMValueRef>
) -> Table {
    let mut new_table = Table {
        scope: HashMap::new(),
        parent: Option::Some(Box::new(named_values)),
    };
    for declaration in &statement.declarations {
        match declaration {
            VarDeclaration::VarDeclaration(_, name) => {
                let inst = LLVMBuildAlloca(builder, LLVMInt32Type(), toC_char(&name));
                new_table.insert(name.clone(), inst);
            }
            VarDeclaration::ArrDeclaration(_, name, size) => {
                let arr_type = LLVMArrayType(LLVMInt32Type(), *size as u32);
                let inst = LLVMBuildAlloca(builder, arr_type, toC_char(&name));
                new_table.insert(name.clone(), inst);
            }
        }
    }
    for statement in &statement.statements {
        new_table = codegenStatement(builder, &statement, new_table, functions);
    }

    return *new_table.parent.unwrap();
}

unsafe fn codegenStatement(
    builder: prelude::LLVMBuilderRef,
    statement: &Statement,
    mut named_values: Table,
    functions: &mut HashMap<String, prelude::LLVMValueRef>
) -> Table {
    match statement {
        Statement::CompoundStatement(compound_statement) => {
            return codegenCompountStatement(builder, compound_statement, named_values, functions)
        }
        Statement::EmptyStatement => {}
        Statement::Expression(expression) => {
            codegenExpression(builder, expression, &mut named_values, functions);
        }
        _ => panic!(),
    }
    return named_values;
}

unsafe fn codegenExpression(
    builder: prelude::LLVMBuilderRef,
    expression: &Expression,
    named_values: &mut Table,
    functions: &mut HashMap<String, prelude::LLVMValueRef>
) -> prelude::LLVMValueRef {
    match expression {
        Expression::IntegerLiteral(value) => LLVMConstInt(LLVMInt32Type(), *value as u64, 0),
        Expression::Operation(lhs, op, rhs) => {
            let lhsV = codegenExpression(builder, lhs, named_values, functions);
            let rhsV = codegenExpression(builder, rhs, named_values, functions);
            match op {
                Operator::Add => LLVMBuildAdd(builder, lhsV, rhsV, toC_char("temp_add")),
                Operator::Sub => LLVMBuildSub(builder, lhsV, rhsV, toC_char("temp_sum")),
                Operator::Div => LLVMBuildUDiv(builder, lhsV, rhsV, toC_char("temp_div")),
                Operator::Mul => LLVMBuildMul(builder, lhsV, rhsV, toC_char("temp_mul")),
                Operator::Eq => LLVMBuildICmp(
                    builder,
                    LLVMIntPredicate::LLVMIntEQ,
                    lhsV,
                    rhsV,
                    toC_char("temp_EQ"),
                ),
                Operator::Ne => LLVMBuildICmp(
                    builder,
                    LLVMIntPredicate::LLVMIntNE,
                    lhsV,
                    rhsV,
                    toC_char("temp_NE"),
                ),
                Operator::Ge => LLVMBuildICmp(
                    builder,
                    LLVMIntPredicate::LLVMIntSGE,
                    lhsV,
                    rhsV,
                    toC_char("temp_GE"),
                ),
                Operator::Gt => LLVMBuildICmp(
                    builder,
                    LLVMIntPredicate::LLVMIntSGT,
                    lhsV,
                    rhsV,
                    toC_char("temp_GT"),
                ),
                Operator::Le => LLVMBuildICmp(
                    builder,
                    LLVMIntPredicate::LLVMIntSLE,
                    lhsV,
                    rhsV,
                    toC_char("temp_LE"),
                ),
                Operator::Lt => LLVMBuildICmp(
                    builder,
                    LLVMIntPredicate::LLVMIntSLT,
                    lhsV,
                    rhsV,
                    toC_char("temp_LT"),
                ),
                Operator::As => {
                    println!("Something went wrong");
                    LLVMConstInt(LLVMInt32Type(), 0, 0)
                }
            }
        }
        Expression::Assignment(lhs, rhs) => {
            let rhsV = codegenExpression(builder, rhs, named_values, functions);
            match &**lhs {
                Var::Var(name) => {
                    let target = named_values.get(&name);
                    LLVMBuildStore(builder, rhsV, target)
                }
                Var::ArrayAccess(name, index) => {
                    let index_v = codegenExpression(builder, index, named_values, functions);
                    let zero = LLVMConstInt(LLVMInt32Type(), 0, 0);
                    let mut indicies: Vec<prelude::LLVMValueRef> = Vec::new();
                    indicies.push(zero);
                    indicies.push(index_v);
                    let lhs_v = LLVMBuildInBoundsGEP2(
                        builder,
                        LLVMInt32Type(),
                        named_values.get(&name),
                        indicies.as_mut_ptr(),
                        2,
                        toC_char("array_access"),
                    );
                    LLVMBuildStore(builder, rhsV, lhs_v)
                }
            }
        }
        Expression::Var(var) => match &**var {
            Var::ArrayAccess(name, index) => {
                let index_v = codegenExpression(builder, index, named_values, functions);
                let zero = LLVMConstInt(LLVMInt32Type(), 0, 0);
                let mut indicies: Vec<prelude::LLVMValueRef> = Vec::new();
                indicies.push(zero);
                indicies.push(index_v);
                let ptr = LLVMBuildInBoundsGEP2(
                    builder,
                    LLVMInt32Type(),
                    named_values.get(&name),
                    indicies.as_mut_ptr(),
                    2,
                    toC_char("array_access"),
                );
                return LLVMBuildLoad2(builder, LLVMInt32Type(), ptr, toC_char("temp_var_load"));
            }
            Var::Var(name) => {
                return LLVMBuildLoad2(builder, LLVMInt32Type(), named_values.get(&name), toC_char("temp_var_load"));
            }
        },
        Expression::Call(function_call) => {
            let function = *functions.get_mut(&function_call.name).unwrap();
            let mut args: Vec<prelude::LLVMValueRef> = Vec::new();
            for arg in &function_call.args {
                args.push(codegenExpression(builder, arg, named_values, functions));
            }
            LLVMBuildCall2(builder, LLVMInt32Type(), function, args.as_mut_ptr(), args.len().try_into().unwrap(), toC_char("temp_call"))
        }
    }
}

fn toC_char(string: &str) -> *mut i8 {
    return CString::new(string.as_bytes()).unwrap().as_ptr() as *mut i8;
}

struct Table {
    scope: HashMap<String, prelude::LLVMValueRef>,
    parent: Option<Box<Table>>,
}

impl Table {
    fn get(&mut self, key: &str) -> prelude::LLVMValueRef {
        match self.scope.get_mut(key) {
            Option::Some(value) => *value,
            None => match &mut self.parent {
                Some(parent) => parent.get(key),
                None => panic!(),
            },
        }
    }

    fn insert(&mut self, key: String, value: prelude::LLVMValueRef) {
        self.scope.insert(key, value);
    }
}
