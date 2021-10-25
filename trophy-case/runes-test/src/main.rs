// This would cause an integer overflow
// https://github.com/Determinant/runes/issues/1

use runes::utils::load_prefix;
use runes::utils::Read;

struct A {}

impl Read for A {
    fn read(&mut self, _buf: &mut [u8]) -> Option<usize> {
        None
    }
}

fn main() {
    load_prefix(&mut [1], 10, &mut A {});
}
