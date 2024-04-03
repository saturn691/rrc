//! Lexer module
//!
//! Tokenises the input source code.
//! Inspired by https://github.com/rust-lang/rust
//!
//! Errors are not thrown at this stage. However, invalid tokens such as an
//! unterminated block comment are detected and reported.

mod cursor;

#[cfg(test)]
mod tests;

use cursor::EOF_CHAR;

pub use crate::cursor::Cursor;

use self::Token::*;
use unicode_properties::UnicodeEmoji;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    /// "//" comment
    LineComment { doc_style: Option<DocStyle> },

    /// "/* block comment */"
    BlockComment {
        doc_style: Option<DocStyle>,
        terminated: bool,
    },

    /// Whitespaces, e,g, " ", "\t", "\n"
    Whitespace,

    /// Identifiers, e.g., "foo", "bar"
    /// Includes keywords like "fn", "let", "if", etc.
    Identifier { id: String },

    /// Identifiers containin invalid characters, e.g., "foo-bar"
    InvalidIdentifier,

    /// Raw identifiers, e.g., "r#foo"
    RawIdentifier,

    /// Unknown prefix, e.g. `foo#`, `foo'`, `foo"`
    UnknownPrefix,

    /// Number literals, e.g. `123`, `12.34`, `b'1234'`, `3.3e-42`
    Number { number: String },

    /// String literals, e.g. `"foo"`
    StrLiteral { string: String, literal_kind: LiteralKind },

    /// Lifetime annotations, e.g. `'a`
    Lifetime { starts_with_number: bool },

    // Keywords
    Fn,

    // Tokens
    
    RightArrow,

    // One-character tokens

    /// ";"
    Semicolon,
    /// ":"
    Colon,
    /// ","
    Comma,
    /// "."
    Dot,
    /// "("
    OpenParen,
    /// ")"
    CloseParen,
    /// "{"
    OpenBrace,
    /// "}"
    CloseBrace,
    /// "["
    OpenBracket,
    /// "]"
    CloseBracket,
    /// "@"
    At,
    /// "#"
    Hash,
    /// "~"
    Tilde,
    /// "?"
    Question,
    /// "$"
    Dollar,
    /// "="
    Eq,
    /// "!"
    Bang,
    /// "<"
    Lt,
    /// ">"
    Gt,
    /// "&"
    And,
    /// "|"
    Or,
    /// "+"
    Plus,
    /// "-"
    Minus,
    /// "*"
    Star,
    /// "/"
    Slash,
    /// "%"
    Percent,
    /// "^"
    Caret,

    // Remaining tokens

    /// Unknown, unexpected tokens e.g. "😃"
    Unknown,

    /// End of file
    Eof,
}

impl Token {
    pub fn new(token: Token) -> Token {
        token
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocStyle {
    Outer,
    Inner,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiteralKind {
    /// Characters e.g. "'a'", "'\\'"
    Char { terminated: bool },
    /// Byte characters e.g. "b'a'", "b'\\'"
    Byte { terminated: bool },
    /// Strings e.g. ""foo"", ""foo"
    Str { terminated: bool },
    /// Byte strings e.g. "b"foo"", "b"foo"
    ByteStr { terminated: bool },
    /// C strings e.g. "c"foo"", "c"foo"
    CStr { terminated: bool },
    /// Raw strings e.g. "r"foo"", "r"foo"
    RawStr { n_hashes: Option<u8> },
    /// Raw byte strings e.g. "br"foo"", "br"foo"
    RawByteStr { n_hashes: Option<u8> },
    /// Raw C strings e.g. "cr"foo"", "cr"foo"
    RawCStr { n_hashes: Option<u8> },
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut cursor = Cursor::new(input);
    let mut tokens = Vec::new();

    loop {
        let token = cursor.lex();
        if token == Eof {
            break;
        }
        if token == Whitespace {
            continue;
        }

        tokens.push(token);
    }

    tokens
}

/// Returns true if the character is a whitespace character.
pub fn is_whitespace(c: char) -> bool {
    matches!(
        c,

        '\u{0009}' // horizontal tab, '\t'
        | '\u{000A}' // line feed, '\n'
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // carriage return, '\r'
        | '\u{0020}' // space, ' '
        | '\u{0085}' // next line
        | '\u{200E}' // left-to-right mark
        | '\u{200F}' // right-to-left mark
        | '\u{2028}' // line separator
        | '\u{2029}' // paragraph separator
    )
}

/// Returns true if the character is a valid identifier start character.
pub fn is_id_start(c: char) -> bool {
    // Underscore is a valid start character but not in is_xid_start
    c == '_' || unicode_xid::UnicodeXID::is_xid_start(c)
}

/// Returns true if the character is a valid identifier continuation character.
pub fn is_id_continue(c: char) -> bool {
    // Underscore is in is_xid_continue
    unicode_xid::UnicodeXID::is_xid_continue(c)
}

impl Cursor<'_> {
    /// Produces the next token from the input source code.
    pub fn lex(&mut self) -> Token {
        let first_char = self.next();
        let token_type = match first_char {
            // Whitespace
            c if is_whitespace(c) => self.whitespace(),

            // Slash could be comments
            '/' => match self.first() {
                '/' => self.line_comment(),
                '*' => self.block_comment(),
                _ => Slash,
            },

            // Raw identifiers, raw string literals or unknown
            'r' => match (self.first(), self.second()) {
                ('#', c) if is_id_start(c) => self.raw_identifier(),
                ('#', _) | ('"', _) => {
                    unimplemented!()
                }
                _ => self.identifier_or_unknown(),
            },

            // Identifiers
            c if is_id_start(c) => self.identifier_or_unknown(),

            // Numeric literals
            c @ '0'..='9' => {
                // TODO - Implement number literals
                let number = self.eat_decimal_digits(c);
                Number { number }
            }

            '-' => match self.first() {
                '>' => {
                    self.next();
                    RightArrow
                }
                _ => Minus,
            }
            
            // One character tokens
            ';' => Semicolon,
            ':' => Colon,
            ',' => Comma,
            '.' => Dot,
            '(' => OpenParen,
            ')' => CloseParen,
            '{' => OpenBrace,
            '}' => CloseBrace,
            '[' => OpenBracket,
            ']' => CloseBracket,
            '@' => At,
            '#' => Hash,
            '~' => Tilde,
            '?' => Question,
            '$' => Dollar,
            '=' => Eq,
            '!' => Bang,
            '<' => Lt,
            '>' => Gt,
            '&' => And,
            '|' => Or,
            '+' => Plus,
            '*' => Star,
            '%' => Percent,
            '^' => Caret,

            // Lifetime or character literal
            '\'' => self.lifetime_or_char(),

            // String literal
            '"' => {
                unimplemented!()
            }

            EOF_CHAR => Eof,

            _ => Unknown,
        };

        let res = Token::new(token_type);
        self.reset_pos();

        res
    }

    /// True if the character is a whitespace character.
    /// See https://doc.rust-lang.org/reference/whitespace.html for more details.
    fn whitespace(&mut self) -> Token {
        debug_assert!(is_whitespace(self.prev()));
        self.eat_while(is_whitespace);
        Whitespace
    }

    fn line_comment(&mut self) -> Token {
        LineComment { doc_style: None }
    }

    fn block_comment(&mut self) -> Token {
        BlockComment {
            doc_style: None,
            terminated: true,
        }
    }

    fn raw_identifier(&mut self) -> Token {
        debug_assert!(
            self.prev() == 'r' && 
            self.first() == '#' && 
            is_id_start(self.second())
        );

        // Eat '#'
        self.next();
        self.eat_while(is_id_continue);

        RawIdentifier
    }

    fn identifier_or_unknown(&mut self) -> Token {
        debug_assert!(is_id_start(self.prev()));
        let ident = self.eat_while(is_id_continue);

        match self.first() {
            c if !c.is_ascii() && c.is_emoji_char() => {
                self.unknown_or_invalid_identifier()
            } 

            _ => {
                match ident.as_str() {
                    "fn" => Fn,
                    _ => Identifier { id: ident },
                }
            }
        }
    }

    // TODO - accept hexademical digits and all literals afterwards
    fn eat_decimal_digits(&mut self, c: char) -> String {
        let mut number = c.to_string();

        while let c @ '0'..='9' = self.next() {
            number.push(c);
        }

        number
    }

    fn unknown_or_invalid_identifier(&mut self) -> Token {
        match self.first() {
            _ => InvalidIdentifier,
        }
    }

    fn lifetime_or_char(&mut self) -> Token {
        match self.first() {
            _ => Lifetime {
                starts_with_number: false,
            },
        }
    }
}
