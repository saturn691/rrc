//! Parser module
//! 
//! Turns the lexer tokens into an AST
//! Uses recursive descent parsing

use ast::{self, TokenKind};
use lexer::Token;

pub struct Parser {
    index: usize,
    tokens: Vec<Token>,
}

impl<'a> Parser {
    pub fn new() -> Parser {
        Parser {
            index: 0,
            tokens: Vec::new()
        }
    }

    /// Parse the tokens into an AST - entry point
    pub fn parse(&mut self, input: &String) -> Result<ast::Node, String> {
        self.tokens = lexer::tokenize(input);
        self.index = 0;
        self.parse_primary()
    }

    /// Primary expression - root of the AST
    fn parse_primary(&mut self) -> Result<ast::Node, String> {
        let token = self.eat_token()
            .ok_or("Unexpected end of input, expected primary")?;
        match token {
            Token::Number {number} => {
                Ok(ast::Node::new(
                    TokenKind::Number{ value: number.to_string() }
                ))
            },
            _ => Err("Unexpected token, expected primary".to_string())
        }
    }

    // Helper functions

    /// Looks at the next (offset) token
    fn peek(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.index + offset)
    }

    /// Eats a token and returns it
    fn eat_token(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.index);
        self.index += 1;
        token
    }
}