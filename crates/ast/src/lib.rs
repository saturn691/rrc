pub mod types;

pub use types::*;

#[derive(Debug)]
pub enum LiteralKind {
    /// Boolean literal e.g. `true`
    Boolean,
    /// Character literal e.g. `'c'`
    Char,
    /// Integer literal e.g. `42`
    Integer,
    /// Float literal e.g. `3.14`
    Float,
    /// String literal e.g. `"hello"`
    String,
    /// Byte literal e.g. `b'c'`
    Byte,
    /// Byte string literal e.g. `b"hello"`
    ByteString,
    /// Byte character literal e.g. `b'c'`
    ByteChar,
    /// Raw string literal e.g. `r"hello"`
    RawString,
    /// Raw byte string literal e.g. `br"hello"`
    RawByteString,
    /// Raw character literal e.g. `r'c'`
    RawChar,
    /// Raw byte character literal e.g. `br'c'`
    RawByteChar,
}

#[derive(Clone, Debug)]
pub enum BinOpKind {
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Multiply,
    /// `/`
    Divide,
    /// `%`
    Modulo,
    /// `^`
    Exponent,
    /// `&`
    And,
    /// `|`
    Or,
    /// `<<`
    ShiftLeft,
    /// `>>`
    ShiftRight,
}

#[derive(Clone, Debug)]
pub enum UnaryOpKind {
    /// `-`
    Negate,
    /// `!`
    Not,
    /// `*`
    Dereference,
    /// `&`
    Reference,
    /// `&mut`
    MutableReference,
}


#[derive(Debug)]
pub enum Node {
    // Root nodes
    Identifier { id: String },
    Number { value: String },
    Literal { kind: LiteralKind, value: String },
    
    // Unary operations
    UnaryOp { kind: UnaryOpKind, operand: Box<Node> },

    // Binary operations
    BinOp { kind: BinOpKind, left: Box<Node>, right: Box<Node> },
    BinOpEqual { kind: BinOpKind, left: Box<Node>, right: Box<Node> },

    // Statements
    FunctionDef { name: String, params: Vec<Node>, return_type: Type, body: Box<Node> },
    FunctionCall { name: String, args: Vec<Node> },
    Return { value: Box<Node> },
    Statements { statements: Vec<Box<Node>> },
}