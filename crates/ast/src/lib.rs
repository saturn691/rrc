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
    BitXor,
    /// `&`
    BitAnd,
    /// `|`
    BitOr,
    /// `<<`
    ShiftLeft,
    /// `>>`
    ShiftRight,
    /// `==`
    Eq,
    /// `!=`
    Ne,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
    /// `&&`
    And,
    /// `||`
    Or,
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
pub enum PatKind {
    Identifier(String)
}

#[derive(Debug)]
/// A pattern e.g. `foo::bar`
pub struct Pat {
    pub kind: PatKind,
}

#[derive(Debug)]
pub struct Param {
    pub ty: Type,
    pub pat: Pat
}

impl Param {
    pub fn id(&self) -> String {
        match &self.pat.kind {
            PatKind::Identifier(id) => id.clone()
        }
    }
}

#[derive(Debug)]
pub struct FnSig {
    pub inputs: Vec<Param>,
    pub return_type: Type,
}

/// A segment of a path e.g. `std` or `io`
#[derive(Debug)]
pub struct PathSegment {
    pub identifier: String,
}

/// A path is a sequence of identifiers separated by `::`
/// e.g. `std::io::Read`
#[derive(Debug)]
pub struct Path {
    pub segments: Vec<PathSegment>,
}

impl Path {
    pub fn new(id: String) -> Self {
        Path {
            segments: vec![PathSegment {
                identifier: id
            }]
        }
    }
}

#[derive(Debug)]
pub enum ExprKind {
    Unary(UnaryOpKind, Box<Expr>),
    Binary(Box<Expr>, BinOpKind, Box<Expr>),
    Literal(String),
    Path(Path),
    /// An if block, with an optional else block
    /// e.g. `if expr { block } else { expr }`
    If(Box<Expr>, Box<Block>, Option<Box<Expr>>),
    Block(Box<Block>),
}

#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
}

#[derive(Debug)]
pub enum LocalKind {
    /// `let x;`
    Decl,
    /// `let x = 42;`
    Init(Box<Expr>),
    /// `let Some(x) = y else { ... }`
    InitElse(Box<Expr>, Box<Block>),
}

#[derive(Debug)]
pub struct Local {
    pub pat: Box<Pat>,
    pub ty: Option<Box<Type>>,
    pub kind: LocalKind,
}

#[derive(Debug)]
pub enum StmtKind {
    /// Local (let) bindings e.g. `let x = 42;`
    Let(Box<Local>),
    /// Item definitions e.g. `fn foo() {}`
    Item(Box<Node>),
    /// Expressions without a semicolon e.g. `foo()`
    Expr(Box<Expr>),
    /// Expr with a semicolon e.g. `foo();`
    Semi(Box<Expr>)
}

#[derive(Debug)]
pub struct Stmt {
    pub kind: StmtKind,
}

/// A block is a sequence of statements
/// e.g. `{ println!() }` as in `fn main() { println!(); }`
#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub struct Attr {
    pub path: Path,
}

#[derive(Debug)]
pub struct Visibility {
    pub kind: VisibilityKind,
}

#[derive(Debug)]
pub enum VisibilityKind {
    Public,
    Private,

}

#[derive(Debug)]
pub struct Fn {
    pub sig: FnSig,
    pub body: Option<Box<Block>>
}

#[derive(Debug)]
pub enum NodeKind {
    // Statements
    Fn(Box<Fn>),
}

#[derive(Debug)]
pub struct Node {
    pub attrs: Vec<Attr>,
    pub vis: Visibility,
    pub kind: NodeKind,
    pub identifier: Option<String>,
}