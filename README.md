# Rustic Rust Compiler

Rustic Rust Compiler (`rrc`) is a Rust to LLVM-IR compiler. 

## Usage

To build the compiler, run the following command:

```bash
cargo build
```

To run the compiler, run the following command:

```bash
target/debug/rrc --input "tests/_example/return_0.rs" --output "./return_0.ll"
```

This can be ran locally using LLVM and Clang. Run the following command:

```bash
# Use LLVM to compile to assembly
llc "./return_0.ll"

# Use Clang (or GCC) to compile to bytecode
clang return_0.s

# Run the program
./a.out

# (Optional) Check the return code
echo $? 
```

## Documentation

See [`docs/README.md`](docs/README.md)

## License

Licensed under MIT.

See [`LICENSE`](LICENSE) for details.