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
    basic_blocks: Vec<BasicBlockData>,

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
                        local_info: LocalInfo::User(String::from("retval"))
                    }
                );

                // Add the function arguments to the local declarations
                for param in &func.sig.inputs {
                    self.local_decls.push(
                        LocalDecl {
                            mutable: false,
                            ty: param.ty,
                            local_info: LocalInfo::User(param.id())
                        }
                    );
                }

                // Create the entry block
                self.basic_blocks.push(
                    BasicBlockData {
                        statements: Vec::new(),
                        terminator: None,
                    }
                );

                // Enter
                match func.body {
                    Some(body) => {
                        self.build_blocks(body, BasicBlock(0));
                        
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
    fn build_blocks(
        &mut self, 
        block: Box<Block>, 
        target_block: BasicBlock
    ) -> BasicBlock {
        let mut bb: BasicBlock = target_block;

        for stmt in block.stmts {
            match stmt.kind {
                StmtKind::Expr(expr) => {
                    bb = self.build_return(expr, target_block);
                }
                StmtKind::Semi(_expr) => unimplemented!(),
                StmtKind::Let(_local) => unimplemented!(),

                _ => unimplemented!()
            }
        }

        bb
    }

    /// Handles expressions from the AST
    fn build_expr(
        &mut self, 
        expr: Box<Expr>, 
        target_block: BasicBlock, 
        target_place: &Place,
        target_type: Option<Type>
    ) -> BasicBlock {
        let mut rvalue: Option<Rvalue> = None;
        let mut bb: BasicBlock = target_block; 
        
        match expr.kind {
            ExprKind::Path(path) => {
                unimplemented!()
            }
            ExprKind::Literal(lit) => {
                rvalue = Some(self.build_number(lit, target_type));
            }
            ExprKind::If(cond, then_block, else_block) => {
                bb = self.build_if(cond, then_block, else_block, target_block, target_place, target_type);
            }
            ExprKind::Binary(left, bin_op , right) => {
                rvalue = Some(self.build_binary(
                    left, bin_op, right, target_block, target_type));
            }
            ExprKind::Unary(unop, expr) => {
                rvalue = Some(self.build_unary(
                    unop, expr, target_block, target_place, target_type));
            },
            ExprKind::Block(block) => {
                bb = self.build_blocks(block, target_block);
            }

            _ => unimplemented!()
        };

        match rvalue {
            Some(rvalue) => {
                self.basic_blocks[target_block.0].statements.push(
                    Statement::Assign(
                        Box::new(target_place.clone()),
                        Box::new(rvalue)
                    )
                );
            }
            None => {}
        }
    
        bb
    }

    /// Handles `if` expressions from the AST
    fn build_if(
        &mut self,
        cond: Box<Expr>,
        then_block: Box<Block>,
        else_block: Option<Box<Expr>>,
        target_block: BasicBlock,
        target_place: &Place,
        target_type: Option<Type>
    ) -> BasicBlock {
        // Create the blocks for ordering
        let then_bb = self.new_block();
        let else_bb: BasicBlock = match else_block {
            Some(_) => self.new_block(),
            None => BasicBlock(0)   // Dummy block
        };
        let end_bb = self.new_block();

        // Then basic block
        self.build_blocks(then_block, then_bb);
        self.basic_blocks[then_bb.0].terminator = Some(Terminator::Goto {
            target: end_bb
        });
        
        // Else basic block
        let mut dest_bb = end_bb;
        match else_block {
            Some(else_block) => {
                self.build_expr(
                    else_block,
                     else_bb, 
                     target_place, 
                     target_type
                );
                self.basic_blocks[else_bb.0].terminator = Some(Terminator::Goto {
                    target: end_bb,
                });
                dest_bb = else_bb;
            },
            None => {}
        }
        
        // Condition basic block
        let bool_type = Type::Primitive(PrimitiveType::I1);
        let cond_place = self.new_local(bool_type, LocalInfo::Temp);
        self.build_expr(cond, target_block, &cond_place, Some(bool_type));
        self.basic_blocks[target_block.0].terminator = Some(Terminator::SwitchInt {
            value: Operand::Copy(cond_place.clone()),
            targets: SwitchTargets {
                values: vec![0, 1],
                blocks: vec![then_bb, dest_bb]
            }
        });

        end_bb
    } 

    /// Handles unary operations from the AST
    fn build_unary(
        &mut self,
        unop: UnaryOpKind,
        expr: Box<Expr>,
        target_block: BasicBlock,
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

    /// Handles binary operations from the AST
    fn build_binary(
        &mut self, 
        left: Box<Expr>, 
        bin_op: BinOpKind, 
        right: Box<Expr>, 
        target_block: BasicBlock,
        target_type: Option<Type>
    ) -> Rvalue {
        let lplace = self.new_local(target_type.unwrap(), LocalInfo::Temp);
        let rplace = self.new_local(target_type.unwrap(), LocalInfo::Temp);
    
        self.call_frame.push(CallFrame::Binary);
        self.build_expr(left, target_block, &lplace, target_type);
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
            BinOpKind::BitAnd => BinOp::BitAnd,
            BinOpKind::BitOr => BinOp::BitOr,
            BinOpKind::BitXor => BinOp::BitXor,
            BinOpKind::Eq => BinOp::Eq,
            BinOpKind::Ne => BinOp::Ne,
            BinOpKind::Lt => BinOp::Lt,
            BinOpKind::Gt => BinOp::Gt,
            BinOpKind::Le => BinOp::Le,
            BinOpKind::Ge => BinOp::Ge,
            _ => unimplemented!()
        };

        Rvalue::BinaryOp(
            binop,
            Box::new(Operand::Copy(lplace)),
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
            ty: Type::Primitive(PrimitiveType::I32),
            value: value,
        });
        self.consts.push(Rc::clone(&this_const));

        Rvalue::Use(Operand::Constant(Rc::clone(&this_const)))
    }

    /// Helper function to generate a new basic block
    fn new_block(&mut self) -> BasicBlock {
        self.basic_blocks.push(
            BasicBlockData {
                statements: Vec::new(),
                terminator: None,
            }
        );

        BasicBlock(self.basic_blocks.len() - 1)
    }

    /// Helper function to generate a new "local" place
    fn new_local(&mut self, ty: Type, local_info: LocalInfo) -> Place {
        let local_decl = LocalDecl {
            mutable: false,
            ty: ty,
            local_info: local_info,
        };
        self.local_decls.push(local_decl);

        Place::Local(self.local_decls.len() - 1)
    }

    /// Loads up the return register `_0` with the value of the expression
    /// with a terminator instruction of `Return`
    fn build_return(
        &mut self, 
        node: Box<Expr>, 
        target_block: BasicBlock
    ) -> BasicBlock {
        self.call_frame.push(CallFrame::Return);
        let target_bb = self.build_expr(
            node, 
            target_block, 
            &Place::Local(0),
            Some(self.local_decls[0].ty)
        );
        self.call_frame.pop();

        self.basic_blocks[target_bb.0].terminator = Some(Terminator::Return);       

        target_bb 
    }

}


