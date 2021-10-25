// This would cause integer overflow
// https://github.com/dropbox/rust-brotli/issues/53

use brotli::enc::command::BrotliDistanceParams;
use brotli::enc::command::Command;

fn main() {
    let mut command = Command::default();
    command.dist_prefix_ = 1000;
    let params = BrotliDistanceParams {
        distance_postfix_bits: 40,
        num_direct_distance_codes: 0,
        alphabet_size: 0,
        max_distance: 0,
    };
    let _ = brotli::enc::command::CommandRestoreDistanceCode(&command, &params);
}
