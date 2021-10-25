#[macro_use]
extern crate macros;

enum A {
    One,
    #[allow(dead_code)]
    Two,
    #[allow(dead_code)]
    Three,
}

#[allow(unused_variables)]
fn main() {
    let a = A::One;
    let b = match a {
        A::One => 1,
        A::Two => 2,
        A::Three => 3,
    };
    // Make sure `b` is 1 here
    verify!(b == 1);
}
