#[macro_use]
extern crate macros;

// Use a function call to avoid constant propagation
fn func(a: i32) -> i32 {
    2 * a
}

#[allow(unused_variables)]
fn main() {
    let a = func(2);
    verify!(a == 4);
    let b = -a;
    verify!(b == -4);
}
