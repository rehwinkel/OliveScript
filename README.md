# OliveScript
A functional dynamically-typed scripting language.

## Building
After [installing rust](https://www.rust-lang.org/tools/install), run `cargo build --release` in the root directory of the project. This will create the OliveScript runtime, compiler and native compiler binaries. Optionally, you can strip any of the generated binaries by running `strip target/release/olv` (other binaries are called `olvc` and `olvn` in the same directory). 