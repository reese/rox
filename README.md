![Rox Logo](./assets/rox-banner.png)

![Build](https://github.com/reese/rox/workflows/Build/badge.svg)
![Security audit](https://github.com/reese/rox/workflows/Security%20audit/badge.svg)
[![Codacy Badge](https://app.codacy.com/project/badge/Grade/415d52959df14ca48c52e1ad5ebe3d0c)](https://www.codacy.com/manual/reese/rox?utm_source=github.com&amp;utm_medium=referral&amp;utm_content=reese/rox&amp;utm_campaign=Badge_Grade)

This is the main repository for the Rox programming language.

## Building from source

Rox compiles programs using the `cargo` command line tool.
To compile, use `cargo run path/to/script.rox`.
For development purposes, Rox always outputs to `test.o` in the working directory.
This object file can be run using your C compiler with `cc test.o -o <executable>`.
