// This would cause out-of-bounds access
// https://github.com/dropbox/rust-brotli/issues/53

use brotli::enc::brotli_bit_stream::BrotliBuildAndStoreHuffmanTreeFast;
use brotli::enc::writer::StandardAlloc;
fn main() {
    let mut alloc = StandardAlloc::default();
    BrotliBuildAndStoreHuffmanTreeFast(
        &mut alloc,
        &[0],
        0,
        0,
        &mut [0],
        &mut [0],
        &mut 99999,
        &mut [0],
    );
}
