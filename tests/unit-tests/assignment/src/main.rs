#[macro_use]
extern crate macros;

#[allow(unused_variables)]
fn main() {
    let a = 1;
    let b = a;

    let c = &a;
    let d = *c;

    // Make sure `b` and `d` are 1, `c` points to `a`
    verify!(b == 1);
    verify!(d == 1);

    // let e = vec![1, 2, 3, 4, 5];
    // let f = e[4];
    // let g = e[d];
    // verify!(f == 5);
    // verify!(g == 2);
}
