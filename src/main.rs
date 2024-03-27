use parser::Parser;

fn main() {
    let input = "123".to_string();

    let mut parser = Parser::new();
    
    let res = parser.parse(&input);
    match res {
        Ok(ast) => println!("{:?}", ast),
        Err(e) => println!("Error: {}", e)
    }
}