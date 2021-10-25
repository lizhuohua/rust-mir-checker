#[macro_use]
extern crate macros;

// fn main() {
//     let mut i = 0;
//     while i < 5 {
//         verify!(i < 5);
//         i = i + 1;
//     }
//     verify!(i >= 5);

//     let mut i = 5;
//     while i > 0 {
//         verify!(i > 0);
//         i = i - 1;
//     }
//     verify!(i <= 0);
// }

fn main() {
    let mut a = 0;
    let r = &mut a;
    while *r < 5 {
        *r += 1;
    }
}
