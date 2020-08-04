![Rox Logo](./assets/rox-banner.png)

[![](https://meritbadge.herokuapp.com/rox-lang)](https://crates.io/crates/rox-lang)
![Build](https://github.com/reese/rox/workflows/Build/badge.svg)
![Security audit](https://github.com/reese/rox/workflows/Security%20audit/badge.svg)

## Warning: Rox is still _very_ early in development and will likely change drastically in upcoming releases.

This is the main repository for the Rox programming language.

## Installation

`Rox` can be installed using the `cargo` command line tool:

```shell script
cargo install rox-lang
```

## Running

`Rox` files can be either output as an executable or run directly using the JIT compiler.

### Compiling an executable

To output an executable, run `rox build yourScript.rox -o yourExecutable`.
This compiles and links the executable using your native C compiler.
If you want to skip the linking step, you can use the `--no-link` flag.

### Using the JIT compiler

To use `Rox`'s JIT compiler, use `rox run yourScript.rox`.

For more details on `Rox`'s CLI, see `rox --help`.
