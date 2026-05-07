# Tautology checker

Rust implementation of Lawrence Paulson's tautology checker from ML for
the Working Programmer.

## Building

    cargo build
    cargo test

## Use

Pass propositions as command-line arguments to the tool. The tool
prints each proposition and if it's a tautology. For example,

    cargo run -- "a | b"
    text: a | b
    prop: a ∨ b
    taut: false

Valid syntax for propositions is:

    proposition = name
                | ~ proposition
                | proposition | proposition
                | proposition & proposition
                | proposition -> proposition
                | ( proposition )
