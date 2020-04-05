<div align="center">
<h1>Rox</h1>

<img src="./assets/geodude.png"  alt="Geodude"/>
</div>

## Building from source

The Rox compiler is built in Rust and can be built using Rust's `cargo` command line tool.
You can start the REPL using `cargo run`, or you can run an individual file using `cargo run path/to/file.rox`.

## Difference between Lox and Rox

Rox is based off of the Lox language introduced in Bob Nystrom's book "Crafting Interpreters."
However, I intend to change some of the design decisions bade in the book.

I intend to keep many of the general syntactic decisions made in the book, but I intend to make the language statically typed.
Previously, I intended to keep the language dynamic while trying to avoid a certain class of errors by removing the `nil` keyword, but in retrospect, I've found that a sound type system is much more productive in the long term, especially with a good type inference system.

## Future Additions

Once the major parts of the compiler are completed, it could be interesting to replace this hand-made VM with LLVM.
Given that we already compile to a fairly standard intermediate representation (and the language is fairly small as-is), this should provide a lot of benefits from the LLVM optimization passes.
Rust bindings for LLVM are pretty sparse, but [Inkwell](https://github.com/TheDan64/inkwell) looks like a good option for Rust-y bindings without excessive use of `unsafe`.