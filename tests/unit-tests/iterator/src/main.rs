#[macro_use]
extern crate macros;

#[allow(unused_assignments)]
#[allow(unused_variables)]
fn main() {
    let a = vec![1, 2, 3, 4, 5];
    let mut b = 0;
    for i in a {
        // verify!(i >= 1 && i <= 5);
        b = i;
        verify!(b == i);
    }
    // verify!(b == 5);
}
