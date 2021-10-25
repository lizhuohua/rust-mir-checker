# 🏆 Trophy Case 🏆

A showcase of bugs found via statically analyzing Rust codebases by this tool. The template of this page is shamelessly stolen from [rust-fuzz](https://github.com/rust-fuzz/trophy-case).

Most of these bugs are not memory-safety issues which are commonly seen in C and C++ projects. That is because Rust is memory-safe by default!

Memory-safety issues are marked with a ❗ in the "Memory-safety?" column. Denial of service, such as panics, is not considered memory-safety issues.

Crate | Version | Information | Category | Memory-Safety?
----- | ------- | ----------- | -------- | --------------
[bitvec](https://crates.io/crates/bitvec) | 0.21.1 | [division by zero](https://github.com/bitvecto-rs/bitvec/issues/123) | `arith`
[brotli](https://crates.io/crates/brotli) | 3.3.0 | [integer overflow](https://github.com/dropbox/rust-brotli/issues/53) | `arith`
[brotli](https://crates.io/crates/brotli) | 3.3.0 | [integer overflow](https://github.com/dropbox/rust-brotli/issues/53) | `arith`
[brotli](https://crates.io/crates/brotli) | 3.3.0 | [out of range access](https://github.com/dropbox/rust-brotli/issues/53) | `oor`
[byte-unit](https://crates.io/crates/byte-unit) | 4.0.10 | [integer overflow](https://github.com/magiclen/Byte-Unit/issues/7) | `arith`
[bytemuck](https://crates.io/crates/bytemuck) | 1.5.1-alpha.0 | [unreachable code](https://github.com/Lokathor/bytemuck/issues/52) | `logic`
[executable-memory](https://crates.io/crates/executable_memory) | 0.1.2 | [integer overflow](https://gitlab.com/nathanfaucett/rs-executable_memory/-/issues/1) | `arith`
[executable-memory](https://crates.io/crates/executable_memory) | 0.1.2 | [segmentation fault](https://gitlab.com/nathanfaucett/rs-executable_memory/-/issues/1) | `segfault`
[gmath](https://github.com/denosaurs/gmath) | 0.1.0 | [use after free](https://github.com/denosaurs/gmath/issues/1) | `uaf` | ❗
[qrcode-generator](https://crates.io/crates/qrcode-generator) | 4.0.4 | [integer overflow and out of range access](https://github.com/magiclen/qrcode-generator/issues/2) | `arith`, `oor`
[r1cs](https://crates.io/crates/r1cs) | 0.4.7 | [division by zero](https://github.com/mir-protocol/r1cs/issues/11) | `arith`
[r1cs](https://crates.io/crates/r1cs) | 0.4.7 | [out of range access](https://github.com/mir-protocol/r1cs/issues/11) | `oor`
[runes](https://crates.io/crates/runes) | 0.2.5 | [integer overflow](https://github.com/Determinant/runes/issues/1) | `arith`
[runes](https://crates.io/crates/runes) | 0.2.5 | [division by zero](https://github.com/Determinant/runes/issues/1) | `arith`
[safe-transmute](https://crates.io/crates/safe-transmute) | 0.11.0 | [division by zero](https://github.com/nabijaczleweli/safe-transmute-rs/issues/65) | `arith`
[scriptful](https://crates.io/crates/scriptful) | 0.2.0 | [call to unwrap on None](https://github.com/aesedepece/scriptful/issues/1) | `unwrap`
[spglib](https://crates.io/crates/spglib) | 1.15.1 | [potential double free](https://github.com/spglib/spglib-rs/issues/1) | `df` | ❗

## Description of categories:

* `arith`: Arithmetic error, eg. overflows
* `logic`: Logic bug
* `loop`: Infinite loop
* `oom`: Out of memory
* `oor`: Out of range access
* `segfault`: Program segfaulted
* `so`: Stack overflow
* `uaf`: Use after free
* `df`: Double free
* `uninit`: Program discloses contents of uninitialized memory
* `unwrap`: Call to `unwrap` on `None` or `Err(_)`
* `utf-8`: Problem with UTF-8 strings handling, eg. get a char not at a char boundary
* `panic`: A panic not covered by any of the above
* `other`: Anything that does not fit in another category, or unclear what the problem is
