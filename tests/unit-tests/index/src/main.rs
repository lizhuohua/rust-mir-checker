#[macro_use]
extern crate macros;

#[allow(unused_variables)]
fn main() {
    let a = [1, 2, 3, 4, 5];
    let b = a[3];
    let c = a[b];

    // Make sure `b` is 4, `c` is 5
    verify!(b == 4);
    verify!(c == 5);
}
