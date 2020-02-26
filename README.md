# OliveScript
A functional dynamically-typed scripting language.

## Building
After [installing rust](https://www.rust-lang.org/tools/install), run `cargo build --release` in the root directory of the project. This will create the OliveScript runtime and compiler binaries. Optionally, you can strip any of the generated binaries by running `strip target/release/olv` (other binary is called `olvc` and in the same directory). 