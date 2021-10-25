// Example for testing widening and narrow
// From the book "Static Program Analysis" by Anders Møller and Michael I. Schwartzbach

#[macro_use]
extern crate macros;

fn main() {
    let mut y = 0;
    let mut x = 7;
    x = x + 1;
    let mut i = 0;
    while i < 10 {
        i = i + 1;
        x = 7;
        x = x + 1;
        y = y + 1;
    }
    verify!(x == 8);
    verify!(y > 0);
}
