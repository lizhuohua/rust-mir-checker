#[macro_use]
extern crate macros;

#[allow(unused_variables)]
fn main() {
    let a = vec![1, 2, 3, 4, 5];
    let b = a[0];
    // let c = a[4];
    verify!(b == 1);
    // verify!(c == 5);
}
