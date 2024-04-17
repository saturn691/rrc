//! The intermediate representation (IR) for the compiler.

pub mod lowering;
pub mod graph;

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
pub struct SwitchTargets {
    pub values: Vec<i128>,
    pub blocks: Vec<BasicBlock>,
}

#[derive(Clone, Debug)]
pub enum Terminator {
    Goto { target: BasicBlock },
    /// A function call
    Call {
        /// The function to call
        func: Operand,
        /// The arguments to the function
        args: Vec<Operand>,
        /// The destination of the return value
        destination: Place,
        /// The block to jump to if the function returns
        target: BasicBlock,
    },
    Return,
    /// Switches based on the computed value
    SwitchInt {
        /// The value to switch on
        value: Operand,
        /// The targets for each case
        targets: SwitchTargets,
    },
}

impl Terminator {
    pub fn successor(&self) -> BasicBlock {
        use Terminator::*;
        match self {
            Goto { target } => *target,
            Call { target, .. } => *target,
            Return => panic!("Return has no successor"),
            SwitchInt { targets, .. } => targets.blocks[0],
        }
    }
    
    pub fn successors(&self) -> Vec<BasicBlock> {
        use Terminator::*;
        match self {
            Goto { target } => vec![*target],
            Call { target, .. } => vec![*target],
            Return => vec![],
            SwitchInt { targets, .. } => targets.blocks.clone(),
        }
    }
}

/// Extra information about a local variable which is needed for code generation.
#[derive(Clone, Debug)]
pub enum LocalInfo {
    /// User defined local variable or function parameter
    User(String),
    /// Temporary variables used by the compiler
    Temp,
}

#[derive(Clone, Debug)]
pub struct LocalDecl {
    pub mutable: bool,
    pub ty: Type,
    pub local_info: LocalInfo
}

impl LocalDecl {
    pub fn size(&self) -> usize {
        self.ty.size()
    }
}

/// Pointer to the basic block (`BasicBlockData`) in the Control Flow Graph (CFG).
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct BasicBlock(pub usize);

/// A node in the Control Flow Graph (CFG).
#[derive(Clone, Debug)]
pub struct BasicBlockData {
    pub statements: Vec<Statement>,
    pub terminator: Option<Terminator>,
}

impl BasicBlockData {
    pub fn terminator(&self) -> &Terminator {
        self.terminator.as_ref().unwrap()
    }
}

/// The main data structure for the intermediate representation.
#[derive(Debug)]
pub struct Body {
    /// The name of the function e.g. `foo`
    pub name: String,

    /// The basic blocks, should be accessed via BasicBlock
    pub basic_blocks: Vec<BasicBlockData>,

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
        basic_blocks: Vec<BasicBlockData>,
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

    pub fn get_operand_type(&self, operand: &Operand) -> Type {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => {
                let idx = match place {
                    Place::Local(idx) => *idx,
                };
                self.local_decls[idx].ty.clone()
            }
            Operand::Constant(constant) => constant.ty.clone(),
        }
    }
}