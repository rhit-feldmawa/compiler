use std::collections::HashMap;
use std::mem;
use crate::ast::IdentifierType;

struct SymbolTableStack<'a> {
    pub head: Box<SymbolTable<'a>>
}

struct SymbolTable<'a> {
    pub current_scope:  HashMap<&'a str, SymbolTableElement>,
    pub parent_scope: Option<Box<SymbolTable<'a>>>,
    depth: i32
}

struct SymbolTableElement {
    pub element_type: IdentifierType,
    pub is_function: bool,
    pub depth: i32
}

impl<'a> SymbolTable<'a> {
    fn symbol_lookup(&mut self, id: &str) -> Option<&SymbolTableElement> {
        match self.current_scope.get(id) {
            Some(element) => Some(element),
            None => match &mut self.parent_scope {
                Some(parent) => parent.symbol_lookup(id),
                None => None
            }
        }
    }

    fn symbol_insert(&mut self, id: &'a str, element_type: IdentifierType, is_function: bool) {
        self.current_scope.insert(id, SymbolTableElement {
            element_type: element_type,
            is_function: is_function,
            depth: self.depth
        });
    }
}
impl<'a> SymbolTableStack<'a> {

    fn enter_scope(&mut self) {
        let empty_symbol_table: SymbolTable = SymbolTable {
            current_scope: HashMap::new(),
            parent_scope: None,
            depth: 0
        };

        let current_scope = SymbolTable {
            current_scope: HashMap::new(),
            parent_scope: Some(Box::new(mem::replace(&mut self.head, empty_symbol_table))),
            depth: self.head.depth+1
        };
        self.head = Box::new(current_scope);
    }

    fn leave_scope(&mut self) {
        let empty_symbol_table = Box::new(SymbolTable {
            current_scope: HashMap::new(),
            parent_scope: None,
            depth: 0
        });

        let old_head = mem::replace(&mut self.head, empty_symbol_table);
        match old_head.parent_scope {
            Some(parent) => self.head = parent,
            None => self.head = old_head
        }


    }
}
