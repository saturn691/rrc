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

#[derive(Debug)]
pub enum TokenKind {
    Identifier { name: String },
    Number { value: String },
    Literal { kind: LiteralKind, value: String, suffix: Option<String> },
    BinOp { kind: BinOpKind },
    BinOpEqual { kind: BinOpKind },
}

#[derive(Debug)]
pub struct Node {
    pub kind: TokenKind,
    pub children: Vec<Box<Node>>,
}

impl Node {
    pub fn new(kind: TokenKind) -> Self {
        Self {
            kind,
            children: Vec::new(),
        }
    }
}