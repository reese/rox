![Rox Logo](./assets/rox-banner.png)

![Build](https://github.com/reese/rox/workflows/Build/badge.svg)
![Security audit](https://github.com/reese/rox/workflows/Security%20audit/badge.svg)

This is the main repository for the Rox programming language.

## Building from source

[![Codacy Badge](https://api.codacy.com/project/badge/Grade/b4a84af0d8a541538f3bddba4eb955ec)](https://app.codacy.com/manual/reese/rox?utm_source=github.com&utm_medium=referral&utm_content=reese/rox&utm_campaign=Badge_Grade_Settings)

Rox compiles programs using the `cargo` command line tool.
To compile, use `cargo run path/to/script.rox`.
For development purposes, Rox always outputs to `test.o` in the working directory.
This object file can be run using your C compiler with `cc test.o -o <executable>`.
