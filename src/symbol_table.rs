use std::collections::HashMap;
use std::sync::Arc;
use crate::ast::{IdentifierType};

pub struct SymbolTable {
    pub current_scope:  HashMap<String, SymbolTableElement>,
    pub parent_scope: Option<Arc<SymbolTable>>,
    pub depth: i32
}

pub enum SymbolTableElement {
    Variable(VariableSymbolTableElement),
    Array(ArraySymbolTableElement),
    ParameterArray(ParameterArraySymbolTableElement),
    Function(FunctionSymbolTableElement)
}

pub struct VariableSymbolTableElement {
    pub element_type: IdentifierType,
    pub depth: i32
}

pub struct ArraySymbolTableElement {
    pub element_type: IdentifierType,
    pub size: i32,
    pub depth: i32
}

pub struct ParameterArraySymbolTableElement {
    pub element_type: IdentifierType,
    pub depth: i32
}

pub struct FunctionSymbolTableElement {
    pub return_type: IdentifierType,
    pub argument_types: Vec<Param>,
    pub depth: i32,
}

pub enum Param {
    Var(IdentifierType),
    Arr(IdentifierType)
}

#[derive(PartialEq)]
pub enum ExpressionType {
    Array,
    Int,
    Void,
}

impl SymbolTable {
    pub fn symbol_lookup(&self, id: &str) -> Option<&SymbolTableElement> {
        match self.current_scope.get(id) {
            Some(element) => Some(element),
            None => match &self.parent_scope {
                Some(parent) => parent.symbol_lookup(id),
                None => None
            }
        }
    }

    pub fn symbol_insert(&mut self, id: &str, element: SymbolTableElement) -> bool {
        if self.current_scope.contains_key(id) {
            return false;
        } else {
            self.current_scope.insert(id.to_string(), element);
            return true;
        }
    }
}

pub fn get_child_table(parent: Arc<SymbolTable>) -> SymbolTable {
    let depth = *&parent.depth;
    SymbolTable {
        parent_scope: Some(parent),
        current_scope: HashMap::new(),
        depth
    }
}
