#[macro_use]
extern crate macros;

struct A {
    x: i32,
    y: i32,
}

#[allow(unused_variables)]
fn main() {
    let a = A { x: 1, y: 0 };
    let b = 1 / a.x; // OK
    verify!(b == 1);
    verify!(a.y == 0);
}
