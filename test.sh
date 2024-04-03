#!/bin/bash

# Compile the program using the new compiler
cargo build
./target/debug/rrc \
    --input tests/_example/return_0.rs \
    --output bin/return_0.ll

# Run the program
llc bin/return_0.ll -o bin/return_0.s
clang bin/return_0.s -o bin/return_0
./bin/return_0

if [ $? -eq 0 ]; then
    echo "Test passed"
else
    echo "Test failed"
fi