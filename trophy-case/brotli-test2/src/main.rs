// This would cause integer overflow
// https://github.com/dropbox/rust-brotli/issues/53

use brotli::enc::command::PrefixEncodeCopyDistance;

fn main() {
    let mut code = 0;
    let mut extra_bits = 0;
    PrefixEncodeCopyDistance(100, 0, 100, &mut code, &mut extra_bits);
}
