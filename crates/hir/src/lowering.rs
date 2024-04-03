//! Lowers the AST to the HIR.

use ast::Node;
use std::rc::Rc;
use crate::*;

/// Entry point for lowering the AST to the HIR
pub fn hir_build(root: Node) -> Result<Body, String> {
    let mut builder: Builder = Builder::new();
    builder.build_fn(root)
}

#[derive(Debug)]
enum CallFrame {
    Return,
}

#[derive(Debug)]
struct Builder {
    basic_blocks: Vec<BasicBlock>,
    local_decls: Vec<LocalDecl>,
    consts: Vec<Rc<Const>>,
    call_frame: Vec<CallFrame>,
}


impl Builder {
    fn new() -> Self {
        Builder {
            basic_blocks: Vec::new(),
            local_decls: Vec::new(),
            consts: Vec::new(),
            call_frame: Vec::new(),
        }
    }
    
    fn build_fn(&mut self, node: Node) -> Result<Body, String> {
        match node {
            Node::FunctionDef { 
                name, 
                params, 
                return_type, 
                body 
            } => {
                // Add the return type to the local declarations
                self.local_decls.push(LocalDecl {
                    mutable: false,
                    ty: return_type,
                });
                
                // Add the function arguments to the local declarations
                for param in &params {
                    self.local_decls.push(LocalDecl {
                        mutable: false,
                        // TODO - Fix this
                        ty: ast::Type::Primitive(ast::PrimitiveType::I8)
                    });
                }

                // Create the entry block
                self.basic_blocks.push(BasicBlock {
                    statements: Vec::new(),
                    terminator: None,
                });

                // Enter
                self.build_blocks(body, 0);
                
                Ok(Body {
                    name: name,
                    basic_blocks: self.basic_blocks.clone(),
                    local_decls: self.local_decls.clone(),
                    consts: self.consts.clone(),
                    arg_count: params.len(),
                })
            }
            _ => Err("Expected a function definition".to_string())
        }
    }

    /// Handles switching between the different enums
    fn build_blocks(&mut self, node: Box<Node>, target_block: usize) {
        match *node {
            Node::Statements { statements } => {
                for statement in statements {
                    self.build_blocks(statement, target_block);
                }
            }
            Node::Number { value } => {
                self.build_number(value, target_block);
            }
            Node::Return { value } => {
                self.build_return(value, target_block);
            }
            _ => {}
        }
    }

    fn build_number(&mut self, value: String, target_block: usize) {
        // Add to the list of constant values
        // TODO types
        let this_const = Rc::new(Const {
            ty: ast::Type::Primitive(ast::PrimitiveType::I32),
            value: value,
        });
        self.consts.push(Rc::clone(&this_const));

        match self.call_frame.last() {
            Some(CallFrame::Return) => {
                self.basic_blocks[target_block].statements.push(
                    Statement::Assign(
                        Place::Local(0),
                        Rvalue::Use(Operand::Constant(Rc::clone(&this_const)))
                    )
                )
            }
            _ => {}
        }

    }

    /// Loads up the return register `_0` with the value of the expression
    /// with a terminator instruction of `Return`
    fn build_return(&mut self, node: Box<Node>, target_block: usize) {

        self.call_frame.push(CallFrame::Return);
        self.build_blocks(node, target_block);
        self.call_frame.pop();

        self.basic_blocks[target_block].terminator = Some(Terminator::Return);        
    }

}


