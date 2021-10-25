// This would cause integer overflow
// https://github.com/magiclen/Byte-Unit/issues/7

use byte_unit;

fn main() {
    // Panic: integer overflow
    println!("{}", byte_unit::n_zb_bytes(std::u128::MAX));
}
