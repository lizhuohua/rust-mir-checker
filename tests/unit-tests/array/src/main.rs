#[macro_use]
extern crate macros;

#[allow(unused_variables)]
fn main() {
    let mut a = [1, 2, 3, 4, 5];
    let c = a;
    let b = c[0];
    let c = a[4];
    verify!(b == 1);
    verify!(c == 5);
    a[4] = 10;
    verify!(a[4] == 10);
    // let c = &a[2..5]; // Constant slice in source
}
