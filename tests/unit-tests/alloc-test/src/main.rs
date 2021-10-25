#[macro_use]
extern crate macros;

extern crate alloc;
use alloc::vec;

#[allow(unused_variables)]
fn main() {
    let v1 = vec![1, 2, 3, 4, 5];
    // let v2 = vec![0; 10];
    verify!(v1[3] == 4);
}
