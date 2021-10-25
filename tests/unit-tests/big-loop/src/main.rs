#[macro_use]
extern crate macros;

fn main() {
    let mut i = 0;
    while i < 1000000 {
        verify!(i < 1000000);
        i += 1;
    }
    verify!(i >= 1000000);

    let mut j = 1000000;
    while j > 0 {
        verify!(j > 0);
        j -= 1;
    }
    verify!(j <= 0);

    let k = 0;
    while k != 1000000 {
        verify!(k != 1000000);
        // Here will be a false positive
        // The checker will always alert that this is a potential integer overflow
        // This is because currently it cannot reason about whether `k` will be exactly 1000000
        // k += 2;
    }
}
