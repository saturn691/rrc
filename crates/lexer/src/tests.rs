use super::*;

use expect_test::{expect, Expect};

fn check_lex(input: &str, expected: Expect) {
    let actual: String = tokenize(input)
        .into_iter()
        .map(|token| format!("{:?}\n", token))
        .collect();
    expected.assert_eq(&actual);
}

#[test]
fn test_basic() {
    check_lex(
        "    ;",
        expect![[r#"
            Token { kind: Whitespace, len: 4 }
            Token { kind: Semicolon, len: 1 }
        "#]],
    )
}

#[test]
fn test_function() {
    check_lex(
        "fn add(a: i32, b: i32) -> i32 { a + b }",
        expect![[r#"
            Token { kind: Identifier, len: 2 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Identifier, len: 3 }
            Token { kind: OpenParen, len: 1 }
            Token { kind: Identifier, len: 1 }
            Token { kind: Colon, len: 1 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Identifier, len: 3 }
            Token { kind: Comma, len: 1 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Identifier, len: 1 }
            Token { kind: Colon, len: 1 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Identifier, len: 3 }
            Token { kind: CloseParen, len: 1 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Minus, len: 1 }
            Token { kind: Gt, len: 1 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Identifier, len: 3 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: OpenBrace, len: 1 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Identifier, len: 1 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Plus, len: 1 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: Identifier, len: 1 }
            Token { kind: Whitespace, len: 1 }
            Token { kind: CloseBrace, len: 1 }
        "#]],
    )
}