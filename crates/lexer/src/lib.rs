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

use self::LiteralKind::*;
use self::TokenKind::*;
use unicode_properties::UnicodeEmoji;

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub len: u32,
}

impl Token {
    fn new(kind: TokenKind, len: u32) -> Token {
        Token { kind, len }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    /// "//" comment
    LineComment { doc_style: Option<DocStyle> },

    /// "/* block comment */"
    BlockComment { doc_style: Option<DocStyle>, terminated: bool },

    /// Whitespaces, e,g, " ", "\t", "\n"
    Whitespace,

    /// Identifiers, e.g., "foo", "bar"
    /// Includes keywords like "fn", "let", "if", etc.
    Identifier,

    /// Identifiers containin invalid characters, e.g., "foo-bar"
    InvalidIdentifier,

    /// Raw identifiers, e.g., "r#foo"
    RawIdentifier,

    /// Unknown prefix, e.g. `foo#`, `foo'`, `foo"` 
    UnknownPrefix,

    /// Literals, e.g. `123`, `12.34`, `b'1234'`, `3.3e-42`
    Literal { kind: LiteralKind, suffix_start: u32 },

    /// Lifetime annotations, e.g. `'a`
    Lifetime { starts_with_number: bool },

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocStyle {
    Outer,
    Inner,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiteralKind {
    /// Integers e.g. "42_u32", "0o777", "0b1"
    Int { base: Base, empty_int: bool },
    /// Floats e.g. "1.0", "1.0f32", "1.0e10"
    Float { base: Base, empty_exponent: bool },
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawStrError {
    InvalidStarter { bad_char: char },
    NoTerminator { expected: u32, found: u32, possible_terminator_offset: u32 },
    TooManyDelimiters { found: u32 },
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Base {
    /// "0b"
    Binary = 2,
    /// "0o"
    Octal = 8,
    /// Lack of a prefix
    Decimal = 10,
    /// "0x"
    Hexadecimal = 16,
}

// Use an iterator to avoid implementing the Copy trait for Token
pub fn tokenize(input: &str) -> impl Iterator<Item = Token> + '_ {
    let mut cursor = Cursor::new(input);
    std::iter::from_fn(move || {
        let token = cursor.lex();
        if token.kind == Eof {
            None
        } else {
            Some(token)
        }
    })
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
            '/' => {
                match self.first() {
                    '/' => self.line_comment(),
                    '*' => self.block_comment(),
                    _ => Slash,
                }
            }

            // Raw identifiers, raw string literals or unknown
            'r' => match(self.first(), self.second()) {
                ('#', c) if is_id_start(c) => self.raw_identifier(),
                ('#', _) | ('"', _) => {
                    let kind = RawStr { n_hashes: None };
                    let suffix_start = self.pos();
                    Literal { kind, suffix_start }
                }
                _ => self.identifier_or_unknown()
            }       

            // Identifiers
            c if is_id_start(c) => self.identifier_or_unknown(),

            // Numeric literals
            c @ '0'..='9' => {
                let kind = self.number_kind(c);
                let suffix_start = self.pos();
                Literal { kind, suffix_start }
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
            '-' => Minus,
            '*' => Star,
            '%' => Percent,
            '^' => Caret,

            // Lifetime or character literal
            '\'' => self.lifetime_or_char(),
            
            // String literal
            '"' => {
            let kind = Str { terminated: true };
                let suffix_start = self.pos();
                Literal { kind, suffix_start }
            }

            EOF_CHAR => Eof,
            
            _ => Unknown,
        };
        
        let res = Token::new(token_type, self.pos());
        self.reset_pos();
        
        res
    }

    /// True if the character is a whitespace character.
    /// See https://doc.rust-lang.org/reference/whitespace.html for more details.
    fn whitespace(&mut self) -> TokenKind {
        debug_assert!(is_whitespace(self.prev()));
        self.eat_while(is_whitespace);
        Whitespace
    }

    fn line_comment(&mut self) -> TokenKind {
        LineComment { doc_style: None }
    }

    fn block_comment(&mut self) -> TokenKind {
        BlockComment { doc_style: None, terminated: true }
    }

    fn raw_identifier(&mut self) -> TokenKind {
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

    fn identifier_or_unknown(&mut self) -> TokenKind {
        debug_assert!(is_id_start(self.prev()));
        self.eat_while(is_id_continue);

        match self.first() {
            c if !c.is_ascii() && c.is_emoji_char() => {
                self.unknown_or_invalid_identifier()
            } 
            _ => Identifier,
        }
    }

    fn number_kind(&mut self, first_digit: char) -> LiteralKind {
        debug_assert!('0' <= self.prev() && self.prev() <= '9');
        
        // Decode this number in the given base
        if first_digit == '0' {
            match self.first() {
                _ => return Int { base: Base::Decimal, empty_int: false },
            }
        } else {
            self.eat_decimal_digits();
        };

        Int { base: Base::Decimal, empty_int: false }
    }

    fn eat_decimal_digits(&mut self) -> bool {
        let mut has_digits = false;

        loop {
            match self.first() {
                '_' => {
                    self.next();
                }
                '0'..='9' => {
                    has_digits = true;
                    self.next();
                }
                _ => break,
            }
        }

        has_digits
    }

    fn unknown_or_invalid_identifier(&mut self) -> TokenKind {
        match self.first() {
            _ => InvalidIdentifier,
        }
    }

    fn lifetime_or_char(&mut self) -> TokenKind {
        match self.first() {
            _ => Lifetime { starts_with_number: false },
        }
    }

    fn eat_literal_suffix(&mut self) {
        while is_id_continue(self.first()) {
            self.next();
        }
    }
    
}