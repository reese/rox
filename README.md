<div align="center">
<h1>Rox</h1>

<img src="./assets/geodude.png"  alt="Geodude"/>
</div>

## Building from source

The Rox compiler is built in Rust and can be built using Rust's `cargo` command line tool.
You can start the REPL using `cargo run`, or you can run an individual file using `cargo run path/to/file.rox`.

## Difference between Lox and Rox

Rox is originally based off of the Lox language introduced in Bob Nystrom's book "Crafting Interpreters."
However, as I've continued through this book, my implementation and design decisions have increasingly split from the original Lox language.
Lox in many ways follows the lead of JavaScript and Python, but in my personal experience with these languages, I've found them much more enjoyable when combined with static analysis tools, such as TypeScript and MyPy.
Having a language with that marries the flexibility of high-level languages while also preventing a whole class of bugs with a robust type system.
I hope that Rox will join the familiarity and readability of C-style syntax with the strong types of functional-style languages like OCaml, Haskell, and Rust.

## Future Additions

One possible future alternative for this VM would be to compile directly to WebAssembly.
There are several reasons why this may be an interesting alternative, but in brief, it would make cross platform compilation better (using something like WASI), as well as make it easier to write for desktop, mobile, web and server development.
Essentially, WASM is an interesting development that looks like it may eventually serve as a recognized cross-platform standard.
