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
        for function in &program.fun_declarations {
            let bb = LLVMAppendBasicBlockInContext(context, functions[i], b"entry\0".as_ptr() as *const _);
            LLVMPositionBuilderAtEnd(builder, bb);
            for param in &program.fun_declarations[i].params {
                match param {
                    Param::Var(_, name) => {
                        // let inst = LLVMBuildAlloca(builder, intType, CString::new(name.as_bytes()).unwrap().as_ptr() as *const _);
                        let inst = LLVMBuildAlloca(builder, intType, toC_char(&name));

                    },
                    _ => {
                        println!("Invaldid param");
                    }
                }
            }
        }
    }
}

fn toC_char(string: &str) -> *mut i8 {
    return CString::new(string.as_bytes()).unwrap().as_ptr() as *mut i8
}
