#[macro_use]
extern crate macros;

#[allow(unused_variables)]
#[allow(unconditional_panic)]
fn main() {
    let n = 0;
    let a = 100;

    verify!(n == 0);
    let b = a / n; // Error: division by zero!

    if n != 0 {
        verify!(n != 0);
        let c = a / n; // OK
    }
}
