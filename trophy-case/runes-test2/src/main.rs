// This would cause a division-by-zero
// https://github.com/Determinant/runes/issues/1

use runes::utils::Sampler;

fn main() {
    Sampler::new(0, 0);
}
