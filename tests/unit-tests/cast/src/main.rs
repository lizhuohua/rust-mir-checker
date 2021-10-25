#[macro_use]
extern crate macros;

#[allow(unused_variables)]
#[allow(unused_assignments)]
fn main() {
    let a: u16 = 1000;
    let mut b: u8 = a as u8;

    verify!(b == 232);
    b = b + 1;
    verify!(b == 233);
}
