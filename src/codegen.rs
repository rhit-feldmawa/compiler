use crate::ast::*;
extern crate llvm_sys as llvm;

use llvm::core::*;
use llvm::*;
use std::ffi::CString;
use std::ptr;
use std::collections::HashMap;

fn codegen(program: Program, file_name: &str) {
    unsafe {
        let context = LLVMContextCreate();
        let module = LLVMModuleCreateWithNameInContext(file_name.as_ptr() as *const _, context);
        let builder = LLVMCreateBuilderInContext(context);
        let intType = LLVMInt32Type();
        let mut functions: Vec<prelude::LLVMValueRef> = Vec::new();
        // let mut namedValues: HashMap<String, prelude::LLVMValueRef> = HashMap::new();
        let mut namedValues = Table {
            scope: HashMap::new(),
            parent: Option::None
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
                    },
                    _ => {println!("Error, formal variable in not an int");}
                }
            }
            let function_type = match &function.return_type {
                IdentifierType::Void => {
                    LLVMFunctionType(LLVMVoidTypeInContext(context), formal_vars.as_mut_ptr(), formal_vars.len().try_into().unwrap(), 0)
                },
                IdentifierType::Int => {
                    LLVMFunctionType(LLVMInt32TypeInContext(context), formal_vars.as_mut_ptr(), formal_vars.len().try_into().unwrap(), 0)
                }
            };
            let llvm_function = LLVMAddFunction(module, toC_char(&function.function_name), function_type);
            functions.push(llvm_function);
        }
        let mut i = 0;
        let mut j = 0;
        for function in &program.fun_declarations {
            namedValues = Table {
                scope: HashMap::new(),
                parent: Option::Some(Box::new(namedValues))
            };
            let bb = LLVMAppendBasicBlockInContext(context, functions[i], b"entry\0".as_ptr() as *const _);
            LLVMPositionBuilderAtEnd(builder, bb);
            for param in &program.fun_declarations[i].params {
                match param {
                    Param::Var(_, name) => {
                        let inst = LLVMBuildAlloca(builder, intType, toC_char(&name));
                        LLVMBuildStore(builder, LLVMGetParam(functions[i], j), inst);
                        namedValues.insert(name.clone(), inst);
                    },
                    _ => {
                        println!("Invaldid param");
                    }
                }
                j = j+1;
            }
            namedValues = codegenCompountStatement(builder, &function.body, namedValues);
            i = i+1;
            namedValues = *namedValues.parent.unwrap();
        }
    }
}

unsafe fn codegenCompountStatement(builder: prelude::LLVMBuilderRef, statement: &CompoundStatement, mut namedValues: Table) -> Table {
    let mut newTable = Table {
        scope: HashMap::new(),
        parent: Option::Some(Box::new(namedValues))
    };
    for declaration in &statement.declarations {
        match declaration {
            VarDeclaration::VarDeclaration(_, name) => {
                let inst = LLVMBuildAlloca(builder, LLVMInt32Type(), toC_char(&name));
                newTable.insert(name.clone(), inst);
            },
            VarDeclaration::ArrDeclaration(_, name, size) => {
                let arrType = LLVMArrayType(LLVMInt32Type(), *size as u32);
                let inst = LLVMBuildAlloca(builder, arrType, toC_char(&name));
                newTable.insert(name.clone(), inst);
            }
        }
    }

    return *newTable.parent.unwrap();
}

unsafe fn codegenStatement(builder: prelude::LLVMBuilderRef, statement: &Statement, mut namedValues: Table) -> Table {
    match statement {
        Statement::CompoundStatement(compoundStatement) => return codegenCompountStatement(builder, compoundStatement, namedValues),
        Statement::EmptyStatement => {}
        Statement::Expression(expression) => {codegenExpression(builder, expression, &namedValues);},
    }
    return namedValues
}

unsafe fn codegenExpression(builder: prelude::LLVMBuilderRef, expression: &Expression, namedValues: &Table) -> prelude::LLVMValueRef {
    match expression {
        Expression::IntegerLiteral(value) => LLVMConstInt(LLVMInt32Type(), *value as u64, 0),
        
    }
}

fn toC_char(string: &str) -> *mut i8 {
    return CString::new(string.as_bytes()).unwrap().as_ptr() as *mut i8
}

struct Table {
    scope: HashMap<String, prelude::LLVMValueRef>,
    parent: Option<Box<Table>>
}

impl Table {
    fn get(&mut self, key: &str) -> Option<prelude::LLVMValueRef> {
        match self.scope.get_mut(key) {
            Option::Some(value) => Option::Some(*value),
            None => match &mut self.parent {
                Some(parent) => parent.get(key),
                None => None
            }
        }
    }

    fn insert(&mut self, key: String, value: prelude::LLVMValueRef) {
        self.scope.insert(key, value);
    }
}


