#[macro_use]
extern crate macros;

fn main() {
    let a = std::mem::size_of::<u32>();
    verify!(a == 4);
}
