//! Parser module
//! 
//! Turns the lexer tokens into an AST
//! Uses recursive descent parsing

use ast;
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
        println!("Tokens: {:#?}", self.tokens);
        self.parse_fn()
    }  

    /// FUNCTION_DEFINITION
    /// : 'fn' IDENTIFIER '(' PARAMS ')' '{' STATEMENTS '}'
    /// ; 'fn' IDENTIFIER '(' PARAMS ')' '->' RETURN_TYPE '{' STATEMENTS '}'
    fn parse_fn(&mut self) -> Result<ast::Node, String> {
        // TODO consider function modifiers like `pub`, `unsafe`, `const`, etc.
        self.expect(Token::Fn);
        let name = self.expect_identifier()?;
        let mut r_type = ast::Type::Primitive(ast::PrimitiveType::Void);

        self.expect(Token::OpenParen);
        // TODO parse parameters
        self.expect(Token::CloseParen);

        match self.peek(0) {
            Some(Token::RightArrow) => {
                self.expect(Token::RightArrow);
                r_type = self.parse_return_type()?;
           }
            _ => {}
        }

        let statements: ast::Block = self.parse_block_common()?;

        // Build the node
        let fn_node = ast::Fn {
            sig: ast::FnSig { 
                inputs: Vec::new(),
                return_type: r_type
            },
            body: Some(Box::new(statements))
        };

        Ok(ast::Node {
            kind: ast::NodeKind::Fn(Box::new(fn_node)),
            identifier: Some(name)
        })
    }

    fn parse_return_type(&mut self) -> Result<ast::Type, String> {
        let id = self.expect_identifier()?;
        
        match id.as_str() {
            "i8" => Ok(ast::Type::Primitive(ast::PrimitiveType::I8)),
            "i16" => Ok(ast::Type::Primitive(ast::PrimitiveType::I16)),
            "i32" => Ok(ast::Type::Primitive(ast::PrimitiveType::I32)),
            "i64" => Ok(ast::Type::Primitive(ast::PrimitiveType::I64)),

            _ => Err("Unknown type".to_string())
        }
    }

    fn parse_block_common(&mut self) -> Result<ast::Block, String> {
        self.expect(Token::OpenBrace);
        let statements = self.parse_statements()?;
        self.expect(Token::CloseBrace);
        
        Ok(
            ast::Block {
                stmts: statements
            }
        )
    }

    fn parse_statements(&mut self) -> Result<Vec<ast::Stmt>, String> {
        let mut statements: Vec::<ast::Stmt> = Vec::new();
        
        while let Some(token) = self.peek(0) {
            match token {
                Token::CloseBrace => break,
                _ => {
                    let statement = self.parse_statement()?;
                    statements.push(statement);
                }
            }
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<ast::Stmt, String> {
        let expr = self.parse_expression()?;
        
        match self.peek(0) {
            Some(Token::Semicolon) => {
                self.expect(Token::Semicolon);
                Ok(ast::Stmt { 
                    kind: ast::StmtKind::Semi(Box::new(expr)) 
                })
            },
            _ => {
                Ok(ast::Stmt { 
                    kind: ast::StmtKind::Expr(Box::new(expr)) 
                })
            }
        }
        // let
        // return
        // if
        // while
        // for
        // loop
        // break
        // continue
        // match
        // unsafe
        // block
        // expression -/
    }

    /// Reference: https://doc.rust-lang.org/reference/expressions.html
    fn parse_expression(&mut self) -> Result<ast::Expr, String> {
        self.parse_assignment()
        
        // expression , assignment_expression
    }

    fn parse_assignment(&mut self) -> Result<ast::Expr, String> {
        self.parse_ellipsis()
        
        // =
        // +=
        // -=
        // *=
        // /=
        // %=
        // ^=
        // &=
        // |=
        // <<=
        // >>=
    }
   
    fn parse_ellipsis(&mut self) -> Result<ast::Expr, String> {
        self.parse_logical_or()
        
        // ..
        // ..=
    }
   
    fn parse_logical_or(&mut self) -> Result<ast::Expr, String> {
        self.parse_logical_and()
        
        // ||
    }

    fn parse_logical_and(&mut self) -> Result<ast::Expr, String> {
        self.parse_comparison()
        
        // &&
    }

    fn parse_comparison(&mut self) -> Result<ast::Expr, String> {
        self.parse_or()
        
        // ==
        // !=
        // >
        // <
        // >=
        // <=
    }
    
    fn parse_or(&mut self) -> Result<ast::Expr, String> {
        self.parse_xor()
        
        // |
    }

    fn parse_xor(&mut self) -> Result<ast::Expr, String> {
        self.parse_and()
        
        // ^
    }

    fn parse_and(&mut self) -> Result<ast::Expr, String> {
        self.parse_shift()
        
        // &
    }

    fn parse_shift(&mut self) -> Result<ast::Expr, String> {
        self.parse_additive()
        
        // <<
        // >>
    }

    /// ADDITIVE_EXPRESSION
    /// : MULTIPLICATIVE_EXPRESSION
    /// | MULTIPLICATIVE_EXPRESSION '+' ADDITIVE_EXPRESSION
    /// | MULTIPLICATIVE_EXPRESSION '-' ADDITIVE_EXPRESSION
    fn parse_additive(&mut self) -> Result<ast::Expr, String> {
        let left = self.parse_multiplicative()?;
        let token = self.peek(0);
        let bin_op_kind: ast::BinOpKind;

        match token {
            Some(Token::Plus) => {
                self.expect(Token::Plus);
                bin_op_kind = ast::BinOpKind::Plus;
            }

            Some(Token::Minus) => {
                self.expect(Token::Minus);
                bin_op_kind = ast::BinOpKind::Minus;
            }
            
            // Short circuit
            _ => return Ok(left)
        }

        let right = self.parse_additive()?;
        Ok(ast::Expr { 
            kind: ast::ExprKind::Binary(
                Box::new(left), 
                bin_op_kind, 
                Box::new(right)
            ) 
        })
    }

    /// MULTIPLICATIVE_EXPRESSION
    /// : CAST_EXPRESSION
    /// | CAST_EXPRESSION '*' MULTIPLICATIVE_EXPRESSION
    /// | CAST_EXPRESSION '/' MULTIPLICATIVE_EXPRESSION
    /// | CAST_EXPRESSION '%' MULTIPLICATIVE_EXPRESSION
    fn parse_multiplicative(&mut self) -> Result<ast::Expr, String> {
        let left = self.parse_cast()?;
        let token = self.peek(0);
        let bin_op_kind: ast::BinOpKind;

        match token {
            Some(Token::Star) => {
                self.expect(Token::Star);
                bin_op_kind = ast::BinOpKind::Multiply;
            }

            Some(Token::Slash) => {
                self.expect(Token::Slash);
                bin_op_kind = ast::BinOpKind::Divide;
            }

            Some(Token::Percent) => {
                self.expect(Token::Percent);
                bin_op_kind = ast::BinOpKind::Modulo;
            }
            
            // Short circuit
            _ => return Ok(left)
        }
        
        let right = self.parse_multiplicative()?;                
        Ok(ast::Expr { 
            kind: ast::ExprKind::Binary(
                Box::new(left), 
                bin_op_kind,
                Box::new(right)
            ) 
        })
    }

    fn parse_cast(&mut self) -> Result<ast::Expr, String> {
        self.parse_unary()
        
        // as
    }

    fn parse_unary(&mut self) -> Result<ast::Expr, String> {
        self.parse_postfix()
        
        // -
        // *
        // !
        // &
        // &mut
    }

    fn parse_postfix(&mut self) -> Result<ast::Expr, String> {
        self.parse_primary()
    
        // Paths
        // Method calls
        // Field expressions
        // Function calls
        // Array indexing
        // `?` operator
    }

    /// Primary expression - root of the AST
    /// 
    /// PRIMARY
    /// : NUMBER
    /// | IDENTIFIER
    /// | STRING
    /// | '(' EXPRESSION ')'
    fn parse_primary(&mut self) -> Result<ast::Expr, String> {
        let token = self.eat_token()
            .ok_or("Unexpected end of input, expected primary")?;
     
        match token {
            Token::Number {number} => {
                Ok(ast::Expr { kind: ast::ExprKind::Literal(number.to_string()) })
            },

            Token::Identifier {id} => {
                Ok(ast::Expr { 
                    kind: ast::ExprKind::Path(
                        ast::Path::new(id.to_string())
                    )
                })
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

    fn expect_identifier(&mut self) -> Result<String, String> {
        let token = self.eat_token()
            .ok_or("Unexpected end of input, expected identifier")?;

        match token {
            Token::Identifier {id} => Ok(id.to_string()),
            _ => Err("Unexpected token, expected identifier".to_string())
        }
    }

    fn expect(&mut self, kind: Token) {
        let token = self.eat_token().expect(&format!(
            "Expected to consume token `{:?}`, but there was no next token",
            kind
        ));

        assert_eq!(
            *token, kind,
            "Expected token `{:?}`, but found `{:?}`",
            kind, *token
        );
    }
}