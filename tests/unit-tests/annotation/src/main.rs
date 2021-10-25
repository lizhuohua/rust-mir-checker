#[macro_use]
extern crate macros;

fn main() {
    let a = 1;
    let b = 2;
    let c = a + b;
    verify!(c == 3);
}
