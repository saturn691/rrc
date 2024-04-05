//! The intermediate representation (IR) for the compiler.

pub mod lowering;

use ast::types::*;
use std::rc::Rc;
pub use lowering::hir_build;

#[derive(Clone, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    BitXor,
    BitAnd,
    BitOr,
    ShiftLeft,
    ShiftRight,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Clone, Debug)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Place {
    /// Points to the `local_decl`
    Local(usize),
    
    // TODO globals
}

#[derive(Clone, Debug)]
pub enum Rvalue {
    /// Operand unchanged
    Use(Operand),
    /// Binary operation
    BinaryOp(BinOp, Box<Operand>, Box<Operand>),
    /// Unary operation
    UnaryOp(UnOp, Box<Operand>),
}

#[derive(Clone, Debug)]
pub struct Const {
    pub ty: Type,
    pub value: String,
}

#[derive(Clone, Debug)]
pub enum Operand {
    Copy(Place),
    Move(Place),
    Constant(Rc<Const>),
}

#[derive(Clone, Debug)]
pub enum Statement {
    Assign(Box<Place>, Box<Rvalue>),
}

#[derive(Clone, Debug)]
pub enum Terminator {
    Goto { target: Box<BasicBlock> },
    /// A function call
    Call {
        /// The function to call
        func: Operand,
        /// The arguments to the function
        args: Vec<Operand>,
        /// The destination of the return value
        destination: Place,
        /// The block to jump to if the function returns
        target: Box<BasicBlock>,
    },
    Return,
}

#[derive(Clone, Debug)]
pub struct LocalDecl {
    pub mutable: bool,
    pub ty: Type
}

/// A node in the Control Flow Graph (CFG).
#[derive(Clone, Debug)]
pub struct BasicBlock {
    pub statements: Vec<Statement>,
    pub terminator: Option<Terminator>,
}


/// The main data structure for the intermediate representation.
#[derive(Debug)]
pub struct Body {
    /// The name of the function e.g. `foo`
    pub name: String,

    pub basic_blocks: Vec<BasicBlock>,

    /// Ordered like:
    /// 1. return value
    /// 2. arguments
    /// 3. locals
    pub local_decls: Vec<LocalDecl>,

    pub consts: Vec<Rc<Const>>,

    pub arg_count: usize,
}

impl Body {
    pub fn new(
        name: String,
        basic_blocks: Vec<BasicBlock>,
        local_decls: Vec<LocalDecl>,
        consts: Vec<Rc<Const>>,
        arg_count: usize,
    ) -> Self {
        Body {
            name,
            basic_blocks,
            local_decls,
            consts,
            arg_count,
        }
    }
}