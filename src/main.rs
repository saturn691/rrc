use clap::{Arg, App};
use parser::Parser;
use hir;
use codegen;

fn main() {
    let matches = App::new("Rustic Rust Compiler")
        .version("0.1.0")
        .author("William Huynh (@saturn691)")
        .about("A simple Rust compiler")
        .arg(Arg::with_name("INPUT")
            .short('i')
            .long("input")
            .value_name("FILE")
            .help("The input file to compile (*.rs)")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("OUTPUT")
            .short('o')
            .long("output")
            .value_name("FILE")
            .help("The output file to (*.ll)")
            .takes_value(true))
        .get_matches();
        
    let input = matches.value_of("INPUT").unwrap();
    let output = matches.value_of("OUTPUT")
        .unwrap_or("bin/output.ll");

    std::fs::create_dir("bin").unwrap_or_default();
    
    let input = std::fs::read_to_string(input)
        .expect("Unable to read file");

    let mut parser = Parser::new();
    
    let root = parser.parse(&input).unwrap();
    
    // println!();
    // println!("{:#?}", root);
    // println!();

    let hir = hir::hir_build(root);
    
    match hir {
        Ok(hir) => {
            hir::graph::visualize(&hir);
            // println!("{:#?}", hir);
            let code = codegen::lir_build(hir).unwrap();
            std::fs::write(output, code)
                .expect("Unable to write file");
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }

}