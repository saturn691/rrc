//! Lowers the AST to the HIR.

use ast::*;
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
    Binary,
    Unary
}

#[derive(Debug)]
struct Builder {
    basic_blocks: Vec<BasicBlock>,

    /// 1. Return value
    /// 2. Function arguments
    /// 3. Local variables
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
        match node.kind {
            NodeKind::Fn(func) => {
                // Add the return type to the local declarations
                self.local_decls.push(
                    LocalDecl {
                        mutable: false,
                        ty: func.sig.return_type,
                    }
                );

                // Add the function arguments to the local declarations
                for param in &func.sig.inputs {
                    self.local_decls.push(
                        LocalDecl {
                            mutable: false,
                            ty: param.ty,
                        }
                    );
                }

                // Create the entry block
                self.basic_blocks.push(
                    BasicBlock {
                        statements: Vec::new(),
                        terminator: None,
                    }
                );

                // Enter
                match func.body {
                    Some(body) => {
                        self.build_blocks(body, 0);
                        
                        Ok(
                            Body {
                                name: node.identifier.unwrap(),
                                basic_blocks: self.basic_blocks.clone(),
                                local_decls: self.local_decls.clone(),
                                consts: self.consts.clone(),
                                arg_count: func.sig.inputs.len(),
                            }
                        )
                    }
                    None => {
                        Err("Function body is missing".to_string())
                    }
                }

            }
           
            _ => Err("Expected a function definition".to_string())
        }
    }

    /// Handles switching between the different enums
    fn build_blocks(&mut self, block: Box<Block>, target_block: usize) {
        for stmt in block.stmts {
            match stmt.kind {
                StmtKind::Expr(expr) => {
                    self.build_return(expr, target_block);
                }

                _ => unimplemented!()
            }
        }
    }

    fn build_expr(
        &mut self, 
        expr: Box<Expr>, 
        target_block: usize, 
        target_place: &Place,
        target_type: Option<Type>
    ) {
        let rvalue: Rvalue = match expr.kind {
            ExprKind::Path(path) => {
                unimplemented!()
            }
            ExprKind::Literal(lit) => {
                self.build_number(lit, target_type)
            }
            ExprKind::Binary(left, bin_op , right) => {
                self.build_binary(left, bin_op, right, target_block, target_place, target_type)
            }
            ExprKind::Unary(unop, expr) => {
                self.build_unary(unop, expr, target_block, target_place, target_type)
            }

            _ => unimplemented!()
        };

        self.basic_blocks[target_block].statements.push(
            Statement::Assign(
                Box::new(target_place.clone()),
                Box::new(rvalue)
            )
        )
    }

    fn build_unary(
        &mut self,
        unop: UnaryOpKind,
        expr: Box<Expr>,
        target_block: usize,
        target_place: &Place,
        target_type: Option<Type>
    ) -> Rvalue {
        let unop = match unop {
            UnaryOpKind::Negate => UnOp::Neg,
            UnaryOpKind::Not => UnOp::Not,
            _ => unimplemented!()
        };

        self.call_frame.push(CallFrame::Unary);
        self.build_expr(expr, target_block, target_place, target_type);
        self.call_frame.pop();

        Rvalue::UnaryOp(
            unop,
            Box::new(Operand::Copy(target_place.clone()))
        )
        
    }

    fn build_binary(
        &mut self, 
        left: Box<Expr>, 
        bin_op: BinOpKind, 
        right: Box<Expr>, 
        target_block: usize,
        target_place: &Place,
        target_type: Option<Type>
    ) -> Rvalue {
        let rplace = self.new_local(target_type.unwrap());
    
        self.call_frame.push(CallFrame::Binary);
        self.build_expr(left, target_block, target_place, target_type);
        self.build_expr(right, target_block, &rplace, target_type);
        self.call_frame.pop();

        let binop = match bin_op {
            BinOpKind::Plus => BinOp::Add,
            BinOpKind::Minus => BinOp::Sub,
            BinOpKind::Multiply => BinOp::Mul,
            BinOpKind::Divide => BinOp::Div,
            BinOpKind::Modulo => BinOp::Rem,
            BinOpKind::ShiftLeft => BinOp::ShiftLeft,
            BinOpKind::ShiftRight => BinOp::ShiftRight,
            BinOpKind::And => BinOp::BitAnd,
            BinOpKind::Or => BinOp::BitOr,
            BinOpKind::Xor => BinOp::BitXor,
            _ => unimplemented!()
        };

        Rvalue::BinaryOp(
            binop,
            Box::new(Operand::Copy(target_place.clone())),
            Box::new(Operand::Copy(rplace))
        )
    }

    fn build_number(
        &mut self, 
        value: String, 
        target_type: Option<Type>
    ) -> Rvalue {
        // Add to the list of constant values
        let this_const = Rc::new(Const {
            ty: target_type.unwrap_or(Type::Primitive(PrimitiveType::I32)),
            value: value,
        });
        self.consts.push(Rc::clone(&this_const));

        Rvalue::Use(Operand::Constant(Rc::clone(&this_const)))
    }

    /// Helper function to generate a new "local" place
    fn new_local(&mut self, ty: Type) -> Place {
        let local_decl = LocalDecl {
            mutable: false,
            ty: ty,
        };
        self.local_decls.push(local_decl);

        Place::Local(self.local_decls.len() - 1)
    }

    /// Loads up the return register `_0` with the value of the expression
    /// with a terminator instruction of `Return`
    fn build_return(&mut self, node: Box<Expr>, target_block: usize) {
        self.call_frame.push(CallFrame::Return);
        self.build_expr(
            node, 
            target_block, 
            &Place::Local(0),
            Some(self.local_decls[0].ty)
        );
        self.call_frame.pop();

        self.basic_blocks[target_block].terminator = Some(Terminator::Return);        
    }

}


