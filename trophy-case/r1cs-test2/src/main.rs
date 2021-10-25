// This would cause an out-of-range access
// https://github.com/mir-protocol/r1cs/issues/11

use r1cs::Bn128;
use r1cs::MdsMatrix;

fn main() {
    MdsMatrix::<Bn128>::new(vec![]);
}
