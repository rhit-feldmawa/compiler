use crate::ast::*;
extern crate llvm_sys as llvm;

use llvm::core::*;
use llvm::*;
use std::collections::HashMap;
use std::ffi::CString;

static mut RET_BLOCK: Option<prelude::LLVMBasicBlockRef> = Option::None;
static mut RET_VAL: Option<prelude::LLVMValueRef> = Option::None;

pub fn codegen(program: &Program, file_name: &str) {
    unsafe {
        let context = LLVMContextCreate();
        let module = LLVMModuleCreateWithNameInContext(to_c_string(file_name), context);
        let builder = LLVMCreateBuilderInContext(context);
        let int_type = LLVMInt32Type();
        let mut functions: HashMap<String, prelude::LLVMValueRef> = HashMap::new();
        let mut named_values = Table {
            scope: HashMap::new(),
            parent: Option::None,
        };
        for var_declaration in &program.var_declarations {
            match &var_declaration {
                VarDeclaration::ArrDeclaration(_, name, size) => {
                    LLVMAddGlobal(
                        module,
                        LLVMArrayType(int_type, *size as u32),
                        to_c_string(name),
                    );
                    let g_var = LLVMGetNamedGlobal(module, to_c_string(&name));
                    LLVMSetLinkage(g_var, LLVMLinkage::LLVMCommonLinkage);
                    let zero = LLVMConstInt(LLVMInt32Type(), 0, 0);
                    let mut array = Vec::new();
                    let mut i = 0;
                    while i < *size {
                        array.push(zero);
                        i = i + 1;
                    }
                    LLVMSetInitializer(g_var, LLVMConstArray(LLVMInt32Type(), array.as_mut_ptr(), *size as u32));
                    named_values.insert(name.clone(), g_var);
                }
                VarDeclaration::VarDeclaration(_, name) => {
                    LLVMAddGlobal(module, int_type, to_c_string(&name));
                    let g_var = LLVMGetNamedGlobal(module, to_c_string(&name));
                    LLVMSetLinkage(g_var, LLVMLinkage::LLVMCommonLinkage);
                    LLVMSetInitializer(g_var, LLVMConstInt(LLVMInt32Type(), 0, 0));
                    named_values.insert(name.clone(), g_var);
                }
            }
        }

        for function in &program.fun_declarations {
            let mut formal_vars: Vec<prelude::LLVMTypeRef> = Vec::new();
            for arg in &function.params {
                match &arg {
                    Param::Var(_, _) => {
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
                LLVMAddFunction(module, to_c_string(&function.function_name), function_type);
            functions.insert(function.function_name.clone(), llvm_function);
        }
        let mut i = 0;
        let mut j = 0;
        for function in &program.fun_declarations {
            named_values = Table {
                scope: HashMap::new(),
                parent: Option::Some(Box::new(named_values)),
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
                        let inst = LLVMBuildAlloca(builder, int_type, to_c_string(&name));
                        LLVMBuildStore(
                            builder,
                            LLVMGetParam(*functions.get_mut(&function.function_name).unwrap(), j),
                            inst,
                        );
                        named_values.insert(name.clone(), inst);
                    }
                    _ => {
                        println!("Invaldid param");
                    }
                }
                j = j + 1;
            }
            RET_VAL = Option::Some(LLVMBuildAlloca(
                builder,
                LLVMInt32Type(),
                to_c_string("ret_value"),
            ));
            named_values = codegen_compount_statement(
                builder,
                &function.body,
                named_values,
                &mut functions,
                context,
                &function.function_name,
            );
            match RET_BLOCK {
                Option::Some(block) => {
                    if LLVMGetBasicBlockTerminator(LLVMGetLastBasicBlock(
                        *functions.get_mut(&function.function_name).unwrap(),
                    )).is_null()
                    {
                        LLVMBuildBr(builder, block);
                    }
                    LLVMAppendExistingBasicBlock(
                        *functions.get_mut(&function.function_name).unwrap(),
                        block,
                    );
                    LLVMPositionBuilderAtEnd(builder, block);
                    match RET_VAL {
                        Option::Some(val) => {
                            let val_v = LLVMBuildLoad2(
                                builder,
                                LLVMInt32Type(),
                                val,
                                to_c_string("ret_val"),
                            );
                            LLVMBuildRet(builder, val_v);
                        }
                        Option::None => {
                            LLVMBuildRetVoid(builder);
                        }
                    }
                }
                Option::None => {
                    LLVMBuildRetVoid(builder);
                }
            }
            RET_VAL = Option::None;
            RET_BLOCK = Option::None;
            i = i + 1;
            named_values = *named_values.parent.unwrap();
        }
        LLVMDumpModule(module);
        // analysis::LLVMVerifyModule(
        //     module,
        //     analysis::LLVMVerifierFailureAction::LLVMPrintMessageAction,
        //     Box::into_raw(Box::new("")) as *mut *mut i8,
        // );
        bit_writer::LLVMWriteBitcodeToFile(module, to_c_string("out.bc"));
    }
}

unsafe fn codegen_compount_statement(
    builder: prelude::LLVMBuilderRef,
    statement: &CompoundStatement,
    named_values: Table,
    functions: &mut HashMap<String, prelude::LLVMValueRef>,
    context: prelude::LLVMContextRef,
    function_name: &str,
) -> Table {
    let mut new_table = Table {
        scope: HashMap::new(),
        parent: Option::Some(Box::new(named_values)),
    };
    for declaration in &statement.declarations {
        match declaration {
            VarDeclaration::VarDeclaration(_, name) => {
                let inst = LLVMBuildAlloca(builder, LLVMInt32Type(), to_c_string(&name));
                new_table.insert(name.clone(), inst);
            }
            VarDeclaration::ArrDeclaration(_, name, size) => {
                let arr_type = LLVMArrayType(LLVMInt32Type(), *size as u32);
                let inst = LLVMBuildAlloca(builder, arr_type, to_c_string(&name));
                new_table.insert(name.clone(), inst);
            }
        }
    }
    for statement in &statement.statements {
        new_table = codegen_statement(
            builder,
            &statement,
            new_table,
            functions,
            context,
            function_name,
        );
    }

    return *new_table.parent.unwrap();
}

unsafe fn codegen_statement(
    builder: prelude::LLVMBuilderRef,
    statement: &Statement,
    mut named_values: Table,
    functions: &mut HashMap<String, prelude::LLVMValueRef>,
    context: prelude::LLVMContextRef,
    function: &str,
) -> Table {
    match statement {
        Statement::CompoundStatement(compound_statement) => {
            return codegen_compount_statement(
                builder,
                compound_statement,
                named_values,
                functions,
                context,
                function,
            )
        }
        Statement::EmptyStatement => {}
        Statement::Expression(expression) => {
            codegen_expression(builder, expression, &mut named_values, functions);
        }
        Statement::ReturnStatement(value) => match value {
            Option::Some(expression) => {
                let value_v = codegen_expression(builder, expression, &mut named_values, functions);
                match RET_BLOCK {
                    Option::Some(block) => {
                        LLVMBuildStore(builder, value_v, RET_VAL.unwrap());
                        LLVMBuildBr(builder, block);
                    }
                    Option::None => {
                        RET_BLOCK = Option::Some(LLVMCreateBasicBlockInContext(
                            context,
                            to_c_string("ret_block"),
                        ));
                        LLVMBuildStore(builder, value_v, RET_VAL.unwrap());
                        LLVMBuildBr(builder, RET_BLOCK.unwrap());
                    }
                }
            }
            Option::None => match RET_BLOCK {
                Option::Some(block) => {
                    LLVMBuildBr(builder, block);
                }
                Option::None => {
                    RET_BLOCK = Option::Some(LLVMCreateBasicBlockInContext(
                        context,
                        to_c_string("ret_block"),
                    ));
                    LLVMBuildBr(builder, RET_BLOCK.unwrap());
                }
            },
        },
        Statement::IfStatement(if_statement) => match &**if_statement {
            IfStatement::IfStmt(cond, stmt) => {
                let cond_v = codegen_expression(builder, cond, &mut named_values, functions);
                let then_block = LLVMAppendBasicBlock(
                    *functions.get_mut(function).unwrap(),
                    to_c_string("then_block"),
                );
                let merge_block =
                    LLVMCreateBasicBlockInContext(context, to_c_string("merge_block"));
                LLVMBuildCondBr(builder, cond_v, then_block, merge_block);
                LLVMPositionBuilderAtEnd(builder, then_block);
                named_values =
                    codegen_statement(builder, stmt, named_values, functions, context, function);
                if LLVMGetBasicBlockTerminator(LLVMGetLastBasicBlock(
                    *functions.get_mut(function).unwrap(),
                )).is_null()
                {
                    LLVMBuildBr(builder, merge_block);
                }
                LLVMAppendExistingBasicBlock(*functions.get_mut(function).unwrap(), merge_block);
                LLVMPositionBuilderAtEnd(builder, merge_block);
            }
            IfStatement::IfElseStmt(cond, stmt1, stmt2) => {
                let cond_v = codegen_expression(builder, cond, &mut named_values, functions);
                let then_block = LLVMAppendBasicBlock(
                    *functions.get_mut(function).unwrap(),
                    to_c_string("then_block"),
                );
                let else_block = LLVMCreateBasicBlockInContext(context, to_c_string("else_block"));
                let merge_block =
                    LLVMCreateBasicBlockInContext(context, to_c_string("merge_block"));
                LLVMBuildCondBr(builder, cond_v, then_block, else_block);
                LLVMPositionBuilderAtEnd(builder, then_block);
                named_values =
                    codegen_statement(builder, stmt1, named_values, functions, context, function);
                if LLVMGetBasicBlockTerminator(LLVMGetLastBasicBlock(
                    *functions.get_mut(function).unwrap(),
                )).is_null()
                {
                    LLVMBuildBr(builder, merge_block);
                }
                LLVMAppendExistingBasicBlock(*functions.get_mut(function).unwrap(), else_block);
                LLVMPositionBuilderAtEnd(builder, else_block);
                named_values =
                    codegen_statement(builder, stmt2, named_values, functions, context, function);
                if LLVMGetBasicBlockTerminator(LLVMGetLastBasicBlock(
                    *functions.get_mut(function).unwrap(),
                )).is_null()
                {
                    LLVMBuildBr(builder, merge_block);
                }
                LLVMAppendExistingBasicBlock(*functions.get_mut(function).unwrap(), merge_block);
                LLVMPositionBuilderAtEnd(builder, merge_block);
            }
        },
        Statement::WhileStatement(stmt) => {
            let cond_block = LLVMAppendBasicBlock(
                *functions.get_mut(function).unwrap(),
                to_c_string("cond_block"),
            );
            let loop_block = LLVMAppendBasicBlock(
                *functions.get_mut(function).unwrap(),
                to_c_string("loop_block"),
            );
            let merge_block = LLVMCreateBasicBlockInContext(context, to_c_string("merge_block"));
            LLVMPositionBuilderAtEnd(builder, cond_block);
            let cond_v = codegen_expression(builder, &stmt.condition, &mut named_values, functions);
            LLVMBuildCondBr(builder, cond_v, loop_block, merge_block);
            LLVMAppendExistingBasicBlock(*functions.get_mut(function).unwrap(), loop_block);
            LLVMPositionBuilderAtEnd(builder, loop_block);
            named_values = codegen_statement(
                builder,
                &stmt.statement,
                named_values,
                functions,
                context,
                function,
            );
            if LLVMGetBasicBlockTerminator(LLVMGetLastBasicBlock(
                *functions.get_mut(function).unwrap(),
            )).is_null()
            {
                LLVMBuildBr(builder, cond_block);
            }
            LLVMAppendExistingBasicBlock(*functions.get_mut(function).unwrap(), merge_block);
            LLVMPositionBuilderAtEnd(builder, merge_block);
        }
    }
    return named_values;
}

unsafe fn codegen_expression(
    builder: prelude::LLVMBuilderRef,
    expression: &Expression,
    named_values: &mut Table,
    functions: &mut HashMap<String, prelude::LLVMValueRef>,
) -> prelude::LLVMValueRef {
    match expression {
        Expression::IntegerLiteral(value) => LLVMConstInt(LLVMInt32Type(), *value as u64, 0),
        Expression::Operation(lhs, op, rhs) => {
            let lhs_v = codegen_expression(builder, lhs, named_values, functions);
            let rhs_v = codegen_expression(builder, rhs, named_values, functions);
            match op {
                Operator::Add => LLVMBuildAdd(builder, lhs_v, rhs_v, to_c_string("temp_add")),
                Operator::Sub => LLVMBuildSub(builder, lhs_v, rhs_v, to_c_string("temp_sum")),
                Operator::Div => LLVMBuildUDiv(builder, lhs_v, rhs_v, to_c_string("temp_div")),
                Operator::Mul => LLVMBuildMul(builder, lhs_v, rhs_v, to_c_string("temp_mul")),
                Operator::Eq => LLVMBuildICmp(
                    builder,
                    LLVMIntPredicate::LLVMIntEQ,
                    lhs_v,
                    rhs_v,
                    to_c_string("temp_EQ"),
                ),
                Operator::Ne => LLVMBuildICmp(
                    builder,
                    LLVMIntPredicate::LLVMIntNE,
                    lhs_v,
                    rhs_v,
                    to_c_string("temp_NE"),
                ),
                Operator::Ge => LLVMBuildICmp(
                    builder,
                    LLVMIntPredicate::LLVMIntSGE,
                    lhs_v,
                    rhs_v,
                    to_c_string("temp_GE"),
                ),
                Operator::Gt => LLVMBuildICmp(
                    builder,
                    LLVMIntPredicate::LLVMIntSGT,
                    lhs_v,
                    rhs_v,
                    to_c_string("temp_GT"),
                ),
                Operator::Le => LLVMBuildICmp(
                    builder,
                    LLVMIntPredicate::LLVMIntSLE,
                    lhs_v,
                    rhs_v,
                    to_c_string("temp_LE"),
                ),
                Operator::Lt => LLVMBuildICmp(
                    builder,
                    LLVMIntPredicate::LLVMIntSLT,
                    lhs_v,
                    rhs_v,
                    to_c_string("temp_LT"),
                ),
                Operator::As => {
                    println!("Something went wrong");
                    LLVMConstInt(LLVMInt32Type(), 0, 0)
                }
            }
        }
        Expression::Assignment(lhs, rhs) => {
            let rhs_v = codegen_expression(builder, rhs, named_values, functions);
            match &**lhs {
                Var::Var(name) => {
                    let target = named_values.get(&name);
                    LLVMBuildStore(builder, rhs_v, target)
                }
                Var::ArrayAccess(name, index) => {
                    let index_v = codegen_expression(builder, index, named_values, functions);
                    let zero = LLVMConstInt(LLVMInt32Type(), 0, 0);
                    let mut indicies: Vec<prelude::LLVMValueRef> = Vec::new();
                    indicies.push(zero);
                    indicies.push(index_v);
                    let lhs_v = LLVMBuildInBoundsGEP2(
                        builder,
                        LLVMInt32Type(),
                        named_values.get(&name),
                        indicies.as_mut_ptr(),
                        1,
                        to_c_string("array_access"),
                    );
                    LLVMBuildStore(builder, rhs_v, lhs_v)
                }
            }
        }
        Expression::Var(var) => match &**var {
            Var::ArrayAccess(name, index) => {
                let index_v = codegen_expression(builder, index, named_values, functions);
                let zero = LLVMConstInt(LLVMInt32Type(), 0, 0);
                let mut indicies: Vec<prelude::LLVMValueRef> = Vec::new();
                indicies.push(zero);
                indicies.push(zero);
                let ptr = LLVMBuildInBoundsGEP2(
                    builder,
                    LLVMInt32Type(),
                    named_values.get(&name),
                    indicies.as_mut_ptr(),
                    1,
                    to_c_string("array_access"),
                );
                return LLVMBuildLoad2(builder, LLVMInt32Type(), ptr, to_c_string("temp_var_load"));
            }
            Var::Var(name) => {
                return LLVMBuildLoad2(
                    builder,
                    LLVMInt32Type(),
                    named_values.get(&name),
                    to_c_string("temp_var_load"),
                );
            }
        },
        Expression::Call(function_call) => {
            let function = *functions.get_mut(&function_call.name).unwrap();
            let mut args: Vec<prelude::LLVMValueRef> = Vec::new();
            for arg in &function_call.args {
                args.push(codegen_expression(builder, arg, named_values, functions));
            }
            LLVMBuildCall2(
                builder,
                LLVMInt32Type(),
                function,
                args.as_mut_ptr(),
                args.len().try_into().unwrap(),
                to_c_string("temp_call"),
            )
        }
    }
}

fn to_c_string(string: &str) -> *mut i8 {
    return CString::new(string.as_bytes()).unwrap().into_raw() as *mut i8;
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
