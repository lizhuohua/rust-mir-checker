// This would cause division by zero
// https://github.com/bitvecto-rs/bitvec/issues/123

use bitvec::mem;

fn main() {
    let _a = mem::elts::<()>(1);
}
