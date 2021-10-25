#[macro_use]
extern crate macros;

#[allow(unused_variables)]
fn main() {
    let a = 1;
    let b = 2;
    let c = a + b;
    let d = a - b;
    let e = a * b;
    let f = a / b;
    let g = a % b;
    let h = a << b;
    let i = a >> b;
    let j = a & b;
    let k = a | b;
    let l = a ^ b;
    verify!(c == 3);
    verify!(d == -1);
    verify!(e == 2);
    verify!(f >= 0 && f <= 1);
    verify!(g == 1);
    verify!(h == 4);
    verify!(i == 0);
    verify!(j == 0);
    verify!(k == 3);
    verify!(l == 3);
}
