# Docmentation

## Overall structure

The `lexer` folder consumes and outputs tokens.

The `parser` folder consumes tokens and builds an abstract syntax tree (AST).

The `ast` folder lays out the structure for the abstract syntax tree.

The `hir` folder takes the AST and transforms it into high-level representation,
to be passed into the MIR

The `mir` folder takes the HIR input and transforms into a control-flow graph
(CFG).

The `codegen` folder takes the MIR and converts it into LLVM-IR.